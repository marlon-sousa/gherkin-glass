# Behaviour

What gherkin-glass promises, in terms a test can observe. Each section is written when its issue is agreed, before it is built, and every rule in it is checked by a fixture under `tests/fixtures/` ([CONTRIBUTING.md](../CONTRIBUTING.md), "Fixtures").

## Status

Agreed so far:

- [Reading the input](#reading-the-input), issue [#3](https://github.com/marlon-sousa/gherkin-glass/issues/3).
- [The command line and its configuration](#the-command-line-and-its-configuration), issue [#11](https://github.com/marlon-sousa/gherkin-glass/issues/11).
- [Anchors](#anchors), issue [#5](https://github.com/marlon-sousa/gherkin-glass/issues/5).
- [Pages](#pages), issue [#4](https://github.com/marlon-sousa/gherkin-glass/issues/4).
- [Previewing a site](#previewing-a-site), issue [#26](https://github.com/marlon-sousa/gherkin-glass/issues/26).

The other features planned for the first version are the [issues](https://github.com/marlon-sousa/gherkin-glass/issues) labelled `feature`. Each adds its own keys and options to "The command line and its configuration" as it is built.

## Reading the input

Before anything is built, gherkin-glass finds the feature files under the input folder and reads every one of them. If any file cannot be read, the build stops and nothing is written.

### Which files are read

1. By default, a file is read when its name ends in `.feature`. Every other file is left out. Fixture: `tests/fixtures/input-files`.
2. By default, every hidden file or folder, one whose name starts with a dot, is left out with everything in it, even a file whose name ends in `.feature`. Fixture: `tests/fixtures/input-files`.
3. A file named `.ghg-ignore` changes these defaults, and nothing else does. It uses the [gitignore syntax](https://git-scm.com/docs/gitignore#_pattern_format) and may sit in any folder of the input. Its rules apply to the paths in that folder and everything below it, as a `.gitignore` file's rules do. Fixture: `tests/fixtures/input-files`.
   - A rule leaves out what it matches, such as `drafts/` or `old-*.feature`.
   - A rule starting with `!` reads again what the defaults or an earlier rule leave out, such as `!*.story` for files ending in `.story`, or `!.github/` for a hidden folder.
   - When several rules match a path, the last matching rule of the `.ghg-ignore` file nearest to the path decides. The defaults decide only when no rule in any `.ghg-ignore` file matches.
   - A path left out with its folder cannot be read again by a rule, because the folder is not looked into, as in git.
   - A `.ghg-ignore` file is never read as a feature, whatever the rules say.
4. No other file decides what is read: not `.gitignore` files, not `.ignore` files, not git's own exclude files, and not the ignore files of the folders above the input. Fixture: `tests/fixtures/input-files`.
5. Symbolic links are not followed, whether they point to a file or to a folder. Fixture: created by the test, since a symbolic link cannot be relied on in a checkout on every system.

### What is read

6. Each file is read as UTF-8 with the [`gherkin`](https://crates.io/crates/gherkin) crate: its feature, backgrounds, rules, scenarios, scenario outlines and their examples, steps, data tables, doc strings, tags and descriptions. Fixture: `tests/fixtures/constructs`.
7. A file written in any spoken language Gherkin supports is read with that language's keywords, given by a `# language:` line at the top of the file. A file without one is read in English. Fixture: `tests/fixtures/constructs`, which has a file in Portuguese.
8. Comments, the lines starting with `#`, are not kept.

### Doc strings

A doc string is the text between two delimiter lines, `"""` or three backticks, under a step. gherkin-glass keeps it as the author wrote it, as [Gherkin's own reference](https://cucumber.io/docs/gherkin/reference/#doc-strings) describes. Fixture: `tests/fixtures/doc-strings`.

9. Text after the opening delimiter, such as `json` in `"""json`, is the doc string's media type, kept apart from its content. A doc string without it has no media type.
10. The content is the lines between the opening and the closing delimiter, joined by line breaks, without a line break before the first line or after the last. A line of the content that starts with `#` is content, not a comment. From a `#` anywhere in a line of the content, a delimiter later on that line does not close the doc string.
11. Each line of the content loses as much of its leading indentation as the opening delimiter has, counting a space or a tab as one character. A line indented less loses all of it. A line indented more keeps the difference.
12. Inside a doc string opened with `"""`, an escaped delimiter `\"\"\"` reads as `"""`. Inside one opened with three backticks, three escaped backticks read as three backticks.
13. A file whose lines end in a carriage return and a line feed is read as if they ended in a line feed alone.

### Files that cannot be read

14. A file that cannot be read fails the build: a feature file that cannot be parsed, such as an empty file or one with only comments, or that cannot be read, such as one that is not UTF-8; a `.ghg-ignore` file with a rule that is not valid, whose other rules still apply; and a folder of the input that cannot be listed, the input folder itself included. There is no option to carry on without it. Fixture: `tests/fixtures/unreadable`.
15. Every such file is reported before the build stops, not only the first. The report has one line per problem, sorted by path, folder by folder, so that a folder's contents stay together, and then by line. Names are compared by their character codes, so capital letters come before small ones. The report is the same on every run. Fixture: `tests/fixtures/unreadable`.
16. Each line reads `<path>:<line>: <message>`, where the path is relative to the input folder, with `/` between folders on every system, the line is where the parser stopped, or the file's last line when it stopped past the end, and the message is the parser's. When the parser has no message of its own, only the list of what it expected, the message is `Syntax error.` A problem with no line, such as a file that cannot be read, reads `<path>: <message>`. A file that cannot be read has the message `cannot be read: <reason>`, a folder that cannot be listed `cannot be listed: <reason>`, a file that changed while it was being read `changed while it was being read`, and a `.ghg-ignore` rule that is not valid the `ignore` crate's message, where the reason is the operating system's words. A problem with the input folder itself shows its path as it was given, and an input folder that does not exist, the default `features` among them, reads `<path>: the input folder does not exist. Create it, or name another with input in ghg.toml or with --input.` Fixture: `tests/fixtures/unreadable`.

## The command line and its configuration

The command is `ghg`. How it builds a site is set by a configuration file, `ghg.toml`, kept with the feature files, and by options on the command line, which override it, so that a site builds the same way locally and in CI. Every sentence it prints is gherkin-glass's own interface text, except the messages of the libraries and the operating system it passes on, which are named where they appear.

### Commands

17. `ghg build` reads the input, as [Reading the input](#reading-the-input) describes, and when every file can be read, writes the site into the output folder, as [Pages](#pages) describes. When it has written the site, it prints nothing and exits with status 0. Fixtures: `tests/fixtures/configuration/default` and `tests/fixtures/pages`.
18. `ghg --help`, or `-h`, prints the commands and their options to standard output and exits with status 0, as it does after a command, such as `ghg build --help`. `ghg --version`, or `-V`, prints `ghg` and its version, such as `ghg 0.1.0`, the same way. Fixture: `tests/fixtures/configuration/default`.
19. A mistake on the command line stops the run before anything is read. It prints one sentence to standard error, naming what was typed wrong when something was, and exits with status 2. The mistakes are: no command; a command ghg does not have; an option the command does not have; an option without its value, or with an empty one, such as `--input=`; a value given to an option that takes none, such as `--help=yes`; an option given more than once; and an argument the command does not take. Fixture: `tests/fixtures/configuration/default`.
20. When the configuration file or the input stops a build, the report goes to standard error, one line per problem, nothing goes to standard output, and the status is 1. A configuration file with a problem stops the run before the input is read. Fixtures: `tests/fixtures/configuration/mistaken` and `tests/fixtures/unreadable`.

### The options of ghg build

An option's value follows it, as `--input specs`, or is joined to it by `=`, as `--input=specs`. Paths on the command line are relative to the current folder.

21. `--config <file>` reads that configuration file instead of `ghg.toml` in the current folder. A file given this way that cannot be read, because it does not exist or for any other reason, stops the run with `<file>: cannot be read: <reason>`, the file shown as it was given and the reason in the operating system's words. Fixture: `tests/fixtures/configuration/configured`.
22. `--input <folder>` reads that folder as the input, whatever the configuration file's `input` says, and `--output <folder>` writes the site into that folder, whatever the configuration file's `output` says. Fixture: `tests/fixtures/configuration/configured`.

### The configuration file

23. Without `--config`, the configuration file is `ghg.toml` in the current folder, and only there: the folders above are not searched. When there is none, every setting takes its default, and that is not a problem. A `ghg.toml` that is there but cannot be read, such as a folder of that name or a file that is not UTF-8, stops the run with `ghg.toml: cannot be read: <reason>`, as rule 21 describes. Fixture: `tests/fixtures/configuration/configured`.
24. The file is written in [TOML](https://toml.io/). Paths in it, and the defaults of the keys that take a path, are relative to the folder the file is in, whichever folder ghg runs from. Without a file, the defaults are relative to the current folder. It may set these keys, and the features that need more add them here as they are built:

    | Key | Takes | Default | What it sets |
    | --- | --- | --- | --- |
    | `input` | text | `features` | The input folder, the folder of feature files to read. It must exist, as rule 16 says. |
    | `allow_unnamed` | `true` or `false` | `true` | Whether features, rules and scenarios may have no name, as [Anchors](#anchors), rule 35, describes. |
    | `output` | text | `site` | The output folder, where the site is written, as [Pages](#pages), rule 38, describes. |
    | `source_link` | text | none | The address of each feature file's source, as [Pages](#pages), rule 47, describes. Without it, pages have no link to their source. |
    | `collapse_rules` | `true` or `false` | `false` | Whether rules start collapsed, as [Pages](#pages), rule 51, describes; checked by `tests/fixtures/pages`. |
    | `collapse_scenarios` | `true` or `false` | `false` | Whether scenarios, scenario outlines and backgrounds start collapsed, as [Pages](#pages), rule 51, describes; checked by `tests/fixtures/pages`. |

    Fixture: `tests/fixtures/configuration/configured`.
25. A setting given on the command line wins over the file's key of the same meaning, and the file's key wins over the default. Fixture: `tests/fixtures/configuration/configured`.
26. A key the file may not set, a table among them, or a value of the wrong type, is never ignored: it stops the run. Every such problem in the file is reported, sorted by line, each as `<file>:<line>: <message>`, the file shown as given and the line that of the key, or for a wrong type that of its value. An unknown key reads `ghg has no setting named <key>.`, a key that takes text and was given something else `<key> must be text, in quotes.`, a key that takes `true` or `false` and was given something else `<key> must be true or false.`, and a path or address given as empty text `<key> must not be empty.` A file that is not valid TOML, such as one with a key given twice or text without quotes, is reported once, at the line where the [`toml`](https://crates.io/crates/toml) crate stopped, with that crate's message. Fixtures: `tests/fixtures/configuration/mistaken`, `tests/fixtures/configuration/mistaken-pages`, `tests/fixtures/configuration/not-boolean`, `tests/fixtures/configuration/empty-input` and `tests/fixtures/configuration/not-toml`.

## Anchors

Every scenario, scenario outline, rule, background and examples block of a feature has an anchor, made from its name by the rule below, and so does every scenario an outline produces from a row of its examples ([#13](https://github.com/marlon-sousa/gherkin-glass/issues/13)). A scenario outline is any scenario with examples, whatever its keyword. A link to a block keeps working as long as its file, its name and the blocks before it on its page that share its anchor stay as they are; an outline's rows keep theirs as long as the rows before them do. A block's anchor is the `id` of its heading on its feature's page, as [Pages](#pages) describes. The anchors of an outline's rows are made now, and are shown on the page once the scenarios an outline produces are ([#13](https://github.com/marlon-sousa/gherkin-glass/issues/13)).

The rule is part of what is promised: changing it is a major version ([CONTRIBUTING.md](../CONTRIBUTING.md), "Releases").

### The anchor rule

27. The name is normalised to Unicode NFC and lower-cased, and the anchor is in NFC. Fixture: `tests/fixtures/anchors`.
28. Apostrophes, `'` and `’`, are removed. Fixture: `tests/fixtures/anchors`.
29. Every run of characters other than letters and digits becomes one `-`, and there is no `-` at either end. Letters and digits are the characters Unicode gives the Alphabetic or Numeric property. A combining mark, such as Devanagari's virama, stays when it follows a letter, a digit or another mark that stays, and so does a zero width joiner or non-joiner between one that stays and a letter or digit, as Persian writes inside words; anywhere else, such as the variation selector after an emoji, it is a gap like any other character. Fixture: `tests/fixtures/anchors`.
30. Letters outside a to z, such as `ã` or `ç`, are kept, not transliterated or percent-encoded: "Saque em conta poupança" becomes `saque-em-conta-poupança`. Fixture: `tests/fixtures/anchors`, which has a feature in Portuguese.
31. A block with no name takes the keyword written in the file instead, made into an anchor by rules 27 to 29, such as `background`, `contexto` or `esquema-do-cenário`. So does a block whose name leaves nothing after rule 29, such as `???`. When the keyword leaves nothing too, as the emoji keywords of Gherkin's `em` language do, the anchor is the block's kind in English: `background`, `rule`, `scenario`, `scenario-outline` or `examples`. Fixture: `tests/fixtures/anchors`.
32. Anchors are unique on a page. Taking the blocks in file order, each takes the anchor rules 27 to 31 make, or, when an earlier block already has it, the first of that anchor followed by `-2`, `-3` and so on that no earlier block has. So an unnamed rule's background, after the feature's, is `background-2`. The `id`s gherkin-glass gives the page's own parts, such as a skip link's target, each contain `_`, which no anchor contains, so that no anchor is ever taken by one of them, nor one of them by an anchor. Fixture: `tests/fixtures/anchors`, and `tests/fixtures/pages` for the page's own `id`s.
33. A scenario produced from a row of an outline's examples takes the outline's anchor, then `-row-`, then the row's number, counted from 1 across all the outline's examples blocks in file order, without their header rows, such as `withdraw-cash-row-3`. These anchors are made after every other anchor on the page, in file order, and one that an earlier anchor already has is made unique as rule 32 says. Fixture: `tests/fixtures/anchors`.
34. A feature has no anchor of its own: it is the page. Fixture: `tests/fixtures/anchors`, where a scenario named as its feature takes the anchor unnumbered.

### Blocks without a name

35. Backgrounds and examples blocks may always be unnamed. Features, rules and scenarios, outlines included, may be unnamed while `allow_unnamed` is `true`, its default. When it is `false`, each unnamed feature, rule, scenario and scenario outline is a problem that stops the build, reported with the problems of [Files that cannot be read](#files-that-cannot-be-read), sorted with them, as rules 15 and 16 describe, at the line of its keyword. The messages are `The feature has no name, and allow_unnamed is false.`, and the same with `rule`, `scenario` or `scenario outline` in place of `feature`. Fixtures: `tests/fixtures/configuration/unnamed` and `tests/fixtures/configuration/unnamed-allowed`.

## Pages

Each feature file becomes one page of the site, which a screen reader can move through by heading, scenario by scenario, and which reads well for everyone. A site is meant to be served over HTTP, by any static web server or host or by `ghg serve` ([#26](https://github.com/marlon-sousa/gherkin-glass/issues/26)), and is not opened from disk ([design note 0026](design/0026-sites-are-served.md)).

### The site

36. The site has one page for each feature file, at the file's path below the input folder with its last extension replaced by `.html`, or with `.html` added when its name has none, as for `README`, or when its only dot is its first character, as for `.feature`, which becomes `.feature.html`. So `billing/refunds.feature` becomes `billing/refunds.html`, and `stories/read.story`, read through a `.ghg-ignore` rule, becomes `stories/read.html`. Folders are separated by `/` on every system. Fixture: `tests/fixtures/pages`.
37. Two files whose pages would have the same path, such as `a.feature` and `a.story`, stop the build, and so do two whose pages' paths differ only in the case of their letters, such as `A.feature` and `a.feature`, since some file systems do not tell them apart. Each such file after the first, in the order of rule 15, is a problem reported with those of [Files that cannot be read](#files-that-cannot-be-read), sorted with them, as `<path>: has the same page, <page>, as <first path>.`, where the page is its own. Fixture: `tests/fixtures/same-page`, and the tests of `src/services/site.rs` for letters in another case, which a checkout cannot hold side by side on every system.
38. The site is written into the output folder, `site` by default, or the folder `output` or `--output` names. The output folder and the folders in it are made as needed. A file already there at the path of a file the build writes is replaced, and nothing else in the output folder is changed or deleted. A file that cannot be written, or a folder that cannot be made for it, stops the build, which reports `<path>: cannot be written: <reason>`, the path being the output folder as it was given followed by the path in it of the file being written, and the reason in the operating system's words, as rule 20 describes. The files written before it stay. Fixtures: `tests/fixtures/pages`, and an output folder that cannot be written, created by the test.
39. Beside the pages, the site has `assets/gherkin-glass.css`, its stylesheet, and `assets/gherkin-glass.js`, the script of its behaviour, an ES module. Both are built into ghg and are the same in every site a version of ghg builds. Every page links them by a relative address, as every link in the site is, so that a site works at any address, below a path such as GitHub Pages' `/<repository>/` included. Fixture: `tests/fixtures/pages`.

### What a page holds

40. A page's content is in its HTML as it is written, never built by its script: every heading, anchor, link, list and table is there as the page loads. Fixture: `tests/fixtures/pages`.
41. The feature's own text, its keywords included, is shown as written in the file. gherkin-glass's own text, such as "Contents" and "Collapse all", is in English, whatever language the file is written in. Fixture: `tests/fixtures/pages`, which has a feature in Portuguese.
42. The page's `lang` is the language of its feature file, as rule 7 reads it, and `en` for a file without a `# language:` line. It is written as Gherkin names the language, such as `pt` or `zh-TW`, except for the names Gherkin gives that are not that language's tag, which are written as the tag: `am`, Armenian, as `hy`; `bm`, Malay, as `ms`; `gj`, Gujarati, as `gu`; `lu`, Luxembourgish, as `lb`; `en-old`, Old English, as `ang`; `en-lol` and `en-tx` as `en`; and `em`, whose keywords are emoji, as `und`, undetermined. Nothing else on the page has a `lang` of its own, so that gherkin-glass's own text is read in the page's language too. Fixture: `tests/fixtures/pages`.
43. A block's heading reads its keyword as written, a colon, a space and its name, such as `Scenario: Withdraw cash` or `Cenário: Saque`. A block without a name reads its keyword alone. The feature's heading is the page's only `h1`. The page's title is the feature's heading, a comma, a space and the file's path below the input folder, such as `Feature: Withdraw cash, banking/withdraw.feature`. Fixture: `tests/fixtures/pages`.
44. The feature's background and its rules are `h2`. A scenario under a rule is `h3`, and a scenario outside any rule is `h2`, at the same level as the rules. A rule's background is at the same level as the rule's scenarios, `h3`. An examples block is one level below its outline, `h3` or `h4`. Here and below, a scenario is a scenario outline too. Fixture: `tests/fixtures/pages`.
45. Under the `h1` come, in this order: the feature's tags, its description, the link to its source (rule 47), the page's contents (rule 46), the buttons of rule 50, and then the feature's background, its scenarios and its rules, in the order of the file. Under a rule's heading come its tags, its description, its background and its scenarios; under a scenario's, its tags, its description, its steps and its examples blocks; under a background's, its description and its steps; and under an examples block's, its tags, its description and its table. Fixture: `tests/fixtures/pages`.
    - **Tags** are a list, each tag as written, with its `@`.
    - **A description** is one paragraph that keeps its line breaks. Its blank lines are not kept, since the `gherkin` crate does not keep them.
    - **Steps** are a numbered list, each step its keyword as written and its text, such as `Given a customer named "Ana"`, followed by its data table or doc string.
    - **A data table** is a table whose first row is its header: its cells are header cells, each heading its column, and the cells of the other rows are data cells. Each cell is its text as rule 6 reads it, with `\|` read as `|`.
    - **A doc string** is preformatted text, its content as [Doc strings](#doc-strings) describes, captioned by its media type when it has one.
    - **An examples block** without a table has nothing under its heading but its tags and its description.
46. The page's contents are a navigation landmark named "Contents", with a list of links to each of the page's rules and scenarios in the order of the file, each rule's scenarios in a list inside the rule's item. Each link's text is its block's heading, and its address is `#` followed by the block's anchor. A page with no rule and no scenario has no contents. Fixture: `tests/fixtures/pages`.
47. When `source_link` is set, the page has a link to its feature file's source, which reads `Source: ` followed by the file's path below the input folder. Its address is `source_link` with every `{path}` replaced by that path, with each byte other than an ASCII letter, a digit, `-`, `.`, `_`, `~` and `/` percent-encoded, and every `{line}` by the line of the feature's keyword, all else kept as written. So `https://github.com/owner/repo/blob/main/features/{path}#L{line}` links the file on GitHub at its feature's line, and a template without `{line}` links the file alone. Without `source_link`, the page has no such link. Fixtures: `tests/fixtures/pages` and `tests/fixtures/configuration/configured`.
48. Every heading of a block has the block's anchor as its `id`, as [Anchors](#anchors) describes, and is the target of a link to that anchor. Fixture: `tests/fixtures/pages`.

### Collapsing

49. The heading of every rule, scenario, background and examples block holds a button whose text is the heading's, and which says, with `aria-expanded`, whether the block is expanded. The block's content is everything under its heading, as rule 45 lists it, and it is hidden while the block is collapsed. The heading stays on the page while its content is hidden, so heading navigation still reaches every rule and scenario, collapsed or not. Fixture: `tests/fixtures/pages`.
50. "Collapse all" and "Expand all" buttons come before the page's first block. A page with no block has neither. Fixture: `tests/fixtures/pages`.
51. When the page loads, every rule is collapsed when `collapse_rules` is `true`, and every scenario and background when `collapse_scenarios` is `true`; otherwise they are expanded. Examples blocks are expanded. The page's HTML is written in this state, so that the page changes as it loads only for the blocks whose state the reader chose before, as rule 54 describes. Fixture: `tests/fixtures/pages`, built once with each setting.
52. Pressing a block's button collapses the block when it is expanded, and expands it when it is collapsed. The blocks inside a collapsed block are hidden with its content, and keep their own states. Fixtures: the script's own tests, `script/test/adapters/dom_page.test.ts`, on `tests/fixtures/pages/expected/billing/refunds.html`, and `script/test/services/collapsing.test.ts`.
53. "Collapse all" collapses every block on the page, and "Expand all" expands every one. Fixtures: `script/test/adapters/dom_page.test.ts` and `script/test/services/collapsing.test.ts`.
54. The browser remembers each block's state, page by page, kept under the page's path and the block's anchor, whenever it changes: by its button, by the buttons of rule 53, which are remembered as the state of every block on the page, or by rule 55. When the page is opened again, a block's remembered state wins over the starting state of rule 51. A block whose anchor changes loses its remembered state. A browser that keeps nothing, such as one whose storage is forbidden, gets the same page, which remembers nothing. Fixtures: `script/test/adapters/dom_page.test.ts`, `script/test/services/collapsing.test.ts` and `script/test/policies/block_state.test.ts`.
55. Following a link to a block's anchor, whether the page is opened at it or the link is on the page, expands that block and every block it is inside. A click that opens the link in another tab or window changes nothing on the page. Fixtures: `script/test/adapters/dom_page.test.ts` and `script/test/services/collapsing.test.ts`.

## Previewing a site

`ghg serve` builds a site and serves it over HTTP on the reader's own machine, so that it can be looked at before it is published. A site gherkin-glass builds is meant to be served, by `ghg serve` or by any static web server or host, and opening its pages straight from disk is not supported ([design note 0026](design/0026-sites-are-served.md)).

### Serving

56. `ghg serve` reads the configuration file and the command line as `ghg build` does, and has the options of `ghg build`, which mean the same. It builds the site into the output folder, as `ghg build` would, then serves that folder. When the configuration or the input stops the build, it reports it as `ghg build` does, as rule 20 describes, serves nothing, and exits with status 1. Fixture: `tests/fixtures/serve/built`.
57. `--no-build` serves the output folder as it is, without reading the input or building anything. Fixture: `tests/fixtures/serve/site`.
58. It listens on the address `127.0.0.1`, so that nothing on the network reaches it unless asked, at port 8000. `--host <address>` listens on another address, such as `0.0.0.0` for every network the machine is on, and `--port <number>` on another port. Port 0 asks the operating system for a free port. A port that is not a whole number from 0 to 65535 is a mistake on the command line, as rule 19 describes, and reads `The option --port needs a whole number from 0 to 65535, not <value>.` Fixture: `tests/fixtures/serve/site`.
59. When the port is already in use, at that address or at every address, as another server listening on `0.0.0.0` does, it serves nothing, never choosing another port by itself, prints `Port <port> is already in use. Choose another with --port.` to standard error, and exits with status 1. When it cannot listen for any other reason, such as an address the machine does not have, it prints `ghg serve cannot listen at <address>, port <port>: <reason>`, the reason in the operating system's words, and exits with status 1. Fixture: `tests/fixtures/serve/site`.
60. Once it listens, it prints `The site is served at <address>. Press Control+C to stop.` to standard output, where the address is the one to open in a browser, such as `http://127.0.0.1:8000/`, with the port the operating system chose for port 0. When it listens on every network, as with `0.0.0.0` or `::`, which a browser cannot open, the address printed is this machine's own, `127.0.0.1` or `[::1]`. It serves until it is stopped. Fixture: `tests/fixtures/serve/site`.

### What it answers

61. It answers `GET` and `HEAD` requests. A request's path names a file below the served folder, its parts decoded from percent-encoding as UTF-8, and its query left aside. A path ending in `/` names the `index.html` file of that folder. A path naming a folder without the `/` is redirected, with status 301, to the same path with the `/`, so that the page's relative links work. Fixture: `tests/fixtures/serve/site`.
62. A file is served with status 200, exactly as it is, and with the content type its extension gives, in this table. Any other extension, and a file with none, is served as `application/octet-stream`. Fixture: `tests/fixtures/serve/site`.

    | Extension | Content type |
    | --- | --- |
    | `.html` | `text/html; charset=utf-8` |
    | `.css` | `text/css; charset=utf-8` |
    | `.js` and `.mjs` | `text/javascript; charset=utf-8` |
    | `.json` and `.map` | `application/json` |
    | `.txt` | `text/plain; charset=utf-8` |
    | `.svg` | `image/svg+xml` |
    | `.png` | `image/png` |
    | `.jpg` and `.jpeg` | `image/jpeg` |
    | `.gif` | `image/gif` |
    | `.webp` | `image/webp` |
    | `.ico` | `image/x-icon` |
    | `.woff2` | `font/woff2` |
    | `.woff` | `font/woff` |

63. Every answer says `Cache-Control: no-cache`, so that a browser asks again for a file before using its copy. Fixture: `tests/fixtures/serve/site`.
64. A path with no file, a path with an empty part, a part `.` or a part `..`, or a path that cannot be decoded, is answered with status 404 and the plain text sentence `Nothing is at this address in the site.` No request's path leads outside the served folder, though a symbolic link inside it is followed. Any method other than `GET` and `HEAD` is answered with status 405 and the sentence `ghg serve answers only GET and HEAD requests.` Fixture: `tests/fixtures/serve/site`.

### Watching

65. `--watch` watches the input folder, everything in it, its `.ghg-ignore` files among them, and the configuration file, whether it exists yet or not. When any of them changes, it reads the configuration file again and rebuilds the whole site. An editor's burst of changes is one rebuild, and a change made during a rebuild starts another once it is done. The folders watched are those ghg serve started with, even when the configuration file names another input. Fixture: `tests/fixtures/serve/built`.
66. With `--watch`, every version of the site, the first one included, is built in memory and served from there, so the output folder is never written or deleted. A new version is served only once all of it is built, so no half-built page is ever served. `ghg build` writes the output folder. Fixture: `tests/fixtures/serve/built`.
67. When a rebuild is served, it prints `The site was rebuilt.` to standard output. When the configuration or the input stops a rebuild, it prints the report `ghg build` would print to standard error, then `The site was not rebuilt, so the last site that built is still served.`, and carries on serving that site. When the first build stops, ghg serve stops, as rule 56 describes. Fixture: `tests/fixtures/serve/built`.
68. `--watch` and `--no-build` together are a mistake on the command line, as rule 19 describes, which reads `The options --watch and --no-build cannot be used together.` When watching cannot start, such as when the system's limit on watched folders is reached, it prints `ghg serve cannot watch <path> for changes: <reason>` to standard error, the reason in the operating system's or the `notify` crate's words, and exits with status 1. Fixture: `tests/fixtures/serve/built`.

### Reloading

69. With `--watch`, every page served as `text/html` has one element added, `<script type="module" src="/__ghg/reload.js?build=<number>"></script>`, just before its last `</body>`, or at its end when it has none, where the number is that of the version it comes from, counted from 1. Nothing else of the page changes, and without `--watch` nothing is added. Fixture: `tests/fixtures/serve/built`.
70. With `--watch`, the path `/__ghg/build` answers the number of the version being served, as plain text, and `/__ghg/reload.js` answers the reload script. Without `--watch`, both are paths of the site like any other. Fixture: `tests/fixtures/serve/built`.
71. The reload script asks `/__ghg/build` once a second, and reloads the page when the number differs from its page's. While ghg serve does not answer, it keeps asking. Fixture: `tests/fixtures/serve/reload`, in the browser, by `tests/browser/reload.mjs`.
72. Before reloading, it remembers the heading the reader was in: when an element of the page has focus and some of it is in the window, the last heading with an `id` at or before that element in the page, and otherwise the last heading with an `id` whose top is at or above the top of the window. After the reload, if the page still has a heading with that `id`, it scrolls the heading to the top of the window and moves focus to it, so that a sighted reader keeps their place and a screen reader resumes there rather than at the top. A focused element scrolled out of the window, such as the heading focused by the last reload after the reader scrolled on with the mouse, does not count. When there is no such heading, the page opens as any page does. The script says nothing of its own, so it has no interface text. Fixture: `tests/fixtures/serve/reload`, in the browser, by `tests/browser/reload.mjs`.
