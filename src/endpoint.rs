use byte_unit::Byte;
use std::{
    fmt::Debug,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::{Arc, Mutex},
    thread,
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
    pub fn run_server(&self) -> (Child, Arc<Mutex<Vec<String>>>) {
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

        (server, server_logs)
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
