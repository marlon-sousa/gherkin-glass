//! Entity: an accent colour that does not reach the contrast its theme requires, a thing that stops the build.

use crate::entities::{Colour, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AccentShortfall {
    pub(crate) theme: Theme,
    pub(crate) accent: Colour,
    /// The theme's background the accent contrasts least with.
    pub(crate) background: Colour,
    /// The ratio it reaches against that background, in hundredths, rounded down.
    pub(crate) hundredths: u32,
}
