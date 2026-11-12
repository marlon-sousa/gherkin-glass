# 0073: A build starts from the trees of the last one

Decided by the maintainer on 2026-11-12, recorded in issues [#73](https://github.com/marlon-sousa/gherkin-glass/issues/73) and [#74](https://github.com/marlon-sousa/gherkin-glass/issues/74), and in decisions 22 to 27 of [#45](https://github.com/marlon-sousa/gherkin-glass/issues/45).

## Decision

gherkin-glass keeps two trees, and a build is the step from one pair of them to the next.

- **The input tree** holds the input as the walk found it: each folder with the rules of its `.ghg-ignore`, each file and folder marked allowed or not, and each allowed file with its revision and what parsing it gave.
- **The site tree** holds the site: the site, its versions, their folders and files, and each file's nodes, one typed kind each, with no type of the `gherkin` crate.
- **One `build`** is given where it starts: from nothing, with the settings, or from the last build, its trees and the site rendered from them, and the paths that changed since, in which case it reads the settings again only once the paths are found to concern what a build reads. It returns the new trees with the new site. `ghg build` calls it from nothing, and so does `ghg serve` first; each rebuild of `ghg serve --watch` calls it from the last build and the bursts waiting, merged, or from nothing when the configuration file changed or changes may have been missed. Starting from nothing is the first build's way and the full rebuild's, so both run the same code.

[ARCHITECTURE.md](../ARCHITECTURE.md), "The pipeline", describes the shape: "How a run flows" for `build` and `serve`, "The services" for how each stage updates its tree, and "The two trees" for the trees themselves.

Both trees are an `ego_tree::Tree` (ego-tree 0.11.1, ISC, allowed by `deny.toml`) whose nodes are one enum of typed kinds, stored by value, such as `SiteNode::Folder(Folder)` or `SiteNode::Scenario(ScenarioBlock)`, so that every node's data stays typed and keeps its derives. A node finds what it inherits through its ancestors, with `NodeRef::parent` (`src/lib.rs`, line 323) and `ancestors` (`src/iter.rs`, line 313). Which kind may sit under which is kept by one builder module per tree, whose typed adds are the only way to put a node in it.

Each build makes its trees anew from the last ones, copying over what it keeps, rather than changing the last trees in place. What parsing gave is held behind an `Arc`, so copying an input node copies no feature, and copying a kept page's nodes costs less than rendering it. ego-tree 0.11.1 has no way to free a node: `detach` (`src/lib.rs`, line 632) unlinks it from its parent and leaves it in the tree's `Vec`, so a tree changed in place for as long as a watch runs would keep every node it ever removed. Its derived `PartialEq` (`src/lib.rs`, line 51) also compares that whole `Vec`, the detached nodes and the order the nodes were added in included, so it cannot tell whether two trees hold the same site. Its `NodeRef`s compare by identity, the node's id and the tree's address (`src/lib.rs`, line 161), so the site writer compares a file node with the last site tree's by walking both with `traverse` (`src/iter.rs`, line 346) and comparing their values.

Every page and report stays the same, byte for byte, and BEHAVIOUR.md gains no rule. One message may change: when the configuration file names another input than the one `ghg serve` started with, a change in the folder still watched is outside the input the last build read, so it rebuilds nothing and `The site was rebuilt.` is no longer printed for it, which rule 65 allows, since no file of that build changed.

## How the code moves there

The work is built as a stack of layers, each keeping every page byte for byte, each saying `Refs #73` or `Refs #74` and reviewed before it opens, once this note has merged and once the layer that renames the gherkin parser and the test support (decisions 23 and 24 of #45) has merged:

1. the controller's tests build fakes of the driving traits instead of the real services (#73, item 3);
2. the site tree becomes one `ego_tree::Tree` of typed kinds, with its builder module, holding the kinds of today;
3. the top of the site tree becomes real: the site, its versions and their folders, with the versions page a node like any other page, and the folder pages, the folder tree's data and the breadcrumbs made from the folder nodes (#73, items 1 and 4);
4. a scenario's inherited tags and backgrounds are found among its ancestors instead of being copied in or passed down, and each need of the site stage keeps one entry point (#73, item 2);
5. the input tree comes in: the gherkin parser walks the input into it and parses into its file nodes, and the controller's one `build` starts from nothing, which is all `ghg build` and `ghg serve` need;
6. `serve` takes the order of decision 26 of #45: the first build, then the server, then the watch, with the bursts waiting merged and judged by walking the input from nothing, with the last build's input folder, into a new input tree compared with the last one before the settings are read again, and `Rebuilds` and the walk handed to the server gone, while every rebuild still starts from nothing;
7. a rebuild starts from the last build: the input tree and the site tree are made from the last ones and the paths, with the tests that compare, after every step, a rebuild from the last build with one from nothing (#74);
8. the site writer renders only the file nodes that changed, and keeps the others' files from the last site.

## What else was considered

- **Changing the last trees in place.** It would copy nothing, but ego-tree cannot free a removed node and compares trees by their hidden `Vec`, as the decision says. A tree made anew holds exactly its nodes, in file order.
- **Boxed trait objects in the tree, each with a `kind()` returning an enum that borrows the node.** They would keep each node typed too, but they lose the derives, `Clone` and `PartialEq`, which the tests and the site writer's comparison use, and add nothing while rendering stays out of the entities.
- **Keeping our own tree of typed structs, with a context passed down.** Each kind would still hold only its own children, checked by the compiler, but what a node needs from above would be handed down each level, and the folders would still be found from the pages' paths. The builder module keeps which kind sits under which instead, checked by a test.
- **Another tree crate.** indextree 4.9.2, rctree 0.6.0, orx-tree 2.2.0 and rowan 0.17.0 were read too, from their published sources, as [#73's comment of 2026-11-12](https://github.com/marlon-sousa/gherkin-glass/issues/73#issuecomment-6823412731) records with the files and lines. Each holds one type per tree, as ego-tree does. rowan's nodes hold only a kind and source text, with nowhere to keep an anchor or a link; indextree's nodes are reached only with their arena beside each call; and rctree's are shared cells, `Rc<RefCell<…>>`, borrowed as the program runs. ego-tree's `NodeRef` carries its tree, so a walk needs nothing beside it, its licence is allowed, and `scraper` 0.29.0 depends on it, as its manifest on crates.io says.
- **A file's modification time instead of a revision.** Some file systems store it coarsely, a checkout or a copy can set it to anything, and some editors keep it on save. The input tree counts its own revisions, so the converter's comparison is exact.
- **A separate tree of parse results.** Parsing gives exactly one result for each allowed file, so it fills in the file's node and makes no tree of its own.
- **Revisions alone to decide what to render again.** A file also changes when a tag page it links to is renamed, or when the folder whose page it is gains a feature or a folder, which no revision of that file says. The site writer compares each file node with the last site tree's instead, which catches every reason, since the converter puts into a file node's own nodes everything its file shows beyond what its path and the settings decide.
- **Reading the settings again for every burst.** Rule 65 says only a change that may change what a build reads has the configuration read again, so a burst is first judged against the last build's input tree, and a burst that concerns nothing reads, builds and says nothing, even while the configuration file is broken.
- **Watching from before the first build, as decision 25 of #45 has it.** It needs the gherkin parser's walk handed to the server as a function, so that the server can judge each burst. Starting the watch once the first build is done, as decision 26 says, needs no walk, since the input tree tells which changes matter; the server still merges the bursts waiting into one rebuild, as `Rebuilds` does today, so that rule 67's message and rule 69's numbers stay as they are. A change saved during the first build is then not seen, which no rule of BEHAVIOUR.md promises.

## Why

A rebuild today walks the whole input to decide whether to rebuild, then reads, parses, converts and renders every file, for a change to one. Keeping the trees between builds lets a change touch only what it concerns. Making the first build and a full rebuild the same build, from nothing, keeps one path through the code for both, and makes the check exact: after any change, a rebuild from the last build gives the same site, byte for byte, as one from nothing.

In the site tree, the hierarchy of folders is built once and held, instead of being worked out from the pages' paths three times, for the folder pages, the folder tree's data and the breadcrumbs. A node finds what it inherits by looking up, so nothing is copied into it or passed down as a flag.
