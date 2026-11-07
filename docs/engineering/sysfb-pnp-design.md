# sysfb as a PnP display driver — design

**Status:** steps 0-2 done (2026-11-07); steps 3-4 partly in (see the plan); step 5 not started.
Proposed 2026-10-11. Owner: unassigned.
**Scope:** `drivers/video/sysfb` (`sysfb-rust`), its registry wiring, and win32k's display
enumeration as far as it constrains the driver. This is the work that lets sysfb sit under a
real GPU driver instead of in place of one.

## Where it stands

sysfb.sys (formerly arm64vid.sys, renamed 2026-10-11) is the only display driver on every
arch. It drives whatever linear framebuffer the firmware handed over:

1. The UEFI loader (`freeldr-rust/src/uefivid.rs`) keeps the firmware's GOP mode. Since
   2026-10-11 it no longer forces GOP mode 1, so a real panel stays at its native size. It
   records the framebuffer in the ARC tree under
   `MultifunctionAdapter\N\DisplayController\M`, as a `CM_FRAMEBUF_DEVICE_DATA`.
2. sysfb is a **legacy, boot-start, non-PnP** driver: `Start=0`, group
   `Boot Bus Extender` (hivesys.inf). Its `DriverEntry`:
   - creates `\Device\Video0` and `\??\DISPLAY1` by hard-coded name;
   - reads the ARC tree;
   - writes `HARDWARE\DEVICEMAP\VIDEO` itself (`\Device\Video0` → `Services\sysfb\Device0`,
     `MaxObjectNumber` = 0);
   - answers the handful of `IOCTL_VIDEO_*` calls framebuf.dll makes.
3. With no framebuffer it runs **headless**, on a 1024x768 frame in pool memory, on every
   arch since 2026-10-11, because win32k bugchecks 0xB4 without a display device.
4. The framebuffer is mapped **write-combined** (since 2026-10-11, previously uncached).

This is the same role Microsoft's BasicDisplay.sys and Linux's efifb/simpledrm play. The
design is right. What is wrong is that sysfb plays it *outside* PnP.

## What breaks on real hardware

| # | Problem | Consequence |
|---|---|---|
| 1 | No PnP device. The PCI display controller (class 03) is never claimed. | Device Manager shows an unknown device. Nothing ties "the framebuffer" to "the GPU that owns it". |
| 2 | `\Device\Video0` is created by name at boot, unconditionally. | A native GPU driver loaded later has no way to take over: it would need Video0, and sysfb never lets go. Two drivers end up scanning out one BAR. |
| 3 | No power IRPs. | S3/S4 resume returns with the GOP framebuffer's contents, and on some firmware its mode, gone. sysfb cannot redraw or refuse the transition. |
| 4 | One display. The loader takes the first GOP handle. | A second head is invisible. Usually fine for a fallback driver, but it should be a decision rather than an accident. |
| 5 | One reported mode. `NumberRed/Green/BlueBits` are hard-coded to 8. | Wrong for a 16bpp 5-6-5 `PixelBitMask` framebuffer: count bits from the masks. The loader's `FrameBufferOffset` is ignored. |
| 6 | ARC-tree discovery walks `MultifunctionAdapter\0..63`. | Works, but the framebuffer is located by scanning the registry rather than as the device's own resource. |

Items 5 and 6 are small and can land on their own. Items 1–3 are this design.

## Target design

sysfb becomes a **PnP function driver with two ways in**, keeping today's behaviour when
nothing better is present.

### 1. PCI binding (the normal case on PCs)

- **INF match:** sysfb.inf matches `PCI\CC_0300` and `PCI\CC_0380` (VGA-compatible and
  other display controllers) at the lowest rank, so any vendor INF outranks it. This is
  BasicDisplay's model. The same INF supplies the class (`Display`, the GUID win32k
  already expects) and the service entry.
- **Ownership check in `IRP_MN_START_DEVICE`:** take the device's translated memory
  resources, and claim the device only if the firmware framebuffer's physical base falls
  inside one of its BARs. A second display controller without the GOP framebuffer fails
  start with `STATUS_DEVICE_CONFIGURATION_ERROR`, rather than pretending to be a display.
- **Device naming:** allocate `\Device\VideoN` from a counter, not a literal. Write
  `DEVICEMAP\VIDEO\\Device\VideoN` → `Services\sysfb\DeviceN` and raise `MaxObjectNumber`;
  win32k's `EngpUpdateGraphicsDeviceList` (win32k-rust `device.rs`) already walks 0..Max.
  Register the `GUID_DEVINTERFACE_DISPLAY_ADAPTER` interface so a future enumeration does
  not depend on the device map.
- **Stop and remove:** `IRP_MN_QUERY_STOP/REMOVE` refuse while win32k holds a mapping
  (an open handle count from `IRP_MJ_CREATE`/`CLOSE`). `IRP_MN_STOP/REMOVE` unmap the
  framebuffer, delete the device-map value and the device object. This is the hand-off a
  vendor driver needs: Windows does the same dance when the vendor driver arrives after
  BasicDisplay.

### 2. Root-enumerated fallback (no PCI display)

The arm64 virt machine's ramfb, a Device Tree `simple-framebuffer` on a Pi, and a headless
server have no PCI display controller to bind to. Keep a **root-enumerated instance**,
`ROOT\SYSFB\0000`, created by setup, which starts only if no PCI instance has claimed the
framebuffer. It runs today's path: the ARC framebuffer if present, otherwise headless.
Arbitration is a named kernel event or a flag in the service key, set by whichever
instance starts first. Today's `Start=0` boot-start registration becomes this instance's.

The boot-start requirement must survive. win32k's `InitVideo` runs before PnP has
necessarily started the PCI instance, which is why sysfb is boot-start today (commit
`e9d568c1e40`). So:

- Load the driver as **boot-start PnP**: `Start=0` with the PCI ID in the
  CriticalDeviceDatabase, exactly as the removed VGA miniport was wired.
- Keep the root instance as the guaranteed fallback.

### 3. Power

- Handle `IRP_MN_QUERY_POWER` / `SET_POWER`. The framebuffer has no save/restore hardware,
  so for **system** sleep states fail `QUERY_POWER` for S3/S4 while sysfb is the active
  display. That is honest: there is nothing to restore the mode with. A vendor driver lifts
  the restriction by replacing sysfb.
- **Device** D0↔D3 is a no-op.

### 4. Not in scope

- Mode setting: GOP cannot change mode after ExitBootServices.
- Multiple heads: they need one PCI instance per GOP handle, and the loader would have to
  record every handle. Revisit when there is hardware to test it on.
- Acceleration.

## As built (steps 1-2, 2026-11-07)

Three choices differ from the target design above, each for a reason found in the tree:

- **The root instance is reported, not created by setup.** `DriverEntry` calls
  `IoReportDetectedDevice` (the HAL's pattern), giving `Root\sysfb\0000` with hardware ID
  `DETECTEDInternal\sysfb`. A reported device counts as started on the spot, so it claims the
  display inside `DriverEntry`, exactly where the legacy driver created `\Device\Video0`. That
  settles the boot-ordering risk below without a kernel event or a service-key flag. The PnP
  root enumerator skips instances reported on an earlier boot but not this one
  (`IopShouldProcessDevice`), so they do not pile up on an installed system.
- **`\Device\Video0` is a control device, not an FDO.** It is created on the first claim and
  attached to no stack. When the PCI instance starts and proves ownership, only the owner
  pointer moves, and the display keeps the name win32k gave it (`\\.\DISPLAY1` comes from
  Video0). A per-instance Video*N* would have renamed the display to DISPLAY2 whenever the
  PCI instance won. The FDOs are unnamed. Each registers `GUID_DEVINTERFACE_DISPLAY_ADAPTER`,
  which is enabled only on the current owner. `IOCTL_VIDEO_INIT_WIN32K_CALLBACKS` now reports
  the owner's PDO, not Video0.
- **`display.inf` is the INF, not a new `sysfb.inf`.** It still bound `PCI\CC_0300` to the
  `vga` service and `vgapnp.sys`, both deleted in `bdea86df186`. It now installs sysfb for
  `PCI\CC_0000/0001/0300/0301/0380` and `DETECTEDInternal\sysfb`. Boot-time binding is the
  `CriticalDeviceDatabase` (`PCI#CC_0300`, `PCI#CC_0380`) in `hivesys.inf`. The device map
  still points at `Services\sysfb\Device0`, because setup's `ProcessDisplayRegistry` writes the
  installed mode there.

Measured on smoke amd64 (`/tmp/vibeos-cli/20261107-113030-3z7c0n3p-smoke-amd64`):
- root AddDevice and claim at serial line 352;
- `[CDDB] MATCH PCI#CC_0300`, then the std-vga's START takes over at line 482;
- `InitVideo()` at line 1136.

arm64 virt has only `ramfb`, so its root instance stays the owner.

A PCI instance releases the display on STOP, SURPRISE_REMOVAL and REMOVE, and it falls back
to the root instance. With no root instance, Video0 and its device-map value are deleted
unless handles are still open on it. QUERY_STOP/QUERY_REMOVE fail with `STATUS_DEVICE_BUSY`
while the owner has the display open. Power IRPs pass down.

## Plan

| Step | Change | Proves it |
|---|---|---|
| 0 | **Done 2026-11-07.** Bit counts from the pixel masks (`mask_bits`, 8 when the loader gave none); `FrameBufferOffset` added to the base and taken off the size, refused past the resource. | smoke ×3 |
| 1 | **Done 2026-11-07** (as built above): `display.inf` binds PCI class 03 and the root ID; the root instance is reported by the driver. | install-smoke: Device Manager shows the GPU under Display adapters — *not yet run* |
| 2 | **Done 2026-11-07.** `AddDevice`/`START_DEVICE` with the BAR ownership check (Memory and MemoryLarge BARs); the Video0 control device with owner hand-off; device-interface registration. | smoke ×3 |
| 3 | **Partly in:** the hand-off and the open-handle refusal are written but no test drives them. Still owed: a modrun or kmtest that disables the device with no session open, then re-enables it. | that test |
| 4 | Power: refuse S3/S4 while active. Today power IRPs only pass down. | `powercfg`-equivalent query, plus a kmtest |
| 5 | Real hardware: Pi 4/5 bench (UEFI firmware, GOP) and an x86 box (the OptiPlex 9020 the ARC-tree comment came from). | screenshots at native resolution, desktop reached |

A rebalance that moves the boot GPU's BAR would leave the framebuffer base stale: the restart
would fail the ownership check and the display would go back to the root instance at the old
address. PCI keeps a boot device's assignment, so this is not expected to happen, but nothing
enforces it.

## Risks

- **Boot ordering** is the real one. If the PCI instance starts after win32k's `InitVideo`,
  the desktop has no device: 0xB4. The root fallback must therefore be able to start alone,
  and step 2 lands only once smoke shows `[sysfb] DriverEntry` ahead of `InitVideo()` on
  all three arches.
- `CriticalDeviceDatabase` entries for `PCI\CC_0300` were removed with the VGA miniport
  (`bdea86df186`). Re-adding them must name `sysfb`, not the dead service.
