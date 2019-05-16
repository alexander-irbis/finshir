// finshir: A coroutines-driven Low & Slow traffic sender, written in Rust
// Copyright (C) 2019  Temirkhan Myrzamadi <gymmasssorla@gmail.com>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// For more information see <https://github.com/Gymmasssorla/finshir>.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::{Read, Write};
use std::net::TcpStream as StdSocket;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::{fmt, io};

use humantime::format_duration;
use may::coroutine;
use may::net::TcpStream as MaySocket;
use openssl::ssl::{HandshakeError, SslConnector, SslMethod, SslStream};

use crate::config::SocketConfig;

pub type TryConnectResult = Result<FinshirSocket, TryConnectError>;

#[derive(Debug)]
pub enum FinshirSocket {
    RawTcp(MaySocket),
    Tls(SslStream<MaySocket>),
}

impl FinshirSocket {
    pub fn connect(config: &SocketConfig) -> Self {
        loop {
            match FinshirSocket::try_connect(config) {
                Ok(socket) => {
                    info!("A new socket has been connected.");
                    trace!("A recently connected socket >>> {:?}", socket);

                    return socket;
                }
                Err(err) => {
                    error!(
                        "Failed to connect a socket >>> {}! Retrying the operation after {}...",
                        err,
                        crate::cyan(format_duration(config.connect_periodicity))
                    );
                    coroutine::sleep(config.connect_periodicity);
                }
            }
        }
    }

    pub fn try_connect(config: &SocketConfig) -> TryConnectResult {
        let socket: StdSocket = StdSocket::connect_timeout(
            &config.receiver.recognised_addrs[0],
            config.connect_timeout,
        )
        .map_err(TryConnectError::IoError)?;

        // We send packets quite rarely (the default is 30secs), so the Nagle algorithm
        // doesn't help us
        socket
            .set_nodelay(true)
            .expect("Cannot disable TCP_NODELAY");
        if let Some(val) = config.ip_ttl {
            socket.set_ttl(val).map_err(TryConnectError::IoError)?;
        }
        socket
            .set_write_timeout(Some(config.write_timeout))
            .map_err(TryConnectError::IoError)?;

        // Convert our ordinary StdSocket into MaySocket due to coroutines-related
        // reasons
        let socket = unsafe { MaySocket::from_raw_fd(socket.into_raw_fd()) };

        Ok(if config.use_tls {
            FinshirSocket::Tls(
                SslConnector::builder(SslMethod::tls())
                    .expect("Couldn't connect to OpenSSL")
                    .build()
                    .connect(&config.receiver.host, socket)
                    .map_err(TryConnectError::HandshakeError)?,
            )
        } else {
            FinshirSocket::RawTcp(socket)
        })
    }
}

#[derive(Debug)]
pub enum TryConnectError {
    IoError(io::Error),
    HandshakeError(HandshakeError<MaySocket>),
}

impl Display for TryConnectError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            TryConnectError::IoError(err) => err.fmt(f),
            TryConnectError::HandshakeError(err) => match err {
                HandshakeError::SetupFailure(err) => err.fmt(f),
                HandshakeError::Failure(err) => err.error().fmt(f),
                HandshakeError::WouldBlock(err) => err.error().fmt(f),
            },
        }
    }
}

impl Error for TryConnectError {}

impl Write for FinshirSocket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            FinshirSocket::RawTcp(s) => s.write(buf),
            FinshirSocket::Tls(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            FinshirSocket::RawTcp(s) => s.flush(),
            FinshirSocket::Tls(s) => s.flush(),
        }
    }
}

impl Read for FinshirSocket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            FinshirSocket::RawTcp(s) => s.read(buf),
            FinshirSocket::Tls(s) => s.read(buf),
        }
    }
}

// Test communication between two sockets
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::time::Duration;

    use may::go;
    use may::net::TcpListener;

    use crate::config::ReceiverAddrs;

    use super::*;

    lazy_static! {
        static ref DATA: &'static [u8] =
            "Hello from a world when programmers are white bananas".as_bytes();
        static ref CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
        static ref CONNECT_PERIODICITY: Duration = Duration::from_secs(0);
        static ref WRITE_TIMEOUT: Duration = Duration::from_secs(10);
        static ref IP_TTL: Option<u32> = Some(15);
    }

    #[test]
    fn test_raw_tcp() {
        let server = TcpListener::bind("0.0.0.0:0").expect("Cannot bind TcpListener");

        let config = SocketConfig {
            receiver: ReceiverAddrs::from_str(&server.local_addr().unwrap().to_string()).unwrap(),
            connect_timeout: *CONNECT_TIMEOUT,
            connect_periodicity: *CONNECT_PERIODICITY,
            write_timeout: *WRITE_TIMEOUT,
            use_tls: false,
            ip_ttl: *IP_TTL,
        };

        // The server must receive the same data as it was sent by our client
        let handle = go!(move || {
            let mut buff = vec![0; DATA.len()];

            let (mut conn, _) = server
                .accept()
                .expect("The server couldn't accept a connection");

            conn.read_exact(&mut buff)
                .expect("Cannot read from a client");
            assert_eq!(buff.as_slice(), *DATA, "Received different data");
        });

        let mut client =
            match FinshirSocket::try_connect(&config).expect("FinshirSocket::try_connect failed") {
                FinshirSocket::RawTcp(tcp) => tcp,
                FinshirSocket::Tls(_) => panic!("TLS socket received but raw TCP was expected"),
            };

        // Send all the data to the server and wait until it ends its work
        client.write_all(*DATA).expect("client.write_all failed");
        client.flush().expect("client.flush failed");

        handle.join().unwrap();
    }
}
