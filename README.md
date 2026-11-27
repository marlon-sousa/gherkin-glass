# gherkin-glass

gherkin-glass turns a folder of Gherkin feature files into a static website that anyone can read, with or without a screen reader. It is one command, `ghg`, for Linux, macOS and Windows.

<!-- site-only
Its source, its issues and how to contribute are in [its repository on GitHub](https://github.com/marlon-sousa/gherkin-glass).
-->

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

<!-- site-only
## The example suite

Each version of gherkin-glass builds the example suite of its repository, to show what its pages look like.
-->

<!-- ghg:contents -->

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
| `replace_existing_version` | `true` or `false` | `true` | Whether `ghg build` replaces the site, and `ghg version add` the version, that an earlier run wrote in the output folder. When `false`, it stops instead. |
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
ghg version add <name> [options]
ghg version remove <name> [options]
ghg preview add <name> [options]
ghg preview remove <name> [options]
```

`ghg build` writes a site without versions, and `ghg serve` builds it and serves it on your machine. `ghg version add` and `ghg version remove` add and remove a version of a site that holds several, as "Versions" below describes, and `ghg preview add` and `ghg preview remove` a preview, as "A site of only some feature files" below describes.

| Option | Command | What it does |
| --- | --- | --- |
| `--config <file>` | every command | Reads that configuration file instead of `ghg.toml` in the current folder. |
| `--input <folder>` | `build`, `serve`, `version add` and `preview add` | Reads that folder, instead of the configuration's `input`. |
| `--output <folder>` | every command | Writes the site into that folder, instead of the configuration's `output`. |
| `--title <text>` | `build`, `serve`, `version add` and `preview add` | The site's title, instead of the configuration's `title`. |
| `--front-page <file>` | `build`, `serve`, `version add` and `preview add` | The Markdown file of the site's front page, instead of the configuration's `front_page`. |
| `--only <file>` | `build`, `serve`, `version add` and `preview add` | Builds the site from the feature files listed in that file, one path on each line, alone, as "A site of only some feature files" below describes. `--only -` reads the list from standard input. |
| `--keep-in-output <pattern>` | every command | Leaves alone what the pattern matches in the output folder, in the syntax of a `.gitignore` file, as "The output folder belongs to ghg" below describes. Give it once for each pattern; the patterns given replace the configuration's `keep_in_output`. |
| `--source-link <address>` | `build`, `serve`, `version add` and `preview add` | The address of each feature file's source, with `{path}` and `{line}`, instead of the configuration's `source_link`. |
| `--allow-unnamed <true or false>` | `build`, `serve`, `version add` and `preview add` | Whether a feature, rule or scenario may have no name, instead of the configuration's `allow_unnamed`. |
| `--collapse-rules <true or false>` | `build`, `serve`, `version add` and `preview add` | Whether rules start collapsed, instead of the configuration's `collapse_rules`. |
| `--collapse-scenarios <true or false>` | `build`, `serve`, `version add` and `preview add` | Whether scenarios, scenario outlines and backgrounds start collapsed, instead of the configuration's `collapse_scenarios`. |
| `--show-tree <true or false>` | `build`, `serve`, `version add` and `preview add` | Whether the folder tree is shown when a reader first opens the site, instead of the configuration's `show_tree`. |
| `--inline-backgrounds <true or false>` | `build`, `serve`, `version add` and `preview add` | Whether pages start with the backgrounds inside each scenario expanded, instead of the configuration's `inline_backgrounds`. |
| `--replace-existing-version <true or false>` | `build`, `serve` and `version add` | Whether `ghg build` replaces the site, and `ghg version add` the version, that an earlier run wrote, instead of the configuration's `replace_existing_version`. |
| `--accent-light <colour>` | `build`, `serve`, `version add` and `preview add` | The light theme's accent colour, such as `'#0b4f9c'`, in quotes so that the shell does not take `#` for a comment, instead of the configuration's `accent_light`. |
| `--accent-dark <colour>` | `build`, `serve`, `version add` and `preview add` | The dark theme's accent colour, written as `--accent-light`'s, instead of the configuration's `accent_dark`. |
| `--tag-link <pattern> <address> <text>` | `build`, `serve`, `version add` and `preview add` | Links the tags the pattern matches to another site, as a table of `tag_links` does, such as `--tag-link '@issue-(\d+)' 'https://github.com/owner/repo/issues/$1' 'Issue $1'`. Give it once for each link; the links given replace the configuration's `tag_links`. |
| `--message-format <format>` | every command but `serve` | `human`, the default, or `json`, which also prints one JSON object to standard output, on one line, saying how the command ended, for a program such as a CI step to read, as "A report for programs" below describes. |
| `--host <address>` | `serve` | Listens on that address instead of `127.0.0.1`, such as `0.0.0.0` to be reached from other machines. |
| `--port <number>` | `serve` | Listens at that port instead of 8000. `0` lets the system choose a free one. |
| `--no-build` | `serve` | Serves the output folder as it is, without building it. It cannot be given with `--watch` or `--only`. |
| `--watch` | `serve` | Rebuilds the site whenever a feature file, the configuration, the front page's Markdown file or the list `--only` names changes, and reloads the pages open in the browser. With `--only`, a change to a feature file the list does not name rebuilds nothing. |

An option on the command line wins over the configuration file's key, and the key wins over the default. `ghg --help` lists the commands and options, and `ghg --version` prints the version.

`ghg build` exits with status 0 when it wrote the site, or when it read no feature file, so that it wrote nothing, as "When there is no feature file" below describes, 1 when the configuration, a feature file or the output folder stopped it, and 2 for a mistake on the command line. `ghg version add` and `ghg preview add` exit the same way. `ghg version remove` and `ghg preview remove` exit with status 0 when they removed it, and also when the output folder holds none of that name, which they say, so that a workflow can remove one that may not be there. `ghg serve` exits the same way, and with 1 when it cannot listen, such as on a port already in use. Control+C, or the signal a terminal or a process manager sends, stops it gracefully: it finishes a build that is running, says "The site is no longer served." and exits with status 0. Asked to stop again while it finishes answering, it ends at once with status 130.

### Looking at a site and publishing it

A site is meant to be served by a web server, not opened from disk. Its pages are drawn in the browser by the site's script, from the site's data, which a browser neither runs nor fetches from disk, so opened from disk a site shows only its front page. Every page opened from disk therefore first opens a dialog, "Opened from disk", that says the site is meant to be served and how to serve it; close it with its button, "I agree", or with Escape.

To look at a site on your own machine, use `ghg serve`. Without `--watch` or `--no-build`, it builds the site into the output folder, as `ghg build` does, then serves it. With `--watch`, the site is built in memory and served from there, the output folder is left as it is, and the page you are reading reloads at the same heading after each change. With `--no-build`, ghg serves a site already built, such as one downloaded from a CI run.

To publish a site, put the output folder on any static web host, GitHub Pages included. Every link in a site is relative, so it works at any address, below a path such as `/<repository>/` too. GitHub Pages publishing from a branch runs Jekyll over it first; publish with GitHub's Pages action instead, or put a `.nojekyll` file at the root of that branch, so that every file is served as it is. To build straight into a checkout of the branch, tell ghg to leave that file alone, and git's own `.git` with it, and `CNAME` when the site has a custom domain: `ghg build --output gh-pages --keep-in-output .nojekyll --keep-in-output .git --keep-in-output CNAME`.

### The output folder belongs to ghg

ghg marks the output folder as its own with a file at its root, `ghg-site.json`, which lists every file ghg wrote there, whether the folder holds a site, versions or previews. From then on the output folder, and every folder in it, belongs to ghg: any command may empty it and write it again, so anything you put there by hand can be lost.

To keep that from happening by mistake:

- ghg marks an output folder only when it is missing or empty. It refuses to build into a folder that holds anything and no marker.
- Before every command that writes it, ghg checks that a marked folder holds only what it wrote there. When it holds anything else, ghg names it and stops, before reading the features, and writes and deletes nothing.
- An output folder that gherkin-glass 1 wrote is left as it is: gherkin-glass 2 cannot add to it. Build into a new folder instead, as "Versions" below shows for a site of versions.

So keep the output folder for the site alone, and put anything you want served beside it somewhere else, unless you name it with `keep_in_output` or `--keep-in-output`. Each is a pattern in the syntax of a `.gitignore` file, such as `.nojekyll`, `/CNAME` for the one at the root only, or `drafts/` for a folder. ghg never writes, empties or deletes what a pattern matches, nor anything in a folder it matches, and does not count it as something it did not write, so a folder holding only those things counts as empty. A pattern that matches anything ghg writes, such as `assets` or the version being built, stops the build before anything is written.

### Versions

One site can hold several versions of the same suite, so that readers read the version they use. Add each version with `ghg version add` and its name, such as `ghg version add v1.2.0` or `ghg version add main`, which reads the feature files as `ghg build` does, with the same options and the same `ghg.toml`. A version's name is one folder name: letters `a` to `z` and `A` to `Z`, digits, `.`, `-`, `_` and `+`, starting with a letter or a digit and not ending with `.`. It cannot be `assets`, `data`, `index.html`, `ghg-site.json` or `previews`, which the site uses, nor a name Windows keeps for a device, such as `con` or `nul`. A name such as `1.2.0` or `v1.2.0` is a release, one such as `2.0.0-rc.1` a pre-release, and any other, such as `main`, a development version.

Each version is written into a folder of its own in the output folder, such as `site/v1.2.0/`, and the output folder's `index.html` lists every version: development versions, releases newest first, and pre-releases, listed collapsed. The versions share the site's data, in `site/data/`, so that a feature file the same in several versions is stored once, and each feature's page, and each of its rules, scenarios and backgrounds, lists the other versions in which it changed, is missing or is the same. Adding a version again replaces it, unless `replace_existing_version` is `false`. `ghg version remove v1.2.0` removes a version, and the data only it used. A site without versions, built with `ghg build`, and versions cannot share an output folder.

ghg does not read git. To build several versions, check each one out into a folder of its own and run `ghg version add` once for each, into the same output folder. To rebuild a site of versions that gherkin-glass 1 built, which gherkin-glass 2 does not add to, do that for every tag, into a new output folder. With git, from the root of the repository:

```sh
site="$PWD/site"
for tag in $(git tag --list 'v*'); do
  git worktree add --detach "../version-$tag" "$tag"
  (cd "../version-$tag" && ghg version add "$tag" --output "$site")
  git worktree remove "../version-$tag"
done
ghg version add main --output "$site"
```

Each version is built from its own `ghg.toml`, as it was at that tag. gherkin-glass 2 no longer has the key `version_name`, so a `ghg.toml` that sets it stops the run: give that tag's settings on the command line instead, with `--config` naming a file without the key.

### A site of only some feature files

To review a change to your feature files, such as a pull request, you can build a site of the feature files it changes or adds alone, and read them as pages rather than as a diff. Give `--only` a list of paths, one on each line, relative to the current folder, such as the list git prints. From the folder of your `ghg.toml`, at the root of the repository:

```sh
git diff --name-only origin/main...HEAD | ghg build --only -
```

Every path in the list that is not a feature file ghg would read is passed over quietly: a removed file, a file that is not a feature file, or a path outside the input folder, so the list needs no filter. A list that names no path at all, such as an empty one, and one whose paths are none of them feature files build nothing, and say so, as "When there is no feature file" below describes. The site's folder pages, folder tree, tags, step sentences and search hold only the listed features, and every page says that the site shows only the feature files given with `--only`. git quotes paths with characters outside ASCII; `git -c core.quotePath=false diff --name-only` prints them as they are. With `ghg serve --watch`, a list given as a file is read again whenever it changes.

To publish such a site for the reviewers of a pull request, add it as a preview, with a name of its own:

```sh
git diff --name-only origin/main...HEAD | ghg preview add pr-131 --only -
```

A preview is written into `previews/<name>/` of the output folder, such as `site/previews/pr-131/`, beside the site or the versions already there, whose data it shares, and every page of it asks search engines not to index it. It reads as a site without versions does, and on a site of versions each of its pages lists, as a version's do, the versions in which it changed, is missing or is the same. Nothing links to it: readers reach it through its link. Its name follows the rules of a version's, adding it again replaces it, and `ghg preview remove pr-131` removes it, such as when the pull request closes. `--only` works with `ghg version add` too, for a version of only some feature files.

### When there is no feature file

An input folder that holds no feature file ghg reads, such as a suite whose first feature is not written yet, or a list given to `--only` that names none, such as the changes of a pull request that touches no feature file, is nothing to build rather than a mistake. `ghg build` then writes nothing, not even the output folder, so that no empty site or empty version is published. It says why on standard error, such as `features: the input folder holds no feature file, so no site was built.`, and exits with status 0. It still checks the output folder first, and an input folder that does not exist still stops it. `ghg serve` does the same, and serves nothing. `ghg serve --watch`, which never writes the output folder, serves the empty site all the same, so that you can start a suite with it and see the first feature appear, and says on standard error that the site served is empty.

### A report for programs

A program that runs `ghg build`, such as a step of a CI workflow, can ask for `--message-format json`. Standard error then holds the same sentences as ever, and standard output one JSON object, on one line, such as:

```json
{"outcome":"built","output":"site","version":null,"preview":null,"features":12,"scenarios":48,"notes":[],"errors":[]}
```

Every key is always there, `null` when it does not apply:

| Key | What it holds |
| --- | --- |
| `outcome` | `built`, when the site, the version or the preview was written; `nothing-to-build`, when no feature file was read; `removed`, when a version or a preview was removed; `nothing-to-remove`, when there was none of that name to remove; `stopped`, when the configuration, a feature file, the output folder or the front page stopped the command. |
| `output` | The output folder, when something was written or removed. |
| `version` | The version's name, when a version was written or removed. |
| `preview` | The preview's name, when a preview was written or removed. |
| `features` | How many feature files the site shows, when it was written. |
| `scenarios` | How many scenarios they hold, when it was written. |
| `notes` | What ghg says of a command that was not stopped, each with a `kind`, `no-title`, `large-site-data`, `nothing-to-build` or `nothing-to-remove`, and the `message` it prints. |
| `errors` | Every problem that stopped the command, each with its `path`, its `line`, or `null`, and its `message`. |

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

<!-- github-only -->
## Contributing

### Requirements

- **Rust**, through [rustup](https://rustup.rs/) 1.28 or later. In the repository, `rustup toolchain install` installs the Rust version the repository pins, with its components.
- **Node**, at the version `script/.node-version` names, with npm, only to change the frontend: the pages' script, stylesheet, templates or interface text. Those changes need `cargo xtask build`, and the browser check, which both need Node. The script is TypeScript, in `script/`, and its compiled form, `assets/gherkin-glass.js`, is committed.
- **cargo-deny 0.20.2**, for the dependency check: `cargo install cargo-deny --version 0.20.2 --locked`.

If you don't change the frontend, Rust is enough: `cargo build` builds `ghg`, `cargo test` runs the tests, and `cargo run -- serve --config project-site/ghg.toml --watch` serves the repository's example suite at <http://127.0.0.1:8000/>.

### The xtask commands

`cargo xtask` runs the repository's own tasks. `cargo xtask --help` lists them.

| Command | What it does | Needs Node |
| --- | --- | --- |
| `cargo xtask build` | Type-checks, tests and compiles the script into `assets/gherkin-glass.js`, writes again the expected sites of the test fixtures and of the example suite, with the pages the script draws from each, the expected stores of the fixtures that commit only their store and of the history of versions, and the crate's README, `README.crate.md`, from this one, and builds the release binary. Run it after changing the script, the stylesheet, a template or interface text, and commit what it writes. After changing only this README, `project-site/crate-readme.sh README.md README.crate.md` writes the crate's README again, with no Node. | Yes |
| `cargo xtask check` | Runs every check a pull request runs: formatting, lints, tests on the pinned and the minimum supported Rust, documentation, dependencies, the package, the script check and the browser check. | Yes |
| `cargo xtask check-script` | Type-checks and tests the script, and fails when the committed `assets/gherkin-glass.js` is not what `script/` compiles to, or a page the script draws from a test site is not the one committed beside it. | Yes |
| `cargo xtask check-browser` | Installs a headless Chromium and checks every test site in it, the README's own front page included: accessibility in both colour schemes, the keyboard, a window 320 pixels wide, pages opened from disk, the folder tree on a narrow window, the reload of `ghg serve --watch`, and the focus and the title as a shell's pages are navigated. | Yes |
| `cargo xtask comment-ratio [path ...]` | Counts the comment lines and the code lines of the Rust and TypeScript files given, or of every one in the repository. | No |

### Interface text and localization

Every word gherkin-glass writes itself, on the pages and on the command line, such as "Skip to content", "Collapse all" or a message about a mistake in `ghg.toml`, comes from one catalogue of strings. Feature files are shown as written, in their own language; only gherkin-glass's own words come from the catalogue.

- The catalogue is in [Fluent](https://projectfluent.org/), in `messages/en/gherkin-glass.ftl`, and built into the binary. English is the only language for now; a translation would be a new file in a folder of its own beside `en/`, but choosing the language is not built yet.
- Code reads a string by its id, with the `fl!` macro, such as `fl!(CATALOGUE, "skip-to-content")`. `fl!` checks at compile time that the id exists in the English catalogue and that its arguments are the ones the message takes, so a missing string is a build error, not a broken page. It checks only arguments passed by name, so arguments are always passed by name.
- No template, Rust code or script writes interface text inline. The templates receive their words from Rust. The script draws the pages with the messages of the catalogue after its line `### ghg:page-messages`, which every page carries and the script reads with Fluent's own library, so the script holds no words.
- A message is a whole sentence with named placeholders, such as `{ $feature }, a feature in { $folder }`, never pieces joined together, since word order differs between languages. A count is a plural form chosen by Fluent, never a word with an `s` added.

Changing what a string says changes what gherkin-glass promises, so it starts as an issue too, as below. To add or change a string, edit it in the catalogue, use it by its id, and run `cargo xtask build` when it changes what pages say, so that the expected sites the tests compare against are written again. One expected page is not rebuilt by it, the versions page of `tests/fixtures/front-page`, `expected-versions.html`; update it by hand.

### How changes are made

Every change starts as an issue. Open one that says what you want and why, or, for a bug, what happened, what you expected and the smallest feature file that shows it. The maintainers then design the plan with you in the issue, and work starts only once it is agreed: a pull request that names its issue.

### How it is built

gherkin-glass is written in Rust, with the [`gherkin`](https://crates.io/crates/gherkin) parser from cucumber-rs. AI is part of how it is made: agents write code and documentation, and a human maintainer reviews everything before it is merged, under the terms of [A manifesto for software engineering with AI](https://marlon-sousa.com/blog/manifesto/).

<!-- end github-only -->

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/marlon-sousa/gherkin-glass/blob/main/LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/marlon-sousa/gherkin-glass/blob/main/LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
