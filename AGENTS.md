# Agent instructions

## GitHub and Jules

- Use the terminal console for GitHub and Jules. Run `gh`, networked `git`, and `jules` outside the sandbox. Do not use browser automation for these operations.
- Treat a numbered request in a pull-request discussion as a PR number. Do not reinterpret it as a milestone task or create a Jules session from that reinterpretation.
- Create Jules sessions only for necessary work that is not already implemented, assigned, or covered by an existing session or PR. Check the session registry, open PRs, branches, and merged code first.
- Reuse existing sessions and retrieve their completed work. Review, correct, test, and integrate the output; a completed Jules session is work to consume, not disposable output to replace with another session.
- Do not close a completed-code PR merely because its branch is stale, conflicts, or needs fixes. Update the existing PR and resolve the issues. If its implementation was integrated elsewhere, verify and show the exact evidence before proposing closure; obtain the user's explicit agreement to close it.
- Keep a versioned task-to-session-to-PR registry with exact base commits and integration commits. Never assign the same work twice. Keep at most ten active Jules tasks and parallelize only independent work.
- Preserve useful implementation and verification evidence when integrating Jules output. Create follow-up sessions only for concrete uncovered requirements, not to regenerate existing work.

## General

- For GitHub operations, prefer the terminal CLI (`gh` and `git`) over browser automation. The console-only rule above applies to this repository.
