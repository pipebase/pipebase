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
    const REDIS_CLIENT_CONFIG_FILE: &str = "resources/rediscli.yml";
    const PERIOD_FOR_BOOTSTRAP: u64 = 40000;
    const PERIOD_FOR_COMPLETE: u64 = 3000;

    #[derive(Serialize)]
    struct Record {
        key: String,
        value: u32,
    }

    #[tokio::test]
    async fn test_pubsub_rabbitmq() -> anyhow::Result<()> {
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
        let records = vec![
            Record {
                key: String::from("foo"),
                value: 1,
            },
            Record {
                key: String::from("bar"),
                value: 2,
            },
        ];
        let ingest_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_INGEST);
        let body = serde_json::to_vec(&records)?;
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
        // avro ser pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "avro_ser");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("avro_ser", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // amqp publisher
        let context_path = append_path(CONTEXT_SERVER_PIPE, "amqp_publisher");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("amqp_publisher", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // amqp consumer
        let context_path = append_path(CONTEXT_SERVER_PIPE, "amqp_consumer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("amqp_consumer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // avro deser pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "avro_deser");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("avro_deser", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // batch redis string writer pipe
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
        assert_eq!(1, value);
        let value = redis_client.get::<&str, u32>("bar")?.unwrap();
        assert_eq!(2, value);
        // shutdown all servers
        let shutdown_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_SHUTDOWN);
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
