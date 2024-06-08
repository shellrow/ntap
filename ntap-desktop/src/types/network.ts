// TypeScript types from the Rust types
export interface DatalinkLayer {
    ethernet: EthernetHeader | null,
    arp: ArpHeader | null,
}

export interface EthernetHeader {
    destination: string,
    source: string,
    ethertype: string,
}

export interface ArpHeader {
    hardware_type: string,
    protocol_type: string,
    hw_addr_len: number,
    proto_addr_len: number,
    operation: string,
    sender_hw_addr: string,
    sender_proto_addr: string,
    target_hw_addr: string,
    target_proto_addr: string,
}

export interface Ipv4OptionHeader {
    copied: number,
    class: number,
    number: string,
    length: number | null,
}

export interface Ipv4Header {
    version: number,
    header_length: number,
    dscp: number,
    ecn: number,
    total_length: number,
    identification: number,
    flags: number,
    fragment_offset: number,
    ttl: number,
    next_level_protocol: string,
    checksum: number,
    source: string,
    destination: string,
    options: Ipv4OptionHeader[],
}

export interface Ipv6Header {
    version: number,
    traffic_class: number,
    flow_label: number,
    payload_length: number,
    next_header: string,
    hop_limit: number,
    source: string,
    destination: string,
}

export interface IcmpHeader {
    icmp_type: string,
    icmp_code: string,
    checksum: number,
}

export interface Icmpv6Header {
    icmpv6_type: string,
    icmpv6_code: string,
    checksum: number,
}

export interface TcpOptionHeader {
    kind: string,
    length: number | null,
    data: number[],
}

export interface TcpHeader {
    source: number,
    destination: number,
    sequence: number,
    acknowledgement: number,
    data_offset: number,
    reserved: number,
    flags: number,
    window: number,
    checksum: number,
    urgent_ptr: number,
    options: TcpOptionHeader[],
}

export interface UdpHeader {
    source: number,
    destination: number,
    length: number,
    checksum: number,
}

export interface IpLayer {
    ipv4: Ipv4Header | null,
    ipv6: Ipv6Header | null,
    icmp: IcmpHeader | null,
    icmpv6: Icmpv6Header | null,
}

export interface TransportLayer {
    tcp: TcpHeader | null,
    udp: UdpHeader | null,
}

export interface PacketFrame {
    capture_no: number,
    if_index: number,
    if_name: string,
    datalink: DatalinkLayer | null,
    ip: IpLayer | null,
    transport: TransportLayer | null,
    packet_len: number,
    timestamp: string,
}

export interface PacketDisplayData {
    capture_no: number,
    timestamp: string,
    if_index: number,
    if_name: string,
    src_addr: string,
    dst_addr: string,
    src_port: number | null,
    dst_port: number | null,
    protocol: string,
    packet_len: number,
    info: string,
}

export interface PacketFrameExt {
    capture_no: number,
    timestamp: string,
    if_index: number,
    if_name: string,
    src_addr: string,
    dst_addr: string,
    src_port: number | null,
    dst_port: number | null,
    protocol: string,
    packet_len: number,
    info: string,
    datalink: DatalinkLayer | null,
    ip: IpLayer | null,
    transport: TransportLayer | null,
}

export interface SocketInfoOption {
    address_family: string[],
    transport_protocol: string[],
}

export interface UserInfo {
    user_id: string,
    group_id: string,
    user_name: string,
    groups: string[],
}

export interface ProcessInfo {
    pid: number,
    name: string,
}

export interface SocketInfo {
    local_ip_addr: string,
    local_port: number,
    remote_ip_addr: string | null,
    remote_port: number | null,
    protocol: string,
    status: string,
    ip_version: string,
    process: ProcessInfo | null,
}

export class TrafficInfo {
    packet_sent: number;
    packet_received: number;
    bytes_sent: number;
    bytes_received: number;
    first_seen: string;
    last_seen: string;

    constructor() {
        this.packet_sent = 0;
        this.packet_received = 0;
        this.bytes_sent = 0;
        this.bytes_received = 0;
        this.first_seen = new Date().toISOString();
        this.last_seen = new Date().toISOString();
    }

    public add_traffic(traffic: TrafficInfo): void {
        this.packet_sent += traffic.packet_sent;
        this.packet_received += traffic.packet_received;
        this.bytes_sent += traffic.bytes_sent;
        this.bytes_received += traffic.bytes_received;
        this.last_seen = new Date().toISOString();
    }

    public total_packet(): number {
        return this.packet_sent + this.packet_received;
    }

    public total_bytes(): number {
        return this.bytes_sent + this.bytes_received;
    }

    public static format_bytes(bytes: number): string {
        const KB = 1024;
        const MB = KB * 1024;
        const GB = MB * 1024;

        if (bytes >= GB) {
            return (bytes / GB).toFixed(2) + " GB";
        } else if (bytes >= MB) {
            return (bytes / MB).toFixed(2) + " MB";
        } else if (bytes >= KB) {
            return (bytes / KB).toFixed(2) + " KB";
        } else {
            return bytes + " B";
        }
    }

    public formatted_total_bytes(): string {
        return TrafficInfo.format_bytes(this.total_bytes());
    }

    public formatted_sent_bytes(): string {
        return TrafficInfo.format_bytes(this.bytes_sent);
    }

    public formatted_received_bytes(): string {
        return TrafficInfo.format_bytes(this.bytes_received);
    }
}

export interface TrafficDisplayInfo {
    packet_sent: number,
    packet_received: number,
    bytes_sent: number,
    bytes_received: number,
    egress_packets_per_sec: number,
    egress_bytes_per_sec: number,
    ingress_packets_per_sec: number,
    ingress_bytes_per_sec: number,
    formatted_sent_bytes: string,
    formatted_received_bytes: string,
    formatted_total_bytes: string,
    formatted_egress_packets_per_sec: string,
    formatted_ingress_packets_per_sec: string,
    formatted_egress_bytes_per_sec: string,
    formatted_ingress_bytes_per_sec: string,
}

export enum TransportProtocol {
    TCP,
    UDP
}

export interface PortInfo {
    port: number,
    protocol: TransportProtocol,
}

export interface RemoteHostInfo {
    if_index: number,
    if_name: string,
    mac_addr: string,
    ip_addr: string,
    hostname: string,
    country_code: string,
    country_name: string,
    asn: number,
    as_name: string,
    traffic_info: TrafficInfo,
    protocol_stat: { [key: string]: TrafficInfo },
    first_seen: string,
    updated_at: string,
}

export interface ProcessTrafficInfo {
    process: ProcessInfo,
    traffic: TrafficInfo,
}

export interface ProcessDisplayInfo {
    pid: number,
    name: string,
    traffic: TrafficDisplayInfo,
}

export interface HostDisplayInfo {
    ip_addr: string,
    host_name: string,
    country_code: string,
    country_name: string,
    asn: number,
    as_name: string,
    traffic: TrafficDisplayInfo,
}

export interface ServiceDisplayInfo {
    port: number,
    protocol: string,
    name: string,
    traffic: TrafficDisplayInfo,
}

export enum NotificationType {
    Traffic,
    RemoteHost,
    Protocol,
}

export interface Notification {
    title: string,
    body: string,
    notification_type: NotificationType,
    timestamp: string,
}

export interface Overview {
    //default_interface: NetworkInterface,
    //captured_packets: number,
    traffic: TrafficDisplayInfo,
    top_processes: ProcessDisplayInfo[],
    top_remote_hosts: HostDisplayInfo[],
    top_app_protocols: ServiceDisplayInfo[],
    notifications: Notification[],
}

export interface IpInfo {
    ip_version: string,
    ip_addr_dec: string,
    ip_addr: string,
    host_name: string,
    network: string,
    asn: string,
    as_name: string,
    country_code: string,
    country_name: string,
}

export interface IpNetwork {
    addr: string,
    prefix_len: number,
    netmask: string,
}

export interface NetworkDevice {
    mac_addr: string,
    ipv4: string[],
    ipv6: string[],
}

export interface NetworkInterface {
    index: number,
    name: string,
    friendly_name: string | null,
    description: string | null,
    if_type: string,
    mac_addr: string | null,
    ipv4: IpNetwork[],
    ipv6: IpNetwork[],
    flags: number,
    transmit_speed: number | null,
    receive_speed: number | null,
    gateway: NetworkDevice | null,
    dns_servers: string[],
    default: boolean,
}

export interface SocketTrafficInfo {
    interface_name: string,
    local_port: number,
    remote_ip_addr: string | null,
    remote_port: number | null,
    protocol: string,
    ip_version: string,
    process: ProcessInfo | null,
    traffic: TrafficInfo,
}

export interface SocketDisplayInfo {
    interface_name: string,
    local_ip_addr: string,
    local_port: number,
    remote_ip_addr: string | null,
    remote_port: number | null,
    protocol: string,
    ip_version: string,
    process: ProcessInfo | null,
    traffic: TrafficDisplayInfo,
}

export interface IpInfoDisplayData {
    ipv4: string,
    ipv4_network: string,
    ipv6: string,
    ipv6_network: string,
    host_name: string,
    asn: string,
    as_name: string,
    country_code: string,
    country_name: string,
}

export type DownloadProgress = ContentLength | Downloaded;

export interface ContentLength {
    ContentLength: number;
}

export interface Downloaded {
    Downloaded: number;
}

export interface NetRoute {
    interface_name: string,
    source: string,
    destination: string,
    netmask: string,
    gateway: string,
}
