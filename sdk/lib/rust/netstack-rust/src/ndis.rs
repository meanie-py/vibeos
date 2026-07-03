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
//! Status: compile-verified scaffold (build `--features ndis`). Getting ndis.sys
//! + a miniport onto the QEMU boot path, then live binding, is the remaining
//! sub-task (design § M11) — it is its own lane with its own bring-up bugs.

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
const NDIS_REQUEST_QUERY_INFORMATION: u32 = 0; // NdisRequestQueryInformation
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
#[repr(C)]
struct NdisPacket {
    private: NdisPacketPrivate,
    // reserved scratch (MiniportReserved/WrapperReserved/Reserved/ProtocolReserved)
    // follows; sized generously so a real NDIS_PACKET allocation isn't undersized.
    reserved: [usize; 12],
}

/// NDIS_REQUEST (legacy) — only the QUERY_INFORMATION arm is used (MAC query).
#[repr(C)]
struct NdisRequest {
    mac_reserved: [usize; 4],
    request_type: u32,
    oid: u32,
    information_buffer: *mut c_void,
    information_buffer_length: u32,
    bytes_written: u32,
    bytes_needed: u32,
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
        protocol_handle: core::ptr::null_mut(),
        binding_handle: AtomicUsize::new(0),
        packet_pool: AtomicUsize::new(0),
        buffer_pool: AtomicUsize::new(0),
        open_status: AtomicUsize::new(0),
        mac: AtomicU64::new(0),
        mac_ready: AtomicUsize::new(0),
    }));
    unsafe { KeInitializeEvent(&mut ctx_box.event, 0 /* NotificationEvent */, 0) };
    CTX.store(ctx_box as *mut NdisCtx as usize, Ordering::Release);

    let chars = NdisProtocolCharacteristics {
        major_version: 5,
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

    // Wait for bind_adapter -> NdisOpenAdapter -> open_adapter_complete to signal.
    if !wait_event(ctx_box, 10_000) || ctx_box.binding_handle.load(Ordering::Acquire) == 0 {
        nsprint!("[NETSTACK] NDIS: no miniport bound (no adapter / open failed)\n");
        return None;
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
    ctx_box.packet_pool.store(pkt_pool as usize, Ordering::Release);
    ctx_box.buffer_pool.store(buf_pool as usize, Ordering::Release);

    query_mac(ctx_box)
}

/// OID-query OID_802_3_CURRENT_ADDRESS and wait for request_complete.
fn query_mac(ctx: &'static NdisCtx) -> Option<[u8; 6]> {
    let binding = ctx.binding_handle.load(Ordering::Acquire) as NdisHandle;
    let mut mac_buf = [0u8; 6];
    let mut req: NdisRequest = unsafe { core::mem::zeroed() };
    req.request_type = NDIS_REQUEST_QUERY_INFORMATION;
    req.oid = OID_802_3_CURRENT_ADDRESS;
    req.information_buffer = mac_buf.as_mut_ptr() as *mut c_void;
    req.information_buffer_length = 6;

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
    if binding.is_null() || pkt_pool.is_null() || buf_pool.is_null() {
        return;
    }
    let len = frame.len();
    unsafe {
        let data = ExAllocatePoolWithTag(NON_PAGED_POOL as _, len as _, TAG) as *mut u8;
        if data.is_null() {
            return;
        }
        core::ptr::copy_nonoverlapping(frame.as_ptr(), data, len);

        let mut s1: NdisStatus = 0;
        let mut packet: *mut NdisPacket = core::ptr::null_mut();
        NdisAllocatePacket(&mut s1, &mut packet, pkt_pool);
        if s1 != NDIS_STATUS_SUCCESS || packet.is_null() {
            ExFreePool(data as PVOID);
            return;
        }
        let mut s2: NdisStatus = 0;
        let mut buffer: *mut _MDL = core::ptr::null_mut();
        NdisAllocateBuffer(&mut s2, &mut buffer, buf_pool, data as *mut c_void, len as u32);
        if s2 != NDIS_STATUS_SUCCESS || buffer.is_null() {
            NdisFreePacket(packet);
            ExFreePool(data as PVOID);
            return;
        }
        // Chain the single buffer at the front (NdisChainBufferAtFront inline) and
        // stash the data pointer in ProtocolReserved so send_complete can free it.
        (*packet).private.head = buffer;
        (*packet).private.tail = buffer;
        (*packet).private.valid_counts = 0;
        (*packet).reserved[0] = data as usize;

        let mut s3: NdisStatus = 0;
        NdisSend(&mut s3, binding, packet);
        if s3 != NDIS_STATUS_PENDING {
            // Completed synchronously — recycle now (send_complete won't run).
            free_tx_packet(packet);
        }
    }
}

/// Free a TX packet's buffer chain + backing data + the packet itself.
unsafe fn free_tx_packet(packet: *mut NdisPacket) {
    let buffer = (*packet).private.head;
    let data = (*packet).reserved[0] as *mut u8;
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
    _bind_context: NdisHandle,
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
    _open_err: NdisStatus,
) {
    let ctx = &*(binding_context as *const NdisCtx);
    ctx.open_status.store(status as usize, Ordering::Release);
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
    _packet: *mut NdisPacket,
    _status: NdisStatus,
    _bytes: u32,
) {
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
/// the RX ring. A larger frame would need NdisTransferData (remaining sub-task).
unsafe extern "system" fn protocol_receive(
    _binding_context: NdisHandle,
    _mac_context: NdisHandle,
    header: *mut c_void,
    header_size: u32,
    lookahead: *mut c_void,
    lookahead_size: u32,
    packet_size: u32,
) -> NdisStatus {
    if packet_size > lookahead_size {
        // Tail not in the lookahead — would require NdisTransferData; drop for now.
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

/// Signal the ctx completion event (KeSetEvent).
fn signal(ctx: &NdisCtx) {
    extern "system" {
        fn KeSetEvent(Event: *mut KEVENT, Increment: i32, Wait: u8) -> i32;
    }
    unsafe { KeSetEvent(&ctx.event as *const _ as *mut KEVENT, 0, 0) };
}
