// The service of src/services/navigation.ts, with fakes of the shell, its address and the timer.

import assert from "node:assert/strict";
import { test } from "node:test";

import type { PageName } from "../../src/policies/address.ts";
import type { PageFrame } from "../../src/policies/page_frame.ts";
import { SETTLE_MILLISECONDS, startNavigation } from "../../src/services/navigation.ts";
import { FakeAddress, FakeCanvas, FakeTimer, settled } from "../support/shell_fakes.ts";

interface Started {
  readonly canvas: FakeCanvas;
  readonly address: FakeAddress;
  readonly timer: FakeTimer;
  readonly drawn: PageName[];
  readonly revealed: string[];
  failures: number;
}

/** A shell opened at `hash`, whose pages have the elements `ids` and draw with the frame titled by their kind, or fail when `failing` says so. */
async function open(hash: string, ids: readonly string[] = [], failing: (page: PageName) => boolean = () => false): Promise<Started> {
  const started: Started = { canvas: new FakeCanvas(), address: new FakeAddress(hash), timer: new FakeTimer(), drawn: [], revealed: [], failures: 0 };
  ids.forEach((id) => started.canvas.ids.add(id));
  await startNavigation({
    canvas: started.canvas,
    address: started.address,
    timer: started.timer,
    draw: async (page) => {
      if (failing(page)) {
        throw new Error("cannot be fetched");
      }
      return (): PageFrame => {
        started.drawn.push(page);
        started.canvas.done.push(`draw ${page.kind}`);
        return { title: `Title of ${page.kind}`, lang: "en", crumbs: [] };
      };
    },
    behave: () => (id) => void started.revealed.push(id),
    failure: "This page could not be loaded.",
    onFailure: () => {
      started.failures += 1;
    },
  });
  return started;
}

// Rule 240: opened at its address, the page is drawn, its title set, nothing said, and the focus moved only to the element its anchor names.
test("a shell opened at an address draws its page, sets its title, says nothing, and moves the focus only to its anchor", async () => {
  const atPage = await open("#billing/refunds");
  assert.deepEqual(atPage.canvas.done, ["busy true", "draw feature", "frame en", "busy false", "title Title of feature"]);
  assert.deepEqual(atPage.timer.waits, []);

  const atAnchor = await open("#billing/refunds#store-credit", ["store-credit"]);
  assert.deepEqual(atAnchor.revealed, ["store-credit"]);
  assert.equal(atAnchor.canvas.done.at(-1), "focus store-credit");
  assert.deepEqual(atAnchor.timer.waits, []);
});

// Rule 240: reached from another page, the title is set and said and the focus moved, 150 milliseconds after drawing.
test("a page reached from another is drawn, then, after the wait, titled, said and focused at its h1", async () => {
  const shell = await open("");
  shell.canvas.done.length = 0;
  shell.address.go("#billing/");
  await settled();
  assert.deepEqual(shell.canvas.done, ["focus main", "busy true", "draw folder", "frame en", "busy false"]);
  assert.deepEqual(shell.timer.waits, [SETTLE_MILLISECONDS]);
  shell.timer.pass();
  assert.deepEqual(shell.canvas.done.slice(5), ["title Title of folder", "announce Title of folder", "focus h1"]);
});

// Rules 239 and 240: an anchor the page has gets the focus; one it lacks leaves it at the h1.
test("a page reached at an anchor focuses the anchor's element, expanded, or its h1 when it has none", async () => {
  const shell = await open("", ["store-credit"]);
  shell.address.go("#billing/refunds#store-credit");
  await settled();
  shell.timer.pass();
  assert.deepEqual(shell.revealed, ["store-credit"]);
  assert.equal(shell.canvas.done.at(-1), "focus store-credit");

  shell.address.go("#billing/archive/old#nowhere");
  await settled();
  shell.timer.pass();
  assert.equal(shell.canvas.done.at(-1), "focus h1");
});

// Rule 240: a link to another anchor of the page shown draws nothing again, and moves the focus at once.
test("another anchor of the page shown is focused at once, with nothing drawn again", async () => {
  const shell = await open("#billing/refunds", ["background"]);
  shell.address.go("#billing/refunds#background");
  await settled();
  assert.equal(shell.drawn.length, 1);
  assert.equal(shell.canvas.done.at(-1), "focus background");
  assert.deepEqual(shell.timer.waits, []);
});

// Rule 236: `##` names an element of the page shown, draws nothing, and leaves the address as it was.
test("an element of the page shown is focused, the address left naming the page", async () => {
  const shell = await open("#billing/refunds", ["ghg_main"]);
  shell.address.go("##ghg_main");
  await settled();
  assert.equal(shell.drawn.length, 1);
  assert.equal(shell.canvas.done.at(-1), "focus ghg_main");
  assert.deepEqual(shell.address.replaced, ["#billing/refunds"]);
});

// Rule 236: opened at `##`, the home page is drawn first.
test("a shell opened at an element's address draws the home page first", async () => {
  const shell = await open("##ghg_main");
  assert.deepEqual(shell.drawn, [{ kind: "home" }]);
});

// Rule 243: a page whose data cannot be fetched says so, and disables what needs the data.
test("a page whose data cannot be fetched draws the paragraph that says so", async () => {
  const shell = await open("#billing/refunds", [], (page) => page.kind === "feature");
  assert.deepEqual(shell.canvas.done, ["busy true", "failure This page could not be loaded.", "busy false"]);
  assert.equal(shell.failures, 1);
});

// Rule 240: of two pages asked for in turn, only the last is drawn, even when the first one's data comes last.
test("a page left before its data came is not drawn", async () => {
  let release: () => void = () => {};
  const canvas = new FakeCanvas();
  const address = new FakeAddress("");
  const drawn: string[] = [];
  await startNavigation({
    canvas,
    address,
    timer: new FakeTimer(),
    draw: async (page) => {
      if (page.kind === "folder") {
        await new Promise<void>((done) => {
          release = done;
        });
      }
      return () => {
        drawn.push(page.kind);
        return { title: page.kind, lang: "en", crumbs: [] };
      };
    },
    behave: () => () => {},
    failure: "",
    onFailure: () => {},
  });
  address.go("#billing/");
  address.go("#tags/");
  await settled();
  release();
  await settled();
  assert.deepEqual(drawn, ["home", "tags"]);
});
