use crate::error::BoarError;
use crate::error::Result;
use std::process::Command;
use std::process::Stdio;

mod error;

fn main() -> Result<()> {
    println!("Hello, world!");

    parse_user_input();

    setup_network()?;

    run_server();

    run_client();

    collect_stats();

    analyze_stats();

    gen_report();

    Ok(())
}

fn parse_user_input() {}

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

fn run_server() {}

fn run_client() {}

fn collect_stats() {}

fn analyze_stats() {}

fn gen_report() {}
