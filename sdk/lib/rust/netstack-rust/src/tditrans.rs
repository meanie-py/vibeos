//! TDI transport data path — message-passing between the TDI dispatch (any
//! thread, via IRPs) and the smoltcp poll worker (sole owner of iface/sockets).
//!
//! The dispatch parses a TDI request, posts a `Cmd` to a spinlock'd inbox, marks
//! the IRP pending (for the ops that wait on the network), and returns. The
//! worker drains the inbox each poll iteration, applies commands to smoltcp, and
//! — in the completion bridge — completes pended IRPs once the socket condition
//! is met (connect→Established, data available for a receive). All IRP
//! completion happens in the worker at PASSIVE with no lock held.

use alloc::vec;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU64, Ordering};

use smoltcp::iface::{Interface, SocketHandle, SocketSet};
use smoltcp::phy::ChecksumCapabilities;
use smoltcp::socket::{icmp, tcp, udp};
use smoltcp::wire::{Icmpv4Packet, Icmpv4Repr, IpAddress, Ipv4Address};

use ntkrnl_sys::ddk::{
    IoAcquireCancelSpinLock, IoReleaseCancelSpinLock, IofCompleteRequest, KIRQL, NTSTATUS,
    PDEVICE_OBJECT, PIRP,
};
use ntkrnl_sys::inline::tier1::IoSetCancelRoutine;
use ntkrnl_sys::guards::{IrqlGuard, SpinLockGuard};
use ntkrnl_sys::ke::KSPIN_LOCK;

use crate::serial::nsprint;
use crate::time::now;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_TIMEOUT: NTSTATUS = 0x0000_0102;
const STATUS_CONNECTION_REFUSED: NTSTATUS = 0xC000_0236u32 as NTSTATUS;
const STATUS_IO_TIMEOUT: NTSTATUS = 0xC000_00B5u32 as NTSTATUS;
/// Windows' default connect timeout: the initial SYN plus two retries (3s, 6s, 12s).
const CONNECT_TIMEOUT_MS: i64 = 21_000;
const STATUS_CONNECTION_RESET: NTSTATUS = 0xC000_020Du32 as NTSTATUS;

/// Identifier the management ICMP socket binds to (distinct from the boot
/// self-test's 0x022b) — replies are correlated back to pending pings by seq.
const MGMT_ICMP_IDENT: u16 = 0x4944;

const STATUS_CANCELLED: NTSTATUS = 0xC000_0120u32 as NTSTATUS;

/// Complete an IRP nobody else can claim: the receive IRP an AFD event handler hands
/// back, which was never pended here.
unsafe fn complete_unregistered(irp: usize, status: NTSTATUS, info: usize) {
    if irp == 0 {
        return;
    }
    let irp = irp as PIRP;
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    (*irp).IoStatus.Information = info as _;
    IofCompleteRequest(irp, 0);
}

/// Worker→IRP completion of a pended IRP. Safe to call at PASSIVE with no engine lock
/// held. A no-op if the cancel routine claimed the IRP first: it has completed it, and
/// it may already be freed, so it must not be touched.
unsafe fn complete(irp: usize, status: NTSTATUS, info: usize) {
    if irp == 0 || !claim(irp) {
        return;
    }
    if IoSetCancelRoutine(irp as PIRP, None).is_none() {
        // IoCancelIrp has already called the routine, which lost the claim to us. It
        // holds the cancel spinlock until its first statement has read CancelIrql;
        // wait for that before the IRP can be freed under it.
        let mut irql: KIRQL = 0;
        IoAcquireCancelSpinLock(&mut irql);
        IoReleaseCancelSpinLock(irql);
    }
    complete_unregistered(irp, status, info);
}

// --------------------------------------------------------------------------- #
// Cancellation. AFD closes a socket by IoCancelIrp-ing its in-flight TDI requests
// and then freeing the FCB those requests' completion routines point at, so a cancel
// must complete the IRP INSIDE the cancel routine, as tcpip's DispCancelRequest does.
// Completing it later from the worker ran AFD's StreamSocketConnectComplete on a freed
// FCB and called TdiReceive on a freed connection object (arm64 BUGCHECK 0x1E in
// IoGetRelatedDeviceObject, rapps closed mid-download, 2026-10-04).
//
// Every pended IRP sits in PENDING until exactly one side claims it: the cancel
// routine (completes STATUS_CANCELLED) or the worker (`complete`). The worker touches
// a pended IRP's buffer only through `with_pending`, under the same lock, so a
// cancelled IRP's memory is never written after its completion.
// --------------------------------------------------------------------------- #

struct Pending {
    lock: UnsafeCell<KSPIN_LOCK>,
    irps: UnsafeCell<Vec<usize>>,
}
unsafe impl Sync for Pending {}

static PENDING: Pending = Pending {
    lock: UnsafeCell::new(0),
    irps: UnsafeCell::new(Vec::new()),
};

/// Take ownership of a pended IRP: true if it was still registered.
fn claim(irp: usize) -> bool {
    unsafe {
        let _lock = SpinLockGuard::acquire(PENDING.lock.get());
        let irps = &mut *PENDING.irps.get();
        match irps.iter().position(|&i| i == irp) {
            Some(pos) => {
                irps.swap_remove(pos);
                true
            }
            None => false,
        }
    }
}

fn is_pending(irp: usize) -> bool {
    unsafe {
        let _lock = SpinLockGuard::acquire(PENDING.lock.get());
        (*PENDING.irps.get()).contains(&irp)
    }
}

/// Run `f` on a pended IRP's buffers while no cancel can complete it underneath;
/// None if it has been cancelled (drop the entry). Runs at DISPATCH_LEVEL under the
/// lock, so `f` must only copy (smoltcp's slice copies are fine) and must not
/// complete anything.
fn with_pending<R>(irp: usize, f: impl FnOnce() -> R) -> Option<R> {
    unsafe {
        let _lock = SpinLockGuard::acquire(PENDING.lock.get());
        if (*PENDING.irps.get()).contains(&irp) {
            Some(f())
        } else {
            None
        }
    }
}

unsafe extern "system" fn cancel_routine(_dev: PDEVICE_OBJECT, irp: PIRP) {
    IoReleaseCancelSpinLock((*irp).CancelIrql);
    if claim(irp as usize) {
        complete_unregistered(irp as usize, STATUS_CANCELLED, 0);
        // The worker drops its references (and aborts a pending connect). It compares
        // the pointer only; the IRP may be gone by the time it looks.
        post(Cmd::Cancel { irp: irp as usize });
    }
}

/// Register an IRP the dispatch is about to pend (already IoMarkIrpPending'd).
/// False if it was cancelled before the cancel routine was in place: it has been
/// completed STATUS_CANCELLED here, and the caller must not post it to the worker.
pub unsafe fn pend(irp: PIRP) -> bool {
    {
        let _lock = SpinLockGuard::acquire(PENDING.lock.get());
        (*PENDING.irps.get()).push(irp as usize);
    }
    IoSetCancelRoutine(irp, Some(cancel_routine));
    if (*irp).Cancel != 0 && IoSetCancelRoutine(irp, None).is_some() {
        // IoCancelIrp ran before the routine existed and found nothing to call.
        if claim(irp as usize) {
            complete_unregistered(irp as usize, STATUS_CANCELLED, 0);
        }
        return false;
    }
    true
}

// --------------------------------------------------------------------------- #
// Commands (dispatch -> worker). Raw pointers are valid across threads (kernel).
// --------------------------------------------------------------------------- #

pub enum Cmd {
    CreateAddr { id: u64, port: u16 },
    CreateConn { id: u64, conn_ctx: usize },
    SetEvent { addr_id: u64, ev_type: u32, handler: usize, context: usize },
    Associate { conn_id: u64, addr_id: u64 },
    Connect { conn_id: u64, ip: [u8; 4], port: u16, irp: usize },
    /// Server-side: bind a fresh socket to the connection's associated local port
    /// and listen; the IRP completes when an inbound connection is established.
    Listen { conn_id: u64, irp: usize, ret_info: usize },
    /// getsockname: report the local address of a connection or address object
    /// into the IRP's MDL (out/out_len), then complete.
    QueryAddress { id: u64, irp: usize, out: usize, out_len: usize },
    /// IOCTL_TCP_SET_INFORMATION_EX route programming (M9 item 4): add or remove
    /// the IPv4 default route via `gw`. The dispatch acks the IRP synchronously;
    /// this only reprograms the worker-owned interface routing table.
    SetDefaultRoute { gw: [u8; 4], add: bool },
    Send { conn_id: u64, irp: usize, data: usize, len: usize },
    Receive { conn_id: u64, irp: usize, buf: usize, len: usize },
    Disconnect { conn_id: u64, irp: usize },
    // Connectionless (UDP) datagram path — rides on the ADDRESS object.
    SendDatagram { addr_id: u64, irp: usize, data: usize, len: usize, ip: [u8; 4], port: u16 },
    ReceiveDatagram { addr_id: u64, irp: usize, buf: usize, len: usize, ret_info: usize },
    CloseConn { id: u64 },
    CloseAddr { id: u64 },
    /// The cancel routine completed this IRP: forget every reference to it.
    Cancel { irp: usize },
    /// ICMP echo (ping): emit an echo request on the management ICMP socket and
    /// complete the IRP with the ICMP_ECHO_REPLY once a reply arrives (or on
    /// timeout). `sysbuf` is the METHOD_BUFFERED SystemBuffer (kernel, written by
    /// the worker); `user_buffer` is the caller's output VA (used only to compute
    /// the reply's `Data` back-pointer — never dereferenced in the kernel).
    IcmpEcho {
        irp: usize,
        sysbuf: usize,
        out_len: usize,
        user_buffer: usize,
        dst: [u8; 4],
        timeout_ms: u32,
        data: Vec<u8>,
    },
}

/// The inbox is a real MPSC command queue: many producers (TDI dispatch, any
/// thread) push; one consumer (the poll worker) drains via `mem::swap`. It is
/// guarded by a **real `KSPIN_LOCK`** (`KeAcquireSpinLock`), which raises IRQL
/// to DISPATCH *and* spins on the lock word.
///
/// HISTORY — the `-smp 2` TDI dgram TIMED OUT flake (~1/3, field manual §9.55):
/// this used to guard the queue with only `IrqlGuard::raise_to_dpc()`, on the
/// premise that "QEMU runs a SINGLE vCPU, so DISPATCH_LEVEL masks all thread
/// preemption = full mutual exclusion." That is a **UP-only** assumption. Under
/// `-smp 2` a raise on CPU0 does nothing to a producer thread on CPU1: two
/// producers (plus the consumer's `mem::swap`) raced the `Vec`'s (ptr,len,cap)
/// — a `push` that reallocates while the consumer swaps = torn read / lost
/// command → the SEND_DATAGRAM or SET_EVENT command vanished → the datagram
/// upcall never fired → the self-test timed out. The comment itself flagged the
/// hazard ("an unsynchronized Vec race between two threads = UB/heap
/// corruption") — it just wrongly trusted the single-vCPU premise.
///
/// `KeAcquireSpinLock` is the exported kernel symbol, so it resolves per-boot:
/// the SMP kernel's `KxAcquireSpinLock` really spins (`feature=smp`); the UP
/// kernel's is a no-op compiler fence — identical to the old raise-only path
/// there. NB: `KeAcquireSpinLockForDpc` (a *different* function) IS still a
/// no-op stub in this tree — do not reach for it.
struct Inbox {
    /// Cross-CPU exclusion for the queue below (see the type doc). `0` = the
    /// KeInitializeSpinLock initial state.
    lock: UnsafeCell<KSPIN_LOCK>,
    queue: UnsafeCell<Vec<Cmd>>,
}
unsafe impl Sync for Inbox {}

static INBOX: Inbox = Inbox {
    lock: UnsafeCell::new(0),
    queue: UnsafeCell::new(Vec::new()),
};
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Allocate a unique file-object id (address or connection).
pub fn alloc_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// Post a command from the TDI dispatch (any thread).
pub fn post(cmd: Cmd) {
    unsafe {
        let _lock = SpinLockGuard::acquire(INBOX.lock.get());
        (*INBOX.queue.get()).push(cmd);
    }
}

fn drain_into(out: &mut Vec<Cmd>) {
    unsafe {
        let _lock = SpinLockGuard::acquire(INBOX.lock.get());
        core::mem::swap(out, &mut *INBOX.queue.get());
    }
}

// --------------------------------------------------------------------------- #
// Worker-private connection/address tables + the completion bridge.
// --------------------------------------------------------------------------- #

struct Addr {
    id: u64,
    /// Local UDP port (0 = let the stack pick an ephemeral on first use).
    local_port: u16,
    /// Lazily-created bound UDP socket for the connectionless datagram path.
    udp: Option<SocketHandle>,
    ev_recv: Option<(usize, usize)>,
    ev_disc: Option<(usize, usize)>,
    /// Registered TDI_EVENT_RECEIVE_DATAGRAM handler (handler, context).
    ev_recv_dg: Option<(usize, usize)>,
    /// Pended RECEIVE_DATAGRAM IRPs (irp, buf_va, buf_len) — used when no
    /// datagram event handler is registered.
    recv_dg: Vec<(usize, usize, usize, usize)>,
    /// Broadcasts this host sent to our port: (payload, source ip, source port), awaiting
    /// delivery. See `loop_back_broadcast`.
    local_dg: Vec<(Vec<u8>, [u8; 4], u16)>,
}

struct Conn {
    id: u64,
    addr_id: u64,
    conn_ctx: usize,
    handle: Option<SocketHandle>,
    connect_irp: usize,
    /// When the pending TDI_CONNECT was issued (ms), for the SYN-SENT timeout.
    connect_ms: i64,
    /// Pended TDI_LISTEN IRP + the caller's ReturnConnectionInformation (filled
    /// with the remote address on accept). 0 when not listening.
    listen_irp: usize,
    listen_ret: usize,
    /// Port this conn's pending TDI_LISTEN waits on (0 = none) — consumed
    /// from the ListenPort ready queue by the bridge.
    listen_port: u16,
    recv: Vec<(usize, usize, usize)>, // (irp, buf_va, buf_len)
    /// Pended TDI_SEND IRPs: (irp, data_va, len, offset-already-queued).
    /// TDI semantics: a send completes only when the transport has consumed
    /// the WHOLE buffer. The old early-complete (short count on a full tx
    /// buffer, REFUSED when can_send()==false) silently truncated large
    /// responses at 8KiB-multiples — the httpd/tcp_hammer 64KiB find,
    /// 2026-07-08.
    send_q: Vec<(usize, usize, usize, usize)>,
    connected: bool,
}

/// An in-flight ICMP echo IOCTL awaiting its reply (or timeout).
struct PendingPing {
    irp: usize,
    sysbuf: usize,
    out_len: usize,
    user_buffer: usize,
    dst: [u8; 4],
    seq: u16,
    /// Request payload to echo (copied out of SystemBuffer at dispatch time).
    data: Vec<u8>,
    /// False until the echo request has actually been emitted (socket may be full).
    sent: bool,
    start_ms: i64,
    deadline_ms: i64,
}

/// Server-side listen backlog for one local port (the httpd/tcp_hammer find,
/// 2026-07-08). smoltcp has no SYN queue — one listening socket absorbs ONE
/// connection — and AFD keeps a single TDI_LISTEN in flight, re-posting only
/// after usermode accept()s. Under concurrent connect load every SYN landing
/// in that window got RST (194/320 hammer failures). Real Windows covers this
/// with a kernel-side backlog; ours: keep BACKLOG shadow sockets listening on
/// the port, park completed handshakes in `ready`, and satisfy TDI_LISTEN
/// IRPs from `ready` instantly. Request bytes arriving before the accept sit
/// in the shadow socket's rx buffer — nothing is dropped.
struct ListenPort {
    port: u16,
    /// Sockets currently in LISTEN / mid-handshake.
    shadows: Vec<SocketHandle>,
    /// ESTABLISHED, awaiting a TDI_LISTEN to claim them (FIFO).
    ready: Vec<SocketHandle>,
}

/// Upper bound on shadow listeners per port. 16 matches the deliberate stress
/// gate (tcp_hammer threads=16). This is only a CAP — the live count is armed
/// to DEMAND (pended accepts + 1 spare), not eagerly filled. Eagerly holding 16
/// idle sockets on a port nobody is hammering (2×2KiB ×16 ≈ 64KiB nonpaged) put
/// enough pressure on arm64 Mm to trigger a registry hive shrink that calls the
/// unimplemented arm64 fastfat `FatOpenExistingDirectory` stub → BUGCHECK 0x23
/// (field manual §9.43 follow-up). Demand-tied arming keeps an idle listen port
/// at ~2 sockets, so the kernel TDI-server self-test (one pended LISTEN, no peer)
/// no longer bloats the pool.
const LISTEN_BACKLOG: usize = 16;
/// Per-shadow socket buffer (rx==tx). 2KiB holds a request; larger responses
/// stream via the pended-send path across pumps. Kept small to bound the
/// nonpaged footprint a burst of shadows adds (see LISTEN_BACKLOG).
const SHADOW_BUF: usize = 2048;

pub struct TdiState {
    addrs: Vec<Addr>,
    conns: Vec<Conn>,
    cmds: Vec<Cmd>,
    listen_ports: Vec<ListenPort>,
    /// Lazily-created management ICMP socket backing the ping IOCTL.
    icmp: Option<SocketHandle>,
    icmp_seq: u16,
    pings: Vec<PendingPing>,
}

impl TdiState {
    pub fn new() -> Self {
        Self {
            addrs: Vec::new(),
            conns: Vec::new(),
            cmds: Vec::new(),
            listen_ports: Vec::new(),
            icmp: None,
            icmp_seq: 0,
            pings: Vec::new(),
        }
    }

    fn conn(&mut self, id: u64) -> Option<&mut Conn> {
        self.conns.iter_mut().find(|c| c.id == id)
    }
    fn addr(&mut self, id: u64) -> Option<&mut Addr> {
        self.addrs.iter_mut().find(|a| a.id == id)
    }
}

/// Called by the worker each poll iteration: drain the inbox, apply commands,
/// then reconcile pended IRPs against socket state.
pub fn pump(iface: &mut Interface, sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    let mut cmds = Vec::new();
    drain_into(&mut cmds);
    for cmd in cmds.drain(..) {
        apply(cmd, iface, sockets, state);
    }
    bridge(sockets, state);
    pump_icmp(sockets, state);
}

fn apply(cmd: Cmd, iface: &mut Interface, sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    match cmd {
        Cmd::CreateAddr { id, port } => {
            state.addrs.push(Addr {
                id,
                local_port: port,
                udp: None,
                ev_recv: None,
                ev_disc: None,
                ev_recv_dg: None,
                recv_dg: Vec::new(),
                local_dg: Vec::new(),
            });
        }
        Cmd::CreateConn { id, conn_ctx } => {
            state.conns.push(Conn {
                id,
                addr_id: 0,
                conn_ctx,
                handle: None,
                connect_irp: 0,
                connect_ms: 0,
                listen_irp: 0,
                listen_ret: 0,
                listen_port: 0,
                recv: Vec::new(),
                send_q: Vec::new(),
                connected: false,
            });
        }
        Cmd::SetEvent { addr_id, ev_type, handler, context } => {
            if let Some(a) = state.addr(addr_id) {
                match ev_type as i32 {
                    crate::tdi::TDI_EVENT_RECEIVE => a.ev_recv = Some((handler, context)),
                    crate::tdi::TDI_EVENT_DISCONNECT => a.ev_disc = Some((handler, context)),
                    crate::tdi::TDI_EVENT_RECEIVE_DATAGRAM => {
                        a.ev_recv_dg = Some((handler, context))
                    }
                    _ => {}
                }
            }
        }
        Cmd::Associate { conn_id, addr_id } => {
            if let Some(c) = state.conn(conn_id) {
                c.addr_id = addr_id;
            }
        }
        Cmd::Connect { conn_id, ip, port, irp } => {
            // Fresh tcp socket for this connection.
            let sock = tcp::Socket::new(
                tcp::SocketBuffer::new(vec![0u8; 8192]),
                tcp::SocketBuffer::new(vec![0u8; 8192]),
            );
            let handle = sockets.add(sock);
            let local = 50000u16.wrapping_add((conn_id & 0x3fff) as u16);
            let r = sockets.get_mut::<tcp::Socket>(handle).connect(
                iface.context(),
                (IpAddress::v4(ip[0], ip[1], ip[2], ip[3]), port),
                local,
            );
            if let Some(c) = state.conn(conn_id) {
                c.handle = Some(handle);
                if r.is_ok() {
                    c.connect_irp = irp;
                    c.connect_ms = crate::time::now().total_millis();
                } else {
                    unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
                }
            } else {
                unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
            }
        }
        Cmd::Listen { conn_id, irp, ret_info } => {
            // The bound local port comes from the address object AFD associated to
            // this connection before posting TDI_LISTEN (port 0 = no bind = reject).
            let local_port = state
                .conn(conn_id)
                .map(|c| c.addr_id)
                .and_then(|aid| state.addrs.iter().find(|a| a.id == aid))
                .map(|a| a.local_port)
                .unwrap_or(0);
            if local_port == 0 || state.conn(conn_id).is_none() {
                unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
            } else {
                // Ensure the port's shadow-listener backlog exists (see ListenPort).
                let lpi = match state.listen_ports.iter().position(|lp| lp.port == local_port) {
                    Some(i) => i,
                    None => {
                        state.listen_ports.push(ListenPort {
                            port: local_port,
                            shadows: Vec::new(),
                            ready: Vec::new(),
                        });
                        nsprint!("[NETSTACK] TDI LISTEN backlog armed on :{}\n", local_port);
                        state.listen_ports.len() - 1
                    }
                };
                // Arm a minimal floor now (this listen + one spare); the bridge
                // refines the count to demand once the IRP is parked below.
                top_up_shadows(&mut state.listen_ports[lpi], sockets, 2);
                // Park the TDI_LISTEN; the bridge satisfies it from `ready`
                // (possibly this same pump iteration). conn checked non-None above.
                if let Some(c) = state.conn(conn_id) {
                    c.listen_irp = irp;
                    c.listen_ret = ret_info;
                    c.listen_port = local_port;
                }
                nsprint!("[NETSTACK] TDI LISTEN conn {} on :{}\n", conn_id, local_port);
            }
        }
        Cmd::QueryAddress { id, irp, out, out_len } => {
            // A connection object reports its socket's local endpoint; an address
            // object reports its bound port + the interface IP. ids are unique
            // across both tables (single allocator), so a lookup miss is benign.
            let (ip, port) = if let Some(c) = state.conns.iter().find(|c| c.id == id) {
                match c.handle.and_then(|h| sockets.get::<tcp::Socket>(h).local_endpoint()) {
                    Some(e) => (ip_octets(e.addr, iface), e.port),
                    None => (iface_ipv4(iface), 0),
                }
            } else if let Some(a) = state.addrs.iter().find(|a| a.id == id) {
                (iface_ipv4(iface), a.local_port)
            } else {
                (iface_ipv4(iface), 0)
            };
            let info = crate::tdi::TdiAddressInfo::new(ip, port);
            let need = core::mem::size_of::<crate::tdi::TdiAddressInfo>();
            if out != 0 && out_len >= need {
                with_pending(irp, || unsafe {
                    core::ptr::copy_nonoverlapping(
                        &info as *const _ as *const u8,
                        out as *mut u8,
                        need,
                    );
                });
            }
            unsafe { complete(irp, STATUS_SUCCESS, need) };
        }
        Cmd::SetDefaultRoute { gw, add } => {
            if add {
                let r = Ipv4Address::new(gw[0], gw[1], gw[2], gw[3]);
                let _ = iface.routes_mut().add_default_ipv4_route(r);
                nsprint!("[NETSTACK] route: default via {}.{}.{}.{}\n", gw[0], gw[1], gw[2], gw[3]);
            } else {
                iface.routes_mut().remove_default_ipv4_route();
                nsprint!("[NETSTACK] route: default removed\n");
            }
        }
        Cmd::Send { conn_id, irp, data, len } => {
            // Queue; the bridge drains into the socket as tx space frees and
            // completes the IRP only once ALL bytes are accepted (see send_q).
            match state.conn(conn_id) {
                Some(c) if c.handle.is_some() => c.send_q.push((irp, data, len, 0)),
                _ => unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) },
            }
        }
        Cmd::Receive { conn_id, irp, buf, len } => {
            if let Some(c) = state.conn(conn_id) {
                c.recv.push((irp, buf, len));
            } else {
                unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
            }
        }
        Cmd::Disconnect { conn_id, irp } => {
            if let Some(c) = state.conn(conn_id) {
                if let Some(h) = c.handle {
                    sockets.get_mut::<tcp::Socket>(h).close();
                }
            }
            unsafe { complete(irp, STATUS_SUCCESS, 0) };
        }
        Cmd::SendDatagram { addr_id, irp, data, len, ip, port } => {
            let mut done = STATUS_CONNECTION_REFUSED;
            let mut sent = 0usize;
            let mut src_port = 0u16;
            if let Some(a) = state.addr(addr_id) {
                if let Some(h) = ensure_udp(a, sockets) {
                    let s = sockets.get_mut::<udp::Socket>(h);
                    src_port = s.endpoint().port;
                    if s.can_send() {
                        let slice = unsafe { core::slice::from_raw_parts(data as *const u8, len) };
                        let ep = (IpAddress::v4(ip[0], ip[1], ip[2], ip[3]), port);
                        if s.send_slice(slice, ep).is_ok() {
                            sent = len;
                            done = STATUS_SUCCESS;
                        }
                    }
                }
            }
            if done == STATUS_SUCCESS && is_broadcast(iface, ip) {
                let slice = unsafe { core::slice::from_raw_parts(data as *const u8, len) };
                loop_back_broadcast(state, sockets, iface_ipv4(iface), src_port, port, slice);
            }
            unsafe { complete(irp, done, sent) };
        }
        Cmd::ReceiveDatagram { addr_id, irp, buf, len, ret_info } => {
            if let Some(a) = state.addr(addr_id) {
                ensure_udp(a, sockets);
                a.recv_dg.push((irp, buf, len, ret_info));
            } else {
                unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
            }
        }
        Cmd::Cancel { irp } => {
            // Already completed by the cancel routine. If the same address is pended
            // again, that is a NEW request reusing a freed IRP's memory: leave it.
            if is_pending(irp) {
                return;
            }
            for c in state.conns.iter_mut() {
                if c.connect_irp == irp {
                    c.connect_irp = 0;
                    if let Some(h) = c.handle {
                        sockets.get_mut::<tcp::Socket>(h).abort();
                    }
                }
                if c.listen_irp == irp {
                    c.listen_irp = 0;
                    c.listen_port = 0;
                }
                c.recv.retain(|e| e.0 != irp);
                c.send_q.retain(|e| e.0 != irp);
            }
            for a in state.addrs.iter_mut() {
                a.recv_dg.retain(|e| e.0 != irp);
            }
            state.pings.retain(|p| p.irp != irp);
        }
        Cmd::CloseConn { id } => {
            if let Some(pos) = state.conns.iter().position(|c| c.id == id) {
                if let Some(h) = state.conns[pos].handle {
                    sockets.remove(h);
                }
                state.conns.swap_remove(pos);
            }
        }
        Cmd::CloseAddr { id } => {
            if let Some(pos) = state.addrs.iter().position(|a| a.id == id) {
                if let Some(h) = state.addrs[pos].udp {
                    sockets.remove(h);
                }
                let port = state.addrs[pos].local_port;
                state.addrs.swap_remove(pos);
                // Last address object on a listened port tears down its
                // shadow-listener backlog (hygiene — server FCBs usually live
                // until process exit).
                if port != 0 && !state.addrs.iter().any(|a| a.local_port == port) {
                    if let Some(lpi) = state.listen_ports.iter().position(|lp| lp.port == port) {
                        let lp = state.listen_ports.swap_remove(lpi);
                        for h in lp.shadows.into_iter().chain(lp.ready.into_iter()) {
                            sockets.get_mut::<tcp::Socket>(h).abort();
                            sockets.remove(h);
                        }
                        nsprint!("[NETSTACK] TDI LISTEN backlog torn down on :{}\n", port);
                    }
                }
            }
        }
        Cmd::IcmpEcho { irp, sysbuf, out_len, user_buffer, dst, timeout_ms, data } => {
            ensure_icmp(state, sockets);
            // Unique non-zero sequence number to correlate the reply.
            state.icmp_seq = state.icmp_seq.wrapping_add(1);
            if state.icmp_seq == 0 {
                state.icmp_seq = 1;
            }
            let seq = state.icmp_seq;
            let now_ms = now().total_millis();
            // A 0 (or absurd) timeout must neither wedge the IRP nor fire instantly.
            let to = (timeout_ms as i64).clamp(1, 60_000);
            state.pings.push(PendingPing {
                irp,
                sysbuf,
                out_len,
                user_buffer,
                dst,
                seq,
                data,
                sent: false,
                start_ms: now_ms,
                deadline_ms: now_ms + to,
            });
            try_send_pings(state, sockets);
        }
    }
}

/// Lazily create + bind the management ICMP socket (the ping IOCTL backing).
fn ensure_icmp(state: &mut TdiState, sockets: &mut SocketSet<'static>) -> SocketHandle {
    if let Some(h) = state.icmp {
        return h;
    }
    let sock = icmp::Socket::new(
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 8], vec![0u8; 2048]),
        icmp::PacketBuffer::new(vec![icmp::PacketMetadata::EMPTY; 8], vec![0u8; 2048]),
    );
    let h = sockets.add(sock);
    let _ = sockets
        .get_mut::<icmp::Socket>(h)
        .bind(icmp::Endpoint::Ident(MGMT_ICMP_IDENT));
    state.icmp = Some(h);
    h
}

/// Emit echo requests for any not-yet-sent pending pings (the ICMP send buffer
/// may have been full when the command was first applied — retry each poll).
fn try_send_pings(state: &mut TdiState, sockets: &mut SocketSet<'static>) {
    let Some(h) = state.icmp else { return };
    for p in state.pings.iter_mut() {
        if p.sent {
            continue;
        }
        let s = sockets.get_mut::<icmp::Socket>(h);
        if !s.can_send() {
            break;
        }
        let dst = IpAddress::v4(p.dst[0], p.dst[1], p.dst[2], p.dst[3]);
        let mut sent_ok = false;
        {
            let repr = Icmpv4Repr::EchoRequest {
                ident: MGMT_ICMP_IDENT,
                seq_no: p.seq,
                data: &p.data,
            };
            if let Ok(buf) = s.send(repr.buffer_len(), dst) {
                let mut pkt = Icmpv4Packet::new_unchecked(buf);
                repr.emit(&mut pkt, &ChecksumCapabilities::default());
                sent_ok = true;
            }
        }
        if sent_ok {
            p.sent = true;
        } else {
            break; // buffer full mid-batch — try again next poll
        }
    }
}

/// ICMP management path: flush pending echo requests, deliver replies to their
/// pending IRPs (matched by sequence number), and time out the rest.
fn pump_icmp(sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    if state.icmp.is_none() {
        return;
    }
    try_send_pings(state, sockets);
    let Some(h) = state.icmp else { return };

    // Drain all available replies into owned tuples (so the socket borrow ends
    // before we touch the pending table / SystemBuffers).
    let mut replies: Vec<(u16, [u8; 4], Vec<u8>)> = Vec::new();
    loop {
        let s = sockets.get_mut::<icmp::Socket>(h);
        if !s.can_recv() {
            break;
        }
        let Ok((payload, src)) = s.recv() else { break };
        let Ok(pkt) = Icmpv4Packet::new_checked(payload) else { continue };
        let Ok(repr) = Icmpv4Repr::parse(&pkt, &ChecksumCapabilities::default()) else {
            continue;
        };
        if let Icmpv4Repr::EchoReply { ident, seq_no, data } = repr {
            if ident == MGMT_ICMP_IDENT {
                let octets = match src {
                    IpAddress::Ipv4(a) => a.octets(),
                };
                replies.push((seq_no, octets, data.to_vec()));
            }
        }
    }

    let now_ms = now().total_millis();
    for (seq, octets, data) in replies {
        if let Some(pos) = state.pings.iter().position(|p| p.sent && p.seq == seq) {
            let p = state.pings.swap_remove(pos);
            let rtt = (now_ms - p.start_ms).max(0) as u32;
            unsafe { complete_ping_success(&p, octets, &data, rtt) };
        }
    }

    // Time out pings whose deadline has passed.
    let mut i = 0;
    while i < state.pings.len() {
        if now_ms >= state.pings[i].deadline_ms {
            let p = state.pings.swap_remove(i);
            unsafe { complete_ping_timeout(&p) };
        } else {
            i += 1;
        }
    }
}

/// Fill the ICMP_ECHO_REPLY for a successful ping and complete the IRP. Mirrors
/// the C transport's ProcessEchoData: header at the front, echoed data at the END
/// of the output buffer, `Data` a back-pointer into the caller's address space.
unsafe fn complete_ping_success(p: &PendingPing, octets: [u8; 4], data: &[u8], rtt_ms: u32) {
    use crate::tdi::{IcmpEchoReply, IpOptionInformation, IP_SUCCESS};
    let reply_sz = core::mem::size_of::<IcmpEchoReply>();
    // Written only while no cancel can complete the IRP underneath.
    let written = with_pending(p.irp, || {
        // Zero the header region (the tail data area is filled below; the gap between
        // is irrelevant — only `Data`/`DataSize` bytes are read by the caller).
        core::ptr::write_bytes(p.sysbuf as *mut u8, 0, reply_sz);

        let reply = p.sysbuf as *mut IcmpEchoReply;
        (*reply).address = u32::from_ne_bytes(octets);
        (*reply).status = IP_SUCCESS;
        (*reply).round_trip_time = rtt_ms;
        (*reply).reserved = 1; // nReplies — IcmpParseReplies returns this
        (*reply).options = IpOptionInformation {
            // smoltcp's ICMP socket doesn't surface the reply's IP TTL; report a
            // plausible default (cosmetic — ping's TTL column only).
            ttl: 128,
            tos: 0,
            flags: 0,
            options_size: 0,
            options_data: core::ptr::null_mut(),
        };

        let remaining = p.out_len - reply_sz;
        let dsize = remaining.min(data.len());
        if dsize > 0 {
            // data sits at the very end of the buffer (out_len - dsize >= reply_sz).
            let dst = (p.sysbuf + p.out_len - dsize) as *mut u8;
            core::ptr::copy_nonoverlapping(data.as_ptr(), dst, dsize);
            (*reply).data = (p.user_buffer + p.out_len - dsize) as *mut u8;
        } else {
            (*reply).data = core::ptr::null_mut();
        }
        (*reply).data_size = dsize as u16;

    });
    if written.is_some() {
        complete(p.irp, STATUS_SUCCESS, p.out_len);
    }
}

/// Complete a timed-out ping: a zeroed ICMP_ECHO_REPLY with Status = IP_REQ_TIMED_OUT
/// (Reserved stays 0, so IcmpSendEcho2 reports the failure). Mirrors EndRequestHandler.
unsafe fn complete_ping_timeout(p: &PendingPing) {
    use crate::tdi::{IcmpEchoReply, IP_REQ_TIMED_OUT};
    let reply_sz = core::mem::size_of::<IcmpEchoReply>();
    let written = with_pending(p.irp, || {
        core::ptr::write_bytes(p.sysbuf as *mut u8, 0, reply_sz);
        (*(p.sysbuf as *mut IcmpEchoReply)).status = IP_REQ_TIMED_OUT;
    });
    if written.is_some() {
        complete(p.irp, STATUS_TIMEOUT, reply_sz);
    }
}

/// Lazily create + bind the UDP socket backing an address object (connectionless
/// datagram path). Bind to the requested local port, or an ephemeral derived from
/// the address id when AFD passed port 0 ("let the stack choose").
fn ensure_udp(addr: &mut Addr, sockets: &mut SocketSet<'static>) -> Option<SocketHandle> {
    if let Some(h) = addr.udp {
        return Some(h);
    }
    let sock = udp::Socket::new(
        udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0u8; 8192]),
        udp::PacketBuffer::new(vec![udp::PacketMetadata::EMPTY; 8], vec![0u8; 8192]),
    );
    let h = sockets.add(sock);
    let port = if addr.local_port != 0 {
        addr.local_port
    } else {
        49152u16.wrapping_add((addr.id & 0x3fff) as u16)
    };
    if sockets.get_mut::<udp::Socket>(h).bind(port).is_err() {
        sockets.remove(h);
        return None;
    }
    addr.udp = Some(h);
    Some(h)
}

/// The limited broadcast, or the directed broadcast of the interface's subnet.
fn is_broadcast(iface: &Interface, ip: [u8; 4]) -> bool {
    if ip == [255, 255, 255, 255] {
        return true;
    }
    iface.ip_addrs().iter().any(|c| match c {
        smoltcp::wire::IpCidr::Ipv4(v4) => v4.broadcast().map(|b| b.octets()) == Some(ip),
    })
}

/// Hand a broadcast this host just sent to every local address object bound to its
/// destination port. NDIS never loops a broadcast back to its sender and neither does
/// smoltcp, which handles broadcast only on egress. So without this, a socket on this host
/// never sees a broadcast another local socket sent (NetBIOS, SSDP-style self-discovery).
/// Upstream tcpip does the same thing through its loopback NCE (b8ee7950af4). The wire copy
/// has already gone out through `send_slice`.
fn loop_back_broadcast(
    state: &mut TdiState,
    sockets: &mut SocketSet<'static>,
    src_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    payload: &[u8],
) {
    /// Per address object; a listener nobody drains must not grow without bound.
    const LOCAL_DG_MAX: usize = 8;
    for a in state.addrs.iter_mut() {
        let bound = match a.udp {
            Some(h) => sockets.get_mut::<udp::Socket>(h).endpoint().port,
            None => a.local_port,
        };
        if bound == dst_port && a.local_dg.len() < LOCAL_DG_MAX {
            a.local_dg.push((payload.to_vec(), src_ip, src_port));
        }
    }
}

/// IpEndpoint -> (IPv4 octets, port). proto-ipv4 only, so the match is exhaustive.
fn ep_to_v4(ep: smoltcp::wire::IpEndpoint) -> ([u8; 4], u16) {
    let octets = match ep.addr {
        IpAddress::Ipv4(a) => a.octets(),
    };
    (octets, ep.port)
}

/// The interface's first IPv4 address (0.0.0.0 if unconfigured).
fn iface_ipv4(iface: &Interface) -> [u8; 4] {
    iface.ipv4_addr().map(|a| a.octets()).unwrap_or([0, 0, 0, 0])
}

/// An IpAddress's octets, falling back to the interface IP when unspecified
/// (a socket's local endpoint can read 0.0.0.0 until traffic flows).
fn ip_octets(a: IpAddress, iface: &Interface) -> [u8; 4] {
    match a {
        IpAddress::Ipv4(v4) => {
            let o = v4.octets();
            if o == [0, 0, 0, 0] { iface_ipv4(iface) } else { o }
        }
    }
}

/// Fill a TDI_CONNECTION_INFORMATION's RemoteAddress with a TA_IP_ADDRESS (the
/// accepted peer), updating RemoteAddressLength to the bytes written. AFD's
/// ListenComplete reads this to cache the connection's remote (getpeername).
unsafe fn fill_remote_address(ret_info: usize, octets: [u8; 4], port: u16) {
    let ci = ret_info as *mut crate::tdi::TDI_CONNECTION_INFORMATION;
    let dst = (*ci).RemoteAddress as *mut u8;
    if dst.is_null() {
        return;
    }
    let cap = (*ci).RemoteAddressLength as usize;
    let ta = crate::tdi::TaIpFull::new(octets, port);
    let n = core::mem::size_of::<crate::tdi::TaIpFull>().min(cap);
    core::ptr::copy_nonoverlapping(&ta as *const _ as *const u8, dst, n);
    (*ci).RemoteAddressLength = n as i32;
}

/// Deliver all queued datagrams on `s` via the AFD datagram event-indication
/// upcall (TDI_EVENT_RECEIVE_DATAGRAM). Each datagram is a whole message: smoltcp
/// dequeues it on recv() regardless of how many bytes AFD takes; if AFD returns a
/// receive IRP for the remainder we fill it with the untaken tail.
fn deliver_datagrams_event(s: &mut udp::Socket, handler: usize, context: usize) {
    while s.can_recv() {
        let Ok((payload, meta)) = s.recv() else { break };
        let (octets, port) = ep_to_v4(meta.endpoint);
        indicate_datagram(handler, context, payload, octets, port);
    }
}

/// One datagram to AFD's TDI_EVENT_RECEIVE_DATAGRAM handler.
fn indicate_datagram(handler: usize, context: usize, payload: &[u8], octets: [u8; 4], port: u16) {
    use crate::tdi::{
        PtdiIndReceiveDatagram, TaIpFull, TDI_RECEIVE_ENTIRE_MESSAGE, TDI_RECEIVE_NORMAL,
    };
    let mut src = TaIpFull::new(octets, port);
    let len = payload.len() as u32;
    let mut taken: u32 = 0;
    let mut irp_out: PIRP = core::ptr::null_mut();
    let f: PtdiIndReceiveDatagram = unsafe { core::mem::transmute(handler) };
    unsafe {
        // AFD datagram indications run at DISPATCH_LEVEL.
        let _irql = IrqlGuard::raise_to_dpc();
        f(
            context as *mut _,
            core::mem::size_of::<TaIpFull>() as i32,
            &mut src as *mut _ as *mut _,
            0,
            core::ptr::null_mut(),
            TDI_RECEIVE_NORMAL | TDI_RECEIVE_ENTIRE_MESSAGE,
            len,
            len,
            &mut taken,
            payload.as_ptr() as *mut _,
            &mut irp_out,
        );
    }
    let taken = (taken as usize).min(payload.len());
    if !irp_out.is_null() {
        unsafe { complete_indication_irp(irp_out, &payload[taken..]) };
    }
}

/// Deliver queued datagrams to pended RECEIVE_DATAGRAM IRPs (no event handler).
///
/// The sender's address goes in the IRP's ReturnDatagramInformation, which AFD copies
/// into the datagram it queues: left empty, recvfrom() reported every UDP reply as
/// coming from 0.0.0.0, and a resolver that checks the reply's source drops it.
fn deliver_datagrams_irp(s: &mut udp::Socket, recv_dg: &mut Vec<(usize, usize, usize, usize)>) {
    while !recv_dg.is_empty() && s.can_recv() {
        let (irp, buf, len, ret_info) = recv_dg.remove(0);
        // A cancelled IRP leaves the datagram queued for the next one.
        let got = with_pending(irp, || match s.recv() {
            Ok((payload, meta)) => {
                let n = payload.len().min(len);
                let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, n) };
                dst.copy_from_slice(&payload[..n]);
                if ret_info != 0 {
                    let (octets, port) = ep_to_v4(meta.endpoint);
                    unsafe { fill_remote_address(ret_info, octets, port) };
                }
                Ok(n)
            }
            Err(_) => Err(()),
        });
        match got {
            Some(Ok(n)) => unsafe { complete(irp, STATUS_SUCCESS, n) },
            Some(Err(())) => unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) },
            None => {}
        }
    }
}

/// Keep a ListenPort's shadow-listener pool at LISTEN_BACKLOG sockets.
fn top_up_shadows(lp: &mut ListenPort, sockets: &mut SocketSet<'static>, desired: usize) {
    // At most ONE new socket per call — a full 16-deep top-up in one pump
    // iteration allocated 256KiB in a burst and faulted the arm64 worker.
    // The bridge runs every poll, so the pool refills within a few polls.
    // `desired` is demand-tied (pended accepts + 1), capped at LISTEN_BACKLOG,
    // so an idle port never bloats past ~2 sockets (see LISTEN_BACKLOG).
    if lp.shadows.len() < desired {
        let sock = tcp::Socket::new(
            tcp::SocketBuffer::new(vec![0u8; SHADOW_BUF]),
            tcp::SocketBuffer::new(vec![0u8; SHADOW_BUF]),
        );
        let h = sockets.add(sock);
        if sockets.get_mut::<tcp::Socket>(h).listen(lp.port).is_ok() {
            lp.shadows.push(h);
        } else {
            sockets.remove(h);
        }
    }
}

/// Backlog bridge pass: harvest established shadow listeners into `ready`,
/// re-arm replacements, purge aborted handshakes, then satisfy pended
/// TDI_LISTEN IRPs FIFO from `ready`.
fn bridge_listen_backlog(sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    use tcp::State;
    // Demand-tied shadow target per port: one listening socket per pended
    // TDI_LISTEN (accept) plus one spare to absorb a SYN while an accept is
    // being satisfied, capped at LISTEN_BACKLOG. An idle port (no pended
    // accept) still keeps 1 spare so the first inbound SYN isn't RST'd.
    // Computed up front under an immutable borrow (disjoint from the mutable
    // shadow work below). Keeps arm64 Mm pressure minimal (see LISTEN_BACKLOG).
    let desired: Vec<(u16, usize)> = state
        .listen_ports
        .iter()
        .map(|lp| {
            let pended = state
                .conns
                .iter()
                .filter(|c| c.listen_port == lp.port && c.listen_irp != 0)
                .count();
            (lp.port, (pended + 1).min(LISTEN_BACKLOG))
        })
        .collect();
    for lp in state.listen_ports.iter_mut() {
        let mut i = 0;
        while i < lp.shadows.len() {
            let h = lp.shadows[i];
            let sock = sockets.get::<tcp::Socket>(h);
            match sock.state() {
                // A real inbound accept has a genuine peer endpoint. Require it:
                // an Established socket whose remote is absent/unspecified is NOT
                // a completed handshake we can hand to an accept — completing a
                // TDI_LISTEN with such a phantom made the kernel TDI-server
                // self-test (tdisrvtest.rs, raw TDI_LISTEN on :8888 that must
                // PEND until a real host connects) proceed into its post-accept
                // path against a peerless "connection" and wedge arm64 boot
                // (field manual §9.43 follow-up). Leave it a shadow until a real
                // peer arrives; when none does, the LISTEN pends — the correct
                // behavior, matching the pre-backlog path.
                State::Established
                    if sock
                        .remote_endpoint()
                        .is_some_and(|ep| ep.port != 0 && !ep.addr.is_unspecified()) =>
                {
                    lp.shadows.swap_remove(i);
                    lp.ready.push(h);
                }
                // Handshake died (RST mid-SYN, peer gave up): recycle.
                State::Closed => {
                    lp.shadows.swap_remove(i);
                    sockets.remove(h);
                }
                _ => i += 1,
            }
        }
        // Established-but-unclaimed connections the peer already abandoned are
        // useless to hand to an accept — drop them.
        let mut j = 0;
        while j < lp.ready.len() {
            let h = lp.ready[j];
            match sockets.get::<tcp::Socket>(h).state() {
                State::Closed => {
                    lp.ready.swap_remove(j);
                    sockets.remove(h);
                }
                _ => j += 1,
            }
        }
        let want = desired
            .iter()
            .find(|(p, _)| *p == lp.port)
            .map(|(_, n)| *n)
            .unwrap_or(1);
        top_up_shadows(lp, sockets, want);
    }
    // Satisfy pended TDI_LISTENs from ready queues (FIFO by conn order).
    for ci in 0..state.conns.len() {
        if state.conns[ci].listen_irp == 0 || state.conns[ci].listen_port == 0 {
            continue;
        }
        let port = state.conns[ci].listen_port;
        let Some(lp) = state.listen_ports.iter_mut().find(|lp| lp.port == port) else {
            continue;
        };
        if lp.ready.is_empty() {
            continue;
        }
        let h = lp.ready.remove(0);
        let irp = state.conns[ci].listen_irp;
        let ret_info = state.conns[ci].listen_ret;
        state.conns[ci].listen_irp = 0;
        state.conns[ci].listen_ret = 0;
        state.conns[ci].listen_port = 0;
        state.conns[ci].handle = Some(h);
        state.conns[ci].connected = true;
        if ret_info != 0 {
            if let Some(ep) = sockets.get::<tcp::Socket>(h).remote_endpoint() {
                let (octets, port) = ep_to_v4(ep);
                with_pending(irp, || unsafe { fill_remote_address(ret_info, octets, port) });
            }
        }
        nsprint!("[NETSTACK] TDI LISTEN accepted (conn {})\n", state.conns[ci].id);
        unsafe { complete(irp, STATUS_SUCCESS, 0) };
    }
}

/// Completion bridge: for each connection, complete a pended connect IRP once
/// ESTABLISHED, and deliver received data — via the AFD event-indication upcall
/// when a TDI_EVENT_RECEIVE handler is registered, else to pended receive IRPs.
fn bridge(sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    bridge_listen_backlog(sockets, state);
    for ci in 0..state.conns.len() {
        let Some(h) = state.conns[ci].handle else { continue };
        // Snapshot what we need from immutable lookups BEFORE the mutable socket /
        // conn[ci] borrows (the address object holds the receive event handler).
        let addr_id = state.conns[ci].addr_id;
        let conn_ctx = state.conns[ci].conn_ctx;
        let ev_recv = state
            .addrs
            .iter()
            .find(|a| a.id == addr_id)
            .and_then(|a| a.ev_recv);

        let s = sockets.get_mut::<tcp::Socket>(h);

        // Connect completion.
        if state.conns[ci].connect_irp != 0 {
            use tcp::State;
            match s.state() {
                State::Established => {
                    let irp = state.conns[ci].connect_irp;
                    state.conns[ci].connect_irp = 0;
                    state.conns[ci].connected = true;
                    nsprint!("[NETSTACK] TDI connect ESTABLISHED (conn {})\n", state.conns[ci].id);
                    unsafe { complete(irp, STATUS_SUCCESS, 0) };
                }
                State::Closed | State::TimeWait | State::Closing => {
                    let irp = state.conns[ci].connect_irp;
                    state.conns[ci].connect_irp = 0;
                    unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
                }
                // smoltcp never gives up on a SYN by itself, and one parked in SYN-SENT
                // for a host that never answers ARP re-ARPs every second -- smoltcp
                // rate-limits ARP for the whole interface, so that starves every other
                // new destination (the DNS server, for one). Fail it the way Windows
                // does, after ~21s of SYN retries.
                State::SynSent
                    if crate::time::now().total_millis() - state.conns[ci].connect_ms
                        > CONNECT_TIMEOUT_MS =>
                {
                    s.abort();
                    let irp = state.conns[ci].connect_irp;
                    state.conns[ci].connect_irp = 0;
                    nsprint!("[NETSTACK] TDI connect timed out in SYN-SENT (conn {})\n", state.conns[ci].id);
                    unsafe { complete(irp, STATUS_IO_TIMEOUT, 0) };
                }
                _ => {}
            }
        }

        // Listen completion now lives in bridge_listen_backlog (the shadow-
        // listener backlog): a conn with a pended TDI_LISTEN has NO socket
        // handle until the backlog assigns one, so this per-handle pass never
        // sees it. (Kept here as a comment for archaeology: the old model
        // armed one socket per TDI_LISTEN and completed it on Established.)
        {
        }

        // Send drain: push pended TDI_SEND bytes as tx space frees; complete
        // each IRP only when its WHOLE buffer is queued (TDI semantics — see
        // Conn.send_q). A dead socket fails the remaining queue loudly.
        while !state.conns[ci].send_q.is_empty() {
            use tcp::State;
            match s.state() {
                State::Established | State::CloseWait => {}
                _ => {
                    for (irp, _, _, _) in state.conns[ci].send_q.drain(..) {
                        unsafe { complete(irp, STATUS_CONNECTION_RESET, 0) };
                    }
                    break;
                }
            }
            let (irp, data, len, off) = state.conns[ci].send_q[0];
            let Some(n) = with_pending(irp, || {
                let remainder =
                    unsafe { core::slice::from_raw_parts((data + off) as *const u8, len - off) };
                if s.can_send() { s.send_slice(remainder).unwrap_or(0) } else { 0 }
            }) else {
                state.conns[ci].send_q.remove(0); // cancelled
                continue;
            };
            if off + n == len {
                state.conns[ci].send_q.remove(0);
                unsafe { complete(irp, STATUS_SUCCESS, len) };
            } else {
                state.conns[ci].send_q[0].3 = off + n;
                break; // tx buffer full — resume next pump iteration
            }
        }

        // Receive delivery.
        if let Some((handler, context)) = ev_recv {
            // AFD model: upcall the registered TDI_EVENT_RECEIVE handler.
            indicate_receive(s, handler, context, conn_ctx);
        } else {
            // Legacy IRP-receive model (kernel self-test before the event path).
            while !state.conns[ci].recv.is_empty() && s.can_recv() {
                let (irp, buf, len) = state.conns[ci].recv.remove(0);
                let got = with_pending(irp, || {
                    let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
                    s.recv_slice(dst)
                });
                match got {
                    Some(Ok(n)) => unsafe { complete(irp, STATUS_SUCCESS, n) },
                    Some(Err(_)) => unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) },
                    None => {} // cancelled
                }
            }
        }
    }

    // Connectionless (UDP) datagram delivery on address objects.
    for ai in 0..state.addrs.len() {
        // Locally looped broadcasts first (they need no smoltcp socket behind the address).
        let a = &mut state.addrs[ai];
        if let Some((handler, context)) = a.ev_recv_dg {
            for (payload, ip, port) in a.local_dg.drain(..) {
                indicate_datagram(handler, context, &payload, ip, port);
            }
        } else {
            while !a.local_dg.is_empty() && !a.recv_dg.is_empty() {
                let (irp, buf, len, ret_info) = a.recv_dg.remove(0);
                let (payload, ip, port) = &a.local_dg[0];
                let n = payload.len().min(len);
                let copied = with_pending(irp, || unsafe {
                    core::ptr::copy_nonoverlapping(payload.as_ptr(), buf as *mut u8, n);
                    if ret_info != 0 {
                        fill_remote_address(ret_info, *ip, *port);
                    }
                });
                if copied.is_some() {
                    a.local_dg.remove(0);
                    unsafe { complete(irp, STATUS_SUCCESS, n) };
                }
            }
        }
        let Some(h) = state.addrs[ai].udp else { continue };
        let ev = state.addrs[ai].ev_recv_dg;
        let s = sockets.get_mut::<udp::Socket>(h);
        if let Some((handler, context)) = ev {
            deliver_datagrams_event(s, handler, context);
        } else {
            deliver_datagrams_irp(s, &mut state.addrs[ai].recv_dg);
        }
    }
}

/// Upcall AFD's TDI_EVENT_RECEIVE handler for all currently-available stream data
/// on `s`. The handler returns how many bytes it took (BytesTaken); we consume
/// exactly that from the socket. If it takes nothing (STATUS_DATA_NOT_ACCEPTED)
/// we stop and re-indicate next poll. A returned receive IRP is completed with
/// whatever the handler didn't take inline.
fn indicate_receive(s: &mut tcp::Socket, handler: usize, context: usize, conn_ctx: usize) {
    use crate::tdi::{PtdiIndReceive, TDI_RECEIVE_ENTIRE_MESSAGE, TDI_RECEIVE_NORMAL};
    loop {
        if !s.can_recv() {
            break;
        }
        // recv() hands us a contiguous slice of the receive ring; we return how many
        // bytes to consume (and surface that count as the call's result).
        let consumed = s
            .recv(|buf: &mut [u8]| {
                if buf.is_empty() {
                    return (0usize, 0usize);
                }
                let len = buf.len() as u32;
                let mut taken: u32 = 0;
                let mut irp_out: PIRP = core::ptr::null_mut();
                let f: PtdiIndReceive = unsafe { core::mem::transmute(handler) };
                let st = unsafe {
                    // AFD receive indications run at DISPATCH_LEVEL.
                    let _irql = IrqlGuard::raise_to_dpc();
                    f(
                        context as *mut _,
                        conn_ctx as *mut _,
                        TDI_RECEIVE_NORMAL | TDI_RECEIVE_ENTIRE_MESSAGE,
                        len,
                        len,
                        &mut taken,
                        buf.as_ptr() as *mut _,
                        &mut irp_out,
                    )
                };
                let mut consumed = (taken as usize).min(buf.len());
                if !irp_out.is_null() {
                    // AFD posted an IRP for the remainder — fill its MDL.
                    let extra = unsafe { complete_indication_irp(irp_out, &buf[consumed..]) };
                    consumed += extra;
                }
                let _ = st;
                (consumed, consumed)
            })
            .unwrap_or(0);
        if consumed == 0 {
            break;
        }
    }
}

/// Complete a TDI receive IRP returned from an event indication by copying the
/// remaining indicated data into its MDL. Returns the number of bytes delivered.
unsafe fn complete_indication_irp(irp: PIRP, data: &[u8]) -> usize {
    let mdl = (*irp).MdlAddress;
    let mut n = 0usize;
    if !mdl.is_null() {
        let va = ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe(mdl, 16);
        if !va.is_null() {
            let cap = (*(mdl as *const ntkrnl_sys::ddk::_MDL)).ByteCount as usize;
            n = data.len().min(cap);
            core::ptr::copy_nonoverlapping(data.as_ptr(), va as *mut u8, n);
        }
    }
    complete_unregistered(irp as usize, STATUS_SUCCESS, n);
    n
}
