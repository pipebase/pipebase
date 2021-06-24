use crate::Context;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait ContextStore {
    fn add_pipe_context(&mut self, pipe_name: String, context: Arc<RwLock<Context>>);
    fn get_pipe_context(&self, pipe_name: &str) -> Option<Arc<RwLock<Context>>>;
}
pub trait Bootstrap: ContextStore {
    fn print();
    fn bootstrap(&mut self) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashMap;
    use std::future::Future;
    use std::ops::Deref;
    use std::pin::Pin;

    #[derive(Bootstrap, ContextStore)]
    #[pipe(
        name = "timer1",
        ty = "poller",
        config(ty = "TimerConfig", path = "resources/catalogs/timer.yml"),
        output = "u128"
    )]
    #[pipe(
        name = "timer2",
        ty = "poller",
        config(ty = "TimerConfig", path = "resources/catalogs/timer.yml"),
        output = "u128"
    )]
    #[pipe(
        name = "printer",
        ty = "exporter",
        upstream = "timer1, timer2",
        config(ty = "PrinterConfig")
    )]
    #[cstore(method(get = "get", insert = "insert"))]
    struct App {
        #[cstore]
        pipe_contexts: HashMap<String, Arc<RwLock<Context>>>,
    }

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
        let mut app = App {
            pipe_contexts: HashMap::new(),
        };
        app.bootstrap().await;
        let timer_context = app.get_pipe_context("timer1").unwrap();
        let timer_context = timer_context.deref().read().await;
        let printer_context = app.get_pipe_context("printer").unwrap();
        let printer_context = printer_context.deref().read().await;
        assert_eq!(State::Done, timer_context.get_state());
        assert_eq!(State::Done, printer_context.get_state());
        assert_eq!(11, timer_context.get_total_run());
        assert_eq!(21, printer_context.get_total_run());
        assert_eq!(11, timer_context.get_success_run());
        assert_eq!(21, printer_context.get_success_run());
    }
}
