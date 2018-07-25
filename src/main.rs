#![cfg_attr(feature = "cargo-clippy", deny(clippy_pedantic))]
extern crate rustyscheme;

extern crate readline;

use readline::{add_history, readline};

use rustyscheme::ast;
use rustyscheme::data::runtime::RuntimeObject;
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
                Err(e) | Ok(Err(e)) => {
                    println!("Error: {:?}", e);
                    continue 'input;
                }
            };

            match *res {
                RuntimeObject::None => (),
                _ => println!("{}", res),
            }

            io::stdout().flush().unwrap();
        }
    }
}
