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
use std::sync::Arc;
use std::thread;

use humantime::format_duration;
use may::{self, coroutine};

use crate::config::{ArgsConfig, TesterConfig};
use crate::testing::socket::FinshirSocket;

mod portions;
mod socket;

/// This is the key function which accepts `ArgsConfig` and spawns all
/// coroutines, returning 0 on success and 1 on failure.
pub fn run(config: ArgsConfig) -> i32 {
    let test_duration = config.tester_config.test_duration;

    let portions = match portions::get_portions(config.portions_file.as_ref()) {
        Err(err) => {
            error!("failed to parse the JSON >>> {}!", err);
            return 1;
        }
        Ok(res) => res,
    };

    warn!(
        "waiting {} and then spawning {} coroutines connected to {}.",
        crate::cyan(format_duration(config.wait)),
        crate::cyan(config.connections),
        crate::cyan(format!(
            "{}:{}",
            &config.tester_config.socket_config.receiver.host,
            &config.tester_config.socket_config.receiver.port
        ))
    );
    std::thread::sleep(config.wait);

    // Starting to spawn all the coroutines. Save the results into the vector to be
    // able to cancel them in future.
    let mut handles = Vec::with_capacity(config.connections.get());

    let portions = Arc::new(portions);
    let tester_config = Arc::new(config.tester_config);

    for _ in 0..config.connections.get() {
        let portions = portions.clone();
        let tester_config = tester_config.clone();

        handles.push(go!(move || run_tester(tester_config, portions)));
    }

    // Wait exactly `test_duration` time and then starting to cancel the spawned
    // coroutines located in `handles`.
    info!(
        "all the coroutines have been spawned. Waiting {} and then exit.",
        crate::cyan(format_duration(test_duration))
    );
    thread::sleep(test_duration);

    handles
        .into_iter()
        .for_each(|h| unsafe { h.coroutine().cancel() });
    info!("all the coroutines have been cancelled due to the expired time.");
    0
}

fn run_tester(config: Arc<TesterConfig>, portions: Arc<Vec<Vec<u8>>>) {
    loop {
        let mut socket = FinshirSocket::connect(&config.socket_config);

        for portion in portions.iter() {
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
                        "sending {} byte(s) failed {} times >>> {}! Reconnecting the socket...",
                        crate::cyan(portion.len()),
                        crate::cyan(config.failed_count),
                        err,
                    );
                    break;
                }
            }

            coroutine::sleep(config.write_periodicity);
        }

        info!("all the data portions have been sent. Reconnecting the socket...");
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
                        "failed to send {} byte(s) >>> {}! Retrying the operation...",
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

    match socket.flush() {
        Ok(_) => res,
        Err(err) => SendPortionResult::Failed(err),
    }
}
