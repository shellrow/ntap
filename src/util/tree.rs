pub fn node_label(label: &str, value: Option<&str>, delimiter: Option<&str>) -> String {
    match value {
        Some(value) => {
            let delimiter = match delimiter {
                Some(delimiter) => delimiter,
                None => ":",
            };
            format!("{}{} {}", label, delimiter, value)
        }
        None => label.to_string(),
    }
}
