use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config::types::ForwardingRule;
use crate::ssh::session::SshSession;
use crate::ssh::types::{ConnectionStatus, TunnelStatus};

#[derive(Debug, Clone)]
pub struct TunnelError {
    pub rule_id: String,
    pub message: Option<String>,
}

pub struct ConnectionState {
    pub session: Option<SshSession>,
    pub status: ConnectionStatus,
    pub tunnel_statuses: Vec<TunnelStatus>,
    pub tunnel_errors: Arc<Mutex<Vec<TunnelError>>>,
}

impl ConnectionState {
    pub fn new_connecting(rules: &[ForwardingRule]) -> Self {
        let tunnel_errors = rules
            .iter()
            .map(|r| TunnelError {
                rule_id: r.id.clone(),
                message: None,
            })
            .collect();
        Self {
            session: None,
            status: ConnectionStatus::Connecting,
            tunnel_statuses: rules
                .iter()
                .map(|r| TunnelStatus {
                    rule_id: r.id.clone(),
                    active: false,
                    error: None,
                })
                .collect(),
            tunnel_errors: Arc::new(Mutex::new(tunnel_errors)),
        }
    }

    pub fn set_connected(&mut self, session: SshSession, rules: &[ForwardingRule]) {
        self.session = Some(session);
        self.status = ConnectionStatus::Connected;
        self.tunnel_statuses = rules
            .iter()
            .filter(|r| r.enabled)
            .map(|r| TunnelStatus {
                rule_id: r.id.clone(),
                active: true,
                error: None,
            })
            .collect();
    }

    pub fn set_error(&mut self, message: String) {
        self.status = ConnectionStatus::Error { message };
    }

    pub fn set_disconnected(&mut self) {
        if let Some(ref mut session) = self.session {
            session.disconnect();
        }
        self.session = None;
        self.status = ConnectionStatus::Disconnected;
        self.tunnel_statuses.clear();
    }
}

#[derive(Clone)]
pub struct AppState {
    pub connections: Arc<Mutex<HashMap<String, ConnectionState>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::{ForwardingKind, ForwardingRule};

    fn make_rules() -> Vec<ForwardingRule> {
        vec![
            ForwardingRule {
                id: "r1".into(),
                kind: ForwardingKind::Local,
                bind_address: "127.0.0.1".into(),
                bind_port: 8080,
                remote_host: "db".into(),
                remote_port: 5432,
                enabled: true,
            },
            ForwardingRule {
                id: "r2".into(),
                kind: ForwardingKind::Dynamic,
                bind_address: "127.0.0.1".into(),
                bind_port: 1080,
                remote_host: String::new(),
                remote_port: 0,
                enabled: false,
            },
        ]
    }

    #[test]
    fn new_connecting_creates_correct_state() {
        let rules = make_rules();
        let state = ConnectionState::new_connecting(&rules);
        assert_eq!(state.status, ConnectionStatus::Connecting);
        assert!(state.session.is_none());
        assert_eq!(state.tunnel_statuses.len(), 2);
        assert!(!state.tunnel_statuses[0].active);
        assert!(!state.tunnel_statuses[1].active);
    }

    #[test]
    fn set_error_transitions() {
        let state_conn = ConnectionState::new_connecting(&[]);
        let mut state = state_conn;
        state.set_error("timeout".into());
        assert_eq!(
            state.status,
            ConnectionStatus::Error { message: "timeout".into() }
        );
    }

    #[test]
    fn set_disconnected_clears_statuses() {
        let mut state = ConnectionState::new_connecting(&make_rules());
        assert_eq!(state.tunnel_statuses.len(), 2);
        state.set_disconnected();
        assert_eq!(state.status, ConnectionStatus::Disconnected);
        assert!(state.tunnel_statuses.is_empty());
        assert!(state.session.is_none());
    }

    #[test]
    fn app_state_new_is_empty() {
        let app = AppState::new();
        let connections = app.connections.try_lock().unwrap();
        assert!(connections.is_empty());
    }

    #[tokio::test]
    async fn tunnel_errors_initialized_for_rules() {
        let rules = make_rules();
        let state = ConnectionState::new_connecting(&rules);
        let errors = state.tunnel_errors.lock().await;
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].rule_id, "r1");
        assert!(errors[0].message.is_none());
    }
}
