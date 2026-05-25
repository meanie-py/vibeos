# Handoff: i386+amd64 BUGCHECK 0x6B SMSS init / IopCreateArcNames

**Audience:** Codex (or any agent picking up this kernel-stage debug).
**Branch:** `continue-rust-port` (freeldr-rust-port just merged in via
commit `e450a4332ee`).
**Status:** arm64 smoke GREEN (5+5skip, unchanged). i386 + amd64 boot
**all the way through freeldr + boot-driver init** but BUGCHECK 0x6B at
the SMSS/user-mode handoff. This doc is everything needed to pick the
debug up cold.

---

## TL;DR

Three boot-loader bugs were fixed today and committed; they unlocked
~22 kernel boot drivers init on i386+amd64. The current blocker is
purely kernel-side and arch-independent:

```
[IO] IopCreateArcNames
[ARC] Disk st=0 found=0                      ← Disk pass succeeded, no boot device
[CDARC] enter ArcBootDeviceName=multi(0)disk(0)cdrom(96)
[CDARC] matched signature, CdRomCount=0 EnabledDisks=0   ← no CD-ROM PDOs
[ARC] Cd st=c0000034                         ← STATUS_OBJECT_NAME_NOT_FOUND
[IO] IopCreateArcNames FAILED
[IO] IopMarkBootPartition FAILED
... SMSS starts, ZwOpenFile(\SystemRoot\System32\ntdll.dll) → STATUS_OBJECT_PATH_NOT_FOUND
*** BUGCHECK: 0x0000006B (0xc000003a, 0x00000002, 0x00000000, 0x00000000)
```

Root cause: the kernel reaches `IopCreateArcNames` (which would create
the `\SystemRoot` symbolic link) **before any storage / CD-ROM device
has been enumerated**. `IopCreateArcNamesCd` walks
`\Device\CdRom*` looking for a signature match and finds nothing
because PnP hasn't enumerated those PDOs yet (or never will). Without
`\SystemRoot`, `PsLocateSystemDll` can't open ntdll.dll →
`KeBugCheckEx(PROCESS1_INITIALIZATION_FAILED, STATUS_OBJECT_PATH_NOT_FOUND, 2, 0, 0)`
at `sdk/lib/rust/ntoskrnl-rust/src/ps/psmgr.rs:298`.

The bug is **not** in freeldr, **not** in WDF, **not** arch-specific
(both i386 and amd64 hit it at the same trace point). It is in the
ntoskrnl/iomgr/PnP region. arm64 doesn't hit it because arm64 boots
from a UEFI ramdisk and never goes through `IopCreateArcNames` (uses
a different SystemRoot resolution path via the ramdisk device).

---

## Reproduce

```bash
# from repo root
.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch i386 \
    --timeout 300 --skip-clean-build -k boot_and_serial
# or
.venv/bin/python -m pytest tests/qemu/test_smoke.py -v --arch amd64 \
    --timeout 300 --skip-clean-build -k boot_and_serial
```

Both will time out at 300s waiting for `[EXPLORER] _tWinMain entered`.
The captured serial log lives at
`/tmp/pytest-of-curi/pytest-*/qemu-{i386,amd64}*/` and the debugcon
trace at the same path. The smoke harness auto-decodes the trace
catalog on failure and prints a triage report; the BUGCHECK line
above appears in `qemu/i386_serial.log` (or `amd64_serial.log`)
when running QEMU directly.

For a fast manual repro (≈90s):

```bash
cd /home/curi/Projects/os-playground/vibeos
rm -f qemu/i386_dbgc.bin qemu/i386_serial.log
qemu-system-i386 -M pc -accel kvm -cpu host -m 512M -vga std \
  -cdrom build-i386/livecd.iso -display none -no-reboot \
  -serial file:qemu/i386_serial.log -debugcon file:qemu/i386_dbgc.bin &
sleep 120 ; pkill qemu-system-i386
grep -E "BLT|BUGCHECK|ARC|SMSS" qemu/i386_serial.log
```

---

## What today's commits did

(merge from freeldr-rust-port — see `git log e450a4332ee --first-parent`
or `git log a676cc3af88..974bd67b754` for the three direct fixes.)

### Commit `4a8fe8ea499` — PartialResourceList rustconv defects (x5)
- amd64 freeldr heap-corruption shortly after rosload's UiInitialize.
- 5 sites where `write_bytes::<T>(ptr, 0, 1)` zeroed `sizeof(*T)` bytes
  past an 8-byte allocation; fixed by casting to `*mut u8` and passing
  the byte count.
- Memory: `feedback_partial_resource_list_write_bytes.md`.

### Commit `94deccf52e9` — machpc.rs ABI mismatches (x3)
- i386 freeldr crashed `EIP=0x10006 INVALID_OPCODE` right after rosload's
  `UiDisplayMenu` epilog.
- `NtLdrGetOption`/`NtLdrGetOptionEx` were declared `extern "system"`
  (stdcall) but the impl is cdecl per `freeldr.spec` →
  each call leaked 8 bytes of ESP, eventually corrupting saved EBP/RIP.
- `CpDoesPortExist`/`Rs232PortInUse` had the OPPOSITE mismatch
  (declared cdecl, actual stdcall) — leaked the other direction.
- `PcHwDetect` needed `#[cfg(not(feature = "uefi"))]` so the linker
  doesn't pull BIOS-only deps into uefildr.efi via CGU sharing.
- Memory: `feedback_machpc_calling_convention.md`.

### Commit `974bd67b754` — freeldr peloader (the big one)
Two independent fixes bundled because they were diagnosed together:

1. **`LoaderBootDriver = 2` → `11`** in
   `sdk/lib/rust/freeldr-rust/src/peloader.rs:338`. rustconv extracted
   the wrong literal; `2` is `LoaderFree` (per `frldr.rs:411` and
   `ntkrnl-sys/src/bindgen_*.rs::_TYPE_OF_MEMORY_LoaderFree`).
   Symptom: every driver allocation marked pages as Free → next
   allocation reused the same range → driver images collided at the
   same `pa2=0xE7E000` → import-table scan saw NULL export directory
   for the clobbered prior driver → `WinLdrLoadDeviceDriver` returned
   FALSE → `UiMessageBox("Cannot load boot driver '%wZ'!")` (modal)
   hung the smoke harness.
   Memory: `feedback_freeldr_ldr_boot_driver_constant.md`.

2. **`PELDR_IMPORTDLL_LOAD_CALLBACK` ABI fix**. Rust type alias in
   `ffi.rs:191` was `extern "C"` (cdecl) but the C typedef is
   `(NTAPI *)` = stdcall on i386. Each invocation leaked 8 bytes of
   ESP, then the next `PeLdrLoadImage(FullDllName, ...)` push-args
   wrote OVER caller's stack-local buffer — the literal `1u8`
   `KernelMapping` from the `PeLdrLoadImage → PeLdrLoadImageEx`
   wrapper landed at `FullDllName[0]`, turning
   `\reactos\system32\bootvid.dll` into `\x01\0`. `ArcOpen` then
   returned `EINVAL`.
   Also fixed the `NtLdrImportDllLoadCallback` impl in `winldr.rs:120`
   from `extern "C"` to `extern "system"`.
   Memory: `feedback_freeldr_peldr_callback_ntapi.md`.

### Commit `8f69eadcc07` — WDF demand-start workaround
- After the freeldr fixes, the kernel reached
  `IopInitializeBuiltinDriver: wdf01000.sys` and BUGCHECK'd 0x1E
  (`ACCESS_VIOLATION` at `wdfldr+0x259D = _GetNameFromPath`).
- Root: `wdfldr.sys` was loaded only as a PE dependency of `Wdf01000`,
  never as a registered boot service, so its `DriverEntry` never ran
  and `WdfLdrGlobals.LoadedModulesList` was zero-initialized. The
  first list walk in `FindLibraryByServicePathLocked` dereferenced NULL.
- Tried registering wdfldr in an earlier service group; wdfldr's own
  `DriverEntry` then hung indefinitely (separate, unsolved bug).
- Workaround: defer both `Wdf01000` and new `WdfLdr` to Start=3
  (Demand) so neither runs at boot. The kernel skips them entirely and
  moves on to the next driver.

### Commit `a676cc3af88` — wdfldr lazy init
- After wdf01000 was demand-started, `cdrom.sys` (and other class
  drivers) still imported `WdfVersionBind` from wdfldr and called it
  from their own `DriverEntry`. `WdfLdrGlobals` was still uninitialized
  → same crash, now triggered by cdrom.
- Fix: `DllInitialize(NULL)` lazy-call at the top of `WdfVersionBind`.
  `DllInitialize` gates on `gAlreadyInitialized`, so subsequent calls
  are no-ops. Idempotent and safe.

After all five, both i386 and amd64 initialize **22 boot drivers**
(acpi, pci, isapnp, partmgr, usbccgp, usbehci, usbhub, usbohci,
usbuhci, mountmgr, uniata, buslogic, usbstor, disk, cdrom, ksecdd,
nmidebug, cdfs, ndis, swenum, ramdisk, mup) and then hit the SMSS
init BUGCHECK described below.

---

## Current blocker — the actual handoff

### Symptom

```
[BLT] IopInitializeBuiltinDriver: mup.sys       ← last boot driver
[IO] IopInitializeBootDrivers done
[IO] IopReinitializeBootDrivers done
[IO] IopCreateArcNames
[IO] IopCreateArcNames FAILED                   ← (you have to add the [ARC] markers below to see why)
[IO] IopMarkBootPartition FAILED
[SYSDRV] PiPerformSyncDeviceAction done
[CMGSDL] CmpLockRegistryExclusive done
[CMGSDL] CmpFindDrivers done
*** BUGCHECK: 0x0000006B (0xc000003a, 0x00000002, 0x00000000, 0x00000000)
```

Args decoded:
- `0x6B` = `PROCESS1_INITIALIZATION_FAILED`
- `0xc000003a` = `STATUS_OBJECT_PATH_NOT_FOUND`
- `2` = third call site in `PsLocateSystemDll`
  (`sdk/lib/rust/ntoskrnl-rust/src/ps/psmgr.rs:298`, the
  `ZwOpenFile(\SystemRoot\System32\ntdll.dll)` step)

### Diagnosis already done

Temporarily add these two `dbg_print!` markers to
`sdk/lib/rust/ntoskrnl-rust/src/iomgr/arcname.rs` around line 590
(then `touch arcname.rs`,
`rm -rf build-i386/ntoskrnl/{rust-kernel-target,libkernel_rust.a,ntoskrnl.exe}`,
`ninja livecd` to force a real kernel rebuild — naked `ninja livecd`
will say "no work to do" because the cargo invocation isn't a ninja
dependency):

```rust
Status = IopCreateArcNamesDisk(LoaderBlock, SingleDisk, &mut FoundBoot);
crate::dbg_print!(b"[ARC] Disk st=%x found=%x\r\n\0",
    Status as usize, FoundBoot as usize);
if Status >= 0 && FoundBoot == 0 {
    Status = IopCreateArcNamesCd(LoaderBlock);
    crate::dbg_print!(b"[ARC] Cd st=%x\r\n\0", Status as usize);
}
```

That produces (already captured this session, before reverting):

```
[ARC] Disk st=0 found=0           ← disk pass returned SUCCESS, no boot device matched
[CDARC] enter ArcBootDeviceName=multi(0)disk(0)cdrom(96)
[CDARC] matched signature, CdRomCount=0 EnabledDisks=0
[ARC] Cd st=c0000034              ← STATUS_OBJECT_NAME_NOT_FOUND
```

`CdRomCount=0 EnabledDisks=0` is the smoking gun: at the moment
`IopCreateArcNamesCd` runs, **no `\Device\CdRom*` symlinks/PDOs exist
in the object namespace**. The CD-ROM enumeration path hasn't
materialized any devices.

### Why this is the right diagnostic

The `[CDARC]` markers are pre-existing in
`sdk/lib/rust/ntoskrnl-rust/src/iomgr/arcname.rs::IopCreateArcNamesCd`
(search for `enter ArcBootDeviceName`); the function explicitly walks
the CD-ROM list, finds nothing, and returns
`STATUS_OBJECT_NAME_NOT_FOUND`. The matched-signature line means the
LoaderBlock's ARC disk signature list does contain an entry for the
CD-ROM the loader booted from — i.e., freeldr correctly told the
kernel "you came in via cdrom(96)", but PnP/the CD class driver hasn't
made a CD-ROM device object out of it.

### Why arm64 doesn't hit it

Arm64 boots via a UEFI ramdisk (`qemu/boot.img` + `qemu/ramdisk.img`),
not a CD-ROM, and the ramdisk driver creates `\SystemRoot` directly
via a different path. The `IopCreateArcNames`/cdrom flow is i386+amd64
LiveCD-only.

### The actual investigation to do

1. **Where do CD-ROM PDOs come from on i386/amd64 boot?** Probably
   `cdrom.sys` (class driver) binding to PDOs created by the ATAPI
   controller driver (`uniata.sys`, which IS in the loaded boot driver
   list — `[BLT] IopInitializeBuiltinDriver: uniata.sys` fires). The
   chain is roughly: `pci.sys` enumerates the PCI bus → finds the
   IDE controller → `uniata.sys` AddDevice creates child PDOs for each
   ATAPI device → `cdrom.sys` AddDevice on each PDO that looks like
   a CD-ROM. If ANY of those steps fail to call
   `IoCreateDevice` / `IoRegisterDeviceInterface` properly, the
   `\Device\CdRom0` etc. symlinks never appear.

2. **When is `IopCreateArcNames` called relative to PnP enumeration?**
   The serial trace shows `IopInitializeBootDrivers done` →
   `IopReinitializeBootDrivers done` → `IopCreateArcNames`. Both reinit
   passes are supposed to drive PnP to enumerate everything. If
   reinit is returning early or not waiting, CD-ROM PDOs may not be
   ready yet when `IopCreateArcNames` runs.

   Note the lines `[DA] PCDA get ClassGUID -> a=00000000 b=00000000`
   followed by `[DA] SM PiCallDriverAddDevice->st a=c0000001 b=00000302`
   appear MANY times for `PCI_HAL\PNP0A03\0` during the
   `PiQueueDeviceAction` loop — `0xc0000001` is
   `STATUS_UNSUCCESSFUL`. `PiCallDriverAddDevice` keeps failing for
   the PCI host bridge. That's likely upstream of CD-ROM not being
   enumerated. **Start here.**

3. **Possible quick wins:**
   - Verify `uniata.sys`'s `DriverEntry` actually returned success
     (add a marker after the `IopInitializeBuiltinDriver` call in
     `sdk/lib/rust/ntoskrnl-rust/src/iomgr/driver.rs` around line 1155
     showing `ret` value per driver).
   - Check if `cdrom.sys`'s `AddDevice` ever runs — add a marker in
     `IopCallDriverAddDevice` for `cdrom`.
   - Bisect: try forcing `\SystemRoot` to point at the ramdisk
     equivalent or a hard-coded CD-ROM device name to see if the rest
     of SMSS works once the ntdll lookup succeeds.

4. **What's known good:** arm64 boots fine through the ramdisk path,
   so the post-`\SystemRoot` SMSS chain works. The fix should be in
   the iomgr / PnP enumeration / cdrom.sys driver region.

### Useful entry points to read

- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/arcname.rs::IopCreateArcNames`
  (line 442) — orchestration.
- Same file, `::IopCreateArcNamesCd` (line 130) — the actual CD-ROM
  PDO walk that finds nothing.
- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/driver.rs::IopInitializeBootDrivers`
  (line 1146-ish) — the group/driver iteration loop that calls each
  driver's DriverEntry.
- `sdk/lib/rust/ntoskrnl-rust/src/iomgr/driver.rs::IopInitializeBuiltinDriver`
  (line 837) — already has `[BLT]` trace.
- The `[DA] PCDA …` traces come from the PnP DeviceAction state
  machine in `sdk/lib/rust/ntoskrnl-rust/src/io/` — grep for
  `PiCallDriverAddDevice` to find the source.

### Anti-patterns to avoid

- **Do NOT revert any of today's freeldr fixes** to "rule them out" —
  they're load-bearing for getting this far. Memory:
  `feedback_translation_philosophy.md`.
- **Do NOT make `wdf01000` boot-start again** until both the
  `wdfldr.DriverEntry` hang AND the `WdfRegisterLibrary` NULL-deref
  are fixed. Today's workaround (demand-start + lazy init) is the
  right state.
- **Do NOT weaken `[EXPLORER] _tWinMain entered`** as the boot marker
  to "make smoke pass." Memory: `smoke-tests.md`.
- **Do force-rebuild the kernel** when adding diagnostic markers in
  Rust kernel code. `ninja livecd` alone WILL say "no work to do" and
  silently use the cached `libkernel_rust.a`. The recipe is:
  ```
  touch sdk/lib/rust/ntoskrnl-rust/src/path/to/file.rs
  rm -rf build-{i386,amd64}/ntoskrnl/rust-kernel-target \
         build-{i386,amd64}/ntoskrnl/libkernel_rust.a \
         build-{i386,amd64}/ntoskrnl/ntoskrnl.exe
  cd build-i386 && ninja livecd     # (or amd64)
  ```
  Similar for freeldr Rust crates (`build-*/boot/freeldr/freeldr/
  rust-freeldr-target/`).
- **Do remember to also rebuild the SYSTEM hive** after any registry
  `.inf` change:
  ```
  rm -f build-i386/boot/bootdata/{system,registry.inf} build-i386/livecd.iso
  cd build-i386 && ninja livecd
  ```

---

## Smoke status snapshot (2026-05-25)

| arch  | Before today                          | After today's merge                  |
| ----- | ------------------------------------- | ------------------------------------ |
| arm64 | GREEN 5+5skip                         | GREEN 5+5skip ✓                      |
| i386  | freeldr-stage `EIP=0x10006 INVALID_OPCODE` after rosload UI | kernel reaches SMSS init → BUGCHECK 0x6B (this doc) |
| amd64 | freeldr-stage heap corruption post-rosload-UI | same as i386                          |

---

## Files touched today (all merged into continue-rust-port)

```
sdk/lib/rust/freeldr-rust/src/ffi.rs                         (+11 -2)
sdk/lib/rust/freeldr-rust/src/peloader.rs                    (+8  -1)
sdk/lib/rust/freeldr-rust/src/winldr.rs                      (+6  -1)
sdk/lib/drivers/wdf/wdf01000_reg.inf                         (+5  -1)
sdk/lib/drivers/wdf/wdfldr/CMakeLists.txt                    (+1)
sdk/lib/drivers/wdf/wdfldr/wdfldr_reg.inf                    (+11) [new]
sdk/lib/drivers/wdf/wdfldr/wdfldr.c                          (+6)
```

Plus the entire 27-commit freeldr sweep history (cold-link, ABI fixes,
PartialResourceList write_bytes fixes, etc.) that was already living on
`freeldr-rust-port` before today.

---

## Pointers for the next agent

- This handoff doc lives at
  `docs/engineering/handoff-smss-init-arc-names.md`.
- Memory entries (`/home/curi/.claude/projects/-home-curi-Projects-os-playground-vibeos/memory/`):
  `feedback_freeldr_ldr_boot_driver_constant.md`,
  `feedback_freeldr_peldr_callback_ntapi.md`,
  `feedback_machpc_calling_convention.md`,
  `feedback_partial_resource_list_write_bytes.md`,
  `feedback_pchw_resource_descriptor_pointer_size.md`,
  `feedback_extern_c_abi.md`,
  `smoke-tests.md`,
  `feedback_translation_philosophy.md`.
- Open task in tracker: #18 (PROCESS1_INITIALIZATION_FAILED).
- The `qemu/i386_dbgc.bin` and `qemu/amd64_dbgc.bin` artifacts in the
  repo root are the captured boot traces from the working state — they
  decode via `qemu/decode_debugcon.py` or the trace catalog in
  `tools/rustconv/data/trace_catalog_{arch}.json`.
