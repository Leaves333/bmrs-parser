mod lexer;

fn main() {
    let src = "let f(x, y) = if x then f(y, false) else true";
    match lexer::tokenize(src) {
        Ok(toks) => {
            for (tok, span) in toks {
                println!("{span:?} {tok:?}");
            }
        }
        Err(e) => println!("{:?}: unexpected character {:?}", e.span, e.found),
    }

    println!("---");

    match lexer::tokenize("let f(x) = @") {
        Ok(toks) => println!("{toks:?}"),
        Err(e) => println!("{:?}: unexpected character {:?}", e.span, e.found),
    }
}
