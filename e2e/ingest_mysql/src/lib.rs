#[cfg(test)]
#[cfg(feature = "itest")]
mod tests {
    use chrono::{NaiveDate, NaiveDateTime};
    use e2e_clients::{
        append_path, build_url, wait, HttpClient, HttpClientConfig, MySQLClient, MySQLClientConfig,
        CONTEXT_SERVER_HEALTH, CONTEXT_SERVER_PIPE, CONTEXT_SERVER_SHUTDOWN,
        INGESTION_SERVER_HEALTH, INGESTION_SERVER_INGEST, INGESTION_SERVER_SHUTDOWN,
    };
    use pipebase::common::{ConfigInto, FromPath, PipeContext};
    use serde::Serialize;

    const HTTP_CLIENT_CONFIG_FILE: &str = "resources/httpcli.yml";
    const CONTEXT_SERVER_ADDRESS: &str = "http://127.0.0.1:8000";
    const INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9000";
    const BATCH_INGESTION_SERVER_ADDRESS: &str = "http://127.0.0.1:9001";
    const MYSQL_CLIENT_CONFIG_FILE: &str = "resources/mysqlcli.yml";
    const PERIOD_FOR_BOOTSTRAP: u64 = 6000;
    const PERIOD_FOR_COMPLETE: u64 = 3000;
    const CREATE_TABLE: &str = "CREATE TABLE IF NOT EXISTS records ( `key` VARCHAR(64) NOT NULL PRIMARY KEY, `value` INTEGER, `timestamp` TIMESTAMP )";
    const SELECT_FOO: &str = "SELECT `key`, `value`, `timestamp` FROM records WHERE `key` = 'foo'";
    const SELECT_BAR: &str = "SELECT `key`, `value`, `timestamp` FROM records WHERE `key` = 'bar'";

    #[derive(Serialize)]
    struct Record {
        key: String,
        value: i32,
        timestamp: String,
    }

    #[derive(Serialize)]
    struct AnotherRecord {
        id: String,
        value: i32,
        timestamp: String,
    }

    #[tokio::test]
    async fn test_ingest_mysql() -> anyhow::Result<()> {
        // setup clients
        let mysql_client_config = MySQLClientConfig::from_path(MYSQL_CLIENT_CONFIG_FILE).await?;
        let mysql_client: MySQLClient = mysql_client_config.config_into().await?;
        let http_client_config = HttpClientConfig::from_path(HTTP_CLIENT_CONFIG_FILE).await?;
        let http_client: HttpClient = http_client_config.config_into().await?;
        wait(PERIOD_FOR_BOOTSTRAP).await;
        // context server health check
        let health_url = build_url(CONTEXT_SERVER_ADDRESS, CONTEXT_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // ingestion server health check
        let health_url = build_url(INGESTION_SERVER_ADDRESS, INGESTION_SERVER_HEALTH);
        http_client.get_assert_ok::<String>(health_url).await?;
        // create table
        mysql_client.execute(CREATE_TABLE).await?;
        let record = Record {
            key: String::from("foo"),
            value: 1,
            timestamp: String::from("2021-08-21T22:45:53"),
        };
        // ingest record
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
        let context_path = append_path(CONTEXT_SERVER_PIPE, "mysql_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("mysql_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // query mysql
        let mut rows = mysql_client.execute(SELECT_FOO).await?;
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let key = row
            .get::<String, usize>(0)
            .expect("'key' not found at column 0");
        let value = row
            .get::<i32, usize>(1)
            .expect("'value' not found at column 1");
        let timestamp = row
            .get::<NaiveDateTime, usize>(2)
            .expect("'timestamp' not found at column 2");
        assert_eq!("foo", &key);
        assert_eq!(1, value);
        assert_eq!(
            NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 53),
            timestamp
        );
        // batch ingest records
        let records = vec![
            AnotherRecord {
                id: String::from("foo"),
                value: 1,
                timestamp: String::from("2021-08-21T22:45:54"),
            },
            AnotherRecord {
                id: String::from("foo"),
                value: 2,
                timestamp: String::from("2021-08-21T22:45:54"),
            },
            AnotherRecord {
                id: String::from("foo"),
                value: 3,
                timestamp: String::from("2021-08-21T22:45:54"),
            },
            AnotherRecord {
                id: String::from("bar"),
                value: 1,
                timestamp: String::from("2021-08-21T22:45:54"),
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
        // batch mysql writer pipe
        let context_path = append_path(CONTEXT_SERVER_PIPE, "batch_mysql_writer");
        let context_url = build_url(CONTEXT_SERVER_ADDRESS, &context_path);
        let context = http_client
            .get_json::<String, PipeContext>(context_url)
            .await?;
        assert_eq!("batch_mysql_writer", context.get_name());
        assert_eq!("receive", context.get_state());
        assert_eq!(1, context.get_total_run());
        assert_eq!(0, context.get_failure_run());
        // query mysql
        let mut rows = mysql_client.execute(SELECT_FOO).await?;
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let key = row
            .get::<String, usize>(0)
            .expect("'key' not found at column 0");
        let value = row
            .get::<i32, usize>(1)
            .expect("'value' not found at column 1");
        let timestamp = row
            .get::<NaiveDateTime, usize>(2)
            .expect("'timestamp' not found at column 2");
        assert_eq!("foo", &key);
        assert_eq!(3, value);
        assert_eq!(
            NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 54),
            timestamp
        );
        // query mysql
        let mut rows = mysql_client.execute(SELECT_BAR).await?;
        assert_eq!(1, rows.len());
        let row = rows.remove(0);
        let key = row
            .get::<String, usize>(0)
            .expect("'key' not found at column 0");
        let value = row
            .get::<i32, usize>(1)
            .expect("'value' not found at column 1");
        let timestamp = row
            .get::<NaiveDateTime, usize>(2)
            .expect("'timestamp' not found at column 2");
        assert_eq!("bar", &key);
        assert_eq!(1, value);
        assert_eq!(
            NaiveDate::from_ymd(2021, 8, 21).and_hms(22, 45, 54),
            timestamp
        );
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
