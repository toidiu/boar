use crate::error::BoarError;
use crate::error::Result;
use regex::Regex;
use std::fmt::Debug;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

mod error;

fn main() -> Result<()> {
    // Cli
    let (setup, plan) = parse_user_input();

    // Network
    setup.delete_network()?;
    setup.setup_network()?;

    // Run
    setup.run_server();

    let mut stats = Vec::new();
    for _ in 0..plan.run_count {
        let stat = setup.run_client();
        stats.push(stat);
    }

    // Data
    // analyze_stats();
    //
    // // Report
    // gen_report();

    Ok(())
}

struct ExecutionPlan {
    run_count: u16,
}

struct RunSetup<S: ToStats> {
    client_binary: String,
    client_logging: String,
    server_binary: String,
    server_ip: String,
    server_port: String,
    stream_bytes_bytes: u64,
    stat: S,
}

fn parse_user_input() -> (RunSetup<DownloadDuration>, ExecutionPlan) {
    let run_setup = RunSetup {
        client_binary: "/Users/akothari/projects/quiche/target/debug/quiche-client".to_string(),
        client_logging: "RUST_LOG=info".to_string(),
        server_binary: "/Users/akothari/projects/quiche/target/debug/examples/async_http3_server"
            .to_string(),
        server_ip: "127.0.0.1".to_string(),
        server_port: "9999".to_string(),
        stream_bytes_bytes: 1000000,
        stat: DownloadDuration::default(),
    };
    let plan = ExecutionPlan { run_count: 5 };
    (run_setup, plan)
}

impl<S: ToStats> RunSetup<S> {
    fn run_server(&self) {
        let server = &self.server_binary;
        let server = format!("{:?} --address 0.0.0.0:{}", server, self.server_port);

        let mut binding = Command::new("sh");
        let cmd = binding.arg("-c").arg(server).stdout(Stdio::piped());
        // dbg!("{:?}", cmd);

        // cmd.status().unwrap();
        cmd.spawn().unwrap();
    }

    fn run_client(&self) -> S::Stat {
        let client = &self.client_binary;
        let client = format!(
            "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{}",
            self.client_logging, client, self.stream_bytes_bytes, self.server_ip, self.server_port
        );

        let mut binding = Command::new("sh");
        let cmd = binding
            .arg("-c")
            .arg(client)
            .stderr(Stdio::piped())
            .stdout(Stdio::null());

        // dbg!("client cmd ---: {:?}", &cmd);

        let res = cmd.output().unwrap();
        let log = str::from_utf8(&res.stderr).unwrap();
        // dbg!("Full logs: {:?}", log);

        let download_duration = self.stat.parse_stat(log);
        println!("{:?}", download_duration);
        download_duration
    }

    // fn collect_stats() {}
    //
    // fn analyze_stats() {}
    //
    // fn gen_report() {}

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
}

pub trait ToStats {
    type Stat: Debug;

    fn parse_stat(&self, log: &str) -> Self::Stat;
}

#[derive(Default)]
struct DownloadDuration;

impl ToStats for DownloadDuration {
    type Stat = Duration;

    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    fn parse_stat(&self, log: &str) -> Self::Stat {
        // Regex to get "received in 12.34ms"
        let re = Regex::new(r"received in \d.*ms").unwrap();
        let log = re.captures(log).unwrap().get(0).unwrap().as_str();

        // trim text and parse download duration
        let download_duraiton = log
            .trim_start_matches("received in")
            .trim_end_matches("ms")
            .trim();
        // dbg!("trimmed log: {:?}", log);
        let download_duraiton = download_duraiton.parse::<f32>().unwrap();
        Duration::from_millis(download_duraiton as u64)
    }
}
