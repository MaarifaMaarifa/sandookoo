use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

mod credentials;

pub use credentials::{load_password, save_password};

/// The app's local preferences: everything that isn't a live database
/// connection. Grows as more settings are added; every field must have a
/// sensible default so an older settings file on disk still loads after a
/// new field is introduced.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub theme: String,
    pub connections: Vec<ConnectionProfile>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "Catppuccin Mocha".to_string(),
            connections: Vec::new(),
        }
    }
}

/// A saved connection's non-secret metadata. The password never lives here;
/// it's kept in the OS keychain, looked up by `name`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub database: String,
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("could not determine the settings directory")]
    NoConfigDir,
    #[error("failed to access settings file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize settings: {0}")]
    Serialize(#[from] toml::ser::Error),
}

impl Settings {
    /// Loads settings from disk, falling back to defaults if the file is
    /// missing, unreadable, or fails to parse.
    pub fn load() -> Self {
        path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_default()
    }

    /// Inserts or replaces a connection profile by name, then saves.
    pub fn upsert_connection(&mut self, profile: ConnectionProfile) -> Result<(), SettingsError> {
        match self.connections.iter_mut().find(|p| p.name == profile.name) {
            Some(existing) => *existing = profile,
            None => self.connections.push(profile),
        }

        self.save()
    }

    /// Persists settings to disk, creating the config directory if needed.
    pub fn save(&self) -> Result<(), SettingsError> {
        let path = path().ok_or(SettingsError::NoConfigDir)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn path() -> Option<PathBuf> {
    ProjectDirs::from("", "", crate::APP_NAME).map(|dirs| dirs.config_dir().join("settings.toml"))
}
