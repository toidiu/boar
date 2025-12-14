use crate::{
    error::{BoarError, Result},
    stats::{AggregateStats, Stats, ToStatMetric},
};
use byte_unit::Byte;
use regex::Regex;
use std::{
    fmt::Debug,
    process::{Child, Command, Stdio},
    time::Duration,
};
use uuid::Uuid;

mod args;
mod error;
mod report;
mod stats;

fn main() -> Result<()> {
    // Cli
    let plan = args::parse();
    // dbg!(&setup, &plan);

    println!("Executing: {:#?}", &plan);

    // Network
    plan.network.cleanup()?;
    plan.network.create()?;

    // Run
    let mut server = plan.endpoint.run_server();

    let mut metrics: Vec<Box<dyn ToStatMetric>> = Vec::new();
    for i in 1..=plan.run_count {
        let logs = plan.endpoint.run_client(&plan.download_bytes);
        let metric = DownloadDurationMetric::new_from_logs(&logs);
        println!(
            "Run [{}/{}]: Download duration: {:?}",
            i, plan.run_count, metric
        );
        metrics.push(Box::new(metric));
    }
    let s = Stats::new(metrics);

    server.kill().unwrap();

    // Report
    let report = report::Report::new(&plan, s);

    println!("{:#?}", report);

    Ok(())
}

#[derive(Debug, Clone)]
struct ExecutionPlan {
    uuid: Uuid,
    network: NetworkSetup,
    endpoint: EndpointSetup,
    download_bytes: Byte,
    run_count: u16,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct NetworkSetup {
    cmd: String,
    delay_ms: u64,
    loss_pct: u64,
    rate_mbit: u64,
}

impl NetworkSetup {
    fn new(cmd: String) -> Self {
        NetworkSetup {
            cmd,
            // Default values in script
            delay_ms: 50,
            // Default values in script
            loss_pct: 0,
            // Default values in script
            rate_mbit: 20,
        }
    }

    fn cleanup(&self) -> Result<()> {
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
            Err(BoarError::Script("NetworkSetup cleanup".to_string()))
        }
    }

    fn create(&self) -> Result<()> {
        let res = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        // dbg!(
        //     "Setup network cmd: {:?}",
        //     str::from_utf8(&res.stdout).unwrap()
        // );

        if res.status.success() {
            Ok(())
        } else {
            Err(BoarError::Script("NetworkSetup create".to_string()))
        }
    }
}

#[derive(Debug, Clone)]
struct EndpointSetup {
    client_binary: String,
    client_logging: String,
    server_binary: String,
    server_ip: String,
    server_port: String,
    server_cca: String,
}

impl EndpointSetup {
    fn run_server(&self) -> Child {
        let server = &self.server_binary;
        let server = format!(
            "{:?} --address 0.0.0.0:{}  --cc-algorithm {}",
            server, self.server_port, self.server_cca
        );

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

    fn run_client(&self, download_bytes: &Byte) -> String {
        let client = &self.client_binary;

        // let download_bytes = Byte::parse_str(plan.download_payload_size, true).unwrap();
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
        let logs = String::from_utf8(res.stderr).unwrap();

        logs
    }
}

#[derive(Default, Debug)]
struct DownloadDurationMetric {
    duration: Duration,
}

impl DownloadDurationMetric {
    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    fn new_from_logs(logs: &str) -> Self {
        // Regex to get "received in 12.34ms"
        //
        // match float: https://stackoverflow.com/a/12643073
        // [+-]?([0-9]*[.])?[0-9]+
        //
        // match "ms" or "s":
        // [m]?s
        let re = Regex::new(r"received in [+-]?([0-9]*[.])?[0-9]+[m]?s").unwrap();
        let logs = re.captures(logs).unwrap().get(0).unwrap().as_str();

        // trim text and parse download duration
        let download_duraiton = logs
            .trim_start_matches("received in ")
            .trim_end_matches("ms")
            .trim_end_matches("s")
            .trim();
        // dbg!("trimmed logs: {} {}", logs, download_duraiton);

        let download_duration = {
            let duration = download_duraiton.parse::<f32>().unwrap();

            if logs.ends_with("ms") {
                duration
            } else if logs.ends_with("s") {
                duration * 1000.0
            } else {
                unreachable!("Expect ms or s. Instead got: {}", logs)
            }
        };

        DownloadDurationMetric {
            duration: Duration::from_millis(download_duration as u64),
        }
    }
}

impl ToStatMetric for DownloadDurationMetric {
    fn as_f64(&self) -> f64 {
        self.duration.as_secs_f64()
    }
}
