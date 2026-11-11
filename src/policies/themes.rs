//! Policy: the built-in colours of each theme, the default accents, and the check of a configured accent.

use crate::entities::{AccentShortfall, Accents, Colour, Palette, Theme};
use crate::policies::{TEXT_CONTRAST, contrast_ratio, hundredths};

/// The accents when the configuration sets none.
pub const DEFAULT_ACCENTS: Accents = Accents::DEFAULT;

pub(crate) fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Light => Palette {
            background: Colour::new(0xff, 0xff, 0xff),
            surface: Colour::new(0xf2, 0xf2, 0xf2),
            text: Colour::new(0x1a, 0x1a, 0x1a),
            muted: Colour::new(0x47, 0x47, 0x47),
            border: Colour::new(0x75, 0x75, 0x75),
            keyword: Colour::new(0x6a, 0x1b, 0x9a),
            parameter: Colour::new(0x14, 0x52, 0x1a),
            number: Colour::new(0x8a, 0x30, 0x00),
            placeholder: Colour::new(0x00, 0x57, 0x5e),
        },
        Theme::Dark => Palette {
            background: Colour::new(0x14, 0x14, 0x14),
            surface: Colour::new(0x22, 0x22, 0x22),
            text: Colour::new(0xec, 0xec, 0xec),
            muted: Colour::new(0xc4, 0xc4, 0xc4),
            border: Colour::new(0x8f, 0x8f, 0x8f),
            keyword: Colour::new(0xe1, 0xb3, 0xff),
            parameter: Colour::new(0x9f, 0xe0, 0xa8),
            number: Colour::new(0xff, 0xbf, 0x8a),
            placeholder: Colour::new(0x7f, 0xdb, 0xe0),
        },
    }
}

/// `accent`, as `theme`'s accent, against the theme's background it
/// contrasts least with, when that is less than [`TEXT_CONTRAST`].
pub(crate) fn check_accent(theme: Theme, accent: Colour) -> Option<AccentShortfall> {
    let (background, ratio) = palette(theme)
        .backgrounds()
        .into_iter()
        .map(|background| (background, contrast_ratio(accent, background)))
        .min_by(|one, other| one.1.total_cmp(&other.1))?;
    (ratio < TEXT_CONTRAST).then(|| AccentShortfall {
        theme,
        accent,
        background,
        hundredths: hundredths(ratio),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policies::PART_CONTRAST;

    #[test]
    fn every_text_colour_reaches_7_to_1_on_every_background_of_its_theme() {
        for theme in Theme::ALL {
            let palette = palette(theme);
            for background in palette.backgrounds() {
                for text in palette.text_colours() {
                    let ratio = contrast_ratio(text, background);
                    assert!(
                        ratio >= TEXT_CONTRAST,
                        "{theme:?}: {text} on {background} is {ratio}"
                    );
                }
                let ratio = contrast_ratio(palette.border, background);
                assert!(ratio >= PART_CONTRAST, "{theme:?}: border is {ratio}");
            }
        }
    }

    #[test]
    fn the_default_accents_pass() {
        for theme in Theme::ALL {
            assert_eq!(check_accent(theme, DEFAULT_ACCENTS.of(theme)), None);
        }
    }

    #[test]
    fn an_accent_short_of_7_to_1_names_the_background_it_reaches_least() {
        let grey = Colour::new(0x59, 0x59, 0x59);
        // 7.00:1 on white, but less on the light theme's surface.
        assert_eq!(
            check_accent(Theme::Light, grey),
            Some(AccentShortfall {
                theme: Theme::Light,
                accent: grey,
                background: palette(Theme::Light).surface,
                hundredths: 625,
            })
        );
        assert_eq!(
            check_accent(Theme::Dark, DEFAULT_ACCENTS.light).map(|found| found.background),
            Some(palette(Theme::Dark).surface)
        );
    }
}
