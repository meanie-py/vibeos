# config/ Rust port debug status (2026-04-30)

## Where we are

Branch `wip-config-rust-port` at `e3c3cd4f` (4 fixes committed). Build clean
both arches, 0 unexpected ud2. Boot reaches PsLocateSystemDll on i386, then
fails with BUGCHECK 0x6B (`PROCESS1_INITIALIZATION_FAILED, 0xC000003A,
0x2`) because `\SystemRoot\System32\ntdll.dll` cannot be opened.

## Fixes shipped (this session)

Four registry/translation bugs were committed:

1. **`cmvalche.rs:144` — bindgen flexible-array indexing panic**
   `(*CellData).u.KeyList[Index]` panicked because bindgen typed `KeyList`
   as `[HCELL_INDEX; 1]` (it's a flexible array in C). Replaced with raw
   pointer arithmetic. (Commit `a4c169b6`.)

2. **`cmconfig.rs` — `CmpMultifunctionTypes` zero-length array indexing**
   Declared as `[CMP_MF_TYPE; 0]` and indexed at runtime, panicking on
   bounds check. Switched to single-element `static mut` + raw pointer
   `.add(i)` arithmetic. (Commit `a4c169b6`.)

3. **`cmparse.rs::CmpDoOpen` — wrong `STATUS_REPARSE` constant**
   Three returns used `0x00000021` instead of `0x00000104`. This silently
   masked symbolic-link reparse, so opening
   `\Registry\Machine\System\CurrentControlSet` returned the volatile
   master-hive symlink itself instead of following to ControlSet001 in
   the SYSTEM hive. Downstream `Control` etc. creates under that handle
   then hit the volatile-master-hive create-rejection path
   → STATUS_INVALID_PARAMETER → BUGCHECK 0x67/0x6E. (Commit `a4c169b6`.)

4. **`cmapi.rs::CmEnumerateKey` — registry-lock recursion leak**
   The C uses `goto Quickie;` to consolidate cleanup
   (`HvReleaseFreeCellRefArray + CmpReleaseKcbLock + CmpUnlockRegistry`).
   The Rust translation inlined every `goto Quickie;` as a bare
   `return Status;` and forgot the trailing Quickie cleanup block. Every
   path leaked one shared registry-lock acquire. Boot driver init iterates
   this function many times during ServiceGroupOrder + per-driver subkey
   walks, accumulating ~88 unreleased shared acquires before
   `CmGetSystemDriverList → CmpLockRegistryExclusive` deadlocked the boot
   thread against itself. Wrapped body in `'Quickie: loop` matching
   cmapi.c:1811-1817. (Commit `e3c3cd4f`.)

## After these fixes — boot timeline on i386

Now reaches:
- All registry init (CmInitSystem1, CmpInitializeHardwareConfiguration)
- IopInitializePlugPlayServices (creates Control / Enum / Root keys)
- IopInitializeBootDrivers + per-builtin DriverEntry (24 drivers:
  sacdrv, wdf01000, acpi, pci, isapnp, partmgr, usb*, mountmgr, uniata,
  buslogic, usbstor, disk, cdrom, ksecdd, cdfs, ndis, swenum, ramdisk, mup)
- IopReinitializeBootDrivers (done)
- IopCreateArcNames → IopCreateArcNamesDisk (C) + IopCreateArcNamesCd (Rust)
- IopMarkBootPartition (failed — tolerated for WinPE)
- IopInitializeSystemDrivers — system drivers attempted to load via
  `\SystemRoot\…` but all fail with `STATUS_OBJECT_PATH_NOT_FOUND`
- IopReassignSystemRoot — runs but `\ArcName\multi(0)disk(0)cdrom(96)`
  symlink doesn't exist → returns "FALSE-as-NTSTATUS" (which the caller
  treats as success — quirk shared with C)
- IoAssignDriveLetters
- PsLocateSystemDll — `\SystemRoot\System32\ntdll.dll` open fails
- BUGCHECK 0x6B

## Current symptom — CDROM not detected → no ARC name → no SystemRoot

`IopCreateArcNamesCd` (in `arcname.rs`) finds the matching
`ArcDiskSignature` for `multi(0)disk(0)cdrom(96)` in the loader block, but
when querying `IoGetConfigurationInformation()->CdRomCount` it gets **0**
— the IO subsystem doesn't see any CDROM device. With `CdRomCount=0` and
`EnabledDisks=0`, the function returns success without creating any
`\ArcName\multi(0)disk(0)cdrom(96)` symlink.

`IopReassignSystemRoot` then opens that ARC symlink → fails with
`STATUS_OBJECT_NAME_NOT_FOUND` → returns "FALSE" → caller treats as
success (because the function's NTSTATUS-returning-BOOLEAN convention
makes 0 = success).

`\SystemRoot` is never created with the boot device path. All subsequent
file opens via `\SystemRoot\…` fail with `STATUS_OBJECT_PATH_NOT_FOUND`.

## Why CDROM is not detected — open question

The CDROM driver `cdrom.sys` was loaded (DriverEntry ran) per the BLT
trace. But somehow no CDROM device shows up in
`IoGetConfigurationInformation()`.

`continue-rust-port` (no Rust config) boots the same LiveCD ISO and
detects CDROMs correctly. So one of the recent Rust translations broke
PnP enumeration of the CDROM, or broke the registry-driven matching that
hooks cdrom.sys onto the IDE device.

Likely candidates:
- A registry value lookup during PnP that returns wrong data due to a
  translation bug we haven't found yet.
- The STATUS_REPARSE fix changed control flow somewhere that's now
  reading the wrong subkey.
- A call into `IopGetRegistryValue` from PnP that returns success but
  with garbage data (LLM-translation-bug class).

## Next steps

1. **Audit `IopFetchConfigurationInformation`** + the registry walk that
   PnP uses to discover devices. Specifically the path from the IDE
   controller PDO → cdrom.sys binding.

2. **Check `cdrom.sys` DriverEntry on this build** — does it actually
   create a `\Device\CdRom0` device object? Trace the AddDevice / bus
   notification path.

3. **Once CDROM detection works**, the chain unblocks:
   IopCreateArcNamesCd creates the symlink → IopReassignSystemRoot
   succeeds → \SystemRoot resolves → ntdll.dll opens → smss/csrss/winlogon
   start → smoke marker `shell ready`.

4. **Audit the IopReassignSystemRoot return convention** — both C and
   Rust return `FALSE = 0` on failure, which is treated as
   `NT_SUCCESS()`. This is a long-standing C bug. Probably outside the
   scope of this branch but should be flagged: failures here go silently
   undetected.

## Diagnostic state currently in tree (uncommitted)

The following files have debug-only traces. Do not commit; revert once
root cause is found:

- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/iomgr.rs` — `io_trace()` helper +
  `[IO]` traces around each step of `IopInitSystem` / `IopFinishIoInitSystem`.
- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/driver.rs` — `[SYSDRV]` traces
  in `IopInitializeSystemDrivers`, `[BLT]` per-builtin-driver trace.
- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/arcname.rs` — `[CDARC]` and
  `[REASSIGN]` traces.
- `sdk/lib/rust/ntoskrnl-rust/src/pnpmgr/pnpinit.rs` — `[PNP]` traces.
- `sdk/lib/rust/ntoskrnl-rust/src/config/cmsysini.rs` — `[CMGSDL]`
  traces in `CmGetSystemDriverList`, plus per-call `[LOCK]` / `[UNLK]`
  counter with `RtlGetCallersAddress` decoding (no longer needed since
  the lock leak was found).

Remove these once the CDROM-detection bug is identified. Some of them
(especially the per-builtin-driver trace in driver.rs and the [PNP]
traces) are useful diagnostic surface to keep in some form for the next
PnP-related debugging session.

## Memory notes consulted

- `feedback_cgu1.md` — LLVM captures(none) regression.
- `feedback_extern_c_abi.md` — extern "C" vs "system" ABI; perennial.
- `feedback_translation_philosophy.md` — never revert .rs to C; never
  delete .rs without approval.
- `feedback_flexible_array.md` — bindgen flexible arrays must use pointer
  arithmetic, not indexing (newly relevant for this session — three of
  the four committed bugs were of this class).
