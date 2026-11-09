use std::error::Error;

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
    println!("{}", prog.get_body());

    Ok(())
}