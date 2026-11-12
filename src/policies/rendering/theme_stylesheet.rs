//! Policy: the stylesheet of colour tokens, one custom property per colour, for each theme a reader can pick.

use crate::entities::{Accents, Colour, Theme};
use crate::policies::palette;

/// The tokens of the light theme on the page's root, overridden by the dark
/// theme's when the reader picked dark, or picked nothing and their system
/// prefers dark. The page's root has `data-theme="light"` or `data-theme="dark"`
/// when the reader picked one, and no `data-theme` for the system's choice.
pub(crate) fn theme_stylesheet(accents: &Accents) -> String {
    let light = tokens(Theme::Light, accents, "  ");
    let dark = tokens(Theme::Dark, accents, "  ");
    let dark_nested = tokens(Theme::Dark, accents, "    ");
    format!(
        ":root,\n:root[data-theme=\"light\"] {{\n  color-scheme: light;\n{light}}}\n\n\
         @media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n    \
         color-scheme: dark;\n{dark_nested}  }}\n}}\n\n\
         :root[data-theme=\"dark\"] {{\n  color-scheme: dark;\n{dark}}}\n"
    )
}

fn tokens(theme: Theme, accents: &Accents, indent: &str) -> String {
    let palette = palette(theme);
    let colours: [(&str, Colour); 10] = [
        ("background", palette.background),
        ("surface", palette.surface),
        ("text", palette.text),
        ("muted", palette.muted),
        ("border", palette.border),
        ("accent", accents.of(theme)),
        ("keyword", palette.keyword),
        ("parameter", palette.parameter),
        ("number", palette.number),
        ("placeholder", palette.placeholder),
    ];
    colours
        .into_iter()
        .map(|(name, colour)| format!("{indent}--ghg-{name}: {colour};\n"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_theme_s_tokens_and_the_system_s_choice() {
        let accents = Accents {
            light: Colour::new(0, 0, 0x80),
            dark: Colour::new(0xff, 0xff, 0x80),
        };
        assert_eq!(
            theme_stylesheet(&accents),
            r#":root,
:root[data-theme="light"] {
  color-scheme: light;
  --ghg-background: #ffffff;
  --ghg-surface: #f2f2f2;
  --ghg-text: #1a1a1a;
  --ghg-muted: #474747;
  --ghg-border: #757575;
  --ghg-accent: #000080;
  --ghg-keyword: #6a1b9a;
  --ghg-parameter: #14521a;
  --ghg-number: #8a3000;
  --ghg-placeholder: #00575e;
}

@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --ghg-background: #141414;
    --ghg-surface: #222222;
    --ghg-text: #ececec;
    --ghg-muted: #c4c4c4;
    --ghg-border: #8f8f8f;
    --ghg-accent: #ffff80;
    --ghg-keyword: #e1b3ff;
    --ghg-parameter: #9fe0a8;
    --ghg-number: #ffbf8a;
    --ghg-placeholder: #7fdbe0;
  }
}

:root[data-theme="dark"] {
  color-scheme: dark;
  --ghg-background: #141414;
  --ghg-surface: #222222;
  --ghg-text: #ececec;
  --ghg-muted: #c4c4c4;
  --ghg-border: #8f8f8f;
  --ghg-accent: #ffff80;
  --ghg-keyword: #e1b3ff;
  --ghg-parameter: #9fe0a8;
  --ghg-number: #ffbf8a;
  --ghg-placeholder: #7fdbe0;
}
"#
        );
        assert!(theme_stylesheet(&Accents::DEFAULT).contains("--ghg-accent: #8cc4ff;"));
    }
}
