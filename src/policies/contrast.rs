//! Policy: the contrast between two colours, as WCAG 2.2 defines it, and the contrast gherkin-glass requires.

use crate::entities::Colour;

/// What text and its highlighting must reach against every background they
/// are written on: WCAG's AAA level, success criterion 1.4.6.
pub(crate) const TEXT_CONTRAST: f64 = 7.0;

/// What a line that carries no text must reach against the background:
/// WCAG's success criterion 1.4.11.
#[cfg(test)]
pub(crate) const PART_CONTRAST: f64 = 3.0;

/// From 1, two equal colours, to 21, black on white.
pub(crate) fn contrast_ratio(one: Colour, other: Colour) -> f64 {
    let (one, other) = (luminance(one), luminance(other));
    let (lighter, darker) = if one > other {
        (one, other)
    } else {
        (other, one)
    };
    (lighter + 0.05) / (darker + 0.05)
}

/// The ratio in hundredths, rounded down, so that a ratio shown as reaching
/// a level always does.
pub(crate) fn hundredths(ratio: f64) -> u32 {
    // A ratio is between 1 and 21, so the cast neither wraps nor saturates.
    (ratio * 100.0).floor() as u32
}

/// WCAG's relative luminance, with the sRGB threshold of 0.04045 that the
/// sRGB standard gives and WCAG 2.2 adopts.
fn luminance(colour: Colour) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(colour.red) + 0.7152 * channel(colour.green) + 0.0722 * channel(colour.blue)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLACK: Colour = Colour::new(0, 0, 0);
    const WHITE: Colour = Colour::new(255, 255, 255);

    #[test]
    fn black_on_white_is_21_and_a_colour_on_itself_is_1() {
        assert_eq!(contrast_ratio(BLACK, WHITE), 21.0);
        assert_eq!(contrast_ratio(WHITE, BLACK), 21.0);
        assert_eq!(contrast_ratio(WHITE, WHITE), 1.0);
    }

    #[test]
    fn known_ratios_agree_with_wcag_s_own_examples() {
        // #767676 on white is the lightest grey that reaches 4.5:1, and #595959 7:1.
        assert_eq!(
            hundredths(contrast_ratio(Colour::new(0x76, 0x76, 0x76), WHITE)),
            454
        );
        assert_eq!(
            hundredths(contrast_ratio(Colour::new(0x59, 0x59, 0x59), WHITE)),
            700
        );
        assert!(contrast_ratio(Colour::new(0x5a, 0x5a, 0x5a), WHITE) < TEXT_CONTRAST);
    }

    #[test]
    fn no_colour_reaches_7_to_1_against_both_white_and_black() {
        let best = (0..=255)
            .map(|value| Colour::new(value, value, value))
            .map(|grey| contrast_ratio(grey, WHITE).min(contrast_ratio(grey, BLACK)))
            .fold(0.0, f64::max);
        assert_eq!(hundredths(best), 455);
    }

    #[test]
    fn hundredths_round_down() {
        assert_eq!(hundredths(6.999), 699);
        assert_eq!(hundredths(7.0), 700);
    }
}
