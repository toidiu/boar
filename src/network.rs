use crate::error::{BoarError, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    process::{Command, Stdio},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkSetup {
    cmd: String,
    pub delay_ms: u64,
    pub rate_mbit: u64,
    pub loss_model: String,
    pub bdp_bytes: u64, // Bandwidth-Delay Product in bytes
}

impl NetworkSetup {
    pub fn new(cmd: String, delay_ms: u64, rate_mbit: u64, loss_model: String) -> Self {
        let bdp_bytes = delay_ms * rate_mbit * 250;
        NetworkSetup {
            cmd,
            delay_ms,
            rate_mbit,
            loss_model,
            bdp_bytes,
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
        let mut cmd = Command::new("sh");
        let cmd = cmd
            .arg("-c")
            .arg(format!(
                "{} {}ms {}mbit '{}'",
                &self.cmd,
                self.delay_ms,
                self.rate_mbit,
                self.loss_model.clone()
            ))
            .stdout(Stdio::piped());
        let res = cmd.output().unwrap();

        // dbg!(
        //     "Setup network cmd: {:?} {:?}",
        //     cmd,
        //     str::from_utf8(&res.stdout).unwrap()
        // );

        if res.status.success() {
            Ok(())
        } else {
            Err(BoarError::Script("NetworkSetup create".to_string()))
        }
    }
}
