//! This module defines functions that gives the most esthetically pleasing spacing between two types of symbols.
//! Functions from this module for instance decide that "f" is followed by less space in "f(" than in "f +".
use crate::font::{AtomType};
use crate::layout::Style;
use crate::dimensions::{Length, Em};

/// Given the type of two subsequent atoms and the current style, 
/// determines how much spacing should occur between the two
/// symbols.
pub fn atom_space(left: AtomType, right: AtomType, style: Style) -> Spacing {
    if style >= Style::TextCramped {
        match (left, right) {
            (AtomType::Alpha,       AtomType::Operator(_)) => Spacing::Thin,
            (AtomType::Alpha,       AtomType::Binary)      => Spacing::Medium,
            (AtomType::Alpha,       AtomType::Relation)    => Spacing::Thick,
            (AtomType::Alpha,       AtomType::Inner)       => Spacing::Thin,
            (AtomType::Ordinary,    AtomType::Operator(_)) => Spacing::Thin,
            (AtomType::Ordinary,    AtomType::Binary)      => Spacing::Medium,
            (AtomType::Ordinary,    AtomType::Relation)    => Spacing::Thick,
            (AtomType::Ordinary,    AtomType::Inner)       => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Alpha)       => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Ordinary)    => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Operator(_)) => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Relation)    => Spacing::Thick,
            (AtomType::Operator(_), AtomType::Inner)       => Spacing::Thin,
            (AtomType::Binary,      AtomType::Alpha)       => Spacing::Medium,
            (AtomType::Binary,      AtomType::Ordinary)    => Spacing::Medium,
            (AtomType::Binary,      AtomType::Operator(_)) => Spacing::Medium,
            (AtomType::Binary,      AtomType::Open)        => Spacing::Medium,
            (AtomType::Binary,      AtomType::Inner)       => Spacing::Medium,
            (AtomType::Relation,    AtomType::Alpha)       => Spacing::Thick,
            (AtomType::Relation,    AtomType::Ordinary)    => Spacing::Thick,
            (AtomType::Relation,    AtomType::Operator(_)) => Spacing::Thick,
            (AtomType::Relation,    AtomType::Open)        => Spacing::Thick,
            (AtomType::Relation,    AtomType::Inner)       => Spacing::Thick,
            (AtomType::Close,       AtomType::Operator(_)) => Spacing::Thin,
            (AtomType::Close,       AtomType::Binary)      => Spacing::Medium,
            (AtomType::Close,       AtomType::Relation)    => Spacing::Thick,
            (AtomType::Close,       AtomType::Inner)       => Spacing::Thin,

            // Here it is better to list everything but Spacing::Thin
            (AtomType::Inner, AtomType::Binary)   => Spacing::Medium,
            (AtomType::Inner, AtomType::Relation) => Spacing::Thick,
            (AtomType::Inner, AtomType::Close)    => Spacing::None,
            (AtomType::Inner, _)                  => Spacing::Thin,

            // Every valid (punct, _) pair is undefined or Thin
            (AtomType::Punctuation, _) => Spacing::Thin,
            _ => Spacing::None,
        }
    } else {
        match (left, right) {
            (AtomType::Alpha, AtomType::Operator(_))       => Spacing::Thin,
            (AtomType::Ordinary, AtomType::Operator(_))    => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Alpha)       => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Ordinary)    => Spacing::Thin,
            (AtomType::Operator(_), AtomType::Operator(_)) => Spacing::Thin,
            (AtomType::Close, AtomType::Operator(_))       => Spacing::Thin,
            (AtomType::Inner, AtomType::Operator(_))       => Spacing::Thin,
            _ => Spacing::None,
        }
    }
}

/// Different types of space
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// no space
    None,
    /// thin space
    Thin,
    /// medium space
    Medium,
    /// thick space
    Thick,
}

impl Spacing {
    /// Returns how much a given type of spaces measure in *em* units
    pub fn to_length(self) -> Length<Em> {
        match self {
            Spacing::None   => Length::<Em>::new(0.0),
            Spacing::Thin   => Length::<Em>::new(1. / 6.),
            Spacing::Medium => Length::<Em>::new(2. / 9.),
            Spacing::Thick  => Length::<Em>::new(1. / 3.),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tex_compliant_spacing() {
        /*
        Table extracted from Knuth's TeX book (chap 18, p. 170)

        | Right Atom | Ord | Op | Bin | Rel | Open | Close | Punct | Inner |
        |------------|-----|----|-----|-----|------|-------|-------|-------|
        | Ord        | 0   | 1  | (2) | (3) | 0    | 0     | 0     | (1)   |
        | Op         | 1   | 1  | *   | (3) | 0    | 0     | 0     | (1)   |
        | Bin        | (2) | (2)| *   | *   | (2)  | *     | *     | (2)   |
        | Rel        | (3) | (3)| *   | 0   | (3)  | 0     | 0     | (3)   |
        | Open       | 0   | 0  | *   | 0   | 0    | 0     | 0     | 0     |
        | Close      | 0   | 1  | (2) | (3) | 0    | 0     | 0     | (1)   |
        | Punct      | (1) | (1)| *   | (1) | (1)  | (1)   | (1)   | (1)   |
        | Inner      | (1) | 1  | (2) | (3) | (1)  | 0     | (1)   | (1)   |

        With the accompanying legend:

        " Here 0, 1, 2, and 3 stand for no space, thin space, medium space, and thick space,
        respectively; the table entry is parenthesized if the space is to be inserted only in
        display and text styles, not in script and scriptscript styles. For example, many of the
        entries in the Rel row and the Rel column are ‘(3)’; this means that thick spaces are
        normally inserted before and after relational symbols like ‘=’, but not in subscripts.
        Some of the entries in the table are ‘*’; such cases never arise, because Bin atoms must
        be preceded and followed by atoms compatible with the nature of binary operations. "
        */


        let tex_truth = [
            [Some((Spacing::None, false))  , Some((Spacing::Thin, false))  , Some((Spacing::Medium, true)) , Some((Spacing::Thick, true))  , Some((Spacing::None, false))  , Some((Spacing::None, false)) , Some((Spacing::None, false)) , Some((Spacing::Thin, true))   ,],
            [Some((Spacing::Thin, false))  , Some((Spacing::Thin, false))  , None                          , Some((Spacing::Thick, true))  , Some((Spacing::None, false))  , Some((Spacing::None, false)) , Some((Spacing::None, false)) , Some((Spacing::Thin, true))   ,],
            [Some((Spacing::Medium, true)) , Some((Spacing::Medium, true)) , None                          , None                          , Some((Spacing::Medium, true)) , None                         , None                         , Some((Spacing::Medium, true)) ,],
            [Some((Spacing::Thick, true))  , Some((Spacing::Thick, true))  , None                          , Some((Spacing::None, false))  , Some((Spacing::Thick, true))  , Some((Spacing::None, false)) , Some((Spacing::None, false)) , Some((Spacing::Thick, true))  ,],
            [Some((Spacing::None, false))  , Some((Spacing::None, false))  , None                          , Some((Spacing::None, false))  , Some((Spacing::None, false))  , Some((Spacing::None, false)) , Some((Spacing::None, false)) , Some((Spacing::None, false))  ,],
            [Some((Spacing::None, false))  , Some((Spacing::Thin, false))  , Some((Spacing::Medium, true)) , Some((Spacing::Thick, true))  , Some((Spacing::None, false))  , Some((Spacing::None, false)) , Some((Spacing::None, false)) , Some((Spacing::Thin, true))   ,],
            [Some((Spacing::Thin, true))   , Some((Spacing::Thin, true))   , None                          , Some((Spacing::Thin, true))   , Some((Spacing::Thin, true))   , Some((Spacing::Thin, true))  , Some((Spacing::Thin, true))  , Some((Spacing::Thin, true))   ,],
            [Some((Spacing::Thin, true))   , Some((Spacing::Thin, false))  , Some((Spacing::Medium, true)) , Some((Spacing::Thick, true))  , Some((Spacing::Thin, true))   , Some((Spacing::None, false)) , Some((Spacing::Thin, true))  , Some((Spacing::Thin, true))   ,],        
        ];

        let atoms  = [AtomType::Ordinary, AtomType::Operator(false), AtomType::Binary, AtomType::Relation, AtomType::Open, AtomType::Close, AtomType::Punctuation, AtomType::Inner,];
        let styles = [Style::ScriptScriptCramped, Style::ScriptScript, Style::ScriptCramped, Style::Script, Style::TextCramped, Style::Text, Style::DisplayCramped, Style::Display,];

        for (i, left_atom) in atoms.iter().cloned().enumerate() {
            for (j, right_atom) in atoms.iter().cloned().enumerate() {
                eprintln!("{:?} before {:?}", left_atom, right_atom);
                if let Some((expected_space, no_space_when_cramped)) = tex_truth[i][j] {
                    for style in styles {
                        let space = atom_space(left_atom, right_atom, style);
                        if style < Style::TextCramped && no_space_when_cramped {
                            assert_eq!(space, Spacing::None);
                        }
                        else {
                            assert_eq!(space, expected_space);
                        }
                    }

                }
            }
        }

    }
}