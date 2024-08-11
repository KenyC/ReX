//! This module defines the main layout functions that will place the parse nodes in space, given the geometrical information provided in the font. 
//! The most important function here is [`layout`](crate::layout::engine::layout). 
//! Given a slice of [`ParseNode`s](crate::parser::nodes::ParseNode) and some [`LayoutSettings`](crate::layout::LayoutSettings), 
//! this function returns a layout. The layout can then be sent to the renderer (cf [`render`](crate::render)) to create a graphical output.


use std::unimplemented;

use super::builders;
use super::convert::AsLayoutNode;
use super::{Alignment, Layout, LayoutNode, LayoutSettings, LayoutVariant, Style};

use crate::font::MathFont;
use crate::font::{
    kerning::{superscript_kern, subscript_kern},
    VariantGlyph,
    TexSymbolType
};
use crate::layout::constants::{BASELINE_SKIP, COLUMN_SEP, DOUBLE_RULE_SEP, JOT, LINE_SKIP_ARRAY, LINE_SKIP_LIMIT_ARRAY, RULE_WIDTH, STRUT_DEPTH, STRUT_HEIGHT};
use super::convert::Scaled;
use super::spacing::{atom_space, Spacing};
use crate::parser::nodes::{Accent, Array, ArrayColumnAlign, BarThickness, ColSeparator, Delimited, ExtendedDelimiter, GenFraction, MathStyle, ParseNode, PlainText, Radical, Scripts, Stack};
use crate::parser::symbols::Symbol;
use crate::dimensions::Unit;
use crate::dimensions::units::Px;
use crate::layout;
use crate::error::{LayoutResult, LayoutError};

/// Entry point to our recursive algorithm
pub fn layout<'a, 'f: 'a, F : MathFont>(nodes: &[ParseNode], config: LayoutSettings<'a, 'f, F>) -> LayoutResult<Layout<'f, F>> {
    layout_recurse(nodes, config, TexSymbolType::Transparent)
}

/// This method takes the parsing nodes and layouts them to layout nodes.
fn layout_recurse<'a, 'f: 'a, F : MathFont>(nodes: &[ParseNode], mut config: LayoutSettings<'a, 'f, F>, parent_next: TexSymbolType) -> LayoutResult<Layout<'f, F>> {
    let mut layout = Layout::new();
    let mut prev = None;
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
        if current == TexSymbolType::Binary {
            if let 
                  None 
                | Some(TexSymbolType::Binary) 
                | Some(TexSymbolType::Relation) 
                | Some(TexSymbolType::Open) 
                | Some(TexSymbolType::Punctuation) 
                | Some(TexSymbolType::Operator(_)) = prev {
                current = TexSymbolType::Alpha;
            } else if let 
                  TexSymbolType::Relation 
                | TexSymbolType::Close 
                | TexSymbolType::Punctuation = next {
                current = TexSymbolType::Alpha;
            }
        }

        let sp = 
            if let Some(prev) = prev 
            { atom_space(prev, current, config.style) }
            else 
            { Spacing::None }
        ;

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

        // Transparent items should be ignored for parsing rules
        if current != TexSymbolType::Transparent {
            prev = Some(current);
        }
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
fn layout_node<'a, 'f: 'a, F : MathFont>(node: &ParseNode, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<Layout<'f, F>> {
    let mut layout = Layout::new();
    layout.dispatch(config, node, TexSymbolType::Transparent)?;
    Ok(layout.finalize())
}

impl<'f, F : MathFont> Layout<'f, F> {

    fn dispatch<'a>(&mut self, config: LayoutSettings<'a, 'f, F>, node: &ParseNode, next: TexSymbolType) -> LayoutResult<()> {
        match *node {
            ParseNode::Symbol(symbol) => self.add_node(self.symbol(symbol, config)?),
            ParseNode::Scripts(ref script) => self.scripts(script, config)?,
            ParseNode::Radical(ref rad) => self.radical(rad, config)?,
            ParseNode::Delimited(ref delim) => self.delimited(delim, config)?,
            ParseNode::ExtendedDelimiter(ref delim) => self.extended_delimiter(delim, config)?,
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

            ParseNode::DummyNode(_) => (),

            ParseNode::PlainText(PlainText {ref text}) => {
                for character in text.chars() {
                    if character.is_ascii_whitespace() {
                        self.add_node(kern![horz : Spacing::Medium.to_length().scaled(config)])
                    }
                    else {
                        self.add_node(config.ctx.glyph(character)?.as_layout(config)?);
                    }
                }
            },

            // TODO: understand whether this is needed anywhere
            ParseNode::Style(_)     => unimplemented!(),
        }
        Ok(())
    }

    fn symbol<'a>(&self, sym: Symbol, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        // Operators are handled specially.  We may need to find a larger
        // symbol and vertical center it.
        match sym.atom_type {
            TexSymbolType::Operator(_) => self.largeop(sym, config),
            _ => config.ctx.glyph(sym.codepoint)?.as_layout(config)
        }
    }

    fn largeop<'a>(&self, sym: Symbol, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<LayoutNode<'f, F>> {
        let glyph = config.ctx.glyph(sym.codepoint)?;
        if config.style > Style::Text {
            let axis_offset = config.ctx.constants.axis_height.scaled(config);
            let largeop = config.ctx.vert_variant(sym.codepoint, config.ctx.constants.display_operator_min_height * config.ctx.units_per_em)?
                .as_layout(config)?;
            let shift = (largeop.height + largeop.depth).scale(0.5) - axis_offset;
            Ok(vbox!(offset: shift; largeop))
        } else {
            glyph.as_layout(config)
        }
    }
    
    fn accent<'a>(&mut self, acc: &Accent, config: LayoutSettings<'a, 'f, F>) -> LayoutResult<()> {
        // [ ] The width of the selfing box is the width of the base.
        // [ ] Bottom accents: vertical placement is directly below nucleus,
        //       no correction takes place.
        // [x] WideAccent vs Accent: Don't expand Accent types.
        let base = layout(&acc.nucleus, config.cramped())?;
        let symbol = &acc.symbol;
        let accent_variant =
            if acc.extend 
                { config.ctx.horz_variant(symbol.codepoint, config.to_font(base.width))? }
            // to not extend, we consider the trivial variant glyph where the glyph itself is used as replacement
            else 
                { VariantGlyph::Replacement(config.ctx.glyph(symbol.codepoint)?.gid) }
        ;
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
                    let offset = (glyph.advance + glyph.italics).scale(0.5);
                    offset.scaled(config)
                }
            }
            None => base.width.scale(0.5),
        };

        let acc_offset = match accent_variant {
            VariantGlyph::Replacement(sym) => {
                let glyph = config.ctx.glyph_from_gid(sym)?;
                if !glyph.attachment.is_zero() {
                    glyph.attachment.scaled(config)
                } else {
                    // For glyphs without attachmens, we must
                    // also account for combining glyphs
                    let offset = (glyph.bbox.2 + glyph.bbox.0).scale(0.5);
                    offset.scaled(config)
                }
            }

            VariantGlyph::Constructable(_, _) => accent.width.scale(0.5),
        };

        // Do not place the accent any further than you would if given
        // an `x` character in the current style.
        let delta = -Unit::min(base.height, config.ctx.constants.accent_base_height.scaled(config));

        // By not placing an offset on this vbox, we are assured that the
        // baseline will match the baseline of `base.as_node()`
        self.add_node(vbox!(hbox!(kern!(horz: base_offset - acc_offset), accent),
                            kern!(vert: delta),
                            base.as_node()));
        
        Ok(())
    }

    fn delimited<'a>(&mut self, delim: &Delimited, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // let inner = layout(&delim.inner, config)?.as_node();
        let mut inners = Vec::with_capacity(delim.inners().len());
        let mut max_height = Unit::ZERO;
        let mut min_depth  = Unit::ZERO;
        for inner_parse_nodes in delim.inners() {
            let inner = layout(inner_parse_nodes.as_slice(), config)?.as_node();
            max_height = Unit::max(max_height, inner.height);
            min_depth  = Unit::min(min_depth,  inner.depth);
            inners.push(inner);
        }


        let delimiters = delim.delimiters();
        for (symbol, inner) in Iterator::zip(delimiters.iter(), inners)  {
            self.add_node(extend_delimiter(*symbol, max_height, min_depth, config)?);
            self.add_node(inner);
        }
        let right_symbol = delimiters.last().unwrap();
        self.add_node(extend_delimiter(*right_symbol, max_height, min_depth, config)?);

        Ok(())
    }
    fn scripts<'a>(&mut self, scripts: &Scripts, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        // See: https://tug.org/TUGboat/tb27-1/tb86jackowski.pdf
        //      https://www.tug.org/tugboat/tb30-1/tb94vieth.pdf
        let base = match scripts.base {
            Some(ref base) => layout_node(base, config)?,
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
            if TexSymbolType::Operator(true) == b.atom_type() {
                self.operator_limits(base, sup, sub, config)?;
                return Ok(());
            }
        }

        // We calculate the vertical positions of the scripts.  The `adjust_up`
        // variable will describe how far we need to adjust the superscript up.
        let mut adjust_up = Unit::ZERO;
        let mut adjust_down = Unit::ZERO;
        let mut sup_kern = Unit::ZERO;
        let mut sub_kern = Unit::ZERO;

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
                if b.atom_type() != TexSymbolType::Operator(false) {
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
                    if TexSymbolType::Operator(false) == b.atom_type() {
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
                let adjust = (gap_min - sup_bot + sub_top).scale(0.5);
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
            None => Unit::ZERO
        };

        // Next we calculate the kerning required to separate the superscript
        // and subscript (respectively) from the base.
        let sup_kern = Unit::max(config.ctx.constants.upper_limit_baseline_rise_min.scaled(config),
                        config.ctx.constants.upper_limit_gap_min.scaled(config) - sup.depth);
        let sub_kern = Unit::max(config.ctx.constants.lower_limit_gap_min.scaled(config),
                        config.ctx.constants.lower_limit_baseline_drop_min.scaled(config) - sub.height) -
                    base.depth;

        // We need to preserve the baseline of the operator when
        // attaching the scripts.  Since the base should already
        // be aligned, we only need to offset by the addition of
        // subscripts.
        let offset = sub.height + sub_kern;

        // We will construct a vbox containing the superscript/base/subscript.
        // We will all of these nodes, so we widen each to the largest.
        let width = max!(base.width, sub.width + delta.scale(0.5), sup.width + delta.scale(0.5));

        self.add_node(vbox![
            offset: offset;
            hbox![align: Alignment::Centered(sup.width);
                width: width;
                kern![horz: delta.scale(0.5)],
                sup.as_node()
            ],

            kern!(vert: sup_kern),
            base.centered(width).as_node(),
            kern!(vert: sub_kern),

            hbox![align: Alignment::Centered(sub.width);
                width: width;
                kern![horz: -delta.scale(0.5)],
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
            BarThickness::None => Unit::ZERO,
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

        let kern_num = Unit::max(shift_up - axis - bar.scale(0.5), gap_num - numer.depth);
        let kern_den = Unit::max(shift_down + axis - denom.height - bar.scale(0.5), gap_denom);
        let offset = denom.height + kern_den + bar.scale(0.5) - axis;

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
                let clearance = Unit::max(inner.height - axis_height, axis_height - inner.depth).scale(2.0);
                let clearance = Unit::max(clearance, config.ctx.constants.delimited_sub_formula_min_height * config.font_size);

                config.ctx.vert_variant(sym.codepoint, config.to_font(clearance))?
                    .as_layout(config)?
                    .centered(axis_height.scaled(config))
            }
        };

        let right = match frac.right_delimiter {
            None => kern!(horz: null_delimiter_space),
            Some(sym) => {
                let clearance = Unit::max(inner.height - axis_height, axis_height - inner.depth).scale(2.0);
                let clearance = Unit::max(clearance, config.ctx.constants.delimited_sub_formula_min_height * config.font_size);

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
        let delta = (sqrt.height - sqrt.depth - inner_height).scale(0.5) + rule_thickness;
        let gap = Unit::max(delta, gap);

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
        let mut widest = Unit::ZERO;
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
            - config.ctx.constants.accent_base_height.scale(2.0)
        } else {
            config.ctx.constants.stack_top_shift_up
            - config.ctx.constants.axis_height
            + config.ctx.constants.stack_bottom_shift_down
            - config.ctx.constants.accent_base_height.scale(2.0)
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
                let gap = Unit::max(gap_min, gap_try - prev);
                vbox.add_node(kern![vert: gap]);
            }
        }

        // Vertically center the stack to the axis
        let offset = (vbox.height + vbox.depth).scale(0.5) - config.ctx.constants.axis_height.scaled(config);
        vbox.set_offset(offset);
        self.add_node(vbox.build());
        
        Ok(())
    }

    fn extended_delimiter<'a>(&mut self, delim: &ExtendedDelimiter, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        let ExtendedDelimiter { symbol, height_enclosed_content }  = delim; 

        let height_enclosed_content = height_enclosed_content.scaled(config);

        self.add_node(extend_delimiter(*symbol, height_enclosed_content, Unit::ZERO, config)?);
        Ok(())
    }


    fn array<'a>(&mut self, array: &Array, config: LayoutSettings<'a, 'f, F>) -> Result<(), LayoutError> {
        let cell_layout_settings = config.layout_style(array.cell_layout_style);
        let normal_baseline_skip = BASELINE_SKIP; 
        let strut_height     = normal_baseline_skip.scale(STRUT_HEIGHT) * config.font_size; 
        let strut_depth      = - normal_baseline_skip.scale(STRUT_DEPTH)  * config.font_size; 

        let jot = if array.extra_row_sep { JOT } else { Unit::ZERO };
        let baseline_skip = normal_baseline_skip * config.font_size + jot * Unit::standard_pt_to_px();
        let line_skip = (LINE_SKIP_ARRAY + jot) * Unit::standard_pt_to_px();
        let line_skip_limit = (LINE_SKIP_LIMIT_ARRAY + jot)  * Unit::standard_pt_to_px();

        let half_col_sep     = COLUMN_SEP      * Unit::standard_pt_to_px(); 
        let rule_width       = RULE_WIDTH      * Unit::standard_pt_to_px();
        let double_rule_sep  = DOUBLE_RULE_SEP * Unit::standard_pt_to_px();

        let null_delimiter_space = config.ctx.constants.null_delimiter_space * config.font_size;


        // Don't bother constructing a new node if there is nothing.
        let num_rows = array.rows.len();
        let num_columns = array.rows.iter().map(Vec::len).max().unwrap_or(0);
        if num_columns == 0 {
            return Ok(());
        }

        // -- LAY OUT ALL NODES OF ARRAY
        // Columns of an array may be separated by @-expressions
        // We treat @-expressions are ordinary columns, except for the fact that
        // @-expression content is the same in every row
        // We compute how many columns there are, when taking into account @-expressions
        let all_separators = &array.col_format.separators;
        // We count the number of columns including @-expr columns
        let num_columns_at = num_columns + all_separators
            .iter().map(|separators| separators.iter()) 
            .flatten()
            .filter(|separator| matches!(separator, ColSeparator::AtExpression(_)))
            .count()
        ;
        // we store alignments information ; we can also use this array to check if a column was an @-expression or not
        let mut alignments : Vec<Option<ArrayColumnAlign>> = Vec::with_capacity(num_columns_at); 
        let mut columns : Vec<Vec<Layout<'f, F>>> = Vec::with_capacity(num_columns_at);
        let mut n_vertical_bars : Vec<u8> = Vec::with_capacity(num_columns_at + 1);
        let mut current_n_vertical_bars = 0;

        for separator in &all_separators[0] {
            match separator {
                ColSeparator::VerticalBars(n_bars) => 
                    current_n_vertical_bars += n_bars,
                ColSeparator::AtExpression(nodes) => {
                    let node = layout(&nodes, cell_layout_settings)?;
                    let mut column = Vec::with_capacity(num_rows);
                    for _ in 0 .. num_rows {
                        column.push(node.clone());
                    }
                    columns.push(column);
                    alignments.push(None);
                    n_vertical_bars.push(std::mem::replace(&mut current_n_vertical_bars, 0));
                },
            }
        }
        for (i, separators) in all_separators[1..].iter().enumerate() {
            // first comes the real column
            let mut column = Vec::with_capacity(num_rows);
            for j in 0 .. num_rows {
                let cell_node = array.rows
                    .get(j)
                    .and_then(|row| row.get(i))
                ;  
                let layout = match cell_node {
                    Some(cell_node) => layout(&cell_node, cell_layout_settings)?,
                    None => Layout::new(),
                };
                column.push(layout);
            }
            columns.push(column);
            alignments.push(Some(array.col_format.alignment[i]));
            n_vertical_bars.push(std::mem::replace(&mut current_n_vertical_bars, 0));

            // then comes the separators
            for separator in separators {
                match separator {
                    ColSeparator::VerticalBars(n_bars) => 
                        current_n_vertical_bars += n_bars,
                    ColSeparator::AtExpression(nodes) => {
                        let node = layout(&nodes, cell_layout_settings)?;
                        let mut column = Vec::with_capacity(num_rows);
                        for _ in 0 .. num_rows {
                            column.push(node.clone());
                        }
                        columns.push(column);
                        alignments.push(None);
                        n_vertical_bars.push(std::mem::replace(&mut current_n_vertical_bars, 0));
                    },
                }
            }
        }
        n_vertical_bars.push(std::mem::replace(&mut current_n_vertical_bars, 0));

        debug_assert_eq!(columns.len(), num_columns_at);
        debug_assert_eq!(alignments.len(), num_columns_at);
        debug_assert_eq!(n_vertical_bars.len(), num_columns_at + 1);


        // let mut columns = Vec::with_capacity(num_columns);
        // for _ in 0..num_columns {
        //     columns.push(Vec::with_capacity(num_rows));
        // }

        // -- COMPUTE COLUMN WIDTHS AND BASELINE DISTS
        // column width
        let mut col_widths = Vec::with_capacity(num_columns_at);

        for column in columns.iter() {
            // TODO: there is no need to do that if the column is an @-expr, since the width is expected to be the same
            let mut col_width = Unit::ZERO;
            for node in column {
                col_width = Unit::max(col_width, node.width);
            }
            col_widths.push(col_width);
        }
        debug_assert_eq!(col_widths.len(), num_columns_at);


        // baseline_dists[0] is dist from top of first line to first baseline (e.g. as though it was preceded by a line of zero-depth)
        // baseline_dists[i] is the dist from row indexed i and row indexed i+1
        let mut baseline_dists = Vec::with_capacity(num_rows);
        let mut prev_depth = Unit::ZERO;

        for i_row in 0 .. num_rows {
            let mut max_height = strut_height;
            let mut max_depth  = strut_depth;

            for column in columns.iter() {
                let cell = &column[i_row];

                max_height = Unit::max(max_height, cell.height);
                max_depth  = Unit::min(max_depth, cell.depth); // depth are negative
            }

            let box_separation = max_height - prev_depth;
            let baseline_dist = 
                if i_row == 0 {
                    box_separation
                }
                else if box_separation + line_skip_limit > baseline_skip && !baseline_dists.is_empty() {
                    box_separation + line_skip
                }
                else {
                    baseline_skip
                }
            ;
            baseline_dists.push(baseline_dist);
            prev_depth = max_depth;
        }
        let last_depth = prev_depth;
        debug_assert_eq!(baseline_dists.len(), num_rows);

        
        // -- CONSTRUCT COLUMNS
        // Each column is a VBox containing (1) the cell's content, (2) vertical space needed to align rows
        // We need to center cells on the horizontal axis (left, center or right)
        // We need to add vertical space between successive cells so that the baselines of all cells are aligned
        let mut column_vboxes = Vec::with_capacity(num_columns_at);
        for (i_col, column) in columns.into_iter().enumerate() {
            let mut vbox = builders::VBox::new();
            let col_width = col_widths[i_col];
            let alignment = alignments[i_col];


            for (i_row, mut cell) in column.into_iter().enumerate() {
                // add vertical space if necessary to align cells of the same row
                let kern = baseline_dists[i_row] - cell.height;
                if kern > Unit::ZERO {
                    vbox.add_node(LayoutNode::vert_kern(kern));
                }

                cell.alignment = match alignment {
                    Some(ArrayColumnAlign::Centered) => Alignment::Centered(cell.width),
                    Some(ArrayColumnAlign::Left)     => Alignment::Left,
                    Some(ArrayColumnAlign::Right)    => Alignment::Right(cell.width),
                    None => Alignment::Default,
                };
                cell.width = col_width;
                vbox.add_node(cell.as_node());

            }

            // add final space to align bottom of boxes
            let kern = - last_depth;
            if kern > Unit::ZERO {
                vbox.add_node(LayoutNode::vert_kern(kern));
            }
            column_vboxes.push(vbox);
        }
        debug_assert_eq!(column_vboxes.len(), num_columns_at);



        // -- CONSTRUCT ARRAY
        // Now columns have been constructed, we lay them out together
        // We insert space between columns and vertical bars between them

        // the body of the matrix is an hbox of column vectors.
        let mut hbox = builders::HBox::new();

        // If there are no delimiters, insert a null space.  Otherwise we insert
        // the delimiters _after_ we have laidout the body of the matrix.
        if array.left_delimiter.is_none() {
            hbox.add_node(LayoutNode::horiz_kern(null_delimiter_space));
        }


        let total_height : Unit<Px> = 
            baseline_dists.iter().cloned().sum::<Unit<Px>>()
            - last_depth
        ;

        let rule_measurements = RuleMeasurements {
            rule_width,
            total_height,
            double_rule_sep,
        };

        // add left vertical bars
        draw_vertical_bars(&mut hbox, n_vertical_bars[0], rule_measurements);


        for (i_col, vbox) in column_vboxes.into_iter().enumerate() {
            // insert half col sep before if:
            //   - vbox is not an @-expression
            //   - if this is the first col, there is no left delimiter
            //   - if not, preceding col is not @-expr
            if alignments[i_col].is_some() && i_col.checked_sub(1).map_or(array.left_delimiter.is_none(), |prec_i| alignments[prec_i].is_some()) {
                hbox.add_node(LayoutNode::horiz_kern(half_col_sep));
            }

            // insert column
            hbox.add_node(vbox.build());

            // insert half col sep before if:
            //   - vbox is not an @-expression
            //   - if this is the last col, there is no right delimiter
            if alignments[i_col].is_some() && alignments.get(i_col + 1).map_or(array.right_delimiter.is_none(), Option::is_some) {
                hbox.add_node(LayoutNode::horiz_kern(half_col_sep));
            }

            // draw vertical bars after
            draw_vertical_bars(&mut hbox, n_vertical_bars[i_col + 1], rule_measurements);
        }

        if array.right_delimiter.is_none() {
            hbox.add_node(LayoutNode::horiz_kern(null_delimiter_space));
        }


        // TODO: Reference array vertical alignment (optional [bt] arguments)
        // Vertically center the array on axis.
        // Note: hbox has no depth, so hbox.height is total height.
        let height = hbox.height;
        let mut vbox = builders::VBox::new();
        let offset = height.scale(0.5) - config.ctx.constants.axis_height.scaled(config);
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
        let clearance = Unit::max(height.scale(config.ctx.constants.delimiter_factor),
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


#[derive(Debug, Clone, Copy)]
struct RuleMeasurements {
    rule_width:      Unit<Px>, 
    total_height:    Unit<Px>, 
    double_rule_sep: Unit<Px>
}

#[inline]
fn draw_vertical_bars<F>(hbox: &mut builders::HBox<F>, n_bars: u8, rule_measurements: RuleMeasurements) {
    if n_bars != 0 {
        let RuleMeasurements { rule_width, total_height, double_rule_sep } = rule_measurements;
        let rule_width = rule_width;
        let total_height = total_height;
        let double_rule_sep = double_rule_sep;
        hbox.add_node(rule![width: rule_width, height: total_height]);
        for _ in 0 .. n_bars - 1 {
            hbox.add_node(kern![horz: double_rule_sep]);
            hbox.add_node(rule![width: rule_width, height: total_height]);
        }
    }
}



fn extend_delimiter<'a, 'f, F : MathFont>(
    symbol : Symbol, 
    height_content: Unit<Px>,
    depth_content:  Unit<Px>, 
    config: LayoutSettings<'a, 'f, F>
) -> Result<LayoutNode<'f, F>, LayoutError> {
    let min_height = config.ctx.constants.delimited_sub_formula_min_height * config.font_size;
    let null_delimiter_space = config.ctx.constants.null_delimiter_space * config.font_size;

    if symbol.codepoint == '.' {
        return Ok(kern!(horz: null_delimiter_space));
    }


    // Only extend if we meet a certain size
    // TODO: This quick height check doesn't seem to be strong enough,
    // reference: http://tug.org/pipermail/luatex/2010-July/001745.html
    if Unit::max(height_content, -depth_content) > min_height.scale(0.5) {
        let axis = config.ctx.constants.axis_height * config.font_size;

        let inner_size = Unit::max(height_content - axis, axis - depth_content).scale(2.0);
        let clearance_px  = Unit::max(
            inner_size.scale(config.ctx.constants.delimiter_factor),
            height_content - depth_content - config.ctx.constants.delimiter_short_fall * config.font_size
        );
        let clearance = config.to_font(clearance_px);

        Ok(
            config.ctx
            .vert_variant(symbol.codepoint, clearance)?
            .as_layout(config)?
            .centered(axis)
        )
    }
    else {
        config.ctx
            .glyph(symbol.codepoint)?
            .as_layout(config)
    }
}