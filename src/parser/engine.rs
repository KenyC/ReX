use crate::error::{ParseError, ParseResult};
use crate::font::{Style, style_symbol, AtomType};
use crate::parser::{
    nodes::{Delimited, ParseNode, Accent, Scripts},
    symbols::Symbol,
    color::RGBA,
    environments::Environment,
};
use super::lexer::{Lexer, Token};
use super::functions::get_command;
use super::macros::{CommandCollection, CustomCommand};
use crate::dimensions::*;

fn expression_until_opt<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, end: Option<Token>) -> ParseResult<'a, Vec<ParseNode>> {
    let mut ml: Vec<ParseNode> = Vec::new();
    loop {
        // TODO: Handle INFIX operators here, once we support them.
        lex.consume_whitespace();
        if Some(lex.current()) == end || lex.current().ends_expression() {
            break;
        }

        let node = alt!(command(lex, local, command_collection),
                        group(lex, local, command_collection),
                        symbol(lex, local, command_collection),
                        implicit_group(lex, local, command_collection));

        // Handle commands that may change the state of the parser
        // ie: fontstyle changes.
        if node.is_none() {
            if let Some(mut nodes) = state_change(lex, local, command_collection)? {
                ml.append(&mut nodes);
                continue;
            }

            if let Some(mut new_input) = custom_command(lex, local, command_collection)? {
                let remaining_string = lex.move_to_end();
                new_input.push_str(remaining_string);
                let mut new_lexer = Lexer::new(&new_input);
                // TODO: no unwrap, deal with errors that occur in a custom command 
                let mut nodes = expression(&mut new_lexer, local, command_collection).unwrap();
                ml.append(&mut nodes);
                return Ok(ml);
            }


            // At this point, if the current `Token` is a Command,
            // then it must be an unrecognized Command.
            if let Token::Command(cmd) = lex.current() {
                return Err(ParseError::UnrecognizedCommand(cmd.into()));
            }
        }

        // Post-fix operators are handled as a special case since they need
        // access to the currently processed node.
        let node = postfix(lex, local, command_collection, node)?;

        // If at this point, we still haven't processed a node then we must have
        // an unrecognized symbol (perhaps from non-english, non-greek).
        // TODO: We should allow for more dialects.
        match node {
            Some(n) => ml.push(n),
            None => {
                match lex.current() {
                    Token::Symbol(c) => return Err(ParseError::UnrecognizedSymbol(c)),
                    _ => unreachable!(),
                }
            }
        }
    }
    Ok(ml)
}

/// Parses a custom command
fn custom_command<'a>(lex: &mut Lexer<'a>, _: Style, command_collection: &CommandCollection) -> ParseResult<'a, Option<String>> {
    if let Token::Command(name) = lex.current() {
        if let Some(command) = command_collection.query(name) {
            let n_args = command.n_args();
            let mut args = Vec::with_capacity(n_args);
            lex.next();
            for i in 0 .. n_args {
                lex.consume_whitespace();
                args.push(lex.group()?);
                if i + 1 != n_args 
                { lex.next(); }
            }
            lex.consume_whitespace();
            return Ok(Some(command.apply(&args)));
        }
    } 
    Ok(None)
}

pub fn expression_until<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, end: Token) -> ParseResult<'a, Vec<ParseNode>> {
    expression_until_opt(lex, local, command_collection, Some(end))
}

/// This function is served as an entry point to parsing input.
/// It can also be used to parse sub-expressions (or more formally known
/// as `mathlists`) which occur when parsing groups; ie: `{<expression>}`.
pub fn expression<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Vec<ParseNode>> {
    expression_until_opt(lex, local, command_collection, None)
}

/// Process post-fix operators.  Post-fix operators require the previous (optional)
/// node to process.  Post-fix processing only occurs while processing expressions
/// (for example, inside a group). In particular, `\hat 2^2` will parse as
/// `\hat{2}^2` and not `\hat{2^2}`.
fn postfix<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, mut prev: Option<ParseNode>) -> ParseResult<'a, Option<ParseNode>> {
    let mut superscript = None;
    let mut subscript = None;
    loop {
        lex.consume_whitespace();
        match lex.current() {
            Token::Symbol('_') => {
                lex.next();
                // If we already have a subscript, bail.
                if subscript.is_some() {
                    return Err(ParseError::ExcessiveSubscripts);
                }
                subscript = Some(required_argument(lex, local, command_collection)?);
            }
            Token::Symbol('^') => {
                lex.next();
                // If we already have a superscript, bail.
                if superscript.is_some() {
                    return Err(ParseError::ExcessiveSuperscripts);
                }
                superscript = Some(required_argument(lex, local, command_collection)?);
            }
            Token::Command("limits") => {
                lex.next();
                let op = prev.as_mut().ok_or(ParseError::LimitsMustFollowOperator)?;
                if let AtomType::Operator(_) = op.atom_type() {
                    op.set_atom_type(AtomType::Operator(true));
                } else {
                    return Err(ParseError::LimitsMustFollowOperator);
                }
            }
            Token::Command("nolimits") => {
                lex.next();
                let op = prev.as_mut().ok_or(ParseError::LimitsMustFollowOperator)?;
                if let AtomType::Operator(_) = op.atom_type() {
                    op.set_atom_type(AtomType::Operator(false));
                } else {
                    return Err(ParseError::LimitsMustFollowOperator);
                }
            }
            _ => break,
        }
    }

    if superscript.is_some() || subscript.is_some() {
        Ok(Some(ParseNode::Scripts(Scripts {
                                       base: prev.map(|b| Box::new(b)),
                                       superscript: superscript,
                                       subscript: subscript,
                                   })))
    } else {
        Ok(prev)
    }
}

/// Theses commands may change the state of the parser.  This includes
/// font style and weight changes.
pub fn state_change<'a>(lex: &mut Lexer<'a>, style: Style, command_collection : &CommandCollection) -> ParseResult<'a, Option<Vec<ParseNode>>> {
    use crate::font::Family;
    if let Token::Command(cmd) = lex.current() {
        let new_style = match cmd {
            "mathbf" => style.with_bold(),
            "mathit" => style.with_italics(),
            "mathrm" => style.with_family(Family::Roman),
            "mathscr" => style.with_family(Family::Script),
            "mathfrak" => style.with_family(Family::Fraktur),
            "mathbb" => style.with_family(Family::Blackboard),
            "mathsf" => style.with_family(Family::SansSerif),
            "mathtt" => style.with_family(Family::Monospace),
            "mathcal" => style.with_family(Family::Script),
            _ => return Ok(None),
        };

        lex.next();
        return required_argument(lex, new_style, command_collection).map(Some);
    }

    Ok(None)
}

/// Parse a TeX command. These commands define the "primitive" commands for our
/// typesetting system.  It tires to include a large portion of the TeX primitives,
/// along with the most useful primitives you find from amsmath and LaTeX.
/// This function can error while parsing macro arguments.
pub fn command<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Option<ParseNode>> {
    if let Token::Command(cmd) = lex.current() {
        match get_command(cmd) {
            Some(ref cmd) => {
                lex.next();
                cmd.parse(lex, local, command_collection).map(Some)
            }
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// Parse an implicit group.  For example `\left ... \right` is an implicit group.
/// This is one point where we will deviate from TeX a little bit.  We won't
/// characterize every command that will start a new implicit group
/// (for instance, `\frac`).
pub fn implicit_group<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Option<ParseNode>> {
    let token = lex.current();

    if token == Token::Command("left") {
        let mut delimiters = Vec::with_capacity(2);
        let mut inners     = Vec::with_capacity(1);

        lex.next();
        let left = symbol(lex, local, command_collection)?
            .ok_or(ParseError::ExpectedSymbol(lex.current().into()))?
            .expect_left()?;
        delimiters.push(left);

        loop {
            let inner = expression(lex, local, command_collection)?;
            inners.push(inner);

            if lex.current().expect_command("middle").is_ok() {
                lex.next();
                let middle = symbol(lex, local, command_collection)?
                    .ok_or(ParseError::ExpectedSymbol(lex.current().into()))?
                    .expect_middle()?;
                delimiters.push(middle);
            }
            else {
                lex.current().expect_command("right")?;
                lex.next();
                let right = symbol(lex, local, command_collection)?
                    .ok_or(ParseError::ExpectedSymbol(lex.current().into()))?
                    .expect_right()?;
                delimiters.push(right);
                break;
            }
        }

        let delimited = Delimited::new(delimiters, inners);
        Ok(Some(ParseNode::Delimited(delimited)))
    } else if token == Token::Command("begin") {
        lex.next();
        let env = required_group_with(lex, local, command_collection, environment_name)?;
        let node = env.parse(lex, local, command_collection)?;
        // Environment parsers are required to quit parsing on `\end`.
        // The current token should be this `\end`.
        lex.next();
        let end = required_group_with(lex, local, command_collection, environment_name)?;

        if env != end {
            return Err(ParseError::UnexpectedEndEnv { expected: env.name(), found: end.name() });
        }

        Ok(Some(node))
    } else {
        Ok(None)
    }
}

/// Parse a group.  Which is defined by `{<expression>}`.
/// This function will return `Ok(None)` if it does not find a `{`,
/// and will `Err` if it finds a `{` with no terminating `}`, or if
/// there is a syntax error from within `<expression>`.
pub fn group<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Option<ParseNode>> {
    if lex.current() == Token::Symbol('{') {
        lex.next();
        let inner = expression(lex, local, command_collection)?;
        lex.current().expect_symbol('}')?;
        lex.next();
        Ok(Some(ParseNode::Group(inner)))
    } else {
        Ok(None)
    }
}

/// Parse a symbol.  Symbols can be found from a TeX command (like `\infty`)
/// or from a character input.
///
/// Note: there are some `char` inputs that don't work here.  For instance,
/// `{` will not be recognized here and will therefore ParseResult in an `None`.
/// In particular a group should be parsed before a `symbol`.
pub fn symbol<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Option<ParseNode>> {
    match lex.current() {
        Token::Command(cs) => {
            if let Some(sym) = Symbol::from_name(cs) {
                lex.next();
                match sym.atom_type {
                    AtomType::Accent | AtomType::Over | AtomType::Under => {
                        let nucleus = required_argument(lex, local, command_collection)?;
                        Ok(Some(accent!(sym, nucleus)))
                    }
                    _ => {
                        let symbol_node = ParseNode::Symbol(Symbol { 
                            codepoint: style_symbol(sym.codepoint, local),
                            atom_type: sym.atom_type,
                        });
                        Ok(Some(symbol_node))
                    }
                }
            } else {
                Ok(None)
            }
        }
        Token::Symbol(c) => {
            match codepoint_atom_type(c) {
                None => Ok(None),
                Some(atom_type) => {
                    lex.next();
                    let symbol_node = ParseNode::Symbol(Symbol { 
                        codepoint: style_symbol(c, local),
                        atom_type,
                    });
                    Ok(Some(symbol_node))
                }
            }
        }
        _ => Ok(None),
    }
}

/// This method expects to parse a single macro argument.
/// A macro argument will consume a single token, unless the next token starts a group
/// `{<expression>}`. In which case, a `required_argument` will strip the surrounding
/// `{` `}`.  Provided that a custom parser is required, use `require_argument_with`.
pub fn required_argument<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection) -> ParseResult<'a, Vec<ParseNode>> {
    lex.consume_whitespace();
    let opt_node = alt!(group(lex, local, command_collection), command(lex, local, command_collection), symbol(lex, local, command_collection));

    match opt_node {
        Some(ParseNode::Group(inner)) => Ok(inner),
        Some(node) => Ok(vec![node]),
        _ => {
            // Check for a state change perhaps, otherwise we don't know.
            match state_change(lex, local, command_collection)? {
                Some(nodes) => Ok(nodes),
                _ => Err(ParseError::RequiredMacroArg),
            }
        }
    }
}

/// This method is similar to `required_argument`, but instead uses a custom parser.
/// For instance, `\color{#012345}{<expression>}` uses a custom parser to parse
/// the color token `#012345`.
pub fn required_argument_with<'a, F, O>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, f: F) -> ParseResult<'a, O>
    where F: FnOnce(&mut Lexer<'a>, Style, &CommandCollection) -> ParseResult<'a, O>
{
    lex.consume_whitespace();
    if lex.current() == Token::Symbol('{') {
        lex.next();
        lex.consume_whitespace();
        let parsed = f(lex, local, command_collection)?;
        lex.consume_whitespace();
        lex.current().expect_symbol('}')?;
        lex.next();
        Ok(parsed)
    } else {
        f(lex, local, command_collection)
    }
}

pub fn required_group_with<'a, F, O>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, f: F) -> ParseResult<'a, O>
    where F: FnOnce(&mut Lexer<'a>, Style, &CommandCollection) -> ParseResult<'a, O>
{
    lex.consume_whitespace();
    if lex.current() == Token::Symbol('{') {
        lex.next();
        lex.consume_whitespace();
        let parsed = f(lex, local, command_collection)?;
        lex.consume_whitespace();
        lex.current().expect_symbol('}')?;
        lex.next();
        Ok(parsed)
    } else {
        Err(ParseError::RequiredMacroArg)
    }
}

pub fn optional_argument_with<'a, F, O>(lex: &mut Lexer<'a>, local: Style, f: F) -> ParseResult<'a, Option<O>>
    where F: for<'l> FnOnce(&'l mut Lexer<'a>, Style) -> ParseResult<'a, Option<O>>
{
    lex.consume_whitespace();
    if lex.current() == Token::Symbol('[') {
        lex.next();
        lex.consume_whitespace();
        let parsed = f(lex, local)?;
        lex.consume_whitespace();
        lex.current().expect_symbol(']')?;
        lex.next();
        Ok(parsed)
    } else {
        Ok(None)
    }
}

/// This method expects that the current token has a given atom type.  This method
/// will frist strip all whitespaces first before inspecting the current token.
/// This function will Err if the expected symbol doesn't have the given type,
/// otherwise it will return `Ok`.
///
/// This function _will_ advance the lexer.
pub fn expect_type<'a>(lex: &mut Lexer<'a>, local: Style, command_collection : &CommandCollection, expected: AtomType) -> ParseResult<'a, Symbol> {
    lex.consume_whitespace();
    if let Some(ParseNode::Symbol(sym)) = symbol(lex, local, command_collection)? {
        if sym.atom_type == expected {
            Ok(sym)
        } else {
            Err(ParseError::ExpectedAtomType(expected, sym.atom_type))
        }
    } else {
        Err(ParseError::ExpectedSymbol(lex.current().into()))
    }
}

/// TODO: to be implemented
pub fn dimension<'a>(_: &mut Lexer<'a>, _: Style, _ : &CommandCollection) -> ParseResult<'a, Unit> {
    unimplemented!()
}

/// Match a valid color token. Valid color tokens are:
///  - Ascii name for css color (ie: `red`).
///  - #RRGGBB (ie: `#ff0000` for red)
///  - #RRGGBBAA (ie: `#00000000` for transparent)
///  - `transparent`

// TODO: implement parsing for other formats.
pub fn color<'a>(lex: &mut Lexer<'a>, _: Style, _ : &CommandCollection) -> ParseResult<'a, RGBA> {
    let color_str = lex.alphanumeric();
    let color = RGBA::from_name(&color_str)
        .ok_or_else(|| ParseError::UnrecognizedColor(color_str))?;
    Ok(color)
}

pub fn environment_name<'a>(lex: &mut Lexer<'a>, _: Style, _ : &CommandCollection) -> ParseResult<'a, Environment> {
    let name = lex.alphanumeric();
    Environment::try_from_str(&name)
        .ok_or(ParseError::UnrecognizedEnvironment(name))
}

/// This function is the API entry point for parsing tex.
pub fn parse(input: &str) -> ParseResult<Vec<ParseNode>> {
    parse_with_custom_commands(input, &CommandCollection::default())
}

/// This function is the API entry point for parsing tex.
pub fn parse_with_custom_commands<'a>(input: & 'a str, custom_commands : &CommandCollection) -> ParseResult<'a, Vec<ParseNode>> {
    let mut lexer = Lexer::new(input);
    let local = Style::new();
    let parse_result = expression(&mut lexer, local, custom_commands)?;
    if lexer.current() != Token::EOF {
        return Err(ParseError::ExpectedEof(lexer.current()));
    }

    Ok(parse_result)
}

/// Helper function for determining an atomtype based on a given codepoint.
/// This is primarily used for characters while processing, so may give false
/// negatives when used for other things.
fn codepoint_atom_type(codepoint: char) -> Option<AtomType> {
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

// --------------
//     TESTS
// --------------

#[cfg(test)]
mod tests {
    use crate::parser::{engine::parse, macros::{CustomCommand, CommandCollection}, ParseNode, nodes::PlainText};

    use super::parse_with_custom_commands;

    #[test]
    fn planck_h() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"h"]);
        display_errors!(errs);
    }

    #[test]
    fn ldots() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"\ldots",r"\vdots",r"\dots"]);
        display_errors!(errs);
    }

    #[test]
    fn fractions() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs, parse, [r"\frac\alpha\beta", r"\frac\int2"]);
        should_fail!(errs, parse, [r"\frac \left(1 + 2\right) 3"]);
        should_equate!(errs,
                       parse,
                       [(r"\frac12", r"\frac{1}{2}"),
                        (r"\frac \sqrt2 3", r"\frac{\sqrt2}{3}"),
                        (r"\frac \frac 1 2 3", r"\frac{\frac12}{3}"),
                        (r"\frac 1 \sqrt2", r"\frac{1}{\sqrt2}")]);
        display_errors!(errs);
    }

    #[test]
    fn radicals() {
        let mut errs: Vec<String> = Vec::new();
        // TODO: Add optional paramaters for radicals
        should_pass!(errs,
                     parse,
                     [r"\sqrt{x}",
                      r"\sqrt2",
                      r"\sqrt\alpha",
                      r"1^\sqrt2",
                      r"\alpha_\sqrt{1+2}",
                      r"\sqrt\sqrt2"]);
        should_fail!(errs, parse, [r"\sqrt", r"\sqrt_2", r"\sqrt^2"]);
        should_equate!(errs, parse, [(r"\sqrt2", r"\sqrt{2}")]);
        should_differ!(errs, parse, [(r"\sqrt2_3", r"\sqrt{2_3}")]);
        display_errors!(errs);
    }

    #[test]
    fn scripts() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs,
                     parse,
                     [r"1_2^3",
                      r"_1",
                      r"^\alpha",
                      r"_2^\alpha",
                      r"1_\frac12",
                      r"2^\alpha",
                      r"x_{1+2}",
                      r"x^{2+3}",
                      r"x^{1+2}_{2+3}",
                      r"a^{b^c}",
                      r"{a^b}^c",
                      r"a_{b^c}",
                      r"{a_b}^c",
                      r"a^{b_c}",
                      r"{a^b}_c",
                      r"a_{b_c}",
                      r"{a_b}_c"]);
        should_fail!(errs,
                     parse,
                     [r"1_", r"1^", r"x_x_x", r"x^x_x^x", r"x^x^x", r"x_x^x_x"]);
        should_equate!(errs,
                       parse,
                       [(r"x_\alpha^\beta", r"x^\beta_\alpha"), (r"_2^3", r"^3_2")]);
        display_errors!(errs);
    }

    #[test]
    fn delimited() {
        let mut errs: Vec<String> = Vec::new();
        should_pass!(errs,
                     parse,
                     [r"\left(\right)",
                      r"\left.\right)",
                      r"\left(\right.",
                      r"\left\vert\right)",
                      r"\left(\right\vert"]);
        should_fail!(errs,
                     parse,
                     [r"\left1\right)",
                      r"\left.\right1",
                      r"\left",
                      r"\left.{1 \right."]);
        display_errors!(errs);
    }


    #[test]
    fn array() {
        const CORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}\end{array}",
            r"\begin{array}{c}1\\2\end{array}",
            r"\begin{array}{c}1\\\end{array}",
        ];

        for formula in CORRECT_FORMULAS.iter().cloned() {
            eprintln!("correct: {}", formula);
            parse(formula).unwrap();
        }

        const INCORRECT_FORMULAS : &[&str] = &[
            r"\begin{array}{c}",
            r"\begin{array}1\\2\end{array}",
        ];

        for formula in INCORRECT_FORMULAS.iter().cloned() {
            eprintln!("incorrect: {}", formula);
            assert!(parse(formula).is_err());
        }
    }

    #[test]
    fn test_custom_command() {
        let command = CustomCommand::parse("#1 + #2").unwrap();
        let mut command_collection = CommandCollection::default();
        command_collection.insert("add", command).unwrap();

        let expected = parse("45 + 68");
        let got      = parse_with_custom_commands(r"\add{45}{68}", &command_collection);
        assert_eq!(expected, got);   

        // something before and after macros
        let expected = parse("145 + 681");
        let got      = parse_with_custom_commands(r"1\add{45}{68}1", &command_collection);
        assert_eq!(expected, got);   

        // commands in macro expansion
        let expected = parse(r"\frac{1}{2} + \frac{3}{4}");
        let got      = parse_with_custom_commands(r"\add{\frac{1}{2}}{\frac{3}{4}}", &command_collection);
        assert_eq!(expected, got);   

        // recursive macros
        let expected = parse("1 + 2 + 34");
        let got      = parse_with_custom_commands(r"\add{1}{\add{2}{3}}4", &command_collection);
        assert_eq!(expected, got);   
    }

    #[test]
    fn text_command() {
        let got = parse(r"\text{re + 43}").unwrap();
        let expected = vec![ParseNode::PlainText(PlainText {text : "re + 43".to_string()})];
        assert_eq!(expected, got);
    }
}
