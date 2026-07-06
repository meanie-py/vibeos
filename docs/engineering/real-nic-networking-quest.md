# Real-NIC networking — eliminating the virtio-net VM shortcut

**Status:** light quest brief / scaffold, opened 2026-07-05. **Lower priority** than the
input and storage quests — networking is *not* on the boot-to-desktop critical path, so
this is a "flesh out later" doc, not a ready-to-implement plan. Sibling of the USB-HID
input quest (`docs/engineering/input-architecture-and-unification.md`); same "delete the
VM-only shortcut" class of problem.

**One-line goal:** feed the smoltcp netstack from a **real, interrupt-driven NIC miniport**
(e.g. e1000) instead of driving a paravirtual **virtio-net** device directly — so
networking works on a physical machine, not only inside QEMU.

---

## The current shortcut: virtio-net + polling

Today the whole guest netstack rides a **virtio-net-pci** NIC, which is a *paravirtual,
hypervisor-only* device — it does not exist on real hardware. And the fast path **polls**
rather than taking interrupts.

- **netstack-rust drives virtio-net directly, bypassing NDIS.** `device.rs:50 find_virtio_net()`
  does the PCI find + bus-master enable + virtio handshake and hands smoltcp a `VirtioPhy`
  (`device.rs:1-6`); the backend is the `virtio-drivers` crate
  (`netstack-rust/Cargo.toml:29`), explicitly "bypassing NDIS" (`Cargo.toml:4-5`). PCI
  access goes over HAL bus-data exports (`pci.rs`), portable across arches.
- **The engine polls.** `engine.rs:40` spawns a poll-loop worker; `engine.rs:153
  iface.poll(...)` runs on a `delay_ms` cadence (`engine.rs:319 poll_delay`). This is a
  busy-poll fallback, arch-independent.
- **Interrupt-driven RX exists, but only on amd64, and still only for virtio.** `isr.rs`
  (M10) connects the virtio **INTx** line via `IoConnectInterrupt` + `HalGetInterruptVector`
  (`isr.rs:100 connect`, `isr.rs:120`) so an RX interrupt wakes the worker
  (`isr.rs:68 isr` → `isr.rs:83 dpc` → event). **arm64 stays on pure polling** because its
  `HalGetInterruptVector` is a stub returning 0 (`isr.rs:18-19`). So even the "interrupt"
  path today is virtio-specific and x86-only.

The smoke configs bolt a virtio NIC onto each arch: `virtio-net-pci,disable-legacy=on`
in `tests/qemu/vm.py:621` (amd64), `:655` (i386), `:765` (arm64). These are opt-in net
variants (`amd64-uefi-net` / `i386-uefi-net` / `arm64-net`), not the default smoke.

**Net-net:** the transport is a VM-only device and (on the arch that matters for real
hardware first, x86) the interrupt path is wired only for virtio. On real silicon there is
no virtio-net NIC to find at all.

---

## The real-hardware target

A physical NIC (Intel e1000/e1000e, Realtek rtl8139/rtl8168, etc.) is an **interrupt-driven
PCI device** with an **NDIS miniport** driver. The target shape:

```
real NIC (e1000)  →  NDIS miniport (ISR + DPC, real IRQ)  →  NDIS protocol shim  →  smoltcp
```

The netstack already anticipates this: an **`ndis` Cargo feature (M11)** swaps `VirtioPhy`
for an **`NdisPhy`** smoltcp `Device` (`netstack-rust/Cargo.toml:17-24`, `lib.rs:27-31`,
`ndisphy.rs`, `ndis.rs`) that sits as an NDIS *protocol* above a real miniport instead of
touching virtio directly. It is **OFF by default** and the comment flags the gap plainly:
"Runtime bring-up (ndis.sys + a miniport on the boot path) is the remaining sub-task"
(`Cargo.toml:23-24`). So the seam is designed but not lit.

---

## What exists vs. what's missing

**Exists:**
- Real NIC **C** miniports in the tree, and they build: `drivers/network/dd/{e1000,rtl8139,netkvm}`
  (`drivers/network/dd/CMakeLists.txt`; `add_subdirectory(dd)` in `drivers/network/CMakeLists.txt:3`).
  e1000 is a proper interrupt-driven NDIS miniport — it has `interrupt.c`, `hardware.c`,
  `send.c`, `ndis.c`, `nete1000.inf` (`drivers/network/dd/e1000/`).
- The **NdisPhy / M11 shim** in netstack-rust (feature-gated, `ndisphy.rs` + `ndis.rs`).
- A working **interrupt→DPC→wake** pattern to copy: `isr.rs` already does
  `IoConnectInterrupt`-driven RX for virtio on amd64 (proves the HAL/IRQ plumbing works on x86).

**Missing / gaps:**
- **No Rust NIC miniport.** None of e1000/rtl8139/netkvm is translated to Rust — there is no
  `e1000-rust` (nor rtl8139/netkvm) crate in `sdk/lib/rust/`. The only NIC-adjacent Rust is
  `netstack-rust` itself (which is the *stack*, not a miniport).
- **The C miniports aren't on the boot path.** No `e1000`/`rtl8139`/`netkvm` registration in
  `boot/bootdata/hivesys.inf` — they build but nothing loads them at boot, so nothing binds
  an NDIS protocol to them today.
- **The NdisPhy path is unexercised.** The `ndis` feature has no smoke and, per its own
  comment, no runtime bring-up (no `ndis.sys` + miniport wired into a boot). It is
  compile-gated scaffolding, not a proven data path.
- **arm64 has no interrupt route at all** (next section).

---

## Dependency: arm64 is blocked on GIC interrupt routing

A real NIC is fundamentally **interrupt-driven**, and on arm64 there is no way to route a
device interrupt yet:

- `hal-rust/src/arm64/halstubs.rs:404 HalGetInterruptVector` is a **stub `{ 0 }`**; the GIC
  is touched only for the timer (`arm64/clock.rs`). This is the *same* blocker the input
  quest calls out (`input-architecture-and-unification.md:35-37`) — everything on arm64
  polls because nothing can take a device IRQ.
- Consequently **this quest is downstream of the arm64-interrupts work** on arm64. Until the
  GIC + `HalGetInterruptVector` + `IoConnectInterrupt` path exists, an arm64 real NIC would
  have to poll — which is exactly the shortcut we're trying to remove.

**x86 is unblocked and should go first.** The x86 HAL has real PIC/APIC routing
(`hal-rust/src/x86/{pic_pic,apic,busemul}.rs`; `HalGetInterruptVector` is real), and `isr.rs`
already demonstrates `IoConnectInterrupt` working for a PCI NIC on amd64. So an
interrupt-driven **e1000 on amd64** is reachable without waiting on arm64.

---

## Rough first step (scaffold — flesh out before building)

Prove the real-NIC data path on the arch where interrupts already work (amd64):

1. Boot QEMU with an **emulated e1000** instead of (or alongside) virtio-net:
   `-device e1000,netdev=net0` on the `amd64-uefi-net` config (a variant of `vm.py:621`).
2. Bring up a **real interrupt-driven e1000 miniport** and feed smoltcp through the
   **NdisPhy/M11** seam (enable the `ndis` feature; light the "ndis.sys + miniport on the
   boot path" sub-task the Cargo comment defers). The e1000 ISR→DPC pattern mirrors the
   virtio one already in `isr.rs`.
   - Rust-vs-C decision for the miniport is open: either translate `drivers/network/dd/e1000`
     to an `e1000-rust` crate (aligns with the port's Rust goal), or first prove the path by
     loading the existing **C** e1000 miniport under the NdisPhy shim. Pick when this doc is
     fleshed out.
3. Success marker: smoltcp completes DHCP / an ICMP echo (as the virtio self-tests do,
   `engine.rs:159` / `icmptest.rs`) **through e1000 + a real IRQ**, with no virtio device
   present.
4. Only then carry the pattern to arm64 — gated on the arm64 GIC/interrupt work landing.

---

## References
- netstack (virtio backend + poll loop): `sdk/lib/rust/netstack-rust/src/{device.rs,engine.rs,pci.rs}`.
- Interrupt-driven RX (virtio, amd64; the pattern to reuse): `netstack-rust/src/isr.rs`.
- NDIS/real-NIC seam (M11, feature-gated): `netstack-rust/src/{ndis.rs,ndisphy.rs}`,
  `netstack-rust/Cargo.toml:17-24`, `lib.rs:27-31`.
- Real NIC C miniports (build, not on boot path): `drivers/network/dd/{e1000,rtl8139,netkvm}/`.
- HAL interrupt vectoring: `sdk/lib/rust/hal-rust/src/x86/{busemul,pic_pic,apic}.rs` (real),
  `hal-rust/src/arm64/halstubs.rs:404 HalGetInterruptVector` (stub — the arm64 blocker).
- Smoke net configs: `tests/qemu/vm.py:577` (amd64), `:633` (i386), `:729` (arm64).
- Netstack design + milestones (incl. M11 real-NIC note): `docs/engineering/smoltcp-netstack-design.md`.
- Sibling quest (same "eliminate VM-only shortcut" class, shares the arm64-interrupts blocker):
  `docs/engineering/input-architecture-and-unification.md`.
