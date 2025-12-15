use crate::stats::ToStatMetric;
use regex::Regex;
use std::{fmt::Debug, time::Duration};

#[derive(Default, Debug)]
pub struct DownloadDuration {
    pub(crate) duration: Duration,
}

impl DownloadDuration {
    // TODO: use named groups to match and parse more efficiently with just Regex:
    // https://stackoverflow.com/a/628563
    pub fn new_from_logs(logs: &str) -> Self {
        // Regex to get "received in 12.34ms"
        //
        // match float: https://stackoverflow.com/a/12643073
        // [+-]?([0-9]*[.])?[0-9]+
        //
        // match "ms" or "s":
        // [m]?s
        let re = Regex::new(r"received in [+-]?([0-9]*[.])?[0-9]+[m]?s").unwrap();
        let logs = re.captures(logs).unwrap().get(0).unwrap().as_str();

        // trim text and parse download duration
        let download_duraiton = logs
            .trim_start_matches("received in ")
            .trim_end_matches("ms")
            .trim_end_matches("s")
            .trim();
        // dbg!("trimmed logs: {} {}", logs, download_duraiton);

        let download_duration = {
            let duration = download_duraiton.parse::<f32>().unwrap();

            if logs.ends_with("ms") {
                duration
            } else if logs.ends_with("s") {
                duration * 1000.0
            } else {
                unreachable!("Expect ms or s. Instead got: {}", logs)
            }
        };

        DownloadDuration {
            duration: Duration::from_millis(download_duration as u64),
        }
    }
}

impl ToStatMetric for DownloadDuration {
    fn name(&self) -> String {
        "DownloadDuration".to_string()
    }

    fn as_f64(&self) -> f64 {
        self.duration.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_duration_ms() {
        let logs = "[2025-12-15T04:12:15.895071000Z INFO  quiche_apps::client] connecting to 127.0.0.1:9999 from 0.0.0.0:52522 with scid eff94d1df3d374a001a807c4c5b7b44fca82e6aa \
         [2025-12-15T04:12:15.914151000Z INFO  quiche_apps::common] 1/1 response(s) received in 18.767083ms, closing... \
         [2025-12-15T04:12:15.914211000Z INFO  quiche_apps::client] connection closed, recv=794 sent=291 lost=0 retrans=0 sent_bytes=15318 recv_bytes=1038727 lost_bytes=0 [local_addr=0.0.0.0:52522 peer_addr=127.0.0.1:9999 validation_state=Validated active=true recv=794 sent=291 lost=0 retrans=0 rtt=923.083µs min_rtt=Some(144.738µs) rttvar=937.037µs cwnd=13500 sent_bytes=15318 recv_bytes=1038727 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=1997003]";

        let metric = DownloadDuration::new_from_logs(logs);
        assert_eq!(metric.duration, Duration::from_millis(18));
    }

    #[test]
    fn download_duration_s() {
        let logs = "[2025-12-15T04:12:15.895071000Z INFO  quiche_apps::client] connecting to 127.0.0.1:9999 from 0.0.0.0:52522 with scid eff94d1df3d374a001a807c4c5b7b44fca82e6aa \
         [2025-12-15T04:12:15.914151000Z INFO  quiche_apps::common] 1/1 response(s) received in 1.335630013s, closing... \
         [2025-12-15T04:12:15.914211000Z INFO  quiche_apps::client] connection closed, recv=794 sent=291 lost=0 retrans=0 sent_bytes=15318 recv_bytes=1038727 lost_bytes=0 [local_addr=0.0.0.0:52522 peer_addr=127.0.0.1:9999 validation_state=Validated active=true recv=794 sent=291 lost=0 retrans=0 rtt=923.083µs min_rtt=Some(144.738µs) rttvar=937.037µs cwnd=13500 sent_bytes=15318 recv_bytes=1038727 lost_bytes=0 stream_retrans_bytes=0 pmtu=1350 delivery_rate=1997003]";

        let metric = DownloadDuration::new_from_logs(logs);
        assert_eq!(metric.duration, Duration::from_millis(1335));
    }
}
