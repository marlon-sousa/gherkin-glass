---
name: review-change
description: How Claude has a change to gherkin-glass reviewed by an independent reviewer agent. Use before opening a pull request, after every later push to an open one, and when the reviewer's findings come back.
---

# Reviewing a change

[AGENTS.md](../../../AGENTS.md), "Reviewing a change", holds the rules and the checklist. This skill adds only how Claude runs the review.

## Starting the reviewer

Start a separate agent with the most capable model available. Its prompt says:

1. It is read-only. It edits, commits and pushes nothing, and never switches branches in the shared checkout. To build a revision, it adds a `git worktree` under the session's scratchpad directory, with a target directory of its own, and removes it afterwards.
2. Which commits to review, as revisions it can diff, the issue they belong to, and the draft commit messages and pull request description.
3. That AGENTS.md, CONTRIBUTING.md, docs/BEHAVIOUR.md and docs/ARCHITECTURE.md bind the change, with the issue's body.
4. The section "Reviewing a change" of AGENTS.md, in full. The reviewer works through every item of its checklist, listing every place in the diff where an item applies.
5. The report it returns, as that section describes it.

Give it the code and the issue, never your own conclusions about them.

## Handling the findings

1. Verify each finding before acting on it. Fix the real ones, including nits that are plainly right.
2. A finding you reject goes back to the reviewer with your reason, as AGENTS.md says.
3. Run the checks, push, and resume the same reviewer with what changed, so that it keeps its context.
4. When you tell the maintainer a pull request is ready, mention the review.
