use std::iter::FlatMap;
use std::str::SplitWhitespace;

/// Monster of a type because of the unpleasant way `std::iter` works :-(
pub type IterableOfStrings<'a> = FlatMap<
    SplitWhitespace<'a>,
    Vec<String>,
    for<'r> fn(&'r str) -> Vec<String>,
>;

/// Turn scheme source into an iterable of tokens (Strings)
pub fn tokenise(source: &str) -> IterableOfStrings {
    source
        .trim()
        .split_whitespace() // interior of s-expressions are whitespace separated
        .flat_map(tokenise_brackets) // split at ( ), producing the '(' and ')' tokens as well
}

// split things like "(cons" into ["(", "cons"]
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
        let res: Vec<String> = super::tokenise(tv).collect();
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
}
