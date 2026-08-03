use tokio::process::Command;
use tracing::debug;

pub async fn check(command: &str, timeout_secs: u64) -> anyhow::Result<bool> {
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let result =
        tokio::time::timeout(timeout, Command::new("sh").arg("-c").arg(command).output()).await;

    match result {
        Ok(Ok(output)) => {
            let healthy = output.status.success();
            debug!(
                command,
                exit_code = output.status.code(),
                healthy,
                "Exec check completed"
            );
            Ok(healthy)
        }
        Ok(Err(e)) => {
            debug!(command, error = %e, "Exec check failed to run");
            Ok(false)
        }
        Err(_) => {
            debug!(command, "Exec check timed out");
            Ok(false)
        }
    }
}
