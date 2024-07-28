use std::error::Error;

use crate::net::socket::{AddressFamily, SocketInfoOption, TransportProtocol};
use clap::ArgMatches;
use comfy_table::presets::NOTHING;
use comfy_table::*;
use nex::packet::ethernet::EtherType;
use nex::packet::ip::IpNextLevelProtocol;

pub fn show_socket_info(app: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let sub_args = match app.subcommand_matches("socket") {
        Some(matches) => matches,
        None => {
            eprintln!("Error: Could not get subcommand matches");
            return Ok(());
        }
    };
    let mut sock_opt: SocketInfoOption = SocketInfoOption::default();
    if sub_args.contains_id("protocols") {
        sock_opt.address_family = Vec::new();
        sock_opt.transport_protocol = Vec::new();
        match sub_args.get_many::<String>("protocols") {
            Some(protocols_ref) => {
                let protocols: Vec<String> = protocols_ref.cloned().collect();
                for protocol in protocols {
                    if let Some(ethertype) = crate::net::packet::get_ethertype_from_str(&protocol) {
                        // IPv4 or IPv6
                        match ethertype {
                            EtherType::Ipv4 => {
                                sock_opt.address_family.push(AddressFamily::IPv4);
                            }
                            EtherType::Ipv6 => {
                                sock_opt.address_family.push(AddressFamily::IPv6);
                            }
                            _ => {}
                        }
                    }
                    if let Some(ip_next_protocol) =
                        crate::net::packet::get_ip_next_protocol_from_str(&protocol)
                    {
                        // TCP or UDP
                        match ip_next_protocol {
                            IpNextLevelProtocol::Tcp => {
                                sock_opt.transport_protocol.push(TransportProtocol::TCP);
                            }
                            IpNextLevelProtocol::Udp => {
                                sock_opt.transport_protocol.push(TransportProtocol::UDP);
                            }
                            _ => {}
                        }
                    }
                }
            }
            None => {}
        }
    }
    if sock_opt.address_family.is_empty() {
        sock_opt.address_family = vec![AddressFamily::IPv4, AddressFamily::IPv6];
    }
    if sock_opt.transport_protocol.is_empty() {
        sock_opt.transport_protocol = vec![TransportProtocol::TCP, TransportProtocol::UDP];
    }

    let sockets = crate::net::socket::get_sockets_info(sock_opt);
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            "Protocol",
            "Local Socket",
            "Remote Socket",
            "PID",
            "Process Name",
        ]);
    for socket in sockets {
        table.add_row(vec![
            Cell::new(&socket.protocol.as_str()),
            Cell::new(format!("{}:{}", socket.local_ip_addr, socket.local_port)),
            if socket.remote_ip_addr.is_none() {
                Cell::new("")
            } else {
                Cell::new(format!(
                    "{}:{}",
                    socket.remote_ip_addr.unwrap(),
                    socket.remote_port.unwrap()
                ))
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
