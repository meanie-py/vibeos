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
use smoltcp::socket::tcp;
use smoltcp::wire::IpAddress;

use ntkrnl_sys::ddk::{IofCompleteRequest, NTSTATUS, PIRP};
use ntkrnl_sys::guards::IrqlGuard;

use crate::serial::nsprint;

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_CONNECTION_REFUSED: NTSTATUS = 0xC000_0236u32 as NTSTATUS;

/// Worker→IRP completion. Safe to call at PASSIVE with no engine lock held.
unsafe fn complete(irp: usize, status: NTSTATUS, info: usize) {
    if irp == 0 {
        return;
    }
    let irp = irp as PIRP;
    (*irp).IoStatus.__bindgen_anon_1.Status = status;
    (*irp).IoStatus.Information = info as _;
    IofCompleteRequest(irp, 0);
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
    Send { conn_id: u64, irp: usize, data: usize, len: usize },
    Receive { conn_id: u64, irp: usize, buf: usize, len: usize },
    Disconnect { conn_id: u64, irp: usize },
    CloseConn { id: u64 },
    CloseAddr { id: u64 },
}

/// The inbox is guarded by raising IRQL to DISPATCH_LEVEL across the brief
/// push/swap. QEMU runs a SINGLE vCPU, so DISPATCH_LEVEL (which masks thread
/// preemption) is full mutual exclusion between the producer (TDI dispatch,
/// any thread, PASSIVE) and the consumer (poll worker, PASSIVE).
///
/// NOTE: `KeAcquireSpinLockForDpc` is a *no-op* in this tree (ntoskrnl/ke/
/// spinlock.c returns the current IRQL without raising or acquiring) — it was
/// stubbed for the early-boot Rust pool allocator. Using it here gave ZERO
/// synchronization (an unsynchronized Vec race between two threads = UB/heap
/// corruption). IrqlGuard uses the real KfRaiseIrql, so this is correct.
struct Inbox {
    queue: UnsafeCell<Vec<Cmd>>,
}
unsafe impl Sync for Inbox {}

static INBOX: Inbox = Inbox {
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
        let _irql = IrqlGuard::raise_to_dpc();
        (*INBOX.queue.get()).push(cmd);
    }
}

fn drain_into(out: &mut Vec<Cmd>) {
    unsafe {
        let _irql = IrqlGuard::raise_to_dpc();
        core::mem::swap(out, &mut *INBOX.queue.get());
    }
}

// --------------------------------------------------------------------------- #
// Worker-private connection/address tables + the completion bridge.
// --------------------------------------------------------------------------- #

struct Addr {
    id: u64,
    ev_recv: Option<(usize, usize)>,
    ev_disc: Option<(usize, usize)>,
}

struct Conn {
    id: u64,
    addr_id: u64,
    conn_ctx: usize,
    handle: Option<SocketHandle>,
    connect_irp: usize,
    recv: Vec<(usize, usize, usize)>, // (irp, buf_va, buf_len)
    connected: bool,
}

pub struct TdiState {
    addrs: Vec<Addr>,
    conns: Vec<Conn>,
    cmds: Vec<Cmd>,
}

impl TdiState {
    pub fn new() -> Self {
        Self { addrs: Vec::new(), conns: Vec::new(), cmds: Vec::new() }
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
}

fn apply(cmd: Cmd, iface: &mut Interface, sockets: &mut SocketSet<'static>, state: &mut TdiState) {
    match cmd {
        Cmd::CreateAddr { id, port: _ } => {
            state.addrs.push(Addr { id, ev_recv: None, ev_disc: None });
        }
        Cmd::CreateConn { id, conn_ctx } => {
            state.conns.push(Conn {
                id,
                addr_id: 0,
                conn_ctx,
                handle: None,
                connect_irp: 0,
                recv: Vec::new(),
                connected: false,
            });
        }
        Cmd::SetEvent { addr_id, ev_type, handler, context } => {
            if let Some(a) = state.addr(addr_id) {
                match ev_type as i32 {
                    crate::tdi::TDI_EVENT_RECEIVE => a.ev_recv = Some((handler, context)),
                    crate::tdi::TDI_EVENT_DISCONNECT => a.ev_disc = Some((handler, context)),
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
                } else {
                    unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
                }
            } else {
                unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) };
            }
        }
        Cmd::Send { conn_id, irp, data, len } => {
            let handle = state.conn(conn_id).and_then(|c| c.handle);
            let mut done = STATUS_CONNECTION_REFUSED;
            let mut sent = 0usize;
            if let Some(h) = handle {
                let s = sockets.get_mut::<tcp::Socket>(h);
                if s.can_send() {
                    let slice = unsafe { core::slice::from_raw_parts(data as *const u8, len) };
                    if let Ok(n) = s.send_slice(slice) {
                        sent = n;
                        done = STATUS_SUCCESS;
                    }
                }
            }
            unsafe { complete(irp, done, sent) };
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
                state.addrs.swap_remove(pos);
            }
        }
    }
}

/// Completion bridge: for each connection, complete a pended connect IRP once
/// ESTABLISHED, and deliver received data — via the AFD event-indication upcall
/// when a TDI_EVENT_RECEIVE handler is registered, else to pended receive IRPs.
fn bridge(sockets: &mut SocketSet<'static>, state: &mut TdiState) {
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
                _ => {}
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
                let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
                match s.recv_slice(dst) {
                    Ok(n) => unsafe { complete(irp, STATUS_SUCCESS, n) },
                    Err(_) => unsafe { complete(irp, STATUS_CONNECTION_REFUSED, 0) },
                }
            }
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
    complete(irp as usize, STATUS_SUCCESS, n);
    n
}
