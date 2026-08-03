use std::net::SocketAddr;
use std::time::Duration;

use hickory_resolver::config::{NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::proto::rr::RecordType;
use hickory_resolver::proto::xfer::Protocol;
use hickory_resolver::{ResolveError, TokioResolver};
use tracing::debug;

use crate::config::DnsQuery;

pub async fn check(
    host: &str,
    port: u16,
    queries: &[DnsQuery],
    timeout_secs: u64,
) -> anyhow::Result<bool> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let timeout = Duration::from_secs(timeout_secs);

    let mut rc = ResolverConfig::new();
    rc.add_name_server(NameServerConfig::new(addr, Protocol::Udp));

    let mut opts = ResolverOpts::default();
    opts.timeout = timeout;
    opts.attempts = 1;

    let mut builder = TokioResolver::builder_with_config(rc, Default::default());
    *builder.options_mut() = opts;
    let resolver = builder.build();

    let check_queries: Vec<(String, String)> = if queries.is_empty() {
        vec![("google.com.".to_string(), "A".to_string())]
    } else {
        queries
            .iter()
            .map(|q| (q.name.clone(), q.record_type.clone()))
            .collect()
    };

    for (name, rtype) in &check_queries {
        let record_type = match rtype.as_str() {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "MX" => RecordType::MX,
            "TXT" => RecordType::TXT,
            "CNAME" => RecordType::CNAME,
            "NS" => RecordType::NS,
            "SOA" => RecordType::SOA,
            "SRV" => RecordType::SRV,
            "PTR" => RecordType::PTR,
            other => anyhow::bail!("unsupported DNS record type: {other}"),
        };

        let result: Result<(), ResolveError> = match record_type {
            RecordType::A | RecordType::AAAA => resolver.lookup_ip(name.as_str()).await.map(|_| ()),
            rt => resolver.lookup(name.as_str(), rt).await.map(|_| ()),
        };

        match result {
            Ok(_) => {
                debug!(
                    query_name = %name,
                    query_type = %rtype,
                    "DNS query succeeded"
                );
            }
            Err(e) => {
                debug!(
                    query_name = %name,
                    query_type = %rtype,
                    error = %e,
                    "DNS query failed"
                );
                return Ok(false);
            }
        }
    }

    Ok(true)
}
