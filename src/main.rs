#[cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
mod tokenise;

fn main() {
    let test_str = String::from(
        "; I am a comment
(disp \"; I am not a comment\")",
    );
    println!("{}", test_str);

    let res: Vec<_> = tokenise::tokenise(&test_str);

    println!("Tokenised: {:?}", res);
}
