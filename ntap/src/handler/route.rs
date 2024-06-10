use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};

use comfy_table::presets::NOTHING;
use comfy_table::*;
use netdev::interface::InterfaceType;

pub fn show_routes() -> Result<(), Box<dyn Error>> {
    let interfaces = netdev::get_interfaces();

    // IPv4 routing table
    println!("IPv4 Routing Table");
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Interface", "Source", "Destination", "Netmask", "Gateway"]);
    for iface in interfaces.clone() {
        if iface.ipv4.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            for ipv4 in &iface.ipv4 {
                let ipv4_gateway = if let Some(gw) = gateway.ipv4.get(0) {
                    gw.to_string()
                } else {
                    String::new()
                };
                if iface.default {
                    table.add_row(vec![
                        Cell::new(format!("{} (default)", &iface.name)),
                        Cell::new(&ipv4.addr),
                        Cell::new(Ipv4Addr::UNSPECIFIED),
                        Cell::new(&ipv4.netmask()),
                        Cell::new(ipv4_gateway),
                    ]);
                }else{
                    table.add_row(vec![
                        Cell::new(&iface.name),
                        Cell::new(&ipv4.addr),
                        Cell::new(&ipv4.network()),
                        Cell::new(&ipv4.netmask()),
                        Cell::new(ipv4_gateway),
                    ]);
                }
            }
        }else{
            if iface.if_type == InterfaceType::Loopback || iface.ipv4[0].addr == Ipv4Addr::LOCALHOST {
                table.add_row(vec![
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv4[0].addr),
                    Cell::new(&iface.ipv4[0].network()),
                    Cell::new(&iface.ipv4[0].netmask()),
                    Cell::new(""),
                ]);
            }
        }    
    }
    println!("{table}");

    // IPv6 routing table
    println!("IPv6 Routing Table");
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Interface", "Source", "Destination", "Netmask", "Gateway"]);
    for iface in interfaces {
        if iface.ipv6.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            for ipv6 in &iface.ipv6 {
                let ipv6_gateway = if let Some(gw) = gateway.ipv6.get(0) {
                    gw.to_string()
                } else {
                    String::new()
                };
                if iface.default {
                    table.add_row(vec![
                        Cell::new(format!("{} (default)", &iface.name)),
                        Cell::new(&ipv6.addr),
                        Cell::new(Ipv6Addr::UNSPECIFIED),
                        Cell::new(&ipv6.netmask()),
                        Cell::new(ipv6_gateway),
                    ]);
                }else{
                    table.add_row(vec![
                        Cell::new(&iface.name),
                        Cell::new(&ipv6.addr),
                        Cell::new(&ipv6.network()),
                        Cell::new(&ipv6.netmask()),
                        Cell::new(ipv6_gateway),
                    ]);
                }
            }
        }else{
            if iface.if_type == InterfaceType::Loopback || iface.ipv6[0].addr == Ipv6Addr::LOCALHOST {
                table.add_row(vec![
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv6[0].addr),
                    Cell::new(&iface.ipv6[0].network()),
                    Cell::new(&iface.ipv6[0].netmask()),
                    Cell::new(""),
                ]);
            }
        }
    }
    println!("{table}");
    Ok(())
}
