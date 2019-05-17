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
use std::fmt::{self, Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::num::{NonZeroUsize, ParseIntError};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use humantime::parse_duration;
use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
#[structopt(
    author = "Temirkhan Myrzamadi <gymmasssorla@gmail.com>",
    about = "A coroutines-driven Low & Slow traffic sender, written in Rust",
    after_help = "By default, Finshir generates 100 empty spaces as data portions. If you want to \
                  override this behaviour, consider using the `--portions-file` option.\n\nFor \
                  more information see <https://github.com/Gymmasssorla/finshir>."
)]
pub struct ArgsConfig {
    /// A waiting time span before test execution used to prevent a launch of an
    /// erroneous (unwanted) test
    #[structopt(
        short = "w",
        long = "wait",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "5secs",
        parse(try_from_str = "parse_duration")
    )]
    pub wait: Duration,

    /// A file which consists of a custom JSON array of data portions, specified
    /// as strings.
    ///
    /// When a coroutine finished sending all portions, it reconnects its socket
    /// and starts sending them again.
    #[structopt(
        short = "f",
        long = "portions-file",
        takes_value = true,
        value_name = "LOCATION"
    )]
    pub portions_file: Option<PathBuf>,

    /// A number of connections the program will handle simultaneously. This
    /// option also equals to a number of coroutines
    #[structopt(
        short = "c",
        long = "connections",
        takes_value = true,
        value_name = "POSITIVE-INTEGER",
        default_value = "1000",
        parse(try_from_str = "parse_non_zero_usize")
    )]
    pub connections: NonZeroUsize,

    #[structopt(flatten)]
    pub tester_config: TesterConfig,

    #[structopt(flatten)]
    pub logging_config: LoggingConfig,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct SocketConfig {
    /// A receiver of generator traffic, specified as an IP address (or a domain
    /// name) and a port number, separated by a colon
    #[structopt(
        short = "r",
        long = "receiver",
        takes_value = true,
        value_name = "SOCKET-ADDRESS"
    )]
    pub receiver: ReceiverAddrs,

    /// Try connect a socket within a specified timeout. If a timeout is reached
    /// and a socket wasn't connected, the program will retry the operation
    /// later
    #[structopt(
        long = "connect-timeout",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "10secs",
        parse(try_from_str = "parse_duration")
    )]
    pub connect_timeout: Duration,

    /// This option will be applied if a socket connection error occurs (the
    /// next connection will be performed after this periodicity)
    #[structopt(
        long = "connect-periodicity",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "10secs",
        parse(try_from_str = "parse_duration")
    )]
    pub connect_periodicity: Duration,

    /// If a timeout is reached and a data portion wasn't sent, the program will
    /// retry the operation later
    #[structopt(
        long = "write-timeout",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "10secs",
        parse(try_from_str = "parse_duration")
    )]
    pub write_timeout: Duration,

    /// Use a TLS connection instead of the ordinary TCP protocol. It might be
    /// used to test HTTPS-based services.
    #[structopt(long = "use-tls")]
    pub use_tls: bool,

    /// Specifies the IP_TTL value for all future sockets. Usually this value
    /// equals a number of routers that a packet can go through
    #[structopt(long = "ip-ttl", takes_value = true, value_name = "UNSIGNED-INTEGER")]
    pub ip_ttl: Option<u32>,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct TesterConfig {
    /// A time interval between writing data portions. This option can be used
    /// to modify test intensity
    #[structopt(
        long = "write-periodicity",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "30secs",
        parse(try_from_str = "parse_duration")
    )]
    pub write_periodicity: Duration,

    /// A number of failed data transmissions used to reconnect a socket to a
    /// remote web server
    #[structopt(
        long = "failed-count",
        takes_value = true,
        value_name = "POSITIVE-INTEGER",
        default_value = "5",
        parse(try_from_str = "parse_non_zero_usize")
    )]
    pub failed_count: NonZeroUsize,

    /// A whole test duration, after which all spawned coroutines will stop
    /// their work
    #[structopt(
        short = "d",
        long = "test-duration",
        takes_value = true,
        value_name = "TIME-SPAN",
        default_value = "64years 64hours 64secs",
        parse(try_from_str = "parse_duration")
    )]
    pub test_duration: Duration,

    #[structopt(flatten)]
    pub socket_config: SocketConfig,
}

#[derive(StructOpt, Debug, Clone, Eq, PartialEq)]
pub struct LoggingConfig {
    /// Enable one of the possible verbosity levels. The zero level doesn't
    /// print anything, and the last level prints everything
    #[structopt(
        short = "v",
        long = "verbosity",
        takes_value = true,
        value_name = "LEVEL",
        default_value = "3",
        raw(possible_values = r#"&["0", "1", "2", "3", "4", "5"]"#)
    )]
    pub verbosity: i32,

    /// A format for displaying local date and time in log messages. Type `man
    /// strftime` to see the format specification
    #[structopt(
        long = "date-time-format",
        takes_value = true,
        value_name = "STRING",
        default_value = "%X",
        parse(try_from_str = "parse_time_format")
    )]
    pub date_time_format: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReceiverAddrs {
    /// This is a host that a user has specified. Examples: example.com,
    /// duckduckgo.com, 127.0.0.1, etc.
    pub host: String,

    /// This is a port number that a user has specified.
    pub port: u16,

    /// Addresses, recognised by the `ToSocketAddrs` trait.
    pub recognised_addrs: Vec<SocketAddr>,
}

impl FromStr for ReceiverAddrs {
    type Err = io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // This line validates the specified string
        let addrs = s.to_socket_addrs()?.collect();

        let (host, port) = s.split_at(s.rfind(':').unwrap());
        let host = String::from(host);

        // Eliminate the first ':' character because we've got ":PORT" here
        let port = &port[1..];
        let port: u16 = port.parse().unwrap();

        Ok(ReceiverAddrs {
            recognised_addrs: addrs,
            host,
            port,
        })
    }
}

fn parse_time_format(s: &str) -> Result<String, time::ParseError> {
    // If the `strftime` call succeeds, then the format is correct
    time::strftime(s, &time::now()).map(|_| String::from(s))
}

fn parse_non_zero_usize(number: &str) -> Result<NonZeroUsize, NonZeroUsizeError> {
    let number: usize = number.parse().map_err(NonZeroUsizeError::InvalidFormat)?;

    NonZeroUsize::new(number).ok_or(NonZeroUsizeError::ZeroValue)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NonZeroUsizeError {
    InvalidFormat(ParseIntError),
    ZeroValue,
}

impl Display for NonZeroUsizeError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            NonZeroUsizeError::InvalidFormat(error) => write!(fmt, "{}", error),
            NonZeroUsizeError::ZeroValue => write!(fmt, "The value equals to zero"),
        }
    }
}

impl Error for NonZeroUsizeError {}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that ordinary formats are passed correctly
    #[test]
    fn parses_valid_time_format() {
        let check = |format| {
            assert_eq!(
                parse_time_format(format),
                Ok(String::from(format)),
                "Parses valid time incorrectly"
            )
        };

        check("%x %X %e");
        check("%H %a %G");
        check("something");
        check("flower %d");
    }

    // Invalid formats must produce the invalid format error
    #[test]
    fn parses_invalid_time_format() {
        let check = |format| {
            assert!(
                parse_time_format(format).is_err(),
                "Parses invalid time correctly"
            )
        };

        check("%_=-%vbg=");
        check("yufb%44htv");
        check("sf%jhei9%990");
    }

    // Check that ordinary values are parsed correctly
    #[test]
    fn parses_valid_non_zero_usize() {
        let check = |num| {
            assert_eq!(
                parse_non_zero_usize(num),
                Ok(NonZeroUsize::new(num.parse().unwrap()).unwrap()),
                "Parses valid NonZeroUsize incorrectly"
            )
        };

        check("1");
        check("3");
        check("26655");
        check("+75");
    }

    // Invalid numbers must produce the invalid format error
    #[test]
    fn parses_invalid_non_zero_usize() {
        let check = |num| {
            assert!(
                parse_non_zero_usize(num).is_err(),
                "Parses invalid NonZeroUsize correctly"
            )
        };

        check("   ");
        check("abc5653odr!");
        check("6485&02hde");
        check("-565642");
        check(&"2178".repeat(50));
    }
}
