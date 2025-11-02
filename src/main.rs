use crate::error::BoarError;
use crate::error::Result;
use std::process::Command;
use std::process::Stdio;

mod error;

fn main() -> Result<()> {
    println!("Hello, world!");

    // Cli
    let plan = parse_user_input();

    // Network
    delete_network()?;
    setup_network()?;

    // Run
    run_server(&plan);
    run_client(&plan);

    // Data
    collect_stats();
    analyze_stats();

    // Report
    gen_report();

    Ok(())
}

struct ExecutionPlan {
    server_binary: String,
    client_binary: String,
    server_ip: String,
    stream_bytes_bytes: u64,
}

fn parse_user_input() -> ExecutionPlan {
    ExecutionPlan {
        server_binary: "/Users/akothari/projects/quiche/target/debug/examples/async_http3_server"
            .to_string(),
        client_binary: "/Users/akothari/projects/quiche/target/debug/quiche-client".to_string(),
        server_ip: "127.0.0.1:9999".to_string(),
        stream_bytes_bytes: 1000,
    }
}

fn run_server(plan: &ExecutionPlan) {
    let server = &plan.server_binary;
    let server = format!("{:?} --address 0.0.0.0:9999", server);

    let mut binding = Command::new("sh");
    let cmd = binding.arg("-c").arg(server).stdout(Stdio::piped());
    // println!("{:?}", cmd);

    // cmd.status().unwrap();
    cmd.spawn().unwrap();
}

fn run_client(plan: &ExecutionPlan) {
    let client = &plan.client_binary;
    let client = format!(
        "{:?} https://test.com/stream-bytes/{} --no-verify --connect-to  {}",
        client, plan.stream_bytes_bytes, plan.server_ip
    );

    let mut binding = Command::new("sh");
    let cmd = binding.arg("-c").arg(client);
    println!("client cmd ---: {:?}", cmd);

    cmd.status().unwrap();
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

    println!("{:?}", str::from_utf8(&res.stdout).unwrap());

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

    println!("{:?}", str::from_utf8(&res.stdout).unwrap());

    if res.status.success() {
        Ok(())
    } else {
        Err(BoarError::Script)
    }
}
