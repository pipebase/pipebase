pub trait Bootstrap {
    fn print();
    fn run();
}

#[cfg(test)]
mod tests {
    use crate::Bootstrap;
    // use pipederive::Bootstrap;

    #[derive(Bootstrap)]
    #[pipe(
        name = "timer",
        kind = "source",
        config(ty = "TimeListenerConfig", path = "resources/catalogs/timer.yml"),
        output(ty = "()")
    )]
    struct App {}

    #[tokio::test]
    async fn test_bootstrap() {
        App::print();
    }
}
