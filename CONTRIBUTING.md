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
- **Check pages with a screen reader.** A pull request that changes what a page says or how it behaves carries a manual check with a screen reader in its description, as AGENTS.md, "Reviewing a change", describes.
- **Merge without squashing.** Pull requests merge with a merge commit or a rebase, never squashed, because a squash folds every commit's message into one, and the release notes are built from each commit's changelog trailers.

### Stacked pull requests

A stack is a chain of pull requests, declared to GitHub as a stack so that it shows and merges as one (GitHub's stacked pull requests, the same ones the `gh stack` extension makes). Neither the `gh stack` extension nor anything else needs to be installed: the REST API does all of it.

1. **Chain the branches.** The bottom layer's base is `main`; each layer above has the branch of the layer below as its base, such as `gh api repos/OWNER/REPO/pulls -f base=<branch below> -f head=<branch>`.
2. **Declare the stack.** Once at least two layers exist, `POST /repos/OWNER/REPO/stacks` with the pull request numbers from bottom to top, `{"pull_requests": [12, 13]}`. A layer added later is appended with `POST /repos/OWNER/REPO/stacks/<stack number>/add`, giving only the new numbers, bottom to top. `GET /repos/OWNER/REPO/stacks?pull_request=<number>` finds the stack a pull request belongs to.
3. **Say where each layer sits.** Each description says its place, such as "2 of 3", and what the layers below it already do.
4. **Merge bottom up.** A layer is merged only after the layers below it.
5. **Keep the stack linear.** Every layer sits on the current tip of the layer below. A fix to a lower layer is a new commit on that layer's branch only; a lower branch is never merged into an upper one, because GitHub's restack drops merge commits, and the conflicts they settled come back. When a layer merges, GitHub retargets the layer above to `main` and rebases the layers above it. When it cannot, each layer above is rebased onto the new tip of the one below, bottom first, and pushed with `git push --force-with-lease`. This rebase, and rewording a pushed commit message that the commit message check refuses, followed by the same rebase of the layers above, are the only rewrites of history allowed, and only of one's own layers.

### Commit messages

Subjects follow [Conventional Commits](https://www.conventionalcommits.org/): `type(scope): description`, with `!` before the colon for a breaking change. The types are `build`, `chore`, `ci`, `docs`, `feat`, `fix`, `perf`, `refactor`, `revert` and `test`.

A feature, a fix or a breaking change carries its changelog entries as trailers, which the changelog is built from:

```text
feat(pages): an anchor for every scenario

Added: every scenario has a heading with a stable anchor.
```

The trailers are `Added`, `Changed`, `Fixed` and `Removed`. A feature or fix that changes nothing a user can see says `Changelog: none` instead.

Each entry is one line. [git-cliff](https://git-cliff.org/) gathers the entries of every commit since the last release, with the template in `cliff.toml`, and the commit message check refuses an entry git-cliff would not read whole.

## Checks

Every pull request runs:

- the issue check, the check that no closing keyword names another repository's issue, and the commit message check, from [itinera-dev/actions](https://github.com/itinera-dev/actions);
- a check that fails while the description has an unticked checkbox, so that a manual check, such as the one with a screen reader, is finished before merging;
- the formatting, the lints with warnings as errors, and the tests, on the pinned toolchain, and the tests again on the minimum supported Rust version;
- the documentation with warnings as errors, the dependencies with `cargo deny`, and the package with a publish dry run;
- the script check, `cargo xtask check-script`: the pages' script, in TypeScript under `script/`, is type-checked, tested and compiled again, and the check fails if the compiled script differs from the committed `assets/gherkin-glass.js`;
- the browser check, `cargo xtask check-browser`: each fixture is built into a site and served with `ghg serve --no-build`, and every page is opened in a browser, checked with axe-core in the light and the dark colour scheme, and driven with the keyboard alone, as [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "Accessibility", describes.

AGENTS.md, "Building and testing", gives the command of each, and `cargo xtask check` runs them all.

Every action a workflow uses is pinned to a commit, with its version in a comment beside it. Each week, `.github/workflows/weekly.yml` runs the tests on Rust beta, so that a change in the next Rust that breaks gherkin-glass is known before it is released.

## Releases

Versions follow [Semantic Versioning](https://semver.org/). The addresses of pages, the anchor rule, the command line, the configuration keys and the names of the release archives are part of what is promised, so changing any of them in a way that breaks an existing site, command or installation is a major version. [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "Releases", says what a release holds.

### Preparing a release

A release starts as a pull request, titled "Release <version>", whose commit `chore(release): release <version>`:

1. raises the version in `Cargo.toml`, and in `Cargo.lock` with `cargo update --workspace`;
2. writes the version's section at the top of `CHANGELOG.md`, starting from what `git cliff --unreleased --tag v<version> --strip all` prints, which is the heading `## [<version>]` and the entries of the commits since the last release;
3. reconciles those entries, since a commit's entries cannot change once it merges: entries a later commit made untrue become one that says what the release holds, entries that say the same thing merge, and a change a user meets that carries no entry gets one.

The template is written for git-cliff 2.14.2, which installs with `cargo install git-cliff --version 2.14.2 --locked` or `pip install git-cliff==2.14.2`.

### Tagging it

Once the pull request has merged, a maintainer runs `cargo publish --dry-run --locked` at the commit the merge put on `main`, then tags that commit and pushes the tag, such as `git tag -a v0.1.0 <commit> -m "gherkin-glass 0.1.0"` and `git push origin v0.1.0`. Only maintainers push tags.

The release workflow, `.github/workflows/release.yml`, then runs these jobs. Publishing to crates.io waits for all the others before it, and the GitHub release waits for crates.io:

1. **release gate** refuses the tag unless it is `v` and the version in `Cargo.toml`, its commit is on `main`, and `CHANGELOG.md` has notes for the version;
2. **checks** runs every check of `.github/workflows/rust.yml`, the formatting, the lints, the tests, the script's checks and the browser check, again on the tagged commit, since their run on `main` after a merge blocks nothing;
3. **release tools** tries the release gate, the changelog template and the archive check against their fixtures;
4. **builds**, one for each platform: Linux on x86-64 and ARM64, statically linked with musl; macOS on Apple silicon and on Intel, both built on Apple silicon; and Windows on x86-64, with the C runtime linked in. Each checks its binaries need nothing installed, packs them with the licence files, and checks the archive;
5. **archives** checks there is one archive for each platform where `cargo binstall` looks for it, and writes `SHA256SUMS` and checks the archives against it;
6. **crates.io**, in the `crates-io` environment, publishes the crate with the `publish-crates` action of [itinera-dev/actions](https://github.com/itinera-dev/actions), through crates.io's trusted publishing, unless that version is there already;
7. **GitHub release** creates the release with the `create-release` action of itinera-dev/actions, with the version's section of `CHANGELOG.md` as its notes, a pre-release for a version such as `0.2.0-rc.1`, and the archives and `SHA256SUMS` attached.

A pull request that changes how releases are made runs the same builds, archives and checks, and packs the crate, without publishing anything.

### Trusted publishing

crates.io trusts only `release.yml` in this repository, running in the `crates-io` environment, so no token is stored anywhere. Since crates.io checks only the workflow's file name and the environment, and a pull request runs its own copy of the workflow, the `crates-io` environment must accept deployments only from tags `v*`: a maintainer sets this once, in the repository's settings, under Environments, as a deployment tag rule. crates.io sets up trusted publishing only for a crate that already exists, so the first version may have to be published by hand: a maintainer logged in with `cargo login` runs `cargo publish --locked` at the commit to be tagged, before pushing its tag, then sets up the trusted publisher. The workflow's publish job then finds that version on crates.io and leaves it as it is. If a release stops after publishing, a maintainer re-runs the workflow's failed jobs.

The repository's own GitHub Pages site shows an example suite and the fixtures as each release renders them, one folder per version, so anyone can see what gherkin-glass produces before downloading it. Each release adds its own version's folder to the `gh-pages` branch and leaves the earlier ones as they were ([docs/BEHAVIOUR.md](docs/BEHAVIOUR.md), "The project's own site"). Pages is set once, under the repository's Settings, Pages: source "Deploy from a branch", branch `gh-pages`, folder root.

## Writing style

Everything we write must be readable with a screen reader: documentation, issues, pull requests and comments. Use headings, lists, prose and simple tables.

Diagrams are Mermaid diagrams, in fenced code blocks marked `mermaid`, which GitHub renders. Every diagram declares `accTitle` and `accDescr`, and is accompanied by text that says everything the diagram shows. Do not use ASCII-art diagrams, arrows drawn with characters, or box-drawing trees.

## Licence

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed under MIT and Apache-2.0, without any additional terms or conditions.
