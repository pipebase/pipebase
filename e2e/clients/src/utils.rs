use tokio::time::{sleep, Duration};

pub async fn wait(millis: u64) {
    sleep(Duration::from_millis(millis)).await;
}

pub fn build_url(base: &str, path: &str) -> String {
    format!("{}{}", base, path)
}

pub fn append_path(base: &str, path: &str) -> String {
    format!("{}/{}", base, path)
}
