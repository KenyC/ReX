#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AtomType {
    Punctuation,
    Ordinal,
    Open,
    Close,
    Binary,
    Relation,
    Accent,
    AccentWide,
    AccentOverlay,
    BotAccent,
    BotAccentWide,
    Alpha,
    Fence,
    Operator(bool),
    Over,
    Under,
    Inner,
    Transparent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub codepoint: char,
    pub name: &'static str,
    pub description: &'static str,
    pub atom_type: AtomType,
}

pub static SYMBOLS: &'static [Symbol] = &include!(concat!(env!("OUT_DIR"), "/symbols.rs"));
