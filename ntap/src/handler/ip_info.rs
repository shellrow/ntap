use std::error::Error;

use comfy_table::presets::NOTHING;
use comfy_table::*;

pub fn show_public_ip_info() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let result : Result<(), Box<dyn Error>> = runtime.block_on(async {
        println!("Public IPv4 Info");
        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic);
        let ipv4_info = ntap_core::net::ip::get_self_ipv4_info().await?;
        if ipv4_info.ip_version != "v4" {
            return Err("Failed to get IPv4 info".into());
        }
        table.add_row(vec![
            Cell::new("IPv4 Address"),
            Cell::new(ipv4_info.ip_addr).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Country Code"),
            Cell::new(ipv4_info.country_code).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Country Name"),
            Cell::new(ipv4_info.country_name).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Network"),
            Cell::new(ipv4_info.network).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("ASN"),
            Cell::new(ipv4_info.asn).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("AS Name"),
            Cell::new(ipv4_info.as_name).set_alignment(CellAlignment::Left),
        ]);
        println!("{table}");
        println!("Public IPv6 Info");
        let mut table = Table::new();
        table
            .load_preset(NOTHING)
            .set_content_arrangement(ContentArrangement::Dynamic);
        let ipv6_info = ntap_core::net::ip::get_self_ip_info().await?;
        if ipv6_info.ip_version != "v6" {
            return Err("Failed to get IPv6 info".into());
        }
        table.add_row(vec![
            Cell::new("IPv6 Address"),
            Cell::new(ipv6_info.ip_addr).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Country Code"),
            Cell::new(ipv6_info.country_code).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Country Name"),
            Cell::new(ipv6_info.country_name).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("Network"),
            Cell::new(ipv6_info.network).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("ASN"),
            Cell::new(ipv6_info.asn).set_alignment(CellAlignment::Left),
        ]);
        table.add_row(vec![
            Cell::new("AS Name"),
            Cell::new(ipv6_info.as_name).set_alignment(CellAlignment::Left),
        ]);
        println!("{table}");
        Ok(())
    });
    result?;
    Ok(())
}
