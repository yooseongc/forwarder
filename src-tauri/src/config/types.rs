use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub profiles: Vec<ConnectionProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub forwarding_rules: Vec<ForwardingRule>,
    #[serde(default)]
    pub auto_connect: bool,
}

fn default_port() -> u16 {
    22
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AuthMethod {
    Password,
    KeyFile {
        path: String,
    },
    KeyFileWithPassphrase {
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardingRule {
    pub id: String,
    pub kind: ForwardingKind,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    pub bind_port: u16,
    #[serde(default)]
    pub remote_host: String,
    #[serde(default)]
    pub remote_port: u16,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ForwardingKind {
    Local,
    Remote,
    Dynamic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_config_default_is_empty() {
        let config = AppConfig::default();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn forwarding_kind_camel_case() {
        assert_eq!(serde_json::to_string(&ForwardingKind::Local).unwrap(), r#""local""#);
        assert_eq!(serde_json::to_string(&ForwardingKind::Remote).unwrap(), r#""remote""#);
        assert_eq!(serde_json::to_string(&ForwardingKind::Dynamic).unwrap(), r#""dynamic""#);
    }

    #[test]
    fn auth_method_tagged_enum_serialization() {
        let pw = AuthMethod::Password;
        let json = serde_json::to_value(&pw).unwrap();
        assert_eq!(json["type"], "password");

        let kf = AuthMethod::KeyFile { path: "/home/.ssh/id_rsa".into() };
        let json = serde_json::to_value(&kf).unwrap();
        assert_eq!(json["type"], "keyFile");
        assert_eq!(json["path"], "/home/.ssh/id_rsa");

        let kfp = AuthMethod::KeyFileWithPassphrase { path: "/key".into() };
        let json = serde_json::to_value(&kfp).unwrap();
        assert_eq!(json["type"], "keyFileWithPassphrase");
    }

    #[test]
    fn auth_method_round_trip() {
        let original = AuthMethod::KeyFile { path: "/test".into() };
        let json = serde_json::to_string(&original).unwrap();
        let restored: AuthMethod = serde_json::from_str(&json).unwrap();
        match restored {
            AuthMethod::KeyFile { path } => assert_eq!(path, "/test"),
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn connection_profile_default_port() {
        let json = r#"{
            "id": "1", "name": "test", "host": "h", "username": "u",
            "authMethod": {"type": "password"},
            "forwardingRules": []
        }"#;
        let profile: ConnectionProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.port, 22);
        assert!(!profile.auto_connect);
    }

    #[test]
    fn forwarding_rule_defaults() {
        let json = r#"{
            "id": "r1", "kind": "local", "bindPort": 8080,
            "remoteHost": "localhost", "remotePort": 3306
        }"#;
        let rule: ForwardingRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.bind_address, "127.0.0.1");
        assert!(rule.enabled);
    }

    #[test]
    fn connection_profile_camel_case_fields() {
        let json = serde_json::json!({
            "id": "1", "name": "srv", "host": "h", "port": 2222,
            "username": "u", "authMethod": {"type": "password"},
            "forwardingRules": [{
                "id": "r1", "kind": "local", "bindAddress": "0.0.0.0",
                "bindPort": 8080, "remoteHost": "db", "remotePort": 5432, "enabled": true
            }],
            "autoConnect": true
        });
        let profile: ConnectionProfile = serde_json::from_value(json).unwrap();
        assert_eq!(profile.port, 2222);
        assert!(profile.auto_connect);
        assert_eq!(profile.forwarding_rules.len(), 1);
    }
}
