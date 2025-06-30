use anyhow::Context;
use errors::RunError;
use scanner::Scanner;
use std::{io::Write, path::Path};

mod errors;
mod scanner;

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
        print!("> ");

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
            Err(RunError::ScannError(err)) => eprintln!("{err}"),
        }
    }
}

fn run(content: String) -> Result<(), RunError> {
    let mut scanner = Scanner::new(content);
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{token}");
    }

    Ok(())
}
