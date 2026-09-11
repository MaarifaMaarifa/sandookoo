use std::collections::HashMap;

use iced::widget::text_editor;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuiStateError {
    DatabaseConnectionError,
}

#[derive(Debug, Clone)]
pub enum QueryOutcome {
    Empty,
    Message(String),
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

#[derive(Default)]
pub struct NewConnectionForm {
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub error: Option<String>,
}

impl NewConnectionForm {
    fn new() -> Self {
        Self {
            host: "localhost".to_string(),
            port: "5432".to_string(),
            ..Self::default()
        }
    }
}

pub struct GuiState {
    databases: Databases,
    selected_connection: Option<String>,
    query_editor: text_editor::Content,
    query_result: QueryOutcome,
    new_connection_form: Option<NewConnectionForm>,
}

impl GuiState {
    pub fn new() -> Self {
        Self {
            databases: Databases::new(),
            selected_connection: None,
            query_editor: text_editor::Content::new(),
            query_result: QueryOutcome::Empty,
            new_connection_form: None,
        }
    }

    pub fn get_databases(&self) -> &Databases {
        &self.databases
    }

    pub fn selected_connection(&self) -> Option<&str> {
        self.selected_connection.as_deref()
    }

    pub fn select_connection(&mut self, name: String) {
        self.selected_connection = Some(name);
    }

    pub fn query_editor(&self) -> &text_editor::Content {
        &self.query_editor
    }

    pub fn perform_query_action(&mut self, action: text_editor::Action) {
        self.query_editor.perform(action);
    }

    pub fn query_result(&self) -> &QueryOutcome {
        &self.query_result
    }

    pub fn set_query_outcome(&mut self, outcome: QueryOutcome) {
        self.query_result = outcome;
    }

    /// Picks up the connection to query and the SQL to run, if a connection
    /// is selected and the editor isn't empty. Also sets an immediate
    /// "running" message so the results panel gives feedback right away.
    pub fn start_query(&mut self) -> Option<(DatabaseConnection, String)> {
        let sql = self.query_editor.text();
        if sql.trim().is_empty() {
            self.query_result = QueryOutcome::Message("Write a query first.".to_string());
            return None;
        }

        let Some(name) = self.selected_connection.as_ref() else {
            self.query_result = QueryOutcome::Message("Select a connection first.".to_string());
            return None;
        };
        let connection = self
            .databases
            .get(name)
            .expect("selected connection always exists in databases")
            .clone();

        self.query_result = QueryOutcome::Message("Running query...".to_string());
        Some((connection, sql))
    }

    pub fn new_connection_form(&self) -> Option<&NewConnectionForm> {
        self.new_connection_form.as_ref()
    }

    pub fn new_connection_form_mut(&mut self) -> Option<&mut NewConnectionForm> {
        self.new_connection_form.as_mut()
    }

    pub fn open_new_connection_form(&mut self) {
        self.new_connection_form = Some(NewConnectionForm::new());
    }

    pub fn cancel_new_connection_form(&mut self) {
        self.new_connection_form = None;
    }

    pub fn set_new_connection_error(&mut self, error: String) {
        if let Some(form) = self.new_connection_form.as_mut() {
            form.error = Some(error);
        }
    }

    /// Validates the open form and, if valid, returns the connection name
    /// together with the config to connect with.
    pub fn validate_new_connection_form(&self) -> Result<(String, DatabaseConfig), String> {
        let form = self
            .new_connection_form
            .as_ref()
            .expect("validate_new_connection_form called without an open form");

        let name = form.name.trim().to_string();
        if name.is_empty() {
            return Err("Connection name is required.".to_string());
        }
        if self.databases.contains(&name) {
            return Err("A connection with this name already exists.".to_string());
        }

        let host = form.host.trim().to_string();
        if host.is_empty() {
            return Err("Host is required.".to_string());
        }

        let port: u16 = form
            .port
            .trim()
            .parse()
            .map_err(|_| "Port must be a number between 0 and 65535.".to_string())?;

        let database = form.database.trim().to_string();
        if database.is_empty() {
            return Err("Database name is required.".to_string());
        }

        Ok((
            name,
            DatabaseConfig {
                host,
                port,
                username: form.username.trim().to_string(),
                password: form.password.clone(),
                database,
            },
        ))
    }

    /// Registers a newly established connection and closes the form.
    pub fn finish_new_connection(&mut self, name: String, connection: DatabaseConnection) {
        self.databases.insert(name.clone(), connection);
        self.selected_connection = Some(name);
        self.new_connection_form = None;
    }
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

/// Runs `sql` against `connection` and turns the outcome into something the
/// results panel can render. `SELECT`/`WITH` statements are fetched as rows;
/// anything else is executed and reported as an affected-row count.
pub async fn run_query(connection: DatabaseConnection, sql: String) -> QueryOutcome {
    let trimmed = sql.trim();
    let lower = trimmed.to_ascii_lowercase();
    let is_select = lower.starts_with("select") || lower.starts_with("with");

    let statement = Statement::from_string(DatabaseBackend::Postgres, trimmed.to_string());

    if is_select {
        match connection.query_all_raw(statement).await {
            Ok(rows) => rows_to_outcome(rows),
            Err(error) => QueryOutcome::Message(format!("Query failed: {error}")),
        }
    } else {
        match connection.execute_raw(statement).await {
            Ok(result) => {
                QueryOutcome::Message(format!("{} row(s) affected.", result.rows_affected()))
            }
            Err(error) => QueryOutcome::Message(format!("Query failed: {error}")),
        }
    }
}

fn rows_to_outcome(rows: Vec<sea_orm::QueryResult>) -> QueryOutcome {
    let columns = rows
        .first()
        .map(sea_orm::QueryResult::column_names)
        .unwrap_or_default();

    let rows = rows
        .iter()
        .map(|row| {
            (0..columns.len())
                .map(|index| cell_to_string(row, index))
                .collect()
        })
        .collect();

    QueryOutcome::Rows { columns, rows }
}

/// Best-effort, type-erased stringification of a cell: tries the common
/// column types in turn since `sea_orm` has no single "get as string" API.
fn cell_to_string(row: &sea_orm::QueryResult, index: usize) -> String {
    macro_rules! try_type {
        ($ty:ty) => {
            if let Ok(value) = row.try_get_by_index::<Option<$ty>>(index) {
                return value.map_or_else(|| "NULL".to_string(), |value| value.to_string());
            }
        };
    }

    try_type!(String);
    try_type!(i64);
    try_type!(i32);
    try_type!(i16);
    try_type!(f64);
    try_type!(f32);
    try_type!(bool);

    "<unreadable>".to_string()
}
