//! Constructs an Abstract Syntax Tree from the token stream

use super::data::scm_static::*;
use super::tokenise::TokenIterator;
use std::collections::LinkedList;
use std::iter::FromIterator;
use ParseError;

/// Iterator over `SchemeObject`s
pub struct ObjectIterator<T>
where
    T: Iterator<Item = char>,
{
    source: TokenIterator<T>,
}

impl<T> ObjectIterator<T>
where
    T: Iterator<Item = char>,
{
    /// Create a new instance of ObjectIterator
    pub fn new(source: TokenIterator<T>) -> Self {
        Self { source }
    }
}

/// Create this from anything from which we can create a `TokenIterator`
impl<T, I> From<T> for ObjectIterator<I>
where
    T: Into<TokenIterator<I>>,
    I: Iterator<Item = char>,
{
    fn from(x: T) -> Self {
        Self::new(x.into())
    }
}

/// controls what kind of error we throw when the end of the source iterator is encountered
enum ParseMode {
    TokenOptional,
    TokenRequired,
}
use self::ParseMode::{TokenOptional, TokenRequired};

impl<T> Iterator for ObjectIterator<T>
where
    T: Iterator<Item = char>,
{
    type Item = Result<SchemeObject, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match parse_token(&mut self.source, &TokenOptional) {
            Err(ParseError::EmptyStream) => None,
            res => Some(res),
        }
    }
}

/// fully consume a token iterator to produce a Vector of scheme objects
pub fn parse_tokens<T, I>(to_object_iter: I) -> Result<Vec<SchemeObject>, ParseError>
where
    T: Iterator<Item = char>,
    I: Into<ObjectIterator<T>>,
{
    to_object_iter.into().collect()
}

/// Consume a stream of tokens and output the first token generated
/// This function just dispatches to the right helper function
fn parse_token<T>(
    token_iter: &mut TokenIterator<T>,
    mode: &ParseMode,
) -> Result<SchemeObject, ParseError>
where
    T: Iterator<Item = char>,
{
    let mode = match token_iter.next() {
        None => {
            return {
                match mode {
                    TokenOptional => Err(ParseError::EmptyStream),
                    TokenRequired => Err(ParseError::MissingToken),
                }
            }
        }
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
fn parse_token_form<T>(token_iter: &mut TokenIterator<T>) -> Result<SchemeObject, ParseError>
where
    T: Iterator<Item = char>,
{
    let mut lst = LinkedList::new();

    loop {
        // parse each item in this list
        let obj = match parse_token(token_iter, &TokenRequired) {
            Ok(o) => o,
            Err(ParseError::ClosingBracket) => break,
            Err(e) => return Err(e),
        };

        lst.push_back(obj);
    }

    Ok(SchemeObject::CodeList(lst))
}

/// Parse a hash token
fn parse_token_hash<T>(token_iter: &mut TokenIterator<T>) -> Result<SchemeObject, ParseError>
where
    T: Iterator<Item = char>,
{
    match parse_token(token_iter, &TokenRequired)? {
        SchemeObject::Symbol(s) => string_to_bool(s.as_str()), // #t, #f
        SchemeObject::CodeList(l) => Ok(SchemeObject::Vector(Vec::from_iter(l))), // #(...)
        obj => Err(ParseError::from(format!(
            "Syntax error: # followed by {:?}",
            obj
        ))),
    }
}

/// Parse a quoted token
fn parse_token_quoted<T>(token_iter: &mut TokenIterator<T>) -> Result<SchemeObject, ParseError>
where
    T: Iterator<Item = char>,
{
    match parse_token(token_iter, &TokenRequired)? {
        SchemeObject::CodeList(lst) => Ok(SchemeObject::QuotedList(lst)),
        obj => Ok(obj),
    }
}

/// helper function for `parse_token_hash`
/// "t" -> true, "f" -> false
/// Separate from `parse_token_hash` because `match` is awkward with `String`
fn string_to_bool(s: &str) -> Result<SchemeObject, ParseError> {
    match s {
        "t" => Ok(SchemeObject::from(true)),
        "f" => Ok(SchemeObject::from(false)),
        obj => Err(ParseError::from(format!(
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
            Ok(SchemeObject::from(&token[1..end]))
        } else {
            // too short to have an ending '"'
            Err(ParseError::PartialStream)
        }
    } else if token.is_empty() {
        // empty symbol
        Err(ParseError::PartialStream)
    } else {
        // valid symbol
        Ok(SchemeObject::sym_from(token))
    }
}

#[cfg(test)]
mod tests {
    use ast::ParseError;
    use data::scm_static::*;
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
        let expected = vec![SchemeObject::from(true), SchemeObject::from(false)];
        run_test("#t #f", Ok(expected))
    }

    #[test]
    fn bad_hash() {
        let expected = ParseError::from("Syntax error: # followed by \"error\"");
        run_test("#error", Err(expected))
    }

    #[test]
    fn symbol() {
        run_test("a_symbol", Ok(vec![SchemeObject::sym_from("a_symbol")]))
    }

    #[test]
    fn string() {
        let expected = vec![SchemeObject::from("I am a string")];
        run_test("\"I am a string\"", Ok(expected))
    }

    #[test]
    fn empty_string() {
        let expected = vec![SchemeObject::from("")];
        run_test("\"\"", Ok(expected))
    }

    #[test]
    fn simple_form() {
        let mut lst = LinkedList::new();
        lst.push_back(SchemeObject::sym_from("one"));
        lst.push_back(SchemeObject::sym_from("two"));

        let expected = vec![SchemeObject::CodeList(lst)];
        run_test("(one two)", Ok(expected))
    }

    #[test]
    fn nested_form() {
        let mut inner_lst = LinkedList::new();
        inner_lst.push_back(SchemeObject::sym_from("one"));
        inner_lst.push_back(SchemeObject::sym_from("two"));
        let inner_obj = SchemeObject::CodeList(inner_lst);

        let mut outer_lst = LinkedList::new();
        outer_lst.push_back(inner_obj);
        outer_lst.push_back(SchemeObject::from("three"));
        let outer_obj = SchemeObject::CodeList(outer_lst);

        let expected = vec![outer_obj];
        run_test("((one two) \"three\")", Ok(expected))
    }

    #[test]
    fn vector() {
        let expected = vec![SchemeObject::Vector(vec![SchemeObject::from(true)])];
        run_test("#(#t)", Ok(expected))
    }

    #[test]
    fn quotes() {
        let scm = "'#t 'symbol '\"string\" '(one two) '#(one two)";
        let v = vec![SchemeObject::sym_from("one"), SchemeObject::sym_from("two")];
        let l = LinkedList::from_iter(v.clone());

        let mut expected = Vec::with_capacity(5);
        expected.push(SchemeObject::from(true));
        expected.push(SchemeObject::sym_from("symbol"));
        expected.push(SchemeObject::from("string"));
        expected.push(SchemeObject::QuotedList(l));
        expected.push(SchemeObject::Vector(v));

        run_test(scm, Ok(expected));
    }
}
