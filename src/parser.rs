use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type DefinitionMap = HashMap<String, Definition>;
type OperationResponseMap = HashMap<String, ResponsePayload>;
pub type DefinitionPropertyMap = HashMap<String, DefinitionProperty>;
type KV = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Schema {
    schemes: Vec<String>,
    host: String,
    #[serde(rename = "basePath")]
    base_path: String,
    paths: HashMap<String, Path>,
    pub definitions: DefinitionMap,
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
pub struct Definition {
    #[serde(rename = "type")]
    type_field: DefinitionType,
    pub properties: Option<DefinitionPropertyMap>,
    #[serde(rename = "enum")]
    pub _enum: Option<Vec<String>>,
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
pub struct DefinitionProperty {
    description: Option<String>,
    #[serde(rename = "type")]
    type_field: Option<DefinitionPropertyType>,
    #[serde(rename = "$ref")]
    _ref: Option<String>,
    items: Option<KV>,
    #[serde(rename = "additionalProperties")]
    additional_properties: Option<KV>,
}

fn encode_kv_to_ts_object(kv: &KV) -> String {
    let mut res = String::new();

    kv.iter().for_each(|(key, value)| {
        res.push_str(format!("{}:{};", key, clear_ref(value)).as_str());
    });

    return res;
}

pub fn normalize_key(key: &String) -> String {
    key.replace(".", "_")
}

pub fn clear_ref(ref_string: &String) -> String {
    normalize_key(ref_string).replace("#/definitions/", "")
}

pub fn parse_enum(props: &Definition) -> String {
    match &props._enum {
        Some(values) => values
            .iter()
            .map(|v| format!("'{}'", v).to_string())
            .collect::<Vec<String>>()
            .join(" | "),
        None => "".to_string(),
    }
}

pub fn create_raw_type_from_properties(props: &DefinitionPropertyMap) -> String {
    let tokens: Vec<String> = props
        .iter()
        .map(|(key, value)| {
            //println!("{} -> {:?}", key, value);

            let mut inner = String::new();

            inner.push_str(format!("{}:", key).as_str());

            if value.type_field.is_some() {
                match value.type_field.clone().unwrap() {
                    DefinitionPropertyType::String => inner.push_str("string;"),
                    DefinitionPropertyType::Integer => inner.push_str("number;"),
                    DefinitionPropertyType::Boolean => inner.push_str("boolean;"),
                    DefinitionPropertyType::Array => {
                        if value._ref.is_some() {
                            inner.push_str(format!("{}[];", value._ref.clone().unwrap()).as_str())
                        }

                        if value.items.is_some() {
                            let items = value.items.clone().unwrap();

                            if items.contains_key("$ref") {
                                inner.push_str(
                                    format!("{}[];", clear_ref(items.get("$ref").unwrap()))
                                        .as_str(),
                                );
                            } else {
                                let encoded = encode_kv_to_ts_object(&items);

                                inner.push_str(format!("{{{}}}[];", encoded).as_str());
                            }
                        }

                        if value._ref.is_none() && value.items.is_none() {
                            warn!("Array type without ref or items");
                        }
                    }
                    DefinitionPropertyType::Object => {
                        if value._ref.is_some() {
                            inner.push_str(format!("{};", value._ref.clone().unwrap()).as_str());
                        }

                        if value.items.is_some() {
                            let encoded = encode_kv_to_ts_object(&value.items.clone().unwrap());

                            inner.push_str(format!("{{{encoded}}};").as_str());
                        }

                        if value.additional_properties.is_some() {
                            let encoded = encode_kv_to_ts_object(
                                &value.additional_properties.clone().unwrap(),
                            );

                            inner.push_str(format!("{{{encoded}}};").as_str());
                        }

                        if value._ref.is_none()
                            && value.items.is_none()
                            && value.additional_properties.is_none()
                        {
                            warn!("Object type without ref, items, additional properties");

                            inner.push_str("never;");
                        }
                    }
                }

                return inner;
            }

            if value._ref.is_some() {
                inner.push_str(clear_ref(&value._ref.clone().unwrap()).as_str());
                inner.push_str(";");

                return inner;
            }

            inner.push_str("never;");

            return inner;
        })
        .collect::<_>();

    let mut res = String::new();
    res.push_str("{");

    tokens.iter().for_each(|t| {
        res.push_str(t.as_str());
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

        properties.insert(
            "some6".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Object),
                _ref: Some("ref_type".to_string()),
                items: None,
            },
        );

        let some_7_items: KV = [("some".to_string(), "string".to_string())]
            .iter()
            .cloned()
            .collect();
        properties.insert(
            "some7".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Object),
                _ref: None,
                items: Some(some_7_items),
            },
        );

        let response = create_raw_type_from_properties(&properties);

        assert!(response.contains("some1:number;"));
        assert!(response.contains("some2:string;"));
        assert!(response.contains("some3:boolean;"));
        assert!(response.contains("some4:ref_type[];"));
        assert!(response.contains("some5:{some:string;}[];"));
        assert!(response.contains("some6:ref_type;"));
        assert!(response.contains("some7:{some:string;}"));
    }

    #[test]
    fn parse_property_with_only_ref() {
        let mut properties: DefinitionPropertyMap = HashMap::new();
        properties.insert(
            "some".into(),
            DefinitionProperty {
                description: None,
                type_field: None,
                _ref: Some("ref_type".to_string()),
                items: None,
            },
        );

        let response = create_raw_type_from_properties(&properties);

        assert!(response.contains("some:ref_type;"));
    }

    #[test]
    fn parse_property_without_nothign() {
        let mut properties: DefinitionPropertyMap = HashMap::new();
        properties.insert(
            "some".into(),
            DefinitionProperty {
                description: None,
                type_field: None,
                _ref: None,
                items: None,
            },
        );

        let response = create_raw_type_from_properties(&properties);

        assert!(response.contains("some:never;"));
    }
}
