use log::warn;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::parser;
use crate::sanitizer;

pub fn generate_file_lines(schema: parser::Schema) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();

    generate_baselines(&mut lines);
    generate_definition_types(&schema, &mut lines);

    schema.paths.iter().for_each(|(key, value)| {
        match &value.get {
            Some(get) => generate_fetcher(key, get.clone(), &mut lines),
            None => (),
        }

        match &value.post {
            Some(post) => generate_mutator(key, &"POST".to_string(), post.clone(), &mut lines),
            None => (),
        }

        // if value.get.is_none() {
        //     warn!("No get method for {}", key);
        //     return;
        // }

        // generate_fetcher(key, value.get.clone().unwrap(), &mut lines)
    });

    return lines;
}

fn generate_fn_name(method: String, path: String) -> String {
    let mut fn_name = method.to_lowercase();
    fn_name.push_str("_");

    path.split("/").filter(|&x| !x.is_empty()).for_each(|x| {
        if x.starts_with("{") && x.ends_with("}") {
            fn_name.push_str("by_");
        } else {
            fn_name.push_str(x);
            fn_name.push_str("_");
        }
    });

    if fn_name.ends_with("_") {
        fn_name = fn_name.trim_end_matches('_').to_string();
    }

    fn_name.replace("-", "_")
}

fn generate_baselines(lines: &mut Vec<String>) {
    lines.push(format!(
        r#"
type Record = {{ [key: string]: string | number | boolean | undefined }};
type Params = {{ query: Record; path: Record }};

function createUrl(url: string, params: Params) {{
	const _url = Object.keys(params || {{}}).reduce(
		(acc, key) => acc.replace(`{{${{key}}}}`, params[key]),
		url,
	);

	const completeUrl = new URL(_url);
	Object.keys(params.query).forEach((key) => {{
		completeUrl.searchParams.append(key, params.query[key]?.toString() ?? "");
	}});

	return completeUrl;
}}

async function fetcher<TResult, TErr>(
  url: string,
  params: {{ query: Record; path: Record }},
  init?: RequestInit,
) {{

const _init = init ? {{ ...init, method: "GET" }} : {{ method: "GET" }};
const _url = Object.keys(params || {{}}).reduce(
	(acc, key) => acc.replace(`{{$key}}`, params[key]),
	url,
);

const completeUrl = new URL(_url);
Object.keys(params.query).forEach((key) => {{
	const val = params.query[key];
	if (!val) {{
		return;
	}}

	completeUrl.searchParams.append(key, val.toString());
}});

  const res = await fetch(completeUrl, _init);
  const bodyData = await res.json();

  if (!res.ok) {{
    return bodyData as TErr;
  }}

  return bodyData as TResult;
}}


async function mutator<TBody, TResult, TErr>(
	method: "POST" | "PUT" | "DELETE" | "PATCH",
	url: string,
	params: Params,
	body: TBody | null,
	init?: RequestInit,
) {{
	const _init = Object.assign(init ?? {{}}, {{
		method,
		body: body ? JSON.stringify(body) : undefined,
	}});

	const res = await fetch(createUrl(url, params), _init);
	const bodyData = await res.json();

	if (!res.ok) {{
		return bodyData as TErr;
	}}

	return bodyData as TResult;
}}

"#
    ));
}

fn generate_definition_types(schema: &parser::Schema, lines: &mut Vec<String>) {
    let mut result: HashMap<String, String> = HashMap::new();

    schema.definitions.iter().for_each(|(key, value)| {
        let parsed_key = parser::normalize_key(key);

        let raw_type = if value._enum.is_some() {
            parser::parse_enum(value)
        } else {
            parser::create_raw_type_from_properties(&value.properties.clone().unwrap_or_default())
        };

        result.insert(parsed_key, raw_type);
    });

    // sort the keys from the hashmap
    let mut keys: Vec<&String> = result.keys().collect();
    keys.sort();

    keys.iter().for_each(|key| {
        lines.push(format!(
            "export type {} = {};\n",
            key,
            result.get(*key).unwrap()
        ));
    });
}

fn generate_fetcher(key: &String, op: parser::Operation, lines: &mut Vec<String>) {
    let _2xx_responses = op
        .responses
        .iter()
        .filter(|(key, _)| key.starts_with("2"))
        .collect::<Vec<_>>();
    let non_2xx_responses = op
        .responses
        .iter()
        .filter(|(key, _)| key.starts_with("4") || key.starts_with("5"))
        .collect::<Vec<_>>();

    let query_type = op.parse_query();
    let path_type = op.parse_path();

    let tmp_key = sanitizer::create_input_type_name_from_path(key, None);
    let fn_name = generate_fn_name("get".to_string(), key.to_string());

    lines.push(format!(
        "\n\ntype {tmp_key} = {{ query: {{{query_type}}}, path: {{{path_type}}} }};\n"
    ));

    lines.push(format!(
        "type {fn_name}_response = {};\n",
        _2xx_responses
            .iter()
            .map(|(_, value)| value.parse_response())
            .fold(HashSet::new(), |mut acc, x| {
                acc.insert(x);
                acc
            })
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    lines.push(format!(
        "type {fn_name}_error = {};\n",
        if non_2xx_responses.is_empty() {
            "never".to_string()
        } else {
            non_2xx_responses
                .iter()
                .map(|(_, value)| value.parse_response())
                .fold(HashSet::new(), |mut acc, x| {
                    acc.insert(x);
                    acc
                })
                .iter()
                .map(|val| val.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        }
    ));
    lines.push(format!(
        r#"export async function get_{fn_name}(props: {tmp_key}) {{
    return fetcher<{fn_name}_response, {fn_name}_error>("{key}", props);
}}
"#
    ));
}

fn generate_mutator(key: &String, method: &String, op: parser::Operation, lines: &mut Vec<String>) {
    let _2xx_responses = op
        .responses
        .iter()
        .filter(|(key, _)| key.starts_with("2"))
        .collect::<Vec<_>>();
    let non_2xx_responses = op
        .responses
        .iter()
        .filter(|(key, _)| key.starts_with("4") || key.starts_with("5"))
        .collect::<Vec<_>>();

    let query_type = op.parse_query();
    let path_type = op.parse_path();

    let tmp_key = sanitizer::create_input_type_name_from_path(key, Some(&method.to_lowercase()));
    let fn_name = generate_fn_name(method.clone(), key.to_string());

    lines.push(format!(
        "\n\ntype {tmp_key} = {{ query: {{{query_type}}}, path: {{{path_type}}} }};\n"
    ));

    lines.push(format!(
        "type {fn_name}_response = {};\n",
        _2xx_responses
            .iter()
            .map(|(_, value)| value.parse_response())
            .fold(HashSet::new(), |mut acc, x| {
                acc.insert(x);
                acc
            })
            .iter()
            .map(|val| val.to_string())
            .collect::<Vec<_>>()
            .join(" | ")
    ));
    lines.push(format!(
        "type {fn_name}_error = {};\n",
        if non_2xx_responses.is_empty() {
            "never".to_string()
        } else {
            non_2xx_responses
                .iter()
                .map(|(_, value)| value.parse_response())
                .fold(HashSet::new(), |mut acc, x| {
                    acc.insert(x);
                    acc
                })
                .iter()
                .map(|val| val.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        }
    ));

    match op.parameters {
        Some(parameters) => {
            let body_type = parameters
                .iter()
                .find(|x| match x.in_field {
                    parser::OperationParameterField::Body => true,
                    _ => false,
                })
                .map(|x| x.parse_body())
                .unwrap_or("never".to_string());

            lines.push(format!("type {fn_name}_body= {body_type};\n",));

            lines.push(format!(
                r#"export async function {fn_name}(props: {tmp_key}, body: {fn_name}_body, init?: RequestInit) {{
    return mutator<{fn_name}_body, {fn_name}_response, {fn_name}_error>("{method}", "{key}", props, body, init);
}}
"#
            ));
        }
        None => {
            lines.push(format!(
                r#"export async function {fn_name}(props: {tmp_key}, init?: RequestInit) {{
    return mutator<never, {fn_name}_response, {fn_name}_error>("{method}", "{key}", props, null, init);
}}
"#
            ));
        }
    }
}
