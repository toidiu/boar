use std::env;
use std::process::Command;

const DEPS_QUICHE: &str = "deps/quiche";

fn main() {
    // Change the current working directory
    env::set_current_dir(&DEPS_QUICHE).expect("Failed to change directory");

    // cargo build --bin quiche-client
    Command::new("cargo")
        .arg("build")
        .arg("--bin")
        .arg("quiche-client")
        .status()
        .unwrap();

    // cargo build --example async_http3_server
    Command::new("cargo")
        .arg("build")
        .arg("--example")
        .arg("async_http3_server")
        .status()
        .unwrap();

    // Tell Cargo to re-run the build script if this file changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=deps/quiche/");
}
