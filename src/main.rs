use anyhow::{Ok, Result};
use std::fs::{self, create_dir_all, File};
use std::io::prelude::*;

mod parser;
mod sanitizer;
mod template;

fn main() -> Result<()> {
    let input_file = fs::read_to_string("input.json").expect("Unable to read file");

    let _path = "lib/types.ts";
    let folder_path = _path
        .split("/")
        .filter(|f| !f.ends_with(".ts"))
        .collect::<Vec<_>>()
        .join("/");

    create_dir_all(folder_path)?;
    let mut output_file = File::create("lib/types.ts").expect("Unable to create file");

    let schema: parser::Schema = serde_json::from_str(&input_file).expect("Unable to parse JSON");

    template::generate_file_lines(schema)
        .iter()
        .for_each(|line| {
            output_file
                .write_all(line.as_bytes())
                .expect("Unable to write data");
        });

    Ok(())
}
