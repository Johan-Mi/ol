use crate::{
    expression::ExpressionOf,
    program::{Class, ClassMethod, Program},
    value::Value,
};
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while_m_n},
    character::complete::{
        alpha1, alphanumeric1, char, digit1, multispace1, none_of, one_of,
        satisfy,
    },
    combinator::{
        all_consuming, eof, map_opt, map_res, not, opt, peek, recognize, value,
        verify,
    },
    error::ParseError,
    multi::{count, many0, many0_count, many1_count, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    Parser,
};
use std::borrow::Cow;

type Input<'a> = &'a str;
type IResult<'a, T> = nom::IResult<Input<'a>, T>;

type Expression = ExpressionOf<String, String>;

pub fn program(input: Input) -> IResult<Program> {
    all_consuming(delimited(ws, separated_list0(ws, class), ws))
        .map(|classes| Program { classes })
        .parse(input)
}

fn class(input: Input) -> IResult<Class> {
    pair(
        preceded(pair(keyword("class"), ws), identifier),
        delimited(
            pair(ws, char('{')),
            many0(preceded(ws, class_method_definition)),
            pair(ws, char('}')),
        ),
    )
    .map(|(name, methods)| Class { name, methods })
    .parse(input)
}

fn class_method_definition(input: Input) -> IResult<ClassMethod> {
    delimited(
        pair(keyword("def"), ws),
        tuple((
            identifier,
            many0(preceded(ws, identifier)),
            preceded(tuple((ws, char('='), ws)), expression),
        )),
        pair(ws, char(';')),
    )
    .map(|(name, parameters, body)| ClassMethod {
        name,
        parameters,
        body,
    })
    .parse(input)
}

fn expression(input: Input) -> IResult<Expression> {
    alt((method_call, expression_but_not_method_call))(input)
}

// Without this, method calls would become right-associative, e.g. `f x y` would
// be parsed as `f (x y)` since the first argument would greedily parse itself
// as a method call as well.
fn expression_but_not_method_call(input: Input) -> IResult<Expression> {
    let unit_literal = value(
        Expression::Literal(Value::Unit),
        tuple((char('('), ws, char(')'))),
    );

    let r#true = value(Expression::Literal(Value::Bool(true)), keyword("true"));
    let r#false =
        value(Expression::Literal(Value::Bool(false)), keyword("false"));

    let local_variable = identifier.map(|ident| ExpressionOf::LocalVariable {
        name_or_de_brujin_index: ident,
    });

    alt((
        unit_literal,
        parenthesized_expression,
        r#true,
        r#false,
        block,
        string_literal.map(Value::String).map(Expression::Literal),
        i32_literal.map(Value::I32).map(Expression::Literal),
        let_in,
        if_then_else,
        local_variable,
    ))(input)
}

fn parenthesized_expression(input: Input) -> IResult<Expression> {
    delimited(pair(char('('), ws), expression, pair(ws, char(')')))(input)
}

fn block(input: Input) -> IResult<Expression> {
    delimited(
        char('{'),
        separated_list0(char(';'), preceded(ws, expression)),
        pair(ws, char('}')),
    )
    .map(Expression::Do)
    .parse(input)
}

fn i32_literal(input: Input) -> IResult<i32> {
    recognize(pair(
        opt(one_of("+-")),
        many1_count(pair(digit1, many0_count(char('_')))),
    ))
    .map(|s: Input| s.replace('_', "").parse().unwrap())
    .parse(input)
}

fn let_in(input: Input) -> IResult<Expression> {
    tuple((
        preceded(pair(keyword("let"), ws), identifier),
        preceded(tuple((ws, char('='), ws)), expression.map(Box::new)),
        preceded(tuple((ws, keyword("in"), ws)), expression.map(Box::new)),
    ))
    .map(|(name, bound, body)| Expression::LetIn { name, bound, body })
    .parse(input)
}

fn if_then_else(input: Input) -> IResult<Expression> {
    tuple((
        preceded(
            pair(keyword("if"), ws),
            parenthesized_expression.map(Box::new),
        ),
        preceded(ws, block.map(Box::new)),
        preceded(tuple((ws, keyword("else"), ws)), block.map(Box::new)),
    ))
    .map(|(condition, if_true, if_false)| Expression::IfThenElse {
        condition,
        if_true,
        if_false,
    })
    .parse(input)
}

fn method_call(input: Input) -> IResult<Expression> {
    tuple((
        identifier,
        preceded(ws, expression_but_not_method_call.map(Box::new)),
        many0(preceded(ws, expression_but_not_method_call)),
    ))
    .map(|(name, this, arguments)| Expression::MethodCall {
        name,
        this,
        arguments,
    })
    .parse(input)
}

fn identifier_or_keyword(input: Input) -> IResult<&str> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    ))(input)
}

fn identifier(input: Input) -> IResult<String> {
    verify(identifier_or_keyword, |ident| !is_keyword(ident))
        .map(ToOwned::to_owned)
        .parse(input)
}

fn keyword(word: &'static str) -> impl FnMut(Input) -> IResult<()> {
    move |input| {
        unit(verify(identifier_or_keyword, |ident: &str| ident == word))(input)
    }
}

fn is_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "class" | "def" | "true" | "false" | "if" | "else" | "let" | "in"
    )
}

fn hex_digit(input: Input) -> IResult<char> {
    satisfy(|c| c.is_ascii_hexdigit())(input)
}

fn string_literal(input: Input) -> IResult<String> {
    let normal = is_not("\"\\\n").map(Cow::Borrowed);
    let null = value(
        Cow::Borrowed("\0"),
        terminated(char('0'), not(peek(digit1))),
    );
    let character_escape_sequence = alt((
        value("\"", char('"')),
        value("'", char('\'')),
        value("\\", char('\\')),
        value("\n", char('n')),
        value("\t", char('t')),
        value("\r", char('r')),
        value("\x08", char('b')),
        value("\x0c", char('f')),
        value("\x11", char('v')),
    ))
    .map(Cow::Borrowed);

    let hex_escape_sequence =
        preceded(char('x'), recognize(count(hex_digit, 2)));
    let hex4digits = recognize(count(hex_digit, 4));
    let bracketed_unicode = delimited(
        char('{'),
        take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit()),
        char('}'),
    );
    let unicode_escape_sequence =
        preceded(char('u'), alt((hex4digits, bracketed_unicode)));
    let escape_sequence = preceded(
        char('\\'),
        alt((
            character_escape_sequence,
            null,
            map_opt(
                map_res(
                    alt((hex_escape_sequence, unicode_escape_sequence)),
                    |digits| u32::from_str_radix(digits, 16),
                ),
                |c| char::from_u32(c).map(String::from).map(Cow::Owned),
            ),
        )),
    );
    let string_char = alt((normal, escape_sequence));

    delimited(char('"'), many0(string_char), char('"'))
        .map(|strs| strs.concat())
        .parse(input)
}

fn eol_comment(input: Input) -> IResult<()> {
    unit(pair(
        tag("//"),
        opt(alt((
            unit(is_not("\n\r")),
            unit(pair(many0_count(none_of("\n\r")), eof)),
        ))),
    ))(input)
}

fn ws(input: Input) -> IResult<()> {
    unit(many0_count(alt((unit(multispace1), eol_comment))))(input)
}

pub fn unit<I, O, E: ParseError<I>, F>(
    parser: F,
) -> impl FnMut(I) -> nom::IResult<I, (), E>
where
    F: Parser<I, O, E>,
{
    value((), parser)
}
