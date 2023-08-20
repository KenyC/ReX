use crate::dimensions::AnyUnit;
use crate::font::{Weight, Family, AtomType, Style, style_symbol};
use crate::layout::Style as LayoutStyle;
use super::lexer::{Lexer, Token};
use super::macros::CommandCollection;
use super::{required_argument, required_argument_with};
use crate::parser as parse;
use crate::parser::nodes::{ParseNode, Radical, MathStyle, GenFraction, Rule, BarThickness, AtomChange,
                    Color, Stack};
use crate::parser::color::RGBA;
use crate::error::{ParseError, ParseResult};
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

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Command {
    Radical,
    Rule,
    Color,
    ColorLit(RGBA),
    Fraction(Option<Symbol>, Option<Symbol>, BarThickness, MathStyle),
    DelimiterSize(u8, AtomType),
    Kerning(AnyUnit),
    Style(LayoutStyle),
    AtomChange(AtomType),
    TextOperator(&'static str, bool),
    SubStack(AtomType),
    Text,
    // // DEPRECATED
    // VExtend,
}

#[cfg_attr(rustfmt, rustfmt_skip)]
impl Command {
    pub fn parse<'a>(self, lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
        use self::Command::*;
        match self {
            Radical              => radical(lex, local, command_collection),
            Rule                 => rule(lex, local, command_collection),
            Text                 => text(lex),
            Color                => color(lex, local, command_collection),
            ColorLit(a)          => color_lit(lex, local, command_collection, a),
            Fraction(a, b, c, d) => fraction(lex, local, command_collection, a, b, c, d),
            DelimiterSize(a, b)  => delimiter_size(lex, local, command_collection, a, b),
            Kerning(a)           => kerning(lex, local, a),
            Style(a)             => style(lex, local, a),
            AtomChange(a)        => atom_change(lex, local, command_collection, a),
            TextOperator(a, b)   => text_operator(lex, local, a, b),
            SubStack(a)          => substack(lex, local, command_collection, a),
            // // DEPRECATED
            // VExtend              => v_extend(lex, local, command_collection),
        }
    }
}

fn text<'a>(lex: &mut Lexer<'a>) -> ParseResult<'a, ParseNode> {
    lex.consume_whitespace();
    let text = lex.group()?.to_owned();
    lex.next();
    Ok(ParseNode::PlainText(parse::nodes::PlainText { text }))
}


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

fn radical<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    let inner = parse::required_argument(lex, local, command_collection)?;
    Ok(ParseNode::Radical(Radical { inner }))
}

/// Parses a `\rule{0.5em}{4em}` command. 
fn rule<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    lex.consume_whitespace();
    lex.next();
    
    let width = required_argument_with(lex, local, command_collection, |lex, _, _| {
        lex.dimension().ok_or(ParseError::ExpectedDimension)
    })?;

    if width.is_negative() {return Err(ParseError::RuleWithNegativeDimension(width));}

    lex.consume_whitespace();
    lex.next();

    let height = required_argument_with(lex, local, command_collection, |lex, _, _| {
        lex.dimension().ok_or(ParseError::ExpectedDimension)
    })?;
    if height.is_negative() {return Err(ParseError::RuleWithNegativeDimension(height));}

    Ok(ParseNode::Rule(Rule { width, height }))
}

// fn v_extend<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
//     let arg = parse::required_argument_with(lex, local, command_collection, parse::symbol)?;
//     let sym = match arg {
//         Some(ParseNode::Symbol(sym)) => sym,

//         // TODO: add better error
//         _ => return Err(ParseError::ExpectedOpenGroup),
//     };

//     let height = parse::required_argument_with(lex, local, command_collection, parse::dimension)?;
//     Ok(ParseNode::Extend(sym.codepoint, height))
// }

fn color<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, ParseNode> {
    let color = parse::required_argument_with(lex, local, command_collection, parse::color)?;
    let inner = parse::required_argument(lex, local, command_collection)?;
    Ok(ParseNode::Color(Color { color, inner }))
}

fn color_lit<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, color: RGBA) -> ParseResult<'a, ParseNode> {
    let inner = parse::required_argument(lex, local, command_collection)?;
    Ok(ParseNode::Color(Color { color, inner }))
}

fn fraction<'a>(
    lex: &mut Lexer<'a>,
    local: Style,
    command_collection : &CommandCollection,
    left_delimiter: Option<Symbol>,
    right_delimiter: Option<Symbol>,
    bar_thickness: BarThickness,
    style: MathStyle
) -> ParseResult<'a, ParseNode> {
    let numerator = parse::required_argument(lex, local, command_collection)?;
    let denominator = parse::required_argument(lex, local, command_collection)?;

    Ok(ParseNode::GenFraction(GenFraction {
        left_delimiter,
        right_delimiter,
        bar_thickness,
        numerator,
        denominator,
        style,
    }))
}

fn delimiter_size<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, _: u8, atom_type: AtomType) -> ParseResult<'a, ParseNode> {
    let symbol = parse::expect_type(lex, local, command_collection, atom_type)?;
    Ok(ParseNode::Symbol(symbol))
}

fn kerning<'a>(_: &mut Lexer<'a>, _: Style, unit: AnyUnit) -> ParseResult<'a, ParseNode> {
    Ok(ParseNode::Kerning(unit))
}

fn style<'a>(_: &mut Lexer<'a>, _: Style, new_style: LayoutStyle) -> ParseResult<'a, ParseNode> {
    Ok(ParseNode::Style(new_style))
}

fn atom_change<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, at: AtomType) -> ParseResult<'a, ParseNode> {
    let inner = parse::required_argument(lex, local, command_collection)?;
    Ok(ParseNode::AtomChange(AtomChange { at, inner }))
}

fn text_operator<'a>(_: &mut Lexer<'a>, style: Style, text: &str, limits: bool) -> ParseResult<'a, ParseNode> {
    const SMALL_SKIP: AnyUnit = AnyUnit::Em(3f64 / 18f64);
    let at = AtomType::Operator(limits);
    let mut inner = Vec::with_capacity(text.len());

    for c in text.chars() {
        if c == ',' {
            inner.push(ParseNode::Kerning(SMALL_SKIP));
        } else {
            let style = Style::default()
                    .with_family(Family::Roman)
                    .with_weight(Weight::None);
            let codepoint = style_symbol(c, style);
            inner.push(ParseNode::Symbol(Symbol { codepoint, atom_type: AtomType::Ordinary }));
        }
    }

    Ok(ParseNode::AtomChange(AtomChange { at, inner }))
}

fn substack<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, atom_type: AtomType) -> ParseResult<'a, ParseNode> {
    if lex.current() != Token::Symbol('{') {
        return Err(ParseError::StackMustFollowGroup);
    }

    let mut lines: Vec<Vec<ParseNode>> = Vec::new();
    lex.next();

    // Continue parsing expressions, until we reach '}'
    loop {
        lines.push(parse::expression(lex, local, command_collection)?);
        match lex.current() {
            Token::Symbol('}') => break,
            Token::Command(r"\") => lex.next(),
            _ => return Err(ParseError::NoClosingBracket),
        };
    }

    // Remove the last line if it's empty.  This is `\crcr`.
    if Some(true) == lines.last().map(|l| l.is_empty()) {
        lines.pop();
    }

    lex.next();
    Ok(ParseNode::Stack(Stack { atom_type, lines }))
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::lexer::Lexer;


    #[test]
    fn test_rule_parse() {
        let mut lexer = Lexer::new(r"\rule{1em}{50px}");
        let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        result.unwrap();

        let mut lexer = Lexer::new(r"\rule{  1.33em}  {  50px}");
        let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        result.unwrap();

        let mut lexer = Lexer::new(r"\rule  {  -0.5em}  {  50px}");
        let result = rule(&mut lexer, Style::default(), &CommandCollection::default());
        assert!(result.is_err());
    }
}