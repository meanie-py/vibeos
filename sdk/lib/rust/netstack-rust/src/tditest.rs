//! Kernel-mode TDI client self-test — exercises netstack's own \Device\Tcp via
//! real IRPs (the exact path AFD uses): open address + connection objects,
//! associate, connect to the slirp `cmd:cat` echo (10.0.2.100:9999), send, then
//! receive the echo. Proves the TDI dispatch + completion bridge end to end with
//! no usermode harness. Runs on its own system thread so the poll worker is free
//! to drive smoltcp and complete the pended IRPs.

use core::ffi::c_void;
use core::mem::{size_of, zeroed};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest,
    IofCallDriver, KeInitializeEvent, KeWaitForSingleObject, KEVENT, LARGE_INTEGER,
    OBJECT_ATTRIBUTES, ObReferenceObjectByHandle, ObfDereferenceObject, PDEVICE_OBJECT, PVOID,
    PsCreateSystemThread, UNICODE_STRING, ZwClose, ZwCreateFile, _FILE_OBJECT,
};
use ntkrnl_sys::inline::tier1::IoGetNextIrpStackLocation;

use crate::engine::DHCP_READY;
use crate::serial::nsprint;
use crate::tdi;
use crate::time::delay_ms;

const STATUS_PENDING: i32 = 0x0000_0103;
const STATUS_TIMEOUT: i32 = 0x0000_0102;
const STATUS_IO_TIMEOUT: i32 = 0xC000_00B5u32 as i32;
const OBJ_CASE_INSENSITIVE: u32 = 0x40;
const OBJ_KERNEL_HANDLE: u32 = 0x0000_0200;
const FILE_OPEN: u32 = 1;
const GENERIC_RW: u32 = 0xC000_0000;
const FILE_SHARE_RW: u32 = 3;
const IRP_MJ_INTERNAL_DEVICE_CONTROL: u8 = 0x0F;
/// METHOD_OUT_DIRECT — makes IoBuildDeviceIoControlRequest build Irp->MdlAddress
/// from the OutputBuffer (used for SEND/RECEIVE data). NT method codes are
/// BUFFERED=0, IN_DIRECT=1, OUT_DIRECT=2, NEITHER=3 — OUT_DIRECT is 2 (passing 3
/// is METHOD_NEITHER, which builds NO MDL → the transport sees MdlAddress=NULL).
const METHOD_OUT_DIRECT: u32 = 2;

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

/// TRANSPORT_ADDRESS + TA_ADDRESS + TDI_ADDRESS_IP, matching the kernel's
/// NATURALLY-ALIGNED (`#[repr(C)]`) layout — the same one netstack's
/// parse_remote_ip and real AFD use. The critical subtlety: `TDI_ADDRESS_IP`
/// is `{ USHORT sin_port; ULONG in_addr; ... }`, and the ULONG forces 4-byte
/// alignment, so there are 2 bytes of padding AFTER sin_port. Declaring this
/// `packed` (no _pad) puts in_addr 2 bytes early — the transport then reads
/// the wrong octets (e.g. 10.0.2.100 → 2.100.0.0) and the connect never
/// establishes. Keep the explicit `_pad`.
#[repr(C, packed)]
struct TaIp {
    count: i32,        // @0  TAAddressCount
    addr_len: u16,     // @4  TA_ADDRESS.AddressLength
    addr_type: u16,    // @6  TA_ADDRESS.AddressType
    sin_port: u16,     // @8  TDI_ADDRESS_IP.sin_port
    _pad: u16,         // @10 ULONG alignment padding before in_addr
    in_addr: u32,      // @12 TDI_ADDRESS_IP.in_addr
    sin_zero: [u8; 8], // @16
}

impl TaIp {
    fn new(ip: [u8; 4], port: u16) -> Self {
        TaIp {
            count: 1,
            addr_len: 14, // TDI_ADDRESS_LENGTH_IP (sin_port + in_addr + sin_zero)
            addr_type: tdi::TDI_ADDRESS_TYPE_IP,
            sin_port: port.to_be(),
            _pad: 0,
            in_addr: u32::from_ne_bytes(ip), // memory bytes == network octets
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
    name: [u8; 17], // "TransportAddress\0"
    value: TaIp,
}

/// FILE_FULL_EA_INFORMATION + "ConnectionContext" + an 8-byte context cookie.
#[repr(C, packed)]
struct ConnEa {
    next: u32,
    flags: u8,
    name_len: u8,
    value_len: u16,
    name: [u8; 18], // "ConnectionContext\0"
    value: u64,
}

// The TDI_EVENT_RECEIVE upcall netstack fires when the echo arrives — the EXACT
// path real AFD uses (it registers a receive handler on the address object; the
// transport upcalls it). We record that it fired + the byte count to prove the
// event-indication path end to end.
static RECV_FIRED: AtomicBool = AtomicBool::new(false);
static RECV_LEN: AtomicUsize = AtomicUsize::new(0);
static RECV_CONN_CTX: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn recv_indication(
    _tdi_ctx: *mut c_void,
    conn_ctx: *mut c_void,
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
    RECV_CONN_CTX.store(conn_ctx as usize, Ordering::Release);
    RECV_FIRED.store(true, Ordering::Release);
    0 // STATUS_SUCCESS
}

pub fn spawn() {
    let mut h = null_mut();
    let start: ntkrnl_sys::ddk::PKSTART_ROUTINE =
        Some(unsafe { core::mem::transmute(worker as *const ()) });
    unsafe {
        PsCreateSystemThread(&mut h, 0x001F_03FF, null_mut(), null_mut(), null_mut(), start, null_mut());
    }
}

unsafe fn open_tcp(ea: PVOID, ea_len: u32) -> *mut core::ffi::c_void {
    open_dev(&TCP_NAME, ea, ea_len)
}

/// Open a transport device file object (control/address/connection per the EA).
/// Shared by the TCP and UDP self-tests.
pub(crate) unsafe fn open_dev(
    dev_name: &[u16],
    ea: PVOID,
    ea_len: u32,
) -> *mut core::ffi::c_void {
    let mut name = UNICODE_STRING {
        Length: (dev_name.len() * 2) as u16,
        MaximumLength: (dev_name.len() * 2) as u16,
        Buffer: dev_name.as_ptr() as *mut u16,
    };
    let mut oa: OBJECT_ATTRIBUTES = zeroed();
    oa.Length = size_of::<OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = &mut name;
    oa.Attributes = OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE;
    let mut iosb: IO_STATUS_BLOCK = zeroed();
    let mut handle = null_mut();
    nsprint!("[NETSTACK] TDI self-test: ZwCreateFile enter (ea_len={})\n", ea_len);
    let st = ZwCreateFile(
        &mut handle,
        GENERIC_RW,
        &mut oa,
        &mut iosb,
        null_mut(),
        0,
        FILE_SHARE_RW,
        FILE_OPEN,
        0,
        ea,
        ea_len,
    );
    if st != 0 {
        nsprint!("[NETSTACK] TDI self-test: ZwCreateFile failed {:#010x}\n", st as u32);
        return null_mut();
    }
    nsprint!("[NETSTACK] TDI self-test: ZwCreateFile OK\n");
    handle
}

/// Run one synchronous internal-device-control IRP against the connection.
pub(crate) unsafe fn run_tdi(
    dev: PDEVICE_OBJECT,
    fo: *mut _FILE_OBJECT,
    minor: u8,
    out_buf: PVOID,
    out_len: u32,
    setup_params: impl FnOnce(*mut u8),
) -> i32 {
    let mut event: KEVENT = zeroed();
    KeInitializeEvent(&mut event, 0 /* NotificationEvent */, 0);
    let mut iosb: IO_STATUS_BLOCK = zeroed();
    let irp = IoBuildDeviceIoControlRequest(
        METHOD_OUT_DIRECT,
        dev,
        null_mut(),
        0,
        out_buf,
        out_len,
        1, // InternalDeviceIoControl
        &mut event,
        &mut iosb,
    );
    if irp.is_null() {
        return -1;
    }
    let sp = IoGetNextIrpStackLocation(irp);
    (*sp).MajorFunction = IRP_MJ_INTERNAL_DEVICE_CONTROL;
    (*sp).MinorFunction = minor;
    (*sp).FileObject = fo;
    setup_params(&mut (*sp).Parameters as *mut _ as *mut u8);

    let mut st = IofCallDriver(dev, irp);
    if st == STATUS_PENDING {
        let mut timeout: LARGE_INTEGER = zeroed();
        timeout.QuadPart = -100_000_000i64; // 10s relative
        let wait = KeWaitForSingleObject(&mut event as *mut _ as PVOID, 0, 0, 0, &mut timeout);
        // Honesty: on timeout the IRP never completed, so iosb is untouched
        // (reads as 0 = STATUS_SUCCESS) — report the timeout instead of a false OK.
        st = if wait == STATUS_TIMEOUT { STATUS_IO_TIMEOUT } else { iosb.__bindgen_anon_1.Status };
    }
    st
}

unsafe extern "system" fn worker(_ctx: PVOID) {
    // Wait for DHCP (need a source address) + a short settle.
    let mut waited = 0;
    while !DHCP_READY.load(Ordering::Acquire) && waited < 120 {
        delay_ms(250);
        waited += 1;
    }
    delay_ms(750);
    nsprint!("[NETSTACK] TDI self-test: opening \\Device\\Tcp\n");

    // Isolation probe: control-channel open (no EA) first.
    let ctrl = open_tcp(null_mut(), 0);
    if !ctrl.is_null() {
        nsprint!("[NETSTACK] TDI self-test: control-channel open OK\n");
        ZwClose(ctrl);
    }

    // Address object (local 0.0.0.0:0).
    let mut addr_ea: AddrEa = zeroed();
    addr_ea.name_len = 16;
    addr_ea.value_len = size_of::<TaIp>() as u16;
    addr_ea.name[..16].copy_from_slice(b"TransportAddress");
    addr_ea.value = TaIp::new([0, 0, 0, 0], 0);
    let addr_h = open_tcp(&mut addr_ea as *mut _ as PVOID, size_of::<AddrEa>() as u32);
    if addr_h.is_null() {
        return;
    }

    // Connection object.
    let mut conn_ea: ConnEa = zeroed();
    conn_ea.name_len = 17;
    conn_ea.value_len = 8;
    conn_ea.name[..17].copy_from_slice(b"ConnectionContext");
    conn_ea.value = 0xC0DE_F00D;
    let conn_h = open_tcp(&mut conn_ea as *mut _ as PVOID, size_of::<ConnEa>() as u32);
    if conn_h.is_null() {
        ZwClose(addr_h);
        return;
    }

    // Resolve the connection FILE_OBJECT + its device.
    let mut conn_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(conn_h, 0, null_mut(), 0, &mut conn_obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] TDI self-test: ref conn failed\n");
        ZwClose(conn_h);
        ZwClose(addr_h);
        return;
    }
    let conn_fo = conn_obj as *mut _FILE_OBJECT;
    let dev = (*conn_fo).DeviceObject;

    // Resolve the ADDRESS FILE_OBJECT too — TDI event handlers are registered on
    // the address object (AFD does TdiSetEventHandler on FCB->TdiAddressObject).
    let mut addr_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(addr_h, 0, null_mut(), 0, &mut addr_obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] TDI self-test: ref addr failed\n");
        ObfDereferenceObject(conn_obj);
        ZwClose(conn_h);
        ZwClose(addr_h);
        return;
    }
    let addr_fo = addr_obj as *mut _FILE_OBJECT;

    // ASSOCIATE the connection with the address object.
    let st = run_tdi(dev, conn_fo, tdi::TDI_ASSOCIATE_ADDRESS, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_ASSOCIATE;
        (*req).AddressHandle = addr_h;
    });
    nsprint!("[NETSTACK] TDI self-test: associate {:#010x}\n", st as u32);

    // CONNECT to the host echo (10.0.2.100:9999).
    let mut remote = TaIp::new([10, 0, 2, 100], 9999);
    let mut info: tdi::TDI_CONNECTION_INFORMATION = zeroed();
    info.RemoteAddress = &mut remote as *mut _ as *mut core::ffi::c_void;
    info.RemoteAddressLength = size_of::<TaIp>() as i32;
    let st = run_tdi(dev, conn_fo, tdi::TDI_CONNECT, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL;
        (*req).RequestConnectionInformation = &mut info;
        (*req).ReturnConnectionInformation = null_mut();
        (*req).RequestFlags = 0;
    });
    if st != 0 {
        nsprint!("[NETSTACK] TDI self-test: CONNECT FAILED {:#010x}\n", st as u32);
        cleanup2(addr_obj, conn_obj, conn_h, addr_h);
        return;
    }
    nsprint!("[NETSTACK] TDI self-test: CONNECT OK\n");

    // Register the TDI_EVENT_RECEIVE handler on the ADDRESS object — the AFD model
    // (event-indication upcall, NOT a receive IRP).
    let st = run_tdi(dev, addr_fo, tdi::TDI_SET_EVENT_HANDLER, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_SET_EVENT;
        (*req).EventType = tdi::TDI_EVENT_RECEIVE as u32;
        (*req).EventHandler = recv_indication as *mut c_void;
        (*req).EventContext = 0xABCD_0000 as *mut c_void;
    });
    nsprint!("[NETSTACK] TDI self-test: set-event(receive) {:#010x}\n", st as u32);

    // SEND a probe. TDI SEND/RECEIVE describe the data buffer via Irp->MdlAddress;
    // IoBuildDeviceIoControlRequest(METHOD_OUT_DIRECT) builds it + locks pages, and
    // the transport maps it with MmGetSystemAddressForMdlSafe. That mapping path
    // needs *pool* pages, not the kernel stack (a stack-backed MDL won't map to a
    // system VA), so use NonPagedPool buffers — exactly what real AFD passes.
    const PROBE: &[u8] = b"TDI-SELFTEST-PROBE\n";
    let txbuf = ExAllocatePoolWithTag(0 /* NonPagedPool */ as _, PROBE.len() as _, u32::from_le_bytes(*b"NetT")) as *mut u8;
    if txbuf.is_null() {
        nsprint!("[NETSTACK] TDI self-test: pool alloc failed\n");
        cleanup2(addr_obj, conn_obj, conn_h, addr_h);
        return;
    }
    core::ptr::copy_nonoverlapping(PROBE.as_ptr(), txbuf, PROBE.len());
    let st = run_tdi(
        dev,
        conn_fo,
        tdi::TDI_SEND,
        txbuf as PVOID,
        PROBE.len() as u32,
        |p| {
            let req = p as *mut tdi::TDI_REQUEST_KERNEL_SEND;
            (*req).SendLength = PROBE.len() as u32;
            (*req).SendFlags = 0;
        },
    );
    nsprint!("[NETSTACK] TDI self-test: send {:#010x}\n", st as u32);

    // RECEIVE via the TDI_EVENT_RECEIVE upcall (the AFD model): wait for netstack
    // to upcall recv_indication when the echo arrives.
    let mut waited = 0;
    while !RECV_FIRED.load(Ordering::Acquire) && waited < 40 {
        delay_ms(250);
        waited += 1;
    }
    if RECV_FIRED.load(Ordering::Acquire) {
        nsprint!(
            "[NETSTACK] TDI self-test: RECEIVE OK — winsock-style round-trip works ({} bytes via TDI_EVENT_RECEIVE upcall, ctx {:#x})\n",
            RECV_LEN.load(Ordering::Acquire),
            RECV_CONN_CTX.load(Ordering::Acquire)
        );
    } else {
        nsprint!("[NETSTACK] TDI self-test: receive upcall TIMED OUT\n");
    }

    ExFreePool(txbuf as PVOID);
    cleanup2(addr_obj, conn_obj, conn_h, addr_h);
}

unsafe fn cleanup(conn_obj: PVOID, conn_h: *mut core::ffi::c_void, addr_h: *mut core::ffi::c_void) {
    ObfDereferenceObject(conn_obj);
    ZwClose(conn_h);
    ZwClose(addr_h);
}

unsafe fn cleanup2(addr_obj: PVOID, conn_obj: PVOID, conn_h: *mut c_void, addr_h: *mut c_void) {
    ObfDereferenceObject(addr_obj);
    cleanup(conn_obj, conn_h, addr_h);
}
