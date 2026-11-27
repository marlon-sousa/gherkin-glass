// Adapter: words of the catalogue and of feature text drawn as text, each isolated part in a `bdi` of its own.

import type { Markup } from "./jsx_runtime.ts";
import type { Parts } from "../../ports/words.ts";

/** `lang` is given to each isolated part's `bdi`, when it is given. */
export function Words({ parts, lang = null }: { readonly parts: Parts; readonly lang?: string | null }): Markup {
  return <>{parts.map((part) => (part.isolated ? <bdi lang={lang}>{part.text}</bdi> : part.text))}</>;
}
