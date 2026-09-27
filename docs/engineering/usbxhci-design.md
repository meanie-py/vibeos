# usbxhci — design

A USB 3 (xHCI) host-controller driver, so a keyboard and mouse work on a physical
Raspberry Pi 4 (whose only host ports are behind a VL805 xHCI) and on modern PCs,
which often have no EHCI at all. `docs/product/driver-roadmap.md` rates xHCI
**Critical**.

**Status (2026-09-26): M1-M4 done.** `usbxhci.sys` (`sdk/lib/rust/usbxhci-rust`,
`drivers/usb/usbxhci`) brings the controller up, enumerates USB keyboards and mice, and
their input reaches the desktop. That holds in QEMU on arm64 at `-smp 1` and `-smp 4`
(rig: `VIBEOS_QEMU_XHCI=1`, below). It also holds **on a physical Raspberry Pi 4**:
- **The path:** the VL805 → the board's VIA USB 2.0 hub → a Pico 2 W composite device.
- **The proof:** text typed on the Pico appears in Notepad, and the Pico's absolute
  pointer clicks land where aimed.

See "The Pi, as built" below.

## Where it plugs in

The USB stack above the host controller is ReactOS's USB 2.0-era design, already
ported to Rust and working (keyboard and mouse enumerate on all three arches in
QEMU): `usbport-rust` (shared port driver), `usbhub-rust`, `hidusb-rust`,
`kbdhid`/`mouhid`. Host controllers are **miniports**: they call
`USBPORT_RegisterUSBPortDriver` with a `USBPORT_REGISTRATION_PACKET` of callbacks
(start/stop controller, open/close endpoint, submit transfer, interrupt service,
root-hub port operations). `usbehci-rust` is the model to follow; the constant
`USB_MINIPORT_VERSION_XHCI = 4` already exists.

**Decision: usbxhci is a usbport miniport**, not a port of Microsoft's KMDF
USBXHCI + UCX stack. Reasons: the stack above already works; KMDF does not exist
here yet (it is its own roadmap item); and the miniport model is what the 2017
ReactOS GSoC attempt used (upstream branch `GSoC_2017/usbxhci`, ~2.5k lines of C).
That branch reached controller init, scratchpad buffers, command/event rings,
interrupts and root-hub port status, but never slot/endpoint setup or transfers,
so it never enumerated a device. It is a reference for register layout, not a
starting point: there is no working body to translate, so this is written fresh
in Rust against the xHCI 1.2 spec.

## The one real impedance mismatch: xHCI owns device addressing

usbport assumes the EHCI world: software assigns a device address by sending
`SET_ADDRESS` as an ordinary control transfer on address 0, then opens endpoints
by describing them to the miniport. xHCI instead manages devices as **slots**
with controller-owned **device contexts**:

| usbport does | the xHCI miniport translates it to |
|---|---|
| control transfer on address 0's default pipe (first use of a new device) | Enable Slot command → Address Device (BSR=1) with an input context for slot + EP0 |
| `SET_ADDRESS` setup packet on the default pipe | **intercepted**, never put on the wire: Address Device (BSR=0); report success |
| open endpoint | Configure Endpoint command with an input context adding the endpoint (type, max packet, interval, TR dequeue pointer) |
| close endpoint | Configure Endpoint dropping it; Reset/Stop Endpoint as needed |
| submit transfer | TRBs on the endpoint's transfer ring (Setup/Data/Status stages for control; Normal TRBs for bulk/interrupt), doorbell |
| interrupt / poll | walk the event ring: Command Completion, Transfer Event, Port Status Change |
| root-hub port ops | PORTSC register reads/writes, per the port's protocol (USB2 vs USB3) |

This interception is the whole trick; everything else is mechanical ring
management. The GSoC branch planned the same approach.

## Scope, in order

1. **M1 — controller alive (QEMU `-device qemu-xhci`).** Map BAR0, read the
   capability registers, halt + reset, allocate the DCBAA, scratchpads, command
   ring and one event ring (interrupter 0, MSI or line interrupt as the PCI
   resources give), run. Root hub: report ports and their speed from PORTSC.
   Evidence: `[XHCI] controller running, N ports` and a Port Status Change event
   when QEMU attaches `usb-kbd`.
2. **M2 — enumeration.** Slot + address-device translation above, EP0 control
   transfers, so usbport reads the device and configuration descriptors and
   usbhub creates the PDO. Evidence: the HID device node appears.
3. **M3 — HID input.** Interrupt-IN endpoints (Configure Endpoint + Normal TRBs);
   keystrokes reach kbdhid. This is the milestone that matters. Gate: a smoke
   mode booting with `qemu-xhci` + `usb-kbd` that types a marker.
4. **M4 — the Pi.** See below.
5. **Later.** Bulk (usbstor, i.e. booting off USB storage instead of a ramdisk),
   SuperSpeed hubs, isochronous, multiple interrupters, runtime power.

Low- and full-speed devices (keyboards, mice, the Pico bridge) sit on the xHCI's
**USB2 protocol ports**, so M1-M3 need no SuperSpeed support: usbhub only ever
sees a USB2 root hub.

## As built (what the plan above did not know)

- **usbport re-opens a pipe by closing it, freeing its buffer and opening it
  again on a new one** (`USBPORT_ReopenPipe`; it calls OpenEndpoint, not
  ReopenEndpoint). It does this to EP0 right after SET_ADDRESS. So nothing that
  must outlive that can sit in an endpoint buffer: each slot's output device
  context and EP0 ring live in the controller's common buffer (two pages per slot,
  16 slots), and a re-opened EP0 finds its slot by usbport's device address. Only
  non-default endpoints keep their ring in their own buffer.
- **The table above is right.** Open EP0 at address 0 → Enable Slot + Address
  Device (BSR=1); the SET_ADDRESS request → Address Device (BSR=0), completed
  without touching the wire; the re-open at address N → Evaluate Context if the
  max packet size changed; other endpoints → Configure Endpoint add/drop.
- **Commands wait by polling the event ring.** usbport calls OpenEndpoint and
  friends at DISPATCH_LEVEL, where the DPC cannot run on the calling CPU. The event
  consumer takes its own lock, so the DPC, the poll timer and a command wait can
  all call it.
- **Interrupts stay off until usbport asks** (EnableInterrupts after StartController).
  A completion interrupt raised during start is declined by usbport's ISR gate and,
  on a level-triggered line, storms. For the same reason pci.sys masks xHCI INTx at
  enumeration (firmware may hand the controller over still interrupting) and the
  driver unmasks it after its reset. Field manual: "A device driver's start hangs
  the box right after `[GIC] unhandled INTID N`".
- **usbport changes.** Its USB2 bandwidth scheduler dereferences a transaction
  translator for every low/full-speed periodic endpoint, which under xHCI sits on a
  root port with none (EHCI never sees one: companions take it). It now skips its
  USB2 budget for xHCI miniports, which budget in hardware. Its root hub also
  labelled an xHCI root hub descriptor as USB 3.0 (0x2A) while building the USB 2.0
  layout; it now says 0x29.
- **Control-pipe STALL is cleared before the next request** (Reset Endpoint + Set
  TR Dequeue), as EHCI hardware does by itself. Other endpoints wait for usbport's
  SetEndpointStatus(RUN). Abort turns a TD into No-Ops and moves the dequeue
  pointer past it when it was next.

### The Pi, as built

- **Publishing XHC0.** No ACPI bus driver runs on arm64. Instead the HAL's own PnP
  root (`hal-rust/src/arm64/pnpdd.rs`, the arm64 counterpart of x86's `halpnpdd`)
  enumerates the ACPI-only devices that `arm64/acpi.rs` finds in the DSDT/SSDTs.
  - **Recognition:** it matches `_HID` and reads the constant AML without evaluating
    methods.
  - **Fixup:** the one method result it cannot compute (the `_CRS` method grows
    Len from 1 to 0x1000) becomes a 4 KiB minimum window.
  - **XHC0's resources:** `ACPI\PNP0D10` with MMIO `0x6_0000_0000` + 4 KiB and GSIV
    175, level-triggered.
  - **Binding:** CDDB `ACPI#PNP0D10` → usbxhci.
  - **On QEMU** the root has no children.
- **Its PCI face.** usbport treats every parent as a PCI function. The child PDO
  serves `BUS_INTERFACE_STANDARD` from the real config window that `_INI`'s
  OperationRegion names (`0xFD508000`, VL805 1106:3483), bus 1 device 0.
  - **The `_INI` write:** at enumeration it writes COMMAND = memory + bus master +
    INTx-disable; usbxhci unmasks INTx after its reset, as on PCI.
  - **Trigger mode:** the HAL now programs level-triggered SPIs level in GICD_ICFGR.
- **Topology.** usbport's miniport interface carries only the immediate parent port
  and the TT hub's address. `USBPORT_SetXhciTopology` walks the device-handle chain
  and passes the rest in fields other miniports ignore:
  - the root port and route string, in `Reserved3`;
  - the TT hub and TT port, in `Reserved6`.

  The slot context then gets the route string, the root port, and Parent Hub Slot /
  Parent Port for a low- or full-speed device below a high-speed hub. A hub's slot
  gets Hub, port count and TT think time from its hub descriptor (usbxhci recognises
  the reply by its type byte; usbhub first asks with wValue 0).
- **VL805 firmware.** StartController's HCRST did not lose it: after the reset the
  controller runs and enumerates without the VideoCore mailbox reload Linux uses
  after a PCIe link reset. We never reset the link.
- **Non-coherent DMA** (`_CCA 0`) works on the hardware, using the explicit cache
  maintenance in `hw.rs` and the private payload arena (`usbxhci-review.md`).

### Not yet (tracked, in rough order)

1. **Interrupt polling interval.** Endpoints open with usbport's Period (32 ms for
   usb-kbd), not the descriptor's bInterval; fine for typing, sluggish for a mouse.
2. **SuperSpeed ports** (a USB 3 root hub, SuperSpeed hubs), bulk-heavy devices
   (usbstor), isochronous. USB 2.0 hubs and TT work.
3. **Slot reclamation.** usbport does not tell the miniport about removals. A
   removed device's slot is freed when usbport reuses its address (SET_ADDRESS) or a
   new device appears at the same place, not at removal. Unplug and replug across hub
   ports works on the Pi. An endpoint command failing on an unplugged device (its
   endpoints halt with split-transaction errors) affects only that device, unless
   the endpoint context still says Running.

### Rig

`VIBEOS_QEMU_XHCI=1` adds `qemu-xhci` (pinned to PCI slot 8) with `usb-kbd` and
`usb-mouse`, and on arm64 drops the virtio keyboard/mouse. QEMU gives host input to
the most recently activated handler, and virtio-input activates after usb-kbd, so it
would take every keystroke. Proof of the USB path: `[KBDHID] read completion`,
`[KBDCLASS] first key delivered` and `[WIN32K] RIT first keyboard input` after
`input-send-event`. The last one matters: kbdclass delivering a key does not mean the
desktop reads that class device (arm64input used to own `KeyboardClass0`).

## Per-arch facts

- **QEMU (all arches):** `qemu-xhci` is a PCI device; the PCI path already works
  on arm64 (`pci.sys` is staged; the EHCI USB work used it). Develop and gate
  here at `-smp 1` first, then `--smp 4`, since the ring code is a natural
  place for the SMP ordering bugs this lane has been finding.
- **Raspberry Pi 4 (verified 2026-09-26 from the board's own tables, pftf
  v1.53):** the VL805 sits behind the BCM2711 PCIe root complex, which is not
  ECAM-compliant, and the firmware publishes **no MCFG** at all. Instead SSDT
  `RPI4XHCI` describes the controller directly:
  - `\_SB.SCB0.XHC0`, `_HID "PNP0D10"` (generic xHCI), parent `SCB0` is an
    `ACPI0004` module device.
  - **MMIO** `0x6_0000_0000`, 4 KB (the `_CRS` method sets the end to base +
    `0xFFF`) - the PCIe outbound window, above 4 GB.
  - **Interrupt** GIC SPI 175 (`0xAF`), level.
  - **`_DMA`** bus addresses `0 .. 0xBFFF_FFFF`, no translation: the VL805
    reaches only the first 3 GB, which is why the firmware caps RAM at 3 GB.
  - **`_CCA 0`: DMA is NOT cache-coherent.** Rings, contexts and transfer
    buffers need non-cacheable mappings or explicit clean/invalidate around every
    hand-off. QEMU will never show a mistake here.
  - **`_INI`** maps the VL805's PCI config space at `0xFD508000` and writes
    `6` (memory space + bus master) to its command register. Nothing on arm64
    runs AML, so we must do this ourselves.

  **arm64 VibeOS has no ACPI bus driver** (`acpi.sys` is not staged), so the HAL
  publishes this device itself; see "The Pi, as built". The general answer,
  `acpi.sys` (ACPICA) on arm64, is a separate and much larger job.

## Risks worth naming now

- **Ordering.** Ring producers and the controller communicate through memory:
  TRB cycle bits, doorbells, the event ring dequeue pointer. On a weakly ordered
  CPU each needs the right barrier (`dmb`/`dsb` before the doorbell, cycle bit
  written last). QEMU will not catch a missing one; the Pi will. Structure ring
  writes behind one small audited helper.
- **Coherent DMA memory.** Rings and contexts must be allocated with
  `AllocateCommonBuffer` (usbport provides it) and mapped the way the device
  sees them, not ad-hoc pool.
- **Controller quirks.** VL805 firmware is loaded by the Pi's VideoCore at boot;
  if a controller reset loses it, Linux re-triggers a firmware load through the
  mailbox. Check whether our reset path needs the same (the GSoC driver only
  ever ran on QEMU and real Intel/AMD parts).
