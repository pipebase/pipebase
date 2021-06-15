use crate::Context;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
pub trait Bootstrap {
    fn print();
    fn bootstrap(&self) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>;
    fn add_pipe_context(&self, _pipe_name: &str, _context: Arc<RwLock<Context>>) {}
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::future::Future;
    use std::pin::Pin;

    #[derive(Bootstrap)]
    #[pipe(
        name = "timer",
        kind = "listener",
        config(ty = "TimeListenerConfig", path = "resources/catalogs/timer.yml"),
        output(ty = "TimeListenerTick")
    )]
    #[pipe(
        name = "printer",
        kind = "sink",
        upstream = "timer",
        config(ty = "PrinterConfig")
    )]
    struct App {}

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
        let app = App {};
        app.bootstrap().await;
    }
}
