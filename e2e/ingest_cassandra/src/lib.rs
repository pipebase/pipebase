#[cfg(test)]
#[cfg(feature = "itest")]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use e2e_clients::{
        append_path, build_url, wait, CqlClient, CqlClientConfig, HttpClient, HttpClientConfig,
        CONTEXT_SERVER_HEALTH, CONTEXT_SERVER_PIPE, CONTEXT_SERVER_SHUTDOWN,
        INGESTION_SERVER_HEALTH, INGESTION_SERVER_INGEST, INGESTION_SERVER_SHUTDOWN,
    };
    use pipebase::common::{ConfigInto, FromPath, PipeContext, Timestamp};
    use serde::Serialize;

    const HTTP_CLIENT_CONFIG_FILE: &str = "resources/httpcli.yml";
    const CONTEXT_SERVER_ADDRESS: &str = "http://127.0.0.1:8000";
    const INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9000";
    const BATCH_INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9001";
    const CQL_CLIENT_CONFIG_FILE: &str = "resources/cqlcli.yml";
    const PERIOD_FOR_BOOTSTRAP: u64 = 120000;
    const PERIOD_FOR_COMPLETE: u64 = 3000;
    const CREATE_KEYSPACE: &str = "CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 1}";
    const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS test.records (key text PRIMARY KEY, value int, date date, timestamp timestamp )";
    const SELECT_FOO: &str =
        "SELECT key, value, date, timestamp FROM test.records WHERE key = 'foo'";
    const SELECT_BAR: &str =
        "SELECT key, value, date, timestamp FROM test.records WHERE key = 'bar'";

    #[derive(Serialize)]
    struct Record {
        key: String,
        value: i32,
        date: String,
        timestamp: String,
    }

    #[derive(Serialize)]
    struct AnotherRecord {
        key: String,
        value: i32,
        date: String,
        timestamp: Timestamp,
    }

    #[tokio::test]
    async fn test_ingest_cassandra() -> anyhow::Result<()> {
        // setup clients
        let cql_client_config = CqlClientConfig::from_path(CQL_CLIENT_CONFIG_FILE).await?;
        let cql_client: CqlClient = cql_client_config.config_into().await?;
        let http_client_config = HttpClientConfig::from_path(HTTP_CLIENT_CONFIG_FILE).await?;
        let http_client: HttpClient = http_client_config.config_into().await?;
        wait(PERIOD_FOR_BOOTSTRAP).await;
        // context server health check
        let health_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // ingestion server health check
        let health_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // create keyspace and table
        cql_client.execute(CREATE_KEYSPACE).await?;
        cql_client.execute(CREATE_TABLE).await?;
        // ingest record
        let record = Record {
            key: String::from("foo"),
            value: 1,
            date: String::from("2021-08-21"),
            timestamp: String::from("2021-08-21T22:45:53"),
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
        // cql writer pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "cql_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("cql_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // query cassandra
        let result = cql_client.execute(SELECT_FOO).await?;
        let mut rows = result.rows.expect("expect record 'foo'");
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let (key, value, date, timestamp) =
            row.into_typed::<(String, i32, chrono::NaiveDate, chrono::Duration)>()?;
        assert_eq!("foo", &key);
        assert_eq!(1, value);
        let expected_date = NaiveDate::from_ymd(2021, 8, 21);
        assert_eq!(expected_date, date);
        // https://docs.rs/scylla/latest/src/scylla/frame/value.rs.html#46
        let expected_datetime: NaiveDateTime = NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 53);
        let actual_datetime: NaiveDateTime =
            NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0) + timestamp;
        assert_eq!(expected_datetime, actual_datetime);
        // batch ingest
        let records = vec![
            AnotherRecord {
                key: String::from("foo"),
                value: 1,
                date: String::from("2021-08-21"),
                timestamp: Timestamp::Secs(1629585954),
            },
            AnotherRecord {
                key: String::from("foo"),
                value: 2,
                date: String::from("2021-08-21"),
                timestamp: Timestamp::Secs(1629585954),
            },
            AnotherRecord {
                key: String::from("foo"),
                value: 3,
                date: String::from("2021-08-21"),
                timestamp: Timestamp::Secs(1629585954),
            },
            AnotherRecord {
                key: String::from("bar"),
                value: 1,
                date: String::from("2021-08-21"),
                timestamp: Timestamp::Secs(1629585954),
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
        // batch swap pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_swap");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_swap", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // batch cql writer pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_cql_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_cql_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // query cassandra for 'foo'
        let result = cql_client.execute(SELECT_FOO).await?;
        let mut rows = result.rows.expect("expect record 'foo'");
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let (key, value, date, timestamp) =
            row.into_typed::<(String, i32, chrono::NaiveDate, chrono::Duration)>()?;
        assert_eq!("foo", &key);
        assert_eq!(3, value);
        let expected_date = NaiveDate::from_ymd(2021, 8, 21);
        assert_eq!(expected_date, date);
        // https://docs.rs/scylla/latest/src/scylla/frame/value.rs.html#46
        let expected_datetime: NaiveDateTime = NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 54);
        let actual_datetime: NaiveDateTime =
            NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0) + timestamp;
        assert_eq!(expected_datetime, actual_datetime);
        // query cassandra for 'bar'
        let result = cql_client.execute(SELECT_BAR).await?;
        let mut rows = result.rows.expect("expect record 'bar'");
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let (key, value, date, timestamp) =
            row.into_typed::<(String, i32, chrono::NaiveDate, chrono::Duration)>()?;
        assert_eq!("bar", &key);
        assert_eq!(1, value);
        let expected_date = NaiveDate::from_ymd(2021, 8, 21);
        assert_eq!(expected_date, date);
        // https://docs.rs/scylla/latest/src/scylla/frame/value.rs.html#46
        let expected_datetime: NaiveDateTime = NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 54);
        let actual_datetime: NaiveDateTime =
            NaiveDate::from_ymd(1970, 1, 1).and_hms(0, 0, 0) + timestamp;
        assert_eq!(expected_datetime, actual_datetime);
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
