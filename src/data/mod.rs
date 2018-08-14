//! Data model

// definition of SchemeObject
mod scm_obj;

// evaluate a scheme object on its own
mod exec;

// evaluate a scheme object with arguments (used in ::exec)
mod exec_args;

// environment variable storage and lookup
pub mod env;
// re-export
pub use self::scm_obj::{SchemeObject, SymFrom};

use std::string::ToString;

/// Possible parse errors
#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    /// Misc. runtime error
    Msg(String),
    /// Name lookup error (reporting the name of the unbound variable)
    NameLookup(String),
}

/// Creates a `RuntimeError::Msg`
impl<'a> From<&'a str> for RuntimeError {
    fn from(s: &'a str) -> Self {
        RuntimeError::Msg(String::from(s))
    }
}

/// Creates a `RuntimeError::Msg`
impl From<String> for RuntimeError {
    fn from(s: String) -> Self {
        RuntimeError::Msg(s)
    }
}

/// For displaying in the REPL
impl ToString for RuntimeError {
    fn to_string(&self) -> String {
        use self::RuntimeError::*;

        match self {
            Msg(s) => s.clone(),
            NameLookup(s) => {
                // catch special forms baked into exec and provide usage hints
                match s.as_str() {
                    "define" => String::from("Built-in: define: (define name value) | (define (function_name arg) (body arg)"),

                    "let" => String::from(
"Built-in: let: (let ((name1 value1)
                     (name2 value2))
                    (body name1 name2))"),

                    "lambda" => String::from("Built-in: lambda: (lambda (arg1 arg2) (body arg1 arg2))"),
                    "if" => String::from("Built-in: if: (if cond true_body false_body)"),

                    _ => format!("Undefined binding: {}", s),
                }
            }
        }
    }
}
