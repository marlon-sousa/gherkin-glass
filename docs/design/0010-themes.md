# 0010: The themes' colours in each page, and a script in its head

Decided while building issue [#10](https://github.com/marlon-sousa/gherkin-glass/issues/10), on 2026-11-10.

## Decisions

1. **Two accent colours, one per theme.** No colour reaches 7:1 against both a light and a dark background: against pure white and pure black at once, the best any colour reaches is about 4.56:1. So `accent_light` and `accent_dark` are set and checked apart. The maintainer chose this.
2. **The colours are custom properties written into each page's head**, in a `style` element, by the build. The stylesheet in `assets/` is the same in every site, as the pages promise, so it cannot hold a site's accents; it takes every colour from the properties instead. Generating the properties in Rust also means the colours the build checks the accents against are the very ones the pages use.
3. **A line of classic script in each page's head applies the remembered theme**, by setting `data-theme` on the page's root before anything is drawn. The pages' own script is an ES module, which runs only once the page has been read, so a reader who chose dark would first see the page light.

## What else was considered

- **One accent colour, with the dark theme's computed from it.** It changes the colour the person chose, and they cannot see what they will get.
- **A generated stylesheet file beside the assets**, such as `assets/theme.css`. It needs one more request before the page can be drawn, and breaks the promise that the assets are the same in every site.
- **Applying the theme from the module.** It draws the page in the system's theme first, a flash a reader sees on every page.

## Why

Each choice keeps the page right from its first frame, in the colours that were checked, without a second source of truth for the colours.
