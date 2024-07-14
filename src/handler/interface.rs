use std::error::Error;

use comfy_table::presets::NOTHING;
use comfy_table::*;
use netdev::mac::MacAddr;

pub fn show_interfaces() -> Result<(), Box<dyn Error>> {
    let interfaces = netdev::get_interfaces();

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["#", "Key", "Value"]);

    for iface in interfaces {
        table.add_row(vec![
            Cell::new(&iface.index),
            Cell::new("Name"),
            Cell::new(&iface.name).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("Friendly Name"),
            Cell::new(&iface.friendly_name.unwrap_or("".to_string()))
                .set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("Type"),
            Cell::new(&iface.if_type.name()).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("MAC"),
            Cell::new(&iface.mac_addr.unwrap_or(MacAddr::zero()))
                .set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("IPv4"),
            Cell::new(format!("{:?}", &iface.ipv4)).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("IPv6"),
            Cell::new(format!("{:?}", &iface.ipv6)).set_alignment(CellAlignment::Left),
        ]);
        if let Some(gateway) = &iface.gateway {
            table.add_row(vec![Cell::new(""), Cell::new("Gateway"), Cell::new("")]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("MAC").set_alignment(CellAlignment::Right),
                Cell::new(format!("{}", &gateway.mac_addr)).set_alignment(CellAlignment::Left),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("IPv4").set_alignment(CellAlignment::Right),
                Cell::new(format!("{:?}", &gateway.ipv4)).set_alignment(CellAlignment::Left),
            ]);
            table.add_row(vec![
                Cell::new(""),
                Cell::new("IPv6").set_alignment(CellAlignment::Right),
                Cell::new(format!("{:?}", &gateway.ipv6)).set_alignment(CellAlignment::Left),
            ]);
        }
    }
    println!("{table}");
    Ok(())
}

pub fn show_default_interface() -> Result<(), Box<dyn Error>> {
    let iface = netdev::get_default_interface()?;
    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["#", "Key", "Value"]);
    table.add_row(vec![
        Cell::new(&iface.index),
        Cell::new("Name"),
        Cell::new(&iface.name).set_alignment(CellAlignment::Left),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Friendly Name"),
        Cell::new(&iface.friendly_name.unwrap_or("".to_string()))
            .set_alignment(CellAlignment::Left),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("Type"),
        Cell::new(&iface.if_type.name()).set_alignment(CellAlignment::Left),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("MAC"),
        Cell::new(&iface.mac_addr.unwrap_or(MacAddr::zero())).set_alignment(CellAlignment::Left),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("IPv4"),
        Cell::new(format!("{:?}", &iface.ipv4)).set_alignment(CellAlignment::Left),
    ]);
    table.add_row(vec![
        Cell::new(""),
        Cell::new("IPv6"),
        Cell::new(format!("{:?}", &iface.ipv6)).set_alignment(CellAlignment::Left),
    ]);
    if let Some(gateway) = &iface.gateway {
        table.add_row(vec![Cell::new(""), Cell::new("Gateway"), Cell::new("")]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("MAC").set_alignment(CellAlignment::Right),
            Cell::new(format!("{}", &gateway.mac_addr)).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("IPv4").set_alignment(CellAlignment::Right),
            Cell::new(format!("{:?}", &gateway.ipv4)).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new(""),
            Cell::new("IPv6").set_alignment(CellAlignment::Right),
            Cell::new(format!("{:?}", &gateway.ipv6)).set_alignment(CellAlignment::Left),
        ]);
    }
    println!("{table}");
    Ok(())
}
