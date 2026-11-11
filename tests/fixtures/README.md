# Fixtures

Each folder here is the input of one build, and is checked against the rules of [docs/BEHAVIOUR.md](../../docs/BEHAVIOUR.md) that name it.

- `input-files`: which files are read, with `.ghg-ignore` files in several folders, hidden files and folders, files of other types, and a `.gitignore` and an `.ignore` file that must have no effect.
- `constructs`: every construct of Gherkin in one feature, and a feature in Portuguese.
- `doc-strings`: doc strings with and without a media type, with both delimiters and their escapes, and with uneven indentation.
- `unreadable`: files that cannot be read, beside one that can and one a `.ghg-ignore` rule leaves out, to check that every problem is reported, sorted by path. Its build is meant to fail.
- `configuration`: the command line and `ghg.toml`, one folder per case, each run as the current folder. `default` has no `ghg.toml` and reads the default input. `configured` names its input, beside a `features` folder that would fail, and has a second file in `ci` whose input is relative to its own folder, and a third in `nested` with no `input`, whose default is relative to its own folder too. `mistaken` has unknown keys and a wrong type, `empty-input` an empty path, and `not-toml` is not valid TOML.

Some files here are left out of git by the fixture's own `.gitignore`, on purpose, and were added with `git add --force`.
