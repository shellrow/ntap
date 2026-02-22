use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Protocol {
    ARP,
    NDP,
    ICMP,
    TCP,
    UDP,
}
