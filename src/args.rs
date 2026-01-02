use crate::{EndpointSetup, ExecutionPlan, NetworkSetup};
use byte_unit::Byte;
use clap::Parser;
use uuid::Uuid;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, default_value = "1mb")]
    download_size: String,

    #[arg(short, default_value_t = 2)]
    run_count: u16,

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
}

pub(crate) fn parse() -> ExecutionPlan {
    let args = Args::parse();

    let (network_setup, server_ip) = {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let net_sim_cmd = "./scripts/virt_config_tc.sh".to_string();
                let server_ip="10.55.10.1".to_string();
            } else {
                let net_sim_cmd = "./scripts/test.sh".to_string();
                let server_ip = "127.0.0.1".to_string();
            }
        }

        (NetworkSetup::new(net_sim_cmd, args.delay_ms), server_ip)
    };

    let endpoint_setup = EndpointSetup {
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
        run_count: args.run_count,
    }
}

mod default {
    pub fn default_cc_algorithm() -> String {
        "bbr2_gcongestion".to_string()
    }

    pub fn default_delay_ms() -> u64 {
        50
    }
}
