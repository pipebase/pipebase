use crate::{
    api::{App, Entity, EntityAccept, Object, Pipe, VisitEntity},
    error::{api_error, Result},
    ops::utils::PipeGraph,
};

use std::{collections::HashMap, fmt};

pub trait Describe {
    fn new() -> Self;
    fn describe(&self) -> Vec<String>;
}

pub struct PipeGraphDescriber {
    graph: PipeGraph<Pipe>,
    pipe_ids: Vec<String>,
}

impl VisitEntity<Pipe> for PipeGraphDescriber {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.pipe_ids.push(pipe.get_id().to_owned());
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
        let mut results: Vec<String> = Vec::new();
        results.push(self.describe_source_pipe_ids());
        results.push(self.describe_sink_pipe_ids());
        results.extend(self.describe_pipe_components());
        results
    }
}

impl PipeGraphDescriber {
    pub(crate) fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let mut results: Vec<String> = Vec::new();
        for pipeline in &self.display_pipelines(pid)? {
            results.push(format!("{}", pipeline))
        }
        Ok(results)
    }

    pub(crate) fn describe_source_pipe_ids(&self) -> String {
        format!("{}", self.display_source_pipe_ids())
    }

    pub(crate) fn describe_sink_pipe_ids(&self) -> String {
        format!("{}", self.display_sink_pipe_ids())
    }

    pub(crate) fn describe_pipe_components(&self) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        for component in self.display_pipe_components() {
            results.push(format!("{}", component))
        }
        results
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
        let mut components_display: Vec<EntityIdsDisplay> = Vec::new();
        for component in self.get_pipe_components() {
            let component_display = EntityIdsDisplay {
                ids: component,
                sep: ENTITY_ID_LIST_SEP.to_owned(),
                label: Some(PIPE_COMPONENT_LABEL.to_owned()),
            };
            components_display.push(component_display);
        }
        components_display
    }

    fn display_pipelines(&self, pid: &str) -> Result<Vec<EntityIdsDisplay>> {
        let mut pipelines_display: Vec<EntityIdsDisplay> = Vec::new();
        for pipeline in self.get_pipelines(pid)? {
            let pipeline = self.format_pipeline_with_output_type(pipeline);
            let pipeline_display = EntityIdsDisplay {
                ids: pipeline,
                sep: PIPE_DIRECT_SEP.to_owned(),
                label: Some(PIPELINE_LABEL.to_owned()),
            };
            pipelines_display.push(pipeline_display);
        }
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

    fn get_pipe_output_type(&self, pid: &str) -> Option<String> {
        let pipe = self.graph.get_pipe_value(pid).unwrap();
        match pipe.get_output_data_type() {
            Some(output) => Some(output.to_literal(0)),
            None => None,
        }
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
        let mut components: Vec<Vec<String>> = Vec::new();
        for component in self.graph.find_components().values() {
            components.push(component.to_owned())
        }
        components
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
        Ok(object.to_literal(0))
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

    fn init_describer<T: EntityAccept<A>, A: Describe + VisitEntity<T>>(entities: &Vec<T>) -> A {
        let mut describer = A::new();
        for entity in entities {
            entity.accept(&mut describer);
        }
        describer
    }

    pub fn describe_pipes(&self) -> Vec<String> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.describe()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.describe_pipelines(pid)
    }

    pub fn get_pipelines(&self, pid: &str) -> Result<Vec<Vec<String>>> {
        let pipes = self.get_app().get_pipes();
        let describer = Self::init_describer::<Pipe, PipeGraphDescriber>(pipes);
        describer.get_pipelines(pid)
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

const SOURCE_PIPE_LABEL: &str = "source";
const SINK_PIPE_LABEL: &str = "sink";
const PIPE_COMPONENT_LABEL: &str = "union";
const PIPELINE_LABEL: &str = "pipeline";
const ENTITY_ID_LIST_SEP: &str = ", ";
const PIPE_DIRECT_SEP: &str = " -> ";
const OBJECTS_LABLE: &str = "objects";

#[cfg(test)]
mod tests {

    use crate::api::App;
    use std::path::Path;

    #[test]
    fn test_describe_timer_tick_pipe() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        for result in app.describe() {
            println!("{}", result)
        }
    }

    #[test]
    fn test_describe_timer_tick_pipeline() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        let pipelines = app.describe_pipelines("printer").expect("pipelines");
        assert_eq!(2, pipelines.len());
        for pipeline in pipelines {
            println!("{}", pipeline)
        }
    }
}
