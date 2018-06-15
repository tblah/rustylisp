#![feature(toowned_clone_into)]
#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
mod tokenise;

fn main() {
    let mut test_str = String::from("; Comment\n(+ 1 2 (plus 3 4))");
    println!("{}", test_str);

    let res: Vec<_> = tokenise::tokenise(&mut test_str).collect();

    println!("{:?}", res);
}
