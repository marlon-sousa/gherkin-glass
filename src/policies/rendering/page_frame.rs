//! Policy: the frame of every shell: its dialog for a page opened from disk, with what its `main` then says, its skip link, its options and its footer, which the versions page shares, and the folder tree's empty landmark.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::PageFrame;

/// The dialog first in the page, opened only when the page is opened from disk.
pub(super) struct DiskNotice<'a> {
    pub(super) frame: &'a PageFrame,
    /// What a shell then says in its `main`: on every page, as a version's does, or on every
    /// page but its home page, as a site's and a preview's do; `None` for the versions page.
    pub(super) shell: Option<ShellOnDisk>,
}

/// Where a shell opened from disk says in its `main` that it cannot be shown.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ShellOnDisk {
    EveryPage,
    AllButTheHomePage,
}

/// The skip link to the page's main content.
pub(super) struct SkipLink<'a> {
    pub(super) frame: &'a PageFrame,
    /// `##ghg_main` in a shell, as an element of the page drawn, `#ghg_main` on the versions page.
    pub(super) href: &'a str,
}

/// The Theme control.
pub(super) struct ThemeControl<'a> {
    pub(super) frame: &'a PageFrame,
}

/// The folder tree's landmark, which the script fills.
pub(super) struct TreeLandmark<'a> {
    pub(super) frame: &'a PageFrame,
}

/// The footer, after the page's main content.
pub(super) struct PageFooter<'a> {
    pub(super) frame: &'a PageFrame,
}

/// A shell's options: the folder tree's checkbox and button, and the Theme control.
pub(super) struct ShellOptions<'a> {
    pub(super) frame: &'a PageFrame,
}

#[derive(Template)]
#[template(path = "page_frame.html")]
struct OptionsTemplate<'a> {
    frame: &'a PageFrame,
    theme_control: String,
}

#[derive(Template)]
#[template(path = "disk_notice.html")]
struct DiskNoticeTemplate<'a> {
    frame: &'a PageFrame,
    /// The paragraph, as a literal of JavaScript.
    cannot_be_shown: Option<String>,
    every_page: bool,
}

#[derive(Template)]
#[template(path = "skip_link.html")]
struct SkipLinkTemplate<'a> {
    frame: &'a PageFrame,
    href: &'a str,
}

#[derive(Template)]
#[template(path = "theme_control.html")]
struct ThemeControlTemplate<'a> {
    frame: &'a PageFrame,
}

#[derive(Template)]
#[template(path = "tree_landmark.html")]
struct TreeLandmarkTemplate<'a> {
    frame: &'a PageFrame,
}

#[derive(Template)]
#[template(path = "page_footer.html")]
struct PageFooterTemplate<'a> {
    frame: &'a PageFrame,
    /// gherkin-glass's repository, as `Cargo.toml` names it.
    repository: &'static str,
}

impl Render for ShellOptions<'_> {
    fn render(&self) -> String {
        rendered(&OptionsTemplate {
            frame: self.frame,
            theme_control: ThemeControl { frame: self.frame }.render(),
        })
    }
}

impl Render for DiskNotice<'_> {
    fn render(&self) -> String {
        rendered(&DiskNoticeTemplate {
            frame: self.frame,
            cannot_be_shown: self
                .shell
                .map(|_| script_text(&self.frame.opened_from_disk.cannot_be_shown)),
            every_page: self.shell == Some(ShellOnDisk::EveryPage),
        })
    }
}

impl Render for SkipLink<'_> {
    fn render(&self) -> String {
        rendered(&SkipLinkTemplate {
            frame: self.frame,
            href: self.href,
        })
    }
}

impl Render for ThemeControl<'_> {
    fn render(&self) -> String {
        rendered(&ThemeControlTemplate { frame: self.frame })
    }
}

impl Render for TreeLandmark<'_> {
    fn render(&self) -> String {
        rendered(&TreeLandmarkTemplate { frame: self.frame })
    }
}

impl Render for PageFooter<'_> {
    fn render(&self) -> String {
        rendered(&PageFooterTemplate {
            frame: self.frame,
            repository: env!("CARGO_PKG_REPOSITORY"),
        })
    }
}

/// `text` as a literal of JavaScript inside a `script` element: in quotes, with nothing in it
/// that could end the element.
fn script_text(text: &str) -> String {
    serde_json::Value::from(text)
        .to_string()
        .replace('<', "\\u003c")
}
