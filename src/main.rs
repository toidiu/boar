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
    server_location: String,
    client_location: String,
}

fn parse_user_input() -> ExecutionPlan {
    ExecutionPlan {
        server_location: "/Users/akothari/projects/quiche/target/debug/examples/async_http3_server"
            .to_string(),
        client_location: "/Users/akothari/projects/quiche/target/debug/examples/quiche-client"
            .to_string(),
    }
}

fn delete_network() -> Result<()> {
    let res = Command::new("sh")
        .arg("-c")
        .arg("./scripts/test.sh")
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    println!("{:?}", str::from_utf8(&res.stdout));

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

    println!("{:?}", str::from_utf8(&res.stdout));

    if res.status.success() {
        Ok(())
    } else {
        Err(BoarError::Script)
    }
}

fn run_server(plan: &ExecutionPlan) {
    let server = &plan.server_location;
    println!("{:?}", server);

    let res = Command::new("sh")
        .arg("-c")
        .arg(server)
        .arg("--address")
        .arg("0.0.0.0:8888")
        .status()
        // .stdout(Stdio::piped())
        // .output()
        .unwrap();

    // println!("{:?}", str::from_utf8(&res.stdout));
}

fn run_client(plan: &ExecutionPlan) {}

fn collect_stats() {}

fn analyze_stats() {}

fn gen_report() {}
