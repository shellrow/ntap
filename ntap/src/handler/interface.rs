use crate::util::tree::node_label;
use netdev::mac::MacAddr;
use netdev::Interface;
use std::error::Error;
use termtree::Tree;

pub fn show_interfaces() -> Result<(), Box<dyn Error>> {
    let interfaces: Vec<Interface> = netdev::get_interfaces();

    let mut tree = Tree::new(node_label("Interfaces", None, None));
    for iface in interfaces {
        let mut iface_tree = Tree::new(node_label(&iface.name, None, None));
        iface_tree.push(node_label("Index", Some(&iface.index.to_string()), None));
        iface_tree.push(node_label("Name", Some(&iface.name), None));
        if let Some(friendly_name) = &iface.friendly_name {
            iface_tree.push(node_label("Friendly Name", Some(friendly_name), None));
        }
        if let Some(desc) = &iface.description {
            iface_tree.push(node_label("Description", Some(desc), None));
        }
        iface_tree.push(node_label("Type", Some(&iface.if_type.name()), None));
        iface_tree.push(node_label(
            "MAC",
            Some(&iface.mac_addr.unwrap_or(MacAddr::zero()).to_string()),
            None,
        ));
        let mut ipv4_tree = Tree::new(node_label("IPv4 Addresses", None, None));
        for ipv4 in &iface.ipv4 {
            ipv4_tree.push(node_label(&ipv4.addr.to_string(), None, None));
        }
        iface_tree.push(ipv4_tree);

        let mut ipv6_tree = Tree::new(node_label("IPv6 Addresses", None, None));
        for ipv6 in &iface.ipv6 {
            ipv6_tree.push(node_label(&ipv6.addr.to_string(), None, None));
        }
        iface_tree.push(ipv6_tree);

        if let Some(gateway) = &iface.gateway {
            let mut gateway_tree = Tree::new(node_label("Gateway", None, None));
            gateway_tree.push(node_label("MAC", Some(&gateway.mac_addr.to_string()), None));
            let mut ipv4_tree = Tree::new(node_label("IPv4 Addresses", None, None));
            for ipv4 in &gateway.ipv4 {
                ipv4_tree.push(node_label(&ipv4.to_string(), None, None));
            }
            gateway_tree.push(ipv4_tree);
            let mut ipv6_tree = Tree::new(node_label("IPv6 Addresses", None, None));
            for ipv6 in &gateway.ipv6 {
                ipv6_tree.push(node_label(&ipv6.to_string(), None, None));
            }
            gateway_tree.push(ipv6_tree);
            iface_tree.push(gateway_tree);
        }

        if iface.dns_servers.len() > 0 {
            let mut dns_tree = Tree::new(node_label("DNS Servers", None, None));
            for server_addr in &iface.dns_servers {
                dns_tree.push(node_label(&server_addr.to_string(), None, None));
            }
            iface_tree.push(dns_tree);
        }
        tree.push(iface_tree);
    }
    println!("{}", tree);
    Ok(())
}

pub fn show_default_interface() -> Result<(), Box<dyn Error>> {
    let iface: Interface = netdev::get_default_interface()?;
    let mut tree = Tree::new(node_label("Interface", None, None));
    tree.push(node_label("Index", Some(&iface.index.to_string()), None));
    tree.push(node_label("Name", Some(&iface.name), None));
    if let Some(friendly_name) = &iface.friendly_name {
        tree.push(node_label("Friendly Name", Some(friendly_name), None));
    }
    if let Some(desc) = &iface.description {
        tree.push(node_label("Description", Some(desc), None));
    }
    tree.push(node_label("Type", Some(&iface.if_type.name()), None));
    tree.push(node_label(
        "MAC",
        Some(&iface.mac_addr.unwrap_or(MacAddr::zero()).to_string()),
        None,
    ));
    let mut ipv4_tree = Tree::new(node_label("IPv4 Addresses", None, None));
    for ipv4 in &iface.ipv4 {
        ipv4_tree.push(node_label(&ipv4.addr.to_string(), None, None));
    }
    tree.push(ipv4_tree);

    let mut ipv6_tree = Tree::new(node_label("IPv6 Addresses", None, None));
    for ipv6 in &iface.ipv6 {
        ipv6_tree.push(node_label(&ipv6.addr.to_string(), None, None));
    }
    tree.push(ipv6_tree);

    if let Some(gateway) = &iface.gateway {
        let mut gateway_tree = Tree::new(node_label("Gateway", None, None));
        gateway_tree.push(node_label("MAC", Some(&gateway.mac_addr.to_string()), None));
        let mut ipv4_tree = Tree::new(node_label("IPv4 Addresses", None, None));
        for ipv4 in &gateway.ipv4 {
            ipv4_tree.push(node_label(&ipv4.to_string(), None, None));
        }
        gateway_tree.push(ipv4_tree);
        let mut ipv6_tree = Tree::new(node_label("IPv6 Addresses", None, None));
        for ipv6 in &gateway.ipv6 {
            ipv6_tree.push(node_label(&ipv6.to_string(), None, None));
        }
        gateway_tree.push(ipv6_tree);
        tree.push(gateway_tree);
    }
    if iface.dns_servers.len() > 0 {
        let mut dns_tree = Tree::new(node_label("DNS Servers", None, None));
        for server_addr in &iface.dns_servers {
            dns_tree.push(node_label(&server_addr.to_string(), None, None));
        }
        tree.push(dns_tree);
    }

    println!("{}", tree);
    Ok(())
}
