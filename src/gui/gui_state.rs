use std::collections::HashMap;

use sea_orm::DatabaseConnection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiStateError {
    DatabaseAlreadyExists,
    DatabaseConnectionError,
}

pub struct GuiState {
    databases: Databases,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            databases: Databases::new(),
        }
    }

    pub fn get_databases(&self) -> &Databases {
        &self.databases
    }
}

pub struct DatabaseConfig {
    pub name: String,
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

    pub async fn add_database(
        &mut self,
        name: String,
        database_config: DatabaseConfig,
    ) -> Result<(), GuiStateError> {
        if self.databases.contains_key(&name) {
            return Err(GuiStateError::DatabaseAlreadyExists);
        }

        // build postgres string
        let postgres_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            database_config.username,
            database_config.password,
            database_config.host,
            database_config.port,
            database_config.database
        );

        // connect to database
        let db = sea_orm::Database::connect(postgres_url)
            .await
            .map_err(|_| GuiStateError::DatabaseConnectionError)?;

        self.databases.insert(name, db);

        Ok(())
    }
}
