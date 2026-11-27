// The drawing of src/adapters/jsx/drawing.ts, of markup written with the runtime of src/adapters/jsx/jsx_runtime.ts.

import assert from "node:assert/strict";
import { test } from "node:test";

import { drawn } from "../../../src/adapters/jsx/drawing.ts";
import type { Child } from "../../../src/adapters/jsx/jsx_runtime.ts";
import { emptyDocument } from "../../support/drawn_pages.ts";

function markupOf(child: Child): string {
  const document = emptyDocument();
  const holder = document.createElement("div");
  holder.append(...drawn(document, child));
  return holder.innerHTML;
}

test("text is drawn as text, never read as markup", () => {
  assert.equal(markupOf(<p>{"<b>bold</b> & <script>alert(1)</script>"}</p>), "<p>&lt;b&gt;bold&lt;/b&gt; &amp; &lt;script&gt;alert(1)&lt;/script&gt;</p>");
});

test("an attribute's text is set as it is, and true sets one with no value", () => {
  assert.equal(markupOf(<a href={'"><script>'} class="ghg-x" hidden>x</a>), '<a href="&quot;><script>" class="ghg-x" hidden="">x</a>');
});

test("attributes are set in the order they are written", () => {
  assert.equal(markupOf(<button type="button" class="ghg-toggle" aria-expanded="false" />), '<button type="button" class="ghg-toggle" aria-expanded="false"></button>');
});

test("false and null set no attribute and draw no child", () => {
  const nothing = (shown: boolean): Child => (
    <div hidden={shown} lang={null} aria-current={shown && "page"}>
      {null}
      {shown && "shown"}
    </div>
  );
  assert.equal(markupOf(nothing(false)), "<div></div>");
});

test("a list draws each of its items, and a fragment its children in its place", () => {
  assert.equal(markupOf(<ul>{["a", "b"].map((name) => <li>{name}</li>)}<>{"c"}<br /></></ul>), "<ul><li>a</li><li>b</li>c<br></ul>");
});

test("a node of the document is moved into the element drawn", () => {
  const document = emptyDocument();
  const form = document.createElement("form");
  document.body.append(form);
  const [section] = drawn(document, <div>{form}</div>);
  assert.equal(section?.firstChild, form);
  assert.equal(document.body.querySelector(":scope > form"), null);
});

test("text written in a template keeps the spaces and line feeds written as text", () => {
  assert.equal(markupOf(<p>{"a"}{" "}{"\n"}b</p>), "<p>a \nb</p>");
});
