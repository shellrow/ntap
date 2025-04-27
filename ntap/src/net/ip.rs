use ipstruct::{client::Client, ipinfo::IpInfo, setting::ClientSetting};
use nex::net::ip::{Ipv4Net, Ipv6Net};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub fn get_network_address(ip_addr: IpAddr) -> Result<String, String> {
    match ip_addr {
        IpAddr::V4(ipv4_addr) => {
            let net: Ipv4Net = Ipv4Net::new(ipv4_addr, 24).map_err(|e| {
                format!("Invalid IPv4 prefix length : {}", e.to_string())
            })?;
            Ok(net.network().to_string())
        }
        IpAddr::V6(ipv6_addr) => {
            let net: Ipv6Net = Ipv6Net::new(ipv6_addr, 24).map_err(|e| {
                format!("Invalid IPv6 prefix length: {}", e.to_string())
            })?;
            Ok(net.network().to_string())
        }
    }
}

pub fn is_global_addr(ip_addr: IpAddr) -> bool {
    match ip_addr {
        IpAddr::V4(ipv4) => nex::net::ip::is_global_ipv4(&ipv4),
        IpAddr::V6(ipv6) => nex::net::ip::is_global_ipv6(&ipv6),
    }
}

pub fn in_same_network(src_ip: IpAddr, dst_ip: IpAddr) -> bool {
    let src_ip_nw = match get_network_address(src_ip) {
        Ok(nw) => nw,
        Err(_) => return false,
    };
    let dst_ip_nw = match get_network_address(dst_ip) {
        Ok(nw) => nw,
        Err(_) => return false,
    };
    if src_ip_nw == dst_ip_nw {
        true
    } else {
        false
    }
}

pub fn guess_initial_ttl(ttl: u8) -> u8 {
    if ttl <= 64 {
        64
    } else if 64 < ttl && ttl <= 128 {
        128
    } else {
        255
    }
}

pub fn ipv4_to_int(ipv4: Ipv4Addr) -> u64 {
    //let ipv4: Ipv4Addr = ip_addr.parse().unwrap_or(Ipv4Addr::LOCALHOST);
    let o1: u64 = ipv4.octets()[0].into();
    let o2: u64 = ipv4.octets()[1].into();
    let o3: u64 = ipv4.octets()[2].into();
    let o4: u64 = ipv4.octets()[3].into();
    let ip_int: u64 = ((o1 << 24) + (o2 << 16) + (o3 << 8) + o4).into();
    return ip_int;
}

pub fn ipv6_to_dec(ipv6: Ipv6Addr) -> u128 {
    //let ipv6: Ipv6Addr = ip_addr.parse().unwrap_or(Ipv6Addr::LOCALHOST);
    let segments: [u16; 8] = ipv6.segments();
    let mut ip_int: u128 = 0;
    for i in 0..ipv6.segments().len() {
        let cur_seg: u128 = segments[(ipv6.segments().len() - 1) - i].into();
        ip_int = (cur_seg << i * 16) | ip_int;
    }
    return ip_int;
}

pub async fn get_self_ip_info() -> Result<IpInfo, Box<dyn std::error::Error>> {
    let setting: ClientSetting = ClientSetting::default();
    let client: Client = Client::new(setting).unwrap();
    client.get_self_ip_info().await
}

pub async fn get_self_ipv4_info() -> Result<IpInfo, Box<dyn std::error::Error>> {
    let setting: ClientSetting = ClientSetting::default();
    let client: Client = Client::new(setting).unwrap();
    client.get_self_ipv4_info().await
}
