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
/// appends `Token::Eof` to the end of the stream.
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
