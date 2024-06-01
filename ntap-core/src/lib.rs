pub fn load_db() {
    println!("ntap_db_as {}", ntap_db_as::get_map().len());
    println!("ntap_db_country {}", ntap_db_country::get_map().len());
    println!("ntap_db_ipv4_asn {}", ntap_db_ipv4_asn::get_map().len());
    println!("ntap_db_ipv4_country {}", ntap_db_ipv4_country::get_map().len());
    println!("ntap_db_ipv6_asn {}", ntap_db_ipv6_asn::get_map().len());
    println!("ntap_db_ipv6_country {}", ntap_db_ipv6_country::get_map().len());
    println!("ntap_db_oui {}", ntap_db_oui::get_map().len());
    println!("ntap_db_tcp_service {}", ntap_db_tcp_service::get_map().len());
    println!("ntap_db_udp_service {}", ntap_db_udp_service::get_map().len());
}
