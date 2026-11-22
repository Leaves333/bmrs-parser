use std::collections::HashMap;

use crate::{
    eval::EvalError::UnboundVariable,
    parser::{Def, Expr, Program},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Value {
    Bool(bool),
    Pos(usize),
}

#[derive(Debug)]
pub enum EvalError {
    TypeError(String),
    NameConflict(String),
    UnboundVariable(String),
    UnboundFunction(String),
    ArityMismatch {
        function: String,
        expected: usize,
        got: usize,
    },
    /// `s` at the last position, or `p` at the first position
    OutOfRange(String),
    Recursion(String),
}

#[derive(Clone)]
enum Entry {
    User(Def),
    Successor,
    Predecessor,
    IsSymbol(char),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub function: String,
    pub args: Vec<Value>,
}

struct Env {
    pub function_table: HashMap<String, Entry>,
    pub cache: HashMap<CacheKey, Value>,
}

impl Env {
    fn new(alphabet: &Vec<char>, defs: &Vec<Def>) -> Self {
        let mut env = Env {
            function_table: HashMap::new(),
            cache: HashMap::new(),
        };

        // insert predecessor/successor functions into env
        env.function_table.insert("s".into(), Entry::Successor);
        env.function_table.insert("p".into(), Entry::Predecessor);

        // insert issymbol functions for each char
        for c in alphabet {
            env.function_table.insert((*c).into(), Entry::IsSymbol(*c));
        }

        // insert user defs
        for def in defs {
            env.function_table
                .insert(def.function.clone(), Entry::User(def.clone()));
        }

        env
    }
}

/// maps variable names to values
struct VariableTable {
    table: HashMap<String, Value>,
}

impl VariableTable {
    fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    fn insert_variable(&mut self, env: &Env, name: &str, value: Value) -> Result<(), EvalError> {
        if env.function_table.contains_key(name) {
            return Err(EvalError::NameConflict(format!(
                "symbol {} already exists as a function",
                name
            )));
        }

        if self.table.contains_key(name) {
            return Err(EvalError::NameConflict(format!(
                "symbol {} already exists as a variable",
                name
            )));
        }

        self.table.insert(name.into(), value);
        Ok(())
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

fn is_symbol_fn(pos: Value, target: char, model: &StringModel) -> Result<Value, EvalError> {
    match pos {
        Value::Bool(_) => Err(EvalError::TypeError(
            "is symbol function received a bool, but expected a domain element!".into(),
        )),
        Value::Pos(i) => match model.chars.get(i) {
            Some(c) => Ok(Value::Bool(*c == target)),
            None => Err(EvalError::OutOfRange(format!("index {i} is out of range!"))),
        },
        Value::LeftFish => Ok(Value::Bool(false)),
        Value::RightFish => Ok(Value::Bool(false)),
    }
}

fn predecessor_fn(pos: Value, model: &StringModel) -> Result<Value, EvalError> {
    match pos {
        Value::Bool(_) => Err(EvalError::TypeError(
            "predecessor function received a bool, but expected a domain element!".into(),
        )),
        Value::Pos(i) => {
            if i == 0 {
                Ok(Value::LeftFish)
            } else {
                Ok(Value::Pos(i - 1))
            }
        }
        Value::LeftFish => Ok(Value::LeftFish),
        Value::RightFish => Ok(Value::RightFish),
    }
}

fn successor_fn(pos: Value, model: &StringModel) -> Result<Value, EvalError> {
    match pos {
        Value::Bool(_) => Err(EvalError::TypeError(
            "successor function received a bool, but expected a domain element!".into(),
        )),
        Value::Pos(i) => {
            if i + 1 >= model.chars.len() {
                Ok(Value::RightFish)
            } else {
                Ok(Value::Pos(i + 1))
            }
        }
        Value::LeftFish => Ok(Value::LeftFish),
        Value::RightFish => Ok(Value::RightFish),
    }
}

fn eval_expr(
    expr: &Expr,
    model: &StringModel,
    env: &mut Env,
    vars: &VariableTable,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Variable(name) => match vars.table.get(name) {
            Some(v) => Ok(v.clone()),
            None => Err(UnboundVariable(name.into())),
        },
        Expr::IfThenElse {
            if_clause,
            then_clause,
            else_clause,
        } => {
            let Value::Bool(cond) = eval_expr(if_clause, model, env, vars)? else {
                return Err(EvalError::TypeError(
                    "conditional in if statement was not a bool!".into(),
                ));
            };

            if cond {
                Ok(eval_expr(then_clause, model, env, vars)?)
            } else {
                Ok(eval_expr(else_clause, model, env, vars)?)
            }
        }
        Expr::Application { function, args } => {
            let Some(entry) = env.function_table.get(function).cloned() else {
                return Err(EvalError::UnboundFunction(function.clone()));
            };

            match entry {
                Entry::Successor => {
                    if args.len() != 1 {
                        return Err(EvalError::ArityMismatch {
                            function: "successor".into(),
                            expected: 1,
                            got: args.len(),
                        });
                    }

                    let arg = eval_expr(&args[0], model, env, vars)?;
                    Ok(successor_fn(arg, model)?)
                }
                Entry::Predecessor => {
                    if args.len() != 1 {
                        return Err(EvalError::ArityMismatch {
                            function: "predecessor".into(),
                            expected: 1,
                            got: args.len(),
                        });
                    }

                    let arg = eval_expr(&args[0], model, env, vars)?;
                    Ok(predecessor_fn(arg, model)?)
                }
                Entry::IsSymbol(target) => {
                    if args.len() != 1 {
                        return Err(EvalError::ArityMismatch {
                            function: format!("is symbol {target}"),
                            expected: 1,
                            got: args.len(),
                        });
                    }

                    let arg = eval_expr(&args[0], model, env, vars)?;
                    Ok(is_symbol_fn(arg, target, model)?)
                }
                Entry::User(def) => {
                    if args.len() != def.params.len() {
                        return Err(EvalError::ArityMismatch {
                            function: def.function,
                            expected: def.params.len(),
                            got: args.len(),
                        });
                    }

                    // turn exprs into values, short circuit on first error
                    let args: Vec<Value> = args
                        .iter()
                        .map(|expr| eval_expr(expr, model, env, vars))
                        .collect::<Result<_, _>>()?;

                    // see if this function + args combo is in the cache
                    let key = CacheKey {
                        function: def.function.clone(),
                        args: args.clone(),
                    };
                    if let Some(value) = env.cache.get(&key) {
                        return Ok(value.clone());
                    }

                    // generate new variable table for the function call
                    let mut new_vars = VariableTable::new();
                    for (param, value) in def.params.iter().zip(&args) {
                        new_vars.insert_variable(env, param, value.clone())?;
                    }

                    // evaluate and put new value in the cache
                    let value = eval_expr(&def.expr, model, env, &new_vars)?;
                    env.cache.insert(key, value);
                    Ok(value)
                }
            }
        }
    }
}

// what should the output of evaluating a program be?
// a truth table for each function, for each position
pub fn eval_program(
    program: Program,
    model: StringModel,
) -> Result<HashMap<CacheKey, Value>, EvalError> {
    // prepopulate env based on string model and defined functions
    let mut env = Env::new(&model.alphabet, &program.defs);

    // evaluate all functions
    for def in program.defs {
        // only evaluate "top level" defs for output
        if !def.top_level {
            continue;
        }

        for i in 0..model.chars.len() {
            // evaluate this function at this position
            // note: under this model, we only cache results for user defined functions
            let key = CacheKey {
                function: def.function.clone(),
                args: vec![Value::Pos(i)],
            };

            // don't re-evaluate if the previous result exists in the cache
            if env.cache.contains_key(&key) {
                continue;
            }

            // check function arity
            // (top level functions must only take one argument)
            if def.params.len() != 1 {
                return Err(EvalError::TypeError(
                    "top level functions can only take one argument!".into(),
                ));
            }

            // map variables to names
            let mut vars = VariableTable::new();
            vars.insert_variable(&env, &def.params[0], Value::Pos(i))?;

            // eval the result and cache it
            let eval_result = eval_expr(&def.expr, &model, &mut env, &vars)?;
            env.cache.insert(key, eval_result);
        }
    }

    Ok(env.cache)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::Parser;

    /// parse `src`, build an env over `word`, evaluate `fname`'s body
    /// with its (single) param bound to position `pos`.
    fn eval_fn_at(src: &str, word: &str, fname: &str, pos: usize) -> Result<Value, EvalError> {
        let tokens = tokenize(src).expect("lex error");
        let program = Parser::new(tokens).parse().expect("parse error");
        let model = StringModel::new(word);
        let mut env = Env::new(&model.alphabet, &program.defs);
        let def = program
            .defs
            .iter()
            .find(|d| d.function == fname)
            .expect("def not found");
        let mut vars = VariableTable::new();
        vars.insert_variable(&env, &def.params[0], Value::Pos(pos))?;
        eval_expr(&def.expr, &model, &mut env, &vars)
    }

    fn ok(src: &str, word: &str, fname: &str, pos: usize) -> Value {
        eval_fn_at(src, word, fname, pos).expect("eval error")
    }

    /// parse `src`, evaluate the whole program over `word`, and return
    /// the resulting memo cache.
    fn eval_cache_result(src: &str, word: &str) -> Result<HashMap<CacheKey, Value>, EvalError> {
        let tokens = tokenize(src).expect("lex error");
        let program = Parser::new(tokens).parse().expect("parse error");
        eval_program(program, StringModel::new(word))
    }

    fn eval_cache(src: &str, word: &str) -> HashMap<CacheKey, Value> {
        eval_cache_result(src, word).expect("eval error")
    }

    fn key(fname: &str, args: Vec<Value>) -> CacheKey {
        CacheKey {
            function: fname.into(),
            args,
        }
    }

    /// assert the eval fails with this error *kind*, ignoring the payload
    macro_rules! assert_err {
        ($src:expr, $word:expr, $fname:expr, $pos:expr, $variant:ident) => {
            match eval_fn_at($src, $word, $fname, $pos) {
                Err(EvalError::$variant { .. }) => {}
                other => panic!(
                    "expected EvalError::{}, got {:?}",
                    stringify!($variant),
                    other
                ),
            }
        };
    }

    // small constructors for readability below
    const T: Value = Value::Bool(true);
    const F: Value = Value::Bool(false);
    const fn p(i: usize) -> Value {
        Value::Pos(i)
    }

    // --- happy path ---

    #[test]
    fn literals_and_variables() {
        assert_eq!(ok("let f(x) = true", "a", "f", 0), T);
        assert_eq!(ok("let f(x) = false", "a", "f", 0), F);
        assert_eq!(ok("let f(x) = x", "ab", "f", 1), p(1));
    }

    #[test]
    fn symbol_predicates_read_the_word() {
        // word "ab": pos 0 is a, pos 1 is b
        assert_eq!(ok("let f(x) = a(x)", "ab", "f", 0), T);
        assert_eq!(ok("let f(x) = a(x)", "ab", "f", 1), F);
    }

    #[test]
    fn successor_and_predecessor() {
        assert_eq!(ok("let f(x) = s(x)", "ab", "f", 0), p(1));
        assert_eq!(ok("let f(x) = p(x)", "ab", "f", 1), p(0));
    }

    #[test]
    fn composition_applies_predicates_to_positions() {
        // "ab": successor of 0 is 'b'
        assert_eq!(ok("let f(x) = b(s(x))", "ab", "f", 0), T);
        assert_eq!(ok("let f(x) = a(s(x))", "ab", "f", 0), F);
    }

    #[test]
    fn if_selects_branch_by_condition() {
        // position-valued branches selected by a bool condition
        let src = "let f(x) = if a(x) then s(x) else p(x)";
        assert_eq!(ok(src, "ab", "f", 0), p(1)); // pos 0 is a -> successor
        assert_eq!(ok(src, "ab", "f", 1), p(0)); // pos 1 is b -> predecessor
    }

    #[test]
    fn dead_branch_is_not_evaluated() {
        // a(true) is a TypeError if evaluated, but lives in the untaken branch
        let src = "let f(x) = if true then a(x) else a(true)";
        assert_eq!(ok(src, "ab", "f", 0), T);
    }

    #[test]
    fn user_def_call_binds_params_in_its_own_frame() {
        // exercises: args evaluated in caller scope, body sees callee's x
        let src = "let not(x) = if x then false else true
                   let f(x) = not(a(x))";
        assert_eq!(ok(src, "ab", "f", 0), F);
        assert_eq!(ok(src, "ab", "f", 1), T);
    }

    #[test]
    fn chained_user_defs() {
        let src = "let foo(x) = bar(x)
                   let bar(x) = a(x)";
        assert_eq!(ok(src, "ab", "foo", 0), T);
        assert_eq!(ok(src, "ab", "foo", 1), F);
    }

    #[test]
    fn user_def_shadows_symbol_predicate() {
        // insertion order in Env::new: defs after natives -> defs win
        assert_eq!(ok("let b(x) = false", "ab", "b", 1), F);
    }

    // --- error kinds ---

    #[test]
    fn unbound_variable() {
        assert_err!("let f(x) = y", "ab", "f", 0, UnboundVariable);
    }

    #[test]
    fn unbound_function() {
        assert_err!("let f(x) = q(x)", "ab", "f", 0, UnboundFunction);
    }

    #[test]
    fn arity_mismatch() {
        let src = "let f(x) = g(x, x)
                   let g(y) = y";
        assert_err!(src, "ab", "f", 0, ArityMismatch);
    }

    #[test]
    fn successor_at_word_end_returns_right_fish() {
        assert_eq!(ok("let f(x) = s(x)", "ab", "f", 1), Value::RightFish);
    }

    #[test]
    fn predecessor_at_word_start_returns_left_fish() {
        assert_eq!(ok("let f(x) = p(x)", "ab", "f", 0), Value::LeftFish);
    }

    #[test]
    fn fish_absorb_successor_and_predecessor() {
        // once off the edge, walking can never get back onto the word
        assert_eq!(ok("let f(x) = p(p(x))", "ab", "f", 0), Value::LeftFish);
        assert_eq!(ok("let f(x) = s(p(x))", "ab", "f", 0), Value::LeftFish);
        assert_eq!(ok("let f(x) = s(s(x))", "ab", "f", 1), Value::RightFish);
        assert_eq!(ok("let f(x) = p(s(x))", "ab", "f", 1), Value::RightFish);
    }

    #[test]
    fn symbol_predicates_are_false_on_fish() {
        // no character in the alphabet matches an out-of-bounds position
        assert_eq!(ok("let f(x) = a(p(x))", "ab", "f", 0), F);
        assert_eq!(ok("let f(x) = b(s(x))", "ab", "f", 1), F);
    }

    #[test]
    fn fish_on_both_sides_of_a_single_char_word() {
        // "a": pos 0 is bracketed by LeftFish and RightFish
        assert_eq!(ok("let f(x) = p(x)", "a", "f", 0), Value::LeftFish);
        assert_eq!(ok("let f(x) = s(x)", "a", "f", 0), Value::RightFish);
    }

    #[test]
    fn if_condition_must_be_bool() {
        assert_err!(
            "let f(x) = if s(x) then true else false",
            "ab",
            "f",
            0,
            TypeError
        );
    }

    #[test]
    fn symbol_predicate_needs_a_position() {
        // not(x) is bool-valued, but a(..) expects a position
        let src = "let not(x) = if x then false else true
                   let f(x) = a(not(x))";
        assert_err!(src, "ab", "f", 0, TypeError);
    }

    #[test]
    fn param_conflicting_with_symbol_predicate_is_name_conflict() {
        // param named `a` vs the symbol predicate injected by the word
        assert_err!("let f(a) = x", "ab", "f", 0, NameConflict);
    }

    #[test]
    fn param_conflicting_with_builtin_is_name_conflict() {
        assert_err!("let f(s) = true", "ab", "f", 0, NameConflict);
    }

    // --- cache contents after whole-program evaluation ---

    #[test]
    fn eval_program_caches_all_top_level_positions() {
        // "ab": pos 0 is a, pos 1 is b
        let cache = eval_cache("let #f(x) = a(x)\nlet #g(x) = b(x)", "ab");
        let mut expected = HashMap::new();
        expected.insert(key("f", vec![p(0)]), T);
        expected.insert(key("f", vec![p(1)]), F);
        expected.insert(key("g", vec![p(0)]), F);
        expected.insert(key("g", vec![p(1)]), T);
        // exactly the top-level entries — built-in applications (a, b) are
        // not memoized, only user-defined results
        assert_eq!(cache, expected);
    }

    #[test]
    fn fish_values_appear_in_truth_tables() {
        // the endpoints of "ab" fall off into fish
        let cache = eval_cache("let #f(x) = p(x)\nlet #g(x) = s(x)", "ab");
        let mut expected = HashMap::new();
        expected.insert(key("f", vec![p(0)]), Value::LeftFish);
        expected.insert(key("f", vec![p(1)]), p(0));
        expected.insert(key("g", vec![p(0)]), p(1));
        expected.insert(key("g", vec![p(1)]), Value::RightFish);
        assert_eq!(cache, expected);
    }

    #[test]
    fn recursion_memosizes_intermediate_calls() {
        // f(0) recurses through f(1); only f(0) starts the chain, but the
        // intermediate f(1) must land in the cache too
        let src = "let #f(x) = if a(x) then true else f(s(x))";
        let cache = eval_cache(src, "baa");
        let mut expected = HashMap::new();
        expected.insert(key("f", vec![p(0)]), T);
        expected.insert(key("f", vec![p(1)]), T);
        expected.insert(key("f", vec![p(2)]), T);
        assert_eq!(cache, expected);
    }

    #[test]
    fn cached_values_match_direct_evaluation() {
        // both marked top level, so each gets a full truth table
        let src = "let #g(x) = a(x)\nlet #f(x) = g(x)";
        let cache = eval_cache(src, "ab");
        for fname in ["f", "g"] {
            for pos in 0..2 {
                assert_eq!(
                    cache[&key(fname, vec![p(pos)])],
                    ok(src, "ab", fname, pos),
                    "cache disagrees with direct eval for {fname} at {pos}"
                );
            }
        }
    }

    #[test]
    fn cache_hit_returns_same_value() {
        let src = "let g(x) = a(x)\nlet f(x) = g(x)";
        let tokens = tokenize(src).expect("lex error");
        let program = Parser::new(tokens).parse().expect("parse error");
        let model = StringModel::new("ab");
        let mut env = Env::new(&model.alphabet, &program.defs);
        let def = program
            .defs
            .iter()
            .find(|d| d.function == "f")
            .expect("def not found");
        let mut vars = VariableTable::new();
        vars.insert_variable(&env, &def.params[0], p(0)).unwrap();

        // first call evaluates and memoizes g(0)
        let first = eval_expr(&def.expr, &model, &mut env, &vars).expect("eval error");
        assert_eq!(env.cache.get(&key("g", vec![p(0)])), Some(&first));

        // second call hits the cache and must return the same value
        let second = eval_expr(&def.expr, &model, &mut env, &vars).expect("eval error");
        assert_eq!(first, second);
    }

    #[test]
    fn top_level_function_with_two_params_is_type_error() {
        match eval_cache_result("let #f(x, y) = a(x)", "ab") {
            Err(EvalError::TypeError { .. }) => {}
            other => panic!("expected EvalError::TypeError, got {other:?}"),
        }
    }

    // --- top-level marking gates evaluation ---

    #[test]
    fn marked_program_with_bool_helper_evaluates() {
        // regression: unmarked helpers are only evaluated when called, so a
        // bool-argument helper like `not` no longer crashes whole-program eval
        let src = "let #f(x) = not(a(x))\nlet not(x) = if x then false else true";
        let cache = eval_cache(src, "ab");
        let mut expected = HashMap::new();
        expected.insert(key("f", vec![p(0)]), F);
        expected.insert(key("f", vec![p(1)]), T);
        // the helper's per-call results are memoized, keyed by their bool args
        expected.insert(key("not", vec![T]), F);
        expected.insert(key("not", vec![F]), T);
        assert_eq!(cache, expected);
    }

    #[test]
    fn unmarked_defs_stay_out_of_the_truth_table() {
        // `g` is a helper never called by `f`, so it gets no top-level entries
        let cache = eval_cache("let #f(x) = a(x)\nlet g(x) = b(x)", "ab");
        let mut expected = HashMap::new();
        expected.insert(key("f", vec![p(0)]), T);
        expected.insert(key("f", vec![p(1)]), F);
        assert_eq!(cache, expected);
    }

    #[test]
    fn program_with_no_marked_defs_has_an_empty_cache() {
        // a program of pure helpers is legal, it just computes nothing
        assert!(eval_cache("let f(x) = a(x)", "ab").is_empty());
    }

    #[test]
    fn unmarked_two_param_def_is_not_a_type_error() {
        // the single-argument restriction applies only to marked defs
        let cache = eval_cache_result("let f(x, y) = a(x)", "ab").expect("eval error");
        assert!(cache.is_empty());
    }
}
