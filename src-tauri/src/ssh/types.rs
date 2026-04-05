use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileStatus {
    pub profile_id: String,
    pub profile_name: String,
    pub status: ConnectionStatus,
    pub tunnel_statuses: Vec<TunnelStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelStatus {
    pub rule_id: String,
    pub active: bool,
    pub error: Option<String>,
}

/// Event payload emitted to the frontend
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusChangeEvent {
    pub profile_id: String,
    pub status: ConnectionStatus,
    pub tunnel_statuses: Vec<TunnelStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_status_simple_variants() {
        assert_eq!(serde_json::to_string(&ConnectionStatus::Disconnected).unwrap(), r#""disconnected""#);
        assert_eq!(serde_json::to_string(&ConnectionStatus::Connecting).unwrap(), r#""connecting""#);
        assert_eq!(serde_json::to_string(&ConnectionStatus::Connected).unwrap(), r#""connected""#);
    }

    #[test]
    fn connection_status_error_variant() {
        let status = ConnectionStatus::Error { message: "timeout".into() };
        let json: serde_json::Value = serde_json::to_value(&status).unwrap();
        assert_eq!(json["error"]["message"], "timeout");
    }

    #[test]
    fn tunnel_status_serialization() {
        let ts = TunnelStatus { rule_id: "r1".into(), active: true, error: None };
        let json: serde_json::Value = serde_json::to_value(&ts).unwrap();
        assert_eq!(json["ruleId"], "r1");
        assert_eq!(json["active"], true);
        assert!(json["error"].is_null());
    }

    #[test]
    fn tunnel_status_with_error() {
        let ts = TunnelStatus { rule_id: "r2".into(), active: false, error: Some("bind failed".into()) };
        let json: serde_json::Value = serde_json::to_value(&ts).unwrap();
        assert_eq!(json["error"], "bind failed");
    }

    #[test]
    fn profile_status_serialization() {
        let ps = ProfileStatus {
            profile_id: "p1".into(),
            profile_name: "Server".into(),
            status: ConnectionStatus::Connected,
            tunnel_statuses: vec![],
        };
        let json: serde_json::Value = serde_json::to_value(&ps).unwrap();
        assert_eq!(json["profileId"], "p1");
        assert_eq!(json["profileName"], "Server");
        assert_eq!(json["status"], "connected");
    }
}
