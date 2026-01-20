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
    /// Bandwidth-Delay Product in bytes
    pub bdp_bytes: u64,
}

impl NetworkSetup {
    /// Calculate Bandwidth-Delay Product (BDP) in bytes.
    ///
    /// BDP = RTT × Bandwidth (converted to bytes)
    ///
    /// Formula derivation:
    /// - RTT = 2 × delay_ms (round-trip = 2 × one-way delay)
    /// - Bandwidth = rate_mbit × 1,000,000 bits/sec
    /// - BDP (bits) = (delay_ms / 1000 × 2) × (rate_mbit × 1,000,000)
    /// - BDP (bytes) = BDP (bits) / 8
    ///              = delay_ms × rate_mbit × 250
    fn bdp(delay_ms: u64, rate_mbit: u64) -> u64 {
        delay_ms * rate_mbit * 250
    }

    pub fn new(cmd: String, delay_ms: u64, rate_mbit: u64, loss_model: String) -> Self {
        NetworkSetup {
            cmd,
            delay_ms,
            rate_mbit,
            loss_model,
            bdp_bytes: Self::bdp(delay_ms, rate_mbit),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bdp_calculation() {
        // delay=50ms (one-way), RTT=100ms, rate=5mbit
        // BDP = 0.1s × 5,000,000 bits/s = 500,000 bits = 62,500 bytes
        assert_eq!(NetworkSetup::bdp(50, 5), 62_500);
    }
}
