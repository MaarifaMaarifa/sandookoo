use iced::widget::{button, center, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Color, Element, Length, Task};
use sea_orm::DatabaseConnection;

use super::state::{Databases, GuiState, GuiStateError};
use super::style;
use crate::settings::{self, ConnectionProfile};

#[derive(Default)]
pub struct NewConnectionForm {
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub show_password: bool,
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
    TogglePasswordVisibility,
    DatabaseChanged(String),
    Submit,
    ConnectionEstablished(ConnectionProfile, Result<DatabaseConnection, GuiStateError>),
}

impl State {
    /// Builds the panel and, for each connection profile saved from a
    /// previous session, a task that looks up its password in the keychain
    /// and reconnects automatically.
    pub fn new(saved_connections: Vec<ConnectionProfile>) -> (Self, Task<Message>) {
        let reconnects = saved_connections.into_iter().filter_map(|profile| {
            let password = match settings::load_password(&profile.name) {
                Ok(password) => password,
                Err(error) => {
                    tracing::warn!(
                        connection = %profile.name,
                        %error,
                        "failed to load saved password from the keychain; skipping reconnect"
                    );
                    return None;
                }
            };

            Some(Task::perform(
                establish(profile, password),
                |(profile, result)| Message::ConnectionEstablished(profile, result),
            ))
        });

        (Self::default(), Task::batch(reconnects))
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
            Message::TogglePasswordVisibility => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.show_password = !form.show_password;
                }
            }
            Message::DatabaseChanged(value) => {
                if let Some(form) = self.new_connection_form.as_mut() {
                    form.database = value;
                }
            }
            Message::Submit => {
                return match self.validate(shared) {
                    Ok((profile, password)) => {
                        Task::perform(establish(profile, password), |(profile, result)| {
                            Message::ConnectionEstablished(profile, result)
                        })
                    }
                    Err(error) => {
                        self.set_error(error);
                        Task::none()
                    }
                };
            }
            Message::ConnectionEstablished(profile, result) => match result {
                Ok(connection) => {
                    shared.add_connection(profile.name.clone(), connection);
                    shared.remember_connection(profile);
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

    /// Validates the open form and, if valid, returns the connection
    /// profile together with the password to connect with.
    fn validate(&self, shared: &GuiState) -> Result<(ConnectionProfile, String), String> {
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
            ConnectionProfile {
                name,
                host,
                port,
                username: form.username.trim().to_string(),
                database,
            },
            form.password.clone(),
        ))
    }

    pub fn view<'a>(&'a self, shared: &'a GuiState) -> Element<'a, Message> {
        container(self.connections_list(shared))
            .width(Length::Fixed(240.0))
            .height(Length::Fill)
            .padding(style::space::MD)
            .style(style::panel)
            .into()
    }

    /// The "new connection" dialog, to be shown as a modal over the whole
    /// app while the form is open.
    pub fn modal(&self) -> Option<Element<'_, Message>> {
        let form = self.new_connection_form.as_ref()?;

        Some(
            container(new_connection_form_view(form))
                .width(Length::Fixed(360.0))
                .padding(style::space::MD)
                .style(style::panel)
                .into(),
        )
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
                    .style(style::list_item(is_selected))
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

/// Connects and, on success, saves the password to the OS keychain. The
/// password is never carried in a `Message`; it only ever lives in this
/// task's local state.
async fn establish(
    profile: ConnectionProfile,
    password: String,
) -> (ConnectionProfile, Result<DatabaseConnection, GuiStateError>) {
    let result = Databases::connect(profile.clone(), password.clone()).await;

    if result.is_ok() {
        let name = profile.name.clone();
        let saved =
            tokio::task::spawn_blocking(move || settings::save_password(&name, &password)).await;

        if let Ok(Err(error)) = saved {
            tracing::warn!(%error, "failed to save password to the keychain");
        }
    }

    (profile, result)
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
        row![
            text_input("Password", &form.password)
                .secure(!form.show_password)
                .style(style::field)
                .on_input(Message::PasswordChanged),
            button(text(if form.show_password { "Hide" } else { "Show" }))
                .padding([style::space::XS, style::space::SM])
                .style(style::ghost_button)
                .on_press(Message::TogglePasswordVisibility),
        ]
        .spacing(style::space::XS)
        .align_y(Alignment::Center),
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
