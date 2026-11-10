# AGENTS.md: working in gherkin-glass

The manual for anyone, human or agent, changing gherkin-glass. [CONTRIBUTING.md](CONTRIBUTING.md) describes the process; this file says how to follow it.

## What to read

- [docs/BEHAVIOUR.md](docs/BEHAVIOUR.md): what the tool promises. Code does what it says, no more and no less.
- [docs/design/](docs/design/): the design notes, for why things are as they are.
- The issue the work belongs to, with its comments: the decisions taken there bind the work.

## Rules

1. **No code before its behaviour.** Work on a feature starts only once its issue is agreed and its behaviour is in docs/BEHAVIOUR.md. If the behaviour is missing, or the code would need to do something the document does not say, stop and raise it in the issue.
2. **Every rule has a fixture.** A rule added to or changed in docs/BEHAVIOUR.md comes with a fixture under `tests/fixtures/` and a test that checks it, in the same pull request as the code that makes it pass.
3. **Accessibility is behaviour.** A page that fails the accessibility check is a failing test, never a warning to skip. Colour and highlighting never carry meaning alone. JavaScript is assumed and drives the interactive parts, but the content is always in the page's HTML, never built by a script, so that headings, anchors and links exist as the page loads.
4. **Addresses are promises.** Never change how a page's path or an anchor is made without a major version, agreed in an issue.
5. **Small pull requests.** One thing each, naming its issue with `Closes #N` or `Refs #N`, with commit messages as CONTRIBUTING.md describes.
6. **Write for a screen reader.** Issues, pull requests, comments and documents use headings, lists, prose and simple tables, never ASCII art.
7. **Mark agent comments.** A comment an agent writes on GitHub says it is an agent's.

## Building and testing

The commands to build, test and check the project are listed here once the project has code.
