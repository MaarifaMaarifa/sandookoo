use iced::widget::{button, center, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Task};
use sea_orm::DatabaseConnection;

use super::state::{DatabaseConfig, Databases, GuiState, GuiStateError};
use super::style;

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

#[derive(Default)]
pub struct State {
    new_connection_form: Option<NewConnectionForm>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Select(String),
    OpenNewConnectionForm,
    CancelNewConnectionForm,
    NameChanged(String),
    HostChanged(String),
    PortChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    DatabaseChanged(String),
    Submit,
    ConnectionEstablished(String, Result<DatabaseConnection, GuiStateError>),
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    /// `shared` is the state this panel reads/mutates in common with the
    /// query panel: the set of open database connections and which one is
    /// currently selected.
    pub fn update(&mut self, message: Message, shared: &mut GuiState) -> Task<Message> {
        match message {
            Message::Select(name) => shared.select_connection(name),
            Message::OpenNewConnectionForm => {
                self.new_connection_form = Some(NewConnectionForm::new())
            }
            Message::CancelNewConnectionForm => self.new_connection_form = None,
            Message::NameChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.name = value;
                }
            }
            Message::HostChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.host = value;
                }
            }
            Message::PortChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.port = value;
                }
            }
            Message::UsernameChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.username = value;
                }
            }
            Message::PasswordChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.password = value;
                }
            }
            Message::DatabaseChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.database = value;
                }
            }
            Message::Submit => {
                return match self.validate(shared) {
                    Ok((name, config)) => {
                        Task::perform(Databases::connect(config), move |result| {
                            Message::ConnectionEstablished(name.clone(), result)
                        })
                    }
                    Err(error) => {
                        self.set_error(error);
                        Task::none()
                    }
                };
            }
            Message::ConnectionEstablished(name, result) => match result {
                Ok(connection) => {
                    shared.add_connection(name, connection);
                    self.new_connection_form = None;
                }
                Err(_) => self.set_error("Failed to connect to the database.".to_string()),
            },
        }

        Task::none()
    }

    fn set_error(&mut self, error: String) {
        if let Some(form) = self.new_connection_form.as_mut() {
            form.error = Some(error);
        }
    }

    /// Validates the open form and, if valid, returns the connection name
    /// together with the config to connect with.
    fn validate(&self, shared: &GuiState) -> Result<(String, DatabaseConfig), String> {
        let form = self
            .new_connection_form
            .as_ref()
            .expect("validate called without an open form");

        let name = form.name.trim().to_string();
        if name.is_empty() {
            return Err("Connection name is required.".to_string());
        }
        if shared.databases().contains(&name) {
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

    pub fn view<'a>(&'a self, shared: &'a GuiState) -> Element<'a, Message> {
        let content = match &self.new_connection_form {
            Some(form) => new_connection_form_view(form),
            None => self.connections_list(shared),
        };

        container(content)
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .padding(style::space::MD)
            .style(style::panel)
            .into()
    }

    fn connections_list<'a>(&'a self, shared: &'a GuiState) -> Element<'a, Message> {
        let names: Vec<Element<'a, Message>> = shared
            .databases()
            .names()
            .map(|name| {
                let is_selected = shared.selected_connection() == Some(name.as_str());
                button(text(name.clone()))
                    .width(Length::Fill)
                    .padding([style::space::SM, style::space::MD])
                    .style(style::connection_item(is_selected))
                    .on_press(Message::Select(name.clone()))
                    .into()
            })
            .collect();

        let list: Element<'a, Message> = if names.is_empty() {
            center(text("No connections yet")).into()
        } else {
            scrollable(column(names).spacing(style::space::XS)).into()
        };

        column![
            row![
                text("Connections").size(16).width(Length::Fill),
                button(text("+ New"))
                    .padding([style::space::XS, style::space::SM])
                    .style(style::primary_button)
                    .on_press(Message::OpenNewConnectionForm),
            ]
            .align_y(Alignment::Center),
            list,
        ]
        .spacing(style::space::MD)
        .into()
    }
}

fn new_connection_form_view(form: &NewConnectionForm) -> Element<'_, Message> {
    let mut content = column![text("New Connection").size(16)].spacing(style::space::SM);

    content = content.push(
        text_input("Name", &form.name)
            .style(style::field)
            .on_input(Message::NameChanged),
    );
    content = content.push(
        text_input("Host", &form.host)
            .style(style::field)
            .on_input(Message::HostChanged),
    );
    content = content.push(
        text_input("Port", &form.port)
            .style(style::field)
            .on_input(Message::PortChanged),
    );
    content = content.push(
        text_input("Username", &form.username)
            .style(style::field)
            .on_input(Message::UsernameChanged),
    );
    content = content.push(
        text_input("Password", &form.password)
            .secure(true)
            .style(style::field)
            .on_input(Message::PasswordChanged),
    );
    content = content.push(
        text_input("Database", &form.database)
            .style(style::field)
            .on_input(Message::DatabaseChanged),
    );

    if let Some(error) = &form.error {
        content = content.push(text(error.clone()).color(Color::from_rgb(0.94, 0.4, 0.4)));
    }

    content = content.push(
        row![
            button(text("Connect"))
                .padding([style::space::XS, style::space::SM])
                .style(style::primary_button)
                .on_press(Message::Submit),
            button(text("Cancel"))
                .padding([style::space::XS, style::space::SM])
                .style(style::ghost_button)
                .on_press(Message::CancelNewConnectionForm),
        ]
        .spacing(style::space::SM),
    );

    scrollable(content).into()
}
