# Anycaster

A lightweight anycast health checker that manages OSPF/BGP IP advertisement by adding and removing IPs on the loopback interface based on service health.

Designed to work with [BIRD](https://bird.network.cz/) or any routing daemon that advertises IPs present on `lo`.

This solves a problem in a small package for my personal homelab.

## How it works

1. BIRD is configured to advertise any IP within a given prefix (e.g. `10.0.0.0/24`) found on the loopback interface.
2. Anycaster runs health checks against local services on a configurable interval.
3. When a service becomes healthy (after `rise` consecutive successes), anycaster adds its IP to `lo` — triggering OSPF/BGP advertisement.
4. When a service becomes unhealthy (after `fall` consecutive failures), anycaster removes the IP — withdrawing the route.

Only state transitions trigger IP changes. On startup, anycaster checks the current `lo` state for clean resume after restart.

## Installation

Download a prebuilt binary from the [releases page](https://github.com/segfault/anycaster/releases), or build from source:

```bash
cargo build --release
```

The binary is statically linked with rustls (no OpenSSL dependency).

## Usage

```bash
anycaster --config /etc/anycaster/config.toml
```

Logging is controlled via the `RUST_LOG` environment variable:

```bash
RUST_LOG=debug anycaster -c config.toml
```

## Configuration

See [`config.example.toml`](config.example.toml) for a complete example.

```toml
[defaults]
check_interval = 5       # seconds between checks
rise = 3                 # consecutive successes before advertising
fall = 2                 # consecutive failures before withdrawing
on_exit = "withdraw"     # "withdraw" or "maintain"
```

Each `[[service]]` defines a service to monitor. Per-service `check_interval`, `rise`, and `fall` override the defaults.

### TCP

Connect to a host:port. Healthy if the connection is established within the timeout.

```toml
[[service]]
name = "cache"
ip = "10.0.0.3"
type = "tcp"
host = "127.0.0.1"
port = 6379
timeout = 3              # seconds (default: 3)
```

### HTTP

Send a GET request and check the response status code.

```toml
[[service]]
name = "web"
ip = "10.0.0.2"
type = "http"
url = "http://127.0.0.1:8080/healthz"
expect_status = 200      # default: 200
timeout = 5              # seconds (default: 3)
```

### DNS

Send DNS queries via [hickory-dns](https://github.com/hickory-dns/hickory-dns) (pure Rust, async, UDP). All queries must return `NOERROR`. If no queries are specified, a default `A` query for `google.com` is used.

```toml
[[service]]
name = "dns"
ip = "10.0.0.1"
type = "dns"
host = "127.0.0.1"
port = 53                # default: 53
timeout = 3              # seconds (default: 3)
queries = [
    { name = "google.com", record_type = "A" },
    { name = "example.com", record_type = "AAAA" },
]
```

Supported record types: `A`, `AAAA`, `MX`, `TXT`, `CNAME`, `NS`, `SOA`, `SRV`, `PTR`.

### NTP

Send an NTPv4 query and verify the server's stratum is within the acceptable range.

```toml
[[service]]
name = "ntp"
ip = "10.0.0.5"
type = "ntp"
host = "127.0.0.1"
port = 123               # default: 123
max_stratum = 1          # default: 1
timeout = 3              # seconds (default: 3)
```

Stratum 0 (kiss-o'-death) always fails. Set `max_stratum = 2` to accept stratum 1 and 2 servers.

### Exec

Run an arbitrary command. Healthy if the exit code is 0.

```toml
[[service]]
name = "custom"
ip = "10.0.0.4"
type = "exec"
command = "/usr/local/bin/check_myservice.sh"
timeout = 3              # seconds (default: 3)
```

## Exit behavior

On shutdown (SIGINT/Ctrl-C), anycaster either withdraws all managed IPs or leaves them in place, controlled by the `on_exit` setting:

- `withdraw` (default) — remove all IPs from `lo`, withdrawing BGP routes
- `maintain` — leave IPs in place, keeping routes advertised until the next health check cycle (e.g. via a process supervisor restart)

## License

[MIT](LICENSE)
