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
//! The station: bring the adapter up, create the station port, scan for the network named in
//! \SystemRoot\bin\wifi.txt, associate, run the WPA2-Personal handshake (wpa.rs) over the
//! data path, install the keys, and then carry Ethernet frames as 802.11 data frames.

use crate::prelude::*;
use crate::ndis::ndis6::{
    MiniportDriverCharacteristics, MiniportInitParameters, NdisMRegisterMiniportDriver,
    NdisMSetMiniportAttributes, NdisMDeregisterMiniportDriver, NdisMSendNetBufferListsComplete,
    NdisObjectHeader, NdisOidRequest, RegistrationAttributes, GeneralAttributes, StatusIndication,
    NetBufferList, LinkState, NblPoolParameters, NdisAllocateNetBufferListPool,
    NdisAllocateNetBufferAndNetBufferList, NdisFreeNetBufferList, NdisMIndicateReceiveNetBufferLists,
    copy_nb, indicate_link_state, MEDIA_CONNECT_STATE_CONNECTED, NDIS_RECEIVE_FLAGS_RESOURCES,
    NDIS_OBJECT_TYPE_DEFAULT, NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES,
    NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES, NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS,
    NDIS_OBJECT_TYPE_OID_REQUEST, NDIS_STATUS_MEDIA_DISCONNECTED,
};
use crate::ndis::wpa;
use ntkrnl_sys::ddk::{ExFreePool, IoAllocateMdl, IoFreeMdl, KeInitializeEvent, KeWaitForSingleObject, MmBuildMdlForNonPagedPool};
use core::ptr::{null_mut, addr_of_mut};
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering};

macro_rules! logw {
    ($fmt:literal $(, $a:expr)* $(,)?) => {
        { ntkrnl_sys::driver_ddk::DbgPrint(concat!("[WDI] ", $fmt, "\0").as_ptr() as *const i8 $(, $a)*); }
    };
}

const NDIS_STATUS_INVALID_LENGTH: NDIS_STATUS = 0xC001_0014u32 as NDIS_STATUS;

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
    pub TxTargetDescInit: Option<unsafe extern "system" fn(PVOID, *mut NetBufferList, *mut NDIS_STATUS)>,
    pub TxTargetDescDeInit: Option<unsafe extern "system" fn(PVOID, *mut NetBufferList)>,
    pub TxDataSend: Option<unsafe extern "system" fn(PVOID, u16, u16, u8, u16, u32, u8)>,
    pub TxTalSend: PVOID,
    pub TxTalSendComplete: Option<unsafe extern "system" fn(PVOID, *mut NetBufferList, u32)>,
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
    pub TalTxRxPeerConfig: Option<unsafe extern "system" fn(PVOID, u16, u16, *mut u32)>,
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
const OID_WDI_TASK_CONNECT: u32 = 0xE440_0006;
const OID_WDI_TASK_DISCONNECT: u32 = 0xE440_0008;
const OID_WDI_TASK_SET_RADIO_STATE: u32 = 0xE440_000B;
const OID_WDI_TASK_CREATE_PORT: u32 = 0xE440_000C;
const OID_WDI_SET_ADD_CIPHER_KEYS: u32 = 0xE440_001D;
const OID_WDI_GET_ADAPTER_CAPABILITIES: u32 = 0xE440_0025;
const OID_WDI_SET_RECEIVE_PACKET_FILTER: u32 = 0xE440_0024;

const DONE_SCAN: NDIS_STATUS = 0x4005_003C;
const DONE_CONNECT: NDIS_STATUS = 0x4005_0040;
const DONE_DISCONNECT: NDIS_STATUS = 0x4005_0015;
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
const TLV_ASSOCIATION_RESULT_PARAMETERS: u16 = 0x2D;
const TLV_ASSOCIATION_REQUEST_FRAME: u16 = 0x2E;
const TLV_CONNECT_PARAMETERS: u16 = 0x33;
const TLV_DISCONNECT_PARAMETERS: u16 = 0x36;
const TLV_CONNECT_BSS_ENTRY: u16 = 0x34;
const TLV_ASSOCIATION_RESULT: u16 = 0x35;
const TLV_AUTH_ALGO_LIST: u16 = 0x3C;
const TLV_MULTICAST_CIPHER_ALGO_LIST: u16 = 0x3D;
const TLV_UNICAST_CIPHER_ALGO_LIST: u16 = 0x3E;
const TLV_CONNECTION_SETTINGS: u16 = 0x3F;
const TLV_PEER_MAC_ADDRESS: u16 = 0x4C;
const TLV_CIPHER_KEY_ID: u16 = 0x4D;
const TLV_CIPHER_KEY_TYPE_INFO: u16 = 0x4E;
const TLV_CIPHER_KEY_RECEIVE_SEQUENCE_COUNT: u16 = 0x4F;
const TLV_CIPHER_KEY_CCMP_KEY: u16 = 0x50;
const TLV_SET_CIPHER_KEY_INFO: u16 = 0x52;
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
// WDI_AUTH_ALGORITHM and WDI_CIPHER_ALGORITHM keep the DOT11_* values: the driver's own
// capability list pairs RSNA_PSK (7) with CCMP (4) and the WEP suites with 0x101.
const WDI_AUTH_ALGO_RSNA_PSK: u32 = 7;
const WDI_CIPHER_ALGO_TKIP: u32 = 2;
const WDI_CIPHER_ALGO_CCMP: u32 = 4;
const WDI_CIPHER_KEY_TYPE_PAIRWISE_KEY: u32 = 1;
const WDI_CIPHER_KEY_TYPE_GROUP_KEY: u32 = 2;
// WDI_CIPHER_KEY_DIRECTION, in its documented order.
const WDI_CIPHER_KEY_DIRECTION_INBOUND: u32 = 0;
const WDI_CIPHER_KEY_DIRECTION_BOTH: u32 = 2;
const WDI_TXRX_PEER_CFG_QOS_CAPABLE: u32 = 1;
const WDI_TX_FRAME_STATUS_OK: u32 = 0;

const ETHERTYPE_EAPOL: u16 = 0x888E;
/// RFC 1042 LLC/SNAP, which carries an Ethertype in an 802.11 data frame. The bridge-tunnel
/// OUI (00-00-F8) is the same on receive.
const SNAP: [u8; 6] = [0xAA, 0xAA, 0x03, 0x00, 0x00, 0x00];
/// A station's data frame header (no QoS: the target adds it) and the LLC/SNAP after it.
const WLAN_HDR: usize = 24;
const WLAN_SNAP: usize = 8;
const ETH_HDR: usize = 14;
const ETH_MAX: usize = 1514;

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
/// The configured network's scan entry, kept for the connect.
const TARGET_BUF: usize = 4 * 1024;
const EAPOL_SLOTS: usize = 4;
const EAPOL_MAX: usize = 512;
/// The largest 802.11 MSDU the receive path copies out.
const RX_BUF: usize = 2400;

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

    /// The network to join (wifi.txt's ssid=) and its scan entry, the body of the
    /// WDI_TLV_BSS_ENTRY the scan reported, which the connect hands back.
    want_ssid: [u8; 32],
    want_ssid_len: usize,
    target: *mut u8,
    target_len: usize,

    /// The association, from WDI_TLV_ASSOCIATION_RESULT: its status, and our RSN element as
    /// the association request carried it.
    assoc_seen: bool,
    assoc_status: u32,
    rsn: [u8; 64],
    rsn_len: usize,
    /// The AP: its BSSID and the peer ID the LE gave it.
    bssid: [u8; 6],
    peer_id: u16,
    /// Keys are installed and the link is up to the protocols.
    connected: bool,

    /// Halt is under way: the worker leaves at its next wait, and sets `worker_done` as it
    /// goes. Halt waits for that before tearing down what the worker uses.
    stopping: AtomicBool,
    stop_event: KEVENT,
    worker_done: KEVENT,

    /// EAPOL frames from the receive path, for the worker's supplicant.
    eapol_lock: KSPIN_LOCK,
    eapol: [[u8; EAPOL_MAX]; EAPOL_SLOTS],
    eapol_len: [usize; EAPOL_SLOTS],
    eapol_head: usize,
    eapol_count: usize,
    eapol_event: KEVENT,
    /// EAPOL frames handed to the LE and not yet transferred, and the event set when that
    /// count reaches zero: message 4 must leave before the PTK that would encrypt it.
    eapol_in_flight: AtomicU32,
    eapol_sent: KEVENT,

    /// Transmit: frames waiting for the LE to dequeue them (port queueing: WDI never sees
    /// peers or TIDs, the LE pulls from one queue).
    tx_lock: KSPIN_LOCK,
    tx_head: *mut NetBufferList,
    tx_tail: *mut NetBufferList,
    tx_queued: u32,
    tx_paused: bool,
    tx_pool: NDIS_HANDLE,
    frame_id: u16,

    /// Receive: an 802.11 MSDU copied out, and the 802.3 frame indicated up from it.
    rx_lock: KSPIN_LOCK,
    rx_buf: *mut u8,
    rx_eth: *mut u8,
    rx_mdl: PMDL,
    rx_nbl: *mut NetBufferList,
    rx_logged: u32,
    /// The send context for a frame the LE injects without one.
    inject_ctx: SendContext,
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

    // A task's completion echoes its command's transaction ID and carries the completion
    // code the LE's command table pairs with the task. Indications the task provokes along
    // the way (a connect's association result) echo the ID too, so both must match.
    let want = (*a).task_code.load(Ordering::Acquire);
    if want != 0 && txid != 0 && txid == (*a).task_txid && s.StatusCode as u32 == want {
        logw!("task %u done: indication %08x, WDI status %x, %u bytes\n", txid, s.StatusCode,
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
        } else if t == TLV_ASSOCIATION_RESULT {
            note_association(a, v);
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

/// The group cipher the network advertises: its RSN element's group data cipher suite, from
/// the probe response (or beacon) in the scan entry. CCMP when it cannot be read.
fn group_cipher(entry: &[u8]) -> u32 {
    for t in [TLV_PROBE_RESPONSE_FRAME, TLV_BEACON_FRAME] {
        let Some(f) = find(entry, t) else { continue };
        let has_header = f.len() >= 36 && (f[0] == 0x50 || f[0] == 0x80) && f[1] == 0;
        let ies = f.get(if has_header { 36 } else { 12 }..).unwrap_or(&[]);
        // 30 len, version (2), group cipher suite 00-0F-AC-n.
        if let Some(rsn) = find_element(ies, 0x30) {
            if rsn.len() >= 8 && rsn[4..7] == [0x00, 0x0F, 0xAC] {
                return match rsn[7] {
                    2 => WDI_CIPHER_ALGO_TKIP,
                    _ => WDI_CIPHER_ALGO_CCMP,
                };
            }
        }
    }
    WDI_CIPHER_ALGO_CCMP
}

/// Associate with the scanned network: WPA2-Personal, CCMP pairwise. The task completes once
/// the 802.11 association does; the 4-way handshake is the UE's to run afterwards.
unsafe fn connect(a: *mut WdiAdapter, hidden: bool) -> NDIS_STATUS {
    if (*a).target_len == 0 {
        return NDIS_STATUS_FAILURE;
    }
    let b = alloc_zeroed(TARGET_BUF + 256) as *mut u8;
    if b.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    let buf = core::slice::from_raw_parts_mut(b, TARGET_BUF + 256);
    let entry = core::slice::from_raw_parts((*a).target, (*a).target_len);
    let group = group_cipher(entry);
    let mut w = TlvWriter::new(buf);
    let params = w.open(TLV_CONNECT_PARAMETERS);
    // roam request, hidden network, exclude unencrypted, MFP, host FIPS, roam-needed reason
    // (u32), roam trigger (u32), BSS transition supported. Unencrypted frames are not
    // excluded: the handshake's own frames are unencrypted, and no privacy exemption is
    // set for them.
    let mut settings = [0u8; 14];
    settings[1] = hidden as u8;
    w.tlv(TLV_CONNECTION_SETTINGS, &settings);
    w.tlv(TLV_SSID, &(&(*a).want_ssid)[..(*a).want_ssid_len]);
    w.tlv(TLV_AUTH_ALGO_LIST, &WDI_AUTH_ALGO_RSNA_PSK.to_le_bytes());
    w.tlv(TLV_MULTICAST_CIPHER_ALGO_LIST, &group.to_le_bytes());
    w.tlv(TLV_UNICAST_CIPHER_ALGO_LIST, &WDI_CIPHER_ALGO_CCMP.to_le_bytes());
    w.close(params);
    // The candidate: the scan's own entry, whose elements are a subset of a connect entry's.
    w.tlv(TLV_CONNECT_BSS_ENTRY, entry);
    let n = w.len;
    let overflow = w.overflow;
    let m = (*a).bssid;
    logw!("connecting to %02x:%02x:%02x:%02x:%02x:%02x (group cipher %u)\n",
        m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32, group);
    (*a).assoc_seen = false;
    (*a).rsn_len = 0;
    let s = if overflow {
        NDIS_STATUS_INVALID_LENGTH
    } else {
        wdi_command(a, OID_WDI_TASK_CONNECT, (*a).port_id, &buf[..n], DONE_CONNECT, 20_000).0
    };
    ExFreePool(b as PVOID);
    s
}

/// Install a CCMP key: the pairwise key for the AP, or a group key by its ID. `rsc` is the
/// first packet number the key may receive.
unsafe fn add_key(a: *mut WdiAdapter, pairwise: bool, key: &[u8], id: u8, rsc: &[u8; 6]) -> NDIS_STATUS {
    let mut b = [0u8; 96];
    let mut w = TlvWriter::new(&mut b);
    let info = w.open(TLV_SET_CIPHER_KEY_INFO);
    if pairwise {
        w.tlv(TLV_PEER_MAC_ADDRESS, &(*a).bssid);
    } else {
        w.tlv(TLV_CIPHER_KEY_ID, &(id as u32).to_le_bytes());
    }
    // algorithm u32, direction u32, keep across roams u8, key type u32
    let mut ti = [0u8; 13];
    ti[0..4].copy_from_slice(&WDI_CIPHER_ALGO_CCMP.to_le_bytes());
    let dir = if pairwise { WDI_CIPHER_KEY_DIRECTION_BOTH } else { WDI_CIPHER_KEY_DIRECTION_INBOUND };
    ti[4..8].copy_from_slice(&dir.to_le_bytes());
    let ty = if pairwise { WDI_CIPHER_KEY_TYPE_PAIRWISE_KEY } else { WDI_CIPHER_KEY_TYPE_GROUP_KEY };
    ti[9..13].copy_from_slice(&ty.to_le_bytes());
    w.tlv(TLV_CIPHER_KEY_TYPE_INFO, &ti);
    w.tlv(TLV_CIPHER_KEY_RECEIVE_SEQUENCE_COUNT, rsc);
    w.tlv(TLV_CIPHER_KEY_CCMP_KEY, key);
    w.close(info);
    let n = w.len;
    let s = wdi_command(a, OID_WDI_SET_ADD_CIPHER_KEYS, (*a).port_id, &b[..n], 0, 0).0;
    logw!("%s key %u installed: %x\n", if pairwise { b"pairwise\0".as_ptr() } else { b"group\0".as_ptr() }, id as u32, s);
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
    // The configured network: keep its whole entry for the connect.
    let want = &(&(*a).want_ssid)[..(*a).want_ssid_len];
    if !want.is_empty() && &b.ssid[..b.ssid_len as usize] == want && entry.len() <= TARGET_BUF && !(*a).target.is_null() {
        core::ptr::copy_nonoverlapping(entry.as_ptr(), (*a).target, entry.len());
        (*a).target_len = entry.len();
        (*a).bssid = b.bssid;
    }
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

/// Record a WDI_TLV_ASSOCIATION_RESULT: whether the association took, and the RSN element our
/// association request carried, which EAPOL message 2 must repeat byte for byte.
unsafe fn note_association(a: *mut WdiAdapter, result: &[u8]) {
    let params = find(result, TLV_ASSOCIATION_RESULT_PARAMETERS).unwrap_or(&[]);
    let status = le32(params, 0);
    let ieee = le32(params, 4);
    let reassoc = params.get(8).copied().unwrap_or(0) != 0;
    logw!("association result: status %u, 802.11 status %u, auth %u, cipher %u/%u\n", status, ieee,
        le32(params, 9), le32(params, 13), le32(params, 17));
    (*a).assoc_status = status;
    if let Some(bssid) = find(result, TLV_BSSID) {
        if bssid.len() >= 6 {
            (*a).bssid.copy_from_slice(&bssid[..6]);
        }
    }
    // The request without its MAC header: capability, listen interval, the current AP's
    // address for a reassociation, then the elements.
    if let Some(req) = find(result, TLV_ASSOCIATION_REQUEST_FRAME) {
        let start = if reassoc { 10 } else { 4 };
        if let Some(rsn) = find_element(req.get(start..).unwrap_or(&[]), 0x30) {
            let n = rsn.len().min(64);
            (&mut (*a).rsn)[..n].copy_from_slice(&rsn[..n]);
            (*a).rsn_len = n;
        }
    }
    (*a).assoc_seen = true;
}

/// The whole element (ID, length, body) with `id` in a run of 802.11 elements.
fn find_element(ies: &[u8], id: u8) -> Option<&[u8]> {
    let mut i = 0;
    while i + 2 <= ies.len() {
        let len = ies[i + 1] as usize;
        if i + 2 + len > ies.len() {
            return None;
        }
        if ies[i] == id {
            return Some(&ies[i..i + 2 + len]);
        }
        i += 2 + len;
    }
    None
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
    (*a).target = alloc_zeroed(TARGET_BUF) as *mut u8;
    (*a).rx_buf = alloc_zeroed(RX_BUF) as *mut u8;
    (*a).rx_eth = alloc_zeroed(ETH_MAX) as *mut u8;
    if (*a).cmd_buf.is_null() || (*a).ind_buf.is_null() || (*a).target.is_null() || (*a).rx_buf.is_null()
        || (*a).rx_eth.is_null()
    {
        return NDIS_STATUS_RESOURCES;
    }
    // One NBL for the receive path: every indication is copied by NDIS before it returns.
    let mut pp: NblPoolParameters = core::mem::zeroed();
    pp.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<NblPoolParameters>() as u16 };
    pp.fAllocateNetBuffer = 1;
    pp.PoolTag = WDI_TAG;
    (*a).tx_pool = NdisAllocateNetBufferListPool(handle, &mut pp);
    (*a).rx_mdl = IoAllocateMdl((*a).rx_eth as PVOID, ETH_MAX as u32, FALSE as BOOLEAN, FALSE as BOOLEAN, null_mut());
    if (*a).tx_pool.is_null() || (*a).rx_mdl.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    MmBuildMdlForNonPagedPool((*a).rx_mdl);
    (*a).rx_nbl = NdisAllocateNetBufferAndNetBufferList((*a).tx_pool, 0, 0, (*a).rx_mdl, 0, 0);
    if (*a).rx_nbl.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    KeInitializeEvent(addr_of_mut!((*a).open_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).m3_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).m4_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).cmd_lock), _EVENT_TYPE_SynchronizationEvent, TRUE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).eapol_event), _EVENT_TYPE_SynchronizationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).eapol_sent), _EVENT_TYPE_NotificationEvent, TRUE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).stop_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeEvent(addr_of_mut!((*a).worker_done), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    KeInitializeSpinLock(addr_of_mut!((*a).bss_lock));
    KeInitializeSpinLock(addr_of_mut!((*a).eapol_lock));
    KeInitializeSpinLock(addr_of_mut!((*a).tx_lock));
    KeInitializeSpinLock(addr_of_mut!((*a).rx_lock));
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
        KeSetEvent(addr_of_mut!((*a).worker_done), 0, FALSE as BOOLEAN);
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

/// The bench's network, from \SystemRoot\bin\wifi.txt (staged by vendordrv): `ssid=`,
/// `psk=` (the WPA2 passphrase, never logged) and `hidden=1` for a network that does not
/// broadcast its name.
#[derive(Clone, Copy)]
struct WifiConfig {
    ssid: [u8; 32],
    ssid_len: usize,
    pass: [u8; 63],
    pass_len: usize,
    hidden: bool,
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
    let mut cfg = WifiConfig { ssid: [0; 32], ssid_len: 0, pass: [0; 63], pass_len: 0, hidden: false };
    for line in text[..n].split(|&c| c == b'\n') {
        let line = match line.last() {
            Some(b'\r') => &line[..line.len() - 1],
            _ => line,
        };
        if let Some(v) = line.strip_prefix(b"ssid=") {
            let k = v.len().min(32);
            cfg.ssid[..k].copy_from_slice(&v[..k]);
            cfg.ssid_len = k;
        } else if let Some(v) = line.strip_prefix(b"psk=") {
            let k = v.len().min(63);
            cfg.pass[..k].copy_from_slice(&v[..k]);
            cfg.pass_len = k;
        } else if let Some(v) = line.strip_prefix(b"hidden=") {
            cfg.hidden = v.first() == Some(&b'1');
        }
    }
    text.fill(0);
    if cfg.ssid_len == 0 {
        logw!("wifi.txt names no ssid\n");
        return None;
    }
    Some(cfg)
}

/// Wait `ms`, or less if halt begins. True when halt has begun.
unsafe fn stop_wait(a: *mut WdiAdapter, ms: u32) -> bool {
    ms_wait(addr_of_mut!((*a).stop_event), ms) || (*a).stopping.load(Ordering::Acquire)
}

/// Scan for the configured network, up to three times. True when it was found.
unsafe fn find_network(a: *mut WdiAdapter, ssid: &[u8]) -> bool {
    for round in 1..=3u32 {
        let mut irql: KIRQL = 0;
        KeAcquireSpinLock(addr_of_mut!((*a).bss_lock), &mut irql);
        (*a).bss_count = 0;
        (*a).target_len = 0;
        KeReleaseSpinLock(addr_of_mut!((*a).bss_lock), irql);
        logw!("scan %u\n", round);
        let s = scan(a, ssid);
        let n = (*a).bss_count;
        logw!("scan %u: status %x, %u networks\n", round, s, n as u32);
        for i in 0..n {
            let b = (*a).bss[i];
            let mut name = [0u8; 33];
            name[..b.ssid_len as usize].copy_from_slice(&b.ssid[..b.ssid_len as usize]);
            let m = b.bssid;
            logw!("  %02x:%02x:%02x:%02x:%02x:%02x ch %3u %d dBm \"%s\"\n",
                m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32,
                b.channel, b.rssi, name.as_ptr());
        }
        if (*a).target_len != 0 {
            logw!("found the configured network\n");
            return true;
        }
        if stop_wait(a, 3000) {
            return false;
        }
    }
    false
}

/// Join the configured network and keep it joined: after a lost link or a failed attempt,
/// reset the LE's connection state and try again, backing off from 5 s to a minute while
/// attempts keep failing. Leaves when halt begins.
unsafe extern "system" fn worker(context: PVOID) {
    let a = context as *mut WdiAdapter;
    if !stop_wait(a, 2000) {
        match read_config() {
            Some(c) if c.pass_len >= 8 => {
                (&mut (*a).want_ssid)[..c.ssid_len].copy_from_slice(&c.ssid[..c.ssid_len]);
                (*a).want_ssid_len = c.ssid_len;
                stay_joined(a, &c);
            }
            Some(c) => {
                // Scan once anyway: the networks in range are worth a line in the log.
                (&mut (*a).want_ssid)[..c.ssid_len].copy_from_slice(&c.ssid[..c.ssid_len]);
                (*a).want_ssid_len = c.ssid_len;
                find_network(a, &c.ssid[..c.ssid_len]);
                logw!("wifi.txt has no WPA2 passphrase (psk=, 8 to 63 characters): not joining\n");
            }
            None => {
                find_network(a, &[]);
            }
        }
    }
    logw!("worker done\n");
    KeSetEvent(addr_of_mut!((*a).worker_done), 0, FALSE as BOOLEAN);
    ntkrnl_sys::ddk::PsTerminateSystemThread(STATUS_SUCCESS);
}

unsafe fn stay_joined(a: *mut WdiAdapter, cfg: &WifiConfig) {
    // Test hook: `/WIFIDROP=<s>` drops the link once, s seconds after it first comes up.
    let drop_after = ntkrnl_sys::bootopt::boot_option_u32(b"WIFIDROP", 30);
    let mut backoff_s = 5u32;
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let end = if find_network(a, &cfg.ssid[..cfg.ssid_len]) {
            join(a, cfg, if attempt == 1 { drop_after } else { None })
        } else {
            JoinEnd::Failed
        };
        if (*a).stopping.load(Ordering::Acquire) || end == JoinEnd::Stopped {
            return;
        }
        // Whatever the LE still holds of the old connection goes before the next connect.
        disconnect(a);
        if end == JoinEnd::Lost {
            backoff_s = 5;
        }
        logw!("joining again in %u s (attempt %u %s)\n", backoff_s, attempt,
            if end == JoinEnd::Lost { b"lost the link\0".as_ptr() } else { b"failed\0".as_ptr() });
        if stop_wait(a, backoff_s * 1000) {
            return;
        }
        if end == JoinEnd::Failed {
            backoff_s = (backoff_s * 2).min(60);
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JoinEnd {
    /// Never got the link up.
    Failed,
    /// Had the link, and lost it.
    Lost,
    /// Halt began.
    Stopped,
}

/// The link is down to the protocols, if it was up. True when it was.
unsafe fn link_down(a: *mut WdiAdapter) -> bool {
    if !core::mem::replace(&mut (*a).connected, false) {
        return false;
    }
    let mut ls: LinkState = core::mem::zeroed();
    ls.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<LinkState>() as u16 };
    ls.MediaConnectState = 2; // MediaConnectStateDisconnected
    indicate_link_state((*a).adapter, &ls);
    true
}

/// End the LE's association, if it has one. Its failure is only logged: the LE refuses a
/// disconnect with nothing to disconnect, and a connect follows either way.
unsafe fn disconnect(a: *mut WdiAdapter) {
    let mut b = [0u8; 16];
    let mut w = TlvWriter::new(&mut b);
    // the peer's address, then the 802.11 reason code (3: the station is leaving).
    let mut p = [0u8; 8];
    p[..6].copy_from_slice(&(*a).bssid);
    p[6..8].copy_from_slice(&3u16.to_le_bytes());
    w.tlv(TLV_DISCONNECT_PARAMETERS, &p);
    let n = w.len;
    let s = wdi_command(a, OID_WDI_TASK_DISCONNECT, (*a).port_id, &b[..n], DONE_DISCONNECT, 5_000).0;
    logw!("disconnect: %x\n", s);
    link_down(a);
}

/// Fresh randomness for the SNonce. The kernel offers no RNG to a driver here, so it is a
/// hash of high-resolution timer readings taken around work whose duration varies (the PSK
/// derivation and the association), plus both addresses.
struct NonceSource {
    h: wpa::Sha1,
}

impl NonceSource {
    unsafe fn stir(&mut self) {
        let t = ntkrnl_sys::ddk::KeQueryPerformanceCounter(null_mut());
        self.h.update(&t.QuadPart.to_le_bytes());
    }
}

/// Associate, run the 4-way handshake, install the keys, and bring the link up. Then stay to
/// answer group rekeys for as long as the association lasts. `drop_after`: the test hook's
/// seconds of link before the worker disconnects on purpose.
unsafe fn join(a: *mut WdiAdapter, cfg: &WifiConfig, drop_after: Option<u32>) -> JoinEnd {
    let mut nonce = NonceSource { h: wpa::Sha1::new() };
    nonce.stir();
    let pmk = wpa::psk(&cfg.pass[..cfg.pass_len], &cfg.ssid[..cfg.ssid_len]);
    nonce.stir();
    let s = connect(a, cfg.hidden);
    nonce.stir();
    if s != NDIS_STATUS_SUCCESS {
        logw!("connect failed: %x\n", s);
        return JoinEnd::Failed;
    }
    if !(*a).assoc_seen || (*a).assoc_status != 0 {
        logw!("connect completed without a successful association (seen %u, status %u)\n",
            (*a).assoc_seen as u32, (*a).assoc_status);
        return JoinEnd::Failed;
    }
    if (*a).rsn_len == 0 {
        logw!("the association request carried no RSN element: cannot run the handshake\n");
        return JoinEnd::Failed;
    }
    if let Some(cfg_peer) = (*a).data.TalTxRxPeerConfig {
        let mut qos = WDI_TXRX_PEER_CFG_QOS_CAPABLE;
        cfg_peer((*a).tal, (*a).port_id, (*a).peer_id, &mut qos);
    }
    nonce.h.update(&(*a).bssid);
    nonce.h.update(&(*a).port_mac);
    nonce.stir();
    let d1 = nonce.h.clone().digest();
    nonce.stir();
    let d2 = nonce.h.digest();
    let mut snonce = [0u8; 32];
    snonce[..20].copy_from_slice(&d1);
    snonce[20..].copy_from_slice(&d2[..12]);
    // Frames left over from an earlier association belong to its handshake.
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).eapol_lock), &mut irql);
    (*a).eapol_count = 0;
    KeReleaseSpinLock(addr_of_mut!((*a).eapol_lock), irql);
    let mut sup = wpa::Supplicant::new(&pmk, &(*a).bssid, &(*a).port_mac, &(&(*a).rsn)[..(*a).rsn_len], &snonce);
    logw!("associated; waiting for the 4-way handshake\n");

    let mut reply = wpa::Reply { frame: [0; wpa::MAX_REPLY], len: 0, install_ptk: false, gtk: None, complete: false };
    let mut frame = [0u8; EAPOL_MAX];
    let mut idle_s = 0u32;
    let mut up_s = 0u32;
    let mut was_up = false;
    loop {
        if (*a).stopping.load(Ordering::Acquire) {
            return JoinEnd::Stopped;
        }
        if was_up && !(*a).connected {
            // peer_delete: the LE ended the association (deauthentication, lost beacons).
            return JoinEnd::Lost;
        }
        let len = match pop_eapol(a, &mut frame) {
            Some(n) => n,
            None => {
                if !ms_wait(addr_of_mut!((*a).eapol_event), 1000) {
                    idle_s += 1;
                    if !(*a).connected && idle_s == 15 {
                        logw!("no handshake from the access point in 15 s\n");
                        return JoinEnd::Failed;
                    }
                    if (*a).connected {
                        up_s += 1;
                        if drop_after == Some(up_s) {
                            logw!("WIFIDROP: dropping the link after %u s\n", up_s);
                            disconnect(a);
                            return JoinEnd::Lost;
                        }
                    }
                }
                continue;
            }
        };
        idle_s = 0;
        let r = sup.receive(&frame[..len], &mut reply);
        let info = if len >= 7 { u16::from_be_bytes([frame[5], frame[6]]) as u32 } else { 0 };
        match r {
            Ok(()) => logw!("EAPOL-Key (info %04x): ok, replying %u bytes\n", info, reply.len as u32),
            Err(e) => {
                logw!("EAPOL-Key (info %04x) rejected: %u\n", info, e as u32);
                continue;
            }
        }
        if reply.len != 0 {
            let bssid = (*a).bssid;
            if !send_frame(a, &bssid, ETHERTYPE_EAPOL, &reply.frame[..reply.len], true) {
                logw!("EAPOL reply not sent\n");
                continue;
            }
        }
        if reply.install_ptk {
            // Message 4 goes out under no key; the PTK it confirms must not encrypt it.
            if !ms_wait(addr_of_mut!((*a).eapol_sent), 1000) {
                logw!("message 4 not transferred after 1 s; installing the keys anyway\n");
            }
            add_key(a, true, sup.tk(), 0, &[0; 6]);
        }
        if let Some(g) = reply.gtk {
            add_key(a, false, &g.key[..g.len], g.id, &g.rsc);
        }
        if reply.complete && !(*a).connected {
            (*a).connected = true;
            was_up = true;
            logw!("4-way handshake complete: link up\n");
            let mut ls: LinkState = core::mem::zeroed();
            ls.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<LinkState>() as u16 };
            ls.MediaConnectState = MEDIA_CONNECT_STATE_CONNECTED;
            ls.MediaDuplexState = 2; // MediaDuplexStateFull
            ls.XmitLinkSpeed = 433_000_000;
            ls.RcvLinkSpeed = 433_000_000;
            indicate_link_state((*a).adapter, &ls);
        }
    }
}

/// The oldest EAPOL frame the receive path queued, copied into `out`.
unsafe fn pop_eapol(a: *mut WdiAdapter, out: &mut [u8; EAPOL_MAX]) -> Option<usize> {
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).eapol_lock), &mut irql);
    let r = if (*a).eapol_count == 0 {
        None
    } else {
        let i = (*a).eapol_head;
        let n = (*a).eapol_len[i];
        out[..n].copy_from_slice(&(&(*a).eapol[i])[..n]);
        (*a).eapol_head = (i + 1) % EAPOL_SLOTS;
        (*a).eapol_count -= 1;
        Some(n)
    };
    KeReleaseSpinLock(addr_of_mut!((*a).eapol_lock), irql);
    r
}

/// Queue an EAPOL frame (from its 802.1X header) for the worker. Runs at DISPATCH_LEVEL.
unsafe fn push_eapol(a: *mut WdiAdapter, frame: &[u8]) {
    if frame.len() > EAPOL_MAX {
        return;
    }
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).eapol_lock), &mut irql);
    if (*a).eapol_count < EAPOL_SLOTS {
        let i = ((*a).eapol_head + (*a).eapol_count) % EAPOL_SLOTS;
        (&mut (*a).eapol[i])[..frame.len()].copy_from_slice(frame);
        (*a).eapol_len[i] = frame.len();
        (*a).eapol_count += 1;
    }
    KeReleaseSpinLock(addr_of_mut!((*a).eapol_lock), irql);
    KeSetEvent(addr_of_mut!((*a).eapol_event), 0, FALSE as BOOLEAN);
}

// ---- the NDIS 6 side -----------------------------------------------------------------------

unsafe extern "system" fn ue_halt(ctx: NDIS_HANDLE, _action: u32) {
    let a = ctx as *mut WdiAdapter;
    let drv = (*a).drv;
    let w = &(*drv).wdi;
    // The worker first: it issues commands and reads the adapter. Its longest wait is one
    // command's completion (20 s for a scan or a connect).
    (*a).stopping.store(true, Ordering::Release);
    KeSetEvent(addr_of_mut!((*a).stop_event), 0, FALSE as BOOLEAN);
    KeSetEvent(addr_of_mut!((*a).eapol_event), 0, FALSE as BOOLEAN);
    if !ms_wait(addr_of_mut!((*a).worker_done), 60_000) {
        logw!("halt: the worker did not stop in 60 s\n");
    }
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

/// Sends: each Ethernet frame becomes an 802.11 data frame to the AP. The frame is copied, so
/// the protocol's NBLs complete at once.
unsafe extern "system" fn ue_send(ctx: NDIS_HANDLE, nbls: *mut NetBufferList, _port: u32, _flags: u32) {
    let a = ctx as *mut WdiAdapter;
    let mut eth = [0u8; ETH_MAX];
    let mut nbl = nbls;
    while !nbl.is_null() {
        let mut status = NDIS_STATUS_MEDIA_DISCONNECTED;
        if (*a).connected {
            status = NDIS_STATUS_SUCCESS;
            let mut nb = (*nbl).u.h.FirstNetBuffer;
            while !nb.is_null() {
                let n = copy_nb(nb, &mut eth);
                if n > ETH_HDR {
                    let mut dst = [0u8; 6];
                    dst.copy_from_slice(&eth[..6]);
                    let ethertype = u16::from_be_bytes([eth[12], eth[13]]);
                    if !send_frame(a, &dst, ethertype, &eth[ETH_HDR..n], false) {
                        status = NDIS_STATUS_RESOURCES;
                    }
                }
                nb = (*nb).u.h.Next;
            }
        }
        (*nbl).Status = status;
        nbl = (*nbl).u.h.Next;
    }
    NdisMSendNetBufferListsComplete((*a).adapter, nbls, 0);
}

unsafe extern "system" fn ue_return(_ctx: NDIS_HANDLE, _nbls: *mut NetBufferList, _flags: u32) {}

// ---- the data path -------------------------------------------------------------------------
//
// Transmit, in the LE's port-queueing mode: the UE builds the 802.11 frame, attaches its
// WDI_FRAME_METADATA (MiniportReserved[0]), has the LE initialize its target descriptor, and
// queues it; TxDataSend tells the LE there is work, and the LE pulls frames with
// TxDequeueIndication as its credits allow (synchronously, from inside TxDataSend, when it
// has them). Each frame comes back by TxTransferCompleteIndication. The UE's own state for a
// frame lives in the NBL's ProtocolReserved: the LE is the miniport here.

/// WDI_FRAME_METADATA: list linkage, the NBL, the frame ID, then the TX/RX union.
#[repr(C)]
struct FrameMetadata {
    linkage: [PVOID; 2],
    nbl: *mut NetBufferList,
    frame_id: u16,
    w_pad: u16,
    d_pad: u32,
    tx: TxMetadata,
}

/// WDI_TX_METADATA, up to ExemptionAction; the replay fields after it stay zero for a frame
/// sent for the first time. Its union holds 64-bit fields, hence the alignment.
#[repr(C, align(8))]
struct TxMetadata {
    port: u16,
    peer: u16,
    ex_tid: u8,
    unicast: u8,
    legacy_rates: u8,
    pad0: u8,
    ethertype: u16,
    tx_complete_required: u8,
    pn_length: u8,
    tx_cost: u16,
    pad1: u16,
    exemption: u32,
}

/// DOT11_EXTSTA_SEND_CONTEXT, the Native Wi-Fi per-frame send context. Realtek's TX path
/// reads it from every frame's MediaSpecificInformation (its exemption action at +4) whether
/// the frame came from WDI or not, so each frame carries one.
#[repr(C)]
struct SendContext {
    header: NdisObjectHeader,
    exemption: u16,
    phy_id: u32,
    delayed_sleep: u32,
    media_specific: PVOID,
    send_flags: u32,
}
/// NetBufferListInfo[MediaSpecificInformation].
const NBL_INFO_MEDIA_SPECIFIC: usize = 6;
const DOT11_PHY_ID_ANY: u32 = 0x8000_0000;
const WDI_EXEMPT_NO_EXEMPTION: u32 = 0;
const WDI_EXEMPT_ON_KEY_MAPPING_KEY_UNAVAILABLE: u32 = 2;

fn send_context(exemption: u32) -> SendContext {
    SendContext {
        header: NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<SendContext>() as u16 },
        exemption: exemption as u16,
        phy_id: DOT11_PHY_ID_ANY,
        delayed_sleep: 0,
        media_specific: null_mut(),
        send_flags: 0,
    }
}

// ProtocolReserved slots of a UE-sent NBL.
const PR_BUFFER: usize = 0;
const PR_PAYLOAD_MDL: usize = 1;
const PR_FLAGS: usize = 2;
const FRAME_EAPOL: usize = 1;
/// Injected by the LE: its NBL, handed back with TxTalSendComplete.
const FRAME_INJECTED: usize = 2;

/// Build and queue one data frame to `dst` through the AP. The first MDL holds the backfill,
/// the 802.11 header and the LLC/SNAP and nothing else, as WDI promises the LE. The send
/// context sits in front of it in the same allocation.
unsafe fn send_frame(a: *mut WdiAdapter, dst: &[u8; 6], ethertype: u16, payload: &[u8], eapol: bool) -> bool {
    const CTX: usize = core::mem::size_of::<SendContext>().next_multiple_of(16);
    let backfill = (*a).backfill as usize;
    let head = backfill + WLAN_HDR + WLAN_SNAP;
    let mem = ExAllocatePoolWithTag(NonPagedPool, (CTX + head + payload.len()) as SIZE_T, WDI_TAG) as *mut u8;
    if mem.is_null() {
        return false;
    }
    // EAPOL goes out unencrypted while there is no pairwise key, as Windows sends it.
    let exemption = if eapol { WDI_EXEMPT_ON_KEY_MAPPING_KEY_UNAVAILABLE } else { WDI_EXEMPT_NO_EXEMPTION };
    let ctx = mem as *mut SendContext;
    ctx.write(send_context(exemption));
    let buf = mem.add(CTX);
    let h = core::slice::from_raw_parts_mut(buf, head);
    h.fill(0);
    let f = &mut h[backfill..];
    f[0] = 0x08; // data
    f[1] = 0x01; // to DS
    f[4..10].copy_from_slice(&(*a).bssid);
    f[10..16].copy_from_slice(&(*a).port_mac);
    f[16..22].copy_from_slice(dst);
    f[WLAN_HDR..WLAN_HDR + 6].copy_from_slice(&SNAP);
    f[WLAN_HDR + 6..WLAN_HDR + 8].copy_from_slice(&ethertype.to_be_bytes());
    core::ptr::copy_nonoverlapping(payload.as_ptr(), buf.add(head), payload.len());

    let m1 = IoAllocateMdl(buf as PVOID, head as u32, FALSE as BOOLEAN, FALSE as BOOLEAN, null_mut());
    let m2 = IoAllocateMdl(buf.add(head) as PVOID, payload.len() as u32, FALSE as BOOLEAN, FALSE as BOOLEAN, null_mut());
    let md = alloc_metadata(a as PVOID) as *mut FrameMetadata;
    let nbl = if m1.is_null() || m2.is_null() || md.is_null() {
        null_mut()
    } else {
        MmBuildMdlForNonPagedPool(m1);
        MmBuildMdlForNonPagedPool(m2);
        (*m1).Next = m2;
        NdisAllocateNetBufferAndNetBufferList((*a).tx_pool, 0, 0, m1, backfill as u32, WLAN_HDR + WLAN_SNAP + payload.len())
    };
    if nbl.is_null() {
        if !m1.is_null() {
            IoFreeMdl(m1);
        }
        if !m2.is_null() {
            IoFreeMdl(m2);
        }
        if !md.is_null() {
            ExFreePool(md as PVOID);
        }
        ExFreePool(mem as PVOID);
        return false;
    }
    (*nbl).NetBufferListInfo[NBL_INFO_MEDIA_SPECIFIC] = ctx as PVOID;
    (*nbl).ProtocolReserved.0[PR_BUFFER] = mem as PVOID;
    (*nbl).ProtocolReserved.0[PR_PAYLOAD_MDL] = m2 as PVOID;
    (*nbl).ProtocolReserved.0[PR_FLAGS] = (if eapol { FRAME_EAPOL } else { 0 }) as PVOID;
    let t = &mut (*md).tx;
    t.port = (*a).port_id;
    t.peer = (*a).peer_id;
    // A station sends everything to the AP: the receiver is always unicast.
    t.unicast = 1;
    t.ethertype = ethertype;
    t.exemption = exemption;
    if eapol {
        (*a).eapol_in_flight.fetch_add(1, Ordering::AcqRel);
        KeClearEvent(addr_of_mut!((*a).eapol_sent));
    }
    queue_frame(a, nbl, md)
}

/// Attach metadata, have the LE initialize its descriptor, queue, and tell the LE.
unsafe fn queue_frame(a: *mut WdiAdapter, nbl: *mut NetBufferList, md: *mut FrameMetadata) -> bool {
    (*md).nbl = nbl;
    (*nbl).MiniportReserved.0[0] = md as PVOID;
    (*nbl).u.h.Next = null_mut();
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).tx_lock), &mut irql);
    (*a).frame_id = (*a).frame_id.wrapping_add(1);
    (*md).frame_id = (*a).frame_id;
    KeReleaseSpinLock(addr_of_mut!((*a).tx_lock), irql);
    let mut status: NDIS_STATUS = NDIS_STATUS_SUCCESS;
    if let Some(init) = (*a).data.TxTargetDescInit {
        init((*a).tal, nbl, &mut status);
    }
    if status != NDIS_STATUS_SUCCESS {
        logw!("TxTargetDescInit refused a frame: %x\n", status);
        finish_frame(a, nbl, false);
        return false;
    }
    KeAcquireSpinLock(addr_of_mut!((*a).tx_lock), &mut irql);
    if (*a).tx_tail.is_null() {
        (*a).tx_head = nbl;
    } else {
        (*(*a).tx_tail).u.h.Next = nbl;
    }
    (*a).tx_tail = nbl;
    (*a).tx_queued += 1;
    KeReleaseSpinLock(addr_of_mut!((*a).tx_lock), irql);
    kick_tx(a);
    true
}

/// Tell the LE frames are waiting, unless it paused the port.
unsafe fn kick_tx(a: *mut WdiAdapter) {
    let (queued, paused) = ((*a).tx_queued, (*a).tx_paused);
    if queued == 0 || paused {
        return;
    }
    if let Some(send) = (*a).data.TxDataSend {
        send((*a).tal, (*a).port_id, (*a).peer_id, 0, queued.min(0xFFFF) as u16, queued, 0);
    }
}

/// A frame the LE is done with (or never took): release its descriptor and the UE's state.
unsafe fn finish_frame(a: *mut WdiAdapter, nbl: *mut NetBufferList, desc: bool) {
    if desc {
        if let Some(deinit) = (*a).data.TxTargetDescDeInit {
            deinit((*a).tal, nbl);
        }
    }
    let flags = (*nbl).ProtocolReserved.0[PR_FLAGS] as usize;
    let md = (*nbl).MiniportReserved.0[0];
    (*nbl).MiniportReserved.0[0] = null_mut();
    if !md.is_null() {
        ExFreePool(md);
    }
    if flags & FRAME_INJECTED != 0 {
        if let Some(done) = (*a).data.TxTalSendComplete {
            done((*a).tal, nbl, WDI_TX_FRAME_STATUS_OK);
        }
        return;
    }
    let nb = (*nbl).u.h.FirstNetBuffer;
    let m1 = (*nb).u.h.MdlChain;
    IoFreeMdl((*nbl).ProtocolReserved.0[PR_PAYLOAD_MDL] as PMDL);
    IoFreeMdl(m1);
    ExFreePool((*nbl).ProtocolReserved.0[PR_BUFFER]);
    NdisFreeNetBufferList(nbl);
    if flags & FRAME_EAPOL != 0 && (*a).eapol_in_flight.fetch_sub(1, Ordering::AcqRel) == 1 {
        KeSetEvent(addr_of_mut!((*a).eapol_sent), 0, FALSE as BOOLEAN);
    }
}

/// The LE pulls queued frames: at most `max`, costing no more than `credit` in all.
unsafe extern "system" fn tx_dequeue(h: PVOID, _quantum: u32, max: u8, credit: u16, out: *mut *mut NetBufferList) {
    let a = h as *mut WdiAdapter;
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).tx_lock), &mut irql);
    let mut first: *mut NetBufferList = null_mut();
    let mut last: *mut NetBufferList = null_mut();
    let (mut n, mut spent) = (0u32, 0u32);
    while !(*a).tx_head.is_null() && n < max as u32 {
        let nbl = (*a).tx_head;
        let md = (*nbl).MiniportReserved.0[0] as *mut FrameMetadata;
        let cost = (*md).tx.tx_cost as u32;
        if spent + cost > credit as u32 {
            break;
        }
        spent += cost;
        (*a).tx_head = (*nbl).u.h.Next;
        if (*a).tx_head.is_null() {
            (*a).tx_tail = null_mut();
        }
        (*a).tx_queued -= 1;
        (*nbl).u.h.Next = null_mut();
        if last.is_null() {
            first = nbl;
        } else {
            (*last).u.h.Next = nbl;
        }
        last = nbl;
        n += 1;
    }
    KeReleaseSpinLock(addr_of_mut!((*a).tx_lock), irql);
    *out = first;
}

unsafe extern "system" fn tx_transfer_complete(h: PVOID, status: u32, nbl: *mut NetBufferList) {
    let a = h as *mut WdiAdapter;
    if status != WDI_TX_FRAME_STATUS_OK {
        logw!("transfer completed with status %u\n", status);
    }
    let mut n = nbl;
    while !n.is_null() {
        let next = (*n).u.h.Next;
        (*n).u.h.Next = null_mut();
        finish_frame(a, n, true);
        n = next;
    }
}

/// Only frames that asked for a send completion get one, and none do.
unsafe extern "system" fn tx_send_complete(_h: PVOID, status: u32, n: u16, _ids: *mut u16, _data: PVOID) {
    logw!("send completion for %u frames (status %u), none requested\n", n as u32, status);
}

unsafe extern "system" fn tx_query_ra_tid(_h: PVOID, _port: u16, _peer: u16, _tid: u8, status: *mut NDIS_STATUS, len: *mut u16) {
    *status = NDIS_STATUS_SUCCESS;
    *len = 0;
}

unsafe extern "system" fn tx_pause(h: PVOID, _port: u16, _peer: u16, _tids: u32, _reason: u32) {
    (*(h as *mut WdiAdapter)).tx_paused = true;
}

unsafe extern "system" fn tx_restart(h: PVOID, _port: u16, _peer: u16, _tids: u32, _reason: u32) {
    let a = h as *mut WdiAdapter;
    (*a).tx_paused = false;
    kick_tx(a);
}

unsafe extern "system" fn tx_release(_h: PVOID, _port: u16, _peer: u16, _tids: u32, _max: u8, _credit: u16, out: *mut *mut NetBufferList) {
    *out = null_mut();
}

/// A frame the LE built itself: queued like the UE's own, and handed back when transferred.
unsafe extern "system" fn tx_inject(h: PVOID, port: u16, peer: u16, tid: u8, nbl: *mut NetBufferList, uni: u8, legacy: u8, ethertype: u16, ex: u32) {
    let a = h as *mut WdiAdapter;
    let md = alloc_metadata(a as PVOID) as *mut FrameMetadata;
    if md.is_null() {
        if let Some(done) = (*a).data.TxTalSendComplete {
            done((*a).tal, nbl, 3 /* WDI_TxFrameStatus_TransferCancelled */);
        }
        return;
    }
    let t = &mut (*md).tx;
    t.port = port;
    t.peer = peer;
    t.ex_tid = tid;
    t.unicast = uni;
    t.legacy_rates = legacy;
    t.ethertype = ethertype;
    t.exemption = ex;
    if (*nbl).NetBufferListInfo[NBL_INFO_MEDIA_SPECIFIC].is_null() {
        (*a).inject_ctx = send_context(ex);
        (*nbl).NetBufferListInfo[NBL_INFO_MEDIA_SPECIFIC] = addr_of_mut!((*a).inject_ctx) as PVOID;
    }
    (*nbl).ProtocolReserved.0[PR_FLAGS] = FRAME_INJECTED as PVOID;
    queue_frame(a, nbl, md);
}

unsafe extern "system" fn datapath_confirm(_h: PVOID) {}
unsafe extern "system" fn tx_suspect(_h: PVOID, _ctx: u64, _nbl: *mut NetBufferList, xfer: *mut u8, send: *mut u8, status: *mut NDIS_STATUS) {
    *xfer = 0;
    *send = 0;
    *status = NDIS_STATUS_SUCCESS;
}

/// Frames the LE has ready, each one decrypted MSDU with its 802.11 header: EAPOL goes to the
/// supplicant, the rest up to the protocols as Ethernet once the link is up.
unsafe extern "system" fn rx_inorder(h: PVOID, _level: u32, peer: u16, tid: u8, _throttle: PVOID, status: *mut NDIS_STATUS) {
    let a = h as *mut WdiAdapter;
    *status = NDIS_STATUS_SUCCESS;
    let (Some(get), Some(ret)) = ((*a).data.RxGetMpdus, (*a).data.RxReturnFrames) else { return };
    let mut chain: *mut NetBufferList = null_mut();
    get((*a).tal, peer, tid, &mut chain);
    let mut irql: KIRQL = 0;
    KeAcquireSpinLock(addr_of_mut!((*a).rx_lock), &mut irql);
    let mut nbl = chain;
    while !nbl.is_null() {
        let mut nb = (*nbl).u.h.FirstNetBuffer;
        while !nb.is_null() {
            let buf = core::slice::from_raw_parts_mut((*a).rx_buf, RX_BUF);
            let n = copy_nb(nb, buf);
            receive_msdu(a, &buf[..n]);
            nb = (*nb).u.h.Next;
        }
        nbl = (*nbl).u.h.Next;
    }
    KeReleaseSpinLock(addr_of_mut!((*a).rx_lock), irql);
    if !chain.is_null() {
        ret((*a).tal, chain);
    }
}

/// One received MSDU. Runs under rx_lock.
unsafe fn receive_msdu(a: *mut WdiAdapter, f: &[u8]) {
    if f.len() < WLAN_HDR {
        return;
    }
    let (fc0, fc1) = (f[0], f[1]);
    if (*a).rx_logged < 4 {
        (*a).rx_logged += 1;
        logw!("rx: %u bytes, frame control %02x %02x, %02x %02x %02x %02x %02x %02x %02x %02x\n", f.len() as u32,
            fc0 as u32, fc1 as u32, f[24] as u32, f[25] as u32, f[26] as u32, f[27] as u32,
            f.get(28).copied().unwrap_or(0) as u32, f.get(29).copied().unwrap_or(0) as u32,
            f.get(30).copied().unwrap_or(0) as u32, f.get(31).copied().unwrap_or(0) as u32);
    }
    // Data frames only, not null-function (subtype bit 2).
    if (fc0 >> 2) & 3 != 2 || fc0 & 0x40 != 0 {
        return;
    }
    // The Protected bit survives the target's decryption (Realtek leaves it set and strips
    // the CCMP header and MIC), so it says nothing here: a frame still encrypted fails the
    // LLC/SNAP check below instead.
    let (to_ds, from_ds) = (fc1 & 1 != 0, fc1 & 2 != 0);
    let mut hdr = WLAN_HDR;
    if to_ds && from_ds {
        hdr += 6;
    }
    if fc0 & 0x80 != 0 {
        hdr += 2; // QoS control
        if fc1 & 0x80 != 0 {
            hdr += 4; // HT control
        }
    }
    if f.len() < hdr + WLAN_SNAP || f[hdr..hdr + 3] != SNAP[..3] || (f[hdr + 3..hdr + 6] != [0, 0, 0] && f[hdr + 3..hdr + 6] != [0, 0, 0xF8]) {
        return;
    }
    let ethertype = u16::from_be_bytes([f[hdr + 6], f[hdr + 7]]);
    let payload = &f[hdr + WLAN_SNAP..];
    if ethertype == ETHERTYPE_EAPOL {
        push_eapol(a, payload);
        return;
    }
    if !(*a).connected || payload.len() > ETH_MAX - ETH_HDR {
        return;
    }
    // Destination is address 1; the source is address 3 from the AP, address 2 otherwise.
    let src = if from_ds { &f[16..22] } else { &f[10..16] };
    let eth = core::slice::from_raw_parts_mut((*a).rx_eth, ETH_MAX);
    eth[..6].copy_from_slice(&f[4..10]);
    eth[6..12].copy_from_slice(src);
    eth[12..14].copy_from_slice(&ethertype.to_be_bytes());
    eth[ETH_HDR..ETH_HDR + payload.len()].copy_from_slice(payload);
    let nb = (*(*a).rx_nbl).u.h.FirstNetBuffer;
    (*nb).u.h.DataLength = ETH_HDR + payload.len();
    (*(*a).rx_mdl).ByteCount = (ETH_HDR + payload.len()) as u32;
    // RESOURCES: NDIS copies the frame and keeps nothing, so the NBL is never returned.
    NdisMIndicateReceiveNetBufferLists((*a).adapter, (*a).rx_nbl, 0, 1, NDIS_RECEIVE_FLAGS_RESOURCES);
}

unsafe extern "system" fn peer_create(h: PVOID, port: u16, peer: u16, mac: WdiMac) {
    let a = h as *mut WdiAdapter;
    let m = mac.0;
    logw!("peer %u on port %u: %02x:%02x:%02x:%02x:%02x:%02x\n", peer as u32, port as u32,
        m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32);
    if m == (*a).bssid {
        (*a).peer_id = peer;
    }
}
unsafe extern "system" fn peer_delete(h: PVOID, port: u16, peer: u16, status: *mut NDIS_STATUS) {
    let a = h as *mut WdiAdapter;
    logw!("peer %u on port %u deleted\n", peer as u32, port as u32);
    if peer == (*a).peer_id && link_down(a) {
        logw!("the access point is gone: link down\n");
        // The worker notices at its next wait, and joins again.
        KeSetEvent(addr_of_mut!((*a).eapol_event), 0, FALSE as BOOLEAN);
    }
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
