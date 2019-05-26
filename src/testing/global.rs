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

//! This file introduces some global definitions. Right now, it contains only a
//! global instance of `Report`.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};

use may::sync::Mutex;

use crate::config::ReportConfig;
use crate::report::{Report, Update};

lazy_static! {
    static ref REPORT: Mutex<Report> = Mutex::new(Report::new(&ReportConfig {
        xml_report: None,
        json_report: None,
        text_report: None,
    }));
    static ref IS_SET: AtomicBool = AtomicBool::new(false);
}

pub fn update_report(update: Update) {
    if !IS_SET.load(Ordering::Relaxed) {
        panic!("REPORT isn't set");
    }

    REPORT.lock().expect("REPORT.lock() failed").update(update);
}

pub fn generate_report() -> io::Result<()> {
    if !IS_SET.load(Ordering::Relaxed) {
        panic!("REPORT isn't set");
    }

    REPORT.lock().expect("REPORT.lock() failed").generate()
}

pub fn init_report(config: &ReportConfig) {
    *REPORT.lock().expect("REPORT.lock() failed") = Report::new(config);
    IS_SET.store(true, Ordering::Relaxed);
}
