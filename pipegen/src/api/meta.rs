use crate::api::utils::indent_literal;
use serde::Deserialize;

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct ProjectMeta {
    pub input: Option<String>,
    pub from: Option<String>,
    pub expr: Option<String>,
    pub alias: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub struct FilterMeta {
    pub predicate: String,
    pub alias: Option<String>,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum DeriveMeta {
    Clone,
    Debug,
    Display,
    Serialize,
    Deserialize,
    Eq,
    Equal,
    PartialEq,
    Project,
    Filter,
    FieldAccess,
    HashedBy,
    OrderedBy,
    AggregateAs,
    GroupAs,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum Tag {
    Hash,
    Group,
    Order,
    Visit,
    Equal,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
pub enum AggregateMeta {
    Top,
    Sum,
}

#[derive(Clone, PartialEq, Debug, Deserialize)]
#[serde(untagged)]
pub enum MetaValue {
    Str(String),
    Int(i32),
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
}

fn meta_value_str(name: &str, value: &str) -> Meta {
    Meta::Value {
        name: name.to_owned(),
        meta: MetaValue::Str(value.to_owned()),
    }
}

fn new_path(name: String) -> Meta {
    Meta::Path { name: name }
}

fn expand_derive(derive: &DeriveMeta) -> Meta {
    let name = match derive {
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
        DeriveMeta::FieldAccess => "FieldAccess",
        DeriveMeta::HashedBy => "HashedBy",
        DeriveMeta::OrderedBy => "OrderedBy",
        DeriveMeta::AggregateAs => "AggregateAs",
        DeriveMeta::GroupAs => "GroupAs",
    };
    new_path(name.to_owned())
}

fn expand_derives(derives: &Vec<DeriveMeta>) -> Meta {
    let metas: Vec<Meta> = derives
        .into_iter()
        .map(|derive| expand_derive(derive))
        .collect();
    Meta::List {
        name: "derive".to_owned(),
        metas: metas,
    }
}

fn expand_project_meta(meta: &ProjectMeta) -> Meta {
    let mut metas: Vec<Meta> = Vec::new();
    match meta.input {
        Some(ref input) => metas.push(meta_value_str("input", input)),
        None => (),
    };
    match meta.from {
        Some(ref from) => metas.push(meta_value_str("from", from)),
        None => (),
    };
    match meta.expr {
        Some(ref expr) => metas.push(meta_value_str("expr", expr)),
        None => (),
    };
    match meta.alias {
        Some(ref alias) => metas.push(meta_value_str("alias", alias)),
        None => (),
    };
    Meta::List {
        name: "project".to_owned(),
        metas: metas,
    }
}

fn expand_filter_meta(meta: &FilterMeta) -> Meta {
    let mut metas: Vec<Meta> = Vec::new();
    metas.push(meta_value_str("predicate", &meta.predicate));
    match meta.alias {
        Some(ref alias) => metas.push(meta_value_str("alias", alias)),
        None => (),
    };
    Meta::List {
        name: "filter".to_owned(),
        metas: metas,
    }
}

fn expand_tag(tag: &Tag) -> Meta {
    match tag {
        Tag::Hash => new_path("hash".to_owned()),
        Tag::Group => new_path("group".to_owned()),
        Tag::Order => new_path("order".to_owned()),
        Tag::Visit => new_path("visit".to_owned()),
        Tag::Equal => new_path("equal".to_owned()),
    }
}

fn expand_aggregate(agg: &AggregateMeta) -> Meta {
    let op = match agg {
        AggregateMeta::Sum => "sum",
        AggregateMeta::Top => "top",
    };
    let meta = new_path(op.to_owned());
    Meta::List {
        name: "agg".to_owned(),
        metas: vec![meta],
    }
}

fn expand_meta_lit(meta: &Meta, mut indent: usize, compact: bool) -> String {
    if compact {
        // if compact no indent
        indent = 0;
    }
    let indent_lit = indent_literal(indent);
    let (name, metas) = match meta {
        Meta::Path { name } => return format!("{}{}", indent_lit, name),
        Meta::Value { name, meta } => match meta {
            MetaValue::Str(value) => return format!(r#"{}{} = "{}""#, indent_lit, name, value),
            MetaValue::Int(value) => return format!("{}{} = {}", indent_lit, name, value),
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
        Meta::List { name, metas } => (name, metas),
    };
    let mut nested_metas_lits: Vec<String> = vec![];
    for nested_meta in metas {
        nested_metas_lits.push(expand_meta_lit(nested_meta, indent + 1, compact));
    }
    let nested_metas_lit = join_meta_lits(nested_metas_lits, compact);
    let left = get_left_parenthesis(compact);
    let right = get_right_parenthesis(compact, indent);
    format!(
        "{}{}{}{}{}",
        indent_lit, name, left, nested_metas_lit, right
    )
}

pub(crate) fn meta_to_literal(meta: &Meta, indent: usize) -> String {
    let indent_lit = indent_literal(indent);
    let meta_lit = expand_meta_lit(meta, indent + 1, false);
    format!("{}#[\n{}\n{}]", indent_lit, meta_lit, indent_lit)
}

pub(crate) fn metas_to_literal(metas: &Vec<Meta>, indent: usize) -> String {
    let mut metas_literal: Vec<String> = vec![];
    for meta in metas {
        metas_literal.push(meta_to_literal(meta, indent))
    }
    metas_literal.join("\n")
}

pub(crate) fn meta_to_display(meta: &Meta) -> String {
    expand_meta_lit(meta, 0, true)
}

pub(crate) fn metas_to_display(metas: &Vec<Meta>) -> String {
    let mut metas_display: Vec<String> = vec![];
    for meta in metas {
        metas_display.push(meta_to_display(meta))
    }
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
