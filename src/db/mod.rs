use anyhow::Result;
use ndb_as::AsDb;
use ndb_ipv4_asn::Ipv4AsnDb;
use ndb_ipv4_country::Ipv4CountryDb;
use ndb_ipv6_asn::Ipv6AsnDb;
use ndb_ipv6_country::Ipv6CountryDb;
use ndb_tcp_service::TcpServiceDb;
use ndb_udp_service::UdpServiceDb;
use std::sync::{OnceLock, RwLock};

pub static TCP_SERVICE_DB: OnceLock<RwLock<TcpServiceDb>> = OnceLock::new();
pub static UDP_SERVICE_DB: OnceLock<RwLock<UdpServiceDb>> = OnceLock::new();
pub static AS_DB: OnceLock<RwLock<AsDb>> = OnceLock::new();
pub static IPV4_ASN_DB: OnceLock<RwLock<Ipv4AsnDb>> = OnceLock::new();
pub static IPV6_ASN_DB: OnceLock<RwLock<Ipv6AsnDb>> = OnceLock::new();
pub static IPV4_COUNTRY_DB: OnceLock<RwLock<Ipv4CountryDb>> = OnceLock::new();
pub static IPV6_COUNTRY_DB: OnceLock<RwLock<Ipv6CountryDb>> = OnceLock::new();

/// Initialize all databases
pub fn init_databases() -> Result<()> {
    tracing::info!("Initializing databases...");
    init_tcp_service_db()?;
    init_udp_service_db()?;
    init_as_db()?;
    init_ipv4_asn_db()?;
    init_ipv6_asn_db()?;
    init_ipv4_country_db()?;
    init_ipv6_country_db()?;
    tracing::info!("Databases initialized successfully.");
    Ok(())
}

pub fn init_tcp_service_db() -> Result<()> {
    // Initialize TCP Service database
    let tcp_service_db = TcpServiceDb::bundled();
    TCP_SERVICE_DB
        .set(RwLock::new(tcp_service_db))
        .map_err(|_| anyhow::anyhow!("Failed to set TCP_SERVICE_DB in OnceLock"))?;
    Ok(())
}

pub fn init_udp_service_db() -> Result<()> {
    // Initialize UDP Service database
    let udp_service_db = UdpServiceDb::bundled();
    UDP_SERVICE_DB
        .set(RwLock::new(udp_service_db))
        .map_err(|_| anyhow::anyhow!("Failed to set UDP_SERVICE_DB in OnceLock"))?;
    Ok(())
}

pub fn init_as_db() -> Result<()> {
    // Initialize AS database
    let as_db = AsDb::bundled();
    AS_DB
        .set(RwLock::new(as_db))
        .map_err(|_| anyhow::anyhow!("Failed to set AS_DB in OnceLock"))?;
    Ok(())
}

pub fn init_ipv4_asn_db() -> Result<()> {
    // Initialize IPv4 ASN database
    let ipv4_asn_db = Ipv4AsnDb::bundled();
    IPV4_ASN_DB
        .set(RwLock::new(ipv4_asn_db))
        .map_err(|_| anyhow::anyhow!("Failed to set IPV4_ASN_DB in OnceLock"))?;
    Ok(())
}

pub fn init_ipv6_asn_db() -> Result<()> {
    // Initialize IPv6 ASN database
    let ipv6_asn_db = Ipv6AsnDb::bundled();
    IPV6_ASN_DB
        .set(RwLock::new(ipv6_asn_db))
        .map_err(|_| anyhow::anyhow!("Failed to set IPV6_ASN_DB in OnceLock"))?;
    Ok(())
}

pub fn init_ipv4_country_db() -> Result<()> {
    // Initialize IPv4 Country database
    let ipv4_country_db = Ipv4CountryDb::bundled();
    IPV4_COUNTRY_DB
        .set(RwLock::new(ipv4_country_db))
        .map_err(|_| anyhow::anyhow!("Failed to set IPV4_COUNTRY_DB in OnceLock"))?;
    Ok(())
}

pub fn init_ipv6_country_db() -> Result<()> {
    // Initialize IPv6 Country database
    let ipv6_country_db = Ipv6CountryDb::bundled();
    IPV6_COUNTRY_DB
        .set(RwLock::new(ipv6_country_db))
        .map_err(|_| anyhow::anyhow!("Failed to set IPV6_COUNTRY_DB in OnceLock"))?;
    Ok(())
}
