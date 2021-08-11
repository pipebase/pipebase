use std::collections::HashSet;
use std::fmt::Display;

use crate::models::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::data::{data_ty_to_literal, DataType};
use super::meta::{meta_to_literal, meta_value_usize, Meta, MetaValue};

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
    // pipe channel buffer
    buffer: Option<usize>,
    // upstream pipe names
    upstreams: Option<Vec<String>>,
    // output data type
    output: Option<DataType>,
}

impl Pipe {
    pub fn init(&mut self) {
        if self.upstreams.is_none() {
            self.upstreams = Some(Vec::new())
        }
    }

    pub fn is_source(&self) -> bool {
        matches!(&self.ty, PipeType::Listener | PipeType::Poller)
    }

    pub fn is_sink(&self) -> bool {
        matches!(&self.ty, PipeType::Exporter)
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
        let mut config_metas = vec![Meta::Value {
            name: "ty".to_owned(),
            meta: MetaValue::Str {
                value: self.config.get_config_type().to_owned(),
                raw: false,
            },
        }];
        if let Some(path) = self.config.get_path() {
            config_metas.push(Meta::Value {
                name: "path".to_owned(),
                meta: MetaValue::Str {
                    value: path.to_owned(),
                    raw: false,
                },
            });
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
                value: data_ty_to_literal(output),
                raw: false,
            },
        };
        Some(meta)
    }

    pub(crate) fn get_output_data_type(&self) -> Option<&DataType> {
        self.output.as_ref()
    }

    pub(crate) fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub(crate) fn get_channel_buffer_meta(&self) -> Option<Meta> {
        let buffer = match self.buffer {
            Some(ref buffer) => buffer,
            None => return None,
        };
        Some(meta_value_usize("buffer", buffer))
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
        let mut metas: Vec<Meta> = vec![
            self.get_name_meta(),
            self.get_type_meta(),
            self.get_config_meta(),
        ];
        if let Some(meta) = self.get_upstream_meta() {
            metas.push(meta)
        };
        if let Some(meta) = self.get_output_data_type_meta() {
            metas.push(meta)
        };
        if let Some(meta) = self.get_channel_buffer_meta() {
            metas.push(meta)
        };
        let meta = Meta::List {
            name: "pipe".to_owned(),
            metas,
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
