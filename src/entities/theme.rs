//! Entity: the themes a reader can pick, and the colours each one paints a page with.

use crate::entities::Colour;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Theme {
    Light,
    Dark,
}

impl Theme {
    #[cfg(test)]
    pub(crate) const ALL: [Theme; 2] = [Theme::Light, Theme::Dark];
}

/// The colours of one theme, apart from its accent, which the configuration sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Palette {
    /// The page's own background.
    pub(crate) background: Colour,
    /// The background of the parts set apart from the page, such as tables and doc strings.
    pub(crate) surface: Colour,
    pub(crate) text: Colour,
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

    /// Every colour text is written in, apart from the accent.
    #[cfg(test)]
    pub(crate) fn text_colours(&self) -> [Colour; 5] {
        [
            self.text,
            self.keyword,
            self.parameter,
            self.number,
            self.placeholder,
        ]
    }
}

/// The accent of each theme, used for links and for the outline of what has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accents {
    pub light: Colour,
    pub dark: Colour,
}

impl Accents {
    pub(crate) fn of(&self, theme: Theme) -> Colour {
        match theme {
            Theme::Light => self.light,
            Theme::Dark => self.dark,
        }
    }
}
