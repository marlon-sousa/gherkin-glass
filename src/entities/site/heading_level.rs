//! Entity: the level of a heading under a page's `h1`, a block's on a feature's page or one of ghg's own on the front page.

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
