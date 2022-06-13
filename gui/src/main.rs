use std::sync::Arc;

use iced::pure::{button, column, row, text, text_input, Application, Element};
use iced::{Alignment, Command, Settings};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

const TOGGLE_COMMAND: u8 = 0xAA;
const ACKNOWLEDGE_COMMAND: u8 = 0x06;

#[derive(Default, Debug)]
struct DisconnectedState {
    address: String,
    port: String,
}

#[derive(Debug)]
struct ConnectionFailedState {
    address: String,
    port: String,
    reason: String,
}

#[derive(Debug)]
struct ConnectedState {
    socket: Arc<Mutex<TcpStream>>,
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
    Connected(tokio::io::Result<tokio::net::TcpStream>),
    ToggleLed,
    ToggledLed(Result<(), String>),
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
            Message::ToggledLed(r) => Message::ToggledLed(r.clone()),
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
                            TcpStream::connect((address, port)),
                            Message::Connected,
                        ))
                    };
                    if let Ok(cmd) = go() {
                        return cmd;
                    } else {
                        *self = Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address.clone(),
                            port: state.port.clone(),
                            reason: "Invalid port".to_string(),
                        });
                    };
                }
                Message::Connected(socket) => {
                    *self = match socket {
                        Ok(socket) => Self::Connected(ConnectedState {
                            socket: Arc::new(Mutex::new(socket)),
                        }),
                        Err(e) => Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address.clone(),
                            port: state.port.clone(),
                            reason: e.to_string(),
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
                    return Command::perform(
                        send_toggle_command(state.socket.clone()),
                        Message::ToggledLed,
                    );
                }
                Message::ToggledLed(_e) => {}
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
                    .push(text("Address"))
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
            Self::Connected(_state) => {
                let message_input = row().push(button("Toggle LED").on_press(Message::ToggleLed));

                column()
                    .padding(20)
                    .push(text("Connected."))
                    .push(message_input)
                    .into()
            }
        }
    }
}

async fn send_toggle_command(socket: Arc<Mutex<TcpStream>>) -> Result<(), String> {
    let mut socket = socket.lock().await;

    let written_len = socket
        .write(&[TOGGLE_COMMAND])
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(written_len, 1);

    let mut ack = [0];
    socket
        .read_exact(&mut ack)
        .await
        .map_err(|e| e.to_string())?;
    assert_eq!(ack, [ACKNOWLEDGE_COMMAND]);
    Ok(())
}

fn main() -> iced::Result {
    App::run(Settings::default())
}
