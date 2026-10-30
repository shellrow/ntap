use super::interface;
use super::{
    host::{HostDisplayInfo, RemoteHostInfo},
    packet::PacketFrame,
    traffic::{Direction, TrafficInfo},
};
use crate::net::socket::{
    AddressFamily, LocalSocket, SocketConnection, SocketDisplayInfo, SocketProcess,
    TransportProtocol,
};
use crate::process::{ProcessDisplayInfo, ProcessInfo};
use bytes::Bytes;
use hickory_resolver::proto::{op::Message, rr::RData};
use netdev::MacAddr;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

/// Bound per-interval and displayed host/connection cardinality under heavy traffic.
pub const MAX_TRACKED_ENTRIES: usize = 16_384;

fn record_connection(
    map: &mut HashMap<SocketConnection, TrafficInfo>,
    key: SocketConnection,
    direction: Direction,
    bytes: usize,
) {
    if map.len() < MAX_TRACKED_ENTRIES || map.contains_key(&key) {
        map.entry(key)
            .or_insert_with(TrafficInfo::new)
            .record(direction, bytes);
    }
}

#[derive(Debug, Clone)]
pub struct NetStatStorage {
    pub started_at: std::time::Instant,
    update_lock: Arc<Mutex<()>>,
    pub traffic: Arc<Mutex<TrafficInfo>>,
    /// Remote Host Traffic Info Map (IpAddr -> RemoteHostInfo)
    pub remote_hosts: Arc<Mutex<HashMap<IpAddr, RemoteHostInfo>>>,
    /// Socket Connection Traffic Map (SocketConnection -> TrafficInfo)
    pub connection_map: Arc<Mutex<HashMap<SocketConnection, TrafficInfo>>>,
    /// Socket Process Map (LocalSocket -> SocketProcess)
    pub local_socket_map: Arc<Mutex<HashMap<LocalSocket, SocketProcess>>>,
    /// Reverse DNS Map (IpAddr -> Hostname)
    pub reverse_dns_map: Arc<Mutex<HashMap<IpAddr, String>>>,
    /// Local IP Map (IpAddr -> Interface Name)
    pub local_ip_map: Arc<Mutex<HashMap<IpAddr, String>>>,
}

impl NetStatStorage {
    pub fn new() -> Self {
        let local_ip_map = interface::get_local_ip_map();
        NetStatStorage {
            started_at: std::time::Instant::now(),
            update_lock: Arc::new(Mutex::new(())),
            traffic: Arc::new(Mutex::new(TrafficInfo::new())),
            remote_hosts: Arc::new(Mutex::new(HashMap::new())),
            connection_map: Arc::new(Mutex::new(HashMap::new())),
            local_socket_map: Arc::new(Mutex::new(HashMap::new())),
            reverse_dns_map: Arc::new(Mutex::new(HashMap::new())),
            local_ip_map: Arc::new(Mutex::new(local_ip_map)),
        }
    }
    /// Get the local_socket_map (thread safe clone)
    pub fn get_local_socket_map(&self) -> HashMap<LocalSocket, SocketProcess> {
        match self.local_socket_map.lock() {
            Ok(local_socket_map) => local_socket_map.clone(),
            Err(e) => {
                tracing::error!("get_local_socket_map error: {:?}", e);
                HashMap::new()
            }
        }
    }
    pub fn get_local_ip_map(&self) -> HashMap<IpAddr, String> {
        match self.local_ip_map.lock() {
            Ok(local_ip_map) => local_ip_map.clone(),
            Err(e) => {
                tracing::error!("get_local_ip_map error: {:?}", e);
                HashMap::new()
            }
        }
    }
    pub fn take_data(&self) -> NetStatData {
        let _update_guard = match self.update_lock.lock() {
            Ok(guard) => guard,
            Err(error) => {
                tracing::error!("failed to lock statistics snapshot: {error}");
                error.into_inner()
            }
        };
        let mut clone: NetStatData = NetStatData::new();
        clone.traffic = std::mem::replace(
            &mut *self
                .traffic
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
            TrafficInfo::new(),
        );
        clone.remote_hosts = std::mem::take(
            &mut *self
                .remote_hosts
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
        clone.connection_map = std::mem::take(
            &mut *self
                .connection_map
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
        );
        clone.local_socket_map = self.get_local_socket_map();
        clone.local_ip_map = self.get_local_ip_map();
        clone
    }
    pub fn parse_dns_packet(&self, dns_packet: &Bytes) {
        let Ok(message) = Message::from_vec(dns_packet) else {
            return;
        };
        let Ok(mut cache) = self.reverse_dns_map.lock() else {
            return;
        };
        for answer in message.answers {
            let ip = match answer.data {
                RData::A(ip) => IpAddr::V4(ip.0),
                RData::AAAA(ip) => IpAddr::V6(ip.0),
                _ => continue,
            };
            if cache.len() >= crate::net::dns::MAX_CACHE_ENTRIES && !cache.contains_key(&ip) {
                continue;
            }
            cache.insert(ip, answer.name.to_utf8().trim_end_matches('.').to_string());
        }
    }
    pub fn update(&self, frame: PacketFrame) {
        let _update_guard = match self.update_lock.lock() {
            Ok(guard) => guard,
            Err(error) => {
                tracing::error!("failed to lock statistics update: {error}");
                error.into_inner()
            }
        };
        let local_ip_map_inner = match self.local_ip_map.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("Failed to lock local_ips: {:?}", e);
                return;
            }
        };
        // Lock traffic field
        let mut traffic_inner = match self.traffic.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("Failed to lock traffic: {:?}", e);
                return;
            }
        };
        // Lock remote_hosts field
        let mut remote_hosts_inner = match self.remote_hosts.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("Failed to lock remote_hosts: {:?}", e);
                return;
            }
        };
        // Lock connection_map field
        let mut connections_inner = match self.connection_map.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("Failed to lock connection_map: {:?}", e);
                return;
            }
        };
        let ethernet = frame
            .datalink
            .as_ref()
            .and_then(|layer| layer.ethernet.as_ref());
        let ip_layer = match frame.ip {
            Some(ip) => ip,
            None => return,
        };
        // Determine if the packet is incoming or outgoing.
        let direction: Direction = if let Some(ipv4) = &ip_layer.ipv4 {
            if local_ip_map_inner.contains_key(&IpAddr::V4(ipv4.source)) {
                Direction::Egress
            } else if local_ip_map_inner.contains_key(&IpAddr::V4(ipv4.destination)) {
                Direction::Ingress
            } else {
                return;
            }
        } else if let Some(ipv6) = &ip_layer.ipv6 {
            if local_ip_map_inner.contains_key(&IpAddr::V6(ipv6.source)) {
                Direction::Egress
            } else if local_ip_map_inner.contains_key(&IpAddr::V6(ipv6.destination)) {
                Direction::Ingress
            } else {
                return;
            }
        } else {
            return;
        };
        // Update TrafficInfo
        traffic_inner.record(direction, frame.packet_len);
        let mac_addr: String = match direction {
            Direction::Egress => {
                if let Some(ethernet) = ethernet {
                    ethernet.destination.address()
                } else {
                    MacAddr::zero().to_string()
                }
            }
            Direction::Ingress => {
                if let Some(ethernet) = ethernet {
                    ethernet.source.address()
                } else {
                    MacAddr::zero().to_string()
                }
            }
        };
        let local_ip_addr: IpAddr = match direction {
            Direction::Egress => {
                if let Some(ipv4) = &ip_layer.ipv4 {
                    IpAddr::V4(ipv4.source)
                } else if let Some(ipv6) = &ip_layer.ipv6 {
                    IpAddr::V6(ipv6.source)
                } else {
                    return;
                }
            }
            Direction::Ingress => {
                if let Some(ipv4) = &ip_layer.ipv4 {
                    IpAddr::V4(ipv4.destination)
                } else if let Some(ipv6) = &ip_layer.ipv6 {
                    IpAddr::V6(ipv6.destination)
                } else {
                    return;
                }
            }
        };
        let interface_name = frame.if_name.clone();
        let local_port: u16 = match direction {
            Direction::Egress => {
                if let Some(transport) = &frame.transport {
                    if let Some(tcp) = &transport.tcp {
                        tcp.source
                    } else if let Some(udp) = &transport.udp {
                        udp.source
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            Direction::Ingress => {
                if let Some(transport) = &frame.transport {
                    if let Some(tcp) = &transport.tcp {
                        tcp.destination
                    } else if let Some(udp) = &transport.udp {
                        udp.destination
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        };
        let remote_ip_addr: IpAddr = match direction {
            Direction::Egress => {
                if let Some(ipv4) = ip_layer.ipv4 {
                    IpAddr::V4(ipv4.destination)
                } else if let Some(ipv6) = ip_layer.ipv6 {
                    IpAddr::V6(ipv6.destination)
                } else {
                    return;
                }
            }
            Direction::Ingress => {
                if let Some(ipv4) = ip_layer.ipv4 {
                    IpAddr::V4(ipv4.source)
                } else if let Some(ipv6) = ip_layer.ipv6 {
                    IpAddr::V6(ipv6.source)
                } else {
                    return;
                }
            }
        };
        let remote_port: u16 = match direction {
            Direction::Egress => {
                if let Some(transport) = &frame.transport {
                    if let Some(tcp) = &transport.tcp {
                        tcp.destination
                    } else if let Some(udp) = &transport.udp {
                        udp.destination
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            Direction::Ingress => {
                if let Some(transport) = &frame.transport {
                    if let Some(tcp) = &transport.tcp {
                        tcp.source
                    } else if let Some(udp) = &transport.udp {
                        udp.source
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
        };
        // Update SocketConnection if the packet is TCP or UDP.
        if let Some(transport) = frame.transport {
            if let Some(_tcp) = transport.tcp {
                let socket_connection: SocketConnection = SocketConnection {
                    interface_name: interface_name.clone(),
                    local_ip_addr,
                    local_port,
                    remote_ip_addr,
                    remote_port,
                    protocol: TransportProtocol::TCP,
                };
                record_connection(
                    &mut connections_inner,
                    socket_connection,
                    direction,
                    frame.packet_len,
                );
            }
            if let Some(_udp) = transport.udp {
                let socket_connection: SocketConnection = SocketConnection {
                    interface_name,
                    local_ip_addr,
                    local_port,
                    remote_ip_addr,
                    remote_port,
                    protocol: TransportProtocol::UDP,
                };
                record_connection(
                    &mut connections_inner,
                    socket_connection,
                    direction,
                    frame.packet_len,
                );
                // Try parse DNS packet
                if remote_port == 53 {
                    self.parse_dns_packet(&frame.payload);
                }
            }
        }
        let reverse_dns_map_inner = match self.reverse_dns_map.lock() {
            Ok(inner) => inner,
            Err(e) => {
                tracing::error!("Failed to lock reverse_dns_map: {:?}", e);
                return;
            }
        };
        // Check Reverse DNS Map
        let mut hostname = String::new();
        if let Some(name) = reverse_dns_map_inner.get(&remote_ip_addr) {
            hostname = name.to_string();
        }
        // Update or Insert RemoteHostInfo
        if remote_hosts_inner.len() >= MAX_TRACKED_ENTRIES
            && !remote_hosts_inner.contains_key(&remote_ip_addr)
        {
            return;
        }
        let remote_host: &mut RemoteHostInfo = remote_hosts_inner
            .entry(remote_ip_addr)
            .or_insert_with(|| RemoteHostInfo::new(mac_addr, remote_ip_addr, hostname.clone()));
        remote_host.traffic_info.record(direction, frame.packet_len);
        if remote_host.hostname.is_empty() && !hostname.is_empty() {
            remote_host.hostname = hostname.clone();
        }

        // Drop the locks
        drop(traffic_inner);
        drop(remote_hosts_inner);
        drop(connections_inner);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetStatData {
    pub traffic: TrafficInfo,
    pub remote_hosts: HashMap<IpAddr, RemoteHostInfo>,
    pub connection_map: HashMap<SocketConnection, TrafficInfo>,
    pub local_socket_map: HashMap<LocalSocket, SocketProcess>,
    pub local_ip_map: HashMap<IpAddr, String>,
}

impl NetStatData {
    pub fn new() -> Self {
        NetStatData {
            traffic: TrafficInfo::new(),
            remote_hosts: HashMap::new(),
            connection_map: HashMap::new(),
            local_socket_map: HashMap::new(),
            local_ip_map: HashMap::new(),
        }
    }
    // merge using entry method to merge traffic info.
    pub fn merge(&mut self, other: NetStatData, duration: Duration) {
        // Update Traffic Info
        self.traffic.update_bytes_per_sec(&other.traffic, duration);
        self.traffic.add_traffic(&other.traffic);
        // Reset idle rates before applying this interval's observations.
        let idle = TrafficInfo::new();
        for host in self.remote_hosts.values_mut() {
            host.traffic_info.update_bytes_per_sec(&idle, duration);
        }
        for traffic in self.connection_map.values_mut() {
            traffic.update_bytes_per_sec(&idle, duration);
        }
        // Keep aggregate totals even when detail cardinality reaches the limit.
        let mut host_capacity = MAX_TRACKED_ENTRIES.saturating_sub(self.remote_hosts.len());
        let mut connection_capacity = MAX_TRACKED_ENTRIES.saturating_sub(self.connection_map.len());
        // Update RemoteHostInfo
        other
            .remote_hosts
            .iter()
            .for_each(|(ip, host)| match self.remote_hosts.entry(*ip) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                    let host_entry = entry.get_mut();
                    host_entry.merge(host, duration);
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    if host_capacity == 0 {
                        return;
                    }
                    host_capacity -= 1;
                    let mut host = host.clone();
                    host.traffic_info
                        .update_bytes_per_sec(&host.traffic_info.clone(), duration);
                    entry.insert(host);
                }
            });
        // Update SocketConnection Traffic Info
        other
            .connection_map
            .iter()
            .for_each(
                |(conn, traffic_info)| match self.connection_map.entry(conn.clone()) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let traffic_info_entry = entry.get_mut();
                        traffic_info_entry.update_bytes_per_sec(traffic_info, duration);
                        traffic_info_entry.add_traffic(traffic_info);
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        if connection_capacity == 0 {
                            return;
                        }
                        connection_capacity -= 1;
                        let mut traffic = traffic_info.clone();
                        traffic.update_bytes_per_sec(traffic_info, duration);
                        entry.insert(traffic);
                    }
                },
            );
        // Socket snapshots replace stale process ownership and closed sockets.
        self.local_socket_map = other.local_socket_map;
        // Update local_ip_map
        self.local_ip_map = other.local_ip_map;
    }

    // Remove old entries from remote_hosts, connection_map, local_socket_map
    // TrafficInfo.last_seen is used to determine if the entry is old.
    // If the entry is older than ttl, it will be removed.
    pub fn remove_old_entries(&mut self, ttl: Duration) {
        let now = std::time::SystemTime::now();
        self.remote_hosts.retain(|_, host| {
            now.duration_since(host.traffic_info.last_seen)
                .unwrap_or_default()
                <= ttl
        });
        self.connection_map
            .retain(|_, traffic| now.duration_since(traffic.last_seen).unwrap_or_default() <= ttl);
    }

    pub fn get_remote_hosts(&self, limit: Option<usize>) -> Vec<HostDisplayInfo> {
        // Create a map to store the traffic info for each remote host.
        let mut host_traffic_map: HashMap<IpAddr, usize> = HashMap::new();
        self.remote_hosts.iter().for_each(|(_ip, host)| {
            match host_traffic_map.get(&host.ip_addr) {
                Some(traffic) => {
                    let traffic = traffic
                        .saturating_add(host.traffic_info.bytes_sent)
                        .saturating_add(host.traffic_info.bytes_received);
                    host_traffic_map.insert(host.ip_addr, traffic);
                }
                None => {
                    host_traffic_map.insert(
                        host.ip_addr,
                        host.traffic_info
                            .bytes_sent
                            .saturating_add(host.traffic_info.bytes_received),
                    );
                }
            }
        });
        let mut host_traffic_vec: Vec<(&IpAddr, &usize)> = host_traffic_map.iter().collect();
        host_traffic_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let mut remote_hosts: Vec<HostDisplayInfo> = Vec::new();
        // limit : if limit is None, return all remote hosts.
        for (ip, _) in host_traffic_vec
            .iter()
            .take(limit.unwrap_or(host_traffic_vec.len()))
        {
            if let Some(host) = self.remote_hosts.get(ip) {
                let host = HostDisplayInfo {
                    ip_addr: host.ip_addr,
                    hostname: host.hostname.clone(),
                    country_code: String::from("N/A"),
                    as_name: String::from("N/A"),
                    traffic: host.traffic_info.to_display_info(),
                };
                remote_hosts.push(host);
            }
        }
        remote_hosts
    }

    fn socket_process(&self, socket: &LocalSocket) -> Option<&SocketProcess> {
        self.local_socket_map.get(socket).or_else(|| {
            let wildcard = LocalSocket::new(
                match socket.local_ip_addr {
                    IpAddr::V4(_) => IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
                    IpAddr::V6(_) => IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED),
                },
                socket.port,
                socket.protocol,
            );
            self.local_socket_map.get(&wildcard)
        })
    }

    pub fn get_processes(&self, limit: Option<usize>) -> Vec<ProcessDisplayInfo> {
        let mut process_traffic_map: HashMap<u32, TrafficInfo> = HashMap::new();
        let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
        self.connection_map.iter().for_each(|(conn, traffic_info)| {
            let local_socket: LocalSocket = LocalSocket {
                local_ip_addr: conn.local_ip_addr,
                port: conn.local_port,
                protocol: conn.protocol,
            };
            if let Some(socket_process) = self.socket_process(&local_socket)
                && let Some(process) = &socket_process.process
            {
                match process_traffic_map.get(&process.pid) {
                    Some(traffic) => {
                        let mut traffic = traffic.clone();
                        traffic.add_traffic(traffic_info);
                        traffic.egress_packets_per_sec = traffic
                            .egress_packets_per_sec
                            .saturating_add(traffic_info.egress_packets_per_sec);
                        traffic.ingress_packets_per_sec = traffic
                            .ingress_packets_per_sec
                            .saturating_add(traffic_info.ingress_packets_per_sec);
                        traffic.egress_bytes_per_sec = traffic
                            .egress_bytes_per_sec
                            .saturating_add(traffic_info.egress_bytes_per_sec);
                        traffic.ingress_bytes_per_sec = traffic
                            .ingress_bytes_per_sec
                            .saturating_add(traffic_info.ingress_bytes_per_sec);
                        process_traffic_map.insert(process.pid, traffic);
                    }
                    None => {
                        process_traffic_map.insert(process.pid, traffic_info.clone());
                    }
                }
                process_map.insert(process.pid, process.clone());
            }
        });
        // Create process total traffic map from process_traffic_map
        let process_total_traffic_map: HashMap<u32, usize> = process_traffic_map
            .iter()
            .map(|(pid, traffic)| (*pid, traffic.total_bytes()))
            .collect();
        // Sort process_total_traffic_map by traffic
        let mut process_total_traffic_vec: Vec<(&u32, &usize)> =
            process_total_traffic_map.iter().collect();
        process_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        // Create top processes from process_total_traffic_vec
        let mut top_processes: Vec<ProcessDisplayInfo> = Vec::new();
        // limit : if limit is None, return all processes.
        for (pid, _) in process_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(process_total_traffic_vec.len()))
        {
            if let Some(traffic) = process_traffic_map.get(pid)
                && let Some(process) = process_map.get(pid)
            {
                let process = ProcessDisplayInfo {
                    pid: process.pid,
                    name: process.name.clone(),
                    traffic: traffic.to_display_info(),
                };
                top_processes.push(process);
            }
        }
        top_processes
    }

    pub fn get_connections(&self, limit: Option<usize>) -> Vec<SocketDisplayInfo> {
        let connection_total_traffic_map: HashMap<SocketConnection, usize> = self
            .connection_map
            .iter()
            .map(|(conn, traffic)| (conn.clone(), traffic.total_bytes()))
            .collect();
        let mut connection_total_traffic_vec: Vec<(&SocketConnection, &usize)> =
            connection_total_traffic_map.iter().collect();
        connection_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let mut top_connections: Vec<SocketDisplayInfo> = Vec::new();
        // limit : if limit is None, return all connections.
        for (conn, _) in connection_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(connection_total_traffic_vec.len()))
        {
            // Get process info from local_socket_map
            let process: Option<ProcessInfo> = match self.socket_process(&LocalSocket {
                local_ip_addr: conn.local_ip_addr,
                port: conn.local_port,
                protocol: conn.protocol,
            }) {
                Some(socket_process) => socket_process.process.clone(),
                None => None,
            };
            if let Some(traffic) = self.connection_map.get(conn) {
                let socket_traffic_info = SocketDisplayInfo {
                    interface_name: conn.interface_name.clone(),
                    local_ip_addr: conn.local_ip_addr,
                    local_port: conn.local_port,
                    remote_ip_addr: Some(conn.remote_ip_addr),
                    remote_port: Some(conn.remote_port),
                    protocol: conn.protocol,
                    ip_version: match conn.remote_ip_addr {
                        IpAddr::V4(_) => AddressFamily::IPv4,
                        IpAddr::V6(_) => AddressFamily::IPv6,
                    },
                    traffic: traffic.to_display_info(),
                    process,
                };
                top_connections.push(socket_traffic_info);
            }
        }
        top_connections
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn observation(bytes: usize) -> NetStatData {
        let mut data = NetStatData::new();
        let ip = "192.0.2.1".parse().unwrap();
        let mut host = RemoteHostInfo::new(String::new(), ip, String::new());
        host.traffic_info.record(Direction::Ingress, bytes);
        data.traffic.record(Direction::Ingress, bytes);
        data.remote_hosts.insert(ip, host);
        data
    }
    #[test]
    fn first_interval_sets_rates_and_idle_interval_clears_them() {
        let mut data = NetStatData::new();
        data.merge(observation(1000), Duration::from_millis(500));
        let ip: IpAddr = "192.0.2.1".parse().unwrap();
        assert_eq!(
            data.remote_hosts[&ip].traffic_info.ingress_bytes_per_sec,
            2000
        );
        data.merge(NetStatData::new(), Duration::from_secs(1));
        assert_eq!(data.remote_hosts[&ip].traffic_info.ingress_bytes_per_sec, 0);
        assert_eq!(data.remote_hosts[&ip].traffic_info.bytes_received, 1000);
    }
    #[test]
    fn snapshots_preserve_socket_ownership_between_polls() {
        let storage = NetStatStorage::new();
        let key = LocalSocket::new("192.0.2.1".parse().unwrap(), 80, TransportProtocol::TCP);
        storage
            .local_socket_map
            .lock()
            .unwrap()
            .insert(key.clone(), SocketProcess::new());
        assert!(storage.take_data().local_socket_map.contains_key(&key));
        assert!(storage.take_data().local_socket_map.contains_key(&key));
    }
    #[test]
    fn closed_socket_ownership_is_not_retained_forever() {
        let mut data = NetStatData::new();
        data.local_socket_map.insert(
            LocalSocket::new("192.0.2.1".parse().unwrap(), 80, TransportProtocol::TCP),
            SocketProcess::new(),
        );
        data.merge(NetStatData::new(), Duration::from_secs(1));
        assert!(data.local_socket_map.is_empty());
    }
    #[test]
    fn raw_ip_packets_are_counted_without_an_ethernet_header() {
        let storage = NetStatStorage::new();
        let local: IpAddr = "192.0.2.1".parse().unwrap();
        storage
            .local_ip_map
            .lock()
            .unwrap()
            .insert(local, "tun-test".into());
        let mut bytes = vec![0; 28];
        bytes[0] = 0x45;
        bytes[3] = 28;
        bytes[8] = 64;
        bytes[9] = 17;
        bytes[12..16].copy_from_slice(&[192, 0, 2, 1]);
        bytes[16..20].copy_from_slice(&[198, 51, 100, 1]);
        bytes[24..26].copy_from_slice(&8u16.to_be_bytes());
        let frame = nex::packet::frame::Frame::from_buf(
            &bytes,
            nex::packet::frame::ParseOption {
                from_ip_packet: true,
                offset: 0,
            },
        )
        .unwrap();
        storage.update(PacketFrame::from_nex_frame(1, 1, "tun-test".into(), frame));
        let data = storage.take_data();
        assert_eq!(data.traffic.packet_sent, 1);
        assert_eq!(data.traffic.bytes_sent, 28);
        assert_eq!(data.connection_map.len(), 1);
    }
    #[test]
    fn wildcard_sockets_resolve_without_conflating_address_families() {
        let mut data = NetStatData::new();
        data.local_socket_map.insert(
            LocalSocket::new("0.0.0.0".parse().unwrap(), 80, TransportProtocol::TCP),
            SocketProcess::new(),
        );
        assert!(
            data.socket_process(&LocalSocket::new(
                "192.0.2.1".parse().unwrap(),
                80,
                TransportProtocol::TCP
            ))
            .is_some()
        );
        assert!(
            data.socket_process(&LocalSocket::new(
                "::1".parse().unwrap(),
                80,
                TransportProtocol::TCP
            ))
            .is_none()
        );
    }

    #[test]
    fn dns_cache_keeps_each_answer_with_its_own_name() {
        // A compressed DNS response containing two A records for example.com.
        let mut bytes = vec![0, 1, 0x81, 0x80, 0, 1, 0, 2, 0, 0, 0, 0];
        bytes.extend_from_slice(b"\x07example\x03com\x00\x00\x01\x00\x01");
        for ip in [[192, 0, 2, 1], [192, 0, 2, 2]] {
            bytes.extend_from_slice(&[0xc0, 0x0c, 0, 1, 0, 1, 0, 0, 0, 60, 0, 4]);
            bytes.extend_from_slice(&ip);
        }
        let storage = NetStatStorage::new();
        storage.parse_dns_packet(&Bytes::from(bytes));
        let cache = storage.reverse_dns_map.lock().unwrap();
        assert_eq!(cache.len(), 2);
        assert!(cache.values().all(|name| name == "example.com"));
    }
}
