# AGENTS.md: working in gherkin-glass

The manual for anyone, human or agent, changing gherkin-glass. [CONTRIBUTING.md](CONTRIBUTING.md) describes the process; this file says how to follow it.

## What to read

- [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md): what the tool promises. Code does what it says, no more and no less.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): how the code is organised and written, before writing any.
- [docs/design/](docs/design/): the design notes, for why things are as they are.
- [.claude/skills/](.claude/skills/): the procedures for reviewing a change, writing a commit, following a stack and keeping docs/ARCHITECTURE.md true. Claude Code loads them on its own; another agent reads the one it needs.
- The issue the work belongs to. Once it is agreed, its body holds every decision and binds the work; its comments are only the history of how they were reached. While it is still being agreed, read its comments too.

## Rules

1. **No code before its behaviour.** Work on a feature starts only once its issue is agreed and its behaviour is in docs/BEHAVIOUR.md. If the behaviour is missing, or the code would need to do something the document does not say, stop and raise it in the issue.
2. **Every rule has a fixture.** A rule added to or changed in docs/BEHAVIOUR.md comes with a fixture under `tests/fixtures/` and a test that checks it, in the same pull request as the code that makes it pass.
3. **Accessibility is behaviour.** A page that fails the accessibility check is a failing test, never a warning to skip. Colour and highlighting never carry meaning alone. JavaScript is assumed and drives the interactive parts, but the content is always in the page's HTML, never built by a script, so that headings, anchors and links exist as the page loads. The exceptions are the folder tree, which the script builds from shared data, since everything it links to is also reachable from the folder pages ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)), and what search finds, its status and its results ([#15](https://github.com/marlon-sousa/gherkin-glass/issues/15)).
4. **Addresses are promises.** Never change how a page's path or an anchor is made without a major version, agreed in an issue.
5. **Small pull requests.** One thing each, naming its issue with `Closes #N` or `Refs #N`, with commit messages as CONTRIBUTING.md describes. Work too large for one is a stack, declared to GitHub through the REST API as CONTRIBUTING.md, "Stacked pull requests", describes, without installing `gh stack`.
6. **Write for a screen reader.** Issues, pull requests, comments and documents use headings, lists, prose and simple tables, never ASCII art.
7. **Mark agent comments.** A comment an agent writes on GitHub says it is an agent's.
8. **Check a library before ruling it in or out.** Before saying that a crate or a package can or cannot do something, read its source or its documentation at the version we would use. Look for its override and extension points, not only its defaults. The issue or pull request says what was checked: the file and line, or the documentation page.
9. **The plain crate before its internals.** A crate's public face, such as `regex`, comes before the crates it is built from, such as `regex-automata` and `regex-syntax`. The internals are used only where the plain crate cannot do what is needed, and the pull request says why, naming the file and line of the plain crate's source, as rule 8 asks.
10. **One question at a time.** A question for the maintainer is asked in plain prose, one decision at a time, with the option recommended and why. The answer is recorded on its issue, as CONTRIBUTING.md, "Issues come first", describes, before the next question is asked.

## Reviewing a change

Every pull request is reviewed by an independent reviewer before it is opened, one given the change, not its author's reasoning. A later push to an open pull request is made once the checks under "Building and testing" pass, and is reviewed while its CI runs; an accepted finding becomes a further push, reviewed the same way. An agent's `.claude/skills/review-change` says how to start and resume the reviewer. The reviewer is given this section in full, the documents under "What to read", the issue, the commits to review and the draft pull request description. For each item, it lists every place in the diff where the item applies, not only the first, and checks each against the rule it names.

1. Every behaviour the change adds, against docs/BEHAVIOUR.md, and every rule added or changed there, against rule 2 above.
2. Every page the change touches, against rule 3 above, and every page path and anchor, against rule 4 above; and that the description carries the screen-reader check when a page's text or behaviour changes.
3. `lib.rs`, against docs/ARCHITECTURE.md, "Folders and files", rules 1 and 2: it exports the two entry points and nothing else. Every new bare `pub` other than the two entry points is a finding, against rule 2.
4. Every file the change adds or makes longer, against docs/ARCHITECTURE.md, "Folders and files", rules 5 and 7: what it holds, its lines before its test module and the lines of its test module. The limits are guides for judgement, not gates. A file with several responsibilities splits at any length. Past a limit, the reviewer says what the file holds and whether it still holds one thing well, and if not, proposes the private submodules it splits into.
5. Every new module: its role line, its folder and its name, against docs/ARCHITECTURE.md, "Module roles" and "Folders and files".
6. Every test the change adds or moves: a test in `tests/` reaches the code only through the entry points or a binary, and a unit test is in the module whose code decides what it checks, against docs/ARCHITECTURE.md, "Folders and files", rule 5; and every point of docs/ARCHITECTURE.md, "Tests", "How a Rust test is written".
7. Every comment and doc comment, against docs/ARCHITECTURE.md, "Comments".
8. Against docs/ARCHITECTURE.md, "Writing the code": every `unwrap`, `expect`, `panic!`, indexing, `todo!` and `unimplemented!` outside tests; every hand-written trait impl and `new`, naming the derive that would produce it or why none does; every helper, naming what already provides it, if anything; every constraint stated in a comment that a type could hold; every struct with one field; every closure in a chain of adapters and every `for` loop; every `#[cfg]`; and every line that changes nothing.
9. Every handler, the controller, every service and every adapter, against docs/ARCHITECTURE.md, "Module roles", its layering, and "Dependencies between parts": an adapter that uses a policy, a service, the controller or a handler, and a call that goes up a layer, are findings. Every type of a wrapped crate that reaches a port, an entity, a policy or a service, and every type of the `gherkin` crate in the site tree, is a finding too.
10. Every statement of docs/ARCHITECTURE.md the change makes untrue, which the change updates itself.
11. The commit messages and the description, against CONTRIBUTING.md and rule 5 above. For a layer of a stack, its branch sits on the current tip of the layer below and holds no merge of a lower branch, as CONTRIBUTING.md, "Stacked pull requests", rule 5, says.
12. Every claim that a library can or cannot do something, against rule 8 above: it names the source it was checked against.
13. Every module the change adds or moves, against docs/ARCHITECTURE.md, "Folders and files", rules 4 and 9: it is in its role's folder, in the folder of the stage, and of the need within it, that it serves when its role has areas, and the area's module says what the area covers and re-exports its face. A need of `policies/` whose face is more than its one entry point, or a stage that re-exports more than its needs' entry points, is a finding.
14. Every error the change adds, moves or raises, against docs/ARCHITECTURE.md, "How the code runs": it is in `errors/`, named with `Error`, `Errors` or `ErrorKind`; its `Display` is the sentence a person reads, unless it is a part worded by the error that holds it, from functions in its own `impl` or in `errors.rs` that call `fl!`, with no `#[error]` text for developers only; its context is put in with `map_err`; `#[from]` is only for an error that carries nothing more; and a cause kept whole stays reachable as `#[source]`.
15. Every crate the change adds or uses in a new way, against rule 9 above: a crate's internals only where the plain crate cannot do the job, with the source that shows it.
16. The checks under "Building and testing", run at the reviewed revision, without changing the checkout the author works in.
17. Every kind of page or node the change adds or moves, against docs/ARCHITECTURE.md, "The pipeline": the node renders through the rendering trait beside its template, Askama, serde for the pages' data and the theme's stylesheet are used only in `rendering`, and no service, controller or handler names a kind of page.
18. Every change that says it changes nothing a person meets, such as a `refactor`: no committed expected site changes, a fixture's or the example suite's, unless the maintainer agreed the change in the issue.
19. Every change that can alter what a page looks like or how it lays out, as the reviewer judges it: the reviewer builds the fixtures' sites and the example suite's, serves them, and looks at the pages the change touches, and at least one feature page, one folder page and the front page, in Chromium through Playwright, at 1366 by 850 and 390 by 844 pixels, in the light and the dark theme, with screenshots, and once more after resizing the window from wide to narrow without a reload. It reports, with the page, the width and the theme, anything out of line with the rest of the page, overlapping, cut off, crowded or touching an edge, any text that wraps badly, and anything that does not follow the resize. A change that cannot reach a page needs no visual check, and the report says so.

A pull request that changes what a page says or how it behaves carries a manual check with a screen reader in its description, as checkboxes, one per check, with the findings written on the unticked items, such as the screen reader's version and what was expected and heard.

The report lists findings, each marked blocking or not, with its `file:line` and a fix, then a verdict: approve, or changes needed. For item 19, it says whether the reviewer judged that the change can alter what a page looks like or how it lays out, and why. A finding the author rejects goes back to the reviewer with the reason, and a blocking one closes only when the reviewer agrees or the maintainer decides. A pull request is reported ready to the maintainer only once both the review and the CI of its latest push have passed.

## Building and testing

The toolchain is pinned in `rust-toolchain.toml`. Running `rustup toolchain install` in the repository installs it with its components; this needs rustup 1.28 or later. Before pushing, run the checks a pull request runs, from the repository's root:

- `cargo fmt --all --check`, the formatting;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`, the lints, with every warning an error;
- `cargo test --workspace --locked`, the tests;
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items --locked`, the documentation, where a broken link fails;
- `cargo deny check`, the dependencies' licences, advisories and sources, as `deny.toml` allows them, with cargo-deny 0.20.2, which installs with `cargo install cargo-deny --version 0.20.2 --locked`;
- `cargo publish --dry-run --locked --allow-dirty`, the crate built from its package alone, as crates.io would receive it, with `--allow-dirty` so that it runs before committing too;
- the tests again with the minimum supported Rust version, the one `rust-version` names in `Cargo.toml`, 1.88 today: first `rustup toolchain install 1.88 --profile minimal`, then `cargo +1.88 test --locked`.

`cargo run` builds the command and runs it.

The pages' script is TypeScript, in `script/`, compiled into `assets/gherkin-glass.js`, which is committed, so plain `cargo build` and `cargo test` need no Node. Changing the script needs Node, at the version `script/.node-version` pins: `cargo xtask build` installs the Node packages from the lockfile, type-checks, tests and compiles the script, writes the fixtures' expected sites again with it, as tests/fixtures/README.md describes, and the example suite's, `project-site/expected`, with `project-site/ghg.toml`, and builds the release binary; commit the compiled script and the expected sites with the sources. `cargo xtask check-browser` installs Playwright's headless Chromium and runs the browser check, `tests/browser/`: every fixture's site and the example suite's served and checked with axe-core in both colour schemes and with the keyboard, pages opened from disk, whose dialog must open, and the reload of `ghg serve --watch`. `cargo xtask check` runs every check above, the script check and the browser check, as CI does, and a test of xtask keeps its list the same as `.github/workflows/rust.yml`'s.

`cargo xtask comment-ratio` counts the comment lines for each line of code in the Rust and TypeScript files given, or in every one git tracks. A pull request's description carries the total line it prints for the files the change touches, so that a reviewer sees comments growing.

The commit messages can be checked before pushing with the `commit-messages` action of [itinera-dev/actions](https://github.com/itinera-dev/actions), which needs Python 3.11 or later. With a clone of it in the shell variable `ACTIONS`, run `PYTHONPATH="$ACTIONS" python3 "$ACTIONS/commit-messages/main.py" origin/main..HEAD` in this repository.
