#[cfg(test)]
#[cfg(feature = "itest")]
mod tests {

    use e2e_clients::{
        append_path, build_url, wait, HttpClient, HttpClientConfig, CONTEXT_SERVER_HEALTH,
        CONTEXT_SERVER_PIPE, CONTEXT_SERVER_SHUTDOWN,
    };
    use pipebase::common::{ConfigInto, FromPath, PipeContext};
    use reqwest::StatusCode;

    const HTTP_CLIENT_CONFIG_FILE: &str = "resources/httpcli.yml";
    const CONTEXT_SERVER_ADDRESS: &str = "http://127.0.0.1:8000";
    const PERIOD_FOR_COMPLETE: u64 = 40000;

    #[tokio::test]
    async fn test_timer() -> anyhow::Result<()> {
        wait(PERIOD_FOR_COMPLETE).await;
        // build http client
        let config = HttpClientConfig::from_path(HTTP_CLIENT_CONFIG_FILE).await?;
        let client: HttpClient = config.config_into().await?;
        // check context server health
        let health_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_HEALTH);
        let response = client.get::<String>(health_url).await?;
        let status = response.status();
        assert_eq!(StatusCode::OK, status);
        // check each pipe context
        let timer1_path = append_path(CONTEXT_SERVER_PIPE, "timer1");
        let timer1_url = build_url(CONTEXT_SERVER_ADDRESS, &timer1_path);
        let timer1_context = client.get_json::<String, PipeContext>(timer1_url).await?;
        assert_eq!("timer1", timer1_context.get_name());
        assert_eq!("done", timer1_context.get_state());
        assert_eq!(10, timer1_context.get_total_run());
        assert_eq!(0, timer1_context.get_failure_run());
        let timer2_path = append_path(CONTEXT_SERVER_PIPE, "timer2");
        let timer2_url = build_url(CONTEXT_SERVER_ADDRESS, &timer2_path);
        let timer2_context = client.get_json::<String, PipeContext>(timer2_url).await?;
        assert_eq!("timer2", timer2_context.get_name());
        assert_eq!("done", timer2_context.get_state());
        assert_eq!(20, timer2_context.get_total_run());
        assert_eq!(0, timer2_context.get_failure_run());
        let printer_path = append_path(CONTEXT_SERVER_PIPE, "printer");
        let printer_url = build_url(CONTEXT_SERVER_ADDRESS, &printer_path);
        let printer_context = client.get_json::<String, PipeContext>(printer_url).await?;
        assert_eq!("printer", printer_context.get_name());
        assert_eq!("done", printer_context.get_state());
        assert_eq!(30, printer_context.get_total_run());
        assert_eq!(0, printer_context.get_failure_run());
        // shutdown
        let shutdown_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_SHUTDOWN);
        let response = client.post::<String, String>(shutdown_url, None).await?;
        let status = response.status();
        assert_eq!(StatusCode::OK, status);
        Ok(())
    }
}
