use crate::config::AppConfig;
use crate::net::packet::{PacketFrame, PacketStorage};
use crate::thread_log;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};

use clap::ArgMatches;

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
            let pcap_option = crate::net::pcap::PacketCaptureOptions::from_interface(&iface);
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
