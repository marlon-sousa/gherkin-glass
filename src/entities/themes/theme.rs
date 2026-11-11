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

    /// Every colour text is written in, apart from the accent.
    #[cfg(test)]
    pub(crate) fn text_colours(&self) -> [Colour; 6] {
        [
            self.text,
            self.muted,
            self.keyword,
            self.parameter,
            self.number,
            self.placeholder,
        ]
    }
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
