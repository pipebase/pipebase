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

pub const HASH: &str = "hash";

pub const ORDER: &str = "order";

pub const CONTEXT_STORE: &str = "cstore";
pub const CONTEXT_STORE_NAME: &str = "cstore.name";
pub const CONTEXT_STORE_CONFIG_TYPE: &str = "cstore.config.ty";
pub const CONTEXT_STORE_CONFIG_PATH: &str = "cstore.config.path";
pub const CONTEXT_STORE_CONFIG_EMPTY_PATH: &str = "";
pub const CONTEXT_STORE_MACRO: &str = "cstore!";
pub const RUN_CONTEXT_STORE_MACRO: &str = "run_cstore!";

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
pub const BOOTSTRAP_MODULE: &str = "bootstrap";
pub const BOOTSTRAP_FUNCTION: &str = "bootstrap";

pub const CHANNEL_MACRO: &str = "channel!";
pub const RUN_PIPE_MACRO: &str = "run_pipe!";
pub const JOIN_PIPES_MACRO: &str = "join_pipes!";

pub const AGGREGATE_SUM: &str = "agg.sum";
pub const AGGREGATE_TOP: &str = "agg.top";

pub const GROUP: &str = "group";

pub const EQUAL: &str = "equal";
