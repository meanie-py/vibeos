# smoltcp Net Stack — Design

**Date:** 2026-06-07
**Status:** design (pre-implementation)
**Lane:** L4 (Rust-ecosystem-crate rewrite) of `docs/engineering/translation-banking-roadmap.md`
**Goal:** Replace the ReactOS C TCP/IP protocol engine with the Rust **`smoltcp`** crate,
reaching a **usable networked desktop in QEMU**, while keeping the Windows-compatible
socket surface (ws2_32 → AFD) unchanged.

This is an **L4 rewrite, not a C→Rust translation.** We are not translating
`tcpip/lwip` (115K LOC) or `tcpip/ip` (8K LOC); we replace that engine with smoltcp and
write the glue. It does not move the "C LOC translated" metric — it moves "usable
desktop in Rust."

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

| # | Milestone | Gate |
|---|---|---|
| M0 | QEMU net harness + net test scaffold (red) | harness boots with virtio-net attached |
| M1 | `Hal` + `VirtIONet` raw frame RX/TX | host sees guest frames |
| M2 | smoltcp `Device` + `Interface` + poll loop; **ICMP/TCP-echo self-test** | host ping reply + TCP echo (no TDI) |
| M3 | DHCPv4 lease + DNS from kernel self-test | lease obtained, name resolved |
| M4 | TDI dispatch (Rust) + completion bridge; **TCP via AFD/ws2_32** | usermode winsock TCP round-trip |
| M5 | UDP via AFD; usermode dhcp/dns services over smoltcp | services get lease/resolve through the socket layer |
| M6 | `netstack.sys` cargo cutover; old `tcpip` ip/lwip removed from build | net test green ×(arches in scope); boot smoke still green |

---

## Risks / open items

- **Completion-bridge correctness** (poll-model ↔ IRP-pending) is the deepest risk —
  M4 is where it's proven.
- **TDI fidelity:** AFD exercises a specific subset of TDI; reimplementing dispatch in
  Rust must match what AFD actually issues (read AFD's TDI usage, don't implement the
  whole spec).
- **Arch coverage:** virtio + DMA `Hal` must be correct per-arch (i386/amd64/arm64
  DMA/coherency). Bring up on one arch (amd64) first, then generalize.
- **smoltcp version pinning** + `no_std` build under our nightly build-std target jsons.
- **Locking under DPC/IRQL:** the global lock is taken from DPC (RX) and passive
  (socket ops) — IRQL discipline (no blocking at DISPATCH_LEVEL) must be designed in.
- **Boot smoke must stay green** — netstack is not boot-critical; ensure a missing/idle
  NIC doesn't stall boot.

---

## Cross-references
- Roadmap / lanes: `docs/engineering/translation-banking-roadmap.md` (L4)
- Driver cargo-cutover recipe: memory `project_device_driver_cargo_cutover`
- DDK substrate for drivers: memory `feedback_driver_ddk_substrate`
- smoltcp: <https://docs.rs/smoltcp> · phy::Device, socket::{tcp,udp,icmp,raw,dhcpv4,dns}
- virtio-drivers: <https://github.com/rcore-os/virtio-drivers> (Hal trait, VirtIONet)
- Prior art: Redox netstack/smolnetd — port-set + ephemeral-port + per-op UDP endpoint lessons
