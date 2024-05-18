/// An enum representing the desired category of a symbol
/// A symbol's category determines its spacing relative to each other, e.g `1+23` ought to be typeset with some space between + and 2, but very little between 2 and 3.
/// The category also determines whether something can be `\left` or `\right` delimiter
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TexSymbolType {
    Punctuation,
    Ordinary,
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
    /// A mathematical operator like `\sum`, 
    /// the boolean parameter sets whether limits are positioned like exponents or not.
    /// For instance, in $\sum_0^1$, the 0 and 1 could be placed above and below ∑ (boolean true)
    /// or like regular exponents, e.g. ∑³, (boolean false).
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
    pub atom_type: TexSymbolType,
}

/// List of symbols  
/// (GUARANTEE: they're listed 'alphabetically', i.e. by byte order, to allow binary search)
pub const SYMBOLS: &'static [Symbol] = &include!(concat!(env!("OUT_DIR"), "/symbols.rs"));


pub const MATH_ALPHANUMERIC_TABLE_RESERVED_REPLACEMENTS: &[(u32, u32)] = &include!(concat!(env!("OUT_DIR"), "/math_alphanumeric_table_reserved_replacements.rs"));

pub fn is_italic(codepoint : char) -> bool {
    let mut codepoint = u32::from(codepoint);
    for (original, replacement) in MATH_ALPHANUMERIC_TABLE_RESERVED_REPLACEMENTS.iter() {
        if codepoint == *replacement {
            codepoint = *original;
            break;
        }
    }

    match codepoint {
        0x1d434 ..=0x1d503 => true, // From '𝐴' MATHEMATICAL CAPITAL A to '𝔃' MATHEMATICAL BOLD SCRIPT SMALL Z
        0x1d608 ..=0x1d66f => true, // From MATHEMATICAL SANS-SERIF ITALIC CAPITAL A to MATHEMATICAL SANS-SERIF ITALIC CAPITAL Z
        0x1d6e2 ..=0x1d755 => true, // From MATHEMATICAL ITALIC CAPITAL ALPHA to MATHEMATICAL SANS-SERIF ITALIC PI SYMBOL
        0x1d790 ..=0x1d7c9 => true, // From MATHEMATICAL SANS-SERIF BOLD ITALIC CAPITAL ALPHA to MATHEMATICAL SANS-SERIF BOLD ITALIC PI SYMBOL
        _ => false
    }
}