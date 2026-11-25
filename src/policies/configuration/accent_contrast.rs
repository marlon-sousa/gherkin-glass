//! Policy: the check of a configured accent colour, the contrast it reaches, as WCAG 2.2 defines it, against its theme's backgrounds.

use std::cmp::Ordering;

use crate::entities::{Colour, Theme};
use crate::errors::AccentShortfall;
use crate::policies::palette;

/// What an accent, which colours text, must reach against every background
/// of its theme: WCAG's AAA level, success criterion 1.4.6.
const TEXT_CONTRAST: f64 = 7.0;

/// `accent`, as `theme`'s accent, against the theme's background it
/// contrasts least with, when that is less than [`TEXT_CONTRAST`].
pub(super) fn check_accent(theme: Theme, accent: Colour) -> Option<AccentShortfall> {
    let background = palette(theme)
        .backgrounds()
        .into_iter()
        .min_by(|one, other| by_contrast_with(accent, *one, *other))?;
    let ratio = contrast_ratio(accent, background);
    (ratio < TEXT_CONTRAST).then(|| AccentShortfall {
        theme,
        accent,
        background,
        hundredths: hundredths(ratio),
    })
}

/// How `one` and `other` compare by the contrast each reaches with `colour`.
fn by_contrast_with(colour: Colour, one: Colour, other: Colour) -> Ordering {
    contrast_ratio(colour, one).total_cmp(&contrast_ratio(colour, other))
}

/// From 1, two equal colours, to 21, black on white.
fn contrast_ratio(one: Colour, other: Colour) -> f64 {
    let (one, other) = (luminance(one), luminance(other));
    (one.max(other) + 0.05) / (one.min(other) + 0.05)
}

/// The ratio in hundredths, rounded down, so that a ratio shown as reaching
/// a level always does.
fn hundredths(ratio: f64) -> u32 {
    // A ratio is between 1 and 21, so the cast neither wraps nor saturates.
    (ratio * 100.0).floor() as u32
}

/// WCAG's relative luminance, with the sRGB threshold of 0.04045 that the
/// sRGB standard gives and WCAG 2.2 adopts.
fn luminance(colour: Colour) -> f64 {
    0.2126 * linear(colour.red) + 0.7152 * linear(colour.green) + 0.0722 * linear(colour.blue)
}

/// One sRGB channel, from 0 to 255, as linear light, from 0 to 1.
fn linear(value: u8) -> f64 {
    let value = f64::from(value) / 255.0;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::entities::Accents;

    const BLACK: Colour = Colour::new(0, 0, 0);
    const WHITE: Colour = Colour::new(255, 255, 255);

    #[rstest]
    #[case::black_on_white(BLACK, WHITE, 2100)]
    #[case::white_on_black(WHITE, BLACK, 2100)]
    #[case::a_colour_on_itself(WHITE, WHITE, 100)]
    // #767676 on white is the lightest grey that reaches 4.5:1, and #595959 7:1.
    #[case::the_lightest_grey_of_4_5_to_1(Colour::new(0x76, 0x76, 0x76), WHITE, 454)]
    #[case::the_lightest_grey_of_7_to_1(Colour::new(0x59, 0x59, 0x59), WHITE, 700)]
    #[case::a_grey_just_short_of_7_to_1(Colour::new(0x5a, 0x5a, 0x5a), WHITE, 689)]
    fn colours_reach_their_well_known_ratios(
        #[case] one: Colour,
        #[case] other: Colour,
        #[case] expected: u32,
    ) {
        assert_eq!(hundredths(contrast_ratio(one, other)), expected);
    }

    #[test]
    fn no_colour_reaches_7_to_1_against_both_white_and_black() {
        let best = (0..=255)
            .map(|value| Colour::new(value, value, value))
            .map(|grey| contrast_ratio(grey, WHITE).min(contrast_ratio(grey, BLACK)))
            .fold(0.0, f64::max);
        assert_eq!(hundredths(best), 455);
    }

    #[rstest]
    #[case::just_short(6.999, 699)]
    #[case::exactly(7.0, 700)]
    fn hundredths_round_down(#[case] ratio: f64, #[case] expected: u32) {
        assert_eq!(hundredths(ratio), expected);
    }

    #[test]
    fn the_default_accents_pass() {
        for theme in Theme::ALL {
            assert_eq!(check_accent(theme, Accents::DEFAULT.of(theme)), None);
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
            check_accent(Theme::Dark, Accents::DEFAULT.light).map(|found| found.background),
            Some(palette(Theme::Dark).surface)
        );
    }
}
