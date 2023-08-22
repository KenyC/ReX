//! Defines structs and parses TeX command, e.g. `\sqrt`

use crate::dimensions::AnyUnit;
use crate::font::{AtomType};
use crate::layout::Style as LayoutStyle;
use crate::parser::nodes::{MathStyle, BarThickness};
use crate::parser::color::RGBA;
use crate::parser::symbols::Symbol;

use super::nodes::Radical;
use super::{ParseNode, Parser};
use super::error::{ParseResult, ParseError};


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


    /// Creates the `Command` corresponding to `\name`
    pub fn from_name(name: &str) -> Option<Self> {
        let command = match name {
            "frac"   => Self::Fraction(None, None, BarThickness::Default, MathStyle::NoChange),
            "tfrac"  => Self::Fraction(None, None, BarThickness::Default, MathStyle::Text),
            "dfrac"  => Self::Fraction(None, None, BarThickness::Default, MathStyle::Display),
            "binom"  => Self::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::NoChange),
            "tbinom" => Self::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::Text),
            "dbinom" => Self::Fraction(sym!('(', open), sym!(')', close), BarThickness::None, MathStyle::Display),

            // Stacking commands
            "substack" => Self::SubStack(AtomType::Inner),

            // Radical commands
            "sqrt" => Self::Radical,

            // Delimiter size commands
            "bigl"  => Self::DelimiterSize(1, AtomType::Open),
            "Bigl"  => Self::DelimiterSize(2, AtomType::Open),
            "biggl" => Self::DelimiterSize(3, AtomType::Open),
            "Biggl" => Self::DelimiterSize(4, AtomType::Open),
            "bigr"  => Self::DelimiterSize(1, AtomType::Close),
            "Bigr"  => Self::DelimiterSize(2, AtomType::Close),
            "biggr" => Self::DelimiterSize(3, AtomType::Close),
            "Biggr" => Self::DelimiterSize(4, AtomType::Close),
            "bigm"  => Self::DelimiterSize(1, AtomType::Relation),
            "Bigm"  => Self::DelimiterSize(2, AtomType::Relation),
            "biggm" => Self::DelimiterSize(3, AtomType::Relation),
            "Biggm" => Self::DelimiterSize(4, AtomType::Relation),
            "big"   => Self::DelimiterSize(1, AtomType::Ordinary),
            "Big"   => Self::DelimiterSize(2, AtomType::Ordinary),
            "bigg"  => Self::DelimiterSize(3, AtomType::Ordinary),
            "Bigg"  => Self::DelimiterSize(4, AtomType::Ordinary),

            // Spacing related commands
            "!"     => Self::Kerning(AnyUnit::Em(-3f64/18f64)),
            ","     => Self::Kerning(AnyUnit::Em(3f64/18f64)),
            ":"     => Self::Kerning(AnyUnit::Em(4f64/18f64)),
            ";"     => Self::Kerning(AnyUnit::Em(5f64/18f64)),
            " "     => Self::Kerning(AnyUnit::Em(1f64/4f64)),
            "quad"  => Self::Kerning(AnyUnit::Em(1.0f64)),
            "qquad" => Self::Kerning(AnyUnit::Em(2.0f64)),
            "rule"  => Self::Rule,

            // // Useful other than debugging?
            // // DEPRECATED
            // "vextend" => Self::VExtend,

            // Display style changes
            "textstyle"         => Self::Style(LayoutStyle::Text),
            "displaystyle"      => Self::Style(LayoutStyle::Display),
            "scriptstyle"       => Self::Style(LayoutStyle::Script),
            "scriptscriptstyle" => Self::Style(LayoutStyle::ScriptScript),
            "text"              => Self::Text,

            // Atom-type changes
            "mathop"  => Self::AtomChange(AtomType::Operator(false)),
            "mathrel" => Self::AtomChange(AtomType::Relation),
            "mathord" => Self::AtomChange(AtomType::Alpha),

            // Color related
            "color"   => Self::Color,
            "blue"    => Self::ColorLit(RGBA(0,0,0xff,0xff)),
            "red"     => Self::ColorLit(RGBA(0xff,0,0,0xff)),
            "gray"    => Self::ColorLit(RGBA(0x80,0x80,0x80,0xff)),
            "phantom" => Self::ColorLit(RGBA(0,0,0,0)),

            // Operators with limits
            "det"     => Self::TextOperator("det", true),
            "gcd"     => Self::TextOperator("gcd", true),
            "lim"     => Self::TextOperator("lim", true),
            "limsup"  => Self::TextOperator("lim,sup", true),
            "liminf"  => Self::TextOperator("lim,inf", true),
            "sup"     => Self::TextOperator("sup", true),
            "supp"    => Self::TextOperator("supp", true),
            "inf"     => Self::TextOperator("inf", true),
            "max"     => Self::TextOperator("max", true),
            "min"     => Self::TextOperator("min", true),
            "Pr"      => Self::TextOperator("Pr", true),

            // Operators without limits
            "sin"     => Self::TextOperator("sin", false),
            "cos"     => Self::TextOperator("cos", false),
            "tan"     => Self::TextOperator("tan", false),
            "cot"     => Self::TextOperator("cot", false),
            "csc"     => Self::TextOperator("csc", false),
            "sec"     => Self::TextOperator("sec", false),
            "arcsin"  => Self::TextOperator("arcsin", false),
            "arccos"  => Self::TextOperator("arccos", false),
            "arctan"  => Self::TextOperator("arctan", false),
            "sinh"    => Self::TextOperator("sinh", false),
            "cosh"    => Self::TextOperator("cosh", false),
            "tanh"    => Self::TextOperator("tanh", false),
            "arg"     => Self::TextOperator("arg", false),
            "deg"     => Self::TextOperator("deg", false),
            "dim"     => Self::TextOperator("dim", false),
            "exp"     => Self::TextOperator("exp", false),
            "hom"     => Self::TextOperator("hom", false),
            "Hom"     => Self::TextOperator("Hom", false),
            "ker"     => Self::TextOperator("ker", false),
            "Ker"     => Self::TextOperator("Ker", false),
            "ln"      => Self::TextOperator("ln", false),
            "log"     => Self::TextOperator("log", false),
            _ => return None
        };
        Some(command)
    }
}


impl<'i, 'c> Parser<'i, 'c> {
    fn parse_required_argument(&mut self) -> ParseResult<Vec<ParseNode>> {
        let Self { input, result, .. } = self;
        
        self.consume_whitespace();

        fn lift(node : Option<ParseResult<ParseNode>>) -> Option<ParseResult<Vec<ParseNode>>> {
            node.map(|maybe_node| maybe_node.map(|node| vec![node]))
        }

        lift(self.parse_control_sequence()) // really clunky
            .or_else(|| self.parse_group())
            .or_else(|| lift(self.parse_symbol()))
            .ok_or(ParseError::RequiredMacroArg)?
    }


    /// After parsing `\sqrt`, we expect to parse a group
    pub fn parse_radical(&mut self) -> ParseResult<Radical> {
        let inner = self.parse_required_argument()?;
        Ok(Radical { inner })
    }
}



#[cfg(test)]
mod tests {
    use super::*;


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