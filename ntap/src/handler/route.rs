use std::error::Error;
use std::net::{Ipv4Addr, Ipv6Addr};

use comfy_table::presets::NOTHING;
use comfy_table::*;
use netdev::interface::InterfaceType;

pub fn show_routes() -> Result<(), Box<dyn Error>> {
    let interfaces = netdev::get_interfaces();

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Destination", "Netmask", "Gateway", "Interface", "Source"]);
    // IPv4
    for iface in interfaces.clone() {
        if iface.ipv4.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            if iface.default {
                table.add_row(vec![
                    Cell::new("default"),
                    Cell::new(Ipv4Addr::UNSPECIFIED),
                    Cell::new(gateway.ipv4[0]),
                    Cell::new(&iface.name),
                    Cell::new(&iface.ipv4[0].addr),
                ]);
            }else{
                table.add_row(vec![
                    Cell::new(&iface.ipv4[0].network()),
                    Cell::new(&iface.ipv4[0].netmask()),
                    Cell::new(gateway.ipv4[0]),
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv4[0].addr),
                ]);
            }
        }else{
            if iface.if_type == InterfaceType::Loopback {
                table.add_row(vec![
                    Cell::new(&iface.ipv4[0].network()),
                    Cell::new(&iface.ipv4[0].netmask()),
                    Cell::new(""),
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv4[0].addr),
                ]);
            }
        }    
    }
    // IPv6
    for iface in interfaces {
        if iface.ipv6.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            if iface.default {
                table.add_row(vec![
                    Cell::new("default"),
                    Cell::new(Ipv6Addr::UNSPECIFIED),
                    Cell::new(gateway.ipv6[0]),
                    Cell::new(&iface.name),
                    Cell::new(&iface.ipv6[0].addr),
                ]);
            }else{
                table.add_row(vec![
                    Cell::new(&iface.ipv6[0].network()),
                    Cell::new(&iface.ipv6[0].netmask()),
                    Cell::new(gateway.ipv6[0]),
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv6[0].addr),
                ]);
            }
        }else{
            if iface.if_type == InterfaceType::Loopback {
                table.add_row(vec![
                    Cell::new(&iface.ipv6[0].network()),
                    Cell::new(&iface.ipv6[0].netmask()),
                    Cell::new(""),
                    Cell::new(iface.name),
                    Cell::new(&iface.ipv6[0].addr),
                ]);
            }
        }
    }
    println!("{table}");
    Ok(())
}
