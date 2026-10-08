//! Management-plane SET self-test — exercises the M9 item-4
//! `IOCTL_TCP_SET_INFORMATION_EX` handler on `\Device\Tcp` end to end. It builds
//! the exact `TCP_REQUEST_SET_INFORMATION_EX` iphlpapi's `createIpForwardEntry`
//! issues (dll/win32/iphlpapi/route_reactos.c) — a default-route IPRouteEntry on
//! the IP network-layer entity — sends the METHOD_BUFFERED IOCTL, and asserts the
//! transport acknowledges it. Proves the dispatch → worker route-program path the
//! usermode `route` command / dhcpcsvc ride on.

use core::mem::size_of;
use core::ptr::null_mut;
use core::sync::atomic::Ordering;

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, IoBuildDeviceIoControlRequest,
    IofCallDriver,
    ObReferenceObjectByHandle, ObfDereferenceObject, PsCreateSystemThread, PVOID, ZwClose,
    _FILE_OBJECT,
};

use crate::engine::DHCP_READY;
use crate::serial::nsprint;
use crate::tdi;
use crate::tditest::open_dev;
use crate::time::delay_ms;

const STATUS_PENDING: i32 = 0x0000_0103;
const NON_PAGED_POOL: u32 = 0;

/// Compile-time ASCII -> UTF-16 for the device name.
const fn wname<const N: usize>(s: &[u8]) -> [u16; N] {
    let mut a = [0u16; N];
    let mut i = 0;
    while i < s.len() {
        a[i] = s[i] as u16;
        i += 1;
    }
    a
}
static TCP_NAME: [u16; 11] = wname(b"\\Device\\Tcp");

// TDIObjectID class/entity/type/id constants (tdiinfo.h) the SET request carries.
const INFO_CLASS_PROTOCOL: u32 = 0x200;
const INFO_TYPE_PROVIDER: u32 = 0x100;
const CL_NL_ENTITY: u32 = 0x301;
const IP_MIB_ARPTABLE_ENTRY_ID: u32 = 0x101;


#[repr(C)]
struct TdiEntityId {
    tei_entity: u32,
    tei_instance: u32,
}
#[repr(C)]
struct TdiObjectId {
    toi_entity: TdiEntityId,
    toi_class: u32,
    toi_type: u32,
    toi_id: u32,
}
/// IPRouteEntry (iphlpapi_private.h) — the route SET buffer payload. 13 ULONGs.
#[repr(C)]
struct IpRouteEntry {
    ire_dest: u32,
    ire_index: u32,
    ire_metric1: u32,
    ire_metric2: u32,
    ire_metric3: u32,
    ire_metric4: u32,
    ire_gw: u32,
    ire_type: u32,
    ire_proto: u32,
    ire_age: u32,
    ire_mask: u32,
    ire_metric5: u32,
    ire_info: u32,
}
/// TCP_REQUEST_SET_INFORMATION_EX header (tdiinfo.h) + the route entry inline.
#[repr(C)]
struct SetRouteRequest {
    id: TdiObjectId,
    buffer_size: u32,
    route: IpRouteEntry,
}

pub fn spawn() {
    let mut h = null_mut();
    let start: ntkrnl_sys::ddk::PKSTART_ROUTINE =
        Some(unsafe { core::mem::transmute(worker as *const ()) });
    unsafe {
        PsCreateSystemThread(
            &mut h, 0x001F_03FF, null_mut(), null_mut(), null_mut(), start, null_mut(),
        );
    }
}

unsafe extern "system" fn worker(_ctx: PVOID) {
    // The route program is only meaningful with a configured interface.
    let mut waited = 0;
    while !DHCP_READY.load(Ordering::Acquire) && waited < 120 {
        delay_ms(250);
        waited += 1;
    }
    if !DHCP_READY.load(Ordering::Acquire) {
        nsprint!("[NETSTACK] SET_INFO self-test: skipped (no network device)\n");
        return;
    }
    // The route programmed is the lease's own default route (10.0.2.2 under slirp).
    // A hard-coded 10.0.2.2 replaced the Pi 5's real default route with one through a
    // gateway nobody answers ARP for, and the ARP retries starved every other peer.
    let Some(gw) = crate::netcfg::gateway() else {
        nsprint!("[NETSTACK] SET_INFO self-test: skipped (the lease named no gateway)\n");
        return;
    };
    // Stagger behind the ICMP self-test so the markers don't interleave on the UART.
    delay_ms(2500);

    let tcp_h = open_dev(&TCP_NAME, null_mut(), 0);
    if tcp_h.is_null() {
        nsprint!("[NETSTACK] SET_INFO self-test: open \\Device\\Tcp failed\n");
        return;
    }
    let mut tcp_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(tcp_h, 0, null_mut(), 0, &mut tcp_obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] SET_INFO self-test: ref \\Device\\Tcp failed\n");
        ZwClose(tcp_h);
        return;
    }
    let tcp_fo = tcp_obj as *mut _FILE_OBJECT;
    let dev = (*tcp_fo).DeviceObject;

    // --- Build TCP_REQUEST_SET_INFORMATION_EX: default route add via the gateway. ---
    let in_len = size_of::<SetRouteRequest>();
    let tag = u32::from_le_bytes(*b"NetS");
    let in_buf = ExAllocatePoolWithTag(NON_PAGED_POOL as _, in_len as _, tag) as *mut u8;
    if in_buf.is_null() {
        ObfDereferenceObject(tcp_obj);
        ZwClose(tcp_h);
        return;
    }
    core::ptr::write_bytes(in_buf, 0, in_len);
    let req = in_buf as *mut SetRouteRequest;
    (*req).id = TdiObjectId {
        toi_entity: TdiEntityId { tei_entity: CL_NL_ENTITY, tei_instance: 0 },
        toi_class: INFO_CLASS_PROTOCOL,
        toi_type: INFO_TYPE_PROVIDER,
        toi_id: IP_MIB_ARPTABLE_ENTRY_ID,
    };
    (*req).buffer_size = size_of::<IpRouteEntry>() as u32;
    (*req).route.ire_dest = 0; // 0.0.0.0 default
    (*req).route.ire_mask = 0;
    (*req).route.ire_gw = u32::from_ne_bytes(gw);
    (*req).route.ire_index = 1;
    (*req).route.ire_type = tdi::IP_FORWARD_ADD;
    (*req).route.ire_proto = 3; // MIB_IPPROTO_NETMGMT

    nsprint!("[NETSTACK] SET_INFO self-test: program default route via {}.{}.{}.{}\n", gw[0], gw[1], gw[2], gw[3]);

    // --- Issue IOCTL_TCP_SET_INFORMATION_EX (METHOD_BUFFERED, output empty). ---
    // Event+IOSB in pool via the shared TdiWaitBlock — the old stack locals +
    // return-on-timeout were the §9.36 dead-frame-completion corruption class.
    let wb = crate::tditest::alloc_wait_block();
    if wb.is_null() {
        ExFreePool(in_buf as PVOID);
        ObfDereferenceObject(tcp_obj);
        ZwClose(tcp_h);
        return;
    }
    let irp = IoBuildDeviceIoControlRequest(
        tdi::IOCTL_TCP_SET_INFORMATION_EX,
        dev,
        in_buf as PVOID,
        in_len as u32,
        null_mut(),
        0,
        0, // InternalDeviceIoControl = FALSE (IRP_MJ_DEVICE_CONTROL)
        core::ptr::addr_of_mut!((*wb).event) as *mut _,
        core::ptr::addr_of_mut!((*wb).iosb) as *mut _,
    );
    if irp.is_null() {
        nsprint!("[NETSTACK] SET_INFO self-test: IoBuildDeviceIoControlRequest failed\n");
        ExFreePool(wb as PVOID);
        ExFreePool(in_buf as PVOID);
        ObfDereferenceObject(tcp_obj);
        ZwClose(tcp_h);
        return;
    }
    let st = IofCallDriver(dev, irp);
    let (st, _leaked) = crate::tditest::tdi_wait_finish(st, irp, wb, -50_000_000i64 /* 5s */);

    if st == 0 {
        nsprint!(
            "[NETSTACK] SET_INFO self-test: route program OK — IOCTL_TCP_SET_INFORMATION_EX OK\n"
        );
    } else {
        nsprint!("[NETSTACK] SET_INFO self-test: FAIL ioctl={:#010x}\n", st as u32);
    }

    ExFreePool(in_buf as PVOID);
    ObfDereferenceObject(tcp_obj);
    ZwClose(tcp_h);
}
