use ntap_core::net::host::HostDisplayInfo;
use ntap_core::net::packet::PacketFrame;
use ntap_core::net::stat::NetStatData;
use ntap_core::net::stat::Overview;
use ntap_core::net::pcap::CaptureReport;
use ntap_core::process::ProcessDisplayInfo;
use ntap_core::net::socket::SocketDisplayInfo;
use ntap_core::net::socket::SocketInfoOption;
use ntap_core::net::stat::NetStatStrage;
use crate::app::AppInfo;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::{Manager, State};

#[tauri::command]
pub async fn start_background_task(handle: tauri::AppHandle) -> Result<(), String> {
    // Load IPDB
    log::info!("Loading IPDB...");
    match handle.emit_all("init", "Loading IPDB...") {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
    let netstat_strage = handle.state::<Arc<NetStatStrage>>();
    netstat_strage.load_ipdb();
    log::info!("Starting background task...");
    match handle.emit_all("init", "Starting background task...") {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
    crate::task::start_background_task(&handle);
    match handle.emit_all("init", "Init complete") {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub fn get_deps_map() -> HashMap<String, bool> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub fn get_deps_map() -> HashMap<String, bool> {
    ntap_core::deps::get_deps_map()
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn download_dep(_app_handle: tauri::AppHandle, _software_name: String) -> Result<u64, String> {
    Ok(0)
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn download_dep(app_handle: tauri::AppHandle, software_name: String) -> Result<u64, String> {
    if software_name.to_lowercase() == ntap_core::deps::NPCAP_SOFTWARE_NAME.to_lowercase() {
        match crate::sys::download_npcap(&app_handle).await {
            Ok(content_length) => {
                Ok(content_length)
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }else{
        return Err("Unknown dependency".into());
    }
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
pub async fn run_dep_installer(_software_name: String) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
#[tauri::command]
pub async fn run_dep_installer(software_name: String) -> Result<(), String> {
    if software_name.to_lowercase() == ntap_core::deps::NPCAP_SOFTWARE_NAME.to_lowercase() {
        // Run npcap installer with admin privilege
        match crate::sys::run_npcap_installer() {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }else{
        return Err("Unknown dependency".into());
    }
}

#[tauri::command]
pub async fn start_packet_capture(app_handle: tauri::AppHandle) -> CaptureReport {
    let mut report = CaptureReport::new();
    let (tx, rx): (Sender<PacketFrame>, Receiver<PacketFrame>) = channel();
    let stop = Arc::new(Mutex::new(false));
    let stop_handle = stop.clone();
    let default_interface = netdev::get_default_interface().unwrap();
    let pcap_option = ntap_core::net::pcap::PacketCaptureOptions::from_interface(&default_interface);
    let pacp_handler = thread::spawn(move || {
        ntap_core::net::pcap::start_capture(pcap_option, tx, &stop, default_interface)
    });
    let stop_pcap_event = app_handle.listen_global("stop_pcap", move |event| {
        log::info!("got stop_pcap with payload {:?}", event.payload());
        match stop_handle.lock() {
            Ok(mut stop) => {
                *stop = true;
            }
            Err(e) => {
                log::error!("Error: {:?}", e);
            }
        }
    });
    let print_handler = thread::spawn(move || {
        while let Ok(frame) = rx.recv() {
            match app_handle.emit_all("packet_frame", frame) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error: {:?}", e);
                }
            }
        }
        app_handle.unlisten(stop_pcap_event);
    });
    match pacp_handler.join() {
        Ok(r) => {
            log::info!("pacp_handler: {:?}", r);
            report = r;
        }
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
    match print_handler.join() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Error: {:?}", e);
        }
    }
    report
}

#[tauri::command]
pub fn get_netstat(
    netstat: State<'_, Arc<Mutex<NetStatData>>>,
    opt: SocketInfoOption,
) -> Vec<SocketDisplayInfo> {
    match netstat.lock() {
        Ok(data) => data.get_connections_with_opt(None, opt),
        Err(e) => {
            log::error!("Error: {:?}", e);
            vec![]
        }
    }
}

#[tauri::command]
pub fn get_remote_hosts(netstat: State<'_, Arc<Mutex<NetStatData>>>) -> Vec<HostDisplayInfo> {
    match netstat.lock() {
        Ok(data) => data.get_remote_hosts(None),
        Err(e) => {
            log::error!("Error: {:?}", e);
            vec![]
        }
    }
}

#[tauri::command]
pub fn get_process_info(netstat: State<'_, Arc<Mutex<NetStatData>>>) -> Vec<ProcessDisplayInfo> {
    match netstat.lock() {
        Ok(data) => data.get_processes(None),
        Err(e) => {
            log::error!("Error: {:?}", e);
            vec![]
        }
    }
}

#[tauri::command]
pub fn get_overview(netstat: State<'_, Arc<Mutex<NetStatData>>>) -> Overview {
    match netstat.lock() {
        Ok(data) => data.get_overview(),
        Err(e) => {
            log::error!("Error: {:?}", e);
            Overview::new()
        }
    }
}

#[tauri::command]
pub async fn get_self_ip_info() -> Result<ipstruct::ipinfo::IpInfo, String> {
    ntap_core::net::ip::get_self_ip_info()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_self_ipv4_info() -> Result<ipstruct::ipinfo::IpInfo, String> {
    ntap_core::net::ip::get_self_ipv4_info()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_default_interface() -> Result<netdev::Interface, String> {
    netdev::get_default_interface().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_interfaces() -> Vec<netdev::Interface> {
    netdev::get_interfaces()
}

#[tauri::command]
pub async fn get_app_info() -> AppInfo {
    AppInfo::new()
}

#[tauri::command]
pub async fn get_app_config() -> ntap_core::config::AppConfig {
    ntap_core::config::AppConfig::load()
}

#[tauri::command]
pub async fn save_app_config(config: ntap_core::config::AppConfig) {
    config.save();
}

#[tauri::command]
pub async fn get_config_dir() -> String {
    ntap_core::sys::get_config_dir_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "".to_owned())
}

#[tauri::command]
pub async fn get_database_config() -> ntap_core::config::DatabaseConfig {
    ntap_core::config::DatabaseConfig::new()
}

#[tauri::command]
pub async fn get_routes() -> Vec<crate::route::Route> {
    crate::route::get_routes()
}
