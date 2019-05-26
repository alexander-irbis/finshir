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

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate may;

use colored::{ColoredString, Colorize};
use structopt::StructOpt;

use crate::config::ArgsConfig;

mod config;
mod logging;
mod report;
mod signal;
mod testing;

fn main() {
    // Initialise OpenSSL
    openssl::init();
    openssl_probe::init_ssl_cert_env_vars();

    let config = ArgsConfig::from_args();
    title();

    logging::setup_logging(&config.logging_config);
    trace!("{:?}", config);

    std::process::exit(testing::run(config));
}

fn title() {
    println!(
        "           {}\n                         {}\n{}",
        r#"       __ _           _     _
                 / _(_)_ __  ___| |__ (_)_ __
                | |_| | '_ \/ __| '_ \| | '__|
                |  _| | | | \__ \ | | | | |
                |_| |_|_| |_|___/_| |_|_|_|"#
            .cyan(),
        format!("version {}", structopt::clap::crate_version!())
            .red()
            .bold(),
        "A coroutines-driven Low & Slow traffic sender, written in Rust\n"
            .green()
            .underline()
    );
}

pub fn cyan<S: ToString>(value: S) -> ColoredString {
    value.to_string().cyan()
}
