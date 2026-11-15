#![allow(dead_code, unused_variables, unused_imports)]

use std::{error::Error, path::PathBuf};

use crate::preprocessor::lexer::Lexer;

mod preprocessor;


fn parse_file(source_file: PathBuf) -> Result<bool, Box<dyn Error>> {
    dbg!(&source_file);

    let prog = preprocessor::collector::Collector::new(source_file)?;
    println!("{}\n", prog.get_body());

    // let tokens = Lexer::new(prog.get_body());
    // let mut index = 0;
    // for token in tokens {
    //     println!("{}: {:?}", index, token);
    //     index += 1;
    // }

    let parser = preprocessor::parser::Parser::new(prog.get_body());
    for res in parser {
        match res {
            Ok(stmt) => println!("{:?}", stmt),
            Err(err) => {
                println!("{:?}", err);
                return Ok(true)
            }
        }
    }

    println!("\n");
    Ok(false)
}

fn main() -> Result<(), Box<dyn Error>> {
    match std::env::args().nth(1) {
        Some(file) => {
            if file.ends_with('*') {
                let mut path = file.as_str().chars();
                path.next_back();
                let paths = std::fs::read_dir(path.as_str())?;
                for source_path in paths {
                    if parse_file(source_path?.path())? {
                        println!("Errored");
                        break;
                    } else {
                        println!("Success\n");
                    }
                }
            } else {
                parse_file(file.into())?;
            }
        },
        None => {
            eprintln!("Usage: compiler <source-file>");
            std::process::exit(1);
        }
    }

    Ok(())
}