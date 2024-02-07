use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type DefinitionMap = HashMap<String, Definition>;
type OperationResponseMap = HashMap<String, ResponsePayload>;
type DefinitionPropertyMap = HashMap<String, DefinitionProperty>;
type KV = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Schema {
    schemes: Vec<String>,
    host: String,
    #[serde(rename = "basePath")]
    base_path: String,
    paths: HashMap<String, Path>,
    definitions: DefinitionMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Operation {
    description: String,
    parameters: Vec<OperationParameter>,
    responses: OperationResponseMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct OperationParameter {
    #[serde(rename = "type")]
    type_field: Option<String>,
    description: String,
    name: String,
    #[serde(rename = "in")]
    in_field: String,
    required: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ResponsePayload {
    description: String,
    schema: Option<SchemaRef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SchemaRef {
    #[serde(rename = "$ref")]
    _ref: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
enum DefinitionType {
    #[serde(rename = "object")]
    Object,
    #[serde(rename = "string")]
    String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Definition {
    #[serde(rename = "type")]
    type_field: DefinitionType,
    properties: Option<DefinitionPropertyMap>,
    #[serde(rename = "enum")]
    _enum: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
enum DefinitionPropertyType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "object")]
    Object,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct DefinitionProperty {
    description: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<DefinitionPropertyType>,
    #[serde(rename = "$ref")]
    _ref: Option<String>,
    items: Option<KV>,
}

fn encode_kv_to_ts_object(kv: &KV) -> String {
    let mut res = String::new();

    kv.iter().for_each(|(key, value)| {
        res.push_str(format!("{}:{};", key, value).as_str());
    });

    return res;
}

fn create_raw_type_from_properties(props: &DefinitionPropertyMap) -> String {
    let mut res = String::new();
    res.push_str("{");

    props.iter().for_each(|(key, _value)| {
        let mut inner = String::new();

        inner.push_str(format!("{}:", key).as_str());

        if _value.type_field.is_some() {
            match _value.type_field.clone().unwrap() {
                DefinitionPropertyType::String => inner.push_str("string;"),
                DefinitionPropertyType::Integer => inner.push_str("number;"),
                DefinitionPropertyType::Boolean => inner.push_str("boolean;"),
                DefinitionPropertyType::Array => {
                    if _value._ref.is_some() {
                        inner.push_str(format!("{}[];", _value._ref.clone().unwrap()).as_str())
                    }

                    if _value.items.is_some() {
                        let encoded = encode_kv_to_ts_object(&_value.items.clone().unwrap());

                        inner.push_str(format!("{{{}}}[];", encoded).as_str());
                    }

                    if _value._ref.is_none() && _value.items.is_none() {
                        warn!("Array type without ref or items");
                    }
                }
                DefinitionPropertyType::Object => {
                    dbg!(_value);
                    inner.push_str("object");
                }
            }
        }

        res.push_str(&inner);
    });

    res.push_str("}");

    return res;
}

// create test block
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_basic_schema() {
        let schema = Schema {
            schemes: vec!["https".to_string()],
            host: "api.example.com".to_string(),
            base_path: "/v1".to_string(),
            paths: HashMap::new(),
            definitions: HashMap::new(),
        };

        assert!(schema.schemes.len() > 0);
    }

    #[test]
    fn parse_basic_object_definition() {
        let mut properties: DefinitionPropertyMap = HashMap::new();
        properties.insert(
            "blocks".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Integer),
                _ref: None,
                items: None,
            },
        );

        let response = create_raw_type_from_properties(&properties);
        let expected = "{blocks:number;}";

        assert_eq!(response, expected);
    }

    #[test]
    fn parse_multiple_properties_object_definition() {
        let mut properties: DefinitionPropertyMap = HashMap::new();
        properties.insert(
            "blocks".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Integer),
                _ref: None,
                items: None,
            },
        );
        properties.insert(
            "some".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::String),
                _ref: None,
                items: None,
            },
        );

        let response = create_raw_type_from_properties(&properties);

        assert!(response.contains("some:string;"));
        assert!(response.contains("blocks:number;"));
    }

    #[test]
    fn parse_all_possible_properties_object_definition() {
        let mut properties: DefinitionPropertyMap = HashMap::new();
        properties.insert(
            "some1".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Integer),
                _ref: None,
                items: None,
            },
        );
        properties.insert(
            "some2".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::String),
                _ref: None,
                items: None,
            },
        );
        properties.insert(
            "some3".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Boolean),
                _ref: None,
                items: None,
            },
        );
        properties.insert(
            "some4".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Array),
                _ref: Some("ref_type".to_string()),
                items: None,
            },
        );

        let some_5_items: KV = [("some".to_string(), "string".to_string())]
            .iter()
            .cloned()
            .collect();
        properties.insert(
            "some5".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Array),
                _ref: None,
                items: Some(some_5_items),
            },
        );

        let response = create_raw_type_from_properties(&properties);

        // println!("{}", response);

        assert!(response.contains("some1:number;"));
        assert!(response.contains("some2:string;"));
        assert!(response.contains("some3:boolean;"));
        assert!(response.contains("some4:ref_type[];"));
        assert!(response.contains("some5:{some:string;}[];"));
    }
}
