use jiff::Timestamp;
use tracing::{error, info, warn};

use crate::checks;
use crate::config::{Defaults, Service};
use crate::ip;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Healthy,
    Unhealthy,
}

pub async fn run_service(service: &Service, defaults: &Defaults) {
    let interval = std::time::Duration::from_secs(service.check_interval(defaults));
    let rise = service.rise(defaults);
    let fall = service.fall(defaults);

    // Check if the IP is already on lo to determine initial state
    let already_present = match ip::is_present(&service.ip).await {
        Ok(v) => v,
        Err(e) => {
            error!(
                service = %service.name,
                ip = %service.ip,
                error = %e,
                "failed to check initial IP state, assuming unhealthy"
            );
            false
        }
    };

    let mut state = if already_present {
        info!(
            service = %service.name,
            ip = %service.ip,
            "IP already present on lo, starting as healthy"
        );
        State::Healthy
    } else {
        info!(
            service = %service.name,
            ip = %service.ip,
            "IP not present on lo, starting as unhealthy"
        );
        State::Unhealthy
    };

    let mut consecutive_ok: u32 = if state == State::Healthy { rise } else { 0 };
    let mut consecutive_fail: u32 = if state == State::Unhealthy { fall } else { 0 };

    loop {
        tokio::time::sleep(interval).await;

        let healthy = match checks::run(&service.check).await {
            Ok(h) => h,
            Err(e) => {
                warn!(
                    service = %service.name,
                    error = %e,
                    "health check error, treating as failure"
                );
                false
            }
        };

        if healthy {
            consecutive_fail = 0;
            consecutive_ok = consecutive_ok.saturating_add(1);

            if state == State::Unhealthy && consecutive_ok >= rise {
                info!(
                    service = %service.name,
                    ip = %service.ip,
                    at = %Timestamp::now(),
                    "service is healthy, advertising"
                );
                if let Err(e) = ip::add(&service.ip).await {
                    error!(
                        service = %service.name,
                        ip = %service.ip,
                        error = %e,
                        "failed to add IP"
                    );
                    continue;
                }
                state = State::Healthy;
            }
        } else {
            consecutive_ok = 0;
            consecutive_fail = consecutive_fail.saturating_add(1);

            if state == State::Healthy && consecutive_fail >= fall {
                warn!(
                    service = %service.name,
                    ip = %service.ip,
                    at = %Timestamp::now(),
                    "service is unhealthy, withdrawing"
                );
                if let Err(e) = ip::remove(&service.ip).await {
                    error!(
                        service = %service.name,
                        ip = %service.ip,
                        error = %e,
                        "failed to remove IP"
                    );
                    continue;
                }
                state = State::Unhealthy;
            }
        }
    }
}
