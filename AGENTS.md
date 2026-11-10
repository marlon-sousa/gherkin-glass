# AGENTS.md: working in gherkin-glass

The manual for anyone, human or agent, changing gherkin-glass. [CONTRIBUTING.md](CONTRIBUTING.md) describes the process; this file says how to follow it.

## What to read

- [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md): what the tool promises. Code does what it says, no more and no less.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the code is organised and written, before writing any.
- [docs/design/](docs/design/): the design notes, for why things are as they are.
- The issue the work belongs to. Once it is agreed, its body holds every decision and binds the work; its comments are only the history of how they were reached. While it is still being agreed, read its comments too.

## Rules

1. **No code before its behaviour.** Work on a feature starts only once its issue is agreed and its behaviour is in docs/BEHAVIOUR.md. If the behaviour is missing, or the code would need to do something the document does not say, stop and raise it in the issue.
2. **Every rule has a fixture.** A rule added to or changed in docs/BEHAVIOUR.md comes with a fixture under `tests/fixtures/` and a test that checks it, in the same pull request as the code that makes it pass.
3. **Accessibility is behaviour.** A page that fails the accessibility check is a failing test, never a warning to skip. Colour and highlighting never carry meaning alone. JavaScript is assumed and drives the interactive parts, but the content is always in the page's HTML, never built by a script, so that headings, anchors and links exist as the page loads. The exceptions are the folder tree, which the script builds from shared data, since everything it links to is also reachable from the folder pages ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)), and what search finds, its status and its results ([#15](https://github.com/marlon-sousa/gherkin-glass/issues/15)).
4. **Addresses are promises.** Never change how a page's path or an anchor is made without a major version, agreed in an issue.
5. **Small pull requests.** One thing each, naming its issue with `Closes #N` or `Refs #N`, with commit messages as CONTRIBUTING.md describes. Work too large for one is a stack, declared to GitHub through the REST API as CONTRIBUTING.md, "Stacked pull requests", describes, without installing `gh stack`.
6. **Write for a screen reader.** Issues, pull requests, comments and documents use headings, lists, prose and simple tables, never ASCII art.
7. **Mark agent comments.** A comment an agent writes on GitHub says it is an agent's.

## Building and testing

The toolchain is pinned in `rust-toolchain.toml`. Running `rustup toolchain install` in the repository installs it with its components; this needs rustup 1.28 or later. Before pushing, run the checks a pull request runs, from the repository's root:

- `cargo fmt --check`, the formatting;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, the lints, with every warning an error;
- `cargo test --locked`, the tests;
- the tests again with the minimum supported Rust version, the one `rust-version` names in `Cargo.toml`, 1.88 today: first `rustup toolchain install 1.88 --profile minimal`, then `cargo +1.88 test --locked`.

`cargo run` builds the command and runs it.

The pages' script is TypeScript, in `script/`, compiled into `assets/gherkin-glass.js`, which is committed, so plain `cargo build` and `cargo test` need no Node. Changing the script needs Node, at the version `script/.node-version` pins: `cargo xtask build` installs the Node packages from the lockfile, type-checks, tests and compiles the script, writes the fixtures' expected sites again with it, as tests/fixtures/README.md describes, and builds the release binary; commit the compiled script and the expected sites with the sources. `cargo xtask check-browser` installs Playwright's headless Chromium and runs the browser check, `tests/browser/`: every fixture's site served and checked with axe-core in both colour schemes and with the keyboard, and the reload of `ghg serve --watch`. `cargo xtask check` runs every check above, the script check and the browser check, as CI does.

The commit messages can be checked before pushing with the `commit-messages` action of [itinera-dev/actions](https://github.com/itinera-dev/actions), which needs Python 3.11 or later. With a clone of it in the shell variable `ACTIONS`, run `PYTHONPATH="$ACTIONS" python3 "$ACTIONS/commit-messages/main.py" origin/main..HEAD` in this repository.
