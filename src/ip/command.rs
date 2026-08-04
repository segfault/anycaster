use std::net::Ipv4Addr;

use tokio::process::Command;
use tracing::{info, warn};

pub async fn is_present(ip: &str) -> anyhow::Result<bool> {
    let output = Command::new("ip")
        .args(["addr", "show", "dev", "lo"])
        .output()
        .await?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(&format!("{ip}/32")))
}

pub async fn add(ip: &str) -> anyhow::Result<()> {
    // Validate it's a real IPv4 address
    ip.parse::<Ipv4Addr>()?;

    let output = Command::new("ip")
        .args(["addr", "add", &format!("{ip}/32"), "dev", "lo"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "RTNETLINK answers: File exists" means it's already there — not an error
        if stderr.contains("File exists") {
            info!(ip, "IP already present on lo");
            return Ok(());
        }
        anyhow::bail!("failed to add {ip}/32 to lo: {stderr}");
    }

    info!(ip, "added IP to lo");
    Ok(())
}

pub async fn remove(ip: &str) -> anyhow::Result<()> {
    ip.parse::<Ipv4Addr>()?;

    let output = Command::new("ip")
        .args(["addr", "del", &format!("{ip}/32"), "dev", "lo"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // "Cannot assign requested address" means it's already gone
        if stderr.contains("Cannot assign requested address") {
            warn!(ip, "IP was not present on lo");
            return Ok(());
        }
        anyhow::bail!("failed to remove {ip}/32 from lo: {stderr}");
    }

    info!(ip, "removed IP from lo");
    Ok(())
}
