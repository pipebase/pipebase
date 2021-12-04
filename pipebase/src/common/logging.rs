use tracing_subscriber;

const FULL_FORMATTER: &str = "full";
const PRETTY_FORMATTER: &str = "pretty";
const JSON_FORMATTER: &str = "json";
const ENV_FORMATTER: &str = "PIPEBASE_LOG_FORMATTER";

fn init_with_full_formatter() {
    tracing_subscriber::fmt().init()
}

fn init_with_pretty_formatter() {
    tracing_subscriber::fmt().pretty().init()
}

fn init_with_json_formatter() {
    tracing_subscriber::fmt().json().flatten_event(true).init()
}

pub fn init_tracing_subscriber() {
    let formatter = std::env::var(ENV_FORMATTER).unwrap_or_else(|_| String::from(FULL_FORMATTER));
    match formatter.as_str() {
        FULL_FORMATTER => init_with_full_formatter(),
        PRETTY_FORMATTER => init_with_pretty_formatter(),
        JSON_FORMATTER => init_with_json_formatter(),
        _ => init_with_full_formatter(),
    }
}
