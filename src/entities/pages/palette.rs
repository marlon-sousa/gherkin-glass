//! Entity: the colours of one theme, with which its pages are painted.

use crate::entities::Colour;

/// The colours of one theme, apart from its accent, which the configuration sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    /// The page's own background.
    pub(crate) background: Colour,
    /// The background of the parts set apart from the page, such as tables and doc strings.
    pub(crate) surface: Colour,
    pub(crate) text: Colour,
    /// For text that is less prominent, such as tags, which still reaches 7:1.
    pub(crate) muted: Colour,
    /// Lines that separate parts, which carry no text.
    pub(crate) border: Colour,
    pub(crate) keyword: Colour,
    pub(crate) parameter: Colour,
    pub(crate) number: Colour,
    pub(crate) placeholder: Colour,
}

impl Palette {
    /// Every background text is written on.
    pub(crate) fn backgrounds(&self) -> [Colour; 2] {
        [self.background, self.surface]
    }
}
