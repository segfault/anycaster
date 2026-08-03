use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::UdpSocket;
use tracing::debug;

const NTP_PACKET_SIZE: usize = 48;

pub async fn check(
    host: &str,
    port: u16,
    max_stratum: u8,
    timeout_secs: u64,
) -> anyhow::Result<bool> {
    let addr: SocketAddr = format!("{host}:{port}").parse()?;
    let timeout = Duration::from_secs(timeout_secs);

    // Build a minimal NTPv4 client request
    // LI=0, VN=4, Mode=3 (client) => 0b00_100_011 = 0x23
    let mut request = [0u8; NTP_PACKET_SIZE];
    request[0] = 0x23;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(addr).await?;

    let result = tokio::time::timeout(timeout, async {
        socket.send(&request).await?;
        let mut buf = [0u8; NTP_PACKET_SIZE];
        let n = socket.recv(&mut buf).await?;
        Ok::<_, anyhow::Error>(buf[..n].to_vec())
    })
    .await;

    match result {
        Ok(Ok(response)) if response.len() >= NTP_PACKET_SIZE => {
            let stratum = response[1];
            debug!(addr = %addr, stratum = stratum, max_stratum = max_stratum, "NTP response received");
            if stratum == 0 || stratum > max_stratum {
                debug!(addr = %addr, stratum = stratum, "NTP stratum out of range");
                return Ok(false);
            }
            Ok(true)
        }
        Ok(Ok(response)) => {
            debug!(addr = %addr, len = response.len(), "NTP response too short");
            Ok(false)
        }
        Ok(Err(e)) => {
            debug!(addr = %addr, error = %e, "NTP check failed");
            Ok(false)
        }
        Err(_) => {
            debug!(addr = %addr, "NTP check timed out");
            Ok(false)
        }
    }
}
