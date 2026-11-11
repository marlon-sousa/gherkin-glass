// The contrast of what is not text, WCAG 2.2's success criterion 1.4.11,
// which axe-core 4.14.0 does not check: its rules are tagged wcag141 to
// wcag144, wcag146 and wcag1412, never wcag1411.

// Every border the page draws, and the outline of its first link with the
// focus, against the background behind them, at 3 to 1. A colour is read as
// the browser draws it, through a canvas, so that any CSS colour is measured,
// and a colour that is not opaque is blended over what is behind it, as it
// is seen. What is behind a border is its element's background, which the
// border box clips, over those of the elements around it; what is behind an
// outline is the background of the elements around its own. Returns one line
// for each kind of element and side that falls short, and whether an outline
// was measured.
export async function partProblems(page) {
  return page.evaluate(() => {
    const canvas = document.createElement("canvas").getContext("2d", { willReadFrequently: true });
    const parse = (text) => {
      canvas.clearRect(0, 0, 1, 1);
      canvas.fillStyle = "rgba(0, 0, 0, 0)";
      canvas.fillStyle = text;
      canvas.fillRect(0, 0, 1, 1);
      const [red, green, blue, alpha] = canvas.getImageData(0, 0, 1, 1).data;
      return { red, green, blue, alpha: alpha / 255 };
    };
    const over = (top, below) => {
      const mix = (key) => top[key] * top.alpha + below[key] * (1 - top.alpha);
      return { red: mix("red"), green: mix("green"), blue: mix("blue"), alpha: 1 };
    };
    const channel = (value) => {
      const share = value / 255;
      return share <= 0.04045 ? share / 12.92 : ((share + 0.055) / 1.055) ** 2.4;
    };
    const luminance = ({ red, green, blue }) => 0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue);
    const ratio = (one, other) => {
      const lighter = Math.max(luminance(one), luminance(other));
      const darker = Math.min(luminance(one), luminance(other));
      return (lighter + 0.05) / (darker + 0.05);
    };
    const white = { red: 255, green: 255, blue: 255, alpha: 1 };
    const backgroundOf = (element) => {
      const layers = [];
      for (let each = element; each; each = each.parentElement) {
        const colour = parse(getComputedStyle(each).backgroundColor);
        layers.push(colour);
        if (colour.alpha === 1) break;
      }
      return layers.reduceRight((below, layer) => over(layer, below), white);
    };
    const describe = (element) => [element.tagName.toLowerCase(), ...element.classList].join(".");
    const hex = ({ red, green, blue }) =>
      `#${[red, green, blue].map((value) => Math.round(value).toString(16).padStart(2, "0")).join("")}`;
    const short = new Map();
    const check = (element, part, colourText, behind) => {
      const colour = parse(colourText);
      if (colour.alpha === 0) return;
      const seen = over(colour, behind);
      const reached = ratio(seen, behind);
      const key = `${describe(element)} ${part}`;
      if (reached < 3 && !short.has(key)) {
        short.set(key, `${key} is ${colourText}, seen as ${hex(seen)} on ${hex(behind)}, ${reached.toFixed(2)} to 1`);
      }
    };
    for (const element of document.querySelectorAll("body *")) {
      if (!element.checkVisibility({ visibilityProperty: true })) continue;
      const style = getComputedStyle(element);
      for (const side of ["Top", "Right", "Bottom", "Left"]) {
        if (parseFloat(style[`border${side}Width`]) === 0 || ["none", "hidden"].includes(style[`border${side}Style`])) continue;
        check(element, `border-${side.toLowerCase()}`, style[`border${side}Color`], backgroundOf(element));
      }
    }
    let outlined = false;
    const link = document.querySelector("main a[href]");
    if (link) {
      link.focus({ focusVisible: true });
      const style = getComputedStyle(link);
      if (style.outlineStyle !== "none" && parseFloat(style.outlineWidth) > 0) {
        outlined = true;
        check(link, "outline with the focus", style.outlineColor, backgroundOf(link.parentElement));
      }
      link.blur();
    }
    const lines = [...short.values()].map((line) => `contrast of a part: ${line}`);
    return { lines, outlined };
  });
}
