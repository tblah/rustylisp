#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
mod tokenise;

fn main() {
    let test_str = "(+ 1 2 (plus 3 4))";
    println!("{}", test_str);

    let res: Vec<_> = tokenise::tokenise(test_str).collect();

    println!("{:?}", res);
}
