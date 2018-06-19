use std::collections::VecDeque;
use std::iter::Iterator;

/// Iterator for tokens
pub struct TokenInterator<'a> {
    source: &'a mut Iterator<Item = char>,
    pending: VecDeque<String>,
}

impl<'a> TokenInterator<'a> {
    /// Create a new instance of TokenIterator
    pub fn new(source: &'a mut Iterator<Item = char>) -> Self {
        Self {
            source,
            pending: VecDeque::with_capacity(2),
        }
    }
}

/// predicate used in next
/// defines characters which we split tokens upon (other than whitespace)
fn is_special(c: char) -> bool {
    c == '(' || c == ')' || c == '\'' || c == '#'
}

impl<'a> Iterator for TokenInterator<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // clear any pending items
        let next = self.pending.pop_front();
        if next.is_some() {
            return next;
        }

        // else we have to do some work:

        // this will be the item returned
        let mut current = String::new();

        // state of the tokeniser
        let mut escaped = false; // '\\'
        let mut in_comment = false; // ';'
        let mut in_string = false; // '"'

        // iterate through available characters in the source iterator
        for c in &mut self.source {
            // comments end at the end of lines
            if in_comment {
                if c == '\n' {
                    in_comment = false;
                }

                // otherwise ignore comment characters
                continue;
            }

            // don't do anything with characters which are escaped
            if escaped {
                escaped = false;
                current.push(c);
                continue;
            }

            // begin escaping
            if c == '\\' {
                escaped = true;
                continue;
            }

            // don't split tokens within strings
            if in_string {
                // add the character to the current token
                current.push(c);

                if c == '"' {
                    // end of a string
                    self.pending.push_back(current);
                    return self.pending.pop_front();
                }

                continue;
            }

            // if we are starting a string
            if c == '"' {
                in_string = true;
                // push old current token
                if !current.is_empty() {
                    self.pending.push_back(current);
                    current = String::new();
                }

                // begin the new current token
                current.push(c);
                continue;
            }

            // if we are starting a comment
            if c == ';' {
                in_comment = true;
                // push old current token
                if !current.is_empty() {
                    self.pending.push_back(current);
                    current = String::new();
                    // don't return here because we want to keep the in_comment state
                }

                continue;
            }

            // if we need to split at a token other than whitespace
            if is_special(c) {
                // flush the previous token
                if !current.is_empty() {
                    self.pending.push_back(current);
                }

                // add this character (e.g. '(' as a token
                self.pending.push_back(c.to_string());

                // safe to return because we can't have any state variables true
                return self.pending.pop_front();
            }

            // if we need to split at a token we don't keep
            if c.is_whitespace() {
                // push current token
                if !current.is_empty() {
                    // safe to return because no state variables can be true
                    return Some(current);
                }

            // else just add a normal character to the current token
            } else {
                current.push(c);
            }
        } // end of source iterator

        // flush any remaining stuff
        if !current.is_empty() {
            return Some(current);
        }

        None
    }
}

/// Turn scheme source into a vector of tokens (Strings)
pub fn tokenise(source: &mut Iterator<Item = char>) -> Vec<String> {
    let iter = TokenInterator::new(source);
    iter.collect()
}

#[cfg(test)]
mod tests {
    fn run_test(tv: &str, expected: &Vec<&str>) {
        let res = super::tokenise(&mut tv.chars());
        assert_eq!(res, *expected);
    }

    #[test]
    fn single_s_expr() {
        // junk whitespace added to (a be c)
        run_test(" (a be \tc)\n\t", &vec!["(", "a", "be", "c", ")"]);
    }

    #[test]
    fn nested_s_expr() {
        run_test(
            "(apple (b cool) d)",
            &vec!["(", "apple", "(", "b", "cool", ")", "d", ")"],
        );
    }

    #[test]
    fn line_comments() {
        run_test(
            "(add ; wild comment
                 1 2 3)",
            &vec!["(", "add", "1", "2", "3", ")"],
        )
    }

    #[test]
    fn multiple_lines() {
        run_test(
            "
             (one two)
             ; two three four
             (five)
",
            &vec!["(", "one", "two", ")", "(", "five", ")"],
        )
    }

    #[test]
    fn boolean() {
        run_test("#t #f", &vec!["#", "t", "#", "f"])
    }

    #[test]
    fn quoting() {
        run_test("'(I am quoted)", &vec!["'", "(", "I", "am", "quoted", ")"])
    }

    #[test]
    fn strings() {
        run_test("\"Hello world(!)\"", &vec!["\"Hello world(!)\""])
    }

    #[test]
    fn escaping() {
        run_test(
            "\"I said \\\"Hello world\\\"\"",
            &vec!["\"I said \"Hello world\"\""],
        )
    }

    #[test]
    fn comment_in_string() {
        run_test(
            ";this is a comment
            \"; this is not a comment\"",
            &vec!["\"; this is not a comment\""],
        )
    }
}
