use std::collections::HashMap;
use std::net::IpAddr;
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

pub const MAX_CACHE_ENTRIES: usize = 4096;

async fn resolve_ip_async(resolver: &TokioResolver, ip: IpAddr) -> Option<String> {
    let timeout = if crate::net::ip::is_global_addr(ip) {
        Duration::from_secs(1)
    } else {
        Duration::from_millis(200)
    };
    let lookup = tokio::time::timeout(timeout, resolver.reverse_lookup(ip))
        .await
        .ok()?
        .ok()?;
    lookup
        .answers()
        .first()
        .map(|record| record.data.to_string().trim_end_matches('.').to_string())
}

pub async fn lookup_ips_async(ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
    let Some(resolver) = create_resolver(Some(Duration::from_secs(1))) else {
        return HashMap::new();
    };
    stream::iter(ips)
        .map(|ip| {
            let resolver = &resolver;
            async move { (ip, resolve_ip_async(resolver, ip).await) }
        })
        .buffer_unordered(10)
        .filter_map(|(ip, name)| async move { name.map(|name| (ip, name)) })
        .collect()
        .await
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
                if lookup_target_ips.len() >= 100 {
                    break;
                }
            }
        }
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);

        let dns_map = runtime.block_on(async {
            tokio::select! {
                result = lookup_ips_async(lookup_target_ips) => result,
                _ = async {
                    while !stop.load(std::sync::atomic::Ordering::Acquire) {
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                } => HashMap::new(),
            }
        });

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
            if reverse_dns_map_inner.len() < MAX_CACHE_ENTRIES
                || reverse_dns_map_inner.contains_key(&ip_addr)
            {
                reverse_dns_map_inner.insert(ip_addr, hostname);
            }
        }
        drop(remote_hosts_inner);
        drop(reverse_dns_map_inner);
        if crate::util::wait_for_stop(stop, std::time::Duration::from_secs(8)) {
            break;
        }
    }
}
