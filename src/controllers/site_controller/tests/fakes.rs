//! The controller's fakes: one for each service it drives, each noting what it was asked in one
//! list they share, so that a test sees the order of the calls, and a listener that records what
//! it was told.

mod building;
mod serving;

use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, PoisonError};

use super::*;
use crate::errors::{
    ConfigError, ConfigErrorKind, ConfigErrors, InputError, InputErrorKind, InputErrors,
    ListenErrorKind, OutputError, OutputErrorKind,
};
use building::{
    FakeConverter, FakeGherkinParser, FakeInputCollector, FakeSettingsService, FakeSiteWriter,
};
use serving::FakeServerService;
pub(super) use serving::Recorded;

/// The calls every fake of one controller was asked, in order, each as the fake's name, the
/// method's and what it was handed.
#[derive(Default)]
pub(super) struct Calls {
    calls: Mutex<Vec<String>>,
}

impl Calls {
    fn note(&self, call: String) {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(call);
    }

    /// The calls so far.
    pub(super) fn taken(&self) -> Vec<String> {
        self.calls
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// The stage of a build that fails.
pub(super) enum Failing {
    Settings,
    OutputCheck,
    Input,
    Writing,
}

/// The fakes one controller is given, sharing the list of their calls.
pub(super) struct World {
    pub(super) calls: Arc<Calls>,
    pub(super) settings: Arc<FakeSettingsService>,
    pub(super) input_collector: Arc<FakeInputCollector>,
    pub(super) gherkin_parser: Arc<FakeGherkinParser>,
    pub(super) converter: Arc<FakeConverter>,
    pub(super) site_writer: Arc<FakeSiteWriter>,
    pub(super) server: Arc<FakeServerService>,
}

impl World {
    /// A controller of these fakes.
    pub(super) fn controller(&self) -> SiteController {
        SiteController::new(
            self.settings.clone(),
            self.input_collector.clone(),
            self.gherkin_parser.clone(),
            self.converter.clone(),
            self.site_writer.clone(),
            self.server.clone(),
        )
    }

    /// Makes `failing` fail from now on.
    pub(super) fn fail(&self, failing: Failing) {
        let flag = match failing {
            Failing::Settings => &self.settings.failing,
            Failing::OutputCheck => &self.site_writer.check_failing,
            Failing::Input => &self.gherkin_parser.failing,
            Failing::Writing => &self.site_writer.write_failing,
        };
        flag.store(true, Ordering::SeqCst);
    }
}

/// Fakes that each do what they are asked, until a test makes one fail.
pub(super) fn world() -> World {
    let calls = Arc::new(Calls::default());
    World {
        settings: Arc::new(FakeSettingsService {
            calls: calls.clone(),
            ..FakeSettingsService::default()
        }),
        input_collector: Arc::new(FakeInputCollector {
            calls: calls.clone(),
        }),
        gherkin_parser: Arc::new(FakeGherkinParser {
            calls: calls.clone(),
            ..FakeGherkinParser::default()
        }),
        converter: Arc::new(FakeConverter {
            calls: calls.clone(),
        }),
        site_writer: Arc::new(FakeSiteWriter {
            calls: calls.clone(),
            ..FakeSiteWriter::default()
        }),
        server: Arc::new(FakeServerService {
            calls: calls.clone(),
            ..FakeServerService::default()
        }),
        calls,
    }
}

/// What the fake site writer says of every site it writes.
pub(super) fn built_site() -> BuiltSite {
    BuiltSite {
        large_search_documents: None,
        untitled: true,
    }
}

pub(super) fn config_errors() -> ConfigErrors {
    ConfigErrors {
        file: PathBuf::from("ghg.toml"),
        errors: vec![ConfigError {
            line: Some(1),
            kind: ConfigErrorKind::UnknownKey {
                key: "inputs".into(),
            },
        }],
    }
}

pub(super) fn input_errors() -> InputErrors {
    let input = Path::new("features");
    InputErrors::new(vec![InputError::new(
        input,
        &input.join("a.feature"),
        None,
        InputErrorKind::CannotBeRead {
            reason: "denied".into(),
        },
    )])
}

pub(super) fn output_error() -> OutputError {
    OutputError {
        output: PathBuf::from("site"),
        version: None,
        kind: OutputErrorKind::NotASite,
    }
}

pub(super) fn listen_error() -> ServeError {
    ServeError::Listen {
        host: "127.0.0.1".into(),
        port: 8000,
        kind: ListenErrorKind::PortTaken,
    }
}
