---
name: verify-abi-tool
description: verify_abi.py tool detects extern "C" vs extern "system" mismatches across all Rust kernel files
type: reference
---

`tools/rustconv/rustconv/verify_abi.py` cross-checks hand-written extern blocks against ntoskrnl.spec (1,567 exports) and bindgen_i386.rs (2,921 functions).

**Usage:**
```bash
.venv/bin/python tools/rustconv/rustconv/verify_abi.py                    # Check all
.venv/bin/python tools/rustconv/rustconv/verify_abi.py path/to/file.rs    # Single file
```

**Current state:** 61 mismatches found (42 stdcall, 19 fastcall). Stdcall can be fixed by changing to `extern "system"`. Fastcall functions (InterlockedCompareExchange, InterlockedExchange, etc.) need C shims since Rust doesn't have stable `extern "fastcall"`.

**How to apply:** Run after any translation to catch ABI bugs before they cause i386 ud2 crashes. Should be added to the rustconv post-integration checklist alongside verify_constants.py.
