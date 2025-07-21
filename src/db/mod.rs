use std::sync::{OnceLock, RwLock};
use anyhow::Result;
use ndb_oui::OuiDb;
use ndb_tcp_service::TcpServiceDb;
use ndb_udp_service::UdpServiceDb;

pub mod oui;
pub mod service;

pub static OUI_DB: OnceLock<RwLock<OuiDb>> = OnceLock::new();
pub static TCP_SERVICE_DB: OnceLock<RwLock<TcpServiceDb>> = OnceLock::new();
pub static UDP_SERVICE_DB: OnceLock<RwLock<UdpServiceDb>> = OnceLock::new();

pub fn init_oui_db() -> Result<()> {
    // Initialize OUI database
    let oui_db = oui::get_oui_db()?;
    OUI_DB.set(RwLock::new(oui_db)).map_err(|_| {
        anyhow::anyhow!("Failed to set OUI_DB in OnceLock")
    })?;
    Ok(())
}

pub fn init_tcp_service_db() -> Result<()> {
    // Initialize TCP Service database
    let tcp_service_db = service::get_tcp_service_db()?;
    TCP_SERVICE_DB.set(RwLock::new(tcp_service_db)).map_err(|_| {
        anyhow::anyhow!("Failed to set TCP_SERVICE_DB in OnceLock")
    })?;
    Ok(())
}

pub fn init_udp_service_db() -> Result<()> {
    // Initialize UDP Service database
    let udp_service_db = service::get_udp_service_db()?;
    UDP_SERVICE_DB.set(RwLock::new(udp_service_db)).map_err(|_| {
        anyhow::anyhow!("Failed to set UDP_SERVICE_DB in OnceLock")
    })?;
    Ok(())
}
