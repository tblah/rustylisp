#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
extern crate rustyscheme;

use rustyscheme::ast;
use rustyscheme::tokenise;

fn main() {
    let test_str = String::from(
        "; I am a comment
(disp \"; I am not a comment\")",
    );
    println!("{}", test_str);

    let tokens = tokenise::tokenise(&mut test_str.chars());
    let ast = ast::parse_tokens(tokenise::TokenIterator::new(&mut test_str.chars()));

    println!("\nTokenised: {:?}", tokens);
    println!("\nAST: {:?}", ast.unwrap());
}
