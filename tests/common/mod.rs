#![allow(dead_code)]
pub mod equation_sample;
pub mod debug_render;
pub mod report;
pub mod img_diff;
pub mod utils;

use std::path::Path;
use crate::common::equation_sample::Equation;


pub fn load_equations<P: AsRef<Path>>(path: P) -> Vec<Equation> {
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(path.as_ref()).expect("failed to open test collection");
    let mut reader = BufReader::new(file);
    let tests: Vec<Equation> = serde_yaml::from_reader(&mut reader)
        .expect("failed to load historical test results");

    tests
}

pub fn find_different_render(old: &[Equation], new: &[Equation]) -> Result<Vec<(Equation, Equation)>, ()> {
    if old.len() != new.len() {
        return Err(());
    }

    let mut different_renders: Vec<(Equation, Equation)> = Vec::new();
    for (left, right) in Iterator::zip(old.iter(), new.iter()) {
        if left != right {
            different_renders.push((left.clone(), right.clone()));
        }
    }

    Ok(different_renders)
}




