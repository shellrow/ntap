pub fn node_label(label: &str, value: Option<&str>, delimiter: Option<&str>) -> String {
    match value {
        Some(value) => {
            let delimiter = delimiter.unwrap_or(":");
            format!("{}{} {}", label, delimiter, value)
        }
        None => label.to_string(),
    }
}
