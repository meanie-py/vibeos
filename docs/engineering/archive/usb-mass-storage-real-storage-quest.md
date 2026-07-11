# USB mass storage & the path to real storage

> **ARCHIVED 2026-07-10** — parked scaffold closed 2026-07-10 — tracked as roadmap §10b (now UNBLOCKED by the USB beachhead). Outstanding items live in
> `docs/product/vibeos-roadmap.md` § "Check-back ledger" / § "Known bugs & flakes".


**Status:** design / quest brief scaffold, opened 2026-07-05. *Light — a skeleton to flesh
out before implementing.* Storage sibling of the input quest
(`docs/engineering/input-architecture-and-unification.md`).

> **🔓 UNBLOCKED (2026-07-10):** the shared prerequisite — a working Rust USB host stack —
> is now PROVEN on all three arches. The input quest's beachhead landed (USB kbd/mouse
> enumerate and deliver through hidusb→hidclass→kbd/mouhid→kbd/mouclass; both walls closed,
> see `input-architecture-and-unification.md` + `archive/wall2-arm64-usb-hid-handoff.md`).
> "One beachhead unlocks both" has happened: this quest starts from a live EHCI/hub/usbd
> substrate, not from scratch — the remaining unknown is only usbstor/classpnp/disk on top.

**One-line quest:** prove the (already-translated) Rust **USB mass-storage** stack can
enumerate a QEMU `usb-storage` device backed by a disk image and expose it as a real block
device — because USB mass storage is the one storage path that is arch-unified, correctly
layered, and viable on real hardware *(and it rides the exact same USB host stack the input
quest is probing, so one beachhead unlocks both).*

---

## TL;DR

Today arm64 boots and runs **entirely from a RAM disk** — the whole OS is a FAT image
(`ramdisk.img`) loaded into memory and mounted read/write. It never touches a storage
controller. That is a VM/dev shortcut that cannot survive real hardware: a Raspberry Pi has
no virtio-blk and no ATA, and this port has **no SD/eMMC/NVMe driver at all**. The realistic
first real-storage path on both a physical ARM SoC and every QEMU machine type is **USB mass
storage** (`usbstor`, USB Bulk-Only Transport → SCSI). The whole `usbstor` driver is already
Rust; what's unproven is whether the USB host stack under it ever *enumerates* anything. That
unknown is shared verbatim with the input quest — resolving it once decides both.

---

## The current shortcut — RAM disk (arm64)

arm64 has no real storage stack in the boot path. The bootloader loads a FAT image into RAM
and the OS mounts it:

- **freeldr config** (`qemu/freeldr.ini`): `SystemPath=ramdisk(0)\reactos`,
  `Options=… /RDPATH=\ramdisk.img`. `BootType=Windows2003`.
- **Boot media** (`qemu/mkramdisk.sh`): a ~140 MB FAT `ramdisk.img` holding *all* system
  files (~120 MB), folded into a FAT32 `boot.img` alongside `uefildr`. QEMU attaches
  `boot.img` on **virtio-blk** (`qemu/run.sh:110` `-drive …,if=virtio`); the UEFI loader
  reads the ramdisk out of it and hands the kernel an in-memory disk.
- **Driver** (`sdk/lib/rust/ramdisk-rust/`, built via
  `drivers/storage/class/ramdisk/`): `RamdiskCreateRamdisk` /
  `RamdiskCreateDiskDevice` / `RamdiskMapPages` (`ramdisk.rs:285,368,699`) present the memory
  region as `\Device\Ramdisk*`. Pure memory — no controller, no interrupts, no bus.

This is fine for a VM and it is the *only* reason arm64 boots without a storage driver. It is
also a dead end for hardware: nothing here generalizes to a physical disk.

**x86 is already more real.** i386/amd64 boot from the LiveCD via emulated IDE/ATAPI:
`hivesys.inf:23` registers **`uniata`** for PCI `CC_0101` (IDE), plus `cdrom`/`disk`/`cdfs`
(`hivesys.inf:59-63`). So x86 exercises a genuine controller → class → filesystem storage
stack today; arm64 does not.

---

## The real-hardware target — USB mass storage + real disk controllers

Real machines don't have virtio-blk. The portable, universally-present removable-storage path
is **USB mass storage**: a USB Bulk-Only Transport (BOT) device that tunnels SCSI commands. It
works on a USB stick, an SD-card-in-a-USB-reader, a USB SSD — and QEMU emulates it on **every**
machine type (`q35`, `pc`, *and* `virt`) via `-device usb-storage` / `-device usb-bot`. That
makes it the storage analogue of USB HID: one driver that is real on hardware and testable in
the VM, on all three arches.

The longer arc is a full real-disk stack (partition manager → volume → filesystem on a fixed
controller), but USB mass storage is the cheapest *first* real block device and the only one
that needs no new bus driver on an ARM SoC.

---

## What's already Rust vs. the gaps

**Already translated to Rust** (`sdk/lib/rust/`):

| Crate | Role |
|---|---|
| `usbstor-rust` | **USB mass storage (BOT/SCSI)** — full driver: `fdo.rs`, `pdo.rs`, `scsi.rs`, `disk.rs`, `descriptor.rs`, `queue.rs` |
| `uniata-rust` | IDE/ATA/ATAPI controller (the x86 LiveCD path) |
| `storport-rust` / `scsiport-rust` | SCSI port miniport frameworks |
| `storahci-rust` | AHCI/SATA miniport |
| `disk-rust` | disk class driver (`\Device\Harddisk*`) |
| `classpnp-rust` | storage class-driver framework |
| `partmgr-rust` | partition manager |
| `cdfs-rust` / `fastfat-rust` / `fs_rec-rust` | filesystems (CDFS, FAT, recognizer) |
| `ramdisk-rust` | the RAM-disk shortcut above |

**The gap that makes USB mass storage matter:** there is **no SD/eMMC driver and no NVMe
driver** in the tree. On a real ARM SoC (Raspberry Pi), the native boot medium is SD/eMMC —
for which this port has *nothing*. So the *only* Rust storage path that could touch real ARM
hardware without writing a brand-new SoC bus driver is **USB mass storage**. (`uniata`/
`storahci` are x86/PC-controller drivers; they don't help a Pi.)

The second, deeper gap is the same one the input quest names: it's unproven that the **USB
host stack** (`usbport`/`usbehci`/`usbohci`/`usbuhci`/`usbhub`) enumerates *any* device today,
and there is no `usbxhci` (only USB2-era HCs). `usbstor` may be translated-but-never-exercised.

---

## The beachhead — prove `usbstor` enumerates a QEMU disk

Same shape as the input probe, storage payload. Boot QEMU (start with **amd64**, fastest
toolchain) with an emulated USB disk backed by a real image and prove the Rust stack brings it
up as a block device:

```
-drive if=none,id=usbdisk,format=raw,file=usbtest.img \
-device usb-ehci,id=ehci -device usb-storage,bus=ehci.0,drive=usbdisk
#   (or -device usb-bot; or -device qemu-xhci if EHCI proves dead + a usbxhci driver exists)
```

**Success** = the chain `usbhub` (port connect) → device/descriptor enumeration →
`usbstor` FDO/PDO created (`usbstor-rust/src/fdo.rs`,`pdo.rs`) → SCSI `INQUIRY`/`READ CAPACITY`
over BOT (`usbstor-rust/src/scsi.rs`) → `disk` class binds → a readable `\Device\Harddisk*` /
volume, verified by reading a known sector/file off `usbtest.img`.

Instrument the same milestone ladder as the input quest (HC start → port connect → device
descriptor → BOT SCSI INQUIRY → READ CAPACITY → first sector read) so a failure localizes to
the exact stage.

**Decision tree:**
- **Enumerates → USB mass storage is the real-storage beachhead.** Wire it into the boot/
  smoke path; then extend toward partmgr/volume/FS on top, and eventually mount root off it on
  arm64 to retire the RAM-disk shortcut.
- **Doesn't enumerate (translated-but-dead)** → the blocker is the shared USB host stack, not
  storage — fix it once (see dependency note) and both quests unblock together.

---

## Dependency note — this shares the USB host stack with the input quest

**KEY SYNERGY.** `usbstor` (mass storage) and `hidusb` (HID input) sit on the **same** USB
host stack: `usbport` + `usbehci`/`usbohci`/`usbuhci` + `usbhub` + `usbd`. The input quest's
beachhead question — *"does the Rust USB stack enumerate a QEMU device at all?"* — is the
**identical gate** for storage. So:

- The input quest's `-device usb-kbd` probe and this `-device usb-storage` probe are the
  **same experiment with different leaf drivers**. Run them together (a keyboard *and* a disk
  on the same EHCI bus) and one boot answers both.
- **If USB comes alive, real input and real storage unlock from one investment.** That shared
  leverage is the reason to prioritize the USB-host-stack probe over any per-driver work.

**arm64 also needs real interrupts eventually.** Like the input path, a production USB stack on
arm64 wants GIC interrupt routing, but `hal-rust/src/arm64/halstubs.rs:404 HalGetInterruptVector`
is a stub `{ 0 }`. A polled bring-up can prove enumeration first (as `arm64input` does for
input); interrupt-driven USB is the follow-on. This is a shared arm64 dependency with the input
quest, not a storage-specific one.

---

## References

- **RAM-disk shortcut:** `qemu/freeldr.ini`, `qemu/mkramdisk.sh`,
  `sdk/lib/rust/ramdisk-rust/src/ramdisk.rs` (`RamdiskCreateRamdisk:699`), boot media in
  `qemu/run.sh:110` (virtio-blk `boot.img`).
- **x86 real storage (LiveCD):** `boot/bootdata/hivesys.inf:23` (`uniata`), `:59-63`
  (`cdrom`/`disk`/`cdfs`); `sdk/lib/rust/uniata-rust/`.
- **USB mass storage driver:** `sdk/lib/rust/usbstor-rust/src/` — `fdo.rs`, `pdo.rs`,
  `scsi.rs`, `disk.rs`, `descriptor.rs`, `queue.rs`.
- **Other storage Rust crates:** `sdk/lib/rust/{disk,classpnp,partmgr,storport,scsiport,storahci,cdfs,fastfat,fs_rec}-rust/`.
- **Shared USB host stack (the gate):** `sdk/lib/rust/{usbport,usbehci,usbohci,usbuhci,usbhub,usbd}-rust/`;
  registered in `boot/bootdata/hivesys.inf` (usb*).
- **Sibling quest (shared beachhead + arm64 interrupt dependency):**
  `docs/engineering/input-architecture-and-unification.md`.
- **arm64 HAL interrupt stub:** `sdk/lib/rust/hal-rust/src/arm64/halstubs.rs:404`.
