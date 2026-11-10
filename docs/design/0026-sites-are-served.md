# 0026: Sites are served, not opened from disk

Decided by the maintainer on 2026-11-10, recorded in issue [#26](https://github.com/marlon-sousa/gherkin-glass/issues/26).

## Decision

A site gherkin-glass builds is meant to be served over HTTP: by any static web server or host, GitHub Pages included, or by `ghg serve` to preview it on the reader's own machine. Opening its pages straight from disk, with a `file://` address, is not supported.

## What else was considered

Supporting both. A page opened from disk cannot load what a served page can, at least in Chromium: there, tried through Playwright on 2026-11-10, both `fetch` and `XMLHttpRequest` of a JSON file beside the page fail on `file://`, and an ES module is blocked. Supporting disk meant writing every script, and every piece of shared data such as the folder tree's and the search index, as classic scripts, and keeping search to an index that loads whole.

## Why

The tool's sites are published, and a published site is served. Dropping disk support lets the pages' script be an ES module, lets shared data be JSON fetched when needed, and opens search to libraries that fetch their index in pieces, such as Pagefind. `ghg serve` covers the one case disk support was for: looking at a site before publishing it, with nothing else installed.

The cost is that a site downloaded as a CI artifact, or sent as a zip, has to be served to work fully, with `ghg serve` or any other server.
