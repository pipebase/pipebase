#[cfg(test)]
#[cfg(feature = "itest")]
mod tests {

    use e2e_clients::{
        append_path, build_url, wait, HttpClient, HttpClientConfig, RedisClient, RedisClientConfig,
        CONTEXT_SERVER_HEALTH, CONTEXT_SERVER_PIPE, CONTEXT_SERVER_SHUTDOWN,
        INGESTION_SERVER_HEALTH, INGESTION_SERVER_INGEST, INGESTION_SERVER_SHUTDOWN,
    };
    use pipebase::common::{ConfigInto, FromPath, PipeContext};
    use serde::Serialize;

    const HTTP_CLIENT_CONFIG_FILE: &str = "resources/httpcli.yml";
    const CONTEXT_SERVER_ADDRESS: &str = "http://127.0.0.1:8000";
    const INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9000";
    const BATCH_INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9001";
    const REDIS_CLIENT_CONFIG_FILE: &str = "resources/rediscli.yml";
    const PERIOD_FOR_BOOTSTRAP: u64 = 5000;
    const PERIOD_FOR_COMPLETE: u64 = 3000;

    #[derive(Serialize)]
    struct Record {
        key: String,
        value: u32,
    }

    #[tokio::test]
    async fn test_ingest_redis() -> anyhow::Result<()> {
        // setup clients
        let redis_client_config = RedisClientConfig::from_path(REDIS_CLIENT_CONFIG_FILE).await?;
        let http_client_config = HttpClientConfig::from_path(HTTP_CLIENT_CONFIG_FILE).await?;
        let mut redis_client: RedisClient = redis_client_config.config_into().await?;
        let http_client: HttpClient = http_client_config.config_into().await?;
        wait(PERIOD_FOR_BOOTSTRAP).await;
        // context server health check
        let health_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // ingestion server health check
        let health_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // ingest record
        let record = Record {
            key: String::from("foo"),
            value: 1,
        };
        let ingest_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_INGEST);
        let body = serde_json::to_vec(&record)?;
        http_client
            .post_assert_ok::<String, Vec<u8>>(ingest_url, Some(body))
            .await?;
        // wait for complete
        wait(PERIOD_FOR_COMPLETE).await;
        // check context
        // ingestion server pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "ingestion_server");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("ingestion_server", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // json des pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "json");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("json", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // redis string writer pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "redis_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("redis_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // check redis
        let value = redis_client.get::<&str, u32>("foo")?.unwrap();
        assert_eq!(1, value);
        // batch ingestion server health check
        let health_url = build_url(BATCH_INGESTION_SERVER_ADDRESS, INGESTION_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // batch ingest records
        let records = vec![
            Record {
                key: String::from("foo"),
                value: 1,
            },
            Record {
                key: String::from("foo"),
                value: 2,
            },
            Record {
                key: String::from("foo"),
                value: 3,
            },
            Record {
                key: String::from("bar"),
                value: 1,
            },
        ];
        let batch_ingest_url = build_url(BATCH_INGESTION_SERVER_ADDRESS, INGESTION_SERVER_INGEST);
        let body = serde_json::to_vec(&records)?;
        http_client
            .post_assert_ok::<String, Vec<u8>>(batch_ingest_url, Some(body))
            .await?;
        // wait for complete
        wait(PERIOD_FOR_COMPLETE).await;
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_ingestion_server");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_ingestion_server", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // json des pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_json");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_json", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // redis string writer pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_redis_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_redis_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // check redis
        let value = redis_client.get::<&str, u32>("foo")?.unwrap();
        assert_eq!(3, value);
        let value = redis_client.get::<&str, u32>("bar")?.unwrap();
        assert_eq!(1, value);
        // shutdown all servers
        let shutdown_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_SHUTDOWN);
        http_client
            .post_assert_ok::<String, String>(shutdown_url, None)
            .await?;
        let shutdown_url = build_url(BATCH_INGESTION_SERVER_ADDRESS, INGESTION_SERVER_SHUTDOWN);
        http_client
            .post_assert_ok::<String, String>(shutdown_url, None)
            .await?;
        let shutdown_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_SHUTDOWN);
        http_client
            .post_assert_ok::<String, String>(shutdown_url, None)
            .await?;
        Ok(())
    }
}
