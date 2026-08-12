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
    config.read_timeout = Some(options.read_timeout);
    config.promiscuous = options.promiscuous;
    config
}

impl PacketCaptureOptions {
    pub fn new() -> PacketCaptureOptions {
        PacketCaptureOptions {
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
        if let Ok(packet) = rx.next() {
            let mut parse_option: ParseOption = ParseOption::default();
            if interface.is_tun()
                || (cfg!(any(target_os = "macos", target_os = "ios")) && interface.is_loopback())
            {
                let payload_offset = if interface.is_loopback() { 14 } else { 0 };
                parse_option.from_ip_packet = true;
                parse_option.offset = payload_offset;
            }
            let frame: Frame = match Frame::from_buf(packet, parse_option) {
                Some(frame) => frame,
                None => {
                    tracing::error!("Failed to parse packet");
                    continue;
                }
            };
            if filter_packet(&frame, &capture_options) {
                let packet_frame =
                    PacketFrame::from_nex_frame(0, interface.index, interface.name.clone(), frame);
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
        if let Ok(packet) = rx.next() {
            let mut parse_option: ParseOption = ParseOption::default();
            if interface.is_tun()
                || (cfg!(any(target_os = "macos", target_os = "ios")) && interface.is_loopback())
            {
                let payload_offset = if interface.is_loopback() { 14 } else { 0 };
                parse_option.from_ip_packet = true;
                parse_option.offset = payload_offset;
            }
            let frame: Frame = match Frame::from_buf(packet, parse_option) {
                Some(frame) => frame,
                None => {
                    tracing::error!("Failed to parse packet");
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

fn filter_packet(frame: &Frame, capture_options: &PacketCaptureOptions) -> bool {
    if let Some(datalink) = &frame.datalink {
        if let Some(ethernet_header) = &datalink.ethernet
            && !filter_ether_type(ethernet_header.ethertype, capture_options)
        {
            return false;
        }
        if let Some(arp_header) = &datalink.arp
            && !filter_host(
                IpAddr::V4(arp_header.sender_proto_addr),
                IpAddr::V4(arp_header.target_proto_addr),
                capture_options,
            )
        {
            return false;
        }
    }
    if let Some(ip) = &frame.ip {
        if let Some(ipv4_header) = &ip.ipv4 {
            if !filter_host(
                IpAddr::V4(ipv4_header.source),
                IpAddr::V4(ipv4_header.destination),
                capture_options,
            ) {
                return false;
            }
            if !filter_ip_protocol(ipv4_header.next_level_protocol, capture_options) {
                return false;
            }
        }
        if let Some(ipv6_header) = &ip.ipv6 {
            if !filter_host(
                IpAddr::V6(ipv6_header.source),
                IpAddr::V6(ipv6_header.destination),
                capture_options,
            ) {
                return false;
            }
            if !filter_ip_protocol(ipv6_header.next_header, capture_options) {
                return false;
            }
        }
    }
    if let Some(transport) = &frame.transport {
        if let Some(tcp_header) = &transport.tcp
            && !filter_port(tcp_header.source, tcp_header.destination, capture_options)
        {
            return false;
        }
        if let Some(udp_header) = &transport.udp
            && !filter_port(udp_header.source, udp_header.destination, capture_options)
        {
            return false;
        }
    }
    true
}

fn filter_host(src_ip: IpAddr, dst_ip: IpAddr, capture_options: &PacketCaptureOptions) -> bool {
    if capture_options.src_ips.is_empty() && capture_options.dst_ips.is_empty() {
        return true;
    }
    capture_options.src_ips.contains(&src_ip) || capture_options.dst_ips.contains(&dst_ip)
}

fn filter_port(src_port: u16, dst_port: u16, capture_options: &PacketCaptureOptions) -> bool {
    if capture_options.src_ports.is_empty() && capture_options.dst_ports.is_empty() {
        return true;
    }
    capture_options.src_ports.contains(&src_port) || capture_options.dst_ports.contains(&dst_port)
}

fn filter_ether_type(ether_type: EtherType, capture_options: &PacketCaptureOptions) -> bool {
    capture_options.ether_types.is_empty() || capture_options.ether_types.contains(&ether_type)
}

fn filter_ip_protocol(protocol: IpNextProtocol, capture_options: &PacketCaptureOptions) -> bool {
    capture_options.ip_protocols.is_empty() || capture_options.ip_protocols.contains(&protocol)
}
