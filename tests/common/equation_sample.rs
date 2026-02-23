use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::debug_render::DebugRender;


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
    /// Equation diff between history test results and current test results.
    /// The first member of the pair is the render from history.
    /// The second member of the pair is the new render.
    pub diffs   : Vec<(&'a Equation, &'a Equation)>,
    /// Current test results with no correspondent in history
    pub new_eqs : Vec<&'a Equation>,
}

impl<'a> EquationDiffs<'a> {
    pub fn no_diff(&self) -> bool {
        self.diffs.is_empty() && self.new_eqs.is_empty()
    }
}
