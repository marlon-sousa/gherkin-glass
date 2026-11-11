//! Policy: the interface text catalogue, embedded in the binary, in English only for now.

use std::sync::LazyLock;

use i18n_embed::LanguageLoader;
use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "messages/"]
struct Catalogue;

/// Read with `fl!(CATALOGUE, "message-id", argument = value)`.
#[expect(
    clippy::expect_used,
    reason = "the English catalogue is embedded and valid"
)]
pub(crate) static CATALOGUE: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();
    loader
        .load_fallback_language(&Catalogue)
        .expect("the English catalogue is embedded and valid");
    // Fluent's isolation marks would reach the terminal and the pages.
    loader.set_use_isolating(false);
    loader
});

#[cfg(test)]
mod tests {
    use i18n_embed_fl::fl;

    use super::*;

    #[test]
    fn a_placeholder_carries_no_isolation_marks() {
        assert_eq!(fl!(CATALOGUE, "version", version = "1.2.3"), "ghg 1.2.3");
    }
}
