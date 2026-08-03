use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(rename = "service")]
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_check_interval")]
    pub check_interval: u64,
    #[serde(default = "default_rise")]
    pub rise: u32,
    #[serde(default = "default_fall")]
    pub fall: u32,
    #[serde(default)]
    pub on_exit: OnExit,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            check_interval: default_check_interval(),
            rise: default_rise(),
            fall: default_fall(),
            on_exit: OnExit::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OnExit {
    #[default]
    Withdraw,
    Maintain,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub name: String,
    pub ip: String,
    #[serde(flatten)]
    pub check: Check,
    pub check_interval: Option<u64>,
    pub rise: Option<u32>,
    pub fall: Option<u32>,
}

impl Service {
    pub fn check_interval(&self, defaults: &Defaults) -> u64 {
        self.check_interval.unwrap_or(defaults.check_interval)
    }

    pub fn rise(&self, defaults: &Defaults) -> u32 {
        self.rise.unwrap_or(defaults.rise)
    }

    pub fn fall(&self, defaults: &Defaults) -> u32 {
        self.fall.unwrap_or(defaults.fall)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Check {
    Tcp {
        host: String,
        port: u16,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
    Http {
        url: String,
        #[serde(default = "default_expect_status")]
        expect_status: u16,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
    Dns {
        host: String,
        #[serde(default = "default_dns_port")]
        port: u16,
        #[serde(default)]
        queries: Vec<DnsQuery>,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
    Ntp {
        host: String,
        #[serde(default = "default_ntp_port")]
        port: u16,
        #[serde(default = "default_max_stratum")]
        max_stratum: u8,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
    Exec {
        command: String,
        #[serde(default = "default_timeout")]
        timeout: u64,
    },
}

#[derive(Debug, Deserialize)]
pub struct DnsQuery {
    pub name: String,
    #[serde(default = "default_record_type")]
    pub record_type: String,
}

fn default_check_interval() -> u64 {
    5
}

fn default_rise() -> u32 {
    3
}

fn default_fall() -> u32 {
    2
}

fn default_timeout() -> u64 {
    3
}

fn default_expect_status() -> u16 {
    200
}

fn default_dns_port() -> u16 {
    53
}

fn default_ntp_port() -> u16 {
    123
}

fn default_max_stratum() -> u8 {
    1
}

fn default_record_type() -> String {
    "A".to_string()
}

pub fn load(path: &Path) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
