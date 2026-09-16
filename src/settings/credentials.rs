use keyring::Entry;

use crate::APP_NAME;

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("keychain access failed: {0}")]
    Keyring(#[from] keyring::Error),
}

/// Every connection's password is stored under the app's keychain service;
/// the account is the connection's name.
pub fn save_password(connection_name: &str, password: &str) -> Result<(), CredentialError> {
    Entry::new(APP_NAME, connection_name)?.set_password(password)?;
    Ok(())
}

pub fn load_password(connection_name: &str) -> Result<String, CredentialError> {
    Ok(Entry::new(APP_NAME, connection_name)?.get_password()?)
}
