# Data-first spike

A spike for issue #173, not part of gherkin-glass. Every page of a version is drawn by one script, `app.js`, from shared data: `data/store.json` holds each block once, under a hash of its content, and `data/versions/<version>.json` lists the blocks each version holds, with their pages, anchors and lines.

The content is the example suite of `project-site/features`, as two versions: `v1.0.0`, as it is, in `tools/features-v1`, and `v1.1.0`, in `tools/features-v2`, where one step changed, one scenario was renamed, one was added and one removed.

## Serving it

From the repository's root, with ghg installed:

```sh
ghg serve --no-build --output spike/data-first
```

Then open the address it prints. Its front page links both forms of address: `hash/`, where an address is `index.html#page#anchor`, and `query/`, where it is `index.html?page=page#anchor`.

## Making the data again

`tools/gen_data.py <work> <out>` reads both versions with the official Python parser, `gherkin-official` 29.0.0, from `<work>/spike-src/v1` and `<work>/spike-src/v2`, copies of `tools/features-v1` and `tools/features-v2`, and takes each block's anchor from the `assets/search.json` that ghg 1.1.0, built from `main`, writes for each into `<work>/spike-src/ghg-v1` and `<work>/spike-src/ghg-v2`. It writes `data/` into `<out>`.
