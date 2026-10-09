//! Kernel-mode TDI *datagram* (UDP) self-test — exercises netstack's own
//! \Device\Udp via real IRPs, the exact path AFD uses for connectionless winsock
//! (sendto/recvfrom, and the usermode DHCP/DNS services): open an ADDRESS object,
//! register a TDI_EVENT_RECEIVE_DATAGRAM handler, TDI_SEND_DATAGRAM a real DNS
//! query to slirp's DNS forwarder (10.0.2.3:53), then receive the reply via the
//! datagram event-indication upcall. Proves the UDP send + datagram completion
//! bridge end to end with no usermode harness.

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
use crate::tditest::{open_dev, run_tdi};
use crate::time::delay_ms;

const fn wdev<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}
static UDP_NAME: [u16; 11] = wdev(b"\\Device\\Udp");

/// The query goes to the lease's DNS server, a real UDP responder on :53: slirp's
/// forwarder (10.0.2.3) under QEMU, the LAN's resolver on a board. A hard-coded
/// 10.0.2.3 timed out on every Pi 5 boot, the router having no route to slirp.
const DNS_PORT: u16 = 53;

/// A static DNS query for "example.com" A: 12-byte header (ID, RD flag, QDCOUNT=1)
/// + QNAME (7"example" 3"com" 0) + QTYPE=A(1) + QCLASS=IN(1).
const DNS_QUERY: &[u8] = &[
    0x12, 0x34, // ID
    0x01, 0x00, // flags: standard query, recursion desired
    0x00, 0x01, // QDCOUNT = 1
    0x00, 0x00, // ANCOUNT
    0x00, 0x00, // NSCOUNT
    0x00, 0x00, // ARCOUNT
    7, b'e', b'x', b'a', b'm', b'p', b'l', b'e', // "example"
    3, b'c', b'o', b'm', // "com"
    0,    // root label
    0x00, 0x01, // QTYPE = A
    0x00, 0x01, // QCLASS = IN
];

/// FILE_FULL_EA_INFORMATION + "TransportAddress" + a local TA_IP_ADDRESS. Uses
/// the aligned tdi::TaIpFull (the kernel's natural layout — see its doc comment).
#[repr(C, packed)]
struct AddrEa {
    next: u32,
    flags: u8,
    name_len: u8,
    value_len: u16,
    name: [u8; 17], // "TransportAddress\0"
    value: tdi::TaIpFull,
}

// The datagram upcall fired by netstack when the DNS reply arrives — the exact
// path real AFD uses for recvfrom. We record the byte count + source IP to prove
// the connectionless event-indication path end to end.
static DG_FIRED: AtomicBool = AtomicBool::new(false);
static DG_LEN: AtomicUsize = AtomicUsize::new(0);
static DG_SRC: AtomicUsize = AtomicUsize::new(0); // packed a.b.c.d for the log

unsafe extern "system" fn recv_dg_indication(
    _tdi_ctx: *mut c_void,
    source_address_length: i32,
    source_address: *mut c_void,
    _options_length: i32,
    _options: *mut c_void,
    _flags: u32,
    bytes_indicated: u32,
    _bytes_available: u32,
    bytes_taken: *mut u32,
    _tsdu: *mut c_void,
    irp_out: *mut ntkrnl_sys::ddk::PIRP,
) -> i32 {
    *bytes_taken = bytes_indicated; // accept the whole datagram inline
    if !irp_out.is_null() {
        *irp_out = null_mut();
    }
    // Record the parsed source endpoint (proves we built a valid TA_IP_ADDRESS).
    if source_address_length as usize >= size_of::<tdi::TaIpFull>() && !source_address.is_null() {
        if let Some((ip, _port)) = tdi::parse_ta(source_address as *const u8) {
            DG_SRC.store(
                u32::from_be_bytes(ip) as usize,
                Ordering::Release,
            );
        }
    }
    DG_LEN.store(bytes_indicated as usize, Ordering::Release);
    DG_FIRED.store(true, Ordering::Release);
    0 // STATUS_SUCCESS
}

// The broadcast listener: a SECOND address object bound to BC_PORT. A datagram this
// host sends to a broadcast address must reach it too. NDIS does not loop broadcasts back
// to the sender, so upstream tcpip (b8ee7950af4) delivers them locally itself, and so
// must we.
const BC_PORT: u16 = 47809;
const BC_PAYLOAD: &[u8] = b"VIBEOS-BCAST";
static BC_COUNT: AtomicUsize = AtomicUsize::new(0);
static BC_LEN: AtomicUsize = AtomicUsize::new(0);

unsafe extern "system" fn bc_dg_indication(
    _tdi_ctx: *mut c_void,
    _source_address_length: i32,
    _source_address: *mut c_void,
    _options_length: i32,
    _options: *mut c_void,
    _flags: u32,
    bytes_indicated: u32,
    _bytes_available: u32,
    bytes_taken: *mut u32,
    _tsdu: *mut c_void,
    irp_out: *mut ntkrnl_sys::ddk::PIRP,
) -> i32 {
    *bytes_taken = bytes_indicated;
    if !irp_out.is_null() {
        *irp_out = null_mut();
    }
    BC_LEN.store(bytes_indicated as usize, Ordering::Release);
    BC_COUNT.fetch_add(1, Ordering::AcqRel);
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
    // Wait for DHCP (need a source address) + a short settle, and let the TCP
    // self-test go first so the serial markers don't interleave at bring-up.
    let mut waited = 0;
    while !DHCP_READY.load(Ordering::Acquire) && waited < 120 {
        delay_ms(250);
        waited += 1;
    }
    // No NIC → DHCP never configured: skip cleanly rather than send-datagram into
    // the void and time out (the UART flood drops adjacent boot markers; see the
    // TCP self-test for the same gate).
    if !DHCP_READY.load(Ordering::Acquire) {
        nsprint!("[NETSTACK] TDI dgram self-test: skipped (no network device)\n");
        return;
    }
    delay_ms(3000);
    nsprint!("[NETSTACK] TDI dgram self-test: opening \\Device\\Udp\n");

    // Address object bound to 0.0.0.0:0 (ephemeral local port).
    let mut addr_ea: AddrEa = zeroed();
    addr_ea.name_len = 16;
    addr_ea.value_len = size_of::<tdi::TaIpFull>() as u16;
    addr_ea.name[..16].copy_from_slice(b"TransportAddress");
    addr_ea.value = tdi::TaIpFull::new([0, 0, 0, 0], 0);
    let addr_h = open_dev(&UDP_NAME, &mut addr_ea as *mut _ as PVOID, size_of::<AddrEa>() as u32);
    if addr_h.is_null() {
        nsprint!("[NETSTACK] TDI dgram self-test: \\Device\\Udp address open failed\n");
        return;
    }

    // Resolve the address FILE_OBJECT + its device.
    let mut addr_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(addr_h, 0, null_mut(), 0, &mut addr_obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] TDI dgram self-test: ref addr failed\n");
        ZwClose(addr_h);
        return;
    }
    let addr_fo = addr_obj as *mut _FILE_OBJECT;
    let dev = (*addr_fo).DeviceObject;

    // Register the datagram receive handler on the address object (AFD model).
    let st = run_tdi(dev, addr_fo, tdi::TDI_SET_EVENT_HANDLER, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_SET_EVENT;
        (*req).EventType = tdi::TDI_EVENT_RECEIVE_DATAGRAM as u32;
        (*req).EventHandler = recv_dg_indication as *mut c_void;
        (*req).EventContext = 0xDA7A_0000u32 as *mut c_void;
    });
    nsprint!("[NETSTACK] TDI dgram self-test: set-event(receive-datagram) {:#010x}\n", st as u32);

    let Some(dns_ip) = crate::netcfg::dns0() else {
        nsprint!("[NETSTACK] TDI dgram self-test: skipped (the lease named no DNS server)\n");
        cleanup(addr_obj, addr_h);
        return;
    };
    // SEND_DATAGRAM the DNS query to the lease's resolver. The data buffer must be pool
    // (METHOD_OUT_DIRECT builds the MDL the transport maps), exactly like AFD.
    let txbuf = ExAllocatePoolWithTag(0 /* NonPagedPool */ as _, DNS_QUERY.len() as _, u32::from_le_bytes(*b"NetD")) as *mut u8;
    if txbuf.is_null() {
        nsprint!("[NETSTACK] TDI dgram self-test: pool alloc failed\n");
        cleanup(addr_obj, addr_h);
        return;
    }
    core::ptr::copy_nonoverlapping(DNS_QUERY.as_ptr(), txbuf, DNS_QUERY.len());

    let mut remote = tdi::TaIpFull::new(dns_ip, DNS_PORT);
    let mut info: tdi::TDI_CONNECTION_INFORMATION = zeroed();
    info.RemoteAddress = &mut remote as *mut _ as *mut c_void;
    info.RemoteAddressLength = size_of::<tdi::TaIpFull>() as i32;
    let st = run_tdi(dev, addr_fo, tdi::TDI_SEND_DATAGRAM, txbuf as PVOID, DNS_QUERY.len() as u32, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_SENDDG;
        (*req).SendLength = DNS_QUERY.len() as u32;
        (*req).SendDatagramInformation = &mut info;
    });
    nsprint!("[NETSTACK] TDI dgram self-test: send-datagram {:#010x}\n", st as u32);

    // Wait for the DNS reply via the datagram upcall.
    let mut waited = 0;
    while !DG_FIRED.load(Ordering::Acquire) && waited < 40 {
        delay_ms(250);
        waited += 1;
    }
    if DG_FIRED.load(Ordering::Acquire) {
        let src = DG_SRC.load(Ordering::Acquire) as u32;
        let o = src.to_be_bytes();
        nsprint!(
            "[NETSTACK] TDI dgram self-test: RECEIVE_DATAGRAM OK — UDP round-trip works ({} bytes from {}.{}.{}.{} via TDI_EVENT_RECEIVE_DATAGRAM upcall)\n",
            DG_LEN.load(Ordering::Acquire), o[0], o[1], o[2], o[3]
        );
    } else {
        nsprint!("[NETSTACK] TDI dgram self-test: datagram upcall TIMED OUT\n");
    }

    ExFreePool(txbuf as PVOID);
    broadcast_stage(dev, addr_fo);
    cleanup(addr_obj, addr_h);
}

/// Send from the first address object to the limited (255.255.255.255) and the directed
/// (the lease's subnet broadcast: 10.0.2.255 under slirp) broadcast on BC_PORT, and
/// check that the local listener received each one.
unsafe fn broadcast_stage(dev: ntkrnl_sys::ddk::PDEVICE_OBJECT, send_fo: *mut _FILE_OBJECT) {
    let mut ea: AddrEa = zeroed();
    ea.name_len = 16;
    ea.value_len = size_of::<tdi::TaIpFull>() as u16;
    ea.name[..16].copy_from_slice(b"TransportAddress");
    ea.value = tdi::TaIpFull::new([0, 0, 0, 0], BC_PORT);
    let h = open_dev(&UDP_NAME, &mut ea as *mut _ as PVOID, size_of::<AddrEa>() as u32);
    if h.is_null() {
        nsprint!("[NETSTACK] TDI dgram self-test: BROADCAST listener open FAILED\n");
        return;
    }
    let mut obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(h, 0, null_mut(), 0, &mut obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] TDI dgram self-test: BROADCAST listener ref FAILED\n");
        ZwClose(h);
        return;
    }
    let fo = obj as *mut _FILE_OBJECT;
    run_tdi((*fo).DeviceObject, fo, tdi::TDI_SET_EVENT_HANDLER, null_mut(), 0, |p| {
        let req = p as *mut tdi::TDI_REQUEST_KERNEL_SET_EVENT;
        (*req).EventType = tdi::TDI_EVENT_RECEIVE_DATAGRAM as u32;
        (*req).EventHandler = bc_dg_indication as *mut c_void;
        (*req).EventContext = 0xBCA5_0000u32 as *mut c_void;
    });

    let buf = ExAllocatePoolWithTag(0 as _, BC_PAYLOAD.len() as _, u32::from_le_bytes(*b"NetB")) as *mut u8;
    if buf.is_null() {
        nsprint!("[NETSTACK] TDI dgram self-test: BROADCAST pool alloc FAILED\n");
        cleanup(obj, h);
        return;
    }
    core::ptr::copy_nonoverlapping(BC_PAYLOAD.as_ptr(), buf, BC_PAYLOAD.len());

    let mut got = [false; 2];
    let directed = crate::netcfg::broadcast_naddr().to_ne_bytes();
    for (i, dst) in [[255u8, 255, 255, 255], directed].iter().enumerate() {
        let before = BC_COUNT.load(Ordering::Acquire);
        let mut remote = tdi::TaIpFull::new(*dst, BC_PORT);
        let mut info: tdi::TDI_CONNECTION_INFORMATION = zeroed();
        info.RemoteAddress = &mut remote as *mut _ as *mut c_void;
        info.RemoteAddressLength = size_of::<tdi::TaIpFull>() as i32;
        let st = run_tdi(dev, send_fo, tdi::TDI_SEND_DATAGRAM, buf as PVOID, BC_PAYLOAD.len() as u32, |p| {
            let req = p as *mut tdi::TDI_REQUEST_KERNEL_SENDDG;
            (*req).SendLength = BC_PAYLOAD.len() as u32;
            (*req).SendDatagramInformation = &mut info;
        });
        let mut waited = 0;
        while BC_COUNT.load(Ordering::Acquire) == before && waited < 12 {
            delay_ms(250);
            waited += 1;
        }
        got[i] = BC_COUNT.load(Ordering::Acquire) > before && BC_LEN.load(Ordering::Acquire) == BC_PAYLOAD.len();
        nsprint!(
            "[NETSTACK] TDI dgram self-test: broadcast to {}.{}.{}.{}:{} send {:#010x} local {}\n",
            dst[0], dst[1], dst[2], dst[3], BC_PORT, st as u32, if got[i] { "received" } else { "MISSED" }
        );
    }
    if got[0] && got[1] {
        nsprint!("[NETSTACK] TDI dgram self-test: BROADCAST local delivery OK (limited + directed)\n");
    } else {
        nsprint!("[NETSTACK] TDI dgram self-test: BROADCAST local delivery FAILED\n");
    }
    ExFreePool(buf as PVOID);
    cleanup(obj, h);
}

unsafe fn cleanup(addr_obj: PVOID, addr_h: *mut c_void) {
    ObfDereferenceObject(addr_obj);
    ZwClose(addr_h);
}
