---
name: stack
description: How Claude opens, extends and follows a stack of pull requests in gherkin-glass. Use when work is split into layers, before amending or force-pushing a branch in a stack, and when a lower layer changes or merges.
---

# Stacks of pull requests

[CONTRIBUTING.md](../../../CONTRIBUTING.md), "Stacked pull requests", holds the rules. This skill is how Claude follows them from a cloud session, where `gh` works only through REST: `gh pr create` and the `gh stack` extension do not work there.

## Opening a layer

1. Branch from the current tip of the layer below: `git fetch origin <lower branch>` then `git switch -c <branch> origin/<lower branch>`.
2. Have the layer reviewed before opening it, with the `review-change` skill.
3. Open the pull request with its base on the lower layer's branch.
4. Add it to the stack: `echo '{"pull_requests":[<number>]}' | gh api -X POST repos/marlon-sousa/gherkin-glass/stacks/<stack>/add --input -`. `gh api "repos/marlon-sousa/gherkin-glass/stacks?pull_request=<lower number>"` finds the stack's number.
5. List the stack again with the same call, and check the new layer is last.

## While the stack is open

- A fix to a lower layer is a new commit on that layer's branch only. Never amend or force-push a layer that has a layer above it, since the upper layer's diff is taken from the last commit the two share.
- Never merge a lower branch into an upper one. GitHub's restack drops merge commits, and the conflicts they settled come back.
- When a lower layer merges, GitHub retargets and rebases the layers above. Before working on a layer after that, check that nothing local would be lost (`git status --short` and `git log origin/<branch>..HEAD` print nothing), fetch, and reset to `origin/<branch>`.
- If GitHub cannot restack a layer, rebase it and every layer above it, bottom first, each onto the new tip of the one below: `git rebase --onto origin/<lower branch> <old base commit> <branch>`, run the checks, and push with `git push --force-with-lease`.

## Reporting

A message about a stack lists its pull requests bottom first, in the order they merge.
