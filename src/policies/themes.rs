//! Policy: the colours, each theme's palette, the default accents and the check of a configured accent, the contrast between colours, and the stylesheet of colour tokens.

mod contrast;
mod palettes;
mod theme_stylesheet;

#[cfg(test)]
pub(crate) use contrast::PART_CONTRAST;
pub(crate) use contrast::{TEXT_CONTRAST, contrast_ratio, hundredths};
pub(crate) use palettes::DEFAULT_ACCENTS;
pub(crate) use palettes::{check_accent, palette};
pub(crate) use theme_stylesheet::theme_stylesheet;
