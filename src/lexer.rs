use logos::Logos;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[token("let")]
    Let,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("=")]
    Equals,
    #[token(",")]
    Comma,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
    /// this variant is never produced by the lexer.
    /// `tokenize` appends it at the end of the input,
    /// so the parser can treat EOF as a lookahead token.
    EOF,
}

#[derive(Debug)]
pub struct LexError {
    pub span: Range<usize>,
    pub found: String,
}

/// tokenizes program source code `src` into a vec of tokens.
/// appends `Token::EOF` to the end of the stream.
pub fn tokenize(src: &str) -> Result<Vec<(Token, Range<usize>)>, LexError> {
    let mut toks = Vec::new();
    let mut lexer = Token::lexer(src);
    while let Some(result) = lexer.next() {
        let span = lexer.span();
        match result {
            Ok(tok) => toks.push((tok, span)),
            Err(_) => {
                return Err(LexError {
                    span: span.clone(),
                    found: src[span].to_string(),
                });
            }
        }
    }
    toks.push((Token::EOF, src.len()..src.len()));
    Ok(toks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    /// lexes `src` and returns just the token sequence (always ends with EOF).
    fn toks(src: &str) -> Vec<Token> {
        tokenize(src)
            .expect("lex error")
            .into_iter()
            .map(|(t, _)| t)
            .collect()
    }

    #[test]
    fn empty_program() {
        assert_eq!(toks(""), [EOF]);
    }

    #[test]
    fn full_program() {
        assert_eq!(
            toks("let f(x, y) = if x then f(y, false) else true"),
            [
                Let,
                Ident("f".into()),
                LeftParen,
                Ident("x".into()),
                Comma,
                Ident("y".into()),
                RightParen,
                Equals,
                If,
                Ident("x".into()),
                Then,
                Ident("f".into()),
                LeftParen,
                Ident("y".into()),
                Comma,
                False,
                RightParen,
                Else,
                True,
                EOF,
            ]
        );
    }

    #[test]
    fn nested_if() {
        assert_eq!(
            toks("if true then if false then true else false else true"),
            [
                If, True, Then, If, False, Then, True, Else, False, Else, True, EOF
            ]
        );
    }

    #[test]
    fn keyword_prefixes_lex_as_ident() {
        // the identifier regex always wins when it matches more characters
        assert_eq!(toks("letter"), [Ident("letter".into()), EOF]);
        assert_eq!(
            toks("iffy then true"),
            [Ident("iffy".into()), Then, True, EOF]
        );
        assert_eq!(toks("truest"), [Ident("truest".into()), EOF]);
        assert_eq!(toks("elsewhere"), [Ident("elsewhere".into()), EOF]);
    }

    #[test]
    fn valid_idents() {
        assert_eq!(toks("_x1"), [Ident("_x1".into()), EOF]);
        assert_eq!(toks("Foo9_bar"), [Ident("Foo9_bar".into()), EOF]);
    }

    #[test]
    fn punctuation_needs_no_whitespace() {
        assert_eq!(
            toks("f(x)"),
            [
                Ident("f".into()),
                LeftParen,
                Ident("x".into()),
                RightParen,
                EOF
            ]
        );
        assert_eq!(
            toks("((x))"),
            [
                LeftParen,
                LeftParen,
                Ident("x".into()),
                RightParen,
                RightParen,
                EOF
            ]
        );
    }

    #[test]
    fn whitespace_kinds_are_skipped() {
        assert_eq!(
            toks("let\t\r\n\x0C f = true"),
            [Let, Ident("f".into()), Equals, True, EOF]
        );
    }

    #[test]
    fn reports_lex_error_with_absolute_span() {
        let err = tokenize("let f(x) = @").expect_err("expected lex error");
        assert_eq!(err.found, "@");
        assert_eq!(err.span, 11..12);
    }

    #[test]
    fn reports_lex_error_after_valid_tokens() {
        let err = tokenize("let f(x) = true\nlet g = %").expect_err("expected lex error");
        assert_eq!(err.found, "%");
        assert_eq!(err.span, 24..25);
    }

    #[test]
    fn spans_are_increasing_and_only_skip_whitespace() {
        let src = "let f ( x , y ) = if x then f ( y , false ) else true";
        let lexed = tokenize(src).expect("lex error");

        let mut prev_end = 0;
        for (tok, span) in &lexed {
            assert!(span.start >= prev_end, "spans overlap: {span:?}");
            // the skipped text between consecutive tokens must be whitespace only
            assert!(
                src[prev_end..span.start]
                    .bytes()
                    .all(|b| b.is_ascii_whitespace()),
                "non-whitespace skipped before {tok:?} at {span:?}"
            );
            prev_end = span.end;
        }
    }

    #[test]
    fn spans_slice_back_to_source_text() {
        let src = "let f(x) = if b then true else false";
        for (tok, span) in tokenize(src).expect("lex error") {
            let text = &src[span.clone()];
            match tok {
                Let => assert_eq!(text, "let"),
                If => assert_eq!(text, "if"),
                Then => assert_eq!(text, "then"),
                Else => assert_eq!(text, "else"),
                True => assert_eq!(text, "true"),
                False => assert_eq!(text, "false"),
                LeftParen => assert_eq!(text, "("),
                RightParen => assert_eq!(text, ")"),
                Equals => assert_eq!(text, "="),
                Comma => assert_eq!(text, ","),
                Ident(s) => assert_eq!(text, s),
                EOF => assert!(text.is_empty(), "EOF span at {span:?} is not empty"),
            }
        }
    }
}
