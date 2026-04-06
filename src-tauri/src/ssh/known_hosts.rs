use anyhow::{Context, Result};
use russh_keys::{key, PublicKeyBase64};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

/// Serialize known_hosts file access.
static KNOWN_HOSTS_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, PartialEq)]
pub enum KeyStatus {
    /// Key matches stored entry.
    Trusted,
    /// No entry for this host — first connection.
    New,
    /// Stored key does not match the server's key.
    Changed,
}

fn known_hosts_path() -> Result<PathBuf> {
    Ok(crate::config::store::config_dir()?.join("known_hosts"))
}

/// Format a host entry key like OpenSSH: `[host]:port` (brackets only if port != 22).
fn host_entry(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{}]:{}", host, port)
    }
}

/// Format a known_hosts line: `host_entry key_type base64_key`
fn format_line(host: &str, port: u16, key: &key::PublicKey) -> String {
    format!(
        "{} {} {}",
        host_entry(host, port),
        key.name(),
        key.public_key_base64()
    )
}

// ── Internal functions (no locking) ──

fn verify_inner(host: &str, port: u16, server_key: &key::PublicKey) -> Result<KeyStatus> {
    let path = known_hosts_path()?;
    if !path.exists() {
        return Ok(KeyStatus::New);
    }

    let contents = fs::read_to_string(&path).context("Failed to read known_hosts")?;
    let entry = host_entry(host, port);
    let server_b64 = server_key.public_key_base64();

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, ' ');
        let Some(stored_host) = parts.next() else {
            continue;
        };
        let Some(_key_type) = parts.next() else {
            continue;
        };
        let Some(stored_b64) = parts.next() else {
            continue;
        };

        if stored_host == entry {
            return if stored_b64.trim() == server_b64 {
                Ok(KeyStatus::Trusted)
            } else {
                Ok(KeyStatus::Changed)
            };
        }
    }

    Ok(KeyStatus::New)
}

fn add_inner(host: &str, port: u16, key: &key::PublicKey) -> Result<()> {
    let path = known_hosts_path()?;
    let line = format_line(host, port, key);

    let mut contents = if path.exists() {
        fs::read_to_string(&path).unwrap_or_default()
    } else {
        String::new()
    };

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&line);
    contents.push('\n');

    fs::write(&path, contents).context("Failed to write known_hosts")
}

fn remove_inner(host: &str, port: u16) -> Result<bool> {
    let path = known_hosts_path()?;
    if !path.exists() {
        return Ok(false);
    }

    let contents = fs::read_to_string(&path).context("Failed to read known_hosts")?;
    let entry = host_entry(host, port);
    let mut removed = false;

    let filtered: Vec<&str> = contents
        .lines()
        .filter(|line| {
            let line_host = line.split_whitespace().next().unwrap_or("");
            if line_host == entry {
                removed = true;
                false
            } else {
                true
            }
        })
        .collect();

    let mut out = filtered.join("\n");
    if !out.is_empty() {
        out.push('\n');
    }
    fs::write(&path, out).context("Failed to write known_hosts")?;
    Ok(removed)
}

fn clear_inner() -> Result<()> {
    let path = known_hosts_path()?;
    if path.exists() {
        fs::write(&path, "").context("Failed to clear known_hosts")?;
    }
    Ok(())
}

// ── Public API (with locking) ──

fn lock() -> Result<std::sync::MutexGuard<'static, ()>> {
    KNOWN_HOSTS_LOCK
        .lock()
        .map_err(|e| anyhow::anyhow!("known_hosts lock poisoned: {}", e))
}

/// Verify a server's host key against the known_hosts file.
#[allow(dead_code)]
pub fn verify_host_key(host: &str, port: u16, server_key: &key::PublicKey) -> Result<KeyStatus> {
    let _lock = lock()?;
    verify_inner(host, port, server_key)
}

/// Verify and auto-store new keys (TOFU). Returns Err on Changed.
pub fn verify_or_store(host: &str, port: u16, server_key: &key::PublicKey) -> Result<KeyStatus> {
    let _lock = lock()?;
    let status = verify_inner(host, port, server_key)?;
    if status == KeyStatus::New {
        add_inner(host, port, server_key)?;
    }
    Ok(status)
}

/// Remove a specific host's key from known_hosts.
pub fn remove_host_key(host: &str, port: u16) -> Result<bool> {
    let _lock = lock()?;
    remove_inner(host, port)
}

/// Remove all entries from the known_hosts file.
pub fn clear_all() -> Result<()> {
    let _lock = lock()?;
    clear_inner()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_temp_dir(f: impl FnOnce()) {
        let _lock = crate::ENV_TEST_LOCK.lock().unwrap();
        let dir = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("FORWARDER_CONFIG_DIR", dir.path()) };
        f();
        unsafe { std::env::remove_var("FORWARDER_CONFIG_DIR") };
    }

    fn make_key_1() -> key::PublicKey {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
        key::PublicKey::Ed25519(secret.verifying_key())
    }

    fn make_key_2() -> key::PublicKey {
        let secret = ed25519_dalek::SigningKey::from_bytes(&[2u8; 32]);
        key::PublicKey::Ed25519(secret.verifying_key())
    }

    #[test]
    fn new_host_returns_new() {
        with_temp_dir(|| {
            let status = verify_inner("example.com", 22, &make_key_1()).unwrap();
            assert_eq!(status, KeyStatus::New);
        });
    }

    #[test]
    fn add_then_verify_returns_trusted() {
        with_temp_dir(|| {
            add_inner("example.com", 22, &make_key_1()).unwrap();
            let status = verify_inner("example.com", 22, &make_key_1()).unwrap();
            assert_eq!(status, KeyStatus::Trusted);
        });
    }

    #[test]
    fn changed_key_returns_changed() {
        with_temp_dir(|| {
            add_inner("example.com", 22, &make_key_1()).unwrap();
            let status = verify_inner("example.com", 22, &make_key_2()).unwrap();
            assert_eq!(status, KeyStatus::Changed);
        });
    }

    #[test]
    fn non_default_port_uses_brackets() {
        with_temp_dir(|| {
            add_inner("example.com", 2222, &make_key_1()).unwrap();

            assert_eq!(
                verify_inner("example.com", 2222, &make_key_1()).unwrap(),
                KeyStatus::Trusted
            );
            assert_eq!(
                verify_inner("example.com", 22, &make_key_1()).unwrap(),
                KeyStatus::New
            );

            let path = known_hosts_path().unwrap();
            let contents = fs::read_to_string(path).unwrap();
            assert!(contents.contains("[example.com]:2222"));
        });
    }

    #[test]
    fn remove_host_key_works() {
        with_temp_dir(|| {
            add_inner("a.com", 22, &make_key_1()).unwrap();
            add_inner("b.com", 22, &make_key_1()).unwrap();

            let removed = remove_inner("a.com", 22).unwrap();
            assert!(removed);

            assert_eq!(verify_inner("a.com", 22, &make_key_1()).unwrap(), KeyStatus::New);
            assert_eq!(verify_inner("b.com", 22, &make_key_1()).unwrap(), KeyStatus::Trusted);
        });
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        with_temp_dir(|| {
            let removed = remove_inner("nope.com", 22).unwrap();
            assert!(!removed);
        });
    }

    #[test]
    fn clear_all_removes_everything() {
        with_temp_dir(|| {
            add_inner("a.com", 22, &make_key_1()).unwrap();
            add_inner("b.com", 22, &make_key_1()).unwrap();
            clear_inner().unwrap();

            assert_eq!(verify_inner("a.com", 22, &make_key_1()).unwrap(), KeyStatus::New);
            assert_eq!(verify_inner("b.com", 22, &make_key_1()).unwrap(), KeyStatus::New);
        });
    }

    #[test]
    fn host_entry_format_test() {
        assert_eq!(host_entry("example.com", 22), "example.com");
        assert_eq!(host_entry("example.com", 2222), "[example.com]:2222");
        assert_eq!(host_entry("192.168.1.1", 22), "192.168.1.1");
        assert_eq!(host_entry("192.168.1.1", 443), "[192.168.1.1]:443");
    }

    #[test]
    fn multiple_hosts_independent() {
        with_temp_dir(|| {
            add_inner("server1.com", 22, &make_key_1()).unwrap();
            add_inner("server2.com", 22, &make_key_2()).unwrap();

            assert_eq!(verify_inner("server1.com", 22, &make_key_1()).unwrap(), KeyStatus::Trusted);
            assert_eq!(verify_inner("server2.com", 22, &make_key_2()).unwrap(), KeyStatus::Trusted);
            assert_eq!(verify_inner("server1.com", 22, &make_key_2()).unwrap(), KeyStatus::Changed);
        });
    }

    #[test]
    fn verify_or_store_adds_new_key() {
        with_temp_dir(|| {
            let status = verify_or_store("fresh.com", 22, &make_key_1()).unwrap();
            assert_eq!(status, KeyStatus::New);

            // Now it should be trusted
            let status = verify_inner("fresh.com", 22, &make_key_1()).unwrap();
            assert_eq!(status, KeyStatus::Trusted);
        });
    }
}
