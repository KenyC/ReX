//! Mathematical symbols from TeX commands.

use unicode_math::{SYMBOLS, AtomType};

/// A LateX symbol is simply a Unicode symbol and a certain category
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// The Unicode symbol
    pub codepoint: char,
    /// Category of the symbol (e.g. open delimiters, alphanumeric, etc.)
    pub atom_type: AtomType
}
impl Symbol {
    /// Given a LateX name (e.g. "alpha"), returns the corresponding symbol
    pub fn from_name(name: &str) -> Option<Self> {
        others(name).or_else(|| symbol(name))
    }
}

fn symbol(name: &str) -> Option<Symbol> {
    SYMBOLS.iter().find(|sym| sym.name == name).map(|sym| {
        Symbol {
            codepoint: sym.codepoint,
            atom_type: sym.atom_type
        }
    })
}

fn others(name: &str) -> Option<Symbol> {
    let sym = match name {
        // Additional commands from TeX
        "Alpha" => Symbol { codepoint: '\u{391}', atom_type: AtomType::Alpha }, // 913
        "Beta" => Symbol { codepoint: '\u{392}', atom_type: AtomType::Alpha }, // 914
        "Gamma" => Symbol { codepoint: '\u{393}', atom_type: AtomType::Alpha }, // 915
        "Delta" => Symbol { codepoint: '\u{394}', atom_type: AtomType::Alpha }, // 916
        "Epsilon" => Symbol { codepoint: '\u{395}', atom_type: AtomType::Alpha }, // 917
        "Zeta" => Symbol { codepoint: '\u{396}', atom_type: AtomType::Alpha }, // 918
        "Eta" => Symbol { codepoint: '\u{397}', atom_type: AtomType::Alpha }, // 919
        "Theta" => Symbol { codepoint: '\u{398}', atom_type: AtomType::Alpha }, // 920
        "Iota" => Symbol { codepoint: '\u{399}', atom_type: AtomType::Alpha }, // 921
        "Kappa" => Symbol { codepoint: '\u{39A}', atom_type: AtomType::Alpha }, // 922
        "Lambda" => Symbol { codepoint: '\u{39B}', atom_type: AtomType::Alpha }, // 923
        "Mu" => Symbol { codepoint: '\u{39C}', atom_type: AtomType::Alpha }, // 924
        "Nu" => Symbol { codepoint: '\u{39D}', atom_type: AtomType::Alpha }, // 925
        "Xi" => Symbol { codepoint: '\u{39E}', atom_type: AtomType::Alpha }, // 926
        "Omicron" => Symbol { codepoint: '\u{39F}', atom_type: AtomType::Alpha }, // 927
        "Pi" => Symbol { codepoint: '\u{3A0}', atom_type: AtomType::Alpha }, // 928
        "Rho" => Symbol { codepoint: '\u{3A1}', atom_type: AtomType::Alpha }, // 929
        "Sigma" => Symbol { codepoint: '\u{3A3}', atom_type: AtomType::Alpha }, // 931
        "Tau" => Symbol { codepoint: '\u{3A4}', atom_type: AtomType::Alpha }, // 932
        "Upsilon" => Symbol { codepoint: '\u{3A5}', atom_type: AtomType::Alpha }, // 933
        "Phi" => Symbol { codepoint: '\u{3A6}', atom_type: AtomType::Alpha }, // 934
        "Chi" => Symbol { codepoint: '\u{3A7}', atom_type: AtomType::Alpha }, // 935
        "Psi" => Symbol { codepoint: '\u{3A8}', atom_type: AtomType::Alpha }, // 936
        "Omega" => Symbol { codepoint: '\u{3A9}', atom_type: AtomType::Alpha }, // 937
        "alpha" => Symbol { codepoint: '\u{3B1}', atom_type: AtomType::Alpha }, // 945
        "beta" => Symbol { codepoint: '\u{3B2}', atom_type: AtomType::Alpha }, // 946
        "gamma" => Symbol { codepoint: '\u{3B3}', atom_type: AtomType::Alpha }, // 947
        "delta" => Symbol { codepoint: '\u{3B4}', atom_type: AtomType::Alpha }, // 948
        "epsilon" => Symbol { codepoint: '\u{3B5}', atom_type: AtomType::Alpha }, // 949
        "zeta" => Symbol { codepoint: '\u{3B6}', atom_type: AtomType::Alpha }, // 950
        "eta" => Symbol { codepoint: '\u{3B7}', atom_type: AtomType::Alpha }, // 951
        "theta" => Symbol { codepoint: '\u{3B8}', atom_type: AtomType::Alpha }, // 952
        "iota" => Symbol { codepoint: '\u{3B9}', atom_type: AtomType::Alpha }, // 953
        "kappa" => Symbol { codepoint: '\u{3BA}', atom_type: AtomType::Alpha }, // 954
        "lambda" => Symbol { codepoint: '\u{3BB}', atom_type: AtomType::Alpha }, // 955
        "mu" => Symbol { codepoint: '\u{3BC}', atom_type: AtomType::Alpha }, // 956
        "nu" => Symbol { codepoint: '\u{3BD}', atom_type: AtomType::Alpha }, // 957
        "xi" => Symbol { codepoint: '\u{3BE}', atom_type: AtomType::Alpha }, // 958
        "omicron" => Symbol { codepoint: '\u{3BF}', atom_type: AtomType::Alpha }, // 959
        "pi" => Symbol { codepoint: '\u{3C0}', atom_type: AtomType::Alpha }, // 960
        "rho" => Symbol { codepoint: '\u{3C1}', atom_type: AtomType::Alpha }, // 961
        "sigma" => Symbol { codepoint: '\u{3C3}', atom_type: AtomType::Alpha }, // 963
        "tau" => Symbol { codepoint: '\u{3C4}', atom_type: AtomType::Alpha }, // 964
        "upsilon" => Symbol { codepoint: '\u{3C5}', atom_type: AtomType::Alpha }, // 965
        "phi" => Symbol { codepoint: '\u{3C6}', atom_type: AtomType::Alpha }, // 966
        "chi" => Symbol { codepoint: '\u{3C7}', atom_type: AtomType::Alpha }, // 967
        "psi" => Symbol { codepoint: '\u{3C8}', atom_type: AtomType::Alpha }, // 968
        "omega" => Symbol { codepoint: '\u{3C9}', atom_type: AtomType::Alpha }, // 969

        "varphi" => Symbol { codepoint: '\u{3C6}', atom_type: AtomType::Alpha }, // codepoint: '\u{3C6}', curly or open small phi, greek
        "varsigma" => Symbol { codepoint: '\u{3C2}', atom_type: AtomType::Alpha }, // codepoint: '\u{3C2}', terminal sigma, greek
        "varbeta" => Symbol { codepoint: '\u{3D0}', atom_type: AtomType::Alpha }, // codepoint: '\u{3D0}', rounded small beta, greek
        "vartheta" => Symbol { codepoint: '\u{3D1}', atom_type: AtomType::Alpha }, // codepoint: '\u{3D1}', /vartheta - curly or open theta
        "varpi" => Symbol { codepoint: '\u{3D6}', atom_type: AtomType::Alpha }, // codepoint: '\u{3D6}', rounded small pi (pomega), greek
        "varkappa" => Symbol { codepoint: '\u{3F0}', atom_type: AtomType::Alpha }, // codepoint: '\u{3F0}', rounded small kappa, greek
        "varrho" => Symbol { codepoint: '\u{3F1}', atom_type: AtomType::Alpha }, // codepoint: '\u{3F1}', rounded small rho, greek
        "varTheta" => Symbol { codepoint: '\u{3F4}', atom_type: AtomType::Alpha }, // codepoint: '\u{3F4}', greek capital theta symbol
        "varepsilon" => Symbol { codepoint: '\u{3F5}', atom_type: AtomType::Alpha }, // codepoint: '\u{3F5}', greek lunate epsilon symbol
        "to" => Symbol { codepoint: '\u{2192}', atom_type: AtomType::Relation }, // codepoint: '\u{2192}', /rightarrow /to a: rightward arrow

        // Symbols from amsmath & stmaryrd
        

        // Symbol escape shim
        "{" => Symbol { codepoint: '\u{7B}', atom_type: AtomType::Open }, // 123
        "}" => Symbol { codepoint: '\u{7D}', atom_type: AtomType::Close }, // 125
        "%" => Symbol { codepoint: '\u{25}', atom_type: AtomType::Ordinary }, // 37
        "&" => Symbol { codepoint: '\u{26}', atom_type: AtomType::Ordinary }, // 38
        "$" => Symbol { codepoint: '\u{24}', atom_type: AtomType::Ordinary }, // 36
        "#" => Symbol { codepoint: '\u{23}', atom_type: AtomType::Ordinary }, // 35

        // Accents shim
        "`" => Symbol { codepoint: '\u{300}', atom_type: AtomType::Accent }, // 768
        "'" => Symbol { codepoint: '\u{301}', atom_type: AtomType::Accent }, // 769
        "^" => Symbol { codepoint: '\u{302}', atom_type: AtomType::Accent }, // 770
        "\"" => Symbol { codepoint: '\u{308}', atom_type: AtomType::Accent }, // 776
        "~" => Symbol { codepoint: '\u{303}', atom_type: AtomType::Accent }, // 771
        "." => Symbol { codepoint: '\u{307}', atom_type: AtomType::Accent }, // 775 

        // Binary operators shim
        "circ" => Symbol { codepoint: '\u{2218}', atom_type: AtomType::Binary }, // 8728
        "bullet" => Symbol { codepoint: '\u{2219}', atom_type: AtomType::Binary }, // 8729
        "diamond" => Symbol { codepoint: '\u{22C4}', atom_type: AtomType::Binary }, // 8900

        // dots shim
        // TODO: offer replacement if the font does not support these symbols
        "cdots" => Symbol { codepoint: '\u{22EF}', atom_type: AtomType::Ordinary }, // 8943
        "dots"  => Symbol { codepoint: '\u{22EF}', atom_type: AtomType::Ordinary }, // 8943
        "ldots" => Symbol { codepoint: '\u{2026}', atom_type: AtomType::Ordinary }, // 8230

        // Misc symbols shim
        "|" => Symbol { codepoint: '\u{2016}', atom_type: AtomType::Fence }, // 8214
        
        _ => return None
    };
    Some(sym)
}

/// Helper function for determining an atomtype based on a given codepoint.
/// This is primarily used for characters while processing, so may give false
/// negatives when used for other things.
pub fn codepoint_atom_type(codepoint: char) -> Option<AtomType> {
    Some(match codepoint {
             'a' ..= 'z' | 'A' ..= 'Z' | '0' ..= '9' | 'Α' ..= 'Ω' | 'α' ..= 'ω' => AtomType::Alpha,
             '*' | '+' | '-' => AtomType::Binary,
             '[' | '(' => AtomType::Open,
             ']' | ')' | '?' | '!' => AtomType::Close,
             '=' | '<' | '>' | ':' => AtomType::Relation,
             ',' | ';' => AtomType::Punctuation,
             '|' => AtomType::Fence,
             '/' | '@' | '.' | '"' => AtomType::Alpha,
             _ => return None,
         })
}
