use std::fs::{self, File};
use std::io::prelude::*;

mod parser;
mod sanitizer;
mod template;

fn main() {
    let input_file = fs::read_to_string("example.json").expect("Unable to read file");
    let mut output_file = File::create("types.ts").expect("Unable to create file");

    let schema: parser::Schema = serde_json::from_str(&input_file).expect("Unable to parse JSON");

    template::generate_file_lines(schema)
        .iter()
        .for_each(|line| {
            output_file
                .write_all(line.as_bytes())
                .expect("Unable to write data");
        });
}
