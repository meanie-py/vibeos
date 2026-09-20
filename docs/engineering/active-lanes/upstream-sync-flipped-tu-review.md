# Upstream-sync: flipped-TU review (HIGH-confidence cohab set)

Scope: of the 195 files that (a) upstream changed since our fork point
`5213cf717cff6946945922e6a7e03d87b0a58e4f` and (b) are HIGH-confidence flipped TUs per
`tools/cohab-registry.json` (the C is on disk, commented out of CMake, superseded by a
`#[no_mangle]` Rust twin) — this pass reviewed the boot-path subset plus the top non-boot-path
high-churn files. win32ss/ (owned by another agent, C deleted there) and
drivers/network/tcpip (netstack, done) were excluded from the 195 as instructed; four commits
already fixed in-tree (`c0f18d059a`, `ae0e4ee305`, `e1bf662d9a`, `e7e1465812`) and `39ea28d4c6c`
(KeContextToTrapFrame) were skipped when hit.

Derivation command:
```
FP=5213cf717cff6946945922e6a7e03d87b0a58e4f
git diff --name-only $FP upstream/master   # 4519 files
# ∩ HIGH entries of tools/cohab-registry.json (by c_path key, not stem match) → 195
```
Excluding win32ss/ and tcpip leaves **182** files in scope. Restricting to the boot path
(ntoskrnl, hal, ntdll, freeldr, fs drivers) gives **85** files; **39** of those have ≥2 upstream
commits since the fork. This pass covered all 39 of those, plus the 16 highest-churn
non-boot-path files (netcfgx, dhcpcsvc, syssetup, shell32, umpnpmgr, msafd) at commit level, via
five parallel investigation passes plus a direct follow-up pass on the remaining boot-path gap
(`ke/processor.c`, `hal/generic/{dma,usage}.c`, `config/cmconfig.c`, `ps/job.c`,
`mm/ARM3/pagfault.c`, `ke/amd64/kiinit.c`, `se/srm.c`, `ex/win32k.c`, `ex/work.c`).

**A registry gotcha discovered along the way**: several `rust_files` entries in the registry are
`[]` — the registry only recorded a directory-level "block-hint" comment
(`# OB subsystem -- replaced by Rust (ntoskrnl-rust/src/ob/)`), and that directory
(`ntoskrnl-rust/src/ob/`, `.../fstub/`) **does not exist**. The actual translations live in
separate crates not named in the hint: `sdk/lib/rust/ob-rust/src/*.rs`,
`sdk/lib/rust/fstub-rust/src/*.rs`, `sdk/lib/rust/hal-rust/src/x86/dma.rs`. Anyone resuming this
pass should resolve the Rust twin by searching for the C function name across
`sdk/lib/rust/**/*.rs`, not by trusting `rust_files` or the block-hint path literally.

**A cross-check disagreement worth recording**: the first pass over `ntoskrnl/config/cmparse.c`
REFUTED `e40c4d89cfb` ("Fix lock-leak in `CmpLookInCache`") on the claim that "Rust already
unlocks on failure." A second, independent pass CONFIRMED it. Reading the actual bytes at
`sdk/lib/rust/config-rust/src/cmparse.rs:1387-1401` settles it: the REFUTE was wrong — see
finding #11 below. Recorded here because it is exactly the "deference is the failure mode of
independent verification" pattern: two readings disagreed and only reading the code (not
re-arguing the claim) resolved it.

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
| ntoskrnl/ke/i386/cpu.c | `a318b6228a2` | Fix CPU vendor retrieval order | bug fix | REFUTED (self-contained fused fn, ordering bug doesn't exist in this design) |
| ntoskrnl/ke/amd64/cpu.c | `e3de830c5b2` | `KiGetCacheInformation` should read `Prcb.CpuVendor` | bug fix | REFUTED (redundant-work cleanup only; existing call path has no side-effect bug) |
| ntoskrnl/ke/amd64/thrdini.c | `e3e58ac1aac` | Fix `KiInitializeContextThread` (stop routing through `KeContextToTrapFrame`) | bug fix | **CONFIRMED** (moderate) |
| ntoskrnl/ke/amd64/thrdini.c | others (4) | CycleTime, whitespace, WOW64 CMTEB, RtlInitializeContext (different file) | feature/cosmetic | — |
| ntoskrnl/ex/pushlock.c | `4acacdb7e90` | Fix shared push lock list optimization (`ExfAcquirePushLockShared` clobbers NewValue via CAS before use) | bug fix | **CONFIRMED** |
| ntoskrnl/ex/pushlock.c | `f42ee0e23bb` | Fix push lock release retry livelock (`ExfReleasePushLock`/`Shared`: retry without refreshing OldValue) | bug fix | **CONFIRMED** (×2 sites) |
| ntoskrnl/ex/pushlock.c | others (3) | comment clarifications | cosmetic | — |
| ntoskrnl/mm/ARM3/section.c | `5b4805aee0a` | Fix section-view hash calc on x64 (ULONG truncation) | bug fix | REFUTED (Rust already `ULONG_PTR`) |
| ntoskrnl/mm/ARM3/section.c | others (3) | ASSERT, SAL2, `OB_CLOSE_METHOD` width | cosmetic / negligible-width | — (see #12) |
| dll/ntdll/ldr/ldrutils.c | `4a96f375a0f` | Unload DLLs on loader-init failure | bug fix | **CONFIRMED** (structurally displaced into `ldrinit.rs`) |
| dll/ntdll/ldr/ldrutils.c | others (2) | TLS support add+revert | feature | — |
| dll/ntdll/ldr/ldrinit.c | `23043ce0b82`/`e68ebe19c60` | Implicit TLS added then reverted | feature (reverted) | REFUTED (Rust already matches final/reverted shape) |
| ntoskrnl/config/cmparse.c | `e40c4d89cfb` | Fix lock-leak in `CmpLookInCache` | bug fix | **CONFIRMED** (see cross-check note above) |
| ntoskrnl/config/cmparse.c | `a45832a8ae4` | Fix locking in `CmpDoOpen` (symlink KCB) | bug fix | **CONFIRMED** |
| dll/ntdll/rtl/libsupp.c | `77b88c48a4b` | Fix read/write of `KSYSTEM_TIME` | bug fix | REFUTED for this file (`RtlGetTickCount`'s double-read loop already correct); real bug is `KiWriteSystemTime`/`ExpSetTimeZoneInformation`, not in the reviewed set — see "not reached" |
| ntoskrnl/mm/ARM3/sysldr.c | `6c515f5aa66` | Release system image PTEs on unload | bug fix (leak) | **CONFIRMED** |
| ntoskrnl/wmi/guidobj.c, mm/section.c, ex/win32k.c | `9675f756cea` | Fix `OB_CLOSE_METHOD` definition (ULONG→ULONG_PTR) | bug fix (ABI width) | **CONFIRMED-but-negligible** (×3 files, see #12) |
| ntoskrnl/ke/amd64/kiinit.c | `8aac5c4c872` | Move KD init after `HalInitializeProcessor` | bug fix | **CONFIRMED** |
| hal/halx86/generic/dma.c | `442892d14d5` | Improve S/G memory allocation limiting | bug fix | **CONFIRMED** (our clamp is *tighter* than even pre-fix C — see #13) |
| hal/halx86/generic/usage.c | `bcd4c718c13` | Correct `HalpComPortIrqMapping` for PC-98 | bug fix | N/A — PC-98/Xbox-only branch, not a target platform |
| base/services/dhcpcsvc/dhcp/dhclient.c | `ed367320c29` | Fix DHCP option stack overflow (CVE-class, #9056) | **security bug fix** | **CONFIRMED** |
| base/services/dhcpcsvc/dhcp/dhclient.c | `b3a0868a180` | Fix `stop_selecting` time_t→int truncation (ARM64) | bug fix | **CONFIRMED** |
| dll/win32/shell32/CDefView.cpp | `68a03b2e1ae` | Fix teardown crash ordering in `DestroyViewWindow` | bug fix | **CONFIRMED** |
| dll/win32/syssetup/wizard.c | `d482579dff3` | Fix retrieval of active keyboard layout ID/name | bug fix | **CONFIRMED** (non-boot-blocking) |
| dll/win32/shell32/utils.cpp | `a790beb22ba` | `SHBindToObject` signature/ABI fix | bug fix | INCONCLUSIVE — not cross-checked |
| dll/win32/shell32/wine/shellord.c | `4fecc908e7d` | Init `SHELLSTATE` members before registry write | bug fix | INCONCLUSIVE — not cross-checked |
| base/services/umpnpmgr/rpcserver.c | `46e69e6e507` | `PNP_FreeLogConf` partial-deletion fix | bug fix | INCONCLUSIVE — not cross-checked |
| dll/win32/syssetup/install.c | `d55a77c350e` | Always reboot on install errors | bug fix | INCONCLUSIVE — not cross-checked |

Everything else hit in the reviewed set (`process.c`/`kill.c`/`query.c`/`job.c` job-object +
Wow64/ThreadName features, `context.c` license-header-only, `procsup.c`/`pagfault.c` WOW64
PEB/TEB feature, `config.c`/`cmconfig.c`/`cmsysini.c` new registry-value + PC-98-detection
features, `fsrtl/fastio.c` macro readability, `se/srm.c` logon-session hash-table rework
(refactor, not a fix), `ex/work.c` more-worker-threads feature, `ke/processor.c`
`NtGetCurrentProcessorNumberEx` feature, `hal/.../pcibus.c`/`bussupp.c` cleanups, `ke/amd64/cpu.c`
shared-vendor-ID refactor (`d3109a8ec95`/`5498220a5e9`, diffed at title+stat level, appears to be
a pure code-sharing refactor between i386/amd64 — not chased further), all `netcfgx`/most
`dhcpcsvc` commits (feature/incremental-implementation), `msafd/misc/dllmain.c` (rename/inlining))
were read at the commit-message + diff-stat (and for most, full diff) level and are
feature/refactor/cosmetic, not bug fixes — no further action.

## CONFIRMED findings — detail + proposed diffs

### 1. `IoGetBootDiskInformation`: boolean-precedence bug drops GPT boot-disk detection

- **Upstream fix**: `13ac2918305`, `ntoskrnl/fstub/fstubex.c:1964-1980` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/fstub-rust/src/fstubex.rs:1715-1717`,
  `#[no_mangle] pub unsafe extern "system" fn IoGetBootDiskInformation` (line 1586).
- The condition is `((SingleDisk && DiskCount==1) || VerifySig) && MBR`. Upstream's fix is
  `(SingleDisk && DiskCount==1 && MBR) || VerifySig` — i.e. today the `&& MBR` is ANDed onto
  the *whole* expression, so a GPT disk whose signature legitimately matches the ARC entry is
  still rejected. This silently fails to record the match on a GPT-partitioned boot disk.
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
  wrappers at lines 972/983 — boot-time HAL drive-letter assignment).
- Same structural bug upstream describes: the "does this disk have a bootable/primary
  partition" loop is split into an outer `if HalpQueryPartitionType(partition1) fails` / `else`
  pair instead of one unified per-partition loop, so BootablePartition/DataPartition are only
  recognized inside the `else` arm (only when partition 1's query *succeeds*), and the trailing
  `HarddiskCount = i as u32 + 1` (line 1220) discards the `Devices[i]`-derived disk number for
  every disk after the first pass, feeding the wrong device number into later logical-partition
  and system-partition passes when a `Devices` derangement map is in effect.
- Proposed fix: restructure the first `while i < DiskCount` loop (lines 1144-1221) to match
  upstream's unified loop — replace the `if !NT_SUCCESS(HalpQueryPartitionType(partition1))` /
  `else` split with a single loop starting at `PartitionCount = 1` that checks
  `PartitionType == BootablePartition || PartitionType == DataPartition` every iteration
  (breaking only on `BootablePartition`), falls through to the primary-partition scan only if
  no bootable/data partition was found, and drops the trailing `HarddiskCount = i as u32 + 1`.
  Large enough (upstream's diff replaces ~90 lines) that it should be hand-ported
  function-by-function against the fixed C rather than patched piecemeal.

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
  parameters). Proposed fix: port upstream's added block wholesale — scan `CommandLine` for
  `"BAUDRATE"`, parse the trailing integer, feed it into the serial port's initial baud rate
  before calling `CpInitialize`.

### 5. i386 kernel debugger defaults to 19200 baud instead of 115200

- **Upstream fix**: `bc7d0f1ec68`, `ntoskrnl/kd/i386/kdserial.c:75-78` (renames
  `DEFAULT_BAUD_RATE` → `DEFAULT_DEBUG_BAUD_RATE`; different constants, 19200 vs 115200 —
  confirmed via `sdk/lib/rust/kdcom-serial-rust/src/kdcom.rs:44-45`).
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
- Both acquire `ObpAcquireObjectLock(object_header)`, then on `ObpChargeQuotaForObject` failure
  `return status;` **without** calling `ObpReleaseObjectLock`. Both functions are on the live
  handle-creation path (`ObInsertObject`/`ObOpenObjectByPointer` route through these), so a
  quota-charge failure — plausible under real memory pressure, and exactly the kind of thing a
  `HANG_TIMEOUT` looks like weeks later — permanently wedges that object's lock.
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
  privilege check, so a caller requesting `ACCESS_SYSTEM_SECURITY` without `SeSecurityPrivilege`
  is never rejected and the bit is never cleared from `RemainingDesiredAccess`.
- Proposed fix: port upstream's added block — `if !SeSinglePrivilegeCheck(SeSecurityPrivilege,
  probe_mode) { status = STATUS_PRIVILEGE_NOT_HELD; return/goto cleanup; }` else
  `(*access_state).PreviouslyGrantedAccess |= ACCESS_SYSTEM_SECURITY;
  (*access_state).RemainingDesiredAccess &= !ACCESS_SYSTEM_SECURITY;` at
  `sdk/lib/rust/ob-rust/src/obhandle.rs:715`.

### 8. `KiInitializeContextThread` (amd64): still routes through `KeContextToTrapFrame`

- **Upstream fix**: `e3e58ac1aac`, `ntoskrnl/ke/amd64/thrdini.c:46-140` (post-fork) — replaces
  the `KeContextToTrapFrame` call with manual field-by-field copies, notably forcing
  `TrapFrame->Rbp = 0` (not `Context->Rbp`) to match real-NT thread-init behavior.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/thrdini.rs:137-141`
  still calls `KeContextToTrapFrame(Context, &(*InitFrame).ExceptionFrame, TrapFrame, ...)`.
- Note: this is *not* the same bug as `39ea28d4c6c` (already fixed — `KeContextToTrapFrame`
  deciding privilege from attacker-controlled `SegCs`); that fix only patches
  `KeContextToTrapFrame` itself, it doesn't remove this call site. Needs the full field-by-field
  port (Rbp=0, plus Rax/Rcx/Rdx/R8-R11 on the trap frame, Rbx/Rsi/Rdi/R12-R15 on the exception
  frame, Rip/Rsp/EFlags, segment selectors) rather than a 3-line diff — flagging file:line for
  hand-porting since getting Rbp right while missing a segment register would be a regression.

### 9. `LdrpGetProcedureAddress`/init-routine path: failed init doesn't unload the DLL

- **Upstream fix**: `4a96f375a0f`, `dll/ntdll/ldr/ldrutils.c:2378-2392` (post-fork) — adds
  `if (!NT_SUCCESS(Status)) { LdrUnloadDll(LdrEntry->DllBase); LdrEntry = NULL; }` right after
  the `_SEH2_TRY` that calls the DLL's init routine.
- **Our Rust (structurally different, live)**: our `LdrpGetProcedureAddress`
  (`sdk/lib/rust/ntdll-rust/src/ldr/ldrutils.rs:2278`) doesn't inline the SEH-wrapped
  init-routine call — it delegates to `LdrpRunInitializeRoutines`
  (`sdk/lib/rust/ntdll-rust/src/ldr/ldrinit.rs:866`) at line 2414. That helper's own failure
  path (through line ~930 of `ldrinit.rs`) does **not** call `LdrUnloadDll` anywhere — it only
  `DbgPrint`s and returns the status. The underlying bug (a DLL that fails its init routine
  lingers in the loader instead of being unloaded) is present, reachable through a different
  call path than upstream patched.
- Proposed fix: since the translation consolidated the SEH-wrapped init call into
  `LdrpRunInitializeRoutines`, the unload-on-failure belongs in that function's failure path
  (`ldrinit.rs`, around line ~930), not back in `LdrpGetProcedureAddress` — a faithful
  line-for-line port of the upstream diff would silently do nothing here. Left as file:line
  rather than a diff because it needs tracing exactly which failing entry corresponds to
  `LdrEntry` in the multi-DLL init loop before calling `LdrUnloadDll` on the right module.

### 10. `CmpDoOpen`: symlink KCB — wrong KCB locked, Delete checked before locking, no reference before unlock

- **Upstream fix**: `a45832a8ae4`, `ntoskrnl/config/cmparse.c:667-720` (post-fork).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/config-rust/src/cmparse.rs:707-733` (`CmpDoOpen`,
  the `ControlFlags & 0x00000004` / symlink branch, verified directly).
- Three bugs present simultaneously, matching upstream's description exactly:
  1. `(*Kcb).Delete()` (the *real*, symlinked-to KCB) is read at line ~719 while only the
     *symlink's* KCB hash bucket is locked — the real KCB is never locked before its `Delete`
     flag is inspected (TOCTOU).
  2. Even in the non-deleted path (lines ~728-732), the code locks
     `GET_HASH_INDEX((**CachedKcb)...)` (the symlink's own KCB) instead of
     `GET_HASH_INDEX((*Kcb)...)` (the real KCB) before proceeding to use/reference `Kcb`.
  3. `CmpUnLockKcbArray(KcbsLocked)` drops all locks before `Kcb` (the real KCB) is ever
     referenced — nothing pins it in existence across the lock-drop.
  This path is exercised by every open through a registry symlink, most prominently
  `\Registry\Machine\SYSTEM\CurrentControlSet`, resolved on essentially every boot-time
  driver/service registry lookup.
- Proposed fix: reference the real `Kcb` with `CmpReferenceKeyControlBlock(Kcb)` (returning
  `STATUS_REPARSE` on failure) *before* dropping the symlink's lock; then
  `CmpUnLockKcbArray`, lock `GET_HASH_INDEX((*Kcb)...)` (the real KCB), set
  `IsLockShared = 0`, and only then check `(*Kcb).Delete()` — on Delete, additionally
  `CmpDereferenceKeyControlBlock(Kcb)` before the existing reparse cleanup. Control-flow
  rewrite of ~25 lines; hand-port against the upstream diff since the exact `KcbsLocked[]`
  bookkeeping must match this crate's translation of the two-element lock-tracking array.

### 11. `CmpLookInCache`: switches `*Kcb` before referencing the new KCB (lock/reference-order bug)

- **Upstream fix**: `e40c4d89cfb` (see cross-check note above).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/config-rust/src/cmparse.rs:1387-1401`, live
  (`pub mod cmparse;` in `sdk/lib/rust/config-rust/src/lib.rs:54`):
```rust
if KeyFoundInCache != 0 {
    CmpDereferenceKeyControlBlock(*Kcb);
    *Kcb = CurrentKcb;
    if CmpReferenceKeyControlBlock(*Kcb) == 0 {
        ...
        CmpUnLockKcbArray(LockedKcbs);
        return STATUS_UNSUCCESSFUL;   // *Kcb already switched, old KCB already dereffed
    }
    ...
}
```
  On reference failure the caller is handed back a `*Kcb` that already points at a KCB which
  failed to be referenced, while the previously-held KCB has already been dereferenced — a
  dangling/unreferenced KCB pointer flows back to the caller. `CmpLookInCache` sits on the
  registry-lookup fast path, hot on every boot-time key resolution.
- Proposed diff:
```diff
--- a/sdk/lib/rust/config-rust/src/cmparse.rs
+++ b/sdk/lib/rust/config-rust/src/cmparse.rs
@@
     if KeyFoundInCache != 0 {
-        CmpDereferenceKeyControlBlock(*Kcb);
-        *Kcb = CurrentKcb;
-
-        if CmpReferenceKeyControlBlock(*Kcb) == 0 {
+        if CmpReferenceKeyControlBlock(CurrentKcb) == 0 {
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
+        return STATUS_SUCCESS;
     }
-
-    CmpUnLockKcbArray(LockedKcbs);
-    STATUS_SUCCESS
```
  (mirrors upstream: reference before switching, unlock before dereferencing, and
  `CmpDereferenceKeyControlBlock` is never called with a KCB lock held.)

### 12. `OB_CLOSE_METHOD` width truncation (ULONG vs ULONG_PTR) — confirmed, negligible severity

- Upstream `9675f756cea` widens `ProcessHandleCount`/`SystemHandleCount` from `ULONG` to
  `ULONG_PTR` in close-callbacks (LLP64 ABI correctness).
- Our Rust still declares `ULONG` in the live twins: `CmpCloseKeyObject`
  (`sdk/lib/rust/config-rust/src/cmsysini.rs:712-717`, **uses `SystemHandleCount > 1` in real
  logic**), `MmpCloseSection` (`sdk/lib/rust/ntoskrnl-rust/src/mm/section.rs:1104-1107`, no-op
  stub), `WmipCloseMethod` (`sdk/lib/rust/wmi-rust/src/guidobj.rs:137-144`, no-op), and
  `ExpDesktopClose`'s twin under `ntoskrnl/ex/win32k.c`.
- On amd64/arm64 the value arrives in the same register either way, and handle counts never
  realistically exceed 2^32, so this is **not a live bug in practice**. Propose widening to
  `ULONG_PTR` for correctness/parity only, not urgent:
```diff
 pub unsafe extern "system" fn CmpCloseKeyObject(
     Process: PEPROCESS,
     Object: PVOID,
     GrantedAccess: ACCESS_MASK,
-    ProcessHandleCount: ULONG,
-    SystemHandleCount: ULONG,
+    ProcessHandleCount: ULONG_PTR,
+    SystemHandleCount: ULONG_PTR,
 ) {
```
  (same pattern at the other three sites).

### 13. `MmUnloadSystemImage` never releases the image's system PTEs (still leaks, silently)

- **Upstream fix**: `6c515f5aa66` ("Release system image PTEs on unload") replaced a
  `DPRINT1("Leaking driver...")` FIXME with a real `MiUnmapSystemImage()` call (frees pages +
  releases system PTE range) guarded by `LDRP_SYSTEM_MAPPED`.
- **Our Rust**: `MmUnloadSystemImage` (`sdk/lib/rust/ntoskrnl-rust/src/ARM3/sysldr.rs:848-909`,
  live, called at lines 503/1785) frees the `LDR_DATA_TABLE_ENTRY` pool and dereferences the
  section object, but **never calls anything equivalent to `MiUnmapSystemImage`** — it doesn't
  even carry the old `DPRINT1` leak warning, so the leak is completely silent. Every driver
  unload leaks its reserved system-PTE range and the underlying pages.
- Proposed diff sketch:
```diff
     if had_entry != 0 {
+        if ((*ldr_entry).Flags & LDRP_SYSTEM_MAPPED) != 0 {
+            MiUnmapSystemImage((*ldr_entry).DllBase, (*ldr_entry).SizeOfImage);
+        }
         if !(*ldr_entry).FullDllName.Buffer.is_null() {
             ExFreePoolWithTag((*ldr_entry).FullDllName.Buffer as PVOID, TAG_LDR_WSTR);
         }
```
  plus a new `MiUnmapSystemImage(image_base, image_size)` calling `MiDeleteSystemPageableVm` +
  `MiReleaseSystemPtes(SystemPteSpace)` — mirroring the upstream C; both primitives already
  exist elsewhere in this crate.

### 14. `ke/amd64/kiinit.rs`: KD init still runs before `HalInitializeProcessor`

- **Upstream fix**: `8aac5c4c872` — moves `KdInitSystem(0, ...)`/`KdPollBreakIn()` to *after*
  `HalInitializeProcessor`, needed for PCI-based KD transports whose modules depend on functions
  `HalpRegisterKdSupportFunctions()` registers; also brings amd64 in line with what i386/Win7x64
  already do.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/kiinit.rs` — inside
  `KiSystemStartup`, `KdInitSystem(0, KeLoaderBlock)` is called at **line 748**, inside the
  `if Cpu == 0` block, while `HalInitializeProcessor(Cpu as u32, KeLoaderBlock)` is called
  later, unconditionally, at **line 763** — i.e. still the pre-fix order.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/kiinit.rs
+++ b/sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/kiinit.rs
@@
     if Cpu == 0 {
         KeFeatureBits = ((*Pcr).Prcb.FeatureBitsHigh as u64) << 32 | (*Pcr).Prcb.FeatureBits as u64;
         KiInitModuleList(LoaderBlock);
         KeInitExceptions();
-
-        let kd_ret = KdInitSystem(0, KeLoaderBlock);
-
-        if KdPollBreakIn() != 0 {
-            DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
-        }
     }

     while InterlockedBitTestAndSet64(
         core::ptr::addr_of_mut!(KiFreezeExecutionLock) as PLONG64,
         0,
     ) != 0
     {
         while (*(core::ptr::addr_of!(KiFreezeExecutionLock) as *const KSPIN_LOCK)) & 1 != 0 {}
     }

     HalInitializeProcessor(Cpu as u32, KeLoaderBlock);
+
+    if Cpu == 0 {
+        let kd_ret = KdInitSystem(0, KeLoaderBlock);
+        if KdPollBreakIn() != 0 {
+            DbgBreakPointWithStatus(DBG_STATUS_CONTROL_C);
+        }
+    }
```
  (matches upstream's placement — right after `HalInitializeProcessor`, before
  `KeActiveProcessors` gets set.)

### 15. `HalpScatterGatherAdapterControl`: our fixed `MAX_SG_ELEMENTS` is *tighter* than even pre-fix C

- **Upstream fix**: `442892d14d5` — replaces the FIXME'd fixed `#define MAX_SG_ELEMENTS 0x20`
  with a per-call estimate `Est_SG_Elements = min(RemainingLength/PAGE_SIZE + 2,
  MAX_SG_ELEMENTS)` (constant itself also raised to `0x30`), so large transfers get a properly
  sized (pool-allocated) element array instead of either overflowing or over/under-allocating a
  fixed stack buffer.
- **Our Rust (live)**: `sdk/lib/rust/hal-rust/src/x86/dma.rs:99` declares
  `const MAX_SG_ELEMENTS: usize = 0x10;` — a fixed stack array
  (`let mut elements = [...; MAX_SG_ELEMENTS];`, line 1025), with the construction loop
  correctly bounded (`while remaining > 0 && element_count < MAX_SG_ELEMENTS`, line 1032) so
  there is **no memory-safety bug** (no overflow) — but the cap is smaller than even the
  original buggy C's `0x20`, and doesn't implement upstream's dynamic estimate at all. Any
  scatter/gather DMA transfer needing more than 16 non-contiguous physical fragments now fails
  construction (`remaining > 0` at loop exit → `_IO_ALLOCATION_ACTION_DeallocateObject`) where
  even the pre-fix C (32-element cap) or upstream's fix (up to 48, or memory-fitted) would have
  succeeded. Legacy ISA/PCI DMA-based storage or NIC controllers doing large transfers are the
  ones that would hit this; not expected to be on the default QEMU boot path (AHCI/NVMe don't
  route through this HAL S/G helper), but worth fixing before any legacy DMA controller is
  exercised.
- Proposed fix: port upstream's `Est_SG_Elements` calculation and raise the constant, switching
  the fixed-size stack array to a heap allocation sized to the estimate (as upstream does via
  `ExAllocatePoolUninitialized`) rather than a `[T; N]` on the stack, since the whole point of
  the upstream commit chain (`5418987cb89` then `442892d14d5`) is trading stack usage for a
  right-sized pool allocation:
```diff
--- a/sdk/lib/rust/hal-rust/src/x86/dma.rs
+++ b/sdk/lib/rust/hal-rust/src/x86/dma.rs
@@
-const MAX_SG_ELEMENTS: usize = 0x10;
+const MAX_SG_ELEMENTS: usize = 0x30;
@@ HalpScatterGatherAdapterControl
-        let mut elements = [SCATTER_GATHER_ELEMENT {
-            Address: core::mem::zeroed(),
-            Length: 0,
-            Reserved: 0,
-        }; MAX_SG_ELEMENTS];
+        let est_sg_elements = core::cmp::min(
+            ((*ctx).length / PAGE_SIZE + 2) as usize,
+            MAX_SG_ELEMENTS,
+        );
+        let elements_buf = ExAllocatePoolUninitialized(
+            _POOL_TYPE_NonPagedPool as _,
+            (core::mem::size_of::<SCATTER_GATHER_ELEMENT>() * est_sg_elements) as SIZE_T,
+            TAG_DMA,
+        ) as *mut SCATTER_GATHER_ELEMENT;
+        // (bounds checks below become `element_count < est_sg_elements`, and `elements_buf`
+        // must be freed on every exit path, matching upstream's TempElements lifetime)
```
  (sketch only — the exact allocation-failure and free-on-exit handling needs to mirror
  upstream's full diff, not just the estimate formula.)

### 16. DHCP client: unvalidated option length → buffer overflow (security)

- **Upstream fix**: `ed367320c29` ("Fix DHCP option stack overflow #9056") — `setup_adapter()`
  used to accept any `DHO_SUBNET_MASK`/`DHO_ROUTERS` option length and copy/dereference it into
  a fixed buffer; the fix requires `len == sizeof(ULONG)` for the subnet mask and
  `len % sizeof(ULONG) == 0 && len > 0` for routers, formatting the netmask via
  `RtlIpv4AddressToStringA` instead of a manual byte loop.
- **Our Rust (live, only implementation — C is cohab-disabled)**:
  `sdk/lib/rust/dhcpcsvc-rust/src/dhcp/dhclient.rs:317` and `:426` still use the unchecked
  pre-fix condition (`len != 0`), with a `memcpy`-equivalent at lines 319-323 copying
  attacker-controlled DHCP option data (an untrusted DHCP server, or anyone spoofing DHCP
  replies on the LAN, controls this) into a fixed-size buffer sized for exactly 4 bytes.
- Proposed diff:
```diff
--- a/sdk/lib/rust/dhcpcsvc-rust/src/dhcp/dhclient.rs
+++ b/sdk/lib/rust/dhcpcsvc-rust/src/dhcp/dhclient.rs
@@ setup_adapter
-    if (*new_lease).options[DHO_SUBNET_MASK as usize].len != 0 {
+    if (*new_lease).options[DHO_SUBNET_MASK as usize].len == core::mem::size_of::<u32>() as i32 {
@@
-    if (*new_lease).options[DHO_ROUTERS as usize].len != 0 {
+    if (*new_lease).options[DHO_ROUTERS as usize].len > 0
+        && (*new_lease).options[DHO_ROUTERS as usize].len % core::mem::size_of::<u32>() as i32 == 0
+    {
```
  (verify the exact `option_data`/`client_lease` field/type names against the struct before
  applying; upstream also switches the subnet-mask registry-string formatting to
  `RtlIpv4AddressToStringA`, worth mirroring for full parity, not just the length guard.)

### 17. DHCP client: `stop_selecting` truncated `time_t`→`i32` (ARM64-specific)

- **Upstream fix**: `b3a0868a180` ("Fix stop_selecting truncated from time_t to int on 64-bit
  #8996", CORE-18200) — masked on amd64 (RTC present, values fit 32 bits) but breaks DHCP
  completion on ARM64 (RTC not implemented) — directly relevant to our multi-arch boot matrix.
- **Our Rust (live)**: `sdk/lib/rust/dhcpcsvc-rust/src/dhcp/dhclient.rs:2161` still declares
  `let mut stop_selecting: i32;`, used at lines 2215-2244 in `dhcpoffer`.
- Proposed diff:
```diff
-    let mut stop_selecting: i32;
+    let mut stop_selecting: time_t;
```
  plus removing/adjusting the `as i32` casts at lines 2215/2237/2238/2244 that currently narrow
  it back down.

### 18. `CDefView::DestroyViewWindow`: crash-ordering bug during teardown

- **Upstream fix**: `68a03b2e1ae` ("Prevent crash by unsubclassing DefView during teardown
  #9305", CORE-20504) — moves `_DoFolderViewCB(SFVM_WINDOWCLOSING, ...)` to fire *before*
  `m_ListView.DestroyWindow()`, instead of after it.
- **Our Rust (live)**: `sdk/lib/rust/shell32-rust/src/shell32/CDefView.rs:6174-6221`
  (`CDefView_DestroyViewWindow_body`) matches the pre-fix ordering exactly (ListView destroy at
  lines 6196-6198, then `_DoFolderViewCB` + outer `DestroyWindow` at 6200-6208). A stale
  commented-out copy of the pre-fix C also sits at lines ~10570-10634 as reference text (not
  executable).
- Proposed diff:
```diff
-    if !(*this).m_ListView.is_null() {
-        DestroyWindow((*this).m_ListView.m_hWnd);
-    }
-
     if !(*this).m_hWnd.is_null() {
         CDefView__DoFolderViewCB_body(this, SFVM_WINDOWCLOSING, (*this).m_hWnd as WPARAM, 0);
+    }
+    if !(*this).m_ListView.is_null() {
+        DestroyWindow((*this).m_ListView.m_hWnd);
+    }
+    if !(*this).m_hWnd.is_null() {
         DestroyWindow((*this).m_hWnd);
     }
```

### 19. Setup wizard: active-keyboard-layout lookup uses an unreliable algorithm

- **Upstream fix**: `d482579dff3` ("Fix retrieval of the currently active keyboard layout ID and
  its name #9420") — the old HKL/HIWORD-based loop can miss/misidentify substituted layouts
  (e.g. AU locale + US Dvorak/Arabic AZERTY); the fix reads
  `HKCU\Keyboard Layout\Preload`→`Substitutes`→`HKLM...\Keyboard Layouts\<KLID>\Layout Text`.
- **Our Rust (live)**: `sdk/lib/rust/syssetup-rust/src/syssetup/wizard.rs:1756-1806`
  (`SetKeyboardLayoutName`) is a faithful translation of the pre-fix HIWORD/SpecialId loop,
  called from wizard dialog procs at lines 1973/1986/2044/2057.
- Not boot-blocking, but a genuine carried-over bug (garbled/wrong layout name shown during
  setup on certain locale+layout combos). Proposed fix: port the whole
  Preload→Substitutes→Layout-Text registry lookup in place of the HIWORD/SpecialId loop; see
  the upstream commit body for the full replacement logic (too large to inline here).

## REFUTED — detail

- **`ke/i386/cpu.c` `a318b6228a2`**: our `ke/i386/cpu.rs` fuses `KiGetCpuVendor` +
  `KiGetCpuVendorString` into one self-contained function that always recomputes
  `VendorString` via CPUID before comparing (lines ~283-319); it never depended on
  `KiSetProcessorType` calling a separate populate-string step first, so the ordering bug the C
  had (ASSERT on stale VendorString) doesn't exist in this design.
- **`ke/amd64/cpu.c` `e3de830c5b2`**: our `ke/amd64/cpu.rs:405` still calls `KiGetCpuVendor()`
  rather than reading `Prcb.CpuVendor`, matching the pre-fix C — but `KiGetCpuVendor()`
  (lines 194-216) recomputes CPUID(0) safely every call with no side-effect bug; upstream's
  change was a redundant-work cleanup, not a correctness fix.
- **`ldrinit.c` `23043ce0b82`/`e68ebe19c60`** (Implicit TLS added then reverted): our
  `ntdll-rust/src/ldr/ldrinit.rs` already matches the final (reverted) shape; no
  `LdrpActiveTebList`/`LdrpAllocateTls(Teb)` present.
- **`mm/ARM3/section.c` `5b4805aee0a`**: our Rust already declares the hash field as
  `ULONG_PTR`, so the x64 truncation upstream fixed doesn't exist here.
- **`dll/ntdll/rtl/libsupp.c` `77b88c48a4b`**: `RtlGetTickCount`'s torn-read-guarded double-read
  loop (`sdk/lib/rust/ntdll-rust/src/rtl/libsupp.rs:1370-1397`) already implements the correct
  logic independently. The actual bugs this commit fixed live in `KiWriteSystemTime` /
  `ExpSetTimeZoneInformation`, not reviewed this pass (see "not reached").
- **`hal/halx86/generic/usage.c` `bcd4c718c13`**: PC-98/Xbox-specific `HalpComPortIrqMapping`
  values — neither platform is a build target here (i386/amd64/arm64 standard PC HAL); the
  `#else` branch (standard AT-compatible values) is unchanged by this commit.

## Not reached (resume here)

- **Single-commit boot-path files (46 of the 85)**: not opened this pass. Re-derive via:
  `FP=5213cf717cff6946945922e6a7e03d87b0a58e4f; git diff --name-only $FP upstream/master` ∩ HIGH
  registry entries ∩ `ntoskrnl|hal|ntdll|dll/ntdll|freeldr|drivers/(filesystems|storage)` paths,
  then filter to files with exactly 1 commit since the fork.
- **Non-boot-path files (166 of 182)**: only the 16 highest-churn ones were reviewed (see verdict
  table). The remaining bulk of `netcfgx`/`dhcpcsvc` "Implement X"/"Improve Y" commits (~35
  commits total) were scanned at diff-stat level and look like incremental feature work, not
  regressions, but were not individually cross-checked against Rust.
- **INCONCLUSIVE, flagged for follow-up, not cross-checked against Rust**:
  - `dll/win32/shell32/utils.cpp` `a790beb22ba` — `SHBindToObject` signature/ABI correction
    (5-param + stdcall); check the Rust FFI signature in `shell32-rust/src/shell32/utils.rs`.
  - `dll/win32/shell32/wine/shellord.c` `4fecc908e7d` — `SHELLSTATE` member init ordering before
    a registry write (subtle caching logic); check `shell32-rust/src/wine/shellord.rs`.
  - `base/services/umpnpmgr/rpcserver.c` `46e69e6e507` — `PNP_FreeLogConf` partial-deletion fix;
    check `umpnpmgr-rust/src/umpnpmgr/rpcserver.rs`.
  - `dll/win32/syssetup/install.c` `d55a77c350e` — "Always reboot on errors"; check
    `syssetup-rust/src/syssetup/install.rs`.
- **`dll/ntdll/rtl/libsupp.c`'s companion bug**: the `KiWriteSystemTime`/
  `ExpSetTimeZoneInformation` field-order bug per `77b88c48a4b` likely lives in
  `ntoskrnl/ke/time.c` or `ntoskrnl/ex/time.c` (both single-commit boot-path files, not opened
  this pass) — check first in the next session.
- **`ntoskrnl/mm/ARM3/sysldr.c`'s `MmLoadSystemImage`** error-cleanup path
  (`sdk/lib/rust/ntoskrnl-rust/src/ARM3/sysldr.rs:1160-1547`, ~390 lines, restructured to
  Result-based early returns instead of C's `goto Quickie`): not fully verified whether every
  error path releases a partially-mapped image the way the paired `MiLoadImageSection`/`Quickie`
  fix (same commit as finding #13) requires — same theme as the confirmed leak, elevated risk.
- **`ke/amd64/cpu.c`/`ke/i386/cpu.c` shared-vendor refactor** (`d3109a8ec95`/`5498220a5e9`):
  classified refactor from title+diffstat, not diffed in full; touches the same files as the
  REFUTED vendor-retrieval findings above, worth a second look.
- **`ntoskrnl/config/cmsysini.c`'s `FirmwareBootDevice` registry export** (`7f433ed74de`): a
  feature addition, not checked for parity (likely just missing functionality, not a bug).
- **`dll/ntdll/rtl/libsupp.c`'s `9d1418fcb6e`** (tick-count-to-ms helper): not reviewed.

## Summary

- **Files reached at commit level**: all 39 boot-path files with ≥2 upstream commits (85
  boot-path files total; the other 46 have exactly 1 commit and were not opened), plus the 16
  highest-churn non-boot-path files (of 166 non-boot-path files in the 182-file scope). Roughly
  **55 of 195** files opened and diffed at commit level this pass.
- **Verdicts**: **19 CONFIRMED** bug fixes present in live Rust (findings #1-#19 above; #12 and
  #15 confirmed but low real-world severity as noted), **10 REFUTED** (Rust already matches or
  exceeds the upstream fix, or the fix doesn't apply to our design/target platforms), **1 N/A**
  (bug lives in a different, not-yet-opened file), **4 INCONCLUSIVE** (identified as plausible
  bug fixes, not cross-checked against Rust — see "Not reached").
- **Highest-value fixes for the boot path**: #6 (object-lock leak, every handle creation), #10
  and #11 (two independent registry-locking bugs on the `CmpDoOpen`/`CmpLookInCache` path, hit
  on essentially every boot-time registry lookup — #11 in particular was misjudged REFUTED on a
  first pass and only settled by reading the code), #1/#2 (GPT boot-disk / drive-letter
  assignment, relevant once GPT/multi-disk QEMU configs are exercised), #7 (SACL
  privilege-check gap), #13 (silent system-PTE leak on every driver unload), #14 (amd64 KD-init
  ordering, blocks PCI-based KD transports).
- **Highest-value non-boot finding**: #16, a DHCP-client stack-overflow reachable from
  attacker-controlled network data (any DHCP server or LAN spoofer), still present verbatim in
  the only implementation we ship.
