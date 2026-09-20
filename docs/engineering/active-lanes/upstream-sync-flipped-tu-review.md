# Upstream-sync: flipped-TU review (HIGH-confidence cohab set)

Scope: of the 195 files that (a) upstream changed since our fork point
`5213cf717cff6946945922e6a7e03d87b0a58e4f` and (b) are HIGH-confidence flipped TUs per
`tools/cohab-registry.json` (the C is on disk, commented out of CMake, superseded by a
`#[no_mangle]` Rust twin) — this pass reviewed the boot-path subset. win32ss/ (owned by
another agent, C deleted there) and drivers/network/tcpip (netstack, done) were excluded from
the 195 as instructed; four commits already fixed in-tree (`c0f18d059a`, `ae0e4ee305`,
`e1bf662d9a`, `e7e1465812`) and `39ea28d4c6c` (KeContextToTrapFrame) were skipped when hit.

Derivation command:
```
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git diff --name-only $FP upstream/master   # 4519 files
# ∩ HIGH entries of tools/cohab-registry.json (by c_path key, not stem match) → 195
```
Excluding win32ss/ and tcpip leaves **182** files in scope for this lane. Restricting to the
boot path (ntoskrnl, hal, ntdll, freeldr, fs drivers) gives **85** files; **39** of those have
≥2 upstream commits since the fork. This pass reviewed those 39, plus a handful of
single-commit boot-path files whose commit message implied a functional fix.

**A registry gotcha discovered along the way**: several `rust_files` entries in the registry
are `[]` — the registry only recorded a directory-level "block-hint" comment
(`# OB subsystem -- replaced by Rust (ntoskrnl-rust/src/ob/)`), and that directory
(`ntoskrnl-rust/src/ob/`, `.../fstub/`) **does not exist**. The actual translations live in
separate crates not named in the hint: `sdk/lib/rust/ob-rust/src/*.rs`,
`sdk/lib/rust/fstub-rust/src/*.rs`. Anyone resuming this pass should resolve the Rust twin by
searching for the C function name across `sdk/lib/rust/**/*.rs`, not by trusting `rust_files`
or the block-hint path literally.

## Verdict table

| C file | Commit | Subject | Class | Verdict |
|---|---|---|---|---|
| ntoskrnl/fstub/fstubex.c | `13ac2918305` | Fix disks matching condition (`IoGetBootDiskInformation`) | bug fix | **CONFIRMED** |
| ntoskrnl/fstub/fstubex.c | `bb30c56d6ff` | Fix `FstubDbgPrintSetPartitionEx` indexing | bug fix (debug-only) | REFUTED |
| ntoskrnl/fstub/fstubex.c | others (5) | formatting / SAL / macro conforming | cosmetic/refactor | — |
| ntoskrnl/fstub/disksup.c | `278234259dc` | Fix drive-letter assignment algorithm | bug fix | **CONFIRMED** |
| ntoskrnl/fstub/disksup.c | others (6) | swprintf/vswprintf, simplifications, SAL | refactor/cosmetic | — |
| ntoskrnl/kd/kdmain.c | `956d9f7973f` | Don't overrun cmdline parsing DEBUGPORT | bug fix | **CONFIRMED** |
| ntoskrnl/kd/kdmain.c | `b7c3432070f` | Restore BAUDRATE option support | bug fix (feature restore) | **CONFIRMED** (low severity) |
| ntoskrnl/kd/kdmain.c | others (3) | const-ify, comments | cosmetic | — |
| ntoskrnl/kd/i386/kdserial.c | `bc7d0f1ec68` | Use `DEFAULT_DEBUG_BAUD_RATE`, not `DEFAULT_BAUD_RATE` | bug fix | **CONFIRMED** |
| ntoskrnl/kd/i386/kdserial.c | others (2) | message text, header move | cosmetic | — |
| ntoskrnl/ob/obhandle.c | `1902929c80b` | Unlock object header on quota-charge failure | bug fix | **CONFIRMED** (×2 sites) |
| ntoskrnl/ob/obhandle.c | `c9780d9095f` | Validate ACCESS_SYSTEM_SECURITY | security fix | **CONFIRMED** |
| ntoskrnl/ob/obhandle.c | others (3) | rename, typo, callback-type NT6 | cosmetic/refactor | — |
| ntoskrnl/ke/i386/cpu.c | `a318b6228a2` | Fix CPU vendor retrieval order | bug fix | REFUTED |
| ntoskrnl/ke/amd64/thrdini.c | `e3e58ac1aac` | Fix `KiInitializeContextThread` (stop routing through `KeContextToTrapFrame`) | bug fix | **CONFIRMED** (moderate) |
| ntoskrnl/ke/amd64/thrdini.c | others (4) | CycleTime, whitespace, WOW64 CMTEB, RtlInitializeContext (different file) | feature/cosmetic | — |
| ntoskrnl/ex/pushlock.c | `4acacdb7e90` | Fix shared push lock list optimization (`ExfAcquirePushLockShared` clobbers NewValue via CAS before use) | bug fix | **CONFIRMED** |
| ntoskrnl/ex/pushlock.c | `f42ee0e23bb` | Fix push lock release retry livelock (`ExfReleasePushLock`/`Shared`: `continue` without refreshing OldValue) | bug fix | **CONFIRMED** (×2 sites) |
| ntoskrnl/ex/pushlock.c | others (3) | comment clarifications | cosmetic | — |
| ntoskrnl/mm/ARM3/section.c | `5b4805aee0a` | Fix section-view hash calc on x64 (ULONG truncation) | bug fix | REFUTED (Rust already `ULONG_PTR`) |
| ntoskrnl/mm/ARM3/section.c | others (3) | ASSERT, SAL2 | cosmetic | — |
| dll/ntdll/ldr/ldrutils.c | `4a96f375a0f` | Unload DLLs on loader-init failure | bug fix | **CONFIRMED** (structurally displaced) |
| dll/ntdll/ldr/ldrutils.c | others (2) | TLS support add+revert | feature | — |
| ntoskrnl/config/cmparse.c | `e40c4d89cfb` | Fix lock-leak in `CmpLookInCache` | bug fix | REFUTED (Rust already unlocks on failure) |
| ntoskrnl/config/cmparse.c | `a45832a8ae4` | Fix locking in `CmpDoOpen` (symlink KCB) | bug fix | **CONFIRMED** |
| dll/ntdll/rtl/libsupp.c | `77b88c48a4b` | Fix read/write of KSYSTEM_TIME | bug fix | N/A for this file (`RtlGetTickCount`'s existing double-read loop was already correct; the real bug is `KiWriteSystemTime`/`ExpSetTimeZoneInformation`, not in the 195 set as reviewed — see "not reached") |
| ntoskrnl/ke/amd64/kiinit.c | `8aac5c4c872` | Move KD init after `HalInitializeProcessor` | bug fix (narrow: PCI KD transport only) | not investigated in Rust (low expected impact on our smoke config) |
| hal/halx86/generic/dma.c | `442892d14d5` | Improve S/G memory allocation limiting | bug fix | not investigated |
| ntoskrnl/mm/ARM3/sysldr.c | `6c515f5aa66` | Release system image PTEs on unload | bug fix (leak) | not investigated |

Everything else hit in the 39-file boot-path ≥2-commit list (`ldrinit.c` TLS revert,
`process.c`/`kill.c`/`query.c`/`job.c` job-object + Wow64/ThreadName features, `context.c`
already-fixed + license change, `procsup.c`/`pagfault.c` WOW64 PEB/TEB feature,
`config.c`/`cmconfig.c`/`cmsysini.c` new registry-value features, `wmi/guidobj.c` /
`mm/section.c` / `ex/win32k.c` NDK callback-type churn, `fsrtl/fastio.c` macro readability,
`se/srm.c` logon-session hash-table rework, `ex/work.c` more-worker-threads feature,
`hal/.../pcibus.c` / `bussupp.c` / `usage.c` cleanups, `ke/amd64/cpu.c` shared-vendor
refactor, `ke/processor.c` new API) were read at the commit-message + diff-stat level and are
feature/refactor/cosmetic, not bug fixes — no further action.

## CONFIRMED findings — detail + proposed diffs

### 1. `IoGetBootDiskInformation`: boolean-precedence bug drops GPT boot-disk detection

- **Upstream fix**: `13ac2918305`, `ntoskrnl/fstub/fstubex.c:1964-1980` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/fstub-rust/src/fstubex.rs:1715-1717`,
  `#[no_mangle] pub unsafe extern "system" fn IoGetBootDiskInformation` (line 1586).
- The condition is `((SingleDisk && DiskCount==1) || VerifySig) && MBR`. Upstream's fix is
  `(SingleDisk && DiskCount==1 && MBR) || VerifySig` — i.e. today the `&& MBR` is ANDed onto
  the *whole* expression, so a GPT disk whose signature legitimately matches the ARC entry is
  still rejected. `IoGetBootDiskInformation` populates `BOOTDISK_INFORMATION`/ARC-name data
  used by setup/recovery tooling on the boot disk; on a GPT-partitioned boot disk this silently
  fails to record the match.
- Proposed diff:
```diff
--- a/sdk/lib/rust/fstub-rust/src/fstubex.rs
+++ b/sdk/lib/rust/fstub-rust/src/fstubex.rs
@@
-            if ((SingleDisk != 0 && DiskCount == 1) ||
-                (IopVerifyDiskSignature(DriveLayout, ArcDiskSignature, core::ptr::addr_of_mut!(Signature)) != 0)) &&
-                ((*DriveLayout).PartitionStyle == 0) {
+            if (SingleDisk != 0 && DiskCount == 1 &&
+                (*DriveLayout).PartitionStyle == 0) ||
+                (IopVerifyDiskSignature(DriveLayout, ArcDiskSignature, core::ptr::addr_of_mut!(Signature)) != 0) {
```

### 2. `xHalIoAssignDriveLetters_impl`: pre-fix drive-letter assignment algorithm

- **Upstream fix**: `278234259dc`, `ntoskrnl/fstub/disksup.c:1041-1131` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/fstub-rust/src/disksup.rs:992-1220`
  (`xHalIoAssignDriveLetters_impl`, dispatched from the `extern "C"`/`extern "fastcall"`
  wrappers at lines 972/983 — this is boot-time HAL drive-letter assignment).
- Same structural bug upstream describes: the "does this disk have a bootable/primary
  partition" loop is split into an outer `if HalpQueryPartitionType(partition1) fails` /
  `else` pair instead of one unified per-partition loop, so BootablePartition/DataPartition are
  only recognized inside the `else` arm (i.e. only when partition 1's query *succeeds*), and
  the trailing `HarddiskCount = i as u32 + 1` (line 1220) discards the `Devices[i]`-derived
  disk number for every disk after the first pass, feeding the wrong device number into the
  later logical-partition and system-partition passes when a `Devices` derangement map is in
  effect.
- Proposed fix: restructure the first `while i < DiskCount` loop (lines 1144-1221) to match
  upstream's unified loop — replace the `if !NT_SUCCESS(HalpQueryPartitionType(partition1))`
  / `else` split with a single loop starting at `PartitionCount = 1` that checks
  `PartitionType == BootablePartition || PartitionType == DataPartition` every iteration
  (breaking only on `BootablePartition`), falls through to the primary-partition scan only if
  no bootable/data partition was found, and drops the trailing `HarddiskCount = i as u32 + 1`
  (dead once every per-iteration `HarddiskCount` assignment above is corrected — see the C diff
  for the exact target shape). This one is large enough (upstream's diff replaces ~90 lines) that
  it should be hand-ported function-by-function against the fixed C rather than patched
  piecemeal; flagging file:line for the next pass to pick up rather than including a full
  mechanical diff here.

### 3. `KdDebuggerInitialize0`: unconditional pointer bump past end of DEBUGPORT string

- **Upstream fix**: `956d9f7973f`, `ntoskrnl/kd/kdmain.c:137-152` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/kd-rust/src/kdmain.rs:206-211`.
- `Port = Port.add(1)` (line 210) runs unconditionally after skipping spaces, even when the
  command line ends right there (e.g. `"...DEBUGPORT  "` with no `=value`), walking one byte
  past the NUL terminator before `KdpGetDebugMode` dereferences it.
- Proposed diff:
```diff
--- a/sdk/lib/rust/kd-rust/src/kdmain.rs
+++ b/sdk/lib/rust/kd-rust/src/kdmain.rs
@@
         Port = Port.add(b"DEBUGPORT\0".len() - 1);
         while *Port == b' ' as i8 {
             Port = Port.add(1);
         }
-        Port = Port.add(1);
+        if *Port != 0 {
+            Port = Port.add(1);
+        }
         Port = KdpGetDebugMode(Port);
```

### 4. `KdDebuggerInitialize0`: BAUDRATE option never restored

- **Upstream fix**: `b7c3432070f`, `ntoskrnl/kd/kdmain.c` (post-fork; restores a regression from
  an earlier upstream commit `95faf65ebf`, predating our fork).
- **Our Rust**: `sdk/lib/rust/kd-rust/src/kdmain.rs` — no `BAUDRATE` string scan or
  `SerialPortInfo.BaudRate` override exists anywhere in the function; the port always inits at
  the compiled-in default baud rate.
- Low boot-path severity (only affects users overriding the debug baud rate via boot
  parameters), but a real missing feature vs. upstream. Proposed fix: port upstream's added
  block wholesale — scan `CommandLine` for `"BAUDRATE"`, parse the trailing integer with an
  `atol` equivalent, and feed it into whichever struct field the Rust twin uses for the serial
  port's initial baud rate before calling `CpInitialize`.

### 5. i386 kernel debugger defaults to 19200 baud instead of 115200

- **Upstream fix**: `bc7d0f1ec68`, `ntoskrnl/kd/i386/kdserial.c:75-78` (renames
  `DEFAULT_BAUD_RATE` → `DEFAULT_DEBUG_BAUD_RATE`; these are *different constants*,
  19200 vs. 115200 — confirmed via `sdk/lib/rust/kdcom-serial-rust/src/kdcom.rs:44-45`).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/i386/kdserial.rs:38-40`,
  `KdPortInitializeEx` — hardcodes the literal `19200` as the fallback when
  `PortInformation.BaudRate == 0`.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntoskrnl-rust/src/i386/kdserial.rs
+++ b/sdk/lib/rust/ntoskrnl-rust/src/i386/kdserial.rs
@@
         if (*PortInformation).BaudRate == 0 {
-            19200
+            DEFAULT_DEBUG_BAUD_RATE
         } else {
             (*PortInformation).BaudRate
         },
```
  (import/define `DEFAULT_DEBUG_BAUD_RATE = 115200` in this crate, matching `kd-rust`'s
  `kdio.rs:110` and `kdcom-serial-rust`'s `kdcom.rs:44`.)

### 6. `ObpIncrementHandleCount` / `ObpIncrementUnnamedHandleCount`: object-lock leak on quota failure

- **Upstream fix**: `1902929c80b`, `ntoskrnl/ob/obhandle.c:826-1060` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ob-rust/src/obhandle.rs`:
  - `ObpIncrementHandleCount` (fn at line 603): lines 633-637.
  - `ObpIncrementUnnamedHandleCount` (fn at line ~775): lines 808-812.
- Both acquire `ObpAcquireObjectLock(object_header)`, then on `ObpChargeQuotaForObject`
  failure `return status;` **without** calling `ObpReleaseObjectLock`. Both functions are on
  the live handle-creation path (`ObInsertObject`/`ObOpenObjectByPointer` etc. route through
  these), so a quota-charge failure — plausible under real memory pressure, and exactly the
  kind of thing a `HANG_TIMEOUT` looks like weeks later — permanently wedges that object's
  lock.
- Proposed diff (same shape at both sites):
```diff
--- a/sdk/lib/rust/ob-rust/src/obhandle.rs
+++ b/sdk/lib/rust/ob-rust/src/obhandle.rs
@@ ObpIncrementHandleCount
     status = ObpChargeQuotaForObject(object_header, object_type, &mut new_object as *mut _);
-    if status < 0 {
-        return status;
-    }
+    if status < 0 {
+        ObpReleaseObjectLock(object_header);
+        return status;
+    }
@@ ObpIncrementUnnamedHandleCount
     status = ObpChargeQuotaForObject(object_header, object_type, &mut new_object as *mut _);
-    if status < 0 {
-        return status;
-    }
+    if status < 0 {
+        ObpReleaseObjectLock(object_header);
+        return status;
+    }
```

### 7. `ObpIncrementHandleCount`: `ACCESS_SYSTEM_SECURITY` granted without privilege check

- **Upstream fix**: `c9780d9095f`, `ntoskrnl/ob/obhandle.c:954-970` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ob-rust/src/obhandle.rs:715` — the
  `RemainingDesiredAccess & ACCESS_SYSTEM_SECURITY` branch is present but empty of any
  privilege check (matches the original `/* FIXME: TODO */ DPRINT1("...not validated!")`
  stub), so a caller requesting `ACCESS_SYSTEM_SECURITY` without `SeSecurityPrivilege` is never
  rejected and the bit is never cleared from `RemainingDesiredAccess`.
- Proposed diff: port upstream's added block — `if !SeSinglePrivilegeCheck(SeSecurityPrivilege,
  probe_mode) { status = STATUS_PRIVILEGE_NOT_HELD; goto/return Quickie-equivalent; }` else
  `(*access_state).PreviouslyGrantedAccess |= ACCESS_SYSTEM_SECURITY;
  (*access_state).RemainingDesiredAccess &= !ACCESS_SYSTEM_SECURITY;` — at
  `sdk/lib/rust/ob-rust/src/obhandle.rs:715`, following the control-flow shape already used at
  the lock-leak fix above (there is already a `Quickie`-style tail in this function per the
  cleanup-on-failure pattern noted in finding #6).

### 8. `KiInitializeContextThread` (amd64): still routes through `KeContextToTrapFrame`

- **Upstream fix**: `e3e58ac1aac`, `ntoskrnl/ke/amd64/thrdini.c:46-140` (post-fork) — replaces
  the `KeContextToTrapFrame` call with manual field-by-field copies, notably forcing
  `TrapFrame->Rbp = 0` (not `Context->Rbp`) to match Vista/real-NT thread-init behavior.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/thrdini.rs:137-141`
  still calls `KeContextToTrapFrame(Context, &(*InitFrame).ExceptionFrame, TrapFrame, ...)`.
- Note: this is *not* the same bug as `39ea28d4c6c` (already fixed — `KeContextToTrapFrame`
  deciding privilege from attacker-controlled `SegCs`); that fix only patches
  `KeContextToTrapFrame` itself, it doesn't remove the call site here. Upstream's rationale for
  bypassing the shared helper for thread *initialization* specifically is the `Rbp` handling
  difference (a freshly-initialized thread's frame must start with `Rbp = 0`, not whatever
  garbage/caller value sits in the `CONTEXT` the creator supplied) plus a few other
  init-specific fields (see the full upstream diff for the complete field list — `Rax`, `Rcx`,
  `Rdx`, `R8`-`R11` on the trap frame; `Rbx`, `Rsi`, `Rdi`, `R12`-`R15` on the exception frame;
  `Rip`/`Rsp`/`EFlags`; segment selectors).
- This one needs the full field-by-field port rather than a 3-line diff; flagging file:line for
  hand-porting rather than guessing at a partial diff, since getting `Rbp` right while missing
  a segment register would be a regression in the other direction.

### 9. `LdrpGetProcedureAddress`/init-routine path: failed init doesn't unload the DLL

- **Upstream fix**: `4a96f375a0f`, `dll/ntdll/ldr/ldrutils.c:2378-2392` (post-fork) — adds
  `if (!NT_SUCCESS(Status)) { LdrUnloadDll(LdrEntry->DllBase); LdrEntry = NULL; }` right after
  the `_SEH2_TRY` that calls the DLL's init routine.
- **Our Rust (structurally different, live)**: `sdk/lib/rust/ntdll-rust/src/ldr/ldrutils.rs`.
  Our `LdrpGetProcedureAddress` (line 2278) doesn't inline the SEH-wrapped init-routine call
  the C does at this site — it delegates to a separate `LdrpRunInitializeRoutines` (declared at
  line 110, defined in `ldrinit.rs:866`) at line 2414. That helper's own failure path (checked
  through line ~930 of `ldrinit.rs`) does **not** call `LdrUnloadDll` anywhere — it only
  `DbgPrint`s on a kernel32-post-import failure and returns the status. So the underlying bug
  upstream fixes (a DLL that fails its init routine lingers in the loader instead of being
  unloaded, "waiting to run their init routine, which can cause failures later") is present,
  just reachable through a different call path than the one upstream patched.
- Proposed fix: since the translation consolidated the SEH-wrapped init call into
  `LdrpRunInitializeRoutines`, the unload-on-failure belongs in that shared function's failure
  path (`sdk/lib/rust/ntdll-rust/src/ldr/ldrinit.rs`, the `Status = func()` /
  `!NT_SUCCESS(Status)` handling around line ~930), not back in `LdrpGetProcedureAddress` — a
  faithful line-for-line port of the upstream diff would silently do nothing here because the
  code it targets no longer exists at that call site. Left as file:line rather than a diff
  because it needs someone to trace exactly which failing entry corresponds to `LdrEntry` in
  the multi-DLL init loop before calling `LdrUnloadDll` on the right module.

### 10. `CmpDoOpen`: symlink KCB — wrong KCB locked, Delete checked before locking, no reference before unlock

- **Upstream fix**: `a45832a8ae4`, `ntoskrnl/config/cmparse.c:667-720` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/config-rust/src/cmparse.rs:650-735`
  (`CmpDoOpen`, the `ControlFlags & 0x00000004` / symlink branch at lines ~717-732).
- Three bugs present simultaneously, matching upstream's description exactly:
  1. `(*Kcb).Delete()` (the *real*, symlinked-to KCB) is read at line 720 while only the
     *symlink's* KCB hash bucket is locked (`GET_HASH_INDEX((**CachedKcb)...)`) — the real
     KCB is never locked before its `Delete` flag is inspected, a TOCTOU race.
  2. Even in the non-deleted path (lines 728-732), the code locks
     `GET_HASH_INDEX((**CachedKcb)...)` (the symlink's own KCB) instead of
     `GET_HASH_INDEX((*Kcb)...)` (the real KCB) before proceeding to use/reference `Kcb`.
  3. `CmpUnLockKcbArray(KcbsLocked)` is called (dropping all locks) before `Kcb` (the real KCB)
     is ever referenced — nothing pins it in existence across the lock-drop.
  This path is exercised by every open through a registry symlink, most prominently
  `\Registry\Machine\SYSTEM\CurrentControlSet`, which is resolved on essentially every boot-time
  driver/service registry lookup.
- Proposed fix: port upstream's restructure — reference the real `Kcb` with
  `CmpReferenceKeyControlBlock(Kcb)` (returning `STATUS_REPARSE` on failure) *before* dropping
  the symlink's lock; then `CmpUnLockKcbArray`, lock `GET_HASH_INDEX((*Kcb)...)` (the real KCB,
  not the symlink's), set `IsLockShared = 0`, and only then check `(*Kcb).Delete()` — on
  Delete, additionally `CmpDereferenceKeyControlBlock(Kcb)` before the existing reparse cleanup.
  This is a control-flow rewrite of ~25 lines at `cmparse.rs:717-735`; hand-port against the
  upstream diff rather than a mechanical patch, since the exact `KcbsLocked[]` bookkeeping must
  match this crate's translation of the two-element lock-tracking array.

## Not reached (resume here)

- **Single-commit boot-path files (46 of the 85)**: not opened at all this pass. List:
  `grep '^1\s' docs-generation-scratch` — re-derive via
  `awk -F'\t' '$1==1' <bootpath-ranked-list>` against the intersection script above (the
  intermediate rank files were scratch, not committed — regenerate with the commands at the
  top of this doc).
- **Non-boot-path files (97 of 182, everything outside ntoskrnl/hal/ntdll/freeldr/fs-drivers)**:
  not reached. Highest-churn ones worth prioritizing next:
  `dll/win32/netcfgx/tcpipconf_notify.c` (19 commits), `base/services/dhcpcsvc/dhcpcsvc.c` (11),
  `dll/win32/netcfgx/inetcfgcomp_iface.c` (11), `dll/win32/syssetup/wizard.c` (10),
  `dll/win32/shell32/wine/shellord.c` (8), `base/services/dhcpcsvc/dhcp/api.c` (8).
- **Named but not chased to a verdict**: `dll/ntdll/rtl/libsupp.c`'s companion bug
  (`KiWriteSystemTime`/`ExpSetTimeZoneInformation` field-order bug per `77b88c48a4b`) likely
  lives in `ntoskrnl/ke/time.c` or `ntoskrnl/ex/time.c` (both single-commit boot-path files,
  not opened this pass) — worth checking first in the next session.
- `ntoskrnl/ke/amd64/kiinit.c` (`8aac5c4c872`, KD-init ordering) and
  `hal/halx86/generic/dma.c` (`442892d14d5`, S/G allocation limiting) and
  `ntoskrnl/mm/ARM3/sysldr.c` (`6c515f5aa66`, PTE release on unload) were identified as
  plausible bug fixes by commit message + diff read but not cross-checked against their Rust
  twins.

## Summary

- Files reached: **~15 of 195** opened at commit level (11 in the first deep batch + 4 more
  fast-checked), out of 39 boot-path files with ≥2 commits; 24 of those 39 were classified from
  diff read as refactor/feature only (not further detailed above beyond one line each).
- Verdicts: **7 CONFIRMED** bugs (findings #1, #2, #3, #6, #7, #8, #10 — #4, #5, #9 also
  CONFIRMED but lower severity/structurally displaced), **4 REFUTED** (Rust already matches or
  exceeds the upstream fix), **1 N/A** (bug lives in a different file), **3 not investigated**
  (identified, not cross-checked).
- Highest-value fixes for the boot path specifically: #6 (object-lock leak, every handle
  creation), #10 (registry symlink race on every `CurrentControlSet` lookup), #1/#2 (GPT
  boot-disk / drive-letter assignment, relevant once GPT/multi-disk QEMU configs are exercised),
  #7 (SACL privilege-check gap).