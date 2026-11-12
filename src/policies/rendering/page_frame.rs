//! Policy: the frame every page of a version's site has: its skip link and its options, which the versions page shares, and the folder tree's empty landmark.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::PageFrame;

/// The skip link to the page's main content.
pub(super) struct SkipLink<'a> {
    pub(super) frame: &'a PageFrame,
}

/// The Theme control.
pub(super) struct ThemeControl<'a> {
    pub(super) frame: &'a PageFrame,
}

/// The folder tree's landmark, which the script fills.
pub(super) struct TreeLandmark<'a> {
    pub(super) frame: &'a PageFrame,
}

#[derive(Template)]
#[template(path = "page_frame.html")]
struct FrameTemplate<'a> {
    frame: &'a PageFrame,
    skip_link: String,
    theme_control: String,
}

#[derive(Template)]
#[template(path = "skip_link.html")]
struct SkipLinkTemplate<'a> {
    frame: &'a PageFrame,
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

impl Render for PageFrame {
    fn render(&self) -> String {
        rendered(&FrameTemplate {
            frame: self,
            skip_link: SkipLink { frame: self }.render(),
            theme_control: ThemeControl { frame: self }.render(),
        })
    }
}

impl Render for SkipLink<'_> {
    fn render(&self) -> String {
        rendered(&SkipLinkTemplate { frame: self.frame })
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
