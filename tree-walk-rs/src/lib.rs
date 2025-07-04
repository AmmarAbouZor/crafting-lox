use anyhow::Context;
use errors::RunError;
use parser::Parser;
use scanner::Scanner;
use std::{io::Write, path::Path};

mod ast;
mod errors;
mod parser;
mod scanner;

pub use scanner::{Token, TokenType};

pub fn run_file(path: &Path) -> anyhow::Result<()> {
    let file_content = std::fs::read_to_string(&path)
        .with_context(|| format!("Error while reading input file. Path: {}", path.display()))?;

    run(file_content)?;

    Ok(())
}

pub fn run_prompt() -> anyhow::Result<()> {
    println!("Welcome to rlox interpreter!");
    println!("To exit press <C-d> or <C-c>");
    let mut content = String::new();
    loop {
        content.clear();
        print!(">>> ");

        std::io::stdout()
            .flush()
            .context("Error while flushing stdout")?;

        let read = std::io::stdin()
            .read_line(&mut content)
            .context("Error while reading from stdin")?;

        if read == 0 {
            println!("Bye Bye!");
            return Ok(());
        }

        match run(content.clone()) {
            Ok(()) => {}
            Err(RunError::Unrecoverable(err)) => return Err(err),
            // Don't stop on Scanning errors
            Err(err @ RunError::Scann(_)) => eprintln!("{err}"),
        }
    }
}

fn run(content: String) -> Result<(), RunError> {
    let scanner = Scanner::new(content);
    let scan_res = scanner.scan_tokens();

    println!("Tokens:");
    for token in &scan_res.tokens {
        println!("  {token}");
    }

    println!("-------------------------------------------");

    let errors_count = scan_res.errors.len();

    if errors_count > 0 {
        println!("Errors: ");
        for err in scan_res.errors {
            eprintln!("  {err}");
        }
        println!("-------------------------------------------");
        return Err(RunError::Scann(errors_count));
    }

    let mut parser = Parser::new(scan_res.tokens);

    println!("Parse Results:");
    match parser.parse() {
        Ok(expr) => println!("{}", expr.print()),
        Err(err) => eprintln!("{err}"),
    }

    Ok(())
}
