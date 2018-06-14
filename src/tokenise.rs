pub fn tokenise(source: &str) -> Vec<String> {
    source
        .trim()
        .split_whitespace()
        .flat_map(tokenise_brackets)
        .map(String::from)
        .collect()
}

fn tokenise_brackets(source: &str) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();
    let mut current = String::new();
    let it = source.chars();

    for c in it {
        if c == '(' || c == ')' {
            if !current.is_empty() {
                ret.push(current.clone());
                current.truncate(0);
            }
            ret.push(c.to_string());
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        ret.push(current.clone());
    }
    ret
}

#[cfg(test)]
mod tests {

    fn run_test(tv: &str, expected: &Vec<&str>) {
        let res: Vec<String> = super::tokenise(tv);
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
