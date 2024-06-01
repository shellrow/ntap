export interface KVItem {
    key: string,
    value: string,
}

export interface OptionItem {
    id: string,
    name: string,
}

export interface AppInfo {
    name: string,
    description: string,
    version: string,
    release_date: string,
    repository: string,
}

export interface LoggingConfig {
    level: string,
    file_path: string,
}

export interface NetworkConfig {
    interfaces: string[],
    reverse_dns: boolean,
    entry_ttl: number,
}

export interface DisplayConfig {
    top_remote_hosts: number,
    connection_count: number,
    tick_rate: number,
    show_bandwidth: boolean,
}

export interface PrivacyConfig {
    hide_private_ip_info: boolean,
    hide_public_ip_info: boolean,
}

export class AppConfig {
    logging: LoggingConfig;
    network: NetworkConfig;
    display: DisplayConfig;
    privacy: PrivacyConfig;

    constructor() {
        this.logging = {
            level: 'ERROR',
            file_path: './ntap.log',
        };
        this.network = {
            interfaces: [],
            reverse_dns: false,
            entry_ttl: 60000,
        };
        this.display = {
            top_remote_hosts: 10,
            connection_count: 10,
            tick_rate: 1000,
            show_bandwidth: false,
        };
        this.privacy = {
            hide_private_ip_info: true,
            hide_public_ip_info: true,
        };
    }
}

export class DatabaseConfig {
    ipv4_asn_db_path: string;
    ipv6_asn_db_path: string;
    ipv4_country_db_path: string;
    ipv6_country_db_path: string;
    country_db_path: string
    asn_db_path: string;
    tcp_service_db_path: string;

    constructor() {
        this.ipv4_asn_db_path = '';
        this.ipv6_asn_db_path = '';
        this.ipv4_country_db_path = '';
        this.ipv6_country_db_path = '';
        this.country_db_path = '';
        this.asn_db_path = '';
        this.tcp_service_db_path = '';
    }
}
