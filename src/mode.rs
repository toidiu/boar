use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// How boar is being run. Selects the network topology and which process owns
/// the server lifecycle.
#[derive(Copy, Clone, Debug, Default, ValueEnum, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// Run on the host. boar builds the ns_s1..ns_c2 chain via scripts/virt_*.sh,
    /// spawns the server in ns_s1, and runs each client in ns_c1.
    #[default]
    Host,
    /// Run inside docker. The server lives in a peer container (boar-server);
    /// boar shapes its own eth0 via scripts/docker_tc.sh and only runs the
    /// client. Server lifecycle is owned by the docker runtime, not boar.
    Docker,
}
