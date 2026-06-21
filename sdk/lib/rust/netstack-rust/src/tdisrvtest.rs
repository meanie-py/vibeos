//! Kernel-mode TDI **server** self-test — the inbound analogue of `tditest.rs`.
//! Exercises netstack's M8 server-side path (`TDI_LISTEN`/`TDI_ACCEPT`) over the
//! exact IRP sequence AFD's `AfdListenSocket`/`AfdAccept` use: open an address
//! object bound to a known port, open + associate a connection object, register a
//! `TDI_EVENT_RECEIVE` handler, then post a `TDI_LISTEN` that PENDS until a host
//! connects. On completion (smoltcp finished the handshake in place → the M8
//! `bridge()` completed the pended listen IRP), it `TDI_ACCEPT`s, waits for the
//! peer's first bytes (the receive upcall fires), and `TDI_SEND`s a fixed echo
//! marker back. Proves LISTEN-completes + receive-event + SEND on a server-
//! accepted connection — end to end, no usermode harness.
//!
//! The host side (`tests/qemu/test_net.py::test_tdi_server_listen_accept`) connects
//! to the guest's `SRV_PORT` via a slirp hostfwd, sends a probe, and expects the
//! echo marker — the inbound mirror of the outbound `cmd:cat` client test.

use core::ffi::c_void;
use core::mem::{size_of, zeroed};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, ObReferenceObjectByHandle, ObfDereferenceObject, PVOID,
    PsCreateSystemThread, ZwClose, _FILE_OBJECT,
};

use crate::engine::DHCP_READY;
use crate::serial::nsprint;
use crate::tdi;
use crate::tditest::{open_dev, run_tdi, run_tdi_to};
use crate::time::delay_ms;

/// The guest TCP port the kernel server listens on. Distinct from the engine's
/// direct-smoltcp echo (:7777) so the two don't collide — this one rides the TDI
/// dispatch (the M8 path under test), that one is the raw socket loop.
const SRV_PORT: u16 = 8888;
const ECHO: &[u8] = b"TDI-SRV-ECHO\n";

static RECV_FIRED: AtomicBool = AtomicBool::new(false);
static RECV_LEN: AtomicUsize = AtomicUsize::new(0);

const fn wdev<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}
static TCP_NAME: [u16; 11] = wdev(b"\\Device\\Tcp");

/// TRANSPORT_ADDRESS + TA_ADDRESS + TDI_ADDRESS_IP, byte-packed (see tditest.rs).
#[repr(C, packed)]
struct TaIp {
    count: i32,
    addr_len: u16,
    addr_type: u16,
    sin_port: u16,
    in_addr: u32,
    sin_zero: [u8; 8],
}

impl TaIp {
    fn new(ip: [u8; 4], port: u16) -> Self {
        TaIp {
            count: 1,
            addr_len: 14,
            addr_type: tdi::TDI_ADDRESS_TYPE_IP,
            sin_port: port.to_be(),
            in_addr: u32::from_ne_bytes(ip),
            sin_zero: [0; 8],
        }
    }
}

/// FILE_FULL_EA_INFORMATION + "TransportAddress" + a local TA_IP_ADDRESS.
#[repr(C, packed)]
struct AddrEa {
    next: u32,
    flags: u8,
    name_len: u8,
    value_len: u16,
    name: [u8; 17],
    value: TaIp,
}

/// FILE_FULL_EA_INFORMATION + "ConnectionContext" + an 8-byte context cookie.
#[repr(C, packed)]
struct ConnEa {
    next: u32,
    flags: u8,
    name_len: u8,
    value_len: u16,
    name: [u8; 18],
    value: u64,
}

/// The TDI_EVENT_RECEIVE upcall netstack fires on the server connection when the
/// peer's stream bytes arrive. We accept them inline and flag arrival; the worker
/// then sends the fixed echo marker back (proving the server-side SEND path).
unsafe extern "system" fn recv_indication(
    _tdi_ctx: *mut c_void,
    _conn_ctx: *mut c_void,
    _flags: u32,
    bytes_indicated: u32,
    _bytes_available: u32,
    bytes_taken: *mut u32,
    _tsdu: *mut c_void,
    irp_out: *mut ntkrnl_sys::ddk::PIRP,
) -> i32 {
    *bytes_taken = bytes_indicated; // accept all inline; no receive IRP
    if !irp_out.is_null() {
        *irp_out = null_mut();
    }
    RECV_LEN.store(bytes_indicated as usize, Ordering::Release);
    RECV_FIRED.store(true, Ordering::Release);
    0
}

pub fn spawn() {
    let mut h = null_mut();
    let start: ntkrnl_sys::ddk::PKSTART_ROUTINE =
        Some(unsafe { core::mem::transmute(worker as *const ()) });
    unsafe {
        PsCreateSystemThread(&mut h, 0x001F_03FF, null_mut(), null_mut(), null_mut(), start, null_mut());
    }
}

unsafe extern "system" fn worker(_ctx: PVOID) {
    // Need an address (DHCP) before we can listen — same gate as the client test.
    let mut waited = 0;
    while !DHCP_READY.load(Ordering::Acquire) && waited < 120 {
        delay_ms(250);
        waited += 1;
    }
    if !DHCP_READY.load(Ordering::Acquire) {
        nsprint!("[NETSTACK] TDI server: skipped (no network device)\n");
        return;
    }
    delay_ms(750);
    nsprint!("[NETSTACK] TDI server: opening \\Device\\Tcp\n");

    // Address object bound to the listen port (0.0.0.0:SRV_PORT). The M8 listen
    // path binds the smoltcp socket to *this* port (via the associated address
    // object), so it must be the real server port, not 0.
    let mut addr_ea: AddrEa = zeroed();
    addr_ea.name_len = 16;
    addr_ea.value_len = size_of::<TaIp>() as u16;
    addr_ea.name[..16].copy_from_slice(b"TransportAddress");
    addr_ea.value = TaIp::new([0, 0, 0, 0], SRV_PORT);
    let addr_h = open_dev(&TCP_NAME, &mut addr_ea as *mut _ as PVOID, size_of::<AddrEa>() as u32);
    if addr_h.is_null() {
        nsprint!("[NETSTACK] TDI server: open address failed\n");
        return;
    }

    // Connection object (one backlog slot).
    let mut conn_ea: ConnEa = zeroed();
    conn_ea.name_len = 17;
    conn_ea.value_len = 8;
    conn_ea.name[..17].copy_from_slice(b"ConnectionContext");
    conn_ea.value = 0x5E12_F00D;
    let conn_h = open_dev(&TCP_NAME, &mut conn_ea as *mut _ as PVOID, size_of::<ConnEa>() as u32);
    if conn_h.is_null() {
        nsprint!("[NETSTACK] TDI server: open connection failed\n");
        ZwClose(addr_h);
        return;
    }

    // Resolve connection + address FILE_OBJECTs.
    let mut conn_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(conn_h, 0, null_mut(), 0, &mut conn_obj, null_mut()) != 0 {
        ZwClose(conn_h);
        ZwClose(addr_h);
        return;
    }
    let conn_fo = conn_obj as *mut _FILE_OBJECT;
    let dev = (*conn_fo).DeviceObject;

    let mut addr_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(addr_h, 0, null_mut(), 0, &mut addr_obj, null_mut()) != 0 {
        ObfDereferenceObject(conn_obj);
        ZwClose(conn_h);
        ZwClose(addr_h);
        return;
    }
    let addr_fo = addr_obj as *mut _FILE_OBJECT;

    // ASSOCIATE the connection with the bound address object — this is what gives
    // the listen socket its local port (SRV_PORT).
    let st = run_tdi(dev, conn_fo, tdi::TDI_ASSOCIATE_ADDRESS, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_ASSOCIATE;
        (*req).AddressHandle = addr_h;
    });
    nsprint!("[NETSTACK] TDI server: associate {:#010x}\n", st as u32);

    // Register the TDI_EVENT_RECEIVE handler on the ADDRESS object (AFD model) so
    // the peer's inbound bytes upcall us after accept.
    let st = run_tdi(dev, addr_fo, tdi::TDI_SET_EVENT_HANDLER, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_SET_EVENT;
        (*req).EventType = tdi::TDI_EVENT_RECEIVE as u32;
        (*req).EventHandler = recv_indication as *mut c_void;
        (*req).EventContext = 0x5E12_0000 as *mut c_void;
    });
    nsprint!("[NETSTACK] TDI server: set-event(receive) {:#010x}\n", st as u32);

    // POST TDI_LISTEN — pends until a host connects (the M8 path under test).
    // ReturnConnectionInformation receives the peer address on completion.
    let mut peer = TaIp::new([0, 0, 0, 0], 0);
    let mut ret_info: tdi::TDI_CONNECTION_INFORMATION = zeroed();
    ret_info.RemoteAddress = &mut peer as *mut _ as *mut core::ffi::c_void;
    ret_info.RemoteAddressLength = size_of::<TaIp>() as i32;
    nsprint!("[NETSTACK] TDI server: listening on :{}\n", SRV_PORT);
    // 5-minute listen window: the host (test_tdi_server_listen_accept) connects
    // only after boot completes, well past the 10s client default.
    let st = run_tdi_to(dev, conn_fo, tdi::TDI_LISTEN, null_mut(), 0, -3_000_000_000i64, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL;
        (*req).RequestConnectionInformation = null_mut();
        (*req).ReturnConnectionInformation = &mut ret_info;
        (*req).RequestFlags = 0;
    });
    if st != 0 {
        nsprint!("[NETSTACK] TDI server: LISTEN did not complete {:#010x}\n", st as u32);
        cleanup(addr_obj, conn_obj, conn_h, addr_h);
        return;
    }
    let o = peer.in_addr.to_ne_bytes();
    let pport = u16::from_be(peer.sin_port);
    nsprint!(
        "[NETSTACK] TDI server: ACCEPTED from {}.{}.{}.{}:{}\n",
        o[0], o[1], o[2], o[3], pport
    );

    // TDI_ACCEPT — synchronous SUCCESS (smoltcp already established the conn).
    let st = run_tdi(dev, conn_fo, tdi::TDI_ACCEPT, null_mut(), 0, |_p| {});
    nsprint!("[NETSTACK] TDI server: accept {:#010x}\n", st as u32);

    // Wait for the peer's first bytes (receive upcall), then echo a fixed marker.
    let mut waited = 0;
    while !RECV_FIRED.load(Ordering::Acquire) && waited < 40 {
        delay_ms(250);
        waited += 1;
    }
    if !RECV_FIRED.load(Ordering::Acquire) {
        nsprint!("[NETSTACK] TDI server: receive upcall TIMED OUT\n");
        cleanup(addr_obj, conn_obj, conn_h, addr_h);
        return;
    }
    nsprint!("[NETSTACK] TDI server: received {} bytes from peer\n", RECV_LEN.load(Ordering::Acquire));

    // SEND the echo marker. METHOD_OUT_DIRECT maps the buffer via MDL → must be
    // pool, not stack (same constraint as the client send).
    let txbuf = ExAllocatePoolWithTag(0 /* NonPagedPool */ as _, ECHO.len() as _, u32::from_le_bytes(*b"NetS")) as *mut u8;
    if !txbuf.is_null() {
        core::ptr::copy_nonoverlapping(ECHO.as_ptr(), txbuf, ECHO.len());
        let st = run_tdi(dev, conn_fo, tdi::TDI_SEND, txbuf as PVOID, ECHO.len() as u32, |p| {
            let req = p as *mut tdi::TDI_REQUEST_KERNEL_SEND;
            (*req).SendLength = ECHO.len() as u32;
            (*req).SendFlags = 0;
        });
        nsprint!("[NETSTACK] TDI server: echoed {} bytes {:#010x} — server round-trip OK\n", ECHO.len(), st as u32);
        ExFreePool(txbuf as PVOID);
    }

    cleanup(addr_obj, conn_obj, conn_h, addr_h);
}

unsafe fn cleanup(addr_obj: PVOID, conn_obj: PVOID, conn_h: *mut c_void, addr_h: *mut c_void) {
    ObfDereferenceObject(addr_obj);
    ObfDereferenceObject(conn_obj);
    ZwClose(conn_h);
    ZwClose(addr_h);
}
