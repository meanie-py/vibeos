//! ICMP echo (ping) self-test — exercises the M9 `IOCTL_ICMP_ECHO_REQUEST`
//! handler on `\Device\Ip` end to end. It opens the device, builds the exact
//! `ICMP_ECHO_REQUEST` input buffer iphlpapi's `IcmpSendEcho2` would
//! (dll/win32/iphlpapi/icmp.c), issues the METHOD_BUFFERED IOCTL to the gateway
//! (10.0.2.2, which slirp answers), and validates the `ICMP_ECHO_REPLY` the
//! worker writes back — proving the dispatch → worker → reply → completion path
//! the usermode `ping` rides on.

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
use crate::tdi::{self, IcmpEchoReply, IcmpEchoRequest};
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
static IP_NAME: [u16; 10] = wname(b"\\Device\\Ip");

/// Gateway address slirp answers ICMP echo for (same target as the engine's
/// boot self-test) and the payload we expect echoed back.
const PAYLOAD: &[u8] = b"vibeos-icmp-selftest";

pub fn spawn() {
    let mut h = null_mut();
    let start: ntkrnl_sys::ddk::PKSTART_ROUTINE =
        Some(unsafe { core::mem::transmute(worker as *const ()) });
    unsafe {
        PsCreateSystemThread(&mut h, 0x001F_03FF, null_mut(), null_mut(), null_mut(), start, null_mut());
    }
}

unsafe extern "system" fn worker(_ctx: PVOID) {
    // Need a configured interface (route + ARP) — wait for the DHCP lease.
    let mut waited = 0;
    while !DHCP_READY.load(Ordering::Acquire) && waited < 120 {
        delay_ms(250);
        waited += 1;
    }
    if !DHCP_READY.load(Ordering::Acquire) {
        nsprint!("[NETSTACK] ICMP self-test: skipped (no network device)\n");
        return;
    }
    // Ping the lease's gateway (10.0.2.2 under slirp; a hard-coded 10.0.2.2 timed out
    // on every real network).
    let Some(ping_dst) = crate::netcfg::gateway() else {
        nsprint!("[NETSTACK] ICMP self-test: skipped (the lease named no gateway)\n");
        return;
    };
    // Settle after the engine's own boot ping so the two don't interleave on the UART.
    delay_ms(1500);

    let ip_h = open_dev(&IP_NAME, null_mut(), 0);
    if ip_h.is_null() {
        nsprint!("[NETSTACK] ICMP self-test: open \\Device\\Ip failed\n");
        return;
    }
    let mut ip_obj: PVOID = null_mut();
    if ObReferenceObjectByHandle(ip_h, 0, null_mut(), 0, &mut ip_obj, null_mut()) != 0 {
        nsprint!("[NETSTACK] ICMP self-test: ref \\Device\\Ip failed\n");
        ZwClose(ip_h);
        return;
    }
    let ip_fo = ip_obj as *mut _FILE_OBJECT;
    let dev = (*ip_fo).DeviceObject;

    // --- Build the ICMP_ECHO_REQUEST input buffer (header + payload). ---
    let req_hdr = size_of::<IcmpEchoRequest>();
    let in_len = req_hdr + PAYLOAD.len();
    let tag = u32::from_le_bytes(*b"NetS");
    let in_buf = ExAllocatePoolWithTag(NON_PAGED_POOL as _, in_len as _, tag) as *mut u8;
    // Output: header + echoed payload + a little slack (mirrors IcmpSendEcho2's
    // sizeof(ICMP_ECHO_REPLY) + RequestSize sizing).
    let out_len = size_of::<IcmpEchoReply>() + PAYLOAD.len() + 8;
    let out_buf = ExAllocatePoolWithTag(NON_PAGED_POOL as _, out_len as _, tag) as *mut u8;
    if in_buf.is_null() || out_buf.is_null() {
        if !in_buf.is_null() { ExFreePool(in_buf as PVOID); }
        if !out_buf.is_null() { ExFreePool(out_buf as PVOID); }
        ObfDereferenceObject(ip_obj);
        ZwClose(ip_h);
        return;
    }
    core::ptr::write_bytes(in_buf, 0, in_len);
    core::ptr::write_bytes(out_buf, 0, out_len);
    let req = in_buf as *mut IcmpEchoRequest;
    (*req).address = u32::from_ne_bytes(ping_dst);
    (*req).timeout = 4000;
    (*req).data_offset = req_hdr as u16;
    (*req).data_size = PAYLOAD.len() as u16;
    (*req).options_offset = req_hdr as u16;
    core::ptr::copy_nonoverlapping(PAYLOAD.as_ptr(), in_buf.add(req_hdr), PAYLOAD.len());

    nsprint!(
        "[NETSTACK] ICMP self-test: ping {}.{}.{}.{}\n",
        ping_dst[0], ping_dst[1], ping_dst[2], ping_dst[3]
    );

    // --- Issue IOCTL_ICMP_ECHO_REQUEST (METHOD_BUFFERED, regular device-control). ---
    // Event+IOSB in pool via the shared TdiWaitBlock — the old stack locals +
    // return-on-timeout were the §9.36 dead-frame-completion corruption class.
    let wb = crate::tditest::alloc_wait_block();
    if wb.is_null() {
        ExFreePool(in_buf as PVOID);
        ExFreePool(out_buf as PVOID);
        ObfDereferenceObject(ip_obj);
        ZwClose(ip_h);
        return;
    }
    let irp = IoBuildDeviceIoControlRequest(
        tdi::IOCTL_ICMP_ECHO_REQUEST,
        dev,
        in_buf as PVOID,
        in_len as u32,
        out_buf as PVOID,
        out_len as u32,
        0, // not internal device control -> IRP_MJ_DEVICE_CONTROL
        core::ptr::addr_of_mut!((*wb).event) as *mut _,
        core::ptr::addr_of_mut!((*wb).iosb) as *mut _,
    );
    if irp.is_null() {
        nsprint!("[NETSTACK] ICMP self-test: IoBuildDeviceIoControlRequest failed\n");
        ExFreePool(wb as PVOID);
        ExFreePool(in_buf as PVOID);
        ExFreePool(out_buf as PVOID);
        ObfDereferenceObject(ip_obj);
        ZwClose(ip_h);
        return;
    }
    let st = IofCallDriver(dev, irp);
    let (st, leaked) = crate::tditest::tdi_wait_finish(st, irp, wb, -60_000_000i64 /* 6s, > the 4s ping timeout */);
    if leaked {
        // The pending IRP is buffered I/O: IopCompleteRequest will copy
        // SystemBuffer back into out_buf (Irp->UserBuffer) whenever it finally
        // completes — out_buf must outlive that, so leak it with the block.
        nsprint!("[NETSTACK] ICMP self-test: timed out; leaking out_buf {:p} with the wait block\n", out_buf);
        ExFreePool(in_buf as PVOID);
        ObfDereferenceObject(ip_obj);
        ZwClose(ip_h);
        return;
    }

    // --- Validate the reply. ---
    let reply = out_buf as *const IcmpEchoReply;
    let status = (*reply).status;
    let reserved = (*reply).reserved;
    let rtt = (*reply).round_trip_time;
    let dsize = (*reply).data_size;
    if status == tdi::IP_SUCCESS && reserved >= 1 {
        nsprint!(
            "[NETSTACK] ICMP self-test: reply OK rtt={}ms data={} bytes — ping round-trip OK\n",
            rtt, dsize
        );
    } else {
        nsprint!(
            "[NETSTACK] ICMP self-test: FAIL ioctl={:#010x} ipstatus={} replies={}\n",
            st as u32, status, reserved
        );
    }

    ExFreePool(in_buf as PVOID);
    ExFreePool(out_buf as PVOID);
    ObfDereferenceObject(ip_obj);
    ZwClose(ip_h);
}
