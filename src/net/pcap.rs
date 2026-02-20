use crate::net::interface;
use crate::net::packet::PacketFrame;
use crate::net::stat::NetStatStrage;
use crate::sys;
use nex::net::interface::Interface;
use nex::packet::frame::Frame;
use nex::packet::frame::ParseOption;
use nex::packet::{ethernet::EtherType, ip::IpNextProtocol};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;

/// Packet capture message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptureReport {
    pub bytes: usize,
    pub packets: usize,
    pub start_time: String,
    pub end_time: String,
    pub duration: Duration,
}

impl CaptureReport {
    pub fn new() -> CaptureReport {
        CaptureReport {
            bytes: 0,
            packets: 0,
            start_time: String::new(),
            end_time: String::new(),
            duration: Duration::from_secs(0),
        }
    }
}

/// Packet capture options
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PacketCaptureOptions {
    /// Interface index
    pub interface_index: u32,
    /// Interface name
    #[allow(dead_code)]
    pub interface_name: String,
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
    /// Capture duration limit
    pub capture_timeout: Duration,
    /// Read Timeout for read next packet (Linux, BPF only)
    pub read_timeout: Duration,
    /// Capture in promiscuous mode
    pub promiscuous: bool,
    /// Receive undefined packets
    pub receive_undefined: bool,
    /// Use TUN interface
    pub tunnel: bool,
    /// Loopback interface
    pub loopback: bool,
}

impl PacketCaptureOptions {
    pub fn default() -> Result<PacketCaptureOptions, String> {
        let iface = netdev::get_default_interface()?;
        let options = PacketCaptureOptions {
            interface_index: iface.index,
            interface_name: iface.name.clone(),
            src_ips: HashSet::new(),
            dst_ips: HashSet::new(),
            src_ports: HashSet::new(),
            dst_ports: HashSet::new(),
            ether_types: HashSet::new(),
            ip_protocols: HashSet::new(),
            capture_timeout: Duration::MAX,
            read_timeout: Duration::from_millis(200),
            promiscuous: false,
            receive_undefined: true,
            tunnel: iface.is_tun(),
            loopback: iface.is_loopback(),
        };
        Ok(options)
    }
    pub fn from_interface_index(if_index: u32) -> Option<PacketCaptureOptions> {
        let iface = interface::get_interface_by_index(if_index)?;
        let options = PacketCaptureOptions {
            interface_index: if_index,
            interface_name: iface.name.clone(),
            src_ips: HashSet::new(),
            dst_ips: HashSet::new(),
            src_ports: HashSet::new(),
            dst_ports: HashSet::new(),
            ether_types: HashSet::new(),
            ip_protocols: HashSet::new(),
            capture_timeout: Duration::MAX,
            read_timeout: Duration::from_millis(200),
            promiscuous: false,
            receive_undefined: true,
            tunnel: iface.is_tun(),
            loopback: iface.is_loopback(),
        };
        Some(options)
    }
    pub fn from_interface_name(if_name: String) -> PacketCaptureOptions {
        let iface = interface::get_interface_by_name(if_name).unwrap();

        PacketCaptureOptions {
            interface_index: iface.index,
            interface_name: iface.name.clone(),
            src_ips: HashSet::new(),
            dst_ips: HashSet::new(),
            src_ports: HashSet::new(),
            dst_ports: HashSet::new(),
            ether_types: HashSet::new(),
            ip_protocols: HashSet::new(),
            capture_timeout: Duration::MAX,
            read_timeout: Duration::from_millis(200),
            promiscuous: false,
            receive_undefined: true,
            tunnel: iface.is_tun(),
            loopback: iface.is_loopback(),
        }
    }
    pub fn from_interface(iface: &Interface) -> PacketCaptureOptions {
        PacketCaptureOptions {
            interface_index: iface.index,
            interface_name: iface.name.clone(),
            src_ips: HashSet::new(),
            dst_ips: HashSet::new(),
            src_ports: HashSet::new(),
            dst_ports: HashSet::new(),
            ether_types: HashSet::new(),
            ip_protocols: HashSet::new(),
            capture_timeout: Duration::MAX,
            read_timeout: Duration::from_millis(200),
            promiscuous: false,
            receive_undefined: true,
            tunnel: iface.is_tun(),
            loopback: iface.is_loopback(),
        }
    }
    pub fn add_ethertype_filter(&mut self, ethertype_name: &str) {
        // Currently, EtherType not support from_str, so we need to match it manually
        let name = ethertype_name.to_lowercase();
        match name.as_str() {
            "arp" => {
                self.ether_types.insert(EtherType::Arp);
            }
            "rarp" => {
                self.ether_types.insert(EtherType::Rarp);
            }
            "aarp" => {
                self.ether_types.insert(EtherType::Aarp);
            }
            "ipv4" => {
                self.ether_types.insert(EtherType::Ipv4);
            }
            "ipv6" => {
                self.ether_types.insert(EtherType::Ipv6);
            }
            "vlan" => {
                self.ether_types.insert(EtherType::Vlan);
            }
            "mpls" => {
                self.ether_types.insert(EtherType::Mpls);
            }
            "wakeonlan" => {
                self.ether_types.insert(EtherType::WakeOnLan);
            }
            "rldp" => {
                self.ether_types.insert(EtherType::Rldp);
            }
            "lldp" => {
                self.ether_types.insert(EtherType::Lldp);
            }
            _ => {}
        }
    }
    pub fn add_ip_next_protocol_filter(&mut self, protocol_name: &str) {
        // Currently, IpNextLevelProtocol not support from_str, so we need to match it manually
        let name = protocol_name.to_lowercase();
        match name.as_str() {
            "icmp" => {
                self.ip_protocols.insert(IpNextProtocol::Icmp);
            }
            "icmpv6" => {
                self.ip_protocols.insert(IpNextProtocol::Icmpv6);
            }
            "tcp" => {
                self.ip_protocols.insert(IpNextProtocol::Tcp);
            }
            "udp" => {
                self.ip_protocols.insert(IpNextProtocol::Udp);
            }
            _ => {}
        }
    }
}

/// Start packet capture
pub fn start_capture(
    capture_options: PacketCaptureOptions,
    msg_tx: Sender<PacketFrame>,
    stop: &Arc<Mutex<bool>>,
    interface: Interface,
) -> CaptureReport {
    let mut report = CaptureReport::new();
    let config = nex::datalink::Config {
        write_buffer_size: 4096,
        read_buffer_size: 4096,
        read_timeout: Some(capture_options.read_timeout),
        write_timeout: None,
        channel_type: nex::datalink::ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: capture_options.promiscuous,
    };
    let (mut _tx, mut rx) = match nex::datalink::channel(&interface, config) {
        Ok(nex::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            tracing::warn!("Unknown channel type");
            return report;
        }
        Err(e) => {
            tracing::error!("Error happened {}", e);
            return report;
        }
    };
    let start_time = Instant::now();
    report.start_time = sys::get_sysdate();
    loop {
        if let Ok(packet) = rx.next() {
            let mut parse_option: ParseOption = ParseOption::default();
            if interface.is_tun()
                || (cfg!(any(target_os = "macos", target_os = "ios")) && interface.is_loopback())
            {
                let payload_offset = if interface.is_loopback() { 14 } else { 0 };
                parse_option.from_ip_packet = true;
                parse_option.offset = payload_offset;
            }
            report.bytes = report.bytes.saturating_add(packet.len());
            report.packets = report.packets.saturating_add(1);
            let frame: Frame = match Frame::from_buf(packet, parse_option) {
                Some(frame) => frame,
                None => {
                    tracing::error!("Failed to parse packet");
                    continue;
                }
            };
            if filter_packet(&frame, &capture_options) {
                let packet_frame = PacketFrame::from_nex_frame(
                    report.packets,
                    interface.index,
                    interface.name.clone(),
                    frame,
                );
                if msg_tx.send(packet_frame).is_ok() {}
            }
        }
        if let Ok(stop) = stop.lock() {
            if *stop {
                break;
            }
        }
        if Instant::now().duration_since(start_time) > capture_options.capture_timeout {
            break;
        }
    }
    report.end_time = sys::get_sysdate();
    report.duration = Instant::now().duration_since(start_time);
    report
}

pub fn start_live_capture(
    capture_options: PacketCaptureOptions,
    msg_tx: Sender<PacketFrame>,
    interface: Interface,
) {
    let config = nex::datalink::Config {
        write_buffer_size: 4096,
        read_buffer_size: 4096,
        read_timeout: Some(capture_options.read_timeout),
        write_timeout: None,
        channel_type: nex::datalink::ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: capture_options.promiscuous,
    };
    let (mut _tx, mut rx) = match nex::datalink::channel(&interface, config) {
        Ok(nex::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            tracing::warn!("Unknown channel type");
            return;
        }
        Err(e) => {
            tracing::error!("Error happened {}", e);
            return;
        }
    };
    let start_time = Instant::now();
    loop {
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
                if msg_tx.send(packet_frame).is_ok() {}
            }
        }
        if Instant::now().duration_since(start_time) > capture_options.capture_timeout {
            break;
        }
    }
}

pub fn start_background_capture(
    capture_options: PacketCaptureOptions,
    netstat_strage: &mut Arc<NetStatStrage>,
    interface: Interface,
) {
    let config = nex::datalink::Config {
        write_buffer_size: 4096,
        read_buffer_size: 4096,
        read_timeout: Some(capture_options.read_timeout),
        write_timeout: None,
        channel_type: nex::datalink::ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: capture_options.promiscuous,
    };
    let (mut _tx, mut rx) = match nex::datalink::channel(&interface, config) {
        Ok(nex::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => {
            tracing::warn!("Unknown channel type");
            return;
        }
        Err(e) => {
            tracing::error!("Error happened {}", e);
            return;
        }
    };
    let start_time = Instant::now();
    loop {
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
                netstat_strage.update(packet_frame);
            }
        }
        if Instant::now().duration_since(start_time) > capture_options.capture_timeout {
            break;
        }
    }
}

fn filter_packet(frame: &Frame, capture_options: &PacketCaptureOptions) -> bool {
    if let Some(datalink) = &frame.datalink {
        if let Some(ethernet_header) = &datalink.ethernet {
            if !filter_ether_type(ethernet_header.ethertype, capture_options) {
                return false;
            }
        }
        if let Some(arp_header) = &datalink.arp {
            if !filter_host(
                IpAddr::V4(arp_header.sender_proto_addr),
                IpAddr::V4(arp_header.target_proto_addr),
                capture_options,
            ) {
                return false;
            }
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
        if let Some(tcp_header) = &transport.tcp {
            if !filter_port(tcp_header.source, tcp_header.destination, capture_options) {
                return false;
            }
        }
        if let Some(udp_header) = &transport.udp {
            if !filter_port(udp_header.source, udp_header.destination, capture_options) {
                return false;
            }
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
