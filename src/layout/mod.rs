//! Converting [`ParseNode`](crate::parser::ParseNode)s to [`Layout`] boxes which are ready to be rendered.
//! 
//! The layout boxes follow a similar model as those found in HTML and TeX in that they both
//! have horizontal and vertical boxes.  One difference will be how glue is handled.  HTML/CSS
//! does not have anything similar to how glue is handled in TeX and so aboslute size will be
//! necessary for these scnarios.  It's unclear if we will be able to induce alignments from
//! glue, such as something being centered, aligned left/right, etc.  These questions may
//! also be useful to answer in SVG.
//!
//! Layout boxes will contain a minimal representation of what will be rendered.
//! This includes the container types: Horizontal/Vertical boxes,
//! and primitive types: Symbols, lines, spacing.
//!
//! While rendering in mathmode, most types require an atomtype to determine the kerning
//! between symbols.  This information must also be present with layout boxes.
//!
//! The units used in layout boxes must be in FontUnit (as defined in CSS).

#[macro_use]
mod builders;
mod convert;
pub mod engine;
pub mod spacing;

use crate::font::common::GlyphId;
use crate::parser::color::RGBA;
use crate::font::FontContext;
use std::ops::Deref;
use std::fmt;
use std::cmp::{max, min};
use std::collections::BTreeMap;
use crate::dimensions::*;

/// Contains a set of [`LayoutNode`s](crate::layout::LayoutNode) that defines the position of glyphs and rules (i.e. filled rectangles) and certain measurements useful for rendering.
/// It serves as input to [`Renderer::render`](crate::render::Renderer::render).
#[derive(Clone, Debug)]
pub struct Layout<'f, F> {
    /// The children nodes contained in the layout
    /// By default, they are laid out, horizontally as a horizontal box.
    pub contents:  Vec<LayoutNode<'f, F>>,
    /// Width of content
    pub width:     Length<Px>,
    /// Height of content ; distance from baseline to the top of the layout
    pub height:    Length<Px>,
    /// Depth of content ; distance from baseline to the bottom of the layout
    pub depth:     Length<Px>,
    /// Offset from the baseline 
    // (NB: does not seem used at the moment)
    pub offset:    Length<Px>,
    /// How to horizontally lay out children nodes
    pub alignment: Alignment,
}

impl<'f, F> Default for Layout<'f, F> {
    fn default() -> Self {
        Self { 
            contents:  Vec::default(),
            width:     Length::default(),
            height:    Length::default(),
            depth:     Length::default(),
            offset:    Length::default(),
            alignment: Alignment::default(),
        }
    }
}

impl<'f, F> Layout<'f, F> {
    /// Make layout into layout node, which can then be inserted in another layout
    pub fn as_node(self) -> LayoutNode<'f, F> {
        LayoutNode {
            width: self.width,
            height: self.height,
            depth: self.depth,
            node: LayoutVariant::HorizontalBox(
                HorizontalBox {
                   contents: self.contents,
                   offset: self.offset,
                   alignment: self.alignment,
                }
            ),
        }
    }

    /// Create new [`Layout`](crate::layout::Layout) ; equivalent to [Default::default]
    pub fn new() -> Layout<'f, F> {
        Layout::default()
    }

    /// Append node at end of layout (i.e. right of layout)
    pub fn add_node(&mut self, node: LayoutNode<'f, F>) {
        self.width += node.width;
        self.height = max(self.height, node.height);
        self.depth = min(self.depth, node.depth);
        self.contents.push(node);
    }

    /// Sets offset of layout
    // TODO: not used at the moment figure out why (perhaps lack of alignement in array?)
    pub fn set_offset(&mut self, offset: Length<Px>) {
        self.offset = offset;
    }

    /// Not clear what this does.
    pub fn finalize(mut self) -> Layout<'f, F> {
        self.depth -= self.offset;
        self.height -= self.offset;
        self
    }

    /// Makes layout's width equal to given arguments, and centers children within that width
    pub fn centered(mut self, new_width: Length<Px>) -> Layout<'f, F> {
        self.alignment = Alignment::Centered(self.width);
        self.width = new_width;
        self
    }
}

impl<'f, F> Layout<'f, F> {

    fn is_symbol(&self) -> Option<LayoutGlyph<'f, F>> {
        if self.contents.len() != 1 {
            return None;
        }
        self.contents[0].is_symbol()
    }
}

/// A sub-part of the layout hierarchy: can contain other nodes and may be contained in other nodes.
pub struct LayoutNode<'f, F> {
    /// Type of node
    pub node: LayoutVariant<'f, F>,
    /// Width
    pub width: Length<Px>,
    /// Height: distance from base line to top of the node
    pub height: Length<Px>,
    /// Height: distance from base line to bottom of the node
    pub depth: Length<Px>,
}

impl<'f, F> Clone for LayoutNode<'f, F> {
    fn clone(&self) -> Self {
        Self {
            node:   self.node.clone(),
            width:  self.width.clone(),
            height: self.height.clone(),
            depth:  self.depth.clone(),
        }
    }
}

/// Different types of layout nodes
pub enum LayoutVariant<'f, F> {
    /// A grid
    Grid(Grid<'f, F>),
    /// A horizontal box
    HorizontalBox(HorizontalBox<'f, F>),
    /// A vertical box
    VerticalBox(VerticalBox<'f, F>),
    /// A symbol (aka glyph) from the font
    Glyph(LayoutGlyph<'f, F>),
    /// A scope within which the main color is changed
    Color(ColorChange<'f, F>),
    /// A filled rectangle
    Rule,
    /// Some (possibly negative) spacing
    Kern,
}

impl<'f, F> Clone for LayoutVariant<'f, F> {
    fn clone(&self) -> Self {
        match self {
            LayoutVariant::Grid(grid)             => LayoutVariant::Grid(grid.clone()),
            LayoutVariant::HorizontalBox(hbox)    => LayoutVariant::HorizontalBox(hbox.clone()),
            LayoutVariant::VerticalBox(vbox)      => LayoutVariant::VerticalBox(vbox.clone()),
            LayoutVariant::Glyph(glyph)           => LayoutVariant::Glyph(glyph.clone()),
            LayoutVariant::Color(color_change)    => LayoutVariant::Color(color_change.clone()),
            LayoutVariant::Rule                   => LayoutVariant::Rule,
            LayoutVariant::Kern                   => LayoutVariant::Kern,
        }
    }
}

/// All children of this node will use the [ColorChange::color] as a fill color
pub struct ColorChange<'f, F> {
    /// Color to use
    pub color: RGBA,
    /// Children of the given node
    pub inner: Vec<LayoutNode<'f, F>>,
}

impl<'f, F> Clone for ColorChange<'f, F> {
    fn clone(&self) -> Self {
        Self { 
            color: self.color, 
            inner: self.inner.clone(),
        }
    }
}

/// Place nodes in a grid-like pattern. 
/// The number of rows and columns is determined automatically
pub struct Grid<'f, F> {
    /// Children nodes and their position in the grid
    pub contents: BTreeMap<(usize, usize), LayoutNode<'f, F>>,
    /// max length of each column
    pub columns: Vec<Length<Px>>,
    /// (max height, max depth) of each row
    pub rows: Vec<(Length<Px>, Length<Px>)>,
}

impl<'f, F> Clone for Grid<'f, F> {
    fn clone(&self) -> Self {
        Self {
            contents: self.contents.clone(),
            columns:  self.columns.clone(),
            rows:     self.rows.clone(),
        }
    }
}

/// A horizontal box : children are placed side by side.
pub struct HorizontalBox<'f, F> {
    /// Children nodes
    pub contents: Vec<LayoutNode<'f, F>>,
    /// Offset
    // Unclear what this does
    pub offset: Length<Px>,
    /// How to align Children nodes
    pub alignment: Alignment,
}

impl<'f, F> Clone for HorizontalBox<'f, F> {
    fn clone(&self) -> Self {
        Self {
            contents:  self.contents.clone(),
            offset:    self.offset.clone(),
            alignment: self.alignment.clone(),
        }
    }
}


// NOTE: A limitation on derive(Clone, Default) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/
impl<'f, F> Default for HorizontalBox<'f, F> {
    fn default() -> Self {
        Self { 
            contents:  Vec::default(), 
            offset:    Length::default(), 
            alignment: Alignment::default(), 
        }
    }
}

/// Vertical box: children are placed on top of each other
pub struct VerticalBox<'f, F> {
    /// Children nodes
    pub contents: Vec<LayoutNode<'f, F>>,
    /// Offset from baseline
    pub offset: Length<Px>,
    /// Horizontal alignment
    pub alignment: Alignment,
}

impl<'f, F> Clone for VerticalBox<'f, F> {
    fn clone(&self) -> Self {
        Self {
            contents:  self.contents.clone(),
            offset:    self.offset.clone(),
            alignment: self.alignment.clone(),
        }
    }
}

// NOTE: A limitation on derive(Clone, Default) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/
impl<'f, F> Default for VerticalBox<'f, F> {
    fn default() -> Self {
        Self { 
            contents:  Vec::default(), 
            offset:    Length::default(), 
            alignment: Alignment::default(), 
        }
    }
}


/// Glyph : this node has no children ; simply specify some glyph (i.e. symbol) to draw
pub struct LayoutGlyph<'f, F> {
    /// glyph id
    pub gid: GlyphId,
    /// width of the symbol
    pub size: Length<Px>,
    /// offset from baseline
    pub offset: Length<Px>,
    /// where to place accents
    pub attachment: Length<Px>,
    /// italic correction: italic symbols, who lean, may come out of the bounding box towards the top ; when the next glyph is not as slanted (i.e. is not italic), the italic glyph may collide with next glyph.
    /// The "italic correction" tells one how much wider the glyph needs to be in order to avoid any collisions with subsequent glyphs.
    pub italics: Length<Px>,
    /// font to render glyph with
    pub font: &'f F,
}


impl<'f, F> Clone for LayoutGlyph<'f, F> {
    fn clone(&self) -> Self {
        Self {
            gid:        self.gid,
            size:       self.size,
            offset:     self.offset,
            attachment: self.attachment,
            italics:    self.italics,
            font:       self.font,
        }
    }
}
impl<'f, F> Copy for LayoutGlyph<'f, F> {}

// NOTE: A limitation on derive(Clone) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/


#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq)]
/// How to horizontally align certain elements
pub enum Alignment {
    /// Centered within the argument width
    Centered(Length<Px>),
    /// Right-aligned within the argument width
    Right(Length<Px>),
    /// Placed left to right, one after the other ; width is determined automatically
    Left,
    /// inherit from previous
    Inherit,
    /// default
    Default,
}

impl Default for Alignment {
    fn default() -> Alignment {
        Alignment::Default
    }
}

impl<'f, F> Deref for HorizontalBox<'f, F> {
    type Target = [LayoutNode<'f, F>];
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<'f, F> Deref for VerticalBox<'f, F> {
    type Target = [LayoutNode<'f, F>];
    fn deref(&self) -> &Self::Target {
        &self.contents
    }
}

impl<'f, F> fmt::Debug for VerticalBox<'f, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.offset.is_zero() {
            write!(f, "VerticalBox({:?})", self.contents)
        } else {
            write!(f,
                   "VerticalBox({:?}, offset: {})",
                   self.contents,
                   self.offset)
        }
    }
}

impl<'f, F> fmt::Debug for HorizontalBox<'f, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HorizontalBox({:?})", self.contents)
    }
}

impl<'f, F> fmt::Debug for LayoutGlyph<'f, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LayoutGlyph({})", Into::<u16>::into(self.gid))
    }
}

impl<'f, F> fmt::Debug for LayoutNode<'f, F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.node {
            LayoutVariant::Grid(ref _grid) =>  write!(f, "Grid(..)"),
            LayoutVariant::HorizontalBox(ref hb) => write!(f, "HBox({:?})", hb.contents),
            LayoutVariant::VerticalBox(ref vb) => write!(f, "VBox({:?})", vb.contents),
            LayoutVariant::Glyph(ref gly) => write!(f, "Glyph({:?})", gly),
            LayoutVariant::Rule => write!(f, "Rule()"),
            LayoutVariant::Kern => {
                let kern = if self.width.is_zero() {
                    self.height
                } else {
                    self.width
                };

                write!(f, "Kern({:.1})", kern)
            }
            LayoutVariant::Color(ref clr) => write!(f, "Color({:?}, {:?})", clr.color, clr.inner),
        }
    }
}


impl<'f, F> LayoutNode<'f, F> {
    
    /// Center the vertical about the axis.
    /// For now this ignores offsets if already applied,
    /// and will break if there already are offsets.
    fn centered(mut self, axis: Length<Px>) -> LayoutNode<'f, F> {
        let shift = (self.height + self.depth) * 0.5 - axis;

        match self.node {
            LayoutVariant::VerticalBox(ref mut vb) => {
                vb.offset = shift;
                self.height -= shift;
                self.depth -= shift;
            }

            LayoutVariant::Glyph(_) => return vbox!(offset: shift; self),

            _ => (),
        }

        self
    }

    fn is_symbol(&self) -> Option<LayoutGlyph<'f, F>> {
        match self.node {
            LayoutVariant::Glyph(gly) => Some(gly),
            LayoutVariant::HorizontalBox(ref hb) => is_symbol(&hb.contents),
            LayoutVariant::VerticalBox(ref vb) => is_symbol(&vb.contents),
            LayoutVariant::Color(ref clr) => is_symbol(&clr.inner),
            _ => None,
        }
    }
}

/// Determines if a set of nodes is a singleton set containing a symbol node
pub fn is_symbol<'a, 'b: 'a, F>(contents: &'a [LayoutNode<'b, F>]) -> Option<LayoutGlyph<'b, F>> {
    if contents.len() != 1 {
        return None;
    }

    contents[0].is_symbol()
}

/// Display styles which are used in scaling glyphs.  The associated
/// methods are taken from pg.441 from the TeXBook
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Style {
    /// sub-script of sub-script with no spacing
    ScriptScriptCramped,
    /// sub-script of sub-script
    ScriptScript,
    /// sub-script with no sapce
    ScriptCramped,
    /// sub-script 
    Script,
    /// TODO
    TextCramped,
    /// TODO
    Text,
    /// TODO
    DisplayCramped,
    /// TODO
    Display,
}

impl Default for Style {
    fn default() -> Style {
        Style::Display
    }
}

#[allow(dead_code)]
impl Style {
    fn cramped(self) -> Style {
        match self {
            Style::ScriptScriptCramped |
            Style::ScriptScript => Style::ScriptScriptCramped,
            Style::ScriptCramped | Style::Script => Style::ScriptCramped,
            Style::TextCramped | Style::Text => Style::TextCramped,
            Style::DisplayCramped | Style::Display => Style::DisplayCramped,
        }
    }

    fn superscript_variant(self) -> Style {
        match self {
            Style::Display | Style::Text => Style::Script,
            Style::DisplayCramped | Style::TextCramped => Style::ScriptCramped,
            Style::Script | Style::ScriptScript => Style::ScriptScript,
            Style::ScriptCramped |
            Style::ScriptScriptCramped => Style::ScriptScriptCramped,
        }
    }

    fn subscript_variant(self) -> Style {
        match self {
            Style::Display | Style::Text | Style::DisplayCramped | Style::TextCramped => {
                Style::ScriptCramped
            }
            Style::Script |
            Style::ScriptScript |
            Style::ScriptCramped |
            Style::ScriptScriptCramped => Style::ScriptScriptCramped,
        }
    }

    fn sup_shift_up<F>(self, config: LayoutSettings<F>) -> Length<Em> {
        match self {
            Style::Display | Style::Text | Style::Script | Style::ScriptScript => {
                config.ctx.constants.superscript_shift_up
            }
            _ => config.ctx.constants.superscript_shift_up_cramped
        }
    }

    fn is_cramped(&self) -> bool {
        match *self {
            Style::Display | Style::Text | Style::Script | Style::ScriptScript => false,
            _ => true,
        }
    }

    fn numerator(self) -> Style {
        match self {
            Style::Display => Style::Text,
            Style::DisplayCramped => Style::TextCramped,
            _ => self.superscript_variant(),
        }
    }

    fn denominator(self) -> Style {
        match self {
            Style::Display | Style::DisplayCramped => Style::TextCramped,
            _ => self.subscript_variant(),
        }
    }
}


// NOTE: A limitation on derive(Clone) forces us to implement clone ourselves.
// cf discussion here: https://stegosaurusdormant.com/understanding-derive-clone/
/// Defines the math font to use, the desired font size and whether to use Roman or Italic or various other scripts
pub struct LayoutSettings<'a, 'f, F> {
    /// Maths font
    pub ctx: &'a FontContext<'f, F>,
    /// Font size in (in pixels per em)
    pub font_size: Scale<Px, Em>,
    /// TODO
    pub style: Style,
}


impl<'a, 'f, F> Clone for LayoutSettings<'a, 'f, F> {
    fn clone(&self) -> Self {
        Self {
            ctx :       self.ctx,
            font_size : self.font_size,
            style :     self.style.clone(),
        }
    }
}
impl<'a, 'f, F> Copy for LayoutSettings<'a, 'f, F> {}




impl<'a, 'f, F> LayoutSettings<'a, 'f, F> {
    /// Creates a new LayoutSettings
    pub fn new(ctx: &'a FontContext<'f, F>, font_size: f64, style: Style) -> Self {
        LayoutSettings {
            ctx,
            font_size: Scale::new(font_size, Px, Em),
            style,
        }
    }

    fn cramped(self) -> Self {
        LayoutSettings {
            style: self.style.cramped(),
            ..self
        }
    }

    fn superscript_variant(self) -> Self {
        LayoutSettings {
            style: self.style.superscript_variant(),
            ..self
        }
    }

    fn subscript_variant(self) -> Self {
        LayoutSettings {
            style: self.style.subscript_variant(),
            ..self
        }
    }

    fn numerator(self) -> Self {
        LayoutSettings {
            style: self.style.numerator(),
            ..self
        }
    }

    fn denominator(self) -> Self {
        LayoutSettings {
            style: self.style.denominator(),
            ..self
        }
    }

    fn with_display(self) -> Self {
        LayoutSettings {
            style: Style::Display,
            ..self
        }
    }

    fn with_text(self) -> Self {
        LayoutSettings {
            style: Style::Text,
            ..self
        }
    }
}