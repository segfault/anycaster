use tracing::debug;

pub async fn check(url: &str, expect_status: u16, timeout_secs: u64) -> anyhow::Result<bool> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()?;

    match client.get(url).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let healthy = status == expect_status;
            debug!(url, status, healthy, "HTTP check completed");
            Ok(healthy)
        }
        Err(e) => {
            debug!(url, error = %e, "HTTP check failed");
            Ok(false)
        }
    }
}
