use std::net::Ipv4Addr;

use rtnetlink::new_connection;
use tracing::{info, warn};

const LOOPBACK_INDEX: u32 = 1;
const PREFIX_LEN: u8 = 32;

pub async fn is_present(ip: &str) -> anyhow::Result<bool> {
    let addr: Ipv4Addr = ip.parse()?;
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut addrs = handle
        .address()
        .get()
        .set_link_index_filter(LOOPBACK_INDEX)
        .execute();

    use futures_util::TryStreamExt;
    while let Some(msg) = addrs.try_next().await? {
        if msg.header.prefix_len == PREFIX_LEN {
            for nla in &msg.attributes {
                if let rtnetlink::netlink_packet_route::address::AddressAttribute::Address(a) = nla
                {
                    if *a == std::net::IpAddr::V4(addr) {
                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

pub async fn add(ip: &str) -> anyhow::Result<()> {
    let addr: Ipv4Addr = ip.parse()?;
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let result = handle
        .address()
        .add(LOOPBACK_INDEX, std::net::IpAddr::V4(addr), PREFIX_LEN)
        .execute()
        .await;

    match result {
        Ok(()) => {
            info!(ip, "added IP to lo");
            Ok(())
        }
        Err(e) => {
            // EEXIST (errno 17) means it's already there
            let msg = e.to_string();
            if msg.contains("File exists") || msg.contains("errno 17") {
                info!(ip, "IP already present on lo");
                Ok(())
            } else {
                Err(e.into())
            }
        }
    }
}

pub async fn remove(ip: &str) -> anyhow::Result<()> {
    let addr: Ipv4Addr = ip.parse()?;
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let result = handle
        .address()
        .del(handle
            .address()
            .add(LOOPBACK_INDEX, std::net::IpAddr::V4(addr), PREFIX_LEN)
            .message_mut()
            .clone())
        .execute()
        .await;

    match result {
        Ok(()) => {
            info!(ip, "removed IP from lo");
            Ok(())
        }
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("Cannot assign") || msg.contains("errno 99") {
                warn!(ip, "IP was not present on lo");
                Ok(())
            } else {
                Err(e.into())
            }
        }
    }
}
