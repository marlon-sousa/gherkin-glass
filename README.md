# gherkin-glass

gherkin-glass turns a folder of Gherkin feature files into a static website that anyone can read, with or without a screen reader. It is one command, `ghg`, for Linux, macOS and Windows.

## Why it exists

Gherkin is the plain-language format behaviour-driven development uses to describe how software should behave. A feature file names a feature and lists its scenarios, each a short sequence of steps such as `Given a customer named "Ana"`, `When she withdraws 50`, `Then her balance is 150`. Tools such as Cucumber run those steps as tests, but the files are written to be read by people: product owners, testers, developers and anyone who wants to know what the software does.

Reading them is harder than it should be. They live in a repository, spread across folders, and are opened one at a time in an editor. gherkin-glass makes those use cases easy to read, browse, search and link to, for everyone, including people who use a screen reader or only a keyboard.

## What you get

- A page for every feature file and for every folder, with a folder tree beside them.
- A heading and a stable link for every rule, scenario and background, so that a link to a scenario keeps working as long as its file and its name do.
- A page for every tag, listing the features and scenarios that carry it, and optional links from tags to another site, such as an issue tracker.
- A page of step sentences, listing every step and where it is used.
- Search across features, rules and scenarios.
- Every Gherkin construct: rules, backgrounds, scenario outlines and their examples, shown as scenarios too, data tables, doc strings, tags, descriptions, and every spoken language Gherkin supports.
- Highlighting of keywords, quoted parameters, placeholders and numbers in steps, which never carries meaning on its own.
- Reader options the browser remembers: a light or a dark theme, showing or hiding the folder tree, showing a background's steps inside each scenario or once at the top, and collapsing rules and scenarios.
- Pages checked for accessibility on every change: headings in order, real tables and lists, a skip link, everything reachable with the keyboard, and a contrast of at least 7:1 for text in both themes.

## Status

Not yet released.

## Installing and running

Once gherkin-glass is released, install it in one of these ways:

- `cargo install gherkin-glass`, which builds it with Rust's package manager;
- `cargo binstall gherkin-glass`, which downloads the released binary instead of building it;
- or download the archive for your system from the repository's releases page, and put `ghg` somewhere on your path. There are archives for Linux on x86-64 and ARM64, macOS on Intel and Apple silicon, and Windows on x86-64.

Each way installs `ghg`, and `cargo-ghg`, which lets you run it as `cargo ghg` beside `cargo test`. `cargo ghg build` is the same as `ghg build`.

Then, in the folder of your project, with your feature files in a folder named `features`:

```sh
ghg build
ghg serve
```

`ghg build` writes the site into a folder named `site`, and `ghg serve` serves it at <http://127.0.0.1:8000/>. Open that address in a browser, and press Control+C to stop.

## Usage

### The configuration file

ghg reads its settings from `ghg.toml`, a [TOML](https://toml.io/) file in the folder you run it from, or the file `--config` names. The file is optional: every setting has a default. Paths in it are relative to the folder the file is in, wherever you run ghg from.

```toml
input = "features"
output = "site"
title = "Our features"
source_link = "https://github.com/owner/repo/blob/main/features/{path}#L{line}"
collapse_scenarios = true

[[tag_links]]
pattern = '@issue-(\d+)'
address = "https://github.com/owner/repo/issues/$1"
text = "Issue $1 on GitHub"
```

| Key | Takes | Default | What it sets |
| --- | --- | --- | --- |
| `input` | a path | `features` | The folder of feature files to read. It must exist. |
| `output` | a path | `site` | The folder the site is written into. |
| `title` | text | `Feature files` | The site's title, shown on its home page and in every breadcrumb. |
| `allow_unnamed` | `true` or `false` | `true` | Whether a feature, rule or scenario may have no name. When `false`, one without a name stops the build. |
| `source_link` | text | none | The address of each feature file's source, so that each page links to its file. `{path}` is replaced by the file's path and `{line}` by the line of its feature. |
| `collapse_rules` | `true` or `false` | `false` | Whether rules start collapsed. |
| `collapse_scenarios` | `true` or `false` | `false` | Whether scenarios, scenario outlines and backgrounds start collapsed. |
| `show_tree` | `true` or `false` | `true` | Whether the folder tree is shown when a reader first opens the site. |
| `inline_backgrounds` | `true` or `false` | `true` | Whether a scenario shows its backgrounds' steps inside it, or links to them. |
| `tag_links` | a list of tables, each written `[[tag_links]]` | none | Links from tags to another site. Each table has a `pattern`, a regular expression matched against the whole tag with its `@`; an `address`; and a `text` for the link. In the address and the text, `$1` or `$name` is replaced by what that group of the pattern matched. |
| `accent_light` | a colour, such as `"#0b4f9c"` | `"#0b4f9c"` | The light theme's colour for links and the focus outline. It must reach a contrast of 7:1 against the theme's backgrounds. |
| `accent_dark` | a colour, such as `"#8cc4ff"` | `"#8cc4ff"` | The dark theme's colour for links and the focus outline, held to the same contrast. |
| `version_name` | text | none | Builds the site as a version of that name, as "Versions" below describes. |
| `replace_existing_version` | `true` or `false` | `true` | Whether a build replaces the site, or the version, that an earlier build wrote in the output folder. When `false`, it stops instead. |

A key ghg does not know, or a value of the wrong kind, stops the run with a message naming the file and the line, so that nothing is silently ignored.

### Which files are read

ghg reads every file whose name ends in `.feature` under the input folder, and leaves out hidden files and folders, those whose names start with a dot. A file named `.ghg-ignore`, in any folder of the input, changes that with the same rules as a `.gitignore` file: `drafts/` leaves a folder out, and `!*.story` reads files ending in `.story` too.

When a file cannot be read, such as one with a syntax mistake, ghg reports every such file, each as `<path>:<line>: <message>`, and writes nothing.

### The command line

```text
ghg build [options]
ghg serve [options]
```

| Option | Of | What it does |
| --- | --- | --- |
| `--config <file>` | both | Reads that configuration file instead of `ghg.toml` in the current folder. |
| `--input <folder>` | both | Reads that folder, instead of the configuration's `input`. |
| `--output <folder>` | both | Writes the site into that folder, instead of the configuration's `output`. |
| `--title <text>` | both | The site's title, instead of the configuration's `title`. |
| `--version-name <name>` | both | Builds the site as that version, instead of the configuration's `version_name`. |
| `--host <address>` | `ghg serve` | Listens on that address instead of `127.0.0.1`, such as `0.0.0.0` to be reached from other machines. |
| `--port <number>` | `ghg serve` | Listens at that port instead of 8000. `0` lets the system choose a free one. |
| `--no-build` | `ghg serve` | Serves the output folder as it is, without building it. |
| `--watch` | `ghg serve` | Rebuilds the site whenever a feature file or the configuration changes, and reloads the pages open in the browser. |

An option on the command line wins over the configuration file's key, and the key wins over the default. `ghg --help` lists the commands and options, and `ghg --version` prints the version.

`ghg build` exits with status 0 when it wrote the site, 1 when the configuration or a feature file stopped it, and 2 for a mistake on the command line.

### Looking at a site and publishing it

A site is meant to be served by a web server, not opened from disk. Opened from disk, a browser does not run the page's script, so the folder tree, search and the reader options do not work. Every page opened from disk therefore first opens a dialog, "Opened from disk", that says the site is meant to be served and how to serve it; close it with its button, "I agree", or with Escape, to read the page.

To look at a site on your own machine, use `ghg serve`. With `--watch`, the site is built in memory and served from there, the output folder is left as it is, and the page you are reading reloads at the same heading after each change. With `--no-build`, ghg serves a site already built, such as one downloaded from a CI run.

To publish a site, put the output folder on any static web host, GitHub Pages included. Every link in a site is relative, so it works at any address, below a path such as `/<repository>/` too.

### The output folder belongs to ghg

`ghg build` marks the output folder as its own with a file at its root, `ghg-site.json`, or `ghg-versions.json` when it builds versions. From then on the output folder, and every folder in it, belongs to ghg: any build may empty it and write it again, so anything you put there by hand can be lost.

To keep that from happening by mistake:

- ghg marks an output folder only when it is missing or empty. It refuses to build into a folder that holds anything and no marker.
- Before every build, ghg checks that a marked folder holds only what it wrote there. When it holds anything else, ghg names it and stops, before reading the features, and writes and deletes nothing.

So keep the output folder for the site alone, and put anything you want served beside it somewhere else.

### Versions

One site can hold several versions of the same suite, so that readers read the version they use. Give each build a version name, with `--version-name` or `version_name`, such as `v1.2.0` or `main`. Each version is written into a folder of its own in the output folder, such as `site/v1.2.0/`, and the output folder's `index.html` lists every version: development versions, releases newest first, and pre-releases.

ghg does not read git. To build several versions, check each one out into a folder of its own and run ghg once for each, into the same output folder. Building a version again replaces it, unless `replace_existing_version` is `false`.

## Contributing

### Requirements

- **Rust**, through [rustup](https://rustup.rs/) 1.28 or later. In the repository, `rustup toolchain install` installs the Rust version the repository pins, with its components.
- **Node**, at the version `script/.node-version` names, with npm, only to change the pages' script or stylesheet. The script is TypeScript, in `script/`, and its compiled form, `assets/gherkin-glass.js`, is committed.
- **cargo-deny 0.20.2**, for the dependency check: `cargo install cargo-deny --version 0.20.2 --locked`.

If you don't change the pages' script or stylesheet, Rust is enough: `cargo build` builds `ghg`, `cargo test` runs the tests, and `cargo run -- serve --config project-site/ghg.toml --watch` serves the repository's example suite at <http://127.0.0.1:8000/>.

### The xtask commands

`cargo xtask` runs the repository's own tasks. `cargo xtask --help` lists them.

| Command | What it does | Needs Node |
| --- | --- | --- |
| `cargo xtask build` | Type-checks, tests and compiles the script into `assets/gherkin-glass.js`, writes again the expected pages the tests compare against, and builds the release binary. Run it after changing the script, the stylesheet or a page's template, and commit what it writes. | Yes |
| `cargo xtask check` | Runs every check a pull request runs: formatting, lints, tests on the pinned and the minimum supported Rust, documentation, dependencies, the package, the script check and the browser check. | Yes |
| `cargo xtask check-script` | Type-checks and tests the script, and fails when the committed `assets/gherkin-glass.js` is not what `script/` compiles to. | Yes |
| `cargo xtask check-browser` | Installs a headless Chromium and checks every test site in it: accessibility in both colour schemes, the keyboard, pages opened from disk, and the reload of `ghg serve --watch`. | Yes |
| `cargo xtask comment-ratio [path ...]` | Counts the comment lines and the code lines of the Rust and TypeScript files given, or of every one in the repository. | No |

### Interface text and localization

Every word gherkin-glass writes itself, on the pages and on the command line, such as "Skip to content", "Collapse all" or a message about a mistake in `ghg.toml`, comes from one catalogue of strings. Feature files are shown as written, in their own language; only gherkin-glass's own words come from the catalogue.

- The catalogue is in [Fluent](https://projectfluent.org/), in `messages/en/gherkin-glass.ftl`, and built into the binary. English is the only language for now; a translation is a new file in a folder of its own beside `en/`.
- Code reads a string by its id, with the `fl!` macro, such as `fl!(CATALOGUE, "skip-to-content")`. `fl!` checks at compile time that the id exists in the English catalogue and that its arguments are the ones the message takes, so a missing string is a build error, not a broken page.
- No template, Rust code or script writes interface text inline. The pages' templates receive their words from Rust, and the script receives its own, such as the search box's messages, from attributes the page carries, so the script holds no words.
- A message is a whole sentence with named placeholders, such as `{ $feature }, a feature in { $folder }`, never pieces joined together, since word order differs between languages. A count is a plural form chosen by Fluent, never a word with an `s` added.

To add or change a string, edit it in the catalogue, use it by its id, and run `cargo xtask build` when it changes what pages say, so that the expected pages the tests compare against are written again.

### How changes are made

Every change starts as an issue. Open one that says what you want and why, or, for a bug, what happened, what you expected and the smallest feature file that shows it. The maintainers then design the plan with you in the issue, and work starts only once it is agreed: a pull request that names its issue.

### How it is built

gherkin-glass is written in Rust, with the [`gherkin`](https://crates.io/crates/gherkin) parser from cucumber-rs. AI is part of how it is made: agents write code and documentation, and a human maintainer reviews everything before it is merged, under the terms of [A manifesto for software engineering with AI](https://marlon-sousa.com/blog/manifesto/).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
