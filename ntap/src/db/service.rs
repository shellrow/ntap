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
    pub fn load() -> Result<ServiceDatabase, Box<dyn std::error::Error>> {
        let mut service_db = ServiceDatabase::new();
        service_db.load_tcp_map()?;
        service_db.load_udp_map()?;
        Ok(service_db)
    }
    pub fn load_tcp_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tcp_map = ntap_db_tcp_service::get_map();
        Ok(())
    }
    pub fn load_udp_map(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.udp_map = ntap_db_udp_service::get_map();
        Ok(())
    }
}
