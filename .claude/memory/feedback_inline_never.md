---
name: inline-never-still-needed
description: shim_exports #[inline(never)] is load-bearing — removing causes boot hang
type: feedback
---

The 324 `#[inline(never)]` annotations on shim_exports.rs functions are STILL NEEDED even with write_volatile and guard defenses in place. Removing them causes an i386 boot hang.

**Why:** `#[no_mangle]` only prevents the symbol from being removed — it does NOT prevent LLVM from inlining the function body into callers within the same CGU. When the inline function bodies from `ntkrnl_sys::inline::*` get inlined into caller code, LLVM applies captures(none) or other optimizations that break kernel invariants.

**How to apply:** Don't remove `#[inline(never)]` from shim_exports.rs. The `#[inline(never)]` and guards/write_volatile are defense-in-depth — both are needed. `#[inline(never)]` prevents LLVM from seeing into the function body. Guards/write_volatile protect the body if LLVM does see it.
