use crate::db;
use std::collections::HashMap;

/// In-memory service database with hash map
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDatabase {
    pub tcp_map: HashMap<u16, String>,
    pub udp_map: HashMap<u16, String>,
}

impl ServiceDatabase {
    pub fn new() -> ServiceDatabase {
        ServiceDatabase {
            tcp_map: HashMap::new(),
            udp_map: HashMap::new(),
        }
    }
    #[cfg(feature = "bundle")]
    pub fn load() -> Result<ServiceDatabase, Box<dyn std::error::Error>> {
        let mut service_db = ServiceDatabase::new();
        service_db.load_tcp_map()?;
        service_db.load_udp_map()?;
        Ok(service_db)
    }
    #[cfg(not(feature = "bundle"))]
    pub fn load() -> Result<ServiceDatabase, Box<dyn std::error::Error>> {
        let mut service_db = ServiceDatabase::new();
        service_db.load_tcp_file()?;
        service_db.load_udp_file()?;
        Ok(service_db)
    }
    #[cfg(feature = "bundle")]
    pub fn load_tcp_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tcp_map = ntap_db_tcp_service::get_map();
        Ok(())
    }
    #[cfg(feature = "bundle")]
    pub fn load_udp_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.udp_map = ntap_db_udp_service::get_map();
        Ok(())
    }
    pub fn load_tcp_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match crate::sys::get_database_dir_path() {
            Some(mut db_dir) => {
                db_dir.push(ntap_db_tcp_service::TCP_SERVICE_BIN_NAME);
                self.tcp_map = ntap_db_tcp_service::get_map_from_file(db_dir);
            }
            None => {
                eprintln!("Error: Could not get database directory path");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not get database directory path",
                )));
            }
        }
        Ok(())
    }
    pub fn load_udp_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match crate::sys::get_database_dir_path() {
            Some(mut db_dir) => {
                db_dir.push(ntap_db_udp_service::UDP_SERVICE_BIN_NAME);
                self.udp_map = ntap_db_udp_service::get_map_from_file(db_dir);
            }
            None => {
                eprintln!("Error: Could not get database directory path");
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not get database directory path",
                )));
            }
        }
        Ok(())
    }
}
