use crate::thread_log;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

#[cfg(not(any(unix, target_os = "windows")))]
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::Resolver;

use futures::stream::{self, StreamExt};

use hickory_resolver::AsyncResolver;
use std::collections::HashMap;
use std::str::FromStr;
use std::thread;

use crate::net::stat::NetStatStrage;

pub fn lookup_host_name(host_name: String) -> Option<IpAddr> {
    let ip_vec: Vec<IpAddr> = resolve_domain(host_name);
    let mut ipv6_vec: Vec<IpAddr> = vec![];
    for ip in ip_vec {
        match ip {
            IpAddr::V4(_) => {
                return Some(ip);
            }
            IpAddr::V6(_) => {
                ipv6_vec.push(ip);
            }
        }
    }
    if ipv6_vec.len() > 0 {
        return Some(ipv6_vec[0]);
    } else {
        None
    }
}

pub async fn lookup_host_name_async(host_name: String) -> Option<IpAddr> {
    let ip_vec: Vec<IpAddr> = resolve_domain_async(host_name).await;
    let mut ipv6_vec: Vec<IpAddr> = vec![];
    for ip in ip_vec {
        match ip {
            IpAddr::V4(_) => {
                return Some(ip);
            }
            IpAddr::V6(_) => {
                ipv6_vec.push(ip);
            }
        }
    }
    if ipv6_vec.len() > 0 {
        return Some(ipv6_vec[0]);
    } else {
        None
    }
}

pub fn lookup_ip_addr(ip_addr: IpAddr) -> Option<String> {
    let names: Vec<String> = resolve_ip(ip_addr);
    if names.len() > 0 {
        return Some(names[0].clone());
    } else {
        return None;
    }
}

pub async fn lookup_ip_addr_async(ip_addr: String) -> String {
    let ips: Vec<String> = resolve_ip_async(ip_addr).await;
    if ips.len() > 0 {
        return ips[0].clone();
    } else {
        return String::new();
    }
}

#[cfg(any(unix, target_os = "windows"))]
fn resolve_domain(host_name: String) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = vec![];
    let resolver = Resolver::from_system_conf().unwrap();
    match resolver.lookup_ip(host_name) {
        Ok(lip) => {
            for ip in lip.iter() {
                ips.push(ip);
            }
        }
        Err(_) => {}
    }
    ips
}

#[cfg(not(any(unix, target_os = "windows")))]
fn resolve_domain(host_name: String) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = vec![];
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    match resolver.lookup_ip(host_name) {
        Ok(lip) => {
            for ip in lip.iter() {
                ips.push(ip);
            }
        }
        Err(_) => {}
    }
    ips
}

#[cfg(any(unix, target_os = "windows"))]
fn resolve_ip(ip_addr: IpAddr) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    let mut system_conf = hickory_resolver::system_conf::read_system_conf().unwrap();
    if crate::net::ip::is_global_addr(ip_addr) {
        system_conf.1.timeout = Duration::from_millis(1000);
    } else {
        system_conf.1.timeout = Duration::from_millis(200);
    }
    let resolver = Resolver::new(system_conf.0, system_conf.1).unwrap();
    match resolver.reverse_lookup(ip_addr) {
        Ok(rlookup) => {
            for record in rlookup.as_lookup().record_iter() {
                match record.data() {
                    Some(data) => {
                        let name = data.to_string();
                        if name.ends_with(".") {
                            names.push(name[0..name.len() - 1].to_string());
                        } else {
                            names.push(name);
                        }
                    }
                    None => {}
                }
            }
            names
        }
        Err(_) => {
            return names;
        }
    }
}

#[cfg(not(any(unix, target_os = "windows")))]
fn resolve_ip(ip_addr: IpAddr) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    match resolver.reverse_lookup(ip_addr) {
        Ok(rlookup) => {
            for record in rlookup.as_lookup().record_iter() {
                match record.data() {
                    Some(data) => {
                        let name = data.to_string();
                        if name.ends_with(".") {
                            names.push(name[0..name.len() - 1].to_string());
                        } else {
                            names.push(name);
                        }
                    }
                    None => {}
                }
            }
            names
        }
        Err(_) => {
            return names;
        }
    }
}

#[cfg(any(unix, target_os = "windows"))]
async fn resolve_domain_async(host_name: String) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = vec![];
    let resolver = AsyncResolver::tokio_from_system_conf().unwrap();
    match resolver.lookup_ip(host_name).await {
        Ok(lip) => {
            for ip in lip.iter() {
                ips.push(ip);
            }
        }
        Err(_) => {}
    }
    ips
}

#[cfg(not(any(unix, target_os = "windows")))]
async fn resolve_domain_async(host_name: String) -> Vec<IpAddr> {
    let mut ips: Vec<IpAddr> = vec![];
    let resolver =
        AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    match resolver.lookup_ip(host_name).await {
        Ok(lip) => {
            for ip in lip.iter() {
                ips.push(ip);
            }
        }
        Err(_) => {}
    }
    ips
}

#[cfg(any(unix, target_os = "windows"))]
async fn resolve_ip_async(ip_addr: String) -> Vec<String> {
    let ip_addr: IpAddr = IpAddr::from_str(ip_addr.as_str()).unwrap();
    let mut names: Vec<String> = vec![];
    let mut system_conf = hickory_resolver::system_conf::read_system_conf().unwrap();
    if crate::net::ip::is_global_addr(ip_addr) {
        system_conf.1.timeout = Duration::from_millis(1000);
    } else {
        system_conf.1.timeout = Duration::from_millis(200);
    }
    let resolver = AsyncResolver::tokio(system_conf.0, system_conf.1);
    match resolver.reverse_lookup(ip_addr).await {
        Ok(rlookup) => {
            for record in rlookup.as_lookup().record_iter() {
                match record.data() {
                    Some(data) => {
                        let name = data.to_string();
                        if name.ends_with(".") {
                            names.push(name[0..name.len() - 1].to_string());
                        } else {
                            names.push(name);
                        }
                    }
                    None => {}
                }
            }
            names
        }
        Err(_) => {
            return names;
        }
    }
}

#[cfg(not(any(unix, target_os = "windows")))]
async fn resolve_ip_async(ip_addr: String) -> Vec<String> {
    let mut names: Vec<String> = vec![];
    let resolver =
        AsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    match resolver
        .reverse_lookup(IpAddr::from_str(ip_addr.as_str()).unwrap())
        .await
    {
        Ok(rlookup) => {
            for record in rlookup.as_lookup().record_iter() {
                match record.data() {
                    Some(data) => {
                        let name = data.to_string();
                        if name.ends_with(".") {
                            names.push(name[0..name.len() - 1].to_string());
                        } else {
                            names.push(name);
                        }
                    }
                    None => {}
                }
            }
            names
        }
        Err(_) => {
            return names;
        }
    }
}

pub async fn lookup_ips_async(ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
    let mut tasks = stream::iter(ips)
        .map(|ip| async move {
            let names = resolve_ip_async(ip.to_string()).await;
            (ip, names)
        })
        .buffer_unordered(10);
    let mut results: HashMap<IpAddr, String> = HashMap::new();
    while let Some(result) = tasks.next().await {
        results.insert(
            result.0,
            result.1.first().unwrap_or(&String::new()).to_string(),
        );
    }
    results
}

pub fn lookup_ips(ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
    let handle = thread::spawn(move || rt.block_on(async { lookup_ips_async(ips).await }));
    handle.join().unwrap()
}

pub fn lookup_host(host: String) -> Vec<IpAddr> {
    resolve_domain(host)
}

pub fn lookup_addr(addr: IpAddr) -> Vec<String> {
    resolve_ip(addr)
}

pub fn start_dns_map_update(netstat_strage: &mut Arc<NetStatStrage>) {
    loop {
        let mut lookup_target_ips: Vec<IpAddr> = vec![];
        // Lock the remote_hosts
        let remote_hosts_inner = match netstat_strage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                thread_log!(error, "[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        // Lock the reverse_dns_map
        let reverse_dns_map_inner = match netstat_strage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                thread_log!(error, "[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        for (ip_addr, _remote_host) in remote_hosts_inner.iter() {
            if !reverse_dns_map_inner.contains_key(ip_addr) {
                lookup_target_ips.push(*ip_addr);
            }
        }
        // Drop the lock before calling lookup_ips
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);
        let mut resolver = DnsResolver::new();
        let dns_map = resolver.lookup_ips(lookup_target_ips);
        // Lock the remote_hosts
        let mut remote_hosts_inner = match netstat_strage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                thread_log!(error, "[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        // Lock the reverse_dns_map
        let mut reverse_dns_map_inner = match netstat_strage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                thread_log!(error, "[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        for (ip_addr, hostname) in dns_map {
            if let Some(remote_host) = remote_hosts_inner.get_mut(&ip_addr) {
                remote_host.hostname = hostname.clone();
            }
            reverse_dns_map_inner.insert(ip_addr, hostname);
        }
        // Drop the lock
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);
        std::thread::sleep(std::time::Duration::from_secs(8));
    }
}

pub struct DnsResolver {
    //pub dns_map: HashMap<IpAddr, String>,
}

impl DnsResolver {
    pub fn new() -> Self {
        DnsResolver {
            //dns_map: HashMap::new(),
        }
    }
    pub fn lookup_ips(&mut self, ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
        let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();
        let handle = thread::spawn(move || rt.block_on(async { lookup_ips_async(ips).await }));
        handle.join().unwrap()
    }
}
