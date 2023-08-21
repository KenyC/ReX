//! Defines structs and parses TeX command, e.g. `\sqrt`

use crate::dimensions::AnyUnit;
use crate::font::{Weight, Family, AtomType, Style, style_symbol};
use crate::layout::Style as LayoutStyle;
use super::lexer::{Lexer, Token};
use super::macros::CommandCollection;
use crate::parser as parse;
use crate::parser::nodes::{ParseNode, Radical, MathStyle, GenFraction, Rule, BarThickness, AtomChange,
                    Color, Stack};
use crate::parser::color::RGBA;
use crate::parser::error::{ParseError, ParseResult};
use crate::parser::symbols::Symbol;


macro_rules! sym {
    (@at ord) => { AtomType::Ordinary };
    (@at bin) => { AtomType::Binary };
    (@at op)  => { AtomType::Operator };
    (@at open) => { AtomType::Open };
    (@at close) => { AtomType::Close };

    ($code:expr, $ord:ident) => ({
        Some(Symbol {
            codepoint: $code,
            atom_type: sym!(@at $ord),
        })
    });
}

/// Recognized TeX commands
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command {
    /// A square root, as created by `\sqrt{..}`
    Radical,
    /// A filled rectangle, or 'rule', as created by `\rule{..}`
    Rule,
    /// A color command as created by `\color{red}`
    Color,
    /// A named color command, as created by one of `\red{..}`, `\blue{..}`, `\gray{..}`, `\phantom{..}`.
    /// These are  fake commands that only exist in ReX.
    // TODO: why do we have this if we have custom commands?
    ColorLit(RGBA),
    /// A fraction as created by `\frac{..}{..}`
    Fraction(Option<Symbol>, Option<Symbol>, BarThickness, MathStyle),
    /// Create delimiter of a given size, as created by `\bigl` or `\big`
    DelimiterSize(u8, AtomType),
    /// Add space between nodes like `\hspace`
    Kerning(AnyUnit),
    /// Any command affecting style like `\scriptstyle`, `\textstyle`
    Style(LayoutStyle),
    /// Any command changing the atom type of a node, like `\mathop` or `\mathbin`.
    /// The atom type decides how much gap to leave between elements.
    AtomChange(AtomType),
    /// A mathematical operator, like `\lim`, `\det`
    TextOperator(&'static str, bool),
    /// `\substack{..}{..}`
    SubStack(AtomType),
    /// `\text{..}`
    Text,
    // // DEPRECATED
    // VExtend,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Command {
    // pub fn parse<'a>(self, lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    //     use self::Command::*;
    //     match self {
    //         Radical              => radical(lex, local, command_collection),
    //         Rule                 => rule(lex, local, command_collection),
    //         Text                 => text(lex),
    //         Color                => color(lex, local, command_collection),
    //         ColorLit(a)          => color_lit(lex, local, command_collection, a),
    //         Fraction(a, b, c, d) => fraction(lex, local, command_collection, a, b, c, d),
    //         DelimiterSize(a, b)  => delimiter_size(lex, local, command_collection, a, b),
    //         Kerning(a)           => kerning(lex, local, a),
    //         Style(a)             => style(lex, local, a),
    //         AtomChange(a)        => atom_change(lex, local, command_collection, a),
    //         TextOperator(a, b)   => text_operator(lex, local, a, b),
    //         SubStack(a)          => substack(lex, local, command_collection, a),
    //         // // DEPRECATED
    //         // VExtend              => v_extend(lex, local, command_collection),
    //     }
    // }
}


/// Creates the `Command` corresponding to `\name`
pub fn get_command(name: &str) -> Option<Command> {
    let command = match name {
        "frac"   => Command::Fraction(None, None, BarThickness::Default, MathStyle::NoChange),
        "tfrac"  => Command::Fraction(None, None, BarThickness::Default, MathStyle::Text),
        "dfrac"  => Command::Fraction(None, None, BarThickness::Default, MathStyle::Display),
        "binom"  => Command::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::NoChange),
        "tbinom" => Command::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::Text),
        "dbinom" => Command::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::Display),

        // Stacking commands
        "substack" => Command::SubStack(AtomType::Inner),

        // Radical commands
        "sqrt" => Command::Radical,

        // Delimiter size commands
        "bigl"  => Command::DelimiterSize(1, AtomType::Open),
        "Bigl"  => Command::DelimiterSize(2, AtomType::Open),
        "biggl" => Command::DelimiterSize(3, AtomType::Open),
        "Biggl" => Command::DelimiterSize(4, AtomType::Open),
        "bigr"  => Command::DelimiterSize(1, AtomType::Close),
        "Bigr"  => Command::DelimiterSize(2, AtomType::Close),
        "biggr" => Command::DelimiterSize(3, AtomType::Close),
        "Biggr" => Command::DelimiterSize(4, AtomType::Close),
        "bigm"  => Command::DelimiterSize(1, AtomType::Relation),
        "Bigm"  => Command::DelimiterSize(2, AtomType::Relation),
        "biggm" => Command::DelimiterSize(3, AtomType::Relation),
        "Biggm" => Command::DelimiterSize(4, AtomType::Relation),
        "big"   => Command::DelimiterSize(1, AtomType::Ordinary),
        "Big"   => Command::DelimiterSize(2, AtomType::Ordinary),
        "bigg"  => Command::DelimiterSize(3, AtomType::Ordinary),
        "Bigg"  => Command::DelimiterSize(4, AtomType::Ordinary),

        // Spacing related commands
        "!"     => Command::Kerning(AnyUnit::Em(-3f64/18f64)),
        ","     => Command::Kerning(AnyUnit::Em(3f64/18f64)),
        ":"     => Command::Kerning(AnyUnit::Em(4f64/18f64)),
        ";"     => Command::Kerning(AnyUnit::Em(5f64/18f64)),
        " "     => Command::Kerning(AnyUnit::Em(1f64/4f64)),
        "quad"  => Command::Kerning(AnyUnit::Em(1.0f64)),
        "qquad" => Command::Kerning(AnyUnit::Em(2.0f64)),
        "rule"  => Command::Rule,

        // // Useful other than debugging?
        // // DEPRECATED
        // "vextend" => Command::VExtend,

        // Display style changes
        "textstyle"         => Command::Style(LayoutStyle::Text),
        "displaystyle"      => Command::Style(LayoutStyle::Display),
        "scriptstyle"       => Command::Style(LayoutStyle::Script),
        "scriptscriptstyle" => Command::Style(LayoutStyle::ScriptScript),
        "text"              => Command::Text,

        // Atom-type changes
        "mathop"  => Command::AtomChange(AtomType::Operator(false)),
        "mathrel" => Command::AtomChange(AtomType::Relation),
        "mathord" => Command::AtomChange(AtomType::Alpha),

        // Color related
        "color"   => Command::Color,
        "blue"    => Command::ColorLit(RGBA(0,0,0xff,0xff)),
        "red"     => Command::ColorLit(RGBA(0xff,0,0,0xff)),
        "gray"    => Command::ColorLit(RGBA(0x80,0x80,0x80,0xff)),
        "phantom" => Command::ColorLit(RGBA(0,0,0,0)),

        // Operators with limits
        "det"     => Command::TextOperator("det", true),
        "gcd"     => Command::TextOperator("gcd", true),
        "lim"     => Command::TextOperator("lim", true),
        "limsup"  => Command::TextOperator("lim,sup", true),
        "liminf"  => Command::TextOperator("lim,inf", true),
        "sup"     => Command::TextOperator("sup", true),
        "supp"    => Command::TextOperator("supp", true),
        "inf"     => Command::TextOperator("inf", true),
        "max"     => Command::TextOperator("max", true),
        "min"     => Command::TextOperator("min", true),
        "Pr"      => Command::TextOperator("Pr", true),

        // Operators without limits
        "sin"     => Command::TextOperator("sin", false),
        "cos"     => Command::TextOperator("cos", false),
        "tan"     => Command::TextOperator("tan", false),
        "cot"     => Command::TextOperator("cot", false),
        "csc"     => Command::TextOperator("csc", false),
        "sec"     => Command::TextOperator("sec", false),
        "arcsin"  => Command::TextOperator("arcsin", false),
        "arccos"  => Command::TextOperator("arccos", false),
        "arctan"  => Command::TextOperator("arctan", false),
        "sinh"    => Command::TextOperator("sinh", false),
        "cosh"    => Command::TextOperator("cosh", false),
        "tanh"    => Command::TextOperator("tanh", false),
        "arg"     => Command::TextOperator("arg", false),
        "deg"     => Command::TextOperator("deg", false),
        "dim"     => Command::TextOperator("dim", false),
        "exp"     => Command::TextOperator("exp", false),
        "hom"     => Command::TextOperator("hom", false),
        "Hom"     => Command::TextOperator("Hom", false),
        "ker"     => Command::TextOperator("ker", false),
        "Ker"     => Command::TextOperator("Ker", false),
        "ln"      => Command::TextOperator("ln", false),
        "log"     => Command::TextOperator("log", false),
        _ => return None
    };
    Some(command)
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;


    #[test]
    fn test_rule_parse() {
        todo!("Replace")
        // let mut lexer = Lexer::new(r"\rule{1em}{50px}");
        // let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        // result.unwrap();

        // let mut lexer = Lexer::new(r"\rule{  1.33em}  {  50px}");
        // let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        // result.unwrap();

        // let mut lexer = Lexer::new(r"\rule  {  -0.5em}  {  50px}");
        // let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        // assert!(result.is_err());
    }
}