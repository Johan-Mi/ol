use crate::{
    program::{Class, ClassMethod, Program},
    value::Value,
};
use std::borrow::Cow;
use winnow::{
    ascii::{alpha1, alphanumeric1, digit1, multispace1},
    combinator::{
        alt, count, delimited, not, opt, preceded, repeat0, repeat1,
        separated0, terminated,
    },
    error::Error,
    stream::AsChar,
    token::{one_of, take_till0, take_till1, take_while, take_while0},
    Parser,
};

type Input<'a> = &'a str;
type IResult<'a, T> = winnow::IResult<Input<'a>, T>;

type Expression = crate::expression::Of<String, String>;

pub fn program(input: Input) -> Result<Program, Error<String>> {
    delimited(ws, separated0(class, ws), ws)
        .map(|classes| Program { classes })
        .parse(input)
        .map_err(Error::into_owned)
}

fn class(input: Input) -> IResult<Class> {
    (
        preceded((keyword("class"), ws), identifier),
        delimited(
            (ws, '{'),
            repeat0(preceded(ws, class_method_definition)),
            (ws, '}'),
        ),
    )
        .map(|(name, methods)| Class { name, methods })
        .parse_next(input)
}

fn class_method_definition(input: Input) -> IResult<ClassMethod> {
    delimited(
        (keyword("def"), ws),
        (
            identifier,
            repeat0(preceded(ws, identifier)),
            preceded((ws, '=', ws), expression),
        ),
        (ws, ';'),
    )
    .map(|(name, parameters, body)| ClassMethod {
        name,
        parameters,
        body,
    })
    .parse_next(input)
}

fn expression(input: Input) -> IResult<Expression> {
    alt((method_call, expression_but_not_method_call)).parse_next(input)
}

// Without this, method calls would become right-associative, e.g. `f x y` would
// be parsed as `f (x y)` since the first argument would greedily parse itself
// as a method call as well.
fn expression_but_not_method_call(input: Input) -> IResult<Expression> {
    let unit_literal = ('(', ws, ')').value(Expression::Literal(Value::Unit));

    let r#true = keyword("true").value(Expression::Literal(Value::Bool(true)));
    let r#false =
        keyword("false").value(Expression::Literal(Value::Bool(false)));

    let local_variable = identifier.map(|ident| Expression::LocalVariable {
        name_or_de_bruijn_index: ident,
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
    ))
    .parse_next(input)
}

fn parenthesized_expression(input: Input) -> IResult<Expression> {
    delimited(('(', ws), expression, (ws, ')')).parse_next(input)
}

fn block(input: Input) -> IResult<Expression> {
    delimited('{', separated0(preceded(ws, expression), ';'), (ws, '}'))
        .map(Expression::Do)
        .parse_next(input)
}

fn i32_literal(input: Input) -> IResult<i32> {
    (
        opt(one_of("+-")),
        repeat1::<_, _, (), _, _>((digit1, take_while0('_'))),
    )
        .recognize()
        .try_map(|s: Input| s.replace('_', "").parse())
        .parse_next(input)
}

fn let_in(input: Input) -> IResult<Expression> {
    (
        preceded((keyword("let"), ws), identifier),
        preceded((ws, '=', ws), expression.map(Box::new)),
        preceded((ws, keyword("in"), ws), expression.map(Box::new)),
    )
        .map(|(name, bound, body)| Expression::LetIn { name, bound, body })
        .parse_next(input)
}

fn if_then_else(input: Input) -> IResult<Expression> {
    (
        preceded((keyword("if"), ws), parenthesized_expression.map(Box::new)),
        preceded(ws, block.map(Box::new)),
        preceded((ws, keyword("else"), ws), block.map(Box::new)),
    )
        .map(|(condition, if_true, if_false)| Expression::IfThenElse {
            condition,
            if_true,
            if_false,
        })
        .parse_next(input)
}

fn method_call(input: Input) -> IResult<Expression> {
    (
        identifier,
        preceded(ws, expression_but_not_method_call.map(Box::new)),
        repeat0(preceded(ws, expression_but_not_method_call)),
    )
        .map(|(name, this, arguments)| Expression::MethodCall {
            name,
            this,
            arguments,
        })
        .parse_next(input)
}

fn identifier_or_keyword(input: Input) -> IResult<&str> {
    (
        alt((alpha1, "_")),
        repeat0::<_, _, (), _, _>(alt((alphanumeric1, "_"))),
    )
        .recognize()
        .parse_next(input)
}

fn identifier(input: Input) -> IResult<String> {
    identifier_or_keyword
        .verify(|ident| !is_keyword(ident))
        .map(ToOwned::to_owned)
        .parse_next(input)
}

fn keyword<'a>(
    word: &'static str,
) -> impl Parser<Input<'a>, (), Error<Input<'a>>> {
    identifier_or_keyword
        .verify(move |ident: &str| ident == word)
        .void()
}

fn is_keyword(ident: &str) -> bool {
    matches!(
        ident,
        "class" | "def" | "true" | "false" | "if" | "else" | "let" | "in"
    )
}

fn hex_digit(input: Input) -> IResult<char> {
    one_of(AsChar::is_hex_digit).parse_next(input)
}

fn string_literal(input: Input) -> IResult<String> {
    let normal = take_till1("\"\\\n").map(Cow::Borrowed);
    let null = terminated('0', not(digit1)).value(Cow::Borrowed("\0"));
    let character_escape_sequence = alt((
        '"'.value("\""),
        '\''.value("'"),
        '\\'.value("\\"),
        'n'.value("\n"),
        't'.value("\t"),
        'r'.value("\r"),
        'b'.value("\x08"),
        'f'.value("\x0c"),
        'v'.value("\x11"),
    ))
    .map(Cow::Borrowed);

    let hex_escape_sequence =
        preceded('x', count::<_, _, (), _, _>(hex_digit, 2).recognize());
    let hex4digits = count::<_, _, (), _, _>(hex_digit, 4).recognize();
    let bracketed_unicode =
        delimited('{', take_while(1..=6, |c: char| c.is_ascii_hexdigit()), '}');
    let unicode_escape_sequence =
        preceded('u', alt((hex4digits, bracketed_unicode)));
    let escape_sequence = preceded(
        '\\',
        alt((
            character_escape_sequence,
            null,
            alt((hex_escape_sequence, unicode_escape_sequence))
                .try_map(|digits| u32::from_str_radix(digits, 16))
                .verify_map(|c| {
                    char::from_u32(c).map(String::from).map(Cow::Owned)
                }),
        )),
    );
    let string_char = alt((normal, escape_sequence));

    delimited('"', repeat0(string_char), '"')
        .map(|strs: Vec<_>| strs.concat())
        .parse_next(input)
}

fn eol_comment(input: Input) -> IResult<()> {
    ("//", take_till0('\n').void()).void().parse_next(input)
}

fn ws(input: Input) -> IResult<()> {
    repeat0(alt((multispace1.void(), eol_comment))).parse_next(input)
}
