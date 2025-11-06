use crate::error::BoarError;
use crate::error::Result;
use byte_unit::Byte;
use regex::Regex;
use std::fmt::Debug;
use std::process::Command;
use std::process::Stdio;
use std::time::Duration;

mod error;

fn main() -> Result<()> {
    // Cli
    let (setup, plan) = parse_user_input();
    // dbg!(&setup, &plan);

    // Network
    setup.delete_network()?;
    setup.setup_network()?;

    // Run
    setup.run_server();

    let mut metrics = Vec::new();
    for _ in 0..plan.run_count {
        let metric = setup.run_client();
        metrics.push(metric.as_secs_f64());
    }

    // Data
    // analyze_metrics();
    let data = gen_cdf(&metrics);

    // Report
    gen_report(data);

    Ok(())
}

#[derive(Debug)]
struct ExecutionPlan {
    run_count: u16,
}

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
    let run_setup = RunSetup {
        // cargo build --bin quiche-client
        client_binary: "../quiche/target/debug/quiche-client".to_string(),
        client_logging: "RUST_LOG=info".to_string(),

        // cargo build --example async_http3_server
        server_binary: "../quiche/target/debug/examples/async_http3_server".to_string(),
        server_ip: "127.0.0.1".to_string(),
        server_port: "9999".to_string(),
        download_payload_size: "1mb".to_string(),
        metric: DownloadDuration::default(),
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

    fn run_client(&self) -> S::Metric {
        let client = &self.client_binary;

        let download_bytes = Byte::parse_str(&self.download_payload_size, true).unwrap();
        let client = format!(
            "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{}",
            self.client_logging, client, download_bytes, self.server_ip, self.server_port
        );

        let mut binding = Command::new("sh");
        let cmd = binding
            .arg("-c")
            .arg(client)
            .stderr(Stdio::piped())
            .stdout(Stdio::null());

        // dbg!("client cmd ---: {:?}", &cmd);n

        let res = cmd.output().unwrap();
        let log = str::from_utf8(&res.stderr).unwrap();
        // dbg!("Full logs: {:?}", log);

        let download_duration = self.metric.parse_metric(log);
        println!("{:?}", download_duration);
        download_duration
    }

    // fn collect_metrics() {}
    //
    // fn analyze_metrics() {}
    //

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
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let script = "./scripts/virt_config_tc.sh";
            } else {
                let script = "./scripts/test.sh";
            }
        }

        let res = Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .output()
            .unwrap();

        dbg!("{:?}", str::from_utf8(&res.stdout).unwrap());

        if res.status.success() {
            Ok(())
        } else {
            Err(BoarError::Script)
        }
    }
}

use plotly::{Scatter, layout::GridPattern, layout::Layout, layout::LayoutGrid};

pub fn gen_cdf(stats: &[f64]) -> Vec<(f64, f64)> {
    // Generate CDF
    let mut x: Vec<f64> = Vec::new();
    x.extend_from_slice(&stats);

    cdf(&x)
}

fn gen_report(data: Vec<(f64, f64)>) {
    plot_cdf(data);
}

fn plot_cdf(data: Vec<(f64, f64)>) {
    let mut plot = plotly::Plot::new();
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().map(|(a, b)| (a, b)).unzip();

    // Graph
    let trace = Scatter::new(x, y)
        // dont show legend for CDF
        .show_legend(false)
        .x_axis("x")
        .y_axis("y");
    plot.add_trace(trace);

    let title = format!("{}", "title");
    let layout = Layout::new()
        .title(format!("{} Cumulative distribution function", title))
        .show_legend(true)
        .height(1000)
        .grid(
            LayoutGrid::new()
                .rows(1)
                .columns(1)
                .pattern(GridPattern::Independent),
        );
    plot.set_layout(layout);
    plot.write_html("plot.html");
    // plot.show();
}

// https://users.rust-lang.org/t/observed-cdf-of-a-vector/77566/4
pub fn cdf(x: &[f64]) -> Vec<(f64, f64)> {
    let ln = x.len() as f64;
    let mut x_ord = x.to_vec();
    x_ord.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if let Some(mut previous) = x_ord.get(0).map(|&f| f) {
        let mut cdf = Vec::new();
        for (i, f) in x_ord.into_iter().enumerate() {
            if f != previous {
                cdf.push((previous, i as f64 / ln));
                previous = f;
            }
        }

        cdf.push((previous, 1.0));
        cdf
    } else {
        Vec::new()
    }
}

pub trait ToStats {
    type Metric: Debug;

    fn parse_metric(&self, log: &str) -> Self::Metric;
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
