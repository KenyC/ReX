use regex::Regex;
use std::path::PathBuf;
use std::{env, fs};
use std::fmt::Write;

const OPERATOR_LIMITS: &[&str] = &[
    "coprod",
    "bigvee",
    "bigwedge",
    "biguplus",
    "bigcap",
    "bigcup",
    "prod",
    "sum",
    "bigotimes",
    "bigoplus",
    "bigodot",
    "bigsqcup",
];

const GREEK: &[(&str, u32)] = &[
    ("Alpha",   0x391),
    ("Beta",    0x392),
    ("Gamma",   0x393),
    ("Delta",   0x394),
    ("Epsilon", 0x395),
    ("Zeta",    0x396),
    ("Eta",     0x397),
    ("Theta",   0x398),
    ("Iota",    0x399),
    ("Kappa",   0x39A),
    ("Lambda",  0x39B),
    ("Mu",      0x39C),
    ("Nu",      0x39D),
    ("Xi",      0x39E),
    ("Omicron", 0x39F),
    ("Pi",      0x3A0),
    ("Rho",     0x3A1),

    ("Sigma",   0x3A3),
    ("Tau",     0x3A4),
    ("Upsilon", 0x3A5),
    ("Phi",     0x3A6),
    ("Chi",     0x3A7),
    ("Psi",     0x3A8),
    ("Omega",   0x3A9),

    ("alpha",   0x3B1),
    ("beta",    0x3B2),
    ("gamma",   0x3B3),
    ("delta",   0x3B4),
    ("epsilon", 0x3B5),
    ("zeta",    0x3B6),
    ("eta",     0x3B7),
    ("theta",   0x3B8),
    ("iota",    0x3B9),
    ("kappa",   0x3BA),
    ("lambda",  0x3BB),
    ("mu",      0x3BC),
    ("nu",      0x3BD),
    ("xi",      0x3BE),
    ("omicron", 0x3BF),
    ("pi",      0x3C0),
    ("rho",     0x3C1),

    ("sigma",   0x3C3),
    ("tau",     0x3C4),
    ("upsilon", 0x3C5),
    ("phi",     0x3C6),
    ("chi",     0x3C7),
    ("psi",     0x3C8),
    ("omega",   0x3C9),
];

fn atom_from_tex(name: &str, kind: &str) -> &'static str {
    match kind {
        "mathalpha" => "Alpha",
        "mathpunct" => "Punctuation",
        "mathopen" => "Open",
        "mathclose" => "Close",
        "mathord" => "Ordinal",
        "mathbin" => "Binary",
        "mathrel" => "Relation",
        "mathop" if OPERATOR_LIMITS.contains(&name) => "Operator(true)",
        "mathop" => "Operator(false)",
        "mathfence" => "Fence",
        "mathover" => "Over",
        "mathunder" => "Under",
        "mathaccent" => "Accent",
        "mathaccentwide" => "AccentWide",
        "mathaccentoverlay" => "AccentOverlay",
        "mathbotaccent" => "BotAccent",
        "mathbotaccentwide" => "BotAccentWide",
        op => panic!("unexpected {:?}", op)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=src/unicode-math-table.tex");
    println!("cargo:rerun-if-changed=build.rs");

    let path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src").join("unicode-math-table.tex");
    let source = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    let mut out = String::new();

    let re = Regex::new(r#"\\UnicodeMathSymbol\{"([[:xdigit:]]+)\}\{\\([[:alpha:]]+)\s*\}\{\\([[:alpha:]]+)\}\{([^\}]*)\}%"#).unwrap();
    writeln!(out, "[").unwrap();
    for line in source.lines() {
        if let Some(c) = re.captures(line) {
            writeln!(out,
                r"    Symbol {{ codepoint: '\u{{{}}}', name: {:?}, atom_type: AtomType::{}, description: {:?} }},",
                &c[1], &c[2], atom_from_tex(&c[2], &c[3]), &c[4]
            ).unwrap();
        }
    }
    for (name, cp) in GREEK {
        writeln!(out,
            r"    Symbol {{ codepoint: '\u{{{:x}}}', name: {:?}, atom_type: AtomType::Alpha, description: {:?} }},",
            cp, name, name
        ).unwrap();
    }
    writeln!(out, "]").unwrap();

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("symbols.rs");
    fs::write(out_path, out.as_bytes()).unwrap();
}
