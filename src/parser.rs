use crate::lexer::Token;

struct Program {
    defs: Vec<Def>,
}

struct Def {
    function: String,
    params: Vec<String>,
    expr: Expr,
}

enum Expr {
    IfThenElse {
        if_clause: Box<Expr>,
        then_clause: Box<Expr>,
        else_clause: Box<Expr>,
    },
    Application {
        function: String,
        args: Vec<Expr>,
    },
    Variable(String),
    Bool(bool),
}

struct ParseError {
    message: String,
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    // returns the next token
    fn peek(&self) -> &Token {
        &self.tokens[self.cursor]
    }

    // returns the token two after this one
    fn peek_two(&self) -> &Token {
        if self.cursor + 1 >= self.tokens.len() {
            &Token::EOF
        } else {
            &self.tokens[self.cursor + 1]
        }
    }

    // advances the cursor by one token
    fn advance(&mut self) {
        if self.cursor + 1 < self.tokens.len() {
            self.cursor += 1;
        }
    }

    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        todo!()
    }
}
