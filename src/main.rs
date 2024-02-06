use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{read_to_string, File};
use std::io::prelude::*;

#[derive(Debug, Deserialize, Serialize)]
struct Schema {
    schemes: Vec<String>,
    host: String,
    #[serde(rename = "basePath")]
    base_path: String,
    paths: HashMap<String, Path>,
    definitions: HashMap<String, Definition>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Path {
    get: Option<Operation>,
    post: Option<Operation>,
    put: Option<Operation>,
    delete: Option<Operation>,
    patch: Option<Operation>,
    options: Option<Operation>,
    head: Option<Operation>,
    trace: Option<Operation>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Operation {
    description: String,
    parameters: Vec<OperationParameter>,
    responses: HashMap<String, ResponsePayload>,
}

#[derive(Debug, Deserialize, Serialize)]
struct OperationParameter {
    #[serde(rename = "type")]
    type_field: Option<String>,
    description: String,
    name: String,
    #[serde(rename = "in")]
    in_field: String,
    required: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ResponsePayload {
    description: String,
    schema: Option<SchemaRef>,
}

#[derive(Debug, Deserialize, Serialize)]
struct SchemaRef {
    #[serde(rename = "$ref")]
    _ref: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Definition {
    #[serde(rename = "type")]
    type_field: String,
    properties: Option<HashMap<String, DefinitionProperty>>,
    #[serde(rename = "enum")]
    _enum: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DefinitionProperty {
    description: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<String>,
    #[serde(rename = "$ref")]
    _ref: Option<String>,
    items: Option<HashMap<String, String>>,
}

fn main() {
    let file = read_to_string("example.json").expect("to read file");

    let schema: Schema = serde_json::from_str(&file).expect("to parse the json");

    let json_str = serde_json::to_string_pretty(&schema).expect("to create string");
    let mut file = File::create("data.json").expect("Failed to create file");
    file.write_all(json_str.as_bytes())
        .expect("Failed to write to file");
}
