// extern crate rex;
// extern crate font_types as font;
// #[macro_use]
// extern crate serde_derive;

use std::path::{Path, PathBuf};

use rex::{GraphicsBackend, FontBackend, Backend};



#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub tex:              String,
    pub description:      String,
    pub render:           Result<EquationRender, String>
}
impl Equation {
    pub fn same_render_as(&self, other: &Equation) -> bool {
        match (&self.render, &other.render) {
            (Ok(a), Ok(b)) => a.same_as(b),
            _ => false
        }
    }
}
impl EquationRender {
    pub fn same_as(&self, other : &Self) -> bool {
        self.width  == other.width  &&
        self.height == other.height &&
        self.render == other.render 
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct EquationRender {
    pub width:            f64,
    pub height:           f64,
    pub render:           DebugRender,
    pub img_render_path:  Option<PathBuf>,
}

#[derive(Debug,)]
pub struct EquationDiffs<'a> {
    /// Equation diff between history test results and current test results
    pub diffs   : Vec<(&'a Equation, &'a Equation)>,
    /// Current test results with no correspondent in history
    pub new_eqs : Vec<&'a Equation>,
}

impl<'a> EquationDiffs<'a> {
    pub fn no_diff(&self) -> bool {
        self.diffs.is_empty() && self.new_eqs.is_empty()
    }
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, Default)]
pub struct DebugRender {
    commands : Vec<DrawCmd>,
}

impl PartialEq for DebugRender {
    fn eq(&self, other: &Self) -> bool {
        for command in self.commands.iter() {
            if !other.commands.contains(command) {
                return false;
            }
        }
        for command in other.commands.iter() {
            if !self.commands.contains(command) {
                return false;
            }
        }
        return true;
    }
}


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub enum DrawCmd {
    Rule {
        pos    : (f64, f64),
        width  : f64,
        height : f64,
    },
    Symbol {
        pos      : (f64, f64),
        glyph_id : u16,
        scale    : f64,  
    }
}

impl<A> Backend<A> for DebugRender {}

impl GraphicsBackend for DebugRender {
    fn rule(&mut self, pos: rex::Cursor, width: f64, height: f64) {
        self.commands.push(DrawCmd::Rule { 
            pos: (pos.x, pos.y), 
            width, height, 
        })
    }

    fn begin_color(&mut self, _color: rex::RGBA) {
    }

    fn end_color(&mut self) {
    }
}

impl<A> FontBackend<A> for DebugRender {
    fn symbol(&mut self, pos: rex::Cursor, gid: rex::font::common::GlyphId, scale: f64, _ctx: &A) {
        self.commands.push(DrawCmd::Symbol { 
            pos: (pos.x, pos.y), 
            glyph_id: gid.into(), 
            scale, 
        });
    }
}