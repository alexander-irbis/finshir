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
