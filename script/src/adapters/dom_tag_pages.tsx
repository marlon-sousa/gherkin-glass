// Adapter: the markup of the index of tags and of a tag's page, drawn from their views, in the elements and classes the stylesheet styles.

import type { Child, Markup } from "./jsx/jsx_runtime.ts";
import { Words } from "./jsx/words.tsx";
import type { LinkInWords, TagFeatureView, TagIndexView, TagPageView } from "../policies/tag_views.ts";

export function tagIndex(view: TagIndexView): Markup {
  return (
    <>
      <h1>{view.title}</h1>
      {view.entries === null ? (
        <p>{view.empty}</p>
      ) : (
        <ul class="ghg-tag-index">
          {view.entries.map((entry) => (
            <li>{linkInWords(entry, <Words parts={entry.words} />)}</li>
          ))}
        </ul>
      )}
    </>
  );
}

export function tagPage(view: TagPageView): Markup {
  return (
    <>
      <h1>
        <Words parts={view.heading} />
      </h1>
      {view.outside && (
        <p>
          <a class="ghg-tag-link" href={view.outside.address}>
            {view.outside.text}
          </a>
        </p>
      )}
      {view.features.map(tagFeature)}
    </>
  );
}

function tagFeature(feature: TagFeatureView): Markup {
  return (
    <>
      <h2>
        <a href={feature.address}>
          <Words parts={feature.before} lang={feature.lang} />
          <span class="ghg-tag-path">{feature.path}</span>
          <Words parts={feature.after} />
        </a>
      </h2>
      {feature.scenarios === null ? (
        <p>{feature.none}</p>
      ) : (
        <ul class="ghg-tag-scenarios">
          {feature.scenarios.map((scenario) => (
            <li>
              {linkInWords(
                scenario,
                <bdi>
                  <Words parts={scenario.words} lang={feature.lang} />
                </bdi>,
              )}
            </li>
          ))}
        </ul>
      )}
    </>
  );
}

/** The words before the link, the link holding `inner`, and the words after it. */
function linkInWords(link: LinkInWords, inner: Child): Markup {
  return (
    <>
      <Words parts={link.before} />
      <a href={link.address}>{inner}</a>
      <Words parts={link.after} />
    </>
  );
}
