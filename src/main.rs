use clap::Parser;
use std::{fs::read, path::PathBuf};

mod eval;
mod lexer;
mod parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    source: PathBuf,
    input: String,
}

fn main() {
    let args = Args::parse();

    let bytes = read(&args.source).expect("error while reading program file!");
    let program_source =
        String::from_utf8(bytes).expect("invalid UTF-8 formatting in program text");

    println!("program source:\n{program_source}");

    let tokens = lexer::tokenize(&program_source)
        .unwrap_or_else(|e| panic!("invalid program: encountered lexing error\n{e:?}"));

    let program = parser::Parser::new(tokens)
        .parse()
        .unwrap_or_else(|e| panic!("invalid program: encountered parsing error\n{e:?}"));

    println!(
        "running program with source:{} input:{}",
        args.source.display(),
        args.input,
    );

    let model = eval::StringModel::new(&args.input);
    let result = eval::eval_program(program, model)
        .unwrap_or_else(|e| panic!("invalid program: encountered runtime error\n{e:?}"));

    println!("{result:?}");
}
