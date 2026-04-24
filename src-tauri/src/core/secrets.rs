//! OS-keychain-backed storage for provider API keys.
//!
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: Secret Service (gnome-keyring / kwallet)

use keyring::Entry;

use super::error::{CoreError, CoreResult};

const SERVICE: &str = "ProcessFox";

fn entry_for(provider: &str) -> CoreResult<Entry> {
    let account = format!("provider:{provider}");
    Entry::new(SERVICE, &account).map_err(|e| CoreError::Keyring(e.to_string()))
}

pub fn set_api_key(provider: &str, value: &str) -> CoreResult<()> {
    let entry = entry_for(provider)?;
    entry
        .set_password(value)
        .map_err(|e| CoreError::Keyring(e.to_string()))
}

pub fn has_api_key(provider: &str) -> CoreResult<bool> {
    let entry = entry_for(provider)?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(CoreError::Keyring(e.to_string())),
    }
}

pub fn get_api_key(provider: &str) -> CoreResult<Option<String>> {
    let entry = entry_for(provider)?;
    match entry.get_password() {
        Ok(v) => Ok(Some(v)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(CoreError::Keyring(e.to_string())),
    }
}

pub fn clear_api_key(provider: &str) -> CoreResult<()> {
    let entry = entry_for(provider)?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(CoreError::Keyring(e.to_string())),
    }
}
