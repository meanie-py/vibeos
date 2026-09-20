# Upstream-sync: the 14 deleted tcpip.sys files upstream kept fixing

Fork point FP = `5213cf717cff6946945922e6a7e03d87b0a58e4f`. Our `3feaf6ca5f6` deleted the
legacy NDIS/tcpip/tdi C stack (`drivers/network/tcpip/**`), which was superseded by a
**from-scratch Rust rewrite** (`netstack-rust`/`netstack-cargo`), not a translation of that C.
Per `rust-port-status.md`: "`netstack.sys` is the only from-scratch Rust rewrite (100%, zero
C, replacing the legacy NDIS/tcpip stack)."

## Scope correction #1 — this is 3 commits, not 14 independent fixes

`git log FP..upstream/master -- <path>` for all 14 listed paths resolves to exactly **3**
upstream commits (each touches several of the 14 files together):

| Upstream SHA | Subject | Files (of the 14) touched |
|---|---|---|
| `248aa75023a` | `[WSHTCPIP][TCPIP] Implement SO_KEEPALIVE, SIO_KEEPALIVE_VALS` | tcp.h, lwip_glue.h, lwip_glue/tcp.c, transport/tcp/tcp.c, opt.h, tcp_priv.h, cinfo.c |
| `606d22d6c0f` | `[TCPIP] Map 127.x.x.x addresses to the loopback address` | neighbor.c, interface.c |
| `07d30c679ee` | `[TCPIP] Read the interface metric from the registry or use the automatic metric feature` | router.h, ip.h, router.c, ninfo.c, lan.c |

## Scope correction #2 — the Rust netstack is a smoltcp wrapper, not a hand-rolled stack

`sdk/lib/rust/netstack-rust/Cargo.toml`'s own header comment: *"the smoltcp network stack
(L4 rewrite, NOT a C translation). Replaces the ReactOS tcpip protocol engine (tcpip/ip ~8K +
tcpip/lwip ~115K C) with the smoltcp crate."* `netstack-rust/src/` (6.8K lines) is entirely
glue: `engine.rs` (smoltcp `Interface`/`SocketSet` poll loop), `device.rs`/`e1000.rs`/`ndis.rs`
(NIC backends), `tdi*.rs` (~2.9K lines, the TDI IOCTL surface talked to by AFD/wshtcpip), plus
test/self-test files. **There is no TCP state machine, no neighbor/ARP cache, and no routing
table in our code** — those live entirely inside the `smoltcp` crate (v0.13.1, external
dependency) and `virtio-drivers`. This flips the framing in the brief: a fix to upstream's
hand-rolled/lwIP TCP or neighbor-cache code is evidence about *their* implementation only,
never evidence about ours, because we did not copy their algorithm — we call a third-party
crate's. The correctness question for anything smoltcp already owns (TCP retransmit/keepalive
timing, ARP/neighbor expiry, route lookup) is really "does smoltcp 0.13.1 handle this," which
is out of scope for a repo audit (it's upstream-to-smoltcp, not upstream-to-us). What IS in
scope is: (a) protocol-level facts our *glue* must still respect (e.g. "127.0.0.0/8 is all
loopback," which is a fact about what traffic the glue must route to what device, independent
of whether smoltcp is doing the TCP math), and (b) Windows-API-surface facts (registry keys,
IOCTL semantics) our TDI/WSH glue must still honor even though the transport under it changed.

## Classification table

| Upstream SHA | File(s) | What it fixes | Class | Our netstack's status |
|---|---|---|---|---|
| 248aa75023a | tcp.h, lwip_glue.*, tcp.c (both), opt.h, tcp_priv.h, cinfo.c | Adds `TCPSetKeepAlive`/`TCPSetKeepAliveValues`, wires `SOF_KEEPALIVE` + `keep_idle`/`keep_intvl`/`keep_cnt` into lwIP's PCB, flips `LWIP_TCP_KEEPALIVE` on, and sets Windows-correct keepalive defaults (1000ms interval, 10 probes per MSDN's `SIO_KEEPALIVE_VALS` doc) instead of BSD lwIP defaults (75000ms/9). Adds the `TCP_SOCKET_KEEPALIVE`/`TCP_SOCKET_KEEPALIVEVALS` TDI info-class dispatch in `cinfo.c`. | **Mostly IMPL-SPECIFIC** (lwIP PCB field wiring), with one **PROTOCOL-HAZARD-adjacent** fact worth carrying forward: the *Windows-specific default keepalive interval/count values*, which are an API-contract fact (MSDN `SIO_KEEPALIVE_VALS`), not an internet-protocol fact. | **Not implemented.** `netstack-rust/src/tdidev.rs:634-636` (`handle_set_info_ex`) explicitly acks all `IOCTL_TCP_SET_INFORMATION_EX` sockopts (including what would be `TCP_SOCKET_KEEPALIVE`) without plumbing them to smoltcp: *"netstack doesn't plumb transport sockopts down to smoltcp yet (roadmap M12+), but acknowledge them so setsockopt(TCP_NODELAY / IP_TTL / ...) succeeds rather than erroring."* `docs/engineering/smoltcp-netstack-design.md:547` lists `SO_KEEPALIVE` explicitly under **M12+ — Completeness & performance**. This is a known, already-tracked gap, not a bug — "not implemented" is the honest answer. |
| 606d22d6c0f | interface.c, neighbor.c | `FindOnLinkInterface` now special-cases the whole `127.0.0.0/8` block as on-link via a new `HasLoopbackPrefix` check (previously only the interface's own configured prefix counted). `NBAddNeighbor`/`NBLocateNeighbor` now canonicalize **any** `127.x.x.x` address to `127.0.0.1` before hashing/inserting/looking up in the neighbor cache, so the whole /8 collapses to one cache entry instead of one per distinct loopback address contacted. | **PROTOCOL-HAZARD.** RFC 1122 §3.2.1.3(g) / RFC 6890 reserve the entire `127.0.0.0/8` block for loopback, and any correct IP stack must treat every address in that block as local-only/on-link and never emit it onto the wire or treat distinct 127.x addresses as distinct "neighbors" needing separate link-layer resolution. This is exactly the class of hazard the brief calls out: a from-scratch implementation is just as capable of missing it independently, because it isn't a bug you inherit — it's a spec fact every stack must encode for itself. | **Not implemented — and this is a real, currently-unaddressed gap**, not merely "different design." `grep -rn "127\.\|LOOPBACK\|loopback\|Loopback" sdk/lib/rust/netstack-rust/src/*.rs` returns **zero hits** in the entire crate. There is exactly one `smoltcp::iface::Interface`, bound to the physical NIC (virtio-net or e1000) and DHCP-configured (`engine.rs`); no loopback device, no local-address short-circuit, and no special-casing of `127.0.0.0/8` anywhere in the TDI dispatch (`tdidev.rs`) or the poll engine. A connect/send to `127.x.x.x` today has no dedicated path at all — it would need to be resolved and routed like any other destination through the single physical-NIC interface, which has no route to `127.0.0.0/8` and would fail (or, worse, if smoltcp ever treated it as off-link, could theoretically try to ARP for it). Notably, `docs/engineering/smoltcp-netstack-design.md:228` **lists loopback as in the intended v1 scope** ("one NIC (QEMU virtio-net) + loopback"), so this isn't just an unplanned gap — the design doc says it should exist and the code doesn't have it. Loopback matters for real workloads (DNS resolvers, local RPC, `localhost`-bound test servers, `ping 127.0.0.1`), so this is worth prioritizing, independent of anything upstream did. |
| 07d30c679ee | router.h, ip.h, router.c, ninfo.c, lan.c | Adds `IP_INTERFACE::Metric`, reads it from the registry (`Tcpip\Parameters\Interfaces\<if>\InterfaceMetric`), and falls back to `ProcessAutoMetric()` — a Windows-documented link-speed-to-metric table (loopback=1; >2Gbps=5; >200Mbps=10; >80Mbps=20; >20Mbps=25; >4Mbps=30; >500Kbps=40; else=50) — when no registry value is set. `RouterCreateRoute`'s `Metric` parameter is dropped in favor of always calling `ProcessAutoMetric(Interface)`, and `InfoTdiSetRoute` now stashes an explicit route's metric onto the interface before creating the route. | **IMPLEMENTATION-SPECIFIC.** This is Windows/NT's own multi-homed-routing convention (how the OS ranks routes across several interfaces when more than one exists), sourced from a Windows registry key and a Microsoft-documented auto-metric table — not an internet-protocol requirement. It only has meaning when a FIB with more than one competing route exists. | **N/A by design, not a gap.** `netstack-rust` has exactly one interface and one default route (`engine.rs:266-283`, `iface.routes_mut().add_default_ipv4_route()`/`remove_default_ipv4_route()` — smoltcp's routing table holds a single default route, no FIB, no per-route metric concept). `tdidev.rs:562-575` (`mib_ip_routetable`) reports that one route to `iphlpapi`/`route print` callers with a **hardcoded `ire_metric1 = 1`**, which is harmless today precisely because there is never more than one route to rank. `iphlpapi-rust`'s `types_*.rs` carry the `dwForwardMetric*`/`Ipv4Metric` struct fields (bindgen surface) but no logic computing them beyond that hardcode. This only becomes a real gap if/when multi-NIC or multi-route support is added (tracked as "Later: multi-NIC/routing" in the design doc's Scope section) — worth a forward note in that milestone, not an open bug now. |

## Detail: the one PROTOCOL-HAZARD (127.0.0.0/8 loopback)

- **Hazard**: any TCP/IP stack must (1) never place `127.0.0.0/8` traffic on the wire, (2)
  treat the entire block as reaching "this host" regardless of the specific address used, and
  (3) not waste per-address neighbor/ARP-cache state on it (RFC 1122, RFC 6890).
- **Upstream's fix mechanism**: two surgical patches — `FindOnLinkInterface` treats the block
  as on-link even when it doesn't match the configured interface prefix, and the neighbor
  cache canonicalizes every `127.x.x.x` to `127.0.0.1` before it ever becomes a cache key.
- **Our status**: genuinely **not implemented**. No loopback device, no address
  canonicalization, no on-link special-case anywhere in `netstack-rust`. This is a real,
  user-visible functionality gap (e.g. `ping 127.0.0.1` or any `localhost`-bound listener
  would not work today), it is *listed in our own design doc's v1 scope* and not yet built,
  and it is independent of anything upstream did — smoltcp does not supply loopback behavior
  automatically; something in our `engine.rs`/`device.rs`/TDI layer would need to special-case
  `127.0.0.0/8` and either loop packets back on a virtual smoltcp device or short-circuit TDI
  connects to that range without touching the physical interface at all.
- **Recommendation** (analysis only, no fix applied here): this belongs on the netstack
  roadmap as an explicit milestone — it currently reads as scope that was declared but
  quietly dropped rather than deferred, which is worth surfacing to whoever owns that lane.

## Where our rewrite already looks better than upstream

- **Route/metric complexity avoided entirely.** Upstream needed a registry read, a
  link-speed-to-metric lookup table, and FIB bookkeeping to get sane route preference across
  multiple interfaces. Our single-default-route model (smoltcp's routing table, `engine.rs`)
  sidesteps that whole class of bug by construction — there's no metric to get wrong because
  there's nothing to rank against. This is a real "less surface, less bug" win, not just
  smaller scope; it will need revisiting if/when multi-NIC support lands, but it is not a
  regression today.
- **Assertions/sockopts fail safe rather than silently wrong.** Where sockopts aren't plumbed
  yet (`TCP_NODELAY`, `SO_KEEPALIVE`, etc.), `tdidev.rs` acknowledges them rather than either
  erroring or pretending to apply them incorrectly — an honest "no-op, ack" stance versus
  upstream's C, which had to actually implement each one correctly or silently misbehave. It's
  a smaller feature set today, but it fails in a legible, documented way (M12+ backlog) rather
  than a subtly wrong one.

## Scope corrections summary

1. 14 files → **3 upstream commits**, not 14 independent fixes.
2. `netstack-rust` is a **from-scratch smoltcp wrapper** (~6.8K lines of glue), not a
   reimplementation of TCP/neighbor/routing algorithms — most of what upstream's C had to
   get right by hand (TCP retransmit timing, neighbor cache eviction, route lookup) is now a
   third-party crate's problem, not ours, so most "protocol hazard" surface from these 3
   commits doesn't apply to us directly.
3. Of the 3 commits, only **one** (loopback mapping) describes a hazard that is genuinely
   ours to handle regardless of implementation strategy, and it is **not currently handled**
   — a real, findable, already-scoped-but-unbuilt gap, distinct from "handled incorrectly."
