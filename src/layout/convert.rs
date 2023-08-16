//! This is a collection of tools used for converting ParseNodes into LayoutNodes.

use crate::font::{Glyph, Direction, VariantGlyph, MathFont};
use crate::dimensions::{Unit, AnyUnit};
use crate::dimensions::units::{Px, Em, FUnit};
use crate::layout::LayoutSettings;

use super::Style;
use super::builders;
use super::{LayoutNode, LayoutVariant, LayoutGlyph};
use crate::parser::nodes::Rule;
use crate::error::LayoutResult;

pub trait AsLayoutNode<'f, F> {
    fn as_layout<'a>(&self, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>>;
}

impl<'f, F> AsLayoutNode<'f, F> for Glyph<'f, F> {
    fn as_layout<'a>(&self, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        Ok(LayoutNode {
            height: self.height().scaled(config),
            width:  self.advance.scaled(config),
            depth:  self.depth().scaled(config),
            node:   LayoutVariant::Glyph(LayoutGlyph {
                font: self.font,
                gid: self.gid,
                size: Unit::<Em>::new(1.0).scaled(config),
                attachment: self.attachment.scaled(config),
                italics: self.italics.scaled(config),
                offset:  Unit::ZERO,
            })
        })
    }
}

impl<'f, F> AsLayoutNode<'f, F> for Rule {
    fn as_layout<'a>(&self, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        Ok(LayoutNode {
            node:   LayoutVariant::Rule,
            width:  self.width .scaled(config),
            height: self.height.scaled(config),
            depth:  Unit::ZERO,
        })
    }
}

impl<'f, F : MathFont> AsLayoutNode<'f, F> for VariantGlyph {
    fn as_layout<'a>(&self, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        match *self {
            VariantGlyph::Replacement(gid) => {
                let glyph = config.ctx.glyph_from_gid(gid)?;
                glyph.as_layout(config)
            },

            VariantGlyph::Constructable(dir, ref parts) => {
                match dir {
                    Direction::Vertical => {
                        let mut contents = builders::VBox::new();
                        for instr in parts.into_iter().rev() {
                            let glyph = config.ctx.glyph_from_gid(instr.gid)?;
                            contents.add_node(glyph.as_layout(config)?);
                            if instr.overlap != 0 {
                                let overlap = Unit::<FUnit>::new(instr.overlap.into());
                                let kern = -(overlap + glyph.depth()).scaled(config);
                                contents.add_node(kern!(vert: kern));
                            }
                        }

                        Ok(contents.build())
                    },

                    Direction::Horizontal => {
                        let mut contents = builders::HBox::new();
                        for instr in parts {
                            let glyph = config.ctx.glyph_from_gid(instr.gid)?;
                            if instr.overlap != 0 {
                                let kern = -Unit::<FUnit>::new(instr.overlap.into()).scaled(config);
                                contents.add_node(kern!(horz: kern));
                            }
                            contents.add_node(glyph.as_layout(config)?);
                        }

                        Ok(contents.build())
                    }
                }
            },
        }
    }
}

impl<'a, 'f, F> LayoutSettings<'a, 'f, F> {
    fn scale_factor(&self) -> f64 {
        match self.style {
            Style::Display |
            Style::DisplayCramped |
            Style::Text |
            Style::TextCramped
                => 1.0,

            Style::Script |
            Style::ScriptCramped
                => self.ctx.constants.script_percent_scale_down,

            Style::ScriptScript |
            Style::ScriptScriptCramped
                => self.ctx.constants.script_script_percent_scale_down,
        }
    }
    fn scale_font_unit(&self, length: Unit<FUnit>) -> Unit<Px> {
        length * (self.font_size / self.ctx.units_per_em).unlift()
    }

    /// Convert a length given in pixels to a length in font units. The resulting value depends on the selected font size.
    pub fn to_font(&self, length: Unit<Px>) -> Unit<FUnit> {
        length  * (self.ctx.units_per_em / self.font_size).unlift()
    }
}
pub trait Scaled {
    fn scaled<F>(self, config: LayoutSettings<F>) -> Unit<Px>;
}

impl Scaled for Unit<FUnit> {
    fn scaled<F>(self, config: LayoutSettings<F>) -> Unit<Px> {
        config.scale_font_unit(self).scale(config.scale_factor())
    }
}

impl Scaled for Unit<Px> {
    fn scaled<F>(self, config: LayoutSettings<F>) -> Unit<Px> {
        self.scale(config.scale_factor())
    }
}
impl Scaled for Unit<Em> {
    fn scaled<F>(self, config: LayoutSettings<F>) -> Unit<Px> {
        (self * config.font_size).scale(config.scale_factor())
    }
}
impl Scaled for AnyUnit {
    fn scaled<F>(self, config: LayoutSettings<F>) -> Unit<Px> {
        let length = match self {
            AnyUnit::Em(em) => Unit::<Em>::new(em) * config.font_size,
            AnyUnit::Px(px) => Unit::<Px>::new(px)
        };
        length.scale(config.scale_factor())
    }
}
