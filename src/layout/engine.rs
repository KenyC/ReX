//! This module defines the main layout functions that will place the parse nodes in space, given the geometrical information provided in the font. 
//! The most important function here is [`layout`](crate::layout::engine::layout). 
//! Given a slice of [`ParseNode`s](crate::parser::nodes::ParseNode) and some [`LayoutSettings`](crate::layout::LayoutSettings), 
//! this function returns a layout. The layout can then be sent to the renderer (cf [`render`](crate::render)) to create a graphical output.


use std::cmp::{min, max};

use super::builders;
use super::convert::AsLayoutNode;
use super::{Alignment, Layout, LayoutNode, LayoutSettings, LayoutVariant, Style};

use crate::font::MathFont;
use crate::font::{
    kerning::{superscript_kern, subscript_kern},
    VariantGlyph,
    AtomType
};
use super::convert::Scaled;
use super::spacing::{atom_space, Spacing};
use crate::parser::nodes::{BarThickness, MathStyle, ParseNode, Accent, Delimited, GenFraction, Radical, Scripts, Stack};
use crate::parser::symbols::Symbol;
use crate::environments::{Array, ArrayColumnAlign};
use crate::dimensions::{*};
use crate::layout;
use crate::error::{LayoutResult, LayoutError};

/// Entry point to our recursive algorithm
pub fn layout<'a, 'f: 'a, F : MathFont>(nodes: &[ParseNode], config: LayoutSettings<'a, 'f, F>) -> LayoutResult<Layout<'f, F>> {
    layout_recurse(nodes, config, AtomType::Transparent)
}

/// This method takes the parsing nodes and layouts them to layout nodes.
fn layout_recurse<'a, 'f: 'a, F : MathFont>(nodes: &[ParseNode], mut config: LayoutSettings<'a, 'f, F>, parent_next: AtomType) -> LayoutResult<Layout<'f, F>> {
    let mut layout = Layout::new();
    let mut prev = AtomType::Transparent;
    let mut italic_correction = None;

    for idx in 0..nodes.len() {
        let node = &nodes[idx];

        // To determine spacing between glyphs, we look at each pair and their types.
        // Obtain the atom_type from the next node,  if we are the last in the node
        // list then we obtain the atomtype from the next node in parent's list.
        let next = match nodes.get(idx + 1) {
            Some(node) => node.atom_type(),
            None => parent_next,
        };

        let mut current = node.atom_type();
        if current == AtomType::Binary {
            if prev == AtomType::Transparent || prev == AtomType::Binary ||
               prev == AtomType::Relation || prev == AtomType::Open ||
               prev == AtomType::Punctuation {
                current = AtomType::Alpha;
            } else if let AtomType::Operator(_) = prev {
                current = AtomType::Alpha;
            } else if next == AtomType::Relation || next == AtomType::Close ||
                      next == AtomType::Punctuation {
                current = AtomType::Alpha;
            }
        }

        let sp = atom_space(prev, current, config.style);
        let italic_correction_to_apply = italic_correction.take();
        if sp != Spacing::None {
            let kern = sp.to_length().scaled(config);
            layout.add_node(kern!(horz: kern));
        }
        // if there is already no space between consecutive nodes
        // we check whether one should apply italic correction
        else if let Some(italic_correction) = italic_correction_to_apply {
            // Discharge italic correction
            if must_apply_italic_correction_before(node) {
                layout.add_node(kern!(horz : italic_correction));
            }
        }



        match *node {
            ParseNode::Style(sty) => config.style = sty,
            ParseNode::Symbol(symbol) => {
                let node = layout.symbol(symbol, config)?;
                italic_correction = node.is_symbol().map(|s| s.italics);
                if !unicode_math::is_italic(symbol.codepoint) {
                    italic_correction = None;
                }

                layout.add_node(node);
            },
            _ => layout.dispatch(config.clone(), node, next)?,
        }
        prev = current;
    }

    Ok(layout.finalize())
}

fn must_apply_italic_correction_before(node: &ParseNode) -> bool {
    if let Some(symbol) = node.is_symbol() {
        if unicode_math::is_italic(symbol.codepoint) {
            return false;
        }
    }
    true
}

// TODO: this should return layout result
fn layout_node<'a, 'f: 'a, F : MathFont>(node: &ParseNode, config: LayoutSettings<'a, 'f, F>) -> Layout<'f, F> {
    let mut layout = Layout::new();
    layout.dispatch(config, node, AtomType::Transparent);
    layout.finalize()
}

impl<'f, F : MathFont> Layout<'f, F> {

    fn dispatch<'a>(&mut self, config: LayoutSettings<'a, 'f, F>, node: &ParseNode, next: AtomType) -> LayoutResult<()> {
        match *node {
            ParseNode::Symbol(symbol) => self.add_node(self.symbol(symbol, config)?),
            ParseNode::Scripts(ref script) => self.scripts(script, config)?,
            ParseNode::Radical(ref rad) => self.radical(rad, config)?,
            ParseNode::Delimited(ref delim) => self.delimited(delim, config)?,
            ParseNode::Accent(ref acc) => self.accent(acc, config)?,
            ParseNode::GenFraction(ref f) => self.frac(f, config)?,
            ParseNode::Stack(ref stack) => self.substack(stack, config)?,
            ParseNode::Array(ref arr) => self.array(arr, config)?,

            ParseNode::AtomChange(ref ac) => self.add_node(layout(&ac.inner, config)?.as_node()),
            ParseNode::Group(ref gp) => self.add_node(layout(gp, config)?.as_node()),
            ParseNode::Rule(rule) => self.add_node(rule.as_layout(config)?),
            ParseNode::Kerning(kern) => self.add_node(kern!(horz: kern.scaled(config))),

            ParseNode::Color(ref clr) => {
                let inner = layout_recurse(&clr.inner, config, next)?;
                self.add_node(builders::color(inner, clr))
            }

            // TODO: deal with ignored parse nodes
            _ => warn!("ignored ParseNode: {:?}", node),
        }
        Ok(())
    }

    fn symbol<'a>(&self, sym: Symbol, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        // Operators are handled specially.  We may need to find a larger
        // symbol and vertical center it.
        match sym.atom_type {
            AtomType::Operator(_) => self.largeop(sym, config),
            _ => config.ctx.glyph(sym.codepoint)?.as_layout(config)
        }
    }

    fn largeop<'a>(&self, sym: Symbol, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        let glyph = config.ctx.glyph(sym.codepoint)?;
        if config.style > Style::Text {
            let axis_offset = config.ctx.constants.axis_height.scaled(config);
            let largeop = config.ctx.vert_variant(sym.codepoint, config.ctx.constants.display_operator_min_height * config.ctx.units_per_em)?
                .as_layout(config)?;
            let shift = (largeop.height + largeop.depth) * 0.5 - axis_offset;
            Ok(vbox!(offset: shift; largeop))
        } else {
            glyph.as_layout(config)
        }
    }
    
    fn accent<'a>(&mut self, acc: &Accent, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<()> {
        // [ ] The width of the selfing box is the width of the base.
        // [ ] Bottom accents: vertical placement is directly below nucleus,
        //       no correction takes place.
        // [ ] WideAccent vs Accent: Don't expand Accent types.
        let base = layout(&acc.nucleus, config.cramped())?;
        let accent_variant = config.ctx.horz_variant(acc.symbol.codepoint, config.to_font(base.width))?;
        let accent = accent_variant.as_layout(config)?;

        // Attachment points for accent & base are calculated by
        //   (a) Non-symbol: width / 2.0,
        //   (b) Symbol:
        //      1. Attachment point (if there is one)
        //      2. Otherwise: (width + ic) / 2.0
        let base_offset = match layout::is_symbol(&base.contents) {
            Some(sym) => {
                let glyph = config.ctx.glyph_from_gid(sym.gid)?;
                if !glyph.attachment.is_zero() {
                    glyph.attachment.scaled(config)
                } else {
                    let offset = (glyph.advance + glyph.italics) * 0.5;
                    offset.scaled(config)
                }
            }
            None => base.width * 0.5,
        };

        let acc_offset = match accent_variant {
            VariantGlyph::Replacement(sym) => {
                let glyph = config.ctx.glyph_from_gid(sym)?;
                if !glyph.attachment.is_zero() {
                    glyph.attachment.scaled(config)
                } else {
                    // For glyphs without attachmens, we must
                    // also account for combining glyphs
                    let offset = (glyph.bbox.2 + glyph.bbox.0) * 0.5;
                    offset.scaled(config)
                }
            }

            VariantGlyph::Constructable(_, _) => accent.width * 0.5,
        };

        // Do not place the accent any further than you would if given
        // an `x` character in the current style.
        let delta = -min(base.height, config.ctx.constants.accent_base_height.scaled(config));

        // By not placing an offset on this vbox, we are assured that the
        // baseline will match the baseline of `base.as_node()`
        self.add_node(vbox!(hbox!(kern!(horz: base_offset - acc_offset), accent),
                            kern!(vert: delta),
                            base.as_node()));
        
        Ok(())
    }

    fn delimited<'a>(&mut self, delim: &Delimited, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        let inner = layout(&delim.inner, config)?.as_node();

        let min_height = config.ctx.constants.delimited_sub_formula_min_height * config.font_size;
        let null_delimiter_space = config.ctx.constants.null_delimiter_space * config.font_size;

        // Only extend if we meet a certain size
        // TODO: This quick height check doesn't seem to be strong enough,
        // reference: http://tug.org/pipermail/luatex/2010-July/001745.html
        if max(inner.height, -inner.depth) > min_height * 0.5 {
            let axis = config.ctx.constants.axis_height * config.font_size;

            let clearance = max(inner.height - axis, axis - inner.depth) * 2.0;
            let clearance = max(clearance * config.ctx.constants.delimiter_factor,
                            inner.height - inner.depth - config.ctx.constants.delimiter_short_fall * config.font_size);
            let clearance = config.to_font(clearance);

            let left = match delim.left.codepoint {
                '.' => kern!(horz: null_delimiter_space),
                _ => {
                    config.ctx.vert_variant(delim.left.codepoint, clearance)?
                        .as_layout(config)?
                        .centered(axis)
                }
            };

            let right = match delim.right.codepoint {
                '.' => kern!(horz: null_delimiter_space),
                _ => {
                    config.ctx.vert_variant(delim.right.codepoint, clearance)?
                        .as_layout(config)?
                        .centered(axis)
                }
            };

            self.add_node(left);
            self.add_node(inner);
            self.add_node(right);
        } else {
            let left = match delim.left.codepoint {
                '.' => kern!(horz: null_delimiter_space),
                _ => config.ctx.glyph(delim.left.codepoint)?.as_layout(config)?,
            };

            let right = match delim.right.codepoint {
                '.' => kern!(horz: null_delimiter_space),
                _ => config.ctx.glyph(delim.right.codepoint)?.as_layout(config)?,
            };

            self.add_node(left);
            self.add_node(inner);
            self.add_node(right);
        }

        Ok(())
    }
    fn scripts<'a>(&mut self, scripts: &Scripts, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // See: https://tug.org/TUGboat/tb27-1/tb86jackowski.pdf
        //      https://www.tug.org/tugboat/tb30-1/tb94vieth.pdf
        let base = match scripts.base {
            Some(ref base) => layout_node(base, config),
            None => Layout::new(),
        };

        let mut sup = match scripts.superscript {
            Some(ref sup) => layout(sup, config.superscript_variant())?,
            None => Layout::new(),
        };

        let mut sub = match scripts.subscript {
            Some(ref sub) => layout(sub, config.subscript_variant())?,
            None => Layout::new(),
        };

        // We use a different algoirthm for handling scripts for operators with limits.
        // This is where he handle Operators with limits.
        if let Some(ref b) = scripts.base {
            if AtomType::Operator(true) == b.atom_type() {
                self.operator_limits(base, sup, sub, config);
                return Ok(());
            }
        }

        // We calculate the vertical positions of the scripts.  The `adjust_up`
        // variable will describe how far we need to adjust the superscript up.
        let mut adjust_up = Length::zero();
        let mut adjust_down = Length::zero();
        let mut sup_kern = Length::zero();
        let mut sub_kern = Length::zero();

        if scripts.superscript.is_some() {
            // Use default font values for first iteration of vertical height.
            adjust_up = match config.style.is_cramped() {
                true => config.ctx.constants.superscript_shift_up_cramped,
                false => config.ctx.constants.superscript_shift_up,
            }
            .scaled(config);

            // TODO: These checks should be recursive?
            let mut height = base.height;
            if let Some(ref b) = scripts.base {
                if b.atom_type() != AtomType::Operator(false) {
                    // For accents whose base is a simple symbol we do not take
                    // the accent into account while positioning the superscript.
                    if let ParseNode::Accent(ref acc) = **b {
                        use crate::parser::is_symbol;
                        if let Some(sym) = is_symbol(&acc.nucleus) {
                            height = config.ctx.glyph(sym.codepoint)?.height().scaled(config);
                        }
                    }
                    // Apply italics correction is base is a symbol
                    else if let Some(base_sym) = base.is_symbol() {
                        // Lookup font kerning of superscript is also a symbol
                        if let Some(sup_sym) = sup.is_symbol() {
                            let bg = config.ctx.glyph_from_gid(base_sym.gid)?;
                            let sg = config.ctx.glyph_from_gid(sup_sym.gid)?;
                            let kern = superscript_kern(&bg, &sg, config.to_font(adjust_up)).scaled(config);
                            sup_kern = base_sym.italics + kern;
                        } else {
                            sup_kern = base_sym.italics;
                        }
                    }
                }
            }

            let drop_max = config.ctx.constants.superscript_baseline_drop_max.scaled(config);
            adjust_up = max!(adjust_up,
                            height - drop_max,
                            config.ctx.constants.superscript_bottom_min.scaled(config) - sup.depth);
        }

        // We calculate the vertical position of the subscripts.  The `adjust_down`
        // variable will describe how far we need to adjust the subscript down.
        if scripts.subscript.is_some() {
            // Use default font values for first iteration of vertical height.
            adjust_down = max!(config.ctx.constants.subscript_shift_down.scaled(config),
                                sub.height - config.ctx.constants.subscript_top_max.scaled(config),
                                config.ctx.constants.subscript_baseline_drop_min.scaled(config) - base.depth);

            // Provided that the base and subscript are symbols, we apply
            // kerning values found in the kerning font table
            if let Some(ref b) = scripts.base {
                if let Some(base_sym) = base.is_symbol() {
                    if AtomType::Operator(false) == b.atom_type() {
                        // This recently changed in LuaTeX.  See `nolimitsmode`.
                        // This needs to be the glyph information _after_ layout for base.
                        sub_kern = -config.ctx.glyph_from_gid(base_sym.gid)?.italics.scaled(config);
                    }
                }

                if let (Some(ssym), Some(bsym)) = (sub.is_symbol(), base.is_symbol()) {
                    let bg = config.ctx.glyph_from_gid(bsym.gid)?;
                    let sg = config.ctx.glyph_from_gid(ssym.gid)?;
                    sub_kern += subscript_kern(&bg, &sg, config.to_font(adjust_down)).scaled(config);
                }
            }
        }

        // TODO: lazy gap fix; see BottomMaxWithSubscript
        if scripts.subscript.is_some() && scripts.superscript.is_some() {
            let sup_bot = adjust_up + sup.depth;
            let sub_top = sub.height - adjust_down;
            let gap_min = config.ctx.constants.sub_superscript_gap_min.scaled(config);
            if sup_bot - sub_top < gap_min {
                let adjust = (gap_min - sup_bot + sub_top) * 0.5;
                adjust_up += adjust;
                adjust_down += adjust;
            }
        }

        let mut contents = builders::VBox::new();
        if scripts.superscript.is_some() {
            if !sup_kern.is_zero() {
                sup.contents.insert(0, kern!(horz: sup_kern));
                sup.width += sup_kern;
            }

            let corrected_adjust = adjust_up - sub.height + adjust_down;
            contents.add_node(sup.as_node());
            contents.add_node(kern!(vert: corrected_adjust));
        }

        contents.set_offset(adjust_down);
        if scripts.subscript.is_some() { 
            if !sub_kern.is_zero() {
                sub.contents.insert(0, kern!(horz: sub_kern));
                sub.width += sub_kern;
            }
            contents.add_node(sub.as_node());
        }

        self.add_node(base.as_node());
        self.add_node(contents.build());

        Ok(())
    }

    fn operator_limits<'a>(&mut self, base: Layout<'f, F>, sup: Layout<'f, F>, sub: Layout<'f, F>, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // Provided that the operator is a simple symbol, we need to account
        // for the italics correction of the symbol.  This how we "center"
        // the superscript and subscript of the limits.
        let delta = match base.is_symbol() {
            Some(gly) => gly.italics,
            None => Length::zero()
        };

        // Next we calculate the kerning required to separate the superscript
        // and subscript (respectively) from the base.
        let sup_kern = max(config.ctx.constants.upper_limit_baseline_rise_min.scaled(config),
                        config.ctx.constants.upper_limit_gap_min.scaled(config) - sup.depth);
        let sub_kern = max(config.ctx.constants.lower_limit_gap_min.scaled(config),
                        config.ctx.constants.lower_limit_baseline_drop_min.scaled(config) - sub.height) -
                    base.depth;

        // We need to preserve the baseline of the operator when
        // attaching the scripts.  Since the base should already
        // be aligned, we only need to offset by the addition of
        // subscripts.
        let offset = sub.height + sub_kern;

        // We will construct a vbox containing the superscript/base/subscript.
        // We will all of these nodes, so we widen each to the largest.
        let width = max!(base.width, sub.width + delta * 0.5, sup.width + delta * 0.5);

        self.add_node(vbox![
            offset: offset;
            hbox![align: Alignment::Centered(sup.width);
                width: width;
                kern![horz: delta * 0.5],
                sup.as_node()
            ],

            kern!(vert: sup_kern),
            base.centered(width).as_node(),
            kern!(vert: sub_kern),

            hbox![align: Alignment::Centered(sub.width);
                width: width;
                kern![horz: -delta * 0.5],
                sub.as_node()
            ]
        ]);
        
        Ok(())
    }

    fn frac<'a>(&mut self, frac: &GenFraction, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        let config = match frac.style {
            MathStyle::NoChange => config.clone(),
            MathStyle::Display => config.with_display(),
            MathStyle::Text => config.with_text(),
        };

        let bar = match frac.bar_thickness {
            BarThickness::Default => config.ctx.constants.fraction_rule_thickness.scaled(config),
            BarThickness::None => Length::zero(),
            BarThickness::Unit(u) => u.scaled(config),
        };

        let mut n = layout(&frac.numerator, config.numerator())?;
        let mut d = layout(&frac.denominator, config.denominator())?;

        if n.width > d.width {
            d.alignment = Alignment::Centered(d.width);
            d.width = n.width;
        } else {
            n.alignment = Alignment::Centered(n.width);
            n.width = d.width;
        }

        let numer = n.as_node();
        let denom = d.as_node();

        let axis = config.ctx.constants.axis_height.scaled(config);
        let shift_up;
        let shift_down;
        let gap_num;
        let gap_denom;

        if config.style > Style::Text {
            shift_up = config.ctx.constants.fraction_numerator_display_style_shift_up.scaled(config);
            shift_down = config.ctx.constants.fraction_denominator_display_style_shift_down.scaled(config);
            gap_num = config.ctx.constants.fraction_num_display_style_gap_min.scaled(config);
            gap_denom = config.ctx.constants.fraction_denom_display_style_gap_min.scaled(config);
        } else {
            shift_up = config.ctx.constants.fraction_numerator_shift_up.scaled(config);
            shift_down = config.ctx.constants.fraction_denominator_shift_down.scaled(config);
            gap_num = config.ctx.constants.fraction_numerator_gap_min.scaled(config);
            gap_denom = config.ctx.constants.fraction_denominator_gap_min.scaled(config);
        }

        let kern_num = max(shift_up - axis - bar * 0.5, gap_num - numer.depth);
        let kern_den = max(shift_down + axis - denom.height - bar * 0.5, gap_denom);
        let offset = denom.height + kern_den + bar * 0.5 - axis;

        let width = numer.width;
        let inner = vbox!(offset: offset;
            numer,
            kern!(vert: kern_num),
            rule!(width: width, height: bar),
            kern!(vert: kern_den),
            denom
        );

        let null_delimiter_space = config.ctx.constants.null_delimiter_space * config.font_size;
        let axis_height = config.ctx.constants.axis_height * config.font_size;
        // Enclose fraction with delimiters if provided, otherwise with a NULL_DELIMITER_SPACE.
        let left = match frac.left_delimiter {
            None => kern!(horz: null_delimiter_space),
            Some(sym) => {
                let clearance = max(inner.height - axis_height, axis_height - inner.depth) * 2.0;
                let clearance = max(clearance, config.ctx.constants.delimited_sub_formula_min_height * config.font_size);

                config.ctx.vert_variant(sym.codepoint, config.to_font(clearance))?
                    .as_layout(config)?
                    .centered(axis_height.scaled(config))
            }
        };

        let right = match frac.right_delimiter {
            None => kern!(horz: null_delimiter_space),
            Some(sym) => {
                let clearance = max(inner.height - axis_height, axis_height - inner.depth) * 2.0;
                let clearance = max(clearance, config.ctx.constants.delimited_sub_formula_min_height * config.font_size);

                config.ctx.vert_variant(sym.codepoint, config.to_font(clearance))?
                    .as_layout(config)?
                    .centered(axis_height.scaled(config))
            }
        };

        self.add_node(left);
        self.add_node(inner);
        self.add_node(right);
        
        Ok(())
    }
    fn radical<'a>(&mut self, rad: &Radical, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // reference rule 11 from pg 443 of TeXBook
        let contents = layout(&rad.inner, config.cramped())?.as_node();

        // obtain minimum clearange between radicand and radical bar
        // and cache other sizes that will be needed
        let gap = match config.style >= Style::Display {
            true => config.ctx.constants.radical_display_style_vertical_gap.scaled(config),
            false => config.ctx.constants.radical_vertical_gap.scaled(config),
        };

        let rule_thickness = config.ctx.constants.radical_rule_thickness.scaled(config);
        let rule_ascender = config.ctx.constants.radical_extra_ascender.scaled(config);

        // determine size of radical glyph
        let inner_height = (contents.height - contents.depth) + gap + rule_thickness;
        let sqrt = config.ctx.vert_variant('√', config.to_font(inner_height))?.as_layout(config)?;

        // pad between radicand and radical bar
        let delta = (sqrt.height - sqrt.depth - inner_height) * 0.5 + rule_thickness;
        let gap = max(delta, gap);

        // offset radical symbol
        let offset = rule_thickness + gap + contents.height;
        let offset = sqrt.height - offset;

        // padding above sqrt
        // TODO: This is unclear
        let top_padding = rule_ascender - rule_thickness;

        self.add_node(vbox![offset: offset; sqrt]);
        self.add_node(vbox![kern!(vert: top_padding),
                            rule!(width:  contents.width, height: rule_thickness),
                            kern!(vert: gap),
                            contents]);
        
        Ok(())
    }

    fn substack<'a>(&mut self, stack: &Stack, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // Don't bother constructing a new node if there is nothing.
        if stack.lines.len() == 0 {
            return Ok(());
        }

        // Layout each line in the substack, and track which line is the widest
        let mut lines: Vec<Layout<F>> = Vec::with_capacity(stack.lines.len());
        let mut widest = Length::zero();
        let mut widest_idx = 0;
        for (n, line) in stack.lines.iter().enumerate() {
            let line = layout(line, config)?;
            if line.width > widest {
                widest = line.width;
                widest_idx = n;
            }
            lines.push(line);
        }

        // Center lines according to widest variant
        for (n, line) in lines.iter_mut().enumerate() {
            if n == widest_idx {
                continue;
            }
            line.alignment = Alignment::Centered(line.width);
            line.width = widest;
        }

        // The line gap will be taken from STACK_GAP constants
        let gap_min = if config.style > Style::Text {
            config.ctx.constants.stack_display_style_gap_min.scaled(config)
        } else {
            config.ctx.constants.stack_gap_min.scaled(config)
        };

        // No idea.
        let gap_try = if config.style > Style::Text {
            config.ctx.constants.stack_top_display_style_shift_up
            - config.ctx.constants.axis_height
            + config.ctx.constants.stack_bottom_shift_down
            - config.ctx.constants.accent_base_height * 2.0
        } else {
            config.ctx.constants.stack_top_shift_up
            - config.ctx.constants.axis_height
            + config.ctx.constants.stack_bottom_shift_down
            - config.ctx.constants.accent_base_height * 2.0
        }
        .scaled(config);

        // Join the lines with appropriate spacing inbetween
        let mut vbox = builders::VBox::new();
        let length = lines.len();
        for (idx, line) in lines.into_iter().enumerate() {
            let prev = line.depth;
            vbox.add_node(line.as_node());

            // Try for an ideal gap, otherwise use the minimum
            if idx < length {
                let gap = max(gap_min, gap_try - prev);
                vbox.add_node(kern![vert: gap]);
            }
        }

        // Vertically center the stack to the axis
        let offset = (vbox.height + vbox.depth) * 0.5 - config.ctx.constants.axis_height.scaled(config);
        vbox.set_offset(offset);
        self.add_node(vbox.build());
        
        Ok(())
    }

    fn array<'a>(&mut self, array: &Array, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // TODO: let jot = UNITS_PER_EM / 4;
        let strut_height = Length::new(0.7, Em) * config.font_size; // \strutbox height = 0.7\baseline
        let strut_depth = Length::new(0.3, Em) * config.font_size; // \strutbox depth  = 0.3\baseline
        let row_sep = Length::new(0.25, Em) * config.font_size;
        let column_sep = Length::new(5.0 / 12.0, Em) * config.font_size;

        // Don't bother constructing a new node if there is nothing.
        let num_rows = array.rows.len();
        let num_columns = array.rows.iter().map(Vec::len).max().unwrap_or(0);
        if num_columns == 0 {
            return Ok(());
        }

        let mut columns = Vec::with_capacity(num_columns);
        for _ in 0..num_columns {
            columns.push(Vec::with_capacity(num_rows));
        }

        // Layout each node in each row, while keeping track of the largest row/col
        let mut col_widths = vec![Length::zero(); num_columns];
        let mut row_heights = Vec::with_capacity(num_rows);
        let mut prev_depth = Length::zero();
        let mut row_max = strut_height;
        for row in &array.rows {
            let mut max_depth = Length::zero();
            for col_idx in 0..num_columns {
                // layout row element if it exists
                let square = match row.get(col_idx) {
                    Some(r) => {
                        // record the max height/width for current row/col
                        let square = layout(r, config)?;
                        row_max = max(square.height, row_max);
                        max_depth = max(max_depth, -square.depth);
                        col_widths[col_idx] = max(col_widths[col_idx], square.width);
                        square
                    },
                    _ => Layout::new(),
                };

                columns[col_idx].push(square);
            }

            // ensure row height >= strut_height
            row_heights.push(row_max + prev_depth);
            row_max = strut_height;
            prev_depth = max(Length::zero(), max_depth - strut_depth);
        }

        // TODO: reference row layout here: crl
        // the body of the matrix is an hbox of column vectors.
        let mut hbox = builders::HBox::new();

        // If there are no delimiters, insert a null space.  Otherwise we insert
        // the delimiters _after_ we have laidout the body of the matrix.
        if array.left_delimiter.is_none() {
            hbox.add_node(kern![horz: config.ctx.constants.null_delimiter_space * config.font_size]);
        }

        // layout the body of the matrix
        let column_iter = 
            Iterator
            ::zip(columns.into_iter(), array.col_format.columns.iter())
            .enumerate()
        ;
        for (col_idx, (col, col_format)) in column_iter {
            let alignment = col_format.alignment;
            let mut vbox = builders::VBox::new();
            for (row_idx, mut row) in col.into_iter().enumerate() {
                // Center columns as necessary
                if row.width < col_widths[col_idx] {
                    row.alignment = match alignment {
                        ArrayColumnAlign::Centered => Alignment::Centered(row.width),
                        ArrayColumnAlign::Left     => Alignment::Left,
                        ArrayColumnAlign::Right    => Alignment::Right(row.width),
                    };
                    row.width = col_widths[col_idx];
                }

                // Add additional strut if required to align rows
                if row.height < row_heights[row_idx] {
                    let diff = row_heights[row_idx] - row.height;
                    vbox.add_node(kern![vert: diff]);
                }

                // add inter-row spacing.  Since vboxes get their depth from the their
                // last entry, we manually add the depth from the last row if it exceeds
                // the row_seperation.
                // FIXME: This should be actual depth, not additional kerning
                let node = row.as_node();
                if row_idx + 1 == num_rows {
                    let depth = max(-node.depth, row_sep);
                    vbox.add_node(node);
                    vbox.add_node(kern![vert: depth]);
                } else {
                    vbox.add_node(node);
                    vbox.add_node(kern![vert: row_sep]);
                }
            }

            // add column to matrix body and column seperation spacing except for last one.
            hbox.add_node(vbox.build());
            if col_idx + 1 < num_columns {
                hbox.add_node(kern![horz: column_sep]);
            }
        }

        if array.right_delimiter.is_none() {
            hbox.add_node(kern![horz: config.ctx.constants.null_delimiter_space * config.font_size]);
        }

        // TODO: Reference array vertical alignment (optional [bt] arguments)
        // Vertically center the array on axis.
        // Note: hbox has no depth, so hbox.height is total height.
        let height = hbox.height;
        let mut vbox = builders::VBox::new();
        let offset = height * 0.5 - config.ctx.constants.axis_height.scaled(config);
        vbox.set_offset(offset);
        vbox.add_node(hbox.build());
        let vbox = vbox.build();

        // Now that we know the layout of the matrix body we can place scaled delimiters
        // First check if there are any delimiters to add, if not just return.
        if array.left_delimiter.is_none() && array.right_delimiter.is_none() {
            self.add_node(vbox);
            return Ok(());
        }

        // place delimiters in an hbox surrounding the matrix body
        let mut hbox = builders::HBox::new();
        let axis = config.ctx.constants.axis_height.scaled(config);
        let clearance = max(height * config.ctx.constants.delimiter_factor,
                            height - config.ctx.constants.delimiter_short_fall * config.font_size);

        if let Some(left) = array.left_delimiter {
            let left = config.ctx.vert_variant(left.codepoint, config.to_font(clearance))?
                .as_layout(config)?
                .centered(axis);
            hbox.add_node(left);
        }

        hbox.add_node(vbox);
        if let Some(right) = array.right_delimiter {
            let right = config.ctx.vert_variant(right.codepoint, config.to_font(clearance))?
                .as_layout(config)?
                .centered(axis);
            hbox.add_node(right);
        }
        self.add_node(hbox.build());

        Ok(())
    }
}

