# NTDLL C→Rust Translation Audit

> **Ground-truth re-verified 2026-06-14** against `dll/ntdll/CMakeLists.txt` + code — accurate: ntdll PE body 100% Rust; only active non-Rust inputs are the per-arch `dispatch/{i386,amd64,arm64}/dispatch.S` trap trampolines (asm by design), generated `ntdll_stubs.c` (spec2def syscall stubs), and the empty cohab stub TUs (`rtl_um_rust_stub.c`, `etwtrace_rust_stub.c`).

Created 2026-05-24. **Status (updated 2026-06-01): the ntdll PE body is
100% flipped to Rust and boots to `shell ready` on all 3 arches.** All
14 planned translation steps are done, and the last two leaf C files were
flipped this session (`nt_0600/ldr/ldrnotify.c` 154 LOC → ungated
`nt_0600/ldr/ldrnotify.rs`; `dispatch/dispatch.c` 89 LOC arm64 dispatchers
→ `#[cfg(aarch64)]` `dispatch/mod.rs`). The only C/asm left is the per-arch
`dispatch/<arch>/dispatch.S`, which **stays asm by design** (kernel→user
trap-entry trampolines). The
original scope/analysis below is preserved as the design record; the new
**[Current state](#current-state-2026-06-01)** and **[Remaining work +
future plans](#remaining-work--future-plans)** sections at the top and
bottom carry the live status. The user-mode substrate the smss-csrss track
built (`ntdll-sys`, rustconv `system_prompt_user.md` + `collapse_sys_crate`)
was reused as-is — ntdll did not need its own Phase 0.

Companion to:
- [smss-csrss-translation-audit.md](smss-csrss-translation-audit.md) — built the user-mode substrate (`ntdll-sys`, rustconv user-mode pipeline). **Hard dep.**
- [ntoskrnl-translation-audit.md](ntoskrnl-translation-audit.md) — pattern source (cohabitation, milestone-16 asm pre-processing, `verify_sizes`); also the kernel half of every kernel↔user contract this audit lists.
- [freeldr-translation-audit.md](freeldr-translation-audit.md) — direct precedent for **spec-driven user-mode DLL** cohabitation (kdcom, bootvid, **peloader**).
- [rtl-translation-audit.md](rtl-translation-audit.md) — the `sdk/lib/rtl/*.c` `Rtl*` API bodies (strings/security/env/process/path/…) → the `rtl-strings` crate, **and** the still-C exception engine / heap / `image.c` that ntdll cohabits with via FFI. **The authoritative reference for everything ntdll links from `librtl.a`.** (On branch `rtl-priv-port`.)
- [static-libs-translation-audit.md](static-libs-translation-audit.md) — `librtl.a` pure-algorithm rlibs (bitmap/avl/crc32/…) and `liblibcntpr.a`, both of which ntdll links and which are *explicitly out of scope here*.
- [pseh-elimination.md](pseh-elimination.md) — overall PSEH/DUMMY_PSEH strategy.
- [kernel-exception-handling.md](kernel-exception-handling.md) — the kernel half of the exception dispatch contract ntdll consumes.

## TL;DR

- **ntdll's own PE body is small: ~11.7K LOC across 14 files.** What
  *sounds* huge (ntdll links ~59K LOC of `rtl` + ~59K of `libcntpr`)
  is shared static-lib surface tracked separately. This audit scopes
  ntdll's own body only, mirroring `ntoskrnl-translation-audit.md`'s
  "source body / static libs" split.
- **No Phase 0.** The smss-csrss track already built every prerequisite:
  `sdk/lib/rust/ntdll-sys/` (cargo-clean ×3, 825/839/832 size assertions),
  the rustconv user-mode pipeline (`system_prompt_user.md`,
  `Target.sys_crate`, `collapse_sys_crate`,
  `strip_user_inline_redeclarations`, `ntdll-sys/src/inlines.rs`
  NtCurrentTeb/Peb/Process/Thread), the user-mode `#[panic_handler]`
  +`rust_eh_personality` template, and `csrss.exe` proves the cohabitation
  pattern end-to-end. **One small substrate delta is owed**: add
  `ntdll/include/ntdllp.h` (264 LOC) to the ntdll-sys wrapper for
  intra-loader `Ldrp*` resolve. Mirrors the `win32k-sys`-pulls-`win32kp.h`
  move.
- **Two hard cores, everything else is leaf→root ordinary work.**
  Core 1 = the kernel↔user boundary (exception dispatch + APC + win32k
  callback). Core 2 = the LDR bootstrap (`ldrinit.c`: runs before its own
  runtime exists, loads itself, holds the project's only ntdll-private
  synchronization primitives). The remaining 7K LOC across `ldrutils.c`/
  `ldrpe.c`/`ldrapi.c`/`verifier.c`/`libsupp.c`/`dbgui.c`/`etw/trace.c`
  is policy + import-resolution + leaves, structurally similar to (and
  with Rust precedent in) `ntoskrnl-rust/src/ARM3/sysldr.rs`.
- **30 `_SEH2_TRY` sites.** Same shape as smss-csrss (no `_SEH2_FINALLY`,
  no lock-cleanup) → smss-csrss method, not the win32k Pass-0/1/2/3
  machine and **not** the `pseh-elimination.md` "reverse shim" retrofit
  (that plan is for code staying C).
- **Smoke is already a perfect gate.** ntdll is on the boot spine — any
  break = no `shell ready`. Existing smoke catches every regression.
  Max blast radius (every process loads ntdll) but loud failure.
- **Cost.** ~$5–10 lifetime via rustconv (~14 files × ~$0.05 × 3–5 iters).
- **Two scoping decisions owed** (still open from the brainstorm session):
  (1) track placement — own audit/branch vs fold into the in-flight
  smss-csrss worktree; (2) timing — gate on libcntpr/rtl tracks landing
  more state. Neither is a substrate blocker; both are coordination
  calls.

## Current state (2026-06-01)

The track ran in worktree `vibeos-wt-ntdll`, branch `ntdll-rust-port`
(10 commits, audit steps 1→14). The Rust crate `sdk/lib/rust/ntdll-rust/`
(`libntdll_rust.a`, default features → ntdll.dll; `--features vista` →
ntdll_vista.dll) carries the whole flipped body. **All 3 arches cold-build
clean and smoke to `shell ready` (arm64 reaches `[EXPLORER] _tWinMain`).**

| File | LOC | Status | Commit |
|---|--:|---|---|
| `nt_0600/DllMain.c` | 31 | ✅ Rust (`nt_0600/dllmain.rs`) | 889740a |
| `nt_0600/ldr/ldrinit.c` (LdrpHeap) | 3 | ✅ Rust (`nt_0600/ldr/ldrinit.rs`) | ae60dfb |
| `rtl/uilist.c` | 40 | ✅ Rust (`rtl/uilist.rs`) | 528c0af |
| `rtl/version.c` | 231 | ✅ Rust (`rtl/version.rs`) | 9584d7a |
| `etw/trace.c` | 277 | ✅ Rust (`etw/trace.rs`, etwtrace lib kept as stub) | 0f48369 |
| `dbg/dbgui.c` | 387 | ✅ Rust (`dbg/dbgui.rs`) | 0f48369 |
| `ldr/verifier.c` | 745 | ✅ Rust (`ldr/verifier.rs`) | 9a0c989 |
| `rtl/libsupp.c` | 1,261 | ✅ Rust (`rtl/libsupp.rs`) | 5a46a15 |
| `ldr/ldrpe.c` | 1,281 | ✅ Rust (`ldr/ldrpe.rs`) | 9dcab30 |
| `ldr/ldrutils.c` | 2,799 | ✅ Rust (`ldr/ldrutils.rs`) | cb04507 |
| `ldr/ldrinit.c` (bootstrap) | 2,735 | ✅ Rust (`ldr/ldrinit.rs`) | 92c31cc |
| `ldr/ldrapi.c` | 1,666 | ✅ Rust (`ldr/ldrapi.rs`) — **cold-green ×3, COMMITTED b3252b81c5a** | b3252b81c5a |
| `nt_0600/ldr/ldrnotify.c` | 154 | ✅ Rust (`nt_0600/ldr/ldrnotify.rs`, ungated) — **COMMITTED b3252b81c5a** | b3252b81c5a |
| `dispatch/dispatch.c` | 89 | ✅ Rust (`dispatch/mod.rs`, `#[cfg(aarch64)]`) — arm64 only (KiRaise/KiUserCallbackDispatcher); **COMMITTED b3252b81c5a** | b3252b81c5a |
| `dispatch/{i386,amd64,arm64}/dispatch.S` | 708 | ⏸ **stays asm** (cmake-compiled, untouched) | — |

**ntdll PE body 100% Rust (committed b3252b81c5a); only the per-arch `dispatch.S` asm remains C by design.**

### How reality diverged from the plan

- **No Phase 0 / substrate delta as written.** `ntdllp.h` was **not** added
  to the `ntdll-sys` wrapper, and `ldrtypes.h`/bindgen-regen was not the
  path. Instead the loader's intra-component types/consts/globals were
  supplied **per-file at integration** (shared header-type modules, inline
  const/extern blocks, truncated-mirror reconciliation) — see
  `feedback_rustconv_header_types_integration`. Cheaper than a wrapper
  rebuild; the cost moved into per-file integration.
- **ABI enforcement is target-agnostic, not `abi_map_ntdll.py`.** Commit
  5a46a15 added a generic ABI enforcer rather than an ntdll-specific
  `abi_map`. The 4 must-not-drift exports are still validated by `nm`
  /smoke (process init crashes without them).
- **Dispatchers stayed plain `.S` in the C build.** The doc's plan to
  pre-process them into `ntdll-rust/data/asm/*.s` + a `build.rs` was **not**
  done — there is no `build.rs`/`data/asm/`. The `dispatch/*.S` files are
  still compiled by cmake unchanged, which is functionally identical (asm
  stays asm) and avoids the cc-rs plumbing. The DUMMY_PSEH safety-net
  blocks therefore live in the original `.S` (untouched), not a checked-in
  pre-processed copy.
- **`rtl_um` → `rtl`.** The crate module is `ntdll-rust/src/rtl/`, not the
  planned `rtl_um/`. etw was absorbed (Option 1) but the `etwtrace` cmake
  lib is kept as a stub (`etwtrace_rust_stub.c`) rather than deleted.

### Bugs found + fixed during the flip (reusable lessons)

- **`ldrapi.rs` i386 hang in `LdrUnloadDll`** (msacm32 DllMain unload path):
  `write_bytes(addr_of_mut!(ActCtx.Frame), 0, size_of::<…>())` missing the
  `as *mut u8` cast → wrote `size_of()²` bytes → stack overflow → hang.
  i386-only (stack-layout-exposed). See `feedback_write_bytes_element_count`.
- **ntdll_vista.dll cold-link cascade.** Flipping `LdrDisableThreadCalloutsForDll`
  (referenced by nt_0600 `DllMain`) into the shared staticlib made the
  companion's cold link pull the whole Rust loader + 20 undefined C callees.
  Fix: `#[cfg(not(feature = "vista"))] pub mod ldr;` in lib.rs (companion
  imports the loader from ntdll.dll). See
  `feedback_cohab_flip_companion_dll_cascade`. **Warm builds masked it** —
  only cold `--clean` exposed it; always cold-verify a cohab flip.
- **FORCEINLINE link gaps:** `NtCurrentPeb`/`RtlInitEmptyUnicodeString` have
  no exported symbol → needed inline Rust shadows (mirror ldrinit.rs).
- **scanner SAL bug** dropped `LdrGetProcedureAddress` +
  `LdrQueryProcessModuleInformation` — hand-recovered as trivial wrappers.
  See `feedback_rustconv_scanner_sal_callback_preproc`.

## Scope — and what's out of scope

**In scope** — `dll/ntdll/` (the PE body of `ntdll.dll`):

```
ldr/ ........... PE loader: imports, KnownDlls, SxS, loader lock, TLS
                 ldrapi.c  1,666  public Ldr* spec exports
                 ldrinit.c 2,735  the bootstrap (last to translate)
                 ldrpe.c   1,281  import descriptor walk + snap
                 ldrutils.c 2,799 dependency graph + map DLL + KnownDlls
                 verifier.c   745 Application Verifier shim
rtl/ ........... user-mode rtl glue (NOT shared rtl)
                 libsupp.c 1,261  rtl callbacks: RtlpGetStackLimits,
                                  RtlpAllocateMemory, atom-table lock, …
                 version.c   231
                 uilist.c     40
dispatch/ ...... kernel→user entry trampolines (asm) + ARM64 C path
                 dispatch.c       89  KiRaise/KiUserCallback (arm64 + fallback)
                 i386/dispatch.S 302
                 amd64/dispatch.S 331
                 arm64/dispatch.S  75
dbg/dbgui.c .... 387             debug-port helpers (DbgUi*)
etw/trace.c .... 277             ETW user-mode tracing (separate libetwtrace)
nt_0600/ ....... ntdll_vista.dll — SEPARATE module (see below)
                 DllMain.c        31
                 ldr/ldrinit.c     3
                 ldr/ldrnotify.c 154
```

**Out of scope — explicitly:**

- `sdk/lib/rtl/` (~59K LOC C, ntdll links `librtl.a`). Tracked under
  **`rtl-translation-audit.md`** (the `Rtl*` API bodies → `rtl-strings`
  crate; as of 2026-06-14 essentially all of `sdk/lib/rtl/*.c` is flipped
  — incl. `registry.c`, the `heap.c` family, and `image.c` — only the
  variadic glue + per-arch asm floor remains C) and
  `static-libs-translation-audit.md` (the algorithm rlibs). The
  interesting exception engine — `RtlDispatchException`,
  `RtlUnwind`, `RtlRaiseException`, frame walking, `.pdata`/`.xdata` on
  amd64/arm64, `fs:[0]` chain walk on i386 — lives here, **not** in
  ntdll. ntdll-rust cohabits with C `RtlDispatchException` via ntdll-sys
  FFI; the rtl exception engine being C is fine and does not block this
  audit.
- `sdk/lib/crt/` libcntpr (~59K LOC). Tracked under `cnt_rust_prep`;
  43 .c already Rust as of 2026-05.
- `sdk/lib/uuid`, `sdk/lib/cryptlib`, `sdk/lib/setjmp`, `sdk/lib/apisets`,
  `sdk/lib/pseh`. Static deps, separate tracks.
- `subsystems/csr/csrlib`. Tracked under smss-csrss audit.
- `ntoskrnl/ntdll.S` (kernel-side syscall asm stubs) and
  `${BIN}/ntdll_stubs.c` (spec2def-generated user-side syscall stubs).
  Both are **mechanically generated**, not hand-written C — not a
  translation target.

**ntdll_vista.dll is its own thing.** Reading `dll/ntdll/nt_0600/CMakeLists.txt`:
nt_0600 produces a *separate* PE module (`ntdll_vista.dll`) with its own
`.spec` and `DllMain`, not a static lib linked into `ntdll.dll`. It
contains the Vista-era LDR notification API (`LdrRegisterDllNotification`,
etc.) — ~190 LOC total. Translation of `ntdll.dll` and `ntdll_vista.dll`
are independent; this audit covers both because they share infrastructure,
but they flip independently.

## Where this fits

| Dependency | Status | Blocking? |
|---|---|---|
| `ntdll-sys` substrate | Done in smss-csrss worktree (commits 06396c51 → 2a8d6e27) | No — reuse as-is |
| rustconv user-mode pipeline | Done in smss-csrss (commit 8eb28d57) | No — reuse as-is |
| User-mode `#[panic_handler]`/`rust_eh_personality` template | Done in smss-csrss (commit e0049bff) | No — copy from `smss-csrss-rust` |
| `librtl.a` exception engine in Rust | **Done** (2026-06-11, `rtl-except`) | No — was never a blocker; ntdll FFIs it either way |
| `librtl.a` heap (`RtlAllocateHeap`/`RtlFreeHeap`) | **Done** (`rtl-heap`/`rtl-heappage`/`rtl-heapuser`) | No — ntdll calls it via FFI either way |
| `liblibcntpr.a` printf-family | Not done (43/X done) | **No** — same cohabitation |
| win32k Rust | Not done (mid-flight) | **No** — only ntdll touch-point is `KiUserCallbackDispatcher`, kernel side already Rust |
| smss/csrss/csrsrv Rust | Mid-flight | **No** — those *consume* ntdll; ntdll body is independent of consumers |

Ntdll body translation is **not gated on any of the in-flight tracks
landing more state**. Coordination is timing, not dependency.

## Today's binaries + source layout

| File | LOC | Tier | Role | Risk |
|---|--:|:--:|---|---|
| `nt_0600/DllMain.c` | 31 | 0 | ntdll_vista entrypoint, trivial | Tiny |
| `nt_0600/ldr/ldrinit.c` | 3 | 0 | empty stub | Trivial |
| `rtl/uilist.c` | 40 | 0 | RtlInitializeAtomPackage helper | Tiny |
| `rtl/version.c` | 231 | 0 | RtlVerifyVersionInfo | Tiny |
| `etw/trace.c` | 277 | 0 | EtwEventRegister/Write user-mode | Tiny, in its own `libetwtrace.a` |
| `dispatch/dispatch.c` | 89 | 0 | arm64 + non-x86 C dispatchers | Tiny |
| `dbg/dbgui.c` | 387 | 0 | DbgUi* — debug-port helpers | Low |
| `nt_0600/ldr/ldrnotify.c` | 154 | 1 | Vista LDR notification API | Low |
| `ldr/verifier.c` | 745 | 1 | Application Verifier shim — optional path | Low |
| `rtl/libsupp.c` | 1,261 | 1 | **rtl↔ntdll glue** (see §libsupp.c) | Medium |
| `ldr/ldrpe.c` | 1,281 | 2 | Import descriptor walk + LdrpSnapThunk | Medium |
| `ldr/ldrapi.c` | 1,666 | 2 | Public `Ldr*` spec exports | Medium |
| `ldr/ldrutils.c` | 2,799 | 2 | KnownDlls, dep graph, LdrpLoadDll, LdrpMapDll | High |
| `ldr/ldrinit.c` | 2,735 | 3 | **Bootstrap — translate LAST** | Highest |
| `dispatch/{i386,amd64,arm64}/dispatch.S` | 708 | asm | Kernel-defined entry trampolines + DUMMY_PSEH patches | **Stays asm** |
| **Total translatable C** | **~11,700** |  |  |  |
| **Asm (stays asm, pre-processed)** | **708** |  |  |  |

Tier 0 = trivial leaves, prove the spec-driven DLL cohabitation pipeline.
Tier 1 = medium leaves, non-bootstrap, can flip in any order.
Tier 2 = the LDR crown — file-by-file, `rustconv translate` not `batch`
(>1000 LOC; `feedback_rustconv_translate_vs_batch`).
Tier 3 = the bootstrap, last.

## Translation surface — the layered analysis

**ntdll's PE loader is *not* monolithic novel code.** Three PE loaders
exist in the project, layered over one shared format primitive layer:

```
shared format primitives — sdk/lib/rtl/image.c (now Rust: rtl-diag/src/image.rs)
  RtlImageNtHeader · RtlImageDirectoryEntryToData · RtlImageRvaTo{Section,Va}
  LdrProcessRelocationBlockLongLong · LdrRelocateImage · LdrRelocateImageWithBias
        ▲                       ▲                       ▲
        │                       │                       │
   freeldr-rust          ntoskrnl-rust            dll/ntdll/ldr/    ← THIS
   /peloader.rs          /src/ARM3/sysldr.rs      (now Rust:
   loads ntoskrnl        loads driver SYS          ntdll-rust/src/ldr/)
   (Rust, 6a48458d)      (Rust, in tree)
```

The relocation/import-format-parse math used to be one shared C layer
ntdll cohabited with via FFI; as of 2026-06-14 that shared layer
(`sdk/lib/rtl/image.c`) is itself Rust (`rtl-diag/src/image.rs`), and
it's *already Rust in three independent places* — all ports are
reference implementations to mine for patterns.

The ntoskrnl Rust precedent is the closest structural analog:

| ntdll C (target) | ntoskrnl Rust precedent | Reuse |
|---|---|---|
| `LdrpSnapThunk` (ldrpe.c:979) | `MiSnapThunk` (ARM3/sysldr.rs:578) | Direct |
| `LdrpWalkImportDescriptor` (ldrpe.c:712) | `MiResolveImageReferences` (sysldr.rs:1466) | Direct |
| `LdrpInsertMemoryTableEntry` (ldrutils.c:1569) | `MiProcessLoaderEntry` (sysldr.rs:517) | Direct |
| `LdrpMapDll` § (ldrutils.c:997) | `MiLoadImageSection` (sysldr.rs:81) | Partial (section mapping; not the user-mode search/SxS layer) |
| `LdrpFetchAddressOfEntryPoint` (ldrutils.c:767) | `MmLoadSystemImage` body | Direct |

**The genuinely novel part — what neither freeldr-rust nor ntoskrnl-rust
has — is the *user-mode loader policy*** in `ldrutils.c`:

- `LdrpCheckForKnownDll` — the `\KnownDlls\*` section cache (kernel
  pre-mapped system DLLs)
- `LdrpResolveDllName` / `LdrpResolveFullName` / `LdrpSearchPath` —
  full Windows DLL search path semantics
- `RtlActivateActivationContext` / SxS / `LdrpInitializeProcessCompat` —
  side-by-side assemblies, manifests, shim engine entrypoints
- `LdrpInitializeDotLocalSupport` — `.local` redirection
- `LdrpLoadDll` / `LdrpUpdateLoadCount` — the **refcounted recursive
  dependency graph** serialized by the loader lock
- `LdrpCallInitRoutine` / `LdrpCallTlsInitializers` —
  `DllMain(DLL_PROCESS_ATTACH)` and TLS callbacks, with reentrancy
  protection under the loader lock
- `LdrpLoadShimEngine` — apphelp shim engine plumbing

This is also where ntdll's security value lives: DLL hijacking,
preloading vulnerabilities, SxS confusion, `.local`-based privilege
issues. The translation case is strongest precisely where the novelty
is highest. **Novelty-weighted, ldrutils.c is the largest *actually new*
file in the audit, not ldrinit.c.** ldrinit.c is bigger LOC-wise but
its risk is bootstrap-ordering (one-shot, single threaded), not
algorithmic novelty.

## Core 1 — The kernel↔user boundary

The exception/APC/callback contract between the kernel and ntdll.
Critical: **the kernel side is already Rust and smoke-green.**
Translating ntdll's body does not renegotiate the contract.

### Full round-trip

```
User mode faults
   │
   ▼
kernel trap → KiDispatchException ............. Rust, ke/{i386/exp,amd64/except}.rs ✅
   builds CONTEXT + EXCEPTION_RECORD + MACHINE_FRAME on USER stack;
   sets TrapFrame.{Rip,Eip} = KeUserExceptionDispatcher .... Rust kernel sets ptr ✅
      (resolved from ntdll exports at PspMapSystemDll;
       ps/psmgr.rs:149-165 binds the three KeUser*Dispatcher statics)
   ▼
iret to user — execution resumes inside ntdll
   │
   ▼
KiUserExceptionDispatcher ..................... ntdll/dispatch/{i386,amd64,arm64}/dispatch.S
   ├─ call RtlDispatchException ............... rtl/{i386,amd64,arm64}/except.c (C, librtl)
   │      handled  → ZwContinue (syscall)
   │      unhandled → ZwRaiseException (syscall, 2nd chance)
   │      DUMMY_PSEH safety net: see below
   └─ on nested fault: KiUserExceptionDispatcherNested → RtlRaiseException

Async path (APC delivery to user mode):
PspUserThreadStartup → KiInitializeUserApc(...PspSystemDllEntryPoint) .. Rust ✅
   ▼ delivers fake APC
LdrInitializeThunk → LdrpInit ........... ntdll dispatch.S + ldr/ldrinit.c

win32k callback path:
kernel KeUserModeCallback → builds UCALLOUT_FRAME, sets RIP
   ▼
KiUserCallbackDispatcher .............. dispatch.S
   ├─ PEB->KernelCallbackTable[Index](buf, len)
   └─ ZwCallbackReturn(NULL, 0, result)
```

### Must-not-drift exports (kernel resolves by name)

Four ntdll exports the kernel binds at `PspMapSystemDll` and stores in
`KeUser*Dispatcher` / `PspSystemDllEntryPoint`. Their exact `.spec`
decoration and ABI are load-bearing:

1. `KiUserExceptionDispatcher` — exception trampoline
2. `KiUserApcDispatcher` — user-APC trampoline
3. `KiUserCallbackDispatcher` — win32k callback trampoline
4. `LdrInitializeThunk` — first user instruction in every new thread

This is the same load-bearing-export class as the FreeLDR-DLL precedent's
`PeLdr*` set (`feedback_freeldr_dll_translation`). Discipline applies:
`nm` the cargo-built archive against the `.spec` expected decoration
*before* the cohabitation flip lands.

### Dispatchers stay asm — and carry non-upstream patches

The trampolines use ml64 SEH unwind pseudo-ops (`.PUSHFRAME`,
`.ALLOCSTACK CONTEXT_FRAME_LENGTH`, `.SAVEREG`, `.SAVEXMM128 xmm6..15`,
MACHINE_FRAME) and exact kernel-built stack contracts. `global_asm!`
can't cleanly express these; same wall as
`feedback_hal_asm_not_global_asm_reducible`. Pattern is the **ntoskrnl
milestone-16 treatment**: pre-process out-of-band via a Python helper
(`tools/gen_preprocessed_asm.py` analog), check in
`sdk/lib/rust/ntdll-rust/data/asm/dispatch-{i386,amd64,arm64}.s`,
compile via cc-rs with `-isystem sdk/include/crt` only.

**Critically — preserve the DUMMY_PSEH safety-net blocks bit-for-bit.**
Both `dispatch/i386/dispatch.S` and `dispatch/amd64/dispatch.S` carry
hand-added `#ifdef _USE_DUMMY_PSEH` / `#if defined(_USE_DUMMY_PSEH) ||
defined(_USE_PSEH_CLANG)` blocks **not present in upstream ReactOS**:

- Swallow continuable software exceptions (code `< 0xC0000000`) by
  `ZwContinue` resume.
- Pass through hardware exceptions (`>= 0xC0000000`) and noncontinuable
  ones to second-chance via `ZwRaiseException`.
- `STATUS_BREAKPOINT` (`0x80000003`) special case: `inc qword ptr
  [rsp + CxRip]` to skip the int3 byte and avoid re-execution loops
  (amd64); i386 has equivalent logic.

These are the user-mode mirror of the kernel-side DUMMY_PSEH swallow
(`docs/engineering/pseh-elimination.md` §"RtlRaiseException swallowing
fixes on amd64/i386"). They are load-bearing for boot under llvm-mingw
DUMMY_PSEH. **The pre-processed checked-in `.s` files must include
these blocks unconditionally** (DUMMY_PSEH is always on for our build) —
audit explicitly flags this so they aren't "cleaned up" as cruft.

### Boundary inventory (what ntdll-rust FFIs across)

| Direction | Surface | Mechanism | Rust status |
|---|---|---|---|
| kernel → ntdll | `KiUser{Exc,Apc,Callback}Dispatcher`, `LdrInitializeThunk` | Kernel sets user RIP + builds frame on user stack | Kernel **Rust ✅**; ntdll **stays asm** |
| ntdll → kernel | `NtContinue`, `NtRaiseException`, `ZwContinue`, `ZwRaiseException`, `ZwCallbackReturn`, `NtTestAlert`, `NtQueryVirtualMemory`, `ZwDelayExecution`, `NtCreateSection`/`NtOpenSection`/`NtMapViewOfSection`, … | `int 2E`/`sysenter`/`svc` stubs auto-generated from `ntdll.spec` via `spec2def` → `ntdll_stubs.c` | Stubs **asm-generated, not translated**; ntdll-rust calls them as `extern "system"` FFI declared via `ntdll-sys` |
| ntdll → rtl | `RtlDispatchException`, `RtlRaiseException`, `RtlRaiseStatus`, `RtlAllocateHeap`, `RtlFreeHeap`, `RtlImageNtHeader`, `LdrRelocateImageWithBias`, `RtlImageDirectoryEntryToData`, … (~50 surface fns) | Plain call into linked `librtl.a` | rtl **stays C** (cohabitation); ntdll-rust FFI via ntdll-sys bindings |
| rtl → ntdll | `RtlpAllocateMemory`/`RtlpFreeMemory`, `RtlpGetStackLimits`, `RtlAcquirePebLock`/`RtlReleasePebLock`, `RtlpHandleDpcStackException`, `RtlpGetMode`, atom-table locks (~20 callbacks) | Plain call back into ntdll | These live in **`dll/ntdll/rtl/libsupp.c`** — see §libsupp.c. The boundary the user-mode rtl walks when it needs the *process* state. |
| kernel ↔ ntdll (win32k hot path) | `KeUserModeCallback` → `KiUserCallbackDispatcher` → `PEB->KernelCallbackTable[Index]` → `ZwCallbackReturn` | PEB-resolved callback table | Dispatcher **stays asm**; the only ntdll piece on the win32k path |

## Core 2 — The bootstrap

`ldrinit.c`'s `LdrpInit` (line 2608) and `LdrpInitializeProcess`
(line 1785) run before the process has a runtime. Three compounding
hazards.

### Hazard A — runs before its own runtime exists

The kernel fabricates a user APC into `LdrInitializeThunk` as the
**very first user instruction in every thread** (`ps/thread.rs:206`
→ `KiInitializeUserApc(... PspSystemDllEntryPoint ...)`). The thunk
zeroes the frame pointer (`xor ebp,ebp` / `mov x29,#0`), grabs the
APC's CONTEXT pointer, jumps to `LdrpInit`.

`LdrpInit` runs with **nothing initialized**:
- `Peb->Ldr == NULL` (no loaded-module list)
- No heap (`RtlCreateHeap` doesn't happen until inside
  `LdrpInitializeProcess`, ~line 2056 of ldrinit.c)
- No loader lock (`Peb->LoaderLock` is set on line 2669, *after* entry)
- No TLS (`LdrpInitializeTls` runs near the end of process init)

Everything before `RtlCreateHeap` **cannot allocate**, cannot touch the
module list, cannot acquire the loader lock — it's the code that
*creates* them. User-mode twin of the kernel's `KiSystemStartup`
"no MM yet" zone.

**Mitigation is structural and free under cohabitation:**
- `ldrinit.c` is Tier 3 — translated **last**.
- The bootstrap prefix keeps its C until everything around it is Rust
  and smoke-proven.
- Per-function cohabitation means even within `ldrinit.c`, `LdrpInit` +
  `LdrpInitializeProcess` can stay C while smaller helpers
  (`LdrpInitializeTls`, `LdrpInitializeExecutionOptions`,
  `LdrpInitializeProcessCompat`, `LdrpInitializeDotLocalSupport`,
  `LdrpInitializeThread`) flip individually.
- The no-alloc prefix is **the** function (`LdrpInitializeProcess`
  lines 1785–2056) — once `RtlCreateHeap` returns, normal allocation
  is available.

### Hazard B — loads itself

`LdrpInitializeProcess` inserts ntdll's *own* `LDR_DATA_TABLE_ENTRY`
into `PEB->Ldr` (the `InLoadOrderModuleList`/`InMemoryOrderModuleList`/
`InInitializationOrderModuleList`), and `ldrpe.c`/`ldrutils.c` are the
code resolving imports — including, recursively, more DLLs.

The chicken-and-egg is resolved by the **kernel** section-mapping ntdll
fully-relocated at `PspMapSystemDll` *before* `LdrpInit` runs. ntdll
never relies on its own loader to fix itself up. For the Rust port this
is benign: the `#[no_mangle]` symbols sit in the same PE, relocated by
the same kernel map.

**But:** the 4 must-not-drift exports above (`LdrInitializeThunk`, three
`KiUser*Dispatcher`) are resolved by the kernel **by name** at
`PspMapSystemDll`. If cohabitation drift moves one, the kernel can't find
it. Apply the FreeLDR-DLL `nm`-the-archive-vs-`.spec` discipline
(`feedback_freeldr_dll_translation`) with these 4 as the frozen set —
no exceptions.

### Hazard C — bootstrap synchronization is ntdll-private static data

`LdrpProcessInitialized` (a `LONG`, manipulated by
`_InterlockedCompareExchange`) and `LdrpLoaderLock` (a
`RTL_CRITICAL_SECTION` struct, exposed via `Peb->LoaderLock = &LdrpLoaderLock`)
are statics in ntdll's `.data`. They're the *only* synchronization
primitives available during early bootstrap, used by every DllMain
sequencing decision in every process forever.

Translating these is squarely the `feedback_volatile_static` /
`feedback_pwchar_pointer_export` hazard family:

- `#[no_mangle] static mut LdrpProcessInitialized: LONG` — wrong section
  (`.bss` vs `.data`) or wrong alignment silently corrupts the
  interlocked CAS. Must match C struct layout bit-for-bit.
- `#[no_mangle] pub static mut LdrpLoaderLock: RTL_CRITICAL_SECTION` —
  layout must match C struct. Use bindgen `_RTL_CRITICAL_SECTION` from
  ntdll-sys; assert size+offsets with a `verify_sizes`-style check.
- Atomic op ABI: Rust `core::sync::atomic::AtomicI32::compare_exchange`
  emits `lock cmpxchg` matching C `_InterlockedCompareExchange` for both
  i386 + amd64 (verified pattern elsewhere in the kernel port). Do NOT
  hand-roll inline asm here.

A bug here would manifest as a non-deterministic boot hang or
intermittent DllMain re-entrancy crash — far worse than a clean failure.

**Treat both as keystone-module statics** (one canonical Rust definition,
no duplicates across rlibs), and add a startup-time
`size_of::<LdrpLoaderLock>() == sizeof(RTL_CRITICAL_SECTION)` const
assertion.

### What the contract requires

The kernel half of the bootstrap is fully Rust + smoke-green
(`PspUserThreadStartup` → `KiInitializeUserApc` → fake APC →
`LdrInitializeThunk`). The contract a translated ntdll must honor is
frozen and minimal:

> Be a PE whose `LdrInitializeThunk` export, entered via a user APC
> carrying a CONTEXT pointer in arg0, eventually returns into the image
> entry point with `Peb->Ldr` populated, `Peb->LoaderLock` initialized,
> heap created, and TLS initialized.

The asm thunk + kernel Rust hold that contract. Translating `ldrinit.c`'s
*body* never renegotiates it.

## libsupp.c — the rtl↔ntdll glue boundary

`dll/ntdll/rtl/libsupp.c` (1,261 LOC, built as an OBJECT lib `rtl_um`)
is the explicit user-mode boundary surface: every callback the *shared*
rtl needs from the *user-mode-specific* environment. Reading the source,
the major buckets:

| Bucket | Functions | Why ntdll-private |
|---|---|---|
| **Heap glue** | `RtlpAllocateMemory`, `RtlpFreeMemory`, `RtlpSetHeapParameters` | rtl algorithms (atom table, registry helpers) call these; ntdll provides the user-mode heap via `RtlAllocateHeap`. Kernel rtl uses pool here instead. |
| **PEB lock** | `RtlAcquirePebLock`, `RtlReleasePebLock`, `RtlGetCurrentPeb` | PEB is a user-mode-only structure; rtl that touches it (path normalization, environment) calls these. Implemented via critical section. |
| **Stack discovery for SEH** | `RtlpGetStackLimits`, `RtlpCaptureStackLimits`, `RtlWalkFrameChain` | **The hot one.** `RtlDispatchException`'s frame walker calls `RtlpGetStackLimits` to know where the user stack ends. User-mode mirror of the kernel-side probe surface. |
| **Atom table lock** | `RtlpInitAtomTableLock`, `RtlpLockAtomTable`, `RtlpUnlockAtomTable`, `RtlpAllocAtomTable`, `RtlpFreeAtomTable`, `RtlpCreateAtomHandle`, `RtlpFreeAtomHandle`, `RtlpGetAtomEntry`, `RtlpAllocAtomTableEntry`, `RtlpFreeAtomTableEntry`, `RtlpDestroyAtomHandleTable` | The atom table algorithm lives in rtl; the user-mode lock impl + handle table lives here. |
| **Mode discovery** | `RtlpGetMode`, `RtlpCheckForActiveDebugger`, `RtlpSetInDbgPrint`, `RtlpClearInDbgPrint` | rtl has shared kernel/user code; these tell it which side it's on. |
| **Exception path callbacks** | `RtlpHandleDpcStackException`, `RtlpCheckLogException` | rtl's `RtlDispatchException` calls these during the user-mode walk. |
| **Loader-callout introspection** | `RtlIsThreadWithinLoaderCallout` | Used by rtl's reentrancy guards. |
| **SxS** | `RtlDosApplyFileIsolationRedirection_Ustr` | The path-redirection entry point (the rest of SxS is in shared rtl actctx.c). |
| **Misc** | `RtlImageNtHeaderEx` (duplicate of the shared rtl/image.c one — historical), `RtlPcToFileHeader`, `RtlGetNtGlobalFlags` | |

**Translation implication:** libsupp.c is a Tier 1 file (1.3K LOC, medium
risk), but it's the **most semantically entangled with rtl** of any
ntdll file. Every function here is a public API rtl will FFI back into
once libsupp.c is Rust. Two consequences:

1. **ABI surface is wider than the spec exports.** These functions are
   exported (in `ntdll.spec`) and *also* called from `librtl.a` —
   intra-library call sites must keep the exact decoration. `verify_abi`
   discipline (`feedback_extern_c_abi`) is critical here.
2. **The `RtlpAllocateMemory`/`RtlpGetStackLimits` pair is bootstrap-
   adjacent.** rtl calls these any time it needs heap or walks a stack.
   They must work *while* `LdrpInitializeProcess` is constructing the
   heap. Probably translate as among the last Tier 1 files — confirm
   they're never called before `RtlCreateHeap` returns.

## SEH

30 `_SEH2_TRY` sites total, all in the loader + libsupp:

| File | Sites |
|---|--:|
| `ldr/ldrinit.c` | 9 |
| `ldr/ldrapi.c` | 7 |
| `ldr/ldrutils.c` | 6 |
| `rtl/libsupp.c` | 3 |
| `ldr/verifier.c` | 2 |
| `ldr/ldrpe.c` | 2 |
| `nt_0600/ldr/ldrnotify.c` | 1 |

No `_SEH2_FINALLY`. No lock-cleanup SEH. Almost all are "probe a
PE-header / user buffer, on fault return `STATUS_*`" shape — the same
class as smss-csrss's 19 sites and unlike the win32k 282-site /
lock-cleanup-heavy class.

**Method = smss-csrss, not win32k.** Under DUMMY_PSEH the handlers are
dead code (`_USE_DUMMY_PSEH=1` per `sdk/cmake/gcc.cmake:65`). When
translating, the `_SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) { return
status; }` becomes a Rust `Result<_, NTSTATUS>` early-return path — the
LLM handles this directly once the per-file shape is annotated for the
rustconv prompt. No Pass-0/1/1b/2/2b/3 machine needed.

**Method ≠ reverse-shim retrofit.** `pseh-elimination.md` lists DLLs
(47 files) → reverse-shim calls as Priority 5, but that plan is for
code *staying C*. ntdll is going to Rust; reverse shims are not the
target shape.

One nuance: `LdrpInit`'s outer `_SEH2_TRY` (ldrinit.c:2675) wraps the
entire `LdrpInitializeProcess` call — its `_SEH2_GetExceptionCode()` is
the canonical way the bootstrap reports process-init failure to
`ZwRaiseHardError`. This is the one site where "swallowed exception
becomes a `Result<NTSTATUS>`" maps to a real behavioral test — make sure
the translation preserves the `STATUS_APP_INIT_FAILURE` hard-error
escalation.

## Substrate — what we have, what we need

### We have (built by smss-csrss, reusable as-is)

`sdk/lib/rust/ntdll-sys/` (i386 1,257 fns / 991 structs / 11,367 consts;
amd64/arm64 comparable), cargo-clean ×3 on
`i686/x86_64-pc-windows-gnu + aarch64-pc-windows-gnullvm`,
`verify_sizes.py` ×3 (825/839/832 compile-time size assertions). Wrapper
roots at the resolved 8-header set:
`ndk/ntndk.h + sm/ns.h + sm/smmsg.h + csr/csr.h + csrmsg.h + csrsrv.h
+ win/base.h + win/basemsg.h`. Inlines module provides `NtCurrentTeb`/
`NtCurrentPeb`/`Nt/ZwCurrent{Process,Thread}` pseudo-handles.

Rustconv user-mode pipeline:
- `templates/system_prompt_user.md`
- `Target.sys_crate`/`Target.sys_umbrella`/`Target.system_prompt_addendum`
- `postprocess.collapse_sys_crate` (rewrites kernel `ntkrnl_sys::*` →
  single `use ntdll_sys::nt::*;`)
- `postprocess.strip_user_inline_redeclarations` (drops LLM extern
  redecls of NtCurrent* + bogus hardcoded `struct PEB`/`TEB`)
- `postprocess_bindgen.add_enum_deprefix_aliases` (~1,500/arch bare-name
  aliases for type-prefixed constified enums)

`smss-csrss-rust` cohabitation template: user-mode `#[panic_handler]`
(`NtTerminateProcess`, not privileged `hlt`/`wfe`) + `rust_eh_personality`
+ `crate-type = ["staticlib"]`. csrss.exe (commit `e0049bff`) is the
proof-of-pattern.

### We need (one small delta)

**Add `dll/ntdll/include/ntdllp.h` (264 LOC) to the ntdll-sys wrapper.**
The current substrate covers the SM/CSR/BASE protocol surface for
*consumers* of ntdll (smss/csrss/csrsrv/basesrv). Translating ntdll's
own LDR needs ntdll's *internal* headers so the rustconv resolver sees
intra-component symbols (`LdrpProcessInitialized`, `LdrpLoaderLock`,
`LdrpInLdrInit`, the various `LDRP_*` structs and constants). This is
the exact `win32k-sys`-pulls-`win32kp.h` move from Phase 0.3 of the
smss-csrss track.

Concrete change:
- Edit `sdk/lib/rust/ntdll-sys/wrapper.h`: add `#include <ndk/ldrtypes.h>`
  and `#include <ntdll/include/ntdllp.h>`.
- Edit `sdk/lib/rust/ntdll-sys/generate_bindings.sh`: add
  `-I dll/ntdll/include` to the bindgen flags.
- Broaden `FN_ALLOW` in postprocess for `Ldrp*`/`Rtlp*` (lowercase-second-
  segment internal prefixes — same move smss-csrss made for `Smp*`/
  `Basep*`/`Csrp*`).
- Regenerate bindgen ×3 + `verify_sizes` ×3.

**~half a session, not a Phase 0.** Mirror of the smss-csrss Phase 0.3
delta.

Also owed (not a blocker): add a Rust `RTL_CRITICAL_SECTION` keystone
module + size assertion for the bootstrap-private `LdrpLoaderLock` static.

## ABI surface — `ntdll.spec`

1,920 entries total. **Important: most are not hand-coded fns to
translate.** Breakdown (verified at audit time):

| Class | Count | Notes |
|---|--:|---|
| Nt* / Zw* syscall stubs | ~250–300 | Auto-generated by spec2def → `ntdll_stubs.c` + the kernel-side `ntoskrnl/ntdll.S` macro expansion. Not hand-C. |
| Rtl* (most) | ~600 | Implemented in **shared `librtl.a`**, forwarded through ntdll. Out of scope (rtl track). |
| Vista+ stubs (`-stub -version=0x600+`) | ~150 | Mostly no-op stubs in `ntdll_vista.dll`. |
| **Hand-coded ntdll-specific** (`Ldr*`, `DbgUi*`, `KiUser*`, `Ki{Raise,Int,Fast}*`, `RtlpInitial*`) | **74** | Direct grep: `awk '/^@/' ntdll.spec \| grep -cE '\b(Ldr[A-Z]\|DbgUi\|KiUser\|KiRaise\|KiInt\|KiFast\|RtlpInitial)[A-Za-z_]+'`. **The real translation export surface.** |
| Other (csrss client API forwards, A_SHA*, etc.) | balance | Mostly thin user-mode wrappers, some forwarded to other libs. |

**The translation surface = ~74 hand-coded ntdll-specific exports +
internal `Ldrp*`/`Rtlp*` helpers** in `dll/ntdll/*.c` (~11.7K LOC). The
1,920-entry spec is misleading as a size proxy.

### ABI enforcement

i386 stdcall decoration (`_Name@N`) is the #1 i386 crash class
(`feedback_extern_c_abi`). The rustconv `enforce_*_abi` machinery
(`feedback_win32k_abi_enforcement`) is already in place for win32k;
extend it for ntdll:

- Build `tools/rustconv/rustconv/abi_map_ntdll.py` from `ntdll.spec`
  (and `ntdll_vista.spec`). Same shape as `abi_map.py` for win32k:
  `(name, cfg-guard) → canonical extern decl`.
- Add `postprocess.enforce_ntdll_abi` (clone of `enforce_win32k_abi`)
  that OVERWRITES the LLM's extern with the canonical dual-cfg decl —
  not LLM-inferred.
- Verify with `nm` against the final archive per file flip.

The 4 must-not-drift exports get an extra-strict path: a
`tests/qemu/test_ntdll_exports.py` that `nm`s the final `ntdll.dll`
binary and asserts the 4 symbols at exact i386/amd64/arm64 decorations.
(Smoke already implicitly tests this — process init would crash without
them — but explicit is cheaper to debug.)

## Cohabitation mechanism

### The precedent: spec-driven user-mode DLL (FreeLDR boot DLLs)

`feedback_freeldr_dll_translation` documents the kdcom/bootvid/peloader
pattern. ntdll inherits it with one wrinkle (see below).

```cmake
# dll/ntdll/CMakeLists.txt — current
add_library(rtl_um OBJECT
    rtl/libsupp.c      # ← OBJECT lib; .obj absorbed into ntdll.dll
    rtl/uilist.c
    rtl/version.c)

add_library(etwtrace etw/trace.c)   # ← separate static lib

list(APPEND SOURCE
    dbg/dbgui.c
    ldr/ldrapi.c
    ldr/ldrinit.c
    ldr/ldrpe.c
    ldr/ldrutils.c
    ldr/verifier.c)

if(ARCH STREQUAL "i386")
    list(APPEND ASM_SOURCE dispatch/i386/dispatch.S)
elseif(...)

add_library(ntdll MODULE ${SOURCE} ${ntdll_asm} ...)

target_link_libraries(ntdll
    ntdll_vista_static etwtrace csrlib rtl rtl_um rtl_vista
    ntdllsys libcntpr setjmp uuid ${PSEH_LIB})
```

### Per-file Rust cohabitation

```
sdk/lib/rust/
├── ntdll-sys/         ← substrate (smss-csrss; +ntdllp.h delta)
└── ntdll-rust/        ← NEW — staticlib crate, libntdll_rust.a
    ├── Cargo.toml     (crate-type = ["staticlib"], copy smss-csrss-rust panic_handler)
    ├── build.rs       (compile pre-processed dispatch-{arch}.s via cc-rs)
    ├── data/asm/
    │   ├── dispatch-i386.s   ← pre-processed asm (preserve DUMMY_PSEH blocks)
    │   ├── dispatch-amd64.s
    │   └── dispatch-arm64.s
    └── src/
        ├── lib.rs       (extern crate ntdll_sys; module wiring)
        ├── ldr/
        │   ├── ldrapi.rs
        │   ├── ldrutils.rs
        │   ├── ldrpe.rs
        │   ├── ldrinit.rs     ← Tier 3, LAST
        │   └── verifier.rs
        ├── rtl_um/
        │   ├── libsupp.rs     ← contains LdrpLoaderLock keystone
        │   ├── version.rs
        │   └── uilist.rs
        ├── dbg/dbgui.rs
        ├── etw/trace.rs       (absorbed; etwtrace lib goes away)
        └── nt_0600/
            ├── dll_main.rs
            ├── ldrinit.rs
            └── ldrnotify.rs
```

Per-file flip = comment the `.c` in `dll/ntdll/CMakeLists.txt`
(`# replaced by Rust (sdk/lib/rust/ntdll-rust/src/ldr/ldrutils.rs)`),
the Rust `#[no_mangle]` symbol takes over. `--allow-multiple-definition`
covers the swap-in window; lld first-match wins. Same as
ntoskrnl-rust/hal-rust cohabitation. Independently revertible per file.

### Cohabitation wrinkle: `rtl_um` is an OBJECT lib

Unlike most kernel cohabitation targets, `rtl_um` is an **OBJECT lib**
(`add_library(rtl_um OBJECT ...)`) — its `.obj`s are absorbed directly
into `ntdll.dll` rather than linked from a `.a`. This means:

- Commenting individual sources in the `rtl_um` OBJECT lib works the
  same way; no separate archive flip needed.
- But: per the `feedback_libcntpr_shared_source_crt` /
  `feedback_cohabitation_consume_vs_provide` lessons — verify rtl_um's
  sources aren't *also* compiled into another module via a shared source
  variable. Quick check: `grep -rn 'libsupp.c\|uilist.c\|version.c'
  sdk/cmake/ dll/`. **Not yet verified** at audit time — flag as open
  question Q3.

### Cohabitation wrinkle: `etwtrace` is its own static lib

`add_library(etwtrace etw/trace.c)` — separate `.a`. Two options:

1. **Absorb into ntdll-rust**: translate `etw/trace.c` as
   `ntdll-rust/src/etw/trace.rs`, delete the `etwtrace` cmake target,
   drop from `target_link_libraries(ntdll …)` and from the advapi32
   side that also links it. Cleaner long-term.
2. **Translate as a sibling crate** `etwtrace-rust/` producing its own
   `libetwtrace_rust.a`. Mirrors the freeldr-DLL pattern more closely.

Recommend Option 1 — etw/trace.c is 277 LOC, ntdll-only consumer is
ntdll itself (advapi32 is a Wine-derived component the project keeps
as a Wine import per `beyond-ntoskrnl-targets.md`). Folding in saves
one cmake target.

## Translation order

> **STATUS 2026-06-01:** Phases 1–5 below are **complete** (steps 1–14, all
> ✅ — see [Current state](#current-state-2026-06-01)). The plan executed
> essentially as written, leaf→root, smoke-gated per flip. Deviations:
> Phase 1's substrate delta was done per-file at integration instead of via a
> wrapper rebuild, and the dispatch `.S` pre-processing (Phase 1 last bullet)
> was skipped (asm left in the C build). Steps below are kept for the record.

Leaf→root, smoke-gated per file flip. Every file is on the boot spine,
so smoke is a complete regression gate (`feedback_no_post_shell_harness_prereq`).

### Phase 1 — Substrate delta (~half session)

- ntdll-sys wrapper: add `ntdllp.h` + `ldrtypes.h`, regen bindgen ×3,
  regen `verify_sizes` ×3.
- rustconv `Target` config: add `ntdll` target (or extend `smss-csrss`
  detect_target to include `dll/ntdll/`). New `abi_map_ntdll.py` +
  `postprocess.enforce_ntdll_abi`.
- Create `sdk/lib/rust/ntdll-rust/` skeleton (Cargo.toml,
  `#[panic_handler]` copied from `smss-csrss-rust`, empty `lib.rs`).
- Add workspace member.
- Pre-process the 3 `dispatch/{arch}/dispatch.S` files via a Python
  helper (clone `tools/gen_preprocessed_asm.py`) → `data/asm/*.s` —
  **verify the DUMMY_PSEH blocks are present in the pre-processed
  output.**

### Phase 2 — Tier 0 leaves (~2 sessions, ~$0.30 in API)

Order: smallest first, prove the spec-driven DLL cohabitation flow.

1. `nt_0600/DllMain.c` (31) — trivial, but it's a *separate DLL*'s
   entry point — proves ntdll_vista cohabitation independently.
2. `nt_0600/ldr/ldrinit.c` (3) — empty stub, free win.
3. `rtl/uilist.c` (40) — RtlInitializeAtomPackage helper.
4. `rtl/version.c` (231) — RtlVerifyVersionInfo.
5. `etw/trace.c` (277) — translates, also retires the `etwtrace` cmake
   target (Option 1 above).
6. `dispatch/dispatch.c` (89) — arm64 + non-x86 dispatchers.
7. `dbg/dbgui.c` (387) — DbgUi*.

After step 7: cohabitation pattern proven; one smoke run validates the
whole tier. **If anything is going to fall over on the DLL pattern, it
falls over here at ~$0.30 of LLM spend.**

### Phase 3 — Tier 1 leaves (~3 sessions, ~$0.50)

8. `nt_0600/ldr/ldrnotify.c` (154) — Vista LDR notification API.
9. `ldr/verifier.c` (745) — Application Verifier shim; off the
   bootstrap path (only enabled with `+VRF` flag).
10. `rtl/libsupp.c` (1,261) — `rustconv translate` (>1K LOC). **The
    most semantically entangled file.** Verify `RtlpAllocateMemory`/
    `RtlpGetStackLimits` translations against rtl call-site contracts.
    Add the `LdrpLoaderLock` keystone + size assertion here.

### Phase 4 — Tier 2 LDR crown (~5 sessions, ~$2)

File-by-file, `rustconv translate`, manual preamble splice
(`feedback_rustconv_translate_vs_batch`). Reference
`ntoskrnl-rust/src/ARM3/sysldr.rs` for the format-side helpers.

11. `ldr/ldrpe.c` (1,281) — import descriptor walk, `LdrpSnapThunk`/
    `LdrpWalkImportDescriptor`. **Most analog to existing Rust**
    (`MiSnapThunk`/`MiResolveImageReferences`). Lowest novelty in the
    tier; do first.
12. `ldr/ldrapi.c` (1,666) — public `Ldr*` spec exports. Boundary fns,
    most are thin wrappers over `Ldrp*` internals — depends on ldrpe.rs
    + (some) ldrutils.rs.
13. `ldr/ldrutils.c` (2,799) — KnownDlls, search path, SxS,
    `LdrpLoadDll`, refcounting, shim engine. **Highest novelty file
    in the audit.** Do last in Tier 2.

### Phase 5 — Tier 3 bootstrap (~2 sessions, ~$1)

14. `ldr/ldrinit.c` (2,735) — **Per-function cohabitation within the
    file.** Helpers (`LdrpInitializeTls`,
    `LdrpInitializeExecutionOptions`, `LdrpInitializeProcessCompat`,
    `LdrpInitializeDotLocalSupport`, `LdrpInitializeThread`) flip
    individually first. `LdrpInit` and `LdrpInitializeProcess`
    (the no-alloc prefix + the bootstrap orchestrator) are the
    absolute last functions to flip. Verify each flip with smoke; the
    bootstrap is single-threaded one-shot so any bug is reproducible.

### Total

~12–15 sessions, ~$5–10 in API spend. Calendar risk dominates over
cost. Every regression is a boot failure — fast to detect, blocking
until fixed.

## Definition of done

- `ntdll.dll` for all 3 arches is `ntdll-rust` + the pre-processed asm
  + still-C `librtl.a`/`liblibcntpr.a` (out of scope) + generated
  syscall stubs.
- `ntdll_vista.dll` likewise (3 files Rust).
- All 3 smoke tests reach `shell ready` on the Rust-built binaries.
- `nm` of every arch's `ntdll.dll` shows the 4 must-not-drift exports
  at correct decorations.
- `verify_abi.py`, `verify_constants.py`, `scan_ud2.py`, `lint_kernel.py`
  clean against ntdll-rust.
- `dll/ntdll/` C tree's `.c` files are all commented in CMakeLists with
  `# replaced by Rust (path)` lines, files kept on disk per the
  never-delete rule.
- DUMMY_PSEH safety-net blocks present in the checked-in dispatch asm
  (verify by `grep _DummyPseh_RaiseSecondChance
  sdk/lib/rust/ntdll-rust/data/asm/*.s` — must hit ×2 per arch file).

## Remaining work + future plans

The body is **100% flipped** and boots green ×3. The C bodies are all done;
what's left is landing + out-of-body follow-ups:

1. **Commit the uncommitted flips** (this session). All three translated +
   integrated, green ×3 (i386 cold-green), currently UNCOMMITTED on
   `ntdll-rust-port`. The user asked to bundle the doc update into this commit:
   - `ldr/ldrapi.rs` (loader public API) + `ldr/mod.rs`, `lib.rs`,
     `nt_0600/dllmain.rs`, `dll/ntdll/CMakeLists.txt`. **Closes `ldr/`.**
   - `nt_0600/ldr/ldrnotify.rs` (+ `nt_0600/ldr/mod.rs`, `nt_0600/CMakeLists.txt`,
     `ldrnotify_rust_stub.c`) — ungated (both ntdll.dll + ntdll_vista.dll, like
     `nt_0600::ldr`'s `LdrpHeap`).
   - `dispatch/mod.rs` (+ `lib.rs` cfg-gate, `CMakeLists.txt`) — `#[cfg(aarch64)]`.
   Then ff-merge `ntdll-rust-port` → `continue-rust-port`.
2. **Dispatcher `.S` stays asm — permanently.** Not a translation target
   (kernel-built stack contracts + ml64 SEH pseudo-ops). The DUMMY_PSEH
   safety-net blocks in `dispatch/{i386,amd64}/dispatch.S` are load-bearing;
   do not "clean up" (see design note in §"Dispatchers stay asm").

**Done this session (was "remaining"):**
- ✅ `nt_0600/ldr/ldrnotify.c` → `ldrnotify.rs`. The self-referential static CS
  + list head became zeroed statics + lazy `RtlInitializeCriticalSection` /
  `InitializeListHead` (AtomicU32 once-guard). Watched the companion-DLL cascade
  rule (`feedback_cohab_flip_companion_dll_cascade`) — clean because the module
  only calls ntdll imports + `LdrpHeap`. i386 cold-green.
- ✅ `dispatch/dispatch.c` → `dispatch/mod.rs` (arm64 only:
  `KiRaiseUserExceptionDispatcher` + `KiUserCallbackDispatcher`; `#[cfg(aarch64)]`
  so the x86 `.S` keeps ownership — `nm` confirms 0 defs in the x86 rust `.a`).
  arm64 green (`shell ready` + `_tWinMain` exercises the win32k callback path).

**Cross-track coordination (smss-csrss-rust agent):**
- **smlib / csrlib client flip — DONE 2026-06-02.** csrlib (`Csr*` client →
  ntdll.dll) and smlib (`SmConnectToSm`/`SmSendMsgToSm` → ntdll_vista.dll)
  flipped to the existing `smss-csrss-rust` translations via lean
  `csrlib_only` / `smlib_only` cargo features (compile ONLY the one client
  module — no csrsrv/server globals). Each built into its own target dir
  (`csrlib-rust-target` → `libcsrlib_rust.a`, `smlib-rust-target` →
  `libsmlib_rust.a`) and added to the ntdll / ntdll_vista link lines; the C
  (api/capture/connect.c, smclient.c) commented out + stubbed. csrsrv.dll +
  smss.exe were already green (their archives carry Rust smlib) — only
  ntdll_vista needed the new archive (the companion cascade,
  `feedback_cohab_flip_companion_dll_cascade`). **GREEN ×3** (i386/amd64 shell
  ready, arm64 live CSR LPC dispatch). One runtime bug fixed:
  `size_of_val()` on bindgen's narrow `CSR_PORT_NAME` under-counted the
  port-name buffer (`feedback_sizeof_narrow_bindgen_wide_literal`). The whole
  smss/csrss spine is now Rust client+server. Branch
  `smlib-csrlib-client-flip` (worktree vibeos-wt-ntdll); see
  `project_smlib_csrlib_client_flip_deferred`. **Do not re-translate** smlib/
  csrlib — they're done.

**Out-of-body follow-ups (separate tracks, not ntdll PE work):**
- The rtl exception engine (`RtlDispatchException`/`RtlUnwind`/`.pdata`),
  rtl heap, and libcntpr printf-family still cohabit as C via FFI. Maturing
  those (rtl-strings / cnt_rust_prep tracks) reduces the C surface ntdll
  links but is **not** required for the ntdll body to be "done".
- Optional hardening: an explicit `tests/qemu/test_ntdll_exports.py` that
  `nm`s the final `ntdll.dll` and asserts the 4 must-not-drift exports at
  exact i386/amd64/arm64 decorations (smoke covers this implicitly today).
- Idiomation pass (Stage 2/3): the flipped files are transliterations
  (`write_volatile`/`Result`/RAII not yet applied). De-transliteration per
  `project_rust_idiomation_plan` is future polish, not correctness.

## Open questions — resolved by implementation

1. ~~**Track placement.**~~ **RESOLVED.** Own worktree `vibeos-wt-ntdll`,
   branch `ntdll-rust-port` (ff to `continue-rust-port`). Ran in parallel
   with smss-csrss without conflict.
2. ~~**Timing / gate on libcntpr+rtl-strings.**~~ **RESOLVED — proceeded
   anyway.** Cohabitation with C rtl/libcntpr worked fine; the body flipped
   without waiting. Some churn absorbed (e.g. rtl-strings cabi alloc stubs,
   `feedback_rtl_strings_alloc_stubs`) but no blocker.
3. ~~**rtl_um OBJECT lib reuse check.**~~ **RESOLVED 2026-05-24.** Sources
   only compiled into ntdll; per-file OBJECT-lib comment-out works directly.
4. ~~**Rust `LdrpLoaderLock` layout / atomic ABI.**~~ **RESOLVED.**
   libsupp.c + ldrinit.c flipped and boot green ×3 — the
   `RTL_CRITICAL_SECTION`/interlocked layout holds across arches.
5. ~~**ntdll_vista ownership.**~~ **RESOLVED — in-scope.** DllMain + LdrpHeap
   flipped; only ldrnotify.c (154 LOC) remains. Surfaced the companion-DLL
   cold-link cascade lesson (`feedback_cohab_flip_companion_dll_cascade`).
6. **PE forwarder decoration on i386** — no forwarder breakage observed
   through the flip; `feedback_i386_pe_decoration` discipline held. Keep the
   check in mind for the remaining ldrnotify flip.
7. **PSEH / `dispatch.S` DUMMY_PSEH blocks** — moot for now: the `.S` files
   were left untouched in the C build, so the blocks are intact. Re-check if
   a future PSEH_CLANG move happens.

## Cross-references

- Bindgen recipe: `feedback_win32k_sys_bindgen`
- ABI enforcement: `feedback_win32k_abi_enforcement`,
  `feedback_postprocess_cfg_dual_def_dedup`, `feedback_extern_c_abi`,
  `feedback_sibling_rlib_ntapi_abi`, `feedback_i386_pe_decoration`
- Cohabitation pattern: `feedback_freeldr_dll_translation`,
  `feedback_libcntpr_shared_source_crt`,
  `feedback_csrss_cohabitation_dual_archive`,
  `feedback_cohab_flip_companion_dll_cascade` (companion-DLL cold-link
  cascade — the ntdll_vista lesson)
- Integration hygiene: `feedback_rustconv_header_types_integration`,
  `feedback_write_bytes_element_count`,
  `feedback_rustconv_scanner_sal_callback_preproc`,
  `feedback_rtl_strings_alloc_stubs`
- Asm pre-processing: `feedback_hal_asm_not_global_asm_reducible`,
  ntoskrnl-translation-audit.md milestone 16
- `static`-on-data-export: `feedback_volatile_static`,
  `feedback_pwchar_pointer_export`,
  `feedback_static_lib_shared_globals_per_consumer`
- LLM translation hygiene: `feedback_rustconv_translate_vs_batch`,
  `feedback_rustconv_translate_side_effects`,
  `feedback_rustconv_stubbed_bodies`, `feedback_degraded_variadic_stubs`
- User-mode substrate: `project_smss_csrss_track`,
  `feedback_win32k_user_critsec_pretranslation`
- Exception strategy: `pseh-elimination.md`,
  `kernel-exception-handling.md`, `feedback_seh_belt_llvm_barrier`,
  `feedback_llm_c_to_c_seh_lowering`
