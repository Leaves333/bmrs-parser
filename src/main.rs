use clap::Parser;
use std::{
    collections::{HashMap, HashSet},
    fs::read,
    path::PathBuf,
};

use bmrs_parser::{
    eval, lexer, parser,
    eval::{CacheKey, StringModel, Value},
    lexer::Token::Hash,
    parser::Program,
};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    source: PathBuf,
    input: String,
}

struct Table {
    rows: HashMap<String, Vec<Value>>,
    num_columns: usize,
}

/// column width of the printed table
const COL_WIDTH: usize = 3;

/// render `s` centered in `COL_WIDTH` columns
fn center(s: &str) -> String {
    let pad = COL_WIDTH.saturating_sub(s.chars().count());
    " ".repeat(pad.div_ceil(2)) + s + &" ".repeat(pad / 2)
}

impl Table {
    fn new(program: &Program, model: &StringModel, results: &HashMap<CacheKey, Value>) -> Self {
        let top_level_defs: HashSet<_> = program
            .defs
            .iter()
            .filter(|def| def.top_level)
            .map(|def| def.function.clone())
            .collect();

        let mut rows = HashMap::new();
        for function in top_level_defs {
            let row = (0..model.len())
                .map(|i| {
                    let key = CacheKey {
                        function: function.clone(),
                        args: vec![Value::Pos(i)],
                    };
                    *results
                        .get(&key)
                        .expect("missing cached result for top-level def")
                })
                .collect();
            rows.insert(function.clone(), row);
        }

        Self {
            rows,
            num_columns: model.len(),
        }
    }

    fn print(&self) {
        // NOTE: prints a table like
        //     | 1  2  3  4 ... 12 13
        // ----|------------   ------
        //  a' | T  T  T  F
        // bar | F  T  F  F

        let max_name_len = self
            .rows
            .keys()
            .fold(0, |acc, name| acc.max(name.chars().count()));

        // print header of domain elements
        println!(
            "{} |{}",
            " ".repeat(max_name_len),
            (1..=self.num_columns)
                .map(|i| center(&i.to_string()))
                .collect::<String>()
        );

        // separator of hyphens
        println!(
            "{}|{}",
            "-".repeat(max_name_len + 1),
            "-".repeat(COL_WIDTH * self.num_columns)
        );

        // print each row
        let mut names: Vec<&String> = self.rows.keys().collect();
        names.sort();
        for name in names {
            print!("{name:>max_name_len$} |");
            for value in &self.rows[name] {
                print!("{}", center(&value.to_string()));
            }
            println!();
        }
    }
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
    let results = eval::eval_program(&program, &model)
        .unwrap_or_else(|e| panic!("invalid program: encountered runtime error\n{e:?}"));

    let table = Table::new(&program, &model, &results);
    table.print();
}
