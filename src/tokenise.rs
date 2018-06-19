/// Turn scheme source into a vector of tokens (Strings)
pub fn tokenise(source: &str) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let mut current = String::new();

    let special = vec!['(', ')', '\'', '#'];

    let mut escaped = false; // '\\'
    let mut in_string = false; // '"'
    let mut in_comment = false; // ';'

    for c in source.chars() {
        // comments end at the end of lines
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }

            continue;
        }

        // don't tokenise anything which is escaped
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

        // don't split tokenise within strings
        if in_string {
            // add the character to the current token
            current.push(c);

            if c == '"' {
                // end of a string
                in_string = false;
                ret.push(current); // current can't be empty here
                current = String::new();
            }

            continue;
        }

        // if we are starting a string
        if c == '"' {
            in_string = true;
            // push old current token
            if !current.is_empty() {
                ret.push(current);
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
                ret.push(current);
                current = String::new();
            }

            continue;
        }

        // if we need to split at a token we keep
        if special.contains(&c) {
            // flush the previous token
            if !current.is_empty() {
                ret.push(current);
                current = String::new();
            }

            // add this token
            ret.push(c.to_string());

        // if we need to split at a token we don't keep
        } else if c.is_whitespace() {
            // push current token
            if !current.is_empty() {
                ret.push(current.clone());
                current.truncate(0);
            }

        // else just add a normal character to the current token
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
        let res = super::tokenise(&mut String::from(tv));
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
