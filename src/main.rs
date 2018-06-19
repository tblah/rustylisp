#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
mod tokenise;

fn main() {
    let test_str = String::from("; Comment\n(disp \"hello world\")");
    println!("{}", test_str);

    let res: Vec<_> = tokenise::tokenise(&test_str);

    println!("Tokenised: {:?}", res);
}
