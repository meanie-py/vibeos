# Upstream-sync bug oracle: user-mode and driver pairs

**Scope.** These are the 129 `(upstream commit, C file)` pairs in
`~/vibeos-rust-staging/upstream-oracle-merged/userdrv-pairs.txt`, taken from the
`28a26f3b664^1..28a26f3b664^2` merge. For each pair the question was whether our live Rust
still carries the bug that the upstream commit fixed in now-dead C. This was a read-only
review. Nothing was built, booted or edited. Every CONFIRMED verdict cites the Rust line and
the upstream hunk it disagrees with. "Latent" marks code that is linked but never reached
in any configuration we boot today. pcix.sys appears only as a file copy in
`txtsetup.sif`: no service or device ID binds it, because `PNP0A03` maps to `pci`.

## Status (2026-09-30)

**Ported:** 1-16. Item 14 is upstream `f69f04bc67c`'s whole `GetLayoutName` rewrite, not just
the leak fix. Item 8 also needed `tcompTYPE_MSZIP` retyped to `TCOMP`: it was `u32`, and the
code behind the never-set cfg had never been type-checked.

**Threadpool (2026-09-30):** the `kernel32_vista/threadpool.c` rows were parked on "ntdll has
no Tp*". That was wrong: `rtl-threadpool` has had every Tp* body since the RTL flip, and they
were only unexported because ntdll.spec gates them at `-version=0x600+`. ntdll_vista now
links the crate and exports them, as upstream's rtl_vista does, and kernel32_vista ports the
wrappers. The `tpprobe` payload drives every object type and found two bugs on its first boot:
`TpReleaseCleanupGroupMembers` walked its list without container_of, so every call faulted,
and ntdll_vista never initialized its own copy's timer/wait queues. Upstream has the second
bug too.

**pcix (17), ported 2026-10-03** as one batch into `pcix-rust`: 20494b84efc, b82fc1e2b38
(with the `PciBuildRequirementsList` and `Device_ChangeResourceSettings` bodies its BarIndex
walk needs), e51b1c1dbfc and 4dabb9e5825. The `Rtl*MemIoResource` helpers that 4dabb9e5825
calls are now `rtl-misc/src/memres.rs` (all of upstream `memres.c`, flipped), and ntoskrnl
exports them as upstream does. Cargo-checked on all three arches and built. pcix is still
not bound, so nothing boots it.

**Still owed:**
- `4dabb9e5825`'s `arb/ar_memio.c` hunks. They patch functions that arrived with arbiter commit
  `023d8af5566`, which `ar_memio.rs` does not have yet, so they land together with that port.

## Summary

| Verdict | Pairs |
|---|---|
| CONFIRMED (live) | 17 |
| CONFIRMED (latent) | 9 |
| ALREADY-FIXED | 5 |
| NOT-A-FIX | 60 |
| NO-RUST-TWIN | 38 |
| LIVE-C | 0 |
| INCONCLUSIVE | 0 (two side observations below) |

These 26 CONFIRMED pairs collapse into 17 findings, listed below in impact order.

## CONFIRMED: detail and proposed diffs

### 1. USBPORT: an EHCI root hub drives its companion controllers through its own miniport (type confusion)
- **Commit/file:** `da13f3ca820` `drivers/usb/usbport/roothub.c`
- **Upstream:** inside the companion loop of `USBPORT_RootHubPowerAndChirpAllCcPorts`,
  `CompanionPacket = &FdoExtension->...` became `&CompanionFdoExtension->MiniPortInterface->Packet`.
- **Ours:** `sdk/lib/rust/usbport-rust/src/roothub.rs:1050` still uses `(*FdoExtension).MiniPortInterface`.
- **Why it is the bug:** the EHCI miniport's `RH_GetRootHubData` and `RH_SetFeaturePortPower`
  are called with the companion's `MiniPortExt`, which is an OHCI or UHCI extension. The
  EHCI code reads and writes that extension as its own layout, so it touches the wrong
  registers and corrupts memory.
- **Impact:** kernel memory and register corruption at USB start on any EHCI controller
  that has companion controllers (the ICH-style EHCI+UHCI arrangement).
- **Proposed diff:**
```diff
-            CompanionPacket = core::ptr::addr_of_mut!((*(*FdoExtension).MiniPortInterface).Packet);
-
             CompanionFdoExtension =
                 (**Entry).DeviceExtension as PUSBPORT_DEVICE_EXTENSION;
+
+            CompanionPacket = core::ptr::addr_of_mut!((*(*CompanionFdoExtension).MiniPortInterface).Packet);
```

### 2. kernel32 `BasepCopyFileExW`: any non-zero copy flag means fail-if-exists
- **Commit/file:** `91146de1cd0` `dll/win32/kernel32/client/file/copy.c`
- **Upstream:** `dwCopyFlags ? CREATE_NEW : CREATE_ALWAYS` became
  `(dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) ? ...`. Upstream also clears a spurious
  `ERROR_ALREADY_EXISTS` after a successful `CREATE_ALWAYS` open.
- **Ours:** `sdk/lib/rust/kernel32-rust/src/file/copy.rs:274`.
- **Why it is the bug, with a live trigger in our own tree:** `file/move.rs:1005-1006`
  passes `(ReplaceIfExists ? 0 : COPY_FILE_FAIL_IF_EXISTS) | COPY_FILE_OPEN_SOURCE_FOR_WRITE`.
  The flags are therefore never zero, so a cross-volume
  `MoveFileEx(..., MOVEFILE_COPY_ALLOWED | MOVEFILE_REPLACE_EXISTING)` fails when the
  destination exists. `CopyFileEx` and `PrivCopyFileExW` with any other flag (for example
  `COPY_FILE_ALLOW_DECRYPTED_DESTINATION`) fail the same way. This is the bug report
  upstream cites (CORE-10271, Total Commander).
- **Proposed diff:**
```diff
-                    if dwCopyFlags != 0 { CREATE_NEW } else { CREATE_ALWAYS },
+                    if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) != 0 { CREATE_NEW } else { CREATE_ALWAYS },
                     FileBasic.FileAttributes,
                     core::ptr::null_mut(),
                 );
                 if INVALID_HANDLE_VALUE != FileHandleDest {
+                    if (dwCopyFlags & COPY_FILE_FAIL_IF_EXISTS) == 0 && GetLastError() == 183 /* ERROR_ALREADY_EXISTS */ {
+                        SetLastError(0 /* ERROR_SUCCESS */);
+                    }
                     errCode = CopyLoop(
```

### 3. pci.sys bus interface rejects `PCIConfiguration` as a data type
- **Commit/file:** `e7518a9c796` `drivers/bus/pci/pdo.c`
- **Upstream:** `InterfaceBusSetBusData` and `InterfaceBusGetBusData` now accept
  `DataType == PCIConfiguration (4)` as well as `PCI_WHICHSPACE_CONFIG (0)`.
- **Ours:** `sdk/lib/rust/pci-rust/src/pdo.rs:1272` and `:1301` accept only 0.
- **Impact:** pci.sys is the live PCI bus driver. StorPort miniports
  (`StorPortGetBusData` / `StorPortSetBusDataByOffset`) pass `PCIConfiguration`, get 0
  bytes back, and fail to start. Upstream observed this with qla1xxx under MS storport.
- **Proposed diff (both sites):**
```diff
-    if DataType != PCI_WHICHSPACE_CONFIG {
+    if DataType != PCI_WHICHSPACE_CONFIG && DataType != BUS_DATA_TYPE_PCIConfiguration as ULONG {
```

### 4. kernel32 `ReadFile`/`WriteFile`: NULL byte-count pointer crashes, and the count is lost on warning statuses
- **Commits/file:** `72bfa0c88b7` and `5e998de2483` in `dll/win32/kernel32/client/file/rw.c`
- **Upstream:** in the synchronous path, `*lpNumberOf...` is now written only when the
  pointer is non-NULL, as on Windows 8+ (Wine's crypt32 relies on this). It is written
  **before** the `NT_SUCCESS` check, so warning statuses such as
  `STATUS_BUFFER_OVERFLOW` (a pipe message larger than the buffer, which surfaces as
  `ERROR_MORE_DATA`) still report the bytes transferred (CORE-17376). The
  `Iosb = {0}` half of the fix is already present: we use `zeroed()`.
- **Ours:** `sdk/lib/rust/kernel32-rust/src/file/rw.rs:131-136` (WriteFile) and `:247-257`
  (ReadFile). Both write unconditionally, and only on success.
- **Impact:** an access violation in any caller that passes NULL with no OVERLAPPED.
  A message-mode pipe read that returns `ERROR_MORE_DATA` reports 0 bytes, which breaks
  the caller's partial-read loop (upstream: Calibre).
- **Proposed diff (WriteFile; ReadFile is symmetric, including the EOF branch at :247):**
```diff
-        if NT_SUCCESS(Status) {
-            *lpNumberOfBytesWritten = Iosb.Information as DWORD;
-        } else {
+        if !lpNumberOfBytesWritten.is_null() {
+            *lpNumberOfBytesWritten = Iosb.Information as DWORD;
+        }
+        if !NT_SUCCESS(Status) {
             BaseSetLastNTError(Status);
             return 0;
         }
```
```diff
         if Status == STATUS_END_OF_FILE {
-            *lpNumberOfBytesRead = 0;
+            if !lpNumberOfBytesRead.is_null() { *lpNumberOfBytesRead = 0; }
             return 1;
         }
-        if NT_SUCCESS(Status) {
-            *lpNumberOfBytesRead = Iosb.Information as DWORD;
-        } else {
+        if !lpNumberOfBytesRead.is_null() {
+            *lpNumberOfBytesRead = Iosb.Information as DWORD;
+        }
+        if !NT_SUCCESS(Status) {
```

### 5. kernel32 `DeviceIoControl`: NULL `lpBytesReturned` crashes on the synchronous path
- **Commit/file:** `e1e3b8aa3de` `dll/win32/kernel32/client/file/deviceio.c`
- **Ours:** `sdk/lib/rust/kernel32-rust/src/file/deviceio.rs:231-236` writes through the
  pointer without a check. The overlapped branch at `:184` already checks.
- **Proposed diff:**
```diff
-        if NT_SUCCESS(Status) {
-            *lpBytesReturned = Iosb.Information as DWORD;
-        } else {
-            if !NT_ERROR(Status) {
+        if NT_SUCCESS(Status) {
+            if !lpBytesReturned.is_null() { *lpBytesReturned = Iosb.Information as DWORD; }
+        } else {
+            if !NT_ERROR(Status) && !lpBytesReturned.is_null() {
                 *lpBytesReturned = Iosb.Information as DWORD;
             }
```

### 6. userinit (LiveCD/MiniNT): the shell is started without the SMSS dynamic environment
- **Commit/file:** `eec8c977083` `base/system/userinit/userinit.c`
- **Upstream:** in LiveCD mode, the `SHELL` case builds
  `CreateEnvironmentBlock(&pEnv, NULL, TRUE)` and passes it with
  `CREATE_UNICODE_ENVIRONMENT` through `StartShell(pEnv)` → `StartProcess(cmd, pEnv)`, then
  calls `DestroyEnvironmentBlock`. Winlogon builds no augmented environment in MiniNT mode.
- **Ours:** `sdk/lib/rust/userinit-rust/src/userinit.rs:249-285` calls `StartProcess` with
  a NULL environment and plain `NORMAL_PRIORITY_CLASS`. `StartShell()` is called at `:759`.
- **Impact:** the livecd that smoke boots runs in exactly this mode. explorer.exe and
  everything it spawns get no `COMPUTERNAME`, `NUMBER_OF_PROCESSORS`, `OS` or
  `PROCESSOR_*`. Any program that reads them sees them missing.
- **Proposed diff (sketch):** add a `pEnvironment: PVOID` parameter to
  `StartProcess`/`StartShell`, pass `NORMAL_PRIORITY_CLASS | CREATE_UNICODE_ENVIRONMENT`
  and `pEnvironment`, and in the `SHELL` arm:
```rust
let mut env: PVOID = core::ptr::null_mut();
if bIsLiveCD != 0 && CreateEnvironmentBlock(&mut env, core::ptr::null_mut(), TRUE) == 0 {
    env = core::ptr::null_mut(); // fall back to the inherited block, as upstream WARNs
}
Success = StartShell(env);
if !env.is_null() { DestroyEnvironmentBlock(env); }
```
  This needs `.import_lib("dll/win32/userenv/userenv.spec", "userenv.dll", "userenv")` in
  `sdk/lib/rust/userinit-cargo/build.rs`. Upstream also passes NULL for the installer
  path. Because this changes the smoke boot's shell launch, it needs a smoke run before it
  lands.

### 7. SMSS `SmpTranslateSystemPartitionInformation`: no BootDir fallback when the system partition has no drive letter
- **Commits/file:** `c498d0930a7` and `aa8d624253a` in `base/system/smss/sminit.c`
- **Upstream:** when the DosDevices scan ends with `STATUS_NO_MORE_ENTRIES`, it falls back
  to `SharedUserData->NtSystemRoot[0..2]`; any other error still returns early. The
  addendum points `DirInfo->Name.Buffer` at the scratch buffer first (the original
  fallback wrote through NULL), and moves both buffers into header-typed structs so they
  are correctly aligned.
- **Ours:** `sdk/lib/rust/smss-csrss-rust/src/smss/sminit.rs:993-994` and `:1045-1046`
  return on any failure, and `:1065` builds BootDir from `(*DirInfo).Name.Buffer`.
- **Impact:** on UEFI boots (amd64 and arm64), the system partition is the ESP, which
  normally has no drive letter. In that case
  `HKLM\Software\Microsoft\Windows\CurrentVersion\Setup\BootDir` is never written. The
  impact is low to moderate: the readers are setup and boot-configuration tools.
- **Proposed diff:**
```diff
-    if !NT_SUCCESS(Status) {
-        return;
-    }
-
     /* Keep searching until we find it */
-    loop {
+    while NT_SUCCESS(Status) {
         ...
-        if !NT_SUCCESS(Status) {
-            break;
-        }
     }
+    let mut drive: [u16; 2] = [0; 2];
     if !NT_SUCCESS(Status) {
-        return;
+        if Status != STATUS_NO_MORE_ENTRIES { return; }
+        drive = [(*SharedUserData).NtSystemRoot[0], (*SharedUserData).NtSystemRoot[1]];
+    } else {
+        drive = [*(*DirInfo).Name.Buffer, *(*DirInfo).Name.Buffer.add(1)];
     }
     ...
-    wcsncpy(LinkBuffer.as_mut_ptr(), (*DirInfo).Name.Buffer, 2);
+    LinkBuffer[0] = drive[0];
+    LinkBuffer[1] = drive[1];
```
  The diff also needs the `SharedUserData` const (copy from `smutil.rs:59`). The
  `[i8; N]` scratch buffers at `:918` and `:921` should become `#[repr(C)]` structs that
  embed the header (see side observation 2).

### 8. cabinet FCI: MSZIP compression compiled out
- **Commit/file:** `4681b3e473a` `dll/win32/cabinet/fci.c`
- **Upstream:** removed the `#ifdef HAVE_ZLIB` guards, so `tcompTYPE_MSZIP` is always
  wired. Before that commit the old C build defined it via
  `add_definitions(-DHAVE_ZLIB)` in `dll/win32/cabinet/CMakeLists.txt` (checked at
  `4681b3e473a^`).
- **Ours:** `sdk/lib/rust/cabinet-rust/src/cabinet/fci.rs:1753` has `#[cfg(HAVE_ZLIB)]` on
  the match arm. Nothing sets that cfg anywhere in the tree (no `--cfg`, build.rs or
  Cargo feature), so the arm is compiled out. `compress_MSZIP` at `:652` exists and is
  dead.
- **Impact:** `FCIAddFile(..., tcompTYPE_MSZIP)` silently falls back to uncompressed with
  a FIXME, so every cabinet we create is stored rather than deflated. Note that this
  happened during translation, not in the old C build.
- **Proposed diff:**
```diff
-            #[cfg(HAVE_ZLIB)]
             tcompTYPE_MSZIP => {
```

### 9. cabinet FDI: next-cabinet path gets a leading `\` when the user path is empty
- **Commit/file:** `4681b3e473a` `dll/win32/cabinet/fdi.c` (the `__REACTOS__` path-paste
  hack is dropped for Wine's logic)
- **Ours:** `sdk/lib/rust/cabinet-rust/src/cabinet/fdi.rs:2723-2733` always does
  `strcat("\\")` before `nextname`. With `pathlen == 0` that produces the root-relative
  `\next.cab` instead of `next.cab`.
- **Impact:** low. A spanned cabinet set opened with an empty or relative path fails with
  `FDIERROR_CABINET_NOT_FOUND`.
- **Proposed diff:**
```diff
                             if pathlen != 0 {
                                 strcpy(fullpath.as_mut_ptr(), userpath.as_ptr());
-                                if fullpath[pathlen as usize - 1] == b'\\' as i8 {
-                                    fullpath[pathlen as usize - 1] = 0;
-                                }
+                                if fullpath[pathlen as usize - 1] != b'\\' as i8 {
+                                    strcat(fullpath.as_mut_ptr(), b"\\\0".as_ptr() as *const i8);
+                                }
                             }
                             if filenamelen != 0 {
-                                strcat(fullpath.as_mut_ptr(), b"\\\0".as_ptr() as *const i8);
                                 strcat(fullpath.as_mut_ptr(), (*cab).mii.nextname);
+                            } else if fullpath[0] != 0 {
+                                fullpath[strlen(fullpath.as_ptr()) - 1] = 0;
                             }
```

### 10. SMSS `SmpTerminate`: exit code is the `NtRaiseHardError` result, not the failure code
- **Commit/file:** `f255c9f046e` `base/system/smss/smss.c`
- **Ours:** `sdk/lib/rust/smss-csrss-rust/src/smss/smss.rs:487`.
- **Impact:** diagnostic only. When SMSS dies, the process exit status (and the critical-process
  bugcheck parameter) is the hard-error return, usually 0, instead of the real NTSTATUS.
- **Proposed diff:**
```diff
-    let Status = NtRaiseHardError(
+    let mut Status = NtRaiseHardError(
     ...
+    if ParameterCount >= 2 { Status = *Parameters.add(1) as NTSTATUS; }
     NtTerminateProcess(NtCurrentProcess(), Status)
```

### 11. SMSS `SmpUnhandledExceptionFilter`: unconditional breakpoint and swapped AV fields (latent)
- **Commit/file:** `5178c3913ed` `base/system/smss/smss.c`
- **Ours:** `sdk/lib/rust/smss-csrss-rust/src/smss/smss.rs:491-505` calls `DbgBreakPoint()`
  unconditionally and labels `ExceptionInformation[0]` / `[1]` the wrong way round. Upstream
  breaks only when `PEB->BeingDebugged` is set, and prints address and R/W only for
  AV, guard-page, stack-overflow and in-page errors.
- **Latent:** no Rust code calls this filter. smss-rust has no SEH, so neither `_main`'s
  `_SEH2_EXCEPT` nor the `SmpApiLoop` wrap from `39a1f3a8f46` is wired. The fix has to
  land before SEH is wired up, or a debug-free boot hits an unhandled breakpoint inside
  the filter.
- **Proposed diff:** wrap the two prints and the break in
  `if (*NtCurrentPeb()).BeingDebugged != 0 { DbgBreakPoint(); }`, and print `[1]` as the
  address and `[0]` as R/W only for those four exception codes.

### 12. SMSS: empty `*Execute` registry strings are queued as commands
- **Commit/file:** `41dadd942f7` `base/system/smss/sminit.c`
- **Ours:** `sminit.rs` table entries for `BootExecute` (:251), `SetupExecute` (:260) and
  `Execute` (:386) all use `SmpConfigureMemoryMgmt` (:642). Nothing filters empty strings.
- **Impact:** low. Each empty string fails `SmpParseCommandLine` with
  `STATUS_UNSUCCESSFUL` and produces a DPRINT1. Boot carries on.
- **Proposed diff:** add a handler and point the three entries at it:
```rust
#[no_mangle]
pub unsafe extern "system" fn SmpConfigureExecute(_n: PWSTR, _t: u32, ValueData: *mut c_void,
        ValueLength: u32, _c: *mut c_void, EntryContext: *mut c_void) -> i32 {
    let mut len: usize = 0;
    if RtlStringCbLengthW(ValueData as *const u16, ValueLength as usize, &mut len) < 0
        || len < core::mem::size_of::<u16>() { return STATUS_SUCCESS; }
    SmpSaveRegistryValue(EntryContext as PLIST_ENTRY, ValueData as PWSTR, core::ptr::null_mut(), 1u8)
}
```

### 13. mlang `IMLangFontLink2::GetStrCodePages` keeps consuming characters after the code-page set goes empty
- **Commit/file:** `15c9005acb6` `dll/win32/mlang/mlang.c` (Wine 10 sync)
- **Upstream:** a character is accepted only if `(cps & cp) != 0` and the priority-cp bits
  agree. Otherwise it does `i--; break`.
- **Ours:** `sdk/lib/rust/mlang-rust/src/mlang/mlang.rs` in `fnIMLangFontLink2_GetStrCodePages`
  (around :4911) does `cps &= cp` unconditionally and breaks only on `priority_cp & cps`.
- **Evidence:** `modules/rostests/winetests/mlang/mlang.c:1278-1281`, from the matching
  winetest sync, expects `processed == 3` for the 4-character string with priority 0. Our
  code returns 4, with `cps` collapsed to 0.
- **Also pre-fix, lower confidence:** `map_font` (around :2190) still uses the
  `GetTextCharset` probe. Upstream replaced it with an `EnumFontFamiliesExW` search that
  checks `fsCsb`. That is a behaviour improvement, not shown to be a defect.
- **Proposed diff:**
```diff
-        if cps == 0 {
-            cps = cp;
-        } else {
-            cps &= cp;
-        }
-
-        if (priority_cp & cps) != 0 {
-            break;
-        }
+        if cps == 0 {
+            cps = cp;
+        } else if (cps & cp) != 0 && ((priority_cp & cps) ^ (priority_cp & cp)) == 0 {
+            cps &= cp;
+        } else {
+            i -= 1;
+            break;
+        }
```
  The `ret_len = min(i + 1, src_len)` tail stays as it is, which matches upstream.

### 14. userinit livecd `GetLayoutName`: DLL and registry-key leaks
- **Commit/file:** `f69f04bc67c` `base/system/userinit/livecd.c`
- **Ours:** `sdk/lib/rust/userinit-rust/src/livecd.rs:253-352`. On a successful
  `LoadStringW` it returns at :328 without `FreeLibrary` (one keyboard-layout DLL leaked
  per enumerated layout). When `Layout Text` fails it returns 0 at :352 without
  `RegCloseKey`. It also uses `LoadLibraryW` instead of
  `LoadLibraryExW(..., LOAD_LIBRARY_AS_DATAFILE)`.
- **Impact:** low. The leaks are confined to the LiveCD language page.
- **Proposed diff:** move `FreeLibrary(hLib)` before the success return, close `hKey` on
  every path (one exit label), and load with `LOAD_LIBRARY_AS_DATAFILE` (0x2).

### 15. userinit livecd: the current KLID is matched case-sensitively
- **Commit/file:** `c45c83a4dc2` `base/system/userinit/livecd.c`
- **Ours:** `livecd.rs:486`, `wcscmp(szLayoutId, szCurrentLayoutId)`.
  `GetKeyboardLayoutNameW` returns uppercase hex (`0000040C`) while the registry subkeys
  can be lowercase (`0000040c`), so the current layout is never pre-selected.
- **Proposed diff:** `wcscmp` → `_wcsicmp`. It is already imported via `crate::preamble`
  (see :176).

### 16. i8042prt reports 4 function keys instead of 12
- **Commit/file:** `c4517f4b0a3` `drivers/input/i8042prt/keyboard.c`
- **Ours:** `sdk/lib/rust/i8042prt-rust/src/keyboard.rs:463`.
- **Impact:** low. The value goes to `IOCTL_KEYBOARD_QUERY_ATTRIBUTES` and on to
  `GetKeyboardType(2)`.
- **Proposed diff:** `NumberOfFunctionKeys = 4;` → `= 12;`

### 17. pcix.sys: fixes ported to dead C (latent, because pcix is not bound)
These pairs are all pre-fix in `sdk/lib/rust/pcix-rust` but are never reached. They should
be ported as one batch before pcix is ever enabled.
- `20494b84efc` enum.c: `PciComputeNewCurrentSettings` must skip shared Port/Memory
  descriptors (anything other than device- or driver-exclusive, e.g. the VGA legacy
  ranges). Ours: `enumr.rs` around :167, `CmResourceTypePort | CmResourceTypeMemory =>
  BaseResource = Partial`. Diff: `if ShareDisposition != CmResourceShareDeviceExclusive &&
  != CmResourceShareDriverExclusive { /* skip */ } else { ... }`.
- `b82fc1e2b38` enum.c: `ResourceArray` is never filled (`BaseResource` is assigned and
  never stored), and the update loop writes `PreviousDescriptor` instead of
  `CurrentDescriptor` (`enumr.rs` around :221-254). Port upstream's `BarIndex` walk over
  `PciResources->Limit[]` and the corrected change predicate. The drain-advance half of
  this commit is **already** in our Rust (`enumr.rs:159-163` advances `Partial`).
  tr_irq.c: for a bridge FDO, `ParentBus` stays 0 (`arb/tr_irq.rs:109-112`). Upstream uses
  `PdoExtension->ParentFdoExtension->BaseBus`.
- `e51b1c1dbfc` utils.c: `PciGetRegistryValue` computes the data size as
  `NeededLength - sizeof(KEY_VALUE_PARTIAL_INFORMATION)`. Because of `Data[1]` plus
  padding, that drops the last 3 bytes of every value. Ours: `utils.rs:366`. Diff:
  `NeededLength = (*PartialInfo).DataLength;`.
- `4dabb9e5825` utils.c, enum.c, device.c, ppbridge.c: the BAR-limit rework (named bounds,
  64-bit BARs above 4 GB, a BAR with no address bits treated as absent,
  `CmResourceTypeMemoryLarge` accepted in the current/query paths) and the
  `PciCanDisableDecodes` hack-flag rework (`PCI_HACK_KEEP_DECODES_ON_STOP` /
  `NEVER_POWER_DOWN`). Ours: `utils.rs:1367` (still `PCI_HACK_DONT_DISABLE_DECODES` and
  `!NO_PM_CAPS`), `utils.rs:1563-1640` (`PciGetLengthFromBar` and
  `PciCreateIoDescriptorFromBarLimit(*mut u32)`). Upstream's message says the old decode
  logic "broke VGA for months". Port it wholesale. It does not reduce to a hunk diff.

## INCONCLUSIVE

There are no INCONCLUSIVE pairs. Two side observations came up during the review. **Both
were ported 2026-09-30.** (1) syssetup-rust now uses `DWLP_USER` at all 22 live sites in
install/proppage/wizard, because the post-fix C uses it in every one of those functions;
`addons.c` is unbuilt. (2) `sminit.rs` was already header-typed. The same class was still in
`smutil.rs` `SmpQueryRegistrySosOption` (a `[u8; N]` that was also half the C's
`WCHAR[N]` size), and that is fixed now. The original notes follow.

1. **`0a2244b0b54` also changes `syssetup/install.c`, `proppage.c` and `wizard.c`**
   (`GWLP_USERDATA` → `DWLP_USER`). Those files are not in this list, and our
   `syssetup-rust/src/syssetup/install.rs:918,935,1081,1098` still use `GWLP_USERDATA`.
   Whether that is a bug depends on whether ReactOS's dialog class or a subclass ever
   claims `GWLP_USERDATA`. To settle it, grep `win32ss/user` for dialog-manager use of
   `GWLP_USERDATA`. This belongs to whoever owns the list containing those files.
2. **SMSS `[i8; N]` header buffers** (`sminit.rs:918,921`; the C fix in `aa8d624253a` is
   about alignment). Casting an align-1 byte array to `*mut KEY_VALUE_PARTIAL_INFORMATION`
   and dereferencing it is unaligned access in Rust's model. User-mode
   `NtQueryValueKey`/`NtQueryDirectoryObject` also probe the buffer for ULONG alignment.
   To settle it: check the stack offsets of these locals in the i386, amd64 and arm64
   smss.exe disassembly, or just switch them to `#[repr(C)]` header-plus-tail structs
   regardless.

## All pairs

| Commit | File | Verdict | Reason |
|---|---|---|---|
| 3518663767d | kernel32_vista/vista.c | ALREADY-FIXED | CompareStringOrdinal ported, kernel32-vista-rust/src/vista.rs:674 |
| 3518663767d | winnls/string/locale.c | NOT-A-FIX | removes a WINVER>=0x600 dead copy (move to vista.c). Our locale.rs:28490 still also exports one |
| 22ca1e736ad | kernel32_vista/threadpool.c | PORTED (2026-09-30) | was "ntdll has no Tp*": wrong, `rtl-threadpool` had them all, just unexported at 0x502. Now kernel32-vista-rust threadpool.rs over ntdll_vista's Tp* |
| dfaac47afc3 | kernel32_vista/SetThreadDescription.c | NOT-A-FIX | refactor to RtlInitUnicodeStringEx. misc.rs:78 differs only at the 0xFFFE-byte edge |
| dfaac47afc3 | kernel32_vista/ThreadDescription.c | ALREADY-FIXED | GetThreadDescription retry loop ported, misc.rs:181 |
| 5e998de2483 | client/file/rw.c | CONFIRMED | #4: byte count lost on warning statuses (rw.rs:131,252) |
| e4aaf68c2de | client/file/rw.c | NOT-A-FIX | whitespace |
| 20494b84efc | pcix/enum.c | CONFIRMED (latent) | #17: shared resources not skipped |
| c498d0930a7 | smss/sminit.c | CONFIRMED | #7: no BootDir fallback |
| aa8d624253a | smss/sminit.c | CONFIRMED | #7: fallback buffer and alignment |
| ae2827f4810 | ide/pciide/pciide.c | NOT-A-FIX | DMA feature in the rewritten pciidex stack |
| ae2827f4810 | pciidex/acpi.c | NO-RUST-TWIN | new TU, unbuilt (pciidex CMakeLists) |
| ae2827f4810 | pciidex/chipset/ahci_generic.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/ahci_hw.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/ahci_io.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/amd.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/ati.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/cmd.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/intel.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/pata_generic.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/pata_hw.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/pata_io.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/pata_legacy.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/pctech.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/sil680.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/svw_pata.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/svw_sata.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/toshiba.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/chipset/via.c | NO-RUST-TWIN | chipset layer unbuilt |
| ae2827f4810 | pciidex/fdo.c | NOT-A-FIX | 15k-line DMA/chipset rewrite. pciidex-rust ports the pre-rewrite driver |
| ae2827f4810 | pciidex/miniport.c | NOT-A-FIX | same rewrite |
| ae2827f4810 | pciidex/pciidex.c | NOT-A-FIX | same rewrite |
| ae2827f4810 | pciidex/pdo.c | NOT-A-FIX | same rewrite |
| ae2827f4810 | pciidex/power.c | NOT-A-FIX | same rewrite |
| ac336478887 | atapi/dev_config.c | NO-RUST-TWIN | new ATA driver, unbuilt. atapi-rust is the NT4 miniport |
| ac336478887 | atapi/dev_error.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/dev_identify.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/dev_power.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/dev_timings.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/enum.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/fdo.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/ioctl.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/pdo.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/portstate.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/satl.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/scsi.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/smart.c | NO-RUST-TWIN | same |
| ac336478887 | atapi/wmi.c | NO-RUST-TWIN | same |
| 62537b39b79 | mprapi/mprapi.c | ALREADY-FIXED | stub ported, mprapi-rust/src/mprapi/mprapi.rs:37 |
| ba7a6fc7d9a | shell32/CShellLink.cpp | NOT-A-FIX | nIconIndex signedness. Rust casts to i32 at every use (shell_link.rs:3472,3642,4502) |
| 15c9005acb6 | mlang/mlang.c | CONFIRMED | #13: GetStrCodePages (winetest-proven) |
| 42d178789b3 | client/procansi.c | NOT-A-FIX | widens a race window. The UAF fix (#8282) is already in proc_getters.rs:454 |
| 9f5b0f17612 | userinit/userinit.c | NOT-A-FIX | message and log nits |
| eec8c977083 | userinit/userinit.c | CONFIRMED | #6: LiveCD shell environment block |
| 4e2cca167dd | winlogon/winlogon.c | NOT-A-FIX | redundant-init cleanup |
| 72bfa0c88b7 | client/file/rw.c | CONFIRMED | #4: NULL out-pointer (Iosb init already zeroed) |
| e1e3b8aa3de | client/file/deviceio.c | CONFIRMED | #5 |
| 9989b9ec1d6 | client/proc.c | NOT-A-FIX | drops an ASSERT tied to an RTL ppb change. Ours is a release-inert debug_assert (proc_params.rs:154), and rtl-ppb still pointer-copies |
| d08a581eb1d | explorer/startmnusite.cpp | NOT-A-FIX | ShowUndockMenuItem feature |
| 2263c9ca0ff | userinit/userinit.c | NOT-A-FIX | feature: shell working directory = profile (Server Core prerequisite) |
| 28ed28dac10 | jscript/decode.c | NOT-A-FIX | literal style, equivalent |
| 91146de1cd0 | client/file/copy.c | CONFIRMED | #2 |
| 5a4ed508fd4 | shlwapi/kernelbase.c | NOT-A-FIX | code move, deliberately unbuilt (shlwapi CMakeLists:30) |
| d7aa4ad3f3b | syssetup/addons.c | NO-RUST-TWIN | addons.c unbuilt, no Rust port |
| d524789ed0e | userinit/livecd.c | NOT-A-FIX | MiniNT key check, equivalent to the MININT option check |
| d524789ed0e | userinit/userinit.c | NOT-A-FIX | rename |
| 064dd8f8320 | userinit/livecd.c | NOT-A-FIX | caches the installer path |
| 064dd8f8320 | userinit/userinit.c | NOT-A-FIX | function move |
| ded99690499 | atapi/fdo.c | NO-RUST-TWIN | new ATA driver, unbuilt |
| 5178c3913ed | smss/smss.c | CONFIRMED (latent) | #11: filter has no caller |
| 41dadd942f7 | smss/sminit.c | CONFIRMED | #12 |
| 41dadd942f7 | smss/smss.c | NOT-A-FIX | DPRINT text |
| 1721f743eca | smss/sminit.c | NOT-A-FIX | debug-print of an uninitialised Type, and buffer naming |
| 1721f743eca | smss/smss.c | NOT-A-FIX | dead SmpEnableDots removal |
| 1721f743eca | smss/smutil.c | NOT-A-FIX | dead SmpQueryRegistrySosOption removal |
| c4517f4b0a3 | i8042prt/keyboard.c | CONFIRMED | #16 |
| b299049410b | userinit/livecd.c | NOT-A-FIX | feature: unattended → installer. Adopting it would change the smoke flow |
| d0e71de116a | pcix/arb/ar_busno.c | NOT-A-FIX | arbiter OrderingList feature. Rust arbiters are stubs |
| d0e71de116a | pcix/arb/ar_memio.c | NOT-A-FIX | same |
| 81f04b39ba4 | pcix/intrface/busintrf.c | NOT-A-FIX | PCI interfaces feature |
| 81f04b39ba4 | pcix/intrface/pmeintf.c | NOT-A-FIX | same |
| 81f04b39ba4 | pcix/pci/config.c | NOT-A-FIX | same |
| 81f04b39ba4 | pcix/pdo.c | NOT-A-FIX | same |
| c1827c06312 | kernel32_vista/threadpool.c | PORTED (2026-09-30) | see 22ca1e736ad |
| b54e416408e | csrsrv/init.c | NOT-A-FIX | `%ld`→`%lu` for session IDs < 2^31 |
| b54e416408e | basesrv/init.c | NOT-A-FIX | same |
| 748daf70c42 | csrsrv/init.c | NOT-A-FIX | comments |
| 4cee655099e | pciidex/chipset/ahci_generic.c | NO-RUST-TWIN | chipset layer unbuilt |
| 82d0c38495a | userinit/livecd.c | NOT-A-FIX | renames |
| c45c83a4dc2 | userinit/livecd.c | CONFIRMED | #15 |
| badbcd28440 | userinit/livecd.c | NOT-A-FIX | UX fallback to en-US |
| f69f04bc67c | userinit/livecd.c | CONFIRMED | #14 |
| 4681b3e473a | cabinet/cabinet_main.c | NOT-A-FIX | includes |
| 4681b3e473a | cabinet/fci.c | CONFIRMED | #8: MSZIP cfg'd out |
| 4681b3e473a | cabinet/fdi.c | CONFIRMED | #9 |
| c6806eaec2c | client/proc.c | ALREADY-FIXED | processor-group APIs, proc_getters.rs:23-43 |
| c6806eaec2c | client/thread.c | ALREADY-FIXED | thread.rs:695 |
| 0a2244b0b54 | syssetup/addons.c | NO-RUST-TWIN | unbuilt (see side observation 1 for siblings) |
| b6bdcab9d0e | smss/smss.c | NOT-A-FIX | string constants |
| b6bdcab9d0e | smss/smsubsys.c | NOT-A-FIX | string constants |
| 8f5a60b2879 | smss/smloop.c | NOT-A-FIX | SAL, const |
| f255c9f046e | smss/smss.c | CONFIRMED | #10 |
| 39a1f3a8f46 | smss/smloop.c | NOT-A-FIX | SEH diagnostic wrap. smss-rust has no SEH at all (see #11) |
| 39a1f3a8f46 | smss/smss.c | NOT-A-FIX | SAL |
| 437759f505d | smss/smloop.c | NOT-A-FIX | DBG tracing |
| 437759f505d | csrsrv/session.c | NOT-A-FIX | tracing. The ApiNumber clamp moved, so behaviour is equivalent |
| b82fc1e2b38 | pcix/arb/tr_irq.c | CONFIRMED (latent) | #17: bridge ParentBus |
| b82fc1e2b38 | pcix/device.c | NOT-A-FIX | implements an UNIMPLEMENTED stub (feature) |
| b82fc1e2b38 | pcix/enum.c | CONFIRMED (latent) | #17: ResourceArray/BAR mapping |
| 3e868179702 | pcix/arb/tr_irq.c | NOT-A-FIX | newlines |
| e7518a9c796 | bus/pci/pdo.c | CONFIRMED | #3 |
| 023d8af5566 | pcix/arb/ar_busno.c | NOT-A-FIX | arbiter implementation (feature) |
| 023d8af5566 | pcix/arb/ar_memio.c | NOT-A-FIX | same |
| 023d8af5566 | pcix/arb/arb_comn.c | NOT-A-FIX | same |
| db6d5bee335 | pcix/enum.c | NOT-A-FIX | comment typo |
| db6d5bee335 | pcix/init.c | NOT-A-FIX | comment typo |
| db6d5bee335 | pcix/utils.c | NOT-A-FIX | DPRINT typo |
| e51b1c1dbfc | pcix/fdo.c | NOT-A-FIX | ECAM feature |
| e51b1c1dbfc | pcix/pci/config.c | NOT-A-FIX | ECAM feature |
| e51b1c1dbfc | pcix/utils.c | CONFIRMED (latent) | #17: registry value truncated by 3 bytes |
| 7baff0a9338 | pcix/enum.c | NOT-A-FIX | PCIe capability discovery feature |
| 7baff0a9338 | pcix/pci/express.c | NOT-A-FIX | same |
| da13f3ca820 | usbport/roothub.c | CONFIRMED | #1 |
| 4dabb9e5825 | pcix/arb/ar_memio.c | NOT-A-FIX | edits arbiter code our Rust stub does not have |
| 4dabb9e5825 | pcix/debug.c | NOT-A-FIX | debug text for a new resource type |
| 4dabb9e5825 | pcix/device.c | CONFIRMED (latent) | #17: BAR limits / 64-bit |
| 4dabb9e5825 | pcix/enum.c | CONFIRMED (latent) | #17: MemoryLarge, decode hack flags |
| 4dabb9e5825 | pcix/pci/ppbridge.c | CONFIRMED (latent) | #17: BAR limits |
| 4dabb9e5825 | pcix/utils.c | CONFIRMED (latent) | #17: PciCanDisableDecodes, BAR length |
