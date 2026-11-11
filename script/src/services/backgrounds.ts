// Service: showing the backgrounds inside each scenario or linking to them, as the reader chose for the site.

import { formKey, rememberedForm, startForm } from "../policies/background_form.ts";
import type { Backgrounds } from "../ports/backgrounds.ts";
import type { Memory } from "../ports/memory.ts";

export function startBackgrounds(backgrounds: Backgrounds, memory: Memory, siteRoot: string): void {
  const key = formKey(siteRoot);
  // Another page of the site may have changed the choice while this one was left.
  const show = (): void => backgrounds.setInline(startForm(backgrounds.writtenInline(), memory.get(key)));
  show();
  backgrounds.onSwitch((inline) => {
    backgrounds.setInline(inline);
    memory.set(key, rememberedForm(inline));
  });
  backgrounds.onReturn(show);
}
