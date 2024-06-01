use ntap_core::net::stat::{NetStatData, NetStatStrage};
use ntap_core::thread_log;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tauri::Manager;

pub fn start_netstat_data_update(
    netstat_strage: &mut Arc<NetStatStrage>,
    netstat_data: &mut Arc<Mutex<NetStatData>>,
    interval: Duration,
) {
    let entry_ttl = Duration::from_millis(60000);
    let mut last_clear = Instant::now();
    loop {
        match netstat_data.lock() {
            Ok(mut data) => {
                if last_clear.elapsed() >= entry_ttl {
                    data.remove_old_entries(entry_ttl);
                    last_clear = Instant::now();
                }
                data.merge(netstat_strage.clone_data_and_reset(), interval);
            }
            Err(e) => {
                thread_log!(error, "Error: {:?}", e);
                continue;
            }
        }
        thread::sleep(interval);
    }
}

pub fn start_background_task(handle: &tauri::AppHandle) {
    let netstat_strage = handle.state::<Arc<NetStatStrage>>();
    let netstat_data = handle.state::<Arc<Mutex<NetStatData>>>();
    let thread_handles = handle.state::<Arc<Mutex<Vec<thread::JoinHandle<()>>>>>();
    // For socket info update
    let mut netstat_strage_socket = Arc::clone(&netstat_strage);
    // For DNS Map update
    let mut netstat_strage_dns = Arc::clone(&netstat_strage);
    // For Data update
    let mut netstat_strage_update = Arc::clone(&netstat_strage);
    let mut netstat_data_update = Arc::clone(&netstat_data);

    // Thread handle management
    let thread_handles = Arc::clone(&thread_handles);
    let mut thread_handles = thread_handles.lock().unwrap();
    //let mut threads: Vec<thread::JoinHandle<()>> = vec![];
    let usable_interfaces = ntap_core::net::interface::get_usable_interfaces();
    let mut pcap_thread_index = 0;
    let pcap_handlers = usable_interfaces
        .iter()
        .map(|iface| {
            let mut netstat_strage_pcap = Arc::clone(&netstat_strage);
            let iface = iface.clone();
            let pcap_option = ntap_core::net::pcap::PacketCaptureOptions::from_interface(&iface);
            let thread_name = format!("pcap-thread-{}", iface.name.clone());
            let pcap_thread =
                thread::Builder::new().name(thread_name.clone());
            let pcap_handler = pcap_thread.spawn(move || {
                ntap_core::net::pcap::start_background_capture(
                    pcap_option,
                    &mut netstat_strage_pcap,
                    iface,
                );
            });
            thread_log!(info, "start thread {:?}", thread_name);
            pcap_thread_index += 1;
            pcap_handler
        })
        .collect::<Vec<_>>();

    let socket_handler = thread::spawn(move || {
        thread_log!(info, "start thread socket_info_update");
        ntap_core::net::socket::start_socket_info_update(&mut netstat_strage_socket);
    });

    let dns_handler = thread::spawn(move || {
        thread_log!(info, "start thread dns_map_update");
        ntap_core::net::dns::start_dns_map_update(&mut netstat_strage_dns);
    });

    let update_handler = thread::spawn(move || {
        thread_log!(info, "start thread netstat_data_update");
        start_netstat_data_update(
            &mut netstat_strage_update,
            &mut netstat_data_update,
            Duration::from_secs(1),
        );
    });

    for pcap_handler in pcap_handlers {
        match pcap_handler {
            Ok(handle) => {
                thread_handles.push(handle);
            }
            Err(e) => {
                thread_log!(error, "Error: {:?}", e);
            }
        }
    }
    thread_handles.push(socket_handler);
    thread_handles.push(dns_handler);
    thread_handles.push(update_handler);
}
