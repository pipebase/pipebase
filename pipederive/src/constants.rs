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
pub const CONTEXT_STORE_IDENT_SUFFIX: &str = "_c";

pub const ERROR_HANDLER: &str = "error";
pub const ERROR_HANDLER_CONFIG_TYPE: &str = "error.config.ty";
pub const ERROR_HANDLER_CONFIG_PATH: &str = "error.config.path";
pub const ERROR_HANDLER_MACRO: &str = "error_handler!";
pub const SUBSCRIBE_ERROR_HANDLER_MACRO: &str = "subscribe_error_handler!";
pub const RUN_ERROR_HANDLER_MACRO: &str = "run_error_handler!";
pub const ERROR_HANDLER_DEFAULT_IDENT: &str = "_error_handler";
pub const ERROR_HANDLER_DEFAULT_RX: &str = "_erx";
pub const ERROR_HANDLER_DEFAULT_TX: &str = "_etx";
pub const ERROR_HANDLER_CHANNEL_BUFFER: &str = "error.buffer";
pub const ERROR_HANDLER_CHANNEL_DEFAULT_BUFFER: usize = 1024;
pub const ERROR_HANDLER_CHANNEL_DEFAULT_TYPE: &str = "PipeError";

pub const BOOTSTRAP_PIPE: &str = "pipe";
pub const BOOTSTRAP_PIPE_NAME: &str = "pipe.name";
pub const BOOTSTRAP_PIPE_TYPE: &str = "pipe.ty";
pub const BOOTSTRAP_PIPE_UPSTREAM: &str = "pipe.upstream";
pub const BOOTSTRAP_PIPE_UPSTREAM_NAME_SEP: &str = ",";
pub const BOOTSTRAP_PIPE_CONFIG_TYPE: &str = "pipe.config.ty";
pub const BOOTSTRAP_PIPE_CONFIG_PATH: &str = "pipe.config.path";
pub const BOOTSTRAP_PIPE_CONFIG_EMPTY_PATH: &str = "";
pub const BOOTSTRAP_PIPE_OUTPUT: &str = "pipe.output";
pub const BOOTSTRAP_PIPE_CHANNEL_BUFFER: &str = "pipe.buffer";
pub const BOOTSTRAP_PIPE_CHANNEL_DEFAULT_BUFFER: usize = 1024;
pub const BOOTSTRAP_MODULE: &str = "bootstrap";
pub const BOOTSTRAP_FUNCTION: &str = "bootstrap";
pub const BOOTSTRAP_PIPE_IDENT_SUFFIX: &str = "_p";

pub const CHANNEL_MACRO: &str = "channel!";
pub const RUN_PIPE_MACRO: &str = "run_pipe!";
pub const JOIN_PIPES_MACRO: &str = "join_pipes!";

pub const AGGREGATE_SUM: &str = "agg.sum";
pub const AGGREGATE_TOP: &str = "agg.top";
pub const AGGREGATE_AVG_F32: &str = "agg.avgf32";
pub const AGGREGATE_AVG_F32_DEFAULT_TYPE: &str = "Averagef32";
pub const AGGREGATE_COUNT32: &str = "agg.count32";
pub const AGGREGATE_COUNT32_DEFAULT_TYPE: &str = "Count32";

pub const GROUP: &str = "group";

pub const EQUAL: &str = "equal";

pub const LEFT: &str = "left";
pub const RIGHT: &str = "right";

pub const RENDER_TEMPLATE: &str = "render.template";
pub const RENDER_POSITION: &str = "render.pos";
pub const RENDER_EXPR: &str = "render.expr";

pub const CONVERT_INPUT: &str = "convert.input";
pub const CONVERT_FROM: &str = "convert.from";

pub const ATTRIBUTE_ALIAS: &str = "attribute.alias";
