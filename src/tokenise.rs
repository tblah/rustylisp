pub struct TokenIterator {
    // Uses String because str is a pain with lifetimes
    iter: Box<Iterator<Item = String>>,
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

impl TokenIterator {
    pub fn new(source: &'static str) -> Self {
        // trim excess whitespace
        let trimmed: &str = source.trim();
        // split tokens at whitespace
        let split = trimmed.split_whitespace();

        let map = split.flat_map(tokenise_brackets);
        Self { iter: Box::new(map) }
    }
}

impl Iterator for TokenIterator {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.iter.next()
    }
}

#[cfg(test)]
mod tests {
    use super::TokenIterator;

    fn run_test(tv: &'static str, expected: &Vec<&str>) {
        let res: Vec<String> = TokenIterator::new(tv).collect();
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
