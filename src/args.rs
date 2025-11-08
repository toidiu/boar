use crate::DownloadDuration;
use crate::ExecutionPlan;
use crate::NetworkSetup;
use crate::RunSetup;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, default_value = "1mb")]
    download_size: String,

    #[arg(short, default_value_t = 5)]
    run_count: u16,
}

pub(crate) fn parse() -> (RunSetup<DownloadDuration>, ExecutionPlan) {
    let args = Args::parse();

    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            let network_setup = "./scripts/virt_config_tc.sh".to_string();
            let server_ip="10.55.10.1".to_string();
        } else {
            let network_setup = "./scripts/test.sh".to_string();
            let server_ip = "127.0.0.1".to_string();
        }
    }

    let run_setup = RunSetup {
        // Client
        // cargo build --bin quiche-client
        client_binary: "../quiche/target/debug/quiche-client".to_string(),
        client_logging: "RUST_LOG=info".to_string(),

        // Server
        // cargo build --example async_http3_server
        server_binary: "../quiche/target/debug/examples/async_http3_server".to_string(),
        server_ip,
        server_port: "9999".to_string(),

        // Testing
        download_payload_size: args.download_size,
        metric: DownloadDuration::default(),
        run_count: args.run_count,
    };

    let plan = ExecutionPlan {
        network_setups: vec![
            NetworkSetup::new(network_setup.clone()),
            NetworkSetup::new(network_setup),
        ],
    };
    (run_setup, plan)
}
