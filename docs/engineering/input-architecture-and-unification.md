# Input architecture & the path to a unified, real-hardware input stack

**Status:** design / quest brief, opened 2026-07-05.

**One-line quest:** prove the (already-translated) Rust **USB HID** stack can enumerate a
QEMU `usb-kbd`/`usb-mouse` and deliver a keystroke to `kbdclass` — because USB HID is the
*single* input path that is simultaneously arch-unified, correctly-layered, and viable on
real hardware. The enumeration probe is a day's work and decides the whole direction.

---

## Why this exists

Today keyboard/mouse input takes **two structurally different paths** depending on arch,
and the arm64 path is both VM-only and architecturally malformed. This doc maps the stack,
shows exactly where the divergence lives, and argues that **USB HID** (not virtio-input,
not PS/2) is the convergence target — then frames a cheap beachhead probe that tells us
whether that target is reachable.

**The whole live input path is already Rust.** Every layer below has been cohab-flipped
C→Rust (C kept on disk as dead residue per the never-delete rule; shipping `.sys` are
cargo-linked from `*-rust` crates). So this is **not a translation task** — it's a
Rust-side architecture refactor.

---

## The end-to-end path (with the arch split)

### Layer 0 — HAL / interrupt vectoring  *(Rust; arch-gated, but arm64-input doesn't use it)*
- Dispatch: `hal/CMakeLists.txt:3-6` — `halx86` (i386/amd64) vs `halarm64`, both pure-Rust
  from `hal-rust` (`sdk/lib/rust/hal-rust/src/lib.rs:54-63`: shared `x86/`, separate `arm64/`).
- **x86 real vectoring:** `hal-rust/src/x86/busemul.rs:245 HalGetInterruptVector` (ISA IRQ →
  vector/IRQL/affinity); PIC `x86/pic_pic.rs`, APIC `x86/apic.rs`. This makes i8042 IRQ1/IRQ12
  routable.
- **arm64:** `hal-rust/src/arm64/halstubs.rs:404 HalGetInterruptVector` is a **stub `{ 0 }`**.
  There is no GIC input routing in the HAL; the GIC is touched only for the timer
  (`arm64/clock.rs`). **arm64 input never traverses the HAL interrupt path at all** — it polls.

### Layer 1 — Miniport  *(Rust; THIS is where the divergence lives)*
- **x86 = `i8042prt`** (`sdk/lib/rust/i8042prt-rust`), interrupt-driven PS/2:
  `pnp.rs:436-462` reads the translated `CmResourceTypeInterrupt`; `pnp.rs:226`/`:270`
  `IoConnectInterrupt` for kbd IRQ1 / mouse IRQ12 → ISRs `keyboard.rs:717
  i8042KbdInterruptService` / `mouse.rs:809`; ISR queues a DPC (`keyboard.rs:91
  KeInsertQueueDpc`) that calls the class callback in `KeyboardData.ClassService`
  (`keyboard.rs:331-338`, the cdecl `SERVICE_CALLBACK_ROUTINE`).
- **arm64 = `arm64input`** (`sdk/lib/rust/arm64input-rust`), **polled** virtio-mmio, built
  arm64-only (`drivers/input/CMakeLists.txt` `if(ARCH STREQUAL "arm64")`): **no interrupt
  registration at all** — a polling thread `arm64input.rs:1420 PsCreateSystemThread` →
  `:1288 VirtioPollThread` → `:1196 VirtioPollDevice`, MMIO via `:338 VirtioRead32`, queue
  `:763 VirtioSetupQueue`, into internal rings (`:261-266 Ring/MouRing`, `:358 RingPush`).

### Layer 2 — Class drivers  *(Rust; arch-agnostic code — but bypassed on arm64)*
- `kbdclass` (`sdk/lib/rust/kbdclass-rust`) / `mouclass` (`mouclass-rust`) are fully
  arch-agnostic. Standard handshake: `kbdclass.rs:565-588` builds
  `CONNECT_DATA{ClassService=ClassCallback}` and sends `IOCTL_INTERNAL_KEYBOARD_CONNECT`
  (`:574`); the miniport later invokes `ClassCallback` (`:491`) to deliver packets. kbdclass
  owns `\Device\KeyboardClass0` and services the RIT's `ZwReadFile`.
- **The malformation:** on arm64 the class drivers are **bypassed**. `arm64input` itself
  creates `\Device\KeyboardClass0` (`arm64input.rs:1340`) + `\Device\PointerClass0` (`:1377`)
  + DosDevices symlinks and services read-IRPs from its own rings. It is a **monolith** that
  collapses miniport+class into one driver. `kbdclass`/`mouclass` are **dead weight on arm64**
  (no port driver ever issues `IOCTL_INTERNAL_*_CONNECT` there).

### Layer 3 — win32k Raw Input Thread  *(Rust; fully shared, arch-agnostic)*
- `sdk/lib/rust/win32k-rust/src/input.rs:334 RawInputThreadMain` opens class devices **by
  name** — `\Device\PointerClass0` (`:390`), `\Device\KeyboardClass0` (`:400`) — then a
  `ZwReadFile` loop (`:424`/`:446`) gated by `KeWaitForMultipleObjects` (`:466`); packets →
  `UserProcessMouseInput` (`:502`) / `UserProcessKeyboardInput` (`:512`) → `WM_*` via the
  message queue. **Identical on all arches**; the only per-arch delta is fastcall-vs-cdecl
  externs (`:58-95`), pure ABI. Because it binds by device *name*, it does not care what
  transport produced the packets.

---

## Divergence map

| Layer | i386 | amd64 | arm64 | Divergent? |
|---|---|---|---|---|
| HAL interrupt vectoring | PIC/APIC (`x86/*`) | shared `x86/` | **stub** (`arm64/halstubs.rs:404`) | Yes, but arm64 input doesn't use HAL |
| Miniport | `i8042prt` PS/2, IRQ+DPC | same | `arm64input` virtio, **polling** | **YES — core divergence** |
| Class (kbdclass/mouclass) | used | used | **bypassed (monolith)** | Structurally divergent |
| win32k RIT + message synth | shared | shared | shared | No (ABI cfg only) |

**Two axes of divergence, both in/around the miniport:** (1) *transport* — interrupt-driven
i8042 vs polled virtio; (2) *layering* — x86's proper 3-tier stack vs arm64's monolith. The
class layer and RIT are already shared Rust.

---

## The transport decision: why not just virtio-input everywhere?

Because **virtio-input is a paravirtualized VM device** — it exists only because a
hypervisor presents it. A physical machine has **no virtio-input at all**. Switching every
arch to virtio would unify the *QEMU* path but move *further* from real hardware. Real
machines have neither PS/2 (rare) nor virtio (never) — they have **USB HID**.

| Candidate "one path" | Unifies QEMU | Fixes the monolith | Real hardware |
|---|:--:|:--:|:--:|
| PS/2 (i8042prt) everywhere | ✗ (arm64 has no 8042) | — | x86 only |
| virtio-input everywhere | ✓ | ✗ (still a monolith) | **✗ VM-only** |
| **USB HID everywhere** | ✓ | **✓ for free** | **✓** |

**USB HID wins on all three at once.** USB HID *structurally* flows
`hidusb → hidclass → kbdhid/mouhid → kbdclass/mouclass → RIT`, so it cannot be a monolith —
it uses the shared class spine by construction. One move gives a single transport, correct
layering, and real-hardware viability, and QEMU emulates real USB on **every** machine type
(`-device qemu-xhci -device usb-kbd` works on q35, pc, *and* virt) — so the smoke would
exercise the identical path a physical machine uses. `i8042prt` and `arm64input` both become
deletable.

**The whole USB+HID stack is already translated to Rust:** `usbport-rust`, `usbehci-rust`,
`usbohci-rust`, `usbuhci-rust`, `usbhub-rust`, `usbd-rust`, `hidusb-rust`, `hidclass-rust`,
`kbdhid-rust`, `mouhid-rust`.

**The catch (the central unknown):** the USB HCs are *registered* in the boot hive
(`boot/bootdata/hivesys.inf`: usbhub/usbehci/usbohci/usbuhci) but there is **no evidence the
stack enumerates a device or delivers a HID report today** — nothing in the smokes drives
USB input, and there is **no `usbxhci`** in the tree (only EHCI/OHCI/UHCI, USB2-era). So the
stack may be translated-but-never-exercised. That is exactly what the beachhead resolves.

---

## The quest — a probe first, then a direction

### Beachhead (do this first; ~a day)
Boot QEMU (start with **amd64**, the fastest toolchain) with an emulated USB keyboard and
prove the Rust USB+HID stack brings it up:
```
-device usb-ehci,id=ehci  -device usb-kbd,bus=ehci.0
# (or -device qemu-xhci + a usbxhci driver if EHCI proves dead)
```
Success = a keystroke injected via QMP `input-send-event` reaches `[KBDCLASS] first key
delivered` **through the USB path** (hidusb → hidclass → kbdhid → kbdclass), not PS/2.
Instrument enumeration milestones (HC start → port connect → device descriptor → HID report
descriptor → first input report) so a failure localizes to the exact stage.

### Decision tree
- **Enumerates → USB HID is the target.** Migrate all arches to USB HID; `i8042prt`,
  `arm64input`, *and* the monolith problem evaporate together. This is the holistic
  "eliminate the whole class of divergence" outcome.
- **Doesn't enumerate (translated-but-dead) → fall back** to the tactical cleanup: convert
  `arm64input` from a monolith into a *pure port miniport* (delete its class-device creation
  `arm64input.rs:1340-1415` + rings; honor `IOCTL_INTERNAL_KEYBOARD_CONNECT`/`_MOUSE_CONNECT`
  and call the class `ServiceCallback` from `VirtioPollDevice`, exactly as `i8042prt` does at
  `keyboard.rs:331-338`). That makes `kbdclass`/`mouclass` own the class devices on arm64 too
  and collapses divergence to "transport only" — keeping virtio for now, revisiting USB when
  the stack is alive. Optional deeper step: implement `arm64/halstubs.rs:404
  HalGetInterruptVector` + a GIC `IoConnectInterrupt` path so arm64input can switch from
  polling to ISR+DPC.

Either branch is real progress; the probe just says which.

---

## References
- Miniports: `sdk/lib/rust/{i8042prt-rust,arm64input-rust}/`.
- Class + RIT spine: `sdk/lib/rust/{kbdclass-rust,mouclass-rust}/`,
  `sdk/lib/rust/win32k-rust/src/input.rs`.
- USB/HID stack: `sdk/lib/rust/{usbport,usbehci,usbohci,usbuhci,usbhub,usbd,hidusb,hidclass,kbdhid,mouhid}-rust/`.
- HAL: `sdk/lib/rust/hal-rust/src/{x86,arm64}/`.
- Existing input-injection smoke scaffolding (delivery-level, C-era arm64 work):
  `tests/qemu/test_smoke.py::test_post_shell_ready_input` / `_mouse` / `test_arm64input_delivery`.
- Boot-hive driver registration: `boot/bootdata/hivesys.inf` (usb*), `hivesys_arm64.inf` (arm64input).
