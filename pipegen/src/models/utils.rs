use crate::models::constants::INDENT_LITERAL;

pub fn indent_literal(indent: usize) -> String {
    repeat_string(INDENT_LITERAL, indent)
}

pub fn repeat_string(origin: &str, repeat: usize) -> String {
    origin.repeat(repeat)
}
