---
name: Avoid frivolous commits
description: Don't commit intermediate states like retranslations that haven't been tested. Keep debugging until smoke tests pass.
type: feedback
---

Don't make commits just for retranslations or intermediate progress. Only commit when smoke tests pass or there's a meaningful working state change. Keep debugging until the file actually works.

**Why:** The user considers retranslation-only commits frivolous — the value is in getting files activated and passing tests, not in having clean translations that haven't been runtime tested.

**How to apply:** When retranslating a file, immediately proceed to activate it, test it, debug any issues, and only commit once it's either working or you've exhausted debugging and need to document why it's blocked.
