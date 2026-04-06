use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::config::store;
use crate::config::types::ConnectionProfile;
use crate::credential;
use crate::error::AppError;
use crate::ssh::session::SshSession;
use crate::ssh::types::{ConnectionStatus, ProfileStatus, StatusChangeEvent};
use crate::state::{AppState, ConnectionState};

type CmdResult<T = ()> = Result<T, AppError>;

// ── Profile CRUD ──

#[tauri::command]
pub async fn get_profiles() -> CmdResult<Vec<ConnectionProfile>> {
    store::get_profiles().map_err(|e| AppError::config(e.to_string()))
}

#[tauri::command]
pub async fn save_profile(profile: ConnectionProfile) -> CmdResult {
    store::save_profile(profile).map_err(|e| AppError::config(e.to_string()))
}

#[tauri::command]
pub async fn delete_profile(id: String, state: State<'_, AppState>) -> CmdResult {
    {
        let mut connections = state.connections.lock().await;
        if let Some(mut conn) = connections.remove(&id) {
            conn.set_disconnected();
        }
    }
    let _ = credential::delete_password(&id);
    store::delete_profile(&id).map_err(|e| AppError::config(e.to_string()))
}

// ── Connection management ──

#[tauri::command]
pub async fn connect_profile(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult {
    do_connect(&id, &app, &state).await
}

#[tauri::command]
pub async fn disconnect_profile(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult {
    {
        let mut connections = state.connections.lock().await;
        if let Some(conn) = connections.get_mut(&id) {
            conn.set_disconnected();
        }
    }
    emit_status(&app, &id, &state).await;
    Ok(())
}

#[tauri::command]
pub async fn reconnect_profile(
    id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult {
    // Disconnect first
    {
        let mut connections = state.connections.lock().await;
        if let Some(conn) = connections.get_mut(&id) {
            conn.set_disconnected();
        }
    }
    emit_status(&app, &id, &state).await;

    // Brief pause to allow socket cleanup
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Reconnect
    do_connect(&id, &app, &state).await
}

#[tauri::command]
pub async fn get_all_status(state: State<'_, AppState>) -> CmdResult<Vec<ProfileStatus>> {
    let profiles = store::get_profiles().map_err(|e| AppError::config(e.to_string()))?;
    let connections = state.connections.lock().await;

    let statuses = profiles
        .iter()
        .map(|p| {
            let (status, tunnel_statuses) = match connections.get(&p.id) {
                Some(conn) => (conn.status.clone(), conn.tunnel_statuses.clone()),
                None => (ConnectionStatus::Disconnected, vec![]),
            };
            ProfileStatus {
                profile_id: p.id.clone(),
                profile_name: p.name.clone(),
                status,
                tunnel_statuses,
            }
        })
        .collect();

    Ok(statuses)
}

// ── Credentials ──

#[tauri::command]
pub async fn save_credential(profile_id: String, password: String) -> CmdResult {
    credential::save_password(&profile_id, &password)
        .map_err(|e| AppError::credential(e.to_string()))
}

#[tauri::command]
pub async fn delete_credential(profile_id: String) -> CmdResult {
    credential::delete_password(&profile_id)
        .map_err(|e| AppError::credential(e.to_string()))
}

#[tauri::command]
pub async fn has_credential(profile_id: String) -> CmdResult<bool> {
    credential::has_password(&profile_id)
        .map_err(|e| AppError::credential(e.to_string()))
}

// ── Autostart ──

#[tauri::command]
pub async fn get_autostart_enabled(app: AppHandle) -> CmdResult<bool> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|e| AppError::internal(e.to_string()))
}

#[tauri::command]
pub async fn set_autostart_enabled(app: AppHandle, enabled: bool) -> CmdResult {
    use tauri_plugin_autostart::ManagerExt;
    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| AppError::internal(e.to_string()))
    } else {
        autostart.disable().map_err(|e| AppError::internal(e.to_string()))
    }
}

// ── File dialog ──

#[tauri::command]
pub async fn open_key_file_dialog(app: AppHandle) -> CmdResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("SSH Key Files", &["pem", "key", "pub", "ppk"])
        .add_filter("All Files", &["*"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });
    rx.recv().map_err(|e| AppError::internal(e.to_string()))
}

// ── Ping ──

#[tauri::command]
pub async fn ping_host(host: String, port: u16) -> CmdResult<u64> {
    use std::time::Instant;
    use tokio::net::TcpStream;

    let addr = format!("{}:{}", host, port);
    let start = Instant::now();
    match tokio::time::timeout(
        tokio::time::Duration::from_secs(5),
        TcpStream::connect(&addr),
    )
    .await
    {
        Ok(Ok(_)) => Ok(start.elapsed().as_millis() as u64),
        Ok(Err(e)) => Err(AppError::connection_failed(format!("Connection refused: {}", e))),
        Err(_) => Err(AppError::connection_failed("Timeout (5s)")),
    }
}

// ── Host key management ──

#[tauri::command]
pub async fn reset_host_key(host: String, port: u16) -> CmdResult {
    use crate::ssh::known_hosts;
    known_hosts::remove_host_key(&host, port)
        .map_err(|e| AppError::internal(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub async fn reset_all_host_keys() -> CmdResult {
    use crate::ssh::known_hosts;
    known_hosts::clear_all().map_err(|e| AppError::internal(e.to_string()))
}

// ── Config import/export ──

#[tauri::command]
pub async fn export_config() -> CmdResult<String> {
    let config = store::load_config().map_err(|e| AppError::config(e.to_string()))?;
    serde_json::to_string_pretty(&config).map_err(|e| AppError::config(e.to_string()))
}

#[tauri::command]
pub async fn import_config(json: String) -> CmdResult {
    let imported: crate::config::types::AppConfig =
        serde_json::from_str(&json).map_err(|e| AppError::config(format!("Invalid config JSON: {}", e)))?;

    // Merge: add imported profiles that don't already exist
    let mut config = store::load_config().map_err(|e| AppError::config(e.to_string()))?;
    let existing_ids: std::collections::HashSet<String> =
        config.profiles.iter().map(|p| p.id.clone()).collect();
    for profile in imported.profiles {
        if !existing_ids.contains(&profile.id) {
            config.profiles.push(profile);
        }
    }
    // Save merged config
    store::save_profile_batch(&config.profiles).map_err(|e| AppError::config(e.to_string()))
}

// ── Tunnel toggle ──

#[tauri::command]
pub async fn enable_tunnel(
    profile_id: String,
    rule_id: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> CmdResult {
    let profile = find_profile(&profile_id)?;
    let rule = profile
        .forwarding_rules
        .iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| AppError::internal(format!("Rule not found: {}", rule_id)))?
        .clone();

    let mut connections = state.connections.lock().await;
    if let Some(conn) = connections.get_mut(&profile_id) {
        if conn.status != ConnectionStatus::Connected {
            return Err(AppError::connection_failed("Not connected"));
        }
        if rule.kind == crate::config::types::ForwardingKind::Remote {
            return Err(AppError::internal(
                "Remote forwarding rules cannot be enabled at runtime. Reconnect to apply changes.".to_string(),
            ));
        }
        if let Some(ref mut session) = conn.session {
            // Add tunnel_errors entry for this rule so errors are tracked
            {
                let mut errs = conn.tunnel_errors.lock().await;
                if !errs.iter().any(|t| t.rule_id == rule.id) {
                    errs.push(crate::state::TunnelError {
                        rule_id: rule.id.clone(),
                        message: None,
                    });
                }
            }
            let tunnel_errors = conn.tunnel_errors.clone();
            session.start_single_tunnel(&rule, tunnel_errors).await
                .map_err(|e| AppError::internal(e.to_string()))?;
            if !conn.tunnel_statuses.iter().any(|t| t.rule_id == rule.id) {
                conn.tunnel_statuses.push(crate::ssh::types::TunnelStatus {
                    rule_id: rule.id.clone(),
                    active: true,
                    error: None,
                });
            }
        }
    }
    drop(connections);
    emit_status(&app, &profile_id, &state).await;
    Ok(())
}

// ── Auto-connect on startup ──

pub async fn auto_connect_on_startup(app: AppHandle) -> anyhow::Result<()> {
    let profiles = store::get_profiles()?;
    for profile in profiles.iter().filter(|p| p.auto_connect) {
        let id = profile.id.clone();
        let name = profile.name.clone();
        let app_clone = app.clone();
        log::info!("Auto-connecting profile: {}", name);
        tauri::async_runtime::spawn(async move {
            let state = app_clone.state::<AppState>();
            if let Err(e) = do_connect(&id, &app_clone, &state).await {
                log::error!("Auto-connect failed for {}: {}", name, e);
            }
        });
    }
    Ok(())
}

// ── Internal helpers ──

async fn do_connect(id: &str, app: &AppHandle, state: &State<'_, AppState>) -> CmdResult {
    do_connect_inner(id, app, state.inner()).await
}

async fn do_connect_inner(id: &str, app: &AppHandle, state: &AppState) -> CmdResult {
    // find_profile uses std::sync::Mutex internally; isolate it in a block
    // so the guard is dropped before any .await point.
    let profile = { find_profile(id)? };

    {
        let mut connections = state.connections.lock().await;
        connections.insert(
            id.to_string(),
            ConnectionState::new_connecting(&profile.forwarding_rules),
        );
    }
    emit_status_with(app, id, state).await;

    match SshSession::connect(&profile, &profile.forwarding_rules).await {
        Ok(mut session) => {
            let mut connections = state.connections.lock().await;
            match connections.get_mut(id).map(|c| c.status.clone()) {
                Some(ConnectionStatus::Connecting) => {}
                _ => {
                    session.disconnect();
                    return Ok(());
                }
            }
            let conn = connections.get_mut(id).unwrap();
            let tunnel_errors = conn.tunnel_errors.clone();
            if let Err(e) = session
                .start_tunnels(&profile.forwarding_rules, tunnel_errors)
                .await
            {
                conn.set_error(e.to_string());
                emit_status_inner(app, id, &connections);
                return Err(AppError::connection_failed(e.to_string()));
            }

            // Start health check task
            let health_handle = session.start_health_check();
            let app_clone = app.clone();
            let id_str = id.to_string();
            let state_clone = state.clone();
            tokio::spawn(async move {
                health_handle.await.ok();
                on_connection_lost(id_str, app_clone, state_clone).await;
            });

            conn.set_connected(session, &profile.forwarding_rules);
            emit_status_inner(app, id, &connections);
            Ok(())
        }
        Err(e) => {
            let mut connections = state.connections.lock().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.set_error(e.to_string());
            }
            emit_status_inner(app, id, &connections);
            Err(AppError::connection_failed(e.to_string()))
        }
    }
}

async fn emit_status_with(app: &AppHandle, profile_id: &str, state: &AppState) {
    let connections = state.connections.lock().await;
    emit_status_inner(app, profile_id, &connections);
}

fn find_profile(id: &str) -> CmdResult<ConnectionProfile> {
    store::get_profiles()
        .map_err(|e| AppError::config(e.to_string()))?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::profile_not_found(id))
}

async fn emit_status(app: &AppHandle, profile_id: &str, state: &State<'_, AppState>) {
    let connections = state.connections.lock().await;
    emit_status_inner(app, profile_id, &connections);
}

fn emit_status_inner(
    app: &AppHandle,
    profile_id: &str,
    connections: &HashMap<String, ConnectionState>,
) {
    if let Some(conn) = connections.get(profile_id) {
        let event = StatusChangeEvent {
            profile_id: profile_id.to_string(),
            status: conn.status.clone(),
            tunnel_statuses: conn.tunnel_statuses.clone(),
        };
        if let Err(e) = app.emit("connection-status-changed", event) {
            log::warn!("Failed to emit connection status event: {}", e);
        }
    }
}

/// Reconnect using an already-loaded profile (avoids CONFIG_LOCK in async context).
/// Returns a health check JoinHandle on success so the caller can wire it.
async fn try_reconnect(
    id: &str,
    app: &AppHandle,
    state: &AppState,
    profile: &ConnectionProfile,
) -> Result<tokio::task::JoinHandle<()>, String> {
    // Set Connecting state
    {
        let mut connections = state.connections.lock().await;
        connections.insert(
            id.to_string(),
            ConnectionState::new_connecting(&profile.forwarding_rules),
        );
    }
    emit_status_with(app, id, state).await;

    match SshSession::connect(profile, &profile.forwarding_rules).await {
        Ok(mut session) => {
            let mut connections = state.connections.lock().await;
            match connections.get_mut(id).map(|c| c.status.clone()) {
                Some(ConnectionStatus::Connecting) => {}
                _ => {
                    session.disconnect();
                    return Err("Connection state changed during reconnect".into());
                }
            }
            let conn = connections.get_mut(id).unwrap();
            let tunnel_errors = conn.tunnel_errors.clone();
            if let Err(e) = session
                .start_tunnels(&profile.forwarding_rules, tunnel_errors)
                .await
            {
                conn.set_error(e.to_string());
                emit_status_inner(app, id, &connections);
                return Err(e.to_string());
            }

            let health_handle = session.start_health_check();
            conn.set_connected(session, &profile.forwarding_rules);
            emit_status_inner(app, id, &connections);
            Ok(health_handle)
        }
        Err(e) => {
            let mut connections = state.connections.lock().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.set_error(e.to_string());
            }
            emit_status_inner(app, id, &connections);
            Err(e.to_string())
        }
    }
}

/// Called when health check detects connection loss.
/// Triggers auto-reconnect if the profile has it enabled.
/// Runs as a self-contained loop: reconnect → monitor → reconnect...
async fn on_connection_lost(id: String, app: AppHandle, state: AppState) {
    loop {
        // Set error state
        {
            let mut conns = state.connections.lock().await;
            if let Some(conn) = conns.get_mut(&id) {
                if conn.status != ConnectionStatus::Connected {
                    return; // Already disconnected by user
                }
                conn.set_error("Connection lost".to_string());
                emit_status_inner(&app, &id, &conns);
            } else {
                return;
            }
        }

        // Check if auto-reconnect is enabled
        let id_clone = id.clone();
        let profile = match tokio::task::spawn_blocking(move || {
            store::get_profiles()
                .ok()
                .and_then(|ps| ps.into_iter().find(|p| p.id == id_clone))
        })
        .await
        .ok()
        .flatten()
        {
            Some(p) if p.auto_reconnect => p,
            _ => return,
        };

        // Auto-reconnect with exponential backoff
        const MAX_ATTEMPTS: u32 = 5;
        let mut delay_secs = 1u64;
        let mut reconnected = false;

        for attempt in 1..=MAX_ATTEMPTS {
            log::info!(
                "Auto-reconnect {}/{} for '{}' in {}s",
                attempt, MAX_ATTEMPTS, profile.name, delay_secs
            );

            // Set reconnecting status
            {
                let mut conns = state.connections.lock().await;
                match conns.get_mut(&id).map(|c| &c.status) {
                    Some(ConnectionStatus::Error { .. })
                    | Some(ConnectionStatus::Reconnecting { .. }) => {
                        let conn = conns.get_mut(&id).unwrap();
                        conn.status = ConnectionStatus::Reconnecting { attempt };
                        emit_status_inner(&app, &id, &conns);
                    }
                    _ => return, // User manually disconnected
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;

            // Check if still in reconnecting state
            {
                let conns = state.connections.lock().await;
                if !matches!(
                    conns.get(&id).map(|c| &c.status),
                    Some(ConnectionStatus::Reconnecting { .. })
                ) {
                    return; // User intervened
                }
            }

            match try_reconnect(&id, &app, &state, &profile).await {
                Ok(health_handle) => {
                    log::info!("Auto-reconnect succeeded for '{}'", profile.name);
                    // Wait for health check to detect next disconnection
                    health_handle.await.ok();
                    reconnected = true;
                    break; // Break inner loop, outer loop will handle next disconnect
                }
                Err(e) => {
                    log::warn!(
                        "Auto-reconnect {}/{} failed for '{}': {}",
                        attempt, MAX_ATTEMPTS, profile.name, e
                    );
                }
            }

            delay_secs = (delay_secs * 2).min(30);
        }

        if !reconnected {
            // All attempts exhausted
            log::error!(
                "Auto-reconnect gave up after {} attempts for '{}'",
                MAX_ATTEMPTS, profile.name
            );
            let mut conns = state.connections.lock().await;
            if let Some(conn) = conns.get_mut(&id) {
                if matches!(conn.status, ConnectionStatus::Reconnecting { .. }) {
                    conn.set_error(format!(
                        "Auto-reconnect failed after {} attempts",
                        MAX_ATTEMPTS
                    ));
                    emit_status_inner(&app, &id, &conns);
                }
            }
            return;
        }
        // reconnected=true: outer loop continues — will detect next connection loss
    }
}
