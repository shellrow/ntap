use crate::config::AppConfig;
use crate::net::stat::NetStatStrage;
use crate::thread_log;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;
use std::thread;

use clap::ArgMatches;

#[cfg(not(feature = "bundle"))]
use inquire::Confirm;

pub fn monitor(app: &ArgMatches) -> Result<(), Box<dyn Error>> {
    // Check .ntap directory
    match crate::sys::get_config_dir_path() {
        Some(_config_dir) => {}
        None => {
            let err_msg = "Could not get config directory path";
            log::error!("{err_msg}");
            return Err(err_msg.into());
        }
    }

    // Check dependencies (Currently only for Windows)
    match crate::sys::check_deps() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {:?}", e);
            return Err(e);
        }
    }

    // Check Database files
    #[cfg(not(feature = "bundle"))]
    match crate::deps::check_db_files() {
        Ok(_) => {}
        Err(e) => {
            log::error!("{}", e.to_string());
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
    }

    // Load AppConfig
    let mut config = AppConfig::load();

    if app.contains_id("tickrate") {
        config.display.tick_rate = *app.get_one("tickrate").unwrap_or(&1000);
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

    let netstat_strage: Arc<NetStatStrage> = Arc::new(NetStatStrage::new());
    let mut netstat_strage_socket = Arc::clone(&netstat_strage);
    let mut netstat_strage_ui = Arc::clone(&netstat_strage);

    let usable_interfaces = crate::net::interface::get_usable_interfaces();
    let mut pcap_thread_index = 0;
    let pcap_handlers = usable_interfaces
        .iter()
        .map(|iface| {
            let mut netstat_strage_pcap = Arc::clone(&netstat_strage);
            let iface = iface.clone();
            let pcap_option = crate::net::pcap::PacketCaptureOptions::from_interface(&iface);
            let thread_name = format!("pcap-thread-{}", iface.name.clone());
            let pcap_thread = thread::Builder::new().name(thread_name.clone());
            let pcap_handler = pcap_thread.spawn(move || {
                if pcap_thread_index == 0 {
                    netstat_strage_pcap.load_ipdb();
                }
                crate::net::pcap::start_background_capture(
                    pcap_option,
                    &mut netstat_strage_pcap,
                    iface,
                );
            });
            thread_log!(info, "start thread {}", thread_name);
            pcap_thread_index += 1;
            pcap_handler
        })
        .collect::<Vec<_>>();

    let socket_handler = thread::spawn(move || {
        thread_log!(info, "start thread socket_info_update");
        crate::net::socket::start_socket_info_update(&mut netstat_strage_socket);
    });

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
    threads.push(socket_handler);

    if config.network.reverse_dns {
        let mut netstat_strage_dns = Arc::clone(&netstat_strage);
        let dns_handler = thread::spawn(move || {
            thread_log!(info, "start thread dns_map_update");
            crate::net::dns::start_dns_map_update(&mut netstat_strage_dns);
        });
        threads.push(dns_handler);
    }

    thread_log!(info, "start TUI, netstat_data_update");

    // Clear screen before starting TUI
    let mut stdout = std::io::stdout();
    crossterm::execute!(
        stdout,
        crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
    )?;
    // Move cursor to top left corner
    crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

    crate::tui::monitor::terminal::run(
        config,
        app.contains_id("enhanced-graphics"),
        &mut netstat_strage_ui,
    )?;
    Ok(())
}
