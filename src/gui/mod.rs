use iced::widget::{
    button, center, column, container, row, scrollable, table, text, text_editor, text_input,
};
use iced::{Alignment, Color, Element, Length, Task};
use sea_orm::DatabaseConnection;

mod gui_state;

pub struct Gui {
    state: gui_state::GuiState,
}

#[derive(Debug, Clone)]
pub enum Message {
    SelectConnection(String),
    QueryEditorAction(text_editor::Action),
    RunQuery,
    OpenNewConnectionForm,
    CancelNewConnectionForm,
    NewConnectionNameChanged(String),
    NewConnectionHostChanged(String),
    NewConnectionPortChanged(String),
    NewConnectionUsernameChanged(String),
    NewConnectionPasswordChanged(String),
    NewConnectionDatabaseChanged(String),
    SubmitNewConnectionForm,
    ConnectionEstablished(String, Result<DatabaseConnection, gui_state::GuiStateError>),
    QueryFinished(gui_state::QueryOutcome),
}

impl Gui {
    pub fn new() -> Self {
        Self {
            state: gui_state::GuiState::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectConnection(name) => self.state.select_connection(name),
            Message::QueryEditorAction(action) => self.state.perform_query_action(action),
            Message::RunQuery => {
                if let Some((connection, sql)) = self.state.start_query() {
                    return Task::perform(
                        gui_state::run_query(connection, sql),
                        Message::QueryFinished,
                    );
                }
            }
            Message::OpenNewConnectionForm => self.state.open_new_connection_form(),
            Message::CancelNewConnectionForm => self.state.cancel_new_connection_form(),
            Message::NewConnectionNameChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.name = value;
                }
            }
            Message::NewConnectionHostChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.host = value;
                }
            }
            Message::NewConnectionPortChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.port = value;
                }
            }
            Message::NewConnectionUsernameChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.username = value;
                }
            }
            Message::NewConnectionPasswordChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.password = value;
                }
            }
            Message::NewConnectionDatabaseChanged(value) => {
                if let Some(form) = self.state.new_connection_form_mut() {
                    form.database = value;
                }
            }
            Message::SubmitNewConnectionForm => {
                return match self.state.validate_new_connection_form() {
                    Ok((name, config)) => Task::perform(
                        gui_state::Databases::connect(config),
                        move |result| Message::ConnectionEstablished(name, result),
                    ),
                    Err(error) => {
                        self.state.set_new_connection_error(error);
                        Task::none()
                    }
                };
            }
            Message::ConnectionEstablished(name, result) => match result {
                Ok(connection) => self.state.finish_new_connection(name, connection),
                Err(_) => self
                    .state
                    .set_new_connection_error("Failed to connect to the database.".to_string()),
            },
            Message::QueryFinished(outcome) => self.state.set_query_outcome(outcome),
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.connections_panel(), self.query_panel()].into()
    }

    fn connections_panel(&self) -> Element<'_, Message> {
        let content = match self.state.new_connection_form() {
            Some(form) => new_connection_form_view(form),
            None => self.connections_list(),
        };

        container(content)
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .padding(8)
            .style(container::bordered_box)
            .into()
    }

    fn connections_list(&self) -> Element<'_, Message> {
        let names: Vec<Element<'_, Message>> = self
            .state
            .get_databases()
            .names()
            .map(|name| {
                let is_selected = self.state.selected_connection() == Some(name.as_str());
                button(text(name.clone()))
                    .width(Length::Fill)
                    .style(if is_selected {
                        button::primary
                    } else {
                        button::secondary
                    })
                    .on_press(Message::SelectConnection(name.clone()))
                    .into()
            })
            .collect();

        let list: Element<'_, Message> = if names.is_empty() {
            center(text("No connections yet")).into()
        } else {
            scrollable(column(names).spacing(4)).into()
        };

        column![
            row![
                text("Connections").size(16).width(Length::Fill),
                button(text("+ New")).on_press(Message::OpenNewConnectionForm),
            ]
            .align_y(Alignment::Center),
            list,
        ]
        .spacing(8)
        .into()
    }

    fn query_panel(&self) -> Element<'_, Message> {
        let editor = container(
            column![
                row![
                    text("Query").size(16),
                    button(text("Run")).on_press(Message::RunQuery)
                ]
                .spacing(8)
                .align_y(Alignment::Center),
                text_editor(self.state.query_editor())
                    .placeholder("Write your query here...")
                    .on_action(Message::QueryEditorAction)
                    .height(Length::Fill),
            ]
            .spacing(8),
        )
        .padding(8)
        .height(Length::FillPortion(2))
        .style(container::bordered_box);

        let results = container(results_view(self.state.query_result()))
            .width(Length::Fill)
            .height(Length::FillPortion(1))
            .padding(8)
            .style(container::bordered_box);

        column![editor, results]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn results_view(outcome: &gui_state::QueryOutcome) -> Element<'_, Message> {
    match outcome {
        gui_state::QueryOutcome::Empty => center(text("Results will appear here.")).into(),
        gui_state::QueryOutcome::Message(message) => center(text(message.clone())).into(),
        gui_state::QueryOutcome::Rows { columns, rows } => {
            if rows.is_empty() {
                return center(text("Query returned no rows.")).into();
            }

            let table_columns = columns.iter().enumerate().map(|(index, name)| {
                table::column(text(name.clone()), move |row: Vec<String>| {
                    text(row[index].clone())
                })
            });

            scrollable(table(table_columns, rows.clone())).into()
        }
    }
}

fn new_connection_form_view(form: &gui_state::NewConnectionForm) -> Element<'_, Message> {
    let mut content = column![text("New Connection").size(16)].spacing(8);

    content = content.push(text_input("Name", &form.name).on_input(Message::NewConnectionNameChanged));
    content = content.push(text_input("Host", &form.host).on_input(Message::NewConnectionHostChanged));
    content = content.push(text_input("Port", &form.port).on_input(Message::NewConnectionPortChanged));
    content = content.push(
        text_input("Username", &form.username).on_input(Message::NewConnectionUsernameChanged),
    );
    content = content.push(
        text_input("Password", &form.password)
            .secure(true)
            .on_input(Message::NewConnectionPasswordChanged),
    );
    content = content.push(
        text_input("Database", &form.database).on_input(Message::NewConnectionDatabaseChanged),
    );

    if let Some(error) = &form.error {
        content = content.push(text(error.clone()).color(Color::from_rgb(0.8, 0.2, 0.2)));
    }

    content = content.push(
        row![
            button(text("Connect")).on_press(Message::SubmitNewConnectionForm),
            button(text("Cancel"))
                .style(button::secondary)
                .on_press(Message::CancelNewConnectionForm),
        ]
        .spacing(8),
    );

    scrollable(content).into()
}
