use super::interface;
use super::{
    host::{HostDisplayInfo, RemoteHostInfo},
    packet::PacketFrame,
    service::ServiceDisplayInfo,
    traffic::{Direction, TrafficDisplayInfo, TrafficInfo},
};
use crate::net::socket::{
    AddressFamily, LocalSocket, ProtocolPort, SocketConnection, SocketDisplayInfo,
    SocketInfoOption, SocketProcess, TransportProtocol,
};
use crate::process::{ProcessDisplayInfo, ProcessInfo};
use bytes::Bytes;
use netdev::{Interface, MacAddr};
use nex::packet::dns::{DnsPacket, DnsType};
use nex::packet::packet::Packet;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[derive(Debug, Clone)]
pub struct NetStatStrage {
    pub interface: Arc<Mutex<Interface>>,
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

impl NetStatStrage {
    pub fn new() -> Self {
        let default_interface = match netdev::get_default_interface() {
            Ok(iface) => iface,
            Err(e) => {
                tracing::error!("NetStatStrage get_default_interface error: {:?}", e);
                Interface::dummy()
            }
        };
        let local_ip_map = interface::get_local_ip_map();
        NetStatStrage {
            interface: Arc::new(Mutex::new(default_interface)),
            traffic: Arc::new(Mutex::new(TrafficInfo::new())),
            remote_hosts: Arc::new(Mutex::new(HashMap::new())),
            connection_map: Arc::new(Mutex::new(HashMap::new())),
            local_socket_map: Arc::new(Mutex::new(HashMap::new())),
            reverse_dns_map: Arc::new(Mutex::new(HashMap::new())),
            local_ip_map: Arc::new(Mutex::new(local_ip_map)),
        }
    }
    // Set interface
    pub fn set_interface(&self, new_interface: Interface) {
        match self.interface.lock() {
            Ok(mut iface) => {
                *iface = new_interface;
            }
            Err(e) => {
                tracing::error!("set_interface error: {:?}", e);
            }
        }
    }
    // Get interface
    pub fn get_interface(&self) -> Interface {
        match self.interface.lock() {
            Ok(iface) => iface.clone(),
            Err(e) => {
                tracing::error!("get_interface error: {:?}", e);
                Interface::dummy()
            }
        }
    }
    // Get the interface index
    pub fn get_if_index(&self) -> u32 {
        match self.interface.lock() {
            Ok(iface) => iface.index,
            Err(e) => {
                tracing::error!("get_if_index error: {:?}", e);
                0
            }
        }
    }
    // Get the interface name
    pub fn get_if_name(&self) -> String {
        match self.interface.lock() {
            Ok(iface) => iface.name.clone(),
            Err(e) => {
                tracing::error!("get_if_name error: {:?}", e);
                String::new()
            }
        }
    }
    /// Get the traffic info. (thread safe clone)
    fn get_trrafic(&self) -> TrafficInfo {
        match self.traffic.lock() {
            Ok(traffic) => traffic.clone(),
            Err(e) => {
                tracing::error!("get_traffic error: {:?}", e);
                TrafficInfo::new()
            }
        }
    }
    /// Get the remote hosts. (thread safe clone)
    pub fn get_remote_hosts(&self) -> HashMap<IpAddr, RemoteHostInfo> {
        match self.remote_hosts.lock() {
            Ok(remote_hosts) => remote_hosts.clone(),
            Err(e) => {
                tracing::error!("get_remote_hosts error: {:?}", e);
                HashMap::new()
            }
        }
    }
    /// Get the connection_map (thread safe clone)
    pub fn get_connection_map(&self) -> HashMap<SocketConnection, TrafficInfo> {
        match self.connection_map.lock() {
            Ok(connection_map) => connection_map.clone(),
            Err(e) => {
                tracing::error!("get_connection_map error: {:?}", e);
                HashMap::new()
            }
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
        match self.local_ip_map.try_lock() {
            Ok(local_ip_map) => local_ip_map.clone(),
            Err(e) => {
                tracing::error!("get_local_ip_map error: {:?}", e);
                HashMap::new()
            }
        }
    }
    fn clear_traffic(&self) {
        match self.traffic.lock() {
            Ok(mut traffic) => {
                *traffic = TrafficInfo::new();
            }
            Err(e) => {
                tracing::error!("clear_traffic error: {:?}", e);
            }
        }
    }
    fn clear_remote_hosts(&self) {
        match self.remote_hosts.lock() {
            Ok(mut remote_hosts) => {
                remote_hosts.clear();
            }
            Err(e) => {
                tracing::error!("clear_remote_hosts error: {:?}", e);
            }
        }
    }
    fn clear_connection_map(&self) {
        match self.connection_map.lock() {
            Ok(mut connection_map) => {
                connection_map.clear();
            }
            Err(e) => {
                tracing::error!("clear_connection_map error: {:?}", e);
            }
        }
    }
    fn clear_local_socket_map(&self) {
        match self.local_socket_map.lock() {
            Ok(mut local_socket_map) => {
                local_socket_map.clear();
            }
            Err(e) => {
                tracing::error!("clear_local_socket_map error: {:?}", e);
            }
        }
    }
    fn clear_reverse_dns_map(&self) {
        match self.reverse_dns_map.lock() {
            Ok(mut reverse_dns_map) => {
                reverse_dns_map.clear();
            }
            Err(e) => {
                tracing::error!("clear_reverse_dns_map error: {:?}", e);
            }
        }
    }
    pub fn reset(&self) {
        self.clear_traffic();
        self.clear_remote_hosts();
        self.clear_connection_map();
        self.clear_local_socket_map();
        self.clear_reverse_dns_map();
    }
    pub fn reset_data(&self) {
        self.clear_traffic();
        self.clear_remote_hosts();
        self.clear_connection_map();
        self.clear_local_socket_map();
    }
    pub fn clone_and_reset(&self) -> Self {
        let clone = self.clone();
        self.reset();
        clone
    }
    pub fn clone_data_and_reset(&self) -> NetStatData {
        let mut clone: NetStatData = NetStatData::new();
        clone.default_interface = self.get_interface();
        clone.traffic = self.get_trrafic();
        clone.remote_hosts = self.get_remote_hosts();
        clone.connection_map = self.get_connection_map();
        clone.local_socket_map = self.get_local_socket_map();
        clone.local_ip_map = self.get_local_ip_map();
        self.reset_data();
        clone
    }
    pub fn clone_data(&self) -> NetStatData {
        let mut clone: NetStatData = NetStatData::new();
        clone.default_interface = self.get_interface();
        clone.traffic = self.get_trrafic();
        clone.remote_hosts = self.get_remote_hosts();
        clone.connection_map = self.get_connection_map();
        clone.local_socket_map = self.get_local_socket_map();
        clone
    }
    pub fn change_interface(&self, interface: &Interface) {
        //self.reset();
        self.set_interface(interface.clone());
        //self.set_local_ips(interface::get_interface_local_ips(interface));
    }
    pub fn interface_changed(&self, if_index: u32) -> bool {
        if if_index != self.get_if_index() {
            return true;
        }
        false
    }
    pub fn parse_dns_packet(&self, dns_packet: &Bytes) {
        if let Some(dns) = DnsPacket::from_buf(dns_packet) {
            let mut name = String::new();
            let mut ip_addr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
            for query in &dns.queries {
                match query.qtype {
                    DnsType::A | DnsType::AAAA => match query.get_qname_parsed() {
                        Ok(qname) => {
                            name = qname.to_string();
                        }
                        Err(e) => {
                            tracing::error!("Failed to parse query name: {:?}", e);
                        }
                    },
                    _ => {}
                }
            }
            for response in &dns.responses {
                match response.rtype {
                    DnsType::A | DnsType::AAAA => {
                        if let Some(ip) = response.get_ip() {
                            ip_addr = ip;
                        } else {
                            tracing::error!(
                                "Failed to get IP address from response: {:?}",
                                response
                            );
                        }
                    }
                    _ => {}
                }
            }
            let mut reverse_dns_map_inner = match self.reverse_dns_map.lock() {
                Ok(inner) => inner,
                Err(e) => {
                    tracing::error!("Failed to lock reverse_dns_map: {:?}", e);
                    return;
                }
            };
            if !name.is_empty() && ip_addr != IpAddr::V4(Ipv4Addr::UNSPECIFIED) {
                reverse_dns_map_inner.insert(ip_addr, name);
            }
            drop(reverse_dns_map_inner);
        }
    }
    pub fn update(&self, frame: PacketFrame) {
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
        let datalink_layer = match frame.datalink {
            Some(datalink) => datalink,
            None => return,
        };
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
        match direction {
            Direction::Egress => {
                traffic_inner.packet_sent += 1;
                traffic_inner.bytes_sent += frame.packet_len;
            }
            Direction::Ingress => {
                traffic_inner.packet_received += 1;
                traffic_inner.bytes_received += frame.packet_len;
            }
        }
        let mac_addr: String = match direction {
            Direction::Egress => {
                if let Some(ethernet) = datalink_layer.ethernet {
                    ethernet.destination.address()
                } else {
                    MacAddr::zero().to_string()
                }
            }
            Direction::Ingress => {
                if let Some(ethernet) = datalink_layer.ethernet {
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
        let interface_name = match local_ip_map_inner.get(&local_ip_addr) {
            Some(name) => name.clone(),
            None => String::from("unknown"),
        };
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
        /* // Update or Insert RemoteHostInfo
        let remote_host: &mut RemoteHostInfo = remote_hosts_inner
            .entry(remote_ip_addr)
            .or_insert(RemoteHostInfo::new(mac_addr, remote_ip_addr));
        match direction {
            Direction::Egress => {
                remote_host.traffic_info.packet_sent += 1;
                remote_host.traffic_info.bytes_sent += frame.packet_len;
            }
            Direction::Ingress => {
                remote_host.traffic_info.packet_received += 1;
                remote_host.traffic_info.bytes_received += frame.packet_len;
            }
        } */
        // Update SocketConnection if the packet is TCP or UDP.
        if let Some(transport) = frame.transport {
            if let Some(_tcp) = transport.tcp {
                let socket_connection: SocketConnection = SocketConnection {
                    interface_name: interface_name.clone(),
                    local_ip_addr: local_ip_addr,
                    local_port: local_port,
                    remote_ip_addr: remote_ip_addr,
                    remote_port: remote_port,
                    protocol: TransportProtocol::TCP,
                };
                let socket_traffic: &mut TrafficInfo = connections_inner
                    .entry(socket_connection)
                    .or_insert(TrafficInfo::new());
                match direction {
                    Direction::Egress => {
                        socket_traffic.packet_sent += 1;
                        socket_traffic.bytes_sent += frame.packet_len;
                    }
                    Direction::Ingress => {
                        socket_traffic.packet_received += 1;
                        socket_traffic.bytes_received += frame.packet_len;
                    }
                }
            }
            if let Some(_udp) = transport.udp {
                let socket_connection: SocketConnection = SocketConnection {
                    interface_name: interface_name,
                    local_ip_addr: local_ip_addr,
                    local_port: local_port,
                    remote_ip_addr: remote_ip_addr,
                    remote_port: remote_port,
                    protocol: TransportProtocol::UDP,
                };
                let socket_traffic: &mut TrafficInfo = connections_inner
                    .entry(socket_connection)
                    .or_insert(TrafficInfo::new());
                match direction {
                    Direction::Egress => {
                        socket_traffic.packet_sent += 1;
                        socket_traffic.bytes_sent += frame.packet_len;
                    }
                    Direction::Ingress => {
                        socket_traffic.packet_received += 1;
                        socket_traffic.bytes_received += frame.packet_len;
                    }
                }
                // Try parse DNS packet
                self.parse_dns_packet(&frame.payload);
            }
        }
        let mut reverse_dns_map_inner = match self.reverse_dns_map.lock() {
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
        let remote_host: &mut RemoteHostInfo =
            remote_hosts_inner
                .entry(remote_ip_addr)
                .or_insert(RemoteHostInfo::new(
                    mac_addr,
                    remote_ip_addr,
                    hostname.clone(),
                ));
        match direction {
            Direction::Egress => {
                remote_host.traffic_info.packet_sent += 1;
                remote_host.traffic_info.bytes_sent += frame.packet_len;
            }
            Direction::Ingress => {
                remote_host.traffic_info.packet_received += 1;
                remote_host.traffic_info.bytes_received += frame.packet_len;
            }
        }
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
pub struct Overview {
    pub traffic: TrafficDisplayInfo,
    pub top_processes: Vec<ProcessDisplayInfo>,
    pub top_remote_hosts: Vec<HostDisplayInfo>,
    pub top_app_protocols: Vec<ServiceDisplayInfo>,
}

impl Overview {
    pub fn new() -> Self {
        Overview {
            traffic: TrafficDisplayInfo::new(),
            top_processes: Vec::new(),
            top_remote_hosts: Vec::new(),
            top_app_protocols: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetStatData {
    pub default_interface: Interface,
    pub traffic: TrafficInfo,
    pub remote_hosts: HashMap<IpAddr, RemoteHostInfo>,
    pub connection_map: HashMap<SocketConnection, TrafficInfo>,
    pub local_socket_map: HashMap<LocalSocket, SocketProcess>,
    pub local_ip_map: HashMap<IpAddr, String>,
}

impl NetStatData {
    pub fn new() -> Self {
        let default_interface = match netdev::get_default_interface() {
            Ok(iface) => iface,
            Err(e) => {
                tracing::error!("NetStatData get_default_interface error: {:?}", e);
                Interface::dummy()
            }
        };
        NetStatData {
            default_interface: default_interface,
            traffic: TrafficInfo::new(),
            remote_hosts: HashMap::new(),
            connection_map: HashMap::new(),
            local_socket_map: HashMap::new(),
            local_ip_map: HashMap::new(),
        }
    }
    // merge using entry method to merge traffic info.
    pub fn merge(&mut self, other: NetStatData, duration: Duration) {
        // Update Interface Info
        self.default_interface = other.default_interface;
        // Update Traffic Info
        self.traffic.update_bytes_per_sec(&other.traffic, duration);
        self.traffic.add_traffic(&other.traffic);
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
                    entry.insert(host.clone());
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
                        entry.insert(traffic_info.clone());
                    }
                },
            );
        // Update local_socket_map
        other
            .local_socket_map
            .iter()
            .for_each(|(local_socket, socket_process)| {
                match self.local_socket_map.entry(local_socket.clone()) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let socket_process_entry = entry.get_mut();
                        socket_process_entry.merge(socket_process);
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(socket_process.clone());
                    }
                }
            });
        // Update local_ip_map
        self.local_ip_map = other.local_ip_map;
    }

    // Remove old entries from remote_hosts, connection_map, local_socket_map
    // TrafficInfo.last_seen is used to determine if the entry is old.
    // If the entry is older than ttl, it will be removed.
    pub fn remove_old_entries(&mut self, ttl: Duration) {
        let now = std::time::SystemTime::now();
        let remote_hosts: HashMap<IpAddr, RemoteHostInfo> = self
            .remote_hosts
            .iter()
            .filter(
                |(_ip, host)| match now.duration_since(host.traffic_info.last_seen) {
                    Ok(duration) => {
                        if duration > ttl {
                            return false;
                        }
                        true
                    }
                    Err(e) => {
                        tracing::error!("remove_old_entries error: {:?}", e);
                        false
                    }
                },
            )
            .map(|(ip, host)| (*ip, host.clone()))
            .collect();
        self.remote_hosts = remote_hosts;
        let mut remove_local_socket: Vec<LocalSocket> = Vec::new();
        let connection_map: HashMap<SocketConnection, TrafficInfo> = self
            .connection_map
            .iter()
            .filter(
                |(conn, traffic_info)| match now.duration_since(traffic_info.last_seen) {
                    Ok(duration) => {
                        if duration > ttl {
                            remove_local_socket.push(LocalSocket {
                                interface_name: conn.interface_name.clone(),
                                port: conn.local_port,
                                protocol: conn.protocol,
                            });
                            return false;
                        }
                        true
                    }
                    Err(e) => {
                        tracing::error!("remove_old_entries error: {:?}", e);
                        remove_local_socket.push(LocalSocket {
                            interface_name: conn.interface_name.clone(),
                            port: conn.local_port,
                            protocol: conn.protocol,
                        });
                        false
                    }
                },
            )
            .map(|(conn, traffic_info)| (conn.clone(), traffic_info.clone()))
            .collect();
        self.connection_map = connection_map;

        for local_socket in remove_local_socket {
            self.local_socket_map.remove(&local_socket);
        }
    }

    pub fn get_remote_hosts(&self, limit: Option<usize>) -> Vec<HostDisplayInfo> {
        // Create a map to store the traffic info for each remote host.
        let mut host_traffic_map: HashMap<IpAddr, usize> = HashMap::new();
        self.remote_hosts.iter().for_each(|(_ip, host)| {
            match host_traffic_map.get(&host.ip_addr) {
                Some(traffic) => {
                    let mut traffic = traffic.clone();
                    traffic += host.traffic_info.bytes_sent;
                    traffic += host.traffic_info.bytes_received;
                    host_traffic_map.insert(host.ip_addr, traffic);
                }
                None => {
                    host_traffic_map.insert(
                        host.ip_addr,
                        host.traffic_info.bytes_sent + host.traffic_info.bytes_received,
                    );
                }
            }
        });
        let mut host_traffic_vec: Vec<(&IpAddr, &usize)> = host_traffic_map.iter().collect();
        host_traffic_vec.sort_by(|a, b| b.1.cmp(a.1));
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

    pub fn get_processes(&self, limit: Option<usize>) -> Vec<ProcessDisplayInfo> {
        let mut process_traffic_map: HashMap<u32, TrafficInfo> = HashMap::new();
        let mut process_map: HashMap<u32, ProcessInfo> = HashMap::new();
        self.connection_map.iter().for_each(|(conn, traffic_info)| {
            let local_socket: LocalSocket = LocalSocket {
                interface_name: conn.interface_name.clone(),
                port: conn.local_port,
                protocol: conn.protocol,
            };
            match self.local_socket_map.get(&local_socket) {
                Some(socket_process) => {
                    if let Some(process) = &socket_process.process {
                        match process_traffic_map.get(&process.pid) {
                            Some(traffic) => {
                                let mut traffic = traffic.clone();
                                traffic.add_traffic(traffic_info);
                                process_traffic_map.insert(process.pid, traffic);
                            }
                            None => {
                                process_traffic_map.insert(process.pid, traffic_info.clone());
                            }
                        }
                        process_map.insert(process.pid, process.clone());
                    }
                }
                None => {}
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
        process_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1));
        // Create top processes from process_total_traffic_vec
        let mut top_processes: Vec<ProcessDisplayInfo> = Vec::new();
        // limit : if limit is None, return all processes.
        for (pid, _) in process_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(process_total_traffic_vec.len()))
        {
            if let Some(traffic) = process_traffic_map.get(pid) {
                if let Some(process) = process_map.get(pid) {
                    let process = ProcessDisplayInfo {
                        pid: process.pid,
                        name: process.name.clone(),
                        traffic: traffic.to_display_info(),
                    };
                    top_processes.push(process);
                }
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
        connection_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1));
        let mut top_connections: Vec<SocketDisplayInfo> = Vec::new();
        // limit : if limit is None, return all connections.
        for (conn, _) in connection_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(connection_total_traffic_vec.len()))
        {
            // Get process info from local_socket_map
            let process: Option<ProcessInfo> = match self.local_socket_map.get(&LocalSocket {
                interface_name: conn.interface_name.clone(),
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
                    process: process,
                };
                top_connections.push(socket_traffic_info);
            }
        }
        top_connections
    }

    pub fn get_connections_with_opt(
        &self,
        limit: Option<usize>,
        opt: SocketInfoOption,
    ) -> Vec<SocketDisplayInfo> {
        let connection_total_traffic_map: HashMap<SocketConnection, usize> = self
            .connection_map
            .iter()
            .map(|(conn, traffic)| (conn.clone(), traffic.total_bytes()))
            .collect();
        let mut connection_total_traffic_vec: Vec<(&SocketConnection, &usize)> =
            connection_total_traffic_map.iter().collect();
        connection_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1));
        let mut top_connections: Vec<SocketDisplayInfo> = Vec::new();
        // limit : if limit is None, return all connections.
        for (conn, _) in connection_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(connection_total_traffic_vec.len()))
        {
            // Get process info from local_socket_map
            let process: Option<ProcessInfo> = match self.local_socket_map.get(&LocalSocket {
                interface_name: conn.interface_name.clone(),
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
                    process: process,
                };
                if opt.address_family.contains(&socket_traffic_info.ip_version)
                    && opt
                        .transport_protocol
                        .contains(&socket_traffic_info.protocol)
                {
                    top_connections.push(socket_traffic_info);
                }
            }
        }
        top_connections
    }

    pub fn get_app_protocols(&self, limit: Option<usize>) -> Vec<ServiceDisplayInfo> {
        let tcp_db = crate::db::TCP_SERVICE_DB.get().unwrap().read().unwrap();
        let udp_db = crate::db::UDP_SERVICE_DB.get().unwrap().read().unwrap();
        let mut protocol_port_map: HashMap<ProtocolPort, TrafficInfo> = HashMap::new();
        self.connection_map.iter().for_each(|(conn, traffic_info)| {
            let protocol_port: ProtocolPort = ProtocolPort {
                protocol: conn.protocol,
                port: conn.remote_port,
            };
            match protocol_port_map.get(&protocol_port) {
                Some(traffic) => {
                    let mut traffic = traffic.clone();
                    traffic.add_traffic(traffic_info);
                    protocol_port_map.insert(protocol_port, traffic);
                }
                None => {
                    protocol_port_map.insert(protocol_port, traffic_info.clone());
                }
            }
        });
        let protocol_total_traffic_map: HashMap<ProtocolPort, usize> = protocol_port_map
            .iter()
            .map(|(protocol_port, traffic)| (*protocol_port, traffic.total_bytes()))
            .collect();
        let mut protocol_total_traffic_vec: Vec<(&ProtocolPort, &usize)> =
            protocol_total_traffic_map.iter().collect();
        protocol_total_traffic_vec.sort_by(|a, b| b.1.cmp(a.1));
        let mut top_app_protocols: Vec<ServiceDisplayInfo> = Vec::new();
        // limit : if limit is None, return all app protocols.
        for (protocol_port, _) in protocol_total_traffic_vec
            .iter()
            .take(limit.unwrap_or(protocol_total_traffic_vec.len()))
        {
            if let Some(traffic) = protocol_port_map.get(protocol_port) {
                let service = ServiceDisplayInfo {
                    port: protocol_port.port,
                    protocol: protocol_port.protocol.as_str().to_string(),
                    name: match protocol_port.protocol {
                        TransportProtocol::TCP => tcp_db
                            .get_name(protocol_port.port)
                            .unwrap_or("Unknown TCP Service")
                            .to_string(),
                        TransportProtocol::UDP => udp_db
                            .get_name(protocol_port.port)
                            .unwrap_or("Unknown UDP Service")
                            .to_string(),
                    },
                    traffic: traffic.to_display_info(),
                };
                top_app_protocols.push(service);
            }
        }
        top_app_protocols
    }

    pub fn get_overview(&self) -> Overview {
        let mut overview = Overview::new();
        overview.traffic = TrafficDisplayInfo::from_traffic(&self.traffic);
        // Get top remote hosts
        overview.top_remote_hosts = self.get_remote_hosts(Some(10));
        // Get top processes
        overview.top_processes = self.get_processes(Some(10));
        // Get top app protocols
        overview.top_app_protocols = self.get_app_protocols(Some(10));
        overview
    }
}

pub fn update_netstat_data(
    netstat_strage: &mut Arc<NetStatStrage>,
    netstat_data: &mut Arc<Mutex<NetStatData>>,
    interval: Duration,
) {
    loop {
        match netstat_data.lock() {
            Ok(mut data) => {
                data.merge(netstat_strage.clone_data_and_reset(), interval);
            }
            Err(e) => {
                tracing::error!("Error: {:?}", e);
                continue;
            }
        }
        thread::sleep(interval);
    }
}
