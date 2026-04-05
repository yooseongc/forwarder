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
        if let Some(ref mut session) = conn.session {
            let tunnel_errors = conn.tunnel_errors.clone();
            session.start_single_tunnel(&rule, tunnel_errors).await
                .map_err(|e| AppError::internal(e.to_string()))?;
            conn.tunnel_statuses.push(crate::ssh::types::TunnelStatus {
                rule_id: rule.id.clone(),
                active: true,
                error: None,
            });
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
    let profile = find_profile(id)?;

    {
        let mut connections = state.connections.lock().await;
        connections.insert(
            id.to_string(),
            ConnectionState::new_connecting(&profile.forwarding_rules),
        );
    }
    emit_status(app, id, state).await;

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
                drop(connections);
                emit_status(app, id, state).await;
                return Err(AppError::connection_failed(e.to_string()));
            }

            // Start health check task
            let health_handle = session.start_health_check();
            let app_clone = app.clone();
            let id_str = id.to_string();
            let state_clone = state.inner().clone();
            tokio::spawn(async move {
                health_handle.await.ok();
                // Connection lost — update state
                let mut conns = state_clone.connections.lock().await;
                if let Some(conn) = conns.get_mut(&id_str) {
                    if conn.status == ConnectionStatus::Connected {
                        conn.set_error("Connection lost".to_string());
                        let event = StatusChangeEvent {
                            profile_id: id_str.clone(),
                            status: conn.status.clone(),
                            tunnel_statuses: conn.tunnel_statuses.clone(),
                        };
                        if let Err(e) = app_clone.emit("connection-status-changed", event) {
                            log::warn!("Failed to emit health check event: {}", e);
                        }
                    }
                }
            });

            conn.set_connected(session, &profile.forwarding_rules);
            drop(connections);
            emit_status(app, id, state).await;
            Ok(())
        }
        Err(e) => {
            let mut connections = state.connections.lock().await;
            if let Some(conn) = connections.get_mut(id) {
                conn.set_error(e.to_string());
            }
            drop(connections);
            emit_status(app, id, state).await;
            Err(AppError::connection_failed(e.to_string()))
        }
    }
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
