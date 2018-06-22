//! Constructs an AST from the token stream

use super::data::*;
use super::tokenise::TokenIterator;

/// Possible parse errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The token stream was empty
    EmptyStream,
    /// Encountered the end of the token iterator before we thought we were done
    PartialStream,
    /// Syntax Error e.g. #a
    SyntaxError(String),
    /// Unimplemented in parser
    Unimplemented,
}

/// fully consume a token iterator to produce a list of scheme objects
pub fn parse_tokens(mut token_iter: TokenIterator) -> Result<Vec<SchemeObject>, ParseError> {
    let mut out = Vec::new();
    loop {
        let res = parse_token(&mut token_iter);

        match res {
            Ok(obj) => out.push(obj),
            Err(ParseError::EmptyStream) => break,
            Err(e) => return Err(e),
        };
    }

    if out.is_empty() {
        Err(ParseError::PartialStream)
    } else {
        Ok(out)
    }
}

/// Consume a stream of tokens  and output the first token generated
fn parse_token(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    let mode = match token_iter.next() {
        None => return Err(ParseError::EmptyStream),
        Some(s) => s,
    };

    // see tokenise.rs::is_special()
    match mode.as_str() {
        "(" => Err(ParseError::Unimplemented), // (...)
        ")" => Err(ParseError::SyntaxError(String::from("Unexpected ')'"))),
        "'" => Err(ParseError::Unimplemented), // quoted
        "#" => parse_token_hash(token_iter), // #t, #f, #(...)
        s => parse_token_other(s), // "string", symbol
    }
}

/// Parse a hash token
fn parse_token_hash(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    match parse_token(token_iter)? {
        SchemeObject::Symbol(s) => string_to_bool(s.as_str()), // #t, #f
        SchemeObject::Form(_) => Err(ParseError::Unimplemented), // #(...)
        obj => Err(ParseError::SyntaxError(format!(
            "Syntax error: # followed by {:?}",
            obj
        ))),
    }
}

/// "t" -> true, "f" -> false
/// Separate from `parse_token_hash` because match is awkward with String
fn string_to_bool(s: &str) -> Result<SchemeObject, ParseError> {
    match s {
        "t" => Ok(SchemeObject::Bool(true)),
        "f" => Ok(SchemeObject::Bool(false)),
        obj => Err(ParseError::SyntaxError(format!(
            "Syntax error: # followed by {:?}",
            obj
        ))),
    }
}

/// Parse a symbol or a string
fn parse_token_other(token: &str) -> Result<SchemeObject, ParseError> {
    // is it a string?
    if token.starts_with('"') {
        if token.len() > 1 {
            let end = token.len() - 1;
            Ok(SchemeObject::String(String::from(&token[1..end])))
        } else { // too short to have an ending '"'
            Err(ParseError::PartialStream)
        }
    } else if token.is_empty() {
        // empty symbol
        Err(ParseError::PartialStream)
    } else {
        // valid symbol
        Ok(SchemeObject::Symbol(String::from(token)))
    }
}

#[cfg(test)]
mod tests {
    use ast::ParseError;
    use data::SchemeObject;
    use tokenise::tokenise;
    use tokenise::TokenIterator;

    fn run_test(tv: &str, expected: Result<Vec<SchemeObject>, ParseError>) {
        let mut tv2 = tv.clone().chars();
        let tokens_iter = TokenIterator::new(&mut tv2);
        let tokens = tokenise(&mut tv.chars());

        let res = super::parse_tokens(tokens_iter);

        assert_eq!(res, expected, "parse_tokens({:?})", tokens);
    }

    #[test]
    fn bool() {
        let expected = vec![SchemeObject::Bool(true), SchemeObject::Bool(false)];
        run_test("#t #f", Ok(expected))
    }

    #[test]
    fn bad_hash() {
        let expected =
            ParseError::SyntaxError(String::from("Syntax error: # followed by \"error\""));
        run_test("#error", Err(expected))
    }

    #[test]
    fn symbol() {
        run_test(
            "a_symbol",
            Ok(vec![SchemeObject::Symbol(String::from("a_symbol"))]),
        )
    }

    #[test]
    fn string() {
        let expected = vec![SchemeObject::String(String::from("I am a string"))];
        run_test("\"I am a string\"", Ok(expected))
    }

    #[test]
    fn empty_string() {
        let expected = vec![SchemeObject::String(String::from(""))];
        run_test("\"\"", Ok(expected))
    }

    #[test]
    fn stray_bracket() {
        let expected = ParseError::SyntaxError(String::from("Unexpected ')'"));
        run_test(")", Err(expected))
    }
}
