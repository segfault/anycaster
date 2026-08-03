mod dns;
mod exec;
mod http;
mod ntp;
mod tcp;

use crate::config::Check;

pub async fn run(check: &Check) -> anyhow::Result<bool> {
    match check {
        Check::Tcp {
            host,
            port,
            timeout,
        } => tcp::check(host, *port, *timeout).await,
        Check::Http {
            url,
            expect_status,
            timeout,
        } => http::check(url, *expect_status, *timeout).await,
        Check::Dns {
            host,
            port,
            queries,
            timeout,
        } => dns::check(host, *port, queries, *timeout).await,
        Check::Ntp {
            host,
            port,
            max_stratum,
            timeout,
        } => ntp::check(host, *port, *max_stratum, *timeout).await,
        Check::Exec { command, timeout } => exec::check(command, *timeout).await,
    }
}
