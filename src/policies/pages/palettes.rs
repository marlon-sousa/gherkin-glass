//! Policy: the built-in colours of each theme a reader can pick.

use crate::entities::{Colour, Palette, Theme};

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
