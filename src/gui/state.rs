use std::collections::HashMap;

use sea_orm::DatabaseConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiStateError {
    DatabaseConnectionError,
}

pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
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

    pub async fn connect(config: DatabaseConfig) -> Result<DatabaseConnection, GuiStateError> {
        let postgres_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        sea_orm::Database::connect(postgres_url)
            .await
            .map_err(|_| GuiStateError::DatabaseConnectionError)
    }
}

/// State shared across every panel. Kept intentionally small: only data
/// more than one panel needs to read or mutate belongs here.
pub struct GuiState {
    databases: Databases,
    selected_connection: Option<String>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            databases: Databases::new(),
            selected_connection: None,
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
        self.databases.insert(name.clone(), connection);
        self.selected_connection = Some(name);
    }
}
