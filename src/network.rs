use crate::error::{BoarError, Result};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    process::{Command, Stdio},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NetworkSetup {
    cmd: String,
    /// Script run by `cleanup()`. Host mode points this at `scripts/test.sh`
    /// (a no-op — the real teardown lives inline at the top of
    /// virt_config_tc.sh); docker mode points it at docker_tc_cleanup.sh,
    /// which actually drops the qdiscs from eth0/ifb0.
    #[serde(default = "default_cleanup_cmd")]
    cleanup_cmd: String,
    pub delay_ms: u64,
    pub rate_mbit: u64,
    pub loss_model: String,
    /// Bandwidth-Delay Product in bytes
    pub bdp_bytes: u64,
}

fn default_cleanup_cmd() -> String {
    "./scripts/test.sh".to_string()
}

impl NetworkSetup {
    pub fn new(
        cmd: String,
        cleanup_cmd: String,
        delay_ms: u64,
        rate_mbit: u64,
        loss_model: String,
    ) -> Self {
        NetworkSetup {
            cmd,
            cleanup_cmd,
            delay_ms,
            rate_mbit,
            loss_model,
            bdp_bytes: Self::bdp(delay_ms, rate_mbit),
        }
    }

    pub fn cleanup(&self) -> Result<()> {
        let res = Command::new("sh")
            .arg("-c")
            .arg(&self.cleanup_cmd)
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

    // TODO: I think this under-estimates the value. We are not accounting for any buffers in the
    // setup. Also we do expect BBR to operate over BDP so we need a way to communicate what the
    // acceptable value is.
    //
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
