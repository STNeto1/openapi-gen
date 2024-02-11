use anyhow::{Context, Ok, Result};
use clap::Command;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use url::Url;

use crate::{parser, template};

pub fn create_cli() -> Command {
    Command::new("api-gen")
        .about("Generate typescript code from OpenAPI spec")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .allow_external_subcommands(true)
        .subcommand(Command::new("init").about("Initialize a new project"))
        .subcommand(
            Command::new("generate")
                .about("Generate the output file")
                .alias("g"),
        )
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    source: String,
    path: String,
}

impl Config {
    fn get_data(&self) -> Result<String> {
        if Url::parse(&self.source).is_ok() {
            let body = reqwest::blocking::get(&self.source)
                .context("Failed to fetch data from URL")?
                .text()
                .context("Failed to read response body")?;

            return Ok(body);
        };

        let file_contents = std::fs::read_to_string(&self.source).context("Unable to read file")?;

        Ok(file_contents)
    }
}

pub fn init() -> Result<()> {
    let config = Config {
        source: "__REPLACE__".to_string(),
        path: "lib/types.ts".to_string(),
    };

    let config_str = serde_json::to_string_pretty(&config).context("Failed to create file data")?;
    std::fs::write("api-gen.json", config_str).context("Failed to write to file")?;

    Ok(())
}

pub fn generate() -> Result<()> {
    let input_file = std::fs::read_to_string("api-gen.json").context("Unable to read file")?;

    let config: Config = serde_json::from_str(&input_file).context("Unable to parse JSON")?;

    let schema: parser::Schema =
        serde_json::from_str(config.get_data()?.as_str()).context("Unable to parse JSON")?;

    let folder_path = config
        .path
        .split("/")
        .filter(|f| !f.ends_with(".ts"))
        .collect::<Vec<_>>()
        .join("/");

    std::fs::create_dir_all(folder_path).context("Unable to create directory")?;
    let mut output_file = std::fs::File::create(&config.path).context("Unable to create file")?;

    template::generate_file_lines(schema)
        .iter()
        .for_each(|line| {
            output_file
                .write_all(line.as_bytes())
                .expect("Unable to write data");
        });

    Ok(())
}
