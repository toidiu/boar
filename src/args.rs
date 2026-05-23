use boar::{EndpointSetup, ExecutionPlan, Mode, NetworkSetup};
use byte_unit::Byte;
use clap::Parser;
use std::net::ToSocketAddrs;
use uuid::Uuid;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, default_value = "1mb")]
    download_size: String,

    #[arg(short, default_value_t = 2)]
    count: u16,

    /// Where boar is running. `host` keeps the existing ns_s1..ns_c2 netns
    /// topology (scripts/virt_*.sh). `docker` expects to be inside a
    /// container talking to a peer `boar-server` over a docker bridge, with
    /// shaping applied to its own eth0.
    #[arg(long, value_enum, default_value_t = Mode::Host)]
    pub mode: Mode,

    /// Server hostname used in docker mode (resolved via docker DNS).
    /// Ignored in host mode.
    #[arg(long, default_value_t = default::default_server_host())]
    pub server_host: String,

    // --------
    // Server
    // --------
    /// Congestion Control algorithm
    #[arg(long,  default_value_t = default::default_cc_algorithm())]
    pub cc_algorithm: String,

    // --------
    // Network
    // --------
    #[arg(long,  default_value_t = default::default_delay_ms())]
    pub delay_ms: u64,

    #[arg(long,  default_value_t = default::default_rate_mbit())]
    pub rate_mbit: u64,

    /// https://man7.org/linux/man-pages/man8/tc-netem.8.html
    ///
    /// ”gemodel 0.1% 90%"
    ///  0.1% : probability of starting bad (lossy) state.
    ///  90%  : probability of exiting bad state.
    #[arg(long,  default_value_t = default::default_loss_model())]
    pub loss_model: String,
}

pub(crate) fn parse() -> ExecutionPlan {
    let args = Args::parse();

    let (net_sim_cmd, net_cleanup_cmd, server_ip) = match args.mode {
        Mode::Docker => (
            "./scripts/docker_tc.sh".to_string(),
            "./scripts/docker_tc_cleanup.sh".to_string(),
            // quiche-client's --connect-to does a bare SocketAddr::parse, so
            // hostnames panic before any traffic flows. Resolve via the
            // OS resolver (docker's embedded DNS) up front and pass the IP.
            resolve_host(&args.server_host),
        ),
        Mode::Host => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "linux")] {
                    let server_ip = "10.55.10.1".to_string();
                    let net_sim_cmd = "./scripts/virt_config_tc.sh".to_string();
                } else {
                    let server_ip = "127.0.0.1".to_string();
                    let net_sim_cmd = "./scripts/test.sh".to_string();
                }
            }
            // Host-mode cleanup stays the legacy no-op; virt_config_tc.sh
            // tears down its own prior state at the top.
            (net_sim_cmd, "./scripts/test.sh".to_string(), server_ip)
        }
    };

    let network_setup = NetworkSetup::new(
        net_sim_cmd,
        net_cleanup_cmd,
        args.delay_ms,
        args.rate_mbit,
        args.loss_model,
    );

    let endpoint_setup = EndpointSetup {
        mode: args.mode,
        // Client
        // cargo build --bin quiche-client
        client_binary: "deps/quiche/target/debug/quiche-client".to_string(),
        client_logging: "RUST_LOG=info".to_string(),

        // Server
        // cargo build --example async_http3_server
        server_binary: "deps/quiche/target/debug/examples/async_http3_server".to_string(),
        server_ip,
        server_port: "9999".to_string(),
        server_cca: args.cc_algorithm,
    };

    let download_bytes = Byte::parse_str(args.download_size, true).unwrap();

    ExecutionPlan {
        uuid: Uuid::new_v4(),
        network_setup,
        endpoint_setup,

        download_bytes,
        count: args.count,
    }
}

/// Block until the OS resolver returns at least one address for `host`. Gives
/// docker's embedded DNS a few seconds of grace if boar starts before the
/// server's network attach completes.
fn resolve_host(host: &str) -> String {
    let target = format!("{host}:0");
    for _ in 0..20 {
        if let Ok(mut addrs) = target.to_socket_addrs() {
            if let Some(addr) = addrs.next() {
                return addr.ip().to_string();
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    panic!("Could not resolve '{host}' after 10s — is boar-server up on the same docker network?");
}

mod default {
    pub fn default_cc_algorithm() -> String {
        "bbr2_gcongestion".to_string()
    }

    pub fn default_server_host() -> String {
        "boar-server".to_string()
    }

    pub fn default_delay_ms() -> u64 {
        50
    }

    pub fn default_rate_mbit() -> u64 {
        20
    }

    pub fn default_loss_model() -> String {
        "random 0%".to_string()
    }
}
