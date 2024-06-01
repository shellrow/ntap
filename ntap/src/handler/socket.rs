use std::error::Error;

use comfy_table::presets::NOTHING;
use comfy_table::*;
use ntap_core::net::socket::SocketInfoOption;

pub fn show_socket_info() -> Result<(), Box<dyn Error>> {
    let sockets = ntap_core::net::socket::get_sockets_info(SocketInfoOption::default());
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Protocol", "Local Socket", "Remote Socket", "PID", "Process Name"]);
    for socket in sockets {
        table.add_row(vec![
            Cell::new(&socket.protocol.as_str()),
            Cell::new(format!("{}:{}", socket.local_ip_addr, socket.local_port)),
            if socket.remote_ip_addr.is_none() {
                Cell::new("")
            } else {
                Cell::new(format!("{}:{}", socket.remote_ip_addr.unwrap(), socket.remote_port.unwrap()))
            },
            if let Some(process) = &socket.process {
                Cell::new(&process.pid.to_string())
            } else {
                Cell::new("")
            },
            if let Some(process) = &socket.process {
                Cell::new(&process.name)
            } else {
                Cell::new("")
            },
        ]);
    }
    println!("{table}");
    Ok(())
}