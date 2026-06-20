# smoltcp Net Stack — Design

**Date:** 2026-06-07 (design) · **Updated:** 2026-06-20 (M7 arm64 net data path — core GREEN; one TX bug open)
**Status:** **M0–M6 DONE + ON MAINLINE** (`c4911ea7c68`, master + continue-rust-port) — the
C `tcpip`/`lwip` stack is removed; `netstack.sys` (smoltcp) is the whole stack, boot-green on
all 3 arches. **Usermode winsock is now GREEN end-to-end** (ws2_32 → msafd → AFD → netstack →
smoltcp → slirp; TCP + UDP, incl. real public-internet TCP; `test_net.py` **11/11** on amd64).
**M7 (arm64 net data path) — CORE GREEN on arm64, one TX bug open.** arm64 was staged-but-inert;
now (on `netstack-arm64-m7`, `arm64-net` config) netstack.sys loads, the new arm64-HAL ECAM
enumerates the NIC, the virtio handshake + DMA work, and the smoltcp core runs: **M1 ARP, DHCP
lease, ICMP ping, and TDI device ownership are all GREEN on arm64.** Required fixing two things:
the arm64 HAL `HalGetBusData*` no-op stubs (real ECAM PCI config), and the arm64 kernel skipping
ALL Start=1 drivers (it now explicitly loads the netstack service). OPEN follow-up: virtio-net
**TX wedges after ~7 frames** on arm64 (`can_send()` goes false and stays false → DNS + outbound
TCP never egress); the worker is healthy (1.85M-iter heartbeat) and RX works — it's a
virtio-drivers TX-queue reclaim bug, not a freeze. See the M7 Remaining-open entry for detail.
The earlier "usermode-winsock blocker root-caused to a *storage* driver" was superseded — the
true blockers were the amd64 kernel **timer-table base skew** (`38b63f15b74`) and a kernel32
**`IsBad*Ptr` DUMMY_PSEH** regression (`b5f42d40421`: IsBad{Read,Write,String}Ptr reported every
valid pointer as bad → ws2_32 `connect()` short-circuited to WSAEFAULT). Both fixed and landed.
**Lane:** L4 (Rust-ecosystem-crate rewrite) of `docs/engineering/translation-banking-roadmap.md`
**Goal:** Replace the ReactOS C TCP/IP protocol engine with the Rust **`smoltcp`** crate,
reaching a **usable networked desktop in QEMU**, while keeping the Windows-compatible
socket surface (ws2_32 → AFD) unchanged.

This is an **L4 rewrite, not a C→Rust translation.** We are not translating
`tcpip/lwip` (115K LOC) or `tcpip/ip` (8K LOC); we replace that engine with smoltcp and
write the glue. It does not move the "C LOC translated" metric — it moves "usable
desktop in Rust."

---

## Status (2026-06-13)

**The networking stack is done.** `netstack.sys` (`netstack-rust` + `netstack-cargo`,
KernelDll cargo cutover) replaces the C `tcpip`/`lwip` stack end-to-end: TCP stream +
UDP datagram transport behind the real AFD/TDI event-indication model, virtio-net
backend, DHCP/DNS/ping. Lane = worktree `vibeos-wt-netstack`, branch `netstack-port`.

- **M0–M6 all landed.** The old C stack (`ip/` + `lwip/` + C TDI dispatch, ~129K C) is
  removed from the build; the net suite (`tests/qemu/test_net.py`) is folded into the
  smoke verdict as the amd64 `net` stage.
- **Cold 3-arch verified** (`netstack.sys` builds+links on i686/x86_64/aarch64):
  amd64 cold build + boot + net (10 pass / 1 skip), i386 boot, arm64 boot (netstack.sys
  staged into the ramdisk). All boot-green.
- **Merged onto current crp** (merge `810d513d3c8`) and re-verified green ×3.
- **Self-test gate:** the throwaway in-kernel TDI self-tests now skip when there is no
  NIC (no DHCP) — otherwise their DISPATCH-level UART flood drops adjacent boot markers
  and false-hangs no-NIC boots. They still run+pass on the net config.

**Usermode `ws2_32` `socket()` — RESOLVED.** It needs the Winsock2 protocol catalog
(msafd providers) in the system hive; the `netstack_winsock.inf` bakes it in and is now
**registered on all 3 arches** (`drivers/network/netstack/CMakeLists.txt`). The earlier
"baking the catalog trips BUGCHECK 0x50 → uniata/scsiport storage bug" theory was a red
herring: the catalog's size shift merely *exposed* an uninitialized `PortConfig` on a
scsiport find-loop retry, fixed in `451ec29d08e`. With that fixed the catalog registers
cleanly and `test_usermode_winsock_roundtrip` asserts (no skip) — `socket()` →
`WSAEINVAL` is gone; the full ws2_32→msafd→AFD→netstack round-trip is GREEN.

**M7 — arm64 net data path (core GREEN; one TX bug open).** arm64 loaded `netstack.sys`
nowhere and enumerated no NIC; three things were fixed on `netstack-arm64-m7`: (1) the arm64
HAL's `HalGetBusData*` were no-op stubs, so `find_virtio_net` (HalGetBusDataByOffset over
PCIConfiguration) could not see the PCIe bus — now a real ECAM implementation for the QEMU
virt GPEX bridge; (2) the arm64 kernel's `IopInitializeSystemDrivers` skipped ALL Start=1
drivers (ZwLoadDriver hangs on a service whose `.sys` is missing from the minimal ramdisk) —
now it explicitly loads the (present) netstack service; (3) `livecd_hives`+`netstack` wired
into the arm64 `ArchBuild`. Plus the `arm64-net` QEMU config (virtio-net-pci on the virt PCIe
bus). Verified on arm64: netstack loads, NIC enumerates, virtio+DMA work, **M1 ARP / DHCP /
ICMP ping / TDI device ownership GREEN.** Remaining (M7 follow-up): virtio-net TX wedges after
~7 frames (`can_send()` stuck false), blocking DNS + outbound TCP — see Remaining-open below.

---

## Settled decisions (2026-06-07)

1. **Splice point = A: smoltcp lives *behind the existing TDI device interface*.** AFD,
   msafd, ws2_32 are untouched; the TDI contract (`\Device\Tcp` / `\Device\Udp` /
   `\Device\Ip`, `TDI_*` ops + events) is preserved. We replace tcpip's *internals*.
2. **Device backend = virtio-net first** (via `virtio-drivers`), **bypassing NDIS** for
   the QEMU target. **NDIS is scoped in this doc but deferred** — the `Device` trait
   keeps it swappable; we ship virtio, add an NDIS `Device` later for real NICs.
3. **UDP/DHCP/DNS need no second crate** — smoltcp has `udp`, `icmp`, `raw`, `dhcpv4`,
   `dns` sockets. ReactOS's existing **usermode** dhcp/dns services keep running, now
   over smoltcp's UDP. (smoltcp's own `dhcpv4`/`dns` sockets remain a fallback.)
4. **De-risk with a throwaway engine-bringup self-test first** (smoltcp + virtio-net +
   poll loop proven in-kernel before the TDI/AFD surface is touched).
5. **A QEMU network test harness is a prerequisite, not an afterthought** — smoke has
   zero net coverage today.

---

## Where it sits

```
   ws2_32 / msafd (usermode)                         UNCHANGED
        │ IOCTL_AFD_{BIND,CONNECT,SEND,RECV,...}
   AFD  drivers/network/afd            5.8K C         UNCHANGED
        │ TDI  (TdiOpenAddress/Connection; TDI_SEND/RECEIVE/CONNECT; TDI_EVENT_*)
   ┌───────────────────────── netstack.sys (NEW, Rust) ─────────────────────────┐
   │  TDI device interface  (Rust reimpl of the TDI dispatch — the upward API)   │
   │        │  socket bridge / completion engine                                 │
   │  smoltcp  Interface + SocketSet  (tcp/udp/icmp/raw/dhcpv4/dns)              │
   │        │  smoltcp::phy::Device                                              │
   │  Device backend:  virtio-net (virtio-drivers)   ──[NDIS Device: deferred]── │
   └────────────────────────────────────────────────────────────────────────────┘
        │ virtqueue (QEMU virtio-net-pci)
   QEMU host network (slirp / tap)
```

**Replaced (C, deleted from the build):** `tcpip/ip` (~8K) + `tcpip/lwip` (~115K) +
the `tcpip/tcpip` TDI dispatch (~6.4K, reimplemented in Rust) + `datalink/lan.c` NDIS
glue (deferred path). **~129K C → smoltcp + ~a few K Rust glue.**
**Untouched:** AFD (5.8K), tdihelpers (used by AFD), msafd/ws2_32, the NDIS library and
miniports (dormant on the virtio path; revived when the NDIS `Device` lands).

> **Sub-decision (TDI dispatch):** reimplement the TDI dispatch in Rust as part of
> `netstack-rust` (clean all-Rust driver). Fallback if bringup is painful: keep the C
> `tcpip/tcpip/dispatch.c` as a thin shim that FFIs down into the Rust engine. Default
> to the Rust reimpl; the completion bridge *is* most of the dispatch anyway.

---

## In-kernel runtime model (the core engineering)

smoltcp is **poll-driven and single-owner** — there is no internal thread and it is not
internally synchronized. We provide the runtime:

- **One owner, one lock.** A global `Interface + SocketSet + port-state` guarded by a
  spinlock/fast-mutex. All access (poll, socket ops, RX indication) goes through it.
- **A poll driver.** A dedicated worker thread (or DPC chain) that calls
  `iface.poll(now, &mut device, &mut sockets)` and is woken by:
  (a) **RX available** (virtio used-ring interrupt → DPC → signal),
  (b) **a socket op** arriving from the TDI layer (connect/send/recv/…),
  (c) **a timer** for smoltcp's own deadlines — `iface.poll_delay(now, &sockets)` says
  when to wake next (retransmit/keepalive/TIME-WAIT).
- **Time + randomness via FFI.** `smoltcp::time::Instant` from `KeQueryInterruptTime`;
  ISN / ephemeral-port randomness from `RtlRandomEx`.
- **The completion bridge — the heart of it.** After every `poll()`, walk the sockets
  and reconcile against pended TDI requests/events:
  - TCP `can_recv` / UDP datagram queued → complete a pending `TDI_RECEIVE` /
    `TDI_EVENT_RECEIVE`.
  - `can_send` / tx buffer space → complete a pending `TDI_SEND`.
  - state → `Established` → complete `TDI_CONNECT`; incoming SYN on a listener →
    `TDI_EVENT_CONNECT` / accept-queue pop.
  - `Closed`/`CloseWait`/RST → disconnect events.
  This is the impedance match between smoltcp's poll model and Windows' IRP-pending
  /overlapped model.
- **State smoltcp does NOT provide (Redox's lesson) — we own it:**
  - **port-in-use table** (smoltcp doesn't enforce local-port uniqueness),
  - **ephemeral port allocator** (smoltcp doesn't assign on `bind(0)`),
  - **listen/accept queue** semantics.
- **Buffers:** copy `virtio rx desc → smoltcp socket ring` and `ring → tx desc`. No
  zero-copy in v1.

---

## Device backend

**v1 — virtio-net via `virtio-drivers`:**
- Implement the crate's `Hal` trait once: DMA alloc (contiguous, cache-coherent),
  phys↔virt translation, share/unshare. This `Hal` is reusable for *all* virtio devices
  (block/gpu/input) — the broader virtio lever.
- Wrap `VirtIONet` in a `smoltcp::phy::Device`: `receive()` yields an `RxToken` whose
  `consume()` hands smoltcp the packet bytes; `transmit()` yields a `TxToken` that copies
  smoltcp's frame into a tx buffer and posts it.
- Medium = Ethernet; MTU from virtio config.

**Deferred — NDIS `Device`:** a second `Device` impl that bridges to the NDIS protocol
shim (`NdisSend` / `ProtocolReceivePacket`, `NDIS_PACKET` ↔ bytes), reviving the
existing miniports (netkvm/e1000/rtl8139) for real-NIC / non-virtio breadth. Same upward
code; only the `Device` swaps. Out of scope for v1, kept architecturally open.

---

## Crate integration

- **`netstack-rust`** workspace rlib (`no_std` + `alloc`; we have a kernel allocator).
  smoltcp deps with features: `medium-ethernet`, `proto-ipv4`, `socket-tcp`,
  `socket-udp`, `socket-icmp`, `socket-raw`, (`socket-dhcpv4`/`socket-dns` optional),
  `alloc`, `defmt`/`log` off. No `std`.
- **`netstack-cargo`** drives the final `netstack.sys` link (KernelDll, `DriverEntry`)
  via `reactos-link`, same pattern as the device-driver cargo cutovers
  (`project_device_driver_cargo_cutover`). Imports: ntoskrnl, hal.
- **FFI substrate:** the time/random/DMA/`Hal` glue + the TDI dispatch surface live in
  `netstack-rust` (driver_ddk-style DDK decls, `feedback_driver_ddk_substrate`).
- Subject to the workspace `lto=false` + `codegen-units=256` captures(none) firewall.

---

## De-risk: throwaway engine bringup (Phase 0 of this lane)

Before any TDI/AFD work, prove the engine in isolation:
1. `Hal` impl + `VirtIONet` up; confirm RX/TX of raw frames in QEMU.
2. smoltcp `Device` wrapper + `Interface` (static IP first, then DHCP).
3. The poll/lock/timer loop in a kernel worker thread.
4. A **kernel self-test**: reply to ICMP echo (ping from host), and a TCP echo on a
   fixed port — driven directly against the smoltcp `SocketSet`, no TDI/AFD.

Green here means the hard, novel part (smoltcp + virtio + in-kernel poll runtime) works.
Only then build the TDI bridge on a known-good engine.

---

## Testing (prerequisite)

- **QEMU net harness:** add a virtio-net NIC + host networking (`-netdev user,...`
  slirp, or tap) to the smoke/CLI QEMU config (a net-enabled variant, not the default
  boot smoke). slirp gives us a built-in gateway/DHCP/DNS forwarder — ideal for a guest
  that should DHCP a lease and resolve a name.
- **Checks, escalating:** (1) self-test ping reply; (2) DHCPv4 lease obtained; (3) TCP
  connect + echo to a host listener; (4) DNS A-record lookup; (5) usermode winsock app
  (ws2_32) round-trip — the real end-to-end proof that AFD/TDI compat held.
- Wire as a dedicated net test in `tests/qemu/`, separate from boot smoke (keep smoke's
  meaning intact; net is its own signal).

---

## Scope

**v1 (usable desktop):** IPv4 only; TCP + UDP + ICMP(ping) + DHCPv4 client + DNS; one
NIC (QEMU virtio-net) + loopback; usermode dhcp/dns services over smoltcp UDP.
**Later:** IPv6; the NDIS `Device` (real NICs); multi-NIC/routing; zero-copy; offloads.

---

## Milestones

All milestones landed (commits on `netstack-port`):

| # | Milestone | Gate | Status |
|---|---|---|---|
| M0 | QEMU net harness + net test scaffold (red) | harness boots with virtio-net attached | ✅ `c9ef0e5d3e1` |
| M1 | `Hal` + `VirtIONet` raw frame RX/TX | host sees guest frames | ✅ `5f477ca4f67` |
| M2 | smoltcp `Device` + `Interface` + poll loop; **ICMP/TCP-echo self-test** | host ping reply + TCP echo (no TDI) | ✅ `56704464f0d` |
| M3 | DHCPv4 lease + DNS from kernel self-test | lease obtained, name resolved | ✅ `988624f0daa` |
| M4 | TDI dispatch (Rust) + completion bridge; **TCP via AFD/ws2_32** | TDI TCP round-trip via the real AFD path | ✅ `45ce3ffdb8e` (kernel TDI self-test; usermode winsock blocked on the storage 0x50) |
| M5 | UDP via AFD; usermode dhcp/dns services over smoltcp | UDP datagram round-trip via the AFD datagram-event model | ✅ `b9127eb3e7c` (transport done; usermode-winsock vehicle built `349043e91b9`, blocked on the storage 0x50) |
| M6 | `netstack.sys` cargo cutover; old `tcpip` ip/lwip removed from build | net green; boot smoke green ×3 arches | ✅ `0ad0e4e59dd` (+ crp merge `810d513d3c8`) |
| M7 | arm64 net data path: ECAM PCI config in arm64 HAL + Start=1 load fix + `arm64-net` config | netstack loads + enumerates NIC + ARP/DHCP/ICMP/TDI green on arm64 | 🟢 core GREEN (`a210ee02856`+`ab442a59122`); ⚠️ TX wedges after ~7 frames → dns/outbound-tcp open (M7 follow-up) |

The kernel TDI self-tests (`tditest.rs` TCP, `tdidgtest.rs` UDP) drive netstack's own
`\Device\Tcp`/`\Device\Udp` through the *exact* IRP path AFD uses, so M4/M5 are proven at
the TDI boundary. The only unproven hop is usermode `ws2_32`→AFD, gated on the storage
0x50 (see Status).

---

## Risks / open items

Original risks — how each resolved:
- ~~**Completion-bridge correctness** (poll-model ↔ IRP-pending)~~ — ✅ proven at M4/M5;
  `tditrans.rs` `bridge()` completes pended connect/recv IRPs and upcalls the AFD
  `TDI_EVENT_RECEIVE`/`RECEIVE_DATAGRAM` handlers. The M4 round-trip was initially a false
  positive (packed-vs-aligned `TDI_ADDRESS_IP` + an unchecked wait status) — both fixed.
- ~~**TDI fidelity**~~ — ✅ we implemented exactly the subset AFD issues (ASSOCIATE/CONNECT/
  SEND/RECEIVE/SET_EVENT/SEND_DATAGRAM/RECEIVE_DATAGRAM), driven by reading AFD's usage.
- ~~**Arch coverage**~~ — ✅ builds+links on all 3 arches; boots green ×3. (Net data path is
  amd64-only by config; i386/arm64 load netstack with no NIC and skip cleanly.)
- ~~**smoltcp version pinning / no_std build-std**~~ — ✅ smoltcp 0.13.1 + virtio-drivers
  0.13.0 build under the nightly kernel target jsons.
- ~~**Locking under DPC/IRQL**~~ — ✅ the worker is the sole socket owner, draining a
  spinlock'd inbox at PASSIVE; `IrqlGuard::raise_to_dpc()` gives UP mutual exclusion
  (note: `KeAcquireSpinLockForDpc` is a no-op in this tree — don't rely on it).
- ~~**Boot smoke must stay green**~~ — ✅ folded into the smoke verdict; the self-test gate
  ensures a missing/idle NIC never stalls boot.

Remaining open:
- ~~**Usermode-winsock storage 0x50** (task 79)~~ — ✅ **RESOLVED + landed.** The real
  ws2_32 round-trip is GREEN (`test_net.py` 11/11). The blockers were *not* a storage driver:
  (1) the amd64 kernel **timer-table base skew** (`38b63f15b74`) made every relative
  KeDelay/KeWait over-wait ~4–60× → multi-RTT TCP connect timed out; (2) a kernel32
  **`IsBad*Ptr` DUMMY_PSEH** regression (`b5f42d40421`) made IsBad{Read,Write,String}Ptr
  report every valid pointer as bad → ws2_32 `connect()` short-circuited to WSAEFAULT before
  reaching msafd (UDP was unaffected — only connect()/rnr.c validate via IsBad*Ptr).
- **M7 — arm64 net data path: CORE GREEN, one TX bug open (`netstack-arm64-m7`).** Was:
  netstack.sys staged on arm64 but Start=1 never ran and no NIC enumerated. Fixed + verified
  on `arm64-net` (virtio-net-pci on the virt GPEX bus): (1) real ECAM PCI config in the arm64
  HAL (`hal-rust/src/arm64/halstubs.rs` — `HalGetBusData*` were no-op stubs); (2) the arm64
  kernel skipped ALL Start=1 drivers (ZwLoadDriver hangs on a missing-from-ramdisk service) —
  now `IopInitializeSystemDrivers` explicitly ZwLoadDrivers the (present) netstack service on
  arm64 (`ab442a59122`); (3) `livecd_hives`+`netstack` wired into the arm64 `ArchBuild`. Boot
  result: `netstack.sys` loads, ECAM enumerates the NIC, virtio handshake + DMA work, and the
  smoltcp core runs — **M1 ARP, DHCP lease, ICMP ping, TDI device ownership all GREEN on arm64.**
  - **OPEN (M7 follow-up): virtio-net TX wedges after ~7 frames on arm64.** DNS (to 10.0.2.3)
    and outbound TCP (to 10.0.2.100) never complete: after ARP/DHCP/ICMP-to-gateway, the guest
    transmits NOTHING more (confirmed by `filter-dump` pcap — no ARP/DNS/SYN frames leave).
    Root-caused: the worker is healthy (heartbeat ran to 1.85M iters, NOT a freeze; RX works),
    but `VirtIONet::can_send()` (`send_queue.available_desc() >= 2`) goes false after ~7 sends
    and stays false → TX descriptors are not replenished. NOT memory-ordering (identical under
    `tcg,thread=single` and `=multi`), NOT DNS/slirp (no frame ever egresses). Next: debug the
    virtio-drivers 0.13 `VirtIONet`/`add_notify_wait_pop` TX-queue reclaim accounting on arm64.
  - **OPEN (M7 landing-blocker): arm64 boot hangs at a netstack thread exit (`LpcExitThread`).**
    Because netstack now loads on *all* arm64 boots (Start=1 fix), the regular arm64 smoke
    (default QEMU NIC) hangs when a netstack TDI self-test thread exits — last serial is
    `[PS] PspExitThread: about to call LpcExitThread`, never reaching the desktop. arm64-net
    reached the desktop, so it may be timing-dependent. Likely a pre-existing arm64 kernel
    thread-exit/LPC bug that netstack is the first to trigger (first driver to spawn + exit
    system threads during arm64 boot). Must be resolved before M7 lands (it would regress the
    regular arm64 smoke). Triage with the kernel-debug toolkit (break in `LpcExitThread`).
- **Later (v1+):** IPv6; NDIS `Device` for real NICs; multi-NIC/routing; zero-copy; offloads;
  server-side TCP (TDI_LISTEN/TDI_ACCEPT — netstack is outbound-client-only today).

---

## Cross-references
- Net test suite: `tests/qemu/test_net.py` (`--arch amd64-uefi-net`); folded into
  `vibeos-cli smoke --arch amd64` as the `net` stage
- Storage 0x50 blocker: `docs/engineering/translation-debugging-field-manual.md` §9.9 +
  `drivers/network/netstack/CMakeLists.txt` comment (task 79)
- Roadmap / lanes: `docs/engineering/translation-banking-roadmap.md` (L4)
- Driver cargo-cutover recipe: memory `project_device_driver_cargo_cutover`
- DDK substrate for drivers: memory `feedback_driver_ddk_substrate`
- smoltcp: <https://docs.rs/smoltcp> · phy::Device, socket::{tcp,udp,icmp,raw,dhcpv4,dns}
- virtio-drivers: <https://github.com/rcore-os/virtio-drivers> (Hal trait, VirtIONet)
- Prior art: Redox netstack/smolnetd — port-set + ephemeral-port + per-op UDP endpoint lessons
