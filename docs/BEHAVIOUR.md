# Behaviour

What gherkin-glass promises, in terms a test can observe. Each section is written when its issue is agreed, before it is built, and every rule in it is checked by a fixture under `tests/fixtures/` ([CONTRIBUTING.md](../CONTRIBUTING.md), "Fixtures").

## Status

Agreed so far:

- [Reading the input](#reading-the-input), issue [#3](https://github.com/marlon-sousa/gherkin-glass/issues/3).

The other features planned for the first version are the [issues](https://github.com/marlon-sousa/gherkin-glass/issues) labelled `feature`. The command line that starts a build, and the input folder it reads, are agreed in [#11](https://github.com/marlon-sousa/gherkin-glass/issues/11).

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
16. Each line reads `<path>:<line>: <message>`, where the path is relative to the input folder, with `/` between folders on every system, the line is where the parser stopped, or the file's last line when it stopped past the end, and the message is the parser's. When the parser has no message of its own, only the list of what it expected, the message is `Syntax error.` A problem with no line, such as a file that cannot be read, reads `<path>: <message>`. A problem with the input folder itself shows its path as it was given. Fixture: `tests/fixtures/unreadable`.
