pub fn apply_string_escapes(src: &str) -> String {
    src.replace("\\\"", "\"").replace("\\\\", "\\")
}
