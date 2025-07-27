use crate::config::AppConfig;
use crate::net::stat::NetStatStrage;
use anyhow::Result;
use std::collections::HashSet;
use std::net::IpAddr;
use std::sync::Arc;
use std::thread;

use clap::ArgMatches;

//#[cfg(not(feature = "bundled"))]
//use inquire::Confirm;

use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextProtocol;

pub fn monitor(app: &ArgMatches) -> Result<()> {
    let sub_args = match app.subcommand_matches("monitor") {
        Some(matches) => matches,
        None => {
            eprintln!("Error: Could not get subcommand matches");
            return Ok(());
        }
    };
    // Check .ntap directory
    match crate::sys::get_config_dir_path() {
        Some(_config_dir) => {}
        None => {
            let err_msg = "Could not get config directory path";
            tracing::error!("{err_msg}");
            anyhow::bail!(err_msg);
        }
    }

    // Check dependencies (Currently only for Windows)
    match crate::sys::check_deps() {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("Error: {:?}", e);
            anyhow::bail!(e.to_string());
        }
    }

    /* // Check Database files
    #[cfg(not(feature = "bundled"))]
    match crate::deps::check_db_files() {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("{}", e.to_string());
            let ans: bool =
                Confirm::new("ntap databases are missing. Do you want to download them now?")
                    .prompt()
                    .unwrap_or(false);
            if ans {
                crate::handler::update::download_db_files()?;
                println!("Please restart ntap");
                return Ok(());
            } else {
                return Err(e.to_string().into());
            }
        }
    } */

    // Load AppConfig
    let mut config = AppConfig::load();

    // Initialize logger
    crate::log::init_logger(&config)?;

    // Initialize DB
    crate::db::init_databases()?;

    if app.contains_id("tickrate") {
        config.display.tick_rate = *app.get_one("tickrate").unwrap_or(&1000);
    }

    // Interface filter
    if sub_args.contains_id("interfaces") {
        match sub_args.get_many::<String>("interfaces") {
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
    let mut ip_next_protocols: HashSet<IpNextProtocol> = HashSet::new();
    if sub_args.contains_id("protocols") {
        match sub_args.get_many::<String>("protocols") {
            Some(protocols_ref) => {
                let protocols: Vec<String> = protocols_ref.cloned().collect();
                for protocol in protocols {
                    if let Some(ethertype) = crate::net::packet::get_ethertype_from_str(&protocol) {
                        ethertypes.insert(ethertype);
                    }
                    if let Some(ip_next_protocol) =
                        crate::net::packet::get_ip_next_protocol_from_str(&protocol)
                    {
                        ip_next_protocols.insert(ip_next_protocol);
                    }
                }
            }
            None => {}
        }
    }

    // IP Address filter
    let ips: HashSet<IpAddr> = match sub_args.get_many::<IpAddr>("ips") {
        Some(ips_ref) => ips_ref.cloned().collect(),
        None => HashSet::new(),
    };

    // Port filter
    let ports: HashSet<u16> = match sub_args.get_many::<u16>("ports") {
        Some(ports_ref) => ports_ref.cloned().collect(),
        None => HashSet::new(),
    };

    if !ip_next_protocols.is_empty() || ips.len() > 0 || ports.len() > 0 {
        ethertypes.insert(EtherType::Ipv4);
        ethertypes.insert(EtherType::Ipv6);
        if ports.len() > 0 {
            ip_next_protocols.insert(IpNextProtocol::Tcp);
            ip_next_protocols.insert(IpNextProtocol::Udp);
        }
    }

    // Start threads
    let mut threads: Vec<thread::JoinHandle<()>> = vec![];

    let netstat_strage: Arc<NetStatStrage> = Arc::new(NetStatStrage::new());
    let mut netstat_strage_socket = Arc::clone(&netstat_strage);
    let mut netstat_strage_ui = Arc::clone(&netstat_strage);

    let target_interfaces: Vec<netdev::Interface>;
    if config.network.interfaces.is_empty() {
        target_interfaces = crate::net::interface::get_usable_interfaces();
    } else {
        target_interfaces =
            crate::net::interface::get_interfaces_by_name(&config.network.interfaces);
    }
    let mut pcap_thread_index = 0;
    let pcap_handlers = target_interfaces
        .iter()
        .map(|iface| {
            let mut netstat_strage_pcap = Arc::clone(&netstat_strage);
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
            let pcap_handler = pcap_thread.spawn(move || {
                crate::net::pcap::start_background_capture(
                    pcap_option,
                    &mut netstat_strage_pcap,
                    iface,
                );
            });
            tracing::info!("start thread {}", thread_name);
            pcap_thread_index += 1;
            pcap_handler
        })
        .collect::<Vec<_>>();

    let socket_handler = thread::spawn(move || {
        tracing::info!("start thread socket_info_update");
        crate::net::socket::start_socket_info_update(&mut netstat_strage_socket);
    });

    for pcap_handler in pcap_handlers {
        match pcap_handler {
            Ok(handle) => {
                threads.push(handle);
            }
            Err(e) => {
                tracing::error!("Error: {:?}", e);
            }
        }
    }
    threads.push(socket_handler);

    if config.network.reverse_dns {
        let mut netstat_strage_dns = Arc::clone(&netstat_strage);
        let dns_handler = thread::spawn(move || {
            tracing::info!("start thread dns_map_update");
            crate::net::dns::start_dns_map_update(&mut netstat_strage_dns);
        });
        threads.push(dns_handler);
    }

    tracing::info!("start TUI, netstat_data_update");

    crate::tui::monitor::terminal::run(
        config,
        app.contains_id("enhanced-graphics"),
        &mut netstat_strage_ui,
    )?;
    Ok(())
}
