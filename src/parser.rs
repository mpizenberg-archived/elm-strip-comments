// Most of this file comes from mpizenberg/elm-test-rs/src/parser.rs

use nom::{
    branch::*, bytes::complete::*, character::complete::*, combinator::*, multi::*, sequence::*, *,
};

pub fn remove_comments(input: &str) -> IResult<&str, String> {
    fold_many0(
        code_element,
        String::new(),
        |mut s: String, item: Element| match item {
            Element::TextLiteral(content) => {
                s.push_str(content);
                s
            }
            Element::Code(content) => {
                s.push_str(content);
                s
            }
            Element::Comment => s,
        },
    )(input)
}

#[derive(Debug)]
enum Element<'a> {
    Comment,
    TextLiteral(&'a str),
    Code(&'a str),
}

fn code_element(input: &str) -> IResult<&str, Element> {
    let forbidden_chars = "'-\"{";
    alt((
        map(comment, |_| Element::Comment),
        map(text_literal, Element::TextLiteral),
        map(move |i| one_of_as_str(forbidden_chars, i), Element::Code),
        map(
            take_till1(move |c| forbidden_chars.contains(c)),
            Element::Code,
        ),
    ))(input)
}

fn one_of_as_str<'a, 'b>(options: &'a str, input: &'b str) -> IResult<&'b str, &'b str> {
    let (_, _) = one_of(options)(input)?;
    take(1usize)(input)
}

// Handling comments

fn comment(input: &str) -> IResult<&str, &str> {
    alt((block_comment, line_comment))(input)
}

fn line_comment(input: &str) -> IResult<&str, &str> {
    preceded(tag("--"), not_line_ending)(input)
}

fn block_comment(input: &str) -> IResult<&str, &str> {
    within_recursive("{-", "-}", input)
}

// Warning, the start or end pattern must not contain the other one.
#[allow(clippy::manual_strip)]
fn within_recursive<'a>(start: &'a str, end: &'a str, input: &'a str) -> IResult<&'a str, &'a str> {
    let (input, _) = tag(start)(input)?;
    let start_count = start.chars().count();
    let end_count = end.chars().count();
    let mut open_count = 1;
    let mut char_indices = input.char_indices();
    loop {
        match char_indices.next() {
            None => return tag("x")(""), // force fail
            Some((index, _)) => {
                let rest = &input[index..];
                if rest.starts_with(start) {
                    open_count += 1;
                    for _ in 1..start_count {
                        char_indices.next();
                    }
                } else if rest.starts_with(end) {
                    open_count -= 1;
                    if open_count == 0 {
                        return Ok((&rest[end.len()..], &input[..index]));
                    }
                    for _ in 1..end_count {
                        char_indices.next();
                    }
                }
            }
        }
    }
}

// ------------------
// Parse strings

fn text_literal(input: &str) -> IResult<&str, &str> {
    alt((char_literal, multiline_string_literal, string_literal))(input)
}

fn char_literal(input: &str) -> IResult<&str, &str> {
    let (_, content) = delimited(
        char('\''),
        escaped(take_till1(|c| c == '\\' || c == '\''), '\\', anychar),
        char('\''),
    )(input)?;
    let content_size = content.len() + 2;
    Ok((&input[content_size..], &input[..content_size]))
}

fn string_literal(input: &str) -> IResult<&str, &str> {
    let (_, content) = delimited(
        char('"'),
        alt((
            escaped(take_till1(|c| c == '\\' || c == '"'), '\\', anychar),
            success(""),
        )),
        char('"'),
    )(input)?;
    let content_size = content.len() + 2;
    Ok((&input[content_size..], &input[..content_size]))
}

fn multiline_string_literal(input: &str) -> IResult<&str, &str> {
    let (_, content) = delimited(
        tag("\"\"\""),
        alt((
            escaped(take_till_escape_or_string_end, '\\', anychar),
            success(""),
        )),
        tag("\"\"\""),
    )(input)?;
    let content_size = content.len() + 6;
    Ok((&input[content_size..], &input[..content_size]))
}

fn take_till_escape_or_string_end(input: &str) -> IResult<&str, &str> {
    for (index, c) in input.char_indices() {
        let rest = &input[index..];
        if c == '\\' || rest.starts_with("\"\"\"") {
            if index == 0 {
                return tag("x")(rest); // for fail
            } else {
                return Ok((rest, &input[..index]));
            }
        }
    }
    tag("x")("") // fail if we reach end of string
}

#[cfg(test)]
mod nom_tests {
    #[test]
    fn char_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::char_literal(input), res);
        assert!(super::char_literal("'").is_err());
        assert!(super::char_literal("''").is_err());
        asrt_eq(r#"'c'a"#, Ok(("a", "'c'")));
        asrt_eq(r#"'\\'a"#, Ok(("a", "'\\\\'")));
        asrt_eq(r#"'\''a"#, Ok(("a", "'\\''")));
        asrt_eq(r#"'\n'a"#, Ok(("a", "'\\n'")));
        asrt_eq(r#"'\r'a"#, Ok(("a", "'\\r'")));
        asrt_eq(r#"'✔'a"#, Ok(("a", "'✔'")));
    }
    #[test]
    fn string_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::string_literal(input), res);
        assert!(super::string_literal(r#"""#).is_err());
        asrt_eq(r#""toto"a"#, Ok(("a", r#""toto""#)));
        asrt_eq(r#""to\"to"a"#, Ok(("a", r#""to\"to""#)));
        asrt_eq(r#""\"toto"a"#, Ok(("a", r#""\"toto""#)));
        asrt_eq(r#""\""a"#, Ok(("a", r#""\"""#)));
        asrt_eq(r#"""a"#, Ok(("a", r#""""#)));
        asrt_eq("\"to\nto\"a", Ok(("a", "\"to\nto\"")));
        asrt_eq(r#""to\nto"a"#, Ok(("a", r#""to\nto""#)));
        asrt_eq(r#""\""a"#, Ok(("a", r#""\"""#)));
        asrt_eq(r#""✔"a"#, Ok(("a", r#""✔""#)));
    }
    #[test]
    fn multiline_string_literal() {
        let asrt_eq = |input: &str, res| assert_eq!(super::multiline_string_literal(input), res);
        assert!(super::multiline_string_literal(r#"""" "#).is_err());
        asrt_eq(r#"""""""a"#, Ok(("a", r#""""""""#)));
        asrt_eq(r#""""toto"""a"#, Ok(("a", r#""""toto""""#)));
        asrt_eq(r#""""to"to"""a"#, Ok(("a", r#""""to"to""""#)));
        asrt_eq(r#""""to""to"""a"#, Ok(("a", r#""""to""to""""#)));
        asrt_eq(r#""""" """a"#, Ok(("a", r#""""" """"#)));
        asrt_eq(r#""""to\"""to"""a"#, Ok(("a", r#""""to\"""to""""#)));
        asrt_eq(r#""""to\"""\"to"""a"#, Ok(("a", r#""""to\"""\"to""""#)));
        asrt_eq(r#""""✔"""a"#, Ok(("a", r#""""✔""""#)));
    }
    #[test]
    fn line_comment() {
        assert!(super::line_comment("a").is_err());
        assert!(super::line_comment("-").is_err());
        assert_eq!(super::line_comment("-- hoho \n"), Ok(("\n", " hoho ")));
        assert_eq!(super::line_comment("-- hoho"), Ok(("", " hoho")));
        assert_eq!(super::line_comment("-- ✔"), Ok(("", " ✔")));
    }
    #[test]
    fn block_comment() {
        let asrt_eq = |input: &str, res| assert_eq!(super::block_comment(input), res);
        assert!(super::block_comment("").is_err());
        assert!(super::block_comment("a").is_err());
        assert!(super::block_comment("{-").is_err());
        asrt_eq("{- hoho -}", Ok(("", " hoho ")));
        asrt_eq("{- before {- hoho -}-}", Ok(("", " before {- hoho -}")));
        asrt_eq("{-{- hoho -} after -}", Ok(("", "{- hoho -} after ")));
        asrt_eq(
            "{-{- first -} between {- second -}-}",
            Ok(("", "{- first -} between {- second -}")),
        );
        asrt_eq("{- ✔ -}", Ok(("", " ✔ ")));
    }
}
