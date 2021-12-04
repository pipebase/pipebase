#[cfg(test)]
#[cfg(feature = "itest")]
mod tests {

    use e2e_clients::{
        append_path, build_url, wait, HttpClient, HttpClientConfig, CONTEXT_SERVER_HEALTH,
        CONTEXT_SERVER_PIPE, CONTEXT_SERVER_SHUTDOWN, INGESTION_SERVER_HEALTH,
        INGESTION_SERVER_INGEST, INGESTION_SERVER_SHUTDOWN,
    };
    use pipebase::common::{ConfigInto, FromPath, PipeContext};
    use serde::{Deserialize, Serialize};

    const HTTP_CLIENT_CONFIG_FILE: &str = "resources/httpcli.yml";
    const CONTEXT_SERVER_ADDRESS: &str = "http://127.0.0.1:8000";
    const INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9000";
    const ELASTIC_SEARCH_ADDRESS: &str = "http://127.0.0.1:9200";
    const ELASTIC_SEARCH_HEALTH: &str = "/_cluster/health";
    const TEST_INDEX: &str = "/records";
    const TEST_INDEX_SEARCH: &str = "/records/_search";
    const PERIOD_FOR_BOOTSTRAP: u64 = 30000;
    const PERIOD_FOR_COMPLETE: u64 = 8000;

    #[derive(Serialize, Deserialize)]
    struct Record {
        key: String,
        value: u32,
    }

    #[derive(Serialize)]
    struct RecordDocument {
        id: u32,
        key: String,
        value: u32,
    }

    #[derive(Serialize)]
    struct DocumentQuery {
        q: String,
        size: usize,
    }

    #[derive(Deserialize)]
    struct DocumentHit<T> {
        _id: String,
        _source: T,
    }

    #[derive(Deserialize)]
    struct DocumentHitsTotal {
        value: usize,
    }

    #[derive(Deserialize)]
    struct DocumentHits<T> {
        total: DocumentHitsTotal,
        hits: Vec<DocumentHit<T>>,
    }

    #[derive(Deserialize)]
    struct DocumentQueryResult<T> {
        hits: DocumentHits<T>,
    }

    #[derive(Serialize)]
    struct HealthQuery {
        wait_for_status: String,
        timeout: String,
    }

    #[tokio::test]
    async fn test_ingest_elasticsearch() -> anyhow::Result<()> {
        // setup clients
        let http_client_config = HttpClientConfig::from_path(HTTP_CLIENT_CONFIG_FILE).await?;
        let http_client: HttpClient = http_client_config.config_into().await?;
        wait(PERIOD_FOR_BOOTSTRAP).await;
        // check elastic search
        let health_url = build_url(ELASTIC_SEARCH_ADDRESS, ELASTIC_SEARCH_HEALTH);
        let query = HealthQuery {
            wait_for_status: String::from("green"),
            timeout: String::from("60s"),
        };
        let _ = http_client.query_assert_ok(health_url, Some(query)).await;
        // context server health check
        let health_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // ingestion server health check
        let health_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // create index
        let index_url = build_url(ELASTIC_SEARCH_ADDRESS, TEST_INDEX);
        let _ = http_client
            .put_assert_ok::<String, String>(index_url, None)
            .await;
        // ingest record
        let count: usize = 10;
        for i in 0..count {
            let record = RecordDocument {
                id: i as u32,
                key: format!("{}", i),
                value: i as u32,
            };
            let ingest_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_INGEST);
            let body = serde_json::to_vec(&record)?;
            http_client
                .post_assert_ok::<String, Vec<u8>>(ingest_url, Some(body))
                .await?;
        }

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
        assert_eq!(10, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // json des pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "json");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("json", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(10, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // text collector
        let context_path = append_path(CONTEXT_SERVER_PIPE, "text_collector");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("text_collector", context.get_name());
        assert_eq!("receive", context.get_state());
        assert!(context.get_total_run() >= 1);
        assert_eq!(0, context.get_failure_run());
        // reqwest poster
        let context_path = append_path(CONTEXT_SERVER_PIPE, "reqwest_poster");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("reqwest_poster", context.get_name());
        assert_eq!("receive", context.get_state());
        assert!(context.get_total_run() >= 1);
        assert_eq!(0, context.get_failure_run());
        // check elastic search
        let search_url = build_url(ELASTIC_SEARCH_ADDRESS, TEST_INDEX_SEARCH);
        let document_query = DocumentQuery {
            q: String::from("key:*"),
            size: count + 1, // avoid partial result
        };
        let query_result = http_client
            .query_json::<String, DocumentQuery, DocumentQueryResult<Record>>(
                search_url,
                Some(document_query),
            )
            .await?;
        let total = query_result.hits.total.value;
        assert_eq!(count, total);
        // expect document id is identical to record key
        let hits = query_result.hits.hits;
        for hit in hits {
            let id = hit._id;
            let record = hit._source;
            assert_eq!(id, record.key);
        }
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
