use tokio::net::TcpStream;
use tracing::debug;

pub async fn check(host: &str, port: u16, timeout_secs: u64) -> anyhow::Result<bool> {
    let addr = format!("{host}:{port}");
    let timeout = std::time::Duration::from_secs(timeout_secs);

    match tokio::time::timeout(timeout, TcpStream::connect(&addr)).await {
        Ok(Ok(_)) => {
            debug!(addr = %addr, "TCP connect succeeded");
            Ok(true)
        }
        Ok(Err(e)) => {
            debug!(addr = %addr, error = %e, "TCP connect failed");
            Ok(false)
        }
        Err(_) => {
            debug!(addr = %addr, "TCP connect timed out");
            Ok(false)
        }
    }
}
