//! WDI: the Microsoft half of a Windows 10 Wi-Fi driver.
//!
//! Not a translation: on Windows this half is wdiwifi.sys, which has no source. A WDI driver
//! (an "IHV lower edge", LE) never registers as an NDIS miniport itself. It hands
//! NdisMRegisterWdiMiniportDriver two tables, its partial NDIS 6 characteristics and its WDI
//! handlers, and expects the "upper edge" (UE, this file) to be the miniport NDIS sees.
//!
//!   NDIS 6 (ndis6.rs)                 UE (here)                      LE (vendor driver)
//!   InitializeHandlerEx         ->    bring-up                 ->    AllocateAdapter, OpenAdapter,
//!                                                                    TalTxRxInitialize, then WDI
//!                                                                    commands over OidRequest
//!   NdisMOidRequestComplete     <-    command completion (M3)  <-    NdisMOidRequestComplete
//!   NdisMIndicateStatusEx       <-    task done (M4), events   <-    NdisMIndicateStatusEx
//!
//! To protocols the adapter is plain 802.3: the UE turns Wi-Fi into Ethernet, as nwifi.sys
//! does on Windows. Commands are the WDI message format, a WDI_MESSAGE_HEADER followed by
//! little-endian TLVs, sent as NdisRequestMethod OIDs. The OID numbers, the task-completion
//! status codes and each message's TLV layout come from the WDK documentation and from the
//! tables the vendor driver's own (generated) TLV parser carries.
//!
//! Stage A: bring the adapter up, create the station port, and scan, logging what is found.

use crate::prelude::*;
use crate::ndis::ndis6::{
    MiniportDriverCharacteristics, MiniportInitParameters, NdisMRegisterMiniportDriver,
    NdisMSetMiniportAttributes, NdisMDeregisterMiniportDriver, NdisMSendNetBufferListsComplete,
    NdisObjectHeader, NdisOidRequest, RegistrationAttributes, GeneralAttributes, StatusIndication,
    NetBufferList, NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES,
    NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES, NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS,
    NDIS_OBJECT_TYPE_OID_REQUEST,
};
use ntkrnl_sys::ddk::{ExFreePool, KeInitializeEvent, KeWaitForSingleObject};
use core::ptr::{null_mut, addr_of_mut};
use core::sync::atomic::{AtomicPtr, AtomicU32, Ordering};

macro_rules! logw {
    ($fmt:literal $(, $a:expr)* $(,)?) => {
        { ntkrnl_sys::driver_ddk::DbgPrint(concat!("[WDI] ", $fmt, "\0").as_ptr() as *const i8 $(, $a)*); }
    };
}

const NDIS_STATUS_INVALID_LENGTH: NDIS_STATUS = 0xC001_0014u32 as NDIS_STATUS;
const NDIS_STATUS_MEDIA_DISCONNECTED: NDIS_STATUS = 0xC001_000Bu32 as NDIS_STATUS;

const WDI_TAG: u32 = u32::from_le_bytes(*b"IDW6");

/// The UE's WDI version: the one Realtek's 2020 drivers declare (1.1.4). Structures without
/// an NDIS header follow the lower of the two sides' versions, and the LE's TLV parser
/// refuses a peer below 1.0.
const UE_WDI_VERSION: u32 = 0x0001_0104;

// ---- WDI layouts (dot11wdi.h) ------------------------------------------------------------

type MiniportHandler<A, R> = Option<unsafe extern "system" fn(A) -> R>;

#[repr(C)]
pub struct WdiCharacteristics {
    pub Header: NdisObjectHeader,
    pub WdiVersion: u32,
    pub AllocateAdapter: Option<unsafe extern "system" fn(NDIS_HANDLE, NDIS_HANDLE, *mut MiniportInitParameters, *mut WdiInitParameters, *mut RegistrationAttributes) -> NDIS_STATUS>,
    pub FreeAdapter: MiniportHandler<NDIS_HANDLE, ()>,
    pub OpenAdapter: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut MiniportInitParameters) -> NDIS_STATUS>,
    pub CloseAdapter: MiniportHandler<NDIS_HANDLE, NDIS_STATUS>,
    pub StartOperation: MiniportHandler<NDIS_HANDLE, NDIS_STATUS>,
    pub StopOperation: MiniportHandler<NDIS_HANDLE, ()>,
    pub PostPause: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID) -> NDIS_STATUS>,
    pub PostRestart: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID) -> NDIS_STATUS>,
    pub HangDiagnose: PVOID,
    pub TalTxRxInitialize: Option<unsafe extern "system" fn(NDIS_HANDLE, NDIS_HANDLE, *mut WdiDataApi, *mut PVOID, *mut WdiDataHandlers, *mut u32) -> NDIS_STATUS>,
    pub TalTxRxDeinitialize: MiniportHandler<PVOID, ()>,
    pub LeIdleNotification: PVOID,
    pub LeCancelIdleNotification: PVOID,
}

#[repr(C)]
pub struct WdiInitParameters {
    pub Header: NdisObjectHeader,
    pub WdiVersion: u32,
    pub OpenAdapterComplete: unsafe extern "system" fn(NDIS_HANDLE, NDIS_STATUS),
    pub CloseAdapterComplete: unsafe extern "system" fn(NDIS_HANDLE, NDIS_STATUS),
    pub IdleNotificationConfirm: unsafe extern "system" fn(NDIS_HANDLE, u32),
    pub IdleNotificationComplete: unsafe extern "system" fn(NDIS_HANDLE),
}

/// NDIS_WDI_DATA_API: the UE's data-path entry points, handed to the LE. dot11wdi.h packs
/// these tables to 4: the first entry point follows the 4-byte header directly (Realtek's
/// driver stamps its handler table 0xCC bytes, 4 + 25 * 8, and copies ours as 4 + 17 * 8).
#[repr(C, packed(4))]
pub struct WdiDataApi {
    pub Header: NdisObjectHeader,
    pub TxDequeueIndication: unsafe extern "system" fn(PVOID, u32, u8, u16, *mut *mut NetBufferList),
    pub TxTransferCompleteIndication: unsafe extern "system" fn(PVOID, u32, *mut NetBufferList),
    pub TxSendCompleteIndication: unsafe extern "system" fn(PVOID, u32, u16, *mut u16, PVOID),
    pub TxQueryRaTidState: unsafe extern "system" fn(PVOID, u16, u16, u8, *mut NDIS_STATUS, *mut u16),
    pub TxSendPauseIndication: unsafe extern "system" fn(PVOID, u16, u16, u32, u32),
    pub TxSendRestartIndication: unsafe extern "system" fn(PVOID, u16, u16, u32, u32),
    pub TxReleaseFrameIndication: unsafe extern "system" fn(PVOID, u16, u16, u32, u8, u16, *mut *mut NetBufferList),
    pub TxInjectFrameIndication: unsafe extern "system" fn(PVOID, u16, u16, u8, *mut NetBufferList, u8, u8, u16, u32),
    pub TxAbortConfirm: unsafe extern "system" fn(PVOID),
    pub RxInorderDataIndication: unsafe extern "system" fn(PVOID, u32, u16, u8, PVOID, *mut NDIS_STATUS),
    pub RxStopConfirm: unsafe extern "system" fn(PVOID),
    pub RxFlushConfirm: unsafe extern "system" fn(PVOID),
    pub PeerCreateIndication: unsafe extern "system" fn(PVOID, u16, u16, WdiMac),
    pub PeerDeleteIndication: unsafe extern "system" fn(PVOID, u16, u16, *mut NDIS_STATUS),
    pub AllocateWiFiFrameMetaData: unsafe extern "system" fn(PVOID) -> PVOID,
    pub FreeWiFiFrameMetaData: unsafe extern "system" fn(PVOID, PVOID),
    pub TxQuerySuspectFrameCompleteStatus: unsafe extern "system" fn(PVOID, u64, *mut NetBufferList, *mut u8, *mut u8, *mut NDIS_STATUS),
}

/// NDIS_MINIPORT_WDI_DATA_HANDLERS: the LE's data-path entry points. Only the ones the UE
/// calls are typed; room is left for revisions that add more.
#[repr(C, packed(4))]
pub struct WdiDataHandlers {
    pub Header: NdisObjectHeader,
    pub TxAbort: PVOID,
    pub TxTargetDescInit: PVOID,
    pub TxTargetDescDeInit: PVOID,
    pub TxDataSend: PVOID,
    pub TxTalSend: PVOID,
    pub TxTalSendComplete: PVOID,
    pub TxTalQueueInOrder: PVOID,
    pub TxPeerBacklog: PVOID,
    pub RxStop: PVOID,
    pub RxFlush: PVOID,
    pub RxRestart: PVOID,
    pub RxGetMpdus: Option<unsafe extern "system" fn(PVOID, u16, u8, *mut *mut NetBufferList)>,
    pub RxReturnFrames: Option<unsafe extern "system" fn(PVOID, *mut NetBufferList)>,
    pub RxResume: PVOID,
    pub RxThrottle: PVOID,
    pub RxPpduRssi: PVOID,
    pub TalTxRxStart: Option<unsafe extern "system" fn(PVOID, *mut TxRxTargetConfiguration, *mut u16) -> NDIS_STATUS>,
    pub TalTxRxStop: MiniportHandler<PVOID, ()>,
    pub TalTxRxAddPort: Option<unsafe extern "system" fn(PVOID, u16, u32)>,
    pub TalTxRxDeletePort: Option<unsafe extern "system" fn(PVOID, u16)>,
    pub TalTxRxSetPortOpMode: PVOID,
    pub TalTxRxResetPort: PVOID,
    pub TalTxRxPeerConfig: PVOID,
    pub TalTxRxPeerDeleteConfirm: PVOID,
    pub TxSuspectFrameAbort: PVOID,
    pub Reserved: [PVOID; 7],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WdiMac(pub [u8; 6]);

/// WDI_TXRX_TARGET_CONFIGURATION (WDI_TXRX_CAPABILITIES + port/peer counts), natural layout.
#[repr(C)]
#[derive(Default)]
pub struct TxRxTargetConfiguration {
    pub InterconnectType: u32,
    pub TargetPriorityQueueing: u8,
    pub MaxScatterGatherElementsPerFrame: u16,
    pub ExplicitSendCompleteFlagRequired: u8,
    pub bPad: u8,
    pub MinEffectiveSize: u16,
    pub FrameSizeGranularity: u16,
    pub RxTxForwarding: u8,
    pub MaxThroughput: u32,
    pub MaxNumPorts: u8,
    pub MaxNumPeers: u8,
}

/// WDI_MESSAGE_HEADER: every command, completion and indication starts with one.
#[repr(C)]
#[derive(Clone, Copy)]
struct MsgHeader {
    PortId: u16,
    Reserved: u16,
    Status: NDIS_STATUS,
    TransactionId: u32,
    IhvSpecificId: u32,
}
const HDR: usize = core::mem::size_of::<MsgHeader>();

/// NDIS_OID_REQUEST's DATA.METHOD_INFORMATION, which carries WDI commands. MethodId comes
/// after the buffer lengths, not after the OID.
#[repr(C)]
struct MethodInfo {
    Oid: u32,
    InformationBuffer: PVOID,
    InputBufferLength: u32,
    OutputBufferLength: u32,
    MethodId: u32,
    BytesWritten: u32,
    BytesRead: u32,
    BytesNeeded: u32,
}
const NDIS_REQUEST_METHOD: u32 = 12;

// NDIS_OBJECT_TYPE values for the WDI tables. The LE stamps its own data-handler header
// (0xC1, revision 2); none of the LE's entry points check the UE's.
const NDIS_OBJECT_TYPE_WDI_DATA_HANDLERS: u8 = 0xC1;
const NDIS_OBJECT_TYPE_WDI_DATA_API: u8 = 0xC2;
const NDIS_OBJECT_TYPE_WDI_INIT_PARAMETERS: u8 = 0xC3;

// ---- the WDI protocol: OIDs, completions, TLVs -------------------------------------------

// From the vendor driver's command table (name, OID, completion status).
const OID_WDI_TASK_SCAN: u32 = 0xE440_0004;
const OID_WDI_TASK_SET_RADIO_STATE: u32 = 0xE440_000B;
const OID_WDI_TASK_CREATE_PORT: u32 = 0xE440_000C;
const OID_WDI_GET_ADAPTER_CAPABILITIES: u32 = 0xE440_0025;
const OID_WDI_SET_RECEIVE_PACKET_FILTER: u32 = 0xE440_0024;

const DONE_SCAN: NDIS_STATUS = 0x4005_003C;
const DONE_SET_RADIO_STATE: NDIS_STATUS = 0x4005_0016;
const DONE_CREATE_PORT: NDIS_STATUS = 0x4005_0044;

const TLV_STATUS: u16 = 0x01;
const TLV_BSSID: u16 = 0x02;
const TLV_SCAN_MODE: u16 = 0x06;
const TLV_SCAN_DWELL_TIME: u16 = 0x07;
const TLV_BSS_ENTRY: u16 = 0x08;
const TLV_PROBE_RESPONSE_FRAME: u16 = 0x09;
const TLV_BEACON_FRAME: u16 = 0x0A;
const TLV_BSS_ENTRY_SIGNAL_INFO: u16 = 0x0B;
const TLV_INTERFACE_CAPABILITIES: u16 = 0x0F;
const TLV_UNICAST_ALGORITHM_LIST: u16 = 0x13;
const TLV_INTERFACE_ATTRIBUTES: u16 = 0x21;
const TLV_STATION_ATTRIBUTES: u16 = 0x22;
const TLV_CREATE_PORT_PARAMETERS: u16 = 0x28;
const TLV_PORT_ATTRIBUTES: u16 = 0x29;
const TLV_BSS_ENTRY_CHANNEL_INFO: u16 = 0x3A;
const TLV_SSID: u16 = 0x3B;
const TLV_PACKET_FILTER_PARAMETERS: u16 = 0x47;
const TLV_RADIO_STATE_PARAMETERS: u16 = 0xA0;
const TLV_DATAPATH_ATTRIBUTES: u16 = 0xB8;
const TLV_DATAPATH_CAPABILITIES: u16 = 0xB9;
const TLV_FIRMWARE_VERSION: u16 = 0xF4;

const WDI_PORT_ANY: u16 = 0xFFFF;
const WDI_OPERATION_MODE_STA: u16 = 0x1;
// WDI_PACKET_FILTER_TYPE bits (NDIS packet-type values).
const WDI_PACKET_FILTER_DIRECTED: u32 = 0x1;
const WDI_PACKET_FILTER_MULTICAST: u32 = 0x2;
const WDI_PACKET_FILTER_BROADCAST: u32 = 0x8;

/// Building a TLV stream in a caller's buffer. Writes past the end are dropped and remembered,
/// so an overflow is one check at the end rather than one per field.
struct TlvWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
    overflow: bool,
}

impl<'a> TlvWriter<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        TlvWriter { buf, len: 0, overflow: false }
    }
    fn bytes(&mut self, v: &[u8]) {
        if self.len + v.len() > self.buf.len() {
            self.overflow = true;
            return;
        }
        self.buf[self.len..self.len + v.len()].copy_from_slice(v);
        self.len += v.len();
    }
    fn tlv(&mut self, t: u16, v: &[u8]) {
        self.bytes(&t.to_le_bytes());
        self.bytes(&(v.len() as u16).to_le_bytes());
        self.bytes(v);
    }
    /// Open a container TLV; `close` fills in its length.
    fn open(&mut self, t: u16) -> usize {
        let at = self.len;
        self.bytes(&t.to_le_bytes());
        self.bytes(&[0, 0]);
        at
    }
    fn close(&mut self, at: usize) {
        if !self.overflow {
            let n = (self.len - at - 4) as u16;
            self.buf[at + 2..at + 4].copy_from_slice(&n.to_le_bytes());
        }
    }
}

/// The TLVs in `buf`, in order. A truncated trailing TLV ends the walk.
struct Tlvs<'a> {
    buf: &'a [u8],
}

impl<'a> Iterator for Tlvs<'a> {
    type Item = (u16, &'a [u8]);
    fn next(&mut self) -> Option<Self::Item> {
        if self.buf.len() < 4 {
            return None;
        }
        let t = u16::from_le_bytes([self.buf[0], self.buf[1]]);
        let n = u16::from_le_bytes([self.buf[2], self.buf[3]]) as usize;
        if self.buf.len() < 4 + n {
            return None;
        }
        let v = &self.buf[4..4 + n];
        self.buf = &self.buf[4 + n..];
        Some((t, v))
    }
}

fn tlvs(buf: &[u8]) -> Tlvs<'_> {
    Tlvs { buf }
}

fn find(buf: &[u8], t: u16) -> Option<&[u8]> {
    tlvs(buf).find(|(tt, _)| *tt == t).map(|(_, v)| v)
}

fn le16(v: &[u8], at: usize) -> u16 {
    v.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]])).unwrap_or(0)
}
fn le32(v: &[u8], at: usize) -> u32 {
    v.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]])).unwrap_or(0)
}

// ---- state ---------------------------------------------------------------------------------

#[repr(C)]
pub struct WdiDriver {
    /// The LE's own NDIS characteristics and its driver context.
    ihv: MiniportDriverCharacteristics,
    ihv_driver_context: NDIS_HANDLE,
    wdi: WdiCharacteristics,
    /// The handle ndis6.rs gave the UE's registration.
    handle: NDIS_HANDLE,
}

const MAX_BSS: usize = 48;

#[derive(Clone, Copy)]
struct Bss {
    bssid: [u8; 6],
    ssid: [u8; 32],
    ssid_len: u8,
    rssi: i32,
    channel: u32,
    band: u32,
}

const CMD_BUF: usize = 32 * 1024;
const IND_BUF: usize = 8 * 1024;

#[repr(C)]
pub struct WdiAdapter {
    /// NDIS's handle for the adapter (PLOGICAL_ADAPTER).
    adapter: NDIS_HANDLE,
    drv: *mut WdiDriver,
    /// The LE's MiniportAdapterContext, from its AllocateAdapter.
    ihv_ctx: NDIS_HANDLE,
    tal: PVOID,
    data: WdiDataHandlers,
    data_api: WdiDataApi,
    init_params: WdiInitParameters,
    metadata_extra: u32,
    /// Set while the UE sets its own attributes, which the LE may not.
    own_attrs: bool,

    open_event: KEVENT,
    open_status: NDIS_STATUS,

    /// One command at a time: commands are serialized, as WDI requires.
    cmd_lock: KEVENT,
    cmd_buf: *mut u8,
    cmd_req: NdisOidRequest,
    m3_event: KEVENT,
    m3_status: NDIS_STATUS,
    txid: u32,
    /// The task whose completion (M4) the UE waits for, its event and copied payload.
    task_code: AtomicU32,
    task_txid: u32,
    m4_event: KEVENT,
    ind_buf: *mut u8,
    ind_len: usize,

    mac: [u8; 6],
    mtu: u32,
    backfill: u16,
    port_id: u16,
    port_mac: [u8; 6],
    up: bool,

    bss_lock: KSPIN_LOCK,
    bss: [Bss; MAX_BSS],
    bss_count: usize,
}

const MAX_WDI_ADAPTERS: usize = 4;
static ADAPTERS: [AtomicPtr<WdiAdapter>; MAX_WDI_ADAPTERS] = [const { AtomicPtr::new(null_mut()) }; MAX_WDI_ADAPTERS];

unsafe fn adapter_of(handle: NDIS_HANDLE) -> *mut WdiAdapter {
    for a in ADAPTERS.iter() {
        let p = a.load(Ordering::Acquire);
        if !p.is_null() && (*p).adapter == handle {
            return p;
        }
    }
    null_mut()
}

unsafe fn alloc_zeroed(size: usize) -> PVOID {
    let p = ExAllocatePoolWithTag(NonPagedPool, size as SIZE_T, WDI_TAG);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, size);
    }
    p
}

unsafe fn ms_wait(event: *mut KEVENT, ms: u32) -> bool {
    let mut timeout: i64 = -(ms as i64) * 10_000;
    KeWaitForSingleObject(event as PVOID, _KWAIT_REASON_Executive, _MODE_KernelMode as _, FALSE as BOOLEAN,
        &mut timeout as *mut i64 as _) == STATUS_SUCCESS
}

unsafe fn sleep_ms(ms: u32) {
    let mut t: i64 = -(ms as i64) * 10_000;
    ntkrnl_sys::ddk::KeDelayExecutionThread(_MODE_KernelMode as _, FALSE as BOOLEAN, &mut t as *mut i64 as _);
}

// ---- hooks ndis6.rs calls ----------------------------------------------------------------

/// The LE completing a WDI command (M3). False when the request is not the UE's.
pub unsafe fn oid_request_complete(handle: NDIS_HANDLE, req: *mut NdisOidRequest, status: NDIS_STATUS) -> bool {
    let a = adapter_of(handle);
    if a.is_null() || req != addr_of_mut!((*a).cmd_req) {
        return false;
    }
    (*a).m3_status = status;
    KeSetEvent(addr_of_mut!((*a).m3_event), 0, FALSE as BOOLEAN);
    true
}

/// A status indication from a WDI adapter's LE: every one of them is the UE's. False for
/// any other adapter.
pub unsafe fn indicate_status(handle: NDIS_HANDLE, s: *mut StatusIndication) -> bool {
    let a = adapter_of(handle);
    if a.is_null() {
        return false;
    }
    let s = &*s;
    let payload: &[u8] = if s.StatusBuffer.is_null() {
        &[]
    } else {
        core::slice::from_raw_parts(s.StatusBuffer as *const u8, s.StatusBufferSize as usize)
    };
    let hdr = if payload.len() >= HDR { Some(core::ptr::read_unaligned(payload.as_ptr() as *const MsgHeader)) } else { None };
    let (txid, port, wstatus) = hdr.map(|h| (h.TransactionId, h.PortId, h.Status)).unwrap_or((0, 0, 0));

    // A task's completion echoes its command's transaction ID. The status code says which
    // completion it is; it is logged, not trusted, since the UE's table of them is inferred.
    let want = (*a).task_code.load(Ordering::Acquire);
    if want != 0 && txid != 0 && txid == (*a).task_txid {
        logw!("task %u done: indication %08x (expected %08x), WDI status %x, %u bytes\n", txid, s.StatusCode, want,
            wstatus, payload.len() as u32);
        let n = payload.len().min(IND_BUF);
        core::ptr::copy_nonoverlapping(payload.as_ptr(), (*a).ind_buf, n);
        (*a).ind_len = n;
        (*a).task_code.store(0, Ordering::Release);
        KeSetEvent(addr_of_mut!((*a).m4_event), 0, FALSE as BOOLEAN);
        return true;
    }
    logw!("indication %08x: port %x, txid %u, WDI status %x, %u bytes\n", s.StatusCode, port as u32, txid, wstatus,
        payload.len() as u32);
    let body = if payload.len() > HDR { &payload[HDR..] } else { &[][..] };
    let mut entries = 0;
    for (t, v) in tlvs(body) {
        if t == TLV_BSS_ENTRY {
            note_bss(a, v);
            entries += 1;
        }
    }
    if entries == 0 {
        logw!("indication %08x port %x status %x, %u bytes\n", s.StatusCode, port as u32, wstatus, payload.len() as u32);
    }
    true
}

/// Attributes the LE tries to set itself: WDI reserves the registration and general ones for
/// the UE. True when the call must be refused.
pub unsafe fn refuse_attributes(handle: NDIS_HANDLE, ty: u8) -> bool {
    let a = adapter_of(handle);
    if a.is_null() || (*a).own_attrs {
        return false;
    }
    let reserved = ty == NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES
        || ty == NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES;
    if reserved {
        logw!("the driver set attributes type %x, which WDI reserves for the UE; ignored\n", ty as u32);
    }
    reserved
}

// ---- registration ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterWdiMiniportDriver(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
    NdisDriverContext: NDIS_HANDLE,
    MiniportDriverCharacteristics: *mut MiniportDriverCharacteristics,
    MiniportWdiCharacteristics: *mut WdiCharacteristics,
    NdisMiniportDriverHandle: *mut NDIS_HANDLE,
) -> NDIS_STATUS {
    let c = &*MiniportDriverCharacteristics;
    let w = &*MiniportWdiCharacteristics;
    logw!("%wZ registers a WDI %x.%x.%x miniport (NDIS %u.%u)\n", RegistryPath,
        w.WdiVersion >> 16, (w.WdiVersion >> 8) & 0xFF, w.WdiVersion & 0xFF,
        c.MajorNdisVersion as u32, c.MinorNdisVersion as u32);
    if c.OidRequestHandler.is_none() || w.AllocateAdapter.is_none() || w.OpenAdapter.is_none()
        || w.TalTxRxInitialize.is_none()
    {
        logw!("a mandatory handler is missing\n");
        return NDIS_STATUS_BAD_CHARACTERISTICS;
    }
    let drv = alloc_zeroed(core::mem::size_of::<WdiDriver>()) as *mut WdiDriver;
    if drv.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    let n = core::cmp::min(c.Header.Size as usize, core::mem::size_of::<MiniportDriverCharacteristics>());
    core::ptr::copy_nonoverlapping(MiniportDriverCharacteristics as *const u8, addr_of_mut!((*drv).ihv) as *mut u8, n);
    let n = core::cmp::min(w.Header.Size as usize, core::mem::size_of::<WdiCharacteristics>());
    core::ptr::copy_nonoverlapping(MiniportWdiCharacteristics as *const u8, addr_of_mut!((*drv).wdi) as *mut u8, n);
    (*drv).ihv_driver_context = NdisDriverContext;

    // The UE is the NDIS 6 miniport.
    let mut ue: MiniportDriverCharacteristics = core::mem::zeroed();
    ue.Header = NdisObjectHeader {
        Type: NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS,
        Revision: 1,
        Size: core::mem::size_of::<MiniportDriverCharacteristics>() as u16,
    };
    ue.MajorNdisVersion = 6;
    ue.MinorNdisVersion = c.MinorNdisVersion;
    ue.MajorDriverVersion = c.MajorDriverVersion;
    ue.MinorDriverVersion = c.MinorDriverVersion;
    ue.InitializeHandlerEx = Some(ue_initialize);
    ue.HaltHandlerEx = Some(ue_halt);
    ue.UnloadHandler = c.UnloadHandler;
    ue.PauseHandler = Some(ue_pause);
    ue.RestartHandler = Some(ue_restart);
    ue.OidRequestHandler = Some(ue_oid_request);
    ue.SendNetBufferListsHandler = Some(ue_send);
    ue.ReturnNetBufferListsHandler = Some(ue_return);
    ue.ShutdownHandlerEx = Some(ue_shutdown);
    let mut handle: NDIS_HANDLE = null_mut();
    let status = NdisMRegisterMiniportDriver(DriverObject, RegistryPath, drv as NDIS_HANDLE, addr_of_mut!(ue), addr_of_mut!(handle));
    if status != NDIS_STATUS_SUCCESS {
        ExFreePool(drv as PVOID);
        return status;
    }
    (*drv).handle = handle;
    core::ptr::write_volatile(NdisMiniportDriverHandle, handle);
    if let Some(set_options) = (*drv).ihv.SetOptionsHandler {
        let s = set_options(handle, NdisDriverContext);
        if s != NDIS_STATUS_SUCCESS {
            logw!("SetOptions failed %x\n", s);
            return s;
        }
    }
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterWdiMiniportDriver(NdisMiniportDriverHandle: NDIS_HANDLE) {
    NdisMDeregisterMiniportDriver(NdisMiniportDriverHandle);
}

#[repr(C)]
pub struct PortCharacteristics {
    pub Header: NdisObjectHeader,
    pub Flags: u32,
    pub PortNumber: u32,
    pub PortType: u32,
    // MediaConnectState and the rest follow; nothing here reads them.
}

/// NDIS port numbers for a miniport's extra ports. 0 is the default port; the rest are
/// handed out in order and never reused, so a stale number cannot alias a live port.
static NEXT_PORT: AtomicU32 = AtomicU32::new(1);

#[no_mangle]
pub unsafe extern "system" fn NdisMAllocatePort(_NdisMiniportHandle: NDIS_HANDLE, PortCharacteristics: *mut PortCharacteristics) -> NDIS_STATUS {
    let n = NEXT_PORT.fetch_add(1, Ordering::AcqRel);
    (*PortCharacteristics).PortNumber = n;
    logw!("NDIS port %u allocated (type %u)\n", n, (*PortCharacteristics).PortType);
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMFreePort(_NdisMiniportHandle: NDIS_HANDLE, PortNumber: u32) -> NDIS_STATUS {
    logw!("NDIS port %u freed\n", PortNumber);
    NDIS_STATUS_SUCCESS
}

// ---- the commands --------------------------------------------------------------------------

/// Send one WDI command and wait for its completion (M3) and, for a task, its done
/// indication (`done`, M4). Returns the status and the bytes of the M3 response (the M4
/// payload, for a task, is left in `ind_buf`).
unsafe fn wdi_command(a: *mut WdiAdapter, oid: u32, port: u16, tlv: &[u8], done: NDIS_STATUS, timeout_ms: u32) -> (NDIS_STATUS, usize) {
    ms_wait(addr_of_mut!((*a).cmd_lock), u32::MAX / 2);
    let r = wdi_command_locked(a, oid, port, tlv, done, timeout_ms);
    KeSetEvent(addr_of_mut!((*a).cmd_lock), 0, FALSE as BOOLEAN);
    r
}

unsafe fn wdi_command_locked(a: *mut WdiAdapter, oid: u32, port: u16, tlv: &[u8], done: NDIS_STATUS, timeout_ms: u32) -> (NDIS_STATUS, usize) {
    if HDR + tlv.len() > CMD_BUF {
        return (NDIS_STATUS_INVALID_LENGTH, 0);
    }
    (*a).txid += 1;
    let txid = (*a).txid;
    let hdr = MsgHeader { PortId: port, Reserved: 0, Status: 0, TransactionId: txid, IhvSpecificId: 0 };
    core::ptr::write_unaligned((*a).cmd_buf as *mut MsgHeader, hdr);
    core::ptr::copy_nonoverlapping(tlv.as_ptr(), (*a).cmd_buf.add(HDR), tlv.len());

    let req = addr_of_mut!((*a).cmd_req);
    core::ptr::write_bytes(req as *mut u8, 0, core::mem::size_of::<NdisOidRequest>());
    (*req).Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_OID_REQUEST, Revision: 1, Size: 236 /* REVISION_1 */ };
    (*req).RequestType = NDIS_REQUEST_METHOD;
    let m = addr_of_mut!((*req).DATA) as *mut MethodInfo;
    (*m).Oid = oid;
    (*m).InformationBuffer = (*a).cmd_buf as PVOID;
    (*m).InputBufferLength = (HDR + tlv.len()) as u32;
    (*m).OutputBufferLength = CMD_BUF as u32;

    if done != 0 {
        (*a).task_txid = txid;
        KeClearEvent(addr_of_mut!((*a).m4_event));
        (*a).task_code.store(done as u32, Ordering::Release);
    }
    KeClearEvent(addr_of_mut!((*a).m3_event));
    let drv = (*a).drv;
    let mut status = ((*drv).ihv.OidRequestHandler.unwrap_unchecked())((*a).ihv_ctx, req);
    if status == NDIS_STATUS_PENDING {
        if !ms_wait(addr_of_mut!((*a).m3_event), 10_000) {
            logw!("command %08x: no completion in 10 s\n", oid);
            (*a).task_code.store(0, Ordering::Release);
            return (NDIS_STATUS_FAILURE, 0);
        }
        status = (*a).m3_status;
    }
    let written = (*m).BytesWritten as usize;
    let wstatus = if written >= HDR { core::ptr::read_unaligned((*a).cmd_buf as *const MsgHeader).Status } else { 0 };
    logw!("command %08x (txid %u): %x, WDI status %x, %u bytes written, %u read, %u needed\n", oid, txid, status, wstatus,
        written as u32, (*m).BytesRead, (*m).BytesNeeded);
    if status != NDIS_STATUS_SUCCESS || wstatus != 0 {
        logw!("command %08x failed: %x, WDI status %x (needed %u)\n", oid, status, wstatus, (*m).BytesNeeded);
        (*a).task_code.store(0, Ordering::Release);
        return (if status != NDIS_STATUS_SUCCESS { status } else { wstatus }, written);
    }
    if done != 0 {
        if !ms_wait(addr_of_mut!((*a).m4_event), timeout_ms) {
            logw!("task %08x: no completion indication in %u ms\n", oid, timeout_ms);
            (*a).task_code.store(0, Ordering::Release);
            return (NDIS_STATUS_FAILURE, written);
        }
        let ws = if (*a).ind_len >= HDR { core::ptr::read_unaligned((*a).ind_buf as *const MsgHeader).Status } else { 0 };
        if ws != 0 {
            logw!("task %08x completed with WDI status %x\n", oid, ws);
            return (ws, written);
        }
    }
    (NDIS_STATUS_SUCCESS, written)
}

/// The body of the last M3 response (after its header).
unsafe fn response<'b>(a: *mut WdiAdapter, written: usize) -> &'b [u8] {
    if written <= HDR {
        return &[];
    }
    core::slice::from_raw_parts((*a).cmd_buf.add(HDR), written.min(CMD_BUF) - HDR)
}

/// The body of the last task's done indication.
unsafe fn task_payload<'b>(a: *mut WdiAdapter) -> &'b [u8] {
    if (*a).ind_len <= HDR {
        return &[];
    }
    core::slice::from_raw_parts((*a).ind_buf.add(HDR), (*a).ind_len - HDR)
}

unsafe fn get_capabilities(a: *mut WdiAdapter, cfg: &mut TxRxTargetConfiguration) -> NDIS_STATUS {
    let (s, n) = wdi_command(a, OID_WDI_GET_ADAPTER_CAPABILITIES, WDI_PORT_ANY, &[], 0, 0);
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }
    let caps = response(a, n);
    logw!("capabilities: %u bytes\n", n as u32);
    let b = core::slice::from_raw_parts((*a).cmd_buf, 32);
    logw!("  %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x %02x  %02x %02x %02x %02x %02x %02x %02x %02x\n",
        b[0] as u32, b[1] as u32, b[2] as u32, b[3] as u32, b[4] as u32, b[5] as u32, b[6] as u32, b[7] as u32,
        b[8] as u32, b[9] as u32, b[10] as u32, b[11] as u32, b[12] as u32, b[13] as u32, b[14] as u32, b[15] as u32,
        b[16] as u32, b[17] as u32, b[18] as u32, b[19] as u32, b[20] as u32, b[21] as u32, b[22] as u32, b[23] as u32);
    for (t, v) in tlvs(caps) {
        logw!("capability TLV %x, %u bytes\n", t as u32, v.len() as u32);
    }
    if let Some(ia) = find(caps, TLV_INTERFACE_ATTRIBUTES) {
        if let Some(ic) = find(ia, TLV_INTERFACE_CAPABILITIES) {
            // MTU u32, multicast list size u32, backfill u16, permanent MAC[6], ...
            (*a).mtu = le32(ic, 0);
            (*a).backfill = le16(ic, 8);
            if ic.len() >= 16 {
                (*a).mac.copy_from_slice(&ic[10..16]);
            }
            let m = (*a).mac;
            logw!("MTU %u, backfill %u, MAC %02x:%02x:%02x:%02x:%02x:%02x, radio hw %u sw %u\n",
                (*a).mtu, (*a).backfill as u32, m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32,
                ic.get(24).copied().unwrap_or(0xFF) as u32, ic.get(25).copied().unwrap_or(0xFF) as u32);
        }
        if let Some(fw) = find(ia, TLV_FIRMWARE_VERSION) {
            let mut s = [0u8; 64];
            let n = fw.len().min(63);
            s[..n].copy_from_slice(&fw[..n]);
            logw!("firmware %s\n", s.as_ptr());
        }
    }
    if let Some(da) = find(caps, TLV_DATAPATH_ATTRIBUTES) {
        if let Some(dc) = find(da, TLV_DATAPATH_CAPABILITIES) {
            // interconnect u32, max peers u8, target priority queueing u8, max SG u16,
            // explicit send complete u8, min effective size u16, granularity u16,
            // rx/tx forwarding u8, max throughput u32
            cfg.InterconnectType = le32(dc, 0);
            cfg.MaxNumPeers = dc.get(4).copied().unwrap_or(1);
            cfg.TargetPriorityQueueing = dc.get(5).copied().unwrap_or(0);
            cfg.MaxScatterGatherElementsPerFrame = le16(dc, 6);
            cfg.ExplicitSendCompleteFlagRequired = dc.get(8).copied().unwrap_or(0);
            cfg.MinEffectiveSize = le16(dc, 9);
            cfg.FrameSizeGranularity = le16(dc, 11);
            cfg.RxTxForwarding = dc.get(13).copied().unwrap_or(0);
            cfg.MaxThroughput = le32(dc, 14);
            logw!("datapath: interconnect %u, %u peers, port queueing %u, %u SG, explicit completes %u, throughput %u\n",
                cfg.InterconnectType, cfg.MaxNumPeers as u32, cfg.TargetPriorityQueueing as u32,
                cfg.MaxScatterGatherElementsPerFrame as u32, cfg.ExplicitSendCompleteFlagRequired as u32, cfg.MaxThroughput);
        }
    }
    if let Some(sa) = find(caps, TLV_STATION_ATTRIBUTES) {
        if let Some(algs) = find(sa, TLV_UNICAST_ALGORITHM_LIST) {
            for p in algs.chunks(2) {
                if p.len() == 2 {
                    logw!("station unicast auth %u cipher %u\n", p[0] as u32, p[1] as u32);
                }
            }
        }
    }
    NDIS_STATUS_SUCCESS
}

unsafe fn set_radio(a: *mut WdiAdapter, on: bool) -> NDIS_STATUS {
    let mut b = [0u8; 16];
    let mut w = TlvWriter::new(&mut b);
    w.tlv(TLV_RADIO_STATE_PARAMETERS, &[on as u8]);
    let n = w.len;
    wdi_command(a, OID_WDI_TASK_SET_RADIO_STATE, WDI_PORT_ANY, &b[..n], DONE_SET_RADIO_STATE, 5_000).0
}

unsafe fn create_port(a: *mut WdiAdapter) -> NDIS_STATUS {
    let mut b = [0u8; 32];
    let mut w = TlvWriter::new(&mut b);
    // operation-mode mask u16, NDIS port number u32 (0: the default port)
    let mut p = [0u8; 6];
    p[0..2].copy_from_slice(&WDI_OPERATION_MODE_STA.to_le_bytes());
    w.tlv(TLV_CREATE_PORT_PARAMETERS, &p);
    let n = w.len;
    let s = wdi_command(a, OID_WDI_TASK_CREATE_PORT, WDI_PORT_ANY, &b[..n], DONE_CREATE_PORT, 5_000).0;
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }
    let pl = task_payload(a);
    match find(pl, TLV_PORT_ATTRIBUTES) {
        Some(pa) if pa.len() >= 8 => {
            (*a).port_mac.copy_from_slice(&pa[0..6]);
            (*a).port_id = le16(pa, 6);
        }
        _ => {
            logw!("create port: no port attributes in the completion (%u bytes)\n", pl.len() as u32);
            return NDIS_STATUS_FAILURE;
        }
    }
    let m = (*a).port_mac;
    logw!("station port %x, MAC %02x:%02x:%02x:%02x:%02x:%02x\n", (*a).port_id as u32,
        m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32);
    NDIS_STATUS_SUCCESS
}

unsafe fn set_packet_filter(a: *mut WdiAdapter, filter: u32) -> NDIS_STATUS {
    let mut b = [0u8; 16];
    let mut w = TlvWriter::new(&mut b);
    w.tlv(TLV_PACKET_FILTER_PARAMETERS, &filter.to_le_bytes());
    let n = w.len;
    wdi_command(a, OID_WDI_SET_RECEIVE_PACKET_FILTER, (*a).port_id, &b[..n], 0, 0).0
}

/// One scan of every channel, probing for `ssid` (a hidden network answers only a probe
/// that names it) and for any network.
unsafe fn scan(a: *mut WdiAdapter, ssid: &[u8]) -> NDIS_STATUS {
    let b = alloc_zeroed(512) as *mut u8;
    if b.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    let buf = core::slice::from_raw_parts_mut(b, 512);
    let mut w = TlvWriter::new(buf);
    w.tlv(TLV_BSSID, &[0xFF; 6]);
    if !ssid.is_empty() {
        w.tlv(TLV_SSID, ssid);
    }
    w.tlv(TLV_SSID, &[]); // the wildcard
    // repeat count u8, WDI_SCAN_TYPE u32, live updates u8, WDI_SCAN_TRIGGER u32.
    // Scan type 1: an active scan whichever way the enum counts (ACTIVE_ONLY is 1 if it
    // follows DOT11_SCAN_TYPE, and AUTO only exists past it).
    let mut mode = [0u8; 10];
    mode[0] = 1;
    mode[1..5].copy_from_slice(&1u32.to_le_bytes());
    mode[5] = 1;
    mode[6..10].copy_from_slice(&0u32.to_le_bytes()); // WDI_SCAN_TRIGGER_MANUAL
    w.tlv(TLV_SCAN_MODE, &mode);
    // active dwell, passive dwell, total time (ms)
    let mut dwell = [0u8; 12];
    dwell[0..4].copy_from_slice(&40u32.to_le_bytes());
    dwell[4..8].copy_from_slice(&110u32.to_le_bytes());
    dwell[8..12].copy_from_slice(&8000u32.to_le_bytes());
    w.tlv(TLV_SCAN_DWELL_TIME, &dwell);
    let n = w.len;
    let s = wdi_command(a, OID_WDI_TASK_SCAN, (*a).port_id, &buf[..n], DONE_SCAN, 20_000).0;
    ExFreePool(b as PVOID);
    s
}

/// Record one WDI_TLV_BSS_ENTRY. Runs at the indication's IRQL.
unsafe fn note_bss(a: *mut WdiAdapter, entry: &[u8]) {
    let mut b = Bss { bssid: [0; 6], ssid: [0; 32], ssid_len: 0, rssi: 0, channel: 0, band: 0 };
    match find(entry, TLV_BSSID) {
        Some(v) if v.len() >= 6 => b.bssid.copy_from_slice(&v[..6]),
        _ => return,
    }
    if let Some(v) = find(entry, TLV_BSS_ENTRY_SIGNAL_INFO) {
        b.rssi = le32(v, 0) as i32;
    }
    if let Some(v) = find(entry, TLV_BSS_ENTRY_CHANNEL_INFO) {
        b.channel = le32(v, 0);
        b.band = le32(v, 4);
    }
    // The SSID element of the probe response (or beacon): the elements follow the 12 fixed
    // bytes (timestamp, interval, capabilities), and the 24-byte MAC header when the frame
    // carries one (frame control 0x50 or 0x80: probe response, beacon). A hidden network's
    // beacon carries an empty SSID.
    for t in [TLV_PROBE_RESPONSE_FRAME, TLV_BEACON_FRAME] {
        if let Some(f) = find(entry, t) {
            let has_header = f.len() >= 36 && (f[0] == 0x50 || f[0] == 0x80) && f[1] == 0;
            let mut i = if has_header { 36 } else { 12 };
            while i + 2 <= f.len() {
                let (id, len) = (f[i], f[i + 1] as usize);
                if i + 2 + len > f.len() {
                    break;
                }
                if id == 0 && len > 0 && len <= 32 && f[i + 2] != 0 {
                    b.ssid[..len].copy_from_slice(&f[i + 2..i + 2 + len]);
                    b.ssid_len = len as u8;
                    break;
                }
                i += 2 + len;
            }
            if b.ssid_len != 0 {
                break;
            }
        }
    }
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).bss_lock), &mut irql);
    let n = (*a).bss_count;
    let slot = (0..n).find(|&i| (*a).bss[i].bssid == b.bssid);
    match slot {
        Some(i) => {
            // A beacon may carry no SSID where the probe response did: keep the one known.
            if b.ssid_len == 0 {
                b.ssid = (*a).bss[i].ssid;
                b.ssid_len = (*a).bss[i].ssid_len;
            }
            (*a).bss[i] = b;
        }
        None if n < MAX_BSS => {
            (*a).bss[n] = b;
            (*a).bss_count = n + 1;
        }
        None => {}
    }
    KeReleaseSpinLock(addr_of_mut!((*a).bss_lock), irql);
}

// ---- bring-up ------------------------------------------------------------------------------

unsafe extern "system" fn open_adapter_complete(handle: NDIS_HANDLE, status: NDIS_STATUS) {
    let a = adapter_of(handle);
    if !a.is_null() {
        (*a).open_status = status;
        KeSetEvent(addr_of_mut!((*a).open_event), 0, FALSE as BOOLEAN);
    }
}
unsafe extern "system" fn close_adapter_complete(handle: NDIS_HANDLE, status: NDIS_STATUS) {
    open_adapter_complete(handle, status);
}
unsafe extern "system" fn idle_confirm(_handle: NDIS_HANDLE, _state: u32) {}
unsafe extern "system" fn idle_complete(_handle: NDIS_HANDLE) {}

unsafe fn remove(a: *mut WdiAdapter) {
    for slot in ADAPTERS.iter() {
        let _ = slot.compare_exchange(a, null_mut(), Ordering::AcqRel, Ordering::Acquire);
    }
}

unsafe extern "system" fn ue_initialize(handle: NDIS_HANDLE, driver_context: NDIS_HANDLE, params: *mut MiniportInitParameters) -> NDIS_STATUS {
    let drv = driver_context as *mut WdiDriver;
    let a = alloc_zeroed(core::mem::size_of::<WdiAdapter>()) as *mut WdiAdapter;
    if a.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*a).adapter = handle;
    (*a).drv = drv;
    (*a).cmd_buf = alloc_zeroed(CMD_BUF) as *mut u8;
    (*a).ind_buf = alloc_zeroed(IND_BUF) as *mut u8;
    if (*a).cmd_buf.is_null() || (*a).ind_buf.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    KeInitializeEvent(addr_of_mut!((*a).open_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).m3_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).m4_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).cmd_lock), _EVENT_TYPE_SynchronizationEvent, TRUE as BOOLEAN);
    KeInitializeSpinLock(addr_of_mut!((*a).bss_lock));
    if !ADAPTERS.iter().any(|s| s.compare_exchange(null_mut(), a, Ordering::AcqRel, Ordering::Acquire).is_ok()) {
        return NDIS_STATUS_RESOURCES;
    }
    let status = bring_up(a, params);
    if status != NDIS_STATUS_SUCCESS {
        logw!("bring-up failed: %x\n", status);
        remove(a);
        return status;
    }
    let mut thread: HANDLE = null_mut();
    let s = ntkrnl_sys::ddk::PsCreateSystemThread(&mut thread as *mut HANDLE as _, 0x1F_FFFF, null_mut(), null_mut(), null_mut(),
        Some(core::mem::transmute(worker as unsafe extern "system" fn(PVOID))), a as PVOID);
    if s == STATUS_SUCCESS {
        ntkrnl_sys::ddk::ZwClose(thread as _);
    } else {
        logw!("no worker thread: %x\n", s);
    }
    NDIS_STATUS_SUCCESS
}

unsafe fn bring_up(a: *mut WdiAdapter, params: *mut MiniportInitParameters) -> NDIS_STATUS {
    let drv = (*a).drv;
    let w = &(*drv).wdi;

    // AllocateAdapter: the LE's software state; it reports its context in the registration
    // attributes rather than setting them.
    (*a).init_params = WdiInitParameters {
        Header: NdisObjectHeader { Type: NDIS_OBJECT_TYPE_WDI_INIT_PARAMETERS, Revision: 1, Size: core::mem::size_of::<WdiInitParameters>() as u16 },
        WdiVersion: UE_WDI_VERSION,
        OpenAdapterComplete: open_adapter_complete,
        CloseAdapterComplete: close_adapter_complete,
        IdleNotificationConfirm: idle_confirm,
        IdleNotificationComplete: idle_complete,
    };
    let mut reg: RegistrationAttributes = core::mem::zeroed();
    reg.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES, Revision: 1, Size: core::mem::size_of::<RegistrationAttributes>() as u16 };
    logw!("AllocateAdapter\n");
    let s = (w.AllocateAdapter.unwrap_unchecked())((*a).adapter, (*drv).ihv_driver_context, params, addr_of_mut!((*a).init_params), &mut reg);
    if s != NDIS_STATUS_SUCCESS {
        logw!("AllocateAdapter failed %x\n", s);
        return s;
    }
    (*a).ihv_ctx = reg.MiniportAdapterContext;

    // The UE is the adapter NDIS dispatches to.
    let mut ours: RegistrationAttributes = core::mem::zeroed();
    ours.Header = reg.Header;
    ours.MiniportAdapterContext = a as NDIS_HANDLE;
    ours.AttributeFlags = reg.AttributeFlags;
    ours.InterfaceType = reg.InterfaceType;
    (*a).own_attrs = true;
    NdisMSetMiniportAttributes((*a).adapter, addr_of_mut!(ours) as *mut NdisObjectHeader);
    (*a).own_attrs = false;

    // OpenAdapter: firmware download and hardware start, completed by callback.
    logw!("OpenAdapter\n");
    KeClearEvent(addr_of_mut!((*a).open_event));
    let s = (w.OpenAdapter.unwrap_unchecked())((*a).ihv_ctx, params);
    if s != NDIS_STATUS_SUCCESS && s != NDIS_STATUS_PENDING {
        logw!("OpenAdapter failed %x\n", s);
        return s;
    }
    if !ms_wait(addr_of_mut!((*a).open_event), 30_000) {
        logw!("OpenAdapter: no completion in 30 s\n");
        return NDIS_STATUS_FAILURE;
    }
    if (*a).open_status != NDIS_STATUS_SUCCESS {
        logw!("OpenAdapter completed with %x\n", (*a).open_status);
        return (*a).open_status;
    }

    // The data path: exchange entry points.
    (*a).data_api = WdiDataApi {
        Header: NdisObjectHeader { Type: NDIS_OBJECT_TYPE_WDI_DATA_API, Revision: 1, Size: core::mem::size_of::<WdiDataApi>() as u16 },
        TxDequeueIndication: tx_dequeue,
        TxTransferCompleteIndication: tx_transfer_complete,
        TxSendCompleteIndication: tx_send_complete,
        TxQueryRaTidState: tx_query_ra_tid,
        TxSendPauseIndication: tx_pause,
        TxSendRestartIndication: tx_restart,
        TxReleaseFrameIndication: tx_release,
        TxInjectFrameIndication: tx_inject,
        TxAbortConfirm: datapath_confirm,
        RxInorderDataIndication: rx_inorder,
        RxStopConfirm: datapath_confirm,
        RxFlushConfirm: datapath_confirm,
        PeerCreateIndication: peer_create,
        PeerDeleteIndication: peer_delete,
        AllocateWiFiFrameMetaData: alloc_metadata,
        FreeWiFiFrameMetaData: free_metadata,
        TxQuerySuspectFrameCompleteStatus: tx_suspect,
    };
    (*a).data.Header = NdisObjectHeader {
        Type: NDIS_OBJECT_TYPE_WDI_DATA_HANDLERS,
        Revision: 1,
        Size: (core::mem::size_of::<WdiDataHandlers>() - core::mem::size_of::<[PVOID; 7]>()) as u16,
    };
    logw!("TalTxRxInitialize\n");
    let s = (w.TalTxRxInitialize.unwrap_unchecked())((*a).ihv_ctx, a as NDIS_HANDLE, addr_of_mut!((*a).data_api),
        addr_of_mut!((*a).tal), addr_of_mut!((*a).data), addr_of_mut!((*a).metadata_extra));
    if s != NDIS_STATUS_SUCCESS {
        logw!("TalTxRxInitialize failed %x\n", s);
        return s;
    }
    logw!("data path: handlers revision %u, %u bytes of metadata space\n", (*a).data.Header.Revision as u32, (*a).metadata_extra);

    let mut cfg = TxRxTargetConfiguration::default();
    let s = get_capabilities(a, &mut cfg);
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }
    let s = set_radio(a, true);
    if s != NDIS_STATUS_SUCCESS {
        logw!("radio on failed %x; carrying on\n", s);
    }
    cfg.MaxNumPorts = 1;
    if let Some(start) = (*a).data.TalTxRxStart {
        let mut max_transfers: u16 = 0;
        let s = start((*a).tal, &mut cfg, &mut max_transfers);
        if s != NDIS_STATUS_SUCCESS {
            logw!("TalTxRxStart failed %x\n", s);
            return s;
        }
        logw!("data path started, %u outstanding transfers\n", max_transfers as u32);
    }
    let s = create_port(a);
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }
    if let Some(add) = (*a).data.TalTxRxAddPort {
        add((*a).tal, (*a).port_id, 0 /* WDI_OPERATION_MODE_STA in the TAL's enum */);
    }
    let s = set_packet_filter(a, WDI_PACKET_FILTER_DIRECTED | WDI_PACKET_FILTER_MULTICAST | WDI_PACKET_FILTER_BROADCAST);
    if s != NDIS_STATUS_SUCCESS {
        logw!("packet filter failed %x; carrying on\n", s);
    }
    if let Some(startop) = w.StartOperation {
        let s = startop((*a).ihv_ctx);
        if s != NDIS_STATUS_SUCCESS {
            logw!("StartOperation failed %x\n", s);
            return s;
        }
    }

    // To NDIS: an Ethernet adapter, cable unplugged until the station associates.
    let mut gen: GeneralAttributes = core::mem::zeroed();
    gen.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES, Revision: 1, Size: core::mem::size_of::<GeneralAttributes>() as u16 };
    gen.MediaType = 0; // NdisMedium802_3
    gen.PhysicalMediumType = 9; // NdisPhysicalMediumNative802_11
    gen.MtuSize = if (*a).mtu != 0 { (*a).mtu.min(1500) } else { 1500 };
    gen.MaxXmitLinkSpeed = 433_000_000;
    gen.MaxRcvLinkSpeed = 433_000_000;
    gen.MediaConnectState = 2; // MediaConnectStateDisconnected
    gen.MediaDuplexState = 2; // MediaDuplexStateFull
    gen.LookaheadSize = gen.MtuSize;
    gen.MacOptions = 0x1 | 0x2 | 0x8; // copy lookahead, receive serialized, no loopback
    gen.SupportedPacketFilters = 0x1 | 0x2 | 0x4 | 0x8;
    gen.MaxMulticastListSize = 32;
    gen.MacAddressLength = 6;
    gen.PermanentMacAddress[..6].copy_from_slice(&(*a).mac);
    gen.CurrentMacAddress[..6].copy_from_slice(&(*a).port_mac);
    (*a).own_attrs = true;
    NdisMSetMiniportAttributes((*a).adapter, addr_of_mut!(gen) as *mut NdisObjectHeader);
    (*a).own_attrs = false;
    (*a).up = true;
    logw!("adapter up\n");
    NDIS_STATUS_SUCCESS
}

// ---- configuration and the worker ----------------------------------------------------------

/// The bench's network: `ssid=` from \SystemRoot\bin\wifi.txt (staged by vendordrv).
#[derive(Clone, Copy)]
struct WifiConfig {
    ssid: [u8; 32],
    ssid_len: usize,
}

unsafe fn read_config() -> Option<WifiConfig> {
    let mut name: [u16; 32] = [0; 32];
    for (i, c) in "\\SystemRoot\\bin\\wifi.txt".encode_utf16().enumerate() {
        name[i] = c;
    }
    let len = name.iter().position(|&c| c == 0).unwrap_or(0);
    let mut us: UNICODE_STRING = core::mem::zeroed();
    us.Length = (len * 2) as u16;
    us.MaximumLength = (len * 2 + 2) as u16;
    us.Buffer = name.as_mut_ptr();
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
    oa.ObjectName = &mut us;
    oa.Attributes = 0x40 | 0x200; // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
    let mut iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut h: HANDLE = null_mut();
    let s = ntkrnl_sys::ddk::ZwCreateFile(&mut h as *mut HANDLE as _, 0x8012_0089 /* GENERIC_READ | SYNCHRONIZE */,
        &mut oa as *mut OBJECT_ATTRIBUTES as _, &mut iosb as *mut IO_STATUS_BLOCK as _, null_mut(), 0x80, 0x1 /* FILE_SHARE_READ */,
        0x1 /* FILE_OPEN */, 0x20 | 0x40 /* FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE */, null_mut(), 0);
    if s != STATUS_SUCCESS {
        logw!("no \\SystemRoot\\bin\\wifi.txt (%x): no network to look for\n", s);
        return None;
    }
    let mut text = [0u8; 512];
    let s = ntkrnl_sys::ddk::ZwReadFile(h as _, null_mut(), None, null_mut(), &mut iosb as *mut IO_STATUS_BLOCK as _,
        text.as_mut_ptr() as PVOID, text.len() as u32, null_mut(), null_mut());
    ntkrnl_sys::ddk::ZwClose(h as _);
    if s != STATUS_SUCCESS {
        return None;
    }
    let n = (iosb.Information as usize).min(text.len());
    let mut cfg = WifiConfig { ssid: [0; 32], ssid_len: 0 };
    for line in text[..n].split(|&c| c == b'\n') {
        let line = match line.last() {
            Some(b'\r') => &line[..line.len() - 1],
            _ => line,
        };
        if let Some(v) = line.strip_prefix(b"ssid=") {
            let k = v.len().min(32);
            cfg.ssid[..k].copy_from_slice(&v[..k]);
            cfg.ssid_len = k;
        }
    }
    if cfg.ssid_len == 0 {
        logw!("wifi.txt names no ssid\n");
        return None;
    }
    Some(cfg)
}

unsafe extern "system" fn worker(context: PVOID) {
    let a = context as *mut WdiAdapter;
    sleep_ms(2000);
    let cfg = read_config();
    let ssid: &[u8] = match &cfg {
        Some(c) => &c.ssid[..c.ssid_len],
        None => &[],
    };
    for round in 1..=3u32 {
        let mut irql: KIRQL = 0;
        KeAcquireSpinLock(addr_of_mut!((*a).bss_lock), &mut irql);
        (*a).bss_count = 0;
        KeReleaseSpinLock(addr_of_mut!((*a).bss_lock), irql);
        logw!("scan %u\n", round);
        let s = scan(a, ssid);
        let n = (*a).bss_count;
        logw!("scan %u: status %x, %u networks\n", round, s, n as u32);
        let mut found = false;
        for i in 0..n {
            let b = (*a).bss[i];
            let mut name = [0u8; 33];
            name[..b.ssid_len as usize].copy_from_slice(&b.ssid[..b.ssid_len as usize]);
            let m = b.bssid;
            logw!("  %02x:%02x:%02x:%02x:%02x:%02x ch %3u %d dBm \"%s\"\n",
                m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32,
                b.channel, b.rssi, name.as_ptr());
            if !ssid.is_empty() && &b.ssid[..b.ssid_len as usize] == ssid {
                found = true;
            }
        }
        if found {
            logw!("found the configured network\n");
            break;
        }
        sleep_ms(3000);
    }
    logw!("worker done\n");
    ntkrnl_sys::ddk::PsTerminateSystemThread(STATUS_SUCCESS);
}

// ---- the NDIS 6 side -----------------------------------------------------------------------

unsafe extern "system" fn ue_halt(ctx: NDIS_HANDLE, _action: u32) {
    let a = ctx as *mut WdiAdapter;
    let drv = (*a).drv;
    let w = &(*drv).wdi;
    if let Some(stop) = w.StopOperation {
        stop((*a).ihv_ctx);
    }
    if let Some(del) = (*a).data.TalTxRxDeletePort {
        del((*a).tal, (*a).port_id);
    }
    if let Some(stop) = (*a).data.TalTxRxStop {
        stop((*a).tal);
    }
    if let Some(deinit) = w.TalTxRxDeinitialize {
        deinit((*a).tal);
    }
    if let Some(close) = w.CloseAdapter {
        KeClearEvent(addr_of_mut!((*a).open_event));
        if close((*a).ihv_ctx) == NDIS_STATUS_SUCCESS {
            ms_wait(addr_of_mut!((*a).open_event), 10_000);
        }
    }
    if let Some(free) = w.FreeAdapter {
        free((*a).ihv_ctx);
    }
    remove(a);
}

unsafe extern "system" fn ue_pause(ctx: NDIS_HANDLE, params: PVOID) -> NDIS_STATUS {
    let a = ctx as *mut WdiAdapter;
    match (*(*a).drv).wdi.PostPause {
        Some(f) => f((*a).ihv_ctx, params),
        None => NDIS_STATUS_SUCCESS,
    }
}

unsafe extern "system" fn ue_restart(ctx: NDIS_HANDLE, params: *mut crate::ndis::ndis6::MiniportRestartParameters) -> NDIS_STATUS {
    let a = ctx as *mut WdiAdapter;
    match (*(*a).drv).wdi.PostRestart {
        Some(f) => f((*a).ihv_ctx, params as PVOID),
        None => NDIS_STATUS_SUCCESS,
    }
}

unsafe extern "system" fn ue_shutdown(ctx: NDIS_HANDLE, action: u32) {
    let a = ctx as *mut WdiAdapter;
    if let Some(f) = (*(*a).drv).ihv.ShutdownHandlerEx {
        f((*a).ihv_ctx, action);
    }
}

/// OIDs from the protocols. The UE answers what it owns; the LE has no OIDs of its own here.
unsafe extern "system" fn ue_oid_request(_ctx: NDIS_HANDLE, req: *mut NdisOidRequest) -> NDIS_STATUS {
    const OID_GEN_CURRENT_PACKET_FILTER: u32 = 0x0001_010E;
    const OID_802_3_MULTICAST_LIST: u32 = 0x0101_0103;
    let q = &mut (*req).DATA.q;
    match ((*req).RequestType, q.Oid) {
        (1, OID_GEN_CURRENT_PACKET_FILTER) | (1, OID_802_3_MULTICAST_LIST) => {
            q.BytesWritten = q.InformationBufferLength;
            NDIS_STATUS_SUCCESS
        }
        _ => NDIS_STATUS_NOT_SUPPORTED,
    }
}

/// Sends: no association yet, so nothing can go out.
unsafe extern "system" fn ue_send(ctx: NDIS_HANDLE, nbls: *mut NetBufferList, _port: u32, _flags: u32) {
    let a = ctx as *mut WdiAdapter;
    let mut nbl = nbls;
    while !nbl.is_null() {
        (*nbl).Status = NDIS_STATUS_MEDIA_DISCONNECTED;
        nbl = (*nbl).u.h.Next;
    }
    NdisMSendNetBufferListsComplete((*a).adapter, nbls, 0);
}

unsafe extern "system" fn ue_return(_ctx: NDIS_HANDLE, _nbls: *mut NetBufferList, _flags: u32) {}

// ---- the data path (stage A: receive is drained, nothing is sent) ---------------------------

unsafe extern "system" fn tx_dequeue(_h: PVOID, _quantum: u32, _max: u8, _credit: u16, out: *mut *mut NetBufferList) {
    *out = null_mut();
}
unsafe extern "system" fn tx_transfer_complete(_h: PVOID, _status: u32, _nbl: *mut NetBufferList) {}
unsafe extern "system" fn tx_send_complete(_h: PVOID, _status: u32, _n: u16, _ids: *mut u16, _data: PVOID) {}
unsafe extern "system" fn tx_query_ra_tid(_h: PVOID, _port: u16, _peer: u16, _tid: u8, status: *mut NDIS_STATUS, len: *mut u16) {
    *status = NDIS_STATUS_SUCCESS;
    *len = 0;
}
unsafe extern "system" fn tx_pause(_h: PVOID, _port: u16, _peer: u16, _tids: u32, _reason: u32) {}
unsafe extern "system" fn tx_restart(_h: PVOID, _port: u16, _peer: u16, _tids: u32, _reason: u32) {}
unsafe extern "system" fn tx_release(_h: PVOID, _port: u16, _peer: u16, _tids: u32, _max: u8, _credit: u16, out: *mut *mut NetBufferList) {
    *out = null_mut();
}
unsafe extern "system" fn tx_inject(_h: PVOID, _port: u16, _peer: u16, _tid: u8, _nbl: *mut NetBufferList, _uni: u8, _legacy: u8, ethertype: u16, _ex: u32) {
    logw!("the driver injected a frame (ethertype %04x); not sent yet\n", ethertype as u32);
}
unsafe extern "system" fn datapath_confirm(_h: PVOID) {}
unsafe extern "system" fn tx_suspect(_h: PVOID, _ctx: u64, _nbl: *mut NetBufferList, xfer: *mut u8, send: *mut u8, status: *mut NDIS_STATUS) {
    *xfer = 0;
    *send = 0;
    *status = NDIS_STATUS_SUCCESS;
}

/// Frames the LE has ready: take them and hand them straight back (nothing consumes them yet).
unsafe extern "system" fn rx_inorder(h: PVOID, _level: u32, peer: u16, tid: u8, _throttle: PVOID, status: *mut NDIS_STATUS) {
    let a = h as *mut WdiAdapter;
    *status = NDIS_STATUS_SUCCESS;
    if let (Some(get), Some(ret)) = ((*a).data.RxGetMpdus, (*a).data.RxReturnFrames) {
        let mut nbl: *mut NetBufferList = null_mut();
        get((*a).tal, peer, tid, &mut nbl);
        if !nbl.is_null() {
            ret((*a).tal, nbl);
        }
    }
}

unsafe extern "system" fn peer_create(_h: PVOID, port: u16, peer: u16, mac: WdiMac) {
    let m = mac.0;
    logw!("peer %u on port %u: %02x:%02x:%02x:%02x:%02x:%02x\n", peer as u32, port as u32,
        m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32);
}
unsafe extern "system" fn peer_delete(_h: PVOID, port: u16, peer: u16, status: *mut NDIS_STATUS) {
    logw!("peer %u on port %u deleted\n", peer as u32, port as u32);
    *status = NDIS_STATUS_SUCCESS;
}

/// WDI_FRAME_METADATA (32 bytes of linkage and frame ID, then the 64-byte TX/RX union) plus
/// the space the LE asked for.
const FRAME_METADATA: usize = 96;

unsafe extern "system" fn alloc_metadata(h: PVOID) -> PVOID {
    let a = h as *mut WdiAdapter;
    alloc_zeroed(FRAME_METADATA + (*a).metadata_extra as usize)
}
unsafe extern "system" fn free_metadata(_h: PVOID, md: PVOID) {
    if !md.is_null() {
        ExFreePool(md);
    }
}
