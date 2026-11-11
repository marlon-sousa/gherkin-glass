# Architecture

How gherkin-glass's code is organised. [BEHAVIOUR.md](BEHAVIOUR.md) says what the tool does; the [design notes](design/) say why the larger choices were made; this document says how the code is laid out and written. Every contributor follows it. Most of the layout below does not exist yet; it is the shape the code is built towards, layer by layer.

It was agreed in [#23](https://github.com/marlon-sousa/gherkin-glass/issues/23), modelled on the architecture of [acter](https://github.com/marlon-sousa/acter/blob/main/docs/ARCHITECTURE.md) and adapted to a program that runs once and turns files into files. Changing a rule here starts as an issue, like any other change.

## The guiding principle

Ports and adapters. The pure part of gherkin-glass, which decides what the site contains, knows nothing of the disk. It defines the seams it needs from the world as traits, the ports, and the code that touches the world implements them, the adapters. The composition root is the one place that chooses the concrete adapters and hands them in, and the one door into the library. Dependencies point inward only.

## The repository

- **One crate**, `gherkin-glass`: a library, `src/lib.rs`, and the binaries built on it: `ghg`, from `src/main.rs`, and the cargo plugin `cargo-ghg`, from `src/cargo_ghg.rs`, which runs the same code as `cargo ghg` ([#18](https://github.com/marlon-sousa/gherkin-glass/issues/18)). The library's composition root, `src/composition_root.rs`, wires the real adapters in, and each binary only hands its entry point the process's arguments. Nothing in the crate is heavy enough to earn a crate of its own.
- **`xtask/`**, the crate behind `cargo xtask build` and `cargo xtask check` ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)). It orchestrates the build and is never shipped.
- **`templates/`**, the askama templates, at the place askama looks by default. They are the views: everything a page says is placed here, except the folder tree, which the script builds from shared data ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)).
- **`script/`**, the TypeScript of the pages' behaviour, an npm package with its own `package.json` and lockfile. Its compiled output is committed, so building the crate needs no Node ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)).
- **`assets/`**, the files the binary embeds and writes into every site: the stylesheet, and the script compiled from `script/`.
- **`messages/`**, the Fluent catalogue of gherkin-glass's own interface text, one folder per language, embedded in the binary, and **`i18n.toml`** at the crate's root, which tells the `fl!` macro where the catalogue is and which language it checks against (see "Interface text").
- **`action.yml`** and **`action/`**, the GitHub Action that installs `ghg` and builds a site with it ([#17](https://github.com/marlon-sousa/gherkin-glass/issues/17)): a composite action whose steps run the bash scripts in `action/`, each saying its role in a comment on its first lines, as a module does.
- **`project-site/`**, the project's own site ([#19](https://github.com/marlon-sousa/gherkin-glass/issues/19)): the example suite it shows, the list of fixtures it leaves out, and the scripts that gather its input and publish each release to the `gh-pages` branch, which `.github/workflows/site.yml` runs.
- **`.claude/skills/`**, the procedures an agent follows to review a change, write a commit, follow a stack and keep this document true. They add no rule of their own: each points at the rules it follows.
- **`tests/`**, the crate's integration tests, and `tests/fixtures/`, the feature files every rule of BEHAVIOUR.md is checked against.

## Module roles

Every module plays exactly one of six roles, and says which on the first line of its `//!` doc comment, such as `//! Policy: the anchor rule.` The other files play none and say so instead: `lib.rs`, the facade; `composition_root.rs`, the composition root, a module of the library; and the two binaries, `main.rs` and `cargo_ghg.rs`, which only call it.

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

The service that builds a site runs these steps; the controller behind the command line calls it; the composition root wires the real adapters in.

## Folders and files

1. **`lib.rs` is a facade with one door**: only `mod` declarations and a `pub use` of the two entry points, one for `ghg` and one for `cargo-ghg`, both in `composition_root.rs`, and no logic. gherkin-glass is a command, not a library: the library exists only so that both binaries share the code and the tests can reach it, so its crate documentation says it is not a stable API and that the way to use gherkin-glass is the `ghg` command. Adapters, ports, services, entities and policies are details, and none of them is exported. A real API, if one is ever wanted, starts as an issue that designs it.
2. **The visibility ladder**: private by default, `pub(crate)` when another module of the crate needs it, and bare `pub` only for the two entry points. `#![warn(unreachable_pub)]` catches a `pub` that is not re-exported, and CI makes the warning an error; a `pub use` in `lib.rs` of anything but the entry points is a review finding.
3. **`module.rs` with a `module/` folder, never `mod.rs`**, so that every file's name says what it holds, which also reads better in a screen reader's list of editor tabs. One concept per file, named after it. A module earns a folder when it has private internals of its own, or when rule 7 splits it.
4. **Folders are named by role**: `entities/`, `policies/`, `ports/`, `adapters/`, `services/`, `controllers/`. The path says the role before the file is opened, and the file's name says the concept. There are no folders named by concept: a concept that spans roles is several files, one in each role's folder. There is no `utils.rs` or `helpers.rs`.
5. **Unit tests live with the code**, in a `#[cfg(test)] mod tests` at the bottom of the file whose code decides what they check, where they can see private items without exporting them. A test module that grows long, past about 150 lines as a guide, is a file of its own: `#[cfg(test)] mod tests;` at the bottom of `module.rs`, with the tests in `module/tests.rs`. The tests in `tests/` reach the code only through the two entry points or by running a binary, as `tests/command_line.rs` does; a test that needs an internal type is a unit test.
6. **Import a symbol, then use it by its bare name**: `use askama::Template;` then `#[derive(Template)]`, never `askama::Template` written inline. A type's own associated items, such as `Feature::parse_path`, and `crate::` or `super::` paths a macro needs, are the import-then-use pattern already and stay as they are.
7. **A file holds one thing, and its length is a signal.** A file holding two things or more, at any length, keeps one and moves each other to a private submodule of its own, in the file's `module/` folder. A file past about 200 lines, counting its code and its documentation but not its test module, is the signal to look for the seams it splits along. The limit is a guide for the reviewer's judgement, not a gate, and no check counts lines: a file past it may stay when it holds one thing well, such as one whose lines all belong to one type, with its own `impl` blocks and the private helpers only it uses, or one a little past the limit that otherwise fits well, while a short file with several responsibilities splits.
8. **Every module is a file of its own.** Inline `mod` blocks are only for tests.
9. **A module with submodules decides its public face**: its submodules are private, and it re-exports what the rest of the crate uses, so that a path names one module level, such as `crate::policies::make_anchors`.
10. **Helpers that the tests of several modules share** live in a test-only module of their own, such as `#[cfg(test)] pub(crate) mod fixtures;`, never copied from one test module to another.

## Writing the code

- **Formatting is plain `rustfmt`**, with no configuration file.
- **Nothing panics while a build or a server runs.** Every failure is a value, a `Result` or a problem reported. Clippy denies `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `todo` and `unimplemented` outside tests, and `clippy.toml` allows them in tests. An exception that cannot fail, such as a regular expression written in the code, says so with `#[expect(clippy::expect_used, reason = "…")]`.
- **`#![forbid(unsafe_code)]`** is in the crate's root and in `xtask`'s.
- **Reuse before writing.** Look first at the standard library and the APIs already at hand, then at the crates already depended on, then at well-known crates. Write it ourselves only when none fits, or when a crate would replace only a couple of lines. A crate is judged by what it replaces across the code, not only by its first use.
- **Derive what a derive produces.** Error types derive with `thiserror`, with `#[from]` instead of a hand-written `From`. `Debug`, `Clone`, `Copy`, `Default`, `PartialEq`, `Eq`, `Hash` and the orderings derive from the standard library. `Display`, `From`, `Into`, `AsRef` and a plain `new` that only assigns every field derive with [`derive_more`](https://crates.io/crates/derive_more) whenever it produces the same code, and the dependency is added with the first impl it replaces. An impl is written by hand only when it needs logic.
- **Types hold every constraint they can.** An enum for a closed set of values, never a string or a `bool` standing for one; a newtype for an identifier or a value with rules of its own. A doc comment stating a constraint that the type could enforce means the type is wrong.
- **A struct with one field is a tuple struct** only while all of its code is derived. Once it has a method or a hand-written impl, its field has a name, so code reads `self.path`, never `self.0`. A newtype never derives `Deref`, which is for smart pointers.
- **A chain of iterator adapters reads as a sentence of names**, such as `.filter(is_feature_file).map(page_address)`, and the details live in the functions it names. A closure in a chain is one call, one method call or one field read, such as `|step| &step.value`; a closure that compares, negates, matches, computes or builds a value becomes a named function.
- **Iterating to get a result uses adapters, not a `for` loop.** A search is `find`, `find_map`, `position`, `any` or `all`; a check of every item that fails on the first bad one is `try_for_each`; building a value is `count`, `sum`, `fold` or `collect`. A `for` loop stays only when its body acts on each item, such as writing a file or asserting in a test.
- **Every line does something.** No line that changes nothing, such as a conversion to the type a value already has.
- **Platforms diverge by size.** A `#[cfg]` on one line or one function is fine; a whole module that would be gated is an adapter of its own, behind the port the domain already has, chosen in the composition root. An answer per platform that is a value, not a call to the world, is a policy taking the platform's name, such as `std::env::consts::OS`, so that every platform's answer is tested on every platform.

## Comments

A comment says only what the code cannot:

- the module's role line;
- a measured fact, with what it was measured against, such as "Chromium, through Playwright, fails a `fetch` from a `file://` page";
- an invariant or a hazard the code does not enforce itself;
- what an absent value, an error or a rejection carries.

One sentence, where it applies, once. Why a decision was taken, what else was considered and what came before belong in the issue and the pull request, which keep them, not in the code. A field, a function or a test whose name already says what it is gets no doc comment.

- A measured fact names the program and its version, never a date.
- A fact appears once in the repository; a second place that needs it says "see" with a path.
- No `TODO` comments, and no bold.
- Code never cites an issue, a pull request or a document, with one exception: a test cites the rule of docs/BEHAVIOUR.md it checks by its number, such as `// Rule 110: the parts are the step's text.`, so that each rule leads to its test.

## Dependencies between parts

No dependency injection framework. Every collaborator is received through a constructor, as a trait object or a generic, and only the composition root names the concrete adapters. Nothing else constructs its own collaborators. Tests hand in fakes written for the purpose, such as an input held in memory and an output that records what was written.

- **A controller holds each service behind a trait**, a driving port, named after the service with `Api`, such as `SiteApi` implemented by `SiteService`, so that the controller's tests hand in fake services. Only the composition root names the concrete service.
- **A service earns its place**: it coordinates at least two of a port, an entity and a policy for a named use case. A single pass-through gets no service, and its caller calls the target directly.
- A pure function never gets a trait around it only to be faked.

## How the code runs

- **Synchronous throughout.** gherkin-glass reads, computes and writes, then exits; it needs no async runtime. `rayon` runs the steps that may be parallel, parsing and rendering ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)), and nothing else.
- **Errors** are `thiserror` types in the library, and `anyhow`, if it is ever needed, only in the composition root. An error carries data, such as a path, a line and its kind, and the controller turns it into words from the catalogue (see "Interface text"); its `#[error]` text and any `anyhow` context are for developers, and never reach the person at the terminal. Every message a person reads must make sense spoken by a screen reader: a path, a line and a sentence, never a dump of a type ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)).
- **Page addresses always use `/`**, on every platform, including Windows. Turning a file's path into a page's address is a policy that takes the path's parts, not the platform's separator, so every platform's answer is tested on every platform.

## Interface text

gherkin-glass's own words, such as "Collapse all", "Skip to content", "From the background:", the tree's labels and every message on the command line, are in English only for now, but are written so that translating them later changes no code ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4), [#23](https://github.com/marlon-sousa/gherkin-glass/issues/23)).

- **The catalogue is in [Fluent](https://projectfluent.org/)**: one `.ftl` file per language, `messages/en/gherkin-glass.ftl` for English, the only language for now. A translation is a new file beside it, which translators can work on in Weblate, whose documentation lists Fluent as a supported format since its version 4.8.
- **It is embedded with [`rust-embed`](https://crates.io/crates/rust-embed)**, a struct deriving `RustEmbed` over `messages/`, **loaded with [`i18n-embed`](https://crates.io/crates/i18n-embed)**, with its `fluent-system` feature, **and read with the `fl!` macro of [`i18n-embed-fl`](https://crates.io/crates/i18n-embed-fl)**. All three are MIT; the Rust `age` crate (rage) is built the same way.
- **What `fl!` checks at compile time**: that every message used exists in the English catalogue, the fallback language `i18n.toml` names, with exactly the arguments it takes, so a missing message is a build error and not a broken page. It checks only English, not the translations, and only arguments passed by name, so arguments are always passed by name, never as a map.
- **Fluent's isolation marks are turned off**, with `set_use_isolating(false)`. By default Fluent wraps every placeholder in the invisible characters U+2068 and U+2069, which would end up in the pages, the accessible names, the golden files and the terminal. Feature text placed inside a message is isolated by the HTML instead, with `<bdi>`, wherever it is in the markup.
- **Every interface string comes from the catalogue, by its id.** Templates receive their text from Rust code that uses `fl!`, and none of the templates, the Rust code or the script writes interface text inline.
- **A message is a whole sentence with named placeholders**, such as `feature-page-title = { $feature }, a feature in { $folder }` in Fluent's syntax, never pieces joined together, since word order differs between languages.
- **Counts are plural forms of one message**, chosen by the number with a Fluent selector, such as `scenario-count = { $count -> [one] one scenario *[other] { $count } scenarios }`, never a word with an `s` added, since languages have different plural rules. A count only the script knows, such as the number of search results, cannot be chosen by Fluent, since the page is written before it is known: it is written as a label and a number, `Results: { $count }.`, which needs no plural form.
- **The script receives its messages from the page**, written by the template, so that the script itself holds no words.
- **The parser's own messages are passed through as they are**, since they are the `gherkin` crate's, not ours.

The feature's own text is never interface text: it is shown as written in the file, in the file's language.

## The pages' script

The TypeScript in `script/` follows the same roles, each file declaring its role in a header comment. It uses no framework, and one runtime dependency only: [MiniSearch](https://www.npmjs.com/package/minisearch), bundled into the script, which runs search ([#15](https://github.com/marlon-sousa/gherkin-glass/issues/15)).

- **The views are the templates.** Everything a page says, its headings, anchors, links and text, is in the HTML the templates write. The script adds behaviour to it, and never writes content, with two agreed exceptions: the folder tree, built from the site's shared tree data ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)), and what search finds, its status and its results ([#15](https://github.com/marlon-sousa/gherkin-glass/issues/15)).
- **Adapters** are the parts that touch the DOM, the browser's storage, the network or the clock: the disclosure buttons, the tree, the reader options, the theme switch, the search box, fetching the search documents, and waiting for the reader to pause. Each sits behind a small interface in `ports/`.
- **Policies** are the pure parts: which state a block starts in, given the configured default and what the reader chose before; how a remembered choice is keyed by page and anchor.
- **`src/main.ts`** is the composition root, as `composition_root.rs` is for the crate.
- **Sources live in `script/src/`, and tests in `script/test/`**, a tree that mirrors it, so a test of `src/adapters/tree.ts` is `test/adapters/tree.test.ts`.
- **What `script/` compiles to is one ES module.** Sites are served over HTTP, never opened from disk ([#26](https://github.com/marlon-sousa/gherkin-glass/issues/26), [design note 0026](design/0026-sites-are-served.md)), so the script may be a module and may fetch. The data the pages share, such as the folder tree's ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)), is JSON written for each site and fetched by the script, at an address resolved against the script's own, such as `new URL("tree.json", import.meta.url)`, so that it works from a page at any depth and on a site served below a path such as GitHub Pages' `/<repository>/`.
- **The reload script `ghg serve --watch` adds to each page is not part of any site.** It is a few lines of plain JavaScript beside the server's adapter, `src/adapters/preview_http/reload.js`, embedded with `include_str!`, so that it needs no build step.
- **One line of script is not in `script/`**: the classic script in every page's head that applies the remembered theme before the page is drawn, which a module, run only once the page is read, cannot do ([#10](https://github.com/marlon-sousa/gherkin-glass/issues/10), [design note 0010](design/0010-themes.md)). It reads the key the theme policy keeps the choice under, and a test of the script checks that the two agree.
- **A reader option that cannot act is disabled**, and says so to a screen reader, rather than acting and then refusing.

## Tests

Cheapest first:

1. **Policy tests**, pure and in memory: the anchor rule, doc string corrections, page addresses, contrast, the state a block starts in.
2. **Service tests** with fake ports: a whole build from an input held in memory, checking what would be written and every problem reported.
3. **Golden fixture tests**: each fixture under `tests/fixtures/` built into a site, and its pages compared with the expected HTML committed beside it, which shows the rules of BEHAVIOUR.md the fixture covers.
4. **The browser check** ([#2](https://github.com/marlon-sousa/gherkin-glass/issues/2)): every fixture's site served with `ghg serve --no-build` ([#26](https://github.com/marlon-sousa/gherkin-glass/issues/26)), as a reader would get it, and each page opened with Playwright, checked with axe-core in each theme, and driven with the keyboard.
5. **The script's own tests**, of its policies and adapters, in `script/test/`.

`cargo xtask check` runs all of them, and CI runs the same command.

How a Rust test is written:

- **A test's name states the rule it checks**, as a sentence, such as `a_feature_in_portuguese_keeps_its_keywords`.
- **A setup that three or more tests of one module start the same way** becomes a helper of those tests, named for what it makes; when the tests go on differently, the helper makes only the part they share.
- **Policies over arbitrary text have property tests**, with [`proptest`](https://crates.io/crates/proptest), such as the anchor rule and step highlighting never panicking on any input.
- **A rule checked on several inputs is one parametrised test**, written with [`rstest`](https://crates.io/crates/rstest), with a named case per input, such as `#[case::a_rule_s_background("rule-background.feature")]`, never a run of asserts or a loop over a table. Each case's name says what its input is. A loop over every value of a closed set, such as every step kind, stays a loop, so that a new value is checked without a new case.

## What a contributor needs

Plain `cargo build` and `cargo test` need only Rust: the toolchain `rust-toolchain.toml` pins, and the minimum version `rust-version` names for the second test run, as AGENTS.md describes. Node, at its pinned version, is needed for `cargo xtask build` and `cargo xtask check`, which install the Node packages from the lockfile, and the browser the check uses, themselves.
