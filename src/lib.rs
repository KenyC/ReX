//! # A mathematical typesetting engine based on LuaTeX and XeTeX.
//! 
//! This is a fork of [ReX](https://github.com/ReTeX/ReX) incorporating modifications by s3bk.
//! 
//! ## General overview
//! 
//! Rendering a formula involves the following steps:
//! 
//!   1. Parsing the formula into [`ParseNode`](`crate::parser::ParseNode`)s, cf [`parse`](crate::parser::parse).
//!   2. Placing the [`ParseNode`](`crate::parser::ParseNode`)s in space relative to each other, yielding a [`Layout`](crate::layout::Layout), cf [`layout`](crate::layout::engine::layout). 
//!      This step requires some mathematical OpenType font to provide various spacing parameters and some other info, like desired font size, cf [`LayoutSettings`](crate::layout::LayoutSettings).
//!   3. Finally, we can render the formula on a certain graphical backend (e.g. the screen, a SVG file, etc). [`Renderer`](crate::render::Renderer) is used to that end, and especially the method [Renderer::render](`crate::render::Renderer::render`).
//!      This step requires a certain graphical backend to be set up (e.g. [Cairo](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/cairo/), [femtovg](https://docs.rs/femtovg/latest/femtovg/index.html), [pathfinder](https://github.com/servo/pathfinder)).
//!  
//! The crate gives you freedom to use your favorite font parsing crate, by implementing the [`MathFont`](crate::font::MathFont) trait and your favourite graphical backend, by implementing [`Backend`](crate::render::Backend). 
//! Some features provide implementations for certain font parsing crates and graphical backends.
//!  
//! ## Implementing backends
//!  
//! ### Font parser backend
//! 
//! The [`MathFont`](crate::font::MathFont) trait demands access to certain information from an otf font file, such as access to a certain list of mathematical parameters from the font table, how to construct
//! extended versions of certain glyphs
//! 
//! ### Graphical backend
//! 
//! The [`Backend`](crate::render::Backend) trait consists of two traits: [`FontBackend<F>`](crate::render::FontBackend) and [`GraphicsBackend`](crate::render::GraphicsBackend).
//! The [`FontBackend<F>`](crate::render::FontBackend) consists in the method `symbol` to draw a glyph, given a particular `F` implementing [`MathFont`](crate::font::MathFont).
//! The [`GraphicsBackend`](crate::render::GraphicsBackend) only contains drawing methods that do not require a particular font: drawing boxes, drawing lines, pushing a certain color on the stack, etc.




#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

pub mod environments;
#[deny(missing_docs)]
pub mod error;
#[deny(missing_docs)]
pub mod dimensions;
pub mod layout;
pub mod lexer;
pub mod parser;
pub mod render;

pub mod font;
mod functions;

pub use render::*;
