---
name: architecture
description: How Claude finds which parts of gherkin-glass's docs/ARCHITECTURE.md a change makes untrue, and checks the whole document against the code. Use when reviewing a change, and when asked whether the document is still true.
---

# Keeping the architecture document true

[AGENTS.md](../../../AGENTS.md), "Reviewing a change", says that a change updates every statement of docs/ARCHITECTURE.md it makes untrue. This skill says how to find them.

## What each section describes

| Section | Also untrue when |
|---|---|
| The repository | a top-level folder, a binary or a crate is added, removed or renamed |
| Module roles | a module's role changes, or a build step moves between roles |
| Folders and files | `lib.rs` exports something new, or a file or a test module grows past its guide |
| Writing the code | a lint, a derive or a dependency it names changes |
| Comments | never by code alone |
| Dependencies between parts | a service, a driving port or the composition root changes |
| How the code runs | concurrency, error handling or page addresses change |
| Interface text | the catalogue, its loading or `i18n.toml` change |
| The pages' script | a file under `script/` changes role, or the script's runtime dependencies change |
| Tests | a test tier, a test dependency or `cargo xtask check` changes |
| What a contributor needs | a tool a contributor must install changes |

## Checking a change

1. List the files the change touches, and the modules, binaries, lints, dependencies and checks it adds, removes or renames.
2. For each, find the sections of the table it touches.
3. Read each section found against the code after the change. Each statement is either still true, or untrue, with the `file:line` that contradicts it.
4. Check that every path and name the section gives still exists.
5. For each untrue statement, check that the change corrects it. A missing correction is a finding, with the text to write.

## Checking the whole document

When asked, do steps 3 and 4 for every section, against the current tip of the stack or `main`.
