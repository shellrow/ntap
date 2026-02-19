use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};
use hickory_resolver::config::ResolverConfig;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::Resolver;

use crate::net::stat::NetStatStrage;

fn create_resolver(timeout: Option<Duration>) -> Option<Resolver<TokioConnectionProvider>> {
    #[cfg(any(unix, target_os = "windows"))]
    {
        let mut builder = Resolver::builder_tokio().ok()?;
        if let Some(timeout) = timeout {
            builder.options_mut().timeout = timeout;
        }
        Some(builder.build())
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        let mut builder =
            Resolver::builder_with_config(ResolverConfig::default(), TokioConnectionProvider::default());
        if let Some(timeout) = timeout {
            builder.options_mut().timeout = timeout;
        }
        Some(builder.build())
    }
}

pub fn lookup_host_name(host_name: String) -> Option<IpAddr> {
    let ip_vec: Vec<IpAddr> = resolve_domain(host_name);
    let mut ipv6_vec: Vec<IpAddr> = vec![];
    for ip in ip_vec {
        match ip {
            IpAddr::V4(_) => return Some(ip),
            IpAddr::V6(_) => ipv6_vec.push(ip),
        }
    }
    ipv6_vec.first().copied()
}

pub async fn lookup_host_name_async(host_name: String) -> Option<IpAddr> {
    let ip_vec: Vec<IpAddr> = resolve_domain_async(host_name).await;
    let mut ipv6_vec: Vec<IpAddr> = vec![];
    for ip in ip_vec {
        match ip {
            IpAddr::V4(_) => return Some(ip),
            IpAddr::V6(_) => ipv6_vec.push(ip),
        }
    }
    ipv6_vec.first().copied()
}

pub fn lookup_ip_addr(ip_addr: IpAddr) -> Option<String> {
    resolve_ip(ip_addr).first().cloned()
}

pub async fn lookup_ip_addr_async(ip_addr: String) -> String {
    resolve_ip_async(ip_addr)
        .await
        .first()
        .cloned()
        .unwrap_or_default()
}

fn resolve_domain(host_name: String) -> Vec<IpAddr> {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return vec![],
    };
    rt.block_on(resolve_domain_async(host_name))
}

fn resolve_ip(ip_addr: IpAddr) -> Vec<String> {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return vec![],
    };
    rt.block_on(resolve_ip_async(ip_addr.to_string()))
}

async fn resolve_domain_async(host_name: String) -> Vec<IpAddr> {
    let resolver = match create_resolver(None) {
        Some(resolver) => resolver,
        None => return vec![],
    };
    match resolver.lookup_ip(host_name).await {
        Ok(lookup) => lookup.iter().collect(),
        Err(_) => vec![],
    }
}

async fn resolve_ip_async(ip_addr: String) -> Vec<String> {
    let ip_addr = match IpAddr::from_str(&ip_addr) {
        Ok(ip_addr) => ip_addr,
        Err(_) => return vec![],
    };
    let timeout = if crate::net::ip::is_global_addr(ip_addr) {
        Duration::from_millis(1000)
    } else {
        Duration::from_millis(200)
    };
    let resolver = match create_resolver(Some(timeout)) {
        Some(resolver) => resolver,
        None => return vec![],
    };
    match resolver.reverse_lookup(ip_addr).await {
        Ok(lookup) => lookup
            .iter()
            .map(|name| name.to_string().trim_end_matches('.').to_string())
            .collect(),
        Err(_) => vec![],
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
    while let Some((ip, names)) = tasks.next().await {
        if let Some(name) = names.first() {
            if !name.is_empty() {
                results.insert(ip, name.clone());
            }
        }
    }
    results
}

pub fn lookup_ips(ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return HashMap::new(),
    };
    rt.block_on(lookup_ips_async(ips))
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
        let remote_hosts_inner = match netstat_strage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        let reverse_dns_map_inner = match netstat_strage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        for (ip_addr, remote_host) in remote_hosts_inner.iter() {
            // Best-effort reverse DNS only for unresolved hosts.
            if remote_host.hostname.is_empty() && !reverse_dns_map_inner.contains_key(ip_addr) {
                lookup_target_ips.push(*ip_addr);
            }
        }
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);

        let resolver = DnsResolver::new();
        let dns_map = resolver.lookup_ips(lookup_target_ips);

        let mut remote_hosts_inner = match netstat_strage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        let mut reverse_dns_map_inner = match netstat_strage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                continue;
            }
        };
        for (ip_addr, hostname) in dns_map {
            if hostname.is_empty() {
                continue;
            }
            if let Some(remote_host) = remote_hosts_inner.get_mut(&ip_addr) {
                if remote_host.hostname.is_empty() {
                    remote_host.hostname = hostname.clone();
                }
            }
            reverse_dns_map_inner.insert(ip_addr, hostname);
        }
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);
        std::thread::sleep(std::time::Duration::from_secs(8));
    }
}

pub struct DnsResolver {
    rt: tokio::runtime::Runtime,
}

impl DnsResolver {
    pub fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        DnsResolver { rt }
    }

    pub fn lookup_ips(&self, ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
        self.rt.block_on(lookup_ips_async(ips))
    }
}
