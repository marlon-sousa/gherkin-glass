//! Entity: the level of a block's heading on a feature's page.

/// The level of a block's heading, under the page's `h1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeadingLevel {
    Two,
    Three,
    Four,
    Five,
}

impl HeadingLevel {
    /// The number of its `h` element, from 2 to 5.
    pub(crate) fn number(self) -> u8 {
        match self {
            HeadingLevel::Two => 2,
            HeadingLevel::Three => 3,
            HeadingLevel::Four => 4,
            HeadingLevel::Five => 5,
        }
    }

    /// The level of the blocks under one at this level. Nothing is under an `h5`'s block, so
    /// the level below it is `h5` again.
    pub(crate) fn below(self) -> HeadingLevel {
        match self {
            HeadingLevel::Two => HeadingLevel::Three,
            HeadingLevel::Three => HeadingLevel::Four,
            HeadingLevel::Four | HeadingLevel::Five => HeadingLevel::Five,
        }
    }
}
