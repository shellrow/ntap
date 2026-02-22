use crate::config::AppConfig;
use crate::net::packet::{PacketFrame, PacketStorage};
use anyhow::Result;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextProtocol;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct LiveOptions {
    pub interfaces: Vec<String>,
    pub protocols: Vec<String>,
    pub ips: Vec<IpAddr>,
    pub ports: Vec<u16>,
    pub tickrate: Option<u64>,
    pub enhanced_graphics: bool,
    pub limit: Option<usize>,
}

pub async fn live_capture(opts: LiveOptions) -> Result<()> {
    if crate::sys::get_config_dir_path().is_none() {
        anyhow::bail!("Could not get config directory path");
    }

    crate::sys::check_deps().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let mut config = AppConfig::load();
    crate::log::init_logger(&config)?;

    if let Some(tick_rate) = opts.tickrate {
        config.display.tick_rate = tick_rate;
    }
    if !opts.interfaces.is_empty() {
        config.network.interfaces = opts.interfaces.clone();
    }

    let mut ethertypes: HashSet<EtherType> = HashSet::new();
    let mut ip_next_protocols: HashSet<IpNextProtocol> = HashSet::new();
    for protocol in &opts.protocols {
        if let Some(ethertype) = crate::net::packet::get_ethertype_from_str(protocol) {
            ethertypes.insert(ethertype);
        }
        if let Some(ip_next_protocol) = crate::net::packet::get_ip_next_protocol_from_str(protocol)
        {
            ip_next_protocols.insert(ip_next_protocol);
        }
    }

    let ips: HashSet<IpAddr> = opts.ips.iter().copied().collect();
    let ports: HashSet<u16> = opts.ports.iter().copied().collect();

    if !ip_next_protocols.is_empty() || !ips.is_empty() || !ports.is_empty() {
        ethertypes.insert(EtherType::Ipv4);
        ethertypes.insert(EtherType::Ipv6);
        if !ports.is_empty() {
            ip_next_protocols.insert(IpNextProtocol::Tcp);
            ip_next_protocols.insert(IpNextProtocol::Udp);
        }
    }

    let storage_capacity = opts.limit.unwrap_or(u8::MAX as usize);
    let packet_strage: Arc<PacketStorage> =
        Arc::new(PacketStorage::with_capacity(storage_capacity));
    let packet_strage_ui: Arc<PacketStorage> = Arc::clone(&packet_strage);
    let target_interfaces = if config.network.interfaces.is_empty() {
        crate::net::interface::get_usable_interfaces()
    } else {
        crate::net::interface::get_interfaces_by_name(&config.network.interfaces)
    };

    let (tx, rx): (Sender<PacketFrame>, Receiver<PacketFrame>) = channel();

    for iface in target_interfaces {
        let iface = iface.clone();
        let mut pcap_option = crate::net::pcap::PacketCaptureOptions::from_interface(&iface);
        pcap_option.ether_types = ethertypes.clone();
        pcap_option.ip_protocols = ip_next_protocols.clone();
        pcap_option.src_ips = ips.clone();
        pcap_option.src_ports = ports.clone();
        pcap_option.dst_ips = ips.clone();
        pcap_option.dst_ports = ports.clone();
        let tx_clone = tx.clone();
        thread::Builder::new()
            .name(format!("live-pcap-{}", iface.name))
            .spawn(move || {
                crate::net::pcap::start_live_capture(pcap_option, tx_clone, iface);
            })?;
    }

    thread::Builder::new()
        .name("live-packet-receiver".to_string())
        .spawn(move || {
            while let Ok(mut frame) = rx.recv() {
                frame.capture_no = packet_strage.generate_capture_no();
                packet_strage.add_packet(frame);
            }
        })?;

    crate::tui::live::terminal::run(config, opts.enhanced_graphics, &packet_strage_ui).await?;
    Ok(())
}
