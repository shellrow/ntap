use crate::config::AppConfig;
use crate::net::packet::{PacketFrame, PacketStorage};
use crate::thread_log;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

use clap::ArgMatches;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextLevelProtocol;

pub fn live_capture(app: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Check .ntap directory
    match crate::sys::get_config_dir_path() {
        Some(_config_dir) => {}
        None => {
            eprintln!("Error: Could not get config directory path");
            return Ok(());
        }
    }

    // Check dependencies (Currently only for Windows)
    match crate::sys::check_deps() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {:?}", e);
            return Ok(());
        }
    }

    // Load AppConfig
    let mut config = AppConfig::load();

    if app.contains_id("tickrate") {
        config.display.tick_rate = *app.get_one("tickrate").unwrap_or(&1000);
    }

    // Interface filter
    if app.contains_id("interfaces") {
        match app.get_many::<String>("interfaces") {
            Some(interfaces) => {
                config.network.interfaces = interfaces.cloned().collect();
            }
            None => {
                config.network.interfaces = Vec::new();
            }
        }
    }

    // Protocol filter
    let mut ethertypes: HashSet<EtherType> = HashSet::new();
    let mut ip_next_protocols: HashSet<IpNextLevelProtocol> = HashSet::new();
    if app.contains_id("protocols") {
        match app.get_many::<String>("protocols") {
            Some(protocols_ref) => {
                let protocols: Vec<String> = protocols_ref.cloned().collect();
                for protocol in protocols {
                    if let Some(ethertype) = crate::net::packet::get_ethertype_from_str(&protocol) {
                        ethertypes.insert(ethertype);
                    }
                    if let Some(ip_next_protocol) = crate::net::packet::get_ip_next_protocol_from_str(&protocol) {
                        ip_next_protocols.insert(ip_next_protocol);
                    }
                }
            }
            None => {}
        }
    }

    // IP Address filter
    let ips: HashSet<IpAddr> = match app.get_many::<IpAddr>("ips") {
        Some(ips_ref) => ips_ref.cloned().collect(),
        None => HashSet::new(),
    };

    // Port filter
    let ports: HashSet<u16> = match app.get_many::<u16>("ports") {
        Some(ports_ref) => ports_ref.cloned().collect(),
        None => HashSet::new(),
    };

    if !ip_next_protocols.is_empty() || ips.len() > 0 || ports.len() > 0 {
        ethertypes.insert(EtherType::Ipv4);
        ethertypes.insert(EtherType::Ipv6);
        if ports.len() > 0 {
            ip_next_protocols.insert(IpNextLevelProtocol::Tcp);
            ip_next_protocols.insert(IpNextLevelProtocol::Udp);
        }
    }

    let storage_capacity: u8;
    if app.contains_id("limit") {
        storage_capacity = *app.get_one("limit").unwrap_or(&100);
    } else {
        storage_capacity = u8::MAX;
    }

    // Init logger
    let log_file_path = if let Some(file_path) = &config.logging.file_path {
        // Convert to PathBuf
        Path::new(&file_path).to_path_buf()
    } else {
        crate::sys::get_user_file_path(crate::thread_log::DEFAULT_LOG_FILE_PATH).unwrap()
    };
    let log_file: File = if log_file_path.exists() {
        File::options().write(true).open(&log_file_path)?
    } else {
        File::create(&log_file_path)?
    };
    let mut log_config_builder = simplelog::ConfigBuilder::default();
    log_config_builder.set_time_format_rfc3339();
    if let Some(offset) = crate::time::get_local_offset() {
        log_config_builder.set_time_offset(offset);
    }
    let default_log_config = log_config_builder.build();

    // Init logger with file and terminal output
    // debug build: log to terminal and file
    // release build: log to file only
    if cfg!(debug_assertions) {
        simplelog::CombinedLogger::init(vec![
            simplelog::TermLogger::new(
                simplelog::LevelFilter::Info,
                default_log_config.clone(),
                simplelog::TerminalMode::Mixed,
                simplelog::ColorChoice::Auto,
            ),
            simplelog::WriteLogger::new(
                config.logging.level.to_level_filter(),
                default_log_config,
                log_file,
            ),
        ])?;
    } else {
        simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
            config.logging.level.to_level_filter(),
            default_log_config,
            log_file,
        )])?;
    }
    // Start threads
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];
    let packet_strage: Arc<PacketStorage> = Arc::new(PacketStorage::with_capacity(storage_capacity as usize));
    let packet_strage_ui: Arc<PacketStorage> = Arc::clone(&packet_strage);
    let target_interfaces: Vec<netdev::Interface>;
    if config.network.interfaces.is_empty() {
        target_interfaces = crate::net::interface::get_usable_interfaces();
    } else {
        target_interfaces = crate::net::interface::get_interfaces_by_name(&config.network.interfaces);
    }
    let mut pcap_thread_index = 0;
    let (tx, rx): (Sender<PacketFrame>, Receiver<PacketFrame>) = channel();
    let pcap_handlers = target_interfaces
        .iter()
        .map(|iface| {
            let iface = iface.clone();
            let mut pcap_option = crate::net::pcap::PacketCaptureOptions::from_interface(&iface);
            pcap_option.ether_types = ethertypes.clone();
            pcap_option.ip_protocols = ip_next_protocols.clone();
            pcap_option.src_ips = ips.clone();
            pcap_option.src_ports = ports.clone();
            pcap_option.dst_ips = ips.clone();
            pcap_option.dst_ports = ports.clone();
            let thread_name = format!("pcap-thread-{}", iface.name.clone());
            let pcap_thread = thread::Builder::new().name(thread_name.clone());
            let tx_clone = tx.clone();
            let pcap_handler = pcap_thread.spawn(move || {
                crate::net::pcap::start_live_capture(
                    pcap_option,
                    tx_clone,
                    iface,
                );
            });
            thread_log!(info, "start thread {}", thread_name);
            pcap_thread_index += 1;
            pcap_handler
        })
        .collect::<Vec<_>>();

    let receiver_handler = thread::spawn(move || {
        thread_log!(info, "start mpsc reveiver thread");
        while let Ok(mut frame) = rx.recv() {
            frame.capture_no = packet_strage.generate_capture_no();
            packet_strage.add_packet(frame);
        }
    });

    threads.push(receiver_handler);

    for pcap_handler in pcap_handlers {
        match pcap_handler {
            Ok(handle) => {
                threads.push(handle);
            }
            Err(e) => {
                thread_log!(error, "Error: {:?}", e);
            }
        }
    }

    thread_log!(info, "start TUI, live_packet_capture");

    // Clear screen before starting TUI
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    // Move cursor to top left corner
    crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

    crate::tui::live::terminal::run(
        config,
        app.contains_id("enhanced-graphics"),
        &packet_strage_ui,
    )?;
    Ok(())
}
