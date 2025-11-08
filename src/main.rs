use crate::error::BoarError;
use crate::error::Result;
use crate::stats::ToStats;
use byte_unit::Byte;
use regex::Regex;
use std::fmt::Debug;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

mod error;
mod stats;

fn main() -> Result<()> {
    // Cli
    let (setup, plan) = parse_user_input();
    // dbg!(&setup, &plan);

    for network_setup in plan.network_setups {
        // Network
        network_setup.delete_network()?;
        network_setup.setup_network()?;

        // Run
        let mut server = setup.run_server();

        let mut metrics = Vec::new();
        for _ in 0..plan.run_count {
            let log = setup.run_client();
            let metric = setup.metric.parse_metric(&log);
            println!("Download duration: {:?}", metric);
            metrics.push(metric.as_secs_f64());
        }

        server.kill().unwrap();

        // Report
        network_setup.gen_report(metrics);
    }

    Ok(())
}

#[derive(Debug)]
struct ExecutionPlan {
    network_setups: Vec<NetworkSetup>,
    run_count: u16,
}

#[derive(Debug)]
struct NetworkSetup {
    cmd: String,
}

impl NetworkSetup {
    fn new(cmd: String) -> Self {
        NetworkSetup { cmd }
    }

    fn gen_report(&self, metrics: Vec<f64>) {
        let data = stats::gen_cdf(&metrics);

        stats::plot_cdf(data);
    }

    fn delete_network(&self) -> Result<()> {
        let res = Command::new("sh")
            .arg("-c")
            .arg("./scripts/test.sh")
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        // dbg!("{:?}", str::from_utf8(&res.stdout).unwrap());

        if res.status.success() {
            Ok(())
        } else {
            Err(BoarError::Script)
        }
    }

    fn setup_network(&self) -> Result<()> {
        let res = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        println!(
            "Setup network cmd: {:?}",
            str::from_utf8(&res.stdout).unwrap()
        );

        if res.status.success() {
            Ok(())
        } else {
            Err(BoarError::Script)
        }
    }
}

impl ExecutionPlan {}

#[derive(Debug)]
struct RunSetup<S: ToStats> {
    client_binary: String,
    client_logging: String,
    server_binary: String,
    server_ip: String,
    server_port: String,
    download_payload_size: String,
    metric: S,
}

fn parse_user_input() -> (RunSetup<DownloadDuration>, ExecutionPlan) {
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
        download_payload_size: "1mb".to_string(),
        metric: DownloadDuration::default(),
    };

    let plan = ExecutionPlan {
        network_setups: vec![NetworkSetup::new(network_setup)],
        run_count: 5,
    };
    (run_setup, plan)
}

impl<S: ToStats> RunSetup<S> {
    fn run_server(&self) -> Child {
        let server = &self.server_binary;
        let server = format!("{:?} --address 0.0.0.0:{}", server, self.server_port);

        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let mut cmd = Command::new("ip");
                let cmd = cmd.args(["netns", "exec", "ns_s1"]);

                let cmd = cmd.args(["sh", "-c"]);
            } else {
                let mut cmd = Command::new("sh");
                let cmd = cmd.arg("-c");
            }
        }

        cmd.arg(server).stdout(Stdio::piped());
        // dbg!("{:?}", &cmd);

        // cmd.status().unwrap();
        let server = cmd.spawn().unwrap();
        server
    }

    fn run_client(&self) -> String {
        let client = &self.client_binary;

        let download_bytes = Byte::parse_str(&self.download_payload_size, true).unwrap();
        let client = format!(
            "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{} --idle-timeout 5",
            self.client_logging, client, download_bytes, self.server_ip, self.server_port
        );

        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let mut cmd = Command::new("ip");
                let cmd = cmd.args(["netns", "exec", "ns_c1"]);

                let cmd = cmd.args(["sh", "-c"]);
            } else {
                let mut cmd = Command::new("sh");
                let cmd = cmd.arg("-c");
            }
        }

        cmd.arg(client).stderr(Stdio::piped());
        // dbg!("client cmd ---: {:?}", &cmd);

        let res = cmd.output().unwrap();
        String::from_utf8(res.stderr).unwrap()
    }
}

#[derive(Default, Debug)]
struct DownloadDuration;

impl ToStats for DownloadDuration {
    type Metric = Duration;

    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    fn parse_metric(&self, log: &str) -> Self::Metric {
        // Regex to get "received in 12.34ms"
        //
        // match float: https://stackoverflow.com/a/12643073
        // [+-]?([0-9]*[.])?[0-9]+
        //
        // match "ms" or "s":
        // [m]?s
        let re = Regex::new(r"received in [+-]?([0-9]*[.])?[0-9]+[m]?s").unwrap();
        let log = re.captures(log).unwrap().get(0).unwrap().as_str();

        // trim text and parse download duration
        let download_duraiton = log
            .trim_start_matches("received in ")
            .trim_end_matches("ms")
            .trim_end_matches("s")
            .trim();
        // dbg!("trimmed log: {} {}", log, download_duraiton);

        let download_duration = {
            let duration = download_duraiton.parse::<f32>().unwrap();

            if log.ends_with("ms") {
                duration
            } else if log.ends_with("s") {
                duration * 1000.0
            } else {
                unreachable!("Expect ms or s. Instead got: {}", log)
            }
        };

        Duration::from_millis(download_duration as u64)
    }
}
