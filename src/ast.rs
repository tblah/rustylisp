//! Constructs an Abstract Syntax Tree from the token stream

use super::data::*;
use super::tokenise::TokenIterator;
use std::collections::LinkedList;
use std::iter::FromIterator;

/// Possible parse errors
#[derive(Debug, PartialEq)]
pub enum ParseError {
    /// The token stream was empty
    EmptyStream,
    /// Encountered the end of the token iterator before we thought we were done
    PartialStream,
    /// Found a ')'
    ClosingBracket,
    /// Syntax Error e.g. #a
    SyntaxError(String),
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

/// Consume a stream of tokens and output the first token generated
/// This function just dispatches to the right helper function
fn parse_token(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    let mode = match token_iter.next() {
        None => return Err(ParseError::EmptyStream),
        Some(s) => s,
    };

    // dispatches to helper functions by the starting symbol
    // see tokenise.rs::is_special()
    match mode.as_str() {
        "(" => parse_token_form(token_iter), // (...)
        ")" => Err(ParseError::ClosingBracket),
        "'" => parse_token_quoted(token_iter), // quoted
        "#" => parse_token_hash(token_iter),   // #t, #f, #(...)
        s => parse_token_other(s),             // "string", symbol
    }
}

/// Recursively parses a form (...)
fn parse_token_form(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    let mut lst = LinkedList::new();

    loop { // parse each item in this list
        let obj = match parse_token(token_iter) {
            Ok(o) => o,
            Err(ParseError::ClosingBracket) => break,
            Err(e) => return Err(e),
        };

        lst.push_back(obj);
    }

    if lst.is_empty() {
        Err(ParseError::SyntaxError(String::from("Empty form: '()'")))
    } else {
        Ok(SchemeObject::CodeList(lst))
    }
}

/// Parse a hash token
fn parse_token_hash(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    match parse_token(token_iter)? {
        SchemeObject::Symbol(s) => string_to_bool(s.as_str()), // #t, #f
        SchemeObject::CodeList(l) => Ok(SchemeObject::Vector(Vec::from_iter(l))), // #(...)
        obj => Err(ParseError::SyntaxError(format!(
            "Syntax error: # followed by {:?}",
            obj
        ))),
    }
}

/// Parse a quoted token
fn parse_token_quoted(token_iter: &mut TokenIterator) -> Result<SchemeObject, ParseError> {
    match parse_token(token_iter)? {
        SchemeObject::CodeList(lst) => Ok(SchemeObject::QuotedList(lst)),
        obj => Ok(obj),
    }
}

/// helper function for `parse_token_hash`
/// "t" -> true, "f" -> false
/// Separate from `parse_token_hash` because `match` is awkward with `String`
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

/// Parse things which aren't preceded by some kind of modifier token
fn parse_token_other(token: &str) -> Result<SchemeObject, ParseError> {
    // is it a string?
    if token.starts_with('"') {
        if token.len() > 1 {
            let end = token.len() - 1; // remove closing '"'
            Ok(SchemeObject::String(String::from(&token[1..end])))
        } else {
            // too short to have an ending '"'
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
    use data::*;
    use std::collections::LinkedList;
    use std::iter::FromIterator;
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
    fn simple_form() {
        let mut lst = LinkedList::new();
        lst.push_back(SchemeObject::Symbol(String::from("one")));
        lst.push_back(SchemeObject::Symbol(String::from("two")));

        let expected = vec![SchemeObject::CodeList(lst)];
        run_test("(one two)", Ok(expected))
    }

    #[test]
    fn nested_form() {
        let mut inner_lst = LinkedList::new();
        inner_lst.push_back(SchemeObject::Symbol(String::from("one")));
        inner_lst.push_back(SchemeObject::Symbol(String::from("two")));
        let inner_obj = SchemeObject::CodeList(inner_lst);

        let mut outer_lst = LinkedList::new();
        outer_lst.push_back(inner_obj);
        outer_lst.push_back(SchemeObject::String(String::from("three")));
        let outer_obj = SchemeObject::CodeList(outer_lst);

        let expected = vec![outer_obj];
        run_test("((one two) \"three\")", Ok(expected))
    }

    #[test]
    fn empty_form() {
        let expected = ParseError::SyntaxError(String::from("Empty form: '()'"));
        run_test("()", Err(expected))
    }

    #[test]
    fn vector() {
        let expected = vec![SchemeObject::Vector(vec![SchemeObject::Bool(true)])];
        run_test("#(#t)", Ok(expected))
    }

    #[test]
    fn quotes() {
        let scm = "'#t 'symbol '\"string\" '(one two) '#(one two)";
        let v = vec!(SchemeObject::Symbol(String::from("one")),
                     SchemeObject::Symbol(String::from("two")));
        let l = LinkedList::from_iter(v.clone());

        let mut expected = Vec::new();
        expected.push(SchemeObject::Bool(true));
        expected.push(SchemeObject::Symbol(String::from("symbol")));
        expected.push(SchemeObject::String(String::from("string")));
        expected.push(SchemeObject::QuotedList(l));
        expected.push(SchemeObject::Vector(v));

        run_test(scm, Ok(expected));
    }
}
