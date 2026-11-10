### gherkin-glass's own interface text, in English.
### docs/ARCHITECTURE.md, "Interface text", says how it is written and used.

## The command line.

help =
    ghg turns a folder of Gherkin feature files into an accessible website.

    Usage: ghg <command> [options]

    Commands:
      build: reads the feature files, and reports every one that cannot be read.

    Options of ghg build:
      --config <file>: the configuration file to read, instead of ghg.toml in the current folder.
      --input <folder>: the folder of feature files to read, instead of the configuration's input.

    Other options:
      --help, -h: prints this help.
      --version, -V: prints the version.

version = ghg { $version }

## A mistake on the command line. $command is "ghg" or "ghg build".

missing-command = ghg needs a command, such as ghg build. ghg --help lists the commands.
unknown-command = ghg has no command named { $name }. ghg --help lists the commands.
unknown-option = { $command } has no option { $option }. ghg --help lists the options.
missing-value = The option { $option } needs a value.
unexpected-value = The option { $option } takes no value.
repeated-option = The option { $option } is given more than once.
unexpected-argument = { $command } takes no argument { $argument }.

## One line of the report of what stops a build: the path, the line when
## there is one, and what is wrong.

problem = { $path }: { $message }
problem-with-line = { $path }:{ $line }: { $message }

## What is wrong with a file or folder of the input. Every $reason is the
## operating system's or a library's words.

input-cannot-be-listed = cannot be listed: { $reason }
input-cannot-be-read = cannot be read: { $reason }
input-syntax-error = Syntax error.
input-changed = changed while it was being read

## A block of a feature with no name, while the configuration's
## allow_unnamed is false. allow_unnamed is a key, never translated.

unnamed-feature = The feature has no name, and allow_unnamed is false.
unnamed-rule = The rule has no name, and allow_unnamed is false.
unnamed-scenario = The scenario has no name, and allow_unnamed is false.
unnamed-scenario-outline = The scenario outline has no name, and allow_unnamed is false.

## What is wrong with the configuration file. $reason is the operating
## system's words, and $key a key as the file writes it.

config-cannot-be-read = cannot be read: { $reason }
config-unknown-key = ghg has no setting named { $key }.
config-not-text = { $key } must be text, in quotes.
config-not-boolean = { $key } must be true or false.
config-empty = { $key } must not be empty.
