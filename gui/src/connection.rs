use thiserror::Error;

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

#[derive(Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("Got no ACK from peer")]
    NoAck,
}

#[derive(Debug)]
pub struct Connection<S> {
    socket: S,
}

impl Connection<TcpStream> {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> eyre::Result<Self> {
        Ok(Self {
            socket: TcpStream::connect(addr).await?,
        })
    }
}

impl<S> Connection<S>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    pub async fn toggle_led(&mut self) -> eyre::Result<LedState> {
        let written_len = self.socket.write(&[TOGGLE_COMMAND]).await?;
        assert_eq!(written_len, 1);

        let mut response = [0; 2];
        self.socket.read_exact(&mut response).await?;

        if response[0] != ACKNOWLEDGE_COMMAND {
            return Err(eyre::Report::new(Error::NoAck));
        }

        Ok(if response[1] == 0 {
            LedState::Off
        } else {
            LedState::On
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn toggle_led_on() {
        let socket = tokio_test::io::Builder::new()
            .write(&[TOGGLE_COMMAND])
            .read(&[ACKNOWLEDGE_COMMAND, 0x01])
            .build();
        let mut conn = Connection { socket };
        let response = conn.toggle_led().await;
        assert!(response.is_ok());
        let new_state = response.unwrap();
        assert_eq!(new_state, LedState::On);
    }

    #[tokio::test]
    async fn toggle_led_off() {
        let socket = tokio_test::io::Builder::new()
            .write(&[TOGGLE_COMMAND])
            .read(&[ACKNOWLEDGE_COMMAND, 0x00])
            .build();
        let mut conn = Connection { socket };
        let response = conn.toggle_led().await;
        assert!(response.is_ok());
        let new_state = response.unwrap();
        assert_eq!(new_state, LedState::Off);
    }

    #[tokio::test]
    async fn toggle_led_no_response() {
        let socket = tokio_test::io::Builder::new()
            .write(&[TOGGLE_COMMAND])
            .read(&[])
            .build();
        let mut conn = Connection { socket };
        let response = conn.toggle_led().await;
        assert!(response.is_err());
        let report = response.unwrap_err();
        assert_eq!(
            report.downcast::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[tokio::test]
    async fn toggle_led_no_ack() {
        let socket = tokio_test::io::Builder::new()
            .write(&[TOGGLE_COMMAND])
            .read(&[0x00, 0x00])
            .build();
        let mut conn = Connection { socket };
        let response = conn.toggle_led().await;
        assert!(response.is_err());
        let report = response.unwrap_err();
        assert_eq!(report.downcast::<Error>().unwrap(), Error::NoAck);
    }
}
