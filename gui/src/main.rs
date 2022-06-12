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
struct ConnectedState {
    socket: TcpStream,
}

#[derive(Debug)]
enum App {
    Disconnected(DisconnectedState),
    Connected(ConnectedState),
}

#[derive(Debug, Clone)]
enum Message {
    AddressChanged(String),
    PortChanged(String),
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
                    let address = state.address;
                    let port: u16 = state.port.parse().unwrap();
                    Self::Connected(ConnectedState {
                        socket: TcpStream::connect((address, port)).unwrap(),
                    })
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
            Self::Connected(state) => {
                let message_input = row().push(button("Toggle LED").on_press(Message::ToggleLed));

                column()
                    .padding(20)
                    .push(text(format!(
                        "Connected to {}.",
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
