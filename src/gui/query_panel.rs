use iced::widget::{button, center, column, container, row, scrollable, table, text, text_editor};
use iced::{Alignment, Element, Length, Task};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};

use super::state::GuiState;
use super::style;

#[derive(Debug, Clone)]
pub enum QueryOutcome {
    Empty,
    Message(String),
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
}

pub struct State {
    query_editor: text_editor::Content,
    query_result: QueryOutcome,
}

#[derive(Debug, Clone)]
pub enum Message {
    EditorAction(text_editor::Action),
    Run,
    Finished(QueryOutcome),
}

impl State {
    pub fn new() -> Self {
        Self {
            query_editor: text_editor::Content::new(),
            query_result: QueryOutcome::Empty,
        }
    }

    /// `shared` provides the connections this panel queries against; it is
    /// only ever read here, since selecting a connection is the connections
    /// panel's job.
    pub fn update(&mut self, message: Message, shared: &GuiState) -> Task<Message> {
        match message {
            Message::EditorAction(action) => self.query_editor.perform(action),
            Message::Run => {
                let sql = self.query_editor.text();
                if sql.trim().is_empty() {
                    self.query_result = QueryOutcome::Message("Write a query first.".to_string());
                    return Task::none();
                }

                let Some(name) = shared.selected_connection() else {
                    self.query_result =
                        QueryOutcome::Message("Select a connection first.".to_string());
                    return Task::none();
                };
                let connection = shared
                    .databases()
                    .get(name)
                    .expect("selected connection always exists in databases")
                    .clone();

                self.query_result = QueryOutcome::Message("Running query...".to_string());
                return Task::perform(run_query(connection, sql), Message::Finished);
            }
            Message::Finished(outcome) => self.query_result = outcome,
        }

        Task::none()
    }

    pub fn view(&self, theme: &iced::Theme) -> Element<'_, Message> {
        let editor = container(
            column![
                row![
                    text("Query").size(16),
                    button(text("Run"))
                        .padding([style::space::XS, style::space::SM])
                        .style(style::primary_button)
                        .on_press(Message::Run)
                ]
                .spacing(style::space::SM)
                .align_y(Alignment::Center),
                text_editor(&self.query_editor)
                    .placeholder("Write your query here...")
                    .on_action(Message::EditorAction)
                    .highlight("sql", style::highlighter_theme(theme))
                    .padding(style::space::SM)
                    .style(style::editor)
                    .height(Length::Fill),
            ]
            .spacing(style::space::SM),
        )
        .padding(style::space::MD)
        .height(Length::FillPortion(2))
        .style(style::panel);

        let results = container(results_view(&self.query_result))
            .width(Length::Fill)
            .height(Length::FillPortion(1))
            .padding(style::space::MD)
            .style(style::panel);

        column![editor, results]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(style::space::MD)
            .into()
    }
}

fn results_view(outcome: &QueryOutcome) -> Element<'_, Message> {
    match outcome {
        QueryOutcome::Empty => center(text("Results will appear here.")).into(),
        QueryOutcome::Message(message) => center(text(message.clone())).into(),
        QueryOutcome::Rows { columns, rows } => {
            if rows.is_empty() {
                return center(text("Query returned no rows.")).into();
            }

            let table_columns = columns.iter().enumerate().map(|(index, name)| {
                table::column(text(name.clone()), move |row: Vec<String>| {
                    text(row[index].clone())
                })
            });

            scrollable(table(table_columns, rows.clone()))
                .direction(scrollable::Direction::Both {
                    vertical: scrollable::Scrollbar::default(),
                    horizontal: scrollable::Scrollbar::default(),
                })
                .into()
        }
    }
}

/// Runs `sql` against `connection` and turns the outcome into something the
/// results panel can render. `SELECT`/`WITH` statements are fetched as rows;
/// anything else is executed and reported as an affected-row count.
async fn run_query(connection: DatabaseConnection, sql: String) -> QueryOutcome {
    let trimmed = sql.trim();
    let lower = trimmed.to_ascii_lowercase();
    let is_select = lower.starts_with("select") || lower.starts_with("with");

    let statement = Statement::from_string(DatabaseBackend::Postgres, trimmed.to_string());

    if is_select {
        match connection.query_all_raw(statement).await {
            Ok(rows) => {
                tracing::info!(rows = rows.len(), "query returned rows");
                rows_to_outcome(rows)
            }
            Err(error) => {
                tracing::warn!(%error, "query failed");
                QueryOutcome::Message(format!("Query failed: {error}"))
            }
        }
    } else {
        match connection.execute_raw(statement).await {
            Ok(result) => {
                let rows_affected = result.rows_affected();
                tracing::info!(rows_affected, "query executed");
                QueryOutcome::Message(format!("{rows_affected} row(s) affected."))
            }
            Err(error) => {
                tracing::warn!(%error, "query failed");
                QueryOutcome::Message(format!("Query failed: {error}"))
            }
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
    try_type!(sea_orm::prelude::Uuid);
    try_type!(sea_orm::prelude::Decimal);
    try_type!(sea_orm::prelude::DateTimeWithTimeZone);
    try_type!(sea_orm::prelude::DateTimeUtc);
    try_type!(sea_orm::prelude::DateTime);
    try_type!(sea_orm::prelude::Date);
    try_type!(sea_orm::prelude::Time);

    "<unreadable>".to_string()
}
