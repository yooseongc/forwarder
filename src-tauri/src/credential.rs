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

    const TEST_PROFILE: &str = "test-credential-forwarder-ci";

    fn cleanup() {
        let _ = delete_password(TEST_PROFILE);
    }

    #[test]
    fn save_and_get_password() {
        cleanup();
        save_password(TEST_PROFILE, "secret123").unwrap();
        let pw = get_password(TEST_PROFILE).unwrap();
        assert_eq!(pw, Some("secret123".to_string()));
        cleanup();
    }

    #[test]
    fn get_nonexistent_returns_none() {
        cleanup();
        let pw = get_password(TEST_PROFILE).unwrap();
        assert_eq!(pw, None);
    }

    #[test]
    fn has_password_reflects_state() {
        cleanup();
        assert!(!has_password(TEST_PROFILE).unwrap());
        save_password(TEST_PROFILE, "pw").unwrap();
        assert!(has_password(TEST_PROFILE).unwrap());
        cleanup();
    }

    #[test]
    fn delete_is_idempotent() {
        cleanup();
        delete_password(TEST_PROFILE).unwrap();
        delete_password(TEST_PROFILE).unwrap();
    }

    #[test]
    fn save_overwrites_existing() {
        cleanup();
        save_password(TEST_PROFILE, "old").unwrap();
        save_password(TEST_PROFILE, "new").unwrap();
        let pw = get_password(TEST_PROFILE).unwrap();
        assert_eq!(pw, Some("new".to_string()));
        cleanup();
    }
}
