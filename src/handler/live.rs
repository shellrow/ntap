use crate::config::AppConfig;
use crate::net::packet::{PacketFrame, PacketStorage};
use anyhow::Result;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextProtocol;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::mpsc::{Receiver, SyncSender, channel, sync_channel};
use std::sync::{Arc, atomic::AtomicBool};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Default)]
pub struct LiveOptions {
    pub interfaces: Vec<String>,
    pub protocols: Vec<String>,
    pub ips: Vec<IpAddr>,
    pub ports: Vec<u16>,
    pub limit: Option<usize>,
    pub write: Option<std::path::PathBuf>,
}

pub async fn live_capture(opts: LiveOptions) -> Result<()> {
    crate::tui::terminal::ensure_terminal()?;
    if crate::sys::get_config_dir_path().is_none() {
        anyhow::bail!("Could not get config directory path");
    }

    crate::sys::check_deps()?;

    let mut config = AppConfig::load()?;
    crate::log::init_logger(&config)?;

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

    let storage_capacity = opts.limit.unwrap_or(u8::MAX as usize);
    let packet_storage: Arc<PacketStorage> =
        Arc::new(PacketStorage::with_capacity(storage_capacity));
    let packet_storage_ui: Arc<PacketStorage> = Arc::clone(&packet_storage);
    let target_interfaces =
        crate::net::interface::resolve_capture_interfaces(&config.network.interfaces)?;

    let mut recording = opts
        .write
        .as_deref()
        .map(crate::net::recording::create)
        .transpose()?;
    if let Some(writer) = &mut recording {
        for interface in &target_interfaces {
            writer.register_interface(
                interface.index,
                &interface.name,
                cfg!(target_os = "linux") && interface.is_tun(),
            )?;
        }
    }
    const CAPTURE_QUEUE_CAPACITY: usize = 4096;
    let (tx, rx): (SyncSender<PacketFrame>, Receiver<PacketFrame>) =
        sync_channel(CAPTURE_QUEUE_CAPACITY);
    let dropped_packets = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let stop = Arc::new(AtomicBool::new(false));
    let mut capture_workers = crate::util::WorkerSet::new(Arc::clone(&stop));
    let (ready_tx, ready_rx) = channel();
    for iface in target_interfaces {
        let iface = iface.clone();
        let mut pcap_option = crate::net::pcap::PacketCaptureOptions::new();
        pcap_option.record = recording.is_some();
        pcap_option.ether_types = ethertypes.clone();
        pcap_option.ip_protocols = ip_next_protocols.clone();
        pcap_option.src_ips = ips.clone();
        pcap_option.src_ports = ports.clone();
        pcap_option.dst_ips = ips.clone();
        pcap_option.dst_ports = ports.clone();
        let tx_clone = tx.clone();
        let capture_stop = Arc::clone(&stop);
        let dropped_packets = Arc::clone(&dropped_packets);
        let ready_tx = ready_tx.clone();
        capture_workers.spawn(format!("live-pcap-{}", iface.name), move || {
            crate::net::pcap::start_live_capture(
                pcap_option,
                tx_clone,
                iface,
                &capture_stop,
                &dropped_packets,
                ready_tx,
            );
        })?;
    }
    let recording_status = ready_tx.clone();
    drop(ready_tx);
    let mut startup_error = None;
    for _ in 0..capture_workers.worker_count() {
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
        capture_workers.shutdown();
        anyhow::bail!(error);
    }

    let receiver_worker = thread::Builder::new()
        .name("live-packet-receiver".to_string())
        .spawn(move || {
            while let Ok(mut frame) = rx.recv() {
                if let Some(writer) = &mut recording
                    && let Err(error) = writer.write_packet(&frame)
                {
                    let _ = recording_status.send(Err(format!("failed to write capture: {error}")));
                    return Err(error);
                }
                frame.raw_data = bytes::Bytes::new();
                frame.capture_no = packet_storage.generate_capture_no();
                packet_storage.add_packet(frame);
            }
            if let Some(writer) = recording {
                writer.finish()?;
            }
            Ok::<_, anyhow::Error>(())
        })?;

    let result =
        crate::tui::live::terminal::run(config, &packet_storage_ui, ready_rx, &dropped_packets)
            .await;
    capture_workers.shutdown();
    drop(tx);
    receiver_worker
        .join()
        .map_err(|_| anyhow::anyhow!("the packet receiver thread panicked"))??;
    result
}
