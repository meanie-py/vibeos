//! NDIS protocol shim (M11 scaffold) — drive a real miniport (e1000 / rtl8139 /
//! netkvm) by registering netstack as an NDIS *protocol* above ndis.sys, instead
//! of driving virtio directly. The upward smoltcp side is unchanged; only the
//! `phy::Device` swaps (`NdisPhy`, see `ndisphy.rs`). Modelled on the surviving
//! reference binding `drivers/network/tcpip/datalink/lan.c`.
//!
//! Lifecycle: `register_and_open` builds NDIS40 protocol characteristics and
//! `NdisRegisterProtocol`s them; NDIS then calls our `bind_adapter` for each
//! 802.3 miniport, which `NdisOpenAdapter`s it; `open_adapter_complete` stores
//! the binding handle + signals; we then OID-query the MAC and return.
//!
//! RX (the hard part): `protocol_receive` / `receive_packet` run at DISPATCH and
//! ONLY copy frame bytes into `ndisphy::RX_RING` (no smoltcp, no pageable alloc);
//! the PASSIVE worker drains the ring. TX marshals a flat frame into an
//! `NDIS_PACKET` + `NDIS_BUFFER` chain and hands it to `NdisSend`; the completion
//! frees them.
//!
//! The engine falls back to this when it finds no NIC of its own (engine.rs), and
//! waits for a miniport to bind for as long as it takes: on a board a downloaded
//! driver's device is installed minutes into boot.

// The FFI surface is intentionally complete (teardown imports + full request /
// packet layouts) ahead of the teardown path being wired — keep the unexercised
// declarations as documentation of the binding contract.
#![allow(dead_code)]

use core::ffi::c_void;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, KeInitializeEvent, KeWaitForSingleObject, KEVENT,
    LARGE_INTEGER, PVOID, UNICODE_STRING, _MDL,
};
use ntkrnl_sys::inline::mm::MmGetSystemAddressForMdlSafe;

use crate::ndisphy::{RX_RING, FRAME_MAX};
use crate::serial::nsprint;

type NdisStatus = i32;
type NdisHandle = *mut c_void;

const NDIS_STATUS_SUCCESS: NdisStatus = 0;
const NDIS_STATUS_PENDING: NdisStatus = 0x0000_0103;
const NDIS_MEDIUM_802_3: u32 = 0; // NdisMedium802_3 (first enum value)
const OID_802_3_CURRENT_ADDRESS: u32 = 0x0101_0102;
const OID_GEN_CURRENT_PACKET_FILTER: u32 = 0x0001_010E;
const OID_GEN_CURRENT_LOOKAHEAD: u32 = 0x0001_010F;
const OID_GEN_MAXIMUM_LOOKAHEAD: u32 = 0x0001_0105;
const NDIS_PACKET_TYPE_DIRECTED: u32 = 0x0001;
const NDIS_PACKET_TYPE_MULTICAST: u32 = 0x0002;
const NDIS_PACKET_TYPE_BROADCAST: u32 = 0x0008;
const NDIS_REQUEST_QUERY_INFORMATION: u32 = 0; // NdisRequestQueryInformation
const NDIS_REQUEST_SET_INFORMATION: u32 = 1; // NdisRequestSetInformation
const NON_PAGED_POOL: u32 = 0;
const TAG: u32 = u32::from_le_bytes(*b"NdsP");

// --- NDIS protocol handler fn-pointer types (NTAPI = extern "system"). --------

type OpenAdapterComplete = unsafe extern "system" fn(NdisHandle, NdisStatus, NdisStatus);
type CloseAdapterComplete = unsafe extern "system" fn(NdisHandle, NdisStatus);
type SendComplete = unsafe extern "system" fn(NdisHandle, *mut NdisPacket, NdisStatus);
type TransferDataComplete = unsafe extern "system" fn(NdisHandle, *mut NdisPacket, NdisStatus, u32);
type ResetComplete = unsafe extern "system" fn(NdisHandle, NdisStatus);
type RequestComplete = unsafe extern "system" fn(NdisHandle, *mut NdisRequest, NdisStatus);
type ReceiveHandler = unsafe extern "system" fn(
    NdisHandle, NdisHandle, *mut c_void, u32, *mut c_void, u32, u32,
) -> NdisStatus;
type ReceiveComplete = unsafe extern "system" fn(NdisHandle);
type StatusHandler = unsafe extern "system" fn(NdisHandle, NdisStatus, *mut c_void, u32);
type StatusComplete = unsafe extern "system" fn(NdisHandle);
type ReceivePacketHandler = unsafe extern "system" fn(NdisHandle, *mut NdisPacket) -> i32;
type BindHandler =
    unsafe extern "system" fn(*mut NdisStatus, NdisHandle, *mut UNICODE_STRING, *mut c_void, *mut c_void);
type UnbindHandler = unsafe extern "system" fn(*mut NdisStatus, NdisHandle, NdisHandle);
type PnpEventHandler = unsafe extern "system" fn(NdisHandle, *mut c_void) -> NdisStatus;
type UnloadHandler = unsafe extern "system" fn();

/// NDIS40_PROTOCOL_CHARACTERISTICS (ndis.h) — the registration descriptor.
#[repr(C)]
struct NdisProtocolCharacteristics {
    major_version: u8,
    minor_version: u8,
    filler: u16,
    flags: u32,
    open_adapter_complete: Option<OpenAdapterComplete>,
    close_adapter_complete: Option<CloseAdapterComplete>,
    send_complete: Option<SendComplete>,
    transfer_data_complete: Option<TransferDataComplete>,
    reset_complete: Option<ResetComplete>,
    request_complete: Option<RequestComplete>,
    receive: Option<ReceiveHandler>,
    receive_complete: Option<ReceiveComplete>,
    status: Option<StatusHandler>,
    status_complete: Option<StatusComplete>,
    name: UNICODE_STRING,
    receive_packet: Option<ReceivePacketHandler>,
    bind_adapter: Option<BindHandler>,
    unbind_adapter: Option<UnbindHandler>,
    pnp_event: Option<PnpEventHandler>,
    unload: Option<UnloadHandler>,
}

/// Leading part of NDIS_PACKET (the rest is reserved scratch we don't touch).
/// `Head`/`Tail` are the NDIS_BUFFER (== MDL) chain.
#[repr(C)]
struct NdisPacketPrivate {
    physical_count: u32,
    total_length: u32,
    head: *mut _MDL,
    tail: *mut _MDL,
    pool: NdisHandle,
    count: u32,
    flags: u32,
    valid_counts: u8,
    ndis_packet_flags: u8,
    ndis_packet_oob_offset: u16,
}
/// NDIS_PACKET up to our ProtocolReserved word. The scaffold kept its TX buffer
/// in the first word after `Private`, which is the MiniportReserved union the
/// miniport writes into, so `send_complete` would have freed the miniport's data.
#[repr(C)]
struct NdisPacket {
    private: NdisPacketPrivate,
    miniport_area: [usize; 4], // MiniportReserved + WrapperReserved (union)
    reserved: [usize; 2],
    protocol_reserved: [usize; 1], // ours: NdisAllocatePacketPool(.., size_of::<usize>())
}

/// NDIS_REQUEST (legacy). `DATA` is a union holding pointers, so on 64-bit it starts
/// 8-aligned at 40; the scaffold's flat fields put `Oid` at 36, where ndis.sys reads
/// the padding. QUERY_INFORMATION and SET_INFORMATION share this layout.
#[repr(C)]
struct NdisRequestData {
    oid: u32,
    information_buffer: *mut c_void,
    information_buffer_length: u32,
    bytes_written: u32,
    bytes_needed: u32,
}
#[repr(C)]
struct NdisRequest {
    mac_reserved: [usize; 4],
    request_type: u32,
    data: NdisRequestData,
    ndis_reserved: [usize; 9],
    protocol_reserved: [usize; 2],
    miniport_reserved: [usize; 2],
}

extern "system" {
    fn NdisRegisterProtocol(
        Status: *mut NdisStatus,
        NdisProtocolHandle: *mut NdisHandle,
        ProtocolCharacteristics: *const NdisProtocolCharacteristics,
        CharacteristicsLength: u32,
    );
    fn NdisDeregisterProtocol(Status: *mut NdisStatus, NdisProtocolHandle: NdisHandle);
    fn NdisTransferData(
        Status: *mut NdisStatus,
        NdisBindingHandle: NdisHandle,
        MacReceiveContext: NdisHandle,
        ByteOffset: u32,
        BytesToTransfer: u32,
        Packet: *mut NdisPacket,
        BytesTransferred: *mut u32,
    );
    fn NdisCompleteBindAdapter(BindAdapterContext: NdisHandle, Status: NdisStatus, OpenStatus: NdisStatus);
    fn NdisOpenAdapter(
        Status: *mut NdisStatus,
        OpenErrorStatus: *mut NdisStatus,
        NdisBindingHandle: *mut NdisHandle,
        SelectedMediumIndex: *mut u32,
        MediumArray: *const u32,
        MediumArraySize: u32,
        NdisProtocolHandle: NdisHandle,
        ProtocolBindingContext: NdisHandle,
        AdapterName: *mut UNICODE_STRING,
        OpenOptions: u32,
        AddressingInformation: *mut c_void,
    );
    fn NdisAllocatePacketPool(
        Status: *mut NdisStatus,
        PoolHandle: *mut NdisHandle,
        NumberOfDescriptors: u32,
        ProtocolReservedLength: u32,
    );
    fn NdisAllocateBufferPool(Status: *mut NdisStatus, PoolHandle: *mut NdisHandle, NumberOfDescriptors: u32);
    fn NdisAllocatePacket(Status: *mut NdisStatus, Packet: *mut *mut NdisPacket, PoolHandle: NdisHandle);
    fn NdisAllocateBuffer(
        Status: *mut NdisStatus,
        Buffer: *mut *mut _MDL,
        PoolHandle: NdisHandle,
        VirtualAddress: *mut c_void,
        Length: u32,
    );
    fn NdisFreePacket(Packet: *mut NdisPacket);
    fn NdisFreeBuffer(Buffer: *mut _MDL);
    fn NdisSend(Status: *mut NdisStatus, NdisBindingHandle: NdisHandle, Packet: *mut NdisPacket);
    fn NdisRequest(Status: *mut NdisStatus, NdisBindingHandle: NdisHandle, NdisRequest: *mut NdisRequest);
}

// --- Driver-global protocol state (one bound adapter) ------------------------ #

/// Pointer to the leaked `NdisCtx` (callbacks reach it; 0 until registered).
static CTX: AtomicUsize = AtomicUsize::new(0);

struct NdisCtx {
    event: KEVENT,
    /// Set by the receive handlers; the worker waits on it between polls.
    rx_event: KEVENT,
    /// bind_adapter's context, kept while its open is pending.
    bind_context: AtomicUsize,
    protocol_handle: NdisHandle,
    binding_handle: AtomicUsize,
    packet_pool: AtomicUsize,
    buffer_pool: AtomicUsize,
    open_status: AtomicUsize, // NdisStatus of the last open/request, as usize
    mac: AtomicU64,
    mac_ready: AtomicUsize,
}

fn ctx() -> Option<&'static NdisCtx> {
    let p = CTX.load(Ordering::Acquire);
    if p == 0 {
        None
    } else {
        Some(unsafe { &*(p as *const NdisCtx) })
    }
}

static PROTO_NAME: [u16; 8] = [b'V' as u16, b'i' as u16, b'b' as u16, b'e' as u16, b'o' as u16, b's' as u16, b'N' as u16, b'S' as u16];

/// Wire the protocol, wait for the first miniport to bind+open, OID-query the
/// MAC, and return it. None on any failure (no adapter, registration error).
pub fn register_and_open() -> Option<[u8; 6]> {
    use alloc::boxed::Box;
    let ctx_box = Box::leak(Box::new(NdisCtx {
        event: unsafe { core::mem::zeroed() },
        rx_event: unsafe { core::mem::zeroed() },
        bind_context: AtomicUsize::new(0),
        protocol_handle: core::ptr::null_mut(),
        binding_handle: AtomicUsize::new(0),
        packet_pool: AtomicUsize::new(0),
        buffer_pool: AtomicUsize::new(0),
        open_status: AtomicUsize::new(0),
        mac: AtomicU64::new(0),
        mac_ready: AtomicUsize::new(0),
    }));
    unsafe { KeInitializeEvent(&mut ctx_box.event, 0 /* NotificationEvent */, 0) };
    unsafe { KeInitializeEvent(&mut ctx_box.rx_event, 1 /* SynchronizationEvent */, 0) };
    CTX.store(ctx_box as *mut NdisCtx as usize, Ordering::Release);

    // 4.0: this struct is the NDIS40 layout, and ndis.sys sizes a 5.0 registration
    // against NDIS50_PROTOCOL_CHARACTERISTICS (four reserved and four CoXxx handlers
    // more), so a 5 here was refused with NDIS_STATUS_BAD_CHARACTERISTICS.
    let chars = NdisProtocolCharacteristics {
        major_version: 4,
        minor_version: 0,
        filler: 0,
        flags: 0,
        open_adapter_complete: Some(open_adapter_complete),
        close_adapter_complete: Some(close_adapter_complete),
        send_complete: Some(send_complete),
        transfer_data_complete: Some(transfer_data_complete),
        reset_complete: Some(reset_complete),
        request_complete: Some(request_complete),
        receive: Some(protocol_receive),
        receive_complete: Some(receive_complete),
        status: Some(status_handler),
        status_complete: Some(status_complete),
        name: UNICODE_STRING {
            Length: (PROTO_NAME.len() * 2) as u16,
            MaximumLength: (PROTO_NAME.len() * 2) as u16,
            Buffer: PROTO_NAME.as_ptr() as *mut u16,
        },
        receive_packet: Some(receive_packet),
        bind_adapter: Some(bind_adapter),
        unbind_adapter: Some(unbind_adapter),
        pnp_event: None,
        unload: None,
    };

    let mut status: NdisStatus = 0;
    let mut handle: NdisHandle = core::ptr::null_mut();
    unsafe {
        NdisRegisterProtocol(
            &mut status,
            &mut handle,
            &chars,
            core::mem::size_of::<NdisProtocolCharacteristics>() as u32,
        );
    }
    if status != NDIS_STATUS_SUCCESS {
        nsprint!("[NETSTACK] NDIS: RegisterProtocol failed {:#010x}\n", status as u32);
        return None;
    }
    ctx_box.protocol_handle = handle;
    nsprint!("[NETSTACK] NDIS: protocol registered, awaiting bind\n");

    // Wait for bind_adapter -> NdisOpenAdapter -> open_adapter_complete. No timeout:
    // a miniport can arrive any time (PnP installs a downloaded driver's device
    // minutes into boot), and the engine has nothing else to drive meanwhile.
    loop {
        wait_event(ctx_box, i64::MAX / 10_000);
        if ctx_box.binding_handle.load(Ordering::Acquire) != 0
            && ctx_box.open_status.load(Ordering::Acquire) as NdisStatus == NDIS_STATUS_SUCCESS
        {
            break;
        }
        // A failed open: forget it and wait for the next adapter.
        ctx_box.binding_handle.store(0, Ordering::Release);
        unsafe { KeInitializeEvent(&ctx_box.event as *const _ as *mut KEVENT, 0, 0) };
    }

    // Allocate the TX packet/buffer pools now the adapter is open.
    let mut s1: NdisStatus = 0;
    let mut s2: NdisStatus = 0;
    let mut pkt_pool: NdisHandle = core::ptr::null_mut();
    let mut buf_pool: NdisHandle = core::ptr::null_mut();
    unsafe {
        NdisAllocatePacketPool(&mut s1, &mut pkt_pool, 32, core::mem::size_of::<usize>() as u32);
        NdisAllocateBufferPool(&mut s2, &mut buf_pool, 32);
    }
    if s1 != NDIS_STATUS_SUCCESS || s2 != NDIS_STATUS_SUCCESS {
        nsprint!("[NETSTACK] NDIS: packet/buffer pool alloc failed\n");
        return None;
    }
    // A buffer pool handle may be NULL on success: NT's buffers are plain MDLs and
    // ndis.sys returns no pool at all, so only the status says whether it worked.
    // (Checking the handle dropped every frame netstack sent.)
    ctx_box.packet_pool.store(pkt_pool as usize, Ordering::Release);
    ctx_box.buffer_pool.store(buf_pool as usize, Ordering::Release);

    let mac = query_mac(ctx_box)?;
    // Without a packet filter a miniport indicates nothing; and a lookahead of the
    // whole frame lets `protocol_receive` take any frame without NdisTransferData.
    let filter = NDIS_PACKET_TYPE_DIRECTED | NDIS_PACKET_TYPE_MULTICAST | NDIS_PACKET_TYPE_BROADCAST;
    // The miniport refuses more than its maximum (the e1000's is 1508), so ask.
    // Frames past whatever is set come in through NdisTransferData.
    let max = query_u32(ctx_box, OID_GEN_MAXIMUM_LOOKAHEAD).unwrap_or(128);
    if !set_oid(ctx_box, OID_GEN_CURRENT_LOOKAHEAD, max.min((FRAME_MAX - 14) as u32)) {
        nsprint!("[NETSTACK] NDIS: lookahead not set; longer frames go through TransferData\n");
    }
    if !set_oid(ctx_box, OID_GEN_CURRENT_PACKET_FILTER, filter) {
        nsprint!("[NETSTACK] NDIS: packet filter refused; no frames will arrive\n");
        return None;
    }
    Some(mac)
}

/// The event the receive handlers set, for the worker's wait between polls.
pub fn rx_event() -> *mut KEVENT {
    match ctx() {
        Some(c) => &c.rx_event as *const _ as *mut KEVENT,
        None => core::ptr::null_mut(),
    }
}

/// Query a 32-bit OID and wait for it.
fn query_u32(ctx: &'static NdisCtx, oid: u32) -> Option<u32> {
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let mut buf: u32 = 0;
    let mut req: NdisRequest = unsafe { core::mem::zeroed() };
    req.request_type = NDIS_REQUEST_QUERY_INFORMATION;
    req.data.oid = oid;
    req.data.information_buffer = &mut buf as *mut u32 as *mut c_void;
    req.data.information_buffer_length = 4;
    unsafe { KeInitializeEvent(&ctx.event as *const _ as *mut KEVENT, 0, 0) };
    let mut status: NdisStatus = 0;
    unsafe { NdisRequest(&mut status, binding, &mut req) };
    if status == NDIS_STATUS_PENDING {
        if !wait_event(ctx, 5_000) {
            return None;
        }
        status = ctx.open_status.load(Ordering::Acquire) as i32;
    }
    (status == NDIS_STATUS_SUCCESS).then_some(buf)
}

/// Set a 32-bit OID and wait for it. True on success.
fn set_oid(ctx: &'static NdisCtx, oid: u32, value: u32) -> bool {
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let mut buf = value;
    let mut req: NdisRequest = unsafe { core::mem::zeroed() };
    req.request_type = NDIS_REQUEST_SET_INFORMATION;
    req.data.oid = oid;
    req.data.information_buffer = &mut buf as *mut u32 as *mut c_void;
    req.data.information_buffer_length = 4;
    unsafe { KeInitializeEvent(&ctx.event as *const _ as *mut KEVENT, 0, 0) };
    let mut status: NdisStatus = 0;
    unsafe { NdisRequest(&mut status, binding, &mut req) };
    if status == NDIS_STATUS_PENDING {
        if !wait_event(ctx, 5_000) {
            return false;
        }
        status = ctx.open_status.load(Ordering::Acquire) as i32;
    }
    if status != NDIS_STATUS_SUCCESS {
        nsprint!("[NETSTACK] NDIS: set OID {:#010x} failed {:#010x}\n", oid, status as u32);
    }
    status == NDIS_STATUS_SUCCESS
}

/// OID-query OID_802_3_CURRENT_ADDRESS and wait for request_complete.
fn query_mac(ctx: &'static NdisCtx) -> Option<[u8; 6]> {
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let mut mac_buf = [0u8; 6];
    let mut req: NdisRequest = unsafe { core::mem::zeroed() };
    req.request_type = NDIS_REQUEST_QUERY_INFORMATION;
    req.data.oid = OID_802_3_CURRENT_ADDRESS;
    req.data.information_buffer = mac_buf.as_mut_ptr() as *mut c_void;
    req.data.information_buffer_length = 6;

    unsafe { KeInitializeEvent(&ctx.event as *const _ as *mut KEVENT, 0, 0) };
    ctx.mac_ready.store(0, Ordering::Release);
    let mut status: NdisStatus = 0;
    unsafe { NdisRequest(&mut status, binding, &mut req) };
    if status == NDIS_STATUS_PENDING {
        if !wait_event(ctx, 5_000) {
            nsprint!("[NETSTACK] NDIS: MAC query timed out\n");
            return None;
        }
        status = ctx.open_status.load(Ordering::Acquire) as i32;
    }
    if status != NDIS_STATUS_SUCCESS {
        nsprint!("[NETSTACK] NDIS: MAC query failed {:#010x}\n", status as u32);
        return None;
    }
    nsprint!(
        "[NETSTACK] NDIS: bound adapter MAC {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}\n",
        mac_buf[0], mac_buf[1], mac_buf[2], mac_buf[3], mac_buf[4], mac_buf[5]
    );
    Some(mac_buf)
}

/// Block on the ctx event with a millisecond timeout. Returns true if signaled.
fn wait_event(ctx: &NdisCtx, timeout_ms: i64) -> bool {
    let mut t: LARGE_INTEGER = unsafe { core::mem::zeroed() };
    t.QuadPart = -(timeout_ms * 10_000);
    let r = unsafe {
        KeWaitForSingleObject(
            &ctx.event as *const _ as PVOID,
            0, // Executive
            0, // KernelMode
            0, // not alertable
            &mut t,
        )
    };
    r == 0 // STATUS_SUCCESS (vs STATUS_TIMEOUT 0x102)
}

/// Marshal a flat frame into an NDIS_PACKET + NDIS_BUFFER and send it. The frame
/// bytes are copied into a non-paged buffer owned by the packet (freed in
/// send_complete). Called from the worker's TX token at PASSIVE.
pub fn send_frame(frame: &[u8]) {
    let Some(ctx) = ctx() else { return };
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let pkt_pool = ctx.packet_pool.load(Ordering::Acquire) as NdisHandle;
    let buf_pool = ctx.buffer_pool.load(Ordering::Acquire) as NdisHandle;
    if binding.is_null() || pkt_pool.is_null() {
        tx_fail("no binding or packet pool", 0);
        return;
    }
    let len = frame.len();
    unsafe {
        let data = ExAllocatePoolWithTag(NON_PAGED_POOL as _, len as _, TAG) as *mut u8;
        if data.is_null() {
            tx_fail("no pool for the frame", 0);
            return;
        }
        core::ptr::copy_nonoverlapping(frame.as_ptr(), data, len);

        let mut s1: NdisStatus = 0;
        let mut packet: *mut NdisPacket = core::ptr::null_mut();
        NdisAllocatePacket(&mut s1, &mut packet, pkt_pool);
        if s1 != NDIS_STATUS_SUCCESS || packet.is_null() {
            tx_fail("NdisAllocatePacket", s1);
            ExFreePool(data as PVOID);
            return;
        }
        let mut s2: NdisStatus = 0;
        let mut buffer: *mut _MDL = core::ptr::null_mut();
        NdisAllocateBuffer(&mut s2, &mut buffer, buf_pool, data as *mut c_void, len as u32);
        if s2 != NDIS_STATUS_SUCCESS || buffer.is_null() {
            tx_fail("NdisAllocateBuffer", s2);
            NdisFreePacket(packet);
            ExFreePool(data as PVOID);
            return;
        }
        // Chain the single buffer at the front (NdisChainBufferAtFront inline) and
        // stash the data pointer in ProtocolReserved so send_complete can free it.
        (*packet).private.head = buffer;
        (*packet).private.tail = buffer;
        (*packet).private.valid_counts = 0;
        (*packet).protocol_reserved[0] = data as usize;

        let mut s3: NdisStatus = 0;
        NdisSend(&mut s3, binding, packet);
        if FIRST_TX.swap(false, Ordering::Relaxed) {
            nsprint!("[NETSTACK] NDIS: first frame sent ({} bytes, status {:#010x})\n", len, s3 as u32);
        }
        if s3 != NDIS_STATUS_PENDING {
            // Completed synchronously — recycle now (send_complete won't run).
            free_tx_packet(packet);
        }
    }
}

/// Say why a send was dropped, once per reason class (the first failure).
fn tx_fail(what: &str, status: NdisStatus) {
    if TX_FAIL_SEEN.swap(true, Ordering::Relaxed) {
        return;
    }
    nsprint!("[NETSTACK] NDIS: send dropped: {} ({:#010x})\n", what, status as u32);
}
static TX_FAIL_SEEN: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

/// Free a TX packet's buffer chain + backing data + the packet itself.
unsafe fn free_tx_packet(packet: *mut NdisPacket) {
    let buffer = (*packet).private.head;
    let data = (*packet).protocol_reserved[0] as *mut u8;
    if !buffer.is_null() {
        NdisFreeBuffer(buffer);
    }
    if !data.is_null() {
        ExFreePool(data as PVOID);
    }
    NdisFreePacket(packet);
}

// --- Protocol callbacks (most run at DISPATCH_LEVEL) ------------------------- #

/// NDIS calls this for each miniport that matches our medium. Open the adapter.
unsafe extern "system" fn bind_adapter(
    status_out: *mut NdisStatus,
    bind_context: NdisHandle,
    device_name: *mut UNICODE_STRING,
    _ss1: *mut c_void,
    _ss2: *mut c_void,
) {
    let Some(ctx) = ctx() else {
        *status_out = NDIS_STATUS_SUCCESS;
        return;
    };
    // Only bind the first adapter (single-NIC scaffold).
    if ctx.binding_handle.load(Ordering::Acquire) != 0 {
        *status_out = NDIS_STATUS_SUCCESS;
        return;
    }
    nsprint!("[NETSTACK] NDIS: binding an adapter\n");
    ctx.bind_context.store(bind_context as usize, Ordering::Release);
    let media = [NDIS_MEDIUM_802_3];
    let mut open_status: NdisStatus = 0;
    let mut open_err: NdisStatus = 0;
    let mut binding: NdisHandle = core::ptr::null_mut();
    let mut selected: u32 = 0;
    NdisOpenAdapter(
        &mut open_status,
        &mut open_err,
        &mut binding,
        &mut selected,
        media.as_ptr(),
        media.len() as u32,
        ctx.protocol_handle,
        ctx as *const _ as NdisHandle, // ProtocolBindingContext
        device_name,
        0,
        core::ptr::null_mut(),
    );
    // NdisOpenAdapter writes the binding handle into the OUT param immediately
    // (valid even on PENDING); OpenAdapterComplete only reports completion. So
    // record it here, then signal inline (success/failure) or let the completion
    // callback signal (PENDING).
    if !binding.is_null() {
        ctx.binding_handle.store(binding as usize, Ordering::Release);
    }
    ctx.open_status.store(open_status as usize, Ordering::Release);
    if open_status != NDIS_STATUS_PENDING {
        ctx.bind_context.store(0, Ordering::Release);
        if open_status != NDIS_STATUS_SUCCESS {
            nsprint!("[NETSTACK] NDIS: OpenAdapter failed {:#010x}\n", open_status as u32);
        }
        signal(ctx);
    }
    *status_out = open_status;
}

unsafe extern "system" fn open_adapter_complete(
    binding_context: NdisHandle,
    status: NdisStatus,
    open_err: NdisStatus,
) {
    let ctx = &*(binding_context as *const NdisCtx);
    ctx.open_status.store(status as usize, Ordering::Release);
    // bind_adapter returned PENDING; NDIS waits for this to finish the bind.
    let bind = ctx.bind_context.swap(0, Ordering::AcqRel) as NdisHandle;
    if !bind.is_null() {
        NdisCompleteBindAdapter(bind, status, open_err);
    }
    if status != NDIS_STATUS_SUCCESS {
        nsprint!("[NETSTACK] NDIS: OpenAdapter completed with {:#010x}\n", status as u32);
    }
    signal(ctx);
}

unsafe extern "system" fn close_adapter_complete(_binding_context: NdisHandle, _status: NdisStatus) {}

unsafe extern "system" fn send_complete(
    _binding_context: NdisHandle,
    packet: *mut NdisPacket,
    _status: NdisStatus,
) {
    if !packet.is_null() {
        free_tx_packet(packet);
    }
}

unsafe extern "system" fn transfer_data_complete(
    _binding_context: NdisHandle,
    packet: *mut NdisPacket,
    status: NdisStatus,
    bytes: u32,
) {
    if !packet.is_null() {
        finish_transfer(packet, status, bytes);
    }
}

/// A transfer buffer: the header, then the frame's data from offset 0.
#[repr(C)]
struct XferBuf {
    header_len: usize,
    frame: [u8; FRAME_MAX],
}

/// Push a transferred frame into the ring and free its packet (DISPATCH).
unsafe fn finish_transfer(packet: *mut NdisPacket, status: NdisStatus, bytes: u32) {
    let xfer = (*packet).protocol_reserved[0] as *mut XferBuf;
    if status == NDIS_STATUS_SUCCESS && !xfer.is_null() {
        let total = ((*xfer).header_len + bytes as usize).min(FRAME_MAX);
        RX_RING.push(&(&(*xfer).frame)[..total]);
        wake_rx();
    }
    free_tx_packet(packet); // same shape: one buffer, our data in ProtocolReserved
}

/// The frame did not fit the lookahead: copy the header, and have the miniport
/// copy the data in after it (NdisTransferData), now or in transfer_data_complete.
unsafe fn transfer_frame(mac_context: NdisHandle, header: *const u8, header_size: usize, packet_size: u32) {
    let Some(ctx) = ctx() else { return };
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let pkt_pool = ctx.packet_pool.load(Ordering::Acquire) as NdisHandle;
    let buf_pool = ctx.buffer_pool.load(Ordering::Acquire) as NdisHandle;
    if binding.is_null() || pkt_pool.is_null() || header_size + packet_size as usize > FRAME_MAX {
        return;
    }
    let xfer = ExAllocatePoolWithTag(NON_PAGED_POOL as _, core::mem::size_of::<XferBuf>() as _, TAG) as *mut XferBuf;
    if xfer.is_null() {
        return;
    }
    (*xfer).header_len = header_size;
    core::ptr::copy_nonoverlapping(header, (*xfer).frame.as_mut_ptr(), header_size);
    let mut s1: NdisStatus = 0;
    let mut packet: *mut NdisPacket = core::ptr::null_mut();
    NdisAllocatePacket(&mut s1, &mut packet, pkt_pool);
    if s1 != NDIS_STATUS_SUCCESS || packet.is_null() {
        ExFreePool(xfer as PVOID);
        return;
    }
    let mut s2: NdisStatus = 0;
    let mut buffer: *mut _MDL = core::ptr::null_mut();
    let data = (*xfer).frame.as_mut_ptr().add(header_size);
    NdisAllocateBuffer(&mut s2, &mut buffer, buf_pool, data as *mut c_void, packet_size);
    if s2 != NDIS_STATUS_SUCCESS || buffer.is_null() {
        NdisFreePacket(packet);
        ExFreePool(xfer as PVOID);
        return;
    }
    (*packet).private.head = buffer;
    (*packet).private.tail = buffer;
    (*packet).private.valid_counts = 0;
    (*packet).protocol_reserved[0] = xfer as usize;
    let mut status: NdisStatus = 0;
    let mut transferred: u32 = 0;
    NdisTransferData(&mut status, binding, mac_context, 0, packet_size, packet, &mut transferred);
    if status != NDIS_STATUS_PENDING {
        finish_transfer(packet, status, transferred);
    }
}

unsafe extern "system" fn reset_complete(_binding_context: NdisHandle, _status: NdisStatus) {}

unsafe extern "system" fn request_complete(
    binding_context: NdisHandle,
    _request: *mut NdisRequest,
    status: NdisStatus,
) {
    let ctx = &*(binding_context as *const NdisCtx);
    ctx.open_status.store(status as usize, Ordering::Release);
    signal(ctx);
}

/// RECEIVE_HANDLER (DISPATCH) — lookahead/header indication. If the whole frame
/// fits the lookahead (the common case for our sizes), copy header+lookahead into
/// the RX ring; a larger frame comes in through NdisTransferData.
unsafe extern "system" fn protocol_receive(
    _binding_context: NdisHandle,
    mac_context: NdisHandle,
    header: *mut c_void,
    header_size: u32,
    lookahead: *mut c_void,
    lookahead_size: u32,
    packet_size: u32,
) -> NdisStatus {
    if packet_size > lookahead_size {
        if !header.is_null() {
            transfer_frame(mac_context, header as *const u8, header_size as usize, packet_size);
        }
        return NDIS_STATUS_SUCCESS;
    }
    let total = (header_size + lookahead_size) as usize;
    if total == 0 || total > FRAME_MAX {
        return NDIS_STATUS_SUCCESS;
    }
    let mut frame = [0u8; FRAME_MAX];
    let hs = header_size as usize;
    if hs > 0 && !header.is_null() {
        core::ptr::copy_nonoverlapping(header as *const u8, frame.as_mut_ptr(), hs);
    }
    let ls = lookahead_size as usize;
    if ls > 0 && !lookahead.is_null() {
        core::ptr::copy_nonoverlapping(lookahead as *const u8, frame.as_mut_ptr().add(hs), ls);
    }
    RX_RING.push(&frame[..total]);
    wake_rx();
    NDIS_STATUS_SUCCESS
}

unsafe extern "system" fn receive_complete(_binding_context: NdisHandle) {}

/// RECEIVE_PACKET_HANDLER (DISPATCH) — packet-mode indication. Walk the NDIS_BUFFER
/// (MDL) chain, copy the whole frame into a flat scratch buffer, push to the RX
/// ring. Returns 0 (we copied; we do NOT retain the packet, so no ref is held).
unsafe extern "system" fn receive_packet(_binding_context: NdisHandle, packet: *mut NdisPacket) -> i32 {
    if packet.is_null() {
        return 0;
    }
    let mut frame = [0u8; FRAME_MAX];
    let mut off = 0usize;
    let mut mdl = (*packet).private.head;
    while !mdl.is_null() {
        let va = MmGetSystemAddressForMdlSafe(mdl as *mut _, 16);
        let len = (*(mdl as *const _MDL)).ByteCount as usize;
        if !va.is_null() && len > 0 {
            let n = len.min(FRAME_MAX - off);
            core::ptr::copy_nonoverlapping(va as *const u8, frame.as_mut_ptr().add(off), n);
            off += n;
            if off >= FRAME_MAX {
                break;
            }
        }
        mdl = (*(mdl as *const _MDL)).Next;
    }
    if off > 0 {
        RX_RING.push(&frame[..off]);
        wake_rx();
    }
    0 // not retained
}

unsafe extern "system" fn status_handler(
    _binding_context: NdisHandle,
    _general_status: NdisStatus,
    _status_buffer: *mut c_void,
    _status_buffer_size: u32,
) {
}

unsafe extern "system" fn status_complete(_binding_context: NdisHandle) {}

unsafe extern "system" fn unbind_adapter(
    status_out: *mut NdisStatus,
    _binding_context: NdisHandle,
    _unbind_context: NdisHandle,
) {
    *status_out = NDIS_STATUS_SUCCESS;
}

extern "system" {
    fn KeSetEvent(Event: *mut KEVENT, Increment: i32, Wait: u8) -> i32;
}

/// Signal the ctx completion event (KeSetEvent).
fn signal(ctx: &NdisCtx) {
    unsafe { KeSetEvent(&ctx.event as *const _ as *mut KEVENT, 0, 0) };
}

static FIRST_TX: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);
static FIRST_RX: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(true);

/// Wake the worker for a received frame (legal at DISPATCH).
fn wake_rx() {
    if FIRST_RX.swap(false, Ordering::Relaxed) {
        nsprint!("[NETSTACK] NDIS: first frame received\n");
    }
    if let Some(c) = ctx() {
        unsafe { KeSetEvent(&c.rx_event as *const _ as *mut KEVENT, 0, 0) };
    }
}
