//! bwfm.sys as an NDIS 6 miniport: an 802.3 adapter to every protocol above it, as any
//! Ethernet card is. The chip is FullMAC, so scanning, association and roaming are
//! firmware commands and 802.11 never shows above this file; WPA2's handshake is the
//! host's (join.rs), and stays the host's for the group rekeys (link.rs).
//!
//! PnP installs it from netbwfm.inf on the HAL's `VIBEOS\CYW43455` PDO; ndis.sys runs the
//! device stack and calls initialize with the PDO's resources. Initialize brings the chip
//! up to a running firmware with its MAC address, which the general attributes need,
//! and starts the worker. The worker owns the bus from then on: it scans, joins (and
//! joins again whenever the link is lost), then carries frames both ways, polling the
//! chip between sends (no card interrupt yet). The stack's send path only copies frames
//! into a ring, because it may run at DISPATCH_LEVEL and the SDIO bus waits.

use core::ptr::{addr_of_mut, null_mut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use ndis6_sys::{
    GeneralAttributes, LinkState, MiniportDriverCharacteristics, MiniportInitParameters, MiniportRestartParameters,
    NblPoolParameters, NdisObjectHeader, NdisOidRequest, NetBuffer, NetBufferList, RegistrationAttributes,
    StatusIndication, MEDIA_CONNECT_STATE_CONNECTED, MEDIA_CONNECT_STATE_DISCONNECTED, MEDIA_DUPLEX_STATE_FULL,
    NDIS_OBJECT_TYPE_DEFAULT, NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES,
    NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES, NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS,
    NDIS_OBJECT_TYPE_STATUS_INDICATION, NDIS_RECEIVE_FLAGS_DISPATCH_LEVEL, NDIS_RECEIVE_FLAGS_RESOURCES,
    NDIS_SEND_COMPLETE_FLAGS_DISPATCH_LEVEL, NDIS_SEND_FLAGS_DISPATCH_LEVEL, NDIS_STATUS_LINK_STATE,
};
use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePoolWithTag, IoAllocateMdl, IoFreeMdl, KeInitializeEvent, KeInitializeSpinLock,
    KeSetEvent, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, PsCreateSystemThread, ZwClose,
    _EVENT_TYPE_NotificationEvent, _EVENT_TYPE_SynchronizationEvent, _KWAIT_REASON_Executive,
    _MM_PAGE_PRIORITY_NormalPagePriority, CM_PARTIAL_RESOURCE_LIST, HANDLE, KEVENT, LARGE_INTEGER, NTSTATUS,
    PDRIVER_OBJECT, PMDL, PUNICODE_STRING, PVOID,
};
use ntkrnl_sys::driver_ddk::MmGetSystemAddressForMdlSafe;
use ntkrnl_sys::guards::{IrqlGuard, SpinLockGuard};

use crate::bus::{Buffers, Bus};
use crate::driver::{self, Kernel, BWFM_POOL_TAG};
use crate::join::{JoinEnd, JoinLog, Station, WifiConfig};
use crate::link::{LinkLog, Serviced};
use crate::net::{TxRing, ETH_FRAME_MAX};
use crate::print::bwprint;

type NdisHandle = PVOID;
type NdisStatus = i32;

const NDIS_STATUS_SUCCESS: NdisStatus = 0;
const NDIS_STATUS_RESOURCES: NdisStatus = 0xC000_009Au32 as i32;
const NDIS_STATUS_NOT_SUPPORTED: NdisStatus = 0xC000_00BBu32 as i32;
const NDIS_STATUS_ADAPTER_NOT_FOUND: NdisStatus = 0xC001_0006u32 as i32;
const NDIS_STATUS_MEDIA_DISCONNECTED: NdisStatus = 0xC001_001Fu32 as i32;
const NDIS_STATUS_PAUSED: NdisStatus = 0xC023_002Au32 as i32;

/// NdisMedium802_3, NdisPhysicalMediumNative802_11.
const MEDIUM_802_3: u32 = 0;
const PHYSICAL_MEDIUM_NATIVE_802_11: u32 = 9;
/// NDIS_MAC_OPTION_COPY_LOOKAHEAD_DATA | _TRANSFERS_NOT_PEND | _NO_LOOPBACK.
const MAC_OPTIONS: u32 = 0x1 | 0x2 | 0x8;
/// NDIS_PACKET_TYPE_DIRECTED | _MULTICAST | _ALL_MULTICAST | _BROADCAST.
const PACKET_FILTERS: u32 = 0x1 | 0x2 | 0x4 | 0x8;
const MULTICAST_LIST: u32 = 32;
const MTU: u32 = 1500;
/// Until the firmware says (`Bus::rate`): 802.11g's top rate.
const DEFAULT_SPEED: u64 = 54_000_000;
/// NdisInterfaceInternal: the SDIO host is no bus NDIS knows.
const INTERFACE_INTERNAL: u32 = 0;
const CM_RESOURCE_TYPE_MEMORY: u8 = 3;

const OID_GEN_CURRENT_PACKET_FILTER: u32 = 0x0001_010E;
const OID_GEN_CURRENT_LOOKAHEAD: u32 = 0x0001_010F;
const OID_802_3_MULTICAST_LIST: u32 = 0x0101_0103;
/// NdisRequestSetInformation.
const OID_SET: u32 = 1;

/// Frames the stack may have waiting for the worker.
const TX_SLOTS: usize = 32;
/// How long the worker sleeps between polls when the chip is quiet and nothing waits
/// to go out. A queued frame wakes it at once.
const IDLE_POLL_MS: i64 = 4;
/// The wait before joining again after a failure, doubling to the cap; a lost link
/// tries again after the first.
const REJOIN_FIRST_S: u32 = 5;
const REJOIN_MAX_S: u32 = 60;

/// One adapter: the Pi has one Wi-Fi chip.
struct Adapter {
    handle: NdisHandle,
    /// The worker's from the moment it starts.
    bus: Option<Bus<Kernel>>,
    cfg: Option<WifiConfig>,
    // guards::SpinLockGuard's spelling of KSPIN_LOCK (a ULONG_PTR either way).
    tx_lock: usize,
    tx: TxRing<TX_SLOTS>,
    /// Signalled when a frame is queued, and at halt.
    wake: KEVENT,
    worker_done: KEVENT,
    stopping: AtomicBool,
    connected: AtomicBool,
    paused: AtomicBool,
    /// Frames the ring refused (full), for the log.
    refused: AtomicU32,
    rx_pool: NdisHandle,
    rx_buf: *mut u8,
    rx_mdl: PMDL,
    rx_nbl: *mut NetBufferList,
}

static mut BUFFERS: Buffers = Buffers::new();

extern "system" {
    fn NdisMRegisterMiniportDriver(
        DriverObject: PDRIVER_OBJECT,
        RegistryPath: PUNICODE_STRING,
        MiniportDriverContext: NdisHandle,
        Characteristics: *mut MiniportDriverCharacteristics,
        NdisMiniportDriverHandle: *mut NdisHandle,
    ) -> NdisStatus;
    fn NdisMSetMiniportAttributes(NdisMiniportHandle: NdisHandle, Attributes: *mut NdisObjectHeader) -> NdisStatus;
    fn NdisMSendNetBufferListsComplete(MiniportAdapterHandle: NdisHandle, NetBufferList: *mut NetBufferList, Flags: u32);
    fn NdisMIndicateReceiveNetBufferLists(
        MiniportAdapterHandle: NdisHandle,
        NetBufferList: *mut NetBufferList,
        PortNumber: u32,
        NumberOfNetBufferLists: u32,
        ReceiveFlags: u32,
    );
    fn NdisMIndicateStatusEx(MiniportAdapterHandle: NdisHandle, StatusIndication: *mut StatusIndication);
    fn NdisAllocateNetBufferListPool(NdisHandle: NdisHandle, Parameters: *mut NblPoolParameters) -> NdisHandle;
    fn NdisAllocateNetBufferAndNetBufferList(
        PoolHandle: NdisHandle,
        ContextSize: u16,
        ContextBackFill: u16,
        MdlChain: PMDL,
        DataOffset: u32,
        DataLength: usize,
    ) -> *mut NetBufferList;
}

fn header(ty: u8, size: usize) -> NdisObjectHeader {
    NdisObjectHeader { Type: ty, Revision: 1, Size: size as u16 }
}

/// DriverEntry's half: register the miniport. ndis.sys owns the device stack from here.
pub unsafe fn register(driver: PDRIVER_OBJECT, registry: PUNICODE_STRING) -> NTSTATUS {
    let mut c: MiniportDriverCharacteristics = core::mem::zeroed();
    c.Header = header(NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS, core::mem::size_of::<MiniportDriverCharacteristics>());
    c.MajorNdisVersion = 6;
    c.MinorNdisVersion = 20;
    c.MajorDriverVersion = 1;
    c.InitializeHandlerEx = Some(initialize);
    c.HaltHandlerEx = Some(halt);
    c.PauseHandler = Some(pause);
    c.RestartHandler = Some(restart);
    c.OidRequestHandler = Some(oid_request);
    c.SendNetBufferListsHandler = Some(send);
    c.ReturnNetBufferListsHandler = Some(return_nbls);
    c.ShutdownHandlerEx = Some(shutdown);
    let mut handle: NdisHandle = null_mut();
    let s = NdisMRegisterMiniportDriver(driver, registry, null_mut(), &mut c, &mut handle);
    if s != NDIS_STATUS_SUCCESS {
        bwprint!("NdisMRegisterMiniportDriver failed {:#010x}\n", s as u32);
    }
    s
}

/// The SDIO host's register window, from the PDO's translated resources.
unsafe fn register_window(params: *mut MiniportInitParameters) -> Option<(u64, usize)> {
    let list = (*params).AllocatedResources as *const CM_PARTIAL_RESOURCE_LIST;
    if list.is_null() {
        return None;
    }
    let descs = core::slice::from_raw_parts((*list).PartialDescriptors.as_ptr(), (*list).Count as usize);
    let mem = descs.iter().find(|d| d.Type == CM_RESOURCE_TYPE_MEMORY)?;
    Some((mem.u.Memory.Start.QuadPart as u64, mem.u.Memory.Length as usize))
}

unsafe extern "system" fn initialize(handle: NdisHandle, _driver_ctx: NdisHandle, params: *mut MiniportInitParameters) -> NdisStatus {
    let Some((pa, len)) = register_window(params) else {
        bwprint!("the adapter's resources carry no register window\n");
        return NDIS_STATUS_ADAPTER_NOT_FOUND;
    };
    let Some(mut chip) = driver::attach(pa, len) else { return NDIS_STATUS_ADAPTER_NOT_FOUND };
    if !driver::run_firmware(&mut chip) {
        return NDIS_STATUS_ADAPTER_NOT_FOUND;
    }
    let mut bus = Bus::new(chip, &mut *(&raw mut BUFFERS));
    if !driver::ready(&mut bus) {
        return NDIS_STATUS_ADAPTER_NOT_FOUND;
    }

    let a = ExAllocatePoolWithTag(0, core::mem::size_of::<Adapter>() as _, BWFM_POOL_TAG) as *mut Adapter;
    let rx_buf = ExAllocatePoolWithTag(0, ETH_FRAME_MAX as _, BWFM_POOL_TAG) as *mut u8;
    if a.is_null() || rx_buf.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    let mac = bus.mac;
    a.write(Adapter {
        handle,
        bus: Some(bus),
        cfg: driver::read_config(),
        tx_lock: 0,
        tx: TxRing::new(),
        wake: core::mem::zeroed(),
        worker_done: core::mem::zeroed(),
        stopping: AtomicBool::new(false),
        connected: AtomicBool::new(false),
        paused: AtomicBool::new(true),
        refused: AtomicU32::new(0),
        rx_pool: null_mut(),
        rx_buf,
        rx_mdl: null_mut(),
        rx_nbl: null_mut(),
    });
    KeInitializeSpinLock(addr_of_mut!((*a).tx_lock).cast());
    KeInitializeEvent(addr_of_mut!((*a).wake), _EVENT_TYPE_SynchronizationEvent, 0);
    KeInitializeEvent(addr_of_mut!((*a).worker_done), _EVENT_TYPE_NotificationEvent, 0);

    // One NBL for every receive: indicated with RESOURCES, so NDIS copies the frame
    // before it returns and never hands the NBL back.
    let mut pp: NblPoolParameters = core::mem::zeroed();
    pp.Header = header(NDIS_OBJECT_TYPE_DEFAULT, core::mem::size_of::<NblPoolParameters>());
    pp.fAllocateNetBuffer = 1;
    pp.PoolTag = BWFM_POOL_TAG;
    (*a).rx_pool = NdisAllocateNetBufferListPool(handle, &mut pp);
    (*a).rx_mdl = IoAllocateMdl(rx_buf as PVOID, ETH_FRAME_MAX as u32, 0, 0, null_mut());
    if (*a).rx_pool.is_null() || (*a).rx_mdl.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    MmBuildMdlForNonPagedPool((*a).rx_mdl);
    (*a).rx_nbl = NdisAllocateNetBufferAndNetBufferList((*a).rx_pool, 0, 0, (*a).rx_mdl, 0, 0);
    if (*a).rx_nbl.is_null() {
        return NDIS_STATUS_RESOURCES;
    }

    let mut reg: RegistrationAttributes = core::mem::zeroed();
    reg.Header = header(NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES, core::mem::size_of::<RegistrationAttributes>());
    reg.MiniportAdapterContext = a as NdisHandle;
    reg.InterfaceType = INTERFACE_INTERNAL;
    let s = NdisMSetMiniportAttributes(handle, addr_of_mut!(reg.Header));
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }
    let mut gen: GeneralAttributes = core::mem::zeroed();
    gen.Header = header(NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES, core::mem::size_of::<GeneralAttributes>());
    gen.MediaType = MEDIUM_802_3;
    gen.PhysicalMediumType = PHYSICAL_MEDIUM_NATIVE_802_11;
    gen.MtuSize = MTU;
    gen.MaxXmitLinkSpeed = DEFAULT_SPEED;
    gen.XmitLinkSpeed = DEFAULT_SPEED;
    gen.MaxRcvLinkSpeed = DEFAULT_SPEED;
    gen.RcvLinkSpeed = DEFAULT_SPEED;
    gen.MediaConnectState = MEDIA_CONNECT_STATE_DISCONNECTED;
    gen.MediaDuplexState = MEDIA_DUPLEX_STATE_FULL;
    gen.LookaheadSize = MTU;
    gen.MacOptions = MAC_OPTIONS;
    gen.SupportedPacketFilters = PACKET_FILTERS;
    gen.MaxMulticastListSize = MULTICAST_LIST;
    gen.MacAddressLength = 6;
    gen.PermanentMacAddress[..6].copy_from_slice(&mac);
    gen.CurrentMacAddress[..6].copy_from_slice(&mac);
    let s = NdisMSetMiniportAttributes(handle, addr_of_mut!(gen.Header));
    if s != NDIS_STATUS_SUCCESS {
        return s;
    }

    if (*a).cfg.is_none() {
        bwprint!("no \\SystemRoot\\bin\\wifi.txt (ssid=, psk=): the adapter stays disconnected\n");
    }
    let mut thread: HANDLE = null_mut();
    let s = PsCreateSystemThread(&mut thread, 0x001F_03FF, null_mut(), null_mut(), null_mut(), Some(worker), a as PVOID);
    if s < 0 {
        bwprint!("no worker thread ({:#010x}): the adapter stays disconnected\n", s as u32);
        KeSetEvent(addr_of_mut!((*a).worker_done), 0, 0);
    } else {
        ZwClose(thread);
    }
    bwprint!("NDIS adapter ready\n");
    NDIS_STATUS_SUCCESS
}

unsafe extern "system" fn halt(ctx: NdisHandle, _action: u32) {
    let a = ctx as *mut Adapter;
    (*a).stopping.store(true, Ordering::Release);
    KeSetEvent(addr_of_mut!((*a).wake), 0, 0);
    // A scan or a join runs to its end first: at most a few seconds.
    let mut t: LARGE_INTEGER = core::mem::zeroed();
    t.QuadPart = -60 * 10_000_000;
    if KeWaitForSingleObject(addr_of_mut!((*a).worker_done).cast(), _KWAIT_REASON_Executive, 0, 0, &mut t) != 0 {
        // Its state is still in use: leave it, rather than free it under the worker.
        bwprint!("halt: the worker did not stop in 60 s\n");
        return;
    }
    if !(*a).rx_mdl.is_null() {
        IoFreeMdl((*a).rx_mdl);
    }
    ExFreePoolWithTag((*a).rx_buf.cast(), BWFM_POOL_TAG);
    ExFreePoolWithTag(a.cast(), BWFM_POOL_TAG);
}

unsafe extern "system" fn shutdown(_ctx: NdisHandle, _action: u32) {}

unsafe extern "system" fn pause(ctx: NdisHandle, _params: PVOID) -> NdisStatus {
    let a = ctx as *mut Adapter;
    (*a).paused.store(true, Ordering::Release);
    NDIS_STATUS_SUCCESS
}

unsafe extern "system" fn restart(ctx: NdisHandle, _params: *mut MiniportRestartParameters) -> NdisStatus {
    let a = ctx as *mut Adapter;
    (*a).paused.store(false, Ordering::Release);
    NDIS_STATUS_SUCCESS
}

/// The OIDs ndis.sys does not answer from the general attributes. The firmware passes
/// the frames for our address, broadcast and multicast as it is, so filters and lists
/// are accepted as they come.
unsafe extern "system" fn oid_request(_ctx: NdisHandle, req: *mut NdisOidRequest) -> NdisStatus {
    let q = &mut (*req).DATA.q;
    match ((*req).RequestType, q.Oid) {
        (OID_SET, OID_GEN_CURRENT_PACKET_FILTER | OID_GEN_CURRENT_LOOKAHEAD | OID_802_3_MULTICAST_LIST) => {
            q.BytesWritten = q.InformationBufferLength;
            NDIS_STATUS_SUCCESS
        }
        _ => NDIS_STATUS_NOT_SUPPORTED,
    }
}

/// Copy a net buffer's data into `dst`: its length, or None when it does not fit.
unsafe fn copy_nb(nb: *mut NetBuffer, dst: &mut [u8; ETH_FRAME_MAX]) -> Option<usize> {
    let h = &(*nb).u.h;
    let total = h.DataLength;
    if total > dst.len() {
        return None;
    }
    let (mut mdl, mut off, mut got) = (h.CurrentMdl, h.CurrentMdlOffset as usize, 0);
    while got < total && !mdl.is_null() {
        let va = MmGetSystemAddressForMdlSafe(mdl, _MM_PAGE_PRIORITY_NormalPagePriority) as *const u8;
        if va.is_null() {
            return None;
        }
        let n = ((*mdl).ByteCount as usize).saturating_sub(off).min(total - got);
        core::ptr::copy_nonoverlapping(va.add(off), dst.as_mut_ptr().add(got), n);
        got += n;
        off = 0;
        mdl = (*mdl).Next;
    }
    (got == total).then_some(total)
}

/// The stack's frames: copied into the ring for the worker, and completed at once.
unsafe extern "system" fn send(ctx: NdisHandle, nbls: *mut NetBufferList, _port: u32, flags: u32) {
    let a = ctx as *mut Adapter;
    let status = if (*a).paused.load(Ordering::Acquire) {
        NDIS_STATUS_PAUSED
    } else if !(*a).connected.load(Ordering::Acquire) {
        NDIS_STATUS_MEDIA_DISCONNECTED
    } else {
        NDIS_STATUS_SUCCESS
    };
    let mut frame = [0u8; ETH_FRAME_MAX];
    let mut queued = false;
    let mut nbl = nbls;
    while !nbl.is_null() {
        (*nbl).Status = status;
        if status == NDIS_STATUS_SUCCESS {
            let mut nb = (*nbl).u.h.FirstNetBuffer;
            while !nb.is_null() {
                let ok = match copy_nb(nb, &mut frame) {
                    Some(n) => {
                        let _g = SpinLockGuard::acquire(addr_of_mut!((*a).tx_lock));
                        (*a).tx.push(&frame[..n])
                    }
                    None => false,
                };
                if ok {
                    queued = true;
                } else {
                    (*a).refused.fetch_add(1, Ordering::Relaxed);
                    (*nbl).Status = NDIS_STATUS_RESOURCES;
                }
                nb = (*nb).u.h.Next;
            }
        }
        nbl = (*nbl).u.h.Next;
    }
    if queued {
        KeSetEvent(addr_of_mut!((*a).wake), 0, 0);
    }
    let done = if flags & NDIS_SEND_FLAGS_DISPATCH_LEVEL != 0 { NDIS_SEND_COMPLETE_FLAGS_DISPATCH_LEVEL } else { 0 };
    NdisMSendNetBufferListsComplete((*a).handle, nbls, done);
}

/// Receives are indicated with RESOURCES: no NBL ever comes back.
unsafe extern "system" fn return_nbls(_ctx: NdisHandle, _nbls: *mut NetBufferList, _flags: u32) {}

/// Hand one received frame to the protocols. At DISPATCH_LEVEL, where NDIS delivers
/// receives and where its filter library takes its locks.
unsafe fn indicate(a: *mut Adapter, eth: &[u8]) {
    if (*a).paused.load(Ordering::Acquire) || eth.len() > ETH_FRAME_MAX {
        return;
    }
    core::ptr::copy_nonoverlapping(eth.as_ptr(), (*a).rx_buf, eth.len());
    let nb = (*(*a).rx_nbl).u.h.FirstNetBuffer;
    (*nb).u.h.DataLength = eth.len();
    (*(*a).rx_mdl).ByteCount = eth.len() as u32;
    let _irql = IrqlGuard::raise_to_dpc();
    NdisMIndicateReceiveNetBufferLists(
        (*a).handle,
        (*a).rx_nbl,
        0,
        1,
        NDIS_RECEIVE_FLAGS_RESOURCES | NDIS_RECEIVE_FLAGS_DISPATCH_LEVEL,
    );
}

/// Tell the protocols the link came or went.
unsafe fn indicate_link(a: *mut Adapter, up: bool, speed: u64) {
    (*a).connected.store(up, Ordering::Release);
    let mut ls: LinkState = core::mem::zeroed();
    ls.Header = header(NDIS_OBJECT_TYPE_DEFAULT, core::mem::size_of::<LinkState>());
    ls.MediaConnectState = if up { MEDIA_CONNECT_STATE_CONNECTED } else { MEDIA_CONNECT_STATE_DISCONNECTED };
    ls.MediaDuplexState = MEDIA_DUPLEX_STATE_FULL;
    ls.XmitLinkSpeed = speed;
    ls.RcvLinkSpeed = speed;
    let mut s: StatusIndication = core::mem::zeroed();
    s.Header = header(NDIS_OBJECT_TYPE_STATUS_INDICATION, core::mem::size_of::<StatusIndication>());
    s.SourceHandle = (*a).handle;
    s.StatusCode = NDIS_STATUS_LINK_STATE;
    s.StatusBuffer = addr_of_mut!(ls).cast();
    s.StatusBufferSize = core::mem::size_of::<LinkState>() as u32;
    NdisMIndicateStatusEx((*a).handle, &mut s);
}

/// Sleep until a frame is queued, halt begins, or `ms` pass. True at halt.
unsafe fn wait(a: *mut Adapter, ms: i64) -> bool {
    let mut t: LARGE_INTEGER = core::mem::zeroed();
    t.QuadPart = -ms * 10_000;
    KeWaitForSingleObject(addr_of_mut!((*a).wake).cast(), _KWAIT_REASON_Executive, 0, 0, &mut t);
    (*a).stopping.load(Ordering::Acquire)
}

/// Sleep `s` seconds, waking only for halt (queued frames are dropped meanwhile: there
/// is no link to send them on). True at halt.
unsafe fn wait_offline(a: *mut Adapter, s: u32) -> bool {
    let end = Kernel::now_us_static() + s as u64 * 1_000_000;
    while Kernel::now_us_static() < end {
        if wait(a, 500) {
            return true;
        }
        let _g = SpinLockGuard::acquire(addr_of_mut!((*a).tx_lock));
        (*a).tx.clear();
    }
    false
}

unsafe extern "system" fn worker(context: PVOID) {
    let a = context as *mut Adapter;
    if let (Some(bus), Some(cfg)) = ((*a).bus.as_mut(), (*a).cfg) {
        stay_joined(a, bus, &cfg);
    }
    KeSetEvent(addr_of_mut!((*a).worker_done), 0, 0);
}

/// Find the network, join it, carry its traffic; when the link fails or is lost, leave
/// and join again, waiting longer after each failure.
unsafe fn stay_joined(a: *mut Adapter, bus: &mut Bus<Kernel>, cfg: &WifiConfig) {
    let mut backoff = REJOIN_FIRST_S;
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        let mut station = None;
        if let Some(target) = driver::find(bus, cfg, attempt == 1) {
            let mut log = JoinLog::default();
            let t0 = Kernel::now_us_static();
            let r = bus.join_station(cfg, &target, &mut log, |_| {});
            let ms = Kernel::now_us_static().wrapping_sub(t0) / 1000;
            driver::log_join(cfg, &target, &log, &r.as_ref().map(|(e, _)| *e).map_err(|e| *e), ms);
            if let Ok((JoinEnd::Up, st)) = r {
                station = st;
            }
        }
        let lost = match station {
            Some(mut st) => {
                backoff = REJOIN_FIRST_S;
                carry(a, bus, &mut st)
            }
            None => false,
        };
        if (*a).stopping.load(Ordering::Acquire) {
            return;
        }
        // Whatever the firmware still holds of the old association goes first.
        let _ = bus.leave();
        bwprint!(
            "joining again in {} s (attempt {}, {})\n",
            backoff,
            attempt,
            if lost { "lost the link" } else { "failed" }
        );
        if wait_offline(a, backoff) {
            return;
        }
        if !lost {
            backoff = (backoff * 2).min(REJOIN_MAX_S);
        }
    }
}

/// The joined link: bring it up to the protocols, then move frames both ways until it
/// is lost (true) or halt begins (false).
unsafe fn carry(a: *mut Adapter, bus: &mut Bus<Kernel>, st: &mut Station) -> bool {
    let speed = match bus.rate() {
        Ok(r) if r > 0 => r,
        _ => DEFAULT_SPEED,
    };
    {
        // Frames queued while there was no link are stale.
        let _g = SpinLockGuard::acquire(addr_of_mut!((*a).tx_lock));
        (*a).tx.clear();
    }
    indicate_link(a, true, speed);
    bwprint!("link up at {} Mb/s: carrying traffic\n", speed / 1_000_000);
    let mut log = LinkLog::default();
    let mut frame = [0u8; ETH_FRAME_MAX];
    let deliver = move |eth: &[u8]| indicate(a, eth);
    let end = loop {
        if (*a).stopping.load(Ordering::Acquire) {
            break None;
        }
        let mut busy = false;
        let mut lost = None;
        // Everything the stack queued, then what the chip has.
        while lost.is_none() {
            let n = {
                let _g = SpinLockGuard::acquire(addr_of_mut!((*a).tx_lock));
                (*a).tx.pop(&mut frame)
            };
            let Some(n) = n else { break };
            busy = true;
            match bus.send_traffic(st, &mut log, &frame[..n], deliver) {
                Ok(Serviced::Lost(e, r)) => lost = Some((e, r)),
                Ok(_) => {}
                Err(e) => bwprint!("send failed: {:?}\n", e),
            }
        }
        if lost.is_some() {
            break lost;
        }
        match bus.service(st, &mut log, deliver) {
            Ok(Serviced::Lost(e, r)) => break Some((e, r)),
            Ok(Serviced::Busy) => busy = true,
            Ok(Serviced::Idle) => {}
            Err(e) => {
                bwprint!("receive failed: {:?}\n", e);
                busy = false;
            }
        }
        if !busy && wait(a, IDLE_POLL_MS) {
            break None;
        }
    };
    indicate_link(a, false, 0);
    bwprint!(
        "link down{}: {} frames delivered, {} group rekeys, {} EAPOL refused (last {:?}), {} sends refused\n",
        match end {
            Some(_) => " (the network ended it)",
            None => " (halt)",
        },
        log.delivered,
        log.rekeys,
        log.rejected,
        log.last_reject,
        (*a).refused.load(Ordering::Relaxed)
    );
    if let Some((e, r)) = end {
        bwprint!("firmware event {} reason {}\n", e, r);
    }
    end.is_some()
}
