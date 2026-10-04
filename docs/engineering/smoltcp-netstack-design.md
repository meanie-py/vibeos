# smoltcp Net Stack — Design

**Date:** 2026-06-07 (design) · **Updated:** 2026-06-20 (M7 arm64 net data path — core GREEN; #91/#92 root-caused to the arm64 no-preemption gap; **added the Post-Parity Roadmap (M8+)** — server TCP, management plane, interrupt-driven RX, NDIS, from a four-pass code audit)
**Status:** **M0–M6 DONE + ON MAINLINE** (`c4911ea7c68`, master + continue-rust-port) — the
C `tcpip`/`lwip` stack is removed; `netstack.sys` (smoltcp) is the whole stack, boot-green on
all 3 arches. **Usermode winsock is now GREEN end-to-end** (ws2_32 → msafd → AFD → netstack →
smoltcp → slirp; TCP + UDP, incl. real public-internet TCP; `test_net.py` was **11/11** on amd64
at the time of writing — the suite has since grown to **15** tests, so run it rather than
trusting a count here).
**M7 (arm64 net data path) — DATA PATH GREEN on arm64.** netstack.sys loads (on
`netstack-arm64-m7`, `arm64-net` config), the new arm64-HAL ECAM enumerates the NIC, virtio +
DMA work, and the **full smoltcp data path runs: ARP, DHCP, ICMP, DNS, outbound TCP, and both
TDI round-trips (TCP RECEIVE + UDP datagram) are GREEN — `test_net.py --arch arm64-net` 9/11**,
and the regular arm64 boot is stable 3/3. Three fixes were needed: (1) the arm64 HAL
`HalGetBusData*` no-op stubs → real ECAM PCI config; (2) the arm64 kernel skipping ALL Start=1
drivers → it now explicitly loads the netstack service; (3) **the killer — arm64
`KUSER_SHARED_DATA.InterruptTime` never advances** (no periodic clock interrupt → no
`KeUpdateSystemTime`), so netstack's `now()` froze smoltcp's clock and every retransmit /
new-neighbour-ARP timer stalled; fixed by reading `cntvct_el0` (`KeQueryPerformanceCounter`)
instead (`72e062966f3`). Open (narrow, not the data path): inbound tcp-echo server (#91 —
root-caused 2026-06-20 to the arm64 **no-preemption** gap: the poll worker stops polling RX at
idle, so it's resolved by arm64-love's periodic-timer quest, not a netstack bug) +
`nettest.exe` not staged for the arm64 usermode-winsock test. See the M7 Remaining-open entry.
The earlier "usermode-winsock blocker root-caused to a *storage* driver" was superseded — the
true blockers were the amd64 kernel **timer-table base skew** (`38b63f15b74`) and a kernel32
**`IsBad*Ptr` DUMMY_PSEH** regression (`b5f42d40421`: IsBad{Read,Write,String}Ptr reported every
valid pointer as bad → ws2_32 `connect()` short-circuited to WSAEFAULT). Both fixed and landed.
**Lane:** L4 (Rust-ecosystem-crate rewrite) of `docs/engineering/archive/translation-banking-roadmap.md`
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
| M7 | arm64 net data path: ECAM PCI config + Start=1 load fix + cntvct clock fix + `arm64-net` config | netstack loads + NIC enum + ARP/DHCP/ICMP/DNS/TCP/TDI green on arm64 | 🟢 **DATA PATH GREEN** — `test_net.py --arch arm64-net` 9/11 (`a210ee02856`+`ab442a59122`+`72e062966f3`); regular arm64 boot 3/3. Open: inbound tcp-echo (#91 — root-caused to arm64 no-preemption, fixed by arm64-love's timer quest) + usermode-winsock staging |

The kernel TDI self-tests (`tditest.rs` TCP, `tdidgtest.rs` UDP) drive netstack's own
`\Device\Tcp`/`\Device\Udp` through the *exact* IRP path AFD uses, so M4/M5 are proven at
the TDI boundary. The only unproven hop is usermode `ws2_32`→AFD, gated on the storage
0x50 (see Status).

---

## Post-Parity Roadmap (M8+)

With M7 closing **arch parity** (the full data path + usermode winsock green on amd64/i386/
arm64), netstack graduates from "MVP proven on three arches" to a stack that has to become
*real*: serve inbound workloads, expose the management surface the OS's own tools depend on,
stop busy-polling, and eventually drive real hardware. The milestones below are grounded in a
**2026-06-20 code audit** (four parallel research passes over AFD, the Rust TDI dispatch, the
`virtio-drivers` crate, `ndis.sys`, and the iphlpapi/services tree); each carries file:line
evidence, an effort read, and its dependency/arch notes. They are roughly priority-ordered, but
several are independent and can run in parallel or be resequenced.

> **Recommended sequencing.** **M8 (server-side TCP)** and **M9 (management plane)** are the two
> highest-value, *arch- and clock-independent* wins — do them first (they can even go in parallel;
> both live in `tditrans.rs`/`tdidev.rs` but touch disjoint dispatch arms). **M10 (interrupt-driven
> RX)** is the architectural cornerstone and is **doable on amd64 today**; its arm64 half folds in
> when arm64-love's GIC work lands (and it's the robust, preemption-independent fix for the #91/#92
> idle-poll-starvation class). **M11 (NDIS real-NIC)** is the big multi-week lane that builds on
> M10's interrupt/IRQL discipline. **M12+** is completeness/perf, done as target apps demand it.

### M8 — Server-side TCP (`TDI_LISTEN` / `TDI_ACCEPT`) — ✅ IMPLEMENTED + RUNTIME-VALIDATED (2026-06-21)

**Status.** Implemented exactly per the plan below: `TDI_LISTEN` (0x04) + `TDI_ACCEPT` (0x05)
now dispatch in `tdidev.rs`; a `Cmd::Listen` rides into the worker, which binds a fresh
`tcp::Socket` to the connection's associated local port and `.listen()`s it; `bridge()` completes
the pended listen IRP once the socket reaches `Established` (smoltcp does the handshake in place),
filling `ReturnConnectionInformation->RemoteAddress` with the peer (`fill_remote_address`).
`TDI_ACCEPT` acknowledges synchronously (the connection is already established + bound). Two
`Conn` fields added (`listen_irp`/`listen_ret`); the post-accept recv/disconnect path reuses the
existing client machinery.

**Runtime validation (✅ 2026-06-21, amd64-net).** A kernel TDI *server* self-test
(`tdisrvtest.rs`, the inbound mirror of `tditest.rs`) binds `:8888`, registers a
`TDI_EVENT_RECEIVE` handler, posts a `TDI_LISTEN` (long-window via `run_tdi_to`, since the host
connects only after boot), and on a host connect `TDI_ACCEPT`s + echoes a fixed marker. The host
side (`test_tdi_server_listen_accept`, a second hostfwd) connects, sends a probe, and reads the
echo. **Green end to end:** `listening on :8888 → ACCEPTED from 10.0.2.2:<port> → received 24 bytes
→ echoed 13 bytes — server round-trip OK`, and the full net suite is **12/12** (no regression).
This exercises the whole M8 path live: `TDI_LISTEN` bind/pend → completion bridge on `Established`
(peer address filled) → `TDI_ACCEPT` → post-accept receive-event → `TDI_SEND`. Now arch-independent
and unblocked (the clock fix landed; #91/#92 idle-starvation no longer applies).

**Why.** netstack is *outbound-client-only* at the TDI surface today — a usermode app can
`connect()` but cannot `bind()`+`listen()`+`accept()`. That blocks every inbound service: SMB/CIFS
file sharing, the RPC endpoint mapper (remote DCOM / service control / remote registry), and any
HTTP/FTP/telnet-style server.

**Key finding — much simpler than a generic TDI server.** ReactOS AFD does **not** register
`TDI_EVENT_CONNECT`. `AfdRegisterEventHandlers` (`drivers/network/afd/afd/event.c:221-271`)
registers only ERROR/DISCONNECT/RECEIVE/RECEIVE_EXPEDITED/CHAINED_RECEIVE. The server model is
**pre-posted `TdiListen` IRPs**: `AfdListenSocket` (`listen.c:225-291`) opens a connection object,
`TdiAssociateAddressFile`s it to the bound address, and posts a `TDI_LISTEN`
(IRP_MJ_INTERNAL_DEVICE_CONTROL, MinorFunction `0x04`) that **pends** until a SYN arrives;
`ListenComplete` (`listen.c:90-223`) queues the connection onto `FCB->PendingConnections` and
immediately re-posts a fresh `TdiListen` to keep a backlog slot open. `accept()` (`AfdAccept`,
`listen.c:337-406`) just hands an already-established connection object to a new socket. **So
netstack needs no event-upcall subsystem** — only the pended listen IRP.

**Delta.** `TDI_LISTEN (0x04)` + `TDI_ACCEPT (0x05)` currently fall to the `_ =>` arm in
`handle_tdi` (`tdidev.rs:482-494`) → `STATUS_NOT_IMPLEMENTED`, so `listen()` fails synchronously.
Already implemented (the client+datagram path): SET_EVENT_HANDLER, ASSOCIATE/DISASSOCIATE, CONNECT,
SEND, RECEIVE, DISCONNECT, SEND/RECEIVE_DATAGRAM.

**smoltcp ↔ TDI bridge.** smoltcp has no `accept()` — a `tcp::Socket.listen(port)` transitions
LISTEN→SYN_RECEIVED→ESTABLISHED **in place**, absorbing exactly one connection. That maps **1:1**
onto AFD's "one connection object per backlog slot": on `TDI_LISTEN`, create a `tcp::Socket`,
`.listen(local_port)` (port from the *associated address object*, via `Conn.addr_id → Addr.local_port`,
**not** an ephemeral port), store the handle + pend the listen IRP (mirror the existing
`connect_irp` field in `tditrans.rs`); in `bridge()`, when that socket reaches Established, complete
the listen IRP filling `ReturnConnectionInformation->RemoteAddress` from `socket.remote_endpoint()`
(the `TaIpFull` builder already exists, `tdi.rs:192-213`). The post-accept recv/disconnect path is
**already implemented**.

**Effort: low-to-moderate** — one focused session in `tditrans.rs` + `tdidev.rs`: 2 dispatch arms,
1-2 new `Cmd` variants, ~3 `Conn` fields, a listen branch in `apply`, a completion check in
`bridge`; `TDI_ACCEPT` can start as a SUCCESS stub. **Independent of arch + the clock fix.** Backlog
exhaustion → smoltcp RSTs the SYN (acceptable; matches Windows backlog behaviour).

### M9 — The management plane (`ipconfig` / `ping` / `getsockname` / iphlpapi)

**Status (2026-06-21).** Item (1) **`getsockname` — ✅ IMPLEMENTED.** `TDI_QUERY_INFORMATION` (0x0C)
now dispatches in `tdidev.rs`; for `TDI_QUERY_ADDRESS_INFO` it routes a `Cmd::QueryAddress` to the
worker, which resolves the local endpoint (a connection object → its socket's `local_endpoint()`;
an address object → its bound port + `iface.ipv4_addr()`) and writes a byte-exact `TdiAddressInfo`
(`ActivityCount` + `TRANSPORT_ADDRESS`, sizeof 26 — the layout msafd's `WSPGetSockName` reads,
verified against `dll/win32/msafd/misc/dllmain.c:2398-2440`) into the IRP's MDL.

Item (2) **MIB classes — ✅ IMPLEMENTED.** `handle_query_info_ex` now answers the PROTOCOL-class
(`INFO_CLASS_PROTOCOL` 0x200) queries the iphlpapi `*_reactos.c` clients issue: `IF_MIB_STATS_ID`
→ `IFEntry` (if_index/type/mtu/MAC/descr), `IP_MIB_STATS_ID` → `IPSNMPInfo` (numif/numaddr/
numroutes), `IP_MIB_ADDRTABLE_ENTRY_ID` → one `IPAddrEntry` (DHCP addr/mask/bcast on if 1),
`IP_MIB_ARPTABLE_ENTRY_ID` → one default `IPRouteEntry` via the gateway (CL_NL) / empty ARP table
(AT), plus zeroed TCP/UDP stats. The AT entity is now advertised so `hasArp()` succeeds. Because
these are **METHOD_NEITHER** (caller-context `UserBuffer`, can't be written by the worker), the
worker publishes a lock-free `{MAC, IP, mask, gw}` snapshot (`netcfg.rs`) that the synchronous
dispatch reads. All struct layouts (IFEntry/IPSNMPInfo 92B, IPAddrEntry 24B, IPRouteEntry 52B) and
the entity/class/id constants verified against `tcpioctl.h`/`tdiinfo.h`. **This unblocks
`GetAdaptersInfo`/`GetIpAddrTable`/`GetIpForwardTable` → `ipconfig`.**

Item (3) **`IOCTL_ICMP_ECHO_REQUEST` (ping) — ✅ IMPLEMENTED + RUNTIME-VALIDATED.** The
METHOD_BUFFERED IOCTL (0x00120000) now dispatches on `\Device\Ip` (`tdidev.rs handle_icmp_echo`):
it parses the `ICMP_ECHO_REQUEST` header from `SystemBuffer`, copies the payload out, and posts a
`Cmd::IcmpEcho` to the worker (`tditrans.rs`). The worker owns a lazily-created management ICMP
socket (bound to ident 0x4944, distinct from the boot self-test's 0x022b), emits an echo request
with a unique sequence number, and on the matching `EchoReply` fills the `ICMP_ECHO_REPLY` exactly
as the C transport's `ProcessEchoData` does — header at the front, echoed data appended at the END
of the buffer, `Data` a back-pointer into the caller's `UserBuffer` (a value, never dereferenced in
kernel) — then completes the pended IRP (timeout → zeroed reply + `IP_REQ_TIMED_OUT`/`STATUS_TIMEOUT`).
All layouts (`IcmpEchoRequest` 20B, `IcmpEchoReply` 28B/40B per arch) are compile-time size-locked
against `ipexport.h`. A kernel self-test (`icmptest.rs`, the path usermode `ping`/`IcmpSendEcho2`
ride) drives the IOCTL to the gateway and validates the reply; **amd64-net 13/13** with serial proof
`ICMP self-test: reply OK rtt=32ms data=20 bytes — ping round-trip OK`. **This unblocks `ping`.**

Item (4) **`IOCTL_TCP_SET_INFORMATION_EX` (route/sockopt programming) — ✅ IMPLEMENTED +
RUNTIME-VALIDATED (amd64-net).** The METHOD_BUFFERED IOCTL (0x0012_8004) now dispatches on
`\Device\Tcp` (`tdidev.rs handle_set_info_ex`): it parses `TCP_REQUEST_SET_INFORMATION_EX`
(`TDIObjectID` + `BufferSize` + `Buffer`), and for an `IPRouteEntry` default-route add/del on the
IP network-layer entity (`CL_NL_ENTITY` / `IP_MIB_ARPTABLE_ENTRY_ID`, `ire_type` `IP_FORWARD_ADD`/
`_DEL`) posts a `Cmd::SetDefaultRoute` to the worker, which reprograms the smoltcp default route
(`iface.routes_mut()`). All other well-formed SET requests (wshtcpip sockopts — `TCP_NODELAY`/
`IP_TTL`/…) are acknowledged with `STATUS_SUCCESS` rather than erroring (netstack self-manages its
L2/L3 config; AFD/wshtcpip flush these best-effort, so a hard error would surface spuriously to
`setsockopt()`/`route` callers — transport-sockopt-to-smoltcp plumbing stays roadmap M12+). A kernel
self-test (`settest.rs`, the path `createIpForwardEntry` / the usermode `route` command / dhcpcsvc
ride) issues the IOCTL to program a default route and validates the ack; **amd64-net 15/15** with
serial proof `SET_INFO self-test: route program OK — IOCTL_TCP_SET_INFORMATION_EX OK`. This unblocks
`ipconfig`/`route` address+route programming. **M9 is now complete (items 1–4).** The findings below
stand.

> **arm64 validation + the `0x50` non-issue (2026-06-26).** An earlier 1-run-each A/B (mainline
> netstack vs this branch on arm64) saw a `BUGCHECK 0x50` at explorer startup on the my-netstack
> run only, raising a suspicion that `settest.rs`'s post-DHCP IOCTL surfaced/introduced an arm64
> fault. **Disproven.** Static review of the only arm64-active new code (`handle_set_info_ex` +
> `Cmd::SetDefaultRoute` + `settest.rs`) found nothing arch-specific (the route apply is pure
> smoltcp; the IOCTL completes synchronously then posts to the PASSIVE worker). Empirically, **4/4
> fresh arm64 boots** (1 gdb-attached + 3 `vibeos-cli smoke`) reached `[EXPLORER] _tWinMain entered`
> + `[WIN32K] shell ready` with **zero bugchecks**, and the M9(4) self-test passed on arm64 every
> time (`SET_INFO self-test: route program OK`, ICMP ping round-trip OK). Against the **known
> ~1/3-nondeterministic win32k taskbar `0x50` class** (memory `project_amd64_taskbar_bugcheck_0x50`),
> P(4 greens | netstack adds a 1/3 crash) ≈ 0.20 — the single earlier crash was that pre-existing
> class caught by chance, **not** netstack. M9(4)/M10-i386/M11-scaffold are clean on arm64.

**Why.** There's a clean split in what works: **AFD/winsock-served paths work** (dnsapi resolves,
basic TCP/UDP sockets), but **everything routed through iphlpapi / `\Device\Ip` is broken** because
netstack returns `STATUS_NOT_IMPLEMENTED` for the query surface. The casualties — `ipconfig`,
`GetAdaptersInfo`, `ping`, route/ARP tables, even `getsockname` — are exactly the visible tools a
user reaches for. These are small, localized, high-leverage fixes.

**Findings.**
- **`getsockname` is broken:** `AfdGetSockName` (`afd/info.c:263`) issues `TDI_QUERY_INFORMATION`
  (TDI_QUERY_ADDRESS_INFO), but `handle_tdi` has no arm for `TDI_QUERY_INFORMATION (0x0C)` /
  `TDI_SET_INFORMATION (0x0D)` (constants exist, `tdi.rs:21-22`) → `NOT_IMPLEMENTED`
  (`tdidev.rs:482-494`). (`getpeername` happens to work — it reads a cached FCB address, no TDI call.)
- **iphlpapi is entirely broken:** built on TDI entity enumeration via `sdk/lib/tdilib`, issuing
  `IOCTL_TCP_QUERY_INFORMATION_EX` for `IF_MIB_STATS_ID` / `IP_MIB_ADDRTABLE_ENTRY_ID` / ARP+route
  table IDs (`ifenum_reactos.c:59-80`, `ipstats_reactos.c:237-276`). netstack's `handle_query_info_ex`
  (`tdidev.rs:291`) implements only ENTITY_LIST + ENTITY_TYPE; all MIB classes return
  `NOT_IMPLEMENTED` (`tdidev.rs:331-338`) → `GetAdaptersInfo`/`GetIpAddrTable`/`GetIpForwardTable`
  all fail → `ipconfig`, netsh net views, dhcpcsvc address-readback all fail.
- **`ping` is broken:** uses `IcmpSendEcho2` → `\Device\Ip` + `IOCTL_ICMP_ECHO_REQUEST`
  (`iphlpapi/icmp.c:219-250`); netstack creates `\Device\Ip` (`tdidev.rs:54`) but has no ICMP-echo
  IOCTL handler — even though the engine already does ICMP internally for self-test.

**Work — a cluster of small items.** (1) a `TDI_QUERY_INFORMATION` arm returning the bound local
`TA_IP_ADDRESS` (fixes `getsockname`); (2) the MIB classes in `handle_query_info_ex` (IF stats, addr
table, route/ARP) — **the single highest-leverage item, unblocking all of iphlpapi at once**; (3)
`IOCTL_ICMP_ECHO_REQUEST` on `\Device\Ip` (makes `ping` work, reusing the engine's ICMP); (4)
optionally `IOCTL_TCP_SET_INFORMATION_EX` for address/route programming so dhcpcsvc can configure
the interface (today the engine self-configures via its own DHCP). **Independent of arch + clock**;
each item is small and individually testable.

### M10 — Interrupt-driven RX (virtio IRQ → ISR → DPC → event)

**Status (2026-06-21). amd64 — ✅ IMPLEMENTED + RUNTIME-VALIDATED.** `isr.rs` (amd64-gated) connects
the virtio-net legacy INTx: `pci.rs read_interrupt_line` reads the routed IRQ from config 0x3C,
`HalGetInterruptVector(PCIBus, …)` maps it, `IoConnectInterrupt` wires the ISR. The ISR (DIRQL)
`ack_interrupt`s the device (a volatile read of the ISR status register — clears it + deasserts
INTx, disjoint from all queue state) and queues a DPC on QUEUE_INTERRUPT; the DPC (DISPATCH)
`KeSetEvent`s the worker's RX `SynchronizationEvent`; the worker (`engine.rs`) swaps `delay_ms` for
`KeWaitForSingleObject(rx_event, …)` with the smoltcp poll deadline as the fallback timeout (so a
pending timer still caps the wait at 10ms, but an idle stack sleeps up to 100ms and a frame wakes it
instantly). The ISR/DPC touch nothing but the ISR register + event, preserving the single-owner-at-
PASSIVE invariant. Validated on amd64-net: `M10: interrupt-driven RX connected (line 11, vector 144,
irql 9)` + `first RX interrupt serviced` (one-shot marker emitted from the DPC, so it can only fire on
a real device IRQ), and RX latency dropped (ICMP rtt 32ms→15ms vs the poll-only path).

**i386 — ✅ CODE COMPLETE (ABI-correct, builds ×3, gated-boot-safe); runtime validation blocked on a
separate i386 net-bringup gap.** `isr.rs` is now gated `any(x86_64, x86)` (was x86_64-only) — i386
shares the working x86 HAL `HalGetInterruptVector` + IOAPIC path. The fix was ABI: the ISR
(`PKSERVICE_ROUTINE`) and DPC (`PKDEFERRED_ROUTINE`) callbacks are NTAPI = **stdcall on i386**, cdecl
on amd64 — so both are `extern "system"` now and passed to `IoConnectInterrupt`/`KeInitializeDpc` via
the same `transmute` idiom engine.rs uses for `PKSTART_ROUTINE` (a plain `Some(fn)` is nominally
distinct from the bindgen stdcall alias and won't coerce). The interrupt path only activates when a
NIC is present, so the **gated i386 boot smoke (no NIC) is unaffected** — `find_virtio_net()` returns
None → no ISR connect → polling fallback. An `i386-uefi-net` QEMU config + the `test_interrupt_driven_rx`
i386 enablement were added to validate it, BUT **i386-net does not boot yet**: the virtio-net bringup
hangs in `VirtIONet::new` (the modern virtio device reset / feature negotiation) **before any DMA
alloc** — `PciTransport::new` maps all 4 capability BARs fine, then the handshake spins. This is the
**i386 net data path being brought up for the very first time** (default i386 smoke has no NIC, so the
self-test always skipped) — the i386 analogue of the multi-fix arm64 M7 bringup lane, NOT an M10
regression. So i386 interrupt-RX is ready and will validate for free once i386 virtio bringup works
(a separate net-bringup milestone). arm64 keeps polling (blocked, see below).

**Why.** The worker busy-polls with `delay_ms` today. Interrupt-driven RX buys lower CPU + latency,
is **mandatory for real NICs** (you cannot poll real hardware efficiently), and is the
**preemption-independent, robust fix** for the arm64 idle-poll-starvation class (#91/#92) — a
DPC-set event wakes the worker without the periodic clock.

**Findings.** `virtio-drivers` already exposes the primitive: `Transport::ack_interrupt()` returns
`InterruptStatus` (QUEUE_INTERRUPT bit), re-exported as `VirtIONet::ack_interrupt()` +
`enable/disable_interrupts()` (`virtio-drivers` `transport/mod.rs:69-72,191-205`,
`device/net/dev.rs:45-55`) — netstack just never calls it. The device is already on **legacy INTx**
(PciTransport never configures MSI-X; `enable_bus_master` leaves INTx enabled, `pci.rs:90`) so it
*does* assert on used-ring advance. The kernel has the full plumbing — `IoConnectInterrupt` +
`HalGetInterruptVector(PCIBus,…)` + `KeInitializeDpc`/`KeInsertQueueDpc`/`KeSetEvent`/
`KeWaitForSingleObject` (all in `ntkrnl-sys` bindgen) — with **`scsiport-rust/src/fdo.rs:410-470`
as a working copy-paste model**.

**Design.** An RX `SynchronizationEvent`; ISR (at DIRQL) = `dev.ack_interrupt()`, if QUEUE_INTERRUPT
queue the DPC + return TRUE (else FALSE for the shared INTx line); DPC (DISPATCH) = `KeSetEvent`;
worker loop swaps `delay_ms` (`engine.rs:268`) for `KeWaitForSingleObject(&rx_event, …, &timeout)`
where the **event is the primary wake** and the `poll_delay()` value is the fallback timeout. The
ISR/DPC **only ack + signal**, never touch the Interface/SocketSet — preserving the single-owner-at-
PASSIVE invariant.

**Arch split + effort.** **amd64: doable now, ~1-2 days** — `HalGetInterruptVector` works on x86
(`hal-rust/src/x86/bussupp.rs:1848`), IOAPIC routing works (scsiport/i8042 use it); add a
config-space `0x3C` Interrupt-Line read to `pci.rs` + the ISR/DPC/event module + the `engine.rs`
wait swap. **arm64: blocked on arm64-love** — `HalGetInterruptVector` is a stub returning `0`
(`hal-rust/src/arm64/halstubs.rs:393-402`), the GIC/GPEX-INTx→SPI routing is unimplemented
(net-new HAL work outside netstack's lane). Note: on arm64 even the `KeWaitForSingleObject`
*timeout* fallback rides the dead clock, so there the event must be the **sole** wake. Risk:
`HalGetInterruptVector(PCIBus)` must return the routed vector for the manually-enumerated NIC
without a PnP resource assignment (netstack enumerates PCI itself in `pci.rs`, so it must read the
Interrupt Line/Pin from config space).

### M11 — Real-NIC support via an NDIS protocol shim

**Owed by M11 (2026-10-03): put the miniport INFs back.** While netstack drives the NIC
itself, `vendor/netkvm/CMakeLists.txt` does not stage netkvm.inf and
`drivers/network/dd/e1000/nete1000.inf` has its 100E/100F models commented out. With them on the
media, setup bound the NDIS miniport to the same device under netstack and the arm64 boot
wedged in netstack's TX wait (field manual, "Completing the media wakes up code that a missing
file was holding back"). When netstack moves above NDIS, restore both.

**Status (2026-06-21) — 🟡 SCAFFOLD LANDED (compile-verified ×3, behind `feature = "ndis"`, OFF by
default); runtime bring-up is the remaining sub-task.** The swappable backend the architecture
promised is now built and integrated:
- **`ndisphy.rs` — the `NdisPhy` smoltcp `phy::Device` + the DISPATCH-safe lock-free RX ring** (the
  design's named "hard part"): a single-producer/single-consumer ring in non-paged `.bss` whose
  producer (`ProtocolReceive[Packet]` at DISPATCH) only **copies frame bytes in — no smoltcp, no
  pageable alloc** — and whose consumer (the PASSIVE worker) drains it. This is exactly lan.c's
  `LanSubmitReceiveWork` model, preserving the single-owner-at-PASSIVE invariant.
- **`ndis.rs` — the NDIS protocol FFI + lifecycle**: NDIS40 protocol characteristics, the
  `NdisRegisterProtocol`/`NdisOpenAdapter`/`NdisSend`/`NdisRequest` imports + the ~13 `ProtocolXxx`
  callbacks (faithful NTAPI signatures), the register→bind→open→OID-MAC-query lifecycle, RX both
  ways (lookahead-`Receive` and packet-mode `ReceivePacket` walking the `NDIS_BUFFER`/MDL chain into
  the ring), and TX marshaling a flat frame into an `NDIS_PACKET` + `NDIS_BUFFER` chain for `NdisSend`
  with `SendComplete` recycling.
- **`engine.rs` seam**: a `cfg(feature = "ndis")` swap of `device::bringup()→VirtioPhy` for
  `ndisphy::bringup()→NdisPhy` at the worker's device construction (the interrupt path stays gated to
  the virtio backend; NDIS RX rides `ProtocolReceive`). The default build is untouched — virtio is the
  shipping path and `ninja netstack` ×3 + the gated boot smoke are unaffected; the `ndis` feature is
  compile-verified for all 3 kernel targets via `cargo build --features ndis` (a staticlib leaves the
  `NdisXxx` externs unresolved, as expected — they resolve at the `netstack.sys` link once an ndis
  import lib is wired).

**Remaining sub-task (the multi-week part):** get ndis.sys + one miniport (e1000/rtl8139/netkvm) onto
the QEMU boot path, add the ndis import lib to the `netstack-cargo` link, flip `feature = "ndis"` on,
and debug the live binding. That is its own lane with its own bring-up bugs (design unchanged below).

**Why.** Drive actual hardware + non-virtio QEMU models (e1000/rtl8139) by sitting netstack as an
NDIS *protocol* driver above existing miniports, instead of driving virtio directly.

**Findings — better than expected.** `ndis.sys` **builds** (`drivers/network/ndis`, registered in
the NDIS boot group, `hivesys.inf:1742`) and three QEMU-relevant miniports are present and kept as
translation targets — **e1000, rtl8139, netkvm** (`drivers/network/dd/*/ndis.c`, all call
`NdisMRegisterMiniport`). The old binding survives intact as a reference:
`drivers/network/tcpip/datalink/lan.c` (full ProtocolReceive/Send/TransferData,
`NdisRegisterProtocol` at :1688, `NdisOpenAdapter` at :1478, `NdisSend` at :1063). The netstack seam
is clean: smoltcp's `phy::Device` is `VirtioPhy` (`device.rs:77-104`); an `NdisPhy` is a drop-in
sibling, swapped at `engine.rs:74`.

**The hard part — IRQL.** `ProtocolReceivePacket` runs at **DISPATCH_LEVEL** (`lan.c:427`) but the
engine is a single PASSIVE worker that solely owns the Interface+SocketSet and allocates freely;
smoltcp/the global allocator **cannot** run from the receive callback. So RX must be a lock-free,
non-paged handoff: the callback only copies bytes into a ring (no smoltcp, no pageable alloc) and
the worker drains it — exactly lan.c's `LanSubmitReceiveWork` model. Plus NDIS_PACKET/buffer-chain
↔ flat `&[u8]` marshaling both ways and a per-arch (i386 stdcall `_Name@N`) FFI surface of ~6
NdisXxx exports + ~8 ProtocolXxx callbacks.

**Options/effort.** (a) full NDIS protocol shim — works with all in-tree miniports, **substantial
multi-week lane** (get ndis.sys + one miniport onto the boot path = its own sub-task with its own
bugs; then the Rust FFI + `NdisPhy` + DISPATCH-safe RX ring + `NdisSend`/SendComplete recycling);
(b) more direct Rust backends per card (~1 card/week, doesn't scale); (c) virtio-only (current).
The architecture already supports (a) cleanly (swappable `Device`, lan.c reference) — the cost is
integration breadth, not redesign. **Depends on M10's interrupt/IRQL discipline.**

### M12+ — Completeness & performance (as target apps demand)

- **Transport sockopts plumbed to smoltcp:** `TCP_NODELAY`, `SO_KEEPALIVE`, `SO_BROADCAST`
  enforcement, `SO_REUSEADDR` semantics (today stored in AFD's FCB only, not pushed to the
  transport — `WSPSetSockOpt` even FIXME-comments this, `msafd dllmain.c:3100`).
- **Multicast/IGMP** (`IP_ADD_MEMBERSHIP`) and **raw sockets** (`\Device\RawIp` is created at
  `tdidev.rs:54` but has no handler; SOCK_RAW unsupported end-to-end).
- **`ConnectEx`/`AcceptEx`:** `SIO_GET_EXTENSION_FUNCTION_POINTER` is UNIMPLEMENTED in `WSPIoctl`
  (`msafd`) — needed by modern async apps.
- **IPv6:** smoltcp already supports it; flip on dual-stack.
- **Performance:** zero-copy RX/TX (copies every frame today, `device.rs`), checksum/TSO/GSO
  offload, multi-queue.

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
  - **✅ FIXED — arm64 DNS + outbound TCP + TDI round-trips (`72e062966f3`).** ROOT CAUSE: the
    arm64 **system clock doesn't advance** — `KUSER_SHARED_DATA.InterruptTime` sits at 0 (no
    periodic clock interrupt is wired to `KeUpdateSystemTime` on this port). netstack's `now()`
    read it, so smoltcp's whole clock was frozen: reply-driven exchanges (DHCP/ICMP) worked, but
    anything needing time to pass — neighbour-cache ARP for a NEW on-link host, ARP/DNS/TCP
    retransmit timers — never fired, so DNS (10.0.2.3) + outbound TCP (10.0.2.100) stalled (the
    fresh ARP never egressed). Confirmed with a dual probe: over `delay_ms(500)`, KUSER advanced
    0 while the cntvct perf counter advanced ~35ms. FIX: arm64 `now()` reads the ARM generic
    timer (`cntvct_el0` via `KeQueryPerformanceCounter`, a real 62.5 MHz counter) instead. Result:
    `test_net.py --arch arm64-net` **9/11** (was 5/7) — DNS, outbound TCP, **TDI TCP RECEIVE OK**,
    **TDI UDP datagram** all GREEN. (NB: `delay_ms`/`KeDelayExecutionThread` still rely on the
    dead InterruptTime so they don't honour the requested duration — the proper kernel/HAL fix is
    to wire a periodic arm64 clock interrupt → `KeUpdateSystemTime`, which would benefit ALL arm64
    timing; tracked separately.)
  - **✅ FIXED (same commit) — the regular arm64 boot is stable again.** The earlier "boot hangs
    at `LpcExitThread`" was NOT a hang there (`PspExitThread` runs to completion); it was a timing
    perturbation from netstack's TDI self-tests 10s-timing-out because the dead clock froze their
    wait timers. With `now()` fixed the timers work, the self-tests finish fast, and the regular
    arm64 smoke reaches the desktop **3/3** (was flaky).
  - **ROOT-CAUSED (2026-06-20) — `test_tcp_echo_roundtrip` (#91) is downstream of the arm64
    no-preemption gap, NOT an independent netstack bug.** The host→guest *inbound* echo times out
    because the poll worker **stops polling the RX queue once the system goes idle**. Mechanism,
    proven by instrumentation (loop heartbeat + virtio-RX inbound-SYN sniffer, one arm64-net boot):
    the echo socket reaches `Listen` and never advances; the inbound SYN is **never pulled off the
    virtio RX virtqueue** (sniffer fired 0×) and the loop **stops iterating at idle** (heartbeat
    fired 0×). Why: on arm64 `KeDelayExecutionThread` is hard-coded to `NtYieldExecution()`
    (`ke/wait.rs:401`) — it doesn't wait, it yields — and with no periodic clock interrupt there is
    **no preemption**, so at idle the yield hands the CPU to the idle thread and the netstack worker
    is never re-scheduled (during boot it resumes only because boot I/O constantly re-enters the
    dispatcher; at idle desktop nothing does). netstack's virtio is **poll-mode** (no RX IRQ), so
    packet arrival generates no CPU event either. Reply-driven paths (DHCP/ICMP/DNS/outbound-TCP/TDI)
    all completed during the busy boot window, which is why only the *unsolicited* inbound SYN fails.
    **→ arm64-love's periodic-timer work (the [[arm64-clock-handoff]] quest) gives the worker a
    quantum re-schedule and resolves #91 for free; re-test inbound after it lands.** The only
    self-contained alternative is an IRQ-driven-RX refactor (wire the NIC's GIC interrupt → DPC →
    event-driven worker via `KeWaitForSingleObject`, which wakes from a DPC-set event without the
    periodic clock) — substantial, and it overlaps arm64-love's interrupt-routing domain. A naive
    busy-poll is NOT viable: the worker runs from DriverEntry (before the desktop), so never yielding
    would starve boot itself. **(b)** `test_usermode_winsock_roundtrip`
    — the whole usermode winsock stack is absent from the arm64 ramdisk: `afd.sys`, `msafd.dll`,
    `mswsock.dll` and `nettest.exe` are all unstaged (only `ws2_32.dll`/`ws2help.dll` are present),
    and `afd.sys` (Start=1) would need the same explicit arm64-load as netstack. So the full
    ws2_32→msafd→AFD→netstack usermode chain is unexercised on arm64 — a separate chunk of work.
  - **OPEN (proper kernel fix):** wire a periodic arm64 clock interrupt → `KeUpdateSystemTime` so
    `InterruptTime`/`delay_ms`/all timer-based kernel paths work on arm64 (currently only the
    cntvct perf counter advances). Benefits all arm64 work, not just netstack.
- **✅ FIXED (2026-10-04) — user-mode name lookup and HTTPS to the real internet.** rapps'
  "Updating Database" failed on arm64. `tools/payloads/netprobe` walks the client path one layer
  at a time, and found six independent breaks:
  1. **No DNS servers for user mode.** iphlpapi's `GetNetworkParams` reads
     `Tcpip\Parameters\Interfaces\*\{NameServer,DhcpNameServer}`; netstack kept DHCP's servers
     to itself. `netcfg::publish_dns_servers` writes `DhcpNameServer` under
     `Interfaces\netstack` on every lease.
  2. **ARP starvation.** smoltcp 0.13 keeps ONE `silent_until` for the whole interface (1 s;
     still global in 0.14): an ARP for host A silences the lookup for host B, so a DNS query
     queued behind the router's ARP never went out. netstack now sends its own ARP requests for
     the router and the on-link DNS servers on each lease, and every 30 s after
     (`engine::warm_neighbors`), so the cache is full before anyone asks. **Owed:** a
     per-address limit in smoltcp itself (a vendor patch, deferred by the user).
  3. **No SYN timeout.** smoltcp sockets never give up on SYN-SENT. A TDI connect is now aborted
     after 21 s and completes `STATUS_IO_TIMEOUT`, which msafd maps to `WSAETIMEDOUT`.
  4. **UDP datagrams came back with no source.** `TDI_RECEIVE_DATAGRAM`'s
     `ReturnDatagramInformation` was never filled, so `recvfrom` reported no sender address.
  5. **AFD select never reported a ready socket.** `UpdatePollWithFCB` wrote the poll status
     into AFD's locked copy (`AFD_HANDLES()` / `PollReq->Exclusive`) instead of the IRP's
     system buffer, and `CopyBackStatus` copies from the latter.
  6. **gethostbyname got no hostent.** ws2_32's `getxyDataEnt` dropped the C's
     `LUP_RETURN_BLOB` for non-hostname lookups, so the NSP returned no blob.
  Past the network, https still failed `ERROR_INTERNET_INVALID_CA` (12045): bcrypt's ECDSA was a
  stub and Let's Encrypt's chain is ECDSA (YR2 → Root YR → ISRG Root X1). Upstream bcrypt is
  restored over mbedTLS 3.6 (`dll/win32/bcrypt`, `vendor/mbedtls` with `MBEDTLS_DSA_C`).
  **A probe trap:** the livecd's boot-time `setup -mini` registers rsaenh only after it has
  started Dnscache, minutes after the desktop on arm64 TCG. A crypt32 that opens the ROOT store
  before then has no CSP and imports nothing (`rootstore.c: Failed to import system certs into
  registry, 00000057`, from `CryptCreateHash(0, ...)`), and caches that result for the process.
  Wait for `setup.exe` to exit first, as appstorm and netprobe do.
- **✅ FIXED (2026-10-04) — pended TDI IRPs could not be cancelled.** No netstack IRP had a
  cancel routine, so `IoCancelIrp` was a no-op. AFD closes a socket by cancelling its in-flight
  TDI requests and then freeing the FCB their completion routines point at; the worker later
  completed the connect anyway, into a freed FCB, and `MakeSocketIntoConnection` called
  `TdiReceive` on a freed connection object (arm64 BUGCHECK 0x1E in `IoGetRelatedDeviceObject`
  when appstorm closed rapps mid-download; and a thread blocked on such an IRP could not exit).
  Now every pended IRP is registered in `tditrans::PENDING`; exactly one side claims it: the cancel
  routine (completes `STATUS_CANCELLED` synchronously, as tcpip's `DispCancelRequest` does, then
  tells the worker to drop references and abort a pending connect) or the worker's `complete`.
  The worker touches a pended IRP's buffer only inside `with_pending`, under the same lock.
  Cancels are routine once they work: AFD cancels its in-flight DNS datagram receives on every
  socket close.
- **Later (v1+):** see the **Post-Parity Roadmap (M8+)** section above — server-side TCP
  (TDI_LISTEN/ACCEPT), the management plane (iphlpapi/ipconfig/ping), interrupt-driven RX,
  NDIS real-NIC support, then completeness/perf (IPv6, multicast, zero-copy, offloads).

---

## Cross-references
- Net test suite: `tests/qemu/test_net.py` (`--arch amd64-uefi-net`); folded into
  `vibeos-cli smoke --arch amd64` as the `net` stage
- Storage 0x50 blocker: `docs/engineering/translation-debugging-field-manual.md` §9.9 +
  `drivers/network/netstack/CMakeLists.txt` comment (task 79)
- Roadmap / lanes: `docs/engineering/archive/translation-banking-roadmap.md` (L4)
- Driver cargo-cutover recipe: memory `project_device_driver_cargo_cutover`
- DDK substrate for drivers: memory `feedback_driver_ddk_substrate`
- smoltcp: <https://docs.rs/smoltcp> · phy::Device, socket::{tcp,udp,icmp,raw,dhcpv4,dns}
- virtio-drivers: <https://github.com/rcore-os/virtio-drivers> (Hal trait, VirtIONet)
- Prior art: Redox netstack/smolnetd — port-set + ephemeral-port + per-op UDP endpoint lessons
