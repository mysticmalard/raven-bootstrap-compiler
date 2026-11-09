use std::error::Error;

use crate::preprocessor::lexer::Lexer;

mod preprocessor;


fn main() -> Result<(), Box<dyn Error>> {
    let source_file = match std::env::args().nth(1) {
        Some(file) => file,
        None => {
            eprintln!("Usage: compiler <source-file>");
            std::process::exit(1);
        }
    };

    let prog = preprocessor::collector::Collector::new(source_file)?;
    println!("{}\n", prog.get_body());

    let tokens = Lexer::new(&prog);
    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}