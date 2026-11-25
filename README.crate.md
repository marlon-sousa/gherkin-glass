<!-- Written from README.md by project-site/crate-readme.sh, which cargo xtask build runs. Edit README.md, then run it again. -->

# gherkin-glass

gherkin-glass turns a folder of Gherkin feature files into a static website that anyone can read, with or without a screen reader. It is one command, `ghg`, for Linux, macOS and Windows.

Its source, its issues and how to contribute are in [its repository on GitHub](https://github.com/marlon-sousa/gherkin-glass).

## Why it exists

Gherkin is the plain-language format behaviour-driven development uses to describe how software should behave. A feature file names a feature and lists its scenarios, each a short sequence of steps such as `Given a customer named "Ana"`, `When she withdraws 50`, `Then her balance is 150`. Tools such as Cucumber run those steps as tests, but the files are written to be read by people: product owners, testers, developers and anyone who wants to know what the software does.

Reading them is harder than it should be. They live in a repository, spread across folders, and are opened one at a time in an editor. gherkin-glass makes those use cases easy to read, browse, search and link to, for everyone, including people who use a screen reader or only a keyboard.

## What you get

- A page for every feature file and for every folder, with a folder tree beside them.
- A heading and a stable link for every rule, scenario and background, so that a link to a scenario keeps working as long as its file and its name do, and no block before it on its page takes the same name.
- A page for every tag, listing the features and scenarios that carry it, and optional links from tags to another site, such as an issue tracker.
- A page of step sentences, listing every distinct step sentence, with quoted values and placeholders as slots, and where it is used.
- Search across features, rules, scenarios, backgrounds and examples, with a page of every result.
- Every Gherkin construct: rules, backgrounds, scenario outlines and their examples, shown as scenarios too, data tables, doc strings, tags, descriptions, and every spoken language Gherkin supports.
- Highlighting of keywords, quoted parameters, placeholders and numbers in steps, which never carries meaning on its own.
- Reader options the browser remembers: a light or a dark theme, or the system's, showing or hiding the folder tree, showing a background's steps inside each scenario or once at the top, and collapsing rules and scenarios.
- Pages built for assistive technology: headings in order, real tables and lists, and a skip link.
- Pages checked in a browser on every change, against WCAG 2.2 at levels A and AA and a contrast of at least 7:1, in both colour schemes and with the keyboard alone.

## Status

The latest release is `1.1.0`, on crates.io and on the [releases page](https://github.com/marlon-sousa/gherkin-glass/releases). Since `1.0.0`, the first stable version, versions follow [Semantic Versioning](https://semver.org/): the addresses of pages and their anchors, the command line, the configuration keys and the names of the release archives change in a way that breaks a site, a command or an installation only in a new major version. The newest code, not yet released, can be built from the repository with `cargo install --git https://github.com/marlon-sousa/gherkin-glass gherkin-glass --locked`, which installs `ghg` and `cargo-ghg` and needs [Rust](https://rustup.rs/).

## The example suite

Each version of gherkin-glass builds the example suite of its repository, to show what its pages look like.

See them on [the project's site](https://marlon-sousa.github.io/gherkin-glass/).

## Installing and running

Install gherkin-glass in one of these ways:

- `cargo install gherkin-glass`, which builds it with Rust's package manager;
- `cargo binstall gherkin-glass`, which downloads the released binary instead of building it;
- or download the archive for your system from the [releases page](https://github.com/marlon-sousa/gherkin-glass/releases), check it against the release's `SHA256SUMS` if you wish, and put `ghg`, and `cargo-ghg` for `cargo ghg`, somewhere on your path. There are archives for Linux on x86-64 and ARM64, macOS on Intel and Apple silicon, and Windows on x86-64 and ARM64.

An alpha or a beta, a preview of a version released before it, such as 1.1.0-alpha.1 before 1.1.0, is released on GitHub only, not on crates.io. Install it from its archive on the releases page, or build it with `cargo install --git https://github.com/marlon-sousa/gherkin-glass --tag <tag> gherkin-glass --locked`, naming its tag, such as `v1.1.0-alpha.1`.

Each way installs `ghg`, and `cargo-ghg`, which lets you run it as `cargo ghg` beside `cargo test`. `cargo ghg build` is the same as `ghg build`.

Then, in the folder of your project, with your feature files in a folder named `features`:

```sh
ghg build
ghg serve
```

`ghg build` writes the site into a folder named `site`. `ghg serve` builds it the same way, then serves it at <http://127.0.0.1:8000/>. Open that address in a browser, and press Control+C to stop.

## Usage

### The configuration file

ghg reads its settings from `ghg.toml`, a [TOML](https://toml.io/) file in the folder you run it from, or the file `--config` names. The file is optional: every setting has a default. Paths in it are relative to the folder the file is in, wherever you run ghg from.

```toml
input = "features"
output = "site"
title = "Our features"
source_link = "https://github.com/owner/repo/blob/main/features/{path}#L{line}"
collapse_scenarios = true

[[tag_links]]
pattern = '@issue-(\d+)'
address = "https://github.com/owner/repo/issues/$1"
text = "Issue $1 on GitHub"
```

| Key | Takes | Default | What it sets |
| --- | --- | --- | --- |
| `input` | a path | `features` | The folder of feature files to read. It must exist. |
| `output` | a path | `site` | The folder the site is written into. |
| `title` | text | `Feature files` | The site's title, shown on its home page and in every breadcrumb. |
| `allow_unnamed` | `true` or `false` | `true` | Whether a feature, rule or scenario may have no name. When `false`, one without a name stops the build. |
| `source_link` | text | none | The address of each feature file's source, so that each page links to its file. `{path}` is replaced by the file's path below the input folder, percent-encoded, and `{line}` by the line of its feature; the link reads `Source: <path>`. |
| `collapse_rules` | `true` or `false` | `false` | Whether rules start collapsed. |
| `collapse_scenarios` | `true` or `false` | `false` | Whether scenarios, scenario outlines and backgrounds start collapsed. |
| `show_tree` | `true` or `false` | `true` | Whether the folder tree is shown when a reader first opens the site. |
| `inline_backgrounds` | `true` or `false` | `true` | Whether pages start with the backgrounds inside each scenario expanded and those in their own place collapsed, or the opposite. |
| `tag_links` | a list of tables, each written `[[tag_links]]` | none | Links from tags to another site. Each table has a `pattern`, a regular expression matched against the whole tag with its `@`; an `address`; and a `text` for the link. In the address and the text, `$1` or `$name` is replaced by what that group of the pattern matched. |
| `accent_light` | a colour, `#` and six or three hexadecimal digits, in quotes, such as `"#0b4f9c"` | `"#0b4f9c"` | The light theme's colour for links and the focus outline. It must reach a contrast of 7:1 against the theme's backgrounds. |
| `accent_dark` | a colour, written as `accent_light`'s, such as `"#8cc4ff"` | `"#8cc4ff"` | The dark theme's colour for links and the focus outline, held to the same contrast. |
| `version_name` | text | none | Builds the site as a version of that name, as "Versions" below describes, unless `--preview` is given. |
| `replace_existing_version` | `true` or `false` | `true` | Whether a build replaces the site, or the version, that an earlier build wrote in the output folder. When `false`, it stops instead. |
| `front_page` | a path | none | A Markdown file that introduces the site on its front page, as "The front page" below describes. |
| `keep_in_output` | a list of text, such as `[".nojekyll", "CNAME"]` | none | Patterns of what ghg leaves alone in the output folder, in the syntax of a `.gitignore` file, as "The output folder belongs to ghg" below describes. |

A key ghg does not know, or a value of the wrong kind, stops the run with a message naming the file and the line, so that nothing is silently ignored.

### Which files are read

ghg reads every file whose name ends in `.feature` under the input folder, and leaves out hidden files and folders, those whose names start with a dot. A file named `.ghg-ignore`, in any folder of the input, changes that with the same rules as a `.gitignore` file: `drafts/` leaves a folder out, and `!*.story` reads files ending in `.story` too. No other file decides what is read: `.gitignore` files are not used.

When a file cannot be read, such as one with a syntax mistake, ghg reports every such file, each as `<path>:<line>: <message>`, and writes nothing.

### The command line

```text
ghg build [options]
ghg serve [options]
```

| Option | Command | What it does |
| --- | --- | --- |
| `--config <file>` | `build` and `serve` | Reads that configuration file instead of `ghg.toml` in the current folder. |
| `--input <folder>` | `build` and `serve` | Reads that folder, instead of the configuration's `input`. |
| `--output <folder>` | `build` and `serve` | Writes the site into that folder, instead of the configuration's `output`. |
| `--title <text>` | `build` and `serve` | The site's title, instead of the configuration's `title`. |
| `--version-name <name>` | `build` and `serve` | Builds the site as that version, instead of the configuration's `version_name`. |
| `--front-page <file>` | `build` and `serve` | The Markdown file of the site's front page, instead of the configuration's `front_page`. |
| `--only <file>` | `build` and `serve` | Builds the site from the feature files listed in that file, one path on each line, alone, as "A site of only some feature files" below describes. `--only -` reads the list from standard input. |
| `--keep-in-output <pattern>` | `build` and `serve` | Leaves alone what the pattern matches in the output folder, in the syntax of a `.gitignore` file, as "The output folder belongs to ghg" below describes. Give it once for each pattern; the patterns given replace the configuration's `keep_in_output`. |
| `--source-link <address>` | `build` and `serve` | The address of each feature file's source, with `{path}` and `{line}`, instead of the configuration's `source_link`. |
| `--allow-unnamed <true or false>` | `build` and `serve` | Whether a feature, rule or scenario may have no name, instead of the configuration's `allow_unnamed`. |
| `--collapse-rules <true or false>` | `build` and `serve` | Whether rules start collapsed, instead of the configuration's `collapse_rules`. |
| `--collapse-scenarios <true or false>` | `build` and `serve` | Whether scenarios, scenario outlines and backgrounds start collapsed, instead of the configuration's `collapse_scenarios`. |
| `--show-tree <true or false>` | `build` and `serve` | Whether the folder tree is shown when a reader first opens the site, instead of the configuration's `show_tree`. |
| `--inline-backgrounds <true or false>` | `build` and `serve` | Whether pages start with the backgrounds inside each scenario expanded, instead of the configuration's `inline_backgrounds`. |
| `--replace-existing-version <true or false>` | `build` and `serve` | Whether a build replaces the site, or the version, that an earlier build wrote, instead of the configuration's `replace_existing_version`. |
| `--accent-light <colour>` | `build` and `serve` | The light theme's accent colour, such as `'#0b4f9c'`, in quotes so that the shell does not take `#` for a comment, instead of the configuration's `accent_light`. |
| `--accent-dark <colour>` | `build` and `serve` | The dark theme's accent colour, written as `--accent-light`'s, instead of the configuration's `accent_dark`. |
| `--tag-link <pattern> <address> <text>` | `build` and `serve` | Links the tags the pattern matches to another site, as a table of `tag_links` does, such as `--tag-link '@issue-(\d+)' 'https://github.com/owner/repo/issues/$1' 'Issue $1'`. Give it once for each link; the links given replace the configuration's `tag_links`. |
| `--preview` | `build` and `serve` | Builds a preview: the site without versions, whatever the configuration's `version_name` says, every page of which asks search engines not to index it, as "A site of only some feature files" below describes. It cannot be given with `--version-name`. |
| `--message-format <format>` | `build` | `human`, the default, or `json`, which also prints one JSON object to standard output, on one line, saying how the build ended, for a program such as a CI step to read, as "A report for programs" below describes. |
| `--host <address>` | `serve` | Listens on that address instead of `127.0.0.1`, such as `0.0.0.0` to be reached from other machines. |
| `--port <number>` | `serve` | Listens at that port instead of 8000. `0` lets the system choose a free one. |
| `--no-build` | `serve` | Serves the output folder as it is, without building it. It cannot be given with `--watch` or `--only`. |
| `--watch` | `serve` | Rebuilds the site whenever a feature file, the configuration, the front page's Markdown file or the list `--only` names changes, and reloads the pages open in the browser. With `--only`, a change to a feature file the list does not name rebuilds nothing. |

An option on the command line wins over the configuration file's key, and the key wins over the default. `ghg --help` lists the commands and options, and `ghg --version` prints the version.

`ghg build` exits with status 0 when it wrote the site, or when it read no feature file, so that it wrote nothing, as "When there is no feature file" below describes, 1 when the configuration, a feature file or the output folder stopped it, and 2 for a mistake on the command line. `ghg serve` exits the same way, and with 1 when it cannot listen, such as on a port already in use. Control+C, or the signal a terminal or a process manager sends, stops it gracefully: it finishes a build that is running, says "The site is no longer served." and exits with status 0. Asked to stop again while it finishes answering, it ends at once with status 130.

### Looking at a site and publishing it

A site is meant to be served by a web server, not opened from disk. Opened from disk, a browser does not run the page's script, so the folder tree, search and the reader options do not work. Every page opened from disk therefore first opens a dialog, "Opened from disk", that says the site is meant to be served and how to serve it; close it with its button, "I agree", or with Escape, to read the page.

To look at a site on your own machine, use `ghg serve`. Without `--watch` or `--no-build`, it builds the site into the output folder, as `ghg build` does, then serves it. With `--watch`, the site is built in memory and served from there, the output folder is left as it is, and the page you are reading reloads at the same heading after each change. With `--no-build`, ghg serves a site already built, such as one downloaded from a CI run.

To publish a site, put the output folder on any static web host, GitHub Pages included. Every link in a site is relative, so it works at any address, below a path such as `/<repository>/` too. GitHub Pages publishing from a branch runs Jekyll, which leaves out files whose names start with `_`, such as the page of `_draft.feature`; publish with GitHub's Pages action instead, or put a `.nojekyll` file at the root of that branch. To build straight into a checkout of the branch, tell ghg to leave that file alone, and git's own `.git` with it, and `CNAME` when the site has a custom domain: `ghg build --output gh-pages --keep-in-output .nojekyll --keep-in-output .git --keep-in-output CNAME`.

### The output folder belongs to ghg

`ghg build` marks the output folder as its own with a file at its root, `ghg-site.json`, or `ghg-versions.json` when it builds versions. From then on the output folder, and every folder in it, belongs to ghg: any build may empty it and write it again, so anything you put there by hand can be lost.

To keep that from happening by mistake:

- ghg marks an output folder only when it is missing or empty. It refuses to build into a folder that holds anything and no marker.
- Before every build, ghg checks that a marked folder holds only what it wrote there. When it holds anything else, ghg names it and stops, before reading the features, and writes and deletes nothing.

So keep the output folder for the site alone, and put anything you want served beside it somewhere else, unless you name it with `keep_in_output` or `--keep-in-output`. Each is a pattern in the syntax of a `.gitignore` file, such as `.nojekyll`, `/CNAME` for the one at the root only, or `drafts/` for a folder. ghg never writes, empties or deletes what a pattern matches, nor anything in a folder it matches, and does not count it as something it did not write, so a folder holding only those things counts as empty. A pattern that matches anything ghg writes, such as `assets` or the version being built, stops the build before anything is written.

### Versions

One site can hold several versions of the same suite, so that readers read the version they use. Give each build a version name, with `--version-name` or `version_name`, such as `v1.2.0` or `main`. A version name is one folder name, of the letters `a` to `z` and `A` to `Z`, digits, `.`, `-`, `_` and `+`, other than `previews`, which is kept for previews, as "A site of only some feature files" below describes. A name such as `1.2.0` or `v1.2.0` is a release, one such as `2.0.0-rc.1` a pre-release, and any other, such as `main`, a development version. Each version is written into a folder of its own in the output folder, such as `site/v1.2.0/`, and the output folder's `index.html` lists every version: development versions, releases newest first, and pre-releases, listed collapsed.

ghg does not read git. To build several versions, check each one out into a folder of its own and run ghg once for each, into the same output folder. Building a version again replaces it, unless `replace_existing_version` is `false`.

### A site of only some feature files

To review a change to your feature files, such as a pull request, you can build a site of the feature files it changes or adds alone, and read them as pages rather than as a diff. Give `--only` a list of paths, one on each line, relative to the current folder, such as the list git prints. From the folder of your `ghg.toml`, at the root of the repository:

```sh
git diff --name-only origin/main...HEAD | ghg build --only -
```

Every path in the list that is not a feature file ghg would read is passed over quietly: a removed file, a file that is not a feature file, or a path outside the input folder, so the list needs no filter. A list that names no path at all, such as an empty one, and one whose paths are none of them feature files build nothing, and say so, as "When there is no feature file" below describes. The site's folder pages, folder tree, tags, step sentences and search hold only the listed features, and every page says that the site shows only the feature files given with `--only`. git quotes paths with characters outside ASCII; `git -c core.quotePath=false diff --name-only` prints them as they are. With `ghg serve --watch`, a list given as a file is read again whenever it changes.

To publish such a site for the reviewers of a pull request, build it as a preview, into a folder of its own:

```sh
git diff --name-only origin/main...HEAD | ghg build --only - --preview --output previews/pr-131
```

A preview is an ordinary site without versions, even when `ghg.toml` names a version, and every page of it asks search engines not to index it. Nothing links to it: readers reach it through its link, wherever you deploy the folder. A versioned output folder keeps a `previews` folder at its top for them: ghg never looks into it, writes or removes anything there, so `--output site/previews/pr-131` deploys a preview beside the versions built into `site`, and the next version built into `site` still runs.

### When there is no feature file

An input folder that holds no feature file ghg reads, such as a suite whose first feature is not written yet, or a list given to `--only` that names none, such as the changes of a pull request that touches no feature file, is nothing to build rather than a mistake. `ghg build` then writes nothing, not even the output folder, so that no empty site or empty version is published. It says why on standard error, such as `features: the input folder holds no feature file, so no site was built.`, and exits with status 0. It still checks the output folder first, and an input folder that does not exist still stops it. `ghg serve` does the same, and serves nothing. `ghg serve --watch`, which never writes the output folder, serves the empty site all the same, so that you can start a suite with it and see the first feature appear, and says on standard error that the site served is empty.

### A report for programs

A program that runs `ghg build`, such as a step of a CI workflow, can ask for `--message-format json`. Standard error then holds the same sentences as ever, and standard output one JSON object, on one line, such as:

```json
{"outcome":"built","output":"site","version":null,"features":12,"scenarios":48,"notes":[],"errors":[]}
```

Every key is always there, `null` when it does not apply:

| Key | What it holds |
| --- | --- |
| `outcome` | `built`, when the site was written; `nothing-to-build`, when no feature file was read; `stopped`, when the configuration, a feature file, the output folder or the front page stopped the build. |
| `output` | The output folder, when the site was written. |
| `version` | The version built, when the site was written as a version. |
| `features` | How many feature files the site shows, when it was written. |
| `scenarios` | How many scenarios they hold, when it was written. |
| `notes` | What ghg says of a build that was not stopped, each with a `kind`, `no-title`, `large-search-documents` or `nothing-to-build`, and the `message` it prints. |
| `errors` | Every problem that stopped the build, each with its `path`, its `line`, or `null`, and its `message`. |

The exit status is the same as without the option, and a mistake on the command line prints no JSON.

### The front page

The front page is the page at the root of the output folder: the home page, or, when the site has versions, the page that lists them. It can open with an introduction you write in Markdown, such as your project's README, named by `front_page` or `--front-page`. Every other page, each version's own home page included, stays as ghg writes it.

Write the comment `<!-- ghg:contents -->` on a line of its own where ghg's part of the page goes: the totals, folders and features of a home page, or the lists of versions. Markdown viewers such as GitHub's do not show the comment, so a README that holds it reads as before there.

```markdown
# Our features

What the suite covers, and how to read it.

## Versions

<!-- ghg:contents -->
```

- The file holds one level-one heading, written with one `#`, which becomes the page's title.
- ghg's own headings, such as "Folders" or "Releases", go one level below the last heading before the comment, so in the example above they are level three. The comment cannot follow a level 5 or level 6 heading, and it cannot be inside a list, a quote or a table.
- Everything Markdown writes is kept, GitHub's tables included. Other comments are left out of the page.
- Links are written exactly as the file has them, neither rewritten nor checked. A relative link is relative to the front page, so a link to a file of your repository, such as `LICENSE`, works on GitHub and not on the site; write the file's full address to reach it from both.
- HTML written in the file, other than comments, is not allowed.

Without `front_page`, the front page has only its title and ghg's part, as without this setting. A file that cannot be read, or that breaks one of these rules, stops the build with a message naming the file, and the line when there is one, and nothing is written. With `ghg serve --watch`, a change to the file rebuilds the site.


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/marlon-sousa/gherkin-glass/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/marlon-sousa/gherkin-glass/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
