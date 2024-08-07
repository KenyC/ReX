use regex::Regex;
use std::path::PathBuf;
use std::{env, fs};

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

const SUPPLEMENTAL_SYMBOLS : &[(u32, &str, &str, &str)] = &[
    (0x003C, "le",       "mathrel",   "less-than sign"),
    (0x003E, "ge",       "mathrel",   "greater-than sign"),
    (0x2260, "neq",      "mathrel",   "not equal to"),
    (0x2016, "lVert",    "mathopen",  "double vertical lign"), // TODO: is this right?
    (0x2016, "rVert",    "mathclose", "double vertical lign"), // TODO: is this right?
    (0x003A, "colon",    "mathpunct", "colon"),
    (0x2205, "emptyset", "mathord",   "circle, dash"), // TODO: is there a better unicode char for the empty set
    (0x210F, "hbar",     "mathalpha", "Planck's constant over 2pi"),
    (0x2026, "hdots",    "mathinner", "horizontal ellipsis"),
    (0x00B6, "P",        "mathord",   "Pilcrow sign"),
    (0x00B6, "gets",     "mathrel",   "leftwards arrow"),

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
        "mathord" => "Ordinary",
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
        "mathinner" => "Inner",
        op => panic!("unexpected {:?}", op)
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    build_symbol_table();
    build_alphanumeric_table_reserved_replacements();
}

fn build_alphanumeric_table_reserved_replacements() {
    use std::fmt::Write;
    println!("cargo:rerun-if-changed=resources/math_alphanumeric_list.html");

    let path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("resources").join("math_alphanumeric_list.html");
    let source = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    let mut out = String::new();

    writeln!(out, "[").unwrap();
    let re = Regex::new(r#"<tr><td><code><a name="[0-9A-F]+">([0-9A-F]+)</a></code></td><td class="c">&nbsp;█&nbsp;</td><td colspan="2"><span class="name">&lt;Reserved&gt;</span></td></tr>\n<tr><td>&nbsp;</td><td class="char">&nbsp;</td><td class="c">→</td><td><code>([0-9A-F]+)</code>"#).unwrap();
    // let re = Regex::new(r#"<tr><td><code><a name="[0-9A-F]+">([0-9A-F]+)<\/a><\/code><\/td><td class="c">&nbsp;█&nbsp;<\/td><td colspan="2"><span class="name">&lt;Reserved&gt;<\/span><\/td><\/tr>\n<tr><td>&nbsp;<\/td><td class="char">&nbsp;<\/td><td class="c">→<\/td><td><code>([0-9A-F]+)<\/code>"#).unwrap();
    let mut n_replacements = 0;
    for capture in re.captures_iter(&source) {
        writeln!(out,
            r"(0x{}, 0x{}),",
            &capture[1], &capture[2],
        ).unwrap();
        n_replacements += 1;
    }

    // Sanity check: there should be as many replacements as the word "Reserved" appears.
    let re = Regex::new(r#"Reserved"#).unwrap();
    assert_eq!(n_replacements, re.captures_iter(&source).count());
    writeln!(out, "]").unwrap();

    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("math_alphanumeric_table_reserved_replacements.rs");
    fs::write(out_path, out.as_bytes()).unwrap();
}

fn build_symbol_table() {
    use std::io::Write;

    println!("cargo:rerun-if-changed=resources/unicode-math-table.tex");
    
    let mut lines = Vec::new();

    let re = Regex::new(r#"\\UnicodeMathSymbol\{"([[:xdigit:]]+)\}\{\\([[:alpha:]]+)\s*\}\{\\([[:alpha:]]+)\}\{([^\}]*)\}%"#).unwrap();

    let path = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("resources").join("unicode-math-table.tex");
    let source = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    for line in source.lines() {
        if let Some(c) = re.captures(line) {
            let symbol_name = c[2].to_string();
            let line = format!(
                r"    Symbol {{ codepoint: '\u{{{}}}', name: {:?}, atom_type: TexSymbolType::{}, description: {:?} }},",
                &c[1], &c[2], atom_from_tex(&c[2], &c[3]), &c[4]
            );
            lines.push(Line {line, symbol_name})
        }
    }

    for (character, name, atom_type, description) in SUPPLEMENTAL_SYMBOLS {
        let symbol_name = name.to_string();
        let line = format!(
            r"    Symbol {{ codepoint: '\u{{{:x}}}', name: {:?}, atom_type: TexSymbolType::{}, description: {:?} }},",
            character, name, atom_from_tex(name, atom_type), description,
        );
        lines.push(Line {line, symbol_name});
    }
    for (name, cp) in GREEK {
        let symbol_name = name.to_string();
        let line = format!(
            r"    Symbol {{ codepoint: '\u{{{:x}}}', name: {:?}, atom_type: TexSymbolType::Alpha, description: {:?} }},",
            cp, name, name
        );
        lines.push(Line {line, symbol_name});
    }

    lines.sort_by(|line1, line2| Ord::cmp(&line1.symbol_name, &line2.symbol_name));


    let out_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("symbols.rs");

    let file = std::fs::File::create(out_path).expect("Couldn't write to file:");
    let mut out = std::io::BufWriter::new(file);
    writeln!(out, "[").unwrap();
    for Line {line, ..} in lines {
        out.write(line.as_bytes()).unwrap();
    }
    writeln!(out, "]").unwrap();
    out.flush().unwrap();

}


struct Line {
    line : String,
    symbol_name : String,
}