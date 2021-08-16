use crate::models::utils::indent_literal;
use serde::Deserialize;

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct ProjectMeta {
    input: Option<String>,
    from: Option<String>,
    expr: Option<String>,
    alias: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct ConvertMeta {
    input: Option<String>,
    from: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct FilterMeta {
    predicate: String,
    alias: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct IntoAttributesMeta {
    alias: String,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum DeriveMeta {
    Clone,
    Convert,
    Debug,
    Display,
    Serialize,
    Deserialize,
    Eq,
    Equal,
    PartialEq,
    Project,
    Filter,
    FieldAccept,
    HashedBy,
    OrderedBy,
    AggregateAs,
    GroupAs,
    LeftRight,
    Render,
    IntoAttributes,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum Tag {
    Hash,
    Group,
    Order,
    Visit,
    Equal,
    Left,
    Right,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum AggregateMeta {
    Top,
    Sum,
    Count32(Option<String>),
    Avgf32(Option<String>),
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct RenderMeta {
    template: Option<String>,
    pos: Option<i32>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    // String Literal, Generate as raw or not
    Str { value: String, raw: bool },
    Int { value: i32 },
    Usize { value: usize },
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum Meta {
    // Base Meta
    Value { name: String, meta: MetaValue },
    List { name: String, metas: Vec<Meta> },
    Path { name: String },
    // Extended Meta
    Derive { derives: Vec<DeriveMeta> },
    Project { project: ProjectMeta },
    Filter { filter: FilterMeta },
    Aggregate { agg: AggregateMeta },
    Tag { tag: Tag },
    Render { render: RenderMeta },
    Convert { convert: ConvertMeta },
    IntoAttributes { attribute: IntoAttributesMeta },
}

pub(crate) fn meta_value_str(name: &str, value: &str, raw: bool) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Str {
            value: value.to_owned(),
            raw,
        },
    }
}

fn meta_value_int(name: &str, value: &i32) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Int {
            value: value.to_owned(),
        },
    }
}

pub(crate) fn meta_value_usize(name: &str, value: &usize) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Usize {
            value: value.to_owned(),
        },
    }
}

fn new_path(name: String) -> Meta {
    Meta::Path { name }
}

fn expand_derive(derive: &DeriveMeta) -> Meta {
    let name = match derive {
        DeriveMeta::Convert => "Convert",
        DeriveMeta::Clone => "Clone",
        DeriveMeta::Debug => "Debug",
        DeriveMeta::Display => "Display",
        DeriveMeta::Serialize => "Serialize",
        DeriveMeta::Deserialize => "Deserialize",
        DeriveMeta::Eq => "Eq",
        DeriveMeta::Equal => "Equal",
        DeriveMeta::PartialEq => "PartialEq",
        DeriveMeta::Project => "Project",
        DeriveMeta::Filter => "Filter",
        DeriveMeta::FieldAccept => "FieldAccept",
        DeriveMeta::HashedBy => "HashedBy",
        DeriveMeta::OrderedBy => "OrderedBy",
        DeriveMeta::AggregateAs => "AggregateAs",
        DeriveMeta::GroupAs => "GroupAs",
        DeriveMeta::LeftRight => "LeftRight",
        DeriveMeta::Render => "Render",
        DeriveMeta::IntoAttributes => "IntoAttributes",
    };
    new_path(name.to_owned())
}

fn expand_derives(derives: &[DeriveMeta]) -> Meta {
    let metas: Vec<Meta> = derives.iter().map(|derive| expand_derive(derive)).collect();
    Meta::List {
        name: "derive".to_owned(),
        metas,
    }
}

fn expand_project_meta(meta: &ProjectMeta) -> Meta {
    let mut metas: Vec<Meta> = Vec::new();
    if let Some(ref input) = meta.input {
        metas.push(meta_value_str("input", input, false))
    }
    if let Some(ref from) = meta.from {
        metas.push(meta_value_str("from", from, false))
    }
    if let Some(ref expr) = meta.expr {
        metas.push(meta_value_str("expr", expr, true))
    }
    if let Some(ref alias) = meta.alias {
        metas.push(meta_value_str("alias", alias, false))
    };
    Meta::List {
        name: "project".to_owned(),
        metas,
    }
}

fn expand_convert_meta(meta: &ConvertMeta) -> Meta {
    let mut metas: Vec<Meta> = Vec::new();
    if let Some(ref input) = meta.input {
        metas.push(meta_value_str("input", input, false))
    }
    if let Some(ref from) = meta.from {
        metas.push(meta_value_str("from", from, false))
    }
    Meta::List {
        name: "convert".to_owned(),
        metas,
    }
}

fn expand_filter_meta(meta: &FilterMeta) -> Meta {
    let mut metas: Vec<Meta> = vec![meta_value_str("predicate", &meta.predicate, true)];
    if let Some(ref alias) = meta.alias {
        metas.push(meta_value_str("alias", alias, false))
    }
    Meta::List {
        name: "filter".to_owned(),
        metas,
    }
}

fn expand_into_attributes_meta(meta: &IntoAttributesMeta) -> Meta {
    let metas = vec![meta_value_str("alias", &meta.alias, false)];
    Meta::List {
        name: "attribute".to_owned(),
        metas,
    }
}

fn expand_tag(tag: &Tag) -> Meta {
    match tag {
        Tag::Hash => new_path("hash".to_owned()),
        Tag::Group => new_path("group".to_owned()),
        Tag::Order => new_path("order".to_owned()),
        Tag::Visit => new_path("visit".to_owned()),
        Tag::Equal => new_path("equal".to_owned()),
        Tag::Left => new_path("left".to_owned()),
        Tag::Right => new_path("right".to_owned()),
    }
}

fn expand_avgf32(avgf32_ty: Option<String>) -> Meta {
    match avgf32_ty {
        Some(ty) => meta_value_str("avgf32", &ty, false),
        None => new_path("avgf32".to_owned()),
    }
}

fn expand_count32(count32_ty: Option<String>) -> Meta {
    match count32_ty {
        Some(ty) => meta_value_str("count32", &ty, false),
        None => new_path("count32".to_owned()),
    }
}

fn expand_sum() -> Meta {
    new_path("sum".to_owned())
}

fn expand_top() -> Meta {
    new_path("top".to_owned())
}

fn expand_aggregate(agg: &AggregateMeta) -> Meta {
    let meta = match agg {
        AggregateMeta::Sum => expand_sum(),
        AggregateMeta::Top => expand_top(),
        AggregateMeta::Count32(ty) => expand_count32(ty.to_owned()),
        AggregateMeta::Avgf32(ty) => expand_avgf32(ty.to_owned()),
    };
    Meta::List {
        name: "agg".to_owned(),
        metas: vec![meta],
    }
}

fn expand_render(render: &RenderMeta) -> Meta {
    let mut metas: Vec<Meta> = Vec::new();
    if let Some(ref template) = render.template {
        metas.push(meta_value_str("template", template, true))
    }
    if let Some(ref pos) = render.pos {
        metas.push(meta_value_int("pos", pos))
    }
    Meta::List {
        name: "render".to_owned(),
        metas,
    }
}

fn meta_path_to_lit(name: &str, indent: usize, compact: bool) -> String {
    let lit = name.to_owned();
    if compact {
        return lit;
    }
    format!("{}{}", indent_literal(indent), lit)
}

fn meta_str_value_to_lit(
    name: &str,
    value: &str,
    raw: &bool,
    indent: usize,
    compact: bool,
) -> String {
    let lit = match raw {
        true => format!(r##"{} = r#"{}"#"##, name, value),
        false => format!(r#"{} = "{}""#, name, value),
    };
    if compact {
        return lit;
    }
    format!("{}{}", indent_literal(indent), lit)
}

fn meta_int_value_to_lit(name: &str, value: &i32, indent: usize, compact: bool) -> String {
    let lit = format!("{} = {}", name, value);
    if compact {
        return lit;
    }
    format!("{}{}", indent_literal(indent), lit)
}

fn meta_usize_value_to_lit(name: &str, value: &usize, indent: usize, compact: bool) -> String {
    let lit = format!("{} = {}", name, value);
    if compact {
        return lit;
    }
    format!("{}{}", indent_literal(indent), lit)
}

fn expand_meta_lit(meta: &Meta, indent: usize, compact: bool) -> String {
    let (name, metas) = match meta {
        Meta::Path { name } => return meta_path_to_lit(name, indent, compact),
        Meta::Value { name, meta } => match meta {
            MetaValue::Str { value, raw } => {
                return meta_str_value_to_lit(name, value, raw, indent, compact)
            }
            MetaValue::Int { value } => return meta_int_value_to_lit(name, value, indent, compact),
            MetaValue::Usize { value } => {
                return meta_usize_value_to_lit(name, value, indent, compact)
            }
        },
        Meta::Derive { derives } => {
            let meta = expand_derives(derives);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Project { project } => {
            let meta = expand_project_meta(project);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Filter { filter } => {
            let meta = expand_filter_meta(filter);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Aggregate { agg } => {
            let meta = expand_aggregate(agg);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Tag { tag } => {
            let meta = expand_tag(tag);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Render { render } => {
            let meta = expand_render(render);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::Convert { convert } => {
            let meta = expand_convert_meta(convert);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::IntoAttributes { attribute } => {
            let meta = expand_into_attributes_meta(attribute);
            return expand_meta_lit(&meta, indent, compact);
        }
        Meta::List { name, metas } => (name, metas),
    };
    let nested_metas_lits: Vec<String> = metas
        .iter()
        .map(|meta| expand_meta_lit(meta, indent + 1, compact))
        .collect();
    let nested_metas_lit = join_meta_lits(nested_metas_lits, compact);
    let left = get_left_parenthesis(compact);
    let right = get_right_parenthesis(compact, indent);
    let lit = format!("{}{}{}{}", name, left, nested_metas_lit, right);
    match compact {
        true => lit,
        false => format!("{}{}", indent_literal(indent), lit),
    }
}

pub(crate) fn meta_to_literal(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let meta_lit = expand_meta_lit(meta, indent + 1, false);
    format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
}

pub(crate) fn metas_to_literal(metas: &[Meta], indent: usize) -> String {
    let metas_literal: Vec<String> = metas
        .iter()
        .map(|meta| meta_to_literal(meta, indent))
        .collect();
    metas_literal.join("\n")
}

pub(crate) fn meta_to_display(meta: &Meta) -> String {
    expand_meta_lit(meta, 0, true)
}

pub(crate) fn metas_to_display(metas: &[Meta]) -> String {
    let metas_display: Vec<String> = metas.iter().map(|meta| meta_to_display(meta)).collect();
    metas_display.join(" ")
}

fn get_left_parenthesis(compact: bool) -> String {
    match compact {
        true => "(".to_owned(),
        false => "(\n".to_owned(),
    }
}

fn get_right_parenthesis(compact: bool, indent: usize) -> String {
    match compact {
        true => ")".to_owned(),
        false => {
            let indent_lit = indent_literal(indent);
            format!("\n{})", indent_lit)
        }
    }
}

fn join_meta_lits(meta_lits: Vec<String>, compact: bool) -> String {
    match compact {
        true => meta_lits.join(","),
        false => meta_lits.join(",\n"),
    }
}
