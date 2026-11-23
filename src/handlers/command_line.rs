//! Handler: the `ghg` command, from its arguments to what the person at the terminal reads, or a program takes apart.

mod build_notes;
mod build_reporter;
mod exit_status;
mod human_reporter;
mod json_reporter;

use std::ffi::OsString;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::process::ExitCode;
use std::sync::Arc;

use build_notes::{empty_site, nothing_to_build};
use build_reporter::BuildReporter;
use exit_status::ExitStatus;
use human_reporter::HumanReporter;
use i18n_embed_fl::fl;
use json_reporter::JsonReporter;

use crate::controllers::{SiteApi, SiteListener};
use crate::entities::{
    BuildOptions, Invocation, MessageFormat, NothingToBuild, ServeEnd, ServeOptions,
};
use crate::errors::{RebuildError, WatchError};
use crate::policies::{CATALOGUE, parse_arguments};
use crate::ports::Terminal;

pub(crate) struct CommandLine {
    site: Arc<dyn SiteApi>,
    terminal: Arc<dyn Terminal>,
}

/// What a run prints when it ends, and the status it exits with. A command
/// that keeps running, `ghg serve`, says what happens meanwhile through the
/// terminal it was given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Reply {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    status: ExitStatus,
}

impl CommandLine {
    pub(crate) fn new(site: Arc<dyn SiteApi>, terminal: Arc<dyn Terminal>) -> Self {
        CommandLine { site, terminal }
    }

    /// `arguments` come after the command's own name.
    pub(crate) fn run(&self, arguments: impl IntoIterator<Item = OsString>) -> Reply {
        match parse_arguments(arguments) {
            Err(error) => Reply::failure(ExitStatus::Mistaken, error.to_string()),
            Ok(Invocation::Help) => Reply::success(fl!(CATALOGUE, "help")),
            Ok(Invocation::Version) => Reply::success(fl!(
                CATALOGUE,
                "version",
                version = env!("CARGO_PKG_VERSION")
            )),
            Ok(Invocation::Build {
                options,
                message_format,
            }) => self.build(&options, message_format),
            Ok(Invocation::Serve(options)) => self.serve(&options),
        }
    }

    fn serve(&self, options: &ServeOptions) -> Reply {
        let listener = TerminalListener {
            terminal: self.terminal.as_ref(),
        };
        match self.site.serve(options, &listener) {
            Ok(ServeEnd::NoLongerServed) => Reply::success(fl!(CATALOGUE, "serve-stopped")),
            Ok(ServeEnd::NothingToBuild(nothing)) => Reply {
                stderr: lines(nothing_to_build(&nothing)),
                ..Reply::success(String::new())
            },
            Err(error) => Reply::failure(ExitStatus::Stopped, error.to_string()),
        }
    }

    /// How the build ended, worded by the reporter `message_format` names.
    fn build(&self, options: &BuildOptions, message_format: MessageFormat) -> Reply {
        reporter(message_format).report(&self.site.build(options))
    }
}

/// The reporter of `message_format`.
fn reporter(message_format: MessageFormat) -> &'static dyn BuildReporter {
    match message_format {
        MessageFormat::Human => &HumanReporter,
        MessageFormat::Json => &JsonReporter,
    }
}

impl Reply {
    fn success(stdout: String) -> Self {
        Reply {
            stdout: lines(stdout),
            stderr: String::new(),
            status: ExitStatus::Done,
        }
    }

    fn failure(status: ExitStatus, stderr: String) -> Self {
        Reply {
            stdout: String::new(),
            stderr: lines(stderr),
            status,
        }
    }

    pub(crate) fn exit_code(&self) -> ExitCode {
        ExitCode::from(self.status as u8)
    }
}

/// ghg serve's events, worded for the terminal.
struct TerminalListener<'a> {
    terminal: &'a dyn Terminal,
}

impl SiteListener for TerminalListener<'_> {
    fn listening(&self, mut address: SocketAddr) {
        // A browser cannot open the address that means every network.
        if address.ip().is_unspecified() {
            address.set_ip(match address {
                SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::LOCALHOST),
                SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::LOCALHOST),
            });
        }
        let address = format!("http://{address}/");
        self.terminal
            .say(&lines(fl!(CATALOGUE, "serve-listening", address = address)));
    }

    fn rebuilt(&self) {
        self.terminal.say(&lines(fl!(CATALOGUE, "serve-rebuilt")));
    }

    fn not_rebuilt(&self, error: &RebuildError) {
        self.terminal.warn(&error.to_string());
    }

    fn not_watched(&self, error: &WatchError) {
        self.terminal.warn(&lines(error.to_string()));
    }

    fn empty(&self, nothing: &NothingToBuild) {
        self.terminal.warn(&lines(empty_site(nothing)));
    }
}

/// Text ending in a line break, unless it is empty.
fn lines(mut text: String) -> String {
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests;
