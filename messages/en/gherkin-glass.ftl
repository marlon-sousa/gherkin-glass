### gherkin-glass's own interface text, in English.
### docs/ARCHITECTURE.md, "Interface text", says how it is written and used.

## The command line.

help =
    ghg turns a folder of Gherkin feature files into an accessible website.

    Usage: ghg <command> [options]

    Commands:
      build: reads the feature files and writes the site, or reports every file that cannot be read.
      serve: builds the site and serves it on this machine, to preview it.

    Options of ghg build and ghg serve:
      --config <file>: the configuration file to read, instead of ghg.toml in the current folder.
      --input <folder>: the folder of feature files to read, instead of the configuration's input.
      --output <folder>: the folder to write the site into, instead of the configuration's output.

    Options of ghg serve only:
      --host <address>: the address to listen on, instead of 127.0.0.1.
      --port <number>: the port to listen at, instead of 8000. 0 lets the system choose one.
      --no-build: serves the output folder as it is, without building it.
      --watch: rebuilds the site when its files change, and reloads the pages open in a browser.

    Other options:
      --help, -h: prints this help.
      --version, -V: prints the version.

version = ghg { $version }

## A mistake on the command line. $command is "ghg", "ghg build" or "ghg serve".

missing-command = ghg needs a command, such as ghg build. ghg --help lists the commands.
unknown-command = ghg has no command named { $name }. ghg --help lists the commands.
unknown-option = { $command } has no option { $option }. ghg --help lists the options.
missing-value = The option { $option } needs a value.
unexpected-value = The option { $option } takes no value.
repeated-option = The option { $option } is given more than once.
unexpected-argument = { $command } takes no argument { $argument }.
not-a-port = The option --port needs a whole number from 0 to 65535, not { $value }.
watch-without-build = The options --watch and --no-build cannot be used together.

## One line of the report of what stops a build: the path, the line when
## there is one, and what is wrong.

problem = { $path }: { $message }
problem-with-line = { $path }:{ $line }: { $message }

## What is wrong with a file or folder of the input. Every $reason is the
## operating system's or a library's words.

input-missing = the input folder does not exist. Create it, or name another with input in ghg.toml or with --input.
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

## Two feature files whose pages would have the same path. $page is that
## path in the site, and $first the input path of the file that has it.

same-page = has the same page, { $page }, as { $first }.

## A file of the site that cannot be written. $reason is the operating
## system's words.

cannot-be-written = cannot be written: { $reason }

## What is wrong with the configuration file. $reason is the operating
## system's words, and $key a key as the file writes it.

config-cannot-be-read = cannot be read: { $reason }
config-unknown-key = ghg has no setting named { $key }.
config-not-text = { $key } must be text, in quotes.
config-not-boolean = { $key } must be true or false.
config-empty = { $key } must not be empty.

## A feature's page. $heading is the feature's keyword and name, and $path
## the feature file's path below the input folder, both as written.

page-title = { $heading }, { $path }
page-contents = Contents
collapse-all = Collapse all
expand-all = Expand all
source-link = Source: { $path }

## ghg serve. $address is the address to open in a browser, $host an
## address to listen on as it was given, $port a port number, $path a
## path, and $reason the operating system's or a library's words.

serve-listening = The site is served at { $address }. Press Control+C to stop.
serve-rebuilt = The site was rebuilt.
serve-not-rebuilt = The site was not rebuilt, so the last site that built is still served.
serve-port-taken = Port { $port } is already in use. Choose another with --port.
serve-cannot-listen = ghg serve cannot listen at { $host }, port { $port }: { $reason }
serve-cannot-watch = ghg serve cannot watch { $path } for changes: { $reason }

## What ghg serve answers a browser with, as plain text.

serve-not-found = Nothing is at this address in the site.
serve-wrong-method = ghg serve answers only GET and HEAD requests.
