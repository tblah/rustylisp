#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
mod tokenise;

fn main() {
    let test_str = "(+ 1 2 (plus 3 4))";
    println!("{}", test_str);
    let res = tokenise::tokenise(test_str);
    println!("{:?}", res);
}
