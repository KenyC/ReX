use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use super::debug_render::DebugRender;


#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub tex:              String,
    pub description:      String,
    pub width:            f64,
    pub height:           f64,
    pub render:           DebugRender,
    pub img_render_path:  Option<PathBuf>,
}
impl Equation {
    pub fn same_as(&self, other : &Self) -> bool {
        self.width  == other.width  &&
        self.height == other.height &&
        self.render == other.render 
    }
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
