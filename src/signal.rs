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

use std::sync::Mutex;

use signal_hook::{self, SigId, SIGINT};

lazy_static! {
    static ref PREVIOUS_ID: Mutex<SigId> = Mutex::new(unsafe {
        signal_hook::register(SIGINT, || {
            info!("cancellation has been received. Exiting the process...");
            std::process::exit(0);
        })
        .expect("Failed to setup a SIGINT handler")
    });
}

pub unsafe fn override_handler<H>(handler: H)
where
    H: Fn() + Sync + Send + 'static,
{
    signal_hook::unregister(*PREVIOUS_ID.lock().expect("PREVIOUS_ID.lock() failed"));
    signal_hook::register(SIGINT, handler).expect("Failed to override a SIGINT handler");
}
