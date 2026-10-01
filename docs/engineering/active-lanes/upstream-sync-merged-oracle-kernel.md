# Upstream-sync bug oracle: kernel, freeldr, sdk/lib, and kd pairs

**Scope.** This review covers the 127 `(upstream commit, C file)` pairs in
`~/vibeos-rust-staging/upstream-oracle-merged/kernel-pairs.txt`. They come from the 1,269 upstream
commits merged in `28a26f3b664^1..28a26f3b664^2`. Every C file in the list is out of our build: it
is either cohab-flipped to Rust, or it is upstream-new and not compiled here. The merge therefore
landed each fix as dead C. For each pair, I read the upstream diff and compared it with the live
Rust twin. This was a read-only review: I built nothing, booted nothing, and edited no code.

How I checked liveness:
- Under `ntoskrnl/ntos.cmake`, every `io/iomgr`, `io/pnpmgr`, and `kd64` source is commented out,
  and the only live C left is `io_rust_shims.c`.
- The freeldr sources are commented out in `boot/freeldr/freeldr/{CMakeLists.txt,uefi.cmake,rosload.cmake}`.
- The relevant `sdk/lib/rtl/CMakeLists.txt` entries are commented out.
- `sdk/lib/{evtlib,cportlib,drivers/arbiter}` and `drivers/base/{kdgdb,kdvm}` each build a stub TU
  plus a Rust archive.
- **The BIOS loader no longer exists.** `boot/freeldr/freeldr/CMakeLists.txt:101` says "BIOS
  freeldr.sys was pruned". `machpc.rs:1907` gates the PC `MachInit` on `not(feature = "uefi")`, and
  uefildr links freeldr-rust with `["uefi", …]` (`uefildr-cargo/build.rs:215`). As a result, the PC
  modules (`pcvideo.rs`, `pcdisk.rs`, `hwdisk.rs`, `freeldr.rs` BootMain) are compiled into the
  UEFI archive but are unreachable. Findings in those modules are marked **dormant**.

## Status (2026-09-30)

**Ported:** 1-12, each re-checked against the upstream diff before porting. Item 1 also needed
the `halacpi.rs` saturating fix that the proposal names. Item 6's device-path walk takes no
`.unwrap()`; with no `HandleProtocol` it keeps the partition-1 fallback.

**Still owed:**
- 13, dormant until a BIOS loader exists.
- The out-of-list IOSB-without-SEH lead (`irp.rs:656/744`).

## Summary

| Verdict | Pairs |
|---|---:|
| CONFIRMED | 16 (2 of them dormant, BIOS-only) |
| ALREADY-FIXED | 5 |
| NOT-A-FIX | 106 |
| LIVE-C | 0 |
| NO-RUST-TWIN | 0 |
| INCONCLUSIVE | 0 |

The 14 live CONFIRMED pairs collapse into 12 distinct findings (#1–#12), because `e1853c66214`
spans two files and the two `device.c` lock commits form one finding. The 2 dormant pairs are #13. Most of the NOT-A-FIX volume comes from the
arbiter-library rewrite (`9ffc373bf62` → `b69ae8bdb87`, 33 pairs), from partition and disk
refactors, and from formatting commits. None of it has correctness content for us. The arbiter
rewrite is a feature we have not adopted: our `arbiter-rust` translates the pre-rewrite library, and
`pnpmgr-rust/src/arb/*.rs` calls that library through its own old API, so the code is consistent
with itself.

## CONFIRMED — detail and proposed diffs

The findings are ranked by impact, with boot-path and memory-corruption findings first. None of the
diffs below has been applied.

### 1. UEFI loader hands ACPI tables to the kernel as free memory (`e1853c66214`, uefimem.c and uefihw.c)

**What upstream fixed.** `UefiConvertToFreeldrDesc` maps `EfiACPIReclaimMemory` to
`LoaderFirmwareTemporary` and `EfiACPIMemoryNVS` to `LoaderReserve`. Upstream maps both to
`LoaderSpecialMemory`. The same commit makes `DetectAcpiBios` stop copying the EFI memory map into
`ACPI_BIOS_DATA.MemoryMap` as if it were E820 data: upstream sets `TableSize =
sizeof(ACPI_BIOS_DATA)` and `Count = 0`. The commit message says: "prevents bugchecks … due to the
ACPI table being overwritten".

**Our Rust.**
- `sdk/lib/rust/freeldr-rust/src/uefimem.rs:145`: `EfiACPIReclaimMemory => LoaderFirmwareTemporary`.
- `uefimem.rs:146`: `EfiACPIMemoryNVS => LoaderReserve`.
- `sdk/lib/rust/freeldr-rust/src/uefihw.rs:85-86,116-121`: still sizes the table as
  `FreeldrDescCount * sizeof(BIOS_MEMORY_MAP)`. It sets `Count = FreeldrDescCount` and byte-copies
  `EfiMemoryMap` (an array of `EFI_MEMORY_DESCRIPTOR`, 40+ bytes each) into 24-byte
  `BIOS_MEMORY_MAP` slots, which produces garbage E820 entries.

**Why it is the bug.** The kernel treats `LoaderFirmwareTemporary` as free memory:
`ntkrnl-sys/src/inline/mm.rs:32` (`MiIsMemoryTypeFree`). The pages that hold the RSDT, XSDT, DSDT,
and so on therefore join the PFN free list at MM init. `acpi.sys` (ACPICA) maps those tables by
physical address later, from the RSDT pointer it reads out of this same block (`acpi-rust/src/osl.rs:331`),
after the pages may already have been reused. OVMF places its tables in `EfiACPIReclaimMemory`, so
every UEFI boot of all three architectures takes this path.

**Impact.** Latent corruption of the ACPI tables on every UEFI boot. When it bites depends on
allocation pressure before `acpi.sys` parses the DSDT. I have **not** tied it to any observed flake.
The HAL also copies the multi-node: `hal-rust/src/x86/halacpi.rs:876-878`.

**Proposed diff.**
```diff
--- a/sdk/lib/rust/freeldr-rust/src/uefimem.rs
-        EfiACPIReclaimMemory => LoaderFirmwareTemporary,
-        EfiACPIMemoryNVS => LoaderReserve,
+        // upstream e1853c66214: ACPI tables must survive into the kernel — never "free".
+        EfiACPIReclaimMemory => LoaderSpecialMemory,
+        EfiACPIMemoryNVS => LoaderSpecialMemory,
--- a/sdk/lib/rust/freeldr-rust/src/uefihw.rs   (DetectAcpiBios)
-        TableSize = FreeldrDescCount * core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG +
-            core::mem::size_of::<ACPI_BIOS_DATA>() as ULONG - core::mem::size_of::<BIOS_MEMORY_MAP>() as ULONG;
+        // An EFI_MEMORY_DESCRIPTOR array is not an E820 map; publish none (upstream e1853c66214).
+        TableSize = core::mem::size_of::<ACPI_BIOS_DATA>() as ULONG;
 ...
-        (*AcpiBiosData).Count = (FreeldrDescCount) as u64 ;
-        core::ptr::copy_nonoverlapping(EfiMemoryMap as *const u8, …MemoryMap…, FreeldrDescCount * 24);
+        (*AcpiBiosData).Count = 0;
--- a/sdk/lib/rust/hal-rust/src/x86/halacpi.rs   (must land WITH the above)
-        + (node_count as usize - 1) * core::mem::size_of::<ACPI_E820_ENTRY>();
+        + (node_count as usize).saturating_sub(1) * core::mem::size_of::<ACPI_E820_ENTRY>();
```

**Why the `halacpi.rs` change must land with it.** Once `Count` is 0, the expression
`node_count as usize - 1` underflows. In C this wraps harmlessly (unsigned). It is also harmless in
our release build today, because `overflow-checks` follows `debug-assertions`, which is off. But
`Cargo.toml:430-452` stages turning `debug-assertions` on per crate, starting with the HAL, and then
the expression panics at HAL init. With `saturating_sub`, the copy is 0x28 bytes, which equals
`sizeof(ACPI_BIOS_DATA)` exactly, so it stays in bounds. `LoaderSpecialMemory` is already in scope
through `crate::frldr::*` (`frldr.rs:465`). The externs `EfiMemoryMap` and `FreeldrDescCount` in
`uefihw.rs:24-25` become unused and can be dropped.

### 2. VPB freed while still mounted (`28373a9f842`, volume.c)

**What upstream fixed.** `IopDereferenceVpbAndFree` freed a VPB whose reference count had reached
zero only while it was still installed, i.e. `RealDevice->Vpb == Vpb`. That leaves
`RealDevice->Vpb` dangling and leaks detached VPBs. Upstream reverses the test to `!=`.

**Our Rust.** `sdk/lib/rust/iomgr-rust/src/volume.rs:338` reads `&& (*(*Vpb).RealDevice).Vpb == Vpb`,
which is the pre-fix condition.

**Impact.** A use-after-free of a mounted volume's VPB. The function is called from the
failure paths of `IopParseDevice` (`file.rs:934,1009,1019,1096,1179,1245,1288,1321`) and from
`volume.rs:854,886`. When the last reference drops on a failed open, the live VPB is freed and the
next mount or parse dereferences freed pool. This is corruption on a boot-reachable path, and it
also leaks dismounted VPBs.

**Proposed diff.**
```diff
-        && (*(*Vpb).RealDevice).Vpb == Vpb
+        && (*(*Vpb).RealDevice).Vpb != Vpb   // upstream 28373a9f842: free only a DETACHED VPB
```

### 3. Device stack attach and lookup not done under the I/O database lock (`cea19d2e0f6`, `b364f702d2f`, device.c)

**What upstream fixed.**
- `IopAttachDeviceToDeviceStackSafe` now holds `LockQueueIoDatabaseLock` across the stack walk, the
  `AttachedDevice` and `AttachedTo` link-up, and the `*AttachedToDeviceObject` store. That store is
  the whole point of the `…Safe` API: the filter's `LowerDevice` must be set before IRPs can reach
  the filter.
- `IoGetAttachedDeviceReference` and `IoGetDeviceAttachmentBaseRef` now walk the stack and reference
  the result under the same lock.

**Our Rust.**
- `sdk/lib/rust/iomgr-rust/src/device.rs:276-308`: no lock.
- `device.rs:1268-1281`: no lock.

Other functions in the same file already use the idiom: `device.rs:490`, `736`, and `1226`, with
`LockQueueIoDatabaseLock = 0xA` at `:225`. None of the callers (`:922`, `:934`) holds the lock
already, so adding it cannot deadlock.

**Impact.** A race during PnP filter attach. It is the documented `IoAttachDeviceToDeviceStack`
gotcha, where an IRP reaches the filter before `LowerDevice` is set, so the filter dereferences
NULL. A second race lets `…Reference` return or reference a device that is being detached
concurrently. This matters most on SMP boots and during driver load or unload.

**Proposed diff.**
```diff
 unsafe fn IopAttachDeviceToDeviceStackSafe(…) -> PDEVICE_OBJECT {
+    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
     let mut AttachedDevice = IoGetAttachedDevice(TargetDevice);
     …
     if !AttachedToDeviceObject.is_null() {
         *AttachedToDeviceObject = AttachedDevice;
     }
+    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
     AttachedDevice
 }
 pub unsafe extern "system" fn IoGetAttachedDeviceReference(DeviceObject: PDEVICE_OBJECT) -> PDEVICE_OBJECT {
     let DeviceObject = opaque_ptr(DeviceObject);
+    let OldIrql = KeAcquireQueuedSpinLock(LockQueueIoDatabaseLock);
     let DeviceObject = IoGetAttachedDevice(DeviceObject);
     ObReferenceObject(DeviceObject as PVOID);
+    KeReleaseQueuedSpinLock(LockQueueIoDatabaseLock, OldIrql);
     DeviceObject
 }
 // IoGetDeviceAttachmentBaseRef: same three lines around IopGetLowestDevice + ObReferenceObject.
```

### 4. GPT boot disk never matched by the kernel, giving bugcheck 0x7B (`7779fb44597`, archwsup.c)

**What upstream fixed.** `AddReactOSArcDiskInfo` gains a `GptDiskGuid` parameter. It zeroes the
entry, sets `IsGpt`, and copies the GUID into `ARC_DISK_SIGNATURE.GptSignature`. `DiskInitialize`
passes the disk GUID for GPT disks. Without this, `IopVerifyDiskSignature` can never match a GPT
boot disk, `IopMarkBootPartition` fails, and the kernel bugchecks with 0x7B
INACCESSIBLE_BOOT_DEVICE.

**Our Rust.**
- `sdk/lib/rust/freeldr-rust/src/archwsup.rs:83-104`: the old three-value signature. It never
  touches `IsGpt` or `GptSignature`, and the static array stays zero.
- Both callers, `uefidisk.rs:887` (in `GetHarddiskInformation`) and `uefidisk.rs:1319`, pass only
  the MBR fields.
- The kernel side already implements the GPT branch: `iomgr-rust/src/arcname.rs:1326-1338` requires
  `(*ArcDiskSignature).IsGpt != 0`.

**Impact.** Booting an installed system on a GPT-partitioned disk, which is the normal layout for a
UEFI install, bugchecks with 0x7B. A protective MBR still has 0xAA55, so `ValidPartitionTable = 1`,
but `IsGpt = 0` and the GUID compare fails. The livecd smoke boots by CD-ROM ARC path, so it never
exercises this. Real-hardware boots from a GPT USB stick or disk would hit it.

**Proposed diff.**
```diff
--- archwsup.rs
 pub unsafe extern "C" fn AddReactOSArcDiskInfo(
-    ArcName: PSTR, Signature: u32, Checksum: u32, ValidPartitionTable: u8) {
+    ArcName: PSTR, GptDiskGuid: *const GUID, Signature: u32, Checksum: u32, ValidPartitionTable: u8) {
+    let e = &mut reactos_arc_disk_info[reactos_disk_count as usize];
+    core::ptr::write_bytes(e as *mut _ as *mut u8, 0, core::mem::size_of_val(e));
     …existing Signature/CheckSum/ValidPartitionTable stores…
+    e.DiskSignature.IsGpt = (!GptDiskGuid.is_null()) as u8;
+    if !GptDiskGuid.is_null() {
+        core::ptr::copy_nonoverlapping(GptDiskGuid as *const u8,
+            e.DiskSignature.GptSignature.as_mut_ptr() as *mut u8, 16);
+    }
--- uefidisk.rs GetHarddiskInformation (after Checksum/ValidPartitionTable are computed from the MBR,
    since UefiReadGptHeader reuses DiskReadBuffer):
+    let mut GptHeader: GPT_TABLE_HEADER = core::mem::zeroed();
+    let GptGuid: *const GUID = if UefiReadGptHeader(DriveNumber, &mut GptHeader) != 0 {
+        &GptHeader.DiskGuid as *const _ as *const GUID } else { core::ptr::null() };
-    AddReactOSArcDiskInfo(ArcName.as_mut_ptr(), Signature, Checksum, ValidPartitionTable);
+    AddReactOSArcDiskInfo(ArcName.as_mut_ptr(), GptGuid, Signature, Checksum, ValidPartitionTable);
```
The same change applies to the extern declarations at `uefidisk.rs:23` and `hwdisk.rs:76`. The
`uefidisk.rs:1319` (ramdisk) and `hwdisk.rs` callers pass `null()`.

### 5. `PiUpdateDeviceState` clears every devnode flag except one (`1b460291e26`, devaction.c)

**What upstream fixed.** The code wrote `DeviceNode->Flags &= DNF_NON_STOPPED_REBALANCE;` where it
meant `&= ~…`.

**Our Rust.** `sdk/lib/rust/pnpmgr-rust/src/devaction.rs:1815` reads
`(*DeviceNode).Flags &= DNF_NON_STOPPED_REBALANCE;`. The same file gets it right at `:2542`.

**Impact.** When a driver reports `PNP_DEVICE_RESOURCE_REQUIREMENTS_CHANGED | PNP_DEVICE_FAILED`,
this line wipes `DNF_STARTED`, `DNF_ENUMERATED`, `DNF_HAS_BOOT_CONFIG`, and every other flag on the
devnode. That corrupts the PnP state machine, although the path is rare.

**Proposed diff.**
```diff
-            (*DeviceNode).Flags &= DNF_NON_STOPPED_REBALANCE;
+            (*DeviceNode).Flags &= !DNF_NON_STOPPED_REBALANCE;
```

### 6. UEFI boot partition hard-coded to 1 for logical-partition boot handles (`7eab8fafec4`, uefidisk.c)

**What upstream fixed.** `UefiSetBootpath` used `FIRST_PARTITION` whenever the boot handle was a
logical partition. Upstream now walks the handle's `EFI_DEVICE_PATH_PROTOCOL` for a
`MEDIA_DEVICE_PATH`/`MEDIA_HARDDRIVE_DP` node and uses `HARDDRIVE_DEVICE_PATH.PartitionNumber`.

**Our Rust.** `sdk/lib/rust/freeldr-rust/src/uefidisk.rs:1212-1216` still sets
`BootPartition = FIRST_PARTITION;`.

**Impact.** A HDD or USB boot where the loader's partition is not partition 1 gets the wrong ARC
boot path, `…partition(1)`. The loader then reads `freeldr.ini` and the system from the wrong
volume. CD boots take the `cdrom(%u)` branch at `:1199` and are not affected.

**Proposed diff.** freeldr-rust has no device-path types yet; `frldr.rs:1375` holds the path as
`*mut c_void`. A minimal byte-level walk is enough:
```diff
         if (*(*BootBlockIo).Media).LogicalPartition != 0 {
-            BootPartition = FIRST_PARTITION;
+            BootPartition = FIRST_PARTITION; // fallback
+            static mut DevicePathGuid: EFI_GUID = EFI_DEVICE_PATH_PROTOCOL_GUID; // 09576e91-6d3f-11d2-8e39-00a0c969723b
+            let mut Node: *mut u8 = core::ptr::null_mut();
+            if HandleProtocol(*handles.add(UefiBootRootIndex as usize), addr_of_mut!(DevicePathGuid),
+                              &mut Node as *mut _ as *mut *mut c_void) == 0 && !Node.is_null() {
+                loop {   // node = {u8 Type, u8 SubType, u16 Length}
+                    let (t, st) = (*Node, *Node.add(1));
+                    let len = u16::from_le_bytes([*Node.add(2), *Node.add(3)]) as usize;
+                    if t == 0x7F && st == 0xFF { break; }        // END_ENTIRE_DEVICE_PATH
+                    if len < 4 { break; }                         // malformed
+                    if t == 0x04 && st == 0x01 {                  // MEDIA_DEVICE_PATH / MEDIA_HARDDRIVE_DP
+                        BootPartition = core::ptr::read_unaligned(Node.add(4) as *const u32); // PartitionNumber
+                        break;
+                    }
+                    Node = Node.add(len);
+                }
+            }
```

### 7. UEFI disk read buffer has no fallback when `AllocatePool` fails (`492aeb639c9`, uefidisk.c)

**What upstream fixed.** If `BootServices->AllocatePool` fails, upstream falls back to
`MmAllocateMemoryWithType(size + align, LoaderFirmwareTemporary)`. It records the fallback in
`DiskReadBufferFallbackPool` so that the reallocation path releases the buffer with `MmFreeMemory`
instead of `FreePool`.

**Our Rust.** `sdk/lib/rust/freeldr-rust/src/uefidisk.rs:187-203` (`UefiEnsureDiskReadBufferAligned`)
calls `FreePool` and, on failure, logs, returns 0, and has no fallback.

**Impact.** On EFI 1.x firmware (Intel Macs and similar), hardware detection fails and the loader
cannot read the disk. There is no effect on OVMF.

**Proposed diff.** Port the upstream hunk as written. Add
`static mut DiskReadBufferFallbackPool: BOOLEAN = 0;`. At `:189`, call `MmFreeMemory(DiskReadBufferRaw)`
when the fallback flag is set, and otherwise call `FreePool`, then clear the flag. At `:198-203`,
replace the early return with
`DiskReadBufferRaw = MmAllocateMemoryWithType(DiskReadBufferSize + RequiredAlignment as usize, LoaderFirmwareTemporary)`,
return 0 only if that is null, and otherwise set `DiskReadBufferFallbackPool = 1`.

### 8. `freeldr.ini` BootType leaks from one OS entry to the next (`81752ca2117`, oslist.c)

**What upstream fixed.** When an OS section exists but has no `BootType=` line,
`IniReadSettingByName` leaves the buffer untouched, so the previous entry's BootType is reused.
Upstream resets the buffer first.

**Our Rust.** `sdk/lib/rust/freeldr-rust/src/oslist.rs:138-139` calls `IniReadSettingByName` with
no reset first. `inifile.rs:158-185` confirms that the not-found path does not write `Buffer`.

**Impact.** A section without `BootType` inherits the wrong loader, for example `BootSector`
instead of the `Windows` default. Every section in our `livecd.ini` sets `BootType`, so this is
latent today.

**Proposed diff.**
```diff
         if HadSection != 0 {
+            *BootType.as_mut_ptr() = 0i8;   // upstream 81752ca2117
             IniReadSettingByName(SectionId, b"BootType\0"…);
```

### 9. INI key names keep trailing whitespace (`f77fcc15ffc`, parse.c)

**What upstream fixed.** `IniExtractSettingName` skipped leading whitespace but not trailing
whitespace, so `Key = value` produced the key `"Key "`, which then fails every `_stricmp` lookup.

**Our Rust.** `sdk/lib/rust/freeldr-rust/src/parse.rs:433-445` breaks on `'='` without trimming.

**Impact.** Any `freeldr.ini` or `txtsetup.sif` key written with a space before `=` is silently
ignored. Our shipped inis use `Key=value`, so this is latent.

**Proposed diff.**
```diff
         if (*SettingNameLine.offset(Idx as isize) == b'=' as i8) || (… == b'\0' as i8) {
+            while DestIdx > 0 {
+                let c = *SettingName.offset((DestIdx - 1) as isize);
+                if c == b' ' as i8 || c == b'\t' as i8 { DestIdx -= 1; } else { break; }
+            }
             break;
         }
```

### 10. Event logs preallocated to `MaxSize` (`24e904e0956`, evtlib.c)

**What upstream fixed.** A new log is created at 64 KiB granularity, and `MaxSize` becomes a
capacity cap. `ElfWriteRecord` then grows the file in 64 KiB steps, and `FileSetSize` failures are
propagated.

**Our Rust.**
- `sdk/lib/rust/evtlib-rust/src/evtlib.rs:575-577` (`ElfpInitNewFile`): sets
  `CurrentSize = MaxSize` and calls `FileSetSize(…, 0)` with the status ignored.
- `evtlib.rs:1405-1411` (`ElfWriteRecord`): grows the file straight to `MaxSize`.

**Impact.** Disk usage only. Every event log file is created at full `MaxSize` (often 512 KiB to
20 MiB), and a `FileSetSize` failure is ignored, so writes can go past the physical end of file. No
corruption on the boot path.

**Proposed diff.** Port the two upstream hunks into those two functions:
- Add `const EVENTLOG_FILE_GRANULARITY: u32 = 64 * 1024;`.
- In `ElfpInitNewFile`: compute
  `InitialSize = round_up_ulong(size_of::<EVENTLOGHEADER>() + size_of::<EVENTLOGEOF>(), GRAN)`, set
  `CurrentSize = InitialSize`, call `FileSetSize(LogFile, InitialSize, _FileSize)` and return its
  failure status, then set `MaxSize = round_up_ulong(max(MaxSize, InitialSize), GRAN)`.
- In `ElfWriteRecord`: compute
  `NewSize = WriteOffset + BufSize + size_of::<EVENTLOGEOF>()`. If
  `WriteOffset < EndOffset || NewSize > MaxSize`, use `NewSize = MaxSize`; otherwise round it up to
  the granularity. Grow only if `NewSize > CurrentSize`, and propagate the status.

### 11. `RtlNtStatusToDosError(STATUS_ILLEGAL_INSTRUCTION)` returns the wrong code (`e9a67ff1bd5`, error.c)

**What upstream fixed.** The `table_c0000001` entry for `c000001d` changes from
`ERROR_INVALID_FUNCTION` to `STATUS_ILLEGAL_INSTRUCTION`, which matches Windows and the
ntdll_winetest. Exception-class codes pass through unchanged, just like the `c0000025` entry already
does.

**Our Rust.** `sdk/lib/rust/rtl-error/src/tables.rs:265` (index 28, i.e. `c000001d`) is
`ERROR_INVALID_FUNCTION`. The index is verified: the table starts at `:237`, which is `c0000001`.

**Impact.** Win32 `GetLastError()` after this status reports 1 instead of `0xC000001D`. This is
user-visible but not a boot-path issue. The rtl-difftest reference C has now moved to the post-fix
table, so expect that harness to flag this entry.

**Proposed diff.**
```diff
--- tables.rs
-    ERROR_INVALID_FUNCTION,          // c000001d (STATUS_ILLEGAL_INSTRUCTION)
+    STATUS_ILLEGAL_INSTRUCTION,      // c000001d — upstream e9a67ff1bd5
--- winerr.rs (next to STATUS_NONCONTINUABLE_EXCEPTION at :694)
+pub const STATUS_ILLEGAL_INSTRUCTION: DWORD = 0xc000001d;
```

### 12. `IoFreeWorkItem` frees without a tag check (`0086992fe78`, iowork.c)

**What upstream fixed.** `ExFreePool` becomes `ExFreePoolWithTag(IoWorkItem, TAG_IOWI)`.

**Our Rust.** `sdk/lib/rust/iomgr-rust/src/iowork.rs:71` calls `ExFreePool`. `TAG_IOWI` is
already defined at `:22` and used for the allocation at `:80`.

**Impact.** Low, diagnostic only. Freeing a non-work-item pointer goes undetected instead of
bugchecking with BAD_POOL_CALLER at the faulting caller.

**Proposed diff.**
```diff
-    ExFreePool(IoWorkItem as PVOID);
+    ExFreePoolWithTag(IoWorkItem as PVOID, TAG_IOWI);
```

### 13. Dormant (BIOS-only, unreachable in our UEFI-only loaders)

These pairs are recorded because the Rust does carry the pre-fix code. They are not worth fixing
until a BIOS loader exists again.

- **`bd881161ac2` pcvideo.c (`PcVideoGetDisplayMode` regression).**
  `freeldr-rust/src/pcvideo.rs:307-313` still derives `BytesPerScanLine` from BDA byte `0x489`,
  so it holds a scan-line count (200/350/400) where it needs the bytes per text row
  (`ScreenWidth*2`). The `PcVideoSetMode80xNN` helpers also do not reset it. The fix is to use
  `BytesPerScanLine = ScreenWidth * 2` in the text-mode branch and in each `PcVideoSetMode80x*`.
- **`e87a74f1312` freeldr.c (`LaunchSecondStageLoader`).** `freeldr-rust/src/freeldr.rs:133-134`
  transmutes rosload's entry to `fn() -> LONG` and returns its value. `RunLoader`
  (`bootmgr.rs:394`) returns nothing, so the "result" is whatever is left in eax/rax, and a nonzero
  value shows "Unable to load second stage loader" (`freeldr.rs:196`). The fix is to call it as
  `fn()` and return `ESUCCESS`. Our UEFI path calls `RunLoader()` directly (`uefildr.rs:173`).

## INCONCLUSIVE

None.

## Out-of-list leads noticed while reading

These are not verdicts. Each needs its own check.

- **`irp.rs:656-662` and `:744-750`: IOSB written without SEH.** `IopCompleteRequest` writes
  `*(*Irp).UserIosb = IoStatus` (and the IOSB32 variant) with no fault protection. Both the pre-fix
  and post-fix C wrap this write in `_SEH2_TRY`. This is the same DUMMY_PSEH class that
  `irp.rs:612-619` already fixed for the buffer copy with `ntkrnl_sys::probe::safe_copy_raw`. A
  user process that frees its IOSB before async completion would turn this into a kernel fault.
- **`IoIs32bitProcess` is a stub.** `irp.rs:1747-1750` always returns FALSE. Upstream `6567d67b815`
  implements it. Our WoW64 IOSB32 handling uses its own `IRP_WOW64_IOSB32` design
  (`iofunc.rs:168-190`), so the IOSB half is covered. The API itself is still a feature gap for
  drivers that thunk IOCTLs.
- **`RtlFindRange` widening (`1500a353099`, rangelist.c).** Already matched: `rtl-rangelist/src/lib.rs:341-342`
  takes `ULONGLONG` Length and Alignment, and the spec is `int64`. Noted here so nobody re-checks it.
- **`4dabb9e5825` PCIX decode fixes.** These touch `drivers/bus/pcix/{utils,enum,device,ppbridge,arb/ar_memio}.c`,
  which are flipped to `pcix-rust` but are not in this list. pcix is not loaded at boot
  (`drivers/bus/pcix/CMakeLists.txt:11`), so they are low priority, but they belong to whoever owns
  the drivers list.

## Every pair

| # | Commit | C file | Verdict | Reason |
|---|---|---|---|---|
| 1 | 492aeb639c9 | arch/uefi/uefidisk.c | CONFIRMED | #7: no AllocatePool fallback (`uefidisk.rs:198-203`) |
| 2 | 3518663767d | sdk/lib/rtl/unicode_vista.c | NOT-A-FIX | Wine sync moved `RtlCompareUnicodeStrings`, same body; ours is in rtl-strings |
| 3 | e1853c66214 | arch/uefi/uefihw.c | CONFIRMED | #1: EFI memory map copied as E820 (`uefihw.rs:116-121`) |
| 4 | e1853c66214 | arch/uefi/uefimem.c | CONFIRMED | #1: ACPI reclaim handed out as FirmwareTemporary (`uefimem.rs:145`) |
| 5 | 1d3f99a87ad | arch/uefi/uefivid.c | NOT-A-FIX | Feature: Apple Graphics Info protocol |
| 6 | 0086992fe78 | io/iomgr/iowork.c | CONFIRMED | #12: untagged free (`iowork.rs:71`) |
| 7 | a204d174f05 | sdk/lib/cportlib/ns16550.c | NOT-A-FIX | Pure rename |
| 8 | dabc74d468b | sdk/lib/cportlib/cport.c | NOT-A-FIX | Split refactor; `TIMEOUT_COUNT` parenthesization has no effect (`cport-rust/src/lib.rs:75`) |
| 9 | dabc74d468b | sdk/lib/cportlib/ns16550.c | NOT-A-FIX | Split refactor |
| 10 | fe6ee1223ae | ntoskrnl/kdbg/kdb_cli.c | NOT-A-FIX | Indentation only (braces unchanged) |
| 11 | 8e8cd2e1f66 | lib/inifile/parse.c | NOT-A-FIX | Formatting / PCSTR |
| 12 | f77fcc15ffc | lib/inifile/parse.c | CONFIRMED | #9: trailing whitespace kept in key (`parse.rs:433-445`) |
| 13 | d012a8c4a31 | kdgdb/amd64_sup.c | NOT-A-FIX | Typo/format |
| 14 | d012a8c4a31 | kdgdb/gdb_input.c | NOT-A-FIX | Typo/format |
| 15 | d012a8c4a31 | kdgdb/i386_sup.c | NOT-A-FIX | Typo/format |
| 16 | d012a8c4a31 | kdgdb/kdpacket.c | NOT-A-FIX | Typo/format |
| 17 | 9669b3124f8 | kdgdb/amd64_sup.c | NOT-A-FIX | MSVC build fix (const) |
| 18 | 9669b3124f8 | kdgdb/gdb_input.c | NOT-A-FIX | MSVC build: `%p` to `PRIxPTR` in thread list; gdb parses both |
| 19 | 9669b3124f8 | kdgdb/i386_sup.c | NOT-A-FIX | MSVC build fix |
| 20 | 6e51f4abb09 | kdvm/kdvbox.c | NOT-A-FIX | Rename |
| 21 | 6e51f4abb09 | kdvm/kdvm.c | NOT-A-FIX | Rename |
| 22 | 9a7f64a0c64 | arch/uefi/uefihw.c | NOT-A-FIX | Feature: BGRT logo |
| 23 | 9a7f64a0c64 | arch/uefi/uefivid.c | NOT-A-FIX | Feature: BGRT logo |
| 24 | 5b7a4c948c1 | ui/minitui.c | NOT-A-FIX | Cosmetic colours |
| 25 | c25c527c254 | arch/i386/hwpci.c | NOT-A-FIX | Refactor (parameterize FindPciBios) |
| 26 | c25c527c254 | arch/i386/pc/machpc.c | NOT-A-FIX | Refactor |
| 27 | 9f42210478d | arch/i386/pc/pcdisk.c | NOT-A-FIX | DiskError modularization |
| 28 | 9f42210478d | arch/uefi/uefidisk.c | NOT-A-FIX | DiskError modularization |
| 29 | 62d71ebbd30 | arch/drivers/hwide.c | NOT-A-FIX | `CONTAINING_RECORD` vs cast; `P` is the first field (`hwide.rs:329-330`) |
| 30 | 56481e1b8d4 | arch/archwsup.c | NOT-A-FIX | Disk-registration refactor |
| 31 | 56481e1b8d4 | arch/i386/hwdisk.c | NOT-A-FIX | Refactor |
| 32 | 56481e1b8d4 | arch/uefi/uefidisk.c | NOT-A-FIX | Refactor |
| 33 | 56481e1b8d4 | arch/uefi/uefihw.c | NOT-A-FIX | Debug channel only |
| 34 | 56481e1b8d4 | lib/fs/fs.c | NOT-A-FIX | Added `RtlZeroMemory` of DEVICE is redundant: loader heap zero-fills (`heap.rs:414`) |
| 35 | e87a74f1312 | freeldr/freeldr.c | CONFIRMED (dormant) | #13: returns garbage rax (`freeldr.rs:133-134`); BIOS only |
| 36 | 03de1f932f7 | ntldr/advopts.c | NOT-A-FIX | Build fix for a file we do not compile (feature tracked in `rosload.cmake:41`) |
| 37 | 45687e82f3a | io/iomgr/device.c | NOT-A-FIX | Formatting |
| 38 | cea19d2e0f6 | io/iomgr/device.c | CONFIRMED | #3: attach without IoDatabaseLock (`device.rs:276-308`) |
| 39 | b364f702d2f | io/iomgr/device.c | CONFIRMED | #3: reference without IoDatabaseLock (`device.rs:1268-1281`) |
| 40 | 9a6dfe313d3 | io/iomgr/device.c | NOT-A-FIX | Macro readability |
| 41 | 9a6dfe313d3 | io/iomgr/deviface.c | NOT-A-FIX | Macro readability |
| 42 | 9a6dfe313d3 | io/iomgr/file.c | NOT-A-FIX | Macro readability; condition reorder is pure |
| 43 | 9a6dfe313d3 | io/iomgr/iotimer.c | NOT-A-FIX | Macro readability |
| 44 | 9a6dfe313d3 | io/pnpmgr/devnode.c | NOT-A-FIX | Macro readability |
| 45 | 9a6dfe313d3 | io/pnpmgr/plugplay.c | NOT-A-FIX | Macro readability |
| 46 | 474348fcb4d | io/iomgr/file.c | NOT-A-FIX | DPRINT text |
| 47 | 1490f34da2f | disk/part_brfr.c | NOT-A-FIX | Split, "no actual code changes" |
| 48 | 1490f34da2f | disk/part_mbr.c | NOT-A-FIX | Split |
| 49 | 1490f34da2f | disk/partition.c | NOT-A-FIX | Split |
| 50 | 00ed55bcbac | arch/uefi/uefidisk.c | NOT-A-FIX | GPT code moved to a module (body identical) |
| 51 | 00ed55bcbac | disk/part_gpt.c | NOT-A-FIX | Moved code |
| 52 | 00ed55bcbac | disk/partition.c | NOT-A-FIX | Moved code |
| 53 | 7779fb44597 | arch/archwsup.c | CONFIRMED | #4: `IsGpt`/`GptSignature` never set, giving 0x7B on GPT (`archwsup.rs:83-104`) |
| 54 | 9336919fec3 | kd64/kdinit.c | NOT-A-FIX | Banner print order |
| 55 | e2c92c0ffb7 | arch/i386/hwdisk.c | NOT-A-FIX | `PARTITION_INFORMATION` interface refactor |
| 56 | e2c92c0ffb7 | arch/i386/ntoskrnl.c | NOT-A-FIX | Interface refactor |
| 57 | e2c92c0ffb7 | arch/uefi/uefidisk.c | NOT-A-FIX | Interface refactor |
| 58 | e2c92c0ffb7 | disk/part_brfr.c | NOT-A-FIX | Interface refactor |
| 59 | e2c92c0ffb7 | disk/part_gpt.c | NOT-A-FIX | Interface refactor |
| 60 | e2c92c0ffb7 | disk/part_mbr.c | NOT-A-FIX | Interface refactor |
| 61 | e2c92c0ffb7 | disk/partition.c | NOT-A-FIX | Interface refactor |
| 62 | 07f36b52aaa | lib/peloader.c | NOT-A-FIX | Nits / comments |
| 63 | 68312dedbec | arch/uefi/uefidisk.c | NOT-A-FIX | Removes an unused global |
| 64 | 45aa3c2a17d | freeldr/oslist.c | NOT-A-FIX | ERR changed to TRACE for an unkeyed entry |
| 65 | 81752ca2117 | freeldr/oslist.c | CONFIRMED | #8: stale BootType (`oslist.rs:138-139`) |
| 66 | f00e1cd6aa2 | io/pnpmgr/pnpmgr.c | ALREADY-FIXED | No free on alloc failure (`pnpmgr.rs:704`) |
| 67 | 15fa25299f6 | sdk/lib/rtl/product.c | NOT-A-FIX | Feature (`RtlGetProductInfo`, 0x600+ export; known gap) |
| 68 | e9a67ff1bd5 | sdk/lib/rtl/error.c | CONFIRMED | #11: `c000001d` entry (`tables.rs:265`) |
| 69 | c8061316750 | sdk/lib/rtl/slist.c | NOT-A-FIX | Feature (`RtlInterlockedPushListSListEx`) |
| 70 | 57a7c56d51a | io/pnpmgr/pnpmap.c | NOT-A-FIX | PC-98 IDs gated to x86; extra table rows inert on amd64/arm64 (`pnpmap.rs:136,152`) |
| 71 | 2f514a5127f | arch/i386/hwdisk.c | ALREADY-FIXED | Rust hard-codes 2048 for CD-ROM (`hwdisk.rs:143-147`); also BIOS-only |
| 72 | 24e904e0956 | sdk/lib/evtlib/evtlib.c | CONFIRMED | #10: preallocates `MaxSize` (`evtlib.rs:575-577,1405-1411`) |
| 73 | 9de4cd4cf29 | sdk/lib/evtlib/evtlib.c | NOT-A-FIX | MSVC macro expansion |
| 74 | ed51a5d1e22 | sdk/lib/rtl/process.c | NOT-A-FIX | Moved `RtlGetCurrentProcessorNumber` to processor.c (gap already in merge-decisions) |
| 75 | 1b460291e26 | io/pnpmgr/devaction.c | CONFIRMED | #5: `&=` without `!` (`devaction.rs:1815`) |
| 76 | 379e6bb05f3 | sdk/lib/rtl/wow64.c | NOT-A-FIX | Feature (`RtlWow64GetProcessMachines`) |
| 77 | 9ffc373bf62 | io/pnpmgr/arb/arbbus.c | NOT-A-FIX | Arbiter rewrite (not adopted) |
| 78 | 9ffc373bf62 | io/pnpmgr/arb/arbdma.c | NOT-A-FIX | Arbiter rewrite |
| 79 | 9ffc373bf62 | io/pnpmgr/arb/arbirq.c | NOT-A-FIX | Arbiter rewrite |
| 80 | 9ffc373bf62 | io/pnpmgr/arb/arbmem.c | NOT-A-FIX | Arbiter rewrite |
| 81 | 9ffc373bf62 | io/pnpmgr/arb/arbport.c | NOT-A-FIX | Arbiter rewrite |
| 82 | 9ffc373bf62 | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Arbiter rewrite |
| 83 | ccc242ed6fe | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Build repair |
| 84 | d2b688a5ca8 | sdk/lib/rtl/name.c | NOT-A-FIX | Feature (`RtlIsNameInExpression`; fsrtl side in bootpath-triage) |
| 85 | 28373a9f842 | io/iomgr/volume.c | CONFIRMED | #2: frees the mounted VPB (`volume.rs:338`) |
| 86 | 6567d67b815 | io/iomgr/irp.c | NOT-A-FIX | Feature (`IoIs32bitProcess`, WoW64 IOSB); see leads |
| 87 | dce72d2c5c5 | arch/i386/pc/pcvideo.c | NOT-A-FIX | Comment / return-path tidy |
| 88 | bd881161ac2 | arch/i386/pc/pcvideo.c | CONFIRMED (dormant) | #13: `0x489` scan-line regression (`pcvideo.rs:307-313`); BIOS only |
| 89 | 064df1779f3 | io/iomgr/volume.c | NOT-A-FIX | String-init simplification |
| 90 | d364e2fb4a9 | io/pnpmgr/arb/arbbus.c | NOT-A-FIX | Feature (new arbiters) |
| 91 | d364e2fb4a9 | io/pnpmgr/arb/arbdma.c | NOT-A-FIX | Feature |
| 92 | d364e2fb4a9 | io/pnpmgr/arb/arbirq.c | NOT-A-FIX | Feature |
| 93 | 4c4e341d923 | io/pnpmgr/arb/arbirq.c | NOT-A-FIX | Whitespace only |
| 94 | 4c4e341d923 | io/pnpmgr/arb/arbmem.c | ALREADY-FIXED | Rust passes `CmResourceTypeMemory` (`arb/arbmem.rs:69`) |
| 95 | 4c4e341d923 | io/pnpmgr/arb/arbport.c | ALREADY-FIXED | Rust passes `CmResourceTypePort` (`arb/arbport.rs:73`) |
| 96 | ed7adce14c6 | sdk/lib/rtl/critical.c | NOT-A-FIX | Feature; Rust is a no-op stub like the pre-fix C (`rtl-sync/src/critical.rs:460`) |
| 97 | d0e71de116a | io/pnpmgr/arb/arbbus.c | NOT-A-FIX | Arbiter OrderingList feature / renames |
| 98 | d0e71de116a | io/pnpmgr/arb/arbdma.c | NOT-A-FIX | Same |
| 99 | d0e71de116a | io/pnpmgr/arb/arbirq.c | NOT-A-FIX | Same |
| 100 | d0e71de116a | io/pnpmgr/arb/arbmem.c | NOT-A-FIX | Same |
| 101 | d0e71de116a | io/pnpmgr/arb/arbport.c | NOT-A-FIX | Same |
| 102 | d0e71de116a | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Same |
| 103 | d0e71de116a | sdk/lib/drivers/arbiter/ordering.c | NOT-A-FIX | New file, not built |
| 104 | 7eab8fafec4 | arch/uefi/uefidisk.c | CONFIRMED | #6: partition hard-coded to 1 (`uefidisk.rs:1212-1216`) |
| 105 | c7e1c79921f | arch/i386/pc/pcvideo.c | NOT-A-FIX | SAL annotation addendum to bd881161ac2 |
| 106 | a6524ef7386 | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Arbiter range logic (feature) |
| 107 | dbdf6845613 | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Arbiter transactions (feature) |
| 108 | dbdf6845613 | sdk/lib/drivers/arbiter/transaction.c | NOT-A-FIX | New file, not built |
| 109 | 1500a353099 | io/pnpmgr/arb/arbbus.c | NOT-A-FIX | Root-arbiter feature |
| 110 | 1500a353099 | io/pnpmgr/arb/arbdma.c | NOT-A-FIX | Root-arbiter feature |
| 111 | 1500a353099 | io/pnpmgr/arb/arbmem.c | NOT-A-FIX | Root-arbiter feature |
| 112 | 1500a353099 | io/pnpmgr/arb/arbport.c | NOT-A-FIX | Root-arbiter feature |
| 113 | c7a64abb535 | arch/i386/hwpci.c | NOT-A-FIX | Reformat / SAL |
| 114 | b82afa1129a | io/pnpmgr/arb/arbdma.c | NOT-A-FIX | Newline format |
| 115 | b69ae8bdb87 | sdk/lib/drivers/arbiter/arbiter.c | NOT-A-FIX | Arbiter library completion (feature) |
| 116 | b69ae8bdb87 | sdk/lib/drivers/arbiter/entry.c | NOT-A-FIX | New file, not built |
| 117 | b69ae8bdb87 | sdk/lib/drivers/arbiter/handler.c | NOT-A-FIX | New file, not built |
| 118 | b69ae8bdb87 | sdk/lib/drivers/arbiter/ordering.c | NOT-A-FIX | New file, not built |
| 119 | b69ae8bdb87 | sdk/lib/drivers/arbiter/transaction.c | NOT-A-FIX | New file, not built |
| 120 | ffa4164e70d | io/pnpmgr/pnpinit.c | NOT-A-FIX | Disables upstream's new arbiters; ours are the old library and boot fine (`pnpinit.rs:396-397`) |
| 121 | f83eb2fba71 | ui/tui.c | NOT-A-FIX | Feature (Red Screen of Hell popup) |
| 122 | 8d4705fcea9 | sdk/lib/rtl/bootdata.c | ALREADY-FIXED | Three new BSD items present (`rtl-bootdata/src/lib.rs:69-71`) |
| 123 | a97929d2e61 | arch/hwpci.c | NOT-A-FIX | Code move (new file) |
| 124 | a97929d2e61 | arch/i386/hwpci.c | NOT-A-FIX | Code move |
| 125 | a97929d2e61 | arch/i386/pc/machpc.c | NOT-A-FIX | Code move |
| 126 | a97929d2e61 | arch/uefi/uefihw.c | NOT-A-FIX | Code move / prep for UEFI PCI detection |
| 127 | 4dabb9e5825 | sdk/lib/drivers/arbiter/handler.c | NOT-A-FIX | New arbiter file, not built (pcix half: see leads) |
