use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ExGetPreviousMode, _KWAIT_REASON_Executive, GUID, IoGetCurrentIrpStackLocation, IoMarkIrpPending, KPROCESSOR_MODE, KeCancelTimer, KeInitializeEvent, KeInitializeMutex, KeInitializeSpinLock, KeReleaseMutex, KeSetTimer, KeWaitForSingleObject, LARGE_INTEGER, _EVENT_TYPE_NotificationEvent, ObReferenceObjectByHandle, PDEVICE_OBJECT, PFILE_OBJECT, PHANDLE, PIO_STACK_LOCATION, PIRP, PLONGLONG, PMDL, RtlFreeUnicodeString, RtlInitUnicodeString, RtlStringFromGUID};
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_HyperCriticalWorkQueue as HyperCriticalWorkQueue;
use ntkrnl_sys::ddk::_MM_PAGE_PRIORITY_NormalPagePriority as NormalPagePriority;
use core::ffi::c_void;
use core::ptr::{addr_of_mut, addr_of};

// IoFileObjectType is an ntoskrnl DATA export (`@ extern`) — cross-DLL data imports need
// raw-dylib (`__imp_` indirection); the substrate's plain extern references the bare symbol
// and fails to link. This shadows it locally for pin.rs's single use site.
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static IoFileObjectType: PVOID;
}

// ---- ks-rust integration glue (pin.rs-local): imports, aliases, helper shims ----
// Explicit import wins over glob ambiguity (DbgPrint lives in driver_ddk).
use ntkrnl_sys::driver_ddk::DbgPrint;
// Shared allocator/completion helpers (defined #[no_mangle] in misc.rs).
use crate::misc::{AllocateItem, FreeItem, CompleteRequest};
// Cross-module Ks* handlers (defined #[no_mangle] in sibling ks modules).
use crate::methods::KspMethodHandlerWithAllocator;
use crate::property::KspPropertyHandler;
use crate::filter::{IKsFilter_AddPin, IKsFilter_RemovePin};
// Interlocked alias (translation uses the MSVC intrinsic spelling).
use ntkrnl_sys::interlocked::InterlockedIncrement as _InterlockedIncrement;
// Enum-member bare aliases (bindgen names them <EnumTag>_<member>).
use crate::types::KSLIST_ENTRY_LOCATION_KsListEntryHead as KsListEntryHead;
use crate::types::KSLIST_ENTRY_LOCATION_KsListEntryTail as KsListEntryTail;
use crate::types::KSIRP_REMOVAL_OPERATION_KsAcquireAndRemoveOnlySingleItem as KsAcquireAndRemoveOnlySingleItem;

// RtlAssert is a real ntoskrnl export (declared here for the rlib; resolved at cohab link).
extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PVOID);
}
// KspEnableEvent: static helper in event.c, not yet translated to a Rust symbol; the cohab
// C ks.sys provides it. Declared extern so the rlib compiles (reported to ks-glue-needs).
extern "system" {
    fn KspEnableEvent(
        Irp: PIRP,
        EventSetsCount: ULONG,
        EventSet: *const KSEVENT_SET,
        EventsList: PLIST_ENTRY,
        EventsFlags: KSEVENTS_LOCKTYPE,
        EventsLock: PVOID,
        Allocator: PFNKSALLOCATOR,
        EventItemSize: ULONG,
    ) -> NTSTATUS;
}

// Internal stream-pointer record alias (struct `_KSISTREAM_POINTER` is defined below).
pub type KSISTREAM_POINTER = _KSISTREAM_POINTER;


// IsEqualGUIDAligned: guiddef inline GUID equality.
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

// All KS property handlers share one concrete signature
// (PKSIDENTIFIER == PKSPROPERTY == *mut KSIDENTIFIER).
type KsPropHandlerFn = unsafe extern "system" fn(PIRP, PKSPROPERTY, PVOID) -> NTSTATUS;

// KSPROPERTY_ITEM get/set handler union builders. const-eval forbids materialising a
// function pointer as the union's integer storage, so the tables are null-initialised
// at const time and the real handler pointers are written at runtime in
// run_static_initializers (the c2rust .init_array mechanism). The handler arg documents
// the intended wiring; it is applied below.
macro_rules! ks_prop_get {
    ($f:expr) => {
        ::core::mem::transmute::<Option<KsPropHandlerFn>, KSPROPERTY_ITEM__bindgen_ty_1>(None)
    };
}
macro_rules! ks_prop_set {
    ($f:expr) => {
        ::core::mem::transmute::<Option<KsPropHandlerFn>, KSPROPERTY_ITEM__bindgen_ty_2>(None)
    };
}
macro_rules! ks_prop_set_null {
    () => {
        ::core::mem::transmute::<Option<KsPropHandlerFn>, KSPROPERTY_ITEM__bindgen_ty_2>(None)
    };
}

// const-eval-safe vtable-slot builder: coerce the zero-sized fn item to a concrete fn
// POINTER (naming its real signature/ABI), then transmute to the slot type. No `as usize`
// hop (const-eval forbids that) — unlike cb!, which is runtime-only. Matches the explicit
// transmute pattern already used in filter.rs/clocks.rs/allocators.rs for static vtables.
macro_rules! cbk {
    // const-valid + ABI-agnostic: coerce the fn item to a thin data pointer (NOT an integer
    // — that is illegal at const-eval) then transmute to the slot type. The $t arg is no
    // longer needed (the substrate slot's per-arch ABI stdcall/C need not be named).
    ($f:expr, $t:ty) => {
        ::core::mem::transmute($f as *const ())
    };
}

// c2rust .init_array initializer: writes the real property handlers into the tables and
// wires PinPropertySet -> {PinConnectionSet, PinStreamSet} (pin.c DEFINE_KSPROPERTY_*SET +
// PinPropertySet[]). Runtime context permits the fn-ptr -> integer store the union needs.
unsafe extern "C" fn run_static_initializers() {
    let set_get = |item: &mut KSPROPERTY_ITEM, f: KsPropHandlerFn| {
        item.__bindgen_anon_1.bindgen_union_field = f as usize as _;
    };
    let set_set = |item: &mut KSPROPERTY_ITEM, f: KsPropHandlerFn| {
        item.__bindgen_anon_2.bindgen_union_field = f as usize as _;
    };
    set_get(&mut PinConnectionSet[0], IKsPin_PinStatePropertyHandler);
    set_set(&mut PinConnectionSet[0], IKsPin_PinStatePropertyHandler);
    set_get(&mut PinConnectionSet[1], IKsPin_PinDataFormatPropertyHandler);
    set_set(&mut PinConnectionSet[1], IKsPin_PinDataFormatPropertyHandler);
    set_get(&mut PinConnectionSet[2], IKsPin_PinAllocatorFramingPropertyHandler);
    set_get(&mut PinStreamSet[0], IKsPin_PinStreamAllocator);
    set_set(&mut PinStreamSet[0], IKsPin_PinStreamAllocator);
    set_get(&mut PinStreamSet[1], IKsPin_PinMasterClock);
    set_set(&mut PinStreamSet[1], IKsPin_PinMasterClock);
    set_get(&mut PinStreamSet[2], IKsPin_PinPipeId);
    set_set(&mut PinStreamSet[2], IKsPin_PinPipeId);

    PinPropertySet[0].Set = &raw const KSPROPSETID_Connection;
    PinPropertySet[0].PropertiesCount =
        (core::mem::size_of_val(&*&raw const PinConnectionSet) / core::mem::size_of::<KSPROPERTY_ITEM>()) as ULONG;
    PinPropertySet[0].PropertyItem = &raw const PinConnectionSet[0];
    PinPropertySet[1].Set = &raw const KSPROPSETID_Stream;
    PinPropertySet[1].PropertiesCount =
        (core::mem::size_of_val(&*&raw const PinStreamSet) / core::mem::size_of::<KSPROPERTY_ITEM>()) as ULONG;
    PinPropertySet[1].PropertyItem = &raw const PinStreamSet[0];
}

// IKsPin_fnSetTransportConfig: present in pin.c (UNIMPLEMENTED stub) but dropped by the
// translation; restored faithfully so the vtable slot below resolves.
#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnSetTransportConfig(
    iface: *mut IKsPin,
    TransportConfig: *const KSPTRANSPORTCONFIG,
    OutTransportIn: *mut *mut IKsTransport,
    OutTransportOut: *mut *mut IKsTransport,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Clock: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x005\x003\x001\x007\x002\x004\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub const KSSTRING_Allocator: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x006\x004\x002\0F\x005\0D\x000\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
    )
};

pub const KSSTRING_TopologyNode: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x000\x006\x002\x001\x000\x006\x001\0A\0-\0E\0E\x007\x005\0-\x001\x001\0D\x000\0-\0B\09\x001\x005\0-\x000\x000\0A\x000\0C\09\x002\x002\x003\x001\09\x006\0}\0\0\0",
    )
};

pub static mut PinConnectionSet: [KSPROPERTY_ITEM; 3] = unsafe {
    [
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CONNECTION_STATE as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinStatePropertyHandler),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<KSSTATE>() as ULONG,
            __bindgen_anon_2: ks_prop_set!(IKsPin_PinStatePropertyHandler),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CONNECTION_DATAFORMAT as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinDataFormatPropertyHandler),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: 0 as ULONG,
            __bindgen_anon_2: ks_prop_set!(IKsPin_PinDataFormatPropertyHandler),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_CONNECTION_ALLOCATORFRAMING_EX as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinAllocatorFramingPropertyHandler),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: 0 as ULONG,
            __bindgen_anon_2: ks_prop_set_null!(),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
    ]
};

pub static mut PinStreamSet: [KSPROPERTY_ITEM; 3] = unsafe {
    [
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_STREAM_ALLOCATOR as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinStreamAllocator),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<HANDLE>() as ULONG,
            __bindgen_anon_2: ks_prop_set!(IKsPin_PinStreamAllocator),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_STREAM_MASTERCLOCK as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinMasterClock),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<HANDLE>() as ULONG,
            __bindgen_anon_2: ks_prop_set!(IKsPin_PinMasterClock),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
        KSPROPERTY_ITEM {
            PropertyId: KSPROPERTY_STREAM_PIPE_ID as ::core::ffi::c_int as ULONG,
            __bindgen_anon_1: ks_prop_get!(IKsPin_PinPipeId),
            MinProperty: ::core::mem::size_of::<KSPROPERTY>() as ULONG,
            MinData: ::core::mem::size_of::<HANDLE>() as ULONG,
            __bindgen_anon_2: ks_prop_set!(IKsPin_PinPipeId),
            Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
            RelationsCount: 0 as ULONG,
            Relations: ::core::ptr::null::<KSPROPERTY>(),
            SupportHandler: None,
            SerializedSize: 0 as ::core::ffi::c_int as ULONG,
        },
    ]
};

pub static mut PinPropertySet: [KSPROPERTY_SET; 2] = [
    KSPROPERTY_SET {
        Set: ::core::ptr::null::<GUID>(),
        PropertiesCount: 0,
        PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
        FastIoCount: 0,
        FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
    },
    KSPROPERTY_SET {
        Set: ::core::ptr::null::<GUID>(),
        PropertiesCount: 0,
        PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
        FastIoCount: 0,
        FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
    },
];

pub static mut KSPROPSETID_Connection: GUID = GUID {
    Data1: 0x1d58c920 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xac9b as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut KSPROPSETID_Stream: GUID = GUID {
    Data1: 0x65aaba60 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x98ae as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x20 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xaf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x56 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut KSPROPSETID_Clock: GUID = GUID {
    Data1: 0xdf12a4c0 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0xac17 as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0xa5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xd6 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x28 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xdb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut vt_IKsPin: IKsPinVtbl = unsafe {
    IKsPinVtbl {
        QueryInterface: cbk!(IKsPin_fnQueryInterface, unsafe extern "system" fn(*mut IKsPin, *const IID, *mut PVOID) -> NTSTATUS),
        AddRef: cbk!(IKsPin_fnAddRef, unsafe extern "system" fn(*mut IKsPin) -> ULONG),
        Release: cbk!(IKsPin_fnRelease, unsafe extern "system" fn(*mut IKsPin) -> ULONG),
        TransferKsIrp: cbk!(IKsPin_fnTransferKsIrp, unsafe extern "system" fn(*mut IKsPin, PIRP, *mut *mut IKsTransport) -> NTSTATUS),
        DiscardKsIrp: cbk!(IKsPin_fnDiscardKsIrp, unsafe extern "system" fn(*mut IKsPin, PIRP, *mut *mut IKsTransport)),
        Connect: cbk!(IKsPin_fnConnect, unsafe extern "system" fn(*mut IKsPin, *mut IKsTransport, *mut *mut IKsTransport, *mut *mut IKsTransport, KSPIN_DATAFLOW) -> NTSTATUS),
        SetDeviceState: cbk!(IKsPin_fnSetDeviceState, unsafe extern "system" fn(*mut IKsPin, KSSTATE, KSSTATE, *mut *mut IKsTransport) -> NTSTATUS),
        SetResetState: cbk!(IKsPin_fnSetResetState, unsafe extern "system" fn(*mut IKsPin, KSRESET, *mut *mut IKsTransport)),
        GetTransportConfig: cbk!(IKsPin_fnGetTransportConfig, unsafe extern "system" fn(*mut IKsPin, *mut KSPTRANSPORTCONFIG, *mut *mut IKsTransport, *mut *mut IKsTransport) -> NTSTATUS),
        SetTransportConfig: cbk!(IKsPin_fnSetTransportConfig, unsafe extern "system" fn(*mut IKsPin, *const KSPTRANSPORTCONFIG, *mut *mut IKsTransport, *mut *mut IKsTransport) -> NTSTATUS),
        ResetTransportConfig: cbk!(IKsPin_fnResetTransportConfig, unsafe extern "system" fn(*mut IKsPin, *mut *mut IKsTransport, *mut *mut IKsTransport) -> NTSTATUS),
        GetStruct: cbk!(IKsPin_fnGetStruct, unsafe extern "system" fn(*mut IKsPin) -> PKSPIN),
        GetProcessPin: cbk!(IKsPin_fnGetProcessPin, unsafe extern "system" fn(*mut IKsPin) -> PKSPROCESSPIN),
        AttemptBypass: cbk!(IKsPin_fnAttemptBypass, unsafe extern "system" fn(*mut IKsPin) -> NTSTATUS),
        AttemptUnbypass: cbk!(IKsPin_fnAttemptUnbypass, unsafe extern "system" fn(*mut IKsPin) -> NTSTATUS),
        GenerateConnectionEvents: cbk!(IKsPin_fnGenerateConnectionEvents, unsafe extern "system" fn(*mut IKsPin, ULONG)),
        ClientSetDeviceState: cbk!(IKsPin_fnClientSetDeviceState, unsafe extern "system" fn(*mut IKsPin, KSSTATE, KSSTATE) -> NTSTATUS),
    }
};

static mut vt_ReferenceClock: IKsReferenceClockVtbl = unsafe {
    IKsReferenceClockVtbl {
        QueryInterface: cbk!(IKsReferenceClock_fnQueryInterface, unsafe extern "system" fn(*mut IKsReferenceClock, *const IID, *mut PVOID) -> NTSTATUS),
        AddRef: cbk!(IKsReferenceClock_fnAddRef, unsafe extern "system" fn(*mut IKsReferenceClock) -> ULONG),
        Release: cbk!(IKsReferenceClock_fnRelease, unsafe extern "system" fn(*mut IKsReferenceClock) -> ULONG),
        GetTime: cbk!(IKsReferenceClock_fnGetTime, unsafe extern "system" fn(*mut IKsReferenceClock) -> LONGLONG),
        GetPhysicalTime: cbk!(IKsReferenceClock_fnGetPhysicalTime, unsafe extern "system" fn(*mut IKsReferenceClock) -> LONGLONG),
        GetCorrelatedTime: cbk!(IKsReferenceClock_fnGetCorrelatedTime, unsafe extern "system" fn(*mut IKsReferenceClock, PLONGLONG) -> LONGLONG),
        GetCorrelatedPhysicalTime: cbk!(IKsReferenceClock_fnGetCorrelatedPhysicalTime, unsafe extern "system" fn(*mut IKsReferenceClock, PLONGLONG) -> LONGLONG),
        GetResolution: cbk!(IKsReferenceClock_fnGetResolution, unsafe extern "system" fn(*mut IKsReferenceClock, PKSRESOLUTION) -> NTSTATUS),
        GetState: cbk!(IKsReferenceClock_fnGetState, unsafe extern "system" fn(*mut IKsReferenceClock, PKSSTATE) -> NTSTATUS),
    }
};

static mut PinDispatchTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: cbk!(IKsPin_DispatchDeviceIoControl, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        Read: cbk!(KsDispatchInvalidDeviceRequest, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        Write: cbk!(KsDispatchInvalidDeviceRequest, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        Flush: cbk!(KsDispatchInvalidDeviceRequest, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        Close: cbk!(IKsPin_Close, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        QuerySecurity: cbk!(KsDispatchQuerySecurity, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        SetSecurity: cbk!(KsDispatchSetSecurity, unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS),
        FastDeviceIoControl: cbk!(KsDispatchFastIoDeviceControlFailure, unsafe extern "system" fn(PFILE_OBJECT, BOOLEAN, PVOID, ULONG, PVOID, ULONG, ULONG, PIO_STATUS_BLOCK, PDEVICE_OBJECT) -> BOOLEAN),
        FastRead: cbk!(KsDispatchFastReadFailure, unsafe extern "system" fn(PFILE_OBJECT, PLARGE_INTEGER, ULONG, BOOLEAN, ULONG, PVOID, PIO_STATUS_BLOCK, PDEVICE_OBJECT) -> BOOLEAN),
        FastWrite: cbk!(KsDispatchFastReadFailure, unsafe extern "system" fn(PFILE_OBJECT, PLARGE_INTEGER, ULONG, BOOLEAN, ULONG, PVOID, PIO_STATUS_BLOCK, PDEVICE_OBJECT) -> BOOLEAN),
    }
};

static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

#[repr(C)]
pub struct _KSISTREAM_POINTER {
    pub Callback: PFNKSSTREAMPOINTER,
    pub Irp: PIRP,
    pub Timer: KTIMER,
    pub TimerDpc: KDPC,
    pub Next: *mut _KSISTREAM_POINTER,
    pub Pin: PKSPIN,
    pub Data: PVOID,
    pub Offset: ULONG,
    pub Length: ULONG,
    pub StreamPointer: KSSTREAM_POINTER,
    pub Lock: KSPIN_LOCK,
}
pub type PKSISTREAM_POINTER = *mut _KSISTREAM_POINTER;

#[repr(C)]
pub struct IKsPinImpl {
    pub BasicHeader: KSBASIC_HEADER,
    pub Pin: KSPIN,
    pub ObjectHeader: PKSIOBJECT_HEADER,
    pub ProcessPin: KSPROCESSPIN,
    pub Entry: LIST_ENTRY,
    pub ref_: LONG,
    pub Filter: *mut IKsFilter,
    pub ProcessingMutex: KMUTEX,
    pub FileObject: PFILE_OBJECT,
    pub AttachedGate: PKSGATE,
    pub OrGate: BOOL,
    pub IrpList: LIST_ENTRY,
    pub IrpListLock: KSPIN_LOCK,
    pub IrpCount: LONG,
    pub ClonedStreamPointer: PKSISTREAM_POINTER,
    pub LeadingEdgeStreamPointer: KSISTREAM_POINTER,
    pub TrailingStreamPointer: KSISTREAM_POINTER,
    pub Sleep: PFNKSPINPOWER,
    pub Wake: PFNKSPINPOWER,
    pub Handshake: PFNKSPINHANDSHAKE,
    pub FrameReturn: PFNKSPINFRAMERETURN,
    pub IrpCompletion: PFNKSPINIRPCOMPLETION,
    pub ClockTable: KSCLOCK_FUNCTIONTABLE,
    pub ClockFileObject: PFILE_OBJECT,
    pub lpVtblReferenceClock: *mut IKsReferenceClockVtbl,
    pub DefaultClock: PKSDEFAULTCLOCK,
    pub PinWorker: PKSWORKER,
    pub PinWorkQueueItem: WORK_QUEUE_ITEM,
    pub FrameComplete: KEVENT,
    pub FrameSize: ULONG,
    pub NumFrames: ULONG,
    pub Dma: PDMA_ADAPTER,
    pub MapRegisters: ULONG,
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinStreamAllocator(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinMasterClock(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;
    let This = CONTAINING_RECORD!((*ObjectHeader).ObjectType, IKsPinImpl, Pin) as *mut IKsPinImpl;
    let Handle = Data as PHANDLE;
    let mut Status = STATUS_SUCCESS;
    let mut FileObject: PFILE_OBJECT = core::ptr::null_mut();
    let mut Mode: KPROCESSOR_MODE = 0;
    let mut Property: KSPROPERTY = KSPROPERTY {
        Set: GUID {
            Data1: 0,
            Data2: 0,
            Data3: 0,
            Data4: [0; 8],
        },
        Id: 0,
        Flags: 0,
    };
    let mut BytesReturned: ULONG = 0;

    if (*Request).Flags & KSPROPERTY_TYPE_GET != 0 {
        if (*(*This).Pin.Descriptor).PinDescriptor.Communication != KSPIN_COMMUNICATION_NONE
            && !(*(*This).Pin.Descriptor).Dispatch.is_null()
            && (*(*This).Pin.Descriptor).Flags & KSPIN_FLAG_IMPLEMENT_CLOCK != 0
        {
            *Handle = core::ptr::null_mut();
            Status = STATUS_SUCCESS;
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    } else if (*Request).Flags & KSPROPERTY_TYPE_SET != 0 {
        if (*This).Pin.ClientState != KSSTATE_STOP {
            Status = STATUS_INVALID_DEVICE_STATE;
        } else if !(*Handle).is_null() {
            Mode = ExGetPreviousMode();
            Status = ObReferenceObjectByHandle(
                *Handle,
                SYNCHRONIZE | DIRECTORY_QUERY,
                IoFileObjectType as _,
                Mode,
                &mut FileObject as *mut PFILE_OBJECT as *mut PVOID,
                core::ptr::null_mut(),
            );
            if NT_SUCCESS(Status) {
                Property.Set = KSPROPSETID_Clock;
                Property.Id = KSPROPERTY_CLOCK_FUNCTIONTABLE as ULONG;
                Property.Flags = KSPROPERTY_TYPE_GET as ULONG;
                Status = KsSynchronousIoControlDevice(
                    FileObject,
                    0i8,
                    IOCTL_KS_PROPERTY,
                    &mut Property as *mut KSPROPERTY as PVOID,
                    core::mem::size_of::<KSPROPERTY>() as ULONG,
                    &mut (*This).ClockTable as *mut KSCLOCK_FUNCTIONTABLE as PVOID,
                    core::mem::size_of::<KSCLOCK_FUNCTIONTABLE>() as ULONG,
                    &mut BytesReturned,
                );
                if NT_SUCCESS(Status) {
                    (*This).ClockFileObject = FileObject;
                } else {
                    ObDereferenceObject(FileObject as PVOID);
                }
            }
        } else {
            core::ptr::write_bytes(
                &mut (*This).ClockTable as *mut KSCLOCK_FUNCTIONTABLE as *mut u8,
                0,
                core::mem::size_of::<KSCLOCK_FUNCTIONTABLE>(),
            );
            Status = STATUS_SUCCESS;
            if !(*This).ClockFileObject.is_null() {
                FileObject = (*This).ClockFileObject;
                (*This).ClockFileObject = core::ptr::null_mut();
                ObDereferenceObject(FileObject as PVOID);
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinPipeId(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinStatePropertyHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;
    let This = ((*ObjectHeader).ObjectType as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    let mut Status = STATUS_SUCCESS;
    let mut OldState: KSSTATE = KSSTATE_STOP;
    let NewState = Data as PKSSTATE;

    KeWaitForSingleObject(
        (*This).BasicHeader.ControlMutex as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );

    if (*Request).Flags & KSPROPERTY_TYPE_GET != 0 {
        *NewState = (*This).Pin.DeviceState;
        (*Irp).IoStatus.Information = core::mem::size_of::<KSSTATE>() as _;
    } else if (*Request).Flags & KSPROPERTY_TYPE_SET != 0 {
        if let Some(set_device_state) = (*(*(*This).Pin.Descriptor).Dispatch).SetDeviceState {
            OldState = (*This).Pin.ClientState;
            (*This).Pin.ClientState = *NewState;
            (*This).Pin.DeviceState = KSSTATE_RUN;
            Status = set_device_state(&raw mut (*This).Pin, *NewState, OldState);
            if !NT_SUCCESS(Status) {
                (*This).Pin.ClientState = OldState;
                (*This).Pin.DeviceState = OldState;
                DbgBreakPoint();
            } else {
                (*This).Pin.DeviceState = *NewState;
            }
        } else {
            (*This).Pin.DeviceState = *NewState;
            (*This).Pin.ClientState = *NewState;
        }
    }

    KeReleaseMutex((*This).BasicHeader.ControlMutex, 0u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinAllocatorFramingPropertyHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let ObjectHeader: PKSIOBJECT_HEADER;
    let This: *mut IKsPinImpl;
    let Size: ULONG;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;

    This = ((*ObjectHeader).ObjectType as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;

    KeWaitForSingleObject(
        (*This).BasicHeader.ControlMutex as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );

    if !((*((*This).Pin.Descriptor)).AllocatorFraming).is_null() {
        Size = (core::mem::size_of::<KSALLOCATOR_FRAMING_EX>() as ULONG)
            .wrapping_add(
                (*(*((*This).Pin.Descriptor)).AllocatorFraming).CountItems as ULONG
                    * core::mem::size_of::<KS_FRAMING_ITEM>() as ULONG,
            )
            .wrapping_sub(core::mem::size_of::<KS_FRAMING_ITEM>() as ULONG);

        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength == 0 {
            Status = STATUS_BUFFER_OVERFLOW;
        } else if Size > (*IoStack).Parameters.DeviceIoControl.OutputBufferLength {
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            core::ptr::copy_nonoverlapping(
                ((*((*This).Pin.Descriptor)).AllocatorFraming) as *const u8,
                Data as *mut u8,
                Size as usize,
            );
            Status = STATUS_SUCCESS;
        }

        (*Irp).IoStatus.Information = Size as _;
    } else {
        Status = STATUS_NOT_FOUND;
    }

    KeReleaseMutex((*This).BasicHeader.ControlMutex, 0u8);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinDataFormatPropertyHandler(
    Irp: PIRP,
    Request: PKSPROPERTY,
    Data: PVOID,
) -> NTSTATUS {
    let IoStack = IoGetCurrentIrpStackLocation(Irp);
    let ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;
    let This = ((*ObjectHeader).ObjectType as usize).wrapping_sub(
        core::mem::offset_of!(IKsPinImpl, Pin),
    ) as *mut IKsPinImpl;

    KeWaitForSingleObject(
        (*This).BasicHeader.ControlMutex as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    let mut Status = STATUS_SUCCESS;

    if (*Request).Flags & KSPROPERTY_TYPE_GET != 0 {
        if (*IoStack).Parameters.DeviceIoControl.OutputBufferLength < (*(*This).Pin.ConnectionFormat).__bindgen_anon_1.as_ref().FormatSize {
            (*Irp).IoStatus.Information = (*(*This).Pin.ConnectionFormat).__bindgen_anon_1.as_ref().FormatSize as _;
            Status = STATUS_BUFFER_TOO_SMALL;
        } else {
            core::ptr::copy_nonoverlapping(
                (*This).Pin.ConnectionFormat as *const u8,
                Data as *mut u8,
                (*(*This).Pin.ConnectionFormat).__bindgen_anon_1.as_ref().FormatSize as usize,
            );
        }
    } else if (*Request).Flags & KSPROPERTY_TYPE_SET != 0 {
        if (*(*This).Pin.Descriptor).Flags & KSPIN_FLAG_FIXED_FORMAT != 0 {
            Status = STATUS_INVALID_DEVICE_REQUEST;
        } else {
            Status = _KsEdit(
                (*This).Pin.Bag,
                &raw mut (*This).Pin.ConnectionFormat as *mut PVOID,
                (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
                (*(*This).Pin.ConnectionFormat).__bindgen_anon_1.as_ref().FormatSize,
                0,
            );
            if NT_SUCCESS(Status) {
                core::ptr::copy_nonoverlapping(
                    Data as *const u8,
                    (*This).Pin.ConnectionFormat as *mut u8,
                    (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize,
                );
            }
        }
    }

    KeReleaseMutex((*This).BasicHeader.ControlMutex, 0u8);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnQueryInterface(
    iface: *mut IKsPin,
    refiid: *const IID,
    Output: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let This = (iface as *mut u8).sub(
        core::mem::offset_of!(IKsPinImpl, BasicHeader.OuterUnknown),
    ) as *mut IKsPinImpl;

    if IsEqualGUIDAligned(refiid, &raw const IID_IUnknown) {
        *Output = &raw mut (*This).BasicHeader.OuterUnknown as *mut core::ffi::c_void;
        _InterlockedIncrement(&raw mut (*This).ref_);
        return STATUS_SUCCESS;
    }

    if !(*This).BasicHeader.ClientAggregate.is_null() {
        let Status = (*(*(*This).BasicHeader.ClientAggregate).lpVtbl).QueryInterface.expect("non-null function pointer")(
            (*This).BasicHeader.ClientAggregate as *mut IUnknown,
            refiid,
            Output,
        );
        if NT_SUCCESS(Status) {
            return Status;
        }
    }

    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnAddRef(iface: *mut IKsPin) -> ULONG {
    let This = ((iface as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, BasicHeader.OuterUnknown)))
        as *mut IKsPinImpl;
    return InterlockedIncrement(&mut (*This).ref_) as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnRelease(iface: *mut IKsPin) -> ULONG {
    let This: *mut IKsPinImpl = CONTAINING_RECORD!(iface, IKsPinImpl, BasicHeader.OuterUnknown);

    InterlockedDecrement(&mut (*This).ref_);

    if (*This).ref_ == 0 {
        FreeItem(This as PVOID);
        return 0;
    }
    return (*This).ref_ as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnTransferKsIrp(
    iface: *mut IKsPin,
    Irp: PIRP,
    OutTransport: *mut *mut IKsTransport,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnDiscardKsIrp(
    iface: *mut IKsPin,
    Irp: PIRP,
    OutTransport: *mut *mut IKsTransport,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"IKsPin_fnDiscardKsIrp\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const ::core::ffi::c_char,
        561i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnConnect(
    iface: *mut IKsPin,
    TransportIn: *mut IKsTransport,
    OutTransportIn: *mut *mut IKsTransport,
    OutTransportOut: *mut *mut IKsTransport,
    DataFlow: KSPIN_DATAFLOW,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnSetDeviceState(
    iface: *mut IKsPin,
    OldState: KSSTATE,
    NewState: KSSTATE,
    OutTransport: *mut *mut IKsTransport,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnSetResetState(
    iface: *mut IKsPin,
    ResetState: KSRESET,
    OutTransportOut: *mut *mut IKsTransport,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsPin_fnSetResetState\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const i8,
        597i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnGetTransportConfig(
    iface: *mut IKsPin,
    TransportConfig: *mut KSPTRANSPORTCONFIG,
    OutTransportIn: *mut *mut IKsTransport,
    OutTransportOut: *mut *mut IKsTransport,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnResetTransportConfig(
    iface: *mut IKsPin,
    OutTransportIn: *mut *mut IKsTransport,
    OutTransportOut: *mut *mut IKsTransport,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnGetStruct(
    iface: *mut IKsPin,
) -> PKSPIN {
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnGetProcessPin(
    iface: *mut IKsPin,
) -> PKSPROCESSPIN {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnAttemptBypass(
    iface: *mut IKsPin,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsPin_fnAttemptBypass\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const i8,
        658i32,
    );
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnAttemptUnbypass(
    iface: *mut IKsPin,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"IKsPin_fnAttemptUnbypass\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const i8,
        667i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnGenerateConnectionEvents(
    iface: *mut IKsPin,
    EventMask: ULONG,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsPin_fnGenerateConnectionEvents\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const i8,
        677i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_fnClientSetDeviceState(
    iface: *mut IKsPin,
    StateIn: KSSTATE,
    StateOut: KSSTATE,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsPin_fnClientSetDeviceState\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/pin.c\0".as_ptr() as *const i8,
        687i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}
#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnQueryInterface(
    iface: *mut IKsReferenceClock,
    refiid: *const IID,
    Output: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let This: *mut IKsPinImpl = (iface as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, lpVtblReferenceClock))
        as *mut IKsPinImpl;
    IKsPin_fnQueryInterface(
        core::ptr::addr_of_mut!((*This).BasicHeader.OuterUnknown) as *mut IKsPin,
        refiid,
        Output,
    )
}
#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnAddRef(iface: *mut IKsReferenceClock) -> ULONG {
    let This: *mut IKsPinImpl = CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
    return IKsPin_fnAddRef(addr_of_mut!((*This).BasicHeader.OuterUnknown) as *mut IKsPin);
}
#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnRelease(iface: *mut IKsReferenceClock) -> ULONG {
    let This: *mut IKsPinImpl = core::ptr::with_exposed_provenance_mut::<IKsPinImpl>((iface as usize).wrapping_sub(
            core::mem::offset_of!(IKsPinImpl, lpVtblReferenceClock),
        )
    );
    return IKsPin_fnRelease(&raw mut (*This).BasicHeader.OuterUnknown as *mut IKsPin);
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetTime(
    iface: *mut IKsReferenceClock,
) -> LONGLONG {
    let This = CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock) as *mut IKsPinImpl;
    let Result: LONGLONG;
    if (*This).ClockFileObject.is_null() || (*This).ClockTable.GetTime.is_none() {
        Result = 0;
    } else {
        Result = (*This).ClockTable.GetTime.unwrap()((*This).ClockFileObject);
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetPhysicalTime(
    iface: *mut IKsReferenceClock,
) -> LONGLONG {
    let This = CONTAINING_RECORD!(
        iface,
        IKsPinImpl,
        lpVtblReferenceClock
    ) as *mut IKsPinImpl;

    // DPRINT removed

    let Result: LONGLONG;
    if (*This).ClockFileObject.is_null() || (*This).ClockTable.GetPhysicalTime.is_none() {
        Result = 0;
    } else {
        Result = ((*This).ClockTable.GetPhysicalTime.unwrap())((*This).ClockFileObject);
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetCorrelatedTime(
    iface: *mut IKsReferenceClock,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    let This = (iface as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, lpVtblReferenceClock))
        as *mut IKsPinImpl;

    if (*This).ClockFileObject.is_null() || (*This).ClockTable.GetCorrelatedTime.is_none() {
        0
    } else {
        (*This)
            .ClockTable
            .GetCorrelatedTime
            .expect("non-null function pointer")((*This).ClockFileObject, SystemTime)
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetCorrelatedPhysicalTime(
    iface: *mut IKsReferenceClock,
    SystemTime: PLONGLONG,
) -> LONGLONG {
    let This = CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock) as *mut IKsPinImpl;
    let Result: LONGLONG;
    if (*This).ClockFileObject.is_null() || (*This).ClockTable.GetCorrelatedPhysicalTime.is_none() {
        Result = 0;
    } else {
        Result = ((*This).ClockTable.GetCorrelatedPhysicalTime.unwrap())(
            (*This).ClockFileObject,
            SystemTime,
        );
    }
    Result
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetResolution(
    iface: *mut IKsReferenceClock,
    Resolution: *mut KSRESOLUTION,
) -> NTSTATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;

    let This = CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);

    if (*This).ClockFileObject.is_null() {
        (*Resolution).Error = 0;
        (*Resolution).Granularity = 1;
        return STATUS_SUCCESS;
    }

    if (*This).ClockFileObject.is_null() {
        return STATUS_DEVICE_NOT_READY;
    }

    Property.Set = KSPROPSETID_Clock;
    Property.Id = KSPROPERTY_CLOCK_RESOLUTION;
    Property.Flags = KSPROPERTY_TYPE_GET;

    KsSynchronousIoControlDevice(
        (*This).ClockFileObject,
        0i8,
        IOCTL_KS_PROPERTY,
        &mut Property as *mut KSPROPERTY as *mut core::ffi::c_void,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        Resolution as *mut core::ffi::c_void,
        core::mem::size_of::<KSRESOLUTION>() as ULONG,
        &mut BytesReturned,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IKsReferenceClock_fnGetState(
    iface: *mut IKsReferenceClock,
    State: PKSSTATE,
) -> NTSTATUS {
    let mut Property: KSPROPERTY = core::mem::zeroed();
    let mut BytesReturned: ULONG = 0;

    let This = core::ptr::with_exposed_provenance_mut::<IKsPinImpl>((iface as usize).wrapping_sub(
            core::mem::offset_of!(IKsPinImpl, lpVtblReferenceClock),
        ),
    );

    if (*This).ClockFileObject.is_null() {
        *State = (*This).Pin.ClientState;
        return STATUS_SUCCESS;
    }

    if (*This).ClockFileObject.is_null() {
        return STATUS_DEVICE_NOT_READY;
    }

    Property.Set = KSPROPSETID_Clock;
    Property.Id = KSPROPERTY_CLOCK_RESOLUTION as ULONG;
    Property.Flags = KSPROPERTY_TYPE_GET as ULONG;

    KsSynchronousIoControlDevice(
        (*This).ClockFileObject,
        0i8,
        IOCTL_KS_PROPERTY,
        &raw mut Property as PVOID,
        core::mem::size_of::<KSPROPERTY>() as ULONG,
        State as PVOID,
        core::mem::size_of::<KSSTATE>() as ULONG,
        &raw mut BytesReturned,
    )
}

#[no_mangle]
pub unsafe extern "system" fn KsPinAcquireProcessingMutex(Pin: PKSPIN) {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    KeWaitForSingleObject(
        &mut (*This).ProcessingMutex as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsPinAttachAndGate(
    Pin: PKSPIN,
    AndGate: PKSGATE,
) {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    (*This).AttachedGate = AndGate;
    (*This).OrGate = 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinAttachOrGate(
    Pin: PKSPIN,
    OrGate: PKSGATE,
) {
    let This: *mut IKsPinImpl = CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
    (*This).AttachedGate = OrGate;
    (*This).OrGate = 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetAndGate(
    Pin: PKSPIN,
) -> PKSGATE {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    return (*This).AttachedGate;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinAttemptProcessing(
    Pin: PKSPIN,
    Asynchronous: BOOLEAN,
) {
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetAvailableByteCount(
    Pin: PKSPIN,
    InputDataBytes: PLONG,
    OutputBufferBytes: PLONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetConnectedFilterInterface(
    Pin: PKSPIN,
    InterfaceId: *const GUID,
    Interface: *mut PVOID,
) -> NTSTATUS {
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetConnectedPinDeviceObject(
    Pin: PKSPIN,
) -> PDEVICE_OBJECT {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetConnectedPinFileObject(
    Pin: PKSPIN,
) -> PFILE_OBJECT {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetConnectedPinInterface(
    Pin: PKSPIN,
    InterfaceId: *const GUID,
    Interface: *mut PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetCopyRelationships(
    Pin: PKSPIN,
    CopySource: *mut PKSPIN,
    DelegateBranch: *mut PKSPIN,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"KsPinGetCopyRelationships\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/pin.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        1078i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetNextSiblingPin(
    Pin: PKSPIN,
) -> PKSPIN {
    return KsGetNextSibling(Pin as PVOID) as PKSPIN;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetParentFilter(
    Pin: PKSPIN,
) -> PKSFILTER {
    let This = (Pin as *mut u8)
        .sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    (*This).BasicHeader.Parent.KsFilter
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetReferenceClockInterface(
    Pin: PKSPIN,
    Interface: *mut PIKSREFERENCECLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_DEVICE_NOT_READY;
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    if !(*This).ClockFileObject.is_null() {
        *Interface = &raw mut ((*This).lpVtblReferenceClock) as PIKSREFERENCECLOCK;
        Status = STATUS_SUCCESS;
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinRegisterFrameReturnCallback(
    Pin: PKSPIN,
    FrameReturn: PFNKSPINFRAMERETURN,
) {
    let This = (Pin as *mut IKsPinImpl).byte_sub(
        core::mem::offset_of!(IKsPinImpl, Pin)
    ) as *mut IKsPinImpl;
    (*This).FrameReturn = FrameReturn;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinRegisterHandshakeCallback(
    Pin: PKSPIN,
    Handshake: PFNKSPINHANDSHAKE,
) {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    (*This).Handshake = Handshake;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinRegisterIrpCompletionCallback(
    Pin: PKSPIN,
    IrpCompletion: PFNKSPINIRPCOMPLETION,
) {
    let This = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;
    (*This).IrpCompletion = IrpCompletion;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinRegisterPowerCallbacks(
    Pin: PKSPIN,
    Sleep: PFNKSPINPOWER,
    Wake: PFNKSPINPOWER,
) {
    let This: *mut IKsPinImpl = CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
    (*This).Sleep = Sleep;
    (*This).Wake = Wake;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinReleaseProcessingMutex(
    Pin: PKSPIN,
) {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;

    KeReleaseMutex(&raw mut (*This).ProcessingMutex, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn KsGetPinFromIrp(
    Irp: PIRP,
) -> PKSPIN {
    let ObjectHeader: PKSIOBJECT_HEADER;
    let Pin: PKSPIN;
    let Header: PKSBASIC_HEADER;
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*(*IoStack).FileObject).FsContext2 as PKSIOBJECT_HEADER;

    if ObjectHeader.is_null() {
        return core::ptr::null_mut();
    }

    Pin = (*ObjectHeader).ObjectType as PKSPIN;
    Header = (Pin as usize).wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>()) as PKSBASIC_HEADER;

    if !((*Header).Type as u32 == KsObjectTypePin as u32) {
        RtlAssert(
            b"Header->Type == KsObjectTypePin\0".as_ptr() as PVOID,
            b"drivers/ksfilter/ks/pin.c\0".as_ptr() as PVOID,
            1231u32,
            core::ptr::null_mut(),
        );
    }

    Pin
}

#[no_mangle]
pub unsafe extern "system" fn KsPinSetPinClockTime(
    Pin: PKSPIN,
    Time: LONGLONG,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"KsPinSetPinClockTime\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/pin.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        1248i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsPinSubmitFrame(
    Pin: PKSPIN,
    Data: PVOID,
    Size: ULONG,
    StreamHeader: PKSSTREAM_HEADER,
    Context: PVOID,
) -> NTSTATUS {
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn KsPinSubmitFrameMdl(
    Pin: PKSPIN,
    Mdl: PMDL,
    StreamHeader: PKSSTREAM_HEADER,
    Context: PVOID,
) -> NTSTATUS {
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn KsProcessPinUpdate(
    ProcessPin: PKSPROCESSPIN,
) -> BOOLEAN {
    return 0u8;
}
#[no_mangle]
pub unsafe extern "C" fn IKsPin_PrepareStreamHeader(
    This: *mut IKsPinImpl,
    StreamPointer: PKSISTREAM_POINTER,
) -> NTSTATUS {
    let mut Header: PKSSTREAM_HEADER = core::ptr::null_mut();
    let mut Length: ULONG;

    (*StreamPointer).Irp = KsRemoveIrpFromCancelableQueue(
        &mut (*This).IrpList,
        &raw mut (*This).IrpListLock as *mut _,
        KsListEntryHead,
        KsAcquireAndRemoveOnlySingleItem,
    );
    if (*StreamPointer).Irp.is_null() {
        return STATUS_DEVICE_NOT_READY;
    }

    InterlockedDecrement(&mut (*This).IrpCount);
    KsDecrementCountedWorker((*This).PinWorker);

    if (*(*StreamPointer).Irp).RequestorMode as i32 == 1i32 as i32 {
        Header = (*(*StreamPointer).Irp).AssociatedIrp.SystemBuffer as PKSSTREAM_HEADER;
    } else {
        Header = (*(*StreamPointer).Irp).UserBuffer as PKSSTREAM_HEADER;
    }

    (*StreamPointer).Callback = None;
    (*StreamPointer).Length = if (*Header).DataUsed > (*Header).FrameExtent {
        (*Header).DataUsed
    } else {
        (*Header).FrameExtent
    };
    (*StreamPointer).Next = core::ptr::null_mut();
    (*StreamPointer).Offset = 0;
    (*StreamPointer).Pin = &mut (*This).Pin;
    (*StreamPointer).Data = (*Header).Data;

    (*StreamPointer).StreamPointer.Context = core::ptr::null_mut();
    (*StreamPointer).StreamPointer.Pin = &mut (*This).Pin;
    (*StreamPointer).StreamPointer.StreamHeader = Header;

    if (*(*This).Pin.Descriptor).PinDescriptor.DataFlow as u32 == KSPIN_DATAFLOW_IN as u32 {
        (*StreamPointer).StreamPointer.Offset = &mut (*StreamPointer).StreamPointer.OffsetIn;
    } else {
        (*StreamPointer).StreamPointer.Offset = &mut (*StreamPointer).StreamPointer.OffsetOut;
    }

    (*(*StreamPointer).StreamPointer.Offset).Count = 0;
    *(*(*StreamPointer).StreamPointer.Offset).__bindgen_anon_1.Data.as_mut() = core::ptr::null_mut();
    (*(*StreamPointer).StreamPointer.Offset).Remaining = 0;

    Length = (*StreamPointer).Length;

    (*(*StreamPointer).StreamPointer.Offset).Count = Length;
    (*(*StreamPointer).StreamPointer.Offset).Remaining = Length;
    *(*(*StreamPointer).StreamPointer.Offset).__bindgen_anon_1.Data.as_mut() = ((*StreamPointer).Data as usize)
        .wrapping_add((*StreamPointer).Offset as usize) as *mut u8;
    (*(*StreamPointer).StreamPointer.StreamHeader).FrameExtent = Length;
    if (*(*StreamPointer).StreamPointer.StreamHeader).DataUsed != 0 {
        (*(*StreamPointer).StreamPointer.StreamHeader).DataUsed = Length;
    }
    (*(*StreamPointer).StreamPointer.StreamHeader).Data = *(*(*StreamPointer).StreamPointer.Offset).__bindgen_anon_1.Data.as_ref() as *mut core::ffi::c_void;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetLeadingEdgeStreamPointer(
    Pin: PKSPIN,
    State: KSSTREAM_POINTER_STATE,
) -> PKSSTREAM_POINTER {
    let This: *mut IKsPinImpl;
    let Status: NTSTATUS;

    This = CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);

    if State == KSSTREAM_POINTER_STATE_LOCKED {
        if (*This).LeadingEdgeStreamPointer.Irp.is_null()
            || (*(*This).LeadingEdgeStreamPointer.StreamPointer.Offset).Remaining == 0
        {
            Status = IKsPin_PrepareStreamHeader(This, &mut (*This).LeadingEdgeStreamPointer);
            if !NT_SUCCESS(Status) {
                return core::ptr::null_mut();
            }
        }
    }

    return &mut (*This).LeadingEdgeStreamPointer.StreamPointer;
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetTrailingEdgeStreamPointer(
    Pin: PKSPIN,
    State: KSSTREAM_POINTER_STATE,
) -> PKSSTREAM_POINTER {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerSetStatusCode(
    StreamPointer: PKSSTREAM_POINTER,
    Status: NTSTATUS,
) -> NTSTATUS {
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerLock(
    StreamPointer: PKSSTREAM_POINTER,
) -> NTSTATUS {
    return STATUS_UNSUCCESSFUL;
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerUnlock(
    StreamPointer: PKSSTREAM_POINTER,
    Eject: BOOLEAN,
) {
    let Pointer: PKSISTREAM_POINTER = CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
    (*Pointer).Irp = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerAdvanceOffsetsAndUnlock(
    StreamPointer: PKSSTREAM_POINTER,
    InUsed: ULONG,
    OutUsed: ULONG,
    Eject: BOOLEAN,
) {
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerDelete(
    StreamPointer: PKSSTREAM_POINTER,
) {
    let Pointer: PKSISTREAM_POINTER = core::ptr::with_exposed_provenance_mut::<_KSISTREAM_POINTER>((StreamPointer as usize)
            .wrapping_sub(core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer))
    );
    DbgBreakPoint();
    let This: *mut IKsPinImpl = core::ptr::with_exposed_provenance_mut::<IKsPinImpl>(((*Pointer).StreamPointer.Pin as usize)
            .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
    );
    let mut Last: PKSISTREAM_POINTER = core::ptr::null_mut();
    let mut Cur: PKSISTREAM_POINTER = (*This).ClonedStreamPointer;
    while Cur != Pointer && !Cur.is_null() {
        Last = Cur;
        Cur = (*Cur).Next;
    }
    if Cur.is_null() {
        return;
    }
    if Last.is_null() {
        (*This).ClonedStreamPointer = (*Pointer).Next;
    } else {
        (*Last).Next = (*Pointer).Next;
    }
    FreeItem(Pointer as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerClone(
    StreamPointer: PKSSTREAM_POINTER,
    CancelCallback: PFNKSSTREAMPOINTER,
    ContextSize: ULONG,
    CloneStreamPointer: *mut PKSSTREAM_POINTER,
) -> NTSTATUS {
    let mut This: *mut IKsPinImpl = core::ptr::null_mut();
    let mut CurFrame: PKSISTREAM_POINTER = core::ptr::null_mut();
    let mut NewFrame: PKSISTREAM_POINTER = core::ptr::null_mut();
    let mut RefCount: ULONG_PTR = 0;
    let mut Status: NTSTATUS = 0;
    let mut Size: ULONG = 0;

    CurFrame = (StreamPointer as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer) as ULONG_PTR)
        as PKSISTREAM_POINTER;

    Size = (core::mem::size_of::<KSISTREAM_POINTER>() as ULONG).wrapping_add(ContextSize);

    NewFrame = AllocateItem(NonPagedPool, Size as SIZE_T) as PKSISTREAM_POINTER;

    if NewFrame.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RefCount = (*(*CurFrame).Irp)
        .Tail
        .Overlay
        .__bindgen_anon_1
        .__bindgen_anon_1
        .DriverContext[0] as ULONG_PTR;

    RefCount = RefCount.wrapping_add(1);

    (*(*CurFrame).Irp)
        .Tail
        .Overlay
        .__bindgen_anon_1
        .__bindgen_anon_1
        .DriverContext[0] = RefCount as PVOID;

    core::ptr::copy_nonoverlapping(
        CurFrame as *const u8,
        NewFrame as *mut u8,
        core::mem::size_of::<KSISTREAM_POINTER>(),
    );

    This = ((*CurFrame).Pin as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin) as ULONG_PTR)
        as *mut IKsPinImpl;

    if (*(*CurFrame).StreamPointer.Offset).Remaining == 0 {
        Status = IKsPin_PrepareStreamHeader(This, NewFrame);
        if !NT_SUCCESS(Status) {
            FreeItem(NewFrame as PVOID);
            return STATUS_DEVICE_NOT_READY;
        }
    }

    if ContextSize != 0 {
        (*NewFrame).StreamPointer.Context = NewFrame.offset(1) as PVOID;
    }

    if (*(*This).Pin.Descriptor).PinDescriptor.DataFlow as u32 == KSPIN_DATAFLOW_IN as u32 {
        (*NewFrame).StreamPointer.Offset = core::ptr::addr_of_mut!((*NewFrame).StreamPointer.OffsetIn);
    } else {
        (*NewFrame).StreamPointer.Offset = core::ptr::addr_of_mut!((*NewFrame).StreamPointer.OffsetOut);
    }

    (*NewFrame).StreamPointer.Pin = core::ptr::addr_of_mut!((*This).Pin);

    *CloneStreamPointer = core::ptr::addr_of_mut!((*NewFrame).StreamPointer);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerAdvanceOffsets(
    StreamPointer: PKSSTREAM_POINTER,
    InUsed: ULONG,
    OutUsed: ULONG,
    Eject: BOOLEAN,
) -> NTSTATUS {
    let CurFrame: PKSISTREAM_POINTER = (StreamPointer as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer) as ULONG_PTR)
        as *mut KSISTREAM_POINTER as PKSISTREAM_POINTER;

    let This: *mut IKsPinImpl = ((*CurFrame).Pin as ULONG_PTR)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin) as ULONG_PTR)
        as *mut IKsPinImpl;

    DbgBreakPoint();

    if (*(*This).Pin.Descriptor).PinDescriptor.DataFlow as u32 == KSPIN_DATAFLOW_IN as u32 {
        (*CurFrame).StreamPointer.OffsetIn.Remaining = (*CurFrame)
            .StreamPointer
            .OffsetIn
            .Remaining
            .wrapping_sub(InUsed);
        *(*CurFrame).StreamPointer.OffsetIn.__bindgen_anon_1.Data.as_mut() =
            (*(*CurFrame).StreamPointer.OffsetIn.__bindgen_anon_1.Data.as_ref() as ULONG_PTR)
                .wrapping_add(InUsed as ULONG_PTR) as _;
    } else {
        if (*CurFrame).StreamPointer.OffsetOut.Remaining == 0 {
            let Status = IKsPin_PrepareStreamHeader(This, CurFrame);
            if !NT_SUCCESS(Status) {
                return STATUS_DEVICE_NOT_READY;
            }
        } else {
            (*CurFrame).StreamPointer.OffsetOut.Remaining = (*CurFrame)
                .StreamPointer
                .OffsetOut
                .Remaining
                .wrapping_sub(OutUsed);
            *(*CurFrame).StreamPointer.OffsetOut.__bindgen_anon_1.Data.as_mut() =
                (*(*CurFrame).StreamPointer.OffsetOut.__bindgen_anon_1.Data.as_ref() as ULONG_PTR)
                    .wrapping_add(OutUsed as ULONG_PTR) as _;
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerAdvance(
    StreamPointer: PKSSTREAM_POINTER,
) -> NTSTATUS {
    DbgBreakPoint();
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerGetMdl(
    StreamPointer: PKSSTREAM_POINTER,
) -> PMDL {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerGetIrp(
    StreamPointer: PKSSTREAM_POINTER,
    FirstFrameInIrp: PBOOLEAN,
    LastFrameInIrp: PBOOLEAN,
) -> PIRP {
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerScheduleTimeout(
    StreamPointer: PKSSTREAM_POINTER,
    Callback: PFNKSSTREAMPOINTER,
    Interval: ULONGLONG,
) {
    let mut DueTime: LARGE_INTEGER = core::mem::zeroed();
    let Pointer: PKSISTREAM_POINTER = (StreamPointer as usize).wrapping_sub(
        core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer),
    ) as *mut KSISTREAM_POINTER as PKSISTREAM_POINTER;

    (*Pointer).Callback = Callback;
    DueTime.QuadPart = Interval as LONGLONG;
    KeSetTimer(
        &raw mut (*Pointer).Timer as *mut _,
        DueTime,
        &raw mut (*Pointer).TimerDpc as *mut _,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerCancelTimeout(
    StreamPointer: PKSSTREAM_POINTER,
) {
    let Pointer: PKSISTREAM_POINTER = {
        let base = core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer);
        (StreamPointer as usize).wrapping_sub(base) as *mut KSISTREAM_POINTER
    };

    KeCancelTimer(core::ptr::addr_of_mut!((*Pointer).Timer));
}

#[no_mangle]
pub unsafe extern "system" fn KsPinGetFirstCloneStreamPointer(
    Pin: PKSPIN,
) -> PKSSTREAM_POINTER {
    let This: *mut IKsPinImpl = (Pin as usize)
        .wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin))
        as *mut IKsPinImpl;

    if (*This).ClonedStreamPointer.is_null() {
        return core::ptr::null_mut();
    }

    return &raw mut (*(*This).ClonedStreamPointer).StreamPointer;
}

#[no_mangle]
pub unsafe extern "system" fn KsStreamPointerGetNextClone(
    StreamPointer: PKSSTREAM_POINTER,
) -> PKSSTREAM_POINTER {
    let Pointer: PKSISTREAM_POINTER;
    DbgBreakPoint();
    Pointer = (StreamPointer as usize).wrapping_sub(
        core::mem::offset_of!(KSISTREAM_POINTER, StreamPointer),
    ) as *mut KSISTREAM_POINTER as PKSISTREAM_POINTER;
    if (*Pointer).Next.is_null() {
        return core::ptr::null_mut::<_KSSTREAM_POINTER>();
    }
    return &raw mut (*(*Pointer).Next).StreamPointer;
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_PinCentricWorker(Parameter: PVOID) {
    let This = Parameter as *mut IKsPinImpl;
    loop {
        let Status = ((*(*(*This).Pin.Descriptor).Dispatch).Process).expect("non-null function pointer")(&mut (*This).Pin);
        if (*This).IrpCount == 0 {
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_DispatchKsStream(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    This: *mut IKsPinImpl,
) -> NTSTATUS {
    let mut ProcessPinIndex: PKSPROCESSPIN_INDEXENTRY = core::ptr::null_mut();
    let mut Header: PKSSTREAM_HEADER = core::ptr::null_mut();
    let mut NumHeaders: ULONG = 0;
    let mut Filter: PKSFILTER = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    ASSERT!((*This).Pin.ResetState == KSRESET_END);

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_WRITE_STREAM {
        Status = KsProbeStreamIrp(
            Irp,
            (KSSTREAM_WRITE | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK | KSPROBE_SYSTEMADDRESS) as ULONG,
            (*This).Pin.StreamHeaderSize,
        );
    } else {
        Status = KsProbeStreamIrp(
            Irp,
            (KSSTREAM_READ | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK | KSPROBE_SYSTEMADDRESS) as ULONG,
            (*This).Pin.StreamHeaderSize,
        );
    }

    if !NT_SUCCESS(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }

    if (*Irp).RequestorMode == 1i8 {
        Header = (*Irp).AssociatedIrp.SystemBuffer as PKSSTREAM_HEADER;
    } else {
        Header = (*Irp).UserBuffer as PKSSTREAM_HEADER;
    }

    if Header.is_null() {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }

    NumHeaders = (*IoStack).Parameters.DeviceIoControl.OutputBufferLength / (*Header).Size;

    ASSERT!((*IoStack).Parameters.DeviceIoControl.OutputBufferLength % (*Header).Size == 0);
    ASSERT!(NumHeaders == 1);

    if (*Irp).RequestorMode == 1i8 {
        ASSERT!(!(*Irp).MdlAddress.is_null());
        (*Header).Data = MmGetSystemAddressForMdlSafe((*Irp).MdlAddress, NormalPagePriority);

        if (*Header).Data.is_null() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_INSUFFICIENT_RESOURCES;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }
    }

    if (*(*(*This).Pin.Descriptor).Dispatch).Process.is_some() {
        IoMarkIrpPending(Irp);
        KsAddIrpToCancelableQueue(
            &mut (*This).IrpList,
            &raw mut (*This).IrpListLock as *mut _,
            Irp,
            KsListEntryTail,
            None,
        );

        // C: `ASSERT(!(Flags & KSPIN_FLAG_DISPATCH_LEVEL_PROCESSING))`. Rust's `!` is
        // bitwise on a ULONG, so this could never be zero — vacuous until spelled `== 0`.
        ASSERT!(((*(*This).Pin.Descriptor).Flags & KSPIN_FLAG_DISPATCH_LEVEL_PROCESSING as ULONG) == 0);
        ASSERT!(!(*This).PinWorker.is_null());

        InterlockedIncrement(&mut (*This).IrpCount);
        KsIncrementCountedWorker((*This).PinWorker);
        Status = STATUS_PENDING;
    } else {
        ASSERT!(!(*This).Filter.is_null());

        ProcessPinIndex = (*(*(*This).Filter).lpVtbl).GetProcessDispatch.expect("non-null function pointer")((*This).Filter);
        Filter = (*(*(*This).Filter).lpVtbl).GetStruct.expect("non-null function pointer")((*This).Filter);

        ASSERT!(!ProcessPinIndex.is_null());
        ASSERT!(!Filter.is_null());
        ASSERT!(!(*Filter).Descriptor.is_null());
        ASSERT!(!(*(*Filter).Descriptor).Dispatch.is_null());

        if (*(*(*Filter).Descriptor).Dispatch).Process.is_none() {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_UNSUCCESSFUL;
        }

        IoMarkIrpPending(Irp);
        KsAddIrpToCancelableQueue(
            &mut (*This).IrpList,
            &raw mut (*This).IrpListLock as *mut _,
            Irp,
            KsListEntryTail,
            None,
        );

        Status = (*(*(*Filter).Descriptor).Dispatch).Process.expect("non-null function pointer")(Filter, ProcessPinIndex);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_DispatchDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let mut ObjectHeader: PKSIOBJECT_HEADER;
    let mut This: *mut IKsPinImpl;
    let mut Status: NTSTATUS;
    let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
    let mut Property: PKSPROPERTY;
    let mut SetCount: ULONG = 0;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*((*IoStack).FileObject)).FsContext2 as PKSIOBJECT_HEADER;

    This = ((*ObjectHeader).ObjectType as usize).wrapping_sub(
        core::mem::offset_of!(IKsPinImpl, Pin),
    ) as *mut IKsPinImpl;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_READ_STREAM
        || (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_WRITE_STREAM
    {
        return IKsPin_DispatchKsStream(DeviceObject, Irp, This);
    }

    Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;

    RtlStringFromGUID(&raw mut (*Property).Set, &raw mut GuidString);
    RtlFreeUnicodeString(&raw mut GuidString);

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_METHOD {
        let mut MethodSet: *const KSMETHOD_SET = core::ptr::null();
        let mut MethodItemSize: ULONG = 0;

        if (*(*(*This).Pin.Descriptor).AutomationTable).MethodSetsCount != 0 {
            SetCount = (*(*(*This).Pin.Descriptor).AutomationTable).MethodSetsCount;
            MethodSet = (*(*(*This).Pin.Descriptor).AutomationTable).MethodSets;
            MethodItemSize = (*(*(*This).Pin.Descriptor).AutomationTable).MethodItemSize;
        }

        Status = KspMethodHandlerWithAllocator(Irp, SetCount, MethodSet, None, MethodItemSize);
    } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY {
        let mut PropertySet: *const KSPROPERTY_SET = core::ptr::null();
        let mut PropertyItemSize: ULONG = 0;

        if (*(*(*This).Pin.Descriptor).AutomationTable).PropertySetsCount != 0 {
            SetCount = (*(*(*This).Pin.Descriptor).AutomationTable).PropertySetsCount;
            PropertySet = (*(*(*This).Pin.Descriptor).AutomationTable).PropertySets;
            PropertyItemSize = (*(*(*This).Pin.Descriptor).AutomationTable).PropertyItemSize;
        }

        *((&raw mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext as *mut PVOID).offset(3) as *mut *const KSPROPERTY_ITEM) = This as *const KSPROPERTY_ITEM;

        Status = KspPropertyHandler(Irp, SetCount, PropertySet, None, PropertyItemSize);
    } else {
        if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_ENABLE_EVENT {
            Status = KspEnableEvent(
                Irp,
                (*(*(*This).Pin.Descriptor).AutomationTable).EventSetsCount,
                (*(*(*This).Pin.Descriptor).AutomationTable).EventSets as PKSEVENT_SET,
                &raw mut (*This).BasicHeader.EventList,
                KSEVENTS_SPINLOCK,
                &raw mut (*This).BasicHeader.EventListLock as PVOID,
                None,
                (*(*(*This).Pin.Descriptor).AutomationTable).EventItemSize,
            );
        } else {
            Status = KsDisableEvent(
                Irp,
                &raw mut (*This).BasicHeader.EventList,
                KSEVENTS_SPINLOCK,
                &raw mut (*This).BasicHeader.EventListLock as PVOID,
            );
        }
    }

    RtlStringFromGUID(&raw mut (*Property).Set, &raw mut GuidString);
    RtlFreeUnicodeString(&raw mut GuidString);

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_Close(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);

    let ObjectHeader: PKSIOBJECT_HEADER = (*IoStack).FileObject.as_ref().unwrap().FsContext2 as PKSIOBJECT_HEADER;

    let This: *mut IKsPinImpl = {
        let offset = core::mem::offset_of!(IKsPinImpl, Pin);
        ((*ObjectHeader).ObjectType as usize).wrapping_sub(offset) as *mut IKsPinImpl
    };

    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if let Some(Close) = (*(*(*This).Pin.Descriptor).Dispatch).Close {
        Status = Close(&mut (*This).Pin, Irp);

        if !NT_SUCCESS(Status) {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }

        IKsFilter_RemovePin(
            (*((*(*This).Filter).lpVtbl))
                .GetStruct
                .expect("non-null function pointer")((*This).Filter),
            &mut (*This).Pin,
        );

        if Status != STATUS_PENDING {
            (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
            CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return Status;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_DispatchCreateAllocator(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    CompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_DispatchCreateClock(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Pin: PKSPIN = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut This: *mut IKsPinImpl = core::ptr::null_mut();
    let mut Resolution: KSRESOLUTION = KSRESOLUTION {
        Granularity: 0,
        Error: 0,
    };
    let mut pResolution: PKSRESOLUTION = core::ptr::null_mut();
    let mut CreateItem: PKSOBJECT_CREATE_ITEM = core::ptr::null_mut();

    CreateItem = *((&mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext as *mut PVOID).offset(0) as *mut PKSOBJECT_CREATE_ITEM);

    Pin = (*CreateItem).Context as PKSPIN;

    This = (Pin as usize).wrapping_sub(core::mem::offset_of!(IKsPinImpl, Pin)) as *mut IKsPinImpl;

    KsAcquireControl(Pin as PVOID);

    if ((*(*This).Pin.Descriptor).PinDescriptor.Communication != KSPIN_COMMUNICATION_NONE &&
        !(*(*This).Pin.Descriptor).Dispatch.is_null()) ||
        ((*(*This).Pin.Descriptor).Flags & KSPIN_FLAG_IMPLEMENT_CLOCK) != 0
    {
        if (*This).DefaultClock.is_null() {
            if !(*(*This).Pin.Descriptor).Dispatch.is_null() &&
               !(*(*(*This).Pin.Descriptor).Dispatch).Clock.is_null()
            {
                if let Some(resolution_fn) = (*(*(*(*This).Pin.Descriptor).Dispatch).Clock).Resolution {
                    resolution_fn(&mut (*This).Pin, &mut Resolution);
                    pResolution = &mut Resolution;
                }

                Status = KsAllocateDefaultClockEx(
                    &mut (*This).DefaultClock,
                    &raw mut (*This).Pin as PVOID,
                    core::mem::transmute::<PFNKSPINSETTIMER, PFNKSSETTIMER>(
                        (*(*(*(*This).Pin.Descriptor).Dispatch).Clock).SetTimer,
                    ),
                    core::mem::transmute::<PFNKSPINCANCELTIMER, PFNKSCANCELTIMER>(
                        (*(*(*(*This).Pin.Descriptor).Dispatch).Clock).CancelTimer,
                    ),
                    core::mem::transmute::<PFNKSPINCORRELATEDTIME, PFNKSCORRELATEDTIME>(
                        (*(*(*(*This).Pin.Descriptor).Dispatch).Clock).CorrelatedTime,
                    ),
                    pResolution,
                    0,
                );
            } else {
                Status = KsAllocateDefaultClockEx(
                    &mut (*This).DefaultClock,
                    &raw mut (*This).Pin as PVOID,
                    None,
                    None,
                    None,
                    core::ptr::null(),
                    0,
                );
            }
        }

        if NT_SUCCESS(Status) {
            Status = KsCreateDefaultClock(Irp, (*This).DefaultClock);
        }
    }

    KsReleaseControl(Pin as PVOID);

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsPin_DispatchCreateNode(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
    CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "C" fn KspCreatePin(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    KsDevice: PKSDEVICE,
    FilterFactory: *mut IKsFilterFactory,
    Filter: *mut IKsFilter,
    Connect: PKSPIN_CONNECT,
    Descriptor: *mut KSPIN_DESCRIPTOR_EX,
) -> NTSTATUS {
    let mut This: *mut IKsPinImpl = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Device: *mut IKsDevice = core::ptr::null_mut();
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();
    let mut CreateItem: PKSOBJECT_CREATE_ITEM = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut DataFormat: PKSDATAFORMAT = core::ptr::null_mut();
    let mut BasicHeader: PKSBASIC_HEADER = core::ptr::null_mut();
    let mut Index: ULONG = 0;
    let mut FrameSize: ULONG = 0;
    let mut NumFrames: ULONG = 0;
    let mut AutomationTable: KSAUTOMATION_TABLE = core::mem::zeroed();

    if (*Descriptor).Dispatch.is_null() {
        RtlAssert(
            b"Descriptor->Dispatch\0".as_ptr() as PVOID,
            b"pin.c\0".as_ptr() as PVOID,
            2306,
            core::ptr::null_mut(),
        );
    }

    if !(*Descriptor).AllocatorFraming.is_null() {
        Index = 0;
        while Index < (*(*Descriptor).AllocatorFraming).CountItems {
            FrameSize = (*(*(*Descriptor).AllocatorFraming).FramingItem.as_ptr().offset(Index as isize)).FramingRange.Range.MaxFrameSize;
            NumFrames = (*(*(*Descriptor).AllocatorFraming).FramingItem.as_ptr().offset(Index as isize)).Frames;
            Index = Index.wrapping_add(1);
        }
    }

    Index = 0;
    while Index < (*Descriptor).PinDescriptor.DataRangesCount {
        let mut GuidString: _UNICODE_STRING = core::mem::zeroed();
        RtlStringFromGUID(
            &mut (**(*Descriptor).PinDescriptor.DataRanges.offset(Index as isize)).__bindgen_anon_1.as_mut().MajorFormat,
            &mut GuidString,
        );
        RtlStringFromGUID(
            &mut (**(*Descriptor).PinDescriptor.DataRanges.offset(Index as isize)).__bindgen_anon_1.as_mut().SubFormat,
            &mut GuidString,
        );
        RtlStringFromGUID(
            &mut (**(*Descriptor).PinDescriptor.DataRanges.offset(Index as isize)).__bindgen_anon_1.as_mut().Specifier,
            &mut GuidString,
        );
        RtlStringFromGUID(
            &mut (**(*Descriptor).PinDescriptor.DataRanges.offset(Index as isize)).__bindgen_anon_1.as_mut().Specifier,
            &mut GuidString,
        );
        Index = Index.wrapping_add(1);
    }

    if FrameSize == 0 {
        FrameSize = 9400;
    }
    if NumFrames == 0 {
        NumFrames = 8;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;
    Device = &raw mut (*(*DeviceExtension).DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;

    This = AllocateItem(NonPagedPool, core::mem::size_of::<IKsPinImpl>()) as *mut IKsPinImpl;
    if This.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    CreateItem = AllocateItem(NonPagedPool, core::mem::size_of::<KSOBJECT_CREATE_ITEM>() * 3) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        FreeItem(This as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*This).BasicHeader.KsDevice = KsDevice;
    (*This).BasicHeader.Type = KsObjectTypePin;
    (*This).BasicHeader.Parent.KsFilter = (*(*Filter).lpVtbl).GetStruct.expect("non-null function pointer")(Filter);
    (*This).BasicHeader.OuterUnknown = &raw mut vt_IKsPin as *mut _ as PUNKNOWN;
    InitializeListHead(&mut (*This).BasicHeader.EventList);
    KeInitializeSpinLock(&mut (*This).BasicHeader.EventListLock);

    if (*This).BasicHeader.Parent.KsFilter.is_null() {
        RtlAssert(
            b"This->BasicHeader.Parent.KsFilter\0".as_ptr() as PVOID,
            b"pin.c\0".as_ptr() as PVOID,
            2417,
            core::ptr::null_mut(),
        );
    }

    BasicHeader = ((*This).BasicHeader.Parent.KsFilter as usize).wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>()) as PKSBASIC_HEADER;
    (*This).BasicHeader.ControlMutex = (*BasicHeader).ControlMutex;

    if (*This).BasicHeader.ControlMutex.is_null() {
        RtlAssert(
            b"This->BasicHeader.ControlMutex\0".as_ptr() as PVOID,
            b"pin.c\0".as_ptr() as PVOID,
            2422,
            core::ptr::null_mut(),
        );
    }

    InitializeListHead(&mut (*This).BasicHeader.EventList);
    KeInitializeSpinLock(&mut (*This).BasicHeader.EventListLock);

    (*This).FrameSize = FrameSize;
    (*This).NumFrames = NumFrames;
    (*This).lpVtblReferenceClock = &raw mut vt_ReferenceClock;
    (*This).ref_ = 1;
    (*This).FileObject = (*IoStack).FileObject;
    (*This).Filter = Filter;
    KeInitializeMutex(&mut (*This).ProcessingMutex, 0);
    InitializeListHead(&mut (*This).IrpList);
    KeInitializeSpinLock(&raw mut (*This).IrpListLock as *mut _);

    (*This).Pin.Bag = AllocateItem(NonPagedPool, core::mem::size_of::<KSIOBJECT_BAG>()) as KSOBJECT_BAG;
    if (*This).Pin.Bag.is_null() {
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*(*Device).lpVtbl).InitializeObjectBag.expect("non-null function pointer")(
        Device,
        (*This).Pin.Bag as PKSIOBJECT_BAG,
        core::ptr::null_mut(),
    );

    (*This).Pin.Descriptor = AllocateItem(NonPagedPool, core::mem::size_of::<KSPIN_DESCRIPTOR_EX>()) as *const KSPIN_DESCRIPTOR_EX;
    if (*This).Pin.Descriptor.is_null() {
        KsFreeObjectBag((*This).Pin.Bag);
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        Descriptor as *const u8,
        (*This).Pin.Descriptor as *mut u8,
        core::mem::size_of::<KSPIN_DESCRIPTOR_EX>(),
    );

    core::ptr::write_bytes(&mut AutomationTable, 0, 1);

    AutomationTable.PropertyItemSize = core::mem::size_of::<KSPROPERTY_ITEM>() as ULONG;
    AutomationTable.PropertySets = &raw const PinPropertySet as *const _ as *mut KSPROPERTY_SET;
    AutomationTable.PropertySetsCount = (core::mem::size_of_val(&*&raw const PinPropertySet) / core::mem::size_of::<KSPROPERTY_SET>()) as ULONG;

    Status = KsMergeAutomationTables(
        &raw const (*(*This).Pin.Descriptor).AutomationTable as *mut PKSAUTOMATION_TABLE,
        (*Descriptor).AutomationTable as PKSAUTOMATION_TABLE,
        &mut AutomationTable,
        (*This).Pin.Bag,
    );

    if Status < 0 {
        KsFreeObjectBag((*This).Pin.Bag);
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return Status;
    }

    DataFormat = Connect.offset(1) as PKSDATAFORMAT;

    (*This).Pin.Context = core::ptr::null_mut();
    (*This).Pin.Id = (*Connect).PinId;
    (*This).Pin.Communication = (*Descriptor).PinDescriptor.Communication;
    (*This).Pin.ConnectionIsExternal = 0u8;

    core::ptr::copy_nonoverlapping(
        &(*Connect).Interface as *const _ as *const u8,
        &mut (*This).Pin.ConnectionInterface as *mut _ as *mut u8,
        core::mem::size_of::<KSPIN_INTERFACE>(),
    );
    core::ptr::copy_nonoverlapping(
        &(*Connect).Medium as *const _ as *const u8,
        &mut (*This).Pin.ConnectionMedium as *mut _ as *mut u8,
        core::mem::size_of::<KSPIN_MEDIUM>(),
    );
    core::ptr::copy_nonoverlapping(
        &(*Connect).Priority as *const _ as *const u8,
        &mut (*This).Pin.ConnectionPriority as *mut _ as *mut u8,
        core::mem::size_of::<KSPRIORITY>(),
    );

    Status = _KsEdit(
        (*This).Pin.Bag,
        &mut (*This).Pin.ConnectionFormat as *mut _ as *mut PVOID,
        (*DataFormat).__bindgen_anon_1.as_ref().FormatSize,
        (*DataFormat).__bindgen_anon_1.as_ref().FormatSize,
        0,
    );

    if Status < 0 {
        KsFreeObjectBag((*This).Pin.Bag);
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        DataFormat as *const u8,
        (*This).Pin.ConnectionFormat as *mut u8,
        (*DataFormat).__bindgen_anon_1.as_ref().FormatSize as usize,
    );

    (*This).Pin.AttributeList = core::ptr::null_mut();
    (*This).Pin.StreamHeaderSize = core::mem::size_of::<KSSTREAM_HEADER>() as ULONG;
    (*This).Pin.DataFlow = (*Descriptor).PinDescriptor.DataFlow;
    (*This).Pin.DeviceState = KSSTATE_STOP;
    (*This).Pin.ResetState = KSRESET_END;
    (*This).Pin.ClientState = KSSTATE_STOP;

    (*CreateItem.offset(0)).Context = &mut (*This).Pin as *mut _ as PVOID;
    (*CreateItem.offset(0)).Create = cb!(IKsPin_DispatchCreateAllocator);
    (*CreateItem.offset(0)).Flags = KSCREATE_ITEM_FREEONSTOP as ULONG;
    RtlInitUnicodeString(
        &mut (*CreateItem.offset(0)).ObjectClass,
        KSSTRING_Allocator.as_ptr() as PCWSTR,
    );

    (*CreateItem.offset(1)).Context = &mut (*This).Pin as *mut _ as PVOID;
    (*CreateItem.offset(1)).Create = cb!(IKsPin_DispatchCreateClock);
    (*CreateItem.offset(1)).Flags = KSCREATE_ITEM_FREEONSTOP as ULONG;
    RtlInitUnicodeString(
        &mut (*CreateItem.offset(1)).ObjectClass,
        KSSTRING_Clock.as_ptr() as PCWSTR,
    );

    (*CreateItem.offset(2)).Context = &mut (*This).Pin as *mut _ as PVOID;
    (*CreateItem.offset(2)).Create = cb!(IKsPin_DispatchCreateNode);
    (*CreateItem.offset(2)).Flags = KSCREATE_ITEM_FREEONSTOP as ULONG;
    RtlInitUnicodeString(
        &mut (*CreateItem.offset(2)).ObjectClass,
        KSSTRING_TopologyNode.as_ptr() as PCWSTR,
    );

    Status = KsAllocateObjectHeader(
        &mut (*This).ObjectHeader as *mut _ as *mut KSOBJECT_HEADER,
        3,
        CreateItem,
        Irp,
        &raw mut PinDispatchTable,
    );

    if Status < 0 {
        KsFreeObjectBag((*This).Pin.Bag);
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return Status;
    }

    (*(*This).ObjectHeader).Type = KsObjectTypePin;
    (*(*This).ObjectHeader).Unknown = &mut (*This).BasicHeader.OuterUnknown as *mut _ as PUNKNOWN;
    (*(*This).ObjectHeader).ObjectType = &mut (*This).Pin as *mut _ as PVOID;

    if (*Descriptor).Dispatch.is_null() || (*(*Descriptor).Dispatch).Process.is_none() {
        (*This).ProcessPin.BytesAvailable = 0;
        (*This).ProcessPin.BytesUsed = 0;
        (*This).ProcessPin.CopySource = core::ptr::null_mut();
        (*This).ProcessPin.Data = core::ptr::null_mut();
        (*This).ProcessPin.DelegateBranch = core::ptr::null_mut();
        (*This).ProcessPin.Flags = 0;
        (*This).ProcessPin.InPlaceCounterpart = core::ptr::null_mut();
        (*This).ProcessPin.Pin = &mut (*This).Pin as *mut _ as PKSPIN;
        (*This).ProcessPin.StreamPointer = &mut (*This).LeadingEdgeStreamPointer.StreamPointer as *mut _ as PKSSTREAM_POINTER;
        (*This).ProcessPin.Terminate = 0u8;

        Status = (*(*Filter).lpVtbl).AddProcessPin.expect("non-null function pointer")(
            Filter,
            &mut (*This).ProcessPin,
        );

        if Status < 0 {
            KsFreeObjectBag((*This).Pin.Bag);
            KsFreeObjectHeader(&mut (*This).ObjectHeader as *mut _ as PVOID);
            FreeItem(This as PVOID);
            FreeItem(CreateItem as PVOID);
            return Status;
        }
    } else if !(*Descriptor).Dispatch.is_null() && (*(*Descriptor).Dispatch).Process.is_some() {
        (*This).PinWorkQueueItem.WorkerRoutine = cb!(IKsPin_PinCentricWorker);
        core::ptr::write_volatile(&mut (*This).PinWorkQueueItem.Parameter as *mut PVOID, This as PVOID);
        (*This).PinWorkQueueItem.List.Flink = core::ptr::null_mut();

        Status = KsRegisterCountedWorker(
            HyperCriticalWorkQueue,
            &mut (*This).PinWorkQueueItem,
            &mut (*This).PinWorker,
        );

        if Status < 0 {
            KsFreeObjectBag((*This).Pin.Bag);
            KsFreeObjectHeader(&mut (*This).ObjectHeader as *mut _ as PVOID);
            FreeItem(This as PVOID);
            FreeItem(CreateItem as PVOID);
            return Status;
        }

        if (*(*This).Pin.Descriptor).PinDescriptor.DataFlow as u32 == KSPIN_DATAFLOW_IN as u32 {
            (*This).LeadingEdgeStreamPointer.StreamPointer.Offset = &mut (*This).LeadingEdgeStreamPointer.StreamPointer.OffsetIn as *mut _ as PKSSTREAM_POINTER_OFFSET;
        } else {
            (*This).LeadingEdgeStreamPointer.StreamPointer.Offset = &mut (*This).LeadingEdgeStreamPointer.StreamPointer.OffsetOut as *mut _ as PKSSTREAM_POINTER_OFFSET;
        }

        KeInitializeEvent(
            &mut (*This).FrameComplete,
            _EVENT_TYPE_NotificationEvent,
            0u8,
        );
    }

    IKsFilter_AddPin(
        (*(*Filter).lpVtbl).GetStruct.expect("non-null function pointer")(Filter),
        &mut (*This).Pin,
    );

    if !(*Descriptor).Dispatch.is_null() && (*(*Descriptor).Dispatch).SetDataFormat.is_some() {
        Status = (*(*Descriptor).Dispatch).SetDataFormat.expect("non-null function pointer")(
            &mut (*This).Pin,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            (*This).Pin.ConnectionFormat as *const KSDATARANGE,
            core::ptr::null(),
        );
    }

    if !(*Descriptor).Dispatch.is_null() && (*(*Descriptor).Dispatch).Create.is_some() {
        Status = (*(*Descriptor).Dispatch).Create.expect("non-null function pointer")(&mut (*This).Pin, Irp);
    }

    if Status < 0 && Status != STATUS_PENDING {
        IKsFilter_RemovePin(
            (*(*Filter).lpVtbl).GetStruct.expect("non-null function pointer")(Filter),
            &mut (*This).Pin,
        );
        KsFreeObjectBag((*This).Pin.Bag);
        KsFreeObjectHeader((*This).ObjectHeader as PVOID);
        FreeItem(This as PVOID);
        return Status;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/worker.c
//  * PURPOSE:         KS pin functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #include <bdamedia.h>
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct _KSISTREAM_POINTER
// {
//     PFNKSSTREAMPOINTER Callback;
//     PIRP Irp;
//     KTIMER Timer;
//     KDPC TimerDpc;
//     struct _KSISTREAM_POINTER *Next;
//     PKSPIN Pin;
//     PVOID Data;
//     ULONG Offset;
//     ULONG Length;
//     KSSTREAM_POINTER StreamPointer;
//     KSPIN_LOCK Lock;
// }KSISTREAM_POINTER, *PKSISTREAM_POINTER;
//
// typedef struct
// {
//     KSBASIC_HEADER BasicHeader;
//     KSPIN Pin;
//     PKSIOBJECT_HEADER ObjectHeader;
//     KSPROCESSPIN ProcessPin;
//     LIST_ENTRY Entry;
//
//     LONG ref;
//
//     IKsFilter * Filter;
//     KMUTEX ProcessingMutex;
//     PFILE_OBJECT FileObject;
//
//     PKSGATE AttachedGate;
//     BOOL OrGate;
//
//     LIST_ENTRY IrpList;
//     KSPIN_LOCK IrpListLock;
//     volatile LONG IrpCount;
//
//     PKSISTREAM_POINTER ClonedStreamPointer;
//     KSISTREAM_POINTER LeadingEdgeStreamPointer;
//     KSISTREAM_POINTER TrailingStreamPointer;
//
//     PFNKSPINPOWER  Sleep;
//     PFNKSPINPOWER  Wake;
//     PFNKSPINHANDSHAKE  Handshake;
//     PFNKSPINFRAMERETURN  FrameReturn;
//     PFNKSPINIRPCOMPLETION  IrpCompletion;
//
//     KSCLOCK_FUNCTIONTABLE ClockTable;
//     PFILE_OBJECT ClockFileObject;
//     IKsReferenceClockVtbl * lpVtblReferenceClock;
//     PKSDEFAULTCLOCK DefaultClock;
//
//     PKSWORKER PinWorker;
//     WORK_QUEUE_ITEM PinWorkQueueItem;
//     KEVENT FrameComplete;
//     ULONG FrameSize;
//     ULONG NumFrames;
//     PDMA_ADAPTER Dma;
//     ULONG MapRegisters;
//
// }IKsPinImpl;
//
// NTSTATUS NTAPI IKsPin_PinStatePropertyHandler(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI IKsPin_PinDataFormatPropertyHandler(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI IKsPin_PinAllocatorFramingPropertyHandler(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI IKsPin_PinStreamAllocator(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI IKsPin_PinMasterClock(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
// NTSTATUS NTAPI IKsPin_PinPipeId(IN PIRP Irp, IN PKSIDENTIFIER Request, IN OUT PVOID Data);
//
//
//
// DEFINE_KSPROPERTY_CONNECTIONSET(PinConnectionSet, IKsPin_PinStatePropertyHandler, IKsPin_PinDataFormatPropertyHandler, IKsPin_PinAllocatorFramingPropertyHandler);
// DEFINE_KSPROPERTY_STREAMSET(PinStreamSet, IKsPin_PinStreamAllocator, IKsPin_PinMasterClock, IKsPin_PinPipeId);
//
// //TODO
// // KSPROPSETID_Connection
// //    KSPROPERTY_CONNECTION_ACQUIREORDERING
// // KSPROPSETID_StreamInterface
// //     KSPROPERTY_STREAMINTERFACE_HEADERSIZE
//
// KSPROPERTY_SET PinPropertySet[] =
// {
//     {
//         &KSPROPSETID_Connection,
//         sizeof(PinConnectionSet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&PinConnectionSet,
//         0,
//         NULL
//     },
//     {
//         &KSPROPSETID_Stream,
//         sizeof(PinStreamSet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&PinStreamSet,
//         0,
//         NULL
//     }
// };
//
// const GUID KSPROPSETID_Connection              = {0x1D58C920L, 0xAC9B, 0x11CF, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
// const GUID KSPROPSETID_Stream                  = {0x65aaba60L, 0x98ae, 0x11cf, {0xa1, 0x0d, 0x00, 0x20, 0xaf, 0xd1, 0x56, 0xe4}};
// const GUID KSPROPSETID_Clock                   = {0xDF12A4C0L, 0xAC17, 0x11CF, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
//
// NTSTATUS
// NTAPI
// IKsPin_PinStreamAllocator(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_PinMasterClock(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PHANDLE Handle;
//     PFILE_OBJECT FileObject;
//     KPROCESSOR_MODE Mode;
//     KSPROPERTY Property;
//     ULONG BytesReturned;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("IKsPin_PinMasterClock\n");
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* sanity check */
//     ASSERT(ObjectHeader);
//
//     /* locate ks pin implementation from KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     /* sanity check */
//     ASSERT(This);
//
//     Handle = (PHANDLE)Data;
//
//     if (Request->Flags & KSPROPERTY_TYPE_GET)
//     {
//         if (This->Pin.Descriptor->PinDescriptor.Communication != KSPIN_COMMUNICATION_NONE &&
//             This->Pin.Descriptor->Dispatch &&
//             (This->Pin.Descriptor->Flags & KSPIN_FLAG_IMPLEMENT_CLOCK))
//         {
//             *Handle = NULL;
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* no clock available */
//             Status = STATUS_UNSUCCESSFUL;
//         }
//     }
//     else if (Request->Flags & KSPROPERTY_TYPE_SET)
//     {
//         if (This->Pin.ClientState != KSSTATE_STOP)
//         {
//             /* can only set in stopped state */
//             Status = STATUS_INVALID_DEVICE_STATE;
//         }
//         else
//         {
//             if (*Handle)
//             {
//                 Mode = ExGetPreviousMode();
//
//                 Status = ObReferenceObjectByHandle(*Handle, SYNCHRONIZE | DIRECTORY_QUERY, *IoFileObjectType, Mode, (PVOID*)&FileObject, NULL);
//
//                 DPRINT("IKsPin_PinMasterClock ObReferenceObjectByHandle %lx\n", Status);
//                 if (NT_SUCCESS(Status))
//                 {
//                     Property.Set = KSPROPSETID_Clock;
//                     Property.Id = KSPROPERTY_CLOCK_FUNCTIONTABLE;
//                     Property.Flags = KSPROPERTY_TYPE_GET;
//
//                     Status = KsSynchronousIoControlDevice(FileObject, KernelMode, IOCTL_KS_PROPERTY, &Property, sizeof(KSPROPERTY), &This->ClockTable, sizeof(KSCLOCK_FUNCTIONTABLE), &BytesReturned);
//
//                     DPRINT("IKsPin_PinMasterClock KSPROPERTY_CLOCK_FUNCTIONTABLE %lx\n", Status);
//
//                     if (NT_SUCCESS(Status))
//                     {
//                         This->ClockFileObject = FileObject;
//                     }
//                     else
//                     {
//                         ObDereferenceObject(FileObject);
//                     }
//                 }
//             }
//             else
//             {
//                 /* zeroing clock handle */
//                 RtlZeroMemory(&This->ClockTable, sizeof(KSCLOCK_FUNCTIONTABLE));
//                 Status = STATUS_SUCCESS;
//                 if (This->ClockFileObject)
//                 {
//                     FileObject = This->ClockFileObject;
//                     This->ClockFileObject = NULL;
//
//                     ObDereferenceObject(This->ClockFileObject);
//                 }
//             }
//         }
//     }
//
//     DPRINT("IKsPin_PinMasterClock Status %lx\n", Status);
//     return Status;
// }
//
//
//
// NTSTATUS
// NTAPI
// IKsPin_PinPipeId(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// IKsPin_PinStatePropertyHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     NTSTATUS Status = STATUS_SUCCESS;
//     KSSTATE OldState;
//     PKSSTATE NewState;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("IKsPin_PinStatePropertyHandler\n");
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* locate ks pin implementation from KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     /* acquire control mutex */
//     KeWaitForSingleObject(This->BasicHeader.ControlMutex, Executive, KernelMode, FALSE, NULL);
//
//     /* grab state */
//     NewState = (PKSSTATE)Data;
//
//     if (Request->Flags & KSPROPERTY_TYPE_GET)
//     {
//         *NewState = This->Pin.DeviceState;
//         Irp->IoStatus.Information = sizeof(KSSTATE);
//     }
//     else if (Request->Flags & KSPROPERTY_TYPE_SET)
//     {
//         if (This->Pin.Descriptor->Dispatch->SetDeviceState)
//         {
//             /* backup old state */
//             OldState = This->Pin.ClientState;
//
//             /* set new state */
//             This->Pin.ClientState  = *NewState;
//             This->Pin.DeviceState = KSSTATE_RUN;
//
//             /* check if it supported */
//             Status = This->Pin.Descriptor->Dispatch->SetDeviceState(&This->Pin, *NewState, OldState);
//
//             DPRINT("IKsPin_PinStatePropertyHandler NewState %lu Result %lx\n", *NewState, Status);
//
//             if (!NT_SUCCESS(Status))
//             {
//                 /* revert to old state */
//                 This->Pin.ClientState = OldState;
//                 This->Pin.DeviceState = OldState;
//                 DPRINT("IKsPin_PinStatePropertyHandler failed to set state %lx Result %lx\n", *NewState, Status);
//                 DbgBreakPoint();
//             }
//             else
//             {
//                 /* update device state */
//                 This->Pin.DeviceState = *NewState;
//             }
//         }
//         else
//         {
//             /* just set new state */
//             This->Pin.DeviceState = *NewState;
//             This->Pin.ClientState = *NewState;
//         }
//     }
//
//     /* release processing mutex */
//     KeReleaseMutex(This->BasicHeader.ControlMutex, FALSE);
//
//     DPRINT("IKsPin_PinStatePropertyHandler Status %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_PinAllocatorFramingPropertyHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     ULONG Size;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* locate ks pin implementation from KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     /* setting allocator flags is not supported */
//     ASSERT(!(Request->Flags & KSPROPERTY_TYPE_SET));
//
//     /* acquire control mutex */
//     KeWaitForSingleObject(This->BasicHeader.ControlMutex, Executive, KernelMode, FALSE, NULL);
//
//     if (This->Pin.Descriptor->AllocatorFraming)
//     {
//         /* calculate size */
//         Size = FIELD_OFFSET(KSALLOCATOR_FRAMING_EX, FramingItem[0]) + This->Pin.Descriptor->AllocatorFraming->CountItems * sizeof(KS_FRAMING_ITEM);
//
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength == 0)
//         {
//             /* no buffer */
//             Status = STATUS_BUFFER_OVERFLOW;
//         }
//         else if (Size > IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//         {
//             /* buffer too small */
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//         else
//         {
//             /* copy buffer */
//             RtlMoveMemory(Data, This->Pin.Descriptor->AllocatorFraming, Size);
//         }
//
//         /* store size */
//         Irp->IoStatus.Information = Size;
//     }
//     else
//     {
//         /* no allocator framing details */
//         Status = STATUS_NOT_FOUND;
//     }
//
//     /* release processing mutex */
//     KeReleaseMutex(This->BasicHeader.ControlMutex, FALSE);
//
//     DPRINT("IKsPin_PinAllocatorFramingPropertyHandler Status %lx\n", Status);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_PinDataFormatPropertyHandler(
//     IN PIRP Irp,
//     IN PKSPROPERTY Request,
//     IN OUT PVOID Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("IKsPin_PinDataFormatPropertyHandler\n");
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* locate ks pin implementation from KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     /* acquire control mutex */
//     KeWaitForSingleObject(This->BasicHeader.ControlMutex, Executive, KernelMode, FALSE, NULL);
//
//     if (Request->Flags & KSPROPERTY_TYPE_GET)
//     {
//         if (IoStack->Parameters.DeviceIoControl.OutputBufferLength < This->Pin.ConnectionFormat->FormatSize)
//         {
//             /* buffer too small */
//             Irp->IoStatus.Information = This->Pin.ConnectionFormat->FormatSize;
//             Status = STATUS_BUFFER_TOO_SMALL;
//         }
//         else
//         {
//             /* copy format */
//             RtlMoveMemory(Data, This->Pin.ConnectionFormat, This->Pin.ConnectionFormat->FormatSize);
//         }
//     }
//     else if (Request->Flags & KSPROPERTY_TYPE_SET)
//     {
//         /* set format */
//         if (This->Pin.Descriptor->Flags & KSPIN_FLAG_FIXED_FORMAT)
//         {
//             /* format cannot be changed */
//             Status = STATUS_INVALID_DEVICE_REQUEST;
//         }
//         else
//         {
//             /* FIXME check if the format is supported */
//             Status = _KsEdit(This->Pin.Bag, (PVOID*)&This->Pin.ConnectionFormat, IoStack->Parameters.DeviceIoControl.OutputBufferLength, This->Pin.ConnectionFormat->FormatSize, 0);
//
//             if (NT_SUCCESS(Status))
//             {
//                 /* store new format */
//                 RtlMoveMemory(This->Pin.ConnectionFormat, Data, IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//             }
//         }
//     }
//
//     /* release processing mutex */
//     KeReleaseMutex(This->BasicHeader.ControlMutex, FALSE);
//
//     DPRINT("IKsPin_PinDataFormatPropertyHandler Status %lx\n", Status);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnQueryInterface(
//     IKsPin * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     NTSTATUS Status;
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, BasicHeader.OuterUnknown);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->BasicHeader.OuterUnknown;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//
//
//     if (This->BasicHeader.ClientAggregate)
//     {
//          /* using client aggregate */
//          Status = This->BasicHeader.ClientAggregate->lpVtbl->QueryInterface(This->BasicHeader.ClientAggregate, refiid, Output);
//
//          if (NT_SUCCESS(Status))
//          {
//              /* client aggregate supports interface */
//              return Status;
//          }
//     }
//
//     DPRINT("IKsPin_fnQueryInterface no interface\n");
//     return STATUS_NOT_SUPPORTED;
// }
//
// ULONG
// NTAPI
// IKsPin_fnAddRef(
//     IKsPin * iface)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, BasicHeader.OuterUnknown);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsPin_fnRelease(
//     IKsPin * iface)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, BasicHeader.OuterUnknown);
//
//     InterlockedDecrement(&This->ref);
//
//     if (This->ref == 0)
//     {
//         FreeItem(This);
//         return 0;
//     }
//     /* Return new reference count */
//     return This->ref;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnTransferKsIrp(
//     IN IKsPin *iface,
//     IN PIRP Irp,
//     IN IKsTransport **OutTransport)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// VOID
// NTAPI
// IKsPin_fnDiscardKsIrp(
//     IN IKsPin *iface,
//     IN PIRP Irp,
//     IN IKsTransport * *OutTransport)
// {
//     UNIMPLEMENTED;
// }
//
//
// NTSTATUS
// NTAPI
// IKsPin_fnConnect(
//     IN IKsPin *iface,
//     IN IKsTransport * TransportIn,
//     OUT IKsTransport ** OutTransportIn,
//     OUT IKsTransport * *OutTransportOut,
//     IN KSPIN_DATAFLOW DataFlow)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnSetDeviceState(
//     IN IKsPin *iface,
//     IN KSSTATE OldState,
//     IN KSSTATE NewState,
//     IN IKsTransport * *OutTransport)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// VOID
// NTAPI
// IKsPin_fnSetResetState(
//     IN IKsPin *iface,
//     IN KSRESET ResetState,
//     OUT IKsTransport * * OutTransportOut)
// {
//     UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnGetTransportConfig(
//     IN IKsPin *iface,
//     IN struct KSPTRANSPORTCONFIG * TransportConfig,
//     OUT IKsTransport ** OutTransportIn,
//     OUT IKsTransport ** OutTransportOut)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnSetTransportConfig(
//     IN IKsPin *iface,
//     IN struct KSPTRANSPORTCONFIG const * TransportConfig,
//     OUT IKsTransport ** OutTransportIn,
//     OUT IKsTransport ** OutTransportOut)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnResetTransportConfig(
//     IN IKsPin *iface,
//     OUT IKsTransport ** OutTransportIn,
//     OUT IKsTransport ** OutTransportOut)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// PKSPIN
// NTAPI
// IKsPin_fnGetStruct(
//     IN IKsPin *iface)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// PKSPROCESSPIN
// NTAPI
// IKsPin_fnGetProcessPin(
//     IN IKsPin *iface)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnAttemptBypass(
//     IN IKsPin *iface)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnAttemptUnbypass(
//     IN IKsPin *iface)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// VOID
// NTAPI
// IKsPin_fnGenerateConnectionEvents(
//     IN IKsPin *iface,
//     IN ULONG EventMask)
// {
//     UNIMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_fnClientSetDeviceState(
//     IN IKsPin *iface,
//     IN KSSTATE StateIn,
//     IN KSSTATE StateOut)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// static IKsPinVtbl vt_IKsPin =
// {
//     IKsPin_fnQueryInterface,
//     IKsPin_fnAddRef,
//     IKsPin_fnRelease,
//     IKsPin_fnTransferKsIrp,
//     IKsPin_fnDiscardKsIrp,
//     IKsPin_fnConnect,
//     IKsPin_fnSetDeviceState,
//     IKsPin_fnSetResetState,
//     IKsPin_fnGetTransportConfig,
//     IKsPin_fnSetTransportConfig,
//     IKsPin_fnResetTransportConfig,
//     IKsPin_fnGetStruct,
//     IKsPin_fnGetProcessPin,
//     IKsPin_fnAttemptBypass,
//     IKsPin_fnAttemptUnbypass,
//     IKsPin_fnGenerateConnectionEvents,
//     IKsPin_fnClientSetDeviceState
// };
//
//
// //==============================================================
//
// NTSTATUS
// NTAPI
// IKsReferenceClock_fnQueryInterface(
//     IKsReferenceClock * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     return IKsPin_fnQueryInterface((IKsPin*)&This->BasicHeader.OuterUnknown, refiid, Output);
// }
//
// ULONG
// NTAPI
// IKsReferenceClock_fnAddRef(
//     IKsReferenceClock * iface)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     return IKsPin_fnAddRef((IKsPin*)&This->BasicHeader.OuterUnknown);
// }
//
// ULONG
// NTAPI
// IKsReferenceClock_fnRelease(
//     IKsReferenceClock * iface)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     return IKsPin_fnRelease((IKsPin*)&This->BasicHeader.OuterUnknown);
// }
//
// LONGLONG
// NTAPI
// IKsReferenceClock_fnGetTime(
//     IKsReferenceClock * iface)
// {
//     LONGLONG Result;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//
//     DPRINT1("IKsReferenceClock_fnGetTime\n");
//
//     if (!This->ClockFileObject || !This->ClockTable.GetTime)
//     {
//         Result = 0;
//     }
//     else
//     {
//         Result = This->ClockTable.GetTime(This->ClockFileObject);
//     }
//
//     return Result;
// }
//
// LONGLONG
// NTAPI
// IKsReferenceClock_fnGetPhysicalTime(
//     IKsReferenceClock * iface)
// {
//     LONGLONG Result;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     DPRINT1("IKsReferenceClock_fnGetPhysicalTime\n");
//
//
//     if (!This->ClockFileObject || !This->ClockTable.GetPhysicalTime)
//     {
//         Result = 0;
//     }
//     else
//     {
//         Result = This->ClockTable.GetPhysicalTime(This->ClockFileObject);
//     }
//
//     return Result;
// }
//
//
// LONGLONG
// NTAPI
// IKsReferenceClock_fnGetCorrelatedTime(
//     IKsReferenceClock * iface,
//     OUT PLONGLONG SystemTime)
// {
//     LONGLONG Result;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     DPRINT1("IKsReferenceClock_fnGetCorrelatedTime\n");
//
//     if (!This->ClockFileObject || !This->ClockTable.GetCorrelatedTime)
//     {
//         Result = 0;
//     }
//     else
//     {
//         Result = This->ClockTable.GetCorrelatedTime(This->ClockFileObject, SystemTime);
//     }
//
//     return Result;
// }
//
//
// LONGLONG
// NTAPI
// IKsReferenceClock_fnGetCorrelatedPhysicalTime(
//     IKsReferenceClock * iface,
//     OUT PLONGLONG SystemTime)
// {
//     LONGLONG Result;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     DPRINT1("IKsReferenceClock_fnGetCorrelatedPhysicalTime\n");
//
//     if (!This->ClockFileObject || !This->ClockTable.GetCorrelatedPhysicalTime)
//     {
//         Result = 0;
//     }
//     else
//     {
//         Result = This->ClockTable.GetCorrelatedPhysicalTime(This->ClockFileObject, SystemTime);
//     }
//
//     return Result;
// }
//
// NTSTATUS
// NTAPI
// IKsReferenceClock_fnGetResolution(
//     IKsReferenceClock * iface,
//     OUT PKSRESOLUTION Resolution)
// {
//     KSPROPERTY Property;
//     ULONG BytesReturned;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     DPRINT1("IKsReferenceClock_fnGetResolution\n");
//
//     if (!This->ClockFileObject)
//     {
//         Resolution->Error = 0;
//         Resolution->Granularity = 1;
//         DPRINT1("IKsReferenceClock_fnGetResolution Using HACK\n");
//         return STATUS_SUCCESS;
//     }
//
//
//     if (!This->ClockFileObject)
//         return STATUS_DEVICE_NOT_READY;
//
//
//     Property.Set = KSPROPSETID_Clock;
//     Property.Id = KSPROPERTY_CLOCK_RESOLUTION;
//     Property.Flags = KSPROPERTY_TYPE_GET;
//
//     return KsSynchronousIoControlDevice(This->ClockFileObject, KernelMode, IOCTL_KS_PROPERTY, &Property, sizeof(KSPROPERTY), Resolution, sizeof(KSRESOLUTION), &BytesReturned);
//
// }
//
// NTSTATUS
// NTAPI
// IKsReferenceClock_fnGetState(
//     IKsReferenceClock * iface,
//      OUT PKSSTATE State)
// {
//     KSPROPERTY Property;
//     ULONG BytesReturned;
//
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(iface, IKsPinImpl, lpVtblReferenceClock);
//
//     DPRINT1("IKsReferenceClock_fnGetState\n");
//
//     if (!This->ClockFileObject)
//     {
//         *State = This->Pin.ClientState;
//         DPRINT1("IKsReferenceClock_fnGetState Using HACK\n");
//         return STATUS_SUCCESS;
//     }
//
//
//     if (!This->ClockFileObject)
//         return STATUS_DEVICE_NOT_READY;
//
//
//     Property.Set = KSPROPSETID_Clock;
//     Property.Id = KSPROPERTY_CLOCK_RESOLUTION;
//     Property.Flags = KSPROPERTY_TYPE_GET;
//
//     return KsSynchronousIoControlDevice(This->ClockFileObject, KernelMode, IOCTL_KS_PROPERTY, &Property, sizeof(KSPROPERTY), State, sizeof(KSSTATE), &BytesReturned);
// }
//
// static IKsReferenceClockVtbl vt_ReferenceClock =
// {
//     IKsReferenceClock_fnQueryInterface,
//     IKsReferenceClock_fnAddRef,
//     IKsReferenceClock_fnRelease,
//     IKsReferenceClock_fnGetTime,
//     IKsReferenceClock_fnGetPhysicalTime,
//     IKsReferenceClock_fnGetCorrelatedTime,
//     IKsReferenceClock_fnGetCorrelatedPhysicalTime,
//     IKsReferenceClock_fnGetResolution,
//     IKsReferenceClock_fnGetState
// };
//
//
// //==============================================================
//
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinAcquireProcessingMutex(
//     IN PKSPIN  Pin)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinAttachAndGate(
//     IN PKSPIN Pin,
//     IN PKSGATE AndGate OPTIONAL)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* FIXME attach to filter's and gate (filter-centric processing) */
//
//     This->AttachedGate = AndGate;
//     This->OrGate = FALSE;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinAttachOrGate(
//     IN PKSPIN Pin,
//     IN PKSGATE OrGate OPTIONAL)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* FIXME attach to filter's and gate (filter-centric processing) */
//
//     This->AttachedGate = OrGate;
//     This->OrGate = TRUE;
// }
//
// /*
//     @implemented
// */
// PKSGATE
// NTAPI
// KsPinGetAndGate(
//     IN PKSPIN  Pin)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     return This->AttachedGate;
// }
//
// /*
//     @unimplemented
// */
// VOID
// NTAPI
// KsPinAttemptProcessing(
//     IN PKSPIN  Pin,
//     IN BOOLEAN  Asynchronous)
// {
//     DPRINT("KsPinAttemptProcessing\n");
//     DbgBreakPoint();
//     UNIMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// NTSTATUS
// NTAPI
// KsPinGetAvailableByteCount(
//     IN PKSPIN  Pin,
//     OUT PLONG  InputDataBytes OPTIONAL,
//     OUT PLONG  OutputBufferBytes OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// NTSTATUS
// NTAPI
// KsPinGetConnectedFilterInterface(
//     IN PKSPIN  Pin,
//     IN const GUID*  InterfaceId,
//     OUT PVOID*  Interface)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// PDEVICE_OBJECT
// NTAPI
// KsPinGetConnectedPinDeviceObject(
//     IN PKSPIN Pin)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//     @unimplemented
// */
// PFILE_OBJECT
// NTAPI
// KsPinGetConnectedPinFileObject(
//     IN PKSPIN Pin)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//     @unimplemented
// */
// NTSTATUS
// NTAPI
// KsPinGetConnectedPinInterface(
//     IN PKSPIN  Pin,
//     IN const GUID*  InterfaceId,
//     OUT PVOID*  Interface)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// VOID
// NTAPI
// KsPinGetCopyRelationships(
//     IN PKSPIN Pin,
//     OUT PKSPIN* CopySource,
//     OUT PKSPIN* DelegateBranch)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//     @implemented
// */
// PKSPIN
// NTAPI
// KsPinGetNextSiblingPin(
//     IN PKSPIN  Pin)
// {
//     return KsGetNextSibling((PVOID)Pin);
// }
//
// /*
//     @implemented
// */
// PKSFILTER
// NTAPI
// KsPinGetParentFilter(
//     IN PKSPIN  Pin)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* return parent filter */
//     return This->BasicHeader.Parent.KsFilter;
// }
//
// /*
//     @implemented
// */
// NTSTATUS
// NTAPI
// KsPinGetReferenceClockInterface(
//     IN PKSPIN  Pin,
//     OUT PIKSREFERENCECLOCK*  Interface)
// {
//     NTSTATUS Status = STATUS_DEVICE_NOT_READY;
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     if (This->ClockFileObject)
//     {
//         /* clock is available */
//         *Interface = (PIKSREFERENCECLOCK)&This->lpVtblReferenceClock;
//         Status = STATUS_SUCCESS;
//     }
//
//     DPRINT("KsPinGetReferenceClockInterface Pin %p Interface %p Status %x\n", Pin, Interface, Status);
//     return Status;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinRegisterFrameReturnCallback(
//     IN PKSPIN  Pin,
//     IN PFNKSPINFRAMERETURN  FrameReturn)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* register frame return callback */
//     This->FrameReturn = FrameReturn;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinRegisterHandshakeCallback(
//     IN PKSPIN  Pin,
//     IN PFNKSPINHANDSHAKE  Handshake)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* register private protocol handshake callback */
//     This->Handshake = Handshake;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinRegisterIrpCompletionCallback(
//     IN PKSPIN  Pin,
//     IN PFNKSPINIRPCOMPLETION  IrpCompletion)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* register irp completion callback */
//     This->IrpCompletion = IrpCompletion;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinRegisterPowerCallbacks(
//     IN PKSPIN  Pin,
//     IN PFNKSPINPOWER  Sleep OPTIONAL,
//     IN PFNKSPINPOWER  Wake OPTIONAL)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* register power callbacks */
//     This->Sleep = Sleep;
//     This->Wake = Wake;
// }
//
// /*
//     @implemented
// */
// VOID
// NTAPI
// KsPinReleaseProcessingMutex(
//     IN PKSPIN  Pin)
// {
//     IKsPinImpl * This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* release processing mutex */
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSPIN
// NTAPI
// KsGetPinFromIrp(
//     IN PIRP Irp)
// {
//     PKSIOBJECT_HEADER ObjectHeader;
//     PKSPIN Pin;
//     PKSBASIC_HEADER Header;
//     PIO_STACK_LOCATION IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("KsGetPinFromIrp\n");
//
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     if (!ObjectHeader)
//         return NULL;
//
//     Pin = (PKSPIN)ObjectHeader->ObjectType;
//     Header = (PKSBASIC_HEADER)((ULONG_PTR)Pin - sizeof(KSBASIC_HEADER));
//
//     /* sanity check */
//     ASSERT(Header->Type == KsObjectTypePin);
//
//     /* return object type */
//     return Pin;
// }
//
//
//
// /*
//     @unimplemented
// */
// VOID
// NTAPI
// KsPinSetPinClockTime(
//     IN PKSPIN  Pin,
//     IN LONGLONG  Time)
// {
//     UNIMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// NTSTATUS
// NTAPI
// KsPinSubmitFrame(
//     IN PKSPIN  Pin,
//     IN PVOID  Data  OPTIONAL,
//     IN ULONG  Size  OPTIONAL,
//     IN PKSSTREAM_HEADER  StreamHeader  OPTIONAL,
//     IN PVOID  Context  OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsPinSubmitFrameMdl(
//     IN PKSPIN  Pin,
//     IN PMDL  Mdl  OPTIONAL,
//     IN PKSSTREAM_HEADER  StreamHeader  OPTIONAL,
//     IN PVOID  Context  OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// BOOLEAN
// NTAPI
// KsProcessPinUpdate(
//     IN PKSPROCESSPIN  ProcessPin)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// NTSTATUS
// IKsPin_PrepareStreamHeader(
//     IN IKsPinImpl * This,
//     IN PKSISTREAM_POINTER StreamPointer)
// {
//     PKSSTREAM_HEADER Header;
//     ULONG Length;
//
//     /* grab new irp */
//     StreamPointer->Irp = KsRemoveIrpFromCancelableQueue(&This->IrpList, &This->IrpListLock, KsListEntryHead, KsAcquireAndRemoveOnlySingleItem);
//     if (!StreamPointer->Irp)
//     {
//         /* run out of mappings */
//         DPRINT("OutOfMappings\n");
//         return STATUS_DEVICE_NOT_READY;
//     }
//
//     InterlockedDecrement(&This->IrpCount);
//     KsDecrementCountedWorker(This->PinWorker);
//
//     /* get stream header */
//     if (StreamPointer->Irp->RequestorMode == UserMode)
//         Header = (PKSSTREAM_HEADER)StreamPointer->Irp->AssociatedIrp.SystemBuffer;
//     else
//         Header = (PKSSTREAM_HEADER)StreamPointer->Irp->UserBuffer;
//
//     /* initialize stream pointer */
//     StreamPointer->Callback = NULL;
//     StreamPointer->Length = max(Header->DataUsed, Header->FrameExtent);
//     StreamPointer->Next = NULL;
//     StreamPointer->Offset = 0;
//     StreamPointer->Pin = &This->Pin;
//     StreamPointer->Data = Header->Data;
//
//     StreamPointer->StreamPointer.Context = NULL;
//     StreamPointer->StreamPointer.Pin = &This->Pin;
//     StreamPointer->StreamPointer.StreamHeader = Header;
//
//     if (This->Pin.Descriptor->PinDescriptor.DataFlow == KSPIN_DATAFLOW_IN)
//         StreamPointer->StreamPointer.Offset = &StreamPointer->StreamPointer.OffsetIn;
//     else
//     StreamPointer->StreamPointer.Offset = &StreamPointer->StreamPointer.OffsetOut;
//
// #ifndef _WIN64
//     StreamPointer->StreamPointer.Offset->Alignment = 0;
// #endif
//     StreamPointer->StreamPointer.Offset->Count = 0;
//     StreamPointer->StreamPointer.Offset->Data = NULL;
//     StreamPointer->StreamPointer.Offset->Remaining = 0;
//
//     ASSERT(StreamPointer->StreamPointer.Offset->Remaining == 0);
//
//     //StreamPointer->Offset += StreamPointer->StreamPointer.Offset->Count;
//
//     ASSERT(StreamPointer->Length > StreamPointer->Offset);
//     ASSERT(StreamPointer->StreamPointer.StreamHeader);
//     ASSERT(This->FrameSize);
//
//     /* calculate length */
//     /* TODO split into frames */
//     Length = StreamPointer->Length;
//
//     /* FIXME */
//     ASSERT(Length);
//
// #ifndef _WIN64
//     StreamPointer->StreamPointer.Offset->Alignment = 0;
// #endif
//     StreamPointer->StreamPointer.Context = NULL;
//     StreamPointer->StreamPointer.Pin = &This->Pin;
//     StreamPointer->StreamPointer.Offset->Count = Length;
//     StreamPointer->StreamPointer.Offset->Remaining = Length;
//     StreamPointer->StreamPointer.Offset->Data = (PVOID)((ULONG_PTR)StreamPointer->Data + StreamPointer->Offset);
//     StreamPointer->StreamPointer.StreamHeader->FrameExtent = Length;
//     if (StreamPointer->StreamPointer.StreamHeader->DataUsed)
//         StreamPointer->StreamPointer.StreamHeader->DataUsed = Length;
//
//     StreamPointer->StreamPointer.StreamHeader->Data = StreamPointer->StreamPointer.Offset->Data;
//
//     return STATUS_SUCCESS;
// }
//
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// PKSSTREAM_POINTER
// NTAPI
// KsPinGetLeadingEdgeStreamPointer(
//     IN PKSPIN Pin,
//     IN KSSTREAM_POINTER_STATE State)
// {
//     IKsPinImpl * This;
//     NTSTATUS Status;
//
//     This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     DPRINT("KsPinGetLeadingEdgeStreamPointer Pin %p State %x Count %lu Remaining %lu\n", Pin, State,
//            This->LeadingEdgeStreamPointer.Length,
//            This->LeadingEdgeStreamPointer.Offset);
//
//     /* sanity check */
//     ASSERT(State == KSSTREAM_POINTER_STATE_LOCKED);
//
//     if (State == KSSTREAM_POINTER_STATE_LOCKED)
//     {
//         if (!This->LeadingEdgeStreamPointer.Irp || This->LeadingEdgeStreamPointer.StreamPointer.Offset->Remaining == 0)
//         {
//             Status = IKsPin_PrepareStreamHeader(This, &This->LeadingEdgeStreamPointer);
//             if (!NT_SUCCESS(Status))
//                 return NULL;
//         }
//
//         DPRINT("KsPinGetLeadingEdgeStreamPointer NewOffset %lu TotalLength %lu\n", This->LeadingEdgeStreamPointer.Offset, This->LeadingEdgeStreamPointer.Length);
//     }
//
//      return &This->LeadingEdgeStreamPointer.StreamPointer;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// PKSSTREAM_POINTER
// NTAPI
// KsPinGetTrailingEdgeStreamPointer(
//     IN PKSPIN Pin,
//     IN KSSTREAM_POINTER_STATE State)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamPointerSetStatusCode(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN NTSTATUS Status)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamPointerLock(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     UNIMPLEMENTED;
//     return STATUS_UNSUCCESSFUL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsStreamPointerUnlock(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN BOOLEAN Eject)
// {
//     PKSISTREAM_POINTER Pointer = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     DPRINT1("KsStreamPointerUnlock StreamPointer %pEject %lu\n", StreamPointer, Eject);
//
//     Pointer->Irp = NULL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsStreamPointerAdvanceOffsetsAndUnlock(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN ULONG InUsed,
//     IN ULONG OutUsed,
//     IN BOOLEAN Eject)
// {
//     DPRINT1("KsStreamPointerAdvanceOffsets InUsed %lu OutUsed %lu Eject %lu\n", InUsed, OutUsed, Eject);
//     DbgBreakPoint();
//     UNIMPLEMENTED;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsStreamPointerDelete(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     IKsPinImpl * This;
//     PKSISTREAM_POINTER Cur, Last;
//     PKSISTREAM_POINTER Pointer = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     DPRINT("KsStreamPointerDelete %p\n", Pointer);
// DbgBreakPoint();
//     This = (IKsPinImpl*)CONTAINING_RECORD!(Pointer->StreamPointer.Pin, IKsPinImpl, Pin);
//
//     /* point to first stream pointer */
//     Last = NULL;
//     Cur = This->ClonedStreamPointer;
//
//     while(Cur != Pointer && Cur)
//     {
//         Last = Cur;
//         /* iterate to next cloned pointer */
//         Cur = Cur->Next;
//     }
//
//     if (!Cur)
//     {
//         /* you naughty driver */
//         return;
//     }
//
//     if (!Last)
//     {
//         /* remove first cloned pointer */
//         This->ClonedStreamPointer = Pointer->Next;
//     }
//     else
//     {
//         Last->Next = Pointer->Next;
//     }
//
//     /* FIXME make sure no timeouts are pending */
//     FreeItem(Pointer);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamPointerClone(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN PFNKSSTREAMPOINTER CancelCallback OPTIONAL,
//     IN ULONG ContextSize,
//     OUT PKSSTREAM_POINTER* CloneStreamPointer)
// {
//     IKsPinImpl * This;
//     PKSISTREAM_POINTER CurFrame;
//     PKSISTREAM_POINTER NewFrame;
//     ULONG_PTR RefCount;
//     NTSTATUS Status;
//     ULONG Size;
//
//     DPRINT("KsStreamPointerClone StreamPointer %p CancelCallback %p ContextSize %p CloneStreamPointer %p\n", StreamPointer, CancelCallback, ContextSize, CloneStreamPointer);
//
//     /* get stream pointer */
//     CurFrame = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     /* calculate context size */
//     Size = sizeof(KSISTREAM_POINTER) + ContextSize;
//
//     /* allocate new stream pointer */
//     NewFrame = (PKSISTREAM_POINTER)AllocateItem(NonPagedPool, Size);
//
//     if (!NewFrame)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* get current irp stack location */
//     RefCount = (ULONG_PTR)CurFrame->Irp->Tail.Overlay.DriverContext[0];
//
//     /* increment reference count */
//     RefCount++;
//     CurFrame->Irp->Tail.Overlay.DriverContext[0] = (PVOID)RefCount;
//
//     /* copy stream pointer */
//     RtlMoveMemory(NewFrame, CurFrame, sizeof(KSISTREAM_POINTER));
//
//     /* locate pin */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(CurFrame->Pin, IKsPinImpl, Pin);
//
//     /* prepare stream header in case required */
//     if (CurFrame->StreamPointer.Offset->Remaining == 0)
//     {
//         Status = IKsPin_PrepareStreamHeader(This, NewFrame);
//         if (!NT_SUCCESS(Status))
//         {
//             FreeItem(NewFrame);
//             return STATUS_DEVICE_NOT_READY;
//         }
//     }
//
//     if (ContextSize)
//         NewFrame->StreamPointer.Context = (NewFrame + 1);
//
//
//     if (This->Pin.Descriptor->PinDescriptor.DataFlow == KSPIN_DATAFLOW_IN)
//         NewFrame->StreamPointer.Offset = &NewFrame->StreamPointer.OffsetIn;
//     else
//         NewFrame->StreamPointer.Offset = &NewFrame->StreamPointer.OffsetOut;
//
//
//
//     NewFrame->StreamPointer.Pin = &This->Pin;
//
//     ASSERT(NewFrame->StreamPointer.Pin);
//     ASSERT(NewFrame->StreamPointer.Context);
//     ASSERT(NewFrame->StreamPointer.Offset);
//     ASSERT(NewFrame->StreamPointer.StreamHeader);
//
//     /* store result */
//     *CloneStreamPointer = &NewFrame->StreamPointer;
//
//     DPRINT("KsStreamPointerClone CloneStreamPointer %p\n", *CloneStreamPointer);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamPointerAdvanceOffsets(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN ULONG InUsed,
//     IN ULONG OutUsed,
//     IN BOOLEAN Eject)
// {
//     PKSISTREAM_POINTER CurFrame;
//     IKsPinImpl * This;
//     NTSTATUS Status;
//
//     DPRINT("KsStreamPointerAdvanceOffsets StreamPointer %p InUsed %lu OutUsed %lu Eject %lu\n", StreamPointer, InUsed, OutUsed, Eject);
//
//     /* get stream pointer */
//     CurFrame = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     /* locate pin */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(CurFrame->Pin, IKsPinImpl, Pin);
//
//     /* TODO */
//     ASSERT(InUsed == 0);
//     ASSERT(Eject == 0);
//     ASSERT(OutUsed);
//
//     DPRINT("KsStreamPointerAdvanceOffsets Offset %lu Length %lu NewOffset %lu Remaining %lu LeadingEdge %p DataUsed %lu\n", CurFrame->Offset, CurFrame->Length, CurFrame->Offset + OutUsed,
// CurFrame->StreamPointer.OffsetOut.Remaining, &This->LeadingEdgeStreamPointer.StreamPointer, CurFrame->StreamPointer.StreamHeader->DataUsed);
// DbgBreakPoint();
//
//     if (This->Pin.Descriptor->PinDescriptor.DataFlow == KSPIN_DATAFLOW_IN)
//     {
//         ASSERT(CurFrame->StreamPointer.OffsetIn.Remaining >= InUsed);
//         CurFrame->StreamPointer.OffsetIn.Remaining -= InUsed;
//         CurFrame->StreamPointer.OffsetIn.Data = (PVOID)((ULONG_PTR)CurFrame->StreamPointer.OffsetIn.Data + InUsed);
//     }
//     else
//     {
//         if (!CurFrame->StreamPointer.OffsetOut.Remaining)
//         {
//             Status = IKsPin_PrepareStreamHeader(This, CurFrame);
//             if (!NT_SUCCESS(Status))
//             {
//                 return STATUS_DEVICE_NOT_READY;
//             }
//         }
//         else
//         {
//             ASSERT(CurFrame->StreamPointer.OffsetOut.Remaining >= OutUsed);
//             CurFrame->StreamPointer.OffsetOut.Remaining -= OutUsed;
//             CurFrame->StreamPointer.OffsetOut.Data = (PVOID)((ULONG_PTR)CurFrame->StreamPointer.OffsetOut.Data + OutUsed);
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsStreamPointerAdvance(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     UNIMPLEMENTED;
//     DbgBreakPoint();
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// PMDL
// NTAPI
// KsStreamPointerGetMdl(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// PIRP
// NTAPI
// KsStreamPointerGetIrp(
//     IN PKSSTREAM_POINTER StreamPointer,
//     OUT PBOOLEAN FirstFrameInIrp OPTIONAL,
//     OUT PBOOLEAN LastFrameInIrp OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return NULL;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsStreamPointerScheduleTimeout(
//     IN PKSSTREAM_POINTER StreamPointer,
//     IN PFNKSSTREAMPOINTER Callback,
//     IN ULONGLONG Interval)
// {
//     LARGE_INTEGER DueTime;
//     PKSISTREAM_POINTER Pointer;
//
//     /* get stream pointer */
//     Pointer = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     /* setup timer callback */
//     Pointer->Callback = Callback;
//
//     /* setup expiration */
//     DueTime.QuadPart = (LONGLONG)Interval;
//
//     /* setup the timer */
//     KeSetTimer(&Pointer->Timer, DueTime, &Pointer->TimerDpc);
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsStreamPointerCancelTimeout(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     PKSISTREAM_POINTER Pointer;
//
//     /* get stream pointer */
//     Pointer = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     KeCancelTimer(&Pointer->Timer);
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSSTREAM_POINTER
// NTAPI
// KsPinGetFirstCloneStreamPointer(
//     IN PKSPIN Pin)
// {
//     IKsPinImpl * This;
//
//     DPRINT("KsPinGetFirstCloneStreamPointer %p\n", Pin);
//
//     This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     if (!This->ClonedStreamPointer)
//         return NULL;
//
//     /* return first cloned stream pointer */
//     return &This->ClonedStreamPointer->StreamPointer;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSSTREAM_POINTER
// NTAPI
// KsStreamPointerGetNextClone(
//     IN PKSSTREAM_POINTER StreamPointer)
// {
//     PKSISTREAM_POINTER Pointer;
//
//     DPRINT("KsStreamPointerGetNextClone\n");
// DbgBreakPoint();
//     /* get stream pointer */
//     Pointer = (PKSISTREAM_POINTER)CONTAINING_RECORD!(StreamPointer, KSISTREAM_POINTER, StreamPointer);
//
//     /* is there a another cloned stream pointer */
//     if (!Pointer->Next)
//         return NULL;
//
//     /* return next stream pointer */
//     return &Pointer->Next->StreamPointer;
// }
//
// VOID
// NTAPI
// IKsPin_PinCentricWorker(
//     IN PVOID Parameter)
// {
//     NTSTATUS Status;
//     IKsPinImpl * This = (IKsPinImpl*)Parameter;
//
//     DPRINT("IKsPin_PinCentricWorker\n");
//
//     /* sanity checks */
//     ASSERT(This);
//     ASSERT(This->Pin.Descriptor);
//     ASSERT(This->Pin.Descriptor->Dispatch);
//     ASSERT(This->Pin.Descriptor->Dispatch->Process);
//     ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);
//     ASSERT(!(This->Pin.Descriptor->Flags & KSPIN_FLAG_DISPATCH_LEVEL_PROCESSING));
//     ASSERT(!(This->Pin.Descriptor->Flags & KSPIN_FLAG_GENERATE_MAPPINGS));
//
//     do
//     {
//         DPRINT("IKsPin_PinCentricWorker calling Pin Process Routine\n");
//
//         Status = This->Pin.Descriptor->Dispatch->Process(&This->Pin);
//         DPRINT("IKsPin_PinCentricWorker Status %lx, Offset %lu Length %lu\n", Status,
//                This->LeadingEdgeStreamPointer.Offset,
//                This->LeadingEdgeStreamPointer.Length);
//         break;
//
//     }while(This->IrpCount);
// }
//
//
// NTSTATUS
// NTAPI
// IKsPin_DispatchKsStream(
//     PDEVICE_OBJECT DeviceObject,
//     PIRP Irp,
//     IKsPinImpl * This)
// {
//     PKSPROCESSPIN_INDEXENTRY ProcessPinIndex;
//     PKSSTREAM_HEADER Header;
//     ULONG NumHeaders;
//     PKSFILTER Filter;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT("IKsPin_DispatchKsStream\n");
//
//     /* FIXME handle reset states */
//     ASSERT(This->Pin.ResetState == KSRESET_END);
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* probe stream pointer */
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_WRITE_STREAM)
//         Status = KsProbeStreamIrp(Irp, KSSTREAM_WRITE | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK | KSPROBE_SYSTEMADDRESS, This->Pin.StreamHeaderSize);
//     else
//         Status = KsProbeStreamIrp(Irp, KSSTREAM_READ  | KSPROBE_ALLOCATEMDL | KSPROBE_PROBEANDLOCK | KSPROBE_SYSTEMADDRESS, This->Pin.StreamHeaderSize);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("KsProbeStreamIrp failed with %x\n", Status);
//
//         Irp->IoStatus.Status = Status;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     if (Irp->RequestorMode == UserMode)
//         Header = (PKSSTREAM_HEADER)Irp->AssociatedIrp.SystemBuffer;
//     else
//         Header = (PKSSTREAM_HEADER)Irp->UserBuffer;
//
//     if (!Header)
//     {
//         DPRINT("NoHeader Canceling Irp %p\n", Irp);
//         Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//
//     /* calculate num headers */
//     NumHeaders = IoStack->Parameters.DeviceIoControl.OutputBufferLength / Header->Size;
//
//     /* assume headers of same length */
//     ASSERT(IoStack->Parameters.DeviceIoControl.OutputBufferLength % Header->Size == 0);
//
//     /* FIXME support multiple stream headers */
//     ASSERT(NumHeaders == 1);
//
//     if (Irp->RequestorMode == UserMode)
//     {
//         /* prepare header */
//         ASSERT(Irp->MdlAddress);
//         Header->Data = MmGetSystemAddressForMdlSafe(Irp->MdlAddress, NormalPagePriority);
//
//         if (!Header->Data)
//         {
//             DPRINT("NoHeader->Data Canceling Irp %p\n", Irp);
//             Irp->IoStatus.Status = STATUS_INSUFFICIENT_RESOURCES;
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//
//     }
//
//
//
//     if (This->Pin.Descriptor->Dispatch->Process)
//     {
//         /* it is a pin centric avstream */
//
//         /* mark irp as pending */
//         IoMarkIrpPending(Irp);
//
//         /* add irp to cancelable queue */
//         KsAddIrpToCancelableQueue(&This->IrpList, &This->IrpListLock, Irp, KsListEntryTail, NULL /* FIXME */);
//
//         /* sanity checks */
//         ASSERT(!(This->Pin.Descriptor->Flags & KSPIN_FLAG_DISPATCH_LEVEL_PROCESSING));
//         ASSERT(This->PinWorker);
//
//         InterlockedIncrement(&This->IrpCount);
//
//         DPRINT("IKsPin_DispatchKsStream IrpCount %lu\n", This->IrpCount);
//
//         /* start the processing loop */
//         KsIncrementCountedWorker(This->PinWorker);
//
//         Status = STATUS_PENDING;
//     }
//     else
//     {
//         /* filter-centric avstream */
//         ASSERT(This->Filter);
//
//         ProcessPinIndex = This->Filter->lpVtbl->GetProcessDispatch(This->Filter);
//         Filter = This->Filter->lpVtbl->GetStruct(This->Filter);
//
//         ASSERT(ProcessPinIndex);
//         ASSERT(Filter);
//         ASSERT(Filter->Descriptor);
//         ASSERT(Filter->Descriptor->Dispatch);
//
//         if (!Filter->Descriptor->Dispatch->Process)
//         {
//             /* invalid device request */
//             DPRINT("Filter Centric Processing No Process Routine\n");
//             Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_UNSUCCESSFUL;
//         }
//
//         /* mark irp as pending */
//         IoMarkIrpPending(Irp);
//
//         /* add irp to cancelable queue */
//         KsAddIrpToCancelableQueue(&This->IrpList, &This->IrpListLock, Irp, KsListEntryTail, NULL /* FIXME */);
//
//         Status = Filter->Descriptor->Dispatch->Process(Filter, ProcessPinIndex);
//
//         DPRINT("IKsPin_DispatchKsStream FilterCentric: Status %lx \n", Status);
//
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_DispatchDeviceIoControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     NTSTATUS Status;
//     UNICODE_STRING GuidString;
//     PKSPROPERTY Property;
//     ULONG SetCount = 0;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* locate ks pin implementation from KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     /* current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_READ_STREAM ||
//         IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_WRITE_STREAM)
//     {
//         /* handle ks stream packets */
//         return IKsPin_DispatchKsStream(DeviceObject, Irp, This);
//     }
//
//     /* get property from input buffer */
//     Property = (PKSPROPERTY)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     /* sanity check */
//     ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(KSIDENTIFIER));
//     ASSERT(This->Pin.Descriptor->AutomationTable);
//
//     RtlStringFromGUID(&Property->Set, &GuidString);
//     DPRINT("IKsPin_DispatchDeviceIoControl property Set |%S| Id %u Flags %x\n", GuidString.Buffer, Property->Id, Property->Flags);
//     RtlFreeUnicodeString(&GuidString);
//
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_METHOD)
//     {
//         const KSMETHOD_SET *MethodSet = NULL;
//         ULONG MethodItemSize = 0;
//
//         /* check if the driver supports method sets */
//         if (This->Pin.Descriptor->AutomationTable->MethodSetsCount)
//         {
//             SetCount = This->Pin.Descriptor->AutomationTable->MethodSetsCount;
//             MethodSet = This->Pin.Descriptor->AutomationTable->MethodSets;
//             MethodItemSize = This->Pin.Descriptor->AutomationTable->MethodItemSize;
//         }
//
//         /* call method set handler */
//         Status = KspMethodHandlerWithAllocator(Irp, SetCount, MethodSet, NULL, MethodItemSize);
//     }
//     else if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY)
//     {
//         const KSPROPERTY_SET *PropertySet = NULL;
//         ULONG PropertyItemSize = 0;
//
//         /* check if the driver supports method sets */
//         if (This->Pin.Descriptor->AutomationTable->PropertySetsCount)
//         {
//             SetCount = This->Pin.Descriptor->AutomationTable->PropertySetsCount;
//             PropertySet = This->Pin.Descriptor->AutomationTable->PropertySets;
//             PropertyItemSize = This->Pin.Descriptor->AutomationTable->PropertyItemSize;
//         }
//
//         /* needed for our property handlers */
//         KSPROPERTY_ITEM_IRP_STORAGE(Irp) = (KSPROPERTY_ITEM*)This;
//
//         /* call property handler */
//         Status = KspPropertyHandler(Irp, SetCount, PropertySet, NULL, PropertyItemSize);
//     }
//     else
//     {
//         /* sanity check */
//         ASSERT(IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_ENABLE_EVENT ||
//                IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_DISABLE_EVENT);
//
//         if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_ENABLE_EVENT)
//         {
//             /* call enable event handlers */
//             Status = KspEnableEvent(Irp,
//                                     This->Pin.Descriptor->AutomationTable->EventSetsCount,
//                                     (PKSEVENT_SET)This->Pin.Descriptor->AutomationTable->EventSets,
//                                     &This->BasicHeader.EventList,
//                                     KSEVENTS_SPINLOCK,
//                                     (PVOID)&This->BasicHeader.EventListLock,
//                                     NULL,
//                                     This->Pin.Descriptor->AutomationTable->EventItemSize);
//         }
//         else
//         {
//             /* disable event handler */
//             Status = KsDisableEvent(Irp, &This->BasicHeader.EventList, KSEVENTS_SPINLOCK, &This->BasicHeader.EventListLock);
//         }
//     }
//
//     RtlStringFromGUID(&Property->Set, &GuidString);
//     DPRINT("IKsPin_DispatchDeviceIoControl property Set |%S| Id %u Flags %x Status %lx ResultLength %lu\n", GuidString.Buffer, Property->Id, Property->Flags, Status, Irp->IoStatus.Information);
//     RtlFreeUnicodeString(&GuidString);
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Status = Status;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_Close(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     IKsPinImpl * This;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//     ASSERT(IoStack->FileObject->FsContext2);
//
//     /* get the object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* locate ks pin implementation fro KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(ObjectHeader->ObjectType, IKsPinImpl, Pin);
//
//     if (This->Pin.Descriptor->Dispatch->Close)
//     {
//         /* call pin close routine */
//         Status = This->Pin.Descriptor->Dispatch->Close(&This->Pin, Irp);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* abort closing */
//             Irp->IoStatus.Status = Status;
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//
//         /* remove pin from filter pin list and decrement reference count */
//         IKsFilter_RemovePin(This->Filter->lpVtbl->GetStruct(This->Filter), &This->Pin);
//
//         if (Status != STATUS_PENDING)
//         {
//             Irp->IoStatus.Status = Status;
//             CompleteRequest(Irp, IO_NO_INCREMENT);
//             return Status;
//         }
//     }
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_DispatchCreateAllocator(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     UNIMPLEMENTED;
//
//     Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_DispatchCreateClock(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PKSPIN Pin;
//     NTSTATUS Status = STATUS_SUCCESS;
//     IKsPinImpl * This;
//     KSRESOLUTION Resolution;
//     PKSRESOLUTION pResolution = NULL;
//     PKSOBJECT_CREATE_ITEM CreateItem;
//
//     DPRINT("IKsPin_DispatchCreateClock\n");
//
//     /* get the create item */
//     CreateItem = KSCREATE_ITEM_IRP_STORAGE(Irp);
//
//     /* sanity check */
//     ASSERT(CreateItem);
//
//     /* get the pin object */
//     Pin = (PKSPIN)CreateItem->Context;
//
//     /* sanity check */
//     ASSERT(Pin);
//
//     /* locate ks pin implementation fro KSPIN offset */
//     This = (IKsPinImpl*)CONTAINING_RECORD!(Pin, IKsPinImpl, Pin);
//
//     /* sanity check */
//     ASSERT(This->BasicHeader.Type == KsObjectTypePin);
//     ASSERT(This->BasicHeader.ControlMutex);
//
//     /* acquire control mutex */
//     KsAcquireControl(Pin);
//
//     if ((This->Pin.Descriptor->PinDescriptor.Communication != KSPIN_COMMUNICATION_NONE &&
//         This->Pin.Descriptor->Dispatch) ||
//         (This->Pin.Descriptor->Flags & KSPIN_FLAG_IMPLEMENT_CLOCK))
//     {
//         if (!This->DefaultClock)
//         {
//             if (This->Pin.Descriptor->Dispatch && This->Pin.Descriptor->Dispatch->Clock)
//             {
//                 if (This->Pin.Descriptor->Dispatch->Clock->Resolution)
//                 {
//                    This->Pin.Descriptor->Dispatch->Clock->Resolution(&This->Pin, &Resolution);
//                    pResolution = &Resolution;
//                 }
//
//                 Status = KsAllocateDefaultClockEx(&This->DefaultClock,
//                                                   (PVOID)&This->Pin,
//                                                   (PFNKSSETTIMER)This->Pin.Descriptor->Dispatch->Clock->SetTimer,
//                                                   (PFNKSCANCELTIMER)This->Pin.Descriptor->Dispatch->Clock->CancelTimer,
//                                                   (PFNKSCORRELATEDTIME)This->Pin.Descriptor->Dispatch->Clock->CorrelatedTime,
//                                                   pResolution,
//                                                   0);
//             }
//             else
//             {
//                 Status = KsAllocateDefaultClockEx(&This->DefaultClock, (PVOID)&This->Pin, NULL, NULL, NULL, NULL, 0);
//             }
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             Status = KsCreateDefaultClock(Irp, This->DefaultClock);
//         }
//     }
//
//     DPRINT("IKsPin_DispatchCreateClock %lx\n", Status);
//
//     /* release control mutex */
//     KsReleaseControl(Pin);
//
//     /* done */
//     Irp->IoStatus.Status = Status;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsPin_DispatchCreateNode(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     UNIMPLEMENTED;
//
//     Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// static KSDISPATCH_TABLE PinDispatchTable =
// {
//     IKsPin_DispatchDeviceIoControl,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     IKsPin_Close,
//     KsDispatchQuerySecurity,
//     KsDispatchSetSecurity,
//     KsDispatchFastIoDeviceControlFailure,
//     KsDispatchFastReadFailure,
//     KsDispatchFastReadFailure
// };
//
// NTSTATUS
// KspCreatePin(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN PKSDEVICE KsDevice,
//     IN IKsFilterFactory * FilterFactory,
//     IN IKsFilter* Filter,
//     IN PKSPIN_CONNECT Connect,
//     IN KSPIN_DESCRIPTOR_EX* Descriptor)
// {
//     IKsPinImpl * This;
//     PIO_STACK_LOCATION IoStack;
//     IKsDevice * Device;
//     PDEVICE_EXTENSION DeviceExtension;
//     PKSOBJECT_CREATE_ITEM CreateItem;
//     NTSTATUS Status;
//     PKSDATAFORMAT DataFormat;
//     PKSBASIC_HEADER BasicHeader;
//     ULONG Index;
//     ULONG FrameSize = 0;
//     ULONG NumFrames = 0;
//     KSAUTOMATION_TABLE AutomationTable;
//
//     /* sanity checks */
//     ASSERT(Descriptor->Dispatch);
//
//     DPRINT("KspCreatePin PinId %lu Flags %x\n", Connect->PinId, Descriptor->Flags);
//
// //Output Pin: KSPIN_FLAG_PROCESS_IN_RUN_STATE_ONLY
// //Input Pin: KSPIN_FLAG_FIXED_FORMAT|KSPIN_FLAG_DO_NOT_USE_STANDARD_TRANSPORT|KSPIN_FLAG_FRAMES_NOT_REQUIRED_FOR_PROCESSING
//
//     DPRINT("KspCreatePin Dataflow %lu\n", Descriptor->PinDescriptor.DataFlow);
//     DPRINT("KspCreatePin Communication %lu\n", Descriptor->PinDescriptor.Communication);
//     if (Descriptor->AllocatorFraming)
//     {
//         DPRINT("KspCreatePin CountItems %lu\n", Descriptor->AllocatorFraming->CountItems);
//         DPRINT("KspCreatePin PinFlags %lx\n", Descriptor->AllocatorFraming->PinFlags);
//         DPRINT("KspCreatePin OutputCompression RatioNumerator %lu RatioDenominator  %lu RatioConstantMargin %lu\n", Descriptor->AllocatorFraming->OutputCompression.RatioNumerator,
//                Descriptor->AllocatorFraming->OutputCompression.RatioDenominator, Descriptor->AllocatorFraming->OutputCompression.RatioConstantMargin);
//         DPRINT("KspCreatePin PinWeight %lx\n", Descriptor->AllocatorFraming->PinWeight);
//
//         for(Index = 0; Index < Descriptor->AllocatorFraming->CountItems; Index++)
//         {
//             DPRINT("KspCreatePin Index %lu MemoryFlags %lx\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].MemoryFlags);
//             DPRINT("KspCreatePin Index %lu BusFlags %lx\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].BusFlags);
//             DPRINT("KspCreatePin Index %lu Flags %lx\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].Flags);
//             DPRINT("KspCreatePin Index %lu Frames %lu\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].Frames);
//             DPRINT("KspCreatePin Index %lu FileAlignment %lx\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].FileAlignment);
//             DPRINT("KspCreatePin Index %lu MemoryTypeWeight %lx\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].MemoryTypeWeight);
//             DPRINT("KspCreatePin Index %lu PhysicalRange MinFrameSize %lu MaxFrameSize %lu Stepping %lu\n", Index, Descriptor->AllocatorFraming->FramingItem[Index].PhysicalRange.MinFrameSize,
//                    Descriptor->AllocatorFraming->FramingItem[Index].PhysicalRange.MaxFrameSize,
//                    Descriptor->AllocatorFraming->FramingItem[Index].PhysicalRange.Stepping);
//
//             DPRINT("KspCreatePin Index %lu FramingRange  MinFrameSize %lu MaxFrameSize %lu Stepping %lu InPlaceWeight %lu NotInPlaceWeight %lu\n",
//                    Index,
//                    Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.Range.MinFrameSize,
//                    Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.Range.MaxFrameSize,
//                    Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.Range.Stepping,
//                    Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.InPlaceWeight,
//                    Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.NotInPlaceWeight);
//
//            FrameSize = Descriptor->AllocatorFraming->FramingItem[Index].FramingRange.Range.MaxFrameSize;
//            NumFrames = Descriptor->AllocatorFraming->FramingItem[Index].Frames;
//         }
//     }
//
//      for (Index = 0; Index < Descriptor->PinDescriptor.DataRangesCount; Index++)
//      {
//          UNICODE_STRING GuidString;
//          /* convert the guid to string */
//          RtlStringFromGUID(&Descriptor->PinDescriptor.DataRanges[Index]->MajorFormat, &GuidString);
//          DPRINT("Index %lu MajorFormat %S\n", Index, GuidString.Buffer);
//          RtlStringFromGUID(&Descriptor->PinDescriptor.DataRanges[Index]->SubFormat, &GuidString);
//          DPRINT("Index %lu SubFormat %S\n", Index, GuidString.Buffer);
//          RtlStringFromGUID(&Descriptor->PinDescriptor.DataRanges[Index]->Specifier, &GuidString);
//          DPRINT("Index %lu Specifier %S\n", Index, GuidString.Buffer);
//          RtlStringFromGUID(&Descriptor->PinDescriptor.DataRanges[Index]->Specifier, &GuidString);
//          DPRINT("Index %lu FormatSize %lu Flags %lu SampleSize %lu Reserved %lu KSDATAFORMAT %lu\n", Index,
//                 Descriptor->PinDescriptor.DataRanges[Index]->FormatSize, Descriptor->PinDescriptor.DataRanges[Index]->Flags, Descriptor->PinDescriptor.DataRanges[Index]->SampleSize, Descriptor->PinDescriptor.DataRanges[Index]->Reserved, sizeof(KSDATAFORMAT));
//
//          if (IsEqualGUIDAligned(&Descriptor->PinDescriptor.DataRanges[Index]->SubFormat, &KSDATAFORMAT_SUBTYPE_BDA_MPEG2_TRANSPORT))
//          {
// #if !defined(NDEBUG)
//              PKS_DATARANGE_BDA_TRANSPORT Transport = (PKS_DATARANGE_BDA_TRANSPORT)&Descriptor->PinDescriptor.DataRanges[Index];
//              DPRINT("KSDATAFORMAT_SUBTYPE_BDA_MPEG2_TRANSPORT AvgTimePerFrame %I64u ulcbPhyiscalFrame %lu ulcbPhyiscalFrameAlignment %lu ulcbPhyiscalPacket %lu\n", Transport->BdaTransportInfo.AvgTimePerFrame, Transport->BdaTransportInfo.ulcbPhyiscalFrame,
//                     Transport->BdaTransportInfo.ulcbPhyiscalFrameAlignment, Transport->BdaTransportInfo.ulcbPhyiscalPacket);
// #endif
//          }
//     }
//     if (!FrameSize)
//     {
//         /* default to 50 * 188 (MPEG2 TS packet size) */
//         FrameSize = 9400;
//     }
//
//     if (!NumFrames)
//     {
//         NumFrames = 8;
//     }
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get ks device interface */
//     Device = (IKsDevice*)&DeviceExtension->DeviceHeader->BasicHeader.OuterUnknown;
//
//     /* first allocate pin ctx */
//     This = AllocateItem(NonPagedPool, sizeof(IKsPinImpl));
//     if (!This)
//     {
//         /* not enough memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* allocate create item */
//     CreateItem = AllocateItem(NonPagedPool, sizeof(KSOBJECT_CREATE_ITEM) * 3);
//     if (!CreateItem)
//     {
//         /* not enough memory */
//         FreeItem(This);
//         DPRINT("KspCreatePin OutOfMemory\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize basic header */
//     This->BasicHeader.KsDevice = KsDevice;
//     This->BasicHeader.Type = KsObjectTypePin;
//     This->BasicHeader.Parent.KsFilter = Filter->lpVtbl->GetStruct(Filter);
//     This->BasicHeader.OuterUnknown = (PUNKNOWN)&vt_IKsPin;
//     InitializeListHead(&This->BasicHeader.EventList);
//     KeInitializeSpinLock(&This->BasicHeader.EventListLock);
//
//     ASSERT(This->BasicHeader.Parent.KsFilter);
//
//     BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)This->BasicHeader.Parent.KsFilter - sizeof(KSBASIC_HEADER));
//
//     This->BasicHeader.ControlMutex = BasicHeader->ControlMutex;
//     ASSERT(This->BasicHeader.ControlMutex);
//
//     InitializeListHead(&This->BasicHeader.EventList);
//     KeInitializeSpinLock(&This->BasicHeader.EventListLock);
//
//     /* initialize pin */
//     This->FrameSize = FrameSize;
//     This->NumFrames = NumFrames;
//     This->lpVtblReferenceClock = &vt_ReferenceClock;
//     This->ref = 1;
//     This->FileObject = IoStack->FileObject;
//     This->Filter = Filter;
//     KeInitializeMutex(&This->ProcessingMutex, 0);
//     InitializeListHead(&This->IrpList);
//     KeInitializeSpinLock(&This->IrpListLock);
//
//     /* allocate object bag */
//     This->Pin.Bag = AllocateItem(NonPagedPool, sizeof(KSIOBJECT_BAG));
//     if (!This->Pin.Bag)
//     {
//         /* not enough memory */
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize object bag */
//     Device->lpVtbl->InitializeObjectBag(Device, This->Pin.Bag, NULL);
//
//     /* allocate pin descriptor */
//     This->Pin.Descriptor = AllocateItem(NonPagedPool, sizeof(KSPIN_DESCRIPTOR_EX));
//     if (!This->Pin.Descriptor)
//     {
//         /* not enough memory */
//         KsFreeObjectBag(This->Pin.Bag);
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy pin descriptor */
//     RtlMoveMemory((PVOID)This->Pin.Descriptor, Descriptor, sizeof(KSPIN_DESCRIPTOR_EX));
//
//     /* initialize automation table */
//     RtlZeroMemory(&AutomationTable, sizeof(KSAUTOMATION_TABLE));
//
//     AutomationTable.PropertyItemSize = sizeof(KSPROPERTY_ITEM);
//     AutomationTable.PropertySets = PinPropertySet;
//     AutomationTable.PropertySetsCount = sizeof(PinPropertySet) / sizeof(KSPROPERTY_SET);
//
//     /* merge in pin property sets */
//     Status = KsMergeAutomationTables((PKSAUTOMATION_TABLE*)&This->Pin.Descriptor->AutomationTable, (PKSAUTOMATION_TABLE)Descriptor->AutomationTable, &AutomationTable, This->Pin.Bag);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* not enough memory */
//         KsFreeObjectBag(This->Pin.Bag);
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return Status;
//     }
//
//     /* get format */
//     DataFormat = (PKSDATAFORMAT)(Connect + 1);
//
//     /* initialize pin descriptor */
//     This->Pin.Context = NULL;
//     This->Pin.Id = Connect->PinId;
//     This->Pin.Communication = Descriptor->PinDescriptor.Communication;
//     This->Pin.ConnectionIsExternal = FALSE; //FIXME
//     RtlMoveMemory(&This->Pin.ConnectionInterface, &Connect->Interface, sizeof(KSPIN_INTERFACE));
//     RtlMoveMemory(&This->Pin.ConnectionMedium, &Connect->Medium, sizeof(KSPIN_MEDIUM));
//     RtlMoveMemory(&This->Pin.ConnectionPriority, &Connect->Priority, sizeof(KSPRIORITY));
//
//     /* allocate format */
//     Status = _KsEdit(This->Pin.Bag, (PVOID*)&This->Pin.ConnectionFormat, DataFormat->FormatSize, DataFormat->FormatSize, 0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to allocate format */
//         KsFreeObjectBag((KSOBJECT_BAG)This->Pin.Bag);
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy format */
//     RtlMoveMemory((PVOID)This->Pin.ConnectionFormat, DataFormat, DataFormat->FormatSize);
//
//     This->Pin.AttributeList = NULL; //FIXME
//     This->Pin.StreamHeaderSize = sizeof(KSSTREAM_HEADER);
//     This->Pin.DataFlow = Descriptor->PinDescriptor.DataFlow;
//     This->Pin.DeviceState = KSSTATE_STOP;
//     This->Pin.ResetState = KSRESET_END;
//     This->Pin.ClientState = KSSTATE_STOP;
//
//     /* initialize allocator create item */
//     CreateItem[0].Context = (PVOID)&This->Pin;
//     CreateItem[0].Create = IKsPin_DispatchCreateAllocator;
//     CreateItem[0].Flags = KSCREATE_ITEM_FREEONSTOP;
//     RtlInitUnicodeString(&CreateItem[0].ObjectClass, KSSTRING_Allocator);
//
//     /* initialize clock create item */
//     CreateItem[1].Context = (PVOID)&This->Pin;
//     CreateItem[1].Create = IKsPin_DispatchCreateClock;
//     CreateItem[1].Flags = KSCREATE_ITEM_FREEONSTOP;
//     RtlInitUnicodeString(&CreateItem[1].ObjectClass, KSSTRING_Clock);
//
//     /* initialize topology node create item */
//     CreateItem[2].Context = (PVOID)&This->Pin;
//     CreateItem[2].Create = IKsPin_DispatchCreateNode;
//     CreateItem[2].Flags = KSCREATE_ITEM_FREEONSTOP;
//     RtlInitUnicodeString(&CreateItem[2].ObjectClass, KSSTRING_TopologyNode);
//
//     /* now allocate object header */
//     Status = KsAllocateObjectHeader((KSOBJECT_HEADER*)&This->ObjectHeader, 3, CreateItem, Irp, &PinDispatchTable);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to create object header */
//         DPRINT("KspCreatePin KsAllocateObjectHeader failed %lx\n", Status);
//         KsFreeObjectBag((KSOBJECT_BAG)This->Pin.Bag);
//         FreeItem(This);
//         FreeItem(CreateItem);
//
//         /* return failure code */
//         return Status;
//     }
//
//      /* add extra info to object header */
//     This->ObjectHeader->Type = KsObjectTypePin;
//     This->ObjectHeader->Unknown = (PUNKNOWN)&This->BasicHeader.OuterUnknown;
//     This->ObjectHeader->ObjectType = (PVOID)&This->Pin;
//
//     if (!Descriptor->Dispatch || !Descriptor->Dispatch->Process)
//     {
//         /* the pin is part of filter-centric processing filter
//          * add process pin to filter
//          */
//         This->ProcessPin.BytesAvailable = 0;
//         This->ProcessPin.BytesUsed = 0;
//         This->ProcessPin.CopySource = NULL;
//         This->ProcessPin.Data = NULL;
//         This->ProcessPin.DelegateBranch = NULL;
//         This->ProcessPin.Flags = 0;
//         This->ProcessPin.InPlaceCounterpart = NULL;
//         This->ProcessPin.Pin = &This->Pin;
//         This->ProcessPin.StreamPointer = (PKSSTREAM_POINTER)&This->LeadingEdgeStreamPointer.StreamPointer;
//         This->ProcessPin.Terminate = FALSE;
//
//         Status = Filter->lpVtbl->AddProcessPin(Filter, &This->ProcessPin);
//         DPRINT("KspCreatePin AddProcessPin %lx\n", Status);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to add process pin */
//             KsFreeObjectBag((KSOBJECT_BAG)This->Pin.Bag);
//             KsFreeObjectHeader(&This->ObjectHeader);
//             FreeItem(This);
//             FreeItem(CreateItem);
//             /* return failure code */
//             return Status;
//         }
//     }
//     else if (Descriptor->Dispatch && Descriptor->Dispatch->Process)
//     {
//         /* pin centric processing filter */
//
//         /* initialize work item */
//         ExInitializeWorkItem(&This->PinWorkQueueItem, IKsPin_PinCentricWorker, (PVOID)This);
//
//         /* allocate counted work item */
//         Status = KsRegisterCountedWorker(HyperCriticalWorkQueue, &This->PinWorkQueueItem, &This->PinWorker);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("Failed to register Worker %lx\n", Status);
//             KsFreeObjectBag((KSOBJECT_BAG)This->Pin.Bag);
//             KsFreeObjectHeader(&This->ObjectHeader);
//             FreeItem(This);
//             FreeItem(CreateItem);
//             return Status;
//         }
//
//         if (This->Pin.Descriptor->PinDescriptor.DataFlow == KSPIN_DATAFLOW_IN)
//             This->LeadingEdgeStreamPointer.StreamPointer.Offset = &This->LeadingEdgeStreamPointer.StreamPointer.OffsetIn;
//         else
//             This->LeadingEdgeStreamPointer.StreamPointer.Offset = &This->LeadingEdgeStreamPointer.StreamPointer.OffsetOut;
//
//
//         KeInitializeEvent(&This->FrameComplete, NotificationEvent, FALSE);
//
//     }
//
//     /* FIXME add pin instance to filter instance */
//     IKsFilter_AddPin(Filter->lpVtbl->GetStruct(Filter), &This->Pin);
//
//     if (Descriptor->Dispatch && Descriptor->Dispatch->SetDataFormat)
//     {
//         Status = Descriptor->Dispatch->SetDataFormat(&This->Pin, NULL, NULL, This->Pin.ConnectionFormat, NULL);
//         DPRINT("KspCreatePin SetDataFormat %lx\n", Status);
//     }
//
//
//     /* does the driver have a pin dispatch */
//     if (Descriptor->Dispatch && Descriptor->Dispatch->Create)
//     {
//         /*  now inform the driver to create a new pin */
//         Status = Descriptor->Dispatch->Create(&This->Pin, Irp);
//         DPRINT("KspCreatePin DispatchCreate %lx\n", Status);
//     }
//
//
//     DPRINT("KspCreatePin Status %lx KsDevice %p\n", Status, KsDevice);
//
//     if (!NT_SUCCESS(Status) && Status != STATUS_PENDING)
//     {
//         /* failed to create pin, release resources */
//         IKsFilter_RemovePin(Filter->lpVtbl->GetStruct(Filter), &This->Pin);
//         KsFreeObjectBag((KSOBJECT_BAG)This->Pin.Bag);
//         KsFreeObjectHeader((KSOBJECT_HEADER)This->ObjectHeader);
//         FreeItem(This);
//
//         /* return failure code */
//         return Status;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
