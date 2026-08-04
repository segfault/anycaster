#[cfg(target_os = "linux")]
mod netlink;
#[cfg(target_os = "linux")]
pub use netlink::*;

#[cfg(not(target_os = "linux"))]
mod command;
#[cfg(not(target_os = "linux"))]
pub use command::*;
