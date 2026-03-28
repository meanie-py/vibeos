---
name: llvm-i386-workarounds
description: LLVM i386 is more aggressive than amd64 — inline asm/read_volatile/black_box may not prevent optimization
type: feedback
---

The i386 LLVM backend is more aggressive at null-pointer deduction and UB exploitation than amd64. Workarounds that work on amd64 may fail on i386.

**Why:** Codex discovered that inline asm, `core::ptr::read_volatile`, AND `core::hint::black_box` all failed to prevent LLVM from proving a pointer was null on i386, while the same code worked on amd64. The fix required restructuring the code:
1. Return object addresses as `ULONG_PTR` (integer) from C shims instead of `PVOID` (pointer)
2. Reconstruct pointers with `core::ptr::with_exposed_provenance_mut` after non-zero check
3. Split large functions with `#[inline(never)]` helpers to break optimizer's cross-function deduction
4. Route output-pointer functions through C shims (same as captures(none) mitigation)

**How to apply:**
- When fighting LLVM optimization on i386, don't waste time with inline asm / read_volatile / black_box — go straight to structural fixes
- For output-pointer functions: C shim returning ULONG_PTR > C shim returning PVOID > inline asm
- For large functions with multiple call sites that trigger optimizer: split into `#[inline(never)]` helpers
- The `captures(none)` cross-CGU issue affects BOTH arches but manifests differently: on amd64 a C shim is sufficient, on i386 you may also need integer-return + provenance reconstruction
- Codex's approach: emit LLVM IR (`--emit=llvm-ir`), search for `unreachable` after the suspect call, trace back to the bad assumption
