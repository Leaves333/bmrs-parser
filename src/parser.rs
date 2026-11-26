use std::ops::Range;

use crate::lexer::{SpannedToken, Token};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub defs: Vec<Def>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Def {
    /// true when the def is marked `#` (its truth table gets computed)
    pub top_level: bool,
    pub function: String,
    pub params: Vec<String>,
    pub expr: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    IfThenElse {
        if_clause: Box<Expr>,
        then_clause: Box<Expr>,
        else_clause: Box<Expr>,
    },
    Application {
        function: String,
        args: Vec<Expr>,
    },
    And {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Or {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Not {
        exp: Box<Expr>,
    },
    Variable(String),
    Bool(bool),
}

#[derive(Debug, PartialEq)]
pub struct ParseError {
    pub span: Range<usize>,
    pub message: String,
}

pub struct Parser {
    tokens: Vec<SpannedToken>,
    cursor: usize,
}

impl Parser {
    // returns the next token
    fn peek(&self) -> &Token {
        &self.tokens[self.cursor].token
    }

    // returns the token two after this one
    fn peek_two(&self) -> &Token {
        if self.cursor + 1 >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.cursor + 1].token
        }
    }

    // advances the cursor by one token
    fn advance(&mut self) -> Token {
        let token = self.tokens[self.cursor].token.clone();
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
        token
    }

    // span of the token the cursor is currently on
    fn span(&self) -> Range<usize> {
        self.tokens[self.cursor].span.clone()
    }

    // builds an error pointing at the current token
    fn error(&self, message: String) -> ParseError {
        ParseError {
            span: self.span(),
            message,
        }
    }

    // advances the cursor if the next token is of the same type as `tok`,
    // otherwise errors
    fn expect(&mut self, expected: &Token) -> Result<Token, ParseError> {
        if std::mem::discriminant(self.peek()) == std::mem::discriminant(expected) {
            let token = self.peek().clone();
            self.advance();
            Ok(token)
        } else {
            Err(self.error(format!(
                "expected {:?}, but got {:?}",
                expected,
                self.peek()
            )))
        }
    }

    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut defs = Vec::new();
        while self.peek() != &Token::EOF {
            defs.push(self.parse_def()?);
        }
        Ok(Program { defs })
    }

    fn parse_def(&mut self) -> Result<Def, ParseError> {
        // NOTE: rule for def:
        // <def>       ::= "let" IDENT "(" <params> ")" "=" <expr>

        self.expect(&Token::Let)?;

        let top_level = if *self.peek() == Token::Hash {
            self.advance();
            true
        } else {
            false
        };

        let name_span = self.span();
        let Token::Ident(function) = self.advance() else {
            return Err(ParseError {
                span: name_span,
                message: "expected function name".into(),
            });
        };

        self.expect(&Token::LeftParen)?;
        let params = self.parse_params()?;
        self.expect(&Token::RightParen)?;
        self.expect(&Token::Equals)?;
        let expr = self.parse_expr()?;

        Ok(Def {
            top_level,
            function,
            params,
            expr,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<String>, ParseError> {
        // NOTE: rule for params:
        // <params>    ::= IDENT ( "," IDENT )*                        // one or more distinct parameters

        // match the first param
        let mut params = Vec::new();
        let name_span = self.span();
        let Token::Ident(var_name) = self.advance() else {
            return Err(ParseError {
                span: name_span,
                message: "expected parameter name".into(),
            });
        };
        params.push(var_name);

        // loop to match remaining params
        loop {
            if *self.peek() == Token::Comma {
                self.advance();
            } else {
                break;
            }
            let name_span = self.span();
            let Token::Ident(var_name) = self.advance() else {
                return Err(ParseError {
                    span: name_span,
                    message: "expected parameter name".into(),
                });
            };
            params.push(var_name);
        }

        Ok(params)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // NOTE: rule for expr:
        // <expr>      ::= "if" <expr> "then" <expr> "else" <expr>
        //               | IDENT "(" [ <expr> ( "," <expr> )* ] ")"    // application
        //               | "(" <expr> ")"                              // grouping
        //               | IDENT                                       // variable
        //               | "true"
        //               | "false"

        if let Token::Ident(name) = self.peek() {
            if *self.peek_two() == Token::LeftParen {
                // function application case

                let function = name.clone();

                // advance past Ident and LeftParen
                self.advance();
                self.advance();

                // match first expr
                let mut args = Vec::new();
                let first = self.parse_expr()?;
                args.push(first);

                // match remaining exprs
                loop {
                    if *self.peek() == Token::Comma {
                        self.advance();
                    } else {
                        break;
                    }
                    let expr = self.parse_expr()?;
                    args.push(expr);
                }

                self.expect(&Token::RightParen)?;

                return Ok(Expr::Application { function, args });
            } else {
                // variable case
                let var_name = name.clone();
                self.advance();
                return Ok(Expr::Variable(var_name));
            }
        }

        // if expr case
        if *self.peek() == Token::If {
            self.advance();
            let if_clause = Box::new(self.parse_expr()?);

            self.expect(&Token::Then)?;
            let then_clause = Box::new(self.parse_expr()?);

            self.expect(&Token::Else)?;
            let else_clause = Box::new(self.parse_expr()?);

            return Ok(Expr::IfThenElse {
                if_clause,
                then_clause,
                else_clause,
            });
        }

        // paren grouping case
        if *self.peek() == Token::LeftParen {
            self.advance();
            let expr = self.parse_expr()?;
            self.expect(&Token::RightParen)?;
            return Ok(expr);
        }

        // bool const cases
        if *self.peek() == Token::True {
            self.advance();
            return Ok(Expr::Bool(true));
        }

        if *self.peek() == Token::False {
            self.advance();
            return Ok(Expr::Bool(false));
        }

        Err(self.error(format!(
            "couldn't parse expression starting with token {:?}",
            self.peek()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// lexes `src` and parses it.
    fn parse_str(src: &str) -> Result<Program, ParseError> {
        let tokens = crate::lexer::tokenize(src).expect("lex error");
        Parser::new(tokens).parse()
    }

    /// parses `src`, panicking on error.
    fn parse_ok(src: &str) -> Program {
        parse_str(src).expect("parse error")
    }

    /// parses `src`, panicking on success, and returns the error.
    fn err(src: &str) -> ParseError {
        match parse_str(src) {
            Ok(p) => panic!("expected parse error, got {p:?}"),
            Err(e) => e,
        }
    }

    /// parses `src`, panicking on success, and returns the error's span.
    /// error assertions match spans, never message text.
    fn err_span(src: &str) -> Range<usize> {
        err(src).span
    }

    fn var(name: &str) -> Expr {
        Expr::Variable(name.into())
    }

    fn app(function: &str, args: Vec<Expr>) -> Expr {
        Expr::Application {
            function: function.into(),
            args,
        }
    }

    // --- connective AST builders ---
    // NOTE: these match the folded AST already in Expr:
    //   Expr::And { lhs, rhs } | Expr::Or { lhs, rhs } | Expr::Not { exp }
    fn not(e: Expr) -> Expr {
        Expr::Not {
            exp: Box::new(e),
        }
    }

    fn and(lhs: Expr, rhs: Expr) -> Expr {
        Expr::And {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    fn or(lhs: Expr, rhs: Expr) -> Expr {
        Expr::Or {
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }
    }

    /// parses `src` as a single def body and returns just its expression.
    fn body(src: &str) -> Expr {
        parse_ok(&format!("let f(x) = {src}")).defs[0].expr.clone()
    }

    #[test]
    fn empty_program() {
        assert_eq!(parse_ok(""), Program { defs: vec![] });
    }

    #[test]
    fn simple_def() {
        assert_eq!(
            parse_ok("let f(x) = x"),
            Program {
                defs: vec![Def {
                    top_level: false,
                    function: "f".into(),
                    params: vec!["x".into()],
                    expr: var("x"),
                }],
            }
        );
    }

    #[test]
    fn bool_consts() {
        match parse_ok("let t(x) = true let e(y) = false").defs.as_slice() {
            [t, e] => {
                assert_eq!(t.expr, Expr::Bool(true));
                assert_eq!(e.expr, Expr::Bool(false));
            }
            defs => panic!("expected 2 defs, got {defs:?}"),
        }
    }

    #[test]
    fn full_program() {
        assert_eq!(
            parse_ok("let f(x, y) = if x then f(y, false) else true"),
            Program {
                defs: vec![Def {
                    top_level: false,
                    function: "f".into(),
                    params: vec!["x".into(), "y".into()],
                    expr: Expr::IfThenElse {
                        if_clause: Box::new(var("x")),
                        then_clause: Box::new(app("f", vec![var("y"), Expr::Bool(false)])),
                        else_clause: Box::new(Expr::Bool(true)),
                    },
                }],
            }
        );
    }

    #[test]
    fn multiple_defs() {
        let p = parse_ok("let a(x) = a(x) let b(y, z) = if y then b(z, y) else z");
        assert_eq!(p.defs.len(), 2);
        assert_eq!(p.defs[0].function, "a");
        assert_eq!(p.defs[1].function, "b");
        assert_eq!(p.defs[1].params, ["y".to_string(), "z".to_string()]);
    }

    #[test]
    fn comment_only_program() {
        assert_eq!(
            parse_ok("-- just a comment\n-- and another"),
            Program { defs: vec![] }
        );
    }

    #[test]
    fn comments_do_not_change_the_ast() {
        // same program with and without comments must parse identically
        assert_eq!(
            parse_ok(
                "-- header\nlet f(x) = x -- trailing\nlet #g(y) = -- inline\n  if y then g(y) else true"
            ),
            parse_ok("let f(x) = x let #g(y) = if y then g(y) else true")
        );
    }

    #[test]
    fn parse_error_span_skips_over_comments() {
        // the comment before the offending token must not shift its span
        assert_eq!(err_span("let f(x) = x\n-- comment\nlet = true"), 28..29);
    }

    #[test]
    fn unmarked_def_is_not_top_level() {
        let p = parse_ok("let f(x) = true");
        assert!(!p.defs[0].top_level);
    }

    #[test]
    fn hash_marker_marks_def_top_level() {
        // `#` sits between `let` and the name; it must not add an AST node
        let p = parse_ok("let #f(x) = true");
        assert!(p.defs[0].top_level);
        assert_eq!(p.defs[0].function, "f");
        assert_eq!(p.defs[0].expr, Expr::Bool(true));
    }

    #[test]
    fn top_level_flag_is_per_def() {
        let p = parse_ok("let #f(x) = true let g(x) = false let #h(x) = true");
        assert!(p.defs[0].top_level);
        assert!(!p.defs[1].top_level);
        assert!(p.defs[2].top_level);
    }

    #[test]
    fn rejects_marker_before_let() {
        // the marker attaches to the name, not the whole def
        assert_eq!(err_span("#let f(x) = true"), 0..1); // `#`
    }

    #[test]
    fn rejects_marker_after_name() {
        assert_eq!(err_span("let f#(x) = true"), 5..6); // `#` where `(` expected
    }

    #[test]
    fn rejects_marker_in_expression() {
        assert_eq!(err_span("let f(x) = #true"), 11..12); // `#` can't start an expr
    }

    #[test]
    fn nesting_ifs_resolves_dangling_else_inward() {
        // `if a then if b then true else false else true`
        // the first `else` belongs to the inner if, the second to the outer
        assert_eq!(
            parse_ok("let f(a, b) = if a then if b then true else false else true"),
            Program {
                defs: vec![Def {
                    top_level: false,
                    function: "f".into(),
                    params: vec!["a".into(), "b".into()],
                    expr: Expr::IfThenElse {
                        if_clause: Box::new(var("a")),
                        then_clause: Box::new(Expr::IfThenElse {
                            if_clause: Box::new(var("b")),
                            then_clause: Box::new(Expr::Bool(true)),
                            else_clause: Box::new(Expr::Bool(false)),
                        }),
                        else_clause: Box::new(Expr::Bool(true)),
                    },
                }],
            }
        );
    }

    #[test]
    fn grouping_is_transparent() {
        // parens must not produce any AST node, nested or around an application
        assert_eq!(parse_ok("let f(x) = ((x))").defs[0].expr, var("x"));
        assert_eq!(
            parse_ok("let f(x) = (f(x))").defs[0].expr,
            app("f", vec![var("x")])
        );
        assert_eq!(
            parse_ok("let f(x) = if x then (true) else (f((x)))").defs[0].expr,
            Expr::IfThenElse {
                if_clause: Box::new(var("x")),
                then_clause: Box::new(Expr::Bool(true)),
                else_clause: Box::new(app("f", vec![var("x")])),
            }
        );
    }

    #[test]
    fn nested_application_args() {
        assert_eq!(
            parse_ok("let f(x) = f(g(x), g(g(x), x))").defs[0].expr,
            app(
                "f",
                vec![
                    app("g", vec![var("x")]),
                    app("g", vec![app("g", vec![var("x")]), var("x")]),
                ]
            )
        );
    }

    #[test]
    fn rejects_missing_then() {
        //                          0123456789_0123456789_0
        //                                    1         2
        assert_eq!(err_span("let f(x) = if x else true"), 16..20); // `else`
    }

    #[test]
    fn rejects_missing_else() {
        // the EOF token has an empty span at the end of the input
        assert_eq!(err_span("let f(x) = if x then true"), 25..25);
    }

    #[test]
    fn rejects_unclosed_arg_list() {
        assert_eq!(err_span("let f(x) = f(x"), 14..14); // EOF
    }

    #[test]
    fn rejects_unclosed_group() {
        assert_eq!(err_span("let f(x) = (x"), 13..13); // EOF
    }

    #[test]
    fn rejects_missing_equals() {
        assert_eq!(err_span("let f(x) true"), 9..13); // `true`
    }

    #[test]
    fn rejects_missing_parens_around_params() {
        assert_eq!(err_span("let f = true"), 6..7); // `=`
    }

    #[test]
    fn rejects_empty_params() {
        // grammar requires one or more params
        assert_eq!(err_span("let f() = true"), 6..7); // `)`
    }

    #[test]
    fn rejects_trailing_comma_in_params() {
        assert_eq!(err_span("let f(x,) = true"), 8..9); // `)`
    }

    #[test]
    fn rejects_def_without_let() {
        assert_eq!(err_span("f(x) = true"), 0..1); // `f`
    }

    #[test]
    fn rejects_trailing_garbage_after_def() {
        assert_eq!(err_span("let f(x) = true false"), 16..21); // `false`
    }

    #[test]
    fn rejects_expr_starting_with_punctuation() {
        assert_eq!(err_span("let f(x) = ,"), 11..12); // `,`
    }

    #[test]
    fn rejects_zero_arg_application() {
        // the grammar's `[ <expr> ... ]` would allow `g()`, but the parser
        // requires at least one argument — see the open question in dsl.md
        assert_eq!(err_span("let g(x) = g()"), 13..14); // `)`
    }

    #[test]
    fn error_spans_point_at_the_offending_token() {
        //                          0123456789_0123456789_0123456789_01
        //                                    1         2          3
        assert_eq!(err_span("let f(x) = if x else true"), 16..20); // `else`

        assert_eq!(err_span("let true(x) = true"), 4..8); // `true` (a digit wouldn't even lex)

        assert_eq!(err_span("let f(,) = true"), 6..7); // `,`

        // the EOF token has an empty span at the end of the input
        assert_eq!(err_span("let f(x) = (x"), 13..13);
    }

    // --- logical connectives: precedence & associativity ---
    // precedence is or < and < not; both binary connectives are left-associative.

    #[test]
    fn not_binds_tighter_than_and() {
        // `not a and b` parses as `(not a) and b`, not `not (a and b)`
        assert_eq!(body("not a and b"), and(not(var("a")), var("b")));
    }

    #[test]
    fn and_binds_tighter_than_or() {
        assert_eq!(body("a or b and c"), or(var("a"), and(var("b"), var("c"))));
        assert_eq!(body("a and b or c"), or(and(var("a"), var("b")), var("c")));
    }

    #[test]
    fn connectives_are_left_associative() {
        assert_eq!(
            body("a and b and c"),
            and(and(var("a"), var("b")), var("c"))
        );
        assert_eq!(
            body("a or b or c"),
            or(or(var("a"), var("b")), var("c"))
        );
    }

    #[test]
    fn grouped_connective_mix() {
        // `a and b or c and d` => `(a and b) or (c and d)`
        assert_eq!(
            body("a and b or c and d"),
            or(and(var("a"), var("b")), and(var("c"), var("d")))
        );
    }

    #[test]
    fn grouping_overrides_precedence() {
        // parens force the `or` to bind tighter than the surrounding `and`
        assert_eq!(
            body("(a or b) and c"),
            and(or(var("a"), var("b")), var("c"))
        );
    }

    // --- logical connectives: operand shapes ---

    #[test]
    fn not_over_grouping_and_application() {
        assert_eq!(
            body("not (a and b)"),
            not(and(var("a"), var("b")))
        );
        assert_eq!(body("not a(x)"), not(app("a", vec![var("x")])));
    }

    #[test]
    fn double_negation_via_grouping() {
        // `not (not a)` is valid; bare `not not a` depends on the not_expr rule
        assert_eq!(body("not (not a)"), not(not(var("a"))));
    }

    #[test]
    fn connectives_in_a_marked_def_body() {
        let p = parse_ok("let #f(x) = not a(x) and b(x)");
        assert!(p.defs[0].top_level);
        assert_eq!(
            p.defs[0].expr,
            and(not(app("a", vec![var("x")])), app("b", vec![var("x")]))
        );
    }

    #[test]
    fn if_branches_are_full_expressions() {
        // if is atomic but each of its three clauses parses a full <expr>
        assert_eq!(
            body("if a then b else c and d"),
            Expr::IfThenElse {
                if_clause: Box::new(var("a")),
                then_clause: Box::new(var("b")),
                else_clause: Box::new(and(var("c"), var("d"))),
            }
        );
    }

    // --- logical connectives: rejections ---

    #[test]
    fn rejects_leading_and_or_or() {
        // and/or are infix only, so they can't start an expression
        assert_eq!(err_span("let f(x) = and b"), 11..14); // `and`
        assert_eq!(err_span("let f(x) = or b"), 11..13); // `or`
    }

    #[test]
    fn rejects_dangling_connective() {
        // a binary connective needs a right operand
        assert_eq!(err_span("let f(x) = a and"), 16..16); // EOF
        assert_eq!(err_span("let f(x) = a or"), 15..15); // EOF
    }
}
