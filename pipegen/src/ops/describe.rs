use crate::{
    error::{api_error, Result},
    models::{data_ty_to_literal, App, Entity, EntityAccept, Object, Pipe, VisitEntity},
    ops::utils::PipeGraph,
};

use std::{collections::HashMap, fmt};

pub trait Describe {
    fn new() -> Self;
    fn describe(&self) -> Vec<String>;
}

pub struct PipeDescriber {
    pipes: HashMap<String, Pipe>,
}

impl VisitEntity<Pipe> for PipeDescriber {
    fn visit(&mut self, pipe: &Pipe) {
        self.pipes.insert(pipe.get_id(), pipe.to_owned());
    }
}

impl Describe for PipeDescriber {
    fn new() -> Self {
        PipeDescriber {
            pipes: HashMap::new(),
        }
    }

    fn describe(&self) -> Vec<String> {
        self.describe_pipes()
    }
}

impl PipeDescriber {
    pub(crate) fn describe_pipes(&self) -> Vec<String> {
        let all_pipes = format!("{}", self.display_pipes());
        vec![all_pipes]
    }

    pub(crate) fn describe_pipe(&self, pid: &str) -> Result<String> {
        let pipe = match self.pipes.get(pid) {
            Some(pipe) => pipe,
            None => return Err(api_error(format!("pipe {} not exists", pid))),
        };
        Ok(format!("{}", pipe))
    }

    fn display_pipes(&self) -> EntityIdsDisplay {
        let pids: Vec<String> = self
            .pipes
            .keys()
            .into_iter()
            .map(|pid| pid.to_owned())
            .collect();
        EntityIdsDisplay {
            ids: pids,
            sep: ENTITY_ID_LIST_SEP.to_owned(),
            label: Some(PIPE_LABEL.to_owned()),
        }
    }
}

pub struct PipeGraphDescriber {
    graph: PipeGraph<Pipe>,
    pipe_ids: Vec<String>,
}

impl VisitEntity<Pipe> for PipeGraphDescriber {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.pipe_ids.push(pipe.get_id());
    }
}

impl Describe for PipeGraphDescriber {
    fn new() -> Self {
        PipeGraphDescriber {
            graph: PipeGraph::new(),
            pipe_ids: Vec::new(),
        }
    }

    fn describe(&self) -> Vec<String> {
        let mut results: Vec<String> = vec![
            self.describe_source_pipe_ids(),
            self.describe_sink_pipe_ids(),
        ];
        results.extend(self.describe_pipe_components());
        results
    }
}

impl PipeGraphDescriber {
    pub(crate) fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let results: Vec<String> = self
            .display_pipelines(pid)?
            .into_iter()
            .map(|pipeline| format!("{}", pipeline))
            .collect();
        Ok(results)
    }

    pub(crate) fn describe_source_pipe_ids(&self) -> String {
        format!("{}", self.display_source_pipe_ids())
    }

    pub(crate) fn describe_sink_pipe_ids(&self) -> String {
        format!("{}", self.display_sink_pipe_ids())
    }

    pub(crate) fn describe_pipe_components(&self) -> Vec<String> {
        self.display_pipe_components()
            .into_iter()
            .map(|component| format!("{}", component))
            .collect()
    }

    fn display_source_pipe_ids(&self) -> EntityIdsDisplay {
        EntityIdsDisplay {
            ids: self.get_source_pipe_ids(),
            sep: ENTITY_ID_LIST_SEP.to_owned(),
            label: Some(SOURCE_PIPE_LABEL.to_owned()),
        }
    }

    fn display_sink_pipe_ids(&self) -> EntityIdsDisplay {
        EntityIdsDisplay {
            ids: self.get_sink_pipe_ids(),
            sep: ENTITY_ID_LIST_SEP.to_owned(),
            label: Some(SINK_PIPE_LABEL.to_owned()),
        }
    }

    fn display_pipe_components(&self) -> Vec<EntityIdsDisplay> {
        self.get_pipe_components()
            .into_iter()
            .map(|component| EntityIdsDisplay {
                ids: component,
                sep: ENTITY_ID_LIST_SEP.to_owned(),
                label: Some(PIPE_COMPONENT_LABEL.to_owned()),
            })
            .collect()
    }

    fn display_pipelines(&self, pid: &str) -> Result<Vec<EntityIdsDisplay>> {
        let pipelines_display: Vec<EntityIdsDisplay> = self
            .get_pipelines(pid)?
            .into_iter()
            .map(|pipeline| EntityIdsDisplay {
                ids: self.format_pipeline_with_output_type(pipeline),
                sep: PIPE_DIRECT_SEP.to_owned(),
                label: Some(PIPELINE_LABEL.to_owned()),
            })
            .collect();
        Ok(pipelines_display)
    }

    fn get_source_pipe_ids(&self) -> Vec<String> {
        self.graph.find_source_pipes()
    }

    fn get_sink_pipe_ids(&self) -> Vec<String> {
        self.graph.find_sink_pipes()
    }

    pub(crate) fn get_pipelines(&self, pid: &str) -> Result<Vec<Vec<String>>> {
        match self.graph.has_pipe(pid) {
            true => Ok(self.graph.search_pipelines(pid)),
            false => Err(api_error(format!("pipe {} not exists", pid))),
        }
    }

    pub(crate) fn get_pipe_component(&self, pid: &str) -> Result<Vec<String>> {
        match self.graph.has_pipe(pid) {
            true => Ok(self.graph.find_component(pid)),
            false => Err(api_error(format!("pipe {} not exists", pid))),
        }
    }

    fn get_pipe_output_type(&self, pid: &str) -> Option<String> {
        let pipe = self.graph.get_pipe_value(pid).unwrap();
        pipe.get_output_data_type().map(data_ty_to_literal)
    }

    fn format_pipeline_with_output_type(&self, pipeline: Vec<String>) -> Vec<String> {
        pipeline
            .into_iter()
            .map(|pid| {
                let output_type = self.get_pipe_output_type(&pid);
                match output_type {
                    Some(output_type) => format!("{}({})", pid, output_type),
                    None => pid,
                }
            })
            .collect()
    }

    fn get_pipe_components(&self) -> Vec<Vec<String>> {
        self.graph
            .find_components()
            .values()
            .map(|component| component.to_owned())
            .collect()
    }
}

pub struct ObjectDescriber {
    objects: HashMap<String, Object>,
}

impl Describe for ObjectDescriber {
    fn new() -> Self {
        ObjectDescriber {
            objects: HashMap::new(),
        }
    }

    fn describe(&self) -> Vec<String> {
        vec![self.describe_objects()]
    }
}

impl VisitEntity<Object> for ObjectDescriber {
    fn visit(&mut self, entity: &Object) {
        self.objects.insert(entity.get_id(), entity.to_owned());
    }
}

impl ObjectDescriber {
    fn describe_objects(&self) -> String {
        format!("{}", self.display_object_ids())
    }

    pub(crate) fn describe_object(&self, oid: &str) -> Result<String> {
        let object = match self.objects.get(oid) {
            Some(object) => object,
            None => return Err(api_error(format!("object {} not exists", oid))),
        };
        Ok(format!("{}", object))
    }

    fn display_object_ids(&self) -> EntityIdsDisplay {
        let object_ids: Vec<String> = self.objects.keys().map(|k| k.to_owned()).collect();
        EntityIdsDisplay {
            ids: object_ids,
            sep: ENTITY_ID_LIST_SEP.to_owned(),
            label: Some(OBJECTS_LABLE.to_owned()),
        }
    }
}

pub struct EntityIdsDisplay {
    ids: Vec<String>,
    sep: String,
    label: Option<String>,
}

impl fmt::Display for EntityIdsDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ids_lit = self.ids.join(&self.sep);
        match self.label {
            Some(ref label) => write!(f, "{}: {}", label, ids_lit),
            None => write!(f, "{}", ids_lit),
        }
    }
}

pub struct AppDescriber {
    app: Option<App>,
}

impl VisitEntity<App> for AppDescriber {
    fn visit(&mut self, app: &App) {
        self.app = Some(app.to_owned())
    }
}

impl Describe for AppDescriber {
    fn new() -> Self {
        AppDescriber { app: None }
    }

    fn describe(&self) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        // describe app basic info
        results.extend(self.describe_pipes());
        results
    }
}

impl AppDescriber {
    fn get_app(&self) -> &App {
        self.app.as_ref().unwrap()
    }

    fn init_describer<T: EntityAccept<A>, A: Describe + VisitEntity<T>>(entities: &[T]) -> A {
        let mut describer = A::new();
        for entity in entities {
            entity.accept_entity_visitor(&mut describer);
        }
        describer
    }

    pub fn describe_pipes(&self) -> Vec<String> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeDescriber>(pipes);
        describer.describe()
    }

    pub fn describe_pipe(&self, pid: &str) -> Result<String> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeDescriber>(pipes);
        describer.describe_pipe(pid)
    }

    pub fn describe_pipe_graph(&self) -> Vec<String> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.describe()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.describe_pipelines(pid)
    }

    pub fn get_pipe_component(&self, pid: &str) -> Result<Vec<String>> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.get_pipe_component(pid)
    }

    pub fn describe_objects(&self) -> Vec<String> {
        let objects = self.get_app().get_objects();
        let describer = Self::init_describer::<Object, ObjectDescriber>(objects);
        describer.describe()
    }

    pub fn describe_object(&self, oid: &str) -> Result<String> {
        let objects = self.get_app().get_objects();
        let describer = Self::init_describer::<Object, ObjectDescriber>(objects);
        describer.describe_object(oid)
    }
}

const PIPE_LABEL: &str = "pipe";
const SOURCE_PIPE_LABEL: &str = "source";
const SINK_PIPE_LABEL: &str = "sink";
const PIPE_COMPONENT_LABEL: &str = "union";
const PIPELINE_LABEL: &str = "pipeline";
const ENTITY_ID_LIST_SEP: &str = ", ";
const PIPE_DIRECT_SEP: &str = " -> ";
const OBJECTS_LABLE: &str = "objects";

#[cfg(test)]
mod tests {

    use crate::models::App;
    use std::path::Path;

    #[test]
    fn test_describe_timer_tick_pipe() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        for result in app.describe() {
            println!("{}", result)
        }
    }

    #[test]
    fn test_describe_timer_tick_pipeline() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::from_path(manifest_path).unwrap();
        app.validate().expect("expect valid");
        let pipelines = app.describe_pipelines("printer").expect("pipelines");
        assert_eq!(2, pipelines.len());
        for pipeline in pipelines {
            println!("{}", pipeline)
        }
    }
}
