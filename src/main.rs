use logos::Logos;
use std::ops::Range;

#[derive(Debug, Clone, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
enum Token {
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
    /// Never produced by the lexer (logos ignores variants without a
    /// `#[token]`/`#[regex]` attribute); `tokenize` appends it when the
    /// input is exhausted, so the parser can treat EOF as a lookahead token.
    Eof,
}

#[derive(Debug)]
struct LexError {
    span: Range<usize>,
    found: String,
}

/// tokenizes program source code `src` into a vec of tokens.
/// appends `Token::Eof` to the end of the stream.
fn tokenize(src: &str) -> Result<Vec<(Token, Range<usize>)>, LexError> {
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
    toks.push((Token::Eof, src.len()..src.len()));
    Ok(toks)
}

fn main() {
    let src = "let f(x, y) = if x then f(y, false) else true";
    match tokenize(src) {
        Ok(toks) => {
            for (tok, span) in toks {
                println!("{span:?} {tok:?}");
            }
        }
        Err(e) => println!("{:?}: unexpected character {:?}", e.span, e.found),
    }

    println!("---");

    match tokenize("let f(x) = @") {
        Ok(toks) => println!("{toks:?}"),
        Err(e) => println!("{:?}: unexpected character {:?}", e.span, e.found),
    }
}
