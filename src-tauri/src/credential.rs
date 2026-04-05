use anyhow::{Context, Result};
use keyring::Entry;

const SERVICE_NAME: &str = "ssh-forwarder";

fn entry_for(profile_id: &str) -> Result<Entry> {
    Entry::new(SERVICE_NAME, profile_id).context("Failed to create keyring entry")
}

pub fn save_password(profile_id: &str, password: &str) -> Result<()> {
    entry_for(profile_id)?.set_password(password)?;
    Ok(())
}

pub fn get_password(profile_id: &str) -> Result<Option<String>> {
    match entry_for(profile_id)?.get_password() {
        Ok(pw) => Ok(Some(pw)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn delete_password(profile_id: &str) -> Result<()> {
    match entry_for(profile_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}

pub fn has_password(profile_id: &str) -> Result<bool> {
    Ok(get_password(profile_id)?.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test uses a unique profile ID to avoid race conditions
    // when tests run in parallel against the shared OS credential store.
    fn test_key(name: &str) -> String {
        format!("test-forwarder-ci-{}", name)
    }

    #[test]
    fn save_and_get_password() {
        let key = test_key("save-get");
        let _ = delete_password(&key);
        save_password(&key, "secret123").unwrap();
        let pw = get_password(&key).unwrap();
        assert_eq!(pw, Some("secret123".to_string()));
        let _ = delete_password(&key);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let key = test_key("nonexistent");
        let _ = delete_password(&key);
        let pw = get_password(&key).unwrap();
        assert_eq!(pw, None);
    }

    #[test]
    fn has_password_reflects_state() {
        let key = test_key("has-pw");
        let _ = delete_password(&key);
        assert!(!has_password(&key).unwrap());
        save_password(&key, "pw").unwrap();
        assert!(has_password(&key).unwrap());
        let _ = delete_password(&key);
    }

    #[test]
    fn delete_is_idempotent() {
        let key = test_key("delete-idempotent");
        delete_password(&key).unwrap();
        delete_password(&key).unwrap();
    }

    #[test]
    fn save_overwrites_existing() {
        let key = test_key("overwrite");
        let _ = delete_password(&key);
        save_password(&key, "old").unwrap();
        save_password(&key, "new").unwrap();
        let pw = get_password(&key).unwrap();
        assert_eq!(pw, Some("new".to_string()));
        let _ = delete_password(&key);
    }
}
