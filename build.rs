use std::env;
use std::process::Command;

const DEPS_QUICHE: &str = "deps/quiche";

fn main() {
    // Change the current working directory
    env::set_current_dir(&DEPS_QUICHE).expect("Failed to change directory");

    // cargo build --bin quiche-client
    let s = Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("quiche-client")
        .status()
        .unwrap();
    assert!(s.success());

    // cargo build --example async_http3_server
    let s = Command::new("cargo")
        .arg("build")
        .arg("--example")
        .arg("async_http3_server")
        .status()
        .unwrap();
    assert!(s.success());

    println!("cargo:rerun-if-changed=deps/quiche/quiche");
    println!("cargo:rerun-if-changed=deps/quiche/tokio-quiche");
    println!("cargo:rerun-if-changed=deps/quiche/apps");
}
