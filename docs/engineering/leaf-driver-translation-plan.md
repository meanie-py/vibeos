# Non-FS Leaf-Driver Translation Plan (storage + serial + HID)

**Date: 2026-06-20.** Branch `filesystems` (post FS-backlog close). Author: filesystems lane.

This is the audit + rustconv-tier strategy for the **9 remaining C-only non-FS leaf
drivers** flagged in `rust-port-status-rollup.md` ("Non-FS drivers — ~20K LOC, 9 C-only").
It is the storage/serial/HID analogue of the FS-backlog work: cohab C→Rust, validated
link-clean ×3, none boot-critical.

## TL;DR — the headline that changes the cost math

The naive read is "~20K LOC of hard hardware drivers." The real picture is better:

1. **The expensive FFI substrate already exists.** `hid-sys` (3.2K), `usb-sys` (39K
   bindgen), `usbd-rust`/`usbdex-rust` (USBD.SYS body, done), `hidparse-rust`+`hidparser-rust`
   (HidP_* done), and `scsiport-rust` (122K, done) are all in-tree and wired. The HID stack
   and serial reuse substrate that's already paid for. Only **storport** needs net-new FFI
   (and it can lean on `scsiport-rust`'s SRB types). This is the same leverage that made
   msfs/npfs cheap once `ntkrnl-sys` existed — except here it's *already* built.
2. **None are boot/smoke-critical.** QEMU storage = IDE/ATAPI (uniata/pciide/atapi, all
   done); QEMU input = PS/2 → i8042prt/kbdclass/mouclass (all done). So like msfs/npfs,
   the cohab flips are validated by **link-clean ×3**, not smoke. Optional functional
   validation needs a QEMU `-device usb-kbd`/`usb-tablet` or `-device buslogic` opt-in.
3. **2 of the 9 are drop/defer candidates** (storahci already disabled; buslogic = legacy
   BT-958 SCSI, never touched by QEMU) — same disposition logic as the nfs/udfs/vfatfs drop.

Net: the *translatable, worth-keeping* set is **7 drivers (~17K LOC)**, most of it MEDIUM,
with substrate already in place for the hardest cluster (HID/USB).

## Inventory (ground-truth, measured from tree)

| Driver | Path | C LOC | Files | In boot media? | QEMU-exercised? | Cohab? |
|---|---|---|---|---|---|---|
| **serial** | `drivers/serial/serial` | 2.7K | 12 | only as KD `/DEBUGPORT=COM1` (kernel owns it, not serial.sys) | no | none |
| **ramdisk** | `drivers/storage/class/ramdisk` | 2.5K | 1 | `ramdisk.sys` + hivesys | no (boot ramdisk = freeldr, not this) | none |
| **hidclass** | `drivers/hid/hidclass` | 2.8K | 4 | `hidclass.sys` + hivesys | only with USB HID | none |
| **hidusb** | `drivers/hid/hidusb` | 2.0K | 1 | `hidusb.sys` + hivesys | only with USB HID | none |
| **kbdhid** | `drivers/hid/kbdhid` | 1.1K | 1 | `kbdhid.sys` + hivesys | only with USB kbd | none |
| **mouhid** | `drivers/hid/mouhid` | 1.3K | 1 | (mouhid.sys) + hivesys | only with USB mouse | none |
| **storport** | `drivers/storage/port/storport` | 3.6K | 8 | `storport.sys` | no (no storport miniport in QEMU path) | none |
| **storahci** | `drivers/storage/port/storahci` | 3.3K | 2 | **disabled** (`#add_cd_file` commented) | no | none |
| **buslogic** | `drivers/storage/port/buslogic` | 4.6K | 7 | `buslogic.sys` (BT-958) | no (would need `-device buslogic`) | none |

## Difficulty characterization (per driver)

### HID cluster — substrate exists, so cheaper than LOC implies
Dependency order: **hidclass first** (foundational), then **hidusb** (minidriver under
hidclass), then **kbdhid + mouhid** in parallel (clients on top of hidclass).

- **hidclass** (HARD): PnP state machine (START fetches the descriptor chain) + nested IRP
  completion routines (`HidClass_ReadCompleteIrp`, IRP reuse via `IoReuseIrp`) + spinlock/
  event sync. ~25-30 DDK fns. *Substrate: `hid-sys` covers hidport/hidclass/hidpi.*
- **hidusb** (HARD): the genuinely hardest — async **URB** construction/submission
  (`UsbBuild*`, `USBD_CreateConfigurationRequestEx`, `IOCTL_INTERNAL_USB_SUBMIT_URB`),
  work-item-based pipe/port **reset error-recovery state machine**, multi-step descriptor
  acquisition with size negotiation. *Substrate: `usb-sys` + `usbd-rust` already done.*
- **kbdhid / mouhid** (MEDIUM, near-twins): async read loop (`IoReuseIrp` +
  completion routine re-arms the read) + HID report parsing via `HidP_*` (→ `hidparse-rust`,
  done) + callback dispatch to class driver at DISPATCH_LEVEL. mouhid is slightly harder
  (absolute/relative branching + wheel feature detection). No URBs, no work items.

### Storage cluster
- **ramdisk** (EASY): the clean one. Class driver, **no miniport / no ISR / no DPC / no
  hardware register I/O**. PnP+Power IRP dispatch, remove-locks, symbolic links, work items,
  registry config. Self-contained, single file. *Substrate: existing `driver_ddk`.*
- **storport** (HARD): the modern port-driver **provider interface** — miniport callbacks
  (`HwInitialize/HwStartIo/HwInterrupt/HwFindAdapter`), ISR + DPC with KIRQL juggling,
  **C-variadic** `StorPortNotification` dispatch, SRB handling, ~100 DDK fns, PnP state
  machine, dense `StorPortRead/WriteRegister`. *Substrate: net-new storport FFI needed;
  reuse `scsiport-rust` SRB/SCSI types.*
- **storahci** (HARD, **DEFER**): AHCI MMIO + DMA/PRDT + FIS protocol + bitfield-union
  registers. **Already disabled from shipping** — translate only if/when re-enabled.
- **buslogic** (VERY HARD, **DROP candidate**): legacy BT-958 SCSI HBA — CCB↔SRB impedance,
  128-entry scatter-gather mailbox queue, WMI (`wmi.c`), variadic logging. Not used by any
  QEMU target. Highest effort, lowest value of the set.

### serial (MEDIUM overall — high LOC, low conceptual depth)
12 files. The bulk is **mechanical**: `devctrl.c` (880 LOC = 37 `IOCTL_SERIAL_*` handlers,
repetitive), UART register I/O (8250/16550), circular buffer. The genuinely tricky files
are `misc.c` (ISR + 3 DPCs + interlocked error counters + WaitOnMask race) and `rw.c`
(work-item read with `KeWaitForMultipleObjects` timeout state machine) and `pnp.c`
(resource parse + `IoConnectInterrupt` + registry). *Substrate: `ntddser` via `driver_ddk`;
`kdcom-serial-rust` is a ready reference for the UART register patterns.* Self-contained.

## rustconv model-tier strategy

**First, the tool question:** use **rustconv (LLM, cohab-aware)**, *not* the deterministic
`c2rust` transpiler. `c2rust` emits unsafe machine Rust against its own libc-style bindings
— it has no concept of our cohab model (`#[no_mangle]` symbol takeover, per-arch NTAPI ABI,
DUMMY_PSEH lowering, the `ntkrnl-sys`/`hid-sys`/`usb-sys` substrate), so its output would
need total rework to link into a `.sys`. rustconv already produces cohab-ready faithful Rust.
"opus/sonnet/deepseek + c2rust" in practice means **rustconv driven by that model**.

**Tier principle:** API cost across all 9 drivers is *low tens of dollars* either way (a
~3K-LOC driver is single-digit dollars on Sonnet sync, low-tens on Opus). The dominant cost
is **integration + BUGCHECK debugging**, and the defect classes that cost multi-hour hunts
(DUMMY_PSEH live-except bodies, `extern "system"` vs `"C"` ABI drift, cast-before-offset,
scrambled flag literals — all catalogued in the field manual) cluster in the **async/ABI-
heavy** drivers. So: **spend model strength where a translation defect is expensive to find,
save it where the code is mechanical.** Minimize *defects*, not API dollars.

| Driver | Tier | Model | Why |
|---|---|---|---|
| ramdisk | EASY | **DeepSeek-reasoner** + `semdiff` vs C (fallback Sonnet) | self-contained, no async/ISR/HW; cheapest path, semdiff catches drift |
| serial (easy/mid files) | MEDIUM | **Sonnet** sync per-function | mechanical register I/O + IOCTL dispatch; Sonnet proven faithful (cdfs/msfs/npfs) |
| serial (`misc.c`/`rw.c`/`pnp.c`) | HARD-ish | **Opus** | ISR/DPC race + timeout state machine + interrupt connect — defect-expensive |
| kbdhid | MEDIUM | **Sonnet** | async read loop + HidP_* (substrate done); twin of mouhid |
| mouhid | MEDIUM | **Sonnet** | ditto + abs/rel branch |
| hidclass | HARD | **Opus** | nested IRP completion + PnP descriptor chain |
| hidusb | HARD | **Opus** | URB async + work-item reset state machine — the single hardest |
| storport | HARD | **Opus** | miniport provider + variadic notify + ISR/DPC |
| storahci | HARD | **DEFER** | disabled from shipping |
| buslogic | VERY HARD | **DROP candidate** | legacy, not in QEMU, highest effort/lowest value |

**Why not DeepSeek more widely?** DeepSeek's win was *mechanical data-table* lift (NLS
casemap/collation) verified by `semdiff`. Driver logic with ABI/PSEH traps is exactly where
a weaker model silently introduces the expensive defect classes. Use it only for the EASY,
self-contained tier (ramdisk) and pair it with `semdiff`. **Why not Sonnet-batch?** Batch
"silently stubs lazy fns" on larger files — use sync `translate` (per-function) for anything
>1000 LOC, which is most of these.

## Recommended execution order (substrate-cost-ordered, cheapest leverage first)

1. **ramdisk** — EASY, single file, existing substrate. Proves the cohab harness for this
   driver family end-to-end. DeepSeek+semdiff.
2. **kbdhid + mouhid** — MEDIUM, substrate (`hid-sys`+`hidparse-rust`) already done; near-twin
   pair, good parallel-subagent batch. Sonnet.
3. **hidclass** — HARD but unblocks the HID cluster's class layer. Opus.
4. **hidusb** — HARD, the URB driver; last of the HID cluster. Opus.
5. **serial** — MEDIUM bulk + 3 hard files; self-contained, no substrate gap. Sonnet+Opus split.
6. **storport** — HARD, needs net-new FFI (build `storport-sys` leaning on `scsiport-rust`).
   Biggest substrate cost; do after the cheaper wins land. Opus.
7. **storahci** — DEFER (disabled). **buslogic** — DROP pending user confirmation.

Each driver: parallel-subagent translation per-file over a CONTRACT.md (write `.rs` to
`~/vibeos-rust-staging/`, no shared-file edits), integrate + cohab-flip + build + link ×3
centrally — the msfs/npfs playbook. Validation = **link-clean ×3** (these don't boot in
smoke); functional check is an optional QEMU USB/SCSI-device opt-in, flagged not silently
skipped.

## Scope decision (settled 2026-06-20, user)

- **storahci**: **DEFER** (keep the C, translate later). It's the modern real-hardware AHCI
  path — valuable to have in Rust eventually — just currently disabled/unfinished in ReactOS.
  Translate when/if AHCI is wired up. Do **not** delete.
- **buslogic**: **DROP** from translation scope (legacy BT-958 SCSI, ~4.6K LOC VERY HARD,
  zero QEMU exposure). Recoverable from ReactOS + rustconv later, same logic as the
  nfs/udfs/vfatfs drop. (Physical deletion vs leave-as-inert-C is a follow-up nicety; it
  links today.)
- **Translate (7):** ramdisk, serial, hidclass, hidusb, kbdhid, mouhid, storport.

## Execution log

- 2026-06-20: HID cluster started (user pick). Substrate verified ready — `hid-sys`
  (HIDP_*), `hidparse-rust` (HidP_* exports), and `ntkrnl-sys` bindgen already cover the
  KEYBOARD_*/MOUSE_*/CONNECT_DATA/HIDP_KEYBOARD_MODIFIER_STATE types; only `USAGE_AND_PAGE`
  (trivial 2×USAGE struct) defined locally. First batch: kbdhid + mouhid (MEDIUM twins).
- 2026-06-21: ✅ **kbdhid + mouhid DONE — link-clean ×3** (`786c5303ac1`, branch `filesystems`).
  2 parallel Sonnet subagents → central integration. `kbdhid-rust` (8 modules) + `mouhid-rust`
  (8 modules) cohab staticlibs; `.c` kept on disk. Both `.sys` build + link clean on i386/
  amd64/arm64, lint-clean (no unprotected output writes; mouhid's lone ud2 is a benign
  divide-by-zero guard, not a captures(none) fold). New field-manual entry: amd64 cohab-driver
  IRQL raise/lower (tier1 `*Internal` unlinkable on amd64; per-crate CR8-asm helper). Next:
  hidclass (Opus) → hidusb (Opus). Awaiting meaniepy ×3 cold-smoke gate (non-boot → link +
  no-regression bar, same as msfs/npfs).
