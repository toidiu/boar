use crate::error::BoarError;
use crate::error::Result;
use regex::Regex;
use std::process::Command;
use std::process::Stdio;

mod error;

fn main() -> Result<()> {
    // Cli
    let setup = parse_user_input();

    // Network
    delete_network()?;
    setup_network()?;

    // Run
    run_server(&setup);
    run_client(&setup);

    // Data
    collect_stats();
    analyze_stats();

    // Report
    gen_report();

    Ok(())
}

struct RunSetup {
    client_binary: String,
    client_logging: String,
    server_binary: String,
    server_ip: String,
    server_port: String,
    stream_bytes_bytes: u64,
}

fn parse_user_input() -> RunSetup {
    RunSetup {
        client_binary: "/Users/akothari/projects/quiche/target/debug/quiche-client".to_string(),
        client_logging: "RUST_LOG=info".to_string(),
        server_binary: "/Users/akothari/projects/quiche/target/debug/examples/async_http3_server"
            .to_string(),
        server_ip: "127.0.0.1".to_string(),
        server_port: "9999".to_string(),
        stream_bytes_bytes: 1000,
    }
}

fn run_server(setup: &RunSetup) {
    let server = &setup.server_binary;
    let server = format!("{:?} --address 0.0.0.0:{}", server, setup.server_port);

    let mut binding = Command::new("sh");
    let cmd = binding.arg("-c").arg(server).stdout(Stdio::piped());
    // dbg!("{:?}", cmd);

    // cmd.status().unwrap();
    cmd.spawn().unwrap();
}

fn run_client(setup: &RunSetup) {
    let client = &setup.client_binary;
    let client = format!(
        "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{}",
        setup.client_logging, client, setup.stream_bytes_bytes, setup.server_ip, setup.server_port
    );

    let mut binding = Command::new("sh");
    let cmd = binding
        .arg("-c")
        .arg(client)
        .stderr(Stdio::piped())
        .stdout(Stdio::null());

    dbg!("client cmd ---: {:?}", &cmd);

    let res = cmd.output().unwrap();
    let log = str::from_utf8(&res.stderr).unwrap();
    // dbg!("Full logs: {:?}", log);

    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    //
    // Regex to get "received in 12.34ms"
    let re = Regex::new(r"received in \d.*ms").unwrap();
    let log = re.captures(log).unwrap().get(0).unwrap().as_str();

    // trim text and parse download duration
    let download_duraiton = log
        .trim_start_matches("received in")
        .trim_end_matches("ms")
        .trim();
    // dbg!("trimmed log: {:?}", log);
    let download_duration = download_duraiton.parse::<f32>().unwrap();

    println!("{:?}", download_duration);
}

fn collect_stats() {}

fn analyze_stats() {}

fn gen_report() {}

fn delete_network() -> Result<()> {
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

fn setup_network() -> Result<()> {
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
