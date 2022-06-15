use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, ToSocketAddrs};

const TOGGLE_COMMAND: u8 = 0xAA;
const ACKNOWLEDGE_COMMAND: u8 = 0x06;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedState {
    On,
    Off,
}

impl std::fmt::Display for LedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Self::On => "on",
                Self::Off => "off",
            }
        )
    }
}

#[derive(Debug)]
pub struct Connection {
    socket: TcpStream,
}

impl Connection {
    pub fn new(socket: TcpStream) -> Self {
        Self { socket }
    }

    pub async fn connect<A: ToSocketAddrs>(addr: A) -> eyre::Result<Self> {
        Ok(Self {
            socket: TcpStream::connect(addr).await?,
        })
    }

    pub async fn toggle_led(&mut self) -> eyre::Result<LedState> {
        let written_len = self.socket.write(&[TOGGLE_COMMAND]).await?;
        assert_eq!(written_len, 1);

        let mut response = [0; 2];
        self.socket.read_exact(&mut response).await?;
        assert_eq!(response[0], ACKNOWLEDGE_COMMAND);

        Ok(if response[1] == 0 {
            LedState::Off
        } else {
            LedState::On
        })
    }
}
