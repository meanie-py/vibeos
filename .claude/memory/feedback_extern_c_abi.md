---
name: extern-c-vs-system-i386
description: extern "C" vs extern "system" ABI mismatch is the #1 cause of i386 Rust kernel crashes (ud2)
type: feedback
---

On i386, `extern "C"` is cdecl and `extern "system"` is stdcall (NTAPI). On amd64/arm64 they're identical. Hand-written extern blocks declaring NTAPI kernel functions as `extern "C"` cause LLVM to detect the ABI inconsistency and optimize the call continuation as unreachable → ud2.

**Why:** This was the root cause of the entire i386 "pre-existing regression" that blocked testing for weeks. Codex traced it through LLVM IR and found it across device.rs, iomgr.rs, and arcname.rs. Three separate functions in three files, all the same pattern.

**How to apply:**
- ALL hand-written extern blocks for Windows kernel NTAPI functions MUST use `extern "system"`, not `extern "C"`
- Only true C-runtime functions (sprintf, wcslen, memset, etc.) should be `extern "C"`
- RTL string functions (RtlInitAnsiString, RtlAnsiStringToUnicodeString, etc.) that are NTAPI should use shims or `extern "system"`
- When enabling Rust modules on i386, audit ALL `extern "C"` blocks first — grep for `extern "C"` and verify each function
- The symptom is always the same: LLVM inserts ud2 after a call to the misdeclared function, BUGCHECK 0x1E with exception 0xC000001D
- This does NOT affect amd64 or arm64 (both ABIs are cdecl on those targets)
- When in doubt, use bindgen-generated imports from `ntkrnl_sys::ddk::` which have correct calling conventions
