---
name: no-auto-commit
description: Do not create git commits without asking the user first
type: feedback
---

Do not make git commits without asking the user first.

**Why:** User explicitly asked "don't make a commit immediately without asking me please."

**How to apply:** After completing work, stage and describe what would be committed, then ask "shall I commit?" before running `git commit`.
