use crate::config::AppConfig;
use crate::net::stat::NetStatStorage;
use anyhow::Result;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextProtocol;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::mpsc::channel;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct MonitorOptions {
    pub interfaces: Vec<String>,
    pub protocols: Vec<String>,
    pub ips: Vec<IpAddr>,
    pub ports: Vec<u16>,
    pub tickrate: Option<u64>,
}

pub async fn monitor(opts: MonitorOptions) -> Result<()> {
    if crate::sys::get_config_dir_path().is_none() {
        anyhow::bail!("Could not get config directory path");
    }

    crate::sys::check_deps()?;

    let mut config = AppConfig::load()?;
    crate::log::init_logger(&config)?;
    if let Some(tick_rate) = opts.tickrate {
        config.display.tick_rate = tick_rate;
    }
    if !opts.interfaces.is_empty() {
        config.network.interfaces = opts.interfaces.clone();
    }
    config.validate()?;

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

    let netstat_storage: Arc<NetStatStorage> = Arc::new(NetStatStorage::new());
    let mut netstat_storage_socket = Arc::clone(&netstat_storage);
    let mut netstat_storage_ui = Arc::clone(&netstat_storage);

    let target_interfaces =
        crate::net::interface::resolve_capture_interfaces(&config.network.interfaces)?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut workers = crate::util::WorkerSet::new(Arc::clone(&stop));
    let (ready_tx, ready_rx) = channel();

    for iface in target_interfaces {
        let mut netstat_storage_pcap = Arc::clone(&netstat_storage);
        let mut pcap_option = crate::net::pcap::PacketCaptureOptions::new();
        pcap_option.ether_types = ethertypes.clone();
        pcap_option.ip_protocols = ip_next_protocols.clone();
        pcap_option.src_ips = ips.clone();
        pcap_option.src_ports = ports.clone();
        pcap_option.dst_ips = ips.clone();
        pcap_option.dst_ports = ports.clone();
        let capture_stop = Arc::clone(&stop);
        let ready_tx = ready_tx.clone();
        workers.spawn(format!("pcap-thread-{}", iface.name), move || {
            crate::net::pcap::start_background_capture(
                pcap_option,
                &mut netstat_storage_pcap,
                iface,
                &capture_stop,
                ready_tx,
            );
        })?;
    }
    drop(ready_tx);
    let mut startup_error = None;
    for _ in 0..workers.worker_count() {
        match ready_rx.recv_timeout(Duration::from_secs(5)) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                startup_error = Some(error);
                break;
            }
            Err(error) => {
                startup_error = Some(format!("capture startup did not complete: {error}"));
                break;
            }
        }
    }
    if let Some(error) = startup_error {
        workers.shutdown();
        anyhow::bail!(error);
    }

    let socket_stop = Arc::clone(&stop);
    workers.spawn("socket-info-update".to_string(), move || {
        crate::net::socket::start_socket_info_update(&mut netstat_storage_socket, &socket_stop);
    })?;

    if config.network.reverse_dns {
        let mut netstat_storage_dns = Arc::clone(&netstat_storage);
        let dns_stop = Arc::clone(&stop);
        workers.spawn("dns-map-update".to_string(), move || {
            crate::net::dns::start_dns_map_update(&mut netstat_storage_dns, &dns_stop);
        })?;
    }

    let result = crate::tui::monitor::terminal::run(config, &mut netstat_storage_ui).await;
    workers.shutdown();
    result
}
