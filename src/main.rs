#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
extern crate rustyscheme;

use rustyscheme::ast;
use rustyscheme::stdlib::get_std_env;
use rustyscheme::tokenise;

fn main() {
    let test_str = String::from(
        "; I am a comment
(disp \"; I am not a comment\")",
    );
    println!("{}", test_str);

    let tokens = tokenise::tokenise(&mut test_str.chars());
    let ast = ast::parse_tokens(tokenise::TokenIterator::new(&mut test_str.chars())).unwrap();

    println!("\nTokenised: {:?}", tokens);
    println!("\nAST: {:?}", &ast);
    println!("\nExecuting...");
    let mut env = get_std_env();
    let res: Vec<_> = ast.iter().map(|scm_obj| scm_obj.exec(&mut env)).collect();
    println!("\nGot {:?}", res);
}
