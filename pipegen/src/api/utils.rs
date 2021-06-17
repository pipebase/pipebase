use crate::api::constants::INDENT_LITERAL;

pub fn indent_literal(indent: usize) -> String {
    repeat_string(INDENT_LITERAL, indent)
}

pub fn repeat_string(origin: &str, repeat: usize) -> String {
    std::iter::repeat(origin).take(repeat).collect::<String>()
}

pub fn snake_to_camel(s: &str) -> String {
    let mut buffer = String::new();
    let mut uppercase = true;
    for char in s.chars() {
        if char == '_' {
            uppercase = true;
            continue;
        }
        if uppercase {
            buffer.push(char.to_ascii_uppercase());
            uppercase = !uppercase;
            continue;
        }
        buffer.push(char)
    }
    buffer
}
