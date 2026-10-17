//! NDIS 6 structure layouts, shared by the two sides of the interface: ndis.sys
//! (ndis-rust's NDIS 6 shim and WDI layer) and the NDIS 6 miniports written in Rust
//! (bwfm). One copy, so the two sides cannot drift apart.
//!
//! Layouts are the WDK's (ndis/nbl.h, miniport.h, oidrequest.h); the tree has no NDIS 6
//! headers to generate them from. Field types come from ntkrnl_sys::ddk, the island
//! both sides already build against. The offsets asserted at the bottom are the WDK's
//! for 64-bit targets.

#![no_std]
#![allow(non_snake_case, non_camel_case_types)]

use ntkrnl_sys::ddk::{BOOLEAN, KIRQL, PBOOLEAN, PDEVICE_OBJECT, PDRIVER_DISPATCH, PDRIVER_OBJECT, PMDL, PUNICODE_STRING, PVOID};

// Private: ndis-rust glob-imports this crate beside its own island, which has both.
type NDIS_HANDLE = PVOID;
type NDIS_STATUS = core::ffi::c_int;
type PNDIS_STRING = PUNICODE_STRING;

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
    pub NdisPoolHandle: PVOID, // the pool the buffer came from: opaque here
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
    pub NdisPoolHandle: PVOID,
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


// ---- status codes and flags (ndis.h values the bindgen islands leave out) -----------------

pub const NDIS_STATUS_INVALID_LENGTH: i32 = 0xC001_0014u32 as i32;
pub const NDIS_STATUS_LINK_STATE: i32 = 0x4001_0017;
/// STATUS_NDIS_MEDIA_DISCONNECTED and STATUS_NDIS_PAUSED (ntstatus.h).
pub const NDIS_STATUS_MEDIA_DISCONNECTED: i32 = 0xC023_001Fu32 as i32;
pub const NDIS_STATUS_PAUSED: i32 = 0xC023_002Au32 as i32;
/// NDIS_MEDIA_CONNECT_STATE: MediaConnectStateConnected / Disconnected.
pub const MEDIA_CONNECT_STATE_CONNECTED: u32 = 1;
pub const MEDIA_CONNECT_STATE_DISCONNECTED: u32 = 2;
/// NDIS_MEDIA_DUPLEX_STATE: MediaDuplexStateFull.
pub const MEDIA_DUPLEX_STATE_FULL: u32 = 2;
pub const NDIS_SEND_FLAGS_DISPATCH_LEVEL: u32 = 0x1;
pub const NDIS_RETURN_FLAGS_DISPATCH_LEVEL: u32 = 0x1;
pub const NDIS_RECEIVE_FLAGS_DISPATCH_LEVEL: u32 = 0x1;
pub const NDIS_RECEIVE_FLAGS_RESOURCES: u32 = 0x2;
pub const NDIS_SEND_COMPLETE_FLAGS_DISPATCH_LEVEL: u32 = 0x1;

// ---- layout checks (the WDK's 64-bit offsets) --------------------------------------

#[cfg(target_pointer_width = "64")]
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
