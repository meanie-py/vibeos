# Handoff: arm64 boot 0x7B — kernel iomgr never dispatches the FAT mount IRP to fastfat

**Audience:** Codex (or any agent) picking up the arm64 boot blocker.
**Worktree:** `/home/curi/Projects/os-playground/vibeos-wt-fastfat`, branch `fastfat-port`.
**Date:** 2026-06-13. **Status:** open. Tracked as task #22 (orig) / #23 (current).

---

## ⚡ UPDATE 2026-06-14 — progress + a critical methodology fix + new localization

**Codex landed the IRP stack-location fix** (committed `913c1c24364`): ntkrnl-sys tier1
IRP helpers now use CurrentStackLocation @ IRP+0xb0, IO_STACK_LOCATION stride 0x60.
This was the real first suspect and it cleared the original `0xC0000010` INVALID_DEVICE_
REQUEST. fastfat rebuilt against the same ntkrnl-sys now agrees (its
`IoGetCurrentIrpStackLocation` also disassembles to `ldr [x0,#0xb0]`). **Necessary but
not sufficient** — arm64 now fails later: `BUGCHECK 0x64 SYMBOLIC_INITIALIZATION_FAILED`,
arg1 = `0xC000003A STATUS_OBJECT_PATH_NOT_FOUND`, loading `\SystemRoot\System32\smss.exe`.

### 🚨 CRITICAL TOOLING GOTCHA — non-volatile UART traces are OPTIMIZED AWAY in release
This invalidated many earlier "function X is/isn't reached" conclusions (both Codex's and
this author's). In the **release/optimized** kernel build, a plain `*uart = byte`
(non-volatile store whose result is never read) is **dead-code-eliminated by LLVM**, and
the associated string constant is dropped too. So a missing `[MOUNT]`/`[CHKVPB]` serial
line did **NOT** prove the code wasn't running — the trace itself was optimized out. The
existing `ARM64_UART_TRACE!` macro uses non-volatile writes and is therefore **unreliable
in release**. **Rule: instrument with `core::ptr::write_volatile` + `#[inline(never)]`
helpers, and ALWAYS gate on `strings build-arm64/ntoskrnl/ntoskrnl.exe | grep -c <marker>`
≥ 1 before booting.** Reusable helper pattern (drop in `iomgr/volume.rs`, `pub(crate)`):
```rust
#[inline(never)]
pub(crate) unsafe fn mntv_str(s: &[u8]) {
    #[cfg(target_arch = "aarch64")]
    { let u = Arm64UartKernelBase as *mut u32; if !u.is_null() { for &c in s { core::ptr::write_volatile(u, c as u32); } } }
    #[cfg(not(target_arch = "aarch64"))] { let _ = s; }
}
#[inline(never)]
pub(crate) unsafe fn mntv_hex(v: u64) {
    #[cfg(target_arch = "aarch64")]
    { let u = Arm64UartKernelBase as *mut u32; if !u.is_null() {
        core::ptr::write_volatile(u, b'0' as u32); core::ptr::write_volatile(u, b'x' as u32);
        for i in (0..16).rev() { let n = ((v >> (i*4)) & 0xf) as u8; core::ptr::write_volatile(u, (if n<10 {b'0'+n} else {b'a'+n-10}) as u32); }
        core::ptr::write_volatile(u, b'\r' as u32); core::ptr::write_volatile(u, b'\n' as u32);
    } }
    #[cfg(not(target_arch = "aarch64"))] { let _ = v; }
}
```
Also: the cargo fingerprint frequently MISSES edits — `touch` the edited `.rs` before
building, and still gate on `strings`. (kernel build: `rm -rf build-arm64/ntoskrnl/{rust-kernel-target,libkernel_rust.a,ntoskrnl.exe}; ninja ntoskrnl`.)

### New localization (with reliable volatile traces) — the failure is in OBJECT MANAGER, not fastfat or the mount logic
Traced the actual smss.exe/ntdll.dll open path:
- **fastfat is healthy and NEVER reached.** `FF_DRVENTRY` fires (DriverEntry registers
  both FS device objects); `FatMountVolume`, `FatFsdFileSystemControl`, `FatFsdCreate` are
  **never called**. The earlier "fastfat create-path bug" hypothesis is WRONG.
- **`IopMountVolume` and `IopCheckVpbMounted` are NEVER reached** (volatile-trace
  confirmed — the prior non-volatile "MOUNT not reached" was unreliable, but the volatile
  re-test agrees they aren't called *for these opens*).
- **`IopParseDevice` is called exactly ONCE** in the whole boot — for a device of type
  `0x2a = FILE_DEVICE_BUS_EXTENDER` with a **null `Vpb`** and `RemainingName.Length == 0`.
  The smss.exe / ntdll.dll opens **never reach `IopParseDevice`**.
- ⇒ The opens of `\SystemRoot\System32\smss.exe` (and the ntdll test open) fail with
  `STATUS_OBJECT_PATH_NOT_FOUND` **inside Object-Manager name resolution**, before the
  device parse routine runs. `\SystemRoot` reassign succeeds
  (→ `\Device\Ramdisk{9e475cab-…}\reactos`), so the break is resolving
  `\Device\Ramdisk{GUID}` (the volume device) in the object namespace, or that volume
  device is mis-created (note the lone parsed device is a **bus-extender with no VPB** —
  disk/cdrom/tape/virtual-disk are the only types that get a VPB and an FS-mount).

### Where to look next (this is OM / ramdisk-volume territory, NOT fastfat)
1. Does a device object named `\Device\Ramdisk{9e475cab-956f-11d0-a0c4-08002be10d8e}`
   actually exist, and with what `DeviceType`? The ramdisk driver logs several
   `UNIMPLEMENTED!` paths (`ramdisk.c:1163` DeviceControl `FSCTL 0xffffffff UNSUPPORTED`,
   `ramdisk.c:895` WorkerThread `Invalid request: 0`) — the volume device may be created
   wrong (wrong type → no VPB → unmountable) or under the wrong name.
2. Trace OM path resolution (`ObpLookupObjectName` / the symlink+device parse) for the
   smss open to see exactly where `STATUS_OBJECT_PATH_NOT_FOUND` originates (it's before
   `IopParseDevice`). Use the volatile-trace pattern above.
3. Re-verify Codex's `IopParseDevice` related-file-object changes with volatile traces
   (some of their original validation may have used now-known-unreliable non-volatile
   traces).

**fastfat is done and exonerated for this bug.** Don't touch fastfat. The remaining work
is OM device-name resolution + the ramdisk volume device — a separate kernel/ramdisk
track.

---

## TL;DR

arm64 fails to boot with **`BUGCHECK 0x0000007B` (INACCESSIBLE_BOOT_DEVICE)**, arg2 =
`0xC0000010` (`STATUS_INVALID_DEVICE_REQUEST`). The root cause is **NOT in fastfat** —
it is in **`ntoskrnl-rust` iomgr**. The kernel registers fastfat correctly and even
calls into its file-system-control dispatch, but it **never sends the
`IRP_MN_MOUNT_VOLUME` IRP** to fastfat, so the FAT volume never mounts, so the boot
device is inaccessible.

This was previously misdiagnosed as "FatMountVolume is a stub." That stub is now a
full faithful Rust translation (committed `af646bd852a`) and the bug is **unchanged** —
proving FatMountVolume was never the cause; it is **never even reached**.

**Your job:** find why `ntoskrnl-rust`'s iomgr does not deliver `IRP_MN_MOUNT_VOLUME`
to the registered FAT filesystem on arm64, and fix it.

---

## What is proven (high confidence)

Evidence came from temporary `DbgPrint`/UART instrumentation in fastfat (since reverted)
and a reliable fastfat build. Serial trace at boot showed, in order:

```
[BLT] IopInitializeBuiltinDriver: fastfat.sys       <- fastfat loads (builtin driver)
FF_DRVENTRY: registered disk=<p> cdrom=<p>          <- DriverEntry ran IoRegisterFileSystem x2 OK
[CHKVPB] NOTMOUNTED -> mount                         <- IopCheckVpbMounted: VPB not mounted, will mount
FF_FSCTL: enter minor=4                              <- fastfat FatFsdFileSystemControl invoked, minor=4
[IO] ZwOpenFile(ntdll.dll) failed: 0xc0000010        <- the open that triggered the mount fails
... (REASSIGN \SystemRoot -> \Device\Ramdisk{GUID}\reactos) ...
*** BUGCHECK: 0x0000007B (<deviceobj>, 0xffffffffc0000010, 0, 0)
```

Established facts:

1. **fastfat is registered.** `DriverEntry` (fatinit.rs) runs
   `IoRegisterFileSystem(FatDiskFileSystemDeviceObject)` and the cdrom one. Confirmed
   by the `FF_DRVENTRY` trace.
2. **fastfat's dispatch is wired and gets called** — but **only ever with
   `MinorFunction = 4` (`IRP_MN_KERNEL_CALL`)**, never `1` (`IRP_MN_MOUNT_VOLUME`).
   `IO_STACK_LOCATION.MinorFunction` is at offset 1 and verified correct, so `minor=4`
   is genuine, not a misread.
3. **minor=4 → `STATUS_INVALID_DEVICE_REQUEST` is correct fastfat behavior.**
   `FatCommonFileSystemControl` (fsctrl.rs) dispatches `0=USER_FS_REQUEST`,
   `1=MOUNT_VOLUME`, `2=VERIFY_VOLUME`, and `default => STATUS_INVALID_DEVICE_REQUEST`.
   The **C** source does exactly the same (`drivers/filesystems/fastfat/fsctrl.c:828`).
   So fastfat returning `0xC0000010` for minor=4 is faithful, not a translation bug.
4. **`IopCheckVpbMounted` runs and decides to mount.** Its `[CHKVPB] NOTMOUNTED -> mount`
   branch fired (VPB present, `VPB_MOUNTED` clear), which calls `IopMountVolume`
   (`sdk/lib/rust/ntoskrnl-rust/src/iomgr/volume.rs:254`).
5. **`IopMountVolume`'s mount path apparently never sends the IRP to fastfat.** Its
   source clearly builds a `minor=1` IRP and `IoCallDriver`s the FS
   (`volume.rs:592-623`), with `[MOUNT] ...` UART traces — but **no `[MOUNT]` trace ever
   appeared** in serial, and fastfat never saw minor=1. (See the build-cache caveat
   below before trusting/“disproving” any kernel trace.)

So the divergence is **between `IopCheckVpbMounted` calling `IopMountVolume` and fastfat
receiving the mount IRP**. The FAT mount IRP (`IRP_MN_MOUNT_VOLUME`) is not arriving.

---

## Open questions / hypotheses (start here)

1. **Where does `minor=4` (`IRP_MN_KERNEL_CALL`) come from?** It is *not* set by:
   - `IopMountVolume` → sets `minor=1` (`volume.rs:594`, `:609`).
   - `IoBuildDeviceIoControlRequest` FSCTL path → sets `minor=0` (`iofunc.rs:~656`).
   Find the code path that builds a `IRP_MJ_FILE_SYSTEM_CONTROL` IRP with
   `MinorFunction=4` and sends it to the FAT FS device. That path is what's actually
   reaching fastfat instead of the mount. (Could be a kernel-mode `NtFsControlFile` /
   `IoSynchronousPageWrite` / verify path, or a mis-set minor in some IRP builder.)

2. **IRP stack-location handling on arm64.** Prime suspect for "kernel sets minor=1 but
   driver reads minor=4": `IoGetNextIrpStackLocation` / `IoCallDriver` (which decrements
   `CurrentStackLocation`) / `IoGetCurrentIrpStackLocation`. If `IoCallDriver`/the stack
   pointer math is off by one or mis-translated, the FS reads a *different*
   (uninitialized) stack location → garbage minor (4). Implementations:
   - `IoCallDriver` / `IofCallDriver`: `sdk/lib/rust/ntoskrnl-rust/src/iomgr/iofunc.rs`
   - `IoGetCurrentIrpStackLocation` / `IoGetNextIrpStackLocation`:
     `sdk/lib/rust/ntoskrnl-rust/src/iomgr/irp.rs`
   - There is an existing notes file worth reading:
     `sdk/lib/rust/ntoskrnl-rust/src/iomgr/old_translations/irp_bugs_found.md`
   Verify the IRP `CurrentStackLocation` init in `IoAllocateIrp` (`volume.rs:577`
   allocates the mount IRP) and that `IoGetNextIrpStackLocation` returns the slot
   `IoCallDriver` will make current.

3. **Is `IopMountVolume` actually entered, or is the call resolving elsewhere?**
   `volume.rs` is the active impl (C `volume.c` is disabled at `ntoskrnl/ntos.cmake:151`),
   and `IopMountVolume` is the only definition (`volume.rs:490`, `#[no_mangle]`). But the
   entry trace never printed. **Before concluding it isn't entered, defeat the build
   cache (below)** — my MNTV entry trace never actually made it into the linked binary.

4. **FS registration queue.** `IopMountVolume` iterates `IopDiskFileSystemQueueHead`
   (`volume.rs:539-565`). Confirm `IoRegisterFileSystem` actually inserts fastfat's
   disk-FS device into that exact list head the arm64 kernel iterates (list-head
   identity / linkage). If the list is empty/wrong, the loop body (and its IoCallDriver)
   never runs → no mount IRP, `Status` stays failure → 0x7B.

---

## ⚠️ Critical tooling gotcha: the kernel final-link cache masks edits

Editing a `ntoskrnl-rust` source and rebuilding can **silently reuse a stale
`ntoskrnl.exe`**, so your UART traces never actually run. Symptoms I hit:
- `cargo build -p ntoskrnl-rust` reported success but `strings` showed my new trace
  string was absent from the `.a` (cargo fingerprint missed the edit until I `touch`ed
  the file).
- `rm ntoskrnl/libkernel_rust.a && ninja ntoskrnl` rebuilt the rlib, but the
  **reactos-link (ntoskrnl-cargo) link step printed `Finished 0.10s` and reused a stale
  `ntoskrnl.exe`** — the new code never linked in.

**The non-negotiable rule: never trust a boot until `strings` confirms your marker is in
the final `build-arm64/ntoskrnl/ntoskrnl.exe`.** That single check would have saved me
hours. The kernel is linked by the `reactos-link` cargo step (`ntoskrnl-cargo` crate);
its layers that can each independently go stale are: (a) the `ntoskrnl-rust` rlib
compile (cargo fingerprint missed my edit until `touch`), (b) `rust-kernel-target`
(`build-arm64/ntoskrnl/rust-kernel-target`), (c) the final `reactos-link` step (reported
`Finished 0.10s` reusing a stale exe).

```bash
cd /home/curi/Projects/os-playground/vibeos-wt-fastfat
# Preferred: the harness does a proper clean kernel build (nukes rust kernel target):
./vibeos-cli build --arch arm64 --clean
# VERIFY before booting — this is the gate:
strings build-arm64/ntoskrnl/ntoskrnl.exe | grep -c "<your-unique-marker>"   # must be >= 1
```
If you must do it by hand, `touch` the edited source first (defeats the cargo
fingerprint miss), then nuke `build-arm64/ntoskrnl/{rust-kernel-target,libkernel_rust.a,
ntoskrnl.exe}` and `ninja ntoskrnl` — but you may *also* have to force the `reactos-link`
step (find its cargo target dir under `build-arm64/`; `cargo` may report the link
"Finished" without relinking). Whichever route: **verify with `strings`, every time.**

---

## Build / boot / capture recipes

### Update the arm64 ramdisk after building a kernel or driver
arm64 boots a **FAT32 ramdisk** (`qemu/ramdisk.img` inside `qemu/boot.img`). After
building, copy the artifact in and re-wrap:
```bash
cd /home/curi/Projects/os-playground/vibeos-wt-fastfat
mcopy -o -i qemu/ramdisk.img build-arm64/ntoskrnl/ntoskrnl.exe ::/reactos/system32/ntoskrnl.exe
# (for a fastfat change instead: build-arm64/drivers/filesystems/fastfat/fastfat.sys -> ::/reactos/system32/drivers/fastfat.sys)
mcopy -o -i qemu/boot.img qemu/ramdisk.img ::/ramdisk.img
# WARNING (CLAUDE.md): never mcopy uefildr.efi into boot.img AFTER the ramdisk — corrupts FAT32.
```

### Boot and capture serial without the slow smoke timeout
The debug toolkit's preflight enforces a GDB/DWARF-richness check you don't need for
serial capture — skip it:
```bash
chmod +x qemu/*.sh   # scripts may have lost +x in this worktree
RUN=$(bash qemu/start.sh arm64 --skip-preflight 2>/dev/null | tail -1)
bash qemu/wait-for.sh "$RUN" --pattern "BUGCHECK|shell ready" --timeout 180
grep -aE "CHKVPB|MOUNT\]|FF_|BUGCHECK|<marker>" qemu/runs/$RUN/serial.log
bash qemu/stop.sh "$RUN"
```
Full serial log: `qemu/runs/<RUN>/serial.log`.

### Working UART trace pattern (this style DOES emit on arm64)
Direct PL011 writes via `Arm64UartKernelBase` (a `usize` static in volume.rs). Do NOT
rely on the `ARM64_UART_TRACE!` macro alone until you've re-verified the binary (it was a
victim of the link-cache, not the macro itself):
```rust
#[cfg(target_arch = "aarch64")]
{
    let _u = Arm64UartKernelBase as *mut ULONG;   // ULONG=u32
    if !_u.is_null() { for &c in b"[MARK] something\r\n" { *_u = c as ULONG; } }
}
```
`sprintf` is NOT in module scope in volume.rs (all uses are commented out) — emit fixed
marker strings or branch on conditions instead of formatting values.

---

## Key files

| Concern | Path |
|---|---|
| Mount entry / VPB check | `sdk/lib/rust/ntoskrnl-rust/src/iomgr/volume.rs` — `IopCheckVpbMounted` (~213), `IopMountVolume` (~490, builds `minor=1` IRP at 592-623) |
| IRP stack-loc / IoCallDriver | `sdk/lib/rust/ntoskrnl-rust/src/iomgr/iofunc.rs` (IoCallDriver, IoBuildDeviceIoControlRequest), `iomgr/irp.rs` (IoGet{Current,Next}IrpStackLocation) |
| Prior IRP-bug notes | `sdk/lib/rust/ntoskrnl-rust/src/iomgr/old_translations/irp_bugs_found.md` |
| 0x7B raise site | `volume.rs` IopMountVolume failure path (mirror of C `ntoskrnl/io/iomgr/volume.c:822`: fail + `DO_SYSTEM_BOOT_PARTITION` + phase<2 → bugcheck) |
| fastfat FS-control dispatch (faithful, NOT the bug) | `sdk/lib/rust/fastfat-rust/src/fsctrl.rs` — `FatFsdFileSystemControl` (~11312), `FatCommonFileSystemControl` minor-switch |
| ntdll-open test that triggers the mount | `sdk/lib/rust/ntoskrnl-rust/src/iomgr/iomgr.rs` (~849) |

## What's done / not to redo
- fastfat `FatMountVolume` is fully translated, builds cargo-clean ×3, integrates,
  committed `af646bd852a`. Do **not** re-touch fastfat for this bug — it's a kernel issue.
- i386 + amd64 both smoke ✓ boot to desktop with the current tree (x86 boots from the
  cdfs LiveCD, so fastfat's mount path isn't exercised there — arm64 is the only arch
  that exercises FAT mount).

## Definition of done
arm64 smoke boots to the desktop (or at least: fastfat's `FatMountVolume` is reached
with `minor=1`, the FAT volume mounts, and the 0x7B is gone). Re-run:
`./vibeos-cli smoke --arch arm64` (after a clean kernel build + ramdisk refresh).
