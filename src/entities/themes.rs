//! Entity: the colours, the themes a reader can pick, and an accent that does not reach its contrast.

mod accent_shortfall;
mod colour;
mod theme;

pub(crate) use accent_shortfall::AccentShortfall;
pub(crate) use colour::Colour;
pub(crate) use theme::Accents;
pub(crate) use theme::{Palette, Theme};
