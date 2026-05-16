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
