# Win32k Translation — Pretranslation Context

This file's C has been **pre-normalized** (see
`docs/engineering/win32k-pretranslation-audit.md`). The deterministic +
LLM pretranslation passes already did the hard structural reasoning so
you don't have to re-derive it per file. A machine-generated
**Pretranslation Map** for THIS file is appended below — trust it; it
is ground truth from the C source, not a hint.

## 1. Probes are already lowered — translate them as `Result`

Every user-mode probe in the win32k keep-set was rewritten from
`_SEH2_TRY { ProbeFor*; … } _SEH2_EXCEPT { … }` to a status-returning
shim call **before you see this file**. You will NOT see `ProbeForRead`
/`ProbeForWrite` inside `_SEH2_TRY` here. Instead you will see:

```c
Status = SafeProbeForRead(P, L, A);          // or SafeProbeForWrite
if (!NT_SUCCESS(Status)) { /* fail path */ } else { /* body */ }
```

`SafeProbeFor{Read,Write}(addr, len, align)` is the C mirror of
`ntkrnl_sys::probe::probe_user_buffer` — validate-only (no touch-test;
DUMMY_PSEH makes a touch-fault uncatchable). Translate each to:

```rust
probe_user_buffer(P as *const c_void, L as usize, A as usize, mode)
    .map_err(|s| s)?;            // or match, preserving the exact fail path
```

Preserve the **exact** NTSTATUS the C produced and the exact success/
fail control flow the normalized C already encodes. Do NOT reintroduce
SEH. Do NOT hardcode `STATUS_SUCCESS` — thread the real status.
Prefer typed `probe_read_*`/`probe_write_*` wrappers when the C copies
a single scalar right after the probe.

## 2. GDI locks → RAII `Drop` guards

`<Prefix>_Lock<Suffix>(handle)` returns a borrowed object that MUST be
released by the paired `<Prefix>_Unlock<Suffix>`. The Pretranslation
Map gives you, per site: the bound variable, the inferred unlock
symbol, and a **shape**:

- `CLEAN_BRACKET` — one acquire (NULL-guarded), one release. Translate
  to an RAII guard whose `Drop` calls the unlock fn; the NULL check
  becomes `?`/early-return. Idiomatic target:
  `let g = DcGuard::lock(hdc).ok_or(STATUS_INVALID_HANDLE)?;`
- `MULTI_UNLOCK` — multi-exit release. Still one RAII guard; the guard's
  `Drop` replaces every textual unlock site. Do NOT emit manual unlocks.
- `BORROWED` — no release in this function: the object is transferred
  or was acquired elsewhere. Do **not** emit a `Drop` here; model it as
  a borrowed `&T` parameter/return. (This is the `GDI_BORROWS` case.)
- `REVIEW` — shape unclear; keep the explicit lock/unlock calls and add
  a `// TODO(win32k-pretranslation): manual RAII review` marker.

## 2.5. USER critical section → `UserExclusiveGuard` / `UserSharedGuard`

`UserEnterExclusive()` / `UserEnterShared()` ... `UserLeave()` is the
win32k global lock. It is a **two-level nest** (KeEnterCriticalRegion +
ERESOURCE) with **inverse release order** — but ReactOS already factored
that into the primitive pair, so:

- The **primitive** `UserEnter{Exclusive,Shared}`/`UserLeave` (in
  `ntuser.c`) translates as **plain direct calls**, exactly like
  ntoskrnl-rust `ex/time.rs` `ExAcquireTimeRefreshLock` /
  `ExReleaseTimeRefreshLock`: acquire = `KeEnterCriticalRegion()` then
  `ExAcquireResource*Lite(&UserLock, TRUE)`; release = **`ExReleaseResourceLite`
  first, then `KeLeaveCriticalRegion()`** (inverse order — never swap).
- Every **caller** is then a single-level RAII problem. Define/use a
  `PfnLockGuard`-shaped guard: `UserExclusiveGuard::enter()` calls
  `UserEnterExclusive()`, its `Drop` calls `UserLeave()`
  (`UserSharedGuard` likewise). Use `drop(guard)` at the **exact**
  original `UserLeave()` line — NEVER let it fall to function end
  (the #1 ntoskrnl RAII-translation failure: lock held across the
  wrong span → system-wide deadlock).

The Pretranslation Map gives, per function: `mode`
(exclusive/shared/mixed), the enter line (`@top` = unconditional
prologue / `@cond` = conditional), the leave lines, and a **shape**:

- `CLEAN_BRACKET` — one enter `@top`, one leave, single exit. One
  whole-fn guard; `let _g = UserExclusiveGuard::enter();` — RAII Drop
  at function end IS correct here (single exit).
- `MULTI_LEAVE` — one enter `@top`, ≥2 leaves. **One** guard; its
  `Drop` replaces *every* textual `UserLeave()`. Emit `drop(_g)` at
  each original leave line on the early-return paths; do NOT emit
  manual `UserLeave()`.
- `REENTRANT` — leave-then-reenter (callout straddle). Do NOT use a
  whole-fn guard; scope the guard so it drops before the callback and
  a fresh one is acquired after. Cross-reference the USER-callouts
  entry for the same function (§3).
- `BORROWED` — enter here, no leave in this fn (released by the caller
  / transferred). Do NOT emit a `Drop`; model the lock as caller-owned
  and document it.
- `REVIEW` — conditional/late enter, or a return path that skips the
  leave. Keep explicit `UserEnter*`/`UserLeave` calls and add a
  `// TODO(win32k-pretranslation): manual USER-lock review` marker.

## 3. USER callouts → preserve the lock straddle exactly

A function flagged as a callout in the Map issues a user-mode callback.
The shape is `UserLeaveCo(); KeUserModeCallback(IDX, in, inlen,
&out, &outlen); UserEnterCo();` then a marshal-back probe. Rules:

- The `UserLeaveCo()`/`UserEnterCo()` pair is a **lock
  release/reacquire around the callout** — it is load-bearing. Model
  it as an explicit scope where the USER lock is dropped then
  reacquired (not a plain RAII guard that spans the callout).
- Keep the `USER32_CALLBACK_*` index, the marshal-in buffer/len and the
  `&out`/`&outlen` exactly.
- If the Map says `missing_leaveco_enterco_pair` (e.g. the UMPD printer
  thunk) the function uses a different lock discipline — do NOT
  synthesize a straddle; mirror the C.
- If `no_marshal_back_probe`, the result is copied via bespoke logic —
  translate that logic verbatim; there is no `SafeProbeFor*` to map.

## 4. Residual SEH

After probe-lowering, any `_SEH2_TRY` still present is **not** a probe
pattern. The Map tags each with its Pass-0 category:

- `c` FAULT_RECOVER — genuine fault-then-recover (e.g.
  `MmProbeAndLockPages`). Keep the SEH semantics; route through the
  deferred kernel-unwinder shim, do not flatten to a probe.
- `d` BELT_SUSPENDERS — paranoid wrapper on an already-validated
  buffer. Translate the body straight; drop the SEH (it never fires).
- `e` NEEDS_REVIEW — has a non-probe call in the TRY; translate
  faithfully and add a `// TODO(win32k-pretranslation)` marker.

When in doubt, **mirror the normalized C exactly** — it already
encodes the decision. Under-idiomatic but faithful beats clever but
divergent. Never reintroduce `_SEH2_*`.

## 5. Cross-crate kernel types — `ntkrnl_sys::ddk` vs `win32k_sys::win32k`

Both `win32k_sys` and `ntkrnl_sys` independently `bindgen` the same
underlying C kernel types (e.g. `_LARGE_INTEGER`, `_KFLOATING_SAVE`,
`_TEB`, `_KTHREAD`, `_KPROCESS`, `_KEVENT`, `_KSEMAPHORE`,
`_KMUTEX`, `_KTIMER`, `_KDPC`, `_KAPC`, `_KSPIN_LOCK`, …) into
**their own namespaces** — Rust sees them as DISTINCT types even
though they're layout-identical.

The blanket `use win32k_sys::win32k::*;` you emit at file top
**shadows** the ntkrnl_sys versions. When the body then calls a
kernel API (e.g. `KeQueryPerformanceCounter`, `KeWaitForSingleObject`,
`KeSetEvent`, `NtCurrentTeb`, `KeSaveFloatingPointState`) the
typecheck fails: `expected ntkrnl_sys::ddk::_X, found
win32k_sys::win32k::_X`.

**Rule:** for the **kernel-shared types** listed above, declare and
allocate them via the fully-qualified `ntkrnl_sys::ddk::_TYPE` form
whenever they will be passed to any `ntkrnl_sys::ddk::*` function.
Do NOT rely on the `win32k_sys::win32k::*` re-bindgen for these.

```rust
// WRONG — typecheck fails at KeQueryPerformanceCounter call
let mut freq: LARGE_INTEGER = core::mem::zeroed();  // = win32k_sys
ntkrnl_sys::ddk::KeQueryPerformanceCounter(&mut freq);

// RIGHT
let mut freq: ntkrnl_sys::ddk::_LARGE_INTEGER = core::mem::zeroed();
ntkrnl_sys::ddk::KeQueryPerformanceCounter(&mut freq);
```

For `NtCurrentTeb`, prefer `ntkrnl_sys::inline::ke::NtCurrentTeb` (the
real per-arch TIB-read intrinsic — `ntkrnl_sys::ddk::NtCurrentTeb` is
just a bindgen extern decl with no link symbol on i386, will fail
link). Cast its `PVOID` return to `*mut ntkrnl_sys::ddk::_TEB` if you
need the field-access form.

## 6. GDI object macros — inline-expand, do NOT extern-declare

`win32ss/gdi/{eng,ntgdi}/*.h` defines per-object-type macro families
that look like function calls but are **`#define` macros** —
attempting to `extern "C" { fn X(); }` them will produce undefined
symbols at link.

Recognized macro families and their canonical expansions:

| Macro form | Expansion |
|---|---|
| `<T>_AllocObjectWithHandle()` | `GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_<T>, sizeof(E<T>)) as P<T>` |
| `<T>_Alloc<X>WithHandle()` | `GDIOBJ_AllocObjWithHandle(GDI_OBJECT_TYPE_<X>, sizeof(<INTERNAL>)) as P<T>` |
| `<T>_AllocObject()` | `GDIOBJ_AllocObj(GDI_OBJECT_TYPE_<T>, sizeof(E<T>)) as P<T>` |
| `<T>_TryLockObject(h)` | `GDIOBJ_TryLockObject(h as HGDIOBJ, GDI_OBJECT_TYPE_<T>) as P<T>` |
| `<T>_LockObject(h)` / `<T>_Lock<X>(h)` | `GDIOBJ_LockObject(h as HGDIOBJ, GDI_OBJECT_TYPE_<T>) as P<T>` |
| `<T>_UnlockObject(p)` / `<T>_Unlock<X>(p)` | `GDIOBJ_vUnlockObject(p as POBJ)` |
| `<T>_ShareLockObject(h)` / `<T>_ShareLock<X>(h)` | `GDIOBJ_ShareLockObj(h as HGDIOBJ, GDI_OBJECT_TYPE_<T>) as P<T>` |
| `<T>_ShareUnlockObject(p)` / `<T>_ShareUnlock<X>(p)` | `GDIOBJ_vReleaseObj(p as POBJ)` |

`<T>` is the object family: `DRIVEROBJ`, `BRUSHOBJ`, `DC`, `PATH`,
`REGION`, `SURFACE`, `PALETTE`, `XLATEOBJ`, `LFONT`, `TEXTOBJ`,
`COLORSPACEOBJ`, `ICMLCS`. The underlying `GDIOBJ_*` primitives are
real functions in the `win32k-sys` bindgen — use them directly.

**Do NOT emit:** `extern "C" { fn DRIVEROBJ_AllocObjectWithHandle() -> PEDRIVEROBJ; }` etc.
**Do emit:** inline-expanded calls, plus a one-line comment pointing
at the C macro source (`gdi/eng/driverobj.h:17`) so reviewers can
verify the expansion.
