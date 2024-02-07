use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;

mod parser;

fn main() {
    let input_file = fs::read_to_string("example.json").expect("Unable to read file");
    let mut output_file = File::create("types.ts").expect("Unable to create file");

    let schema: parser::Schema = serde_json::from_str(&input_file).expect("Unable to parse JSON");

    let mut result: HashMap<String, String> = HashMap::new();

    schema.definitions.iter().for_each(|(key, value)| {
        let parsed_key = parser::normalize_key(key);
        let raw_type = if value._enum.is_some() {
            parser::parse_enum(value)
        } else {
            parser::create_raw_type_from_properties(&value.properties.clone().unwrap_or_default())
        };

        // if parsed_key == "main_statusCode" {
        //     println!("{}: {} -> {:?}", parsed_key, raw_type, value);
        // }

        result.insert(parsed_key, raw_type);
    });

    // sort the keys from the hashmap
    let mut keys: Vec<&String> = result.keys().collect();
    keys.sort();

    keys.iter().for_each(|key| {
        output_file
            .write_all(format!("export type {} = {};\n", key, result.get(*key).unwrap()).as_bytes())
            .expect("Unable to write data");
    });
}
