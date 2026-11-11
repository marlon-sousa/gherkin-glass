# 0001: Rust, the gherkin crate, askama, and 64-bit binaries

Decided on 2026-11-10, before issue [#1](https://github.com/marlon-sousa/gherkin-glass/issues/1).

## Why a tool of our own

Existing viewers were tried on a real suite of 66 features, 380 scenarios, before deciding to write one:

- **Pickles** (.NET, 4.0.3, 2022) runs on Linux as a command, but its project is end of life. It stops on any `Rule:`, opens every page with the whole navigation tree, and has no tag indexes.
- **piccali** (Rust and React, 0.10, 2026) looks good, but is a single-page app. A link straight to a page returns 404 from a static host. Scenarios have no anchors and their names are not headings, and the page has two nested `main` landmarks. Its crates.io release did not build, and its npm binary was not executable.
- **LivingDocGen** (.NET, 3.0, 2026) is built around test results. It reported an integrity error on its own fresh output, put the build machine's path in its navigation, and loads its icons from a CDN.
- **FeatureBook** is unmaintained. **sphinx-gherkindoc** needs Sphinx and produces input for it, not a site.

## Decisions

1. **Shipped as binaries**, not as a package of an interpreted language: one file to download for Linux, macOS and Windows, all 64-bit, with a GitHub Action that installs it ([#16](https://github.com/marlon-sousa/gherkin-glass/issues/16), [#17](https://github.com/marlon-sousa/gherkin-glass/issues/17)) and a cargo plugin ([#18](https://github.com/marlon-sousa/gherkin-glass/issues/18)).
2. **Rust.** It makes single static binaries and cross-compiles from one workflow, and the maintainer works in it. Go was the alternative, with Cucumber's official Go parser and a slightly simpler cross-compilation.
3. **The [`gherkin`](https://crates.io/crates/gherkin) crate** from cucumber-rs, 0.16.0 when this was written, as the parser. Tried on the same suite and on files of its own, it read every feature, rules with their own backgrounds, outlines with tagged and named examples, descriptions, the `\|` and `\n` escapes in table cells, and a `# language: pt` file with its Portuguese keywords. Every element carries its position in the file.

   One gap: **doc strings come back raw**. The media type is glued to the content (`"json\n..."`), the indentation is taken away by the least indented line rather than by the opening delimiter, and an escaped `\"\"\"` stays escaped. gherkin-glass corrects them after parsing, using the positions the crate returns ([#3](https://github.com/marlon-sousa/gherkin-glass/issues/3)). Comments are dropped, which is what the pages want.
4. **[askama](https://crates.io/crates/askama) templates**, checked when the binary is compiled, so a mistake in a template is a build error and not a broken page. Templates, styles and scripts are built into the binary.
5. **JavaScript is assumed**, since every browser has it. It drives the reader options, collapsing and search. But each page's content is in its HTML, never built by a script, so headings, anchors and links exist as the page loads, and a link to a scenario works on any static host. The one exception, agreed later, is the folder tree, built from shared data ([#7](https://github.com/marlon-sousa/gherkin-glass/issues/7)).
