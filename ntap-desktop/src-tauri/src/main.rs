// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod route;
mod sys;
mod task;
mod app;

use commands::{
    start_background_task, get_deps_map, download_dep, run_dep_installer, get_default_interface, get_netstat, get_overview, get_process_info, get_remote_hosts,
    get_self_ip_info, get_self_ipv4_info, start_packet_capture, get_app_info, get_app_config, 
    save_app_config, get_interfaces, get_config_dir, get_database_config, get_routes
};
use ntap_core::net::stat::{NetStatData, NetStatStrage};
use std::sync::{Arc, Mutex};

fn main() {
    //let netstat_strage: Arc<Mutex<NetStatStrage>> = Arc::new(Mutex::new(NetStatStrage::new()));
    let netstat_strage: Arc<NetStatStrage> = Arc::new(NetStatStrage::new());
    let netstat_data: Arc<Mutex<NetStatData>> = Arc::new(Mutex::new(NetStatData::new()));
    let thread_handles: Arc<Mutex<Vec<std::thread::JoinHandle<()>>>> = Arc::new(Mutex::new(Vec::new()));
    tauri::Builder::default()
        .manage(netstat_strage)
        .manage(netstat_data)
        .manage(thread_handles)
        .invoke_handler(tauri::generate_handler![
            start_background_task,
            get_deps_map,
            download_dep,
            run_dep_installer,
            get_overview,
            get_remote_hosts,
            get_netstat,
            get_process_info,
            start_packet_capture,
            get_self_ip_info,
            get_self_ipv4_info,
            get_default_interface,
            get_app_info,
            get_app_config,
            save_app_config,
            get_interfaces,
            get_config_dir,
            get_database_config,
            get_routes
        ])
        .setup(|app| {
            let app_handle = app.handle();
            match sys::init(&app_handle) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("Error: {:?}", e);
                    std::process::exit(1);
                }
            }
            Ok(())
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::Destroyed => {
                sys::cleanup();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
