use anyhow::Result;
use ndb_tcp_service::TcpServiceDb;
use ndb_udp_service::UdpServiceDb;
use std::sync::{OnceLock, RwLock};

pub static TCP_SERVICE_DB: OnceLock<RwLock<TcpServiceDb>> = OnceLock::new();
pub static UDP_SERVICE_DB: OnceLock<RwLock<UdpServiceDb>> = OnceLock::new();

pub fn init_databases() -> Result<()> {
    tracing::info!("Initializing databases...");
    init_tcp_service_db()?;
    init_udp_service_db()?;
    tracing::info!("Databases initialized successfully.");
    Ok(())
}

pub fn init_tcp_service_db() -> Result<()> {
    let tcp_service_db = TcpServiceDb::bundled();
    if TCP_SERVICE_DB.get().is_none() {
        TCP_SERVICE_DB
            .set(RwLock::new(tcp_service_db))
            .map_err(|_| anyhow::anyhow!("Failed to set TCP_SERVICE_DB in OnceLock"))?;
    }
    Ok(())
}

pub fn init_udp_service_db() -> Result<()> {
    let udp_service_db = UdpServiceDb::bundled();
    if UDP_SERVICE_DB.get().is_none() {
        UDP_SERVICE_DB
            .set(RwLock::new(udp_service_db))
            .map_err(|_| anyhow::anyhow!("Failed to set UDP_SERVICE_DB in OnceLock"))?;
    }
    Ok(())
}
