use std::io::{Read, Write};
use std::net::TcpStream;

use iced::pure::{button, column, row, text, text_input, Element, Sandbox};
use iced::{Alignment, Settings};

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
    socket: TcpStream,
}

#[derive(Debug)]
enum App {
    Disconnected(DisconnectedState),
    ConnectionFailed(ConnectionFailedState),
    Connected(ConnectedState),
}

#[derive(Debug, Clone)]
enum Message {
    AddressChanged(String),
    PortChanged(String),
    RetryConnect,
    Connect,
    ToggleLed,
}

impl Sandbox for App {
    type Message = Message;

    fn new() -> Self {
        Self::Disconnected(DisconnectedState::default())
    }

    fn title(&self) -> String {
        "LED Control".to_string()
    }

    fn update(&mut self, msg: Self::Message) {
        if let Message::Connect = msg {
            take_mut::take(self, |state| {
                if let Self::Disconnected(state) = state {
                    let address = state.address.as_str();
                    let port = if let Ok(port) = state.port.parse::<u16>() {
                        port
                    } else {
                        return Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address,
                            port: state.port,
                            reason: "Invalid port".to_string(),
                        });
                    };

                    match TcpStream::connect((address, port)) {
                        Ok(socket) => Self::Connected(ConnectedState { socket }),
                        Err(e) => Self::ConnectionFailed(ConnectionFailedState {
                            address: state.address,
                            port: state.port,
                            reason: e.to_string(),
                        }),
                    }
                } else {
                    unreachable!()
                }
            });
            return;
        }

        match self {
            Self::Disconnected(state) => match msg {
                Message::AddressChanged(address) => {
                    state.address = address;
                }
                Message::PortChanged(port) => {
                    state.port = port;
                }
                _ => unreachable!(),
            },
            Self::ConnectionFailed(_state) => match msg {
                Message::RetryConnect => *self = Self::new(),
                _ => unreachable!(),
            },
            Self::Connected(state) => match msg {
                Message::ToggleLed => {
                    let written_len = state.socket.write(&[TOGGLE_COMMAND]).unwrap();
                    assert_eq!(written_len, 1);

                    let mut ack = [0];
                    state.socket.read_exact(&mut ack).unwrap();
                    assert_eq!(ack, [ACKNOWLEDGE_COMMAND]);
                }
                _ => unreachable!(),
            },
        }
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
            Self::Connected(state) => {
                let message_input = row().push(button("Toggle LED").on_press(Message::ToggleLed));

                column()
                    .padding(20)
                    .push(text(format!(
                        "Connected to `{}`.",
                        state.socket.peer_addr().unwrap()
                    )))
                    .push(message_input)
                    .into()
            }
        }
    }
}

pub fn main() -> iced::Result {
    App::run(Settings::default())
}
