# Changelog

Every release of gherkin-glass: the `ghg` command, the `cargo ghg` plugin and the sites they build. Each section is written before its release, from the changelog entries its commits carry, as [CONTRIBUTING.md](CONTRIBUTING.md), "Releases", describes, and becomes the notes of its [GitHub release](https://github.com/marlon-sousa/gherkin-glass/releases). Versions follow [Semantic Versioning](https://semver.org/).

## [0.1.0-alpha.1]

The first release, an alpha: a preview of gherkin-glass, released on GitHub only, not on crates.io.

### Added

- ghg build turns a folder of Gherkin feature files into a static website: a page for every feature file and for every folder, with a heading and a stable link for every rule, scenario and background.
- ghg serve builds the site and serves it on the reader's machine, and with --watch rebuilds it when a feature file, the configuration or the front page's template changes.
- ghg.toml sets the input, the output and every other setting, each of which also has a default; a mistaken setting stops the run, naming its file and line.
- the --title option and the title setting give the site its title; ghg build hints how to set one when the site has none.
- a folder tree beside every page, with the path to the current page expanded, shown or hidden by a checkbox and remembered, and the show_tree setting.
- a Skip to content link at the start of every page.
- each scenario shows its backgrounds' steps, or links to them, with a checkbox to switch between the two and the inline_backgrounds setting for the default; while the steps are shown inside each scenario, the background blocks start collapsed.
- a page for every tag under tags/, and an index of every tag, linked from the home page; tag_links links tags to other sites.
- keywords, quoted parameters, numbers and outline placeholders are highlighted in every step, never carrying meaning alone.
- pages have a light and a dark theme, following the reader's system, with a Theme control, System, Light or Dark, which the browser remembers; accent_light and accent_dark set each theme's accent colour, and a build stops when one has too little contrast.
- links, and the Collapse all and Expand all buttons, have a contrast of at least 7 to 1 in both colour schemes.
- --version-name and version_name build one version of a site into its own folder, beside the others, with a page listing them, which ghg-versions.json at the output folder's root lists too; replace_existing_version keeps an earlier build instead of replacing it, and a version whose name differs from another only in its capitals is refused.
- ghg never writes over files it did not write: it marks only a missing or empty output folder, and a build stops when the output folder holds anything else, naming what it found.
- each scenario outline shows the scenarios its examples produce, under "Examples as scenarios" and the outline's name.
- steps.html lists every step sentence, with how often it is used and links to the steps that use it.
- a search box on every page, over the names, steps and tags of the site's features, rules and scenarios.
- --front-page, the front_page setting and the action's front-page input give the site's front page an introduction written in Markdown, around ghg's own part of the page.
- a page opened from disk opens a dialog saying that the site is meant to be served, and how to serve it.
- ghg and cargo-ghg, so that a Rust project can run cargo ghg beside cargo test.
- archives for Linux on x86-64 and ARM64, macOS on Intel and Apple silicon, and Windows on x86-64, with a SHA256SUMS file, on every GitHub release; cargo binstall gherkin-glass installs from them once a version is on crates.io.
- a GitHub Action, `uses: marlon-sousa/gherkin-glass@v0.1.0-alpha.1`, that installs ghg on a runner and, with build: true, builds the site.
- the project's own site shows what each release makes of an example suite and of the test fixtures, and opens on the README of the release.
- an alpha or a beta, such as this one, is released on GitHub with its archives and its site, but not published to crates.io.
