use crate::error::{LayoutError, Error};
pub use crate::font::MathFont;
use crate::dimensions::*;
use crate::layout::{LayoutNode, LayoutVariant, Alignment, Style, LayoutSettings, Layout, Grid};
pub use crate::parser::{color::RGBA};

pub struct Renderer {
    pub debug: bool,
}

#[derive(Copy, Clone, Default)]
pub struct Cursor {
    pub x: f64,
    pub y: f64,
}

impl Cursor {
    pub fn translate(self, dx: f64, dy: f64) -> Cursor {
        Cursor {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
    pub fn left(self, dx: f64) -> Cursor {
        Cursor {
            x: self.x - dx,
            y: self.y,
        }
    }
    pub fn right(self, dx: f64) -> Cursor {
        Cursor {
            x: self.x + dx,
            y: self.y,
        }
    }
    pub fn up(self, dy: f64) -> Cursor {
        Cursor {
            x: self.x,
            y: self.y - dy,
        }
    }
    pub fn down(self, dy: f64) -> Cursor {
        Cursor {
            x: self.x,
            y: self.y + dy,
        }
    }
}

pub trait Backend {
    fn bbox(&mut self, _pos: Cursor, _width: f64, _height: f64, role: Role) {}
    fn symbol(&mut self, pos: Cursor, gid: u16, scale: f64, ctx: &MathFont);
    fn rule(&mut self, pos: Cursor, width: f64, height: f64);
    fn begin_color(&mut self, color: RGBA);
    fn end_color(&mut self);
}

pub enum Role {
    Glyph,
    VBox,
    HBox,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            debug: false,
        }
    }
    pub fn layout<'s, 'a, 'f>(&self, tex: &'s str, layout_settings: LayoutSettings<'a, 'f>) -> Result<Layout<'f>, Error<'s>> {
        use crate::parser::parse;
        use crate::layout::engine::layout;

        let mut parse = parse(tex)?;
        Ok(layout(&mut parse, layout_settings)?)
    }
    // (x0, y0, x1, y1)
    pub fn size(&self, layout: &Layout) -> (f64, f64, f64, f64) {
        (
            0.0,
            layout.depth / Px,
            layout.width / Px,
            layout.height / Px
        )
    }
    pub fn render(&self, layout: &Layout, out: &mut impl Backend) {
        let pos = Cursor {
            x: 0.0,
            y: 0.0,
        };
        self.render_hbox(out, pos, &layout.contents, layout.height / Px, layout.width / Px, Alignment::Default);
    }

    fn render_grid(&self, out: &mut impl Backend, pos: Cursor, width: f64, height: f64, grid: &Grid) {
        let x_offsets = grid.x_offsets();
        let y_offsets = grid.y_offsets();
        for (&(row, column), node) in grid.contents.iter() {
            let width = grid.columns[column];
            let (height, depth) = grid.rows[row];

            self.render_node(
                out,
                pos.translate(x_offsets[column] / Px, (y_offsets[row] + height) / Px),
                node
            );
        }
    }

    fn render_hbox(&self, out: &mut impl Backend, mut pos: Cursor, nodes: &[LayoutNode], height: f64, nodes_width: f64, alignment: Alignment) {
        if self.debug {
            out.bbox(pos.up(height), nodes_width, height, Role::HBox);
        }
        if let Alignment::Centered(w) = alignment {
            pos.x += (nodes_width - w / Px) * 0.5;
        }

        for node in nodes {
            self.render_node(out, pos, node);

            pos.x += node.width / Px;
        }
    }
    fn render_vbox(&self, out: &mut impl Backend, mut pos: Cursor, nodes: &[LayoutNode]) {
        for node in nodes {
            match node.node {
                LayoutVariant::Rule => out.rule(pos, node.width / Px, node.height / Px),
                LayoutVariant::Grid(ref grid) => self.render_grid(out, pos, node.height / Px, node.width / Px, grid),
                LayoutVariant::HorizontalBox(ref hbox) => {
                    self.render_hbox(out,
                                     pos.down(node.height / Px),
                                     &hbox.contents,
                                     node.height / Px,
                                     node.width / Px,
                                     hbox.alignment)
                }

                LayoutVariant::VerticalBox(ref vbox) => {
                    if self.debug {
                        out.bbox(pos, node.width / Px, (node.height - node.depth) / Px, Role::VBox);
                    }
                    self.render_vbox(out, pos, &vbox.contents);
                }

                LayoutVariant::Glyph(ref gly) => {
                    if self.debug {
                        out.bbox(pos, node.width / Px, (node.height - node.depth) / Px, Role::Glyph);
                    }
                    out.symbol(pos.down(node.height / Px), gly.gid, gly.size / Px, gly.font);
                }

                LayoutVariant::Color(_) => panic!("Shouldn't have a color in a vertical box???"),

                LayoutVariant::Kern => { /* NOOP */ }
            }

            pos.y += node.height / Px;
        }
    }

    fn render_node<'a>(&self, out: &mut impl Backend, pos: Cursor, node: &LayoutNode<'a>) {
        match node.node {
            LayoutVariant::Glyph(ref gly) => {
                if self.debug {
                    out.bbox(pos.up(node.height / Px), node.width / Px, (node.height - node.depth) / Px, Role::Glyph);
                }
                out.symbol(pos, gly.gid, gly.size / Px, gly.font);
            }

            LayoutVariant::Rule => out.rule(pos.up(node.height / Px), node.width / Px, node.height / Px),

            LayoutVariant::VerticalBox(ref vbox) => {
                if self.debug {
                    out.bbox(pos.up(node.height / Px), node.width / Px, (node.height - node.depth) / Px, Role::VBox);
                }
                self.render_vbox(out, pos.up(node.height / Px), &vbox.contents);
            }

            LayoutVariant::HorizontalBox(ref hbox) => {
                self.render_hbox(out, pos, &hbox.contents, node.height / Px, node.width / Px, hbox.alignment);
            }
            LayoutVariant::Grid(ref grid) => self.render_grid(out, pos, node.height / Px, node.width / Px, grid),

            LayoutVariant::Color(ref clr) => {
                out.begin_color(clr.color);
                self.render_hbox(out, pos, &clr.inner, node.height / Px, node.width / Px, Alignment::Default);
                out.end_color();
            }

            LayoutVariant::Kern => { /* NOOP */ }
        } // End macth

    }
}

#[cfg(feature="pathfinder-backend")]
pub mod scene;
#[cfg(feature="pathfinder-backend")]
pub use scene::SceneWrapper;

#[cfg(feature="femtovg-backend")]
pub mod femtovg;
