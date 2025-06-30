use std::path::PathBuf;

use anyhow::bail;
use tree_walk_rs::{run_file, run_prompt};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    match args.len() {
        0 => panic!("Environment arguments must starts with the path of the binary file"),
        // No args => Run interactive REPL session.
        1 => run_prompt(),
        // File provided => Use it
        2 => run_file(&PathBuf::from(&args[1])),
        // We don't support more handling more than one file.
        _ => bail!("Usage: rlox [script]"),
    }
}
