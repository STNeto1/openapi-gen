pub fn create_input_type_name_from_path(path: &String, prefix: Option<&String>) -> String {
    let clear = path
        .replace("/", "_")
        .replace("{", "")
        .replace("}", "")
        .replace("-", "_");
    let mut result = String::new();
    result.push_str(format!("{}", prefix.unwrap_or(&String::new())).as_str());
    result.push_str(&clear);
    result.push_str("_Params");
    result
}
