use crate::stats::ToStatMetric;
use regex::Regex;
use std::fmt::Debug;

#[derive(Default, Debug)]
pub struct StartupExit {
    // cwnd: usize,
    bandwidth: usize,
}

impl StartupExit {
    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    pub fn new_from_logs(logs: &[String]) -> Vec<Self> {
        let mut metrics = Vec::new();

        for log in logs {
            // println!("{}", log);
            // Regex to get "bandwidth: Some(74244584)"
            let re = Regex::new(r"bandwidth: Some\([0-9]*\)").unwrap();

            let Some(log) = re.captures(log).map(|log| log.get(0).unwrap().as_str()) else {
                // line did not contain path stats
                continue;
            };

            // trim text and parse delivery_rate
            let bandwidth = log
                .trim_start_matches("bandwidth: Some(")
                .trim_end_matches(")")
                .trim();

            let bandwidth = bandwidth.parse::<usize>().unwrap();

            let metric = StartupExit { bandwidth };
            metrics.push(metric);
        }

        metrics
    }
}

impl ToStatMetric for StartupExit {
    fn as_f64(&self) -> f64 {
        self.bandwidth as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_exit_none() {
        let logs: Vec<String> = [
            "[2025-12-20T07:09:59.547130000Z INFO  async_http3_server] received new connection!",
            "[2025-12-20T07:09:59.551243000Z INFO  async_http3_server::server] received unhandled event: IncomingSettings { settings: [(553772674384608777, 2680223962524332553)] }",
            "[2025-12-20T07:09:59.551419000Z INFO  async_http3_server::server] received unhandled event: BodyBytesReceived { stream_id: 0, num_bytes: 0, fin: true }",
            "[2025-12-20T07:09:59.565448000Z INFO  async_http3_server::server] stream closed: Some(local_addr=0.0.0.0:9999 peer_addr=127.0.0.1:61757 validation_state=Validated active=true recv=274 sent=794 lost=0 retrans=0 rtt=251.283µs min_rtt=Some(100.291µs) rttvar=165.768µs cwnd=101824 sent_bytes=1038667 recv_bytes=14030 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=110391822 max_bandwidth=Some(110391822) startup_exit=None )",
        ].into_iter().map(|v| v.to_string()).collect();

        let metric = StartupExit::new_from_logs(&logs);
        assert!(metric.is_empty());
    }

    #[test]
    fn startup_exit_some() {
        let logs: Vec<String> = [
            "[2025-12-20T07:09:59.547130000Z INFO  async_http3_server] received new connection!",
            "[2025-12-20T07:09:59.551243000Z INFO  async_http3_server::server] received unhandled event: IncomingSettings { settings: [(553772674384608777, 2680223962524332553)] }",
            "[2025-12-20T07:09:59.551419000Z INFO  async_http3_server::server] received unhandled event: BodyBytesReceived { stream_id: 0, num_bytes: 0, fin: true }",
            "[2025-12-20T07:09:59.565448000Z INFO  async_http3_server::server] stream closed: Some(local_addr=0.0.0.0:9999 peer_addr=127.0.0.1:61757 validation_state=Validated active=true recv=274 sent=794 lost=0 retrans=0 rtt=251.283µs min_rtt=Some(100.291µs) rttvar=165.768µs cwnd=101824 sent_bytes=1038667 recv_bytes=14030 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=110391822 max_bandwidth=Some(110391822) startup_exit=Some(StartupExit { cwnd: 28079, bandwidth: Some(12345), reason: BandwidthPlateau }) )",
        ].into_iter().map(|v| v.to_string()).collect();

        let metric = StartupExit::new_from_logs(&logs);
        assert_eq!(metric[0].bandwidth, 12345);
    }

    #[test]
    fn startup_exit_multiple_stats() {
        let logs: Vec<String> = [
            "[2025-12-20T07:09:59.547130000Z INFO  async_http3_server] received new connection!",
            "[2025-12-20T07:09:59.551243000Z INFO  async_http3_server::server] received unhandled event: IncomingSettings { settings: [(553772674384608777, 2680223962524332553)] }",
            "[2025-12-20T07:09:59.551419000Z INFO  async_http3_server::server] received unhandled event: BodyBytesReceived { stream_id: 0, num_bytes: 0, fin: true }",
            // Some(54321)
            "[2025-12-20T07:09:59.565448000Z INFO  async_http3_server::server] stream closed: Some(local_addr=0.0.0.0:9999 peer_addr=127.0.0.1:61757 validation_state=Validated active=true recv=274 sent=794 lost=0 retrans=0 rtt=251.283µs min_rtt=Some(100.291µs) rttvar=165.768µs cwnd=101824 sent_bytes=1038667 recv_bytes=14030 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=110391822 max_bandwidth=Some(110391822) startup_exit=Some(StartupExit { cwnd: 28079, bandwidth: Some(54321), reason: BandwidthPlateau }) )",
            // None
            "[2025-12-20T07:09:59.565448000Z INFO  async_http3_server::server] stream closed: Some(local_addr=0.0.0.0:9999 peer_addr=127.0.0.1:61757 validation_state=Validated active=true recv=274 sent=794 lost=0 retrans=0 rtt=251.283µs min_rtt=Some(100.291µs) rttvar=165.768µs cwnd=101824 sent_bytes=1038667 recv_bytes=14030 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=110391822 max_bandwidth=Some(110391822) startup_exit=Some(StartupExit { cwnd: 28079, bandwidth: None, reason: BandwidthPlateau }) )",
            // Some(12345)
            "[2025-12-20T07:09:59.565448000Z INFO  async_http3_server::server] stream closed: Some(local_addr=0.0.0.0:9999 peer_addr=127.0.0.1:61757 validation_state=Validated active=true recv=274 sent=794 lost=0 retrans=0 rtt=251.283µs min_rtt=Some(100.291µs) rttvar=165.768µs cwnd=101824 sent_bytes=1038667 recv_bytes=14030 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=110391822 max_bandwidth=Some(110391822) startup_exit=Some(StartupExit { cwnd: 28079, bandwidth: Some(12345), reason: BandwidthPlateau }) )",
        ].into_iter().map(|v| v.to_string()).collect();

        let metric = StartupExit::new_from_logs(&logs);
        assert_eq!(metric.len(), 2);
        assert_eq!(metric[1].bandwidth, 12345);
        assert_eq!(metric[0].bandwidth, 54321);
    }
}
