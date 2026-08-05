use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tracing::{info, warn};

use crate::config::LoadConfig;

pub fn overloaded() -> Arc<AtomicBool> {
    Arc::new(AtomicBool::new(false))
}

pub async fn monitor(config: &LoadConfig, flag: Arc<AtomicBool>) {
    let interval = std::time::Duration::from_secs(config.interval);
    let threshold = config.threshold;

    info!(
        threshold,
        interval_secs = config.interval,
        "load monitor started"
    );

    loop {
        tokio::time::sleep(interval).await;

        let load = match read_load_avg() {
            Ok(l) => l,
            Err(e) => {
                warn!(error = %e, "failed to read load average");
                continue;
            }
        };

        let was_overloaded = flag.load(Ordering::Relaxed);
        let is_overloaded = load >= threshold;

        if is_overloaded && !was_overloaded {
            warn!(load, threshold, "system overloaded, withdrawing all routes");
            flag.store(true, Ordering::Relaxed);
        } else if !is_overloaded && was_overloaded {
            info!(load, threshold, "system load recovered, allowing routes");
            flag.store(false, Ordering::Relaxed);
        }
    }
}

#[cfg(target_os = "linux")]
fn read_load_avg() -> anyhow::Result<f64> {
    let contents = std::fs::read_to_string("/proc/loadavg")?;
    let load_1m: f64 = contents
        .split_whitespace()
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty /proc/loadavg"))?
        .parse()?;
    Ok(load_1m)
}

#[cfg(not(target_os = "linux"))]
fn read_load_avg() -> anyhow::Result<f64> {
    // libc::getloadavg works on macOS/FreeBSD
    let mut loads = [0f64; 1];
    let ret = unsafe { libc::getloadavg(loads.as_mut_ptr(), 1) };
    if ret < 1 {
        anyhow::bail!("getloadavg failed");
    }
    Ok(loads[0])
}
