use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use futures::stream::{self, StreamExt};
use hickory_resolver::TokioResolver;

use crate::net::stat::NetStatStorage;

fn create_resolver(timeout: Option<Duration>) -> Option<TokioResolver> {
    #[cfg(any(unix, target_os = "windows"))]
    {
        let mut builder = TokioResolver::builder_tokio().ok()?;
        if let Some(timeout) = timeout {
            builder.options_mut().timeout = timeout;
        }
        builder.build().ok()
    }
    #[cfg(not(any(unix, target_os = "windows")))]
    {
        let _ = timeout;
        None
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
            .answers()
            .iter()
            .map(|record| record.data.to_string().trim_end_matches('.').to_string())
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
        if let Some(name) = names.first()
            && !name.is_empty()
        {
            results.insert(ip, name.clone());
        }
    }
    results
}

pub fn start_dns_map_update(
    netstat_storage: &mut Arc<NetStatStorage>,
    stop: &std::sync::atomic::AtomicBool,
) {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!("failed to create reverse DNS runtime: {error}");
            return;
        }
    };
    loop {
        if stop.load(std::sync::atomic::Ordering::Acquire) {
            break;
        }
        let mut lookup_target_ips: Vec<IpAddr> = vec![];
        let remote_hosts_inner = match netstat_storage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                if crate::util::wait_for_stop(stop, std::time::Duration::from_millis(25)) {
                    break;
                }
                continue;
            }
        };
        let reverse_dns_map_inner = match netstat_storage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                if crate::util::wait_for_stop(stop, std::time::Duration::from_millis(25)) {
                    break;
                }
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

        let dns_map = runtime.block_on(lookup_ips_async(lookup_target_ips));

        let mut remote_hosts_inner = match netstat_storage.remote_hosts.try_lock() {
            Ok(remote_hosts) => remote_hosts,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                if crate::util::wait_for_stop(stop, std::time::Duration::from_millis(25)) {
                    break;
                }
                continue;
            }
        };
        let mut reverse_dns_map_inner = match netstat_storage.reverse_dns_map.try_lock() {
            Ok(reverse_dns_map) => reverse_dns_map,
            Err(e) => {
                tracing::error!("[dns_map_update] lock error: {}", e);
                if crate::util::wait_for_stop(stop, std::time::Duration::from_millis(25)) {
                    break;
                }
                continue;
            }
        };
        for (ip_addr, hostname) in dns_map {
            if hostname.is_empty() {
                continue;
            }
            if let Some(remote_host) = remote_hosts_inner.get_mut(&ip_addr)
                && remote_host.hostname.is_empty()
            {
                remote_host.hostname = hostname.clone();
            }
            reverse_dns_map_inner.insert(ip_addr, hostname);
        }
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);
        if crate::util::wait_for_stop(stop, std::time::Duration::from_secs(8)) {
            break;
        }
    }
}
