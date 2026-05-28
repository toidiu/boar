use crate::mode::Mode;
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EndpointSetup {
    #[serde(default)]
    pub mode: Mode,
    pub client_binary: String,
    pub client_logging: String,
    pub server_binary: String,
    pub server_ip: String,
    pub server_port: String,
    pub server_cca: String,
}

impl EndpointSetup {
    /// In `Host` mode boar spawns the server in `ns_s1` and tails its stderr so
    /// the `StartupExit` metric can be built. In `Docker` mode the server is a
    /// peer container owned by the runtime — boar returns `None` and skips the
    /// startup-exit capture.
    pub fn run_server(&self) -> Option<(Child, Arc<Mutex<Vec<String>>>)> {
        if self.mode == Mode::Docker {
            return None;
        }

        let server = &self.server_binary;
        let server = format!(
            "{} {:?} --address 0.0.0.0:{}  --cc-algorithm {}",
            self.client_logging, server, self.server_port, self.server_cca
        );

        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let mut cmd = Command::new("ip");
                let cmd = cmd.args(["netns", "exec", "ns_s1"]);

                let cmd = cmd.args(["sh", "-c"]);
            } else {
                let mut cmd = Command::new("sh");
                let cmd = cmd.arg("-c");
            }
        }

        cmd.arg(&server).stdout(Stdio::piped());
        cmd.arg(&server).stderr(Stdio::piped());
        // dbg!("{:?}", &cmd);

        // cmd.status().unwrap();
        let mut server = cmd.spawn().unwrap();

        let stdout = server.stderr.take().unwrap();
        let server_logs = Arc::new(Mutex::new(Vec::with_capacity(100)));
        let server_logs_clone = server_logs.clone();
        let _log = thread::spawn(move || {
            // let server_logs = server_logs.clone();
            // thread::sleep(Duration::from_millis(1));
            let reader = BufReader::new(stdout);
            reader
                .lines()
                .filter_map(|line| line.ok())
                .for_each(|line| {
                    let mut server_logs = server_logs_clone.lock().unwrap();
                    server_logs.push(line);
                });
        });

        Some((server, server_logs))
    }

    pub fn run_client(&self, download_bytes: &Byte) -> String {
        let client = &self.client_binary;

        // let download_bytes = Byte::parse_str(plan.download_payload_size, true).unwrap();
        let client = format!(
            "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{} --idle-timeout 5",
            self.client_logging, client, download_bytes, self.server_ip, self.server_port
        );

        // Host mode runs the client inside ns_c1 so it talks to the server
        // through the netns chain. Docker mode runs it on the container's own
        // network — the docker bridge already routes us to the peer container.
        let use_netns = cfg!(target_os = "linux") && self.mode == Mode::Host;

        let mut cmd;
        if use_netns {
            cmd = Command::new("ip");
            cmd.args(["netns", "exec", "ns_c1", "sh", "-c"]);
        } else {
            cmd = Command::new("sh");
            cmd.arg("-c");
        }

        cmd.arg(client).stderr(Stdio::piped());
        // dbg!("client cmd ---: {:?}", &cmd);

        let res = cmd.output().unwrap();

        String::from_utf8(res.stderr).unwrap()
    }
}
