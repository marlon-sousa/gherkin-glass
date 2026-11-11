// Adapter: the reload script ghg serve adds to every page under --watch,
// docs/BEHAVIOUR.md, "Reloading". It says nothing of its own.

const POLL_EVERY = 1000;
const REMEMBERED = "ghg-serve-reload-heading";
const HEADINGS = "h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]";

const build = new URL(import.meta.url).searchParams.get("build");

// Whether any of the element is inside the window.
function inWindow(element) {
  const box = element.getBoundingClientRect();
  return box.bottom > 0 && box.top < innerHeight && box.right > 0 && box.left < innerWidth;
}

// The last heading with an id at or before the focused element, when it is
// in the window, or else the last whose top is at or above the window's top.
function headingTheReaderIsIn() {
  const headings = [...document.querySelectorAll(HEADINGS)];
  const focused = document.activeElement;
  if (
    focused &&
    focused !== document.body &&
    focused !== document.documentElement &&
    inWindow(focused)
  ) {
    const before = headings.filter(
      (heading) =>
        heading === focused ||
        heading.contains(focused) ||
        heading.compareDocumentPosition(focused) & Node.DOCUMENT_POSITION_FOLLOWING,
    );
    if (before.length > 0) {
      return before[before.length - 1];
    }
  }
  const above = headings.filter((heading) => heading.getBoundingClientRect().top <= 1);
  return above.length > 0 ? above[above.length - 1] : null;
}

function remember() {
  const heading = headingTheReaderIsIn();
  try {
    if (heading) {
      sessionStorage.setItem(
        REMEMBERED,
        JSON.stringify({ page: location.pathname, id: heading.id }),
      );
    } else {
      sessionStorage.removeItem(REMEMBERED);
    }
  } catch {
    // Without storage, the page reloads at the top.
  }
}

function restore() {
  let remembered = null;
  try {
    remembered = JSON.parse(sessionStorage.getItem(REMEMBERED) ?? "null");
    sessionStorage.removeItem(REMEMBERED);
  } catch {
    return;
  }
  if (!remembered || remembered.page !== location.pathname) {
    return;
  }
  const heading = document.getElementById(remembered.id);
  if (!heading || !heading.matches(HEADINGS)) {
    return;
  }
  if (!heading.hasAttribute("tabindex")) {
    // Focusable by script only, and never by the Tab key.
    heading.setAttribute("tabindex", "-1");
  }
  heading.scrollIntoView({ block: "start" });
  heading.focus({ preventScroll: true });
}

async function poll() {
  try {
    const answer = await fetch("/__ghg/build", { cache: "no-store" });
    if (answer.ok) {
      const current = (await answer.text()).trim();
      if (current !== build) {
        remember();
        // The browser's own restored scroll would undo the heading's.
        history.scrollRestoration = "manual";
        location.reload();
        return;
      }
    }
  } catch {
    // ghg serve is not answering, perhaps restarting: ask again.
  }
  setTimeout(poll, POLL_EVERY);
}

// After the load, so that neither the address's fragment nor the browser
// moves the page afterwards.
if (document.readyState === "complete") {
  restore();
} else {
  addEventListener("load", restore, { once: true });
}
setTimeout(poll, POLL_EVERY);
