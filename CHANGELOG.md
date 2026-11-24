# Changelog

Every release of gherkin-glass: the `ghg` command, the `cargo ghg` plugin and the sites they build. Each section is written before its release, from the changelog entries its commits carry, as [CONTRIBUTING.md](CONTRIBUTING.md), "Releases", describes, and becomes the notes of its [GitHub release](https://github.com/marlon-sousa/gherkin-glass/releases). Versions follow [Semantic Versioning](https://semver.org/).

## [1.1.0]

### Added

- --keep-in-output <pattern>, of ghg build and ghg serve, and the keep_in_output key of ghg.toml name what ghg leaves alone in its output folder, such as .nojekyll or CNAME, in the gitignore syntax.
- a release archive for Windows on ARM64, ghg-<tag>-aarch64-pc-windows-msvc.zip, which cargo binstall installs.
- ghg serve --watch says on standard error when the site it serves holds no feature file.
- ghg build --message-format json prints one JSON object on standard output saying how the build ended, for a program to read.

### Changed

- ghg build and ghg serve write nothing when they read no feature file, saying so on standard error and exiting with status 0, instead of writing an empty site.
- a list of --only that names no path builds nothing and exits with status 0, instead of stopping with status 1.

## [1.0.0]

The first stable release, on crates.io and on GitHub. It holds everything the two alphas brought, described here as a whole, with the one change since 0.1.0-alpha.2, pages that fit a window 320 pixels wide, folded in. From this version on, the addresses of pages and their anchors, the command line, the configuration keys and the names of the release archives change in a way that breaks a site, a command or an installation only in a new major version.

### Added

- ghg build turns a folder of Gherkin feature files, in any language Gherkin knows, into a static website: a page for every feature file and for every folder, with a heading and a stable link for every rule, scenario and background, and a breadcrumb on every page but the versions page.
- every feature file that cannot be read is reported, each as path:line: message, and then nothing is written.
- .ghg-ignore files choose which files are read, with the rules of .gitignore, and hidden files are left out.
- ghg.toml sets the input, the output and every other setting, each of which also has a default; a mistaken setting stops the run, naming its file and line.
- the --title option and the title setting give the site its title; ghg build hints how to set one when the site has none.
- allow_unnamed lets a feature, rule or scenario have no name, or stops the build at one; a block without a name still gets a link, an unnamed outline's runs are named "Scenario outline", and an unnamed examples block is named by its place among its outline's, such as "Examples 2".
- a heading names its block's kind, in gherkin-glass's own words, such as Scenario or Scenario outline, and its name; a block written with another of Gherkin's keywords, or in another language, says so in a note at the top of its content, whose words from "defined" on are a button that opens a dialog explaining it.
- in a file with rules, the scenarios outside every rule are grouped in a block of their own, "Outside any rule", which collapses like a rule and is listed in the page's contents, the folder pages and the folder tree.
- every rule, scenario, background and examples block has a button that collapses it, with Collapse all and Expand all, each block's state remembered, and a link to an anchor expanding its block; collapse_rules and collapse_scenarios make rules or scenarios start collapsed.
- each scenario holds the backgrounds it starts with as blocks of their own, headed "From the background" or "From the rule's background", followed by its own steps, headed "From the scenario"; the checkbox "Show backgrounds inside each scenario" expands them there and collapses the backgrounds in their own place, or the opposite, and inline_backgrounds chooses which state pages start in.
- an outline's examples are a list after its steps, introduced by "Runs once for each row of these examples:", each a collapsed item with its own button, holding the scenarios its rows produce under "Runs, one per row"; an examples block with no row says "(nothing to run)" after its name, one with nothing to show has a disabled button, and an outline none of whose examples has a row does not say it runs once for each row.
- keywords, quoted parameters, numbers and outline placeholders are highlighted in every step, never carrying meaning alone.
- feature text, names, steps, tables, doc strings, tags and file names included, reaches every page, the folder tree and search as text, never as markup a browser reads; on folder pages, tag pages, the page of step sentences and the folder tree, only a feature's own words take its language, so that gherkin-glass's words around them are read in English.
- source_link links each page to its feature file's source, such as the file on GitHub.
- a page for every tag under tags/, and an index of every tag, linked from the home page; tag_links links tags to other sites.
- steps.html lists every step sentence, with how often it is used and links to the steps that use it.
- a search box on every page, a combobox over the names, steps and tags of the site's features, rules, scenarios, backgrounds and examples blocks: Down and Up move through the results while the focus stays in the field, Enter opens the chosen result, or with none chosen the page of every result, which a link right after the field, "All results for", also opens, and Escape closes the list before it empties the field; that page, search.html, lists every result grouped by feature, with the words searched for in its address and what each result matched, and a feature file whose page would also be search.html stops the build.
- a folder tree beside every page, with the path to the current page expanded; on a window 48em wide or wider, the checkbox "Show the folder tree" and the show_tree setting show or hide it, and on a narrower one it starts hidden behind a "Folder tree" button, which Escape in the tree closes again; when a resize or a zoom hides the folder tree, or the control, that the keyboard focus is in, the focus moves to the control it shows.
- a Skip to content link at the start of every page, a footer naming the version of ghg that wrote it, and an icon of its own, a gherkin in a glass.
- pages have a light and a dark theme, following the reader's system, with a Theme control, System, Light or Dark, which the browser remembers; accent_light and accent_dark set each theme's accent colour, and a build stops when one has too little contrast.
- links, and the Collapse all and Expand all buttons, have a contrast of at least 7 to 1 in both colour schemes.
- every page fits a window 320 pixels wide without scrolling sideways: a long word, such as a file's name, a tag or an address, breaks where it must, and only a code block scrolls within itself.
- --front-page and the front_page setting give the site's front page an introduction written in Markdown, around ghg's own part of the page, whose tables are drawn as a feature file's.
- --version-name and version_name build one version of a site into its own folder, beside the others, with a page listing them, which ghg-versions.json at the output folder's root lists too; replace_existing_version keeps an earlier build instead of replacing it, and a version whose name differs from another only in its capitals is refused.
- ghg build --only <file> and ghg serve --only <file> build a site of only the feature files a list names, or the list on standard input with --only -, each page saying that it shows only the files given; a list that names no feature file builds a home page saying so, one that names no path stops the build, and ghg serve refuses --only with --no-build.
- ghg build --preview and ghg serve --preview build a site without versions, every page of which asks search engines not to index it, and refuse --version-name; a versioned output folder keeps a previews folder at its top, which ghg never looks into, writes or removes, and previews cannot be a version name.
- ghg never writes over files it did not write: it marks only a missing or empty output folder as its own, and a build stops, naming what it found, when the output folder holds anything ghg did not write.
- ghg serve builds the site and serves it on the reader's machine, at the address --host and --port give, or serves a site already built with --no-build; with --watch it rebuilds the site when a feature file, the configuration, the front page's template or the list of --only changes, and the open pages reload at the heading the reader was at.
- ghg serve stops gracefully on Control+C, SIGTERM and SIGHUP, or Control+C and Control+Break on Windows, saying "The site is no longer served." and exiting with status 0, after a build that is running finishes; a second request to stop ends it at once with status 130, closing the console on Windows ends it at once, and when the system refuses to tell ghg serve of those signals, it serves nothing and stops with status 1, saying why.
- a page opened from disk opens a dialog saying that the site is meant to be served, and how to serve it.
- two commands, ghg and cargo-ghg, so that a Rust project can run cargo ghg beside cargo test; cargo install gherkin-glass builds both, and cargo binstall gherkin-glass downloads them.
- archives for Linux on x86-64 and ARM64, macOS on Intel and Apple silicon, and Windows on x86-64, with a SHA256SUMS file, on every GitHub release; an alpha or a beta is released on GitHub with its archives and its site, but not published to crates.io.
- the project's own site shows what each release makes of an example suite and of the test fixtures, and opens on the README of the release.

## [0.1.0-alpha.2]

The second alpha, released on GitHub only, not on crates.io: sites of only the feature files a change touches, previews, a page of every search result, a new layout for outlines and backgrounds, and a graceful stop of ghg serve.

### Added

- after the backgrounds a scenario holds, its own steps are headed "From the scenario", at the backgrounds' level.
- on a window narrower than 48em, a "Folder tree" button shows and hides the folder tree, which starts hidden on every page, and Escape in the tree hides it again.
- every page links an icon of its own, assets/gherkin-glass.svg, a gherkin in a glass.
- every page ends with a footer that reads "Built with gherkin-glass" and the version of ghg that wrote it, linking gherkin-glass's repository.
- in a file with rules, the scenarios outside every rule are grouped in a block of their own, "Outside any rule", which collapses like a rule and is listed in the page's contents, the folder pages and the folder tree.
- an examples block with no row says "(nothing to run)" after its name, one with nothing to show has a disabled button, and an outline none of whose examples has a row does not say it runs once for each row.
- ghg serve stops gracefully on Control+C, SIGTERM and SIGHUP, or Control+C and Control+Break on Windows, saying "The site is no longer served." and exiting with status 0; closing the console on Windows ends it at once.
- a second request to stop while ghg serve waits for the requests being answered ends it at once with status 130.
- a request to stop while a build runs lets the build finish first, then stops without starting to serve; when the system refuses to tell ghg serve of those signals, it builds and serves nothing and stops with status 1, saying "ghg serve cannot handle Control+C and the signals that stop it" and why.
- ghg build --only <file> and ghg serve --only <file> build a site of only the feature files the list in the file names, or the list on standard input with --only -, passing over every other path, each page saying that it shows only the files given; a list that names no feature file builds a home page saying so, and under ghg serve --watch, a change to the list's file rebuilds the site from the new list.
- ghg build --only and ghg serve --only stop when the list names no path, and ghg serve refuses --only with --no-build, which builds nothing.
- the GitHub Action's input only passes --only to ghg build.
- ghg build --preview and ghg serve --preview build a site without versions, whatever version_name says, every page of which asks search engines not to index it, and refuse --version-name.
- the GitHub Action's input preview passes --preview to ghg build when it is true.
- a versioned output folder keeps a previews folder at its top, which ghg never looks into, writes or removes, and previews cannot be a version name.
- the page of search results, search.html, lists every result grouped by feature, with the words searched for in its address, and the steps, tags and cells each result matched; its own search box works as on every other page.
- search finds backgrounds, by their names and steps, and examples blocks, by their names, tags and cells.
- feature text, names, steps, tables, doc strings, tags and file names included, reaches every page, the folder tree and search as text, never as markup a browser reads.

### Changed

- each scenario holds the backgrounds it starts with as blocks of their own, headed "From the background" or "From the rule's background", one heading level below the scenario, which expand and collapse like any other block.
- checking "Show backgrounds inside each scenario" expands the backgrounds inside the scenarios and collapses those in their own place, and unchecking it does the opposite; inline_backgrounds chooses which state pages start in.
- the checkbox "Show the folder tree" shows only on a window 48em wide or wider, and its remembered choice no longer shows the tree on a narrow one.
- a heading names only its block's kind, in gherkin-glass's own words, such as Scenario or Scenario outline, and its name; a block written with another of Gherkin's keywords, or in another language, says so in a note at the top of its content, such as "Note: in the Gherkin feature file, this scenario is defined with the keyword "Example".", whose words from "defined" on are a button that opens a dialog explaining it.
- an outline's examples are a list after its steps, introduced by "Runs once for each row of these examples:", each a collapsed item with its own button instead of a heading, holding the runs its rows produce under "Runs, one per row".
- on folder pages, tag pages, the page of step sentences and the folder tree, only a feature's own words, its names and keywords, take its language, so that gherkin-glass's words around them are read in English.
- an unnamed outline's runs are named "Scenario outline", whatever keyword the file writes.
- an unnamed examples block is named by its place among its outline's, such as "Examples 2".
- the search box is a combobox: Down and Up move through the results while the focus stays in the field, Enter opens the chosen result or the page of every result, and Escape closes the list before it empties the field.
- the search box has a link, "All results for", to the page of search results, and typing narrows its results at once.
- a feature file whose page would be search.html stops the build, as one at steps.html does.

### Fixed

- the folder tree follows a resize across 48em without a reload, so that it no longer pushes the content down on a phone after a resize.
- the search form, the options bar, the search results, the folder tree's wrapped items, the breadcrumb's separators and a tag page's feature headings are laid out in line with the rest of the page.
- on a narrow window, a feature page's buttons and the backgrounds checkbox wrap to the next line instead of squeezing their words.
- when resizing or zooming the window hides the folder tree while the keyboard focus is in it, or hides the "Show the folder tree" checkbox or the "Folder tree" button while the focus is on it, the focus moves to the control the resize shows instead of the page's start.

### Removed

- the link from each scenario to its backgrounds, which showed while "Show backgrounds inside each scenario" was unchecked.

## [0.1.0-alpha.1]

The first release, an alpha: a preview of gherkin-glass, released on GitHub only, not on crates.io.

### Added

- ghg build turns a folder of Gherkin feature files, in any language Gherkin knows, into a static website: a page for every feature file and for every folder, with a heading and a stable link for every rule, scenario and background, and a breadcrumb on every page but the versions page.
- every feature file that cannot be read is reported, each as path:line: message, and then nothing is written.
- .ghg-ignore files choose which files are read, with the rules of .gitignore, and hidden files are left out.
- allow_unnamed lets a feature, rule or scenario have no name, or stops the build at one; a block without a name still gets a link.
- source_link links each page to its feature file's source, such as the file on GitHub.
- every rule, scenario, background and examples heading has a button that collapses its block, with Collapse all and Expand all, each block's state remembered, and a link to an anchor expanding its block; collapse_rules and collapse_scenarios make rules or scenarios start collapsed.
- ghg serve builds the site and serves it on the reader's machine, at the address --host and --port give, or serves a site already built with --no-build; with --watch it rebuilds the site when a feature file, the configuration or the front page's template changes, and the open pages reload at the heading the reader was at.
- ghg.toml sets the input, the output and every other setting, each of which also has a default; a mistaken setting stops the run, naming its file and line.
- the --title option and the title setting give the site its title; ghg build hints how to set one when the site has none.
- a folder tree beside every page, with the path to the current page expanded, shown or hidden by a checkbox and remembered, and the show_tree setting.
- a Skip to content link at the start of every page.
- each scenario shows its backgrounds' steps, or links to them, with a checkbox to switch between the two and the inline_backgrounds setting for the default; while the steps are shown inside each scenario, the background blocks start collapsed, checking the box collapses them all and unchecking it expands them all, and each form remembers their state apart.
- a page for every tag under tags/, and an index of every tag, linked from the home page; tag_links links tags to other sites.
- keywords, quoted parameters, numbers and outline placeholders are highlighted in every step, never carrying meaning alone.
- pages have a light and a dark theme, following the reader's system, with a Theme control, System, Light or Dark, which the browser remembers; accent_light and accent_dark set each theme's accent colour, and a build stops when one has too little contrast.
- links, and the Collapse all and Expand all buttons, have a contrast of at least 7 to 1 in both colour schemes.
- --version-name and version_name build one version of a site into its own folder, beside the others, with a page listing them, which ghg-versions.json at the output folder's root lists too; replace_existing_version keeps an earlier build instead of replacing it, and a version whose name differs from another only in its capitals is refused.
- ghg never writes over files it did not write: it marks only a missing or empty output folder as its own, and a build stops, naming what it found, when the output folder holds anything ghg did not write.
- each scenario outline shows the scenarios its examples produce, under "Examples as scenarios" and the outline's name.
- steps.html lists every step sentence, with how often it is used and links to the steps that use it.
- a search box on every page, over the names, steps and tags of the site's features, rules and scenarios.
- --front-page, the front_page setting and the action's front-page input give the site's front page an introduction written in Markdown, around ghg's own part of the page.
- a page opened from disk opens a dialog saying that the site is meant to be served, and how to serve it.
- two commands, ghg and cargo-ghg, so that a Rust project can run cargo ghg beside cargo test.
- archives for Linux on x86-64 and ARM64, macOS on Intel and Apple silicon, and Windows on x86-64, with a SHA256SUMS file, on every GitHub release; cargo binstall gherkin-glass installs from them once a version is on crates.io.
- a GitHub Action that installs ghg on a runner and, with build: true, builds the site; this alpha is installed with `uses: marlon-sousa/gherkin-glass@v0.1.0-alpha.1` and the input `version: 0.1.0-alpha.1`.
- the project's own site shows what each release makes of an example suite and of the test fixtures, and opens on the README of the release.
- an alpha or a beta, such as this one, is released on GitHub with its archives and its site, but not published to crates.io.
