# Architecture

How gherkin-glass's code is organised. [BEHAVIOUR.md](BEHAVIOUR.md) says what the tool does; the [design notes](design/) say why the larger choices were made; this document says how the code is laid out and written. Every contributor follows it. Most of the layout below does not exist yet; it is the shape the code is built towards, layer by layer.

It was agreed in [#23](https://github.com/marlon-sousa/gherkin-glass/issues/23), modelled on the architecture of [acter](https://github.com/marlon-sousa/acter/blob/main/docs/ARCHITECTURE.md) and adapted to a program that runs once and turns files into files. Changing a rule here starts as an issue, like any other change.

## The guiding principle

Ports and adapters. The pure part of gherkin-glass, which decides what the site contains, knows nothing of the disk. It defines the seams it needs from the world as traits, the ports, and the code that touches the world implements them, the adapters. The binary is the composition root: the one place that chooses the concrete adapters and hands them in. Dependencies point inward only.

## The repository

- **One crate**, `gherkin-glass`: a library, `src/lib.rs`, and the binary built on it, `src/main.rs`. Nothing in it is heavy enough to earn a crate of its own.
- **`xtask/`**, the crate behind `cargo xtask build` and `cargo xtask check` ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)). It orchestrates the build and is never shipped.
- **`templates/`**, the askama templates, at the place askama looks by default. They are the views: everything a page says is written here, except the folder tree, which the script builds from shared data ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)).
- **`script/`**, the TypeScript of the pages' behaviour, an npm package with its own `package.json` and lockfile. Its compiled output is committed, so building the crate needs no Node ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)).
- **`assets/`**, the files the binary embeds and writes into every site: the stylesheet, and the script compiled from `script/`.
- **`messages/`**, the Fluent catalogue of gherkin-glass's own interface text, one folder per language, embedded in the binary (see "Interface text").
- **`tests/`**, the crate's integration tests, and `tests/fixtures/`, the feature files every rule of BEHAVIOUR.md is checked against.

## Module roles

Every module plays exactly one of six roles, and says which on the first line of its `//!` doc comment, such as `//! Policy: the anchor rule.` Two files play none and say so instead: `lib.rs`, the facade, and `main.rs`, the composition root.

1. **Entity or value**: data and the invariants it keeps, such as a parsed feature, a page, or a problem found in the input.
2. **Policy**: a pure computation, the same output for the same input every time. Correcting doc strings, making anchors, building the site's model, the page addresses and the contrast check are policies, and so is rendering a template into a string.
3. **Port**: a trait the domain needs the world to implement, such as reading the input or writing the site.
4. **Adapter**: an implementation of a port that touches the world. Walking the input folder with its `.ghg-ignore` files, reading a file and writing a page are adapters.
5. **Service**: a named use case in the domain's own words, such as building a site. It coordinates policies, entities and ports, and touches the world only through the ports it was given, so a test can run it entirely in memory with fake ports.
6. **Controller**: the delivery layer. It turns the command line and the configuration into a call to a service, and the service's result into what the person at the terminal reads, including the exit status. Deleting a controller loses no behaviour, only the way in.

The question that classifies a module: **does it do anything nondeterministic, such as reading or writing files, reading the environment or the clock?** If yes, it is an adapter, behind a port. If no, it is one of the other five. If it still fits none, it does two things and is split until each part fits one. A pure function never gets a trait around it only to be faked: it is already testable.

Layering goes one way. Controllers call services; services use entities, policies and ports; adapters implement ports. Nothing reaches around a service into another module's internals.

### How a build maps onto the roles

The four steps of a build ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)) fall on the roles like this:

1. **Collect**: an adapter walks the input with the `.ghg-ignore` rules and returns the paths of the feature files, behind a port.
2. **Parse**: an adapter reads each file and parses it with the `gherkin` crate, behind a port, keeping the parser's own message for every file that fails. A policy then corrects the doc strings. The files are parsed in parallel with `rayon`; collecting its results into a `Vec` keeps them in file order, so the step stays deterministic.
3. **Build the model**: policies, in one place and in file order, build the folder tree, the anchors and the indexes.
4. **Render and write**: policies render each page from its template into a string, which may again run in parallel; an adapter writes the strings, and the embedded assets, behind a port.

The service that builds a site runs these steps; the controller behind the command line calls it; `main.rs` wires the real adapters in.

## Folders and files

1. **`lib.rs` is a facade**: only `mod` declarations and `pub use` re-exports, no logic. The crate's public interface fits on one screen.
2. **The visibility ladder**: private by default, `pub(crate)` when another module of the crate needs it, and bare `pub` only for what `lib.rs` re-exports. `#![warn(unreachable_pub)]` catches a `pub` that is not re-exported, and CI makes the warning an error; keeping the rest private is left to review.
3. **`module.rs` with a `module/` folder, never `mod.rs`**, so that every file's name says what it holds, which also reads better in a screen reader's list of editor tabs. One concept per file, named after it. A module earns a folder when it has private internals of its own.
4. **Folders are named by role**: `entities/`, `policies/`, `ports/`, `adapters/`, `services/`, `controllers/`. The path says the role before the file is opened, and the file's name says the concept. There are no folders named by concept: a concept that spans roles is several files, one in each role's folder. There is no `utils.rs` or `helpers.rs`.
5. **Unit tests live with the code**, in a `#[cfg(test)] mod tests` at the bottom of the file they test, where they can see private items without exporting them. Tests that build a whole site from a fixture live in `tests/`.
6. **Import a symbol, then use it by its bare name**: `use askama::Template;` then `#[derive(Template)]`, never `askama::Template` written inline. A type's own associated items, such as `Feature::parse_path`, and `crate::` or `super::` paths a macro needs, are the import-then-use pattern already and stay as they are.

## Comments

A comment says only what the code cannot:

- the module's role line;
- a measured fact, with what it was measured against, such as "Chromium, through Playwright, blocks an ES module on a `file://` page";
- an invariant or a hazard the code does not enforce itself;
- what an absent value, an error or a rejection carries.

One sentence, where it applies, once. Why a decision was taken, what else was considered and what came before belong in the issue and the pull request, which keep them, not in the code. A field, a function or a test whose name already says what it is gets no doc comment.

## Dependencies between parts

No dependency injection framework. Every collaborator is received through a constructor, as a trait object or a generic, and only `main.rs`, the composition root, names the concrete adapters. Nothing else constructs its own collaborators. Tests hand in fakes written for the purpose, such as an input held in memory and an output that records what was written.

## How the code runs

- **Synchronous throughout.** gherkin-glass reads, computes and writes, then exits; it needs no async runtime. `rayon` runs the steps that may be parallel, parsing and rendering ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)), and nothing else.
- **Errors** are `thiserror` types in the library and `anyhow` only in the binary. Every message a person reads must make sense spoken by a screen reader: a path, a line and a sentence, never a dump of a type ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)).
- **Page addresses always use `/`**, on every platform, including Windows. Turning a file's path into a page's address is a policy that takes the path's parts, not the platform's separator, so every platform's answer is tested on every platform.

## Interface text

gherkin-glass's own words, such as "Collapse all", "Skip to content", "From the background:", the tree's labels and every message on the command line, are in English only for now, but are written so that translating them later changes no code ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)).

- **The catalogue is in [Fluent](https://projectfluent.org/)**: one `.ftl` file per language, `messages/en/gherkin-glass.ftl` for English, the only language for now. A translation is a new file beside it, which translators can work on in Weblate, whose documentation lists Fluent as a supported format since its version 4.8.
- **It is loaded with [`i18n-embed`](https://crates.io/crates/i18n-embed)**, with its `fluent-system` feature, which embeds the files in the binary, **and read with the `fl!` macro of [`i18n-embed-fl`](https://crates.io/crates/i18n-embed-fl)**, which checks at compile time that every message used exists, with the arguments it names, so a missing message is a build error and not a broken page. Both are MIT; the same approach is used by the `age` encryption tool.
- **Every interface string comes from the catalogue, by its id.** Templates receive their text from Rust code that uses `fl!`, and none of the templates, the Rust code or the script writes interface text inline.
- **A message is a whole sentence with named placeholders**, such as `{ $count } scenarios` in Fluent's syntax, never pieces joined together, since word order differs between languages.
- **Counts are plural forms of one message**, chosen by the number, never a word with an `s` added, since languages have different plural rules.
- **The script receives its messages from the page**, written by the template, so that the script itself holds no words.
- **The parser's own messages are passed through as they are**, since they are the `gherkin` crate's, not ours.

The feature's own text is never interface text: it is shown as written in the file, in the file's language.

## The pages' script

The TypeScript in `script/` follows the same roles, each file declaring its role in a header comment. It uses no framework and no runtime dependencies.

- **The views are the templates.** Everything a page says, its headings, anchors, links and text, is in the HTML the templates write. The script adds behaviour to it, and never writes content, with one agreed exception: the folder tree, built from the site's shared tree data ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)).
- **Adapters** are the parts that touch the DOM or the browser's storage: the disclosure buttons, the tree, the reader options, the theme switch. Each sits behind a small interface in `ports/`.
- **Policies** are the pure parts: which state a block starts in, given the configured default and what the reader chose before; how a remembered choice is keyed by page and anchor.
- **`src/main.ts`** is the composition root, as `main.rs` is for the crate.
- **Sources live in `script/src/`, and tests in `script/test/`**, a tree that mirrors it, so a test of `src/adapters/tree.ts` is `test/adapters/tree.test.ts`.
- **What `script/` compiles to is one classic script**, not an ES module, so that a site opened from disk still runs it ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)). The folder tree's data is a second classic script, written for each site ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)).
- **A reader option that cannot act is disabled**, and says so to a screen reader, rather than acting and then refusing.

## Tests

Cheapest first:

1. **Policy tests**, pure and in memory: the anchor rule, doc string corrections, page addresses, contrast, the state a block starts in.
2. **Service tests** with fake ports: a whole build from an input held in memory, checking what would be written and every problem reported.
3. **Golden fixture tests**: each fixture under `tests/fixtures/` built into a site, and its pages compared with the expected HTML committed beside it, which shows the rules of BEHAVIOUR.md the fixture covers.
4. **The browser check** ([#2](https://github.com/marlon-sousa/gherkin-glass/issues/2)): every fixture page opened with Playwright, checked with axe-core in each theme, and driven with the keyboard.
5. **The script's own tests**, of its policies and adapters, in `script/test/`.

`cargo xtask check` runs all of them, and CI runs the same command.

## What a contributor needs

Plain `cargo build` and `cargo test` need only Rust: the toolchain `rust-toolchain.toml` pins, and the minimum version `rust-version` names for the second test run, as AGENTS.md describes. Node, at its pinned version, is needed for `cargo xtask build` and `cargo xtask check`, which install the Node packages from the lockfile, and the browser the check uses, themselves.
