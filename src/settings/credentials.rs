use keyring::Entry;

/// The keychain service name every connection's password is stored under;
/// the account is the connection's name.
const SERVICE: &str = "Sandookoo";

#[derive(Debug, thiserror::Error)]
pub enum CredentialError {
    #[error("keychain access failed: {0}")]
    Keyring(#[from] keyring::Error),
}

pub fn save_password(connection_name: &str, password: &str) -> Result<(), CredentialError> {
    Entry::new(SERVICE, connection_name)?.set_password(password)?;
    Ok(())
}

pub fn load_password(connection_name: &str) -> Result<String, CredentialError> {
    Ok(Entry::new(SERVICE, connection_name)?.get_password()?)
}
