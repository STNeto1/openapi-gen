use anyhow::{bail, Ok, Result};

mod cli;
mod parser;
mod sanitizer;
mod template;

fn main() -> Result<()> {
    let matches = cli::create_cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => cli::init()?,
        Some(("generate", _)) => cli::generate()?,
        _ => bail!("No subcommand provided"),
    };

    Ok(())
}
