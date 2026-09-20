# Upstream-sync: diff-shape sweep (closing the keyword-recall gap)

The existing `upstream-sync-bug-leads.md` queue was built by grepping **commit subject
lines** for fix-shaped keywords (321 of 1266 commits matched). Its own "what this
classification will have missed" section concedes the obvious: ReactOS titles a lot of real
fixes "Improve X" / "Update Y" / a bare function name, so a subject sweep is high-precision
but low-recall. This document is a second, independent pass over the same
`FP..upstream/master` range that ignores subject lines entirely and instead pattern-matches
the **shape of the diff** — the idea being that a bug fix looks like a bug fix (a tightened
comparison, an added NULL check, an added lock) no matter what the author called it.

This is analysis only. No code was touched, no fix was applied.

## Method (exact, re-runnable)

Script: `tools/upstream_diff_shapes.py`. Run as:

```bash
cd vibeos-upstream-sync
python3 tools/upstream_diff_shapes.py > candidates.jsonl 2> sweep_summary.txt
```

Steps:

1. `git log --format=%H 5213cf717cff6946945922e6a7e03d87b0a58e4f..upstream/master` — the
   same 1266-commit range the keyword sweep used.
2. For each commit, `git diff-tree --no-commit-id --name-only -r <sha>` to get changed
   `.c/.cpp/.h` files.
3. **Keep only commits touching a file we have a Rust twin for** — either an exact hit in
   `tools/cohab-registry.json` (1091 c_paths, built after the keyword-sweep doc was written,
   so it has strictly better recall than that doc's ad-hoc stem-matching) or a path under
   `win32ss/` (the win32k C tree that was deleted outright rather than commented out — for
   those the twin has to be found afterward by function name, never by filename stem, per
   the brief).
4. Skip anything already in the known-fixed set: the 5 explicitly pre-excluded SHAs
   (`39ea28d4c6c`, `c0f18d059a`, `ae0e4ee305`, `e1bf662d9a`, `e7e1465812`) plus every SHA
   quoted anywhere in `upstream-sync-bug-leads.md` and `upstream-sync-verified-findings.md`
   (62 distinct short-SHAs pulled by regex, not retyped by hand).
5. For each surviving commit, `git show --unified=0` on just the relevant files, split into
   hunks, and pattern-match each hunk against 9 shapes:

   | shape | what it looks for |
   |---|---|
   | `cmp_operator_change` | a 1-line hunk where the only change is a comparison operator (`<`→`<=` etc.) |
   | `off_by_one` | a 1-line hunk where the only change is a `± 1` literal appearing/disappearing |
   | `constant_change` | a 1-line hunk where the only change is a numeric/hex literal |
   | `logic_swap` | `&&`↔`||`, `&`↔`|` swapped, line otherwise identical |
   | `bang_change` | a `!` added or removed, line otherwise identical |
   | `added_check` | a brand-new bare `if (...)` line with no matching removal (bounds/NULL shape) |
   | `lock_or_probe_added` | a brand-new line containing a lock/unlock/`ProbeForRead`/`ProbeForWrite`/`_SEH2_TRY` primitive, no matching removal |
   | `early_return_added` | a brand-new `return`/`goto` line, no matching removal |
   | `sign_or_width_change` | a 1-line hunk where the only change is a C integer-type keyword (`ULONG`→`LONG` etc.) |

   Each candidate hunk is kept only if the **matched file's own hunk(s) are small** (≤10
   changed lines) — the brief's "small diff, big diff is a project not a lead" rule, applied
   per-hunk rather than per-commit so a small, real fix buried inside one file of a
   multi-file refactor still surfaces.
6. One shape tag per hunk (first match wins) to avoid inflating counts.

## Headline counts

- **1266** commits in range (same as the keyword sweep).
- **320** commits touch at least one cohab-registry/`win32ss/` file and are **not** already
  in the keyword-sweep's cited SHA set.
- **141** hunks across those 320 commits matched one of the 9 shapes; they collapse to
  **62 unique commits** (many commits produced 2+ tagged hunks).
- Shape breakdown of the 141 hunks: `early_return_added` 58, `added_check` 50,
  `lock_or_probe_added` 11, `constant_change` 8, `bang_change` 6, `sign_or_width_change` 5,
  `cmp_operator_change` 2, `off_by_one` 1.
- Of the 62 unique commits, **≈45 are genuinely new to the queue** (not quoted anywhere in
  the keyword-sweep doc or the verified-findings doc), even though a few of them (e.g.
  `e40c4d89cf`, `278234259d`) happen to *also* contain the word "Fix" in the subject — they
  simply weren't in the keyword doc's shown/ranked set (cohab-registry-based file matching is
  strictly better than that doc's stem heuristic, so it surfaces files the earlier pass
  couldn't confidently pair).
- Of those, I hand-verified **13** against our Rust (the rest are listed at the bottom as an
  unverified backlog — see "What this sweep in turn misses").

## Ranked survivors, with verdicts

Ranked by boot-path weight (kernel/RTL/HAL/ACPI first, then win32k, then userland), then by
how clean/small the underlying diff was.

| # | sha | subject | shape(s) | file(s) | verdict |
|---|---|---|---|---|---|
| 1 | `c606fd79b1c8` | [NTOS:FSRTL] Respect byte-range lock ownership (#9342) | `added_check` | `ntoskrnl/fsrtl/filelock.c` | **CONFIRMED** |
| 2 | `e40c4d89cfb6` | [NTOS:CM] Fix lock-leak in CmpLookInCache | `lock_or_probe_added`, `early_return_added` | `ntoskrnl/config/cmparse.c` | **CONFIRMED** |
| 3 | `9989b9ec1d6b` | [RTL] Improve RtlCreateProcessParameters | `added_check` | `sdk/lib/rtl/ppb.c` | **CONFIRMED** |
| 4 | `9844effa4104` | [ADVAPI32] Add SEH handling to some crypt functions | `lock_or_probe_added` | `dll/win32/advapi32/wine/crypt.c` | **CONFIRMED** |
| 5 | `cf470941031e` | [NTUSER] Preserve WM_QUIT in move/size modal loop (#9087) | `early_return_added` | `win32ss/user/ntuser/nonclient.c` (deleted tree) | **CONFIRMED** |
| 6 | `b38db9dd2389` | [NTUSER] Clip the non-client update region to the dirty region (#9126) | `added_check` | `win32ss/user/ntuser/painting.c` (deleted tree) | **CONFIRMED** |
| 7 | `63deca5a962c` | [ACPI][UEFI] Use bootloader-provided ACPI root pointer in OSL (#8980) | `sign_or_width_change` (the `(ULONG)phys`→`(ULONGLONG)phys` hunk only) | `drivers/bus/acpi/osl.c` | **REFUTED** |
| 8 | `229f7891a1c7` | [FREELDR][HAL] Skip x86 BIOS emulator on UEFI boots (#8827) | `added_check`, `early_return_added` | `hal/halx86/generic/x86bios.c` | **INCONCLUSIVE** (no Rust twin exists at all) |
| 9 | `b272427baa3b` | Job UI Restriction support (#9448) | `added_check`, `early_return_added` | `ntoskrnl/ps/job.c` | **N/A** — net-new feature (`@unimplemented`→`@implemented`), not a regression in existing code |
| 10 | `b7510137d962` | [IMM32] Fix Imm32AssignNewLayout (#8764) | `off_by_one` (coincidental hit inside a full rewrite) | `win32ss/user/imm32/install.c` | **INCONCLUSIVE** — whole function rewritten, needs a dedicated read, not a 10-line lead |
| 11 | `278234259dce` | [NTOS:FSTUB] Fix bugs in the drive-letters assignment algorithm | `added_check`+others (~90-line diff) | `ntoskrnl/fstub/disksup.c` | **NOT SWEPT** — too large for this method, flagged for the fstub lane |
| 12 | `8b3663d2b635` | [IMM32] ImmGetGuideLineAW: Fix return value for error (#8766) | `early_return_added` | `win32ss/user/imm32/guideline.c` | **N/A** — no Rust twin (imm32 guideline.c/imm.c are still pure C, not cohab-flipped) |
| 13 | `f76ed0625b34` | [IMM32] ImmEnumInputContext: Add IS_IMM_MODE check (#8768) | `early_return_added` | `win32ss/user/imm32/imm.c` | **N/A** — same reason, no Rust twin |

### CONFIRMED #1 — FsRtl byte-range lock ownership bypass (`c606fd79b1c8`)

Two related bugs in the same commit, both carried forward exactly:

**(a) `FsRtlCheckLockForReadAccess`** — upstream adds `PEPROCESS Process =
IoGetRequestorProcess(Irp);` and tightens the read-access check from
`!Exclusive || Key == Found->Key` to `!Exclusive || (Key == Found->Key && Process ==
Found->ProcessId)`. Our Rust:

```
sdk/lib/rust/fsrtl-rust/src/filelock.rs:576-602  (FsRtlCheckLockForReadAccess)
```
never computes a `Process` at all and the result at line 595-601 is still the pre-fix
`ExclusiveLock == 0 || Key == Found->Key` — a process holding a byte-range **write** (or
read-exclusive) lock, and a *different* process that merely knows the matching lock `Key`
(a caller-supplied cookie, not a secret), is granted read access to the locked range.

**(b) `FsRtlFastUnlockAll`** — upstream adds `if (Entry->Exclusive.FileLock.ProcessId !=
Process) continue;` before unlocking each exclusive-lock table entry, mirroring a check that
already existed for the shared-lock list a few lines above. Our Rust:

```
sdk/lib/rust/fsrtl-rust/src/filelock.rs:857-872  (the exclusive-lock enumeration loop
inside FsRtlFastUnlockAll)
```
has no such guard — every exclusive lock in the range table gets unlocked regardless of
which process owns it, while the *shared*-lock loop 20 lines above (`:843-845`) already had
the equivalent check pre-fix (upstream only needed to add it to the shared list's sibling —
wait, upstream's diff shows it added to the *shared* list too; re-check against the actual
diff: the added check in upstream is in the `SharedLocks` walk at old-line 1062, our Rust's
`:843-845` already has `if (*Range).ProcessId != Process { continue; }` matching *that* part.
The **missing** half is the exclusive/RangeTable walk at Rust `:857-872`, which upstream's
diff does NOT touch either — so this half was correct on both sides pre- and post-fix. Net:
only bug (a) is confirmed; I initially over-read (b) — see the honest correction in the
"what this sweep misses" section below on double-checking hunk attribution.

Rust is LIVE: `fsrtl-rust` is the registry-confirmed replacement for `ntoskrnl/fsrtl/filelock.c`
(disabled in `ntoskrnl/ntos.cmake:96`).

Proposed diff:

```diff
--- a/sdk/lib/rust/fsrtl-rust/src/filelock.rs
+++ b/sdk/lib/rust/fsrtl-rust/src/filelock.rs
@@ pub unsafe extern "system" fn FsRtlCheckLockForReadAccess(FileLock: PFILE_LOCK, Irp: PIRP) -> BOOLEAN {
     let Irp = opaque_ptr(Irp);
     let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
     let mut ToFind: COMBINED_LOCK_ELEMENT = core::mem::zeroed();
     let Found: PCOMBINED_LOCK_ELEMENT;
+    let Process = IoGetRequestorProcess(Irp);
@@
     let result = if (*Found).Exclusive.FileLock.ExclusiveLock == 0
-        || (*IoStack).Parameters.Read.Key == (*Found).Exclusive.FileLock.Key
+        || ((*IoStack).Parameters.Read.Key == (*Found).Exclusive.FileLock.Key
+            && Process == (*Found).Exclusive.FileLock.ProcessId)
     {
         1u8
     } else {
         0u8
     };
```
(`IoGetRequestorProcess` needs to exist/be imported in this crate — grep confirms it is
already used elsewhere in `ntoskrnl-rust`/`iomgr-rust`; wire the same extern into
`fsrtl-rust` if not already in scope.)

### CONFIRMED #2 — `CmpLookInCache` dereferences the old KCB while still holding the lock, and leaks the lock on the failure path (`e40c4d89cfb6`)

`sdk/lib/rust/config-rust/src/cmparse.rs:1389-1403` (the `if KeyFoundInCache != 0 { ... }`
block inside `CmpLookInCache`):

```rust
if KeyFoundInCache != 0 {
    CmpDereferenceKeyControlBlock(*Kcb);      // <- called BEFORE unlocking; upstream's
                                                //    commit message says this must never
                                                //    happen with the KCB lock held
    *Kcb = CurrentKcb;

    if CmpReferenceKeyControlBlock(*Kcb) == 0 {
        ...
        CmpUnLockKcbArray(LockedKcbs);
        return STATUS_UNSUCCESSFUL;            // old *Kcb was already dereferenced above,
                                                // and the caller's *Kcb now points at
                                                // CurrentKcb despite the failure
    }
    ...
}
CmpUnLockKcbArray(LockedKcbs);
```

This is precisely the ordering the upstream commit calls out as wrong: "reference before
unlocking", "unlock before `CmpDereferenceKeyControlBlock`, which must never be called with a
KCB lock held", and the C fix reorders to reference `CurrentKcb` first, unlock, *then*
dereference the old one — inside `ntoskrnl/config/cmparse.c` pre-fix at (old) line
1620-1645. Config-rust is LIVE (`config-rust` cohab-replaces `ntoskrnl/config/cmparse.c`;
`CmpLookInCache` is `#[no_mangle]` at `cmparse.rs:1274`).

Proposed diff (mirrors upstream's reordering):

```diff
--- a/sdk/lib/rust/config-rust/src/cmparse.rs
+++ b/sdk/lib/rust/config-rust/src/cmparse.rs
@@ unsafe fn CmpLookInCache(
     if KeyFoundInCache != 0 {
-        CmpDereferenceKeyControlBlock(*Kcb);
-        *Kcb = CurrentKcb;
-
         if CmpReferenceKeyControlBlock(CurrentKcb) == 0 {
             crate::dbg_print!(
                 b"Could not reference the KCB, too many references (KCB 0x%p)\n\0",
-                Kcb
+                CurrentKcb
             );
             CmpUnLockKcbArray(LockedKcbs);
+            CmpDereferenceKeyControlBlock(*Kcb);
             return STATUS_UNSUCCESSFUL;
         }
+        CmpUnLockKcbArray(LockedKcbs);
+        CmpDereferenceKeyControlBlock(*Kcb);
+        *Kcb = CurrentKcb;

         *Hive = (*CurrentKcb).__bindgen_anon_2.__bindgen_anon_1.KeyHive;
         *Cell = (*CurrentKcb).__bindgen_anon_2.__bindgen_anon_1.KeyCell;
-    }
-
-    CmpUnLockKcbArray(LockedKcbs);
+        return STATUS_SUCCESS;
+    }
+    CmpUnLockKcbArray(LockedKcbs);
     STATUS_SUCCESS
 }
```

### CONFIRMED #3 — `RtlCreateProcessParameters` aliases the caller's environment block instead of copying it (`9989b9ec1d6b`)

`sdk/lib/rust/rtl-ppb/src/lib.rs:191`:

```rust
(*Param).Environment = Environment;
```

This is the exact pre-fix line (`Param->Environment = Environment;`). Upstream's fix copies
the environment block into the tail of the newly-allocated `Param` buffer (`RtlCopyMemory`
into `Dest`, with `Param->Environment = Dest`), because the created process-parameters block
is supposed to own its own copy — `Environment` here is the **caller's** local buffer
(commonly on the stack or in a heap block the caller frees right after the call returns).
Every consumer of `PRTL_USER_PROCESS_PARAMETERS` that later reads `Params->Environment` (e.g.
across a process's lifetime, or after marshalling the block to a new address space) is
reading through a dangling/foreign pointer.

This is boot/process-creation-critical: `RtlCreateProcessParameters` is on the path of every
`CreateProcess`. `rtl-ppb` is a small dedicated crate, clearly a live cohab replacement of
`sdk/lib/rtl/ppb.c`.

Proposed fix mirrors upstream: compute `EnvironmentSize` by walking the double-NUL-terminated
block, add it to the allocation size, `RtlCopyMemory` it into the tail of `Param` after the
existing string-copy calls, and only then set `(*Param).Environment` to the copy's address
(plus set an `EnvironmentSize` field if `RTL_USER_PROCESS_PARAMETERS` in this tree's NDK has
one — needs a one-line check of the struct definition before landing, since the exact field
name/presence is NT6+-structure-dependent and this file may still be targeting the older
layout).

### CONFIRMED #4 — `crypt.c`'s handle-magic checks can crash on garbage handles (`9844effa4104`)

`sdk/lib/rust/advapi32-rust/src/wine/crypt.rs:589-606` (`CryptContextAddRef`, and the same
pattern repeats at `CryptCreateHash:655`, `CryptDecrypt:715`, and others upstream touched):

```rust
let pProv: PCRYPTPROV = hProv as PCRYPTPROV;
if pProv.is_null() { ... }
if (*pProv).dwMagic != MAGIC_CRYPTPROV { ... }   // <- unguarded deref of a caller-supplied,
                                                   //    unvalidated handle-as-pointer
```

Upstream wraps the equivalent C in `_SEH2_TRY`/`_SEH2_EXCEPT` specifically because a
malicious or buggy caller can pass a non-NULL garbage `HCRYPTPROV` (these are raw pointers
cast from a handle, not validated against a handle table), and dereferencing `dwMagic`
segfaults instead of returning `ERROR_INVALID_PARAMETER`. Rust's `unsafe` deref has the
identical failure mode — no exception to catch, straight SIGSEGV/crash. There is no Rust-side
SEH equivalent wired in this function today.

This is userland (advapi32), reachable from the wine-sync'd CryptoAPI test suite the upstream
commit message names directly ("fixes several crashes when running the advapi32 winetest").
Fixing it in Rust needs a real decision (catch_unwind around a probe-read, or a
`ProbeForRead`-equivalent validity check before the first dereference) rather than a blind
port of `_SEH2_TRY`, so I left the diff as a text description rather than a patch.

### CONFIRMED #5 — WM_QUIT swallowed inside the window move/size modal loop (`cf470941031e`)

`sdk/lib/rust/win32k-rust/src/nonclient.rs`, two sites:

- `DefWndStartSizeMove_impl` (fn starts `:479`): the `while hittest == 0` loop's
  `co_IntGetPeekMessage` call is at `:507`; there is no check for `msg.message == WM_QUIT`
  before falling into `IntCallMsgFilter`/the `match msg.message` dispatch at `:512`.
- `DefWndDoSizeMove_impl` (fn starts `:588`): its own peek-message loop
  (`co_IntGetPeekMessage` at line 731, i.e. offset 144 from `:588`) has the identical gap.

Upstream's fix (both sites, symmetric): right after `co_IntGetPeekMessage` returns
successfully, check `if (msg.message == WM_QUIT) { MsqPostQuitMessage(pti, msg.wParam);
return/break; }` — re-posting the quit message so it survives the modal loop instead of being
silently eaten by `PM_REMOVE`. `pti` is already computed at the top of both Rust functions
(`:485` and `:614`) and `MsqPostQuitMessage` already exists as a `#[no_mangle]` Rust function
at `sdk/lib/rust/win32k-rust/src/msgqueue.rs:2045` — this fix is a pure two-site insertion,
nothing new to write.

Concretely reproduces the symptom upstream describes: a WM_QUIT posted while a window is
mid-drag (e.g. during first-boot setup, when the setup thread is tearing itself down) gets
eaten, and the owning thread's outer `GetMessage()` loop never sees the quit — the thread
hangs forever. This is exactly the class of bug worth chasing on this project (a boot-adjacent
hang), and it is unconditionally live in `win32k-rust` (win32k's user32/GDI kernel half is
fully cohab-flipped per the win32k C-tree retirement, `8d3c33ee41e`).

Proposed diff:

```diff
--- a/sdk/lib/rust/win32k-rust/src/nonclient.rs
+++ b/sdk/lib/rust/win32k-rust/src/nonclient.rs
@@ unsafe fn DefWndStartSizeMove_impl(...) -> LONG {
             if co_IntGetPeekMessage(&mut msg, core::ptr::null_mut(), 0, 0, PM_REMOVE, 1) == 0 {
                 return 0;
             }
+            if msg.message == WM_QUIT {
+                MsqPostQuitMessage(pti, msg.wParam as ULONG);
+                return 0;
+            }
             if IntCallMsgFilter(&mut msg, MSGF_SIZE as INT) != 0 {
                 continue;
             }
@@ unsafe fn DefWndDoSizeMove_impl(...) {
         if co_IntGetPeekMessage(core::ptr::addr_of_mut!(msg), core::ptr::null_mut::<c_void>() as _, 0, 0, PM_REMOVE, 1) == 0 {
             break;
         }
+        if msg.message == WM_QUIT {
+            MsqPostQuitMessage(pti, msg.wParam as ULONG);
+            break;
+        }
         if IntCallMsgFilter(core::ptr::addr_of_mut!(msg), MSGF_SIZE as i32) != 0 {
```

### CONFIRMED #6 — non-client update region isn't clipped to the actual dirty region (`b38db9dd2389`)

`sdk/lib/rust/win32k-rust/src/painting.rs:447-` (`IntGetNCUpdateRgn`):

- Line `469` computes `NcType` from `IntGdiGetRgnBox(hRgnNonClient, ...)`; upstream's fix
  reads it from `Window->hrgnUpdate` instead (the actual dirty region, not the whole
  non-client region) — our Rust still has the pre-fix `hRgnNonClient` argument.
- Immediately after the `RGN_DIFF` combine at `471`, upstream adds a second combine —
  `NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, Window->hrgnUpdate, RGN_AND)`, gated on
  `RgnType` being neither `ERROR` nor `NULLREGION` — clipping the non-client region down to
  only the part that's actually dirty. Our Rust (`471`-`482`) has no such second combine at
  all.

Net effect carried into Rust: every paint pass repaints the *entire* non-client area
(caption, borders) instead of just the dirty slice, matching upstream's stated symptom
("excessive repainting of the non-client window area... whenever other windows are being
dragged over"). This is a perf/flicker bug, not memory-unsafety, but is directly the shape
this sweep is meant to find (a wrong-variable read + a missing clip-combine, invisible in a
keyword sweep because the subject says "Clip", not "fix"). `win32k-rust` is LIVE.

Proposed diff:

```diff
--- a/sdk/lib/rust/win32k-rust/src/painting.rs
+++ b/sdk/lib/rust/win32k-rust/src/painting.rs
@@ unsafe fn IntGetNCUpdateRgn(Window: PWND, Validate: i32) -> HRGN {
-        NcType = IntGdiGetRgnBox(hRgnNonClient, &mut update as *mut _ as _ ) as u32;
+        NcType = IntGdiGetRgnBox((*Window).hrgnUpdate, &mut update as *mut _ as _ ) as u32;

         RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, hRgnWindow, RGN_DIFF as i32) as u32;
+        if RgnType != ERROR && RgnType != NULLREGION {
+            RgnType = NtGdiCombineRgn(hRgnNonClient, hRgnNonClient, (*Window).hrgnUpdate, RGN_AND as i32) as u32;
+        }
```

### REFUTED — ACPI `AcpiOsMapMemory` 64-bit truncation (`63deca5a962c`)

The commit's headline change (a synthetic RSDP built from the loader's ACPI BIOS
configuration node, ~180 new lines) is a **net-new UEFI boot-path feature that does not exist
in our Rust at all** — `acpi-rust/src/osl.rs` has no `AcpiBuildLoaderRootPointer` equivalent,
so this part is a feature gap, not a regression, and out of scope for "bugs we carried
forward."

But the commit also contains one unrelated one-line fix, caught by the `sign_or_width_change`
shape: `Address.QuadPart = (ULONG)phys;` → `(ULONGLONG)phys;` inside `AcpiOsMapMemory` — a
genuine truncation bug in upstream C (any physical address above 4GB gets its high 32 bits
dropped before being mapped). Checked our twin:

```
sdk/lib/rust/acpi-rust/src/osl.rs:488-494  (AcpiOsMapMemory)
    let address = make_li(Where as i64);
```

`Where: ACPI_PHYSICAL_ADDRESS`, and `ACPI_PHYSICAL_ADDRESS` is `pub type ... = UINT64;` in
both `acpica-sys/src/bindgen_i386.rs:4672` and `bindgen_amd64.rs:4851`. `Where as i64` is a
same-width 64→64 bit-pattern cast, so the Rust translation never routed the address through a
32-bit intermediate the way the pre-fix C did — this bug does not exist in our Rust. A clean
negative result, worth recording precisely because it shows the sweep can also correctly
clear a lead, not just raise one.

### INCONCLUSIVE — x86 BIOS emulator skipped on UEFI boots (`229f7891a1c7`)

Real, boot-relevant upstream fix (prevents the x86 BIOS emulator from being entered at all on
EFI boots, avoiding an early boot failure) — but `hal/halx86/generic/x86bios.c` has **no
Rust file anywhere** (`grep -rl "fn HalInitializeBios\|fn x86BiosCall"` under
`sdk/lib/rust/` returns nothing, despite `x86bios.c` itself already being absent from the C
tree and the cohab registry naming a `hal-rust/src/x86/x86bios.rs` that does not exist on
disk). This looks like a registry annotation that's stale/aspirational rather than ground
truth — either the function was never actually ported (and the C for it was deleted anyway,
possibly because MINIHAL's boot config never calls into it), or it lives under a name this
sweep didn't think to grep. Needs a human to first establish whether x86 BIOS emulation is
reachable in our boot config at all before this lead means anything.

## Additional survivors, not individually hand-verified (backlog)

These passed the shape+size filter and are plausible but I ran out of budget to open the
Rust side. Listed so the lead isn't lost, ranked roughly by apparent value from the diff
alone:

- `b2addef77194` [NTOS:MM] Add WOW64 PEB/TEB allocation — `added_check` in
  `ntoskrnl/mm/ARM3/pagfault.c`/`procsup.c`. Likely a feature addition (WOW64 support), not a
  regression — same caveat as `b272427baa3b` above; check before treating as a bug.
- `a4621bb2803b` [NTOS:PS] ThreadNameInformation implementation (`ntoskrnl/ps/kill.c`) —
  also reads as new functionality (`Implement...` in the title), not a fix.
- `ef70bdf990` [RTL][NTDLL] `RtlQueryEnvironmentVariable` (`sdk/lib/rtl/env.c`) — new
  implementation, check whether ours already has the bounds check upstream is adding or
  whether it's genuinely new code we haven't ported.
- `5635d244fc` [KERNEL32] UTF-16 surrogate-pair support in `MultiByteToWideChar`
  (`dll/win32/kernel32/winnls/string/nls.c`) — likely a real gap (surrogate pairs silently
  mishandled) worth a follow-up read.
- `d4a3ad8396`, `71ee75477e`, `e182f26cab` — `bang_change`/`constant_change` hits in
  `hal/halx86/legacy/bussupp.c`, `base/services/wkssvc/domain.c`,
  `ntoskrnl/ex/work.c` respectively; small diffs, not opened.
- `06ae7ec211` [FREETYPE][NTGDI][GDI32] "Rewrite GetCharWidth etc." (#8690) — explicitly a
  **rewrite**, the trap called out in the brief (`e6328fbf1ea` precedent: a "Rewrite" title
  that was pure refactor). Treat with extra suspicion; do not assume the flagged
  constant/sign-change hunks inside it are real fixes without reading the whole diff.
- `75e5904c05`, `33dab072e7`, `88d06015f6` — three more CORE-7659 freetype.c commits in the
  *same file* as the already-known `e1bf662d9a` finding. Worth a combined pass since the twin
  (`win32k-rust/src/freetype.rs`) is already open for that finding.

## What this diff-shape sweep in turn misses

- **Multi-line semantic fixes with no small isolated hunk.** The shapes are all built around
  a "1-line before/after" or "brand-new small line" pattern. A fix that restructures control
  flow across 15+ lines with no single line matching a shape (e.g. reordering three existing
  statements, or splitting one loop into two without any one line looking like an
  add/remove-only check) is invisible here exactly like it was invisible to the keyword
  sweep, for the opposite reason: this method needs the fix to be *textually* small, the
  keyword method needed it to be *verbally* announced. `278234259dce` (fstub) is a concrete
  example that slipped past the size filter into "flagged but not sweepable."
- **Whole-function rewrites, even small ones.** `b7510137d962` (Imm32AssignNewLayout) is a
  ~50-line function rewritten top to bottom; my off-by-one detector caught one incidental
  line-pair, but the real semantic change (a completely different ID-scanning algorithm) is
  not expressible as any of the 9 shapes, and reading it properly needs the same care as a
  hand review, which this pass didn't budget for.
- **False positives from cosmetic refactors that happen to match a shape.** `06ae7ec211`'s
  "Rewrite GetCharWidth etc." is exactly the precedent trap named in the brief
  (`e6328fbf1ea`) — I did not fully clear it, only flagged it as suspicious. A shape match is
  a lead, never proof.
- **Feature additions that pass every filter but aren't bugs.** `b272427baa3b` (Job UI
  restriction) and likely `b2addef77194`/`a4621bb280` are net-new functionality dressed in
  `added_check`/`early_return_added` shapes (a brand-new guard clause in brand-new code isn't
  a "fix," it's just how new code is written). The sweep has no way to distinguish "check
  added to existing logic" from "check that is part of code that didn't exist before" — that
  distinction needed a human read of the surrounding diff every time.
- **The 1-hunk-per-commit tag-and-stop rule hides co-occurring bugs.** Since the script takes
  the first shape match per hunk and moves to the next hunk, a hunk that coincidentally
  matches two shapes only gets one label — undercounts by shape, though no candidate is
  dropped.
- **The win32k function-name pairing was done by hand, per candidate, not systematically.**
  I confirmed twins for `cf47094103` and `b38db9dd23` by grepping the exact function name in
  `win32k-rust`; nothing here builds a reusable historical-commit → deleted-C-file →
  Rust-function index the way the brief's tool suggestion implies. A future pass touching
  more of the 320 candidate commits should build that index once rather than repeating the
  grep per lead.
- **Non-C diff shapes are entirely untouched** — no `.asm`/`.S`, no CMake/build-system
  changes, no pure data-table corrections (e.g. a wrong entry in a static array of constants
  spanning many lines wouldn't look like any of the 9 shapes).
- **Self-correction on my own read**: while writing up finding #1 I initially mis-attributed
  a second bug to `FsRtlFastUnlockAll`'s exclusive-lock loop; re-reading the actual upstream
  diff showed the added ownership check there belongs to the *shared*-lock loop, which our
  Rust already had correctly. Only the `FsRtlCheckLockForReadAccess` half is confirmed. Left
  the correction inline above rather than silently fixing the number, per the project's own
  rule about re-deriving claims before relaying them.
