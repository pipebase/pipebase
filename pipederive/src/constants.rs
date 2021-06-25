pub const PROJECT: &str = "project";
pub const PROJECT_INPUT: &str = "project.input";
pub const PROJECT_FROM: &str = "project.from";
pub const PROJECT_EXPR: &str = "project.expr";
pub const PROJECT_ALIAS: &str = "project.alias";
pub const PROJECT_ALIAS_DEFAULT: &str = "a";

pub const FILTER: &str = "filter";
pub const FILTER_PREDICATE: &str = "filter.predicate";
pub const FILTER_ALIAS: &str = "filter.alias";
pub const FILTER_ALIAS_DEFAULT: &str = "a";

pub const FIELD_VISIT: &str = "visit";

pub const HASH_KEY: &str = "hkey";

pub const ORDER_KEY: &str = "okey";

pub const CONTEXT_STORE: &str = "cstore";

pub const CONTEXT_STORE_METHOD_INSERT: &str = "cstore.method.insert";
pub const CONTEXT_STORE_METHOD_INSERT_DEFAULT: &str = "insert";

pub const CONTEXT_STORE_METHOD_GET: &str = "cstore.method.get";
pub const CONTEXT_STORE_METHOD_GET_DEFAULT: &str = "get";

pub const BOOTSTRAP_PIPE: &str = "pipe";
pub const BOOTSTRAP_PIPE_NAME: &str = "pipe.name";
pub const BOOTSTRAP_PIPE_TYPE: &str = "pipe.ty";
pub const BOOTSTRAP_PIPE_UPSTREAM: &str = "pipe.upstream";
pub const BOOTSTRAP_PIPE_UPSTREAM_NAME_SEP: &str = ",";
pub const BOOTSTRAP_PIPE_CONFIG_TYPE: &str = "pipe.config.ty";
pub const BOOTSTRAP_PIPE_CONFIG_PATH: &str = "pipe.config.path";
pub const BOOTSTRAP_PIPE_CONFIG_EMPTY_PATH: &str = "";
pub const BOOTSTRAP_PIPE_OUTPUT: &str = "pipe.output";
pub const BOOTSTRAP_PIPE_CHANNEL_DEFAULT_BUFFER: usize = 1024;

pub const CHANNEL_MACRO: &str = "channel!";
pub const SPAWN_JOIN_MACRO: &str = "spawn_join!";
