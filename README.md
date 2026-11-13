# gherkin-glass

A static viewer for Gherkin `.feature` files that anyone can read, with or without a screen reader.

gherkin-glass turns a folder of feature files into a static website: a page for every feature, a heading and a stable link for every scenario, an index for every tag, and a page for every folder. The content is in the HTML of each page, and a stylesheet, a script and the data they share, written once with the site, style the pages and drive their interactive parts. The site is meant to be served, by any static web server or host, GitHub Pages included, or by `ghg serve` on your own machine. It ships as a single binary for Linux, macOS and Windows.

## Status

Everything [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md) describes is built, and nothing is released yet. Until the first release, you build gherkin-glass from this repository, as "Trying it from a clone" below describes. What is planned is in the [issues](https://github.com/marlon-sousa/gherkin-glass/issues). Once the first release is published, [the project's own site](https://marlon-sousa.github.io/gherkin-glass/) shows what each release makes of an example suite and of the test fixtures.

## Goals

- **Readable by everyone.** Headings in order, real tables, lists for steps, and a skip link. Colour and highlighting help, but never carry meaning on their own. Accessibility is checked on every pull request.
- **Links that last.** A scenario's address comes from its file and its name, by a documented rule, so a link keeps working as long as both do.
- **Reader options.** Readers can choose to show a background's steps inside each scenario or once at the top, and to show or hide the folder tree. Rules and scenarios can be collapsed from their headings, and the browser remembers every choice.
- **Every Gherkin construct.** Rules, backgrounds, scenario outlines and their examples, data tables, doc strings, tags, descriptions, and every spoken language Gherkin supports.
- **One file to install.** A single binary, configured by a TOML file, that also runs as a GitHub Action.

## Trying it from a clone

You need [rustup](https://rustup.rs/) 1.28 or later, which installs Rust. Nothing else: the pages' script is written in TypeScript, but its compiled form is committed, so building gherkin-glass needs no Node.

1. Clone the repository and go into it:

   ```sh
   git clone https://github.com/marlon-sousa/gherkin-glass.git
   cd gherkin-glass
   ```

2. Run `rustup toolchain install`, which installs the Rust version the repository pins in `rust-toolchain.toml`, then `cargo build`. The stylesheet and the script are built into the binary, which writes them into every site.

3. Look at the example suite, in `project-site/features`:

   ```sh
   cargo run -- serve --config project-site/ghg.toml --watch
   ```

   Then open <http://127.0.0.1:8000/> in a browser, and press Control+C in the terminal to stop. With `--watch`, the site is built in memory and nothing is written to disk, and the page reloads when you change a feature file. Without it, `ghg serve` writes the site into the configuration's output folder first, which for the example suite is the committed `project-site/expected`.

4. To have the `ghg` command everywhere, install it from the clone with `cargo install --path . --locked`. That installs `ghg`, and `cargo-ghg`, which lets cargo run it as `cargo ghg`.

## Using it on your own features

In the folder of your project, write a `ghg.toml`:

```toml
input = "features"
output = "site"
title = "Our features"
```

Then:

- `ghg build` reads every `.feature` file under `features`, and writes the site into `site`, or reports every file it cannot read and writes nothing.
- `ghg serve` builds the site and serves it at <http://127.0.0.1:8000/>. Add `--watch` to rebuild the site whenever a feature file or `ghg.toml` changes, and reload the pages open in the browser. While watching, the site is built in memory, and the output folder is left as it is; `ghg build` writes it.
- `ghg serve --no-build` serves the output folder as it is, such as a site you downloaded from a CI run.

Every setting has a default, so `ghg.toml` is optional: without it, ghg reads `features` and writes `site`, both in the current folder. `input`, `output`, `title` and `version_name` can also be given on the command line, as `--input`, `--output`, `--title` and `--version-name`, such as `ghg build --input specs --title "Our features"`, and the command line wins. The other settings are set only in `ghg.toml`. `ghg --help` lists the options, and [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "The command line and its configuration", describes every option and every key of `ghg.toml`, such as collapsing rules and scenarios, linking each page to its source, linking tags to an issue tracker, the accent colours and building versions.

## A site must be served

Open a site through a web server, never by opening its files straight from disk. A page opened from disk, with an address that starts with `file:`, still shows its content, but the browser refuses to run its script there, so the folder tree is empty, search does nothing, and the reader options do not work. The stylesheet does little on its own, so such a page looks nearly unstyled too. Every page opened from disk therefore first opens a dialog, "Opened from disk", that says the site is meant to be served and how to serve it; close it with its button, "I agree", or with Escape, to read the page.

To look at a site on your own machine, use `ghg serve`, or `ghg serve --no-build` for a site already built. To publish it, put the output folder on any static host, such as GitHub Pages; every link in a site is relative, so it works under any path. Why sites are served is explained in [design note 0026](docs/design/0026-sites-are-served.md).

## The output folder belongs to ghg

`ghg build` writes the site into its output folder, `site` unless you name another, and marks that folder as its own with a file at its root, `ghg-site.json`, or `ghg-versions.json` when it builds versions. From then on the output folder, and every folder in it, belongs to ghg: any build may empty it and write it again, so anything you put there by hand can be lost.

To keep that from happening by mistake:

- ghg marks an output folder only when it is missing or empty. It refuses to build into a folder that holds anything and no marker.
- Before every build, ghg checks that a marked folder holds only what it wrote there. When it holds anything else, ghg names it and stops, before reading the features, and writes and deletes nothing.

So keep the output folder for the site alone, and put anything you want served beside it somewhere else.

## Installing a release

Once the first version is released, it can be installed in three ways:

- **Downloaded.** Each release on GitHub holds an archive for each platform, with `ghg` and `cargo-ghg` in it.
- **Through cargo.** A Rust project can install it from [crates.io](https://crates.io/) with `cargo install gherkin-glass`, or download the release's binary with `cargo binstall gherkin-glass`. Either installs `ghg`, and `cargo-ghg`, so that `cargo ghg build` is `ghg build`, beside `cargo test`.
- **In a GitHub Actions workflow.** The repository is also an action that installs `ghg` and, with `build: true`, builds a site in the workspace. Every option of `ghg build` that sets how a site is built, `config`, `input`, `output`, `title` and `version-name`, is an input of the same name:

  ```yaml
  - uses: marlon-sousa/gherkin-glass@v0.1.0
    with:
      build: true
      output: site
  ```

  Before version 1.0, the action is named by a release's own tag, such as `@v0.1.0`. From 1.0 on, `@v1` follows the newest 1.x release. [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "The GitHub Action", describes every input.

## Working on gherkin-glass

How changes are proposed, agreed and reviewed is in [CONTRIBUTING.md](CONTRIBUTING.md), and how the code is organised is in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). This section is what you run.

### What you need

- **Rust**, through rustup 1.28 or later. Running `rustup toolchain install` in the repository installs the pinned version with the components `rust-toolchain.toml` names. The tests also run on the minimum supported Rust version, the one `rust-version` names in `Cargo.toml`.
- **cargo-deny 0.20.2**, for the dependency check: `cargo install cargo-deny --version 0.20.2 --locked`.
- **Node**, at the version `script/.node-version` pins, with npm, to run `cargo xtask build`, `check`, `check-script` or `check-browser`. Plain `cargo build` and `cargo test` need none.

### The checks

These are the checks a pull request runs, from the repository's root:

- `cargo fmt --all --check`, the formatting;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, the lints, with every warning an error;
- `cargo test --workspace --locked`, the tests;
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked`, the documentation, where a broken link fails;
- `cargo deny check`, the dependencies' licences, advisories and sources;
- `cargo publish --dry-run --locked --allow-dirty`, the crate built from its package alone, as crates.io would receive it;
- `cargo +<version> test --locked`, the tests again with the minimum supported Rust version, once `rustup toolchain install <version> --profile minimal` has installed it.

`cargo xtask check` runs all of them, then the script check and the browser check, as CI does.

### The xtask commands

`cargo xtask` runs the repository's own tasks, from the `xtask` crate. `cargo xtask --help` lists them.

| Command | What it does | Needs Node |
| --- | --- | --- |
| `cargo xtask build` | Installs the Node packages from `script/package-lock.json`, type-checks and tests the script, and compiles it into `assets/gherkin-glass.js`. Then it writes again the expected sites of the fixtures `pages`, `backgrounds`, `tags` and `highlighting`, and of the example suite, `project-site/expected`, and builds the release binary. | Yes |
| `cargo xtask check` | Runs every check above, then the script check and the browser check. | Yes |
| `cargo xtask check-script` | Type-checks and tests the script, and fails when `assets/gherkin-glass.js` is not what `script/` compiles to. | Yes |
| `cargo xtask check-browser` | Installs Playwright's headless Chromium, builds `ghg`, and checks the site of every fixture and of the example suite in it: axe-core in the light and the dark colour scheme, and the keyboard. It also checks that `ghg serve --watch` reloads a page. | Yes |
| `cargo xtask comment-ratio [path ...]` | Counts the comment lines and the code lines of the Rust and TypeScript files given, or of every one git tracks, with the total last. A pull request's description carries the total for the files it touches. | No |

Only `cargo xtask build` writes files to commit. The others write only what git ignores: the Node packages in `script/node_modules`, the script check's compiled copy in `target/xtask/`, `ghg` in `target/debug/`, and Playwright's Chromium in its cache in your home folder. `cargo xtask check` also installs the minimum supported Rust version with rustup, and `comment-ratio` only prints.

### Changing the pages

The pages are rendered from Askama templates in `src/policies/rendering/`, styled by `assets/gherkin-glass.css`, and driven by the TypeScript under `script/`. After changing any of them, run `cargo xtask build`, and commit what it writes with your sources: the compiled `assets/gherkin-glass.js` and the expected sites. A few fixtures keep a single expected page, such as `versions`, `step-sentences` and `no-steps`, which `cargo xtask build` does not write: their tests compare the page ghg builds with it, so a failing test shows which page to write again; [tests/step_sentences.rs](tests/step_sentences.rs) says how the step sentences page is built. A test compares every page ghg builds from a fixture with its expected site, byte for byte, so a change to any page shows in review. Then run `cargo xtask check-browser`, since a page that fails the accessibility check is a failing test.

## How it is built

gherkin-glass is written in Rust, with the [`gherkin`](https://crates.io/crates/gherkin) parser from cucumber-rs.

AI is part of how it is made. Agents write code and documentation, and a human maintainer reviews everything before it is merged, under the terms of [A manifesto for software engineering with AI](https://marlon-sousa.com/blog/manifesto/). How to contribute is in [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
