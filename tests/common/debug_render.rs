// extern crate rex;
// extern crate font_types as font;
// #[macro_use]
// extern crate serde_derive;


use rex::{GraphicsBackend, FontBackend, Backend};



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