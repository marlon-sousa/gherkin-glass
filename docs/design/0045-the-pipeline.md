# 0045: The code follows the pipeline, as a compiler's does

Decided by the maintainer on 2026-11-11 and 2026-11-12, recorded in issue [#45](https://github.com/marlon-sousa/gherkin-glass/issues/45).

## Decision

gherkin-glass collects its feature files already parsed, converts them into one tree of the site, and writes the site by walking the tree. [ARCHITECTURE.md](../ARCHITECTURE.md), "The pipeline", describes the shape:

- handlers, the ways in, call one controller, the orchestrator;
- the controller runs the settings service, the gherkin parser, the converter and the site writer in order, and `serve` adds the server;
- each node of the tree renders its own piece through one trait, beside its template.

The code is moved to this shape layer by layer, and every layer keeps every page byte for byte, as the fixtures' and the example suite's committed sites check.

## How the code moves there

The redesign is built as a stack of six layers, each saying `Refs #45` and reviewed before it opens, once this note has merged:

1. the input walk moves into the gherkin parser, behind the `InputFolders` port;
2. the handler and the controller split, and the server service comes in, with its channel and the handler's listener;
3. the destination goes into the settings, with the `FileWriter` port and its two adapters;
4. the tree's node types come in, and the converter produces them, still feeding today's templates;
5. the rendering trait comes in over the nodes, and the templates move beside them;
6. the folders move into their stages.

An intermediate layer of this stack that cannot keep a test passing may disable it for its CI to run, naming it in its description; the last layer enables every such test again, and has every test passing.

## What else was considered

- **The stages as the top folders, each with its own roles.** That would show the pipeline first, but it would spread each role across several places and undo the folders by role, then by area. Instead the roles stay on top, and the stages are the areas of `entities/` and `policies/`.
- **A configuration crate to merge the file and the arguments.** figment 0.10.19 and config 0.15.27 were read. Neither gives an error's line: figment's `Error` and `Metadata` hold no position (its `src/error.rs`, line 89, and `src/metadata.rs`), nor does config's `ConfigError` (its `src/error.rs`, line 65), and both stop at the first problem, since both deserialise through serde. BEHAVIOUR.md rule 26 asks for every problem with its line, so the file is still checked key by key.
- **The sequence of stages in a service that the watch calls.** This was set aside for one controller, the orchestrator, called by the handlers. The server service hands the controller a channel of the changes that call for a rebuild, and the controller calls its own `build` for each one. Nothing calls back up a layer, and a rebuild reads the configuration again for free.
- **One template for each kind of page, walking plain data.** This was set aside for nodes. Today's single block type, with fields that only some kinds use, is the loose shape that a type for each node replaces.
- **The input walk inside its adapter.** The walk decided which folders to enter, with the `.ghg-ignore` policy. It moves into the gherkin parser, behind a port that only lists a folder and reads a file, so that the adapter decides nothing.
- **The `gherkin` crate's types in the tree.** Keeping them would spread the crate across every pass. They stop at the converter's first pass instead.

## Why

The tool turns files into files in fixed stages, as a compiler turns sources into output. Today the stages are there, but the site is a set of flat lists. One function names every kind of page and calls each one's rendering by hand. One tree, rendered through one trait, lets a new kind of page add one implementation without changing any service. One type for each node lets the types hold the constraints that comments hold today.
