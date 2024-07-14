#![allow(unused)]

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NotificationType {
    Traffic,
    RemoteHost,
    Protocol,
}

impl NotificationType {
    pub fn name(&self) -> String {
        match self {
            NotificationType::Traffic => "Traffic".to_string(),
            NotificationType::RemoteHost => "Remote Host".to_string(),
            NotificationType::Protocol => "Protocol".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Notification {
    pub title: String,
    pub body: String,
    pub notification_type: NotificationType,
    pub timestamp: String,
}
