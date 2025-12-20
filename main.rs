use crate::{
    error::{BoarError, Result},
    stats::{
        Stats, delivery_rate::DeliveryRate, download_duration::DownloadDuration,
        startup_exit::StartupExit,
    },
};
use byte_unit::Byte;
use std::{
    fmt::Debug,
    process::{Child, Command, Stdio},
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

    // println!("Executing: {:#?}", &plan);

    // Network
    plan.network.cleanup()?;
    plan.network.create()?;

    // Run
    let mut server = plan.endpoint.run_server();

    let mut metrics = Vec::new();
    let mut ametrics = Vec::new();
    for i in 1..=plan.run_count {
        let logs = plan.endpoint.run_client(&plan.download_bytes);
        let metric = DownloadDuration::new_from_logs(&logs);
        StartupExit::new_from_logs(&logs);
        let ametric = DeliveryRate::new_from_logs(&logs);
        println!(
            "Run [{}/{}]: Download duration: {:?}",
            i, plan.run_count, metric
        );
        metrics.push(Box::new(metric));
        ametrics.push(Box::new(ametric));
    }

    let s = Stats::new(metrics.into_iter().map(|ty| ty as _).collect());
    let ast = Stats::new(ametrics.into_iter().map(|ty| ty as _).collect());

    // let o = server.wait_with_output().unwrap();
    //
    // let out = String::from_utf8(o.stdout);
    // let err = String::from_utf8(o.stderr);
    // println!("{:?} {:?}", out, err);
    server.kill().unwrap();

    // Report
    let report = report::Report::new(&plan, vec![s, ast]);

    // println!("{:#?}", report);

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

        cmd.arg(&server).stdout(Stdio::piped());
        cmd.arg(&server).stderr(Stdio::piped());
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
