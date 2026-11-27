// Adapter: the lists of a page, or of a block, in other versions, drawn from their view as disclosures, each a button that shows or hides what follows it.

import type { ElementBuilder } from "./element_builder.ts";
import type { OtherVersionsView, VersionLink } from "../policies/other_versions.ts";

export function otherVersionsOf(make: ElementBuilder, view: OtherVersionsView): HTMLElement {
  return disclosure(
    make,
    "ghg-other-versions",
    view.label,
    make.make("div", {}, ...view.groups.map((group) => disclosure(make, "ghg-version-group", group.label, make.make("ul", { class: "ghg-versions" }, ...group.links.map((link) => linkItem(make, link)))))),
  );
}

/** A button, closed, that shows `shown` when pressed and hides it when pressed again; its state is not remembered. */
function disclosure(make: ElementBuilder, className: string, label: string, shown: HTMLElement): HTMLElement {
  const button = make.make("button", { type: "button", class: "ghg-toggle", "aria-expanded": "false" }, label);
  shown.classList.add("ghg-content");
  shown.hidden = true;
  button.addEventListener("click", () => {
    const opened = button.getAttribute("aria-expanded") !== "true";
    button.setAttribute("aria-expanded", String(opened));
    shown.hidden = !opened;
  });
  return make.make("div", { class: className }, button, shown);
}

function linkItem(make: ElementBuilder, link: VersionLink): HTMLLIElement {
  return make.make("li", {}, ...link.item.map((piece) => ("slot" in piece ? make.make("a", { href: link.address }, link.text) : piece.words)));
}
