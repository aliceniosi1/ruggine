use iced::{Application, Command, Element, Settings, Subscription, Theme};
use iced::widget::{Column, Row, Text, TextInput, Button, Container};
use iced::subscription;
use tokio::sync::{Mutex};
use tokio::sync::mpsc::UnboundedReceiver;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use common::Message;

mod network;
use network::{send_message, connect_and_listen};

#[derive(Debug, Clone)]
pub enum MessageGUI {
    LoginInput(String),
    LoginSubmit,
    CreateGroupPopup,
    GroupInputChanged(String),
    ConfirmCreateGroup,
    CancelCreateGroup,
    InvitePopup,
    InviteInputChanged(String),
    ConfirmInvite,
    CancelInvite,
    JoinGroup,
    LeaveGroup,
    SelectGroup(String),
    ChatInput(String),
    SendMessage,
    Received(Message),
    ReceivedRx(Arc<Mutex<UnboundedReceiver<Message>>>),
}

struct ChatGUI {
    username: String,
    login_input: String,
    current_group: Option<String>,
    groups: HashSet<String>,
    joined_groups: HashSet<String>,
    messages: HashMap<String, Vec<String>>,
    chat_input: String,
    group_input: String,
    invite_input: String,
    creating_group: bool,
    inviting_user: bool,
    rx: Option<Arc<Mutex<UnboundedReceiver<Message>>>>,
}

impl Application for ChatGUI {
    type Message = MessageGUI;
    type Executor = iced::executor::Default;
    type Flags = ();
    type Theme = iced::Theme;

    fn new(_flags: ()) -> (Self, Command<MessageGUI>) {
        (
            ChatGUI {
                username: String::new(),
                login_input: String::new(),
                current_group: None,
                groups: HashSet::new(),
                joined_groups: HashSet::new(),
                messages: HashMap::new(),
                chat_input: String::new(),
                group_input: String::new(),
                invite_input: String::new(),
                creating_group: false,
                inviting_user: false,
                rx: None,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String { "Chat GUI Client".into() }
    fn theme(&self) -> Self::Theme { Theme::Dark }

    fn update(&mut self, msg: MessageGUI) -> Command<MessageGUI> {
        match msg {
            MessageGUI::LoginInput(v) => { self.login_input = v; Command::none() }
            MessageGUI::LoginSubmit => {
                let name = self.login_input.trim().to_string();
                if !name.is_empty() {
                    self.username = name.clone();
                    return Command::perform(async move {
                        let rx = connect_and_listen("127.0.0.1:8080").await.expect("connessione fallita");
                        send_message(&Message::Login { username: name }).await;
                        Arc::new(Mutex::new(rx))
                    }, MessageGUI::ReceivedRx);
                }
                Command::none()
            }
            MessageGUI::ReceivedRx(rx) => { self.rx = Some(rx); Command::none() }
            MessageGUI::SelectGroup(group) => { self.current_group = Some(group); Command::none() }
            MessageGUI::ChatInput(v) => { self.chat_input = v; Command::none() }
            MessageGUI::SendMessage => {
                if let Some(group) = &self.current_group {
                    if self.joined_groups.contains(group) {
                        let msg = Message::Text {
                            group: group.clone(), from: self.username.clone(), content: self.chat_input.clone(),
                        };
                        self.chat_input.clear();
                        return Command::perform(async move {
                            send_message(&msg).await;
                        }, |_| MessageGUI::ChatInput(String::new()));
                    }
                }
                Command::none()
            }
            MessageGUI::CreateGroupPopup => { self.creating_group = true; self.group_input.clear(); Command::none() }
            MessageGUI::GroupInputChanged(v) => { self.group_input = v; Command::none() }
            MessageGUI::ConfirmCreateGroup => {
                let group_name = self.group_input.trim().to_string();
                if !group_name.is_empty() {
                    self.groups.insert(group_name.clone());
                    self.current_group = Some(group_name.clone());
                    self.group_input.clear();
                    self.creating_group = false;
                    let username = self.username.clone();
                    return Command::perform(async move {
                        send_message(&Message::Create { group: group_name.clone() }).await;
                        send_message(&Message::Join { username, group: group_name }).await;
                    }, |_| MessageGUI::ChatInput(String::new()));
                }
                Command::none()
            }
            MessageGUI::CancelCreateGroup | MessageGUI::CancelInvite => {
                self.creating_group = false;
                self.inviting_user = false;
                self.group_input.clear();
                self.invite_input.clear();
                Command::none()
            }
            MessageGUI::InvitePopup => { self.inviting_user = true; self.invite_input.clear(); Command::none() }
            MessageGUI::InviteInputChanged(v) => { self.invite_input = v; Command::none() }
            MessageGUI::ConfirmInvite => {
                if let Some(group) = &self.current_group {
                    let user = self.invite_input.trim().to_string();
                    if !user.is_empty() {
                        let group = group.clone();
                        self.invite_input.clear();
                        self.inviting_user = false;
                        return Command::perform(async move {
                            send_message(&Message::Invite { group, user }).await;
                        }, |_| MessageGUI::ChatInput(String::new()));
                    }
                }
                Command::none()
            }
            MessageGUI::JoinGroup => {
                if let Some(group) = &self.current_group {
                    let username = self.username.clone();
                    let group = group.clone();
                    return Command::perform(async move {
                        send_message(&Message::Join { username, group }).await;
                    }, |_| MessageGUI::ChatInput(String::new()));
                }
                Command::none()
            }
            MessageGUI::LeaveGroup => {
                if let Some(group) = &self.current_group {
                    let group = group.clone();
                    self.joined_groups.remove(&group);
                    self.groups.remove(&group);
                    self.current_group = None;
                    return Command::perform(async move {
                        send_message(&Message::Leave { group }).await;
                    }, |_| MessageGUI::ChatInput(String::new()));
                }
                Command::none()
            }
            MessageGUI::Received(msg) => {
                match msg {
                    Message::Text { group, from, content } => {
                        if group != "system" {
                            self.groups.insert(group.clone());
                            self.messages
                                .entry(group.clone())
                                .or_default()
                                .push(format!("<{}>: {}", from, content));
                        }
                    }
                    Message::Invite { group, user } => {
                        if user == self.username && group != "system" {
                            self.groups.insert(group);
                        }
                    }
                    Message::Ack => {
                        if let Some(group) = &self.current_group {
                            let was_joined = self.joined_groups.insert(group.clone());
                            if !was_joined {
                                self.messages.entry(group.clone())
                                    .or_default()
                                    .push("<system>: Sei entrato nel gruppo.".to_string());
                            }
                        }
                    }

                    Message::Error { reason } => {
                        if let Some(group) = &self.current_group {
                            self.messages.entry(group.clone())
                                .or_default()
                                .push(format!("<system>: ERRORE: {}", reason));
                        }
                    }
                    _ => {}
                }
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<MessageGUI> {
        if let Some(rx) = &self.rx {
            let rx = rx.clone();
            subscription::unfold("network", rx, |rx| async move {
                let msg = {
                    let mut guard = rx.lock().await;
                    guard.recv().await
                };
                if let Some(msg) = msg {
                    (MessageGUI::Received(msg), rx)
                } else {
                    (MessageGUI::ChatInput(String::new()), rx)
                }
            })
        } else {
            Subscription::none()
        }
    }

    fn view(&self) -> Element<'_, MessageGUI> {
        if self.username.is_empty() {
            return Column::new()
                .padding(20)
                .spacing(10)
                .push(Text::new("Username:"))
                .push(TextInput::new("Username", &self.login_input).on_input(MessageGUI::LoginInput))
                .push(Button::new(Text::new("Login")).on_press(MessageGUI::LoginSubmit))
                .into();
        }

        let mut sidebar = Column::new()
            .spacing(10)
            .push(Text::new(format!("User: {}", self.username)))
            .push(Button::new(Text::new("Create Group")).on_press(MessageGUI::CreateGroupPopup));

        if self.creating_group {
            sidebar = sidebar
                .push(TextInput::new("Group name", &self.group_input)
                    .on_input(MessageGUI::GroupInputChanged))
                .push(Row::new()
                    .spacing(5)
                    .push(Button::new(Text::new("Confirm")).on_press(MessageGUI::ConfirmCreateGroup))
                    .push(Button::new(Text::new("Cancel")).on_press(MessageGUI::CancelCreateGroup)));
        }

        for group in &self.groups {
            let label = if Some(group) == self.current_group.as_ref() {
                format!("> {}", group)
            } else {
                group.to_string()
            };
            let btn = Button::new(Text::new(label))
                .on_press(MessageGUI::SelectGroup(group.clone()));
            sidebar = sidebar.push(btn);
        }

        let mut right_panel = Column::new().spacing(10);
        if let Some(group) = &self.current_group {
            right_panel = right_panel.push(Text::new(format!("Group: {}", group)));
            if !self.joined_groups.contains(group) {
                right_panel = right_panel.push(Button::new(Text::new("Join Group"))
                    .on_press(MessageGUI::JoinGroup));
            } else {
                if let Some(msgs) = self.messages.get(group) {
                    right_panel = right_panel.push(Column::new().spacing(5).push(
                        msgs.iter().fold(Column::new(), |col, msg| col.push(Text::new(msg))),
                    ));
                }
                if self.inviting_user {
                    right_panel = right_panel.push(TextInput::new("Invite user", &self.invite_input)
                        .on_input(MessageGUI::InviteInputChanged));
                    right_panel = right_panel.push(Row::new()
                        .spacing(5)
                        .push(Button::new(Text::new("Confirm Invite"))
                            .on_press(MessageGUI::ConfirmInvite))
                        .push(Button::new(Text::new("Cancel"))
                            .on_press(MessageGUI::CancelInvite)));
                } else {
                    right_panel = right_panel.push(Row::new()
                        .spacing(10)
                        .push(Button::new(Text::new("Invite User")).on_press(MessageGUI::InvitePopup))
                        .push(Button::new(Text::new("Leave Group")).on_press(MessageGUI::LeaveGroup)));
                }
                right_panel = right_panel.push(TextInput::new("Message", &self.chat_input)
                    .on_input(MessageGUI::ChatInput));
                right_panel = right_panel.push(Button::new(Text::new("Send")).on_press(MessageGUI::SendMessage));
            }
        }

        Row::new()
            .padding(20)
            .spacing(30)
            .push(Container::new(sidebar).width(iced::Length::Fixed(200.0)))
            .push(right_panel)
            .into()
    }
}

fn main() {
    ChatGUI::run(Settings::default());
}
