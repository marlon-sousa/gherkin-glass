//! Entity: the themes a reader can pick, and the accent colour of each, which the configuration may set.

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

/// The accent of each theme, used for links and for the outline of what has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Accents {
    pub(crate) light: Colour,
    pub(crate) dark: Colour,
}

impl Accents {
    /// The accents when the configuration sets none, each reaching 7:1 in its theme.
    pub(crate) const DEFAULT: Accents = Accents {
        light: Colour::new(0x0b, 0x4f, 0x9c),
        dark: Colour::new(0x8c, 0xc4, 0xff),
    };

    pub(crate) fn of(&self, theme: Theme) -> Colour {
        match theme {
            Theme::Light => self.light,
            Theme::Dark => self.dark,
        }
    }
}
