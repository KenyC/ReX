//! Defines structs and parses TeX command, e.g. `\sqrt`

use crate::dimensions::AnyUnit;
use crate::font::{AtomType, Style, Family};
use crate::layout::Style as LayoutStyle;
use crate::parser::nodes::{MathStyle, BarThickness, GenFraction};
use crate::parser::color::RGBA;
use crate::parser::symbols::Symbol;

use super::nodes::{Radical, Rule, Stack};
use super::utils::fmap;
use super::{ParseNode, Parser, ParseDelimiter};
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
    /// A change in mathface (e.g. `\mathit`)
    FaceChange(FaceChange),
    /// A mathematical operator, like `\lim`, `\det`
    TextOperator(&'static str, bool),
    /// `\substack{..}`
    SubStack,
    /// `\text{..}`
    Text,
    // // DEPRECATED
    // VExtend,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Command {


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
            "substack" => Self::SubStack,

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

            // Face changes
            "mathbf"   => Self::FaceChange(FaceChange::MakeBold),
            "mathit"   => Self::FaceChange(FaceChange::MakeItalic),
            "mathrm"   => Self::FaceChange(FaceChange::SetFamily(Family::Roman)),
            "mathscr"  => Self::FaceChange(FaceChange::SetFamily(Family::Script)),
            "mathfrak" => Self::FaceChange(FaceChange::SetFamily(Family::Fraktur)),
            "mathbb"   => Self::FaceChange(FaceChange::SetFamily(Family::Blackboard)),
            "mathsf"   => Self::FaceChange(FaceChange::SetFamily(Family::SansSerif)),
            "mathtt"   => Self::FaceChange(FaceChange::SetFamily(Family::Monospace)),
            "mathcal"  => Self::FaceChange(FaceChange::SetFamily(Family::Script)),


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


/// A type representing a change of font style request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceChange {
    /// Make font bold
    MakeBold,
    /// Make font italic
    MakeItalic,
    /// Set font family to given value
    SetFamily(Family),
}

impl FaceChange {
    /// Applies the change to they style passed as argument
    pub fn apply_to(self, style : Style) -> Style {
        match self {
            FaceChange::MakeBold          => style.with_bold(),
            FaceChange::MakeItalic        => style.with_italics(),
            FaceChange::SetFamily(family) => style.with_family(family),
        }
    }
}



impl<'i, 'c> Parser<'i, 'c> {
    /// Parses the argument of a control sequence, or a '_' subscript or superscipt
    pub fn parse_required_argument(&mut self) -> ParseResult<Vec<ParseNode>> {
        self.consume_whitespace();


        fmap(self.parse_control_sequence_token(), |node| vec![node]) // really clunky
            .or_else(|| self.parse_group())
            .or_else(|| fmap(self.parse_symbol(), |symbol| vec![ParseNode::Symbol(symbol)]))
            .ok_or(ParseError::RequiredMacroArg)?
    }

    /// Like [`Parser::parse_control_sequence`], except that we assume that the control sequence is followed by an end group token
    fn parse_control_sequence_token(&mut self) -> Option<ParseResult<ParseNode>> {
        let name = self.parse_control_sequence_name()?;
        let mut parser = self.fork();
        parser.input = "}";
        Some(parser.parse_control_sequence_args(name))
    }


    /// After parsing `\sqrt`, we expect to parse a group
    pub fn parse_radical(&mut self) -> ParseResult<Radical> {
        let inner = self.parse_required_argument()?;
        Ok(Radical { inner })
    }

    /// After parsing `\frac`, we expect two arguments
    pub fn parse_fraction(
        &mut self, 
        left_delimiter:  Option<Symbol>, 
        right_delimiter: Option<Symbol>, 
        bar_thickness: BarThickness, 
        style: MathStyle
    ) -> ParseResult<GenFraction> {
        let numerator   = self.parse_required_argument()?;
        let denominator = self.parse_required_argument()?;

        Ok(GenFraction {
            numerator, denominator,
            left_delimiter, right_delimiter,
            bar_thickness,
            style,
        })
    }

    /// Parses the argument of e.g. `\mathrm`, `\mathit`, ...
    pub fn parse_face_change(&mut self, face_change: FaceChange) -> ParseResult<Vec<ParseNode>> {
        let mut parser = self.fork();
        parser.local_style = face_change.apply_to(self.local_style);
        let result = parser.parse_required_argument();
        self.input = parser.input;
        result
    }


    /// This method should be called after having parsed `\rule`
    /// It will take care of parsing two required arguments, containing the desired dimension of the rule
    pub fn parse_rule(&mut self) -> ParseResult<Rule> {
        self.consume_whitespace();
        let group1 = self.parse_group_as_string().ok_or(ParseError::RequiredMacroArg)?;
        let width = Parser::new(group1).parse_dimension()?;

        self.consume_whitespace();
        let group2 = self.parse_group_as_string().ok_or(ParseError::RequiredMacroArg)?;
        let height = Parser::new(group2).parse_dimension()?;

        Ok(Rule {width, height,})
    }


    /// This method parses the argument of `\substack` which is a group of the form `{ ... \\ ... \\ ... }`
    pub fn parse_substack(&mut self) -> ParseResult<Stack> {
        self.consume_whitespace();

        self.try_parse_char('{').ok_or(ParseError::RequiredMacroArg)?;

        let mut lines  = Vec::with_capacity(2);

        loop {
            let mut parser = self.fork();
            let delimiter = parser.parse_expression()?;

            self.input = parser.input;
            let results = parser.to_results();

            match delimiter {
                ParseDelimiter::EndOfLine    => lines.push(results),
                ParseDelimiter::CloseBracket => {
                    if !results.is_empty(){
                        lines.push(results)
                    }
                    break
                },
                other => return Err(ParseError::ExpectedDelimiter { found: other, expected: ParseDelimiter::CloseBracket })
            }

        }

        Ok(Stack { lines, })
    }


    /// Parses the symbol after e.g. `\bigl` and `\biggr`
    pub fn parse_delimiter_size(&mut self, atom_type : AtomType) -> ParseResult<Symbol> {
        let delimiter = self.parse_delimiter().ok_or(ParseError::MissingSymbolAfterDelimiter)??;
        if delimiter.atom_type != atom_type {
            return Err(ParseError::ExpectedAtomType { found: delimiter.atom_type, expected: atom_type });
        }
        Ok(delimiter)
    }

}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_substack() {
        // successes
        let mut parser = Parser::new(r"  { 1   \\ 2 }");
        let result = parser.parse_substack();
        result.unwrap();

        let mut parser = Parser::new(r"  { 1   \\ 2 \\ 3 }");
        let result = parser.parse_substack();
        result.unwrap();

        let mut parser = Parser::new(r"  { 1   \\ 2 \\ \frac{3+1} {5+ 6} }");
        let result = parser.parse_substack();
        result.unwrap();

        // failures
        let mut parser = Parser::new(r"  { 1   \\ 2 ");
        let result = parser.parse_substack();
        result.unwrap_err();

        let mut parser = Parser::new(r"  { 1    \\ ");
        let result = parser.parse_substack();
        result.unwrap_err();

        let mut parser = Parser::new(r"  123 \\ 2}");
        let result = parser.parse_substack();
        result.unwrap_err();
    }


    #[test]
    fn test_rule_parse() {
        let mut parser = Parser::new(r"{1em}{50px}");
        let result = parser.parse_rule();
        result.unwrap();

        let mut parser = Parser::new(r"{  1.33em}  {  50px}");
        let result = parser.parse_rule();
        result.unwrap();

        let mut parser = Parser::new(r"  {  -0.5em}  {  50px}");
        let result = parser.parse_rule();
        result.unwrap();
    }
}