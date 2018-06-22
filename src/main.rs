#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
extern crate rustyscheme;

use rustyscheme::tokenise;

fn main() {
    let test_str = String::from(
        "; I am a comment
(disp \"; I am not a comment\")",
    );
    println!("{}", test_str);

    let res: Vec<_> = tokenise::tokenise(&mut test_str.chars());

    println!("Tokenised: {:?}", res);
}
