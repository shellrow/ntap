use anyhow::{Result, bail};
use nex::net::interface::Interface;
use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
};

pub fn get_local_ip_map() -> HashMap<IpAddr, String> {
    let mut ip_map = HashMap::new();
    for interface in nex::net::interface::get_interfaces() {
        for ip in &interface.ipv4 {
            ip_map.insert(IpAddr::V4(ip.addr()), interface.name.clone());
        }
        for ip in &interface.ipv6 {
            ip_map.insert(IpAddr::V6(ip.addr()), interface.name.clone());
        }
    }
    ip_map
}

fn is_usable(interface: &Interface) -> bool {
    interface.is_up() && (!interface.ipv4.is_empty() || !interface.ipv6.is_empty())
}

pub fn resolve_capture_interfaces(names: &[String]) -> Result<Vec<Interface>> {
    let available = nex::net::interface::get_interfaces();
    if names.is_empty() {
        let interfaces: Vec<Interface> = available.into_iter().filter(is_usable).collect();
        if interfaces.is_empty() {
            bail!("no usable capture interfaces were found");
        }
        Ok(interfaces)
    } else {
        let interfaces: Vec<Interface> = available
            .into_iter()
            .filter(|interface| names.contains(&interface.name) && is_usable(interface))
            .collect();
        let resolved: HashSet<&str> = interfaces
            .iter()
            .map(|interface| interface.name.as_str())
            .collect();
        let missing: Vec<&str> = names
            .iter()
            .map(String::as_str)
            .filter(|name| !resolved.contains(name))
            .collect();
        if !missing.is_empty() {
            bail!(
                "unknown or unavailable interface(s): {}",
                missing.join(", ")
            );
        }
        Ok(interfaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_name_of_an_unavailable_interface() {
        let name = "ntap-interface-that-does-not-exist".to_string();
        let error = resolve_capture_interfaces(std::slice::from_ref(&name))
            .expect_err("a fabricated interface must be rejected");
        assert!(error.to_string().contains(&name));
    }
}
