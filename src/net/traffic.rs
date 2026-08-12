use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub enum Direction {
    Egress,
    Ingress,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrafficInfo {
    pub packet_sent: usize,
    pub packet_received: usize,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub egress_packets_per_sec: usize,
    pub egress_bytes_per_sec: usize,
    pub ingress_packets_per_sec: usize,
    pub ingress_bytes_per_sec: usize,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
}

impl TrafficInfo {
    pub fn new() -> Self {
        TrafficInfo {
            packet_sent: 0,
            packet_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            egress_packets_per_sec: 0,
            egress_bytes_per_sec: 0,
            ingress_packets_per_sec: 0,
            ingress_bytes_per_sec: 0,
            first_seen: SystemTime::now(),
            last_seen: SystemTime::now(),
        }
    }
    pub fn add_traffic(&mut self, traffic: &TrafficInfo) {
        self.packet_sent = self.packet_sent.saturating_add(traffic.packet_sent);
        self.packet_received = self.packet_received.saturating_add(traffic.packet_received);
        self.bytes_sent = self.bytes_sent.saturating_add(traffic.bytes_sent);
        self.bytes_received = self.bytes_received.saturating_add(traffic.bytes_received);
        self.last_seen = SystemTime::now();
    }
    pub fn record(&mut self, direction: Direction, bytes: usize) {
        match direction {
            Direction::Egress => {
                self.packet_sent = self.packet_sent.saturating_add(1);
                self.bytes_sent = self.bytes_sent.saturating_add(bytes);
            }
            Direction::Ingress => {
                self.packet_received = self.packet_received.saturating_add(1);
                self.bytes_received = self.bytes_received.saturating_add(bytes);
            }
        }
        self.last_seen = SystemTime::now();
    }
    pub fn update_egress_packets_per_sec(&mut self, sent_packets: usize, duration: Duration) {
        self.egress_packets_per_sec = rate_per_second(sent_packets, duration);
    }
    pub fn update_egress_bytes_per_sec(&mut self, sent_bytes: usize, duration: Duration) {
        self.egress_bytes_per_sec = rate_per_second(sent_bytes, duration);
    }
    pub fn update_ingress_packets_per_sec(&mut self, received_packets: usize, duration: Duration) {
        self.ingress_packets_per_sec = rate_per_second(received_packets, duration);
    }
    pub fn update_ingress_bytes_per_sec(&mut self, received_bytes: usize, duration: Duration) {
        self.ingress_bytes_per_sec = rate_per_second(received_bytes, duration);
    }
    pub fn update_bytes_per_sec(&mut self, traffic: &TrafficInfo, duration: Duration) {
        self.update_egress_packets_per_sec(traffic.packet_sent, duration);
        self.update_ingress_packets_per_sec(traffic.packet_received, duration);
        self.update_egress_bytes_per_sec(traffic.bytes_sent, duration);
        self.update_ingress_bytes_per_sec(traffic.bytes_received, duration);
    }
    pub fn total_bytes(&self) -> usize {
        self.bytes_sent.saturating_add(self.bytes_received)
    }
    pub fn format_bytes(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
    pub fn format_packets(packets: usize) -> String {
        if packets >= 1_000_000 {
            format!("{:.2} Mp", packets as f64 / 1000000.0)
        } else if packets >= 1_000 {
            format!("{:.2} Kp", packets as f64 / 1000.0)
        } else {
            format!("{} p", packets)
        }
    }
    pub fn formatted_total_bytes(&self) -> String {
        Self::format_bytes(self.total_bytes())
    }
    pub fn formatted_sent_bytes(&self) -> String {
        Self::format_bytes(self.bytes_sent)
    }
    pub fn formatted_received_bytes(&self) -> String {
        Self::format_bytes(self.bytes_received)
    }
    pub fn formatted_egress_packets_per_sec(&self) -> String {
        format!("{}ps", Self::format_packets(self.egress_packets_per_sec))
    }
    pub fn formatted_ingress_packets_per_sec(&self) -> String {
        format!("{}ps", Self::format_packets(self.ingress_packets_per_sec))
    }
    pub fn formatted_egress_bytes_per_sec(&self) -> String {
        format!("{}ps", Self::format_bytes(self.egress_bytes_per_sec))
    }
    pub fn formatted_ingress_bytes_per_sec(&self) -> String {
        format!("{}ps", Self::format_bytes(self.ingress_bytes_per_sec))
    }
    pub fn to_display_info(&self) -> TrafficDisplayInfo {
        TrafficDisplayInfo::from_traffic(self)
    }
}

fn rate_per_second(value: usize, duration: Duration) -> usize {
    let seconds = duration.as_secs_f64();
    if seconds <= f64::EPSILON {
        0
    } else {
        (value as f64 / seconds) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_packet_magnitudes_in_the_correct_order() {
        assert_eq!(TrafficInfo::format_packets(999), "999 p");
        assert_eq!(TrafficInfo::format_packets(1_000), "1.00 Kp");
        assert_eq!(TrafficInfo::format_packets(1_000_000), "1.00 Mp");
    }

    #[test]
    fn zero_duration_produces_a_zero_rate() {
        assert_eq!(rate_per_second(42, Duration::ZERO), 0);
    }

    #[test]
    fn recording_traffic_updates_counts_bytes_and_time() {
        let mut traffic = TrafficInfo::new();
        let previous = traffic.last_seen;
        traffic.record(Direction::Ingress, 1_500);
        assert_eq!(traffic.packet_received, 1);
        assert_eq!(traffic.bytes_received, 1_500);
        assert!(traffic.last_seen >= previous);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrafficDisplayInfo {
    pub packet_sent: usize,
    pub packet_received: usize,
    pub bytes_sent: usize,
    pub bytes_received: usize,
    pub egress_packets_per_sec: usize,
    pub egress_bytes_per_sec: usize,
    pub ingress_packets_per_sec: usize,
    pub ingress_bytes_per_sec: usize,
    pub formatted_sent_bytes: String,
    pub formatted_received_bytes: String,
    pub formatted_total_bytes: String,
    pub formatted_egress_packets_per_sec: String,
    pub formatted_ingress_packets_per_sec: String,
    pub formatted_egress_bytes_per_sec: String,
    pub formatted_ingress_bytes_per_sec: String,
}

impl TrafficDisplayInfo {
    pub fn from_traffic(traffic: &TrafficInfo) -> Self {
        TrafficDisplayInfo {
            packet_sent: traffic.packet_sent,
            packet_received: traffic.packet_received,
            bytes_sent: traffic.bytes_sent,
            bytes_received: traffic.bytes_received,
            egress_packets_per_sec: traffic.egress_packets_per_sec,
            ingress_packets_per_sec: traffic.ingress_packets_per_sec,
            egress_bytes_per_sec: traffic.egress_bytes_per_sec,
            ingress_bytes_per_sec: traffic.ingress_bytes_per_sec,
            formatted_sent_bytes: traffic.formatted_sent_bytes(),
            formatted_received_bytes: traffic.formatted_received_bytes(),
            formatted_total_bytes: traffic.formatted_total_bytes(),
            formatted_egress_packets_per_sec: traffic.formatted_egress_packets_per_sec(),
            formatted_ingress_packets_per_sec: traffic.formatted_ingress_packets_per_sec(),
            formatted_egress_bytes_per_sec: traffic.formatted_egress_bytes_per_sec(),
            formatted_ingress_bytes_per_sec: traffic.formatted_ingress_bytes_per_sec(),
        }
    }
}
