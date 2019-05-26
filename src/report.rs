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

use std::path::PathBuf;
use std::time::Instant;
use std::{fs, io};

use humantime::format_duration;
use serde_json::json;
use treexml::{Document, ElementBuilder as E};

use crate::config::ReportConfig;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Update {
    TransmissionSucceed(usize),
    TransmissionFailed,

    ConnectionSucceed,
    ConnectionFailed,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum ReportType {
    Json(PathBuf),
    Xml(PathBuf),
    Text(PathBuf),
    Unspecified,
}

#[derive(Debug, Clone)]
pub struct Report {
    report_type: ReportType,

    beginning: Instant,
    total_bytes_sent: usize,
    total_errors: usize,

    successful_connections: usize,
    failed_connections: usize,
    total_connections: usize,

    successful_transmissions: usize,
    failed_transmissions: usize,
    total_transmissions: usize,
}

impl Report {
    pub fn new(config: &ReportConfig) -> Self {
        Report {
            report_type: {
                if let Some(filename) = &config.json_report {
                    ReportType::Json(filename.clone())
                } else if let Some(filename) = &config.xml_report {
                    ReportType::Xml(filename.clone())
                } else if let Some(filename) = &config.text_report {
                    ReportType::Text(filename.clone())
                } else {
                    ReportType::Unspecified
                }
            },

            beginning: Instant::now(),
            total_bytes_sent: 0,
            total_errors: 0,

            successful_connections: 0,
            failed_connections: 0,
            total_connections: 0,

            successful_transmissions: 0,
            failed_transmissions: 0,
            total_transmissions: 0,
        }
    }

    pub fn update(&mut self, update: Update) {
        match update {
            Update::TransmissionSucceed(count) => {
                self.successful_transmissions += 1;
                self.total_transmissions += 1;
                self.total_bytes_sent += count;
            }
            Update::TransmissionFailed => {
                self.failed_transmissions += 1;
                self.total_transmissions += 1;
                self.total_errors += 1;
            }

            Update::ConnectionSucceed => {
                self.successful_connections += 1;
                self.total_connections += 1;
            }
            Update::ConnectionFailed => {
                self.failed_connections += 1;
                self.total_connections += 1;
                self.total_errors += 1;
            }
        }
    }

    pub fn generate(&self) -> io::Result<()> {
        match &self.report_type {
            ReportType::Json(filename) => self.gen_json(filename),
            ReportType::Xml(filename) => self.gen_xml(filename),
            ReportType::Text(filename) => self.gen_text(filename),
            ReportType::Unspecified => {
                self.gen_term();
                Ok(())
            }
        }
    }

    fn gen_json(&self, filename: &PathBuf) -> io::Result<()> {
        let report = json!({
            "test-duration": format_duration(self.beginning.elapsed()).to_string(),
            "total-bytes-sent": self.total_bytes_sent.to_string(),
            "total-errors": self.total_errors.to_string(),

            "connections": {
                "successful": self.successful_connections.to_string(),
                "failed": self.failed_connections.to_string(),
                "total": self.total_connections.to_string(),
            },

            "transmissions": {
                "successful": self.successful_transmissions.to_string(),
                "failed": self.failed_transmissions.to_string(),
                "total": self.total_transmissions.to_string(),
            },
        });

        info!(
            "generating a JSON report to \"{}\"...",
            filename.to_str().expect("PathBuf::to_str failed")
        );
        fs::write(filename, format!("{:#}", report))
    }

    fn gen_xml(&self, filename: &PathBuf) -> io::Result<()> {
        let report = Document::build(E::new("finshir-report").children(vec![
            E::new("test-duration").text(format_duration(self.beginning.elapsed())),
            E::new("total-bytes-sent").text(self.total_bytes_sent.to_string()),
            E::new("total-errors").text(self.total_errors.to_string()),
            E::new("connections").children(vec![
                E::new("successful").text(self.successful_connections.to_string()),
                E::new("failed").text(self.failed_connections.to_string()),
                E::new("total").text(self.total_connections.to_string()),
            ]),
            E::new("transmissions").children(vec![
                E::new("successful").text(self.successful_transmissions.to_string()),
                E::new("failed").text(self.failed_transmissions.to_string()),
                E::new("total").text(self.total_transmissions.to_string()),
            ]),
        ]));

        info!(
            "generating an XML report to \"{}\"...",
            filename.to_str().expect("PathBuf::to_str failed")
        );
        fs::write(filename, format!("{}", report))
    }

    fn gen_text(&self, filename: &PathBuf) -> io::Result<()> {
        #[rustfmt::skip]
            fs::write(
            filename,
            format!(
                "{title}\n\
                 Test duration:            {duration}\n\
                 Total bytes sent:         {bytes}\n\
                 Total errors:             {errors}\n\n\
                 Successful connections:   {successful_conns}\n\
                 Failed connections:       {failed_conns}\n\
                 Total connections:        {total_conns}\n\n\
                 Successful transmissions: {successful_transmissions}\n\
                 Failed transmissions:     {failed_transmissions}\n\
                 Total transmissions:      {total_transmissions}\n\
                 {end_title}\n",
                title = "*********************** FINSHIR REPORT ***********************",
                end_title = "**************************************************************",
                duration = format_duration(self.beginning.elapsed()),
                bytes = self.total_bytes_sent,
                errors = self.total_errors,
                successful_conns = self.successful_connections,
                failed_conns = self.failed_connections,
                total_conns = self.total_connections,
                successful_transmissions = self.successful_transmissions,
                failed_transmissions = self.failed_transmissions,
                total_transmissions = self.total_transmissions,
            ),
        )?;

        info!(
            "generating a TXT report to \"{}\"...",
            filename.to_str().expect("PathBuf::to_str failed")
        );
        Ok(())
    }

    fn gen_term(&self) {
        #[rustfmt::skip]
        info!(
            "the statistics:\n\
             \tTest duration:            {duration}\n\
             \tTotal bytes sent:         {bytes}\n\
             \tTotal errors:             {errors}\n\n\
             \tSuccessful connections:   {successful_conns}\n\
             \tFailed connections:       {failed_conns}\n\
             \tTotal connections:        {total_conns}\n\n\
             \tSuccessful transmissions: {successful_transmissions}\n\
             \tFailed transmissions:     {failed_transmissions}\n\
             \tTotal transmissions:      {total_transmissions}",
            duration = crate::cyan(format_duration(self.beginning.elapsed())),
            bytes = crate::cyan(self.total_bytes_sent),
            errors = crate::cyan(self.total_errors),
            successful_conns = crate::cyan(self.successful_connections),
            failed_conns = crate::cyan(self.failed_connections),
            total_conns = crate::cyan(self.total_connections),
            successful_transmissions = crate::cyan(self.successful_transmissions),
            failed_transmissions = crate::cyan(self.failed_transmissions),
            total_transmissions = crate::cyan(self.total_transmissions),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_initial_empty_report() {
        let report = Report::new(&ReportConfig {
            json_report: None,
            xml_report: None,
            text_report: None,
        });

        assert_eq!(report.total_bytes_sent, 0);
        assert_eq!(report.total_errors, 0);

        assert_eq!(report.successful_connections, 0);
        assert_eq!(report.failed_connections, 0);
        assert_eq!(report.total_connections, 0);

        assert_eq!(report.successful_transmissions, 0);
        assert_eq!(report.failed_transmissions, 0);
        assert_eq!(report.total_transmissions, 0);
    }

    #[test]
    fn is_correct_report_type() {
        // The tests won't create files with these names, they are defined just to be
        // applied by `ReportConfig`.
        let json = PathBuf::from("report.json");
        let xml = PathBuf::from("report.xml");
        let text = PathBuf::from("report.txt");

        assert_eq!(
            Report::new(&ReportConfig {
                json_report: None,
                xml_report: None,
                text_report: None,
            })
            .report_type,
            ReportType::Unspecified
        );

        assert_eq!(
            Report::new(&ReportConfig {
                json_report: Some(json.clone()),
                xml_report: None,
                text_report: None,
            })
            .report_type,
            ReportType::Json(json.clone())
        );

        assert_eq!(
            Report::new(&ReportConfig {
                json_report: None,
                xml_report: Some(xml.clone()),
                text_report: None,
            })
            .report_type,
            ReportType::Xml(xml.clone())
        );

        assert_eq!(
            Report::new(&ReportConfig {
                json_report: None,
                xml_report: None,
                text_report: Some(text.clone()),
            })
            .report_type,
            ReportType::Text(text.clone())
        );
    }

    #[test]
    fn report_updates_correctly() {
        let mut report = Report::new(&ReportConfig {
            xml_report: None,
            json_report: None,
            text_report: None,
        });

        // Let's assume that we've got one connection established
        report.update(Update::ConnectionSucceed);

        assert_eq!(report.total_bytes_sent, 0);
        assert_eq!(report.total_errors, 0);

        assert_eq!(report.successful_connections, 1);
        assert_eq!(report.failed_connections, 0);
        assert_eq!(report.total_connections, 1);

        assert_eq!(report.successful_transmissions, 0);
        assert_eq!(report.failed_transmissions, 0);
        assert_eq!(report.total_transmissions, 0);

        // Let's assume that we've got one connection established
        report.update(Update::ConnectionFailed);

        assert_eq!(report.total_bytes_sent, 0);
        assert_eq!(report.total_errors, 1);

        assert_eq!(report.successful_connections, 1);
        assert_eq!(report.failed_connections, 1);
        assert_eq!(report.total_connections, 2);

        assert_eq!(report.successful_transmissions, 0);
        assert_eq!(report.failed_transmissions, 0);
        assert_eq!(report.total_transmissions, 0);

        // Let's assume that we've send 7456 bytes without errors
        report.update(Update::TransmissionSucceed(7456));

        assert_eq!(report.total_bytes_sent, 7456);
        assert_eq!(report.total_errors, 1);

        assert_eq!(report.successful_connections, 1);
        assert_eq!(report.failed_connections, 1);
        assert_eq!(report.total_connections, 2);

        assert_eq!(report.successful_transmissions, 1);
        assert_eq!(report.failed_transmissions, 0);
        assert_eq!(report.total_transmissions, 1);

        // Let's assume that we've got one failed transmission
        report.update(Update::TransmissionFailed);

        assert_eq!(report.total_bytes_sent, 7456);
        assert_eq!(report.total_errors, 2);

        assert_eq!(report.successful_connections, 1);
        assert_eq!(report.failed_connections, 1);
        assert_eq!(report.total_connections, 2);

        assert_eq!(report.successful_transmissions, 1);
        assert_eq!(report.failed_transmissions, 1);
        assert_eq!(report.total_transmissions, 2);

        // Our program logic must ALWAYS satisfy these equations
        assert_eq!(
            report.successful_connections + report.failed_connections,
            report.total_connections
        );
        assert_eq!(
            report.successful_transmissions + report.failed_transmissions,
            report.total_transmissions
        );
    }
}
