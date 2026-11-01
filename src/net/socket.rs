use crate::net::stat::NetStatStorage;
use crate::net::traffic::TrafficDisplayInfo;
use crate::process::ProcessInfo;
use netsock::family::AddressFamilyFlags;
use netsock::protocol::ProtocolFlags;
use netsock::socket::ProtocolSocketInfo;
use netsock::state::TcpState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(test)]
use std::net::Ipv4Addr;
use std::net::{IpAddr, SocketAddr};
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
    #[cfg(test)]
    pub fn new() -> Self {
        SocketProcess {
            socket_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
            protocol: TransportProtocol::TCP,
            status: SocketStatus::Unknown,
            process: None,
        }
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AddressFamily {
    IPv4,
    IPv6,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash, Eq, PartialOrd, Ord, Copy)]
#[allow(clippy::upper_case_acronyms)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord)]
pub struct LocalSocket {
    pub local_ip_addr: IpAddr,
    pub port: u16,
    pub protocol: TransportProtocol,
}

impl LocalSocket {
    pub fn new(local_ip_addr: IpAddr, port: u16, protocol: TransportProtocol) -> Self {
        LocalSocket {
            local_ip_addr,
            port,
            protocol,
        }
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
        match netsock::get_sockets(af_flags, proto_flags) {
            Ok(sockets) => sockets,
            Err(e) => {
                tracing::warn!("[get_sockets_info] failed to fetch socket list: {}", e);
                return Vec::new();
            }
        };
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
                    process: si
                        .processes
                        .first()
                        .map(|proc| ProcessInfo::new(proc.pid, proc.name.clone())),
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
                    process: si
                        .processes
                        .first()
                        .map(|proc| ProcessInfo::new(proc.pid, proc.name.clone())),
                };
                sockets_info.push(socket_info);
            }
        }
    }
    sockets_info
}

pub fn start_socket_info_update(
    netstat_storage: &mut Arc<NetStatStorage>,
    stop: &std::sync::atomic::AtomicBool,
) {
    loop {
        if stop.load(std::sync::atomic::Ordering::Acquire) {
            break;
        }
        let local_ip_map = crate::net::interface::get_local_ip_map();
        if let Ok(mut map) = netstat_storage.local_ip_map.lock() {
            *map = local_ip_map.clone();
        }
        let sockets_info = get_sockets_info(SocketInfoOption::default());
        let sockets: HashMap<LocalSocket, SocketProcess> = sockets_info
            .into_iter()
            .map(|socket| {
                let key =
                    LocalSocket::new(socket.local_ip_addr, socket.local_port, socket.protocol);
                let process = SocketProcess {
                    socket_addr: SocketAddr::new(socket.local_ip_addr, socket.local_port),
                    protocol: socket.protocol,
                    status: socket.status,
                    process: socket.process,
                };
                (key, process)
            })
            .collect();
        if let Ok(mut current) = netstat_storage.local_socket_map.lock() {
            *current = sockets;
        }
        if crate::util::wait_for_stop(stop, std::time::Duration::from_secs(10)) {
            break;
        }
    }
}
