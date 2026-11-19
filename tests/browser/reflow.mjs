// docs/BEHAVIOUR.md, "Accessibility", rule 221: a page fits a window 320 CSS
// pixels wide, measured as the page's width against the window's.

// The width the rule names, and a phone's height.
const NARROW = { width: 320, height: 844 };

// The page at a window 320 pixels wide, with what its main part hides shown,
// such as collapsed blocks, which a reader can expand: one line naming the
// page's width and the outermost elements past the window's edge when the
// page scrolls sideways, or none. The page is put back as it was, and the
// window given its own size back.
export async function reflowProblems(page) {
  const wide = page.viewportSize();
  await page.setViewportSize(NARROW);
  const measured = await page.evaluate(() => {
    const hidden = [...document.querySelectorAll("main [hidden]")];
    for (const element of hidden) element.hidden = false;
    const window = document.documentElement.clientWidth;
    const width = document.documentElement.scrollWidth;
    const past = [...document.body.querySelectorAll("*")]
      .filter((element) => element.getBoundingClientRect().right > window)
      .filter((element) => !element.parentElement.closest("pre"))
      .slice(0, 3)
      .map((element) => [element.tagName.toLowerCase(), ...element.classList].join("."));
    for (const element of hidden) element.hidden = true;
    return { window, width, past };
  });
  await page.setViewportSize(wide);
  if (measured.width <= measured.window) return [];
  return [
    `reflow: at ${NARROW.width} pixels the page is ${measured.width} wide, past the window: ${measured.past.join(", ")}`,
  ];
}
