# Upstream-sync bug oracle: the 97-commit tail

**Scope.** This review covers the 49 `(upstream commit, C file)` pairs in
`~/vibeos-rust-staging/upstream-oracle-merged/tail97-pairs.txt`, drawn from the 97 commits in
`28a26f3b664^2..upstream/master` that `92e60381ae4` merged. For each pair I read the upstream
diff, checked whether the C is still built, and compared the live Rust twin against the
**post-fix** C. The last six pairs are files we deleted (the C HAL's `apicsmp.c`, `smp.c`,
`mps/mps.c`, and `i386/spinup.c`, plus the old `tcpip.sys`'s `udp.c`). I read those with
`git show <commit> -- <path>`. This was a read-only review: no code edits, no builds, no boots,
no rustconv.

How I checked liveness: `hal/halx86/CMakeLists.txt` builds hal.dll and halmp.dll from cargo only
(`hal-rust`), and the C component lists in `acpi.cmake` and `legacy.cmake` are commented out.
`ntoskrnl/ntos.cmake` comments out the `config/*.c` sources. The flipped DLLs and drivers
(kernel32, syssetup, mprapi, rasapi32, msafd, afd, serial, shell32, explorer) each list their
`.c`/`.cpp` as "replaced by Rust" in their CMakeLists. `sdk/lib/rtl/waitaddress.c` is not
referenced by any CMake file. `drivers/network/tcpip/` no longer exists; `drivers/network/netstack`
(netstack-rust, a smoltcp rewrite) replaced it.

Some pairs were already handled before this review. Commits `4490bf2afb3` (396019984c9),
`60ab98a547d` (938c14e3276 and 445d0881b82) and `67297f438ab` (091855fc4f9) ported those fixes.
I re-checked each port against the upstream hunk below.

## Status (2026-09-30)

**Ported:** 1-7, each checked against the upstream source first.

The MADT rewrite (1) follows upstream's `madt.c` function for function, with one
divergence: the boot APIC id comes from CPUID(1), the value `KiGetFeatureBits` stores in
`InitialApicId`, rather than through the HAL's `_KPRCB` view, which has disagreed with the
kernel's on offsets before. Smoked `--smp 4` on amd64 and i386, with all three APs online
on each.

**UDP broadcast (b8ee7950af4), settled 2026-10-03:** the two-socket test, added to
`tdidgtest.rs` and asserted by `test_net.py::test_tdi_broadcast_local_delivery`, showed the
gap was real: both limited and directed broadcasts were MISSED by the local listener. The fix
went into `tditrans.rs`. A successful SendDatagram to a broadcast address is queued to every
local address object bound to the destination port, and both cases now pass.

**Still owed:**
- The pre-existing SMP divergences listed under "HAL SMP: the rest of the AP-startup
  comparison", for the SMP lane.

## Summary

| Verdict | Pairs |
|---|---|
| CONFIRMED | 7 |
| ALREADY-FIXED | 7 |
| NOT-A-FIX | 17 |
| FEATURE-NOT-ADOPTED | 15 |
| LIVE-C | 2 |
| NO-RUST-TWIN | 1 |
| INCONCLUSIVE | 0 (one NO-RUST-TWIN has an open behaviour question, listed below) |

CONFIRMED findings, ranked by impact: MADT walk (SMP/HAL, boot-path), syssetup COM init,
kernel32 default timer queue race, `KeIsWaitListEmpty`, `MprConfigServerConnect`, msafd
`SO_CONNECT_TIME`, and the 82489DX SIPI guard.

## CONFIRMED — detail + proposed diffs

### 1. hal-rust MADT walk aborts on any unhandled subtable, counts absent CPUs, and never places the BSP first (cae3c053d47, `hal/halx86/acpi/madt.c`)

**Rust:** `sdk/lib/rust/hal-rust/src/x86/madt.rs:159-249`, `HalpParseApicTables`. This is a
faithful translation of the **pre-fix** C: the `default: return;`, the `Bus != 0` → `return`, the
duplicate-IOAPIC `return`, the `ONLINE_CAPABLE | ENABLED` test, and no BSP handling are all in the
old `madt.c`.

**Upstream:** the walk was rewritten as `HalpMadtParseEntries` with three helpers,
`HalpMadtAddProcessor`, `HalpMadtAddIoApic` and `HalpMadtAddInterruptOverride`, plus a new
`HalpMadtPlaceBootProcessor`. Here is where ours diverges:

| # | Rust (line) | Post-fix C | Consequence in ours |
|---|---|---|---|
| a | `:236-238` any other type → `return` | `default: break;` (skip) | The walk stops at the first LAPIC-NMI (4), x2APIC (9), x2APIC-NMI (0xA), or OEM entry. Every LAPIC entry after that point is lost, so the box runs with fewer CPUs. Firmware that interleaves NMI or x2APIC entries with LAPIC entries silently loses processors. |
| b | `:198` accepts `ONLINE_CAPABLE \| ENABLED` | only `ACPI_MADT_ENABLED` | A hot-plug slot that is online-capable but not present gets counted in `ProcessorCount`. `HalStartNextProcessor` (`smp_processor.rs:579`) INIT/SIPIs an absent LAPIC. `KeStartAllProcessors` (`ke/amd64/mproc.rs:338-352`) spins 10^9 PAUSEs, prints "NEVER SIGNALED", and `break`s, so every real CPU listed after the phantom is never started. |
| c | none | `HalpMadtPlaceBootProcessor`: moves the entry whose `LapicId == InitialApicId` to index 0 and sets `BSPCheck`. If the BSP is missing, it keeps only the BSP. | `NEXT_PROCESSOR` starts at 1 and assumes entry 0 is the BSP (`smp_processor.rs:397`). If the MADT does not list the BSP first, one AP is never started and **the BSP INITs itself** when its index comes up. On real hardware that should put the BSP into wait-for-SIPI and hang the boot. |
| d | none | `ApicId >= 0xFF` → ignored. Duplicate LAPIC IDs (LAPIC plus x2APIC entry for the same CPU) are deduplicated. | 0xFF is the broadcast ID, so an INIT "to" it would hit every CPU. Duplicate entries double-count a CPU, which is case (b) again. |
| e | `:220-221` duplicate IOAPIC id → `return` | keep the first, skip the rest. `Address == 0` → skip. | Dell PowerEdge-style duplicates truncate the walk, which is case (a) again. |
| f | `:232-235` `Bus != 0` → `return`; never fills the redirect | skip non-ISA. `HalpPicVectorRedirect[SourceIrq] = GlobalIrq` | `halacpi.rs:1368` resolves the SCI through `HalpPicVectorRedirect`, which stays identity. A firmware that remaps the SCI's ISA IRQ gets the wrong SCI vector. |
| g | `:166-168`, `:162` no MADT or a short MADT → `ProcessorCount = 0` | the boot processor is still recorded (count 1) | Harmless today: every consumer of `ProcessorCount` treats 0 like 1. |
| h | `:212-215`, `:245-248` | no "landed exactly on TableEnd" requirement; a truncated tail is only recorded | Cosmetic. The early `return` there is the last statement anyway. |

**Why our SMP smoke never sees this:** QEMU's `build_madt` lists the BSP first, then the LAPICs,
one IOAPIC, ISA-bus ISOs, and the LAPIC-NMI entry **last**. On that layout the abort in (a)
happens after everything we use has been parsed. On real x86 firmware none of that ordering is
guaranteed.

**Not adopted, deliberately:** upstream widened `PROCESSOR_IDENTITY.{ProcessorId,LapicId}` from
`UCHAR` to `ULONG` and made `HalpProcessorIdentity` a real array instead of a pointer.
`HalpProcessorIdentity` has no C consumer left in our tree (the only references are dead C under
`hal/halx86/`), and IDs of 0xFF and above are filtered by (d). The `UCHAR` layout and pointer
export in `madt.rs:28-36,148-151` are therefore self-consistent and need no change.

**Proposed diff** (against `madt.rs`; not applied):

```diff
@@ use ntkrnl_sys::hal_ddk::{
-    AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE, AcpiMadtType_ACPI_MADT_TYPE_IO_APIC,
-    AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC, HalAcpiGetTable, ACPI_MADT_INTERRUPT_OVERRIDE,
-    ACPI_MADT_IO_APIC, ACPI_MADT_LOCAL_APIC, ACPI_SUBTABLE_HEADER, ACPI_TABLE_MADT, APIC_SIGNATURE,
+    AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE, AcpiMadtType_ACPI_MADT_TYPE_IO_APIC,
+    AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC, AcpiMadtType_ACPI_MADT_TYPE_LOCAL_X2APIC,
+    HalAcpiGetTable, ACPI_MADT_ENABLED, ACPI_MADT_INTERRUPT_OVERRIDE, ACPI_MADT_IO_APIC,
+    ACPI_MADT_LOCAL_APIC, ACPI_MADT_LOCAL_X2APIC, ACPI_SUBTABLE_HEADER, ACPI_TABLE_MADT, APIC_SIGNATURE,
     MAXIMUM_PROCESSORS, PLOADER_PARAMETER_BLOCK,
 };
+use crate::x86::halacpi::HalpPicVectorRedirect;
+
+// Upstream cae3c053d47: one entry per ENABLED processor, never the broadcast id,
+// never twice (firmware may describe a CPU with both a LAPIC and an x2APIC entry).
+unsafe fn madt_add_processor(apic_id: u32, processor_id: u32, flags: u32) {
+    if flags & ACPI_MADT_ENABLED == 0 { return; }
+    if apic_id >= 0xFF { return; }                       // LapicId is 8 bits; 0xFF = broadcast
+    let n = HalpApicInfoTable.ProcessorCount as usize;
+    let base = (&raw mut HalpStaticProcessorIdentity).cast::<PROCESSOR_IDENTITY>();
+    for i in 0..n { if (*base.add(i)).LapicId as u32 == apic_id { return; } }
+    if n >= MAXIMUM_PROCESSORS as usize { return; }
+    (*base.add(n)).ProcessorId = processor_id as UCHAR;
+    (*base.add(n)).LapicId = apic_id as UCHAR;
+    HalpApicInfoTable.ProcessorCount += 1;
+}
+
+// NT processor numbers follow entry order and HalStartNextProcessor starts at 1, so the
+// boot processor MUST be entry 0 -- otherwise we INIT ourselves. If the MADT omits the
+// BSP it cannot be trusted: run UP.
+unsafe fn madt_place_boot_processor() {
+    #[cfg(target_arch = "x86_64")] use core::arch::x86_64::__cpuid;
+    #[cfg(target_arch = "x86")]    use core::arch::x86::__cpuid;
+    let boot_id = (__cpuid(1).ebx >> 24) as UCHAR;     // what the kernel stores as InitialApicId
+    let base = (&raw mut HalpStaticProcessorIdentity).cast::<PROCESSOR_IDENTITY>();
+    let n = HalpApicInfoTable.ProcessorCount as usize;
+    let mut boot = core::mem::zeroed::<PROCESSOR_IDENTITY>();
+    match (0..n).find(|&i| (*base.add(i)).LapicId == boot_id) {
+        Some(i) => { boot = *base.add(i); core::ptr::copy(base, base.add(1), i); }
+        None => { boot.LapicId = boot_id; HalpApicInfoTable.ProcessorCount = 1; }
+    }
+    boot.BSPCheck = 1;
+    *base = boot;
+}
@@ pub unsafe extern "system" fn HalpParseApicTables(
     let MadtTable = HalAcpiGetTable(LoaderBlock, APIC_SIGNATURE) as *mut ACPI_TABLE_MADT;
-    if MadtTable.is_null() {
-        return;
-    }
+    if MadtTable.is_null() { madt_place_boot_processor(); return; }
     let madt_length = ...;
-    if (madt_length as usize) < core::mem::size_of::<ACPI_TABLE_MADT>() {
-        return;
-    }
+    if (madt_length as usize) < core::mem::size_of::<ACPI_TABLE_MADT>() {
+        madt_place_boot_processor();
+        return;
+    }
@@ while ... {
-        if (header_length as usize) < size_of::<ACPI_SUBTABLE_HEADER>() { return; }
-        if (AcpiHeader as ULONG_PTR + header_length as ULONG_PTR) > TableEnd { return; }
+        if (header_length as usize) < size_of::<ACPI_SUBTABLE_HEADER>()
+            || (AcpiHeader as ULONG_PTR + header_length as ULONG_PTR) > TableEnd
+        { break; }                                       // truncated: keep what we have
         let header_type = (*AcpiHeader).Type;
         if header_type == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC as u8 {
-            if (header_length as usize) != size_of::<ACPI_MADT_LOCAL_APIC>() { return; }
-            ... ONLINE_CAPABLE | ENABLED ... array full ... fill slot ...
+            if (header_length as usize) >= size_of::<ACPI_MADT_LOCAL_APIC>() {
+                let l = AcpiHeader as *const ACPI_MADT_LOCAL_APIC;
+                madt_add_processor(addr_of!((*l).Id).read_unaligned() as u32,
+                                   addr_of!((*l).ProcessorId).read_unaligned() as u32,
+                                   addr_of!((*l).LapicFlags).read_unaligned());
+            }
+        } else if header_type == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_X2APIC as u8 {
+            if (header_length as usize) >= size_of::<ACPI_MADT_LOCAL_X2APIC>() {
+                let x = AcpiHeader as *const ACPI_MADT_LOCAL_X2APIC;
+                madt_add_processor(addr_of!((*x).LocalApicId).read_unaligned(),
+                                   addr_of!((*x).Uid).read_unaligned(),
+                                   addr_of!((*x).LapicFlags).read_unaligned());
+            }
         } else if header_type == AcpiMadtType_ACPI_MADT_TYPE_IO_APIC as u8 {
-            if (header_length as usize) != size_of::<ACPI_MADT_IO_APIC>() { return; }
+            if (header_length as usize) >= size_of::<ACPI_MADT_IO_APIC>() {
             ... read io_id / io_address / io_irq_base ...
-            if HalpApicInfoTable.IoApicPA[io_id as usize] != 0 { return; }
-            ... record ...
+            // Keep the first unit on a duplicated id (Dell PowerEdge); skip a zero address.
+            if io_address != 0 && HalpApicInfoTable.IoApicPA[io_id as usize] == 0 {
+                ... record ...
+            }
+            }
         } else if header_type == AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE as u8 {
-            if (header_length as usize) != size_of::<ACPI_MADT_INTERRUPT_OVERRIDE>() { return; }
-            ... if bus != 0 { return; } // TODO redirect
+            if (header_length as usize) >= size_of::<ACPI_MADT_INTERRUPT_OVERRIDE>() {
+                let o = AcpiHeader as *const ACPI_MADT_INTERRUPT_OVERRIDE;
+                let src = addr_of!((*o).SourceIrq).read_unaligned() as usize;
+                if addr_of!((*o).Bus).read_unaligned() == 0 && src < 16 {
+                    HalpPicVectorRedirect[src] = addr_of!((*o).GlobalIrq).read_unaligned();
+                }
+            }
-        } else {
-            return;
         }
+        // every other type (LAPIC NMI, x2APIC NMI, OEM, reserved) is skipped, not fatal
         AcpiHeader = (AcpiHeader as ULONG_PTR + header_length as ULONG_PTR) as *mut _;
     }
-    if AcpiHeader as ULONG_PTR != TableEnd { return; }
+    madt_place_boot_processor();
 }
```

Two things to verify when applying. First, `HalpPicVectorRedirect` is `pub static mut` in
`halacpi.rs:288`; both modules compile in the APIC flavor, so check that the `use` resolves on
i386 `apic` and on amd64. Second, after this change entry 0 is always the BSP, so the
`BSPCheck` stamping in `HalpSetupProcessorsTable` (`smp_processor.rs:597-607`) is redundant. Its
condition is also true for every AP, because entry N is CPU N's own LAPIC. Nothing in hal-rust
reads `BSPCheck`. Upstream's `HalpSetupProcessorsTable` sets `ProcessorPrcb` instead, which ours
never does. That gap is a pre-existing divergence, not part of this commit. Gate: smoke amd64 and
i386 on the UP and MP (`[LiveCD_SMP]`) entries.

### 2. syssetup `RegisterDlls` never initialises COM, so COM-based `DllRegisterServer` calls fail in second-stage setup (588c5df465e, `dll/win32/syssetup/wizard.c`)

**Rust:** `sdk/lib/rust/syssetup-rust/src/syssetup/wizard.rs:2931-3001`. It runs on the
`ItemCompletionThread` worker (`:3033`, thread created at `:3099`), and nothing on that path calls
`CoInitialize`. The only `CoInitialize` in the crate is in `install.rs:287` (`CreateShortcuts`),
which runs on a different thread.

**Upstream:** `CoInitialize(NULL)` runs before `SetupInstallFromInfSectionW(... SPINST_REGSVR ...)`,
and `CoUninitialize()` runs after it if init succeeded. The commit's own comment explains why:
quartz, qcap, devenum and qedit register through `IFilterMapper2`, get `CO_E_NOTINITIALIZED`, and
setup then reports "DllRegisterServer failed". This matters more after the Wine-10 DirectX
reimport in this same merge.

**Impact:** on installed (non-LiveCD) boots, second-stage setup fails to register every
COM-using DLL and shows errors. Smoke does not run second-stage setup.

```diff
@@ unsafe extern "C" fn RegisterDlls(
     RegistrationData.pNotify = pNotify;
+
+    // Upstream 588c5df465e: a DllRegisterServer may use COM (quartz/qcap/devenum/qedit
+    // register through IFilterMapper2). This worker thread never initialised it, so each
+    // failed CO_E_NOTINITIALIZED. regsvr32 brackets the same call for the same reason.
+    let HrCoInit = CoInitialize(core::ptr::null_mut());
 
     if SetupInstallFromInfSectionW(
@@
         Error = GetLastError();
     }
+
+    if HrCoInit >= 0 {
+        CoUninitialize();
+    }
 
     SetupTermDefaultQueueCallback(RegistrationData.DefaultContext);
```

ole32 is already in syssetup's `add_importlibs` (`dll/win32/syssetup/CMakeLists.txt:91`), and
`CoInitialize` and `CoUninitialize` are already bound in `types_*.rs`. The C also wraps the call
in `_SEH2_TRY`; the Rust does not, which is a pre-existing divergence outside this commit.

### 3. kernel32 `BasepCreateDefaultTimerQueue` race leaks a timer queue and its thread (73e69a62d39, `dll/win32/kernel32/client/timerqueue.c`)

**Rust:** `sdk/lib/rust/kernel32-rust/src/client/timerqueue.rs:11-22`. It calls
`RtlCreateTimerQueue(&raw mut BasepDefaultTimerQueue)` directly into the global, which is the
pre-fix `/* FIXME - make this thread safe */` body. Callers (`:107`, `:216`) do an unlocked
`is_null()` check and then create.

**Upstream:** the queue is created into a local, published with `InterlockedCompareExchangePointer`
against NULL, and the loser deletes its queue with `RtlDeleteTimerQueueEx`. An early return also
covers the case where the queue is already set.

**Impact:** if two threads make their first default-queue `CreateTimerQueueTimer` or
`ChangeTimerQueueTimer` call at the same time, both create a queue. One queue, and its worker
thread, leaks. Timers created on the losing queue live on a queue the global no longer names.

```diff
 pub unsafe extern "system" fn BasepCreateDefaultTimerQueue() -> BOOL {
-    let mut Status: NTSTATUS;
-
-    Status = RtlCreateTimerQueue(&raw mut BasepDefaultTimerQueue);
+    if !BasepDefaultTimerQueue.is_null() {
+        return TRUE;
+    }
+    let mut TimerQueue: HANDLE = core::ptr::null_mut();
+    let Status: NTSTATUS = RtlCreateTimerQueue(&mut TimerQueue);
     if !NT_SUCCESS(Status) {
         BaseSetLastNTError(Status);
-        // DPRINT removed
         return 0;
     }
-
+    // Upstream 73e69a62d39: publish once; a racing creator throws its queue away.
+    if !crate::k32_externs::InterlockedCompareExchangePointer(
+        (&raw mut BasepDefaultTimerQueue).cast(), TimerQueue, core::ptr::null_mut(),
+    ).is_null() {
+        RtlDeleteTimerQueueEx(TimerQueue, core::ptr::null_mut());
+    }
     return 1;
 }
```

Readers load the global with a plain read, as the C does. The CAS is SeqCst, so on arm64 the
publish side is ordered, but strictly the reader should use an Acquire load. That is a follow-up,
not part of upstream's fix.

### 4. `KeIsWaitListEmpty` always reports "has waiters" (1477b340bec, `ntoskrnl/ke/wait.c`)

**Rust:** `sdk/lib/rust/ntoskrnl-rust/src/ke/wait.rs:451-455` returns `0u8`, copying the pre-fix
`UNIMPLEMENTED; return FALSE;`. The function is exported (`ntoskrnl.spec:630`) and has no
in-tree caller. Impact: any third-party driver that polls an object's wait list sees a
permanent waiter. Low impact.

```diff
 #[no_mangle]
-pub unsafe extern "system" fn KeIsWaitListEmpty(_Object: PVOID) -> BOOLEAN {
-    let _Object = opaque_ptr(_Object);
-    0u8
+pub unsafe extern "system" fn KeIsWaitListEmpty(Object: PVOID) -> BOOLEAN {
+    // Upstream 1477b340bec: barrier for a consistent snapshot, then IsListEmpty.
+    let Header = opaque_ptr(Object) as *mut ntkrnl_sys::ddk::_DISPATCHER_HEADER;
+    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
+    let Head = core::ptr::addr_of_mut!((*Header).WaitListHead) as PLIST_ENTRY;
+    (core::ptr::read_volatile(core::ptr::addr_of!((*Head).Flink)) == Head) as BOOLEAN
 }
```

### 5. `MprConfigServerConnect` reports success and leaves the out-handle unwritten (38538cdb108, `dll/win32/mprapi/mprapi.c`)

**Rust:** `sdk/lib/rust/mprapi-rust/src/mprapi/mprapi.rs:37-45` returns `ERROR_SUCCESS` without
writing `*phmpr_config`. That is the intermediate ReactOS stub, which was added to the Rust
during the previous sync. Upstream's Wine-10 version sets `*hmprconfig = NULL` and returns
`ERROR_NOT_SUPPORTED`. With ours, a caller gets success plus an uninitialised handle and passes
that handle on.

```diff
     if phmpr_config.is_null() {
         return ERROR_INVALID_PARAMETER;   // keep: Wine would fault here
     }
-    ERROR_SUCCESS
+    // Upstream 38538cdb108 (Wine-10): a stub must not claim success with no handle.
+    *phmpr_config = core::ptr::null_mut();
+    ERROR_NOT_SUPPORTED
```

`ERROR_NOT_SUPPORTED` (50) is not yet referenced anywhere in mprapi-rust. Import it, or define it
from the SDK's `winerror.h`.

### 6. msafd `SO_CONNECT_TIME` on an unconnected socket returns a wall-clock value instead of `0xFFFFFFFF` (a68bfa2fb41, `dll/win32/msafd/misc/dllmain.c`)

This is the one existing-code fix inside the AcceptEx feature commit. **Rust:**
`sdk/lib/rust/msafd-rust/src/misc/dllmain.rs:3359-3363` computes
`GetCurrentTimeInSeconds() - ConnectTime` unconditionally. When `ConnectTime == 0` the result is
seconds-since-1970. Upstream returns `-1` (`0xFFFFFFFF`) in that case, which is the documented
Windows result for "not connected".

```diff
                 SO_CONNECT_TIME => {
-                    DwordBuffer = GetCurrentTimeInSeconds() - (*(*Socket).SharedData).ConnectTime;
+                    let ct = (*(*Socket).SharedData).ConnectTime;
+                    DwordBuffer = if ct != 0 { GetCurrentTimeInSeconds().wrapping_sub(ct) } else { 0xFFFF_FFFF };
```

### 7. AP startup sends SIPIs without the 82489DX version guard (e34b04c3037, deleted `hal/halx86/apic/apicsmp.c`)

**Rust:** `sdk/lib/rust/hal-rust/src/x86/smp_processor.rs:355-370` always sends both SIPIs.
Upstream reads `APIC_VER` and sends them only when `Version >= 0x10`, because the discrete 82489DX
APIC has no STARTUP IPI (MP Spec B.4). Practical impact is nil: no target we run (QEMU i386 or
amd64, any x86-64 part) has an 82489DX. Neither upstream nor we implement the warm-reset-vector
path that such an AP would need, so the guard only stops us sending a reserved IPI.

```diff
+const APIC_VER: ULONG = 0x0030;
@@ pub unsafe extern "system" fn ApicStartApplicationProcessor(
     crude_stall(STALL_10MS);
 
-    // ---- 3. SIPI x2 (edge-triggered startup) ----
+    // ---- 3. SIPI x2 -- 82489DX discrete APICs (version < 0x10) have no STARTUP IPI:
+    // MP Spec B.4, upstream e34b04c3037.
+    if (apic_read(APIC_VER) & 0xFF) < 0x10 {
+        return;
+    }
     let sipi: u64 = ...
```

#### HAL SMP: the rest of the AP-startup comparison

- **23de82ff1e1** (10 ms INIT→SIPI, SIPI, 200 µs, SIPI, 200 µs) is **already true** of ours at
  `smp_processor.rs:352-370`. Ours was written from the MP spec, not from the pre-fix
  single-SIPI C, so it never carried that bug.
- These are pre-existing divergences that none of the 97 commits touches. They are listed so the
  SMP lane can weigh them, not as oracle verdicts:
  - The INIT assert uses `TGM_LEVEL` (`:331-337`). The C uses `APIC_TGM_Edge`, and Intel's
    canonical INIT is edge plus assert (`0x000C4500`). Both forms work on QEMU and modern parts.
  - The stalls are uncalibrated PAUSE loops (`:223-240`; the comment owes
    `KeStallExecutionProcessor`). `STALL_10MS` is only 10 ms if one PAUSE takes at least
    10 cycles at 5 GHz. The `ke/amd64/mproc.rs` AP wait loop is bounded the same way.

## INCONCLUSIVE

### UDP broadcasts not delivered to local sockets (b8ee7950af4, deleted `drivers/network/tcpip/ip/transport/udp/udp.c`)

The pair is classed **NO-RUST-TWIN**: `tcpip.sys` was pruned, and its replacement netstack-rust
is a smoltcp rewrite, not a translation. The *behaviour* the fix adds is an open question.
Upstream builds a second packet and sends it through the loopback NCE when the destination is
255.255.255.255 or an interface's directed broadcast. Its reason is that NDIS does not loop
broadcasts back to the sender.

Ours: `netstack-rust/src/tditrans.rs:450-466` (`Cmd::SendDatagram`) only does
`udp::Socket::send_slice`. In smoltcp 0.13.1, `iface/interface/mod.rs` has broadcast handling
only on the egress side (`is_broadcast` at `:967`, `:1050`), with no local-delivery path. So
the same gap very likely exists: a local socket bound to the port never sees a broadcast sent
from the same host. That breaks local discovery protocols such as NetBIOS and SSDP-style
self-discovery.

**What settles it:** a two-socket TDI test in the `tdidgtest.rs` style. One address object
binds port P, a second sends to `255.255.255.255:P`, and the test asserts receipt. If it fails,
the fix belongs in `tditrans.rs` (on a broadcast destination, also deliver to the matching
local UDP sockets), not in a translation.

## Every pair

| # | Commit | C path | Verdict | Reason |
|---|---|---|---|---|
| 1 | cf89458d9e2 | hal/halx86/legacy/halpnpdd.c | FEATURE-NOT-ADOPTED | New Legacy-PC IRQ arbiter hook. `Status` starts as `STATUS_NOT_SUPPORTED`, so existing paths are unchanged. Rust `legacy_halpnpdd.rs:300` is untouched. |
| 2 | cf89458d9e2 | hal/halx86/legacy/pir/legacypcirqarb.c | FEATURE-NOT-ADOPTED | New file (PIR/PIC arbiter) for legacy no-MPS boxes. No Rust twin, by design. |
| 3 | 4ab7fb63af7 | ntoskrnl/config/cminit.c | NOT-A-FIX | Comment typo. |
| 4 | 4ab7fb63af7 | ntoskrnl/config/cmkcbncb.c | NOT-A-FIX | Comment typo. |
| 5 | 4ab7fb63af7 | ntoskrnl/ps/security.c | NOT-A-FIX | Comment typo. |
| 6 | 4ab7fb63af7 | ntoskrnl/ps/thread.c | NOT-A-FIX | Comment typo. |
| 7 | dedb46771d0 | ntoskrnl/config/cmhvlist.c | NOT-A-FIX | Style and SAL changes; `HiveListValueName` renamed to `HiveListKeyName`. |
| 8 | 396019984c9 | ntoskrnl/config/cmapi.c | ALREADY-FIXED | `config-rust/src/cmapi.rs:1922-1929`: Remove happens before Close (4490bf2afb3). |
| 9 | 396019984c9 | ntoskrnl/config/cmhvlist.c | ALREADY-FIXED | `cmhvlist.rs:339` captures `HiveRootPath`; `:361-400` uses and frees it; `KEY_SET_VALUE` at `:206` and in Remove. |
| 10 | 396019984c9 | ntoskrnl/config/cminit.c | ALREADY-FIXED | `cminit.rs:241` initialises `HiveRootPath`. The CMHIVE offset asserts exist (`offset_check_*.rs`). |
| 11 | adeab17b054 | ntoskrnl/config/cmconfig.c | NOT-A-FIX | Formatting. `12` → `_countof(TempBuffer)`, and `TempBuffer` is `CHAR[12]` (`cmconfig.c:33`). |
| 12 | 38538cdb108 | dll/win32/mprapi/mprapi.c | **CONFIRMED** | `mprapi.rs:44` returns success with an unwritten handle (finding 5). |
| 13 | b232717e207 | dll/win32/rasapi32/rasapi.c | NOT-A-FIX | Printf-format sync, plus new `RasSetCredentials{A,W}` stubs already present at `rasapi.rs:522,534` (`ERROR_UNKNOWN`). |
| 14 | 17f63e110ff | dll/win32/snmpapi/main.c | NOT-A-FIX | `HeapAlloc` → `malloc` swap. snmpapi-rust is internally consistent (Heap* only: 17 alloc, 5 realloc, 19 free). |
| 15 | 4513c2d95a3 | dll/directx/wine/d3d10/effect.c | LIVE-C | Compiled by `d3d10/CMakeLists.txt:15` (under `DLL_EXPORT_VERSION >= 0x600`). No Rust. |
| 16 | 4513c2d95a3 | dll/directx/wine/d3d11/device.c | LIVE-C | Compiled by `d3d11/CMakeLists.txt:19`. No Rust. |
| 17 | e98fa71fdcc | sdk/lib/rtl/waitaddress.c | NOT-A-FIX | Adds UNIMPLEMENTED stubs and is in no CMake. `rtl-sync/src/waitaddr.rs:173` is already a real `RtlWaitOnAddress`. |
| 18 | 588c5df465e | dll/win32/syssetup/wizard.c | **CONFIRMED** | `wizard.rs:2931` `RegisterDlls` has no `CoInitialize` (finding 2). |
| 19 | a68bfa2fb41 | dll/win32/msafd/misc/dllmain.c | **CONFIRMED** | Mostly the AcceptEx feature, but the `SO_CONNECT_TIME` unconnected→`0xFFFFFFFF` hunk is a fix missing at `dllmain.rs:3360` (finding 6). |
| 20 | a68bfa2fb41 | dll/win32/msafd/misc/stubs.c | FEATURE-NOT-ADOPTED | Removes stubs that the new AcceptEx code replaces. |
| 21 | a68bfa2fb41 | drivers/network/afd/afd/bind.c | FEATURE-NOT-ADOPTED | State guard protecting the AcceptEx acceptor socket, which is locked in `CONNECTING`. |
| 22 | a68bfa2fb41 | drivers/network/afd/afd/connect.c | FEATURE-NOT-ADOPTED | `MakeSocketIntoConnection(FCB, Receive)` for super-accept. CONNECTED→`INVALID_PARAMETER` serves the same lock; `connect.rs:655` still returns SUCCESS. Adopt together with AcceptEx. |
| 23 | a68bfa2fb41 | drivers/network/afd/afd/listen.c | FEATURE-NOT-ADOPTED | `AfdSuperAccept`, `SatisfySuperAccept`. |
| 24 | a68bfa2fb41 | drivers/network/afd/afd/main.c | FEATURE-NOT-ADOPTED | `IOCTL_AFD_SUPER_ACCEPT` dispatch and cancel. |
| 25 | a68bfa2fb41 | drivers/network/afd/afd/read.c | FEATURE-NOT-ADOPTED | `AcceptExReceiveComplete`. |
| 26 | 938c14e3276 | drivers/hid/hidclass/pdo.c | ALREADY-FIXED | `hidclass-rust/src/pdo.rs:680`: `PhysicalDeviceObject = PDODeviceObject` (60ab98a547d). |
| 27 | 445d0881b82 | drivers/hid/hidclass/pdo.c | ALREADY-FIXED | `pdo.rs:558-` handles `GUID_HID_INTERFACE_HIDPARSE` and `NOTIFY` (60ab98a547d). |
| 28 | 091855fc4f9 | ntoskrnl/ke/i386/thrdini.c | ALREADY-FIXED | `ke/i386/thrdini.rs:531-550`, under `feature = "smp"` (which ntkrnlmp sets, `ntkrnlmp-cargo/build.rs:318`). It skips the save rather than ASSERTing when `NpxThread != OldThread`. |
| 29 | 1477b340bec | ntoskrnl/ke/wait.c | **CONFIRMED** | `wait.rs:451` is still the `return FALSE` stub (finding 4). |
| 30 | 73e69a62d39 | dll/win32/kernel32/client/timerqueue.c | **CONFIRMED** | `timerqueue.rs:11` creates straight into the global, which races (finding 3). |
| 31 | ae50db9f96d | drivers/serial/serial/pnp.c | FEATURE-NOT-ADOPTED | New `IRP_MN_SURPRISE_REMOVAL` handler. `serial-rust/src/pnp.rs:734` forwards it without disconnecting the interrupt or removing the COM link, which is the pre-fix behaviour. Owed if hot-removable 16550s matter. |
| 32 | b74ed4ac15e | ntoskrnl/ob/obinit.c | FEATURE-NOT-ADOPTED | `ObShutdownSystem` UNIMPLEMENTED stub with no caller. |
| 33 | b74ed4ac15e | ntoskrnl/ps/kill.c | NOT-A-FIX | Rename to `PsShutdownSystem`. Rust keeps its private `PspShutdownProcessManager` (`ps-rust/src/kill.rs:291`), and nothing calls the new name. |
| 34 | 4a1e2f85cc2 | dll/win32/shell32/stubs.cpp | FEATURE-NOT-ADOPTED | `PathIsSlowW` moves out of stubs. Rust `shell32/stubs.rs:613` is still the stub. |
| 35 | 4a1e2f85cc2 | dll/win32/shell32/utils.cpp | FEATURE-NOT-ADOPTED | `PathIsSlowW` implementation. |
| 36 | 1fd8b503be2 | dll/win32/shell32/wine/shellord.c | FEATURE-NOT-ADOPTED | `DAD_*` drag-image functions implemented. Rust `wine/shellord.rs:2100-2120` is still FALSE stubs. |
| 37 | 00a09fd24aa | base/shell/explorer/taskswnd.cpp | FEATURE-NOT-ADOPTED | Win+<n> hotkeys (new). |
| 38 | 00a09fd24aa | base/shell/explorer/traywnd.cpp | FEATURE-NOT-ADOPTED | Win+<n> hotkey registration (new). |
| 39 | cae3c053d47 | hal/halx86/acpi/madt.c | **CONFIRMED** | `madt.rs` is the pre-fix walk: aborts, counts phantom CPUs, no BSP placement (finding 1). |
| 40 | add87f81b9e | drivers/input/kbdclass/kbdclass.c | NOT-A-FIX | Tabs→spaces (`git show -w` is empty). |
| 41 | add87f81b9e | drivers/input/kbdclass/misc.c | NOT-A-FIX | Tabs→spaces. |
| 42 | ca435603375 | drivers/input/mouclass/misc.c | NOT-A-FIX | Tabs→spaces. |
| 43 | ca435603375 | drivers/input/mouclass/mouclass.c | NOT-A-FIX | Tabs→spaces. |
| 44 | 23de82ff1e1 | hal/halx86/apic/apicsmp.c | ALREADY-FIXED | `smp_processor.rs:352-370` already does INIT, 10 ms, SIPI, 200 µs, SIPI, 200 µs. |
| 45 | e34b04c3037 | hal/halx86/apic/apicsmp.c | **CONFIRMED** | No `APIC_VER >= 0x10` guard. Legacy-only impact (finding 7). |
| 46 | cae3c053d47 | hal/halx86/smp/smp.c | NOT-A-FIX | Drops the `extern PPROCESSOR_IDENTITY` after the array-type change. |
| 47 | cae3c053d47 | hal/halx86/smp/mps/mps.c | NOT-A-FIX | Same declaration refactor. |
| 48 | cae3c053d47 | hal/halx86/smp/i386/spinup.c | NOT-A-FIX | Same declaration refactor. |
| 49 | b8ee7950af4 | drivers/network/tcpip/ip/transport/udp/udp.c | NO-RUST-TWIN | tcpip.sys was pruned; netstack-rust (smoltcp) is a rewrite. The broadcast-loopback gap is likely present; see INCONCLUSIVE. |
