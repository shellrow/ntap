use crate::config::AppConfig;
use crate::net::stat::NetStatStrage;
use anyhow::Result;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextProtocol;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct MonitorOptions {
    pub interfaces: Vec<String>,
    pub protocols: Vec<String>,
    pub ips: Vec<IpAddr>,
    pub ports: Vec<u16>,
    pub tickrate: Option<u64>,
    pub enhanced_graphics: bool,
}

pub async fn monitor(opts: MonitorOptions) -> Result<()> {
    if crate::sys::get_config_dir_path().is_none() {
        anyhow::bail!("Could not get config directory path");
    }

    crate::sys::check_deps().map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let mut config = AppConfig::load();
    crate::log::init_logger(&config)?;
    crate::db::init_databases()?;

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

    let netstat_strage: Arc<NetStatStrage> = Arc::new(NetStatStrage::new());
    let mut netstat_strage_socket = Arc::clone(&netstat_strage);
    let mut netstat_strage_ui = Arc::clone(&netstat_strage);

    let target_interfaces = if config.network.interfaces.is_empty() {
        crate::net::interface::get_usable_interfaces()
    } else {
        crate::net::interface::get_interfaces_by_name(&config.network.interfaces)
    };

    for iface in target_interfaces {
        let mut netstat_strage_pcap = Arc::clone(&netstat_strage);
        let mut pcap_option = crate::net::pcap::PacketCaptureOptions::from_interface(&iface);
        pcap_option.ether_types = ethertypes.clone();
        pcap_option.ip_protocols = ip_next_protocols.clone();
        pcap_option.src_ips = ips.clone();
        pcap_option.src_ports = ports.clone();
        pcap_option.dst_ips = ips.clone();
        pcap_option.dst_ports = ports.clone();
        thread::Builder::new()
            .name(format!("pcap-thread-{}", iface.name))
            .spawn(move || {
                crate::net::pcap::start_background_capture(
                    pcap_option,
                    &mut netstat_strage_pcap,
                    iface,
                );
            })?;
    }

    thread::Builder::new()
        .name("socket-info-update".to_string())
        .spawn(move || {
            crate::net::socket::start_socket_info_update(&mut netstat_strage_socket);
        })?;

    let mut netstat_strage_dns = Arc::clone(&netstat_strage);
    thread::Builder::new()
        .name("dns-map-update".to_string())
        .spawn(move || {
            crate::net::dns::start_dns_map_update(&mut netstat_strage_dns);
        })?;

    crate::tui::monitor::terminal::run(config, opts.enhanced_graphics, &mut netstat_strage_ui)
        .await?;
    Ok(())
}
