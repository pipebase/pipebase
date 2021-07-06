use crate::Context;

pub trait Bootstrap {
    fn print();
    fn bootstrap(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + Sync>>;
}

#[cfg(test)]
mod tests {
    use crate::*;

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
    struct App {}

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
        let mut app = App {};
        app.bootstrap().await;
        /*
        let timer_context = app.get_pipe_context("timer1").unwrap();
        let printer_context = app.get_pipe_context("printer").unwrap();
        assert_eq!(State::Done, timer_context.get_state());
        assert_eq!(State::Done, printer_context.get_state());
        assert_eq!(11, timer_context.get_total_run());
        assert_eq!(21, printer_context.get_total_run());
        assert_eq!(11, timer_context.get_success_run());
        assert_eq!(21, printer_context.get_success_run());
        */
    }
}
