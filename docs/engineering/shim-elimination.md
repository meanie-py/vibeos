# Shim Elimination Plan (rewritten 2026-05-04, status updated 2026-05-05 post-Phase-2-mm-macros)

## Status snapshot

**Phase 1 is COMPLETE for cm + ke + ob + ps + se + common + mm. Phase 2 has LANDED on ke + io + mm** — 16 ke fns (across pilot/2/B/A in `ke-macros`), 9 io fns (across pilot/1/2/3 in `io-inlines`), and 9 mm fns (across pilot/1/2 in a brand-new sibling `mm-macros` rlib). Surfaces closed: ke (`ke_rust_shims.c` macro/intrinsic), io (`io_rust_shims.c` `static __inline`/FORCEINLINE/`#define` from `io_x.h` + `iofuncs.h` + `io.h`), and the 4 trivial bit-twiddle macros + MI_MAKE_HARDWARE_PTE family + MI_UPDATE_VALID_PTE + MmInitializeMdl block in mm. Same augment+isolated-rlib pattern as Phase 1: C trampolines stay as ABI anchors (NTAPI on i386 where applicable), bodies collapse to one-line `_rs_*` delegations, the rlib boundary cuts the cross-CGU captures(none) wire.

**Phase 1 final state (≈165 fns).** All wrappers live in 13 sibling `*-inlines` rlibs under `sdk/lib/rust/`. M8 (PFN ref-counting, 5 fns) is the only remaining Phase 1 batch and is intentionally deferred to Phase 4. **The next planned work is more Phase 2 (~165 macros remaining across mm, io, kdbg, common subsystems) and Phase 4 (per-function captures(none) review, 9 fns + complex delegations).**

The kernel still has **~480 active C shim functions** (plus ~50 globals, ~17 LLVM-workaround functions, and ~5 vararg trampolines whose ANSI/wide-format-parsing now runs in Rust) spread across **7 `*_rust_shims.c` / `kernel_c_helpers.c` files**, totaling ~6,400 LOC. The function count hasn't dropped because Phase 1 *augments* — the C symbols stay alive as ABI anchors with one-line delegating bodies; only the FORCEINLINE outline LOC inside them collapsed. Phase 2+ is where the actual count drops. The shims exist because Rust can't link directly to certain C constructs:

- C `FORCEINLINE` / `static __inline` functions (no exported linker symbol)
- C macros (no symbol at all)
- C vararg functions (Rust can't safely emit C `va_list` calls)
- i386 calling-convention bridges (`fastcall` ↔ `extern "system"`)
- LLVM LTO miscompilation workarounds (a few specific functions)

**This is the second pass on this doc.** The March 2026 version is preserved in git history (`docs/engineering/shim-elimination.md@a~`); it's now stale on counts (340 → 552), file list (missed kernel_c_helpers.c, kdbg, kd, cm), and category retirement (claimed captures(none) was solved by CGU=1, but project chose CGU=256+opaque_ptr instead, which contracted but didn't eliminate the category).

## Current shim inventory (post-Phase 0)

| File | LOC | Active fns | Primary purpose |
|------|----:|-----------:|-----------------|
| `ntoskrnl/ke/ke_rust_shims.c` | 2,115 | ~155 | KE_x.h FORCEINLINEs + i386 register/V86/FXSAVE shims |
| `ntoskrnl/mm/mm_rust_shims.c` | 1,512 | ~85 | MM PTE accessors, AVL helpers, working-set lock wrappers |
| `ntoskrnl/io/iomgr/io_rust_shims.c` | 1,465 | ~44 | IO macros, captures(none) workarounds, complex delegations |
| `ntoskrnl/kernel_c_helpers.c` | 635 | ~16 | **Canonical "must-stay" home** — varargs, LTO workarounds, AVL include trick |
| `ntoskrnl/kdbg/kdbg_rust_shims.c` | 539 | ~16 + 2 large data tables | Mostly data (ComponentTable, KdbDebuggerCommands) |
| `ntoskrnl/config/cm_rust_shims.c` | 342 | ~37 | KCB lock wrappers + i386 dual-decoration scheme |
| `ntoskrnl/kd/kd_rust_shims.c` | 28 | 2 | HAL `KdComPortInUse` getter/setter (dllimport) |
| **TOTAL** | **~6,640** | **~395** | (counts include the data-table LOC, not "active fns") |

Plus the new shared C-side vararg adapter:

| File | LOC | Active fns | Primary purpose |
|------|----:|-----------:|-----------------|
| `ntoskrnl/ntprintf_trampoline.c` | ~100 | 1 public + 1 static | va_list → typed-slot pull → Rust `ntprintf_format_a` |

Phase 0 (2026-05-04) deleted six dead-on-disk shim files (ex, ob, ps, tier1, se, kd64 — −2,602 LOC, 245 functions) and resolved 12 duplications between mm and kernel_c_helpers. Eight vestigial functions inside the surviving files were also deleted. See the audit doc for the full retro.

## Categories (current taxonomy)

| Category | Count | Cost to eliminate | Notes |
|----------|------:|-------------------|-------|
| **FORCEINLINE_RENAME** | ~210 | Mechanical (LLM batch) | Largest bucket. The Phase 1 `_saved_*` rename + thin wrapper pattern. Concentrated in ke_x.h, miarm.h, ob_x.h, ps_x.h. |
| **MACRO_MATERIALIZATION** | ~180 | Per-file incremental | C macros manually re-implemented. Mix of trivial (InitializeObjectAttributes) and structural (HAL dispatch, KxMultiThreadWait). |
| **GLOBAL_ACCESSOR** | ~50 | Move global to Rust | Cheap. Each is just `return Global;` or `Global = X;`. Eliminate by giving the global a Rust home. |
| **OTHER (i386 register/V86/seg)** | ~50 | **Stay (mostly)** | i386-only. `Ke386Set*/Get*Reg`, `KiEnterTrap`/`KiEnterV86Trap`, `Ki386InitializeTss`, FXSAVE+MXCSR sanitization. Some moveable to Rust `core::arch::asm!`, but the trap-entry ones are intertwined with PSEH and may need to stay. |
| **FASTCALL_BRIDGE** | ~28 | Mechanical (`extern "fastcall"` works on stable) | i386 fastcall↔cdecl bridges + cm dual-decoration scheme. |
| **LLVM_WORKAROUND** | ~13 | **Stay** | SepInitTokensShim (i386 fat-LTO ud2), 9× io captures(none), IopInitializePlugPlayServices_shim (black_box mis-fold), amd64 KiSaveXState/KiRestoreXState (KVM bug, not LLVM). |
| **COMPLEX_DELEGATION** | ~14 | Per-function review | IopInitializeDriverModule (240 LOC), PiQueueDeviceAction_c, KiAttachProcess, MmFixupImageSectionSegments, ExpKdbgExt* family. |
| **VARARG** | ~5 | **Stay (thin)** | KdbPrintf, KdbpPrint, KdpDprintf, RtlStringCbPrintfA, RtlStringCbPrintfW — all converted to ~6-line `va_arg` pulls that hand off to Rust `ntprintf_format_a` / `ntprintf_w_format` (no `_vsnprintf`/`_vsnwprintf` anywhere in the active kernel). Plus `vDbgPrintExWithPrefixInternal` (`sdk/lib/rtl/debug.c`, weak-symbol Rust path that falls back to `_vsnprintf` only in user-mode `ntdll.dll`) and the non-variadic `DbgPrintEmitPreformatted`. **No remaining holdouts.** |

**Total stay-bucket: ~70 functions** (vararg + LLVM + most i386 register/V86 + kdbg data tables). Everything else (~482 functions) is in principle eliminable.

## What's changed since March

### 1. Captures(none) workarounds: contracted, not eliminated

The March doc's Category 3 claimed ~15 functions and proposed a one-line `codegen-units = 1` fix. The project chose CGU=256 + `opaque_ptr` laundering + `read/write_volatile` + `#[inline(never)]` splits instead, which solved most cases. **But 9 io_rust_shims captures(none) workarounds remain alive** with explicit comments documenting the bug:

- `ObCreateObject_shim`, `ObCreateObjectAddress_shim`, `ObInsertObject_shim`, `ObReferenceObjectByHandle_shim`
- `IopGetRegistryValue_shim`, `IopOpenRegistryKeyEx_shim`, `IopCreateRegistryKeyEx_shim`, `IopLoadDriver_shim`
- `PiInitCacheGroupInformation_shim` — comment notes "even with volatile reads", i.e. the opaque_ptr/volatile barrier doesn't save it.

Plus `IopInitializePlugPlayServices_shim` (a different LLVM bug — `core::hint::black_box` mis-folds to ud2 on i386) and `KiSaveXState`/`KiRestoreXState` on amd64 (KVM/hypervisor-level bug, not LLVM).

These need per-function investigation before they can be eliminated. They're not free wins.

### 2. `kernel_c_helpers.c` is the new canonical "must-stay" home

Header comment: *"Residual C helper functions that cannot be implemented in Rust (variadic, LTO workarounds, macro-generated tables, globals)."* This file collects:

- All variadic functions (KdpDprintf, RtlStringCbPrintfA)
- The fat-LTO ud2 workaround (SepInitTokensShim)
- AVL tree algorithms that depend on `#include <sdk/lib/rtl/avlsupp.c>`
- MM macros that need `__FILE__`/`__LINE__` C preprocessor info
- amd64/i386 KfRaiseIrql/KfLowerIrql calling-convention bridges

But **the consolidation is incomplete** — see #3.

### 3. Source-level duplication between shim files

~20 functions are defined in two files at once. The build presumably picks one via subsystem-active conditional compilation, but the duplication is fragile. Examples:

- `MmVerifyImageIsOkForMpUse` — mm_rust_shims AND kernel_c_helpers
- `SepInitTokensShim` — se AND kernel_c_helpers
- `KdpDprintf` — kd64 AND kernel_c_helpers
- `RtlStringCbPrintfA` — ex AND kernel_c_helpers
- `ExpKdbgExtFileCache`, `ExpKdbgExtDefWrites` — kd64 AND kernel_c_helpers
- `ExpKdbgExtHandle` — ex AND kernel_c_helpers
- `ARM64_UART_TRACE`, `ARM64_UART_TRACE_HEX` — ps AND kernel_c_helpers
- The full PS globals block (PsNtDllPathName, PspProcessMapping, PspThreadMapping, SharedUserData, …) — ps AND kernel_c_helpers
- `MmLockSectionSegment`, `MmUnlockSectionSegment`, `MmGetPageEntrySectionSegment` — mm AND kernel_c_helpers
- The AVL `Rtl*Avl` family + `Rtlp*AvlTreeNode` — mm AND kernel_c_helpers (both `#include <sdk/lib/rtl/avlsupp.c>`)
- `InterlockedExchange`/`InterlockedCompareExchange` (i386) — mm AND kernel_c_helpers
- `KfRaiseIrql`/`KfLowerIrql` (amd64 + i386 RustShim variants) — mm AND kernel_c_helpers
- `RustShimRtlFindSetBits`, `RustShimKdbpGetHexNumber` — mm AND kernel_c_helpers

This needs canonicalization (probably "kernel_c_helpers wins for must-stay; subsystem file wins for in-flight") before any elimination work, otherwise we'll be deleting one copy and leaving the other.

### 4. Vestigial shims confirmed (deletion candidates)

Verified by grepping all `.rs` files in `ntoskrnl-rust/src/` (and decoration variants `_Name`, `_Name@N`, `@Name@N`) — nine functions have **zero Rust callers**:

| File | Function | Why deletable |
|------|----------|---------------|
| `cm_rust_shims.c:78` | `CmpAcquireKcbLockSharedByKey` | Rust uses `*ByIndex` variant only |
| `kd_rust_shims.c:26` | `KdpRustGetComPortInUse` | Setter is called; getter never |
| `kdbg_rust_shims.c:422` | `Ke386GetTr_Shim` | Cdecl alias dead; stdcall variant satisfied by ke_rust_shims |
| `ke_rust_shims.c:772` | `KiCpuIdEx` | Rust uses `KiCpuId` only |
| `ke_rust_shims.c:2093` | `ke_trace_char` | Leftover debug helper |
| `mm_rust_shims.c:1429` | `MiArchCreateProcessAddressSpace_checked` | Leftover ARM64 IRQL instrumentation |
| `mm_rust_shims.c:1449` | `KeAttachProcess_irql_check` | Leftover ARM64 IRQL instrumentation |
| `mm_rust_shims.c:1465` | `KeDetachProcess_irql_checked` | Leftover ARM64 IRQL instrumentation |
| `mm_rust_shims.c:1480` | `MmCheckIrqlAfterCall` | Leftover ARM64 IRQL instrumentation |

### 5. Vararg trampolines: format-parsing now Rust, callers bypass C entirely

Status as of 2026-05-04 mid-day. All of the ANSI-format kernel `printf`-shaped functions either no longer exist as a Rust↔C variadic FFI hop, or have a thin `va_arg`-pull wrapper that hands a typed slot array to Rust for actual format-string parsing. The only thing that stays in C is the `va_arg(ap, …)` pull itself (Rust can't safely cross that boundary).

**Architecture.** Two-pass design:
1. Rust **`ntprintf` crate** (`sdk/lib/rust/ntprintf/`, no_std, ~700 LOC) implements a Windows-flavored printf core: `scanner.rs` walks the format string and emits a tag byte per `%`-specifier; `formatter.rs` consumes a slot array and renders into a caller-supplied `&mut [u8]` via a `Writer` trait. FFI entries: `ntprintf_scan_arg_types(fmt, out_tags, max)` and `ntprintf_format_a(dst, cap, fmt, slots, n)`. Validated against host libc by a per-spec `tests/libc_diff.rs` harness that rewrites Windows specifiers (`%I64x`, `%p`, `%ld`) into libc-compatible ones and renders both via `snprintf` for byte-identity.
2. **C-side trampoline** (`ntoskrnl/ntprintf_trampoline.c`): one shared `ntprintf_pull_va(fmt, slots, ap)` static that calls the scanner, walks `ap` once at the right primitive width per tag, and a public `ntprintf_va_format_a(dst, cap, fmt, ap)` that wraps it for `_vsnprintf`-shaped callers.

**Variadic shims migrated.** Each one's body collapsed to `va_start; ntprintf_va_format_a(...); va_end; sink(buffer)`:
- `KdbPrintf` (`kdbg_rust_shims.c`)
- `KdbpPrint` (`kdbg_rust_shims.c`; ARM64 stub remains a no-op)
- `KdpDprintf` (`kernel_c_helpers.c`)
- `RtlStringCbPrintfA` (`kernel_c_helpers.c`)
- `vDbgPrintExWithPrefixInternal` (`sdk/lib/rtl/debug.c` — backbone of `DbgPrint`/`DbgPrintEx`/`DbgPrintReturnControlC`/`vDbgPrintEx*`). The `ntprintf_va_format_a` reference is `__attribute__((weak))` so user-mode `ntdll.dll` (which also pulls `librtl.a`) falls back to legacy `_vsnprintf` when the Rust crate isn't linked. New non-variadic entry `DbgPrintEmitPreformatted(body, len, comp, level)` for Rust callers that want to skip the va_list hop entirely.

The only legacy `_vsnprintf`/`_vsnwprintf` callers that remain in the active kernel build are wide-format paths (`RtlStringCbPrintfW` in `cm_rust_shims.c`, plus `swprintf` callers in `pnpmgr/devaction.rs`, `ex/locale.rs`, `iomgr/{device,bootlog}.rs`, `fstub/disksup.rs`).

**Rust callers.** ~445 call sites swept off the variadic FFI hop and onto Rust-native macros at `ntoskrnl-rust` crate root:
- `kdb_printf!`, `kdbp_print!`, `kdp_dprintf!`, `rtl_string_cb_printf_a!`, `dbg_print!`, `dbg_print_ex!` — each takes `($fmt:expr $(,)? | $fmt:expr, $($arg:expr),+ $(,)?)` and routes through `dbgprint::IntoSlot::into_slot` which packs every primitive integer / pointer / `bool` into a `u64` slot at the call site.
- The supporting machinery (`sdk/lib/rust/ntoskrnl-rust/src/dbgprint.rs`, ~270 LOC) owns one `RawSink` (raw-pointer sink with `opaque_ptr` laundering — the slice variant of this trips i386 captures(none) → ud2; the comment in the file documents why), the per-target Rust shims (`rust_kdb_printf`/`rust_kdbp_print`/`rust_kdp_dprintf`/`rust_dbg_print[_ex]`/`rust_rtl_string_cb_printf_a`), and a `rust_dbg_print_cstr`/`rust_kdbp_print_cstr` pair for call sites whose format pointer is resolved at runtime (e.g. `EflagsBits[i]`, `DbgPrint(message, …)` where `message: PCHAR`).

**Migration tooling.** `tools/migrate_dbgprint.py` does the mechanical sweep — regex matches `\b{fname}\(\s*(b?"…")\s*\.as_ptr\(\)(?:\s+as\s+CAST)?\s*[,)]` and rewrites to `crate::{macro}!(b"…", …args)`, then drops the matching `extern "C" { fn X(...) -> ...; }` decl when no active call sites remain. Three patterns of irregular call shape (variable format pointer, format pointer pulled out of an array, `extern "C"` extern decl inside `extern "C" { … }` block on a single line) were handled either by hand or via a complementary one-shot Python script.

**Footprint impact.** No net reduction in shim function count (the C `va_arg` pull is the function), but the implementation surface inside each trampoline collapsed from ~30-line `_vsnprintf`-and-emit blocks to ~6-line `va_arg`-pull-and-Rust-format blocks, and the entire Rust-side `extern "C" { fn KdbPrintf(...) -> ULONG; }` family (~33 extern decls across 33 files) is gone. Smoke: i386 7/7 in 55s, amd64 7/7 in 3:56.

**Open follow-ups.**
- ~~`RtlStringCbPrintfW` + ~20 `swprintf` callers~~ — landed 2026-05-04 (see #6 below).
- Kani proofs on `format_into` truncation safety + `scanner` bounds — passed on; the libc-diff harness is more pragmatic and bug surface is bounded.

### 6. Vararg trampolines: wide (UTF-16) round

Status: 2026-05-04 evening. Closes the ANSI/wide split. New `sdk/lib/rust/ntprintf_w/` crate (~700 LOC) — UTF-16 sibling of `ntprintf` with the same two-call FFI shape. Specifier semantics flip per Microsoft's rule: in W-fmt `%s`/`%c` mean wide, `%hs`/`%hc` mean narrow, `%S` means narrow (the inverse of A-fmt's `%S`=wide). Slot ABI is identical (`u64`), so the C trampoline reuses one `va_arg` pull loop; only the scanner differs (`scan_format(&[u16], …)` instead of `&[u8]`).

**Trampolines wired:**
- `RtlStringCbPrintfW` (`cm_rust_shims.c`): body collapsed from `_saved_RtlStringCbVPrintfW` hop to `va_start; ntprintf_va_format_w(...); va_end` + NTSTATUS truncation check. Ships in `ntoskrnl/ntprintf_trampoline.c` next to its ANSI sibling (`ntprintf_va_format_a`). `cb` is byte count → divided by `sizeof(WCHAR)` for the wchar-counted Rust formatter.
- `RtlStringCbPrintfW_2lx` (custom shim from `io_rust_shims.c` for the single `IopGetParentIdPrefix` `"%lx&%lx"` callsite) deleted; replaced by `crate::rtl_string_cb_printf_w!(...)` macro at the call site.

**Rust side:**
- `dbgprint.rs`: new `RawSinkW` (raw-pointer wide sink with same `opaque_ptr` laundering as ANSI), `rust_rtl_string_cb_printf_w(dst: *mut u16, cb: usize, fmt: &[u16], slots: &[u64]) -> i32`, `rust_swprintf(dst, cap_chars, fmt, slots) -> i32`.
- `lib.rs`: 3 new macros at crate root — `rtl_string_cb_printf_w!`, `swprintf!`, plus `wstr!("…")` ASCII-widening compile-time literal helper that returns `&'static [u16]` (no trailing NUL — the formatter strips it anyway).

**Sweep tally (~30 call sites):**
- 5 `RtlStringCbPrintfW` (cmsysini ×4, cmboot ×1) → `rtl_string_cb_printf_w!`
- 1 `RtlStringCbPrintfW_2lx` (pnpmgr) → `rtl_string_cb_printf_w!` + delete the C shim
- 17 `swprintf` (fstub/disksup ×13, ex/locale ×4, iomgr/{device,bootlog} ×4, pnpmgr/devaction ×1) → `swprintf!`
- 6 orphan `extern "C" { fn swprintf(…)/RtlStringCbPrintfW(…) }` decls dropped

**Validation:** new `tests/libc_diff_w.rs` (~9 byte-identity tests against host libc snprintf for numeric specifiers — the wide ASCII output is downcast to bytes for comparison; string specifiers covered by unit tests since W-vs-libc string semantics differ enough that direct diff isn't useful). 19 unit tests + 9 libc-diff tests all green.

**Footprint impact.** Same pattern as the ANSI round: no net reduction in shim function count (the `va_arg` pull is the function), but the W trampolines collapsed from `_saved_*` chain hops to ~6-line va_arg-pull-and-Rust-format blocks. The custom `RtlStringCbPrintfW_2lx` shim is deleted outright. Smoke: i386 7/7 in 57s, amd64 7/7 in 3:53.

**End state for the vararg theme:** zero `_vsnprintf`/`_vsnwprintf` callers in the active kernel build. The only remaining legacy printf core call is the user-mode fallback in `sdk/lib/rtl/debug.c::vDbgPrintExWithPrefixInternal` (weak-symbol gated; only fires when `ntdll.dll` links `librtl.a` without the Rust `ntprintf` crate).

## Phased elimination plan

### Phase 0 — Free wins (~1 day)

1. **Delete the 9 vestigial functions.** Build all 3 arches; smoke i386 + amd64.
2. **Resolve the ~20 source-level duplications.** Pick canonical home per function (default rule: must-stay → kernel_c_helpers.c; everything else → subsystem file). Remove the duplicate; verify build still picks correct symbol.
3. **Document the must-stay set explicitly.** Add a top-level comment in each subsystem shim file listing which functions are intentional-stay vs in-flight, so future audits don't have to re-discover this.

Net result: ~30 functions eliminated, codebase clarity improved, no risk.

### Phase 1 — FORCEINLINE batch — **COMPLETE for cm + ke + ob + ps + se + common + mm** (2026-05-04 → 2026-05-05)

**TL;DR.** Pilot 1 (early 2026-05-04) added `cm_inlines.rs` to `ntoskrnl-rust`; cascade lit up 23 unexpected ud2 in pre-existing config code (cmparse / cmkcbncb / cmvalche / …) and i386 BSOD'd at boot. Reverted same day. Pilot 2 (evening) put the same eight KCB-lock translations in a brand-new isolated `sdk/lib/rust/cm-inlines/` rlib, kept the C outlines alive in `cm_rust_shims.c` as one-line trampolines that delegate to the Rust impls (vararg-style "augment, don't replace" — same shape as `RtlStringCbPrintfA`'s `va_start; ntprintf_va_format_a; va_end` body). i386 7/7 PASS in 59s. amd64 7/7 PASS in 4:02. **Zero unexpected ud2** on either arch — the cascade did not return.

**Phase 1 final landing.** Over 2026-05-04 → 2026-05-05 the augment+isolated-rlib pattern was applied to the rest of the Phase 1 surface. **13 sibling rlibs under `sdk/lib/rust/`:** `cm-inlines` (8 KCB locks), `ke-inlines` (49 ke_x.h FORCEINLINEs across K1-K11), `ob-inlines` (22 ob_x.h, extracted from `ntoskrnl-rust::shim_exports`), `ps-inlines` (18 ps_x.h, same route), `se-inlines` (10 Se*/Sep* security helpers), `common-inlines` (34 cross-subsystem helpers — Probe* family, splay-link, list-ops, RTL string helpers, interlocked, FsRtl region, init helpers, PsGetCurrentThread), `mm-pte-class-inlines` (M1, 7), `mm-pte-mut-inlines` (M2, 8), `mm-util-inlines` (M3, 6), `mm-ws-lock-inlines` (M4, 13), `mm-addr-space-inlines` (M5, 5), `mm-pte-atomic-inlines` (M6, 2), `mm-lock-inlines` (M7, 4). M8 (PFN ref-counting, 5 fns) deferred to Phase 4 per the "PFN database design needs its own thinking" note in the original miarm.h backlog. **End-state metrics:** i386 7/7 PASS in ~50s, amd64 7/7 PASS in ~4:00, arm64 ntoskrnl builds clean (3.97 MB), 0 unexpected ud2 on i386+amd64. `ntoskrnl-rust::shim_exports.rs` line count dropped ~3340 → ~2700 LOC across the ob/ps/se/common extractions (the cm/ke/mm rlibs are pure C-trampoline replacements and don't touch shim_exports).

**Lessons captured during the sweep, applicable to Phase 2+:**
- **i386 stdcall→cdecl `.set` aliases must be co-located with target.** GAS evaluates `.set` at assembly time, so a `.set _PsGetCurrentThread, _PsGetCurrentThread@0` in `shim_exports.rs` (via `core::arch::global_asm!`) breaks linking when the target moves to a sibling rlib — lld can't resolve the alias across rlib archives. Fix: move the `global_asm!` block into the same crate as the wrapper. Captured in `feedback_global_asm_alias_locality.md`.
- **`extern crate <name>;` requirement.** Adding a Cargo dep is not enough; `kernel-rust/src/lib.rs` needs `extern crate <name>;` or the rlib's `#[no_mangle]` symbols won't end up in `libkernel_rust.a`. Captured in `feedback_extern_crate_for_staticlib.md`.
- **`bindgen ULONG_PTR` ≠ hand-written `ULONG_PTR`.** `ddk::ULONG_PTR` is `u32` on i386, hand-written `base::ULONG_PTR` is `usize`. Some `inline::mm` impls return `base::ULONG_PTR` and FFI signatures need `as ULONG_PTR` casts at the wrapper boundary. Surfaced in M1 (`MiDetermineUserGlobalPteMask`) and M3 (`ProbeForWriteSize_t`).
- **Cross-crate `#[inline(always)]` for `core::arch::asm!`-bodied fns doesn't reliably inline at `opt-level=1`.** Symptoms: undefined `_KeGetCurrentThread` / `_KeGetPcr` references when a sibling rlib calls one via `inline::mm` or `inline::ke`. Workaround: replicate the asm directly into the consumer crate. Captured in `feedback_cross_crate_inline_asm.md`.
- **Calling-convention freedom for `_rs_*` impls.** The C trampoline body declares the prototype, so the Rust impl can be uniformly `extern "C"` (cdecl) regardless of whether the trampoline is NTAPI/stdcall/fastcall. Simplifies the rlib boilerplate.

**Batch 2** (same day, 2026-05-04): same pattern applied to `ke_x.h` — eight in-stack-queued spinlock FORCEINLINEs (APC lock family ×5: `KiAcquireApcLockRaiseToSynch`, `KiAcquireApcLockAtSynchLevel`, `KiAcquireApcLockRaiseToDpc`, `KiReleaseApcLock`, `KiReleaseApcLockFromSynchLevel`; device queue lock ×2: `KiAcquireDeviceQueueLock`, `KiReleaseDeviceQueueLock`; process lock ×1: `KiReleaseProcessLockFromSynchLevel` — the only ProcessLock fn whose C trampoline lives in `ke_rust_shims.c` rather than `mm_rust_shims.c`). New `sdk/lib/rust/ke-inlines/` rlib (~190 LOC). C trampolines in `ke_rust_shims.c` collapsed from `_saved_*` (FORCEINLINE outline) calls to one-line `_rs_Ki*` delegations. Build: 3-arch clean, 0 unexpected ud2. The `_force_ke_inlines_symbols` block in `kernel-rust::lib.rs` mirrors `_force_cm_inlines_symbols`. Per-batch cost was ~30 minutes from scope-pick to build-clean.

**Why isolation worked where in-crate didn't.** Workspace `lto = false`, `codegen-units = 256`. The cascade-trigger is rustc-side cross-CGU attribute deduction: when a new `#[no_mangle] extern "C"` Rust fn appears in a CGU adjacent to existing config code in the *same* crate, attribute inference re-shuffles and lands ud2 in the neighbours. A separate rlib compiles in its own rustc invocation with no source visibility into ntoskrnl-rust's bodies, so the inference can never fire across the boundary. The C `Cmp*KcbLock*` / `Ki*Lock*` symbols staying alive (now thin trampolines instead of FORCEINLINE outlines) means ntoskrnl-rust's existing `extern "C" { fn CmpAcquireKcbLockExclusive(...) }` and `extern "C" { fn KiAcquireApcLockRaiseToSynch(...) }` decls are unchanged — zero source touches in `ntoskrnl-rust` for these rounds.

**One incidental gotcha (cm-inlines pilot).** `ntkrnl_sys::mm::KeGetCurrentThread` is `#[inline(always)]` over `core::arch::asm!`, but cross-crate `#[inline(always)]` for inline-asm fns doesn't inline reliably at our `opt-level = 1`. cm-inlines emitted a `_KeGetCurrentThread` symbol reference with no definition (the C-side equivalent is itself a FORCEINLINE in `ke_x.h` — no exported symbol). Replicated the asm directly into cm-inlines. Same pattern needed in ke-inlines for `KeGetPcr` / `KeGetCurrentPrcb` (used by `_rs_KiAcquireDeviceQueueLock` / `_rs_KiReleaseDeviceQueueLock` to check `KPRCB.DpcThreadActive`); replicated the FS/GS/TPIDR_EL1 reads directly.

**End state for the sixteen FORCEINLINE outlines.** Bodies live in Rust (`sdk/lib/rust/cm-inlines/src/lib.rs` and `sdk/lib/rust/ke-inlines/src/lib.rs`). C symbols still exist in `cm_rust_shims.c` / `ke_rust_shims.c` and are what every Rust + C caller still resolves to. Each C body is one delegating line: `_rs_CmpAcquireKcbLockExclusive(Kcb)` / `_rs_KiAcquireApcLockRaiseToSynch(Thread, Handle)` etc. The `_fi_*` (cm) and `_saved_*` (ke) FORCEINLINE-rename macros are kept in scope as a documented fallback path (if the *-inlines crate is ever unwired). Note: `shim_exports.rs` contains *dead* duplicate definitions for the eight `Ki*Lock*` symbols — they were dead before this batch (the .obj from `ke_rust_shims.c` wins over the .a member at link time) and remain dead. Cleanup deferred to a future commit; their presence does not affect the augment pattern's correctness.

---

### Phase 1 — original PILOT 1 retro (preserved for context)

The original Phase 1 plan was: pull the FORCEINLINE bodies out of headers like `cm_x.h`, write Rust equivalents as `#[no_mangle] pub unsafe extern "C" fn Foo(...)`, and remove the C-side outline wrapper from `*_rust_shims.c`. This **does not work** as designed; pilot crashed boot.

**Pilot scope.** Ob shims got deleted in Phase 0 so the new pilot was `cm_rust_shims.c` — 8 KCB-lock FORCEINLINE outlines (`CmpAcquireKcbLockExclusive`, …, `CmpConvertKcbSharedToExclusive`). Source bodies in `ntoskrnl/include/internal/cm_x.h` (304 LOC); each inline is 3-7 lines calling well-known primitives. Hand translation into a new `sdk/lib/rust/ntoskrnl-rust/src/config/cm_inlines.rs` + `_force_cm_inlines_symbols` block in `lib.rs` + outline removal from `cm_rust_shims.c`.

**Result.** All 3 arches built clean. amd64 smoke 7/7 PASS. **i386 boot BSOD** at boot: `STOP 0x1E (KMODE_EXCEPTION_NOT_HANDLED), 0xC000001D STATUS_ILLEGAL_INSTRUCTION` at `0x8045C00F` — which symbolicates to `CmpCreateKeyControlBlock+0x...`, a function I never touched.

`scan_ud2` confirmed the cause: **23 unexpected ud2** instructions, only 4 in the new Rust functions; 19 in pre-existing config code (`CmpCreateKeyControlBlock`, `CmpParseKey`, `CmpReferenceKeyControlBlock`, `CmpDelayCloseWorker`, `CmDeleteValueKey`, `CmEnumerateValueKey`, `CmQueryValueKey`, `CmSetValueKey`, `CmpDoOpen`, `CmpLookInCache`, …). Pre-Phase-1 baseline: **0 unexpected ud2**.

**Why the cascade.** This is the LLVM `captures(none)` cross-CGU contagion documented in `feedback_cgu1.md`: when a `#[no_mangle] extern "C" fn` is added to a Rust crate, LLVM re-evaluates cross-CGU declarations for the whole subsystem, infers `captures(none)` more aggressively on the affected pointers, and proves "success path is dead → emit ud2" in unrelated functions. Same family of bug as the late-March `write_volatile` sweep; the project chose `CGU=256 + opaque_ptr + read_volatile + #[inline(never)]` instead of `CGU=1` to handle it.

**Defenses I tried (all insufficient):**
1. `opaque_ptr(Kcb)` laundering at every public-fn entry. → still 23 ud2.
2. Direct access to `crate::config::cmkcbncb::CmpCacheTable` via `read_volatile(addr_of!(...))` instead of an `extern "C" { static mut ... }` block. → still 23 ud2.
3. `extern "C"` (cdecl, what bindgen expects) confirmed correct via decoration check.

The cascade triggers at the **CGU partitioning level** — any `#[no_mangle] extern "C"` Rust function added to a CGU adjacent to existing config code re-shuffles the optimizer's cross-CGU inference, and the captures(none) annotations can't be locally suppressed. opaque_ptr at the *new* function's boundary doesn't help because the ud2 lands in the *neighboring* old code.

**Current state.** Pilot fully reverted. Build clean, smoke green, 0 unexpected ud2. cm_inlines.rs deleted. cm_rust_shims.c restored to its post-Phase-0 state. **The Phase 1 approach as written needs to be redesigned before any further functions can be moved.**

**Possible Phase 1 redesigns** (originally listed as guesses; status updated):
1. **Pre-harden affected config functions** with opaque_ptr / write_volatile before moving any FORCEINLINE. Touch ~19 functions in `cmparse.rs` / `cmkcbncb.rs` / `cmvalche.rs` / `cmapi.rs`. — Not attempted; superseded by 2+3.
2. **Isolated crate** (`cm-inlines` sibling to `ntoskrnl-rust`). Crate boundary should make cross-CGU inference impossible. — **Validated by Pilot 2 (see retro above).**
3. **Don't replace, augment.** Keep the C `_rust_shims.c` outline alive as a thin trampoline that delegates to a Rust impl with a different mangled name. — **Combined with #2 in Pilot 2.** The augment pattern means existing Rust callers' `extern "C"` decls are unchanged; only the C body collapses to one delegating line. Mirrors the `ntprintf` vararg trampolines (`RtlStringCbPrintfA::body = va_start; ntprintf_va_format_a; va_end`).
4. **Accept the cascade and add ud2-tolerance** with `#[inline(never)]` + `core::hint::black_box` chains. — Not attempted (combination of 2+3 sufficed).

**Lesson for the rest of the elimination plan.** *Moving* any single `_rust_shims.c` function out of C and adding it to `ntoskrnl-rust` cascades ud2 across the subsystem. *Augmenting* (C symbol stays as an ABI anchor; Rust impl in a sibling rlib; C body delegates) does not. Phases 2-4 below should default to the augment+isolated-crate pattern unless there's a specific reason to move-and-replace.

### Phase 1 — original batch backlog (scoped 2026-05-04, **all landed 2026-05-05** unless noted)

Captured here for historical retro. Augment pattern held at scale across all 13 rlibs. Effort estimates ran ~30 min per Easy batch, ~60-90 min per Medium batch from scope-pick to smoke-green — the original cm/ke pilot baseline turned out to be representative.

**Big finding from the survey:** `ob_x.h` and `ps_x.h` are **already done at the shim-elimination level** — both `ob_rust_shims.c` and `ps_rust_shims.c` were deleted in Phase 0, all 22 ob FORCEINLINEs and all 19 ps FORCEINLINEs already live as private `pub fn` in `ntkrnl_sys::inline::{ob,ps}` and are exported via `#[no_mangle] extern "C"` wrappers in `ntoskrnl-rust::shim_exports`. That's a *different* mechanism than the augment+isolated-rlib pattern — `shim_exports` wraps an inline impl with no C trampoline standing between — but the end state is functionally equivalent (Rust body owns the symbol, C side has nothing to delete). No Phase 1 work needed for ob or ps; the only reframing would be cosmetic (extracting shim_exports's ob/ps wrappers into a sibling rlib for taxonomic consistency). **Redirect remaining Phase 1 effort to ke + mm.**

#### ke_x.h backlog — **K1 through K11 LANDED** (49 fns, ~1100 LOC of Rust impls in `ke-inlines/src/lib.rs`)

| # | Batch | Functions | Complex | LOC | Status |
|---|-------|-----------|---------|----:|--------|
| K1 | dispatcher-locks | KiAcquireDispatcherLock, KiReleaseDispatcherLock, KiAcquire/ReleaseDispatcherLockAtSynchLevel, KiAcquire/ReleaseDispatcherObject | Easy | 40 | **DONE** |
| K2 | thread-prcb-locks | KiAcquire/ReleaseThreadLock, KiTryThreadLock, KiAcquire/ReleasePrcbLock | Easy | 25 | **DONE** |
| K3 | timer-lock-mgmt | KiAcquire/ReleaseTimerLock, KiComputeTimerTableIndex | Easy | 30 | **DONE** |
| K4 | nmi-list-lock | KiAcquire/ReleaseNmiListLock | Easy | 5 | **DONE** |
| K5 | sched-helpers | KiInsertDeferredReadyList, KiRescheduleThread, KiSetThreadSwapBusy, KiCheckDeferredReadyList, KiRequestApcInterrupt | Easy | 60 | **DONE** |
| K6 | spinlock-primitives | KxAcquireSpinLock, KxReleaseSpinLock | Medium | 45 | **DONE** |
| K7 | timer-helpers | KiComputeDueTime, KxSetTimerForThreadWait, KxInsertTimer, KxRemoveTreeTimer, KiRecalculateDueTime | Medium | 150 | **DONE** (commit `7ecf997f`) — DISPATCHER_HEADER bitfield writes via bindgen 5-level setter accessors; KeQueryInterruptTime/SystemTime via SharedUserData macro on amd64/arm64 + stdcall extern on i386 |
| K8 | guarded-mutex | _KeInitializeGuardedMutex, _KeAcquire/ReleaseGuardedMutex, _KeAcquire/ReleaseGuardedMutexUnsafe, _KeTryToAcquireGuardedMutex | Medium | 120 | **DONE** (commit `3f2028d1`) — `AtomicI32::from_ptr` fast path + KGATE wait |
| K9 | unwait-helpers | KxUnwaitThread, KxUnwaitThreadForEvent, KxQueueReadyThread | Medium | 100 | **DONE** (commit `720e226c`) — preserves the C trampoline's defensive WaitThread null-guard |
| K10 | ready-select | KiSelectReadyThread, KiComputeNewPriority | Medium | 80 | **DONE** (commit `b502bdd9`) — BitScanReverse + dynamic-priority decay |
| K11 | stack-alert | KiCheckThreadStackSwap, KiCheckAlertability | Medium | 50 | **DONE** (commit `b9f5c4db`) |
| **Phase 2** | wait-setup macros | KxDelayThreadWait, KxMultiThreadWait, KxSingleThreadWait, KxQueueThreadWait | — | — | These are `#define` blocks not FORCEINLINEs — Phase 2 macro-materialization. **Note: 2026-05-05 amd64 taskbar wait-block-corruption bug was a `KxMultiThreadWait` C trampoline issue (half-init of `Timer.WaitListHead.Flink/Blink` collided with subsequent `KeWaitForMultipleObjects` `InsertTailList` to produce self-referencing TimerBlock). Fix: trampoline now uses `InitializeListHead` matching the original macro semantics. See translation audit headline.** |

#### miarm.h backlog — **M1 through M7 LANDED** (45 fns across 7 rlibs)

`mm_rust_shims.c` uses `_orig_*` rename for some + direct outline impl for others (no uniform `_saved_*` pattern). Header comment at `mm_rust_shims.c:840` documents that PCH-covered FORCEINLINEs *can't* be `_saved_`-renamed from this file — those need a different strategy or stay in C. Each MM batch got its own isolated rlib because the bodies are fatter than ke's locks.

| # | Batch (rlib) | Functions | Complex | LOC | Status |
|---|--------------|-----------|---------|----:|--------|
| M1 | mm-pte-class-inlines | MiIsMappedPte, MiIsPageWriteable, MiIsPageDirty, MiIsUserPde, MiIsUserPte, MiIsPdeForAddressValid, MiDetermineUserGlobalPteMask | Easy | 80-100 | **DONE** (commit `c442d718`) — and uncovered the `MiDetermineUserGlobalPteMask` 4-level paging bug that gated amd64 boot from M1 commit forward; fix landed `5857abfc` adding `MiIsUserPxe` + `MiIsUserPpe` ports cfg-gated to non-x86 |
| M2 | mm-pte-mut-inlines | MiMakeDirtyPage, MiMakeCleanPage, MiMakeAccessedPage, MiMakeOwnerPage, MiMakeWritePage, MiPageDisableCache, MiPageWriteThrough, MiPageWriteCombined | Easy | 60-80 | **DONE** (commit `867aa6c4`) |
| M3 | mm-util-inlines | InterlockedIncrementUL, InterlockedExchangeAddUL, MmSetDirtyPage, ProbeForWriteSize_t, ProbeForWriteIoStatusBlock, ExAllocatePoolZero | Easy | 50-70 | **DONE** (commit `2d80d672`) |
| M4 | mm-ws-lock-inlines | MiLock/UnlockWorkingSet, MiLock/UnlockWorkingSetShared, MiLock/UnlockProcessWorkingSet, MiLock/UnlockProcessWorkingSetUnsafe, MiLock/UnlockProcessWorkingSetShared, MM_ANY_WS_LOCK_HELD, plus ForFault pair (13 fns total) | Medium | 120-150 | **DONE** (commit `37a490e5`) — ARM64 IRQL UART tracing + MmDbgLast{Safe,Excl}{Locker,Unlocker} debug instrumentation kept in C trampoline (needs `__builtin_return_address(0)` from C frame) |
| M5 | mm-addr-space-inlines | MmGetCurrentAddressSpace, MmGetKernelAddressSpace, MmGetAddressSpaceOwner, MmLockAddressSpace, MmUnlockAddressSpace | Medium | 60-80 | **DONE** (commit `37a490e5`) — PCH-covered FORCEINLINEs, kept Mm* ABI-anchor symbol with one-line `_rs_*` body |
| M6 | mm-pte-atomic-inlines | MiInterlockedExchangePte, MiInterlockedCompareExchangePte | Medium | 40-50 | **DONE** (commit `37a490e5`) — backed by inline::mm's `core::sync::atomic` impls (32-bit on i386, 64-bit on amd64/arm64); no inline asm needed cross-crate |
| M7 | mm-lock-inlines | MiAcquire/ReleaseExpansionLock, MiAcquire/ReleasePfnLock | Medium | 40-60 | **DONE** (commit `37a490e5`) — ExpansionLock delegates to inline::mm; PfnLock uses `KeAcquire/ReleaseQueuedSpinLock` with `LockQueuePfnLock = 2` (replicated locally to preserve queued-spinlock semantics — inline::mm uses the regular spinlock variant). C trampoline retains MiExpansionLockOwner debug tracking + ARM64 IRQL trace |
| M8 | mm-pfn-ref-inlines | MiReferenceProbedPageAndBumpLockCount, MiReferenceUnusedPageAndBumpLockCount, MiReferenceUsedPageAndBumpLockCount, MiDropLockCount, MiRemoveZeroPageSafe | **Hard — Phase 4** | 100-140 | Deferred — touches MmPfnDatabase global, PfnLock, SMP-specific; per-arch PFN struct. PFN database design needs its own thinking. |
| **skip** | AVL trees | Rtl*Avl*, Rtlp*AvlTreeNode | — | — | `#include <sdk/lib/rtl/avlsupp.c>` — must stay in C |
| **skip** | __FILE__/__LINE__ macros | MmLockSectionSegment, MmUnlockSectionSegment, MmGetPageEntrySectionSegment | — | — | Need C preprocessor; must stay |

#### ob_x.h / ps_x.h status — done

- **ob_x.h**: 22 FORCEINLINEs all in `ntkrnl_sys::inline::ob` + `shim_exports.rs` wrappers. `ob_rust_shims.c` deleted in Phase 0. Verified via `_force_shim_symbols` + scan_ud2 clean. **No Phase 1 batches needed.**
- **ps_x.h**: 19 FORCEINLINEs all in `ntkrnl_sys::inline::ps` + `shim_exports.rs` wrappers. `ps_rust_shims.c` deleted in Phase 0. **No Phase 1 batches needed.** Cosmetic extraction into a `ps-inlines` rlib would only standardize the layout — functionally equivalent.

#### Aggregate Phase 1 status (final)

- **cm**: 1 batch DONE (Pilot 2, 8 fns)
- **ke**: 11 batches DONE (49 fns, ~1100 LOC)
- **ob**: 22 fns extracted into `ob-inlines` rlib
- **ps**: 18 fns extracted into `ps-inlines` rlib
- **se**: 10 fns extracted into `se-inlines` rlib
- **common**: 34 cross-subsystem fns extracted into `common-inlines` rlib
- **mm**: 7 batches DONE (M1-M7, 45 fns), M8 deferred to Phase 4
- **Total landed**: ~165 fns of Rust impl across 13 sibling rlibs. Per-batch wall-clock matched the cm/ke pilot baseline (~30 min Easy, ~60-90 min Medium scope-pick → smoke-green).
- **Remaining Phase 1 surface**: M8 only — PFN ref-counting fns deferred to Phase 4.

### Phase 2 — Macro materializations (started 2026-05-05; ke + io + mm-trivial surfaces CLOSED)

~180 functions total, mix of trivial and structural. Two passes:
1. **Trivial macros** (~80): `InitializeObjectAttributes`, `CONTAINING_RECORD`, `IsListEmpty`, `ObReferenceObject`, etc. Mostly 1-5 lines each. Inline as Rust helpers in subsystem modules; already partially done (`iomgr.rs` has its own `InitializeObjectAttributes`).
2. **Structural macros** (~100): HAL dispatch (`HalInitPnpDriver` → `HALDISPATCH->HalInitPnpDriver()`), wait macros (`KxMultiThreadWait`), IRP stack helpers. Per-subsystem; some need new Rust APIs (e.g. a `HalDispatch` accessor crate).

**ke macros LANDED (2026-05-05).** 16 fns translated to a brand-new sibling `sdk/lib/rust/ke-macros/` rlib (separate from `ke-inlines` for Phase 1 / FORCEINLINE; same augment+isolated-rlib pattern as Phase 1). The macro/intrinsic surface of `ke_rust_shims.c` is now empty:

| # | Batch | Functions | Commit |
|---|-------|-----------|--------|
| 2-pilot | wait-list satisfy/add | KiSatisfyMutantWait, KiSatisfyNonMutantWait, KiSatisfyObjectWait, KiAddThreadToWaitList | `f77b9c18` |
| 2 | wait-setup | KxSingleThreadWait, KxDelayThreadWait, KxMultiThreadWait, KxQueueThreadWait | `bdf21228` |
| B | trap-frame / arch intrinsics | YieldProcessor (pause/yield asm), KeGetTrapFramePc (Eip/Rip/Pc field), KiRundownThread (i386 NpxThread cleanup; no-op elsewhere), KiGetLinkedTrapFrame (per-arch path to previous trap frame), KiGetUserModeStackAddress (pointer to Rsp/Sp/HardwareEsp on current trap frame) | `c87d114c` |
| A | HAL dispatch | HalQuerySystemInformation, HalSetSystemInformation, HalVectorToIDTEntry. Bodies pull `Option<fn>` slot from `HalDispatchTable` / `HalPrivateDispatchTable`, call if `Some`, return `STATUS_NOT_IMPLEMENTED` on `None` (safer than the original NULL-fn-ptr crash). | `36802240` |

**io macros + inlines LANDED (2026-05-05).** 9 fns translated to a brand-new sibling `sdk/lib/rust/io-inlines/` rlib (third Phase 2 sibling alongside `ke-inlines` Phase 1 and `ke-macros` Phase 2; same augment+isolated-rlib pattern). The macro/inline FORCEINLINE surface of `io_rust_shims.c` is now CLOSED — every `static __inline` from `io_x.h`, the `IoCopyCurrentIrpStackLocationToNext` FORCEINLINE from `iofuncs.h`, and the `IoGetDevObjExtension` `#define` from `io.h` have Rust bodies:

| # | Batch | Functions | Commit |
|---|-------|-----------|--------|
| io-pilot | trivial inlines | IopValidateOpenPacket (type/size sanity), IoGetDevObjExtension (DEVICE_OBJECT → EXTENDED_DEVOBJ_EXTENSION cast), IoCopyCurrentIrpStackLocationToNext (`offset_of!(_IO_STACK_LOCATION, CompletionRoutine)` for the IRP stack memcpy length) | `6b6ea901` |
| io-1 | IRP/thread queueing | IopQueueIrpToThread, IopUnQueueIrpFromThread. `KeEnterGuardedRegionThread` / `KeLeaveGuardedRegionThread` macros inlined locally — `SpecialApcDisable` decrement/increment with kernel-APC delivery check on transition to zero. KTHREAD::ApcState lives inside `__bindgen_anon_1` on all 3 arches. | `36123f11` |
| io-2 | file-object lock pair | IopLockFileObject (fast path: `AtomicU32::swap(SeqCst)` on `FILE_OBJECT::Busy`; slow path: `IopAcquireFileObjectLock` with Alertable derived from `Flags & FO_ALERTABLE_IO`), IopUnlockFileObject (fires `KeSetEvent(&Lock, 0, FALSE)` if `Waiters != 0`). `Obf{Reference,Dereference}Object` declared `extern "fastcall"` on i386, `extern "C"` elsewhere. | `6a8c0453` |
| io-3 | IO counters | IopUpdateOperationCount, IopUpdateTransferCount. `ExInterlockedAddLargeStatistic` rendered as `AtomicI64::fetch_add(SeqCst)` — same observable semantics on all 3 arches (lock cmpxchg8b on i386 vs the C macro's lock addl + lock adcl pair). `PsGetCurrentProcess` replicated locally to dodge cross-crate inline-asm-non-inline gotcha. | `78bc204f` |

**mm trivial macros + PTE construction LANDED (2026-05-05).** 9 fns translated to a brand-new sibling `sdk/lib/rust/mm-macros/` rlib (mirror of `ke-macros` for ke; same augment+isolated-rlib pattern). Three batches:

| # | Batch | Functions | Commit |
|---|-------|-----------|--------|
| mm-pilot | trivial bit-twiddles | RMAP_IS_SEGMENT (high-bits sentinel), PAGE_ROUND_DOWN (`& ~(PAGE_SIZE-1)`), PFN_FROM_SSE (`>> PAGE_SHIFT`), MM_IS_WAIT_PTE (low-bit + sentinel match; `MM_WAIT_ENTRY` is `0x7FFFFFFFFFFFFC00` on 64-bit, `0x7ffffc00` on 32-bit, cfg-gated). | `8d442a97` |
| mm-1 | MI_MAKE_HARDWARE_PTE family + MI_UPDATE_VALID_PTE | MI_MAKE_HARDWARE_PTE_KERNEL, MI_MAKE_HARDWARE_PTE_USER, MI_MAKE_HARDWARE_PTE, MI_UPDATE_VALID_PTE. Rust impls already lived in `ntkrnl_sys::inline::mm`; rlib exposes them as `_rs_*` no-mangle symbols with `opaque_ptr` laundering (same shape as `mm-pte-mut-inlines` from Phase 1 batch M2). | `6beac1fc` |
| mm-2 | MmInitializeMdl | MDL header init: zero Next, compute Size from PFN count via inlined `ADDRESS_AND_SIZE_TO_SPAN_PAGES`, zero MdlFlags, page-align StartVa via inlined `PAGE_ALIGN`, set ByteOffset via inlined `BYTE_OFFSET`. The 3 macro helpers are inlined directly using PAGE_SIZE/PAGE_SHIFT from the pilot. | `5a95fb86` |

`InterlockedExchangeAddUL` and `MmSetDirtyPage` already lived in `mm-util-inlines` (Phase 1 batch M3) — not re-translated.

**The 2026-05-05 `KxMultiThreadWait` half-init bug fix (the amd64 taskbar wedge)** is now in Rust source; the C trampoline + ke_x.h macro defensive variants are still there (the macros are still instantiated in C from `ntoskrnl/ke/wait.c::KeDelayExecutionThread` etc., which haven't been ported yet) but the Rust impl in `ke-macros::_rs_KxMultiThreadWait` matches it bit-for-bit (uses `InitializeListHead` for the multi-wait path so `KeWaitForMultipleObjects`'s `InsertTailList` loop appends cleanly; uses defensive full-init for the single/delay/queue paths that don't go through the multi-wait loop).

**Lessons captured during the ke-macros + io-inlines sweeps:**
- **`extern "C"` ABI for 8-byte aggregate union args is NOT interchangeable with `i64` on i386.** Declared `_rs_KxSetTimerForThreadWait` extern with `interval: i64` instead of `interval: _LARGE_INTEGER`. amd64 7/7 PASS, i386 booted to WinLogon → GinaInit then hung the kernel in a tight win32k message-retry loop (`(\\win32ss\\user\\ntuser\\msgqueue.c:1096) err: Sending to Suspended Thread Msg 121` spamming forever). Root cause: i386 cdecl lowers union-aggregate args differently from scalar i64 args even when both are 8 bytes; the receiver read garbage off the stack. amd64 Windows x64 ABI tolerates the mismatch (both go in the same RDX register slot for the second arg). Captured in `feedback_i64_vs_largeinteger_abi.md`. Rule: **mirror the receiver's exact type at the FFI boundary; pass `*ptr` as the dereferenced struct, not `(*ptr).QuadPart`**.
- **NTAPI trampoline + `extern "C"` Rust impl works cleanly.** HAL dispatch macros are NTAPI (stdcall on i386, default elsewhere) but the Rust `_rs_*` impls are uniformly `extern "C"` (cdecl). The C trampoline keeps the NTAPI decoration and bridges the call — same pattern that worked for the wait-setup batch. No need to mark `_rs_*` as `extern "system"`.
- **`KTHREAD::ApcState` is wrapped in an anonymous union on every arch.** Path is `__bindgen_anon_1.ApcState.ApcListHead[KernelMode]`, not bare `ApcState.ApcListHead[KernelMode]`. Easy to miss because `KTHREAD::ApcStateIndex` IS a direct field; the union is for the `ApcStateFill[]` overlay that `ETHREAD::SavedApcState` mirrors.
- **`PsGetCurrentProcess` cross-crate gotcha.** The `ntkrnl-sys::inline::tier1::PsGetCurrentProcess` inline ultimately reaches an inline-asm-bodied `KeGetCurrentThread`. At opt-level=1 a sibling rlib that calls it doesn't reliably inline the chain — emits `_KeGetCurrentThread` undefined ref. Workaround (now used in `ke-macros`, `ke-inlines`, and `io-inlines`): replicate the PCR/PRCB/KTHREAD walk locally with `core::arch::asm!`. Same root cause as `feedback_cross_crate_inline_asm.md`.

Remaining Phase 2 surface (~146 fns) is concentrated in:
- `mm_rust_shims.c` (~76 fns left) — pilot/1/2 closed the trivial macro + PTE construction surface; remaining shims are AVL trees (`#include <sdk/lib/rtl/avlsupp.c>` from `Rtlp{Find,Insert,Delete}AvlTreeNode` — Phase 4 complex delegation, see "AVL refactor" note below), `Mmsp*` segment helpers (qsort callbacks with global state), `MmVerifyImageIsOkForMpUse` (complex), `Arm64DumpPte` (debug helper, intentional-stay), and `__FILE__/__LINE__`-flavored macros that must stay in C. The MM AVL flavor (`MMADDRESS_NODE` / `MM_AVL_TABLE`) has incompatible layout with the Rtl flavor (`avltable` rlib): the Mm node packs Balance:2 into the low bits of Parent and embeds StartingVpn/EndingVpn directly. Two paths to address: (a) generic-ize `avltable::tree` over a `BalancedLinks` trait + add a second instantiation, or (b) write a separate `mm-avl` rlib. Either way Phase 4 work — current `#include <sdk/lib/rtl/avlsupp.c>` arrangement is the cheapest correct option until then.
- `io_rust_shims.c` (~35 fns left) — macro/inline surface CLOSED; remaining shims are captures(none) workarounds (`ObCreateObject_shim`, `ObInsertObject_shim`, `ObReferenceObjectByHandle_shim`, `IopInitializePlugPlayServices_shim`), the i386 `_shim`-suffixed cdecl→stdcall string bridges (`RtlAnsiStringToUnicodeString_shim`, etc.), `IopInitializeDriverModule` (Phase 4 complex delegation), and the PnP request paths (`IopQueryDeviceCapabilities_c`, `IopInitiatePnpIrp_c`, `IopCompleteRequest_c`, `PiQueueDeviceAction_c`, `PiPerformSyncDeviceAction_c`, `IoRequestDeviceEject_c`, `PiSetDevNodeText_c`).
- `kdbg_rust_shims.c` (~16 fns + 2 large data tables) — KDBG extension entry points, mostly stay.
- `cm_rust_shims.c` (~37 fns) — mostly KCB-lock variants (Phase 1 already covered the main ones) + i386 dual-decoration scheme.
- `kernel_c_helpers.c` (~16 fns) — the canonical "must-stay" home; very little here can move to Rust.

Cost: case-by-case, but well-bounded. Per-batch wall-clock in the io-inlines sweep was ~25–40 minutes scope-pick → smoke-green for the four batches; the IO counter batch (most novel work, including local `PsGetCurrentProcess` replication and the `AtomicI64::fetch_add` rendering of `ExInterlockedAddLargeStatistic`) was the longest at ~40 minutes.

#### Phase 2 backlog (explicitly deferred)

The following Phase 2 candidates have been identified but are *not* being translated in the current sweep. Each entry has a category and a one-line reason; the rationale here is "what's left and why we're not touching it yet" so future contributors don't re-derive the analysis.

**mm_rust_shims.c (~14 fns left):**

| Function(s) | Category | Reason for deferral |
|---|---|---|
| `RtlpFindAvlTableNodeOrParent`, `RtlpInsertAvlTreeNode`, `RtlpDeleteAvlTreeNode` | **Phase 4 — AVL refactor** | The Mm flavor (`MMADDRESS_NODE`, `MM_AVL_TABLE`) is layout-incompatible with the existing `avltable` rlib's Rtl flavor (`RtlBalancedLinks`): Mm packs `Balance:2` into the low bits of `Parent` and embeds `StartingVpn`/`EndingVpn` directly in the node. C uses `#include <sdk/lib/rtl/avlsupp.c>` with `#define Rtl* → Mi*` rename in `miavl.h` for textual reuse. Rust resolution path: generic-ize `avltable::tree` over a `BalancedLinks` trait + add a second instantiation, OR write a separate `mm-avl` rlib (~400 LOC duplicate). Estimated half-day to one day. |
| `RtlRightChildAvl`, `RtlLeftChildAvl`, `RtlParentAvl`, `RtlIsRightChildAvl`, `RtlIsLeftChildAvl` | **Phase 4 — AVL refactor (5 trivial accessors)** | Field accessors / Balance-from-Parent low-bit mask. Each is 1-3 lines and individually trivial, but pulling them into Rust makes sense only after the AVL flavor split lands (otherwise we'd be duplicating `MMADDRESS_NODE` field paths). Tackle as a follow-on to the AVL refactor. |
| `MmspCompareSegments_Shim`, `MmspSortSegments_Shim`, `MmspCheckSegmentBounds_Shim`, `MmspPageAlignSegments_Shim` | **Phase 4 — complex delegation** | qsort callbacks with global state over `MM_IMAGE_SECTION_OBJECT`. Touch image-section parsing, which is itself only partially Rust-translated. Best handled as part of broader image-section work. |
| `MmFixupImageSectionSegments` | **Phase 4 — complex delegation** | Same family as the `Mmsp*` helpers; full image-section fixup pass. |
| `MmVerifyImageIsOkForMpUse` | **Phase 4 — captures(none) workaround OR complex delegation** | Currently a stub returning TRUE; not yet analyzed. Could be a free win if it stays a stub, but worth a per-function look first. |
| `Arm64DumpPte` | **Intentional-stay** | Debug helper that calls `MiAddressToPte` + `ARM64_UART_TRACE_HEX` macros. Pure printf-shaped tracing; no value in moving to Rust. |
| `ExAllocatePoolZero` | **Intentional-stay (LLVM workaround)** | Already delegates to `_rs_ExAllocatePoolZero`; trampoline stays as the captures(none) firewall. |
| `KeFlushProcessTb` | **Intentional-stay** | Per-arch TLB flush primitives (`__writecr3` / `tlbi vmalle1is`). Tiny inline asm; lives where the rest of the per-arch shim asm lives. |

**io_rust_shims.c (~30 fns left):**

| Function(s) | Category | Reason for deferral |
|---|---|---|
| `ObCreateObject_shim`, `ObCreateObjectAddress_shim`, `ObInsertObject_shim`, `ObReferenceObjectByHandle_shim` | **Phase 4 — captures(none) workaround** | Documented in-file: "LLVM cross-CGU `captures(none)` attribute propagation bug — when Ob* functions land in a different codegen unit, LLVM concludes the output pointer is never written → eliminates the success path." Re-probe each individually (per the lpc/connect precedent — some old captures(none) excuses are now stale). |
| `IopInitializePlugPlayServices_shim` | **Phase 4 — captures(none) workaround** | "Calling it through `core::hint::black_box()` in iomgr.rs miscompiles on i386 into an immediate `ud2`. Route through a C shim so LLVM can't fold the continuation in IoInitSystem away." Re-probe. |
| `RtlAnsiStringToUnicodeString_shim`, `RtlUnicodeStringToAnsiString_shim`, `RtlInitAnsiString_shim`, `RtlFreeUnicodeString_shim` | **Phase 3 — calling convention bridge** | i386 cdecl→stdcall bridges. Targeted by Phase 3 batch 3. |
| `IopQueryDeviceCapabilities_c`, `IopGetBusTypeGuidIndex_c`, `IopInitiatePnpIrp_c`, `IopCompleteRequest_c`, `PiSetDevNodeText_c`, `IoRequestDeviceEject_c`, `PiQueueDeviceAction_c`, `PiPerformSyncDeviceAction_c` | **Phase 4 — complex delegation** | The `_c`-suffixed PnP request paths. Each is a real algorithm (dozens of LOC each, calls into PnP subsystem). The `_c` suffix marks them as "C version preserved while Rust port lands" — they should eventually become Rust translations, not shim wrappers. |
| `IopInitializeDriverModule` | **Phase 4 — complex delegation** | Documented in-file: "complex function that was not translated in the IO Tier 1 batch". Real translation work. |
| `PiInitCacheGroupInformation_shim`, `IopIsValidPhysicalDeviceObject`, `HalGetDmaAdapter`, `CmiGetNextResourceDescriptor`, `IopDeviceNodeSetFlag` | **Mixed — needs per-fn analysis** | Some are macros (small), some are real calls into other subsystems. Triage when the captures(none) re-probe happens. |
| `RtlUnicodeStringPrintf_4wZ_shim` | **Intentional-stay** | Variadic `RtlUnicodeStringPrintf` wrapper with a 4-arg specialization (`"%wZ%wZ#%wZ%wZ"`). The `RtlUnicodeStringPrintf` call site is a single hot path in `iomgr/deviface.rs`; routing it through Rust ntprintf_w is more work than the savings justify. |
| `ProbeForReadLargeInteger`, `ProbeForWriteHandle`, `ProbeForWriteIoStatusBlock`, `ObReferenceObject`, `ObDereferenceObject`, `ExRaiseStatus` | **Already done (Phase 1 — `common-inlines`)** | These C definitions are dead weight — the `common-inlines` rlib's Rust impls win at link time. Cleanup deferred (cosmetic only). |

**kdbg_rust_shims.c (~16 fns + 2 large data tables):**

| Function(s) | Category | Reason for deferral |
|---|---|---|
| `KDB_ACCESS_TYPE_TO_STRING`, `KDB_ENTER_CONDITION_TO_STRING`, `NPX_STATE_TO_STRING` | **Intentional-stay** | Enum-name-to-string lookup tables for KDBG output formatting. Pure string interpolation; no Rust value-add. |
| `Ke386GetGlobalDescriptorTable`, `Ke386GetLocalDescriptorTable`, `Ke386GetTr_Shim` | **Intentional-stay (i386 inline asm)** | Direct `sgdt`/`sldt`/`str` instruction wrappers used only by KDBG-mode register dumps. Same family as the `~50 i386 register/V86/segment shims` documented in Phase 5. |
| `KeGetContextFrameRegister`, `KeSetContextFrameRegister`, `KeGetTrapFrameFrameRegister`, `KeGetTrapFrameStackRegister` | **Intentional-stay (per-arch CONTEXT/KTRAP_FRAME field accessors)** | Per-arch accessors duplicated for 32-bit and 64-bit CONTEXT layouts. Pure field reads; debugger-only. |
| `KdbPrintf`, `KdbpPrint` | **Intentional-stay (vararg trampolines)** | Variadic format-string entry points used by KDBG-mode dumps; routed through ntprintf already. Keeping the C trampoline as the va_list anchor is the documented vararg pattern. |

**cm_rust_shims.c (~25 fns left after Phase 1 KCB-lock extraction):**

| Function(s) | Category | Reason for deferral |
|---|---|---|
| `_shim_*` family with `__asm__("_Name@N")` symbol-renames (~14 fns: `_shim_CmpAcquireKcbLockShared` through `_shim_CmpIsKeyValueSmall`) | **Phase 3 follow-up — dual-decoration scheme** | Each KCB-lock / hive-cell / utility function has BOTH a bare cdecl symbol (`_Name`) for bindgen-generated callers AND an `_Name@N` stdcall variant for hand-written `extern "system"` callers. The right resolution is to align all hand-written callers on bindgen (`use ntkrnl_sys::ddk::*;`) and drop the `_shim_*` family. Larger Phase 3 batch — defer until the simple fastcall/stdcall bridges land. |
| Stub macros (`CMP_ASSERT_REGISTRY_LOCK`, `CMP_ASSERT_FLUSH_LOCK`, `CMP_ASSERT_KCB_LOCK`, `CMP_ASSERT_REGISTRY_LOCK_OR_LOADING`, `ASSERT_VALUE_BIG`, `CMTRACE`, `IS_NULL_TERMINATED`) | **Intentional-stay** | Empty-body assertion stubs and trace macros. ~1-2 lines each; no Rust value-add. |
| `CmpIsKcbLockedExclusive_outline`, `_outline_GET_HASH_INDEX` | **Phase 4 follow-up** | These are PCH-extracted FORCEINLINE outlines with `__attribute__((alias))` aliases. Better to fold into the cm_x.h Phase 1 work whenever cm-inlines gets its second sweep. |
| `RtlStringCbPrintfW`, `CmpIsKeyValueSmall`, `HvGetCellType`, `CM_CHECK_REGISTRY_SUCCESS`, `LANGIDFROMLCID`, `PtrToUlong`, `_saved_PtrToUlong` | **Mixed — Phase 2 candidates that overlap with vararg/macro work already done** | The vararg-overhaul round (May 4) put `RtlStringCbPrintfW` body in Rust ntprintf_w; the C trampoline here is the va_list anchor and stays. The others are tiny utility macros that could land in a future cm-macros rlib if a sweep is justified. |

**kernel_c_helpers.c (~70 fns):**

The canonical "must-stay" home. Categories per Phase 5:
- 7 vararg trampolines (need C va_list)
- 5 LLVM/KVM workarounds (compiler/hypervisor bugs)
- ~50 i386 register/V86/segment shims (Phase 5 audit; `core::arch::asm!` in Rust could move some)
- ~5 kdbg data tables (macro-generated; could be `const fn` arrays in Rust but low priority)
- 2-3 RtlpAvl/MiAvl bridges (move with the AVL refactor)
- ~2 fastcall→cdecl bridges (Phase 3 batch 1)
- 2 stdcall→cdecl bridges (Phase 3 batch 2)

### Phase 3 — Calling conventions (~3 days)

~28 functions. `extern "fastcall"` works on stable Rust for `i686-pc-windows-gnu` (verified in March doc); just hasn't been adopted everywhere. Mechanical work:
- Replace `extern "system" fn KfRaiseIrql(...)` declarations with `#[cfg(target_arch="x86")] extern "fastcall" fn KfRaiseIrql(...)` + amd64/arm64 fallback.
- Remove the fastcall→cdecl bridge from the shim file.
- Resolve cm_rust_shims.c's dual-decoration scheme (the ~17 `__asm__("_Name@N")` symbol-renames) by aligning the bindgen-generated callers and hand-written `extern "system"` callers on a single decoration.

### Phase 4 — Per-function review (~2 weeks)

- **9 captures(none) workarounds**: investigate per function whether a Rust-side `opaque_ptr` + `write_volatile` + `#[inline(never)]` triple is sufficient (it was for most of the originals), or whether a structural change is needed (e.g. return `Result<(T, NTSTATUS), NTSTATUS>` instead of output pointer).
- **~14 complex delegations**: per-function judgment. Some (PiQueueDeviceAction_c, IopInitializeDriverModule) might warrant Rust translation; others (HAL dispatch wrappers) might stay.
- **~50 GLOBAL_ACCESSOR functions**: each gets eliminated by moving the underlying global to Rust. Coordinate with other translation work — these come for free as subsystems get translated.

### Phase 5 — Document the must-stay set (~1 day)

Final state: ~70 functions remain, all in `kernel_c_helpers.c` (or with explicit comments in subsystem files). Each documented with its reason:
- 7 varargs (need C va_list)
- 5 LLVM/KVM workarounds (compiler/hypervisor bugs)
- ~50 i386 register/V86/segment shims (need Rust `core::arch::asm!` audit; some may move, some stay)
- ~5 kdbg data tables (macro-generated; could be `const fn` arrays in Rust but low priority)

## Effort summary

| Phase | Functions removed | Effort | Risk | Status |
|-------|------------------:|--------|------|--------|
| 0. Free wins | ~30 | 1 day | None | **DONE** (2026-05-04) |
| 1. FORCEINLINE batch | ~165 augmented | ~3 days actual | Low | **DONE except M8** (2026-05-04 → 2026-05-05) — M8 (5 fns) deferred to Phase 4 |
| 2. Macro materializations | ~180 augmented (34 landed) | ~1 week remaining | Low-medium (per-file) | **Partial** — ke + io + mm-trivial surfaces CLOSED (16 ke fns in `ke-macros`, 9 io fns in `io-inlines`, 9 mm fns in `mm-macros`); ~146 fns left in mm-AVL/io-cnone-workarounds/kdbg/cm/common (many will stay — see notes above) |
| 3. Calling conventions | ~28 | 3 days | Low (mechanical) | Pending |
| 4. Per-function review | ~50 | 2 weeks | Medium (each captures(none) is its own debug session) | Pending; M8 (5 fns) folded in |
| 5. Document must-stays | ~0 | 1 day | None | Pending |
| **Total reduction (planned)** | **~498 / ~552 (90%)** | **~5 weeks** | | |

Phase 1 functions are *augmented*, not removed — the C symbols stay alive as one-line ABI-anchor trampolines that delegate to `_rs_*` Rust impls in sibling rlibs. The function count drops in Phase 2 (where the underlying C macro instantiations get replaced) and Phase 4 (where the captures(none) workarounds are reviewed individually). End state: ~70 must-stay functions in kernel_c_helpers.c. ~9,238 LOC of shim C code → roughly ~1,500 LOC.

## Verification tools (current)

| Tool | What it checks | Location |
|------|---------------|----------|
| `verify_constants.py` | Constant values vs C headers | `tools/rustconv/rustconv/verify_constants.py` |
| `verify_abi.py` | extern "C" vs extern "system" ABI | `tools/rustconv/rustconv/verify_abi.py` |
| `verify_ffi.py` | Combined: ABI + params + C macros + flexible arrays | `tools/rustconv/rustconv/verify_ffi.py` |
| `lint_kernel.py` | Unprotected output pointer writes (captures(none) target) | `tools/lint_kernel.py` |
| `scan_ud2.py` | LLVM miscompilation (ud2 in named functions) | `tools/scan_ud2.py` |

Smoke after each phase: `pytest tests/qemu/test_smoke.py -v --arch i386 --arch amd64` (without `--skip-clean-build` for final verification — see project rule about cargo's incremental cache producing corrupt staticlibs).

## Open questions before starting

1. **Pilot subsystem confirmation.** ob_rust_shims.c is the cleanest target on paper (42/44 FORCEINLINE_RENAME) — agree to start there?
2. **Duplication resolution rule.** Default proposed: must-stay → kernel_c_helpers.c; in-flight → subsystem file. Are there cases where this is wrong?
3. **Phase ordering.** Is the Phase 0 → 1 → 2 → 3 → 4 → 5 sequence right, or do we want to interleave (e.g. Phase 1 + Phase 3 in parallel since they touch disjoint files)?
4. **Acceptance bar.** Does each phase need a smoke pass, or only at end-of-phase boundaries? (Recommendation: smoke at end of each subsystem-batch, not each function.)
