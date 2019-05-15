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

use std::io::{self, Write};
use std::num::NonZeroUsize;
use std::time::Instant;

use humantime::format_duration;
use may::{self, coroutine, go};

use crate::config::{ArgsConfig, TesterConfig};
use crate::testing::socket::FinshirSocket;

mod portions;
mod socket;

/// This is the key function which accepts `ArgsConfig` and spawns all
/// coroutines, returning 0 on success and 1 on failure.
pub fn run(config: &ArgsConfig) -> i32 {
    let portions = match portions::get_portions(config.portions_file.as_ref()) {
        Err(err) => {
            error!("Failed to parse the JSON >>> {}!", err);
            return 1;
        }
        Ok(res) => res,
    };
    let portions: Vec<&[u8]> = portions.iter().map(Vec::as_slice).collect();

    warn!(
        "Waiting {} and then spawning {} coroutines connected through the {}.",
        crate::cyan(format_duration(config.wait)),
        crate::cyan(config.connections),
        if config.tester_config.socket_config.use_tor {
            "Tor network"
        } else {
            "regular Web"
        }
    );
    std::thread::sleep(config.wait);

    coroutine::scope(|scope| {
        let portions = &portions;
        let config = &config;
        let iters = config.connections.get();

        for _ in 0..iters {
            go!(scope, move || run_tester(&config.tester_config, portions));
        }

        info!("All the coroutines have been spawned.");
    });

    0
}

fn run_tester(config: &TesterConfig, portions: &[&[u8]]) {
    let start = Instant::now();

    loop {
        let mut socket = FinshirSocket::connect(&config.socket_config);

        for &portion in portions {
            if start.elapsed() >= config.test_duration {
                info!("The allotted time has expired. Exiting the coroutine...");
                return;
            }

            match send_portion(&mut socket, portion, config.failed_count) {
                SendPortionResult::Success => {
                    info!(
                        "{} byte(s) have been sent. Waiting {}...",
                        crate::cyan(portion.len()),
                        crate::cyan(format_duration(config.write_periodicity))
                    );
                }
                SendPortionResult::Failed(err) => {
                    error!(
                        "Sending {} byte(s) failed {} times >>> {}! Reconnecting the socket...",
                        crate::cyan(portion.len()),
                        crate::cyan(config.failed_count),
                        err,
                    );
                    break;
                }
            }

            coroutine::sleep(config.write_periodicity);
        }

        info!("All the data portions have been sent. Reconnecting the socket...");
    }
}

#[derive(Debug)]
enum SendPortionResult {
    Success,
    Failed(io::Error),
}

fn send_portion(
    socket: &mut FinshirSocket,
    portion: &[u8],
    failed_count: NonZeroUsize,
) -> SendPortionResult {
    let res = {
        for _ in 0..(failed_count.get() - 1) {
            match socket.write_all(portion) {
                Ok(_) => return SendPortionResult::Success,
                Err(err) => {
                    error!(
                        "Failed to send {} byte(s) >>> {}! Retrying the operation...",
                        crate::cyan(portion.len()),
                        err
                    );
                }
            }
        }

        match socket.write_all(portion) {
            Ok(_) => SendPortionResult::Success,
            Err(err) => SendPortionResult::Failed(err),
        }
    };

    socket
        .flush()
        .map_or_else(SendPortionResult::Failed, |_| res)
}
