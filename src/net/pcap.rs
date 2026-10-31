use crate::net::packet::PacketFrame;
use crate::net::stat::NetStatStorage;
use nex::net::interface::Interface;
use nex::packet::frame::Frame;
use nex::packet::frame::ParseOption;
use nex::packet::{ethernet::EtherType, ip::IpNextProtocol};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::mpsc::{Sender, SyncSender, TrySendError};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

/// Packet capture options
#[derive(Debug, Clone)]
pub struct PacketCaptureOptions {
    /// Preserve capture bytes for optional PCAPNG recording.
    pub record: bool,
    /// Source IP addresses to filter. If empty, all source IP addresses will be captured
    pub src_ips: HashSet<IpAddr>,
    /// Destination IP addresses to filter. If empty, all destination IP addresses will be captured
    pub dst_ips: HashSet<IpAddr>,
    /// Source ports to filter. If empty, all source ports will be captured
    pub src_ports: HashSet<u16>,
    /// Destination ports to filter. If empty, all destination ports will be captured
    pub dst_ports: HashSet<u16>,
    /// Ether types to filter. If empty, all ether types will be captured
    pub ether_types: HashSet<EtherType>,
    /// IP protocols to filter. If empty, all IP protocols will be captured
    pub ip_protocols: HashSet<IpNextProtocol>,
    /// Read Timeout for read next packet (Linux, BPF only)
    pub read_timeout: Duration,
    /// Capture in promiscuous mode
    pub promiscuous: bool,
}

fn datalink_config(options: &PacketCaptureOptions) -> nex::datalink::Config {
    let mut config = nex::datalink::Config::default();
    // Leave room for full-size IP packets, jumbo frames, and backend record headers.
    config.read_buffer_size = 256 * 1024;
    config.read_timeout = Some(options.read_timeout);
    config.promiscuous = options.promiscuous;
    config
}

impl PacketCaptureOptions {
    pub fn new() -> PacketCaptureOptions {
        PacketCaptureOptions {
            record: false,
            src_ips: HashSet::new(),
            dst_ips: HashSet::new(),
            src_ports: HashSet::new(),
            dst_ports: HashSet::new(),
            ether_types: HashSet::new(),
            ip_protocols: HashSet::new(),
            read_timeout: Duration::from_millis(200),
            promiscuous: false,
        }
    }
}

pub fn start_live_capture(
    capture_options: PacketCaptureOptions,
    msg_tx: SyncSender<PacketFrame>,
    interface: Interface,
    stop: &AtomicBool,
    dropped_packets: &std::sync::atomic::AtomicUsize,
    ready: Sender<Result<(), String>>,
) {
    let config = datalink_config(&capture_options);
    let (mut _tx, mut rx) = match nex::datalink::channel(&interface, config) {
        Ok(nex::datalink::Channel::Ethernet(tx, rx)) => {
            let _ = ready.send(Ok(()));
            (tx, rx)
        }
        Ok(_) => {
            let _ = ready.send(Err(format!(
                "unsupported capture channel for interface {}",
                interface.name
            )));
            return;
        }
        Err(e) => {
            let _ = ready.send(Err(format!(
                "failed to capture on interface {}: {e}",
                interface.name
            )));
            return;
        }
    };
    loop {
        if stop.load(Ordering::Acquire) {
            break;
        }
        {
            let packet = match rx.next() {
                Ok(packet) => packet,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    continue;
                }
                Err(error) => {
                    let message =
                        format!("capture stopped on interface {}: {error}", interface.name);
                    tracing::error!("{message}");
                    let _ = ready.send(Err(message));
                    break;
                }
            };
            let mut parse_option: ParseOption = ParseOption::default();
            // nex normalizes BPF and Npcap link headers to Ethernet. Linux TUN
            // devices still expose raw IP through AF_PACKET.
            if cfg!(target_os = "linux") && interface.is_tun() {
                parse_option.from_ip_packet = true;
            }
            let frame: Frame = match Frame::from_buf(packet, parse_option) {
                Some(frame) => frame,
                None => {
                    tracing::debug!("ignored an unsupported or malformed frame");
                    continue;
                }
            };
            if filter_packet(&frame, &capture_options) {
                let mut packet_frame =
                    PacketFrame::from_nex_frame(0, interface.index, interface.name.clone(), frame);
                if capture_options.record {
                    packet_frame.raw_data = bytes::Bytes::copy_from_slice(packet);
                    packet_frame.raw_ip = cfg!(target_os = "linux") && interface.is_tun();
                }
                match msg_tx.try_send(packet_frame) {
                    Ok(()) => {}
                    Err(TrySendError::Full(_)) => {
                        let dropped = dropped_packets.fetch_add(1, Ordering::Relaxed) + 1;
                        if dropped == 1 || dropped.is_power_of_two() {
                            tracing::warn!(dropped, "live capture queue is full; dropping packets");
                        }
                    }
                    Err(TrySendError::Disconnected(_)) => break,
                }
            }
        }
    }
}

pub fn start_background_capture(
    capture_options: PacketCaptureOptions,
    netstat_storage: &mut Arc<NetStatStorage>,
    interface: Interface,
    stop: &AtomicBool,
    ready: Sender<Result<(), String>>,
) {
    let config = datalink_config(&capture_options);
    let (mut _tx, mut rx) = match nex::datalink::channel(&interface, config) {
        Ok(nex::datalink::Channel::Ethernet(tx, rx)) => {
            let _ = ready.send(Ok(()));
            (tx, rx)
        }
        Ok(_) => {
            let _ = ready.send(Err(format!(
                "unsupported capture channel for interface {}",
                interface.name
            )));
            return;
        }
        Err(e) => {
            let _ = ready.send(Err(format!(
                "failed to capture on interface {}: {e}",
                interface.name
            )));
            return;
        }
    };
    loop {
        if stop.load(Ordering::Acquire) {
            break;
        }
        {
            let packet = match rx.next() {
                Ok(packet) => packet,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::TimedOut
                            | std::io::ErrorKind::WouldBlock
                            | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    continue;
                }
                Err(error) => {
                    let message =
                        format!("capture stopped on interface {}: {error}", interface.name);
                    tracing::error!("{message}");
                    let _ = ready.send(Err(message));
                    break;
                }
            };
            let mut parse_option: ParseOption = ParseOption::default();
            // nex normalizes BPF and Npcap link headers to Ethernet. Linux TUN
            // devices still expose raw IP through AF_PACKET.
            if cfg!(target_os = "linux") && interface.is_tun() {
                parse_option.from_ip_packet = true;
            }
            let frame: Frame = match Frame::from_buf(packet, parse_option) {
                Some(frame) => frame,
                None => {
                    tracing::debug!("ignored an unsupported or malformed frame");
                    continue;
                }
            };
            if filter_packet(&frame, &capture_options) {
                let packet_frame =
                    PacketFrame::from_nex_frame(0, interface.index, interface.name.clone(), frame);
                netstat_storage.update(packet_frame);
            }
        }
    }
}

/// Protocol choices form a union; endpoint and port constraints intersect it.
fn filter_packet(frame: &Frame, options: &PacketCaptureOptions) -> bool {
    let ethernet = frame
        .datalink
        .as_ref()
        .and_then(|layer| layer.ethernet.as_ref());
    let arp = frame.datalink.as_ref().and_then(|layer| layer.arp.as_ref());
    let ipv4 = frame.ip.as_ref().and_then(|layer| layer.ipv4.as_ref());
    let ipv6 = frame.ip.as_ref().and_then(|layer| layer.ipv6.as_ref());
    let tcp = frame
        .transport
        .as_ref()
        .and_then(|layer| layer.tcp.as_ref());
    let udp = frame
        .transport
        .as_ref()
        .and_then(|layer| layer.udp.as_ref());
    let ether_type = ethernet.map(|header| header.ethertype).or_else(|| {
        if ipv4.is_some() {
            Some(EtherType::Ipv4)
        } else if ipv6.is_some() {
            Some(EtherType::Ipv6)
        } else {
            None
        }
    });
    let protocol = if tcp.is_some() {
        Some(IpNextProtocol::Tcp)
    } else if udp.is_some() {
        Some(IpNextProtocol::Udp)
    } else if frame.ip.as_ref().is_some_and(|ip| ip.icmp.is_some()) {
        Some(IpNextProtocol::Icmp)
    } else if frame.ip.as_ref().is_some_and(|ip| ip.icmpv6.is_some()) {
        Some(IpNextProtocol::Icmpv6)
    } else {
        ipv4.map(|header| header.next_level_protocol)
            .or_else(|| ipv6.map(|header| header.next_header))
    };
    if !(options.ether_types.is_empty() && options.ip_protocols.is_empty()
        || ether_type.is_some_and(|value| options.ether_types.contains(&value))
        || protocol.is_some_and(|value| options.ip_protocols.contains(&value)))
    {
        return false;
    }
    if !options.src_ips.is_empty() || !options.dst_ips.is_empty() {
        let endpoints = ipv4
            .map(|h| (IpAddr::V4(h.source), IpAddr::V4(h.destination)))
            .or_else(|| ipv6.map(|h| (IpAddr::V6(h.source), IpAddr::V6(h.destination))))
            .or_else(|| {
                arp.map(|h| {
                    (
                        IpAddr::V4(h.sender_proto_addr),
                        IpAddr::V4(h.target_proto_addr),
                    )
                })
            });
        if !endpoints.is_some_and(|(src, dst)| {
            options.src_ips.contains(&src) || options.dst_ips.contains(&dst)
        }) {
            return false;
        }
    }
    if !options.src_ports.is_empty() || !options.dst_ports.is_empty() {
        let ports = tcp
            .map(|h| (h.source, h.destination))
            .or_else(|| udp.map(|h| (h.source, h.destination)));
        if !ports.is_some_and(|(src, dst)| {
            options.src_ports.contains(&src) || options.dst_ports.contains(&dst)
        }) {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ipv4(protocol: u8, transport: &[u8], ethernet: bool) -> Frame {
        let mut bytes = if ethernet { vec![0; 14] } else { vec![] };
        if ethernet {
            bytes[12..14].copy_from_slice(&[8, 0]);
        }
        let offset = bytes.len();
        bytes.extend_from_slice(&[0; 20]);
        bytes[offset] = 0x45;
        bytes[offset + 2..offset + 4]
            .copy_from_slice(&((20 + transport.len()) as u16).to_be_bytes());
        bytes[offset + 8] = 64;
        bytes[offset + 9] = protocol;
        bytes[offset + 12..offset + 16].copy_from_slice(&[192, 0, 2, 1]);
        bytes[offset + 16..offset + 20].copy_from_slice(&[198, 51, 100, 2]);
        bytes.extend_from_slice(transport);
        Frame::from_buf(
            &bytes,
            ParseOption {
                from_ip_packet: !ethernet,
                offset: 0,
            },
        )
        .unwrap()
    }

    #[test]
    fn port_filter_rejects_packets_without_ports() {
        let frame = ipv4(1, &[8, 0, 0, 0, 0, 0, 0, 0], true);
        let mut options = PacketCaptureOptions::new();
        options.dst_ports.insert(443);
        assert!(!filter_packet(&frame, &options));
    }

    #[test]
    fn protocol_and_port_filters_intersect() {
        let udp = ipv4(17, &[0, 53, 0, 80, 0, 8, 0, 0], true);
        let mut options = PacketCaptureOptions::new();
        options.ip_protocols.insert(IpNextProtocol::Tcp);
        options.dst_ports.insert(80);
        assert!(!filter_packet(&udp, &options));
        options.ip_protocols.insert(IpNextProtocol::Udp);
        assert!(filter_packet(&udp, &options));
    }

    #[test]
    fn protocol_choices_form_a_union_across_layers() {
        let udp = ipv4(17, &[0, 53, 0, 80, 0, 8, 0, 0], true);
        let mut options = PacketCaptureOptions::new();
        options.ether_types.insert(EtherType::Ipv4);
        options.ip_protocols.insert(IpNextProtocol::Tcp);
        assert!(filter_packet(&udp, &options));
    }

    #[test]
    fn raw_ip_frames_obey_ip_version_and_endpoint_filters() {
        let udp = ipv4(17, &[0, 53, 0, 80, 0, 8, 0, 0], false);
        let mut options = PacketCaptureOptions::new();
        options.ether_types.insert(EtherType::Ipv6);
        assert!(!filter_packet(&udp, &options));
        options.ether_types.insert(EtherType::Ipv4);
        options.dst_ips.insert("198.51.100.2".parse().unwrap());
        assert!(filter_packet(&udp, &options));
        options.dst_ips.clear();
        options.src_ips.insert("203.0.113.1".parse().unwrap());
        assert!(!filter_packet(&udp, &options));
    }

    #[test]
    fn missing_headers_do_not_bypass_filters() {
        let frame = Frame {
            datalink: None,
            ip: None,
            transport: None,
            payload: bytes::Bytes::new(),
            packet_len: 0,
        };
        let mut options = PacketCaptureOptions::new();
        assert!(filter_packet(&frame, &options));
        options.src_ips.insert("192.0.2.1".parse().unwrap());
        assert!(!filter_packet(&frame, &options));
        options.src_ips.clear();
        options.ip_protocols.insert(IpNextProtocol::Tcp);
        assert!(!filter_packet(&frame, &options));
    }
    #[test]
    fn receive_buffer_preserves_large_datagrams() {
        let config = datalink_config(&PacketCaptureOptions::new());
        let mut udp = vec![0x5a; 60_008];
        udp[..8].copy_from_slice(&[0, 53, 0, 80, 0xea, 0x68, 0, 0]);
        let frame = ipv4(17, &udp, true);
        assert!(config.read_buffer_size >= frame.packet_len + 256);
        assert_eq!(frame.payload.len(), 60_000);
    }
}
