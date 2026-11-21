use std::collections::HashMap;

use crate::parser::{Def, Expr, Program};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {
    Bool(bool),
    Pos(usize),
}

pub enum EvalError {
    TypeError(String),
}

enum Entry {
    User(Def),
    Successor,
    Predecessor,
    IsSymbol(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    function: String,
    args: Vec<Value>,
}

struct Env {
    symbol_table: HashMap<String, Entry>,
    cache: HashMap<CacheKey, Value>,
}

impl Env {
    fn new(alphabet: &Vec<char>, defs: &Vec<Def>) -> Self {
        let mut env = Env {
            symbol_table: HashMap::new(),
            cache: HashMap::new(),
        };

        // insert predecessor/successor functions into env
        env.symbol_table.insert("s".into(), Entry::Successor);
        env.symbol_table.insert("p".into(), Entry::Predecessor);

        // insert issymbol functions for each char
        for c in alphabet {
            env.symbol_table.insert((*c).into(), Entry::IsSymbol(*c));
        }

        // insert user defs
        for def in defs {
            env.symbol_table
                .insert(def.function.clone(), Entry::User(def.clone()));
        }

        env
    }
}

pub struct StringModel {
    chars: Vec<char>,
    alphabet: Vec<char>,
}

impl StringModel {
    pub fn new(src: &str) -> Self {
        let chars = src.chars().collect();
        let mut alphabet: Vec<char> = Vec::new();
        for c in &chars {
            if !alphabet.contains(c) {
                alphabet.push(*c);
            }
        }
        StringModel { chars, alphabet }
    }
}

pub fn eval_expr(expr: &Expr, model: &StringModel) -> Result<Value, EvalError> {
    todo!()
}

// what should the output of evaluating a program be?
// a truth table for each function, for each position
pub fn eval_program(program: Program, model: StringModel) -> Result<(), EvalError> {
    // prepopulate env based on string model and defined functions
    let mut env = Env::new(&model.alphabet, &program.defs);

    // evaluate all functions
    // TODO: only evaluate "top level" defs for the output in the future
    for def in program.defs {
        for i in 0..model.chars.len() {
            // evaluate this function at this position
            let key = CacheKey {
                function: def.function.clone(),
                args: vec![Value::Pos(i)],
            };

            // don't re-evaluate if the previous result exists in the cache
            if env.cache.contains_key(&key) {
                continue;
            }

            env.cache.insert(key, eval_expr(&def.expr, &model)?);
        }
    }

    todo!()
}
