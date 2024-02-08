use std::collections::HashMap;
use std::fs::{self, File};
use std::io::prelude::*;

mod parser;
mod sanitizer;

fn main() {
    let input_file = fs::read_to_string("example.json").expect("Unable to read file");
    // let mut output_file = File::create("types.ts").expect("Unable to create file");
    // let mut fetcher_output_file = File::create("fetcher.ts").expect("Unable to create file");

    let schema: parser::Schema = serde_json::from_str(&input_file).expect("Unable to parse JSON");

    // let mut result: HashMap<String, String> = HashMap::new();

    schema.paths.iter().for_each(|(key, value)| {
        if key == "/apps" || key == "/apps/{app_name}/volumes/{volume_id}" {
            let _get = value.get.clone().unwrap();

            let query_type = if value.get.is_some() {
                _get.parse_query()
            } else {
                unimplemented!("No query type for {}", key)
            };

            let path_type = if value.get.is_some() {
                _get.parse_path()
            } else {
                unimplemented!("No path type for {}", key)
            };

            let tmp_key = sanitizer::create_input_type_name_from_path(key);
            let fn_name = _get.operation_id;
            println!("type {tmp_key} = {{ query: {{{query_type}}}, path: {{{path_type}}} }};");
            println!(
                r#"export async function get_{fn_name}(props: {tmp_key}) {{
    return fetcher<unknown, unknown>("{key}", props);
}}"#
            );
            println!("");
        }
    });

    // let mut keys: Vec<&String> = result.keys().collect();
    // keys.sort();
    //
    // keys.iter().for_each(|key| {
    //     fetcher_output_file
    //         .write_all(format!("export type {} = {};\n", key, result.get(*key).unwrap()).as_bytes())
    //         .expect("Unable to write data");
    // });
}

fn main_2() {
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
