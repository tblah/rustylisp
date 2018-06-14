
#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]

mod tokenise;
use self::tokenise::TokenIterator;

fn main() {
    let test_str = "(+ 1 2 (plus 3 4))";
    println!("{}", test_str);
    let res: Vec<_> = TokenIterator::new(test_str).collect();
    println!("{:?}", res);
}
