pub fn create_input_type_name_from_path(path: &String) -> String {
    let clear = path.replace("/", "_").replace("{", "").replace("}", "");
    let mut result = clear.to_string();
    result.push_str("_Params");
    result
}
