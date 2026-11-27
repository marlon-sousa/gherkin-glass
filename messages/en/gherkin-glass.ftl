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
      --front-page <file>: the Markdown template of the site's front page, instead of the configuration's front_page.
      --only <file>: builds the site from the feature files the file lists, one path on each line, alone. --only - reads the list from standard input.
      --preview: builds a preview, a site without versions whose every page asks search engines not to index it.
      --keep-in-output <pattern>: leaves alone what the pattern matches in the output folder, in the gitignore syntax, instead of the configuration's keep_in_output. Give it once for each pattern.
      --source-link <address>: the address of each feature file's source, with {"{"}path{"}"} and {"{"}line{"}"}, instead of the configuration's source_link.
      --allow-unnamed <true or false>: whether features, rules and scenarios may have no name, instead of the configuration's allow_unnamed.
      --collapse-rules <true or false>: whether rules start collapsed, instead of the configuration's collapse_rules.
      --collapse-scenarios <true or false>: whether scenarios and backgrounds start collapsed, instead of the configuration's collapse_scenarios.
      --show-tree <true or false>: whether the folder tree is shown when a reader first opens the site, instead of the configuration's show_tree.
      --inline-backgrounds <true or false>: whether pages start with the backgrounds inside each scenario expanded, instead of the configuration's inline_backgrounds.
      --replace-existing-version <true or false>: whether a build replaces the site or the version an earlier build wrote, instead of the configuration's replace_existing_version.
      --accent-light <colour>: the light theme's accent colour, such as "#0b4f9c", instead of the configuration's accent_light.
      --accent-dark <colour>: the dark theme's accent colour, such as "#8cc4ff", instead of the configuration's accent_dark.
      --tag-link <pattern> <address> <text>: links the tags the pattern matches to that address, with that text. Give it once for each link; the links given replace the configuration's tag_links.

    Options of ghg build only:
      --message-format <format>: human, the default, or json, which also prints one JSON object to standard output saying how the build ended, for a program to read.

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
not-a-message-format = The option --message-format takes human or json, not { $value }.
watch-without-build = The options --watch and --no-build cannot be used together.
only-without-build = The options --only and --no-build cannot be used together.
preview-with-version-name = The option --preview builds a site without versions, so it cannot be given with --version-name.

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

## A feature file whose page would be the page of step sentences. $page is
## that page's path in the site.

reserved-page = has the same page, { $page }, as the step sentences.

## A feature file whose page would be the page of search results. $page is
## that page's address, search.html.

search-page-clash = has the same page, { $page }, as the search results.

## A file of the site that cannot be written. $reason is the operating
## system's words.

cannot-be-written = cannot be written: { $reason }

## A warning after a build that succeeded. $size is the size of the search
## documents in millions of bytes, with one decimal, such as 6.3.

search-documents-large = The search documents are { $size } MB, larger than the 5 MB above which search can be slow to start, mostly on phones.

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
config-not-text-list = keep_in_output must be a list of text, each in quotes.
config-missing = { $key } is missing.
config-not-regex = { $key } is not a regular expression: { $reason }
config-regex-too-large = { $key } is too large a regular expression.
config-unknown-group = { $key } names the group { $group }, which the pattern does not have.

## A version name that cannot be one, given with --version-name or
## version_name. assets, index.html, ghg-versions.json, previews, con and nul are names,
## never translated.

invalid-kept-pattern = { $pattern } is not a pattern in the gitignore syntax: { $reason }
invalid-version-name = { $name } cannot be a version name. A version name is made of letters, digits, dots, hyphens, underscores and plus signs, starts with a letter or a digit, does not end with a dot, and is not assets, index.html, ghg-versions.json, previews, or a name Windows keeps for a device, such as con or nul.

## What stops a build without a version, said of the output folder.
## replace_existing_version is a key, never translated.

site-already-built = a site is already built here, and replace_existing_version is false.
site-holds-versions = the output folder holds versions of a site, so build with a version name, or into another folder.
site-not-a-site = this is not a site ghg built, so it is left as it is. Build into an empty or new folder.

## What stops writing a build, said of its output folder or of a version's
## folder. Every $reason is the operating system's words.

output-cannot-be-read = cannot be read: { $reason }
output-cannot-be-written = cannot be written: { $reason }
output-not-safe = ghg did not write this, so the output folder is left as it is.
output-kept-clash = the pattern { $pattern } keeps this, but ghg writes it or the folder it is in, so the output folder is left as it is.

## What stops a build writing its front page, said of the front page's
## template. $reason is the operating system's words.

front-page-cannot-be-read = cannot be read: { $reason }
front-page-no-title = has no level-one heading, written with one #, for the front page's title.
front-page-second-title = holds a second level-one heading, but a front page has one only.
front-page-no-marker = has no marker <!-- ghg:contents --> on a line of its own, where ghg puts its part of the front page.
front-page-second-marker = holds a second marker <!-- ghg:contents -->, but ghg puts its part of the front page in one place only.
front-page-marker-not-alone = the marker <!-- ghg:contents --> must be on a line of its own, outside any list, quote or table.
front-page-marker-too-deep = the marker <!-- ghg:contents --> follows a level 5 or level 6 heading, so ghg's own headings would have no level below it.
front-page-raw-html = raw HTML cannot be written in a front page; only the marker <!-- ghg:contents --> and other comments can.

## What stops a versioned build, said of the version's folder, or of the
## output folder. replace_existing_version is a key, never translated.

version-already-built = the version { $name } is already built, and replace_existing_version is false.
version-missing = the marker lists this version, but its folder is missing, so the output folder is left as it is.
version-holds-site = a site without versions is built here, so build without a version name, or into another folder.
version-other-capitals = the output folder holds a version whose name differs from this one only in its capitals, which some systems would put in the same folder, so it is left as it is.

## A feature's page. $heading is the feature's keyword and name, and $path
## the feature file's path below the input folder, both as written.

page-title = { $heading }, { $path }
page-contents = Contents
collapse-all = Collapse all
expand-all = Expand all
source-link = Source: { $path }

## The page an address that names no page of its shell draws, its title
## its $heading, then the $site's title, and the paragraph drawn in place of
## a page whose site's data cannot be fetched or read.

page-missing = Nothing is at this address
page-missing-title = { $heading }, { $site }
page-missing-text = This site has no page at this address.
page-not-loaded = This page could not be loaded. Load it again, or check that the site is served.

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

## What ghg build and ghg serve say when they read no feature file, so that
## they write nothing:
## $input is the input folder, and $list the list of --only, - for standard
## input.

nothing-to-build = { $input }: the input folder holds no feature file, so no site was built.
nothing-to-build-only = { $list }: names no feature file of the input, so no site was built.
nothing-to-build-no-path = { $list }: names no path, so there is nothing to build.

## What ghg serve --watch says when the site it serves was built from no
## feature file, $input and $list as above.

empty-site = { $input }: the input folder holds no feature file, so the site served shows none.
empty-site-only = { $list }: names no feature file of the input, so the site served shows none.
empty-site-no-path = { $list }: names no path, so the site served shows none.

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
serve-cannot-handle-signals = ghg serve cannot handle Control+C and the signals that stop it: { $reason }
serve-stopped = The site is no longer served.

## What ghg serve answers a browser with, as plain text.

serve-not-found = Nothing is at this address in the site.
serve-wrong-method = ghg serve answers only GET and HEAD requests.

## First in every page, the dialog opened only when the page is opened from
## disk: its name, what it says, and its one button, which closes it.

opened-from-disk-title = Opened from disk
opened-from-disk = This site is meant to be served, so its folder tree and search do not work when a page is opened from disk. Run ghg serve in the folder of your ghg.toml, or serve the site's folder with any web server, then open the address it gives.
opened-from-disk-agree = I agree

## Before every page's content: the link past the folder tree, the checkbox
## that shows or hides it, and the name of its landmark.

skip-to-content = Skip to content
show-folder-tree = Show the folder tree
folder-tree = Folder tree

## After every page's content: what built it. $name is gherkin-glass, a link
## to its repository, and $version the version of ghg, such as 0.1.0-alpha.2.

built-with = Built with { $name } { $version }

## First in every page's main content when the site was built with --only,
## and, on its home page, in place of its folders and features when none of
## the files --only lists is read.

only-note = This site shows only the feature files given with --only.
only-nothing-given = No feature file was given with --only.

## The heading of each background a scenario holds, the heading of the
## scenario's own steps after them, and the checkbox that chooses whether
## those or the backgrounds in their own place are expanded.

from-background = From the background
from-rule-background = From the rule's background
from-scenario = From the scenario
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

## A block's heading on a feature's page: its kind alone, or its kind and
## its $name. $kind is one of feature, background, rule, scenario,
## scenario-outline and examples, never shown; $name is as written in the
## file.

unnamed-heading =
    { $kind ->
        [feature] Feature
        [background] Background
        [rule] Rule
        [scenario-outline] Scenario outline
        [examples] Examples
       *[scenario] Scenario
    }
named-heading =
    { $kind ->
        [feature] Feature: { $name }
        [background] Background: { $name }
        [rule] Rule: { $name }
        [scenario-outline] Scenario outline: { $name }
        [examples] Examples: { $name }
       *[scenario] Scenario: { $name }
    }

## At the top of a block whose file writes its $kind, as unnamed-heading
## selects it, with another keyword: a note, whose $opener is a button that
## opens the dialog below, worded by keyword-note-opener with the $keyword as
## written in the file. The opener ends the sentence, its full stop included,
## so that nothing is left after the button, which wraps as one box, on a
## line of its own.

keyword-note =
    { $kind ->
        [feature] Note: in the Gherkin feature file, this feature is { $opener }
        [background] Note: in the Gherkin feature file, this background is { $opener }
        [rule] Note: in the Gherkin feature file, this rule is { $opener }
        [scenario-outline] Note: in the Gherkin feature file, this scenario outline is { $opener }
        [examples] Note: in the Gherkin feature file, these examples are { $opener }
       *[scenario] Note: in the Gherkin feature file, this scenario is { $opener }
    }
keyword-note-opener = defined with the keyword "{ $keyword }".

## The dialog that explains the note above, one on each page with such a
## note: its heading, its text and its one button.

note-dialog-title = About this note
note-dialog-text = Gherkin lets authors use different words for the same kind of block. For example, Scenario and Example both define a scenario, and Examples and Scenarios both define examples. This site names every block by its kind, so pages read the same whichever word was used. The note only tells you which word the feature file uses; it changes nothing about the block.
note-dialog-close = Got it

## An unnamed examples block, by its $place among its outline's blocks, from 1.

numbered-examples = Examples { $place }

## The block of the scenarios outside every rule, in a file with a rule.

outside-rules = Outside any rule

## After an outline's steps, before the list of its examples.

runs-for-each-row = Runs once for each row of these examples:

## An examples block with no row below its header, after its $heading.

nothing-to-run = { $heading } (nothing to run)

## In an examples block, the runs its rows produce, one per row. A
## produced scenario is named by its outline's $name and its row's $values, each
## value worded by example-value and separated by example-values-separator.
## $column and $value are as written.

runs-one-per-row = Runs, one per row
example-scenario-name = { $name } ({ $values })
example-value = { $column }: { $value }
example-values-separator = {", "}

## The lists of a feature's page, and of each of its blocks, in the other
## versions of a versioned site. $version is a version's name; $count is a
## number of blocks; $link is the link to a version, and $changes the counts
## of other-version-changed-blocks, other-version-not-there and
## other-version-only-there, joined by other-version-changes-separator.

other-versions-page = This page in other versions
other-versions-block = In other versions
other-versions-changed = Changed in
other-versions-missing = Not in
other-versions-unchanged = Unchanged in
other-version-newer = { $version }, newer
other-version-changes = { $link }: { $changes }
other-version-changed-blocks = { $count ->
    [one] 1 block changed
   *[other] { $count } blocks changed
}
other-version-not-there = { $count } not there
other-version-only-there = { $count } only there
other-version-changes-separator = {", "}

## The page of step sentences. $site is the site's title; $count is how many steps use a sentence;
## $heading is the heading of a step's block, and $path its feature file's
## path below the input folder, both as written; $link is the link that
## reads them, and $values the step's values, as written.

sentences-title = Step sentences
sentences-page-title = Step sentences, { $site }
sentences-given = Given
sentences-when = When
sentences-then = Then
sentences-none = No feature has a step.
sentence-uses = { $count ->
    [one] Used once.
   *[other] Used { $count } times.
}
sentence-use = { $heading }, { $path }
sentence-use-values = { $link }, with { $values }
sentence-values-separator = {", "}

## The search box on every page. The script puts what it found in place of
## each placeholder that is not $shown: $count is a number of results, $name,
## $feature, $rule and $outline the names of a block, its feature, its rule and
## its outline, $place an examples block's place among its outline's, from 1,
## and $words the words searched for.

search-label = Search features, rules and scenarios
search-loading = Loading search.
search-failed = Search could not be loaded.
search-none = No results.
search-count = Results: { $count }.
search-count-limited = Results: { $count }. The first { $shown } are listed.
# Stands in for the name of a block, a feature or a rule that has none, in
# the place of $name, $feature or $rule in the messages below.
search-no-name = no name
search-result-feature = { $name }, feature
search-result-rule = { $name }, rule in { $feature }
search-result-scenario = { $name }, scenario in { $feature }
search-result-scenario-in-rule = { $name }, scenario in { $rule }, { $feature }
search-result-scenario-outline = { $name }, scenario outline in { $feature }
search-result-scenario-outline-in-rule = { $name }, scenario outline in { $rule }, { $feature }
search-result-background = { $name }, background in { $feature }
search-result-background-in-rule = { $name }, background in { $rule }, { $feature }
search-result-examples = { $name }, examples in { $outline }, { $feature }
search-result-examples-in-rule = { $name }, examples in { $outline }, { $rule }, { $feature }
search-all-results = All results for "{ $words }"

## The page of search results. $site is the site's title. The script puts
## what it found in place of the other placeholders: $words are the words
## searched for, $count a number of results and $features of features,
## $blocks the headings of the blocks a result is in, each separated by
## search-page-separator.

search-page-heading = Search results
search-page-title = Search results, { $site }
search-page-heading-for = Results for "{ $words }"
search-page-title-for = Results for "{ $words }", { $site }
search-page-count = Results: { $count }. Features: { $features }.
search-page-within = In { $blocks }.
search-page-separator = {", "}
search-page-matching-steps = Matching steps:
search-page-matching-tags = Matching tags:
search-page-matching-cells = Matching cells:
