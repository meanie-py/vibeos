# DUMMY_PSEH Verdict — Replace the Semantics in Rust, Not the Mechanism in C

Last updated 2026-07-06 (lane `dummy-pseh-question`). Companion to
[pseh-elimination.md](archive/pseh-elimination.md) (the layer map this supersedes in
direction) and [kernel-exception-handling.md](archive/kernel-exception-handling.md)
(the deferred trap-to-panic bridge — still deferred, see Tier 2).

## The verdict

> **Update 2026-09-26:** Rust kernel code now has real `try_except`/`try_finally`
> (`ntkrnl_sys::seh` over `ntoskrnl/sehtry.c`: native SEH on amd64/arm64, a hand-built
> `fs:[0]` frame on i386). That changes step 3 below: the FS drivers' raise model can be
> wired to its already-translated filters as written, instead of converting every raise
> site to `Result`. Cc/Mm in-page raises, which no `Result` conversion inside cdfs could
> reach, are caught the same way.

**Keep DUMMY_PSEH as the *default* compile mode for C; give Rust code the
three semantics PSEH provided, idiomatically. Kernel-mode catch is no
longer impossible — this lane made it work (see the PoC below) — so
native SEH graduates from "rejected" to "a per-module tool for the
handful of load-bearing C sites," while the Rust program below remains
the direction:**

| What PSEH did | Rust idiom | Status |
|---|---|---|
| 1. Catch hardware faults probing user memory | Validate-first `probe.rs` (`Result<T, NTSTATUS>`) | Exists; SE+OB migrated; keep migrating |
| 2. Catch **software exceptions raised by design** (`ExRaiseStatus`, `MmProbeAndLockPages`, Fat/Cd raise conventions) | **Convert the contract at the callee**: `try_*` Result-returning variants; the raising export survives only as a thin ABI shim for foreign binaries | **The gap. This is the program below.** |
| 3. `__finally` guaranteed cleanup | RAII `Drop` guards (`IrqlGuard`, `SpinLockGuard`, …) | Exists |

**The exception *dispatcher* is infrastructure, not a consumer — it stays and
must remain correct** (it delivers faults/raises to user-mode processes and to
any foreign-compiled driver with real unwind tables). That distinction —
kernel stops *using* SEH but keeps *dispatching* it — was already right in
`pseh-elimination.md` and this audit re-confirmed every piece of it works.

Why this over the alternatives: the direction of the codebase is Rust
(~99%+ of ntoskrnl; only **22** C files with `_SEH2_TRY` remain in the entire
build). Rust has no SEH and doesn't want one — `Result` + `?` + `Drop` *is*
the idiomatic shape of "protected region with cleanup and error propagation."
Rebuilding catch-at-a-distance (any flavor: PSEH_CLANG, native clang SEH, a
panic-unwinder) so that translated code can keep pretending to be C is
backwards. The honest translation of `__try { MmProbeAndLockPages(...) }
__except { Status = code }` is `MmTryProbeAndLockPages(...)?` — and that
conversion is *mechanical* at the callee, where raising and returning are the
same information.

## Evidence base

### A. Real exposure is small and enumerable — not "everywhere, scary"

Of **327** repo files with `_SEH2_TRY`, only **22 are in the actual build**
(identical set on i386/amd64/arm64 — the Rust port already consumed the
rest, including *all* of ntoskrnl's former 14 SEH files and all of win32k).
Classified (full detail: lane analysis, 2026-07-06):

- **Can bugcheck a booted default image** (kernel, loaded, guards
  `MmProbeAndLockPages` against untrusted user buffers): KMDF
  `probeandlock.c`, `tdihelpers/tdi.c` (compiled into afd.sys, reachable via
  any socket), `sptilib.c`, `copysup.c`, `aux_klib.c`. All are *call-shaped*
  raise-catch sites — class 2 above, fixable by Result-contract conversion
  or a reverse shim.
- **App-crash-contained** (user DLLs; a bad caller buffer crashes the
  calling process instead of returning an error): `psapi`, `ws2_32`,
  `advapi32`, `setupapi`, `opengl32`, `eventvwr`, syssetup. Mostly *direct
  dereferences* of caller pointers. Annoying, not fatal; dies with
  translation.
- **Not loaded**: `rdbsslib`/`rxce` (50 of the 327's scariest tries) belong
  to the SMB redirector, which has no service entry in any boot hive.
- **Not real**: fastfat `rust_cohab_stub.c`'s "tries" are the DUMMY shim
  definitions themselves.

On the Rust side, translations preserved DUMMY semantics faithfully:
cdfs-rust/fastfat-rust carry literal no-op `_SEH2_TRY()` shims
(`cdmacros.rs:1453`) with their exception filters (`CdExceptionFilter`,
`FatExceptionFilter`, `FatProcessException`, `CdProcessException`) **fully
translated but unreachable** — dead arms behind `if _SEH2_Volatile0`-shaped
guards. The catch logic exists; only the wiring is dead.

### B. The load-bearing class is raise-by-design — validate-first cannot close it

Validate-first handles class 1 (pre-check the pointer instead of
fault-and-catch). It categorically cannot handle class 2, because the API
*contract* is raise-on-failure:

- `MmProbeAndLockPages` (`ARM3/mdlsup.rs`) **raises** on bad user MDL —
  5 `ExRaiseStatus` sites in that one file. NT-compatible, and every
  internal caller that "handles" the failure is dead code.
- Rust `ProbeForRead`/`ProbeForWrite` (`ex/exintrin.rs:159–193`) raise
  AV/misalignment — these are the NT exports, distinct from the
  Result-returning `probe.rs`.
- 36 live `ExRaiseStatus`/`RtlRaiseStatus` call sites in
  ntoskrnl-rust + cdfs/fastfat/cc crates alone (pool-corruption raises in
  `expool.rs`, quota, wait-limit, uuid, FS error paths), plus ~400 C
  `ProbeFor*` sites still built.
- Terminal behavior today: raise → `RtlDispatchException` finds no scope
  tables (DUMMY code emits none) → `KiDispatchException` →
  **`KeBugCheckEx(0x1E)`**. The only survivable case is a user-pointer
  fault with `PreviousMode == UserMode`, which is delivered to (and kills)
  the offending process.
- This is not theoretical: the field manual's post-shell-ready `C00000D8`
  hunt (cdfs `STATUS_CANT_WAIT` raise), the `[RAISE]`/`[EXRAISE]`
  instrumentation in `rtl-diag/src/exception.rs:97` — added *specifically*
  because unhandled raises bugcheck anonymously — and the
  `amd64-misalign-231` FIXME in `exintrin.rs` are all this class.
- **Live bug found during this audit**: `ex/profile.rs:391` computes
  `probe_result` as unconditionally `STATUS_SUCCESS` around a raising
  `MmProbeAndLockPages`, so its mutex-release/deref/free cleanup arm is
  unreachable — a bad profile buffer bugchecks and leaks. This is the
  poster child for the contract conversion.

### C. The toolchain verdict is stale — and it doesn't change the answer

Empirically re-tested on our exact llvm-mingw clang 19.1.7 (2026-07-06):

- **amd64 + arm64: native `__try`/`__except`/`__finally` compiles today**
  with `-fms-extensions` — correct `.pdata`/`.xdata` scope tables, outlined
  filters, `__C_specific_handler` personality, `_exception_code()` works.
- **Coverage is calls-only**: a direct memory dereference inside `__try`
  does NOT keep the handler alive (LLVM DCEs it; `-fasync-exceptions` is a
  no-op on mingw targets, MSVC-env only). Raise-catch and probe-via-call
  patterns work; `*ptr = x` in the try body silently doesn't.
- **i386: genuinely broken** (`L__ehtable$...` assembler-label errors — the
  32-bit SEH lowering is MSVC-env-only). i386 would need PSEH_CLANG
  (written, disabled, "TODO: debug") — a permanent second mechanism.
- **FORCEINLINE callees inside `__try` become undefined symbols**
  (found 2026-07-10, conformance lane, kmtests): clang refuses to inline
  ANY callee lexically inside a `__try` region (funclet EH restriction,
  every -O level), and our mingw C `__forceinline` is
  `extern inline __attribute__((always_inline, gnu_inline))`
  (`sdk/include/vcruntime/_mingw.h:194`) — **GNU inline semantics, which
  never emit an out-of-line body**. So every header-only FORCEINLINE SDK
  helper called inside `__try` (`RtlSecureZeroMemory`,
  `RtlInitEmptyUnicodeString`, `_InterlockedAdd64` on amd64, …) links fine
  under DUMMY_PSEH and breaks the moment a module flips to
  `_USE_NATIVE_SEH`. Fix pattern: one shim TU per module providing real
  out-of-line definitions **without including the SDK headers** (same-TU
  redefinition of a gnu_inline function is an error) — see
  `modules/rostests/kmtests/kmt_native_seh_shims.c`. Note the C99
  extern-declaration trick (6.7.4p7) does NOT work here — gnu_inline
  inverts it. Budget for this in any future native-SEH migration sweep.
- **The runtime dispatch chain is fully present and real** — this audit
  traced it end-to-end: amd64 ntoskrnl.exe already *contains*
  `__C_specific_handler` + `RtlDispatchException` + `RtlUnwindEx` +
  `RtlVirtualUnwind`; kernel-mode `KiDispatchException` calls
  `RtlDispatchException` on amd64/arm64 (`ke/amd64/except.rs:460`,
  `ke/arm64/stubs.rs:947`); ntdll has the complete user-mode walk
  (`rtl-except` amd64 `.pdata` unwind, i386 `fs:[0]` chain) and exports it.
  arm64's `__C_specific_handler` is translated (`rtl-except/arm64/except.rs:438`)
  and would link the moment something references it.
- **PoC ran (2026-07-06) — kernel-mode catch WORKS end-to-end, for the
  first time in this project's history.** `ntoskrnl/seh_poc.c`
  (`__try { ExRaiseStatus(0xE0DEAD01) } __except(...)`, `-fms-extensions`,
  wired via the cargo-link `c_shims`, called from Phase-1 init) prints
  `[SEH-POC] native __try/__except: caught=1 code=0xe0dead01 (PASS)` on
  serial, and the system continues to a green desktop boot. The full chain
  — raise → `KiDispatchException` → `RtlDispatchException` →
  `RtlLookupFunctionEntry`/`RtlVirtualUnwind` over `.pdata` →
  `__C_specific_handler` → filter → `RtlUnwindEx` → handler — is real and
  correct, translated unwinder included.
- **It only works after fixing a latent target-spec bug this PoC flushed
  out.** On first run the raise **double-faulted at
  `RtlVirtualUnwind+0x165`** (BUGCHECK 0x7F P1=8 — kernel stack exhaustion
  via recursive dispatch: the fatal instruction is the `call memcpy` copying
  a 0x4D0-byte CONTEXT per unwind level). Root cause: **the kernel image
  shipped with no unwind info for any Rust function** — the custom
  `*-kernel.json` rustc targets set `"requires-uwtable": false` (every real
  Windows target sets it `true`; the Win64 ABI mandates `.pdata` for
  non-leaf functions). The image's whole `.pdata` was 380 entries, all from
  the cc-rs C shims; the first Rust frame in any walk hit the bogus
  leaf-fallback (`Rip = [Rsp]`) and the walk went wild. Flipping
  `requires-uwtable: true` (amd64 + arm64 kernel targets) grows `.pdata` to
  ~7,900 entries (~1–3% image size) and the PoC passes. Field manual §9.42.
  **Independent payoff:** this also fixes today's *unhandled*-raise
  diagnostics (clean 0x1E instead of the 0x7F "RtlVirtualUnwind fault
  amplification" cascade the `C00000D8` hunt hit) and makes every
  stack-walk consumer (debugger, `RtlWalkFrameChain`) able to walk Rust
  frames. The PoC stays armed at boot as a permanent self-test tripwire for
  the exception machinery.

So on amd64 (and, pending the ×3 gate, arm64) **native SEH is now a
working, validated option** — for **C**. What keeps it from being *the*
answer: (a) **Rust cannot consume it** *(superseded 2026-09-26: it can, one frame down, through C trampolines; `ntkrnl_sys::seh`, `ntoskrnl/sehtry.c`, all three arches incl. i386, boot-tested by `[SEH-TRY]`)* — Rust has no `__try`; giving Rust
catch semantics still requires the trap-to-panic bridge, which is the
expensive deferred project — so the ~36 Rust raise sites and the dead FS
filters need the `Result` conversion regardless; (b) i386 native SEH does
not compile (`L__ehtable$` assembler errors — MSVC-env-only lowering), so
i386 C would need PSEH_CLANG debugged anyway; (c) calls-only coverage is a
silent trap for deref-shaped try bodies (the user-mode probe class — the
handler quietly never attaches; a lint must guard any module flipped to
native SEH). Where native SEH now *earns its keep*: the five
bugcheck-capable kernel C files (KMDF `probeandlock.c`, afd/tdihelpers,
sptilib, copysup, aux_klib) are all call-shaped raise-catch sites on
amd64/arm64-relevant paths — flipping those modules off DUMMY (per-file
`-fms-extensions`, exactly the seh_poc.c recipe) restores their intended
recovery semantics for a few lines of build config each, without waiting
for their translation. And foreign-compiled modules (MSVC drivers, if ever
supported) get correct dispatch for free — that was always infrastructure
we keep.

### D. User-mode status (for completeness)

`KiUserExceptionDispatcher` (asm, i386/amd64) carries the deliberate
DUMMY_PSEH swallow: continuable software exceptions `< 0xC0000000` are
resumed (DbgPrint, RPC raises, breakpoints skip the int3), everything
`>= 0xC0000000` or non-continuable goes second-chance. The dispatcher-side
machinery would run a real handler end-to-end if one existed; what's
degraded today is module-side: `IsBad*Ptr` can't detect bad pointers
(forced-dead except), advapi32/ws2_32/psapi param probes crash the caller,
and **first-chance VEH is compiled out** (shared librtl is built as
`_NTOSKRNL_`; `RtlCallVectoredExceptionHandlers` elided —
`rtl-except/src/amd64/except.rs:28`). Known TODO: nested-exception
disposition `debugbreak()`s (`amd64/unwind.rs:689,844`).

**Consequence for RPC clients: a failed call kills the caller with exit code 1.** The `RpcExcept`
arm around every stub never runs, the dispatcher resumes the `RPC_S_*` raise (it is `< 0xC0000000`),
`RaiseException` returns, and `RpcRaiseException` then does its "handler continued execution" `ERR`
and `ExitProcess(1)`. Nothing crashes, so no second chance is logged; the only trace is that `ERR`
line. On amd64, wordpad and winver died this way for as long as the spooler was down, because
winspool's printer calls went to a server that was not listening. Until the class-2 `try_` work
reaches rpcrt4, a dead RPC server means every one of its clients dies.

## The program — Tier 1: convert raise contracts to `Result` (do this)

Mechanical, incremental, each step independently smoke-gateable:

0. ✅ **LANDED (2026-07-08/10, crash-value lane)** — ~~Land the
   `requires-uwtable: true` fix + the boot self-test~~: shipped with the
   uwtable sweep + shared panic handler + user-SEH self-test
   (`466f0096bc3`) and the arm64 unwinder fixes from the fex lane
   (EstablisherFrame=x29, E==1 epilog-index — `f74ee598fa1`/`c447dbdfcdb`).
   Rust frames ship real `.pdata`/`.xdata` on all arches and the unwinder
   works (field manual §9.35). The WinDbg-attach capstone landed too
   (`windbg-kd-attach-handoff.md`).
0b. **Flip the five bugcheck-capable kernel C files to native SEH**
   (amd64/arm64): KMDF `probeandlock.c`, `tdihelpers/tdi.c` (afd),
   `sptilib.c`, `copysup.c`, `aux_klib.c` — call-shaped handlers, the
   validated seh_poc.c recipe, restores real recovery for bad user buffers
   through KMDF/AFD without waiting for translation. (i386 keeps DUMMY
   there; its exposure is the same as today's, not worse.)
1. **`MmTryProbeAndLockPages(mdl, mode, op) -> NTSTATUS`** in mdlsup.rs
   holding the real body; `MmProbeAndLockPages` becomes
   `if let Err(s) = try { ExRaiseStatus(s) }` — a 5-line NT-ABI shim.
   Migrate the ~12 internal Rust callers to the `try_` form. This alone
   fixes the profile.rs bug class and the KMDF/afd bugcheck exposure path
   from the kernel side.
   **DONE 2026-09-25 for the function and three callers** (`MiDoMappedCopy`,
   `MmProbeAndLockProcessPages`, `ex/profile.rs`); `#[no_mangle]`, so any crate
   linked into the kernel image can declare it. Still raising, to migrate: iomgr
   (`iofunc.rs` ×4, `irp.rs` ×2), `ex/sysinfo.rs`, and the driver crates (afd, cdfs,
   fastfat, classpnp, ks, mup, cdrom, win32k `mem.rs`), which also need it in
   `ntoskrnl.spec`. Each site must reproduce its C handler, not just swallow the status.
2. **Same split for the probe exports**: raising `ProbeForRead/Write`
   (exintrin.rs) stay as the ABI edge; internal Rust callers move to
   `probe.rs`. Grep-audit: no internal Rust caller of a raising export.
3. **DONE for cdfs (2026-09-26), via real SEH rather than `Result`.** `CdFsdDispatch`,
   `CdFspDispatch`, `CdPerformVerify` and `CdProcessException` run under
   `ntkrnl_sys::seh::try_except` with the translated `CdExceptionFilter` /
   `CdProcessException`; all 35 `__finally` sites are `try_finally`; the four
   `__except(EXECUTE_HANDLER)` sites (the two MDL locks, `SafeZeroMemory`, the directory
   buffer fill) are real. appstorm's `fsraise` check forces a cdfs raise at every run.
   **fastfat is next** (74 except / 139 finally). Original plan for reference:
   **Re-wire the translated FS filters**: cdfs/fastfat error paths call
   `Cd/FatExceptionFilter` + `Cd/FatProcessException` **explicitly on
   `Err`** instead of via the dead `_SEH2_EXCEPT` arm; `Cd/FatRaiseStatus`
   becomes an early-return of `Err(status)` within our crates. The filter
   logic is already translated and correct — only the wiring is dead. This
   is a rustconv-automatable pattern (Stage-2 probe-migrate cousin);
   `docs/engineering/rustconv-pipeline.md` gets a "raise-migrate" stage.
4. **Triage the remaining raise sites** (36 in Rust): keep the raise where
   dying loud is correct (pool corruption in expool.rs — that's a
   bugcheck-worthy invariant), convert where user input can trigger it
   (quota, wait-limit, uuid, anything reachable from a syscall).
5. **Lint it shut**: extend `vibeos-cli check` with a raise-reachability
   scan — no raising export called from internal Rust outside the ABI-shim
   allowlist; no new no-op `_SEH2_*` shims. Same 0-NEW baseline-diff gate
   as the silent-failure scans.
6. **Residual 22 C files**: leave under DUMMY. The five kernel
   bugcheck-capable ones become no-ops once step 1 lands (their `__try`
   guards a callee that no longer raises for kernel-internal paths) or get
   reverse shims per pseh-elimination.md; the user-mode ones die with
   translation. rdbss/rxce stay parked until the SMB redirector is real.

## Tier 2: the trap-to-panic bridge stays deferred

`kernel-exception-handling.md`'s `try_user_access` bridge is the *only*
mechanism that can close the residual class-1 gap validate-first leaves:
the probe-to-use race (user unmaps between `probe_read_user`'s check and a
later direct access) and hot paths that want fault-and-catch instead of
double-touch. Its decision tree remains correct — pull it forward only if
win32k/measured reality surfaces a structurally-fault-dependent pattern, a
CVE-class "malicious pointer bugchecks the box" incident, or the residual
race is observed in the wild. Tier 1 does not conflict with it; it shrinks
the surface Tier 2 would have to cover.

## What it costs, what it buys

- **Cost**: step 1–2 are a day-ish each (callee split + caller migration +
  gate ×3); step 3 is the big one but automatable (rustconv raise-migrate
  over cdfs/fastfat/cc error paths); step 5 is a scanner in the existing
  lint family. No unwinder, no new runtime mechanism, no binary-size or
  panic-model change, no `panic="abort"` fight.
- **Buys**: every "bad user buffer → 0x1E bugcheck" path through
  MDL/probe/FS raise becomes a returned `NTSTATUS`; the translated-but-dead
  FS retry/verify machinery comes alive; the raise-by-design class stops
  growing (lint); and the remaining SEH story fits in one sentence:
  *"Rust returns `Result`; the NT raise ABI is a shim at the edge; the
  dispatcher serves foreign code and user mode."*

## First increment (concrete, this lane)

`MmTryProbeAndLockPages` + migrate its Rust callers + fix
`ex/profile.rs:391` + smoke ×3. Small, high-value, and exercises the whole
pattern the rest of the program repeats.
