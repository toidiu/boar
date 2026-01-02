use crate::error::{BoarError, Result};
use std::{
    fmt::Debug,
    process::{Command, Stdio},
};

#[derive(Debug, Clone)]
pub(crate) struct NetworkSetup {
    cmd: String,
    #[allow(dead_code)]
    delay_ms: u64,
    #[allow(dead_code)]
    loss_pct: u64,
    #[allow(dead_code)]
    rate_mbit: u64,
}

impl NetworkSetup {
    pub fn new(cmd: String, delay_ms: u64) -> Self {
        NetworkSetup {
            cmd,
            // Default values in script
            delay_ms,
            // Default values in script
            loss_pct: 0,
            // Default values in script
            rate_mbit: 20,
        }
    }

    pub fn cleanup(&self) -> Result<()> {
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

    pub fn create(&self) -> Result<()> {
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
