use std::iter::FlatMap;
use std::str::SplitWhitespace;

/// Monster of a type because of the unpleasant way `std::iter` works :-(
pub type IterableOfStrings<'a> =
    FlatMap<SplitWhitespace<'a>, Vec<String>, for<'r> fn(&'r str) -> Vec<String>>;

/// Turn scheme source into an iterable of tokens (Strings)
/// source is modified in place removing comments
pub fn tokenise(source: &mut String) -> IterableOfStrings {
    let trimmed = trim_comments(source.as_ref());
    // copy back into the source buffer so that the lifetime of the iterable is sufficient
    trimmed.clone_into(source);
    source.split_whitespace() // interior of s-expressions are whitespace separated
        .flat_map(tokenise_brackets) // split at ( ), producing the '(' and ')' tokens as well
}

/// remove line comments
fn trim_comments(source: &str) -> String {
    let mut ret = String::new();
    let mut in_comment = false;

    for c in source.chars() {
        if in_comment {
            if c == '\n' {
                // comments end at the end of the lines
                in_comment = false;
            }
            continue;
        }

        // if we are not in a comment
        if c == ';' {
            in_comment = true;
        } else {
            ret.push(c); // only output non-comment characters
        }
    }

    ret
}

/// split things like "(cons" into ["(", "cons"]
fn tokenise_brackets(source: &str) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let mut current = String::new();

    for c in source.chars() {
        // if we need to split
        if c == '(' || c == ')' {
            // flush the previous token
            if !current.is_empty() {
                ret.push(current.clone());
                current.truncate(0);
            }

            // add this token
            ret.push(c.to_string());

        // else continue building up the non-bracket token
        } else {
            current.push(c);
        }
    }

    // flush any remaining non-bracket tokens
    if !current.is_empty() {
        ret.push(current.clone());
    }

    ret
}

#[cfg(test)]
mod tests {

    fn run_test(tv: &str, expected: &Vec<&str>) {
        let res: Vec<String> = super::tokenise(&mut String::from(tv)).collect();
        assert_eq!(res, *expected);
    }

    #[test]
    fn single_s_expr() {
        // junk whitespace added to (a be c)
        let tv = " (a be \tc)\n\t";
        let expected = vec!["(", "a", "be", "c", ")"];
        run_test(tv, &expected);
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
}
