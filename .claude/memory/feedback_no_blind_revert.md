---
name: no-blind-revert
description: Never use git checkout/restore to discard changes — always stash instead
type: feedback
---

Never use `git checkout --`, `git restore`, or delete files to discard uncommitted work. Always use `git stash` to preserve changes.

**Why:** Subagent work on 3 files (pagfault.rs, ARM3/section.rs, mm/section.rs) was permanently lost when `git checkout --` was used to bisect a regression. This work cost significant time and API calls to produce.

**How to apply:** When bisecting regressions or temporarily reverting changes:
- Use `git stash save "description"` to preserve ALL uncommitted work first
- Use `git stash push -m "description" -- file1 file2` to stash specific files
- Never use `git checkout -- file` or `git restore file` on work that was expensive to produce (LLM translations, subagent output, etc.)
- After bisecting, use `git stash pop` or `git stash apply` to restore
