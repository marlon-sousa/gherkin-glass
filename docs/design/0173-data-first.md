# 0173: Data first

Decided by the maintainer between 2026-11-25 and 2026-11-27, recorded in issue [#173](https://github.com/marlon-sousa/gherkin-glass/issues/173), whose body holds every point under "Agreed", and in its comments "Decided so far". It is gherkin-glass 2.0. [#167](https://github.com/marlon-sousa/gherkin-glass/issues/167), [#168](https://github.com/marlon-sousa/gherkin-glass/issues/168) and [#169](https://github.com/marlon-sousa/gherkin-glass/issues/169) move into it.

## Decision

A site stores each feature file's content once, shared by every version and preview that has it, and the pages' script draws every page from that store, at addresses after a `#`. [BEHAVIOUR.md](../BEHAVIOUR.md) says what this promises: "Addresses", "How a page is drawn" and "The site's data", rules 235 to 256, with the commands of "Versions" and "A preview".

- **Addresses after a hash.** `v1.0.0/#catalogue/searching#searching-by-title`: the shell of a version, then the page, then the anchor. The anchor rule does not change. `##` names an element of the page shown, such as `##ghg_main`.
- **One shell for each version, preview and site without versions**, its `index.html`, from which the script draws every page but the front page. The versions page stays a whole page of HTML, and the front page of a site without versions arrives in its shell's `main`.
- **One store**, `data/`, at the output folder's root: an object for each feature file's content, named by the SHA-256 of its bytes; a descriptor for each version, preview or site; the list of versions; and the index of pages across versions. Each build removes what no descriptor references.
- **One format number for the whole store**, converted forward as ghg builds, never read by an older ghg.
- **One marker**, `ghg-site.json`, for every output folder; what it holds follows from its descriptors. An output folder of gherkin-glass 1 is refused, and the first release of 2.0 rebuilds the project site's `gh-pages`, adding again every version it held, each from its own tag, with ghg 2.
- **Commands for versions and previews**: `ghg version add` and `remove`, and `ghg preview add` and `remove`. `ghg build` always builds a site without versions. `version_name`, `--version-name` and `--preview` go away.
- **Only real clashes stop a build.** A page is a name, not a file, so the clashes that came from files, `index.feature` against a folder's page and names that differ only in case, go away.

## Why

Measured on the 22 tags and `main` of [itinera-dev/conformance](https://github.com/itinera-dev/conformance), each built as a version with ghg 1.1.0 into one output folder: 63.5 MB in 3,097 files, of which 490 files, 26.9 MB, are distinct. The step sentences page alone was 18.0 MB across versions. Most versions hold the same feature files, so most pages repeat another version's. Stored once, the content is 1.55 MB in 179 objects for all 23 versions, and a page can say what changed in other versions, which pages written once and alone cannot.

## The architecture

The architecture of 2.0 below is what the layers that build it work to. [ARCHITECTURE.md](../ARCHITECTURE.md) describes the code as it is: each layer brings it up to date with what that layer adds or removes, so that it never describes code that is not there.

### A store in place of the site tree

The collector, the gherkin parser and the input tree stay as they are. The converter makes a store entity in place of the site tree, holding:

- the objects, one for each feature file's content, made from the parsed feature with its blocks' hashes and anchors and its steps' kinds, sentences, slots and highlighted parts;
- the descriptor, with the features and tags sorted and the settings' results worked out;
- the few files Rust still writes as HTML, the shells, the versions page and the front page of a site without versions, with `anchors.json` of #168.

Rendering becomes serde to JSON for the store, and Askama only for those few pages. ghg's rules, anchors, slots, sentences and highlighting, stay in Rust as policies, now feeding objects. The views move to the script. Content addressing replaces the revisions with which `--watch` converts again only what changed: an unchanged file's object is kept from the last build by its hash.

Keeping the site tree and rendering its feature nodes to JSON was set aside, since it would carry page and view nodes nothing renders, and revisions the hashes already give.

### What counts as identical

There are two comparisons, kept apart.

1. **The same object, for storage.** Two objects are the same file when their bytes are the same. So ghg always writes the same bytes for the same content: serde to compact JSON, structs only, whose members keep a fixed order, lists in file order, no unsorted maps and no numbers with decimals. This decides only what is stored once, never what a reader is told.
2. **The same block or page, for readers.** Unchanged is decided by the blocks' hashes, compared at the same address, never by the objects' names, as BEHAVIOUR.md, rules 251 and 261, says. So a later ghg of the same format that writes a different object for the same file, such as with a member added, makes a new object while readers are still told, rightly, that nothing changed. The script never hashes: it only compares the hashes ghg wrote.

The crate that computes SHA-256, such as `sha2`, is to be checked at the version to be pinned before it is chosen, as AGENTS.md, rule 8, asks.

### The script

The script keeps its roles and uses no framework:

- **policies**: the address, read into a typed address and written back, and one view for each kind of page, worked out from the descriptor and the objects: the feature page, the folder page, the tag pages, the grouping of the step sentences, the totals, the search documents and the comparison across versions of #169;
- **a port and adapter for the store**, fetching descriptors, objects, `versions.json` and `pages.json` at addresses resolved against the script's own, keeping each object for the rest of the visit by its hash, since objects never change once written;
- **one drawing adapter for each kind of page**, turning a view into the elements today's templates write, with the same classes, through a small typed element builder, `createElement` and `textContent`, never `innerHTML`, so that rule 203 holds by construction ([#183](https://github.com/marlon-sousa/gherkin-glass/issues/183) later wrote these adapters in JSX, against a runtime of the script's own that keeps the same guarantee);
- **a navigation service**, which reads the address, has the page drawn and applies the rule of the focus and the announcement, rule 240; today's services, collapsing, the folder tree, search, the theme and the note's dialog, attach to each page as it is drawn.

No framework, since a page is drawn and replaced whole, the focus must stay the script's to move, nothing is added to keep up with, and the element builder rules out inserting HTML. The maintainer's condition: this is kept under rigorous evaluation, and if the script grows too complex, external libraries are weighed, checked at the version they would be used.

### The site writer

- **`check`**, still before the input is read, also reads the published store's descriptors through the `OutputFolders` port, and the objects' names from the listing it already makes, never their content. A store in a newer format stops there.
- **A pure policy, the store plan**, decides the whole write from what `check` found and the new descriptor: the objects to write and to remove, the same for the folders `assets/<ghg version>/`, the new `versions.json` and `pages.json`, and the new marker. Being pure, it is what the property test exercises at every step.
- **`write` follows the plan in a safe order**: the marker first, listing every file about to exist, then the new objects, the shells, the descriptor and the derived files, and the removals last, so that an interrupted build leaves only files the marker lists, and the next build finishes the job.
- **A conversion** is a chain of pure policies, one for each step from a format to the next, which the site writer runs on the whole store read into memory; the same plan then writes the result.
- **Under `--watch`**, with memory as the destination, nothing changes: one descriptor and its objects, nothing published to read and nothing removed.

`ghg version add`, `ghg version remove`, `ghg preview add` and `ghg preview remove` are subcommands of clap 4.6.7, the version pinned, which has nested subcommands as it has top-level ones; `Command::subcommand_required` (clap_builder 4.6.7, `src/builder/command.rs`, line 3153) makes `ghg version` alone a mistake.

## What else was considered

- **A query, `v1.0.0/?page=catalogue/searching#anchor`**, in place of the hash. Set aside once a spike showed the hash form working with the maintainer's screen reader.
- **A small file at every old address**, for compatibility with 1.x. Set aside: no one publishes with ghg 1 but the project site.
- **Step sentences as objects of their own.** On itinera's `main`, 2,686 step uses have 310 sentences, a step's text is 35 bytes at the median, shorter than a hash, and one file per sentence would cost the step sentences page 310 requests. So each step carries its sentence.
- **A store folder named `_data`.** A host that runs Jekyll leaves out folders starting with `_` unless the site has `.nojekyll`.
- **Starting the project site afresh at 2.0, holding only its new version.** Set aside for rebuilding every 1.x release with ghg 2, so that the site still shows every release, though each as 2.0 draws its suite, not as its own ghg drew it; every old address breaks either way.
- **Keeping a 1.x folder's versions as frozen versions** beside the 2.0 ones, listed on the versions page but never converted. Set aside: 2.0 breaks compatibility, and its README shows how to rebuild a versioned site instead, by adding each tag again with `ghg version add`.

## Measured

- The pages were tried before the plumbing, on the branch `claude/project-thread-mgl7m3`, folder `spike/data-first`, which is never to be merged: the example suite's home, folder and feature pages as two versions sharing their blocks, in the markup and stylesheet of ghg 1.1.0. axe-core 4.14.0 found no violation on its 36 pages in either colour scheme, and no page was wider than 320 pixels.
- The maintainer's screen reader lost its place when the focus moved as soon as a page was drawn, and kept it when the focus stayed in `main` and moved 150 milliseconds after drawing. The wait was measured with that one screen reader and browser, and is to be checked with others.
