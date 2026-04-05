use super::types::{AppConfig, ConnectionProfile};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialize all config file access to prevent concurrent read-modify-write races.
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

fn config_dir() -> Result<PathBuf> {
    let dir = if let Ok(override_dir) = std::env::var("FORWARDER_CONFIG_DIR") {
        PathBuf::from(override_dir)
    } else {
        dirs::config_dir()
            .context("Failed to find config directory")?
            .join("forwarder")
    };
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn config_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("config.json"))
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let data = std::fs::read_to_string(&path)?;
    match serde_json::from_str::<AppConfig>(&data) {
        Ok(config) => Ok(config),
        Err(e) => {
            // Config is corrupted — back up and reset
            let backup = path.with_extension("json.bak");
            log::error!(
                "Config file is corrupted ({}). Backing up to {:?} and resetting.",
                e,
                backup
            );
            let _ = std::fs::copy(&path, &backup);
            Ok(AppConfig::default())
        }
    }
}

pub fn get_profiles() -> Result<Vec<ConnectionProfile>> {
    Ok(load_config()?.profiles)
}

pub fn save_profile(profile: ConnectionProfile) -> Result<()> {
    let _lock = CONFIG_LOCK.lock().map_err(|e| anyhow::anyhow!("Config lock poisoned: {}", e))?;
    let mut config = load_config()?;
    if let Some(existing) = config.profiles.iter_mut().find(|p| p.id == profile.id) {
        *existing = profile;
    } else {
        config.profiles.push(profile);
    }
    let path = config_path()?;
    let data = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn save_profile_batch(profiles: &[ConnectionProfile]) -> Result<()> {
    let _lock = CONFIG_LOCK.lock().map_err(|e| anyhow::anyhow!("Config lock poisoned: {}", e))?;
    let config = AppConfig {
        profiles: profiles.to_vec(),
    };
    let path = config_path()?;
    let data = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, data)?;
    Ok(())
}

pub fn delete_profile(id: &str) -> Result<()> {
    let _lock = CONFIG_LOCK.lock().map_err(|e| anyhow::anyhow!("Config lock poisoned: {}", e))?;
    let mut config = load_config()?;
    config.profiles.retain(|p| p.id != id);
    let path = config_path()?;
    let data = serde_json::to_string_pretty(&config)?;
    std::fs::write(&path, data)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::types::AuthMethod;

    use std::sync::Mutex as StdMutex;
    /// Serialize config tests since they share FORWARDER_CONFIG_DIR env var.
    static TEST_LOCK: StdMutex<()> = StdMutex::new(());

    fn with_temp_config<F: FnOnce()>(f: F) {
        let _lock = TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("FORWARDER_CONFIG_DIR", dir.path()) };
        f();
        unsafe { std::env::remove_var("FORWARDER_CONFIG_DIR") };
    }

    fn make_profile(id: &str, name: &str) -> ConnectionProfile {
        ConnectionProfile {
            id: id.into(),
            name: name.into(),
            host: "192.168.1.1".into(),
            port: 22,
            username: "user".into(),
            auth_method: AuthMethod::Password,
            forwarding_rules: vec![],
            auto_connect: false,
        }
    }

    #[test]
    fn load_empty_dir_returns_default() {
        with_temp_config(|| {
            let config = load_config().unwrap();
            assert!(config.profiles.is_empty());
        });
    }

    #[test]
    fn save_and_load_round_trip() {
        with_temp_config(|| {
            let p = make_profile("1", "Server A");
            save_profile(p.clone()).unwrap();
            let profiles = get_profiles().unwrap();
            assert_eq!(profiles.len(), 1);
            assert_eq!(profiles[0].name, "Server A");
        });
    }

    #[test]
    fn save_profile_updates_existing() {
        with_temp_config(|| {
            save_profile(make_profile("1", "Old")).unwrap();
            save_profile(make_profile("1", "New")).unwrap();
            let profiles = get_profiles().unwrap();
            assert_eq!(profiles.len(), 1);
            assert_eq!(profiles[0].name, "New");
        });
    }

    #[test]
    fn delete_profile_removes_it() {
        with_temp_config(|| {
            save_profile(make_profile("1", "A")).unwrap();
            save_profile(make_profile("2", "B")).unwrap();
            delete_profile("1").unwrap();
            let profiles = get_profiles().unwrap();
            assert_eq!(profiles.len(), 1);
            assert_eq!(profiles[0].id, "2");
        });
    }

    #[test]
    fn save_profile_batch_overwrites_all() {
        with_temp_config(|| {
            save_profile(make_profile("1", "Old")).unwrap();
            let new_profiles = vec![make_profile("2", "X"), make_profile("3", "Y")];
            save_profile_batch(&new_profiles).unwrap();
            let profiles = get_profiles().unwrap();
            assert_eq!(profiles.len(), 2);
            assert_eq!(profiles[0].id, "2");
        });
    }

    #[test]
    fn corrupted_json_backs_up_and_resets() {
        with_temp_config(|| {
            let path = config_path().unwrap();
            std::fs::write(&path, "{ invalid json!!!").unwrap();
            let config = load_config().unwrap();
            assert!(config.profiles.is_empty());
            // Backup file should exist
            assert!(path.with_extension("json.bak").exists());
        });
    }
}
