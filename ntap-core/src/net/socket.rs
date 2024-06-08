use crate::net::stat::NetStatStrage;
use crate::net::traffic::{TrafficDisplayInfo, TrafficInfo};
use crate::process::ProcessInfo;
use crate::thread_log;
use netsock::family::AddressFamilyFlags;
use netsock::protocol::ProtocolFlags;
use netsock::socket::ProtocolSocketInfo;
use netsock::state::TcpState;
use nex::packet::tcp::TcpFlags;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct SocketConnection {
    pub interface_name: String,
    pub local_ip_addr: IpAddr,
    pub local_port: u16,
    pub remote_ip_addr: IpAddr,
    pub remote_port: u16,
    pub protocol: TransportProtocol,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq)]
pub enum SocketStatus {
    Closed,
    Listen,
    SynSent,
    SynReceived,
    Established,
    FinWait1,
    FinWait2,
    CloseWait,
    Closing,
    LastAck,
    TimeWait,
    DeleteTcb,
    Unknown,
}

impl SocketStatus {
    pub fn from_netstat2_state(state: TcpState) -> Self {
        match state {
            TcpState::Closed => SocketStatus::Closed,
            TcpState::Listen => SocketStatus::Listen,
            TcpState::SynSent => SocketStatus::SynSent,
            TcpState::SynReceived => SocketStatus::SynReceived,
            TcpState::Established => SocketStatus::Established,
            TcpState::FinWait1 => SocketStatus::FinWait1,
            TcpState::FinWait2 => SocketStatus::FinWait2,
            TcpState::CloseWait => SocketStatus::CloseWait,
            TcpState::Closing => SocketStatus::Closing,
            TcpState::LastAck => SocketStatus::LastAck,
            TcpState::TimeWait => SocketStatus::TimeWait,
            TcpState::DeleteTcb => SocketStatus::DeleteTcb,
            _ => SocketStatus::Unknown,
        }
    }
    pub fn from_xenet_tcp_flags(flags: u8) -> Self {
        // match is cause unreachable pattern. so use if-else.
        if flags == TcpFlags::SYN {
            SocketStatus::SynSent
        } else if flags == TcpFlags::SYN | TcpFlags::ACK {
            SocketStatus::SynReceived
        } else if flags == TcpFlags::ACK {
            SocketStatus::Established
        } else if flags == TcpFlags::FIN | TcpFlags::ACK {
            SocketStatus::Closing
        } else if flags == TcpFlags::FIN {
            SocketStatus::FinWait1
        } else {
            SocketStatus::Unknown
        }
    }
}

impl std::fmt::Display for SocketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SocketStatus::Closed => "CLOSED",
                SocketStatus::Listen => "LISTEN",
                SocketStatus::SynSent => "SYN_SENT",
                SocketStatus::SynReceived => "SYN_RCVD",
                SocketStatus::Established => "ESTABLISHED",
                SocketStatus::FinWait1 => "FIN_WAIT_1",
                SocketStatus::FinWait2 => "FIN_WAIT_2",
                SocketStatus::CloseWait => "CLOSE_WAIT",
                SocketStatus::Closing => "CLOSING",
                SocketStatus::LastAck => "LAST_ACK",
                SocketStatus::TimeWait => "TIME_WAIT",
                SocketStatus::DeleteTcb => "DELETE_TCB",
                SocketStatus::Unknown => "UNKNOWN",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketProcess {
    pub socket_addr: SocketAddr,
    pub protocol: TransportProtocol,
    pub status: SocketStatus,
    pub process: Option<ProcessInfo>,
}

impl SocketProcess {
    pub fn new() -> Self {
        SocketProcess {
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            protocol: TransportProtocol::TCP,
            status: SocketStatus::Unknown,
            process: None,
        }
    }
    pub fn merge(&mut self, other: &SocketProcess) {
        self.socket_addr = other.socket_addr;
        self.protocol = other.protocol;
        self.status = other.status;
        self.process = other.process.clone();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketInfo {
    pub local_ip_addr: IpAddr,
    pub local_port: u16,
    pub remote_ip_addr: Option<IpAddr>,
    pub remote_port: Option<u16>,
    pub protocol: TransportProtocol,
    pub status: SocketStatus,
    pub ip_version: AddressFamily,
    pub process: Option<ProcessInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketTrafficInfo {
    pub interface_name: String,
    pub local_ip_addr: IpAddr,
    pub local_port: u16,
    pub remote_ip_addr: Option<IpAddr>,
    pub remote_port: Option<u16>,
    pub protocol: TransportProtocol,
    pub ip_version: AddressFamily,
    pub process: Option<ProcessInfo>,
    pub traffic: TrafficInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketDisplayInfo {
    pub interface_name: String,
    pub local_ip_addr: IpAddr,
    pub local_port: u16,
    pub remote_ip_addr: Option<IpAddr>,
    pub remote_port: Option<u16>,
    pub protocol: TransportProtocol,
    pub ip_version: AddressFamily,
    pub process: Option<ProcessInfo>,
    pub traffic: TrafficDisplayInfo,
}

impl SocketDisplayInfo {
    pub fn from_socket_traffic_info(socket_traffic_info: &SocketTrafficInfo) -> Self {
        SocketDisplayInfo {
            interface_name: socket_traffic_info.interface_name.clone(),
            local_ip_addr: socket_traffic_info.local_ip_addr,
            local_port: socket_traffic_info.local_port,
            remote_ip_addr: socket_traffic_info.remote_ip_addr,
            remote_port: socket_traffic_info.remote_port,
            protocol: socket_traffic_info.protocol,
            ip_version: socket_traffic_info.ip_version.clone(),
            process: socket_traffic_info.process.clone(),
            traffic: socket_traffic_info.traffic.to_display_info(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord, Copy)]
pub enum TransportProtocol {
    TCP,
    UDP,
}

impl TransportProtocol {
    pub fn as_str(&self) -> &str {
        match self {
            TransportProtocol::TCP => "TCP",
            TransportProtocol::UDP => "UDP",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct ProtocolSocketAddress {
    pub socket: SocketAddr,
    pub protocol: TransportProtocol,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct LocalSocket {
    pub interface_name: String,
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl LocalSocket {
    pub fn new(interface_name: String, port: u16, protocol: TransportProtocol) -> Self {
        LocalSocket {
            interface_name: interface_name,
            port: port,
            protocol: protocol,
        }
    }
    pub fn to_key_string(&self) -> String {
        format!(
            "{}-{}-{}",
            self.interface_name,
            self.port,
            self.protocol.as_str()
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct ProtocolPort {
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl ProtocolPort {
    pub fn new(port: u16, protocol: TransportProtocol) -> Self {
        ProtocolPort {
            port: port,
            protocol: protocol,
        }
    }
    pub fn to_key_string(&self) -> String {
        format!("{}-{}", self.port, self.protocol.as_str())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SocketInfoOption {
    pub address_family: Vec<AddressFamily>,
    pub transport_protocol: Vec<TransportProtocol>,
}

impl Default for SocketInfoOption {
    fn default() -> SocketInfoOption {
        SocketInfoOption {
            address_family: vec![AddressFamily::IPv4, AddressFamily::IPv6],
            transport_protocol: vec![TransportProtocol::TCP, TransportProtocol::UDP],
        }
    }
}

impl SocketInfoOption {
    pub fn new(
        address_family: Vec<AddressFamily>,
        transport_protocol: Vec<TransportProtocol>,
    ) -> SocketInfoOption {
        SocketInfoOption {
            address_family: address_family,
            transport_protocol: transport_protocol,
        }
    }
    pub fn get_address_family_flags(&self) -> AddressFamilyFlags {
        let mut flags: AddressFamilyFlags = AddressFamilyFlags::empty();
        for af in &self.address_family {
            match af {
                AddressFamily::IPv4 => {
                    flags |= AddressFamilyFlags::IPV4;
                }
                AddressFamily::IPv6 => {
                    flags |= AddressFamilyFlags::IPV6;
                }
            }
        }
        flags
    }
    pub fn get_protocol_flags(&self) -> ProtocolFlags {
        let mut flags: ProtocolFlags = ProtocolFlags::empty();
        for tp in &self.transport_protocol {
            match tp {
                TransportProtocol::TCP => {
                    flags |= ProtocolFlags::TCP;
                }
                TransportProtocol::UDP => {
                    flags |= ProtocolFlags::UDP;
                }
            }
        }
        flags
    }
}

pub fn get_sockets_info(opt: SocketInfoOption) -> Vec<SocketInfo> {
    let af_flags: AddressFamilyFlags = opt.get_address_family_flags();
    let proto_flags: ProtocolFlags = opt.get_protocol_flags();
    let sockets: Vec<netsock::socket::SocketInfo> =
        netsock::get_sockets(af_flags, proto_flags).unwrap();
    let mut sockets_info: Vec<SocketInfo> = Vec::new();

    for si in sockets {
        match si.protocol_socket_info {
            ProtocolSocketInfo::Tcp(tcp_si) => {
                if tcp_si.local_port == 0 {
                    continue;
                }
                let socket_info = SocketInfo {
                    local_ip_addr: tcp_si.local_addr,
                    local_port: tcp_si.local_port,
                    remote_ip_addr: Some(tcp_si.remote_addr),
                    remote_port: Some(tcp_si.remote_port),
                    protocol: TransportProtocol::TCP,
                    status: SocketStatus::from_netstat2_state(tcp_si.state),
                    ip_version: if tcp_si.local_addr.is_ipv4() {
                        AddressFamily::IPv4
                    } else {
                        AddressFamily::IPv6
                    },
                    process: if let Some(proc) = si.processes.first() {
                        Some(ProcessInfo::new(proc.pid, proc.name.clone()))
                    } else {
                        None
                    },
                };
                sockets_info.push(socket_info);
            }
            ProtocolSocketInfo::Udp(udp_si) => {
                if udp_si.local_port == 0 {
                    continue;
                }
                let socket_info = SocketInfo {
                    local_ip_addr: udp_si.local_addr,
                    local_port: udp_si.local_port,
                    remote_ip_addr: None,
                    remote_port: None,
                    protocol: TransportProtocol::UDP,
                    status: SocketStatus::Unknown,
                    ip_version: if udp_si.local_addr.is_ipv4() {
                        AddressFamily::IPv4
                    } else {
                        AddressFamily::IPv6
                    },
                    process: if let Some(proc) = si.processes.first() {
                        Some(ProcessInfo::new(proc.pid, proc.name.clone()))
                    } else {
                        None
                    },
                };
                sockets_info.push(socket_info);
            }
        }
    }
    sockets_info
}

pub fn start_socket_info_update(netstat_strage: &mut Arc<NetStatStrage>) {
    let mut local_ip_map: HashMap<IpAddr, String> = HashMap::new();
    loop {
        if local_ip_map.is_empty() {
            local_ip_map = netstat_strage.get_local_ip_map();
        }
        let sockets_info = get_sockets_info(SocketInfoOption::default());
        // Create Vec<LocalSocket>
        let mut local_sockets: HashSet<LocalSocket> = HashSet::new();
        for si in &sockets_info {
            match local_ip_map.get(&si.local_ip_addr) {
                Some(interface_name) => {
                    local_sockets.insert(LocalSocket::new(
                        interface_name.to_owned(),
                        si.local_port,
                        si.protocol,
                    ));
                }
                None => {}
            }
        }
        // Lock the local_socket_map
        let mut local_socket_inner = match netstat_strage.local_socket_map.try_lock() {
            Ok(connections) => connections,
            Err(e) => {
                thread_log!(error, "[socket_info_update] lock error: {}", e);
                continue;
            }
        };
        // Remove old socket info
        let mut remove_keys: Vec<LocalSocket> = vec![];
        for conn in local_socket_inner.iter() {
            if !local_sockets.contains(conn.0) {
                remove_keys.push(conn.0.clone());
            }
        }
        for key in remove_keys {
            local_socket_inner.remove(&key);
        }
        // Update socket info
        for socket_info in sockets_info {
            match local_ip_map.get(&socket_info.local_ip_addr) {
                Some(interface_name) => {
                    let local_socket = LocalSocket::new(
                        interface_name.to_owned(),
                        socket_info.local_port,
                        socket_info.protocol,
                    );
                    let socket_process = local_socket_inner
                        .entry(local_socket)
                        .or_insert(SocketProcess::new());
                    socket_process.status = socket_info.status;
                    socket_process.process = socket_info.process.clone();
                }
                None => {}
            }
        }
        // Drop the lock
        drop(local_socket_inner);
        std::thread::sleep(std::time::Duration::from_secs(10));
    }
}