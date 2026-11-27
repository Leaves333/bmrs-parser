//! Shared AST builders for the crate's unit tests.
//!
//! These let each stage's unit tests build inputs *directly* without depending
//! on the stage(s) that produce them: evaluator tests construct `Expr`/`Def`
//! trees instead of lexing + parsing source text, and parser tests reuse the
//! same constructors for expected ASTs. Keeping them here avoids duplicating
//! the constructors in every module's `#[cfg(test)] mod tests`.
//!
//! NOTE: test-only module — declared in `lib.rs` under `#[cfg(test)]`.

use crate::parser::{Def, DefKind, Expr};

// --- expression constructors ---

pub(crate) fn var(name: &str) -> Expr {
    Expr::Variable(name.into())
}

pub(crate) fn app(function: &str, args: Vec<Expr>) -> Expr {
    Expr::Application {
        function: function.into(),
        args,
    }
}

/// boolean literal
pub(crate) fn lit(b: bool) -> Expr {
    Expr::Bool(b)
}

pub(crate) fn ite(cond: Expr, then: Expr, els: Expr) -> Expr {
    Expr::IfThenElse {
        if_clause: Box::new(cond),
        then_clause: Box::new(then),
        else_clause: Box::new(els),
    }
}

pub(crate) fn not(exp: Expr) -> Expr {
    Expr::Not {
        exp: Box::new(exp),
    }
}

pub(crate) fn and(lhs: Expr, rhs: Expr) -> Expr {
    Expr::And {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

pub(crate) fn or(lhs: Expr, rhs: Expr) -> Expr {
    Expr::Or {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

// --- definition constructors ---

/// a non-top-level def (`let f(x, ...) = expr`)
pub(crate) fn def(function: &str, params: &[&str], expr: Expr) -> Def {
    Def {
        kind: DefKind::Hidden,
        function: function.into(),
        params: params.iter().map(|p| p.to_string()).collect(),
        expr,
    }
}

/// a `#`-marked debug row: a truth table is computed, but it is not enrolled in the
/// phoneme validity check. single parameter.
pub(crate) fn tdef(function: &str, param: &str, expr: Expr) -> Def {
    Def {
        kind: DefKind::Debug,
        function: function.into(),
        params: vec![param.into()],
        expr,
    }
}

/// an `@phoneme`-decorated output def: a row that *does* join the exactly-one check.
/// `label` is the output phoneme and may differ from the function's own name.
#[allow(dead_code)] // consumed by the upcoming validity tests
pub(crate) fn odef(label: &str, function: &str, param: &str, expr: Expr) -> Def {
    Def {
        kind: DefKind::Output(label.into()),
        function: function.into(),
        params: vec![param.into()],
        expr,
    }
}
