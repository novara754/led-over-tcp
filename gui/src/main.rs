mod connection;

use std::sync::Arc;

use connection::{Connection, LedState};
use iced::pure::{button, column, row, text, text_input, Application, Element};
use iced::{Alignment, Command, Settings};

use tokio::sync::Mutex;

#[derive(Default, Debug)]
struct DisconnectedState {
    address: String,
    port: String,
}

#[derive(Debug)]
struct ConnectionFailedState {
    address: String,
    port: String,
    reason: eyre::Report,
}

#[derive(Debug)]
struct ConnectedState {
    connection: Arc<Mutex<Connection>>,
    led_state: LedState,
}

#[derive(Debug)]
enum App {
    Disconnected(DisconnectedState),
    ConnectionFailed(ConnectionFailedState),
    Connected(ConnectedState),
}

#[derive(Debug)]
enum Message {
    AddressChanged(String),
    PortChanged(String),
    Connect,
    RetryConnect,
    Connected(eyre::Result<Connection>),
    ToggleLed,
    ToggledLed(eyre::Result<LedState>),
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Message::AddressChanged(address) => Message::AddressChanged(address.clone()),
            Message::PortChanged(port) => Message::PortChanged(port.clone()),
            Message::Connect => Message::Connect,
            Message::RetryConnect => Message::RetryConnect,
            Message::Connected(_result) => {
                panic!("Message::clone: Cannot clone Message::Connected")
            }
            Message::ToggleLed => Message::ToggleLed,
            Message::ToggledLed(_result) => {
                panic!("Message::clone: Cannot clone Message::ToggledLed")
            }
        }
    }
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self::Disconnected(DisconnectedState::default()),
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "LED Control".to_string()
    }

    fn update(&mut self, msg: Self::Message) -> Command<Message> {
        match self {
            Self::Disconnected(state) => match msg {
                Message::AddressChanged(address) => {
                    state.address = address;
                }
                Message::PortChanged(port) => {
                    state.port = port;
                }
                Message::Connect => {
                    let go = || -> eyre::Result<_> {
                        let address = state.address.clone();
                        let port = state.port.parse::<u16>()?;
                        Ok(Command::perform(
                            Connection::connect((address, port)),
                            Message::Connected,
                        ))
                    };
                    if let Ok(cmd) = go() {
                        return cmd;
                    } else {
                        *self = Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address.clone(),
                            port: state.port.clone(),
                            reason: eyre::Report::msg("Invalid port"),
                        });
                    };
                }
                Message::Connected(socket) => {
                    *self = match socket {
                        Ok(socket) => Self::Connected(ConnectedState {
                            connection: Arc::new(Mutex::new(socket)),
                            led_state: LedState::Off,
                        }),
                        Err(e) => Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address.clone(),
                            port: state.port.clone(),
                            reason: e,
                        }),
                    }
                }
                _ => unreachable!(),
            },
            Self::ConnectionFailed(_state) => match msg {
                Message::RetryConnect => *self = Self::Disconnected(DisconnectedState::default()),
                _ => unreachable!(),
            },
            Self::Connected(state) => match msg {
                Message::ToggleLed => {
                    let conn = state.connection.clone();
                    return Command::perform(
                        async move {
                            let mut conn = conn.lock().await;
                            conn.toggle_led().await
                        },
                        Message::ToggledLed,
                    );
                }
                Message::ToggledLed(r) => match r {
                    Ok(led_state) => state.led_state = led_state,
                    Err(e) => {
                        eprintln!("ToggledLed: {:?}", e);
                    }
                },
                _ => unreachable!(),
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        match self {
            Self::Disconnected(state) => {
                let address_input: Element<_> = row()
                    .push(text("Address"))
                    .push(text_input(
                        "127.0.0.1",
                        &state.address,
                        Message::AddressChanged,
                    ))
                    .into();
                let port_input: Element<_> = row()
                    .push(text("Port"))
                    .push(text_input("1234", &state.port, Message::PortChanged))
                    .into();

                column()
                    .padding(20)
                    .align_items(Alignment::Center)
                    .push(address_input)
                    .push(port_input)
                    .push(button("Connect").on_press(Message::Connect))
                    .into()
            }
            Self::ConnectionFailed(state) => column()
                .padding(20)
                .push(text(format!(
                    "Connected to `{}:{}` failed.\nReason: {}",
                    state.address, state.port, state.reason
                )))
                .push(button("Back").on_press(Message::RetryConnect))
                .into(),
            Self::Connected(state) => {
                let message_input = row().push(button("Toggle LED").on_press(Message::ToggleLed));

                column()
                    .padding(20)
                    .push(text("Connected."))
                    .push(text(format!("LED: {}", state.led_state)))
                    .push(message_input)
                    .into()
            }
        }
    }
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
