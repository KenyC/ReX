use unicode_math::AtomType;

use crate::{dimensions::AnyUnit, font::{Family, Weight}, layout::Style as LayoutStyle, parser::{nodes::{BarThickness, MathStyle}, symbols::Symbol}, RGBA};

use super::{error::{ParseError, ParseResult}, macros::CommandCollection, nodes::Color, textoken::TexToken, Parser};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PrimitiveControlSequence {
    /// Represents LaTeX `\sqrt{..}`
    Radical,
    Rule,
    /// Represents ReX's command `\color{..}{..}`
    Color,
    /// Represents ReX's command `\blue{..}`, `\red{..}`
    ColorLit(RGBA),
    /// Represents LaTeX `\frac{..}`
    Fraction(Option<Symbol>, Option<Symbol>, BarThickness, MathStyle),
    DelimiterSize(u8, AtomType),
    Kerning(AnyUnit),
    StyleCommand(LayoutStyle),
    AtomChange(AtomType),
    TextOperator(&'static str, bool),
    SubStack(AtomType),
    SymbolCommand(Symbol),
    StyleChange { family: Option<Family>, weight: Option<Weight> },
    BeginEnv,
    EndEnv,
    Left,
    Middle,
    Right,
    Text,
}


impl PrimitiveControlSequence {
    pub fn from_name(name: &str) -> Option<Self> {
        Option::or_else(
            
            Self::parse_command_name(name),
            || Symbol::from_name(name).map(PrimitiveControlSequence::SymbolCommand),
        )
    }

    fn parse_command_name(name: &str) -> Option<Self> {
        // TODO: use a lookup table
        const OPEN_PAREN  : Option<Symbol> = Some(Symbol { codepoint : '(', atom_type : AtomType::Open  });
        const CLOSE_PAREN : Option<Symbol> = Some(Symbol { codepoint : ')', atom_type : AtomType::Close });
        Some(match name {
            "frac"   => Self::Fraction(None, None,              BarThickness::Default, MathStyle::NoChange),
            "tfrac"  => Self::Fraction(None, None,              BarThickness::Default, MathStyle::Text),
            "dfrac"  => Self::Fraction(None, None,              BarThickness::Default, MathStyle::Display),
            "binom"  => Self::Fraction(OPEN_PAREN, CLOSE_PAREN, BarThickness::None,    MathStyle::NoChange),
            "tbinom" => Self::Fraction(OPEN_PAREN, CLOSE_PAREN, BarThickness::None,    MathStyle::Text),
            "dbinom" => Self::Fraction(OPEN_PAREN, CLOSE_PAREN, BarThickness::None,    MathStyle::Display),

            // Stacking commands
            "substack" => Self::SubStack(AtomType::Inner),

            // Radical commands
            "sqrt" => Self::Radical,

            // Style-change command
            "mathbf"   => Self::StyleChange {family: None,                     weight: Some(Weight::Bold)   },
            "mathit"   => Self::StyleChange {family: None,                     weight: Some(Weight::Italic) },
            "mathrm"   => Self::StyleChange {family: Some(Family::Roman),      weight: None                 },
            "mathscr"  => Self::StyleChange {family: Some(Family::Script),     weight: None                 },
            "mathfrak" => Self::StyleChange {family: Some(Family::Fraktur),    weight: None                 },
            "mathbb"   => Self::StyleChange {family: Some(Family::Blackboard), weight: None                 },
            "mathsf"   => Self::StyleChange {family: Some(Family::SansSerif),  weight: None                 },
            "mathtt"   => Self::StyleChange {family: Some(Family::Monospace),  weight: None                 },
            "mathcal"  => Self::StyleChange {family: Some(Family::Script),     weight: None                 },


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


            // Display style changes
            "textstyle"         => Self::StyleCommand(LayoutStyle::Text),
            "displaystyle"      => Self::StyleCommand(LayoutStyle::Display),
            "scriptstyle"       => Self::StyleCommand(LayoutStyle::Script),
            "scriptscriptstyle" => Self::StyleCommand(LayoutStyle::ScriptScript),
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

            // Environment
            "begin" => Self::BeginEnv,
            "end"   => Self::EndEnv,

            // Environment
            "left"    => Self::Left,
            "middle"  => Self::Middle,
            "right"   => Self::Right,

            _ => return None
        })
    }
}




pub fn parse_color<'a, I : Iterator<Item = TexToken<'a>>>(token_iter : I) -> ParseResult<RGBA> {
    let mut color_name = String::with_capacity("#11223344".len()); // #rrggbbaa, preparing for the worst case
    for token in token_iter {
        match token {
            TexToken::Char(c) => color_name.push(c),
            TexToken::ControlSequence(_) => todo!(),
            _ => todo!()
        }
    }
    let color : RGBA = color_name.parse().map_err(|_| ParseError::UnrecognizedColor(color_name.into_boxed_str()))?;
    Ok(color)
}

