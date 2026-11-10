# Behaviour

What gherkin-glass promises, in terms a test can observe. Each section is written when its issue is agreed, before it is built, and every rule in it is checked by a fixture under `tests/fixtures/` ([CONTRIBUTING.md](../CONTRIBUTING.md), "Fixtures").

## Status

Agreed so far:

- [Reading the input](#reading-the-input), issue [#3](https://github.com/marlon-sousa/gherkin-glass/issues/3).
- [The command line and its configuration](#the-command-line-and-its-configuration), issue [#11](https://github.com/marlon-sousa/gherkin-glass/issues/11).
- [Anchors](#anchors), issue [#5](https://github.com/marlon-sousa/gherkin-glass/issues/5).

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
16. Each line reads `<path>:<line>: <message>`, where the path is relative to the input folder, with `/` between folders on every system, the line is where the parser stopped, or the file's last line when it stopped past the end, and the message is the parser's. When the parser has no message of its own, only the list of what it expected, the message is `Syntax error.` A problem with no line, such as a file that cannot be read, reads `<path>: <message>`. A file that cannot be read has the message `cannot be read: <reason>`, a folder that cannot be listed `cannot be listed: <reason>`, a file that changed while it was being read `changed while it was being read`, and a `.ghg-ignore` rule that is not valid the `ignore` crate's message, where the reason is the operating system's words. A problem with the input folder itself shows its path as it was given. Fixture: `tests/fixtures/unreadable`.

## The command line and its configuration

The command is `ghg`. How it builds a site is set by a configuration file, `ghg.toml`, kept with the feature files, and by options on the command line, which override it, so that a site builds the same way locally and in CI. Every sentence it prints is gherkin-glass's own interface text, except the messages of the libraries and the operating system it passes on, which are named where they appear.

### Commands

17. `ghg build` reads the input, as [Reading the input](#reading-the-input) describes. When every file can be read, it prints nothing and exits with status 0. It writes nothing yet: writing the site is agreed in [#4](https://github.com/marlon-sousa/gherkin-glass/issues/4), and is added to this rule when it is built. Fixture: `tests/fixtures/configuration/default`.
18. `ghg --help`, or `-h`, prints the commands and their options to standard output and exits with status 0, as it does after a command, such as `ghg build --help`. `ghg --version`, or `-V`, prints `ghg` and its version, such as `ghg 0.1.0`, the same way. Fixture: `tests/fixtures/configuration/default`.
19. A mistake on the command line stops the run before anything is read. It prints one sentence to standard error, naming what was typed wrong when something was, and exits with status 2. The mistakes are: no command; a command ghg does not have; an option the command does not have; an option without its value, or with an empty one, such as `--input=`; a value given to an option that takes none, such as `--help=yes`; an option given more than once; and an argument the command does not take. Fixture: `tests/fixtures/configuration/default`.
20. When the configuration file or the input stops a build, the report goes to standard error, one line per problem, nothing goes to standard output, and the status is 1. A configuration file with a problem stops the run before the input is read. Fixtures: `tests/fixtures/configuration/mistaken` and `tests/fixtures/unreadable`.

### The options of ghg build

An option's value follows it, as `--input specs`, or is joined to it by `=`, as `--input=specs`. Paths on the command line are relative to the current folder.

21. `--config <file>` reads that configuration file instead of `ghg.toml` in the current folder. A file given this way that cannot be read, because it does not exist or for any other reason, stops the run with `<file>: cannot be read: <reason>`, the file shown as it was given and the reason in the operating system's words. Fixture: `tests/fixtures/configuration/configured`.
22. `--input <folder>` reads that folder as the input, whatever the configuration file's `input` says. Fixture: `tests/fixtures/configuration/configured`.

### The configuration file

23. Without `--config`, the configuration file is `ghg.toml` in the current folder, and only there: the folders above are not searched. When there is none, every setting takes its default, and that is not a problem. A `ghg.toml` that is there but cannot be read, such as a folder of that name or a file that is not UTF-8, stops the run with `ghg.toml: cannot be read: <reason>`, as rule 21 describes. Fixture: `tests/fixtures/configuration/configured`.
24. The file is written in [TOML](https://toml.io/). Paths in it, and the defaults of the keys that take a path, are relative to the folder the file is in, whichever folder ghg runs from. Without a file, the defaults are relative to the current folder. It may set these keys, and the features that need more add them here as they are built:

    | Key | Takes | Default | What it sets |
    | --- | --- | --- | --- |
    | `input` | text | `features` | The input folder, the folder of feature files to read. |
    | `allow_unnamed` | `true` or `false` | `true` | Whether features, rules and scenarios may have no name, as [Anchors](#anchors), rule 35, describes. |

    Fixture: `tests/fixtures/configuration/configured`.
25. A setting given on the command line wins over the file's key of the same meaning, and the file's key wins over the default. Fixture: `tests/fixtures/configuration/configured`.
26. A key the file may not set, a table among them, or a value of the wrong type, is never ignored: it stops the run. Every such problem in the file is reported, sorted by line, each as `<file>:<line>: <message>`, the file shown as given and the line that of the key, or for a wrong type that of its value. An unknown key reads `ghg has no setting named <key>.`, a key that takes text and was given something else `<key> must be text, in quotes.`, a key that takes `true` or `false` and was given something else `<key> must be true or false.`, and a path given as empty text `<key> must not be empty.` A file that is not valid TOML, such as one with a key given twice or text without quotes, is reported once, at the line where the [`toml`](https://crates.io/crates/toml) crate stopped, with that crate's message. Fixtures: `tests/fixtures/configuration/mistaken`, `tests/fixtures/configuration/not-boolean`, `tests/fixtures/configuration/empty-input` and `tests/fixtures/configuration/not-toml`.

## Anchors

Every scenario, scenario outline, rule, background and examples block of a feature has an anchor, made from its name by the rule below, and so does every scenario an outline produces from a row of its examples ([#13](https://github.com/marlon-sousa/gherkin-glass/issues/13)). A scenario outline is any scenario with examples, whatever its keyword. A link to a block keeps working as long as its file and its name do. The anchor is the `id` of the block's heading on its feature's page, once pages are written ([#4](https://github.com/marlon-sousa/gherkin-glass/issues/4)); until then, the build makes the anchors and the fixture's test checks them.

The rule is part of what is promised: changing it is a major version ([CONTRIBUTING.md](../CONTRIBUTING.md), "Releases").

### The anchor rule

27. The name is normalised to Unicode NFC and lower-cased. Fixture: `tests/fixtures/anchors`.
28. Apostrophes, `'` and `’`, are removed. Fixture: `tests/fixtures/anchors`.
29. Every run of characters other than letters and digits becomes one `-`, and there is no `-` at either end. Letters and digits are the characters Unicode gives the Alphabetic or Numeric property, and the marks that combine with them, such as Devanagari's virama, so that a word of any script stays whole. Fixture: `tests/fixtures/anchors`.
30. Letters outside a to z, such as `ã` or `ç`, are kept, not transliterated or percent-encoded: "Saque em conta poupança" becomes `saque-em-conta-poupança`. A browser reaches such an anchor whether a link writes it plainly or percent-encoded, as in `#saque-em-conta-poupan%C3%A7a`, the form an address bar shows. Fixture: `tests/fixtures/anchors`, which has a feature in Portuguese.
31. A block with no name takes the keyword written in the file instead, made into an anchor by rules 27 to 29, such as `background`, `contexto` or `esquema-do-cenário`. So does a block whose name leaves nothing after rule 29, such as `???`. When the keyword leaves nothing too, as the emoji keywords of Gherkin's `em` language do, the anchor is the block's kind in English: `background`, `rule`, `scenario`, `scenario-outline` or `examples`. Fixture: `tests/fixtures/anchors`.
32. Anchors are unique on a page. Taking the blocks in file order, each takes the anchor rules 27 to 31 make, or, when an earlier block already has it, the first of that anchor followed by `-2`, `-3` and so on that no earlier block has. So an unnamed rule's background, after the feature's, is `background-2`. Fixture: `tests/fixtures/anchors`.
33. A scenario produced from a row of an outline's examples takes the outline's anchor, then `-row-`, then the row's number, counted from 1 across all the outline's examples blocks in file order, without their header rows, such as `withdraw-cash-row-3`. These anchors are made after every other anchor on the page, in file order, and one that an earlier anchor already has is made unique as rule 32 says. Fixture: `tests/fixtures/anchors`.
34. A feature has no anchor of its own: it is the page.

### Blocks without a name

35. Backgrounds and examples blocks may always be unnamed. Features, rules and scenarios, outlines included, may be unnamed while `allow_unnamed` is `true`, its default. When it is `false`, each unnamed feature, rule, scenario and scenario outline is a problem that stops the build, reported with the problems of [Files that cannot be read](#files-that-cannot-be-read), sorted with them, as rules 15 and 16 describe, at the line of its keyword. The messages are `The feature has no name, and allow_unnamed is false.`, and the same with `rule`, `scenario` or `scenario outline` in place of `feature`. Fixtures: `tests/fixtures/configuration/unnamed` and `tests/fixtures/configuration/unnamed-allowed`.
