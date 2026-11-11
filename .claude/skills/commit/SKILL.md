---
name: commit
description: How Claude writes a commit message in gherkin-glass, choosing its type, scope and changelog entries, and what to do when a pushed message is wrong. Use before every commit.
---

# Writing a commit

[CONTRIBUTING.md](../../../CONTRIBUTING.md), "Commit messages", sets the rules. This skill adds how Claude decides each part.

## The subject

1. Choose the type by what the commit does, not by the files it touches:
   - `feat` adds or changes what a person using `ghg` meets: the command line, the configuration, the pages;
   - `fix` puts right what does not do what docs/BEHAVIOUR.md says;
   - `refactor` changes code without changing what a person meets, `perf` makes it faster, `test` changes only tests and their fixtures, `docs` only documentation, `ci` the workflows, `build` the manifests, the toolchain or `deny.toml`, and `chore` anything else, such as a release commit;
   - `revert` undoes an earlier commit, naming it in the body.
2. Choose the scope as the area of the change, such as `pages`, `search`, `cli` or `release`. Leave it out when the change spans several.
3. Add `!` only when something docs/BEHAVIOUR.md promised in the last release breaks. `git show v<last version>:docs/BEHAVIOUR.md` settles what the last release promised. A change to something added since the last release breaks nothing released.
4. Write the description in lower case, saying what the commit makes true, as the existing subjects do.

## The entries

1. Compare what a person meets after the commit with the last release, not with the commit before. Each difference is one entry, in the section that names it: `Added`, `Changed`, `Fixed` or `Removed`.
2. One entry per line, however long.
3. When an earlier commit since the last release already has an entry that stays true, add none. When that entry is now untrue, add the entry that says what is true; the release pull request reconciles the two.
4. A `feat` or `fix` commit with no entry says `Changelog: none`, as CONTRIBUTING.md says.
5. Claude closes the message with the trailers, in this order: the entries or `Changelog: none`, `Refs #N` or `Closes #N`, then the attribution lines.

## Before pushing

Check the commits not pushed yet with the `commit-messages` action, as AGENTS.md, "Building and testing", says, with the lower layer's branch as the base for a layer of a stack. Claude pushes only once it passes.

## After pushing

1. A pushed message the check accepts is never reworded. The next commit carries the entry that is true.
2. A pushed message the check refuses is reworded, the one rewrite CONTRIBUTING.md, "Stacked pull requests", rule 5, allows besides a rebase. On a layer with layers above it, reword it, then rebase every layer above, as the `stack` skill says.
