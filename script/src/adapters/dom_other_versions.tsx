// Adapter: the lists of a page, or of a block, in other versions, drawn from their view as disclosures, each a button that shows or hides what follows it.

import type { Markup } from "./jsx/jsx_runtime.ts";
import type { OtherVersionsView, VersionLink } from "../policies/other_versions.ts";

export function otherVersionsOf(view: OtherVersionsView): Markup {
  return disclosure(
    "ghg-other-versions",
    view.label,
    <div class="ghg-content" hidden>
      {view.groups.map((group) =>
        disclosure(
          "ghg-version-group",
          group.label,
          <ul class="ghg-versions ghg-content" hidden>
            {group.links.map(linkItem)}
          </ul>,
        ),
      )}
    </div>,
  );
}

/** Has each disclosure drawn in `root` show what follows its button when pressed, and hide it when pressed again; its state is not remembered. */
export function openOtherVersions(root: ParentNode): void {
  for (const button of root.querySelectorAll<HTMLButtonElement>(".ghg-other-versions > button.ghg-toggle, .ghg-version-group > button.ghg-toggle")) {
    const shown = button.parentElement?.querySelector<HTMLElement>(":scope > .ghg-content");
    button.addEventListener("click", () => {
      const opened = button.getAttribute("aria-expanded") !== "true";
      button.setAttribute("aria-expanded", String(opened));
      if (shown) {
        shown.hidden = !opened;
      }
    });
  }
}

/** A button, closed, followed by `shown`, which must be written hidden and with the class `ghg-content`, by which `openOtherVersions` finds it. */
function disclosure(className: string, label: string, shown: Markup): Markup {
  return (
    <div class={className}>
      <button type="button" class="ghg-toggle" aria-expanded="false">
        {label}
      </button>
      {shown}
    </div>
  );
}

function linkItem(link: VersionLink): Markup {
  return <li>{link.item.map((piece) => ("slot" in piece ? <a href={link.address}>{link.text}</a> : piece.words))}</li>;
}
