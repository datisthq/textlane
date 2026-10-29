---
name: change-review
description: Review a pull request, or the local branch before it lands, for correctness, security and repo conventions — inline PR comments, or a ranked list locally. Invoked by the review workflow or a person; never fires on its own.
disable-model-invocation: true
---

# Review a change

Read the diff, judge it, post findings as inline comments. Be useful, not thorough-looking:
a review that flags twenty things teaches nothing, and the twenty-first — the real one — is lost
in it.

## What to look for

**Correctness**

- Logic that is wrong for an input the change makes reachable: empty input, non-ASCII text,
  Windows path separators.
- Error handling that swallows a failure or reports success on a partial result.
- A binding that panics across the FFI boundary instead of returning the crate's error as a
  JavaScript exception or Python exception.
- Performance where the change makes it structurally worse (a tree re-walked per node, a value
  copied across the binding per call), not where it is merely not optimal.

**Security — reported only when reachable**

- `unsafe` outside `bindings::node`, where only napi's macros may expand to it.
- Secrets in code, logs or CI; a workflow step with more permissions than it uses.

**This repo's own invariants** — where a generic reviewer is blind:

- behaviour added to a binding (`typescript/`, `python/`) instead of the crate;
- a changed plugin name, option, AST node shape or binding export without the matching
  `python/docs/`, `typescript/docs/` and `rust/docs/` pages and shipped `.skills/` skill;
- `PLATFORMS` and the release matrix drifting apart (**`package-distribution`**);
- an `unwrap` or `expect` outside tests, or a default feature that pulls in napi or PyO3;
- a commit typed `feat` or `fix` that does not change what an installer gets, or a package change
  typed `chore`; a commit marked breaking;
- `AGENTS.md`, a skill or `CONTRIBUTING.md` that the diff made untrue (**`human-documentation`**).

## How to report

Rate each finding **CRITICAL / HIGH / MEDIUM / LOW**, and say what input or sequence produces
the failure. A finding that cannot name the case that breaks is a hunch: either verify it or
leave it out. Post specific issues as inline comments on the lines they concern; keep the
summary short. If the change is good, say so briefly and stop.

## Reviewing locally

With no pull request, the change is everything this branch adds over `main`:
`git diff main...HEAD` plus the uncommitted and untracked files. Print the findings in the
terminal, worst first, each with its `file:line`. Post nothing, and edit nothing unless the
caller asks for the fixes.
