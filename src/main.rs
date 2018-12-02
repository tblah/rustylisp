#![cfg_attr(feature = "cargo-clippy", deny(clippy::pedantic))]
extern crate rustyscheme;

extern crate readline;

use readline::{add_history, readline};

use rustyscheme::ast;
use rustyscheme::data::SchemeObject;
use rustyscheme::stdlib::get_std_env;

use std::io;
use std::io::Write;

fn main() {
    let prompt = "demo> ";
    let env = get_std_env();

    'input: while let Ok(s) = readline(prompt) {
        add_history(&s).unwrap();

        let code = ast::ObjectIterator::from(s.chars());

        for scm_obj in code {
            let res = match scm_obj.map(|obj| obj.exec(&env)) {
                Ok(Ok(r)) => r,
                Err(e) => {
                    println!("Parse Error: {:?}", e);
                    continue 'input;
                }
                Ok(Err(e)) => {
                    println!("{}", e.to_string());
                    continue 'input;
                }
            };

            if let SchemeObject::None = *res {
                println!("None")
            } else {
                println!("{}", res)
            };

            io::stdout().flush().unwrap();
        }
    }
}
