use std::collections::HashMap;

use sea_orm::DatabaseConnection;

use crate::settings::{ConnectionProfile, Settings};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiStateError {
    DatabaseConnectionError,
}

pub struct Databases {
    databases: HashMap<String, DatabaseConnection>,
}

impl Databases {
    pub fn new() -> Self {
        Self {
            databases: HashMap::new(),
        }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.databases.contains_key(name)
    }

    pub fn insert(&mut self, name: String, connection: DatabaseConnection) {
        self.databases.insert(name, connection);
    }

    pub fn get(&self, name: &str) -> Option<&DatabaseConnection> {
        self.databases.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.databases.keys()
    }

    pub async fn connect(
        profile: ConnectionProfile,
        password: String,
    ) -> Result<DatabaseConnection, GuiStateError> {
        let postgres_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            profile.username, password, profile.host, profile.port, profile.database
        );

        sea_orm::Database::connect(postgres_url).await.map_err(|error| {
            tracing::warn!(connection = %profile.name, host = %profile.host, %error, "failed to connect to database");
            GuiStateError::DatabaseConnectionError
        })
    }
}

/// State shared across every panel. Kept intentionally small: only data
/// more than one panel needs to read or mutate belongs here.
pub struct GuiState {
    databases: Databases,
    selected_connection: Option<String>,
    settings: Settings,
    /// Resolved from `settings.editor_font`, and re-resolved whenever it
    /// changes — unlike the UI font, this one applies live.
    editor_font: iced::Font,
}

impl GuiState {
    pub fn new(settings: Settings) -> Self {
        let editor_font =
            super::style::resolve_font(settings.editor_font.as_deref(), iced::Font::MONOSPACE);

        Self {
            databases: Databases::new(),
            selected_connection: None,
            settings,
            editor_font,
        }
    }

    pub fn editor_font(&self) -> iced::Font {
        self.editor_font
    }

    /// Persists the UI font choice. Takes effect on next launch: see the
    /// comment in `main.rs` for why it can't apply live.
    pub fn set_ui_font(&mut self, name: Option<String>) {
        self.settings.ui_font = name;
        if let Err(error) = self.settings.save() {
            tracing::warn!(%error, "failed to save UI font setting");
        }
    }

    /// Persists the editor font choice and applies it immediately.
    pub fn set_editor_font(&mut self, name: Option<String>) {
        self.editor_font = super::style::resolve_font(name.as_deref(), iced::Font::MONOSPACE);
        self.settings.editor_font = name;
        if let Err(error) = self.settings.save() {
            tracing::warn!(%error, "failed to save editor font setting");
        }
    }

    pub fn databases(&self) -> &Databases {
        &self.databases
    }

    pub fn selected_connection(&self) -> Option<&str> {
        self.selected_connection.as_deref()
    }

    pub fn select_connection(&mut self, name: String) {
        self.selected_connection = Some(name);
    }

    /// Registers a newly established connection and makes it the selected one.
    pub fn add_connection(&mut self, name: String, connection: DatabaseConnection) {
        tracing::info!(connection = %name, "connected to database");
        self.databases.insert(name.clone(), connection);
        self.selected_connection = Some(name);
    }

    /// Saves a connection's metadata so it reconnects automatically next
    /// launch. Errors are non-fatal: the connection still works this
    /// session even if it can't be persisted.
    pub fn remember_connection(&mut self, profile: ConnectionProfile) {
        if let Err(error) = self.settings.upsert_connection(profile) {
            tracing::warn!(%error, "failed to save connection settings");
        }
    }

    /// Updates and persists the selected theme.
    pub fn set_theme(&mut self, theme: &iced::Theme) {
        self.settings.theme = theme.to_string();
        if let Err(error) = self.settings.save() {
            tracing::warn!(%error, "failed to save theme setting");
        }
    }
}
