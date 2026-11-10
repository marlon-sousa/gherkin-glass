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
      --title <text>: the site's title, instead of the configuration's title.
      --version-name <name>: builds the site as the version of that name, instead of the configuration's version_name.

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

## A feature file whose page would be a folder's. $page is that path in the
## site, and $folder the folder's path below the input folder.

same-as-folder-page = has the same page, { $page }, as the page of the folder { $folder }.
same-as-home-page = has the same page, { $page }, as the home page.

## A folder whose page differs from an earlier folder's only in case. $page
## is its own page, and $first the earlier folder's path below the input folder.

same-folder-page = has the same page, { $page }, as the folder { $first }.
## A feature file whose page would be in the folder of the tag pages. $page
## is that page's path in the site; tags is the folder's name, never translated.

in-tags-folder = has its page, { $page }, in the folder tags, which holds the tag pages.

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
config-not-colour = { $key } must be a colour in hexadecimal, in quotes, such as "#0b4f9c".

## An accent colour that does not reach 7:1 against one of its theme's
## backgrounds. $colour and $background are colours in hexadecimal, such as
## #0b4f9c, and $ratio is the ratio reached, rounded down, such as 4.47.

config-accent-light-contrast = { $key } is { $colour }, which reaches only { $ratio }:1 against the light theme's background { $background }. It needs at least 7:1.
config-accent-dark-contrast = { $key } is { $colour }, which reaches only { $ratio }:1 against the dark theme's background { $background }. It needs at least 7:1.
config-not-table-list = tag_links must be a list of tables, each written [[tag_links]].
config-missing = { $key } is missing.
config-not-regex = { $key } is not a regular expression: { $reason }
config-regex-too-large = { $key } is too large a regular expression.
config-unknown-group = { $key } names the group { $group }, which the pattern does not have.

## A version name that cannot be one, given with --version-name or
## version_name. assets, index.html, con and nul are names, never translated.

invalid-version-name = { $name } cannot be a version name. A version name is made of letters, digits, dots, hyphens, underscores and plus signs, starts with a letter or a digit, does not end with a dot, and is not assets, index.html, or a name Windows keeps for a device, such as con or nul.

## What stops a build without a version, said of the output folder.
## replace_existing_version is a key, never translated.

site-already-built = a site is already built here, and replace_existing_version is false.
site-holds-versions = the output folder holds versions of a site, so build with a version name, or into another folder.
site-not-a-site = this is not a site ghg built, so it is left as it is. Build into an empty or new folder.

## What stops writing a build, said of its output folder or of a version's
## folder. Every $reason is the operating system's words.

output-cannot-be-read = cannot be read: { $reason }
output-cannot-be-written = cannot be written: { $reason }
output-cannot-be-listed = cannot be listed: { $reason }

## What stops a versioned build, said of the version's folder, or of the
## output folder. replace_existing_version is a key, never translated.

version-already-built = the version { $name } is already built, and replace_existing_version is false.
version-not-a-version = this is not a version ghg built, so it is left as it is.
version-holds-site = a site without versions is built here, so build without a version name, or into another folder.
version-other-capitals = a folder whose name differs from this one only in its capitals is in its place, so it is left as it is.

## A feature's page. $heading is the feature's keyword and name, and $path
## the feature file's path below the input folder, both as written.

page-title = { $heading }, { $path }
page-contents = Contents
collapse-all = Collapse all
expand-all = Expand all
source-link = Source: { $path }

## The control every page has to choose its theme, and its three choices.

theme = Theme
theme-system = System
theme-light = Light
theme-dark = Dark
## The name of every page's breadcrumb, a navigation landmark.

breadcrumb = Breadcrumb

## A folder's page. $folder is the folder's path below the input folder, and
## $site the site's title, the configuration's or default-title.

default-title = Feature files

## What ghg build adds, once the site is written, when neither the command
## line nor ghg.toml gives the site a title. title is a key, never translated.

no-title = The site has no title, so its home page is called Feature files. Set title in ghg.toml, or give --title.
folder-page-title = { $folder }, { $site }
folders = Folders
features = Features

## Counts, on the home page and beside each feature of a folder page.

folder-count =
    { $count ->
        [one] { $count } folder
       *[other] { $count } folders
    }
feature-count =
    { $count ->
        [one] { $count } feature
       *[other] { $count } features
    }
rule-count =
    { $count ->
        [one] { $count } rule
       *[other] { $count } rules
    }
scenario-count =
    { $count ->
        [one] { $count } scenario
       *[other] { $count } scenarios
    }

## The accessible name of the button that shows a feature's contents on a
## folder page. It starts with the words of scenario-count, which the button
## shows. $feature is the feature's heading, as written.

feature-scenarios =
    { $count ->
        [one] { $count } scenario of { $feature }
       *[other] { $count } scenarios of { $feature }
    }

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

## Before every page's content: the link past the folder tree, the checkbox
## that shows or hides it, and the name of its landmark.

skip-to-content = Skip to content
show-folder-tree = Show the folder tree
folder-tree = Folder tree

## How a scenario shows the steps of the backgrounds it starts with, and the
## checkbox that switches between showing them inline and linking to them.

from-background = From the background:
from-rule-background = From the rule's background:
inline-backgrounds = Show backgrounds inside each scenario
## The tag pages. $tag is a tag as written, with its @, $scenario a
## scenario's heading, and $count a number of scenarios.

tag-index-title = Tags
tag-index-entry = { $tag }, { $count ->
    [0] no scenarios
    [one] one scenario
   *[other] { $count } scenarios
}
tag-index-empty = No feature file has a tag.
tag-page-title = Tag { $tag }
tag-no-scenario = None of its scenarios carries the tag.
tag-through-examples = { $scenario }, through its examples
tag-through-rule = { $scenario }, through its rule
tag-through-feature = { $scenario }, through its feature
## The versions page, at the root of a versioned site. $name is a version's
## name, as given.

versions-title = Versions
versions-development = Development versions
versions-releases = Releases
versions-pre-releases = Pre-releases
versions-development-version = { $name }, a development version
