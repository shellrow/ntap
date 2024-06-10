use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Route {
    pub interface_name: String,
    pub source: IpAddr,
    pub destination: IpAddr,
    pub netmask: IpAddr,
    pub gateway: IpAddr,
}

pub fn get_routes() -> Vec<Route> {
    let mut routes = Vec::new();
    let interfaces = netdev::get_interfaces();
    // IPv4 routing table
    for iface in interfaces.clone() {
        if iface.ipv4.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            for ipv4 in &iface.ipv4 {
                if iface.default {
                    routes.push(Route {
                        interface_name: format!("{} (default)", iface.name),
                        source: IpAddr::V4(ipv4.addr),
                        destination: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                        netmask: IpAddr::V4(ipv4.netmask()),
                        gateway: IpAddr::V4(*gateway.ipv4.first().unwrap_or(&Ipv4Addr::UNSPECIFIED)),
                    });
                } else {
                    routes.push(Route {
                        interface_name: iface.name.clone(),
                        source: IpAddr::V4(ipv4.addr.clone()),
                        destination: IpAddr::V4(ipv4.network()),
                        netmask: IpAddr::V4(ipv4.netmask()),
                        gateway: IpAddr::V4(*gateway.ipv4.first().unwrap_or(&Ipv4Addr::UNSPECIFIED)),
                    });
                }
            }
        } else {
            if iface.if_type == netdev::interface::InterfaceType::Loopback
                || iface.ipv4[0].addr == Ipv4Addr::LOCALHOST
            {
                routes.push(Route {
                    interface_name: iface.name,
                    source: IpAddr::V4(iface.ipv4[0].addr),
                    destination: IpAddr::V4(iface.ipv4[0].network()),
                    netmask: IpAddr::V4(iface.ipv4[0].netmask()),
                    gateway: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                });
            }
        }
    }
    // IPv6 routing table
    for iface in interfaces {
        if iface.ipv6.len() == 0 {
            continue;
        }
        if let Some(gateway) = iface.gateway {
            for ipv6 in &iface.ipv6 {
                if iface.default {
                    routes.push(Route {
                        interface_name: format!("{} (default)", iface.name),
                        source: IpAddr::V6(ipv6.addr),
                        destination: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                        netmask: IpAddr::V6(ipv6.netmask()),
                        gateway: IpAddr::V6(*gateway.ipv6.first().unwrap_or(&Ipv6Addr::UNSPECIFIED)),
                    });
                } else {
                    routes.push(Route {
                        interface_name: iface.name.clone(),
                        source: IpAddr::V6(ipv6.addr.clone()),
                        destination: IpAddr::V6(ipv6.network()),
                        netmask: IpAddr::V6(ipv6.netmask()),
                        gateway: IpAddr::V6(*gateway.ipv6.first().unwrap_or(&Ipv6Addr::UNSPECIFIED)),
                    });
                }
            }
        } else {
            if iface.if_type == netdev::interface::InterfaceType::Loopback
                || iface.ipv6[0].addr == Ipv6Addr::LOCALHOST
            {
                routes.push(Route {
                    interface_name: iface.name,
                    source: IpAddr::V6(iface.ipv6[0].addr),
                    destination: IpAddr::V6(iface.ipv6[0].network()),
                    netmask: IpAddr::V6(iface.ipv6[0].netmask()),
                    gateway: IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                });
            }
        }
    }
    routes
}
