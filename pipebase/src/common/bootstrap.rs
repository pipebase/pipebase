pub trait Bootstrap {
    fn print();
    fn bootstrap(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(Bootstrap)]
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
    #[cstore(
        name = "context_printer",
        config(
            ty = "ContextPrinterConfig",
            path = "resources/catalogs/context_printer.yml"
        )
    )]
    struct App {}

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
        let mut app = App {};
        app.bootstrap().await;
    }
}
