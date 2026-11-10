# gherkin-glass

A static viewer for Gherkin `.feature` files that anyone can read, with or without a screen reader.

gherkin-glass turns a folder of feature files into a static website: a page for every feature, a heading and a stable link for every scenario, an index for every tag, and a page for every folder. The content is in the HTML of each page, and a stylesheet, a script and the data they share, written once with the site, style the pages and drive their interactive parts. The site is meant to be served: any static web server or host serves it, GitHub Pages included, and a planned `ghg serve` command ([#26](https://github.com/marlon-sousa/gherkin-glass/issues/26)) will preview it on your own machine. Opening its pages straight from disk is not supported. It ships as a single binary for Linux, macOS and Windows.

## Status

Planning. Nothing is released yet. What the first version will do is tracked in the [issues](https://github.com/marlon-sousa/gherkin-glass/issues), and what is agreed is written in [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md) before it is built.

## Goals

- **Readable by everyone.** Headings in order, real tables, lists for steps, and a skip link. Colour and highlighting help, but never carry meaning on their own. Accessibility is checked on every pull request.
- **Links that last.** A scenario's address comes from its file and its name, by a documented rule, so a link keeps working as long as both do.
- **Reader options.** Readers can choose to show a background's steps inside each scenario or once at the top, and to show or hide the folder tree. Rules and scenarios can be collapsed from their headings, and the browser remembers every choice.
- **Every Gherkin construct.** Rules, backgrounds, scenario outlines and their examples, data tables, doc strings, tags, descriptions, and every spoken language Gherkin supports.
- **One file to install.** A single binary, configured by a TOML file, that also runs as a GitHub Action.

## Installing through cargo

Once it is released, a Rust project can install gherkin-glass from [crates.io](https://crates.io/) with `cargo install gherkin-glass`. That installs two commands: `ghg`, and `cargo-ghg`, which lets cargo run it as `cargo ghg`, beside `cargo test`. Both do the same, so `cargo ghg build` is `ghg build`.

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
