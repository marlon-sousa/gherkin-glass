# Contributing to gherkin-glass

Thank you for your interest. This guide describes how gherkin-glass is planned, built and released.

## How gherkin-glass is built

AI is an integral part of how gherkin-glass is made. Agents write code and documentation, review pull requests, and evaluate and manage issues. A human maintainer reviews everything before it is merged. The review asks whether the work is right: whether it does what [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md) says, whether it is safe to change, and whether the next person can read it. Work that is right is used, whoever or whatever produced it. Work that is not right is not merged.

These are the terms of [A manifesto for software engineering with AI](https://marlon-sousa.com/blog/manifesto/). By opening an issue or a pull request, you accept them: you may write your contribution with AI or without it, you have read what you submit and answer for it, and agents may read, evaluate and comment on your issues and pull requests, always marked as an agent's comment.

## Issues come first

Every change starts as an issue, and is agreed in the issue before any code is written. The expensive mistake is building the wrong thing, and the issue is the cheapest place to find that out.

- **A feature** says what is wanted and why. It can be short; the details are worked out in the issue.
- **A bug** says what happened, what was expected, and the smallest feature file that shows it.

Once an issue is agreed, every decision taken in its comments is written into its body, which then binds the work. The comments stay as the history of how each decision was reached, and nothing in them may contradict the body.

Issues carry one of these labels: `feature`, `bug`, `docs` or `process`.

Please do not open a pull request for a feature before its issue is agreed and its behaviour is written in [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md). Put implementation ideas in the issue instead, where they are welcome.

## The documents

- **[docs/BEHAVIOUR.md](docs/BEHAVIOUR.md)** is what the tool promises, in terms a test can observe: the command line, the configuration, the pages and their addresses, the anchor rule, the reader options and the accessibility guarantees. A feature is written here, in a pull request of its own, before it is built. When we say "the spec", we mean this document.
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** is how the code is organised and written: the module roles, the folder and visibility conventions, what a comment may say, and the test tiers.
- **[docs/design/](docs/design/)** holds short design notes, `NNNN-title.md`, written only when a decision is not obvious from the code, such as how a collapsed block stays reachable by heading. Each says what was decided, what else was considered and why.

## Fixtures

`tests/fixtures/` holds small feature files, each built into a site by the tests, which check the pages against docs/BEHAVIOUR.md: their headings, anchors, links, tables and reader options. Every rule of docs/BEHAVIOUR.md is checked by at least one fixture, and a change to a rule changes or adds the fixture that checks it.

## Pull requests

We work trunk-based: small pull requests straight to `main`, no long-lived branches.

- **Name the issue.** Every pull request names an open issue of this repository: `Closes #N` when it finishes the issue, `Refs #N` when it contributes to it. A check refuses a pull request that names none.
- **Stack large changes.** A change too large for one pull request becomes a stack: a chain of pull requests, each building on the one before, reviewed and merged in order. Each says `Refs #N`, and the last says `Closes #N`. See "Stacked pull requests" below.
- **Keep it focused.** One thing per pull request, described in plain prose: what changed and why.

### Stacked pull requests

A stack is a chain of pull requests, declared to GitHub as a stack so that it shows and merges as one (GitHub's stacked pull requests, the same ones the `gh stack` extension makes). Neither the `gh stack` extension nor anything else needs to be installed: the REST API does all of it.

1. **Chain the branches.** The bottom layer's base is `main`; each layer above has the branch of the layer below as its base, such as `gh api repos/OWNER/REPO/pulls -f base=<branch below> -f head=<branch>`.
2. **Declare the stack.** Once at least two layers exist, `POST /repos/OWNER/REPO/stacks` with the pull request numbers from bottom to top, `{"pull_requests": [12, 13]}`. A layer added later is appended with `POST /repos/OWNER/REPO/stacks/<stack number>/add`, giving only the new numbers, bottom to top. `GET /repos/OWNER/REPO/stacks?pull_request=<number>` finds the stack a pull request belongs to.
3. **Say where each layer sits.** Each description says its place, such as "2 of 3", and what the layers below it already do.
4. **Merge bottom up.** A layer is merged only after the layers below it.
5. **Fix forward.** A change to a lower layer is merged up into the layers above it, never by rewriting their history.

### Commit messages

Subjects follow [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): description`, with `!` before the colon for a breaking change. The types are `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert` and `test`.

A feature, a fix or a breaking change carries its changelog entries as trailers, which the changelog is built from:

```text
feat(pages): an anchor for every scenario

Added: every scenario has a heading with a stable anchor.
```

The trailers are `Added`, `Changed`, `Fixed` and `Removed`. A feature or fix that changes nothing a user can see says `Changelog: none` instead.

## Checks

Every pull request runs:

- the issue check and the commit message check, from [itinera-dev/actions](https://github.com/itinera-dev/actions);
- `cargo fmt --check`, `cargo clippy` with warnings as errors, and `cargo test`;
- the script check, `cargo xtask check-script`: the pages' script, in TypeScript under `script/`, is type-checked, tested and compiled again, and the check fails if the compiled script differs from the committed `assets/gherkin-glass.js`;
- the browser check, `cargo xtask check-browser`: each fixture is built into a site and served with `ghg serve --no-build`, and every page is opened in a browser, checked with axe-core in the light and the dark colour scheme, and driven with the keyboard alone, as [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "Accessibility", describes.

## Releases

Versions follow [Semantic Versioning](https://semver.org/). The addresses of pages, the anchor rule, the command line and the configuration keys are part of what is promised, so changing any of them in a way that breaks an existing site or command is a major version.

A maintainer releases by pushing a tag, such as `v0.1.0`. The release workflow then builds the binaries, for Linux (x86-64 and ARM64), macOS (Intel and Apple silicon) and Windows (x86-64), with their checksums, writes the release notes from the changelog trailers, and attaches everything to the GitHub release.

The repository's own GitHub Pages site shows the fixtures as the latest release renders them, so anyone can see what gherkin-glass produces before downloading it.

## Writing style

Everything we write must be readable with a screen reader: documentation, issues, pull requests and comments. Use headings, lists, prose and simple tables.

Diagrams are Mermaid diagrams, in fenced code blocks marked `mermaid`, which GitHub renders. Every diagram declares `accTitle` and `accDescr`, and is accompanied by text that says everything the diagram shows. Do not use ASCII-art diagrams, arrows drawn with characters, or box-drawing trees.

## Licence

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed under MIT and Apache-2.0, without any additional terms or conditions.
