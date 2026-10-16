//! NDIS 6 miniports under the NDIS 5 wrapper.
//!
//! Not a translation: ReactOS's ndis.sys is NDIS 5.1 and has no NDIS 6 source to port. A
//! driver that calls NdisMRegisterMiniportDriver gets an ordinary NDIS_M_DRIVER_BLOCK whose
//! 5.1 characteristics are the shims below, so PnP start, binding, request queueing and the
//! protocols all run the existing NDIS 5 code unchanged. The shims translate at the edge:
//!
//!   NDIS 5 side                         NDIS 6 miniport
//!   InitializeHandler              ->   InitializeHandlerEx, then RestartHandler
//!   MiniDoRequest (query/set)      ->   OidRequestHandler (some OIDs answered from attributes)
//!   SendPacketsHandler (NDIS_PACKET) -> SendNetBufferListsHandler (one NBL per packet)
//!   MiniIndicateData (lookahead)   <-   NdisMIndicateReceiveNetBufferLists
//!   MiniSendComplete               <-   NdisMSendNetBufferListsComplete
//!   MiniStatus (media connect)     <-   NdisMIndicateStatusEx (link state)
//!
//! The adapter's MiniportAdapterContext is our `Ndis6Adapter`, which carries the driver's
//! own context; the NDIS 5 code only ever hands it back to the shims.
//!
//! Layouts are the WDK's (ndis/nbl.h, miniport.h, oidrequest.h); the tree has no NDIS 6
//! headers to generate them from. The offsets asserted at the bottom are the amd64 ones.

use crate::prelude::*;
use crate::ndis::miniport::{
    MiniSendComplete, NdisInitializeWrapper, NdisMIndicateStatus, NdisMIndicateStatusComplete,
    NdisMQueryInformationComplete, NdisMRegisterMiniport, NdisMSetInformationComplete,
};
use crate::ndis::io::NdisMInitializeScatterGatherDma;
use crate::ndis::hardware::{NdisReadPciSlotInformation, NdisWritePciSlotInformation};
use crate::ndis::config::NdisOpenConfiguration;
use ntkrnl_sys::ddk::{
    ExFreePool, ExQueueWorkItem, IoAllocateMdl, IoConnectInterrupt, IoCreateDevice, IoCreateSymbolicLink,
    IoDeleteDevice, IoDeleteSymbolicLink, IoDisconnectInterrupt, IoFreeMdl, KeCancelTimer, KeInitializeDpc,
    KeInitializeEvent, KeInitializeTimer, KeInsertQueueDpc, KeRemoveQueueDpc, KeSetTimerEx,
    KeSynchronizeExecution, KeWaitForSingleObject, MmBuildMdlForNonPagedPool, MmGetPhysicalAddress,
};
use core::ptr::{null_mut, addr_of_mut};
use core::sync::atomic::{AtomicI32, AtomicPtr, Ordering};

// ---- layouts --------------------------------------------------------------------------------

/// DECLSPEC_ALIGN(MEMORY_ALLOCATION_ALIGNMENT): 16 on 64-bit, 8 on i386.
#[cfg(target_pointer_width = "64")]
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct MemAlign<T: Copy>(pub T);
#[cfg(target_pointer_width = "32")]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct MemAlign<T: Copy>(pub T);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NdisObjectHeader {
    pub Type: u8,
    pub Revision: u8,
    pub Size: u16,
}

pub const NDIS_OBJECT_TYPE_DEFAULT: u8 = 0x80;
pub const NDIS_OBJECT_TYPE_MINIPORT_INIT_PARAMETERS: u8 = 0x81;
pub const NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS: u8 = 0x8A;
pub const NDIS_OBJECT_TYPE_OID_REQUEST: u8 = 0x96;
pub const NDIS_OBJECT_TYPE_STATUS_INDICATION: u8 = 0x98;
pub const NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES: u8 = 0x9E;
pub const NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES: u8 = 0x9F;

/// The header of an SLIST_HEADER-sized union arm (Next/First on the list, Link when pooled).
#[cfg(target_pointer_width = "64")]
type SListHeader = MemAlign<[u64; 2]>;
#[cfg(target_pointer_width = "32")]
type SListHeader = MemAlign<[u64; 1]>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NbHead {
    pub Next: *mut NetBuffer,
    pub CurrentMdl: PMDL,
    pub CurrentMdlOffset: u32,
    pub DataLength: usize, // union { ULONG DataLength; SIZE_T stDataLength; }
    pub MdlChain: PMDL,
    pub DataOffset: u32,
}
#[repr(C)]
pub union NbHeadU {
    pub h: NbHead,
    pub Link: SListHeader,
}

#[repr(C)]
pub struct NetBuffer {
    pub u: NbHeadU,
    pub ChecksumBias: u16,
    pub Reserved: u16,
    pub NdisPoolHandle: *mut NbPool,
    pub NdisReserved: MemAlign<[PVOID; 2]>,
    pub ProtocolReserved: MemAlign<[PVOID; 6]>,
    pub MiniportReserved: MemAlign<[PVOID; 4]>,
    pub DataPhysicalAddress: i64,
    pub SharedMemoryInfo: PVOID, // NDIS 6.20: union with ScatterGatherList
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct NblHead {
    pub Next: *mut NetBufferList,
    pub FirstNetBuffer: *mut NetBuffer,
}
#[repr(C)]
pub union NblHeadU {
    pub h: NblHead,
    pub Link: SListHeader,
}

/// Slots in NetBufferListInfo. Windows 10 has about 20; nothing indexes past its own enum, so
/// a generous count costs only pool.
pub const NBL_INFO_SLOTS: usize = 32;

#[repr(C)]
pub struct NetBufferList {
    pub u: NblHeadU,
    pub Context: *mut NblContext,
    pub ParentNetBufferList: *mut NetBufferList,
    pub NdisPoolHandle: *mut NblPool,
    pub NdisReserved: MemAlign<[PVOID; 2]>,
    pub ProtocolReserved: MemAlign<[PVOID; 4]>,
    pub MiniportReserved: MemAlign<[PVOID; 2]>,
    pub Scratch: PVOID,
    pub SourceHandle: NDIS_HANDLE,
    pub NblFlags: u32,
    pub ChildRefCount: i32,
    pub Flags: u32,
    pub Status: i32,
    pub NetBufferListInfo: [PVOID; NBL_INFO_SLOTS],
}

#[repr(C)]
pub struct NblContext {
    pub Next: *mut NblContext,
    pub Size: u16,
    pub Offset: u16,
    pub ContextData: MemAlign<[u8; 0]>,
}

#[repr(C)]
pub struct NblPoolParameters {
    pub Header: NdisObjectHeader,
    pub ProtocolId: u8,
    pub fAllocateNetBuffer: u8,
    pub ContextSize: u16,
    pub PoolTag: u32,
    pub DataSize: u32,
}
#[repr(C)]
pub struct NbPoolParameters {
    pub Header: NdisObjectHeader,
    pub PoolTag: u32,
    pub DataSize: u32,
}

#[repr(C)]
pub struct MiniportDriverCharacteristics {
    pub Header: NdisObjectHeader,
    pub MajorNdisVersion: u8,
    pub MinorNdisVersion: u8,
    pub MajorDriverVersion: u8,
    pub MinorDriverVersion: u8,
    pub Flags: u32,
    pub SetOptionsHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, NDIS_HANDLE) -> NDIS_STATUS>,
    pub InitializeHandlerEx: Option<unsafe extern "system" fn(NDIS_HANDLE, NDIS_HANDLE, *mut MiniportInitParameters) -> NDIS_STATUS>,
    pub HaltHandlerEx: Option<unsafe extern "system" fn(NDIS_HANDLE, u32)>,
    pub UnloadHandler: Option<unsafe extern "system" fn(PDRIVER_OBJECT)>,
    pub PauseHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID) -> NDIS_STATUS>,
    pub RestartHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut MiniportRestartParameters) -> NDIS_STATUS>,
    pub OidRequestHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut NdisOidRequest) -> NDIS_STATUS>,
    pub SendNetBufferListsHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut NetBufferList, u32, u32)>,
    pub ReturnNetBufferListsHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut NetBufferList, u32)>,
    pub CancelSendHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID)>,
    pub CheckForHangHandlerEx: Option<unsafe extern "system" fn(NDIS_HANDLE) -> BOOLEAN>,
    pub ResetHandlerEx: Option<unsafe extern "system" fn(NDIS_HANDLE, PBOOLEAN) -> NDIS_STATUS>,
    pub DevicePnPEventNotifyHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID)>,
    pub ShutdownHandlerEx: Option<unsafe extern "system" fn(NDIS_HANDLE, u32)>,
    pub CancelOidRequestHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID)>,
    // NDIS 6.1+
    pub DirectOidRequestHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, *mut NdisOidRequest) -> NDIS_STATUS>,
    pub CancelDirectOidRequestHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID)>,
    // NDIS 6.80: SynchronousOidRequestHandler.
}

#[repr(C)]
pub struct MiniportInitParameters {
    pub Header: NdisObjectHeader,
    pub Flags: u32,
    pub AllocatedResources: PVOID, // PNDIS_RESOURCE_LIST == PCM_PARTIAL_RESOURCE_LIST, translated
    pub IMDeviceInstanceContext: NDIS_HANDLE,
    pub MiniportAddDeviceContext: NDIS_HANDLE,
    pub IfIndex: u32,
    pub NetLuid: u64,
    pub DefaultPortAuthStates: PVOID,
    pub PciDeviceCustomProperties: PVOID,
}

#[repr(C)]
pub struct MiniportRestartParameters {
    pub Header: NdisObjectHeader,
    pub RestartAttributes: PVOID,
    pub Flags: u32,
}

#[repr(C)]
pub struct MiniportPauseParameters {
    pub Header: NdisObjectHeader,
    pub Flags: u32,
    pub PauseReason: u32,
}

#[repr(C)]
pub struct RegistrationAttributes {
    pub Header: NdisObjectHeader,
    pub MiniportAdapterContext: NDIS_HANDLE,
    pub AttributeFlags: u32,
    pub CheckForHangTimeInSeconds: u32,
    pub InterfaceType: u32,
}

#[repr(C)]
pub struct GeneralAttributes {
    pub Header: NdisObjectHeader,
    pub Flags: u32,
    pub MediaType: u32,
    pub PhysicalMediumType: u32,
    pub MtuSize: u32,
    pub MaxXmitLinkSpeed: u64,
    pub XmitLinkSpeed: u64,
    pub MaxRcvLinkSpeed: u64,
    pub RcvLinkSpeed: u64,
    pub MediaConnectState: u32,
    pub MediaDuplexState: u32,
    pub LookaheadSize: u32,
    pub PowerManagementCapabilities: PVOID,
    pub MacOptions: u32,
    pub SupportedPacketFilters: u32,
    pub MaxMulticastListSize: u32,
    pub MacAddressLength: u16,
    pub PermanentMacAddress: [u8; 32],
    pub CurrentMacAddress: [u8; 32],
    // RecvScaleCapabilities and the interface fields follow; nothing here reads them.
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OidQuery {
    pub Oid: u32,
    pub InformationBuffer: PVOID,
    pub InformationBufferLength: u32,
    pub BytesWritten: u32, // BytesRead for a set
    pub BytesNeeded: u32,
}
#[repr(C)]
pub union OidData {
    pub q: OidQuery,
    pub method: [u8; 40],
}

pub const NDIS_OID_REQUEST_NDIS_RESERVED_SIZE: usize = 16;

#[repr(C)]
pub struct NdisOidRequest {
    pub Header: NdisObjectHeader,
    pub RequestType: u32,
    pub PortNumber: u32,
    pub Timeout: u32,
    pub RequestId: PVOID,
    pub RequestHandle: NDIS_HANDLE,
    pub DATA: OidData,
    pub NdisReserved: [PVOID; NDIS_OID_REQUEST_NDIS_RESERVED_SIZE],
    pub MiniportReserved: [PVOID; 2],
    pub SourceReserved: [PVOID; 2],
    pub SupportedRevision: u8,
    pub Reserved1: u8,
    pub Reserved2: u16,
    // NDIS 6.50 adds SwitchId, VPortId, Flags: room for them, zeroed.
    pub Tail: [u32; 8],
}

#[repr(C)]
pub struct StatusIndication {
    pub Header: NdisObjectHeader,
    pub SourceHandle: NDIS_HANDLE,
    pub PortNumber: u32,
    pub StatusCode: NDIS_STATUS,
    pub Flags: u32,
    pub DestinationHandle: NDIS_HANDLE,
    pub RequestId: PVOID,
    pub StatusBuffer: PVOID,
    pub StatusBufferSize: u32,
}

#[repr(C)]
pub struct LinkState {
    pub Header: NdisObjectHeader,
    pub MediaConnectState: u32,
    pub MediaDuplexState: u32,
    pub XmitLinkSpeed: u64,
    pub RcvLinkSpeed: u64,
}

#[repr(C)]
pub struct InterruptCharacteristics {
    pub Header: NdisObjectHeader,
    pub InterruptHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PBOOLEAN, *mut u32) -> BOOLEAN>,
    pub InterruptDpcHandler: Option<unsafe extern "system" fn(NDIS_HANDLE, PVOID, PVOID, PVOID)>,
    pub DisableInterruptHandler: Option<unsafe extern "system" fn(NDIS_HANDLE)>,
    pub EnableInterruptHandler: Option<unsafe extern "system" fn(NDIS_HANDLE)>,
    pub MsiSupported: BOOLEAN,
    pub MsiSyncWithAllMessages: BOOLEAN,
    pub MessageInterruptHandler: PVOID,
    pub MessageInterruptDpcHandler: PVOID,
    pub DisableMessageInterruptHandler: PVOID,
    pub EnableMessageInterruptHandler: PVOID,
    pub InterruptType: u32,
    pub MessageInfoTable: PVOID,
}
pub const NDIS_CONNECT_LINE_BASED: u32 = 1;

#[repr(C)]
pub struct SgDmaDescription {
    pub Header: NdisObjectHeader,
    pub Flags: u32,
    pub MaximumPhysicalMapping: u32,
    pub ProcessSGListHandler: Option<unsafe extern "system" fn(PDEVICE_OBJECT, PVOID, *mut SgList, PVOID)>,
    pub SharedMemAllocateCompleteHandler: PVOID,
    pub ScatterGatherListSize: u32,
}
pub const NDIS_SG_DMA_64_BIT_ADDRESS: u32 = 0x1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SgElement {
    pub Address: i64,
    pub Length: u32,
    pub Reserved: usize,
}
#[repr(C)]
pub struct SgList {
    pub NumberOfElements: u32,
    pub Reserved: usize,
    pub Elements: [SgElement; 0],
}

#[repr(C)]
pub struct TimerCharacteristics {
    pub Header: NdisObjectHeader,
    pub AllocationTag: u32,
    pub TimerFunction: Option<unsafe extern "system" fn(PVOID, PVOID, PVOID, PVOID)>,
    pub FunctionContext: PVOID,
}

#[repr(C)]
pub struct ConfigurationObject {
    pub Header: NdisObjectHeader,
    pub NdisHandle: NDIS_HANDLE,
    pub Flags: u32,
}

#[repr(C)]
pub struct DeviceObjectAttributes {
    pub Header: NdisObjectHeader,
    pub DeviceName: PNDIS_STRING,
    pub SymbolicName: PNDIS_STRING,
    pub MajorFunctions: *mut PDRIVER_DISPATCH,
    pub ExtensionSize: u32,
    pub DefaultSDDLString: PVOID,
    pub DeviceClassGuid: PVOID,
}

#[repr(C)]
pub struct LockStateEx {
    pub OldIrql: KIRQL,
    pub LockState: u8,
    pub Flags: u8,
}

// NDIS 6 status / flag values used here.
// ndis.h values the bindgen island leaves out.
const NDIS_STATUS_INVALID_LENGTH: NDIS_STATUS = 0xC001_0014u32 as NDIS_STATUS;
const NDIS_STATUS_INVALID_PARAMETER: NDIS_STATUS = 0xC000_000Du32 as NDIS_STATUS; // STATUS_INVALID_PARAMETER
const NDIS_STATUS_LINK_STATE: NDIS_STATUS = 0x4001_0017;
const NDIS_STATUS_MEDIA_CONNECT_: NDIS_STATUS = 0x4001_000B;
const NDIS_STATUS_MEDIA_DISCONNECT_: NDIS_STATUS = 0x4001_000C;
pub const MEDIA_CONNECT_STATE_CONNECTED: u32 = 1;
const NDIS_SEND_FLAGS_DISPATCH_LEVEL: u32 = 0x1;
const NDIS_RETURN_FLAGS_DISPATCH_LEVEL: u32 = 0x1;
pub const NDIS_RECEIVE_FLAGS_RESOURCES: u32 = 0x2;
const NDIS_RWL_AT_DISPATCH_LEVEL: u8 = 0x1;
const NDIS6_TAG: u32 = u32::from_le_bytes(*b"6SDN");

// ---- per-driver and per-adapter state ---------------------------------------------------------

#[repr(C)]
pub struct Ndis6Driver {
    pub block: PNDIS_M_DRIVER_BLOCK,
    pub driver_context: NDIS_HANDLE,
    pub chars: MiniportDriverCharacteristics,
}

#[repr(C)]
pub struct Ndis6Adapter {
    pub adapter: PLOGICAL_ADAPTER,
    pub drv: *mut Ndis6Driver,
    /// The miniport's own context, from its registration attributes.
    pub ctx: NDIS_HANDLE,
    pub gen: GeneralAttributes,
    pub have_gen: bool,
    pub link_up: bool,
    /// Set across InitializeHandlerEx: NdisOpenConfigurationEx needs a wrapper context.
    pub wrapper_ctx: NDIS_HANDLE,
    pub sg: *mut SgDma,
    pub restart_event: KEVENT,
    pub restart_status: NDIS_STATUS,
    /// The NDIS 5 request a pended OID request answers.
    pub pending_req: AtomicPtr<NDIS_REQUEST>,
    pub send_pool: *mut NblPool,
}

const MAX_DRIVERS: usize = 8;
const MAX_ADAPTERS: usize = 8;
static DRIVERS: [AtomicPtr<Ndis6Driver>; MAX_DRIVERS] = [const { AtomicPtr::new(null_mut()) }; MAX_DRIVERS];
static ADAPTERS: [AtomicPtr<Ndis6Adapter>; MAX_ADAPTERS] = [const { AtomicPtr::new(null_mut()) }; MAX_ADAPTERS];

unsafe fn driver_of(block: PNDIS_M_DRIVER_BLOCK) -> *mut Ndis6Driver {
    for d in DRIVERS.iter() {
        let p = d.load(Ordering::Acquire);
        if !p.is_null() && (*p).block == block {
            return p;
        }
    }
    null_mut()
}

/// The NDIS 6 state of an adapter, or null for an NDIS 5 miniport.
pub unsafe fn adapter_of(adapter: PLOGICAL_ADAPTER) -> *mut Ndis6Adapter {
    for a in ADAPTERS.iter() {
        let p = a.load(Ordering::Acquire);
        if !p.is_null() && (*p).adapter == adapter {
            return p;
        }
    }
    null_mut()
}

/// The driver block behind any handle an NDIS 6 driver holds: its NdisMiniportDriverHandle,
/// or an adapter handle (drivers pass either where "an NDIS handle" is asked for).
unsafe fn driver_block_of(handle: NDIS_HANDLE) -> PNDIS_M_DRIVER_BLOCK {
    for d in DRIVERS.iter() {
        let p = d.load(Ordering::Acquire);
        if !p.is_null() && p as NDIS_HANDLE == handle {
            return (*p).block;
        }
    }
    let a6 = adapter_of(handle as PLOGICAL_ADAPTER);
    if !a6.is_null() {
        return (*(*a6).adapter).NdisMiniportBlock.DriverHandle;
    }
    null_mut()
}

fn insert<T>(table: &[AtomicPtr<T>], p: *mut T) -> bool {
    table.iter().any(|slot| slot.compare_exchange(null_mut(), p, Ordering::AcqRel, Ordering::Acquire).is_ok())
}

unsafe fn alloc_zeroed(size: usize) -> PVOID {
    let p = ExAllocatePoolWithTag(NonPagedPool, size as SIZE_T, NDIS6_TAG);
    if !p.is_null() {
        core::ptr::write_bytes(p as *mut u8, 0, size);
    }
    p
}

macro_rules! log6 {
    ($fmt:literal $(, $a:expr)* $(,)?) => {
        { ntkrnl_sys::driver_ddk::DbgPrint(concat!("[NDIS6] ", $fmt, "\0").as_ptr() as *const i8 $(, $a)*); }
    };
}

// ---- registration ----------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterMiniportDriver(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
    MiniportDriverContext: NDIS_HANDLE,
    Characteristics: *mut MiniportDriverCharacteristics,
    NdisMiniportDriverHandle: *mut NDIS_HANDLE,
) -> NDIS_STATUS {
    let c = &*Characteristics;
    log6!("%wZ registers an NDIS %u.%u miniport (driver %u.%u)\n", RegistryPath,
        c.MajorNdisVersion as u32, c.MinorNdisVersion as u32,
        c.MajorDriverVersion as u32, c.MinorDriverVersion as u32);
    if c.Header.Type != NDIS_OBJECT_TYPE_MINIPORT_DRIVER_CHARACTERISTICS || c.MajorNdisVersion != 6 {
        log6!("bad characteristics: type %x, version %u\n", c.Header.Type as u32, c.MajorNdisVersion as u32);
        return NDIS_STATUS_BAD_VERSION;
    }
    if c.InitializeHandlerEx.is_none() || c.HaltHandlerEx.is_none() || c.OidRequestHandler.is_none()
        || c.SendNetBufferListsHandler.is_none() || c.ReturnNetBufferListsHandler.is_none()
        || c.PauseHandler.is_none() || c.RestartHandler.is_none()
    {
        log6!("a mandatory handler is missing\n");
        return NDIS_STATUS_BAD_CHARACTERISTICS;
    }

    let mut wrapper: NDIS_HANDLE = null_mut();
    NdisInitializeWrapper(addr_of_mut!(wrapper), DriverObject as PVOID, RegistryPath as PVOID, null_mut());
    if wrapper.is_null() {
        return NDIS_STATUS_RESOURCES;
    }

    let drv = alloc_zeroed(core::mem::size_of::<Ndis6Driver>()) as *mut Ndis6Driver;
    if drv.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*drv).block = wrapper as PNDIS_M_DRIVER_BLOCK;
    (*drv).driver_context = MiniportDriverContext;
    let n = core::cmp::min(c.Header.Size as usize, core::mem::size_of::<MiniportDriverCharacteristics>());
    core::ptr::copy_nonoverlapping(Characteristics as *const u8, addr_of_mut!((*drv).chars) as *mut u8, n);
    if !insert(&DRIVERS, drv) {
        log6!("too many NDIS 6 drivers\n");
        ExFreePool(drv as PVOID);
        return NDIS_STATUS_RESOURCES;
    }

    let mut ch: NDIS51_MINIPORT_CHARACTERISTICS = core::mem::zeroed();
    ch.MajorNdisVersion = 5;
    ch.MinorNdisVersion = 1;
    ch.InitializeHandler = Some(shim_initialize);
    ch.HaltHandler = Some(shim_halt);
    ch.ResetHandler = Some(shim_reset);
    ch.CheckForHangHandler = Some(shim_check_for_hang);
    ch.QueryInformationHandler = Some(shim_query);
    ch.SetInformationHandler = Some(shim_set);
    ch.SendPacketsHandler = Some(shim_send_packets);
    ch.TransferDataHandler = Some(shim_transfer_data);
    ch.AdapterShutdownHandler = Some(shim_shutdown);
    let status = NdisMRegisterMiniport(
        wrapper,
        addr_of_mut!(ch) as PNDIS_MINIPORT_CHARACTERISTICS,
        core::mem::size_of::<NDIS51_MINIPORT_CHARACTERISTICS>() as UINT,
    );
    if status != NDIS_STATUS_SUCCESS {
        log6!("NdisMRegisterMiniport failed %x\n", status);
        return status;
    }
    *NdisMiniportDriverHandle = drv as NDIS_HANDLE;

    if let Some(set_options) = (*drv).chars.SetOptionsHandler {
        let s = set_options(drv as NDIS_HANDLE, MiniportDriverContext);
        if s != NDIS_STATUS_SUCCESS {
            log6!("SetOptions failed %x\n", s);
            return s;
        }
    }
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterMiniportDriver(NdisMiniportDriverHandle: NDIS_HANDLE) {
    let drv = NdisMiniportDriverHandle as *mut Ndis6Driver;
    for slot in DRIVERS.iter() {
        let _ = slot.compare_exchange(drv, null_mut(), Ordering::AcqRel, Ordering::Acquire);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetOptionalHandlers(_NdisHandle: NDIS_HANDLE, OptionalHandlers: *mut NdisObjectHeader) -> NDIS_STATUS {
    // Offload, chimney, PM and the like: ours never call them, so accepting is enough.
    log6!("optional handlers type %x accepted, unused\n", (*OptionalHandlers).Type as u32);
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSetMiniportAttributes(
    NdisMiniportHandle: NDIS_HANDLE,
    Attributes: *mut NdisObjectHeader,
) -> NDIS_STATUS {
    let a6 = adapter_of(NdisMiniportHandle as PLOGICAL_ADAPTER);
    if a6.is_null() {
        return NDIS_STATUS_INVALID_PARAMETER;
    }
    if crate::ndis::wdi::refuse_attributes(NdisMiniportHandle, (*Attributes).Type) {
        return NDIS_STATUS_SUCCESS;
    }
    match (*Attributes).Type {
        NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_REGISTRATION_ATTRIBUTES => {
            let r = &*(Attributes as *const RegistrationAttributes);
            (*a6).ctx = r.MiniportAdapterContext;
            let block = &mut (*(*a6).adapter).NdisMiniportBlock;
            if r.CheckForHangTimeInSeconds != 0 {
                block.CheckForHangSeconds = r.CheckForHangTimeInSeconds as _;
            }
            log6!("registration: context %p, flags %x, hang check %us\n", r.MiniportAdapterContext,
                r.AttributeFlags, r.CheckForHangTimeInSeconds);
        }
        NDIS_OBJECT_TYPE_MINIPORT_ADAPTER_GENERAL_ATTRIBUTES => {
            core::ptr::copy_nonoverlapping(
                Attributes as *const u8,
                addr_of_mut!((*a6).gen) as *mut u8,
                core::mem::size_of::<GeneralAttributes>(),
            );
            (*a6).have_gen = true;
            (*a6).link_up = (*a6).gen.MediaConnectState == MEDIA_CONNECT_STATE_CONNECTED;
            let m = &(*a6).gen.CurrentMacAddress;
            log6!("general: medium %u, MTU %u, lookahead %u, MAC %02x:%02x:%02x:%02x:%02x:%02x, link %u\n",
                (*a6).gen.MediaType, (*a6).gen.MtuSize, (*a6).gen.LookaheadSize,
                m[0] as u32, m[1] as u32, m[2] as u32, m[3] as u32, m[4] as u32, m[5] as u32,
                (*a6).gen.MediaConnectState);
        }
        t => log6!("attributes type %x accepted, unused\n", t as u32),
    }
    NDIS_STATUS_SUCCESS
}

// ---- the NDIS 5.1 shims --------------------------------------------------------------------

unsafe extern "system" fn shim_initialize(
    OpenErrorStatus: PNDIS_STATUS,
    SelectedMediumIndex: PUINT,
    MediumArray: PNDIS_MEDIUM,
    MediumArraySize: UINT,
    MiniportAdapterHandle: NDIS_HANDLE,
    WrapperConfigurationContext: NDIS_HANDLE,
) -> NDIS_STATUS {
    let adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    *OpenErrorStatus = NDIS_STATUS_SUCCESS;
    let drv = driver_of((*adapter).NdisMiniportBlock.DriverHandle);
    if drv.is_null() {
        return NDIS_STATUS_FAILURE;
    }
    let a6 = alloc_zeroed(core::mem::size_of::<Ndis6Adapter>()) as *mut Ndis6Adapter;
    if a6.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*a6).adapter = adapter;
    (*a6).drv = drv;
    (*a6).wrapper_ctx = WrapperConfigurationContext;
    KeInitializeEvent(addr_of_mut!((*a6).restart_event), _EVENT_TYPE_NotificationEvent, FALSE as BOOLEAN);
    if !insert(&ADAPTERS, a6) {
        ExFreePool(a6 as PVOID);
        return NDIS_STATUS_RESOURCES;
    }
    // NDIS 6 miniports are deserialized by definition.
    (*adapter).NdisMiniportBlock.MiniportAdapterContext = a6 as NDIS_HANDLE;
    (*adapter).NdisMiniportBlock.Flags |= NDIS_ATTRIBUTE_DESERIALIZE as ULONG;

    let mut params: MiniportInitParameters = core::mem::zeroed();
    params.Header = NdisObjectHeader {
        Type: NDIS_OBJECT_TYPE_MINIPORT_INIT_PARAMETERS,
        Revision: 1,
        Size: core::mem::size_of::<MiniportInitParameters>() as u16,
    };
    let translated = (*adapter).NdisMiniportBlock.AllocatedResourcesTranslated;
    if !translated.is_null() {
        params.AllocatedResources = addr_of_mut!((*translated).List[0].PartialResourceList) as PVOID;
    }
    params.IfIndex = 1;

    let init = (*drv).chars.InitializeHandlerEx.unwrap_unchecked();
    log6!("InitializeHandlerEx\n");
    let status = init(adapter as NDIS_HANDLE, (*drv).driver_context, addr_of_mut!(params));
    (*a6).wrapper_ctx = null_mut();
    if status != NDIS_STATUS_SUCCESS {
        log6!("InitializeHandlerEx failed %x\n", status);
        remove_adapter(a6);
        return status;
    }
    if (*a6).ctx.is_null() || !(*a6).have_gen {
        log6!("the miniport set no %s attributes\n",
            if (*a6).ctx.is_null() { b"registration\0".as_ptr() } else { b"general\0".as_ptr() });
        return NDIS_STATUS_FAILURE;
    }

    let mut found = MediumArraySize;
    for i in 0..MediumArraySize {
        if *MediumArray.add(i as usize) as u32 == (*a6).gen.MediaType {
            found = i;
            break;
        }
    }
    if found == MediumArraySize {
        log6!("medium %u is not one NDIS 5 offers\n", (*a6).gen.MediaType);
        return NDIS_STATUS_UNSUPPORTED_MEDIA;
    }
    *SelectedMediumIndex = found;

    (*a6).send_pool = new_nbl_pool(NDIS6_TAG, true, 0);
    if (*a6).send_pool.is_null() {
        return NDIS_STATUS_RESOURCES;
    }

    // A miniport comes up Paused; NDIS restarts it before any traffic.
    let mut rp: MiniportRestartParameters = core::mem::zeroed();
    rp.Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<MiniportRestartParameters>() as u16 };
    KeClearEvent(addr_of_mut!((*a6).restart_event));
    let mut status = ((*drv).chars.RestartHandler.unwrap_unchecked())((*a6).ctx, addr_of_mut!(rp));
    if status == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(addr_of_mut!((*a6).restart_event) as PVOID, _KWAIT_REASON_Executive, _MODE_KernelMode as _, FALSE as BOOLEAN, null_mut());
        status = (*a6).restart_status;
    }
    if status != NDIS_STATUS_SUCCESS {
        log6!("RestartHandler failed %x\n", status);
        return status;
    }
    log6!("adapter %p running\n", adapter);
    NDIS_STATUS_SUCCESS
}

unsafe fn remove_adapter(a6: *mut Ndis6Adapter) {
    for slot in ADAPTERS.iter() {
        let _ = slot.compare_exchange(a6, null_mut(), Ordering::AcqRel, Ordering::Acquire);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisMRestartComplete(MiniportAdapterHandle: NDIS_HANDLE, Status: NDIS_STATUS) {
    let a6 = adapter_of(MiniportAdapterHandle as PLOGICAL_ADAPTER);
    if !a6.is_null() {
        (*a6).restart_status = Status;
        KeSetEvent(addr_of_mut!((*a6).restart_event), 0, FALSE as BOOLEAN);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisMPauseComplete(MiniportAdapterHandle: NDIS_HANDLE) {
    NdisMRestartComplete(MiniportAdapterHandle, NDIS_STATUS_SUCCESS);
}

unsafe extern "system" fn shim_halt(context: NDIS_HANDLE) {
    let a6 = context as *mut Ndis6Adapter;
    let c = &(*(*a6).drv).chars;
    let mut pp = MiniportPauseParameters {
        Header: NdisObjectHeader { Type: NDIS_OBJECT_TYPE_DEFAULT, Revision: 1, Size: core::mem::size_of::<MiniportPauseParameters>() as u16 },
        Flags: 0,
        PauseReason: 0x4, // NDIS_PAUSE_MINIPORT_DEVICE_REMOVE
    };
    KeClearEvent(addr_of_mut!((*a6).restart_event));
    if (c.PauseHandler.unwrap_unchecked())((*a6).ctx, addr_of_mut!(pp) as PVOID) == NDIS_STATUS_PENDING {
        KeWaitForSingleObject(addr_of_mut!((*a6).restart_event) as PVOID, _KWAIT_REASON_Executive, _MODE_KernelMode as _, FALSE as BOOLEAN, null_mut());
    }
    (c.HaltHandlerEx.unwrap_unchecked())((*a6).ctx, 1 /* NdisHaltDeviceDisabled */);
    remove_adapter(a6);
}

unsafe extern "system" fn shim_shutdown(context: PVOID) {
    let a6 = context as *mut Ndis6Adapter;
    if let Some(f) = (*(*a6).drv).chars.ShutdownHandlerEx {
        f((*a6).ctx, 0 /* NdisShutdownPowerOff */);
    }
}

unsafe extern "system" fn shim_reset(AddressingReset: PBOOLEAN, context: NDIS_HANDLE) -> NDIS_STATUS {
    let a6 = context as *mut Ndis6Adapter;
    match (*(*a6).drv).chars.ResetHandlerEx {
        Some(f) => f((*a6).ctx, AddressingReset),
        None => NDIS_STATUS_SUCCESS,
    }
}

unsafe extern "system" fn shim_check_for_hang(context: NDIS_HANDLE) -> BOOLEAN {
    let a6 = context as *mut Ndis6Adapter;
    match (*(*a6).drv).chars.CheckForHangHandlerEx {
        Some(f) => f((*a6).ctx),
        None => FALSE as BOOLEAN,
    }
}

// MiniDoRequest routes an NDIS 6 adapter to `do_request` before these could be reached; they
// exist because NdisMRegisterMiniport insists on both.
unsafe extern "system" fn shim_query(_: NDIS_HANDLE, _: NDIS_OID, _: PVOID, _: ULONG, _: PULONG, _: PULONG) -> NDIS_STATUS {
    NDIS_STATUS_NOT_SUPPORTED
}
unsafe extern "system" fn shim_set(_: NDIS_HANDLE, _: NDIS_OID, _: PVOID, _: ULONG, _: PULONG, _: PULONG) -> NDIS_STATUS {
    NDIS_STATUS_NOT_SUPPORTED
}

// ---- OID requests ----------------------------------------------------------------------------

unsafe fn put_u32(buf: PVOID, len: u32, written: *mut u32, needed: *mut u32, v: u32) -> NDIS_STATUS {
    put_bytes(buf, len, written, needed, &v.to_le_bytes())
}

unsafe fn put_bytes(buf: PVOID, len: u32, written: *mut u32, needed: *mut u32, v: &[u8]) -> NDIS_STATUS {
    if (len as usize) < v.len() {
        *needed = v.len() as u32;
        return NDIS_STATUS_INVALID_LENGTH;
    }
    core::ptr::copy_nonoverlapping(v.as_ptr(), buf as *mut u8, v.len());
    *written = v.len() as u32;
    NDIS_STATUS_SUCCESS
}

/// The OIDs NDIS 6 answers itself from the general attributes: a 6.x miniport is entitled
/// never to see them.
unsafe fn answer_from_attributes(a6: *mut Ndis6Adapter, oid: u32, buf: PVOID, len: u32, written: *mut u32, needed: *mut u32) -> Option<NDIS_STATUS> {
    let g = &(*a6).gen;
    let mac = &g.CurrentMacAddress[..g.MacAddressLength.min(32) as usize];
    let perm = &g.PermanentMacAddress[..g.MacAddressLength.min(32) as usize];
    let s = match oid {
        OID_GEN_MEDIA_SUPPORTED | OID_GEN_MEDIA_IN_USE => put_u32(buf, len, written, needed, g.MediaType),
        OID_GEN_MAXIMUM_LOOKAHEAD | OID_GEN_CURRENT_LOOKAHEAD => {
            let la = if g.LookaheadSize != 0 { g.LookaheadSize } else { g.MtuSize };
            put_u32(buf, len, written, needed, la)
        }
        OID_GEN_MAXIMUM_FRAME_SIZE => put_u32(buf, len, written, needed, g.MtuSize),
        OID_GEN_MAXIMUM_TOTAL_SIZE => put_u32(buf, len, written, needed, g.MtuSize + 14),
        OID_GEN_LINK_SPEED => put_u32(buf, len, written, needed, (g.XmitLinkSpeed / 100) as u32),
        OID_GEN_MEDIA_CONNECT_STATUS => put_u32(buf, len, written, needed,
            if (*a6).link_up { _NDIS_MEDIA_STATE_NdisMediaStateConnected as u32 } else { _NDIS_MEDIA_STATE_NdisMediaStateDisconnected as u32 }),
        OID_GEN_MAC_OPTIONS => put_u32(buf, len, written, needed, g.MacOptions),
        OID_GEN_MAXIMUM_SEND_PACKETS => put_u32(buf, len, written, needed, 1),
        OID_802_3_CURRENT_ADDRESS => put_bytes(buf, len, written, needed, mac),
        OID_802_3_PERMANENT_ADDRESS => put_bytes(buf, len, written, needed, perm),
        OID_802_3_MAXIMUM_LIST_SIZE => put_u32(buf, len, written, needed, g.MaxMulticastListSize),
        _ => return None,
    };
    Some(s)
}

/// MiniDoRequest for an NDIS 6 adapter. Runs at the caller's IRQL: an NDIS 6 OID handler
/// expects PASSIVE_LEVEL, where NDIS 5 raised to DISPATCH.
pub unsafe fn do_request(a6: *mut Ndis6Adapter, req: PNDIS_REQUEST) -> NDIS_STATUS {
    let set = (*req).RequestType == NdisRequestSetInformation;
    let (oid, buf, len, written, needed) = if set {
        let d = &mut (*req).DATA.SET_INFORMATION;
        (d.Oid, d.InformationBuffer, d.InformationBufferLength, addr_of_mut!(d.BytesRead) as *mut u32, addr_of_mut!(d.BytesNeeded) as *mut u32)
    } else {
        let d = &mut (*req).DATA.QUERY_INFORMATION;
        (d.Oid, d.InformationBuffer, d.InformationBufferLength, addr_of_mut!(d.BytesWritten) as *mut u32, addr_of_mut!(d.BytesNeeded) as *mut u32)
    };
    if !set {
        if let Some(s) = answer_from_attributes(a6, oid, buf, len, written, needed) {
            return s;
        }
    } else if oid == OID_GEN_CURRENT_LOOKAHEAD {
        // NDIS 6 owns the lookahead: indications carry whole frames.
        *written = len;
        return NDIS_STATUS_SUCCESS;
    }

    let r = alloc_zeroed(core::mem::size_of::<NdisOidRequest>()) as *mut NdisOidRequest;
    if r.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*r).Header = NdisObjectHeader { Type: NDIS_OBJECT_TYPE_OID_REQUEST, Revision: 1, Size: 236 /* REVISION_1 */ };
    (*r).RequestType = if set { NdisRequestSetInformation as u32 } else { NdisRequestQueryInformation as u32 };
    (*r).RequestId = req as PVOID;
    (*r).DATA.q = OidQuery { Oid: oid, InformationBuffer: buf, InformationBufferLength: len, BytesWritten: 0, BytesNeeded: 0 };
    (*r).SourceReserved[0] = req as PVOID;
    let status = ((*(*a6).drv).chars.OidRequestHandler.unwrap_unchecked())((*a6).ctx, r);
    if status == NDIS_STATUS_PENDING {
        return status; // NdisMOidRequestComplete copies back and frees
    }
    *written = (*r).DATA.q.BytesWritten;
    *needed = (*r).DATA.q.BytesNeeded;
    ExFreePool(r as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NdisMOidRequestComplete(
    MiniportAdapterHandle: NDIS_HANDLE,
    OidRequest: *mut NdisOidRequest,
    Status: NDIS_STATUS,
) {
    // A WDI driver completing one of the WDI layer's own commands.
    if crate::ndis::wdi::oid_request_complete(MiniportAdapterHandle, OidRequest, Status) {
        return;
    }
    let req = (*OidRequest).SourceReserved[0] as PNDIS_REQUEST;
    let set = (*req).RequestType == NdisRequestSetInformation;
    if set {
        (*req).DATA.SET_INFORMATION.BytesRead = (*OidRequest).DATA.q.BytesWritten;
        (*req).DATA.SET_INFORMATION.BytesNeeded = (*OidRequest).DATA.q.BytesNeeded;
    } else {
        (*req).DATA.QUERY_INFORMATION.BytesWritten = (*OidRequest).DATA.q.BytesWritten;
        (*req).DATA.QUERY_INFORMATION.BytesNeeded = (*OidRequest).DATA.q.BytesNeeded;
    }
    ExFreePool(OidRequest as PVOID);
    // The NDIS 5 completion path reads the request back from PendingRequest.
    (*(MiniportAdapterHandle as PLOGICAL_ADAPTER)).NdisMiniportBlock.PendingRequest = req;
    if set {
        NdisMSetInformationComplete(MiniportAdapterHandle, Status);
    } else {
        NdisMQueryInformationComplete(MiniportAdapterHandle, Status);
    }
}

// ---- status ----------------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn NdisMIndicateStatusEx(MiniportAdapterHandle: NDIS_HANDLE, StatusIndication: *mut StatusIndication) {
    // A WDI driver's indications are all for the WDI layer.
    if crate::ndis::wdi::indicate_status(MiniportAdapterHandle, StatusIndication) {
        return;
    }
    let s = &*StatusIndication;
    if s.StatusCode != NDIS_STATUS_LINK_STATE || s.StatusBuffer.is_null() {
        log6!("status %x indicated\n", s.StatusCode);
        return;
    }
    indicate_link_state(MiniportAdapterHandle, &*(s.StatusBuffer as *const LinkState));
}

/// A link-state change, told to the NDIS 5 protocols as media connect/disconnect. The WDI
/// layer, whose indications never reach NdisMIndicateStatusEx's own path, calls it directly.
pub unsafe fn indicate_link_state(MiniportAdapterHandle: NDIS_HANDLE, ls: &LinkState) {
    let a6 = adapter_of(MiniportAdapterHandle as PLOGICAL_ADAPTER);
    if a6.is_null() {
        return;
    }
    let up = ls.MediaConnectState == MEDIA_CONNECT_STATE_CONNECTED;
    (*a6).gen.XmitLinkSpeed = ls.XmitLinkSpeed;
    (*a6).gen.RcvLinkSpeed = ls.RcvLinkSpeed;
    log6!("link %s, %u Mb/s\n", if up { b"up\0".as_ptr() } else { b"down\0".as_ptr() }, (ls.XmitLinkSpeed / 1_000_000) as u32);
    if up != (*a6).link_up {
        (*a6).link_up = up;
        let code = if up { NDIS_STATUS_MEDIA_CONNECT_ } else { NDIS_STATUS_MEDIA_DISCONNECT_ };
        NdisMIndicateStatus(MiniportAdapterHandle, code, null_mut(), 0);
        NdisMIndicateStatusComplete(MiniportAdapterHandle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisMNetPnPEvent(_MiniportAdapterHandle: NDIS_HANDLE, _NetPnPEventNotification: PVOID) -> NDIS_STATUS {
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMIdleNotificationComplete(_MiniportAdapterHandle: NDIS_HANDLE) {}

#[no_mangle]
pub unsafe extern "system" fn NdisMIdleNotificationConfirm(_MiniportAdapterHandle: NDIS_HANDLE, _IdlePowerState: u32) {}

// ---- NET_BUFFER_LIST / NET_BUFFER pools ------------------------------------------------------

#[repr(C)]
pub struct NblPool {
    pub tag: u32,
    pub alloc_nb: bool,
    pub context_size: u16,
    pub data_size: u32,
}
#[repr(C)]
pub struct NbPool {
    pub tag: u32,
    pub data_size: u32,
}

/// An NBL from one of our pools: the NBL, then (when the pool allocates them) its NB.
#[repr(C)]
struct NblBlock {
    nbl: NetBufferList,
    nb: NetBuffer,
}

unsafe fn new_nbl_pool(tag: u32, alloc_nb: bool, context_size: u16) -> *mut NblPool {
    let p = alloc_zeroed(core::mem::size_of::<NblPool>()) as *mut NblPool;
    if !p.is_null() {
        (*p).tag = tag;
        (*p).alloc_nb = alloc_nb;
        (*p).context_size = context_size;
    }
    p
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateNetBufferListPool(_NdisHandle: NDIS_HANDLE, Parameters: *mut NblPoolParameters) -> NDIS_HANDLE {
    let pp = &*Parameters;
    let p = new_nbl_pool(pp.PoolTag, pp.fAllocateNetBuffer != 0, pp.ContextSize);
    if !p.is_null() {
        (*p).data_size = pp.DataSize;
        if pp.DataSize != 0 {
            log6!("NBL pool %x asks NDIS for %u-byte data buffers\n", pp.PoolTag, pp.DataSize);
        }
    }
    p as NDIS_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeNetBufferListPool(PoolHandle: NDIS_HANDLE) {
    if !PoolHandle.is_null() {
        ExFreePool(PoolHandle);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateNetBufferPool(_NdisHandle: NDIS_HANDLE, Parameters: *mut NbPoolParameters) -> NDIS_HANDLE {
    let p = alloc_zeroed(core::mem::size_of::<NbPool>()) as *mut NbPool;
    if !p.is_null() {
        (*p).tag = (*Parameters).PoolTag;
        (*p).data_size = (*Parameters).DataSize;
    }
    p as NDIS_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeNetBufferPool(PoolHandle: NDIS_HANDLE) {
    if !PoolHandle.is_null() {
        ExFreePool(PoolHandle);
    }
}

/// Point an NB at `DataOffset` bytes into `MdlChain`.
unsafe fn init_nb(nb: *mut NetBuffer, pool: *mut NbPool, MdlChain: PMDL, DataOffset: u32, DataLength: usize) {
    (*nb).NdisPoolHandle = pool;
    let h = &mut (*nb).u.h;
    h.Next = null_mut();
    h.MdlChain = MdlChain;
    h.DataOffset = DataOffset;
    h.DataLength = DataLength;
    let mut mdl = MdlChain;
    let mut off = DataOffset;
    while !mdl.is_null() && off >= (*mdl).ByteCount && !(*mdl).Next.is_null() {
        off -= (*mdl).ByteCount;
        mdl = (*mdl).Next;
    }
    h.CurrentMdl = mdl;
    h.CurrentMdlOffset = off;
}

unsafe fn alloc_context(nbl: *mut NetBufferList, pool_ctx: u16, ContextSize: u16, ContextBackFill: u16) -> bool {
    let total = pool_ctx as usize + ContextSize as usize + ContextBackFill as usize;
    if total == 0 {
        return true;
    }
    let c = alloc_zeroed(core::mem::size_of::<NblContext>() + total) as *mut NblContext;
    if c.is_null() {
        return false;
    }
    (*c).Size = total as u16;
    // ContextSize bytes are in use from the start; the rest is backfill for later.
    (*c).Offset = (total - ContextSize as usize) as u16;
    (*nbl).Context = c;
    true
}

unsafe fn alloc_nbl(pool: *mut NblPool, ContextSize: u16, ContextBackFill: u16) -> *mut NetBufferList {
    let blk = alloc_zeroed(core::mem::size_of::<NblBlock>()) as *mut NblBlock;
    if blk.is_null() {
        return null_mut();
    }
    let nbl = addr_of_mut!((*blk).nbl);
    (*nbl).NdisPoolHandle = pool;
    if !alloc_context(nbl, (*pool).context_size, ContextSize, ContextBackFill) {
        ExFreePool(blk as PVOID);
        return null_mut();
    }
    nbl
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateNetBufferList(PoolHandle: NDIS_HANDLE, ContextSize: u16, ContextBackFill: u16) -> *mut NetBufferList {
    alloc_nbl(PoolHandle as *mut NblPool, ContextSize, ContextBackFill)
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateNetBufferAndNetBufferList(
    PoolHandle: NDIS_HANDLE,
    ContextSize: u16,
    ContextBackFill: u16,
    MdlChain: PMDL,
    DataOffset: u32,
    DataLength: usize,
) -> *mut NetBufferList {
    let pool = PoolHandle as *mut NblPool;
    let nbl = alloc_nbl(pool, ContextSize, ContextBackFill);
    if nbl.is_null() {
        return nbl;
    }
    let nb = addr_of_mut!((*(nbl as *mut NblBlock)).nb);
    init_nb(nb, null_mut(), MdlChain, DataOffset, DataLength);
    (*nbl).u.h.FirstNetBuffer = nb;
    nbl
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeNetBufferList(NetBufferList: *mut NetBufferList) {
    if NetBufferList.is_null() {
        return;
    }
    let mut c = (*NetBufferList).Context;
    while !c.is_null() {
        let next = (*c).Next;
        ExFreePool(c as PVOID);
        c = next;
    }
    // An NB allocated with the NBL lives in the same block; a separately allocated one is the
    // caller's to free.
    ExFreePool(NetBufferList as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateNetBuffer(PoolHandle: NDIS_HANDLE, MdlChain: PMDL, DataOffset: u32, DataLength: usize) -> *mut NetBuffer {
    let nb = alloc_zeroed(core::mem::size_of::<NetBuffer>()) as *mut NetBuffer;
    if !nb.is_null() {
        init_nb(nb, PoolHandle as *mut NbPool, MdlChain, DataOffset, DataLength);
    }
    nb
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeNetBuffer(NetBuffer: *mut NetBuffer) {
    if !NetBuffer.is_null() {
        ExFreePool(NetBuffer as PVOID);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateMdl(_NdisHandle: NDIS_HANDLE, VirtualAddress: PVOID, Length: UINT) -> PMDL {
    let mdl = IoAllocateMdl(VirtualAddress, Length, FALSE as BOOLEAN, FALSE as BOOLEAN, null_mut());
    if !mdl.is_null() {
        MmBuildMdlForNonPagedPool(mdl);
    }
    mdl
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeMdl(Mdl: PMDL) {
    IoFreeMdl(Mdl);
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateMemoryWithTagPriority(_NdisHandle: NDIS_HANDLE, Length: UINT, Tag: ULONG, _Priority: u32) -> PVOID {
    ExAllocatePoolWithTag(NonPagedPool, Length as SIZE_T, Tag)
}

// ---- send ------------------------------------------------------------------------------------

unsafe fn mdl_va(mdl: PMDL) -> *mut u8 {
    MmGetSystemAddressForMdl!(mdl) as *mut u8
}

unsafe extern "system" fn shim_send_packets(context: NDIS_HANDLE, PacketArray: PPNDIS_PACKET, NumberOfPackets: UINT) {
    let a6 = context as *mut Ndis6Adapter;
    let adapter = (*a6).adapter;
    let flags = if KeGetCurrentIrql() as u32 >= DISPATCH_LEVEL as u32 { NDIS_SEND_FLAGS_DISPATCH_LEVEL } else { 0 };
    for i in 0..NumberOfPackets as usize {
        let packet = *PacketArray.add(i);
        let head = (*packet).Private.Head as PMDL;
        let mut len = 0usize;
        let mut m = head;
        while !m.is_null() {
            len += (*m).ByteCount as usize;
            m = (*m).Next;
        }
        let nbl = NdisAllocateNetBufferAndNetBufferList((*a6).send_pool as NDIS_HANDLE, 0, 0, head, 0, len);
        if nbl.is_null() {
            MiniSendComplete(adapter as NDIS_HANDLE, packet, NDIS_STATUS_RESOURCES);
            continue;
        }
        (*nbl).NdisReserved.0[0] = packet as PVOID;
        (*nbl).SourceHandle = adapter as NDIS_HANDLE;
        ((*(*a6).drv).chars.SendNetBufferListsHandler.unwrap_unchecked())((*a6).ctx, nbl, 0, flags);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSendNetBufferListsComplete(MiniportAdapterHandle: NDIS_HANDLE, NetBufferList: *mut NetBufferList, _SendCompleteFlags: u32) {
    let mut nbl = NetBufferList;
    while !nbl.is_null() {
        let next = (*nbl).u.h.Next;
        let packet = (*nbl).NdisReserved.0[0] as PNDIS_PACKET;
        let status = (*nbl).Status;
        NdisFreeNetBufferList(nbl);
        MiniSendComplete(MiniportAdapterHandle, packet, status);
        nbl = next;
    }
}

// ---- receive ---------------------------------------------------------------------------------

/// What a TransferData during an indication reads from: the frame being indicated.
#[repr(C)]
struct Indicated {
    frame: *const u8,
    len: usize,
}

/// Copy `nb`'s data into `dst`, returning the bytes copied.
pub unsafe fn copy_nb(nb: *mut NetBuffer, dst: &mut [u8]) -> usize {
    let h = &(*nb).u.h;
    let mut want = core::cmp::min(h.DataLength, dst.len());
    let mut mdl = h.CurrentMdl;
    let mut off = h.CurrentMdlOffset as usize;
    let mut got = 0;
    while want > 0 && !mdl.is_null() {
        let avail = ((*mdl).ByteCount as usize).saturating_sub(off);
        let n = core::cmp::min(avail, want);
        if n > 0 {
            core::ptr::copy_nonoverlapping(mdl_va(mdl).add(off), dst.as_mut_ptr().add(got), n);
        }
        got += n;
        want -= n;
        off = 0;
        mdl = (*mdl).Next;
    }
    got
}

#[no_mangle]
pub unsafe extern "system" fn NdisMIndicateReceiveNetBufferLists(
    MiniportAdapterHandle: NDIS_HANDLE,
    NetBufferList: *mut NetBufferList,
    _PortNumber: u32,
    _NumberOfNetBufferLists: u32,
    ReceiveFlags: u32,
) {
    let adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let a6 = adapter_of(adapter);
    let filter = (*adapter).NdisMiniportBlock.__bindgen_anon_1.EthDB;
    let hdr = (*adapter).MediumHeaderSize as usize;
    let mut frame = [0u8; 1536];
    let mut nbl = NetBufferList;
    let mut any = false;
    while !nbl.is_null() {
        let mut nb = (*nbl).u.h.FirstNetBuffer;
        while !nb.is_null() {
            let len = copy_nb(nb, &mut frame);
            if len >= hdr && !filter.is_null() {
                let mut ind = Indicated { frame: frame.as_ptr(), len };
                let p = frame.as_mut_ptr();
                EthFilterDprIndicateReceive(filter, addr_of_mut!(ind) as NDIS_HANDLE, p as PCHAR, p as PVOID,
                    hdr as UINT, p.add(hdr) as PVOID, (len - hdr) as UINT, (len - hdr) as UINT);
                any = true;
            }
            nb = (*nb).u.h.Next;
        }
        (*nbl).Status = NDIS_STATUS_SUCCESS;
        nbl = (*nbl).u.h.Next;
    }
    if any && !filter.is_null() {
        EthFilterDprIndicateReceiveComplete(filter);
    }
    // Every frame was copied out, so the NBLs go straight back, unless the miniport said they
    // were its last (RESOURCES), in which case they were never ours to return.
    if ReceiveFlags & NDIS_RECEIVE_FLAGS_RESOURCES == 0 && !a6.is_null() {
        let rflags = if KeGetCurrentIrql() as u32 >= DISPATCH_LEVEL as u32 { NDIS_RETURN_FLAGS_DISPATCH_LEVEL } else { 0 };
        ((*(*a6).drv).chars.ReturnNetBufferListsHandler.unwrap_unchecked())((*a6).ctx, NetBufferList, rflags);
    }
}

/// A protocol that wants more than the lookahead: the whole frame was the lookahead, so this
/// only ever copies from it.
unsafe extern "system" fn shim_transfer_data(
    Packet: PNDIS_PACKET,
    BytesTransferred: PUINT,
    _context: NDIS_HANDLE,
    MiniportReceiveContext: NDIS_HANDLE,
    ByteOffset: UINT,
    BytesToTransfer: UINT,
) -> NDIS_STATUS {
    let ind = &*(MiniportReceiveContext as *const Indicated);
    let start = 14 + ByteOffset as usize;
    let n = core::cmp::min(BytesToTransfer as usize, ind.len.saturating_sub(start));
    *BytesTransferred = CopyBufferToBufferChain((*Packet).Private.Head, 0, ind.frame.add(start) as PUCHAR, n as UINT);
    NDIS_STATUS_SUCCESS
}

// ---- interrupts ------------------------------------------------------------------------------

#[repr(C)]
pub struct Ndis6Interrupt {
    object: PKINTERRUPT,
    dpc: KDPC,
    ctx: NDIS_HANDLE,
    isr: unsafe extern "system" fn(NDIS_HANDLE, PBOOLEAN, *mut u32) -> BOOLEAN,
    dpc_handler: unsafe extern "system" fn(NDIS_HANDLE, PVOID, PVOID, PVOID),
}

#[repr(C)]
struct ReceiveThrottle {
    max_nbls: u32,
    more_pending: u32, // MoreNblsPending:1
}

unsafe extern "system" fn isr6(_interrupt: PKINTERRUPT, context: PVOID) -> BOOLEAN {
    let i = context as *mut Ndis6Interrupt;
    let mut queue: BOOLEAN = 0;
    let mut targets: u32 = 0;
    let mine = ((*i).isr)((*i).ctx, &mut queue, &mut targets);
    if queue != 0 || targets != 0 {
        KeInsertQueueDpc(addr_of_mut!((*i).dpc), null_mut(), null_mut());
    }
    mine
}

unsafe extern "system" fn dpc6(_dpc: PKDPC, context: PVOID, _a1: PVOID, _a2: PVOID) {
    let i = context as *mut Ndis6Interrupt;
    let mut throttle = ReceiveThrottle { max_nbls: u32::MAX, more_pending: 0 };
    ((*i).dpc_handler)((*i).ctx, null_mut(), addr_of_mut!(throttle) as PVOID, null_mut());
    if throttle.more_pending & 1 != 0 {
        KeInsertQueueDpc(addr_of_mut!((*i).dpc), null_mut(), null_mut());
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterInterruptEx(
    MiniportAdapterHandle: NDIS_HANDLE,
    MiniportInterruptContext: NDIS_HANDLE,
    MiniportInterruptCharacteristics: *mut InterruptCharacteristics,
    NdisInterruptHandle: *mut NDIS_HANDLE,
) -> NDIS_STATUS {
    let adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let ic = &mut *MiniportInterruptCharacteristics;
    let (Some(isr), Some(dpc)) = (ic.InterruptHandler, ic.InterruptDpcHandler) else {
        log6!("line-based interrupt handlers missing (MSI only?)\n");
        return NDIS_STATUS_NOT_SUPPORTED;
    };
    let list = (*adapter).NdisMiniportBlock.AllocatedResourcesTranslated;
    if list.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    let partial = addr_of_mut!((*list).List[0].PartialResourceList);
    let descs = addr_of_mut!((*partial).PartialDescriptors) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut d: *mut CM_PARTIAL_RESOURCE_DESCRIPTOR = null_mut();
    for k in 0..(*partial).Count as usize {
        if (*descs.add(k)).Type as u32 == CmResourceTypeInterrupt as u32 {
            d = descs.add(k);
            break;
        }
    }
    if d.is_null() {
        log6!("no interrupt resource\n");
        return NDIS_STATUS_RESOURCES;
    }
    let i = alloc_zeroed(core::mem::size_of::<Ndis6Interrupt>()) as *mut Ndis6Interrupt;
    if i.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*i).ctx = MiniportInterruptContext;
    (*i).isr = isr;
    (*i).dpc_handler = dpc;
    KeInitializeDpc(addr_of_mut!((*i).dpc), Some(dpc6), i as PVOID);
    let irq = (*d).u.Interrupt;
    let latched = (*d).Flags as u32 & CM_RESOURCE_INTERRUPT_LATCHED as u32 != 0;
    let shared = (*d).ShareDisposition as u32 == _CM_SHARE_DISPOSITION_CmResourceShareShared as u32;
    let status = IoConnectInterrupt(
        addr_of_mut!((*i).object),
        Some(isr6),
        i as PVOID,
        null_mut(),
        irq.Vector,
        irq.Level as KIRQL,
        irq.Level as KIRQL,
        if latched { _KINTERRUPT_MODE_Latched } else { _KINTERRUPT_MODE_LevelSensitive },
        shared as BOOLEAN,
        irq.Affinity,
        FALSE as BOOLEAN,
    );
    if !NT_SUCCESS!(status) {
        log6!("IoConnectInterrupt(vector %u) failed %x\n", irq.Vector, status);
        ExFreePool(i as PVOID);
        return NDIS_STATUS_RESOURCES;
    }
    ic.InterruptType = NDIS_CONNECT_LINE_BASED;
    ic.MessageInfoTable = null_mut();
    *NdisInterruptHandle = i as NDIS_HANDLE;
    log6!("interrupt connected: vector %u level %u %s\n", irq.Vector, irq.Level,
        if latched { b"latched\0".as_ptr() } else { b"level\0".as_ptr() });
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterInterruptEx(NdisInterruptHandle: NDIS_HANDLE) {
    let i = NdisInterruptHandle as *mut Ndis6Interrupt;
    if i.is_null() {
        return;
    }
    IoDisconnectInterrupt((*i).object);
    KeRemoveQueueDpc(addr_of_mut!((*i).dpc));
    ExFreePool(i as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSynchronizeWithInterruptEx(
    NdisInterruptHandle: NDIS_HANDLE,
    _MessageId: u32,
    SynchronizeFunction: PVOID,
    SynchronizeContext: PVOID,
) -> BOOLEAN {
    let i = NdisInterruptHandle as *mut Ndis6Interrupt;
    KeSynchronizeExecution((*i).object, core::mem::transmute(SynchronizeFunction), SynchronizeContext)
}

// ---- scatter-gather DMA ----------------------------------------------------------------------

#[repr(C)]
pub struct SgDma {
    adapter: PLOGICAL_ADAPTER,
    process: unsafe extern "system" fn(PDEVICE_OBJECT, PVOID, *mut SgList, PVOID),
    dma64: bool,
}

#[no_mangle]
pub unsafe extern "system" fn NdisMRegisterScatterGatherDma(
    MiniportAdapterHandle: NDIS_HANDLE,
    DmaDescription: *mut SgDmaDescription,
    NdisMiniportDmaHandle: *mut NDIS_HANDLE,
) -> NDIS_STATUS {
    let adapter = MiniportAdapterHandle as PLOGICAL_ADAPTER;
    let d = &mut *DmaDescription;
    let Some(process) = d.ProcessSGListHandler else { return NDIS_STATUS_INVALID_PARAMETER };
    let dma64 = d.Flags & NDIS_SG_DMA_64_BIT_ADDRESS != 0;
    // NdisMAllocateSharedMemory needs the DMA adapter this sets up.
    (*adapter).NdisMiniportBlock.Flags |= NDIS_ATTRIBUTE_BUS_MASTER as ULONG;
    let s = NdisMInitializeScatterGatherDma(adapter as NDIS_HANDLE, dma64 as BOOLEAN, d.MaximumPhysicalMapping);
    if s != NDIS_STATUS_SUCCESS {
        log6!("NdisMInitializeScatterGatherDma failed %x\n", s);
        return s;
    }
    let sg = alloc_zeroed(core::mem::size_of::<SgDma>()) as *mut SgDma;
    if sg.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*sg).adapter = adapter;
    (*sg).process = process;
    (*sg).dma64 = dma64;
    let pages = d.MaximumPhysicalMapping as usize / PAGE_SIZE as usize + 2;
    d.ScatterGatherListSize = (core::mem::size_of::<SgList>() + pages * core::mem::size_of::<SgElement>()) as u32;
    *NdisMiniportDmaHandle = sg as NDIS_HANDLE;
    log6!("SG DMA: %u-bit, max mapping %u, list size %u\n", if dma64 { 64u32 } else { 32 },
        d.MaximumPhysicalMapping, d.ScatterGatherListSize);
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMDeregisterScatterGatherDma(NdisMiniportDmaHandle: NDIS_HANDLE) {
    let sg = NdisMiniportDmaHandle as *mut SgDma;
    if sg.is_null() {
        return;
    }
    let block = &mut (*(*sg).adapter).NdisMiniportBlock;
    let dma = block.SystemAdapterObject;
    if !dma.is_null() {
        ((*(*dma).DmaOperations).PutDmaAdapter.unwrap_unchecked())(dma);
        block.SystemAdapterObject = null_mut();
    }
    ExFreePool(sg as PVOID);
}

/// The physical runs of an NB's data, merged where pages are contiguous. Without an IOMMU a
/// bus address is the physical address.
unsafe fn build_sg(nb: *mut NetBuffer, out: *mut SgList, max: usize) -> Option<usize> {
    let h = &(*nb).u.h;
    let mut left = h.DataLength;
    let mut mdl = h.CurrentMdl;
    let mut off = h.CurrentMdlOffset as usize;
    let mut n = 0usize;
    let elems = addr_of_mut!((*out).Elements) as *mut SgElement;
    while left > 0 && !mdl.is_null() {
        let mut avail = core::cmp::min(((*mdl).ByteCount as usize).saturating_sub(off), left);
        let mut va = mdl_va(mdl).add(off);
        while avail > 0 {
            let in_page = PAGE_SIZE as usize - (va as usize & (PAGE_SIZE as usize - 1));
            let chunk = core::cmp::min(in_page, avail);
            let pa = MmGetPhysicalAddress(va as PVOID).QuadPart;
            if n > 0 && (*elems.add(n - 1)).Address + (*elems.add(n - 1)).Length as i64 == pa {
                (*elems.add(n - 1)).Length += chunk as u32;
            } else {
                if n == max {
                    return None;
                }
                *elems.add(n) = SgElement { Address: pa, Length: chunk as u32, Reserved: 0 };
                n += 1;
            }
            va = va.add(chunk);
            avail -= chunk;
            left -= chunk;
        }
        off = 0;
        mdl = (*mdl).Next;
    }
    Some(n)
}

const SG_ALLOCATED: usize = 0x5347_4E44; // "SGND": the list is ours to free

#[no_mangle]
pub unsafe extern "system" fn NdisMAllocateNetBufferSGList(
    NdisMiniportDmaHandle: NDIS_HANDLE,
    NetBuffer: *mut NetBuffer,
    Context: PVOID,
    _Flags: u32,
    ScatterGatherListBuffer: PVOID,
    ScatterGatherListBufferSize: u32,
) -> NDIS_STATUS {
    let sg = NdisMiniportDmaHandle as *mut SgDma;
    let hdr = core::mem::size_of::<SgList>();
    let es = core::mem::size_of::<SgElement>();
    // Worst case: one element per page touched, plus one per MDL boundary.
    let mut worst = 0usize;
    let mut m = (*NetBuffer).u.h.CurrentMdl;
    while !m.is_null() {
        worst += (*m).ByteCount as usize / PAGE_SIZE as usize + 2;
        m = (*m).Next;
    }
    let need = hdr + worst * es;
    let (list, ours) = if !ScatterGatherListBuffer.is_null() && ScatterGatherListBufferSize as usize >= need {
        (ScatterGatherListBuffer as *mut SgList, false)
    } else {
        let p = ExAllocatePoolWithTag(NonPagedPool, need as SIZE_T, NDIS6_TAG) as *mut SgList;
        if p.is_null() {
            return NDIS_STATUS_RESOURCES;
        }
        (p, true)
    };
    let Some(n) = build_sg(NetBuffer, list, worst) else {
        if ours {
            ExFreePool(list as PVOID);
        }
        return NDIS_STATUS_RESOURCES;
    };
    if !(*sg).dma64 {
        let elems = addr_of_mut!((*list).Elements) as *mut SgElement;
        for k in 0..n {
            if (*elems.add(k)).Address as u64 + (*elems.add(k)).Length as u64 > 0x1_0000_0000 {
                log6!("a 32-bit DMA miniport was handed memory above 4 GB; no bounce buffers here\n");
                if ours {
                    ExFreePool(list as PVOID);
                }
                return NDIS_STATUS_RESOURCES;
            }
        }
    }
    (*list).NumberOfElements = n as u32;
    (*list).Reserved = if ours { SG_ALLOCATED } else { 0 };
    let dev = (*(*sg).adapter).NdisMiniportBlock.DeviceObject;
    ((*sg).process)(dev, null_mut(), list, Context);
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisMFreeNetBufferSGList(_NdisMiniportDmaHandle: NDIS_HANDLE, pSGL: *mut SgList, _NetBuffer: *mut NetBuffer) {
    if !pSGL.is_null() && (*pSGL).Reserved == SG_ALLOCATED {
        ExFreePool(pSGL as PVOID);
    }
}

// ---- configuration, bus data, device objects -------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn NdisOpenConfigurationEx(ConfigObject: *mut ConfigurationObject, ConfigurationHandle: *mut NDIS_HANDLE) -> NDIS_STATUS {
    let handle = (*ConfigObject).NdisHandle;
    let a6 = adapter_of(handle as PLOGICAL_ADAPTER);
    if a6.is_null() || (*a6).wrapper_ctx.is_null() {
        // A driver handle, or an adapter outside its InitializeHandlerEx: no driver key open.
        log6!("NdisOpenConfigurationEx(%p): only an adapter's, during initialization\n", handle);
        return NDIS_STATUS_FAILURE;
    }
    let mut status: NDIS_STATUS = 0;
    NdisOpenConfiguration(addr_of_mut!(status), ConfigurationHandle, (*a6).wrapper_ctx);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NdisMGetBusData(NdisMiniportHandle: NDIS_HANDLE, _WhichSpace: u32, Offset: u32, Buffer: PVOID, Length: u32) -> u32 {
    NdisReadPciSlotInformation(NdisMiniportHandle, 0, Offset, Buffer, Length)
}

#[no_mangle]
pub unsafe extern "system" fn NdisMSetBusData(NdisMiniportHandle: NDIS_HANDLE, _WhichSpace: u32, Offset: u32, Buffer: PVOID, Length: u32) -> u32 {
    NdisWritePciSlotInformation(NdisMiniportHandle, 0, Offset, Buffer, Length)
}

/// NdisRegisterDeviceEx's device extension: NDIS_M_DEVICE_BLOCK, then the driver's bytes.
fn device_block_size() -> usize {
    (core::mem::size_of::<NDIS_M_DEVICE_BLOCK>() + 15) & !15
}

#[no_mangle]
pub unsafe extern "system" fn NdisRegisterDeviceEx(
    NdisHandle: NDIS_HANDLE,
    DeviceObjectAttributes: *mut DeviceObjectAttributes,
    pDeviceObject: *mut PDEVICE_OBJECT,
    NdisDeviceHandle: *mut NDIS_HANDLE,
) -> NDIS_STATUS {
    let a = &*DeviceObjectAttributes;
    let block = driver_block_of(NdisHandle);
    if block.is_null() {
        log6!("NdisRegisterDeviceEx: %p is no NDIS handle we issued\n", NdisHandle);
        return NDIS_STATUS_INVALID_PARAMETER;
    }
    let mut dev: PDEVICE_OBJECT = null_mut();
    let status = IoCreateDevice(
        (*block).DriverObject,
        (device_block_size() + a.ExtensionSize as usize) as ULONG,
        a.DeviceName as PUNICODE_STRING,
        FILE_DEVICE_NETWORK as ULONG,
        0,
        FALSE as BOOLEAN,
        addr_of_mut!(dev),
    );
    if !NT_SUCCESS!(status) {
        log6!("NdisRegisterDeviceEx: IoCreateDevice %x\n", status);
        return status as NDIS_STATUS;
    }
    if !a.SymbolicName.is_null() {
        let s = IoCreateSymbolicLink(a.SymbolicName as PUNICODE_STRING, a.DeviceName as PUNICODE_STRING);
        if !NT_SUCCESS!(s) {
            IoDeleteDevice(dev);
            return s as NDIS_STATUS;
        }
    }
    let dblock = (*dev).DeviceExtension as PNDIS_M_DEVICE_BLOCK;
    for k in 0..=IRP_MJ_MAXIMUM_FUNCTION as usize {
        (*dblock).MajorFunction[k] = if a.MajorFunctions.is_null() { None } else { *a.MajorFunctions.add(k) };
    }
    (*dblock).DeviceObject = dev;
    (*dblock).SymbolicName = a.SymbolicName;
    (*dev).Flags &= !(DO_DEVICE_INITIALIZING as ULONG);
    *pDeviceObject = dev;
    *NdisDeviceHandle = dblock as NDIS_HANDLE;
    log6!("control device %wZ registered\n", a.DeviceName);
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisDeregisterDeviceEx(NdisDeviceHandle: NDIS_HANDLE) {
    let block = NdisDeviceHandle as PNDIS_M_DEVICE_BLOCK;
    if !(*block).SymbolicName.is_null() {
        IoDeleteSymbolicLink((*block).SymbolicName as PUNICODE_STRING);
    }
    IoDeleteDevice((*block).DeviceObject);
}

#[no_mangle]
pub unsafe extern "system" fn NdisGetDeviceReservedExtension(DeviceObject: PDEVICE_OBJECT) -> PVOID {
    ((*DeviceObject).DeviceExtension as *mut u8).add(device_block_size()) as PVOID
}

// ---- timers ----------------------------------------------------------------------------------

#[repr(C)]
pub struct NdisTimer6 {
    timer: KTIMER,
    dpc: KDPC,
    func: unsafe extern "system" fn(PVOID, PVOID, PVOID, PVOID),
    ctx: PVOID,
}

unsafe extern "system" fn timer_dpc(dpc: PKDPC, context: PVOID, _a1: PVOID, _a2: PVOID) {
    let t = context as *mut NdisTimer6;
    ((*t).func)(dpc as PVOID, (*t).ctx, null_mut(), null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateTimerObject(_NdisHandle: NDIS_HANDLE, TimerCharacteristics: *mut TimerCharacteristics, pTimerObject: *mut NDIS_HANDLE) -> NDIS_STATUS {
    let Some(func) = (*TimerCharacteristics).TimerFunction else { return NDIS_STATUS_INVALID_PARAMETER };
    let t = alloc_zeroed(core::mem::size_of::<NdisTimer6>()) as *mut NdisTimer6;
    if t.is_null() {
        return NDIS_STATUS_RESOURCES;
    }
    (*t).func = func;
    (*t).ctx = (*TimerCharacteristics).FunctionContext;
    KeInitializeTimer(addr_of_mut!((*t).timer));
    KeInitializeDpc(addr_of_mut!((*t).dpc), Some(timer_dpc), t as PVOID);
    *pTimerObject = t as NDIS_HANDLE;
    NDIS_STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NdisSetTimerObject(TimerObject: NDIS_HANDLE, DueTime: i64, MillisecondsPeriod: i32, FunctionContext: PVOID) -> BOOLEAN {
    let t = TimerObject as *mut NdisTimer6;
    if !FunctionContext.is_null() {
        (*t).ctx = FunctionContext;
    }
    let mut due: LARGE_INTEGER = core::mem::zeroed();
    due.QuadPart = DueTime;
    KeSetTimerEx(addr_of_mut!((*t).timer), due, MillisecondsPeriod, addr_of_mut!((*t).dpc))
}

#[no_mangle]
pub unsafe extern "system" fn NdisCancelTimerObject(TimerObject: NDIS_HANDLE) -> BOOLEAN {
    KeCancelTimer(addr_of_mut!((*(TimerObject as *mut NdisTimer6)).timer))
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeTimerObject(TimerObject: NDIS_HANDLE) {
    let t = TimerObject as *mut NdisTimer6;
    KeCancelTimer(addr_of_mut!((*t).timer));
    KeRemoveQueueDpc(addr_of_mut!((*t).dpc));
    ExFreePool(t as PVOID);
}

// ---- I/O work items --------------------------------------------------------------------------

#[repr(C)]
pub struct NdisIoWorkItem {
    item: WORK_QUEUE_ITEM,
    routine: Option<unsafe extern "system" fn(PVOID, NDIS_HANDLE)>,
    ctx: PVOID,
}

unsafe extern "system" fn io_work(param: PVOID) {
    let w = param as *mut NdisIoWorkItem;
    if let Some(r) = (*w).routine {
        r((*w).ctx, w as NDIS_HANDLE);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateIoWorkItem(_NdisObjectHandle: NDIS_HANDLE) -> NDIS_HANDLE {
    alloc_zeroed(core::mem::size_of::<NdisIoWorkItem>()) as NDIS_HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn NdisQueueIoWorkItem(NdisIoWorkItemHandle: NDIS_HANDLE, Routine: Option<unsafe extern "system" fn(PVOID, NDIS_HANDLE)>, WorkItemContext: PVOID) {
    let w = NdisIoWorkItemHandle as *mut NdisIoWorkItem;
    (*w).routine = Routine;
    (*w).ctx = WorkItemContext;
    ExInitializeWorkItem!(addr_of_mut!((*w).item), Some(io_work), w);
    ExQueueWorkItem(addr_of_mut!((*w).item), DelayedWorkQueue);
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeIoWorkItem(NdisIoWorkItemHandle: NDIS_HANDLE) {
    ExFreePool(NdisIoWorkItemHandle);
}

// ---- reader/writer locks ---------------------------------------------------------------------

/// NDIS_RW_LOCK_EX: >0 readers, -1 a writer. Readers may nest, as NDIS allows.
#[repr(C)]
pub struct RwLockEx {
    state: AtomicI32,
}

#[no_mangle]
pub unsafe extern "system" fn NdisAllocateRWLock(_NdisHandle: NDIS_HANDLE) -> *mut RwLockEx {
    alloc_zeroed(core::mem::size_of::<RwLockEx>()) as *mut RwLockEx
}

#[no_mangle]
pub unsafe extern "system" fn NdisFreeRWLock(Lock: *mut RwLockEx) {
    ExFreePool(Lock as PVOID);
}

unsafe fn rw_enter(state: *mut LockStateEx, flags: u8, mode: u8) {
    (*state).Flags = flags;
    (*state).LockState = mode;
    if flags & NDIS_RWL_AT_DISPATCH_LEVEL == 0 {
        KeRaiseIrql(DISPATCH_LEVEL as KIRQL, addr_of_mut!((*state).OldIrql));
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisAcquireRWLockRead(Lock: *mut RwLockEx, LockState: *mut LockStateEx, Flags: u8) {
    rw_enter(LockState, Flags, 1);
    loop {
        let s = (*Lock).state.load(Ordering::Relaxed);
        if s >= 0 && (*Lock).state.compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            return;
        }
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisAcquireRWLockWrite(Lock: *mut RwLockEx, LockState: *mut LockStateEx, Flags: u8) {
    rw_enter(LockState, Flags, 2);
    while (*Lock).state.compare_exchange_weak(0, -1, Ordering::Acquire, Ordering::Relaxed).is_err() {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub unsafe extern "system" fn NdisReleaseRWLock(Lock: *mut RwLockEx, LockState: *mut LockStateEx) {
    if (*LockState).LockState == 2 {
        (*Lock).state.store(0, Ordering::Release);
    } else {
        (*Lock).state.fetch_sub(1, Ordering::Release);
    }
    if (*LockState).Flags & NDIS_RWL_AT_DISPATCH_LEVEL == 0 {
        KeLowerIrql((*LockState).OldIrql);
    }
}

// ---- layout checks (amd64 offsets from the WDK headers) --------------------------------------

#[cfg(target_arch = "x86_64")]
const _: () = {
    use core::mem::{offset_of, size_of};
    assert!(offset_of!(NetBuffer, ChecksumBias) == 48);
    assert!(offset_of!(NetBuffer, NdisPoolHandle) == 56);
    assert!(offset_of!(NetBuffer, ProtocolReserved) == 80);
    assert!(offset_of!(NetBuffer, MiniportReserved) == 128);
    assert!(offset_of!(NetBuffer, DataPhysicalAddress) == 160);
    assert!(size_of::<NetBuffer>() == 176);
    assert!(offset_of!(NetBufferList, Context) == 16);
    assert!(offset_of!(NetBufferList, NdisReserved) == 48);
    assert!(offset_of!(NetBufferList, ProtocolReserved) == 64);
    assert!(offset_of!(NetBufferList, MiniportReserved) == 96);
    assert!(offset_of!(NetBufferList, SourceHandle) == 120);
    assert!(offset_of!(NetBufferList, Status) == 140);
    assert!(offset_of!(NetBufferList, NetBufferListInfo) == 144);
    assert!(offset_of!(NblContext, ContextData) == 16);
    assert!(offset_of!(MiniportDriverCharacteristics, SetOptionsHandler) == 16);
    assert!(offset_of!(MiniportDriverCharacteristics, CancelOidRequestHandler) == 128);
    assert!(offset_of!(MiniportInitParameters, NetLuid) == 40);
    assert!(size_of::<MiniportInitParameters>() == 64);
    assert!(offset_of!(GeneralAttributes, MaxXmitLinkSpeed) == 24);
    assert!(offset_of!(GeneralAttributes, MediaConnectState) == 56);
    assert!(offset_of!(GeneralAttributes, PowerManagementCapabilities) == 72);
    assert!(offset_of!(GeneralAttributes, MacAddressLength) == 92);
    assert!(offset_of!(GeneralAttributes, CurrentMacAddress) == 126);
    assert!(offset_of!(NdisOidRequest, DATA) == 32);
    assert!(offset_of!(NdisOidRequest, NdisReserved) == 72);
    assert!(offset_of!(NdisOidRequest, SourceReserved) == 216);
    assert!(offset_of!(NdisOidRequest, SupportedRevision) == 232);
    assert!(offset_of!(StatusIndication, StatusBuffer) == 48);
    assert!(offset_of!(LinkState, XmitLinkSpeed) == 16);
    assert!(offset_of!(InterruptCharacteristics, MsiSupported) == 40);
    assert!(offset_of!(InterruptCharacteristics, InterruptType) == 80);
    assert!(offset_of!(SgDmaDescription, ProcessSGListHandler) == 16);
    assert!(offset_of!(SgDmaDescription, ScatterGatherListSize) == 32);
    assert!(size_of::<SgElement>() == 24);
    assert!(offset_of!(SgList, Elements) == 16);
    assert!(offset_of!(DeviceObjectAttributes, ExtensionSize) == 32);
};
