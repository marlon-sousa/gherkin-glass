// Adapter: the markup of a folder's page, drawn from its view, in the elements and classes the stylesheet styles.

import { contentsList } from "./dom_feature_page.tsx";
import type { Markup } from "./jsx/jsx_runtime.ts";
import { MessageParts } from "./jsx/message_parts.tsx";
import type { FolderFeatureView, FolderView, LinkView } from "../policies/folder_view.ts";

/** The content of a folder's `main`. */
export function folderPage(view: FolderView): Markup {
  return (
    <>
      <h1>{view.heading}</h1>
      {view.totals.length > 0 && (
        <ul class="ghg-totals">
          {view.totals.map((total) => (
            <li>{total}</li>
          ))}
        </ul>
      )}
      {view.tagIndex && <p class="ghg-tag-index-link">{link(view.tagIndex)}</p>}
      {view.sentences && <p class="ghg-sentences">{link(view.sentences)}</p>}
      {view.nothingGiven && <p class="ghg-nothing-given">{view.nothingGiven}</p>}
      {view.folders && [
        <h2>{view.folders.heading}</h2>,
        <ul class="ghg-folders">
          {view.folders.links.map((folder) => (
            <li>{link(folder)}</li>
          ))}
        </ul>,
      ]}
      {view.features && [<h2>{view.features.heading}</h2>, <ul class="ghg-features">{view.features.entries.map(featureEntry)}</ul>]}
    </>
  );
}

function link(view: LinkView): Markup {
  return <a href={view.address}>{view.text}</a>;
}

function featureEntry(feature: FolderFeatureView): Markup {
  const heading = (
    <a href={feature.address}>
      <MessageParts parts={feature.heading} lang={feature.lang} />
    </a>
  );
  if (feature.countName === null) {
    return (
      <li class="ghg-feature">
        {heading}
        {"\n"}
        <span class="ghg-count">{feature.count}</span>
      </li>
    );
  }
  return (
    <li class="ghg-feature">
      {heading}
      {"\n"}
      <button type="button" class="ghg-toggle" aria-expanded="false" aria-label={feature.countName}>
        {feature.count}
      </button>
      <div class="ghg-content" hidden>
        {contentsList(feature.contents, feature.lang)}
      </div>
    </li>
  );
}
