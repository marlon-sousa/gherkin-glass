# 0045: The code follows the pipeline, as a compiler's does

Decided by the maintainer on 2026-11-11 and 2026-11-12, recorded in issue [#45](https://github.com/marlon-sousa/gherkin-glass/issues/45).

## Decision

gherkin-glass collects its feature files already parsed, converts them into one tree of the site, and writes the site by walking the tree. [ARCHITECTURE.md](../ARCHITECTURE.md), "The pipeline", describes the shape:

- handlers, the ways in, call one controller, the orchestrator;
- the controller runs four services in order: the settings service, the gherkin parser, the converter and the site writer;
- each node of the tree renders its own piece through one trait, beside its template.

The code is moved to this shape layer by layer, and every layer keeps every page byte for byte, as the fixtures' and the example suite's committed sites check.

## What else was considered

- **The stages as the top folders, each with its own roles.** That would show the pipeline first, but it would spread each role across several places and undo the folders by role, then by area. Instead the roles stay on top, and the stages are the areas of `entities/` and `policies/`.
- **A configuration crate to merge the file and the arguments.** figment 0.10.19 and config 0.15.27 were read. Neither gives an error's line, and both stop at the first problem, since both deserialise through serde. BEHAVIOUR.md rule 26 asks for every problem with its line, so the file is still checked key by key.
- **The sequence of stages in a service that the watch calls.** This was set aside for handlers. The watch is a way in like the command line, so it calls the controller. Nothing calls back up a layer, and a rebuild reads the configuration again for free.
- **One template for each kind of page, walking plain data.** This was set aside for nodes. Today's single block type, with fields that only some kinds use, is the loose shape that a type for each node replaces.
- **The input walk inside its adapter.** The walk decided which folders to enter, with the `.ghg-ignore` policy. It moves into the gherkin parser, behind a port that only lists a folder and reads a file, so that the adapter decides nothing.
- **The `gherkin` crate's types in the tree.** Keeping them would spread the crate across every pass. They stop at the converter's first pass instead.

## Why

The tool turns files into files in fixed stages, as a compiler turns sources into output. Today the stages are there, but the site is a set of flat lists. One function names every kind of page and calls each one's rendering by hand. One tree, rendered through one trait, lets a new kind of page add one implementation without changing any service. One type for each node lets the types hold the constraints that comments hold today.
