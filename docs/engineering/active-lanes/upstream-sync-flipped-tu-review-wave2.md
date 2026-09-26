# Upstream-sync: flipped-TU review, wave 2

Continuation of `upstream-sync-flipped-tu-review.md` (wave 1: 19 CONFIRMED, ~55/195 files
reached). This pass worked the "Not reached (resume here)" list in the priority order given:
(1) the `KiWriteSystemTime`/`ExpSetTimeZoneInformation` companion bug, (2) the four INCONCLUSIVE
leads, (3) a scan of the 43 single-commit boot-path files (`git diff --name-only $FP
upstream/master` ∩ HIGH registry entries ∩ boot-path prefixes ∩ exactly-1-commit), triaged by
commit subject then verified in full for the ones that read as real bug fixes rather than
refactor/cosmetic/feature-add. `FP=5213cf717cff6946945922e6a7e03d87b0a58e4f`.

## Coverage this pass

- Priority 1 (companion bug): reached, CONFIRMED (2 call sites).
- Priority 2 (4 INCONCLUSIVE leads): all 4 resolved — 3 CONFIRMED, 1 REFUTED.
- Priority 3 (43 single-commit boot-path files): all 43 triaged by commit subject/diffstat
  (never verdicted from the subject alone); ~14 that read as pure refactor/cosmetic
  (whitespace, license header, rename, parameter rename, `swprintf` cleanup) were spot-checked
  against the diff and skipped as REFUTED-by-construction (diff has no behavior change); **8
  that read as real bug fixes or feature completions were opened in full and cross-checked
  against the Rust twin** — 5 CONFIRMED (one with two independent bugs), 1 REFUTED, plus the
  companion bug and the two of the four leads that also live in this bucket (syssetup, PNP).
  ~20 of the 43 remain genuinely un-opened (see "Not reached").
- Priority 4 (non-boot-path remainder, ~150 files): **not reached this pass** — budget was
  spent on priorities 1–3, which were explicitly ranked higher.

**Totals this pass: 8 CONFIRMED findings (10 distinct bug instances across them), 2 REFUTED.**

## CONFIRMED — detail + proposed diffs

### W2-1. `KiWriteSystemTime` (32-bit): field-write order matches the bug the upstream commit named, not its fix

- **Upstream fix**: `77b88c48a4b` ("[NDK][NTOS][NTDLL][KRNEL32] Fix read/write of
  KSYSTEM_TIME") — moves `KiWriteSystemTime` to NDK and reorders the 32-bit write path from
  `High1Time, LowPart, High2Time` to `High2Time, LowPart, High1Time` (write the "in progress"
  marker first, the trusted high word last, so a concurrent `KiReadSystemTime`-style loop can
  never observe a torn pair without noticing the mismatch — notably including the case where
  the high word doesn't change, which the old order didn't protect at all).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ke/time.rs:51-63`,
  `KiWriteSystemTime` — the `target_pointer_width = "32"` arm writes, in order,
  `High1Time`, `LowPart`, `High2Time` (the code comment even says "matching C code", i.e. the
  pre-fix C). This is the single writer used by `KeUpdateSystemTimeImpl` for
  `InterruptTime`, `SystemTime`, `KeTickCount`, and the shared `TickCount` — i.e. every clock
  tick on i386, live.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntoskrnl-rust/src/ke/time.rs
+++ b/sdk/lib/rust/ntoskrnl-rust/src/ke/time.rs
@@ unsafe fn KiWriteSystemTime
     #[cfg(target_pointer_width = "32")]
     {
-        // Write order must be: High1Time, LowPart, High2Time (matching C code)
-        // This allows readers to detect torn reads by comparing High1Time == High2Time
-        core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).High1Time), NewTime.__bindgen_anon_1.HighPart as i32);
-        core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).LowPart), NewTime.__bindgen_anon_1.LowPart);
+        // Write order must be High2Time, LowPart, High1Time: mark "update in
+        // progress" first, write the trusted high word last (upstream 77b88c48a4b).
+        core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).High2Time), NewTime.__bindgen_anon_1.HighPart as i32);
+        core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).LowPart), NewTime.__bindgen_anon_1.LowPart);
+        core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).High1Time), NewTime.__bindgen_anon_1.HighPart as i32);
+        return;
     }
-    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SystemTime).High2Time), NewTime.__bindgen_anon_1.HighPart as i32);
```
  (the trailing `High2Time` write must be removed from the shared tail — the 32-bit arm now
  does all three writes itself in the corrected order; the 64-bit arm keeps its existing single
  atomic write.)

### W2-2. `ExpSetTimeZoneInformation`: TimeZoneBias write order is the exact bug upstream named

- **Upstream fix**: same commit `77b88c48a4b` — the commit message says explicitly
  "`ExpSetTimeZoneInformation` had it wrong for the TimeZoneBias"; pre-fix order was
  `High1Time, High2Time, LowPart` (the worst of the three orders in the file — the trusted
  word `High1Time` is written *first*), replaced by `KiWriteSystemTime(&...TimeZoneBias, ...)`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ex-rust/src/time.rs`, `ExpSetTimeZoneInformation`
  — writes `TimeZoneBias.High1Time`, then `.High2Time`, then `.LowPart`, in that exact
  pre-fix order. Also note: **none of these three fields are written with
  `write_volatile`** (plain field assignments), unlike the equivalent code in
  `ke/clock.rs::KeSetSystemTime` which explicitly comments that plain stores to disjoint
  fields are reorderable by LLVM and uses `write_volatile` for that reason — so even fixing
  the order here without adding volatility leaves the store order unenforced against the
  optimizer.
- `ExRefreshTimeZoneInformation` in the same file (`TimeZoneBias` and `SystemTime` writes)
  already uses the **correct** `High2Time, LowPart, High1Time` order (matches the pre-fix C,
  which the commit message says was already right for these two) — REFUTED as a live bug, but
  has the same missing-`write_volatile` issue.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ex-rust/src/time.rs
+++ b/sdk/lib/rust/ex-rust/src/time.rs
@@ ExpSetTimeZoneInformation
-    (*SharedUserData).TimeZoneBias.High1Time = ExpTimeZoneBias.u.HighPart;
-    (*SharedUserData).TimeZoneBias.High2Time = ExpTimeZoneBias.u.HighPart;
-    (*SharedUserData).TimeZoneBias.LowPart = ExpTimeZoneBias.u.LowPart;
+    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).TimeZoneBias.High2Time), ExpTimeZoneBias.u.HighPart);
+    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).TimeZoneBias.LowPart), ExpTimeZoneBias.u.LowPart);
+    core::ptr::write_volatile(core::ptr::addr_of_mut!((*SharedUserData).TimeZoneBias.High1Time), ExpTimeZoneBias.u.HighPart);
```
  (also worth doing the same volatile-ization for `ExRefreshTimeZoneInformation`'s two writes,
  same rationale, not order-broken but currently unprotected against reordering.)

### W2-3. `SHGetSetSettings`: writes a partially-uninitialized `SHELLSTATE` to the registry

- **Upstream fix**: `4fecc908e7d` (INCONCLUSIVE lead #2) — adds `SSF_ALL`/`SSF_IMPLEMENTED`
  masks and changes the pre-read condition from `dwMask & ~g_CachedSSF` to
  `notcached & ~dwMask` where `notcached = ~g_CachedSSF & SSF_IMPLEMENTED`, so that on a
  `bSet` (write) call, *every* SHELLSTATE member not already cached and not itself the one
  being set gets read from the registry first — because the whole struct is about to be
  written back.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/shell32-rust/src/wine/shellord.rs:1149-1210`,
  `SHGetSetSettings` — line 1208 is `if (dwMask & !g_CachedSSF) != 0 { SHGetSetSettings(&mut
  tempstate, dwMask, FALSE); }`, byte-for-byte the pre-fix condition (uses `dwMask` as the
  read-mask, not `notcached & ~dwMask`; no `SSF_ALL`/`SSF_IMPLEMENTED` defined anywhere in the
  file). Any single-flag `SHGetSetSettings(..., SSF_SOMEFLAG, TRUE)` call whose flag is already
  in `g_CachedSSF` skips the backfill entirely and writes whatever garbage/zeroed fields
  `tempstate`/`gpss` holds for every other member straight to the registry.
- Proposed fix: port upstream's masks and condition:
```diff
--- a/sdk/lib/rust/shell32-rust/src/wine/shellord.rs
+++ b/sdk/lib/rust/shell32-rust/src/wine/shellord.rs
@@
+const SSF_ALL: DWORD = 0x07FFFFFF & !0x40;
+const SSF_IMPLEMENTED: DWORD = SSF_ALL & !SSF_SERVERADMINUI; // written by Explorer, read by IsOS()
@@ pub unsafe extern "system" fn SHGetSetSettings
     if bSet != 0 {
         let mut changed: DWORD = 0;
-        if (dwMask & !g_CachedSSF) != 0 {
+        let notcached = !g_CachedSSF & SSF_IMPLEMENTED;
+        if (notcached & !dwMask) != 0 {
             let mut tempstate: SHELLSTATE = core::mem::zeroed();
-            SHGetSetSettings(&mut tempstate, dwMask, FALSE); // read entries not in g_CachedSSF
+            SHGetSetSettings(&mut tempstate, notcached & !dwMask, FALSE);
         }
```
  (verify `SSF_SERVERADMINUI`'s constant value is already defined in this crate; it's an
  existing `SSF_*` flag so should already be in scope.)

### W2-4. `PNP_FreeLogConf`: deleting one alternative from a multi-alternative resource requirements list is a silent no-op

- **Upstream fix**: `46e69e6e507` (INCONCLUSIVE lead #3) — implements the
  `REG_RESOURCE_REQUIREMENTS_LIST` / `AlternativeLists > 1` branch, which was a `/* FIXME */`
  stub: bounds-checks `ulLogConfTag`, walks to the target alternative with
  `NextResourceRequirement`, either truncates (if it's the last alternative) or
  `MoveMemory`s the remainder down (if not), decrements `AlternativeLists`, updates
  `ListSize`, and writes the shrunk buffer back with `RegSetValueEx`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/umpnpmgr-rust/src/umpnpmgr/rpcserver.rs:4476-4480`
  — the `else` arm (multi-alternative case) is a bare empty block: `} else {\n}`. **This is
  worse than upstream's pre-fix FIXME**, which was at least a visible marker: our version
  returns `ret = CR_SUCCESS` (set earlier at function entry and never changed on this path) to
  the RPC caller while doing nothing, so `PNP_FreeLogConf` on a multi-alternative requirements
  list reports success without deleting anything.
- Proposed fix: port the block wholesale — same shape as upstream's diff, against the already-
  present `PIO_RESOURCE_REQUIREMENTS_LIST`/`NextResourceRequirement`-equivalent helper this
  crate already has for the sibling `REG_RESOURCE_LIST` branch just above it (see
  `NextResourceDescriptor` used there at line ~4441 for the pattern to mirror with whatever the
  crate's `NextResourceRequirement`-equivalent is named — grep for it before writing the body,
  it may not exist yet and need porting too).

### W2-5. `InstallReactOS`/`InstallLiveCD`: every setup error path skips the reboot

- **Upstream fix**: `d55a77c350e` (INCONCLUSIVE lead #4) — `InstallReactOS`'s early failure
  checks (`InitializeProgramFilesDir`, `InitializeProfiles`, `SaveDefaultUserHive`,
  `CopySystemProfile`, `CommonInstall`) change from `return 0;` to `goto Quit;`, where `Quit:`
  now unconditionally enables `SE_SHUTDOWN_NAME` and calls `ExitWindowsEx(EWX_REBOOT, 0)` (a
  `pSetupEnablePrivilege` failure is now non-fatal, just logged). `InstallLiveCD`'s `error:`
  label gets the same reboot call added after its `MessageBoxW`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/syssetup-rust/src/syssetup/install.rs` —
  `InstallReactOS` (line 1947) `return 0;`s directly on every one of those five failure checks
  (lines ~1960, 1966, 2001, 2005, 2018) with no reboot; `CommonInstall`'s failure additionally
  skips even the privilege/reboot code at the tail since it's a plain early return past it.
  `InstallLiveCD` (line 1467) similarly just shows the `MessageBoxW` and returns 0 on its error
  path, no `ExitWindowsEx`. A failed unattended/automated install (the entire point of these
  functions during OS setup) currently leaves the machine sitting in a dead process instead of
  rebooting into a recoverable state.
- Proposed fix: restructure `InstallReactOS` around a single tail block (Rust doesn't have
  `goto`, but the existing tail code from `if !AdminInfo.Name.is_null() { ... }` through
  `ExitWindowsEx(...)` is already a self-contained cleanup+reboot sequence — factor it into a
  local closure/inner fn and call it from every one of the five early-failure sites instead of
  `return 0`, dropping the `FatalError`-then-bail-without-reboot shape). For `InstallLiveCD`,
  add the `pSetupEnablePrivilege(SE_SHUTDOWN_NAME, TRUE)` + `ExitWindowsEx(EWX_REBOOT, 0)` pair
  right after the existing `MessageBoxW` in the error path.

### W2-6. `FsRtlCheckLockForReadAccess` / `FsRtlFastUnlockAll`: byte-range lock ownership not checked

- **Upstream fix**: `c606fd79b1c` ("Respect byte-range lock ownership", #9342) — two
  independent fixes in the same file: (1) `FsRtlCheckLockForReadAccess` grants read access
  through an exclusive lock when only the IRP key matches, without checking the requestor's
  process against the lock's owning process; fix adds `Process ==
  Found->Exclusive.FileLock.ProcessId` to the OR-condition. (2) `FsRtlFastUnlockAll` walks the
  exclusive-lock generic table and unlocks every entry regardless of which process owns it;
  fix adds `if (Entry->Exclusive.FileLock.ProcessId != Process) continue;` before acting on
  each entry (matching the ownership check already present in the shared-lock loop and in
  `FsRtlFastUnlockAllByKey`).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/fsrtl-rust/src/filelock.rs`:
  - `FsRtlCheckLockForReadAccess` (line 576-601): the `result` computation at lines 594-599
    is `(*Found).Exclusive.FileLock.ExclusiveLock == 0 || (*IoStack).Parameters.Read.Key ==
    (*Found).Exclusive.FileLock.Key` — no process check at all, matching pre-fix exactly. A
    process that merely guesses/reuses another process's lock key gets read access through an
    exclusive lock it doesn't own.
  - `FsRtlFastUnlockAll` (line 820-874): the exclusive-lock loop (from
    `RtlEnumerateGenericTable`, lines ~854-872) unlocks every entry unconditionally — no
    `ProcessId != Process` skip, matching pre-fix exactly. (The **shared**-lock loop just
    above it, lines ~836-853, already has the correct `if (*Range).ProcessId != Process {
    continue; }` check — only the exclusive-lock loop is missing it, exactly mirroring
    upstream's diff scope.) Any process calling `FsRtlFastUnlockAll` on a file object can force-
    unlock another process's exclusive byte-range locks.
- Proposed diff:
```diff
--- a/sdk/lib/rust/fsrtl-rust/src/filelock.rs
+++ b/sdk/lib/rust/fsrtl-rust/src/filelock.rs
@@ FsRtlCheckLockForReadAccess
+    let Process = IoGetRequestorProcess(Irp); // add PEPROCESS import if not already in scope
     let result = if (*Found).Exclusive.FileLock.ExclusiveLock == 0
-        || (*IoStack).Parameters.Read.Key == (*Found).Exclusive.FileLock.Key
+        || ((*IoStack).Parameters.Read.Key == (*Found).Exclusive.FileLock.Key
+            && Process == (*Found).Exclusive.FileLock.ProcessId)
     {
@@ FsRtlFastUnlockAll (exclusive-lock loop)
     while !Entry.is_null() {
+        if (*Entry).Exclusive.FileLock.ProcessId != Process as PVOID {
+            Entry = RtlEnumerateGenericTable(core::ptr::addr_of_mut!((*InternalInfo).RangeTable), 0u8) as PCOMBINED_LOCK_ELEMENT;
+            continue;
+        }
         let mut Length: LARGE_INTEGER = core::mem::zeroed();
```
  (verify `IoGetRequestorProcess` is already imported/available in this crate — it's used
  elsewhere in `filelock.rs`'s sibling `FsRtlCheckLockForWriteAccess`, which already does this
  correctly via `(*(*Irp).Tail.Overlay.Thread).Tcb.Process`; reuse that exact pattern instead
  of introducing a new helper call, for consistency with the file's existing style.)

### W2-7. `ExpReleaseOrWaitForKeyedEvent`: waits in `KernelMode`, blocking thread termination

- **Upstream fix**: `d1c281c95e0` ("Fix wait mode in keyed event implementation") — the
  `KeWaitForSingleObject` call on `CurrentThread->KeyedWaitSemaphore` used `KernelMode`
  unconditionally; fix captures `PreviousMode = KeGetPreviousMode()` and passes that instead,
  "to allow user mode threads to be terminated" (a `KernelMode` wait cannot be interrupted for
  APC-based termination the way a `UserMode` wait can).
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ex-rust/src/keyedevt.rs:223-228`,
  `ExpReleaseOrWaitForKeyedEvent` — `KeWaitForSingleObject(..., 7u32 /* WrKeyedEvent */,
  0i8 /* KernelMode */, Alertable, Timeout)`, hardcoded `0i8` for the wait mode. This is on the
  live keyed-event wait path (used by ntdll critical sections / `RtlAcquireSRWLockExclusive`
  contention under low-memory fallback and directly by `NtWaitForKeyedEvent`) — a user-mode
  thread blocked here cannot be terminated by `NtTerminateThread` until the keyed event is
  signaled or the timeout elapses, unlike real Windows/upstream-fixed behavior.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ex-rust/src/keyedevt.rs
+++ b/sdk/lib/rust/ex-rust/src/keyedevt.rs
@@ ExpReleaseOrWaitForKeyedEvent
+    let PreviousMode = KeGetPreviousMode();
     let mut Status = KeWaitForSingleObject(
         core::ptr::addr_of_mut!((*CurrentThread).__bindgen_anon_4.KeyedWaitSemaphore) as PVOID,
         7u32,
-        0i8,
+        PreviousMode,
         Alertable,
         Timeout,
     );
```
  (`KeGetPreviousMode` is already declared/used elsewhere in this same file at line 78/282, so
  this is a same-file, zero-new-import change.)

### W2-8. `ExpBoostOwnerThread`: locks the waiter's thread, not the resource owner's, while mutating the owner

- **Upstream fix**: `e6d2efe90708` ("Lock the resource owner in ExpBoostOwnerThread") — "The
  waiter's lock does not protect the owner thread's dispatcher state." Both
  `KiAcquireThreadLock(Thread)`/`KiReleaseThreadLock(Thread)` calls change to
  `KiAcquireThreadLock(OwnerThread)`/`KiReleaseThreadLock(OwnerThread)`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ex-rust/src/resource.rs:476-491`,
  `ExpBoostOwnerThread` — `KiAcquireThreadLock(Thread)` at line 479 and
  `KiReleaseThreadLock(Thread)` at line 491 bracket writes to `(*OwnerThread).PriorityDecrement`,
  a quantum-reset helper call, and `KiSetPriorityThread(OwnerThread, 14)` — i.e. the code
  mutates `OwnerThread`'s dispatcher state (priority boost, quantum, priority decrement) while
  holding `Thread`'s (the waiter's) lock, not `OwnerThread`'s. This is the ERESOURCE
  priority-inversion boost path, live on every contended resource acquisition where the owner
  has lower priority than a waiter — an unsynchronized write race against anything else that
  concurrently touches `OwnerThread`'s dispatcher fields under its own correct lock (e.g. a
  concurrent priority change, or `OwnerThread` finishing and being boosted/reset elsewhere).
- Proposed diff:
```diff
--- a/sdk/lib/rust/ex-rust/src/resource.rs
+++ b/sdk/lib/rust/ex-rust/src/resource.rs
@@ ExpBoostOwnerThread
-            KiAcquireThreadLock(Thread);
+            KiAcquireThreadLock(OwnerThread);
@@
-            KiReleaseThreadLock(Thread);
+            KiReleaseThreadLock(OwnerThread);
```

### W2-9. `KiInitializeUserApc` (amd64): EFLAGS "sanitize" mask leaks stale TF/DF into new threads

- **Upstream fix**: `6bc61687e6ed` ("Fix KiInitializeUserApc" — "Set EFlags instead of
  'fixing' it. Prevents crashes ... when the thread has set the direction flag or trap flag.")
  — replaces `TrapFrame->EFlags &= EFLAGS_USER_SANITIZE; TrapFrame->EFlags |=
  EFLAGS_INTERRUPT_MASK;` with a flat `TrapFrame->EFlags = EFLAGS_INTERRUPT_MASK;`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/usercall.rs:199-201`
  — still does `(*TrapFrame).EFlags &= EFLAGS_USER_SANITIZE; (*TrapFrame).EFlags |=
  EFLAGS_INTERRUPT_MASK;` with `EFLAGS_USER_SANITIZE: u32 = 0x003F4DD7` (line 79). Checked the
  mask bit-by-bit: bit 8 (TF, trap flag, `0x100`) and bit 10 (DF, direction flag, `0x400`) are
  both **set** in `0x003F4DD7` (the low 16 bits are `0x4DD7`; bits 8–11 are the nibble `D` =
  `1101`, so bits 8, 10, 11 all pass through) — i.e. this "sanitize" mask does **not** clear
  TF or DF, it preserves whatever was already in `TrapFrame->EFlags` at those bit positions.
  `KiInitializeUserApc` runs once per new user thread to seed its initial trap frame; if the
  freshly-allocated kernel stack's `EFlags` slot happens to contain nonzero garbage at bits
  8/10 (uninitialized/reused stack memory, not attacker-controlled but not zeroed either), the
  new thread starts execution with the trap flag or direction flag already set — TF causes an
  immediate single-step `#DB` on the thread's first user instruction, matching exactly the
  "crashes in the thread initialization APC" upstream describes.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/usercall.rs
+++ b/sdk/lib/rust/ntoskrnl-rust/src/ke/amd64/usercall.rs
@@ KiInitializeUserApc
-    (*TrapFrame).EFlags &= EFLAGS_USER_SANITIZE;
-    (*TrapFrame).EFlags |= EFLAGS_INTERRUPT_MASK;
+    (*TrapFrame).EFlags = EFLAGS_INTERRUPT_MASK;
```
  (i386's `usercall.rs` uses an unrelated `Ke386SanitizeFlags` mechanism — not touched by this
  upstream commit, not in scope here; not re-checked for an analogous bug this pass.)

### W2-10. `NtFreeVirtualMemory`: `NoChange` VADs (PEB/TEB/shared user page) can be freed

- **Upstream fix**: `fca332046ed4` ("Handle NoChange Vads in NtFreeVirtualMemory" — "Fixes
  assertion failure ... This also prevents freeing the PEB and TEB.") — replaces
  `ASSERT(Vad->u.VadFlags.NoChange == 0);` with a real runtime check: `if
  (Vad->u.VadFlags.NoChange != 0) { Status = STATUS_INVALID_PAGE_PROTECTION; goto FailPath; }`.
- **Our Rust (pre-fix, live)**: `sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs:2945` — the
  ASSERT was translated to a bare comment `// ASSERT(NoChange == 0)` with no code at all (which
  is the *correct* translation of a debug-only ASSERT for release-build parity — the bug is
  not the translation, it's that **upstream's pre-fix C had no protection here either**, and
  our Rust inherited that same absence without also getting upstream's follow-up real check).
  Net effect: `NtFreeVirtualMemory` on this crate's live implementation currently has **no**
  guard against a caller targeting a `NoChange` VAD — i.e. a process can free its own PEB, TEB,
  or the shared user data page mapping via `NtFreeVirtualMemory`, proceeding straight into the
  MEM_RELEASE/MEM_DECOMMIT logic below (visible starting line 2946) instead of being rejected.
- Proposed diff:
```diff
--- a/sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs
+++ b/sdk/lib/rust/ntoskrnl-rust/src/ARM3/virtual.rs
@@
-    // ASSERT(NoChange == 0)
+    if (*Vad).u.VadFlags.NoChange() != 0 {
+        // DPRINT1: Attempt to free protected VAD
+        fail_path!(STATUS_INVALID_PAGE_PROTECTION);
+    }
```
  (use the same `fail_path!` macro already used a few lines above at 2930/2934/2935 for
  consistency with this function's existing early-exit style; confirm it releases whatever
  locks/references are held at this point in the control flow the same way the other
  `fail_path!` call sites do before landing this.)

## REFUTED — detail

- **`ExRefreshTimeZoneInformation`** (`ex-rust/src/time.rs`): TimeZoneBias and SystemTime
  writes already use the correct `High2Time, LowPart, High1Time` order (matches what upstream
  says was "already right" pre-fix for these two) — not a live bug, though see W2-2's note
  about missing `write_volatile`.
- **`SHBindToObject` ABI/signature** (`a790beb22ba`, INCONCLUSIVE lead #1): upstream widens
  `SHBindToObject` to 5 params + `SHSTDAPI`(stdcall) and removes the `SHBindToObjectEx`
  wrapper. Our Rust (`shell32-rust/src/shell32/utils.rs:664-706`) still has both functions in
  the pre-fix shape (4-param cdecl `SHBindToObject` wrapping `SHBindToObjectEx`). Checked every
  caller across the tree (`grep -rl "SHBindToObject("` — `wine/shellpath.rs`, `island_{amd64,
  i386,arm64}.rs`, `win32.rs`, `shldataobject.rs`, `changenotify.rs`,
  `CRecycleBinItemContextMenu.rs`, `CRegFolder.rs`, `desktop_folder.rs`, `brfolder.rs`): all
  are internal to shell32 and all declare/call the matching 4-param signature — `SHBindToObject`
  is not in `shell32.spec` (not a public export), so there is no live ABI mismatch reachable
  today. Genuinely behind upstream's API shape, but not a bug with current callers.

## Not reached

- **~20 of the 43 single-commit boot-path files**: triaged by subject/diffstat only (read as
  clear refactor/feature-add/cosmetic — e.g. `[NTOS:OB] Return handle attributes by
  ObReferenceFileObjectForWrite` (feature), `[NTOS:PS] Basic support for jobs...` (feature, huge
  diff, own subsystem), `[NDK][NTOS] Change Object callback types to NT6 style` (signature
  refactor, hits 3 files: `cmse.c`/`obsecure.c`/`semgr.c`), `[NTOS:ARM3] Change calculation for
  more system PTEs` (3 files, tuning not correctness), `[NTOS] Improve build generation`,
  `[HALX86][MINIHAL] Further code cleanup`) — not opened in full, not cross-checked against
  Rust. Full list is reproducible via the derivation command at the top of this doc combined
  with the wave-1 doc's boot-path prefix filter; the ones NOT covered by W2-1 through W2-10 or
  wave-1's findings are the remainder.
- **`ntoskrnl/config/cmdata.c` / `controller.c`** (`9174dcbd615`, "Implement Win7+ I/O System
  case-insensitivity support") — feature addition, not checked for parity; worth a look since
  case-sensitivity mismatches are the kind of thing that silently breaks path lookups rather
  than crashing.
- **`ntoskrnl/ke/i386/kiinit.c` / `thrdini.c` / `thrdobj.c`** (`55090b348062`, "Implement
  support for CycleTime accounting") — 3-file feature addition, not checked; CycleTime is used
  by scheduler diagnostics, not obviously boot-blocking, but not verified.
- **~150 non-boot-path files** (priority 4): not reached at all this pass. Wave 1 already
  covered the 16 highest-churn ones; the remainder (mostly `netcfgx`/`dhcpcsvc` incremental
  "Implement X"/"Improve Y" commits per wave 1's note, plus whatever else is in the 182-file
  scope minus the 85 boot-path files) is fully untouched by either pass.
- **`ntoskrnl/mm/ARM3/sysldr.c`'s `MmLoadSystemImage` error-cleanup path** — flagged by wave 1
  as elevated-risk and not fully verified; still not verified.
- **`ke/amd64/cpu.c`/`ke/i386/cpu.c` shared-vendor refactor** (`d3109a8ec95`/`5498220a5e9`) —
  flagged by wave 1, still not diffed in full.
- **`ntoskrnl/config/cmsysini.c`'s `FirmwareBootDevice` registry export`** (`7f433ed74de`) and
  **`dll/ntdll/rtl/libsupp.c`'s `9d1418fcb6e`** (tick-count-to-ms helper) — both flagged by
  wave 1 as not reviewed, still not reviewed.

## Summary

- **8 CONFIRMED findings, 10 distinct bug instances** (W2-3 and W2-6 each bundle two related
  fixes from one commit): W2-1/W2-2 (the KSYSTEM_TIME companion bug, i386-only, live on every
  clock tick and every timezone change), W2-3 (SHELLSTATE partial-init registry write),
  W2-4 (PNP_FreeLogConf silent no-op, reports false success), W2-5 (setup never reboots on
  error), W2-6 (byte-range file-lock ownership bypass — read-through-exclusive-lock and
  cross-process force-unlock), W2-7 (keyed-event wait blocks thread termination), W2-8
  (ERESOURCE priority-boost race, wrong thread locked), W2-9 (amd64 new-thread EFLAGS leak,
  can crash on TF/DF stack garbage), W2-10 (NtFreeVirtualMemory can free PEB/TEB/shared page).
- **2 REFUTED**: `ExRefreshTimeZoneInformation` (already correct order), `SHBindToObject`
  ABI drift (real but not live — no caller hits the mismatch).
- **Highest-value findings**: W2-1/W2-2 (boot-path, every tick, i386), W2-9 (boot-path, every
  new amd64 user thread, direct crash mechanism identified), W2-10 (security-relevant, no
  test/apitest needed to trigger — any usermode caller), W2-6 (security-relevant, LAN/multi-
  process file-locking bypass, similar class to wave 1's DHCP finding).
