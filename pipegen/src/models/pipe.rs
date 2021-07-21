use std::collections::HashSet;
use std::fmt::Display;

use crate::models::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::data::DataField;
use super::meta::{meta_to_literal, Meta, MetaValue};

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum PipeType {
    #[strum(to_string = "listener")]
    Listener,
    #[strum(to_string = "poller")]
    Poller,
    #[strum(to_string = "mapper")]
    Mapper,
    #[strum(to_string = "collector")]
    Collector,
    #[strum(to_string = "selector")]
    Selector,
    #[strum(to_string = "exporter")]
    Exporter,
    #[strum(to_string = "streamer")]
    Streamer,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PipeConfig {
    ty: String,
    path: Option<String>,
}

impl PipeConfig {
    pub fn get_path(&self) -> Option<&String> {
        self.path.as_ref()
    }
    pub fn get_config_type(&self) -> &String {
        &self.ty
    }
}

impl Display for PipeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.path {
            Some(ref path) => write!(f, "{{ type: {}, path: {} }}", self.ty, path),
            None => write!(f, "{{ type: {} }}", self.ty),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pipe {
    name: String,
    ty: PipeType,
    config: PipeConfig,
    // upstream pipe names
    upstreams: Option<Vec<String>>,
    // output data type
    output: Option<DataField>,
}

impl Pipe {
    pub fn init(&mut self) {
        if self.output.is_some() {
            self.output.as_mut().unwrap().init();
        }
        if self.upstreams.is_none() {
            self.upstreams = Some(Vec::new())
        }
    }

    pub fn is_source(&self) -> bool {
        match &self.ty {
            PipeType::Listener | PipeType::Poller => true,
            _ => false,
        }
    }

    pub fn is_sink(&self) -> bool {
        match &self.ty {
            PipeType::Exporter => true,
            _ => false,
        }
    }

    fn get_name_meta(&self) -> Meta {
        Meta::Value {
            name: "name".to_owned(),
            meta: MetaValue::Str {
                value: self.name.to_owned(),
                raw: false,
            },
        }
    }

    fn get_type_meta(&self) -> Meta {
        Meta::Value {
            name: "ty".to_owned(),
            meta: MetaValue::Str {
                value: self.ty.to_string(),
                raw: false,
            },
        }
    }

    fn get_config_meta(&self) -> Meta {
        let mut config_metas = vec![];
        config_metas.push(Meta::Value {
            name: "ty".to_owned(),
            meta: MetaValue::Str {
                value: self.config.get_config_type().to_owned(),
                raw: false,
            },
        });
        match self.config.get_path() {
            Some(path) => {
                config_metas.push(Meta::Value {
                    name: "path".to_owned(),
                    meta: MetaValue::Str {
                        value: path.to_owned(),
                        raw: false,
                    },
                });
            }
            None => (),
        };
        Meta::List {
            name: "config".to_owned(),
            metas: config_metas,
        }
    }

    fn get_upstream_meta(&self) -> Option<Meta> {
        let upstreams = self.upstreams.as_ref().expect("upstreams not inited");
        if upstreams.is_empty() {
            return None;
        }
        let meta = Meta::Value {
            name: "upstream".to_owned(),
            meta: MetaValue::Str {
                value: upstreams.join(", "),
                raw: false,
            },
        };
        Some(meta)
    }

    fn get_output_data_type_meta(&self) -> Option<Meta> {
        let output = match self.output {
            Some(ref output) => output,
            None => return None,
        };
        let meta = Meta::Value {
            name: "output".to_owned(),
            meta: MetaValue::Str {
                value: output.to_literal(0),
                raw: false,
            },
        };
        Some(meta)
    }

    pub(crate) fn get_output_data_type(&self) -> Option<&DataField> {
        self.output.as_ref()
    }

    pub(crate) fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub fn filter_upstreams(&mut self, pipe_id_filter: &HashSet<String>) {
        let upstreams = match self.upstreams {
            Some(ref upstreams) => upstreams,
            None => return,
        };
        self.upstreams = Some(
            upstreams
                .to_owned()
                .into_iter()
                .filter(|id| pipe_id_filter.contains(id))
                .collect(),
        )
    }
}

impl Entity for Pipe {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.upstreams {
            Some(ref upstreams) => upstreams.to_owned(),
            None => vec![],
        }
    }

    // to pipe meta
    fn to_literal(&self, indent: usize) -> String {
        let mut metas: Vec<Meta> = Vec::new();
        metas.push(self.get_name_meta());
        metas.push(self.get_type_meta());
        metas.push(self.get_config_meta());
        match self.get_upstream_meta() {
            Some(meta) => metas.push(meta),
            None => (),
        };
        match self.get_output_data_type_meta() {
            Some(meta) => metas.push(meta),
            None => (),
        };
        let meta = Meta::List {
            name: "pipe".to_owned(),
            metas: metas,
        };
        meta_to_literal(&meta, indent)
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name:   {}", self.name)?;
        writeln!(f, "Type:   {}", self.ty)?;
        writeln!(f, "Config: {}", self.config)?;
        let upstream = match self.upstreams {
            Some(ref upstreams) => upstreams.join(", "),
            None => "".to_owned(),
        };
        writeln!(f, "Upstream: [{}]", upstream)
    }
}

impl<V: VisitEntity<Pipe>> EntityAccept<V> for Pipe {}