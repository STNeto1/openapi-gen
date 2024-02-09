use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type DefinitionMap = HashMap<String, Definition>;
pub type OperationResponseMap = HashMap<String, ResponsePayload>;
pub type DefinitionPropertyMap = HashMap<String, DefinitionProperty>;
type KV = HashMap<String, String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Schema {
    schemes: Vec<String>,
    host: String,
    #[serde(rename = "basePath")]
    base_path: String,
    pub paths: HashMap<String, Path>,
    pub definitions: DefinitionMap,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Path {
    pub get: Option<Operation>,
    post: Option<Operation>,
    put: Option<Operation>,
    delete: Option<Operation>,
    patch: Option<Operation>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Operation {
    description: String,
    #[serde(rename = "operationId")]
    pub operation_id: String,
    pub parameters: Vec<OperationParameter>,
    pub responses: OperationResponseMap,
}

impl Operation {
    pub fn parse_query(&self) -> String {
        let mut builder = String::new();

        self.parameters
            .iter()
            .filter_map(|p| match p.in_field.clone() {
                OperationParameterField::Query => Some(p),
                _ => None,
            })
            .for_each(|param| {
                if !builder.is_empty() {
                    builder.push_str(", ");
                }

                builder.push_str(param.name.as_str());
                builder.push_str(": ");
                builder.push_str(self.parse_inner_type(param).as_str());
            });

        return builder;
    }

    pub fn parse_path(&self) -> String {
        let mut builder = String::new();

        self.parameters
            .iter()
            .filter_map(|p| match p.in_field.clone() {
                OperationParameterField::Path => Some(p),
                _ => None,
            })
            .for_each(|param| {
                if !builder.is_empty() {
                    builder.push_str(", ");
                }

                builder.push_str(param.name.as_str());
                builder.push_str(": ");

                builder.push_str(self.parse_inner_type(param).as_str());
            });

        return builder;
    }

    fn parse_inner_type(&self, operation: &OperationParameter) -> String {
        let mut tokens = Vec::with_capacity(2);

        match (&operation.type_field, operation.required) {
            (None, None) => tokens.push("never"),
            (None, Some(_)) => tokens.push("never"),
            (Some(field), None) => {
                match field {
                    OperationParameterType::String => tokens.push("string"),
                    OperationParameterType::Integer => tokens.push("number"),
                    OperationParameterType::Boolean => tokens.push("boolean"),
                    _ => todo!("[not impl] -> {:?}", field),
                }

                tokens.push("undefined");
            }
            (Some(field), Some(is_req)) => {
                match field {
                    OperationParameterType::String => tokens.push("string"),
                    OperationParameterType::Integer => tokens.push("number"),
                    OperationParameterType::Boolean => tokens.push("boolean"),
                    _ => todo!("[not impl] -> {:?}", field),
                }

                if !is_req {
                    tokens.push("undefined | null");
                }
            }
        }

        return tokens.join(" | ");
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OperationParameterType {
    #[serde(rename = "string")]
    String,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "array")]
    Array,
    #[serde(rename = "object")]
    Object,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum OperationParameterField {
    #[serde(rename = "query")]
    Query,
    #[serde(rename = "body")]
    Body,
    #[serde(rename = "path")]
    Path,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OperationParameter {
    #[serde(rename = "type")]
    pub type_field: Option<OperationParameterType>,
    description: String,
    pub name: String,
    #[serde(rename = "in")]
    pub in_field: OperationParameterField,
    pub required: Option<bool>,
    #[serde(rename = "schema")]
    pub ref_field: Option<SchemaRef>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResponsePayload {
    description: String,
    pub schema: Option<SchemaRef>,
}

impl ResponsePayload {
    pub fn parse_response(&self) -> String {
        match self.schema {
            Some(ref schema) => {
                let mut builder = String::new();

                match schema.type_ref {
                    Some(ref _type) => {
                        builder.push_str("Promise<");
                        builder.push_str(clear_ref(_type).as_str());
                        builder.push_str(">");
                    }
                    None => {
                        builder.push_str("Promise<unknown>");
                    }
                }

                return builder;
            }
            None => {
                warn!("No schema found for response");
                return "Promise<unknown>".to_string();
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    type_ref: Option<String>,
    items: Option<SchemaRefItems>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SchemaRefItems {
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
                additional_properties: None,
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
                additional_properties: None,
            },
        );
        properties.insert(
            "some".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::String),
                _ref: None,
                items: None,
                additional_properties: None,
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
                additional_properties: None,
            },
        );
        properties.insert(
            "some2".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::String),
                _ref: None,
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "some3".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Boolean),
                _ref: None,
                items: None,
                additional_properties: None,
            },
        );
        properties.insert(
            "some4".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Array),
                _ref: Some("ref_type".to_string()),
                items: None,
                additional_properties: None,
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
                additional_properties: None,
            },
        );

        properties.insert(
            "some6".into(),
            DefinitionProperty {
                description: None,
                type_field: Some(DefinitionPropertyType::Object),
                _ref: Some("ref_type".to_string()),
                items: None,
                additional_properties: None,
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
                additional_properties: None,
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
                additional_properties: None,
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
                additional_properties: None,
            },
        );

        let response = create_raw_type_from_properties(&properties);

        assert!(response.contains("some:never;"));
    }
}
