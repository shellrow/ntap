use crate::util::tree::node_label;
use std::error::Error;
use termtree::Tree;

pub fn show_public_ip_info() -> Result<(), Box<dyn Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    let result: Result<(), Box<dyn Error>> = runtime.block_on(async {
        let ipv4_info = crate::net::ip::get_self_ipv4_info().await?;
        let ipv6_info = crate::net::ip::get_self_ip_info().await?;
        let mut tree = Tree::new(node_label("Public IP Info", None, None));
        if ipv4_info.ip_version == "v4" {
            let mut ipv4_tree = Tree::new(node_label("IPv4 Info", None, None));
            ipv4_tree.push(node_label("IPv4 Address", Some(&ipv4_info.ip_addr), None));
            ipv4_tree.push(node_label(
                "IPv4 Address Decimal",
                Some(&ipv4_info.ip_addr_dec),
                None,
            ));
            ipv4_tree.push(node_label(
                "Country Code",
                Some(&ipv4_info.country_code),
                None,
            ));
            ipv4_tree.push(node_label(
                "Country Name",
                Some(&ipv4_info.country_name),
                None,
            ));
            ipv4_tree.push(node_label("Network", Some(&ipv4_info.network), None));
            ipv4_tree.push(node_label("ASN", Some(&ipv4_info.asn), None));
            ipv4_tree.push(node_label("AS Name", Some(&ipv4_info.as_name), None));
            tree.push(ipv4_tree);
        } else {
            tree.push(node_label(
                "IPv4 Info",
                Some("Failed to get IPv4 info"),
                None,
            ));
        }
        if ipv6_info.ip_version == "v6" {
            let mut ipv6_tree = Tree::new(node_label("IPv6 Info", None, None));
            ipv6_tree.push(node_label("IPv6 Address", Some(&ipv6_info.ip_addr), None));
            ipv6_tree.push(node_label(
                "IPv6 Address Decimal",
                Some(&ipv6_info.ip_addr_dec),
                None,
            ));
            ipv6_tree.push(node_label(
                "Country Code",
                Some(&ipv6_info.country_code),
                None,
            ));
            ipv6_tree.push(node_label(
                "Country Name",
                Some(&ipv6_info.country_name),
                None,
            ));
            ipv6_tree.push(node_label("Network", Some(&ipv6_info.network), None));
            ipv6_tree.push(node_label("ASN", Some(&ipv6_info.asn), None));
            ipv6_tree.push(node_label("AS Name", Some(&ipv6_info.as_name), None));
            tree.push(ipv6_tree);
        } else {
            tree.push(node_label(
                "IPv6 Info",
                Some("Failed to get IPv6 info"),
                None,
            ));
        }
        println!("{}", tree);
        Ok(())
    });
    result?;
    Ok(())
}
