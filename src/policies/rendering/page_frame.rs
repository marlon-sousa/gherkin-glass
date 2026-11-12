//! Policy: the frame every page of a version's site has: its skip link, its options, with the Theme control the versions page has too, and the folder tree's empty landmark.

use askama::Template;

use super::node_rendering::{Render, rendered};
use crate::entities::PageFrame;

#[derive(Template)]
#[template(path = "page_frame.html")]
struct FrameTemplate<'a> {
    frame: &'a PageFrame,
    theme_control: String,
}

/// The Theme control, in the words of `PageFrame`.
pub(super) struct ThemeControl<'a>(pub(super) &'a PageFrame);

#[derive(Template)]
#[template(path = "theme_control.html")]
struct ThemeControlTemplate<'a> {
    frame: &'a PageFrame,
}

impl Render for PageFrame {
    fn render(&self) -> String {
        rendered(&FrameTemplate {
            frame: self,
            theme_control: ThemeControl(self).render(),
        })
    }
}

impl Render for ThemeControl<'_> {
    fn render(&self) -> String {
        rendered(&ThemeControlTemplate { frame: self.0 })
    }
}

/// The folder tree's landmark, which the script fills.
pub(super) struct TreeLandmark<'a>(pub(super) &'a PageFrame);

#[derive(Template)]
#[template(path = "tree_landmark.html")]
struct TreeLandmarkTemplate<'a> {
    frame: &'a PageFrame,
}

impl Render for TreeLandmark<'_> {
    fn render(&self) -> String {
        rendered(&TreeLandmarkTemplate { frame: self.0 })
    }
}
