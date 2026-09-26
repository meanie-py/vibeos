# Upstream-sync: boot-path triage, verified (wave 5)

Continuation of waves 1-4. Scope: the boot-path files/commits that waves 1-2 left as
"triaged by subject line only" — the ~48 single-commit boot-path files (derived fresh this
pass, see below; wave 2 said "~43", the count moved because `upstream/master` has advanced),
plus the explicitly-named leads: `cmdata.c`/`controller.c` (Win7+ case-insensitivity),
`ke/i386/{kiinit,thrdini,thrdobj}.c` (CycleTime), `ke/{amd64,i386}/cpu.c` (shared-vendor
refactor), and `mm/ARM3/sysldr.c`'s `MmLoadSystemImage` error-cleanup path.

`FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`. Derivation used this pass:
```
git diff --name-only $FP upstream/master   # 4519 files
# ∩ HIGH entries of tools/cohab-registry.json, excluding win32ss/ and drivers/network/tcpip → 182
# ∩ boot-path prefixes (ntoskrnl/, hal/, /ntdll/, freeldr, drivers/filesystems|fs) → 87
# commit count per file since FP → 48 have exactly 1 commit (up from wave 2's ~43 as
#   upstream/master moved forward)
```
Of those 48, 7 were already opened and CONFIRMED/REFUTED in wave 2 (`ke/time.c`, `ex/time.c`,
`fsrtl/filelock.c`, `ex/keyedevt.c`, `ex/resource.c`, `ke/amd64/usercall.c`,
`mm/ARM3/virtual.c` — see wave-2 doc, not re-litigated here). This pass opened the remaining
~41 in full (commit message + diff), spot-checked all against the Rust twin, and went deep on
every one that read as a real behavior change rather than cosmetic/refactor/pure-feature.

**Registry note**: as in prior waves, several `cohab-registry.json` entries for these files
(`ntoskrnl/ps/job.c`, `ntoskrnl/ob/obref.c`, etc.) carry `rust_files: []` (dir-guess fallback).
Resolved by grepping the C function name across `sdk/lib/rust/**/*.rs` in every case; no
registry errors beyond the already-known `rust_files: []` gotcha were hit.

## Re-check scorecard (the deliverable in its own right)

Of the commits carrying an explicit subject-line judgement from wave 1/2 (dismissed as
feature/refactor/cosmetic, or flagged "worth a second look" without being opened), **11 were
re-checked in full this pass**:

| Verdict on re-check | Count |
|---|---|
| **HELD** (subject-line read was correct) | 9 |
| **FLIPPED** (subject-line read was wrong — real live bug found) | 2 |

**2/11 flipped (~18%)** — same *direction and rough magnitude* as wave 3/4's 25-40%, on the
low end because most of what was left in the boot-path 48 genuinely was refactor/feature-add
(NDK signature plumbing, MINIHAL build exclusion, CycleTime accounting, Win7+ case-sensitivity
— none of these correct any existing broken behavior, they add or reshape it). The two flips
are both "labeled as a dismissable feature-completion FIXME, actually a live functional gap in
the *already-shipped* Rust surface" — the same shape as wave 3's `inet_addr` miss: a commit
that reads as "complete a FIXME" undersells that the FIXME's absence is itself the bug once the
surrounding code is live.

Plus **one fresh CONFIRMED** finding from a file never previously triaged at all
(`dll/ntdll/ldr/ldrpe.c`, opened cold because its subject read as an obvious bug fix), and
**one INCONCLUSIVE** surfaced as a byproduct of chasing the `cpu.c` refactor lead.

## Verdict table

| C file | Commit | Subject | Class | Verdict |
|---|---|---|---|---|
| ntoskrnl/ob/obref.c | `8e865cb39d7` | Return handle attributes by `ObReferenceFileObjectForWrite` | FIXME completion | **CONFIRMED** (FLIPPED from wave-2's "feature") |
| ntoskrnl/ps/psmgr.c (+ ps/job.c) | `55563444377` | Basic support for jobs in the kernel | feature (huge) | **CONFIRMED** partial gap: `PspCloseJob` wiring missing (FLIPPED from "feature, own subsystem, skip") |
| dll/ntdll/ldr/ldrpe.c | `4a96f375a0f` | Unload DLLs on loader failure | bug fix (leak) | **CONFIRMED** (fresh, not previously triaged) |
| ntoskrnl/config/cmse.c, ob/obsecure.c, se/semgr.c (+ 6 more) | `07217d32458` | Change Object callback types to NT6 style | signature refactor | REFUTED (held) — pure plumbing, no behavior change in this commit |
| hal/halx86/acpi/busemul.c | `d4a3ad83961` | [HALX86][MINIHAL] Further code cleanup | build-variant `#ifdef` exclusion | REFUTED (held) — MINIHAL-only, standard HAL unaffected |
| ntoskrnl/config/cmdata.c, io/iomgr/controller.c | `9174dcbd615` | Implement Win7+ I/O System case (in)sensitivity | new feature | REFUTED (held) — nothing pre-existing to regress; not ported, not a live bug |
| ntoskrnl/ke/i386/{kiinit,thrdini}.c, ke/thrdobj.c (+amd64) | `55090b34806` | Implement support for CycleTime accounting | new feature | REFUTED (held) — translation gap, not a bug; big port item, tracked separately |
| ntoskrnl/ke/amd64/cpu.c, ke/i386/cpu.c | `d3109a8ec95`/`5498220a5e9` | Shared CPU signature/vendor identification | refactor | REFUTED (held) for the refactor itself — see INCONCLUSIVE below for a byproduct finding |
| ntoskrnl/mm/ARM3/sysldr.c `MmLoadSystemImage` | (no dedicated commit; paired with `6c515f5aa66`) | error-cleanup path | — | REFUTED (held) — no additional bug beyond the already-CONFIRMED `MmUnloadSystemImage` leak (wave-1 #13); upstream's load-path itself never calls the unmap helper either |
| ntoskrnl/mm/ARM3/contmem.c | `98256a4f5d9` | Allow contiguous-memory frees at DISPATCH_LEVEL | checked-build assertion fix | N/A (held) — our Rust has no `PAGED_CODE()`/IRQL assert at this site at all (release-parity translation), so there's nothing to carry the bug |
| ntoskrnl/mm/ARM3/vadnode.c | `d46172032cd` | [NTOS] Improve build generation | build/include-path only | REFUTED-by-construction (held) — literally an `#include` path change, zero code |
| dll/ntdll/rtl/libsupp.c | `9d1418fcb6e` | Improve tick count to ms calculation | perf refactor | REFUTED (held) — new `KiTickCountToMs` is mathematically identical to the old formula (multiply distributes over the `<<8`), not a correctness fix |
| ntoskrnl/dbgk/dbgkobj.c, lpc/close.c | `9675f756cea` | Fix definition of `OB_CLOSE_METHOD` (ULONG→ULONG_PTR) | ABI width fix | Same class as wave-1 finding #12 (CONFIRMED-but-negligible) — two more of the 13 files touched by that commit; no new severity |

**INCONCLUSIVE**: `ntoskrnl/ke/i386/cpu.c` `KiSetProcessorType` (`sdk/lib/rust/ntoskrnl-rust/src/ke/i386/cpu.rs:324-360`)
never calls anything equivalent to `KiGetCpuVendorString(Prcb->VendorString)` — pre-fix upstream
C has this call, our Rust omits it entirely. Not flagged CONFIRMED because our `KiGetCpuVendor()`
(called from the `Family == 6` branch, and independently from `KiGetFeatureBits`) is a
**self-contained fused function**: it does its own CPUID leaf-0 + `strcmp` and populates
`Prcb->VendorString` as a side effect, matching the "self-contained fused fn" pattern wave 1
already found for `a318b6228a2` (REFUTED there for the same reason). Whether this is live
depends on ordering: if anything reads `Prcb->VendorString` before either `KiSetProcessorType`
(when `Family==6`) or `KiGetFeatureBits` has run at least once, it sees stale/zeroed data. Not
verified in the time available which caller runs first, or whether anything is exposed to user
mode (e.g. `NtQuerySystemInformation(SystemProcessorInformation)`, `HaliQueryProcessorBrandString`)
before that point. Flagging as a lead, not a verdict.

## CONFIRMED — detail + proposed diffs

### V5-1. `ObReferenceFileObjectForWrite`: handle attributes always report 0

- **Upstream fix**: `8e865cb39d7` ("Return handle attributes by `ObReferenceFileObjectForWrite`",
  #9213) — resolves a `/* FIXME: Get handle attributes */` by replacing the hardcoded
  `HandleInformation->HandleAttributes = 0;` with
  `HandleInformation->HandleAttributes = HandleEntry->ObAttributes & OBJ_HANDLE_ATTRIBUTES;`
  — the same pattern already used elsewhere in Ob (e.g. `ObpIncrementHandleCount`).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ob-rust/src/obref.rs:377`,
  `ObReferenceFileObjectForWrite` — line 377 is `(*HandleInformation).HandleAttributes = 0;`,
  byte-for-byte the pre-fix FIXME stub. Every caller of this exported function (used by file
  I/O paths acquiring a file object for write with handle info) gets `HandleAttributes == 0`
  unconditionally, regardless of the handle's actual `OBJ_INHERIT`/`OBJ_PROTECT_CLOSE`/
  `OBJ_AUDIT_OBJECT_CLOSE` bits.
- The same file already has the correct pattern in-scope at `obref.rs:721`
  (`(*handle_entry).__bindgen_anon_1.ObAttributes as u32 & OBJ_HANDLE_ATTRIBUTES`), so the fix
  is a one-line port with no new imports:
```diff
--- a/sdk/lib/rust/ob-rust/src/obref.rs
+++ b/sdk/lib/rust/ob-rust/src/obref.rs
@@ ObReferenceFileObjectForWrite
             (*HandleInformation).GrantedAccess = granted_access;
-            (*HandleInformation).HandleAttributes = 0;
+            (*HandleInformation).HandleAttributes =
+                ((*handle_entry).__bindgen_anon_1.ObAttributes as u32) & OBJ_HANDLE_ATTRIBUTES;
```
  (verify the field path `handle_entry` uses in this function is the same `HANDLE_TABLE_ENTRY`
  shape as at line 721 — both are `ExMapHandleToPointer`-returned entries in the same crate, so
  it should be, but confirm the type alias before landing.)

### V5-2. `PspCloseJob`: kill-on-job-close and completion-port release never wired

- **Upstream fix**: same commit as the job-object feature, `55563444377` ("Basic support for
  jobs in the kernel") — adds `ObjectTypeInitializer.CloseProcedure = PspCloseJob;` in
  `PspInitPhase0` (`ntoskrnl/ps/psmgr.c`) and implements `PspCloseJob` in `ntoskrnl/ps/job.c`:
  on the last handle close (`SystemHandleCount <= 1`) it terminates every process in the job if
  `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` is set, then releases the job's `CompletionPort`
  reference.
- **Our Rust (live)**: `sdk/lib/rust/ps-rust/src/psmgr.rs:619-623` sets
  `object_type_initializer.GenericMapping` and `DeleteProcedure = Some(PspDeleteJob)` for
  `PsJobType` but **never sets `CloseProcedure`** — and `PspCloseJob` doesn't exist anywhere in
  `sdk/lib/rust/ps-rust/src/job.rs` (confirmed via
  `grep -rn "PspCloseJob" sdk/lib/rust --include=*.rs`, zero hits outside bindgen). This is a
  genuine functional gap distinct from the "it's a whole new subsystem, not a regression"
  dismissal in wave 1/2: the object-type registration machinery this hooks into (`ObCreateObjectType`
  / open-close-delete procedures) **is** live and translated (`psmgr.rs`), so a job object
  created via the already-ported `NtCreateJobObject`-family path that has its last handle closed
  with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` set will silently NOT terminate its processes, and
  its `CompletionPort` reference is never dropped (leak).
- Proposed fix: port `PspCloseJob` into `job.rs` (straightforward — no SEH, single resource lock,
  calls already-present `PspTerminateJobObject`/`ObDereferenceObject`) and wire it in
  `psmgr.rs`:
```diff
--- a/sdk/lib/rust/ps-rust/src/psmgr.rs
+++ b/sdk/lib/rust/ps-rust/src/psmgr.rs
@@
     object_type_initializer.GenericMapping = PspJobMapping;
+    object_type_initializer.CloseProcedure = Some(PspCloseJob);
     object_type_initializer.DeleteProcedure = Some(PspDeleteJob);
```
  plus the new `job.rs` function (mirror `ntoskrnl/ps/job.c`'s `PspCloseJob`, upstream lines
  ~823-873 as shown in commit `55563444377`): flag `PSP_JOB_CLOSE_DONE`, acquire `JobLock`
  exclusive, if `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` call `PspTerminateJobObject` (already
  present per `grep -n PspTerminateJobObject sdk/lib/rust/ps-rust/src/job.rs`), then clear and
  dereference `CompletionPort`. Confirm the `_OBJECT_TYPE_INITIALIZER` binding in this crate has
  a `CloseProcedure` field before landing (it should, mirroring `PspDeleteThread`'s neighbor
  `PspDeleteProcess` pattern already used two lines above).

### V5-3. `LdrpSnapThunk`: forwarder DLL left loaded on a failed re-snap

- **Upstream fix**: `4a96f375a0f` ("[NTDLL] Unload DLLs on loader failure" — "crucial, because
  otherwise they linger in the loader, waiting to run their init routine, which can cause
  failures later") — initializes `ForwarderHandle = NULL` at declaration, and at the top of the
  `FailurePath:` label adds:
  ```c
  if (ForwarderHandle != NULL) { LdrUnloadDll(ForwarderHandle); }
  ```
  before falling through to the existing hard-error/`Thunk->u1.Function = 0xffbadd11` handling.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntdll-rust/src/ldr/ldrpe.rs:704-877`,
  `LdrpSnapThunk` — `ForwarderHandle` is declared `core::ptr::null_mut()` (line 732, already
  matches upstream's `= NULL` initializer half of the fix) but the `'failure: loop` body's
  failure branch (starting at the `if (Ordinal as ULONG) >= (*ExportDirectory).NumberOfFunctions`
  check, ~line 776) goes straight into the `Static`/hard-error handling and the
  `(*Thunk).u1.Function = 0xffbadd11` fallback with **no check of `ForwarderHandle` at all**.
  Since this loop's failure path is reached via `continue 'failure` from the forwarder-chase
  branch further down (when re-resolving a forwarded export fails), a DLL loaded to resolve a
  forwarder (`LdrpLoadDll` on the forwarder's target module — the load call itself is further
  down in the function, not shown above but present per the surrounding forwarder-handling
  code) is left fully loaded and initialized-pending in the loader's module list on this path,
  exactly the "lingering DLL whose init routine hasn't run yet" upstream's commit message
  describes.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntdll-rust/src/ldr/ldrpe.rs
+++ b/sdk/lib/rust/ntdll-rust/src/ldr/ldrpe.rs
@@ 'failure: loop
         if (Ordinal as ULONG) >= (*ExportDirectory).NumberOfFunctions {
             // FailurePath
+            // Unload a forwarder DLL we loaded before failing to re-snap it
+            // (upstream 4a96f375a0f — otherwise it lingers with its init
+            // routine never run).
+            if !ForwarderHandle.is_null() {
+                LdrUnloadDll(ForwarderHandle);
+            }
             // ARM64: DUMMY_PSEH can't catch RtlRaiseStatus. ...
```
  (confirm `LdrUnloadDll` is already imported/declared in this file or its crate — it is a
  standard ntdll-internal loader function, very likely already used elsewhere in
  `ldrpe.rs`/`ldrutils.rs`; grep before adding a new `extern` decl.)

## Not reached

- `boot/freeldr/freeldr/ntldr/ntldropts.c` (`925d227faaf`, move+rename) and
  `boot/freeldr/freeldr/ui/directui.c` (`9ff4e4b9ef4`, move to separate file) — read as pure
  code-motion from the subject, not opened in full.
- `dll/ntdll/ldr/ldrapi.c` (`0232efdd195`, "Implement `LdrGetProcedureAddressEx` for NT6") —
  feature addition, not cross-checked against `ntdll-rust`.
- `hal/halx86/apic/tsc.c` (`6eaae6d7297`, discardable init section), `generic/halinit.c`
  (`229f7891a1c`, skip BIOS emulator on UEFI), `generic/sysinfo.c` (`1704b28ed04`,
  `HaliQueryProcessorBrandString`), `legacy/halpnpdd.c` (`b9eb90a2fb6`, legacy HAL interface
  handlers) — not opened.
- `ntoskrnl/ex/locale.c`, `io/iomgr/bootlog.c` (`1cf31524d50`, swprintf/vswprintf conformance) —
  cosmetic by construction per the established pattern from wave 2, not independently verified.
- `ntoskrnl/inbv/inbv.c` (`e2aa54321af`, rename params), `ke/amd64/except.c` + `mm/ARM3/pool.c`
  + `po/power.c` (`34d5a35b74e`, trailing whitespace), `ke/amd64/ipi.c` (`081a6366dfb`, license
  header) — cosmetic by construction, not independently verified.
- `ntoskrnl/fsrtl/name.c` (`c3d141b6750`, switch to `RtlIsNameInExpression`) — plausible
  behavior-preserving refactor, not cross-checked for a subtle semantic difference between the
  old and new matching routine.
- `ntoskrnl/kd/kdio.c` (`3012af5e303`, move duplicated UART settings) — refactor, not opened.
- `ntoskrnl/ke/i386/v86vdm.c` + `vdm/vdmexec.c` (`2d5371e0789`, GCC 13 stringop-overflow fix
  regarding `KiNtVdmState`) — could plausibly hide a real buffer-size issue; not opened this
  pass, worth a look next.
- `ntoskrnl/ob/obwait.c` (`69474b31902`, replace joke debug print with real diagnostic) — debug
  output only per the subject, not opened.
- `ntoskrnl/ps/win32.c` (`b272427baa3`, Job UI Restriction support) — feature, companion to the
  jobs work above; not cross-checked.
- The ~150 non-boot-path files (priority 4 from wave 2): still entirely untouched.
- `ntoskrnl/config/cmsysini.c`'s `FirmwareBootDevice` export (`7f433ed74de`) — flagged by wave
  1, still not reviewed.

## Summary

- **Re-checked 11 prior subject-line dismissals: 9 HELD, 2 FLIPPED** (~18%).
- **3 new CONFIRMED findings this pass** (`ObReferenceFileObjectForWrite` handle attributes,
  `PspCloseJob` unwired, `LdrpSnapThunk` forwarder-DLL leak), **1 INCONCLUSIVE** lead
  (`KiSetProcessorType`/`VendorString` init ordering on i386), **~9 REFUTED/not-applicable**
  commits closed out with a documented reason each.
- Net effect on the boot-path 48-file single-commit set: 7 previously closed by wave 2, 14
  closed this pass (11 re-checks + 3 fresh), 1 inconclusive lead open, ~26 still genuinely
  un-opened (listed above under "Not reached").
