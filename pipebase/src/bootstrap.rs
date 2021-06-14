pub trait Bootstrap {
    fn print();
    fn bootstrap();
}

#[cfg(test)]
mod tests {
    use crate::*;

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
        config(ty = "PrinterConfig", path = "")
    )]
    struct App {}

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
    }
}
