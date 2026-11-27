//! End-to-end integration tests: BMRS source text -> lex -> parse -> eval -> truth table.
//!
//! These exercise the whole library route (not just one stage) and compare the
//! *computed* truth table — the data the program prints — against expected
//! values. Asserting on stdout is intentionally avoided so the tests stay
//! stable across formatting changes.

use std::collections::{BTreeMap, HashMap};

use bmrs_parser::{
    eval::{CacheKey, StringModel, Value, eval_program},
    lexer, parser,
};

const T: Value = Value::Bool(true);
const F: Value = Value::Bool(false);

/// Run `source` over `word` and collect one row per top-level (`#`) def:
/// the values at each domain position, in position order.
fn rows(source: &str, word: &str) -> BTreeMap<String, Vec<Value>> {
    let toks = lexer::tokenize(source).expect("lex error");
    let prog = parser::Parser::new(toks).parse().expect("parse error");
    let model = StringModel::new(word);
    let cache: HashMap<CacheKey, Value> = eval_program(&prog, &model).expect("eval error");

    prog.defs
        .iter()
        .filter(|d| d.top_level)
        .map(|d| {
            let row = (0..model.len())
                .map(|i| {
                    cache[&CacheKey {
                        function: d.function.clone(),
                        args: vec![Value::Pos(i)],
                    }]
                })
                .collect();
            (d.function.clone(), row)
        })
        .collect()
}

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(path).expect("fixture not found")
}

// --- source-string cases over the implemented grammar ---

#[test]
fn marked_def_predicate() {
    // "ab": pos 0 is 'a', pos 1 is 'b'
    let t = rows("let #f(x) = a(x)", "ab");
    assert_eq!(t["f"], vec![T, F]);
}

#[test]
fn if_composition_end_to_end() {
    let t = rows("let #f(x) = if a(x) then true else b(x)", "ab");
    assert_eq!(t["f"], vec![T, T]);
}

#[test]
fn recursion_memoizes_end_to_end() {
    let t = rows("let #f(x) = if a(x) then true else f(s(x))", "baa");
    assert_eq!(t["f"], vec![T, T, T]);
}

#[test]
fn fish_boundaries_end_to_end() {
    let t = rows("let #f(x) = p(x)\nlet #g(x) = s(x)", "ab");
    assert_eq!(t["f"], vec![Value::LeftFish, Value::Pos(0)]);
    assert_eq!(t["g"], vec![Value::Pos(1), Value::RightFish]);
}

#[test]
fn only_marked_defs_produce_rows() {
    let t = rows("let #f(x) = a(x)\nlet helper(x) = b(x)", "ab");
    assert_eq!(t.keys().collect::<Vec<_>>(), vec!["f"]); // `helper` is unmarked
}

// --- fixtures: run the existing .bmrs programs through the whole pipeline ---

#[test]
fn isl_fixture() {
    // "baab" is the word where the two transducer variants differ.
    // The isl variant reads the `b` predicate directly at the predecessor.
    let t = rows(&fixture("isl.bmrs"), "baab");
    insta_like(&t, &[("a'", &[F, F, T, F]), ("b'", &[T, T, F, T])]);
}

#[test]
fn osl_fixture() {
    // The osl variant recurses through `b'`, so once `b` fires the carry
    // propagates rightwards across the rest of the word.
    let t = rows(&fixture("osl.bmrs"), "baab");
    insta_like(&t, &[("a'", &[F, F, F, F]), ("b'", &[T, T, T, T])]);
}

/// Assert `rows` equals the expected (name -> row) pairs. Names compared by the
/// BTreeMap's sorted key order.
fn insta_like(rows: &BTreeMap<String, Vec<Value>>, expected: &[(&str, &[Value])]) {
    let got: Vec<(&String, &Vec<Value>)> = rows.iter().collect();
    let want: Vec<(&str, &[Value])> = expected.iter().map(|(n, r)| (*n, *r)).collect();
    assert_eq!(got.len(), want.len(), "row count mismatch: {rows:?}");
    for ((gname, grow), (wname, wrow)) in got.iter().zip(want.iter()) {
        assert_eq!(gname.as_str(), *wname);
        assert_eq!(grow.as_slice(), *wrow, "row {wname} mismatch");
    }
}

// --- connectives: exercise the whole route once parse_expr supports them ---

#[test]
fn connectives_end_to_end() {
    let t = rows("let #f(x) = a(x) and not b(x)", "ab");
    assert_eq!(t["f"], vec![T, F]);
}
