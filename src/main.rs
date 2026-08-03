mod checks;
mod config;
mod ip;
mod runner;

use std::path::PathBuf;

use clap::Parser;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "anycaster", version, about = "Anycast health checker and IP advertiser")]
struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "/etc/anycaster/config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let cfg = config::load(&cli.config)?;

    info!(
        services = cfg.services.len(),
        on_exit = ?cfg.defaults.on_exit,
        "loaded configuration"
    );

    let on_exit = cfg.defaults.on_exit;
    let service_ips: Vec<String> = cfg.services.iter().map(|s| s.ip.clone()).collect();

    let mut handles = Vec::new();
    for service in cfg.services {
        let defaults = cfg.defaults.clone();
        let handle = tokio::spawn(async move {
            runner::run_service(&service, &defaults).await;
        });
        handles.push(handle);
    }

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("received shutdown signal");

    // Abort all service tasks
    for handle in &handles {
        handle.abort();
    }

    // Handle exit behavior
    if on_exit == config::OnExit::Withdraw {
        info!("withdrawing all IPs on exit");
        for svc_ip in &service_ips {
            if let Err(e) = ip::remove(svc_ip).await {
                warn!(ip = %svc_ip, error = %e, "failed to withdraw IP on exit");
            }
        }
    } else {
        info!("maintaining IPs on exit");
    }

    Ok(())
}
