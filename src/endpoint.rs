use byte_unit::Byte;
use std::{
    fmt::Debug,
    process::{Child, Command, Stdio},
};

#[derive(Debug, Clone)]
pub struct EndpointSetup {
    pub client_binary: String,
    pub client_logging: String,
    pub server_binary: String,
    pub server_ip: String,
    pub server_port: String,
    pub server_cca: String,
}

impl EndpointSetup {
    pub fn run_server(&self) -> Child {
        let server = &self.server_binary;
        let server = format!(
            "{:?} --address 0.0.0.0:{}  --cc-algorithm {}",
            server, self.server_port, self.server_cca
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

        cmd.arg(server).stdout(Stdio::piped());
        // dbg!("{:?}", &cmd);

        // cmd.status().unwrap();
        let server = cmd.spawn().unwrap();
        server
    }

    pub fn run_client(&self, download_bytes: &Byte) -> String {
        let client = &self.client_binary;

        // let download_bytes = Byte::parse_str(plan.download_payload_size, true).unwrap();
        let client = format!(
            "{} {} https://test.com/stream-bytes/{} --no-verify --connect-to  {}:{} --idle-timeout 5",
            self.client_logging, client, download_bytes, self.server_ip, self.server_port
        );

        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                let mut cmd = Command::new("ip");
                let cmd = cmd.args(["netns", "exec", "ns_c1"]);

                let cmd = cmd.args(["sh", "-c"]);
            } else {
                let mut cmd = Command::new("sh");
                let cmd = cmd.arg("-c");
            }
        }

        cmd.arg(client).stderr(Stdio::piped());
        // dbg!("client cmd ---: {:?}", &cmd);

        let res = cmd.output().unwrap();
        let logs = String::from_utf8(res.stderr).unwrap();

        logs
    }
}
