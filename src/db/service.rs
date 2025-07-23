use anyhow::Result;
use ndb_tcp_service::TcpServiceDb;
use ndb_udp_service::UdpServiceDb;

#[cfg(not(feature = "bundled"))]
use std::path::PathBuf;

pub const TCP_SERVICE_CSV_NAME: &str = "tcp.csv";
pub const TCP_SERVICE_R2_URL: &str = "https://r2.ntap.io/tcp-services.csv";

pub const UDP_SERVICE_CSV_NAME: &str = "udp.csv";
pub const UDP_SERVICE_R2_URL: &str = "https://r2.ntap.io/udp-services.csv";

#[cfg(not(feature = "bundled"))]
pub fn get_tcp_db_filepath() -> Option<PathBuf> {
    match crate::sys::get_database_dir_path() {
        Some(mut db_dir) => {
            db_dir.push(TCP_SERVICE_CSV_NAME);
            Some(db_dir)
        }
        None => {
            eprintln!("Error: Could not get database directory path");
            None
        }
    }
}

#[cfg(not(feature = "bundled"))]
pub fn get_udp_db_filepath() -> Option<PathBuf> {
    match crate::sys::get_database_dir_path() {
        Some(mut db_dir) => {
            db_dir.push(UDP_SERVICE_CSV_NAME);
            Some(db_dir)
        }
        None => {
            eprintln!("Error: Could not get database directory path");
            None
        }
    }
}

#[cfg(feature = "bundled")]
pub fn get_tcp_service_db() -> Result<TcpServiceDb> {
    Ok(TcpServiceDb::bundled())
}

#[cfg(not(feature = "bundled"))]
pub fn get_tcp_service_db() -> Result<TcpServiceDb> {
    let path = get_tcp_db_filepath()
        .ok_or_else(|| anyhow::anyhow!("Failed to get TCP database file path"))?;
    let reader = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open TCP database file: {}", e))?;
    TcpServiceDb::from_csv(reader)
}

#[cfg(feature = "bundled")]
pub fn get_udp_service_db() -> Result<UdpServiceDb> {
    Ok(UdpServiceDb::bundled())
}

#[cfg(not(feature = "bundled"))]
pub fn get_udp_service_db() -> Result<UdpServiceDb> {
    let path = get_udp_db_filepath()
        .ok_or_else(|| anyhow::anyhow!("Failed to get UDP database file path"))?;
    let reader = std::fs::File::open(path)
        .map_err(|e| anyhow::anyhow!("Failed to open UDP database file: {}", e))?;
    UdpServiceDb::from_csv(reader)
}

/* /// In-memory service database with hash map
pub struct ServiceDatabase {
    pub tcp: TcpServiceDb,
    pub udp: UdpServiceDb,
}

impl ServiceDatabase {
    #[cfg(feature = "bundled")]
    pub fn load() -> Result<ServiceDatabase> {
        let db = ServiceDatabase {
            tcp: TcpServiceDb::bundled(),
            udp: UdpServiceDb::bundled(),
        };
        Ok(db)
    }
    #[cfg(not(feature = "bundled"))]
    pub fn load() -> Result<ServiceDatabase> {
        use std::fs::File;
        let tcp_path = get_tcp_db_filepath().ok_or_else(|| {
            anyhow::anyhow!("Failed to get TCP database file path")
        })?;
        let udp_path = get_udp_db_filepath().ok_or_else(|| {
            anyhow::anyhow!("Failed to get UDP database file path")
        })?;
        let mut tcp_reader = File::open(tcp_path)
            .map_err(|e| anyhow::anyhow!("Failed to open TCP database file: {}", e))?;
        let mut udp_reader = File::open(udp_path)
            .map_err(|e| anyhow::anyhow!("Failed to open UDP database file: {}", e))?;
        let db = ServiceDatabase {
            tcp: TcpServiceDb::from_csv(tcp_reader)?,
            udp: UdpServiceDb::from_csv(udp_reader)?,
        };
        Ok(db)
    }
} */
