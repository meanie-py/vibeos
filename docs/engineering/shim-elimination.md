# Shim Elimination Plan (rewritten 2026-05-04)

## Status snapshot

The kernel currently has **~480 active C shim functions** (plus ~50 globals, ~17 LLVM-workaround functions, and ~5 vararg trampolines whose ANSI-format-parsing now runs in Rust) spread across **7 `*_rust_shims.c` / `kernel_c_helpers.c` files**, totaling ~6,400 LOC. They exist because Rust can't link directly to certain C constructs:

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

### Phase 1 — FORCEINLINE batch — **PILOT FAILED, plan blocked** (2026-05-04)

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

**Possible Phase 1 redesigns** (none validated; all are guesses):
1. **Add opaque_ptr / write_volatile defenses to the *affected* config functions** before moving any FORCEINLINE. Touch ~19 functions in `cmparse.rs` / `cmkcbncb.rs` / `cmvalche.rs` / `cmapi.rs` to make them captures(none)-resistant first. Then attempt the move. Cost: significantly higher than estimated.
2. **Keep the Rust replacements in a separate isolated CGU** by giving them their own crate (`cm-inlines` sibling to `ntoskrnl-rust` in the workspace). Crate boundaries should make captures(none) cross-CGU inference impossible since LLVM only sees declarations across crate boundaries.
3. **Don't replace, augment.** Leave the C-side `_rust_shims.c` outline in place and add the Rust version with a different mangled name; pivot Rust callers to use the Rust version. The C side becomes dead but stays linked. Bigger code, no churn for cross-CGU.
4. **Accept the cascade and add ud2-tolerance**. Use `#[inline(never)]` + `core::hint::black_box` chains on neighbors. Brittle and would need wide application.

Option 2 (isolated crate) feels cleanest in principle but requires Cargo.toml workspace work; option 1 is most direct but the cost is a lot higher than the original plan implied.

**Lesson for the rest of the elimination plan.** Moving any single `_rust_shims.c` function to Rust may cascade ud2 across its subsystem. Phases 2-4 below need the same caveat; they were estimated assuming the move-and-replace pattern works.

### Phase 2 — Macro materializations (~1 week)

~180 functions, mix of trivial and structural. Two passes:
1. **Trivial macros** (~80): `InitializeObjectAttributes`, `CONTAINING_RECORD`, `IsListEmpty`, `ObReferenceObject`, etc. Mostly 1-5 lines each. Inline as Rust helpers in subsystem modules; already partially done (`iomgr.rs` has its own `InitializeObjectAttributes`).
2. **Structural macros** (~100): HAL dispatch (`HalInitPnpDriver` → `HALDISPATCH->HalInitPnpDriver()`), wait macros (`KxMultiThreadWait`), IRP stack helpers. Per-subsystem; some need new Rust APIs (e.g. a `HalDispatch` accessor crate).

Cost: case-by-case, but well-bounded.

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

| Phase | Functions removed | Effort | Risk |
|-------|------------------:|--------|------|
| 0. Free wins | ~30 | 1 day | None |
| 1. FORCEINLINE batch | ~210 | 1 week + ~$10 API | Low (pilot first) |
| 2. Macro materializations | ~180 | 1 week | Low-medium (per-file) |
| 3. Calling conventions | ~28 | 3 days | Low (mechanical) |
| 4. Per-function review | ~50 | 2 weeks | Medium (each captures(none) is its own debug session) |
| 5. Document must-stays | ~0 | 1 day | None |
| **Total reduction** | **~498 / ~552 (90%)** | **~5 weeks** | |

End state: ~70 must-stay functions in kernel_c_helpers.c. ~9,238 LOC of shim C code → roughly ~1,500 LOC.

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
