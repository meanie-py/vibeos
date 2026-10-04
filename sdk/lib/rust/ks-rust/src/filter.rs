use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{_KWAIT_REASON_Executive, GUID, IoGetCurrentIrpStackLocation, KeInitializeMutex, KeInitializeSpinLock, KeReadStateMutex, KeReleaseMutex, KeWaitForSingleObject, LARGE_INTEGER, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIO_STATUS_BLOCK, PIRP, RtlFreeUnicodeString, RtlInitUnicodeString, RtlStringFromGUID};
use ntkrnl_sys::ddk::_WORK_QUEUE_TYPE_HyperCriticalWorkQueue as HyperCriticalWorkQueue;
use ntkrnl_sys::ddk::RtlAssert;
use ntkrnl_sys::ddk::_GUID;
use ntkrnl_sys::inline::ex::ExInitializeWorkItem;
use ntkrnl_sys::interlocked::InterlockedIncrement as _InterlockedIncrement;
use ntkrnl_sys::driver_ddk::DbgPrint;
use crate::misc::{FreeItem, AllocateItem, CompleteRequest};
use crate::connectivity::{KspPinPropertyHandler, KspValidateConnectRequest};
use crate::property::KspPropertyHandler;
use crate::methods::KspMethodHandlerWithAllocator;
use crate::pin::KspCreatePin;
use crate::event::KspEnableEvent;
use core::ffi::c_void;

// IsEqualGUIDAligned (guiddef.h inline): byte-equal GUID compare. Defined locally to match
// the sibling ks modules (api.rs/pin.rs/swenum.rs each define their own private copy); also
// reported in ~/vibeos-rust-staging/ks-glue-needs-filter.txt for the integrator to centralize.
#[inline]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> BOOLEAN {
    ((*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4) as BOOLEAN
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_Pin: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x001\x004\x006\0F\x001\0A\08\x000\0-\x004\x007\09\x001\0-\x001\x001\0D\x000\0-\0A\x005\0D\x006\0-\x002\08\0D\0B\x000\x004\0C\x001\x000\x000\x000\x000\0}\0\0\0",
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

pub static mut IID_IKsControl: GUID = _GUID {
    Data1: 0x28f54685 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x6fd as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xb2 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x7a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut IID_IKsFilter: GUID = _GUID {
    Data1: 0x3ef6ee44 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0xd41 as ::core::ffi::c_ushort,
    Data3: 0x11d2 as ::core::ffi::c_ushort,
    Data4: [
        0xbe as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xda as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x4f as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x57 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut KSPROPSETID_Topology: GUID = _GUID {
    Data1: 0x720d4ac0 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x7533 as ::core::ffi::c_ushort,
    Data3: 0x11d0 as ::core::ffi::c_ushort,
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

pub static mut KSPROPSETID_Pin: GUID = _GUID {
    Data1: 0x8c134960 as ::core::ffi::c_ulong as ::core::ffi::c_uint,
    Data2: 0x51ad as ::core::ffi::c_ushort,
    Data3: 0x11cf as ::core::ffi::c_ushort,
    Data4: [
        0x87 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x94 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

pub static mut KSPROPSETID_General: GUID = _GUID {
    Data1: 0x1464eda5 as ::core::ffi::c_long as ::core::ffi::c_uint,
    Data2: 0x6a8f as ::core::ffi::c_ushort,
    Data3: 0x11d1 as ::core::ffi::c_ushort,
    Data4: [
        0x9a as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xa0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x31 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x96 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

// Property-item tables. The Get/Set handler slots live in bindgen unions whose only
// storage is a plain u32, so a function pointer cannot be placed there in a const
// initializer (fn-ptr -> integer is not const-evaluable). The unions are const-init'd
// zeroed here; the handlers are installed at runtime by run_static_initializers() (the
// c2rust .init_array constructor), exactly as the C designated-initializer tables would
// have them. All scalar/size fields are const-initialized faithfully from the DEFINE_*
// macros in ks.h.

const fn ks_union1_zero() -> KSPROPERTY_ITEM__bindgen_ty_1 {
    KSPROPERTY_ITEM__bindgen_ty_1 {
        GetPropertyHandler: __BindgenUnionField::new(),
        GetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0,
    }
}
const fn ks_union2_zero() -> KSPROPERTY_ITEM__bindgen_ty_2 {
    KSPROPERTY_ITEM__bindgen_ty_2 {
        SetPropertyHandler: __BindgenUnionField::new(),
        SetSupported: __BindgenUnionField::new(),
        bindgen_union_field: 0,
    }
}
const fn ks_item(PropertyId: ULONG, MinProperty: ULONG, MinData: ULONG) -> KSPROPERTY_ITEM {
    KSPROPERTY_ITEM {
        PropertyId,
        __bindgen_anon_1: ks_union1_zero(),
        MinProperty,
        MinData,
        __bindgen_anon_2: ks_union2_zero(),
        Values: ::core::ptr::null::<KSPROPERTY_VALUES>(),
        RelationsCount: 0,
        Relations: ::core::ptr::null::<KSPROPERTY>(),
        SupportHandler: None,
        SerializedSize: 0,
    }
}

// GetPropertyHandler = FilterTopologyPropertyHandler (installed at runtime).
pub static mut IKsFilterTopologySet: [KSPROPERTY_ITEM; 4] = [
    ks_item(KSPROPERTY_TOPOLOGY_CATEGORIES as ULONG, ::core::mem::size_of::<KSPROPERTY>() as ULONG, 0),
    ks_item(KSPROPERTY_TOPOLOGY_NODES as ULONG, ::core::mem::size_of::<KSPROPERTY>() as ULONG, 0),
    ks_item(KSPROPERTY_TOPOLOGY_CONNECTIONS as ULONG, ::core::mem::size_of::<KSPROPERTY>() as ULONG, 0),
    ks_item(KSPROPERTY_TOPOLOGY_NAME as ULONG, ::core::mem::size_of::<KSP_NODE>() as ULONG, 0),
];

// DEFINE_KSPROPERTY_PINPROPOSEDATAFORMAT(IKsFilterPinSet, FilterPinPropertyHandler x3).
// Items 0..=10 use GetPropertyHandler = FilterPinPropertyHandler; item 11
// (PROPOSEDATAFORMAT) uses SetPropertyHandler = FilterPinPropertyHandler. Installed
// at runtime by run_static_initializers().
pub static mut IKsFilterPinSet: [KSPROPERTY_ITEM; 12] = [
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_CINSTANCES as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, ::core::mem::size_of::<KSPIN_CINSTANCES>() as ULONG),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_CTYPES as ULONG, ::core::mem::size_of::<KSPROPERTY>() as ULONG, ::core::mem::size_of::<ULONG>() as ULONG),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_DATAFLOW as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, ::core::mem::size_of::<KSPIN_DATAFLOW>() as ULONG),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_DATARANGES as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_DATAINTERSECTION as ULONG, (::core::mem::size_of::<KSP_PIN>() + ::core::mem::size_of::<KSMULTIPLE_ITEM>()) as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_INTERFACES as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_MEDIUMS as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_COMMUNICATION as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, ::core::mem::size_of::<KSPIN_COMMUNICATION>() as ULONG),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_CATEGORY as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, ::core::mem::size_of::<GUID>() as ULONG),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_NAME as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_CONSTRAINEDDATARANGES as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, 0),
    ks_item(KSPROPERTY_PIN_KSPROPERTY_PIN_PROPOSEDATAFORMAT as ULONG, ::core::mem::size_of::<KSP_PIN>() as ULONG, ::core::mem::size_of::<KSDATAFORMAT>() as ULONG),
];

// GetPropertyHandler = FilterGeneralComponentIdHandler (installed at runtime).
pub static mut IKsFilterGeneralSet: [KSPROPERTY_ITEM; 1] = [
    ks_item(KSPROPERTY_GENERAL_COMPONENTID as ULONG, ::core::mem::size_of::<KSPROPERTY>() as ULONG, ::core::mem::size_of::<KSCOMPONENTID>() as ULONG),
];

// KSPROPERTY_SET is not Copy, so the `[x; 3]` repeat syntax can't be used; spell out the
// three elements. The Set/Count/PropertyItem fields are filled at runtime by
// run_static_initializers() (they take the address of other statics, not const-evaluable).
const fn ks_empty_propset() -> KSPROPERTY_SET {
    KSPROPERTY_SET {
        Set: ::core::ptr::null::<GUID>(),
        PropertiesCount: 0,
        PropertyItem: ::core::ptr::null::<KSPROPERTY_ITEM>(),
        FastIoCount: 0,
        FastIoTable: ::core::ptr::null::<KSFASTPROPERTY_ITEM>(),
    }
}
pub static mut FilterPropertySet: [KSPROPERTY_SET; 3] =
    [ks_empty_propset(), ks_empty_propset(), ks_empty_propset()];

// NOTE: vtbl slots below are bindgen `extern "system"` Option<fn> fields; the methods are
// defined `extern "system"` (== stdcall on i386). `Some(f as extern "C" fn)` both mis-casts
// the ABI and mis-matches the slot, so each slot is filled via a const-safe fn-ptr transmute
// (transmuting the fn POINTER is const-evaluable; `f as usize` would not be).
static mut vt_IKsProcessingObject: IKsProcessingObjectVtbl = unsafe {
    IKsProcessingObjectVtbl {
        QueryInterface: ::core::mem::transmute(
            IKsProcessingObject_fnQueryInterface
                as *const (),
        ),
        AddRef: ::core::mem::transmute(
            IKsProcessingObject_fnAddRef as *const (),
        ),
        Release: ::core::mem::transmute(
            IKsProcessingObject_fnRelease
                as *const (),
        ),
        ProcessingObjectWork: ::core::mem::transmute(
            IKsProcessingObject_fnProcessingObjectWork
                as *const (),
        ),
        GetAndGate: ::core::mem::transmute(
            IKsProcessingObject_fnGetAndGate
                as *const (),
        ),
        Process: ::core::mem::transmute(
            IKsProcessingObject_fnProcess
                as *const (),
        ),
        Reset: ::core::mem::transmute(
            IKsProcessingObject_fnReset as *const (),
        ),
        TriggerNotification: ::core::mem::transmute(
            IKsProcessingObject_fnTriggerNotification
                as *const (),
        ),
    }
};

static mut vt_IKsControl: IKsControlVtbl = unsafe {
    IKsControlVtbl {
        QueryInterface: ::core::mem::transmute(
            IKsControl_fnQueryInterface
                as *const (),
        ),
        AddRef: ::core::mem::transmute(IKsControl_fnAddRef as *const ()),
        Release: ::core::mem::transmute(IKsControl_fnRelease as *const ()),
        KsProperty: ::core::mem::transmute(
            IKsControl_fnKsProperty
                as *const (),
        ),
        KsMethod: ::core::mem::transmute(
            IKsControl_fnKsMethod
                as *const (),
        ),
        KsEvent: ::core::mem::transmute(
            IKsControl_fnKsEvent
                as *const (),
        ),
    }
};

static mut vt_IKsFilter: IKsFilterVtbl = unsafe {
    IKsFilterVtbl {
        QueryInterface: ::core::mem::transmute(
            IKsFilter_fnQueryInterface
                as *const (),
        ),
        AddRef: ::core::mem::transmute(IKsFilter_fnAddRef as *const ()),
        Release: ::core::mem::transmute(IKsFilter_fnRelease as *const ()),
        GetStruct: ::core::mem::transmute(IKsFilter_fnGetStruct as *const ()),
        DoAllNecessaryPinsExist: ::core::mem::transmute(
            IKsFilter_fnDoAllNecessaryPinsExist as *const (),
        ),
        CreateNode: ::core::mem::transmute(
            IKsFilter_fnCreateNode
                as *const (),
        ),
        BindProcessPinsToPipeSection: ::core::mem::transmute(
            IKsFilter_fnBindProcessPinsToPipeSection
                as *const (),
        ),
        UnbindProcessPinsFromPipeSection: ::core::mem::transmute(
            IKsFilter_fnUnbindProcessPinsFromPipeSection
                as *const (),
        ),
        AddProcessPin: ::core::mem::transmute(
            IKsFilter_fnAddProcessPin
                as *const (),
        ),
        RemoveProcessPin: ::core::mem::transmute(
            IKsFilter_fnRemoveProcessPin
                as *const (),
        ),
        ReprepareProcessPipeSection: ::core::mem::transmute(
            IKsFilter_fnReprepareProcessPipeSection
                as *const (),
        ),
        DeliverResetState: ::core::mem::transmute(
            IKsFilter_fnDeliverResetState
                as *const (),
        ),
        IsFrameHolding: ::core::mem::transmute(
            IKsFilter_fnIsFrameHolding as *const (),
        ),
        RegisterForCopyCallbacks: ::core::mem::transmute(
            IKsFilter_fnRegisterForCopyCallbacks
                as *const (),
        ),
        GetProcessDispatch: ::core::mem::transmute(
            IKsFilter_fnGetProcessDispatch
                as *const (),
        ),
    }
};

static mut DispatchTable: KSDISPATCH_TABLE = unsafe {
    KSDISPATCH_TABLE {
        DeviceIoControl: ::core::mem::transmute(
            IKsFilter_DispatchDeviceIoControl
                as *const (),
        ),
        Read: ::core::mem::transmute(
            KsDispatchInvalidDeviceRequest
                as *const (),
        ),
        Write: ::core::mem::transmute(
            KsDispatchInvalidDeviceRequest
                as *const (),
        ),
        Flush: ::core::mem::transmute(
            KsDispatchInvalidDeviceRequest
                as *const (),
        ),
        Close: ::core::mem::transmute(
            IKsFilter_DispatchClose as *const (),
        ),
        QuerySecurity: ::core::mem::transmute(
            KsDispatchQuerySecurity as *const (),
        ),
        SetSecurity: ::core::mem::transmute(
            KsDispatchSetSecurity as *const (),
        ),
        FastDeviceIoControl: ::core::mem::transmute(
            KsDispatchFastIoDeviceControlFailure
                as *const (),
        ),
        FastRead: ::core::mem::transmute(
            KsDispatchFastReadFailure
                as *const (),
        ),
        FastWrite: ::core::mem::transmute(
            KsDispatchFastReadFailure
                as *const (),
        ),
    }
};

#[used]
#[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
static INIT_ARRAY: [unsafe extern "C" fn(); 1] = [run_static_initializers];

// c2rust .init_array constructor: installs the property-item handler slots (which cannot
// be const-initialized because the bindgen union storage is a u32) and fills the
// FilterPropertySet[] table (whose C initializer takes the address of other statics).
unsafe extern "C" fn run_static_initializers() {
    let topo: PFNKSHANDLER = crate::cb!(FilterTopologyPropertyHandler);
    let p = (&raw mut IKsFilterTopologySet) as *mut KSPROPERTY_ITEM;
    let mut i = 0usize;
    while i < 4 {
        *(*p.add(i)).__bindgen_anon_1.GetPropertyHandler.as_mut() = topo;
        i += 1;
    }

    let pin: PFNKSHANDLER = crate::cb!(FilterPinPropertyHandler);
    let q = (&raw mut IKsFilterPinSet) as *mut KSPROPERTY_ITEM;
    i = 0;
    while i < 11 {
        *(*q.add(i)).__bindgen_anon_1.GetPropertyHandler.as_mut() = pin;
        i += 1;
    }
    // PROPOSEDATAFORMAT (item 11) installs its handler in the Set slot.
    *(*q.add(11)).__bindgen_anon_2.SetPropertyHandler.as_mut() = pin;

    let g = (&raw mut IKsFilterGeneralSet) as *mut KSPROPERTY_ITEM;
    *(*g).__bindgen_anon_1.GetPropertyHandler.as_mut() = crate::cb!(FilterGeneralComponentIdHandler);

    let fps = (&raw mut FilterPropertySet) as *mut KSPROPERTY_SET;
    (*fps.add(0)).Set = &raw const KSPROPSETID_Topology;
    (*fps.add(0)).PropertiesCount = 4;
    (*fps.add(0)).PropertyItem = (&raw const IKsFilterTopologySet) as *const KSPROPERTY_ITEM;
    (*fps.add(1)).Set = &raw const KSPROPSETID_Pin;
    (*fps.add(1)).PropertiesCount = 12;
    (*fps.add(1)).PropertyItem = (&raw const IKsFilterPinSet) as *const KSPROPERTY_ITEM;
    (*fps.add(2)).Set = &raw const KSPROPSETID_General;
    (*fps.add(2)).PropertiesCount = 1;
    (*fps.add(2)).PropertyItem = (&raw const IKsFilterGeneralSet) as *const KSPROPERTY_ITEM;
}

// ---- KSGATE inline helpers (ks.h); used only by this module ----
#[inline]
unsafe fn ks_InterlockedCompareExchange(dest: *mut LONG, exch: LONG, comp: LONG) -> LONG {
    let a = &*(dest as *const core::sync::atomic::AtomicI32);
    match a.compare_exchange(
        comp,
        exch,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) | Err(v) => v,
    }
}
#[inline]
unsafe fn KsGateTurnInputOn(mut Gate: PKSGATE) {
    while !Gate.is_null() && InterlockedIncrement(&mut (*Gate).Count) == 1 {
        Gate = (*Gate).NextGate;
    }
}
#[inline]
unsafe fn KsGateTurnInputOff(mut Gate: PKSGATE) {
    while !Gate.is_null() && InterlockedDecrement(&mut (*Gate).Count) == 0 {
        Gate = (*Gate).NextGate;
    }
}
#[inline]
unsafe fn KsGateInitialize(
    Gate: PKSGATE,
    InitialCount: LONG,
    NextGate: PKSGATE,
    StateToPropagate: BOOLEAN,
) {
    (*Gate).Count = InitialCount;
    (*Gate).NextGate = NextGate;
    if !NextGate.is_null() {
        if InitialCount > 0 {
            if StateToPropagate != 0 {
                KsGateTurnInputOn(NextGate);
            }
        } else if StateToPropagate == 0 {
            KsGateTurnInputOff(NextGate);
        }
    }
}
#[inline]
unsafe fn KsGateInitializeAnd(AndGate: PKSGATE, NextOrGate: PKSGATE) {
    KsGateInitialize(AndGate, 1, NextOrGate, 1 as BOOLEAN);
}
#[inline]
unsafe fn KsGateCaptureThreshold(Gate: PKSGATE) -> BOOLEAN {
    let captured = (ks_InterlockedCompareExchange(&mut (*Gate).Count, 0, 1) == 1) as BOOLEAN;
    if captured != 0 {
        KsGateTurnInputOff((*Gate).NextGate);
    }
    captured
}

#[repr(C)]
pub struct IKsFilterImpl {
    pub Header: KSBASIC_HEADER,
    pub Filter: KSFILTER,
    pub lpVtblKsControl: *mut IKsControlVtbl,
    pub FilterFactory: *mut IKsFilterFactory,
    pub lpVtblKsProcessingObject: *mut IKsProcessingObjectVtbl,
    pub ref_: LONG,
    pub ObjectHeader: PKSIOBJECT_HEADER,
    pub Topology: KSTOPOLOGY,
    pub Factory: PKSFILTERFACTORY,
    pub FileObject: PFILE_OBJECT,
    pub ControlMutex: KMUTEX,
    pub ProcessingMutex: KMUTEX,
    pub Worker: PKSWORKER,
    pub WorkItem: WORK_QUEUE_ITEM,
    pub Gate: KSGATE,
    pub Sleep: PFNKSFILTERPOWER,
    pub Wake: PFNKSFILTERPOWER,
    pub PinInstanceCount: *mut ULONG,
    pub FirstPin: *mut PKSPIN,
    pub ProcessPinIndex: PKSPROCESSPIN_INDEXENTRY,
}

pub type PIKsFilterImpl = *mut IKsFilterImpl;

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnQueryInterface(
    iface: *mut IKsProcessingObject,
    refiid: *const IID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(
        iface as *mut u8,
        IKsFilterImpl,
        lpVtblKsProcessingObject,
    ) as *mut IKsFilterImpl;

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) != 0 {
        *Output = &mut (*This).Header.OuterUnknown as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref_);
        return STATUS_SUCCESS;
    }
    STATUS_UNSUCCESSFUL
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnAddRef(
    iface: *mut IKsProcessingObject,
) -> ULONG {
    let This = CONTAINING_RECORD!(
        iface,
        IKsFilterImpl,
        lpVtblKsProcessingObject,
    ) as *mut IKsFilterImpl;

    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnRelease(
    iface: *mut IKsProcessingObject,
) -> ULONG {
    let This = CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
    InterlockedDecrement(&mut (*This).ref_);
    (*This).ref_ as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnProcessingObjectWork(
    iface: *mut IKsProcessingObject,
) {
    let mut Status: NTSTATUS;
    let mut TimeOut: LARGE_INTEGER = core::mem::zeroed();
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(
        iface as *mut u8,
        IKsFilterImpl,
        lpVtblKsProcessingObject,
    ) as *mut IKsFilterImpl;

    if KeGetCurrentIrql() == PASSIVE_LEVEL {
        KeWaitForSingleObject(
            &mut (*This).ProcessingMutex as *mut _ as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            core::ptr::null_mut(),
        );
    } else {
        if KeReadStateMutex(&mut (*This).ProcessingMutex) == 0 {
            return;
        }

        TimeOut.QuadPart = 0;
        Status = KeWaitForSingleObject(
            &mut (*This).ProcessingMutex as *mut _ as *mut core::ffi::c_void,
            _KWAIT_REASON_Executive,
            0i8,
            FALSE,
            &mut TimeOut,
        );

        if Status == STATUS_TIMEOUT {
            return;
        }
    }

    loop {
        if (*This).Gate.Count != 0 {
            break;
        }

        break;
    }

    KeReleaseMutex(&mut (*This).ProcessingMutex, FALSE);
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnGetAndGate(
    iface: *mut IKsProcessingObject,
) -> PKSGATE {
    let This = CONTAINING_RECORD!(
        iface,
        IKsFilterImpl,
        lpVtblKsProcessingObject,
    ) as *mut IKsFilterImpl;
    return core::ptr::addr_of_mut!((*This).Gate);
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnProcess(
    iface: *mut IKsProcessingObject,
    Asynchronous: BOOLEAN,
) {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);

    if Asynchronous != 0 {
        KsQueueWorkItem((*This).Worker, &mut (*This).WorkItem);
        return;
    }

    if ((*(*This).Filter.Descriptor).Flags
        & (KSFILTER_FLAG_DISPATCH_LEVEL_PROCESSING
            | KSFILTER_FLAG_CRITICAL_PROCESSING
            | KSFILTER_FLAG_HYPERCRITICAL_PROCESSING))
        != 0
        && KeGetCurrentIrql() > PASSIVE_LEVEL
    {
        KsQueueWorkItem((*This).Worker, &mut (*This).WorkItem);
        return;
    }

    ((*(*iface).lpVtbl).ProcessingObjectWork.unwrap())(iface);
}

#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnReset(
    iface: *mut IKsProcessingObject,
) {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(
        iface as *mut u8,
        IKsFilterImpl,
        lpVtblKsProcessingObject,
    ) as *mut IKsFilterImpl;

    KeWaitForSingleObject(
        &mut (*This).ProcessingMutex as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    if !(*((*This).Filter.Descriptor)).Dispatch.is_null() {
        if let Some(reset_fn) = (*(*((*This).Filter.Descriptor)).Dispatch).Reset {
            reset_fn(&mut (*This).Filter);
        }
    }

    KeReleaseMutex(&mut (*This).ProcessingMutex, 0u8);
}
#[no_mangle]
pub unsafe extern "system" fn IKsProcessingObject_fnTriggerNotification(
    iface: *mut IKsProcessingObject,
) {
}

#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnQueryInterface(
    iface: *mut IKsControl,
    refiid: *const IID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) != 0 {
        *Output = &mut (*This).Header.OuterUnknown as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref_);
        return STATUS_SUCCESS;
    }
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnAddRef(iface: *mut IKsControl) -> ULONG {
    let This = CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl) as *mut IKsFilterImpl;
    InterlockedIncrement(&mut (*This).ref_ as *mut LONG) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnRelease(
    iface: *mut IKsControl,
) -> ULONG {
    let This = CONTAINING_RECORD!(
        iface,
        IKsFilterImpl,
        lpVtblKsControl,
    ) as *mut IKsFilterImpl;

    InterlockedDecrement(&mut (*This).ref_);

    (*This).ref_ as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnKsProperty(
    iface: *mut IKsControl,
    Property: PKSPROPERTY,
    PropertyLength: ULONG,
    PropertyData: PVOID,
    DataLength: ULONG,
    BytesReturned: *mut ULONG,
) -> NTSTATUS {
    let This = core::ptr::with_exposed_provenance_mut::<IKsFilterImpl>((iface as usize)
            - core::mem::offset_of!(IKsFilterImpl, lpVtblKsControl),
    );

    KsSynchronousIoControlDevice(
        (*This).FileObject,
        0i8,
        IOCTL_KS_PROPERTY,
        Property as PVOID,
        PropertyLength,
        PropertyData,
        DataLength,
        BytesReturned,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnKsMethod(
    iface: *mut IKsControl,
    Method: PKSMETHOD,
    MethodLength: ULONG,
    MethodData: PVOID,
    DataLength: ULONG,
    BytesReturned: *mut ULONG,
) -> NTSTATUS {
    let This = core::ptr::with_exposed_provenance_mut::<IKsFilterImpl>((iface as usize)
            .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, lpVtblKsControl)),
    );
    KsSynchronousIoControlDevice(
        (*This).FileObject,
        0i8,
        IOCTL_KS_METHOD,
        Method as PVOID,
        MethodLength,
        MethodData,
        DataLength,
        BytesReturned,
    )
}

#[no_mangle]
pub unsafe extern "system" fn IKsControl_fnKsEvent(
    iface: *mut IKsControl,
    Event: *mut KSEVENT,
    EventLength: ULONG,
    EventData: *mut core::ffi::c_void,
    DataLength: ULONG,
    BytesReturned: *mut ULONG,
) -> NTSTATUS {
    let This = (iface as *mut u8).sub(
        core::mem::offset_of!(IKsFilterImpl, lpVtblKsControl)
    ) as *mut IKsFilterImpl;

    if !Event.is_null() {
        return KsSynchronousIoControlDevice(
            (*This).FileObject,
            0i8,
            IOCTL_KS_ENABLE_EVENT,
            Event as *mut core::ffi::c_void,
            EventLength,
            EventData,
            DataLength,
            BytesReturned,
        );
    } else {
        return KsSynchronousIoControlDevice(
            (*This).FileObject,
            0i8,
            IOCTL_KS_DISABLE_EVENT,
            EventData,
            DataLength,
            core::ptr::null_mut(),
            0,
            BytesReturned,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnQueryInterface(
    iface: *mut IKsFilter,
    refiid: *const IID,
    Output: *mut PVOID,
) -> NTSTATUS {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);

    if IsEqualGUIDAligned(refiid, &IID_IUnknown) != 0 ||
       IsEqualGUIDAligned(refiid, &raw const IID_IKsFilter) != 0
    {
        *Output = &mut (*This).Header.OuterUnknown as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref_);
        return STATUS_SUCCESS;
    } else if IsEqualGUIDAligned(refiid, &raw const IID_IKsControl) != 0 {
        *Output = &mut (*This).lpVtblKsControl as *mut _ as PVOID;
        _InterlockedIncrement(&mut (*This).ref_);
        return STATUS_SUCCESS;
    }

    if !(*This).Header.ClientAggregate.is_null() {
        let Status = (*(*(*This).Header.ClientAggregate).lpVtbl).QueryInterface.expect("non-null function pointer")(
            (*This).Header.ClientAggregate as *mut IUnknown,
            refiid,
            Output,
        );
        if NT_SUCCESS(Status) {
            return Status;
        }
    }

    return STATUS_NOT_SUPPORTED;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnAddRef(iface: *mut IKsFilter) -> ULONG {
    let This = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnRelease(iface: *mut IKsFilter) -> ULONG {
    let This = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown) as *mut IKsFilterImpl;

    InterlockedDecrement(&mut (*This).ref_);

    if (*This).ref_ == 0 {
        FreeItem(This as PVOID);
        return 0;
    }

    (*This).ref_ as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnGetStruct(iface: *mut IKsFilter) -> PKSFILTER {
    let This = ((iface as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Header.OuterUnknown)))
        as *mut IKsFilterImpl;
    &mut (*This).Filter
}
#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnDoAllNecessaryPinsExist(
    iface: *mut IKsFilter,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsFilter_fnDoAllNecessaryPinsExist\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/filter.c\0"
            .as_ptr() as *const i8,
        469i32,
    );
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnCreateNode(
    iface: *mut IKsFilter,
    Irp: PIRP,
    Pin: *mut IKsPin,
    ListEntry: PLIST_ENTRY,
) -> NTSTATUS {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"IKsFilter_fnCreateNode\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/filter.c\0".as_ptr() as *const ::core::ffi::c_char,
        481i32,
    );
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnBindProcessPinsToPipeSection(
    iface: *mut IKsFilter,
    Section: *mut KSPROCESSPIPESECTION,
    Create: PVOID,
    KsPin: PKSPIN,
    Pin: *mut *mut IKsPin,
    OutGate: *mut PKSGATE,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnUnbindProcessPinsFromPipeSection(
    iface: *mut IKsFilter,
    Section: *mut KSPROCESSPIPESECTION,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnAddProcessPin(
    iface: *mut IKsFilter,
    ProcessPin: PKSPROCESSPIN,
) -> NTSTATUS {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);

    KeWaitForSingleObject(
        &mut (*This).ProcessingMutex as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        FALSE,
        core::ptr::null_mut(),
    );

    let Status = _KsEdit(
        (*This).Filter.Bag,
        &mut (*((*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize))).Pins as *mut _ as *mut PVOID,
        (((*((*This).Filter.Descriptor)).PinDescriptorsCount as ULONG).wrapping_add(1) as usize)
            .wrapping_mul(core::mem::size_of::<PVOID>()) as ULONG,
        ((*((*This).Filter.Descriptor)).PinDescriptorsCount as usize)
            .wrapping_mul(core::mem::size_of::<PVOID>()) as ULONG,
        0,
    );

    if NT_SUCCESS(Status) {
        let idx = (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count as isize;
        *((*((*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize))).Pins.offset(idx)) = ProcessPin;
        (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count =
            (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count.wrapping_add(1);
    }

    KeReleaseMutex(&mut (*This).ProcessingMutex as *mut _, FALSE);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnRemoveProcessPin(
    iface: *mut IKsFilter,
    ProcessPin: PKSPROCESSPIN,
) -> NTSTATUS {
    let mut Index: ULONG;
    let mut Count: ULONG;
    let mut Pins: *mut PKSPROCESSPIN;

    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);

    KeWaitForSingleObject(
        &mut (*This).ProcessingMutex as *mut _ as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        FALSE,
        core::ptr::null_mut(),
    );

    Count = (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count;
    Pins = (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Pins;

    Index = 0;
    while Index < Count {
        if *Pins.offset(Index as isize) == ProcessPin {
            core::ptr::copy(
                Pins.offset((Index + 1) as isize),
                Pins.offset(Index as isize),
                (Count - (Index + 1)) as usize,
            );
            break;
        }
        Index += 1;
    }

    (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count -= 1;

    if (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Count == 0 {
        (*(*This).ProcessPinIndex.offset((*(*ProcessPin).Pin).Id as isize)).Pins = core::ptr::null_mut();
    }

    KeReleaseMutex(&mut (*This).ProcessingMutex as *mut _ as PKMUTEX, FALSE);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnReprepareProcessPipeSection(
    iface: *mut IKsFilter,
    PipeSection: *mut KSPROCESSPIPESECTION,
    Data: PULONG,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsFilter_fnReprepareProcessPipeSection\0".as_ptr() as PCHAR,
        b"drivers/ksfilter/ks/filter.c\0".as_ptr() as PCHAR,
        599i32,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnDeliverResetState(
    iface: *mut IKsFilter,
    PipeSection: *mut KSPROCESSPIPESECTION,
    ResetState: KSRESET,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsFilter_fnDeliverResetState\0".as_ptr() as PCHAR,
        b"drivers/ksfilter/ks/filter.c\0".as_ptr() as PCHAR,
        610i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnIsFrameHolding(
    iface: *mut IKsFilter,
) -> BOOL {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCHAR,
        b"IKsFilter_fnIsFrameHolding\0".as_ptr() as *const i8,
        b"drivers/ksfilter/ks/filter.c\0"
            .as_ptr() as *const i8,
        618i32,
    );
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnRegisterForCopyCallbacks(
    iface: *mut IKsFilter,
    Queue: *mut IKsQueue,
    Register: BOOL,
) {
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"IKsFilter_fnRegisterForCopyCallbacks\0".as_ptr() as *const ::core::ffi::c_char,
        b"drivers/ksfilter/ks/filter.c\0"
            .as_ptr() as *const ::core::ffi::c_char,
        629i32,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_fnGetProcessDispatch(
    iface: *mut IKsFilter,
) -> PKSPROCESSPIN_INDEXENTRY {
    let this = CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
    (*this).ProcessPinIndex
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_GetFilterFromIrp(
    Irp: PIRP,
    Filter: *mut *mut IKsFilter,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let ObjectHeader: PKSIOBJECT_HEADER;
    let Status: NTSTATUS;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*(*IoStack).FileObject).FsContext2 as PKSIOBJECT_HEADER;

    Status = (*(*(*ObjectHeader).Unknown).lpVtbl).QueryInterface.expect("non-null function pointer")(
        (*ObjectHeader).Unknown as *mut IUnknown,
        &raw const IID_IKsFilter,
        Filter as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
        return Status;
    }
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_DispatchClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut Filter: *mut IKsFilter = core::ptr::null_mut();
    let mut This: *mut IKsFilterImpl = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    Status = IKsFilter_GetFilterFromIrp(Irp, &mut Filter);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    This = CONTAINING_RECORD!(Filter, IKsFilterImpl, Header.OuterUnknown);

    if !(*This).Filter.Descriptor.is_null()
        && !(*(*This).Filter.Descriptor).Dispatch.is_null()
        && (*(*(*This).Filter.Descriptor).Dispatch).Close.is_some()
    {
        Status = (*(*(*This).Filter.Descriptor).Dispatch)
            .Close
            .expect("non-null function pointer")(&mut (*This).Filter, Irp);
    }

    if NT_SUCCESS(Status) && Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as i8);

        IKsFilter_RemoveFilterFromFilterFactory(This, (*This).Factory);
        KsFreeObjectHeader((*This).ObjectHeader as *mut _);
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn KspHandlePropertyInstances(
    IoStatus: PIO_STATUS_BLOCK,
    Request: PKSIDENTIFIER,
    Data: PVOID,
    This: *mut IKsFilterImpl,
    _Global: BOOL,
) -> NTSTATUS {
    let Pin = Request as *mut KSP_PIN;

    if (*This).Filter.Descriptor.is_null() || (*(*This).Filter.Descriptor).PinDescriptorsCount == 0 {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        return STATUS_NOT_IMPLEMENTED;
    }

    let Instances = Data as *mut KSPIN_CINSTANCES;
    (*Instances).PossibleCount = (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).InstancesPossible;
    (*Instances).CurrentCount = *(*This).PinInstanceCount.offset((*Pin).PinId as isize);

    (*IoStatus).Information = core::mem::size_of::<KSPIN_CINSTANCES>() as _;
    (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspHandleNecessaryPropertyInstances(
    IoStatus: PIO_STATUS_BLOCK,
    Request: PKSIDENTIFIER,
    Data: PVOID,
    This: *mut IKsFilterImpl,
) -> NTSTATUS {
    let Pin = Request as *mut KSP_PIN;

    if (*This).Filter.Descriptor.is_null() || (*(*This).Filter.Descriptor).PinDescriptorsCount == 0 {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        return STATUS_NOT_IMPLEMENTED;
    }

    let Result = Data as PULONG;
    *Result = (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).InstancesNecessary;

    (*IoStatus).Information = core::mem::size_of::<ULONG>() as _;
    (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspHandleDataIntersection(
    Irp: PIRP,
    IoStatus: PIO_STATUS_BLOCK,
    Request: PKSIDENTIFIER,
    Data: PVOID,
    DataLength: ULONG,
    This: *mut IKsFilterImpl,
) -> NTSTATUS {
    let MultipleItem: PKSMULTIPLE_ITEM;
    let mut DataRange: PKSDATARANGE;
    let mut Status: NTSTATUS = STATUS_NO_MATCH;
    let mut Index: ULONG;
    let mut Length: ULONG = 0;
    let IoStack: PIO_STACK_LOCATION;
    let Pin: *mut KSP_PIN = Request as *mut KSP_PIN;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    MultipleItem = Pin.offset(1) as PKSMULTIPLE_ITEM;
    DataRange = MultipleItem.offset(1) as PKSDATARANGE;

    if (*This).Filter.Descriptor.is_null() || (*(*This).Filter.Descriptor).PinDescriptorsCount == 0
    {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        return STATUS_NOT_IMPLEMENTED;
    }

    if (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).IntersectHandler.is_none()
        || (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).PinDescriptor.DataRanges.is_null()
        || (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).PinDescriptor.DataRangesCount == 0
    {
        (*IoStatus).__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        return STATUS_NOT_IMPLEMENTED;
    }

    Index = 0;
    while Index < (*MultipleItem).Count {
        let mut MajorFormat: _UNICODE_STRING = core::mem::zeroed();
        let mut SubFormat: _UNICODE_STRING = core::mem::zeroed();
        let mut Specifier: _UNICODE_STRING = core::mem::zeroed();

        RtlStringFromGUID(&mut (*DataRange).__bindgen_anon_1.as_mut().MajorFormat, &mut MajorFormat);
        RtlStringFromGUID(&mut (*DataRange).__bindgen_anon_1.as_mut().SubFormat, &mut SubFormat);
        RtlStringFromGUID(&mut (*DataRange).__bindgen_anon_1.as_mut().Specifier, &mut Specifier);

        Status = (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).IntersectHandler.expect("non-null function pointer")(
            &mut (*This).Filter as *mut _ as PVOID,
            Irp,
            Pin as PKSP_PIN,
            DataRange,
            *(*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Pin).PinId as isize)).PinDescriptor.DataRanges.offset(0),
            DataLength,
            Data,
            &mut Length,
        );

        if Status == STATUS_SUCCESS || Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL {
            (*IoStatus).Information = Length as _;
            break;
        }

        DataRange = (DataRange as PUCHAR).offset((*DataRange).__bindgen_anon_1.as_ref().FormatSize as isize) as PKSDATARANGE;
        Index = Index.wrapping_add(1);
    }

    (*IoStatus).__bindgen_anon_1.Status = Status;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn FilterTopologyPropertyHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: *mut core::ffi::c_void,
) -> NTSTATUS {
    let This: *mut IKsFilterImpl = *((*Irp)
        .Tail
        .Overlay
        .__bindgen_anon_1
        .__bindgen_anon_1.DriverContext
        .as_mut_ptr()
        .offset(3isize) as *mut *mut IKsFilterImpl);

    return KsTopologyPropertyHandler(
        Irp,
        Request as PKSPROPERTY,
        Data,
        core::ptr::addr_of_mut!((*This).Topology),
    );
}

#[no_mangle]
pub unsafe extern "system" fn FilterGeneralComponentIdHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION;
    let This: *mut IKsFilterImpl;

    This = *((&raw mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext as *mut PVOID).offset(3) as *mut PVOID as *mut *const KSPROPERTY_ITEM) as *mut IKsFilterImpl;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*This).Filter.Descriptor.as_ref().unwrap().ComponentId != core::ptr::null_mut() {
        core::ptr::copy_nonoverlapping(
            (*This).Filter.Descriptor.as_ref().unwrap().ComponentId as *const u8,
            Data as *mut u8,
            core::mem::size_of::<KSCOMPONENTID>(),
        );
        (*Irp).IoStatus.Information = core::mem::size_of::<KSCOMPONENTID>() as _;
        return STATUS_SUCCESS;
    } else {
        return STATUS_NOT_FOUND;
    }
}

#[no_mangle]
pub unsafe extern "system" fn FilterPinPropertyHandler(
    Irp: PIRP,
    Request: PKSIDENTIFIER,
    Data: PVOID,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut This: *mut IKsFilterImpl = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    This = *((&raw mut (*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext as *mut PVOID)
        .offset(3isize) as *mut PVOID as *mut *const KSPROPERTY_ITEM) as *mut IKsFilterImpl;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    match (*Request).Id {
        1 | 2 | 3 | 5 | 6 | 7 | 11 | 12 | 13 => {
            Status = KspPinPropertyHandler(
                Irp,
                Request as PKSPROPERTY,
                Data,
                (*(*This).Filter.Descriptor).PinDescriptorsCount,
                (*(*This).Filter.Descriptor).PinDescriptors as *const KSPIN_DESCRIPTOR,
                (*(*This).Filter.Descriptor).PinDescriptorSize,
            );
        }
        8 => {
            Status = KspHandlePropertyInstances(&raw mut (*Irp).IoStatus, Request, Data, This, 1i32);
        }
        0 => {
            Status = KspHandlePropertyInstances(&raw mut (*Irp).IoStatus, Request, Data, This, 0i32);
        }
        9 => {
            Status = KspHandleNecessaryPropertyInstances(&raw mut (*Irp).IoStatus, Request, Data, This);
        }
        4 => {
            Status = KspHandleDataIntersection(
                Irp,
                &raw mut (*Irp).IoStatus,
                Request,
                Data,
                (*IoStack).Parameters.DeviceIoControl.OutputBufferLength,
                This,
            );
        }
        _ => {
            Status = STATUS_NOT_FOUND;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_DispatchDeviceIoControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut Filter: *mut IKsFilter = core::ptr::null_mut();
    let mut This: *mut IKsFilterImpl = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut FilterInstance: PKSFILTER = core::ptr::null_mut();
    let mut GuidString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Property: PKSPROPERTY = core::ptr::null_mut();
    let mut SetCount: ULONG = 0;
    let mut NodeProperty: PKSP_NODE = core::ptr::null_mut();
    let mut NodeDescriptor: PKSNODE_DESCRIPTOR = core::ptr::null_mut();

    Status = IKsFilter_GetFilterFromIrp(Irp, &mut Filter);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    This = core::ptr::with_exposed_provenance_mut::<IKsFilterImpl>((Filter as usize).wrapping_sub(
            core::mem::offset_of!(IKsFilterImpl, Header.OuterUnknown),
        ),
    );

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    Property = (*IoStack).Parameters.DeviceIoControl.Type3InputBuffer as PKSPROPERTY;
    FilterInstance = (*(*Filter).lpVtbl).GetStruct.unwrap()(Filter);

    KeWaitForSingleObject(
        (*This).Header.ControlMutex as PVOID,
        _KWAIT_REASON_Executive,
        0i32 as KPROCESSOR_MODE,
        0u8,
        core::ptr::null_mut(),
    );

    if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_METHOD {
        let mut MethodSet: *const KSMETHOD_SET = core::ptr::null();
        let mut MethodItemSize: ULONG = 0;

        if (*(*(*FilterInstance).Descriptor).AutomationTable).MethodSetsCount != 0 {
            SetCount = (*(*(*FilterInstance).Descriptor).AutomationTable).MethodSetsCount;
            MethodSet = (*(*(*FilterInstance).Descriptor).AutomationTable).MethodSets;
            MethodItemSize = (*(*(*FilterInstance).Descriptor).AutomationTable).MethodItemSize;
        }

        Status = KspMethodHandlerWithAllocator(Irp, SetCount, MethodSet, None, MethodItemSize);
    } else if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_PROPERTY {
        let mut PropertySet: *const KSPROPERTY_SET = core::ptr::null();
        let mut PropertyItemSize: ULONG = 0;

        if (*Property).Flags & KSPROPERTY_TYPE_TOPOLOGY != 0 {
            NodeProperty = Property as PKSP_NODE;
            NodeDescriptor = (*(*FilterInstance).Descriptor).NodeDescriptors
                .offset((*(*FilterInstance).Descriptor).NodeDescriptorSize as isize * (*NodeProperty).NodeId as isize)
                as PKSNODE_DESCRIPTOR;

            if !(*NodeDescriptor).AutomationTable.is_null() {
                SetCount = (*(*NodeDescriptor).AutomationTable).PropertySetsCount;
                PropertySet = (*(*NodeDescriptor).AutomationTable).PropertySets;
                PropertyItemSize = 0;
            }
        } else if (*(*(*FilterInstance).Descriptor).AutomationTable).PropertySetsCount != 0 {
            SetCount = (*(*(*FilterInstance).Descriptor).AutomationTable).PropertySetsCount;
            PropertySet = (*(*(*FilterInstance).Descriptor).AutomationTable).PropertySets;
            PropertyItemSize = (*(*(*FilterInstance).Descriptor).AutomationTable).PropertyItemSize;
            PropertyItemSize = 0;
        }

        *((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext.as_mut_ptr().offset(3) as *mut *const KSPROPERTY_ITEM) = This as *mut KSPROPERTY_ITEM;

        Status = KspPropertyHandler(Irp, SetCount, PropertySet, None, PropertyItemSize);
    } else {
        if (*IoStack).Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_ENABLE_EVENT {
            Status = KspEnableEvent(
                Irp,
                (*(*(*FilterInstance).Descriptor).AutomationTable).EventSetsCount,
                (*(*(*FilterInstance).Descriptor).AutomationTable).EventSets as PKSEVENT_SET,
                &raw mut (*This).Header.EventList,
                KSEVENTS_SPINLOCK,
                &raw mut (*This).Header.EventListLock as PVOID,
                None,
                (*(*(*FilterInstance).Descriptor).AutomationTable).EventItemSize,
            );
        } else {
            Status = KsDisableEvent(
                Irp,
                &raw mut (*This).Header.EventList,
                KSEVENTS_SPINLOCK,
                &raw mut (*This).Header.EventListLock as PVOID,
            );
        }
    }

    RtlStringFromGUID(&raw mut (*Property).Set, &raw mut GuidString);
    RtlFreeUnicodeString(&raw mut GuidString);

    (*(*Filter).lpVtbl).Release.unwrap()(Filter);
    KeReleaseMutex((*This).Header.ControlMutex, 0u8);

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_CreateDescriptors(
    This: *mut IKsFilterImpl,
    FilterDescriptor: *mut KSFILTER_DESCRIPTOR,
) -> NTSTATUS {
    let mut Index: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut NodeDescriptor: PKSNODE_DESCRIPTOR;

    (*This).FirstPin = core::ptr::null_mut();
    (*This).PinInstanceCount = core::ptr::null_mut();
    (*This).ProcessPinIndex = core::ptr::null_mut();

    (*This).Topology.CategoriesCount = (*FilterDescriptor).CategoriesCount;
    (*This).Topology.Categories = (*FilterDescriptor).Categories;
    (*This).Topology.TopologyNodesCount = (*FilterDescriptor).NodeDescriptorsCount;
    (*This).Topology.TopologyConnectionsCount = (*FilterDescriptor).ConnectionsCount;
    (*This).Topology.TopologyConnections = (*FilterDescriptor).Connections;

    if (*FilterDescriptor).PinDescriptorsCount != 0 {
        Status = _KsEdit(
            (*This).Filter.Bag,
            &raw const (*(*This).Filter.Descriptor).PinDescriptors as *mut PVOID,
            (*FilterDescriptor).PinDescriptorSize.wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            (*FilterDescriptor).PinDescriptorSize.wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            0,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = _KsEdit(
            (*This).Filter.Bag,
            &raw mut (*This).PinInstanceCount as *mut PVOID,
            (core::mem::size_of::<ULONG>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            (core::mem::size_of::<ULONG>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            0,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = _KsEdit(
            (*This).Filter.Bag,
            &raw mut (*This).FirstPin as *mut PVOID,
            (core::mem::size_of::<PVOID>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            (core::mem::size_of::<PVOID>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            0,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        core::ptr::copy_nonoverlapping(
            (*FilterDescriptor).PinDescriptors,
            (*(*This).Filter.Descriptor).PinDescriptors as *mut _,
            ((*FilterDescriptor).PinDescriptorSize.wrapping_mul((*FilterDescriptor).PinDescriptorsCount)) as usize,
        );

        Status = _KsEdit(
            (*This).Filter.Bag,
            &raw mut (*This).ProcessPinIndex as *mut PVOID,
            (core::mem::size_of::<KSPROCESSPIN_INDEXENTRY>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            (core::mem::size_of::<KSPROCESSPIN_INDEXENTRY>() as ULONG).wrapping_mul((*FilterDescriptor).PinDescriptorsCount),
            0,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if (*FilterDescriptor).ConnectionsCount != 0 {
        Status = _KsEdit(
            (*This).Filter.Bag,
            &raw const (*(*This).Filter.Descriptor).Connections as *mut PVOID,
            ((*FilterDescriptor).ConnectionsCount as usize).wrapping_mul(core::mem::size_of::<KSTOPOLOGY_CONNECTION>()) as ULONG,
            ((*FilterDescriptor).ConnectionsCount as usize).wrapping_mul(core::mem::size_of::<KSTOPOLOGY_CONNECTION>()) as ULONG,
            0,
        );

        (*This).Topology.TopologyConnections = (*(*This).Filter.Descriptor).Connections;
        (*This).Topology.TopologyConnectionsCount = (*FilterDescriptor).ConnectionsCount;
        (*((*This).Filter.Descriptor as PKSFILTER_DESCRIPTOR)).ConnectionsCount = (*FilterDescriptor).ConnectionsCount;
    }

    if (*FilterDescriptor).NodeDescriptorsCount != 0 {
        (*This).Topology.TopologyNodes = AllocateItem(
            NonPagedPool,
            (core::mem::size_of::<GUID>() as SIZE_T).wrapping_mul((*FilterDescriptor).NodeDescriptorsCount as SIZE_T),
        ) as *const GUID;

        if (*This).Topology.TopologyNodes.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*This).Topology.TopologyNodesNames = AllocateItem(
            NonPagedPool,
            (core::mem::size_of::<GUID>() as SIZE_T).wrapping_mul((*FilterDescriptor).NodeDescriptorsCount as SIZE_T),
        ) as *const GUID;

        if (*This).Topology.TopologyNodesNames.is_null() {
            FreeItem((*This).Topology.TopologyNodes as PVOID);
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        NodeDescriptor = (*FilterDescriptor).NodeDescriptors as PKSNODE_DESCRIPTOR;
        Index = 0;
        while Index < (*FilterDescriptor).NodeDescriptorsCount {
            if !(*NodeDescriptor).Type.is_null() {
                core::ptr::copy_nonoverlapping(
                    (*NodeDescriptor).Type,
                    (*This).Topology.TopologyNodes.add(Index as usize) as *mut GUID,
                    1, // one GUID — size_of here copied 16 GUIDs
                );
            }

            if !(*NodeDescriptor).Name.is_null() {
                core::ptr::copy_nonoverlapping(
                    (*NodeDescriptor).Name,
                    (*This).Topology.TopologyNodesNames.add(Index as usize) as *mut GUID,
                    1, // one GUID — size_of here copied 16 GUIDs
                );
            }

            NodeDescriptor = (NodeDescriptor as usize).wrapping_add((*FilterDescriptor).NodeDescriptorSize as usize) as PKSNODE_DESCRIPTOR;
            Index = Index.wrapping_add(1);
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_CopyFilterDescriptor(
    This: *mut IKsFilterImpl,
    FilterDescriptor: *const KSFILTER_DESCRIPTOR,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut AutomationTable: KSAUTOMATION_TABLE = core::mem::zeroed();

    (*This).Filter.Descriptor = AllocateItem(
        NonPagedPool,
        core::mem::size_of::<KSFILTER_DESCRIPTOR>(),
    ) as *const KSFILTER_DESCRIPTOR;
    if (*This).Filter.Descriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Status = KsAddItemToObjectBag(
        (*This).Filter.Bag,
        (*This).Filter.Descriptor as PVOID,
        None,
    );
    if !NT_SUCCESS(Status) {
        FreeItem((*This).Filter.Descriptor as PVOID);
        (*This).Filter.Descriptor = core::ptr::null();
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(
        FilterDescriptor as *const u8,
        (*This).Filter.Descriptor as *mut u8,
        core::mem::size_of::<KSFILTER_DESCRIPTOR>(),
    );

    core::ptr::write_bytes(
        &mut AutomationTable as *mut KSAUTOMATION_TABLE,
        0,
        1,
    );

    AutomationTable.PropertyItemSize = core::mem::size_of::<KSPROPERTY_ITEM>() as ULONG;
    AutomationTable.PropertySetsCount = 3;
    AutomationTable.PropertySets = &raw const FilterPropertySet as *const KSPROPERTY_SET;

    Status = KsMergeAutomationTables(
        &raw const (*((*This).Filter.Descriptor)).AutomationTable as *mut PKSAUTOMATION_TABLE,
        (*FilterDescriptor).AutomationTable as PKSAUTOMATION_TABLE,
        &mut AutomationTable,
        (*This).Filter.Bag,
    );

    return Status;
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_AddPin(
    Filter: PKSFILTER,
    Pin: PKSPIN,
) {
    let mut NextPin: PKSPIN = core::ptr::null_mut();
    let mut CurPin: PKSPIN = core::ptr::null_mut();
    let mut BasicHeader: PKSBASIC_HEADER = core::ptr::null_mut();
    let This: *mut IKsFilterImpl = (Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter))
        as *mut IKsFilterImpl;

    if !((*Pin).Id < (*(*This).Filter.Descriptor).PinDescriptorsCount) {
        RtlAssert(
            b"Pin->Id < This->Filter.Descriptor->PinDescriptorsCount\0".as_ptr() as PVOID,
            b"drivers/ksfilter/ks/filter.c\0".as_ptr() as PVOID,
            1346u32,
            core::ptr::null_mut(),
        );
    }

    if (*(*This).FirstPin.offset((*Pin).Id as isize)).is_null() {
        *(*This).FirstPin.offset((*Pin).Id as isize) = Pin;
        *(*This).PinInstanceCount.offset((*Pin).Id as isize) = (*(*This).PinInstanceCount.offset((*Pin).Id as isize)).wrapping_add(1);
        return;
    }

    CurPin = *(*This).FirstPin.offset((*Pin).Id as isize);

    // Walk to the last sibling. Divergence from filter.c, whose loop body says
    // `NextPin = CurPin;`: it never advances, so a filter with two instances of
    // one pin spun forever adding a third (LLVM proved no progress and emitted a
    // bare `b .`, which scan_ud2 reported). Advance the cursor instead.
    loop {
        NextPin = KsPinGetNextSiblingPin(CurPin);
        if NextPin.is_null() {
            break;
        }
        CurPin = NextPin;
    }

    BasicHeader = (CurPin as usize).wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>()) as PKSBASIC_HEADER;
    (*BasicHeader).Next.Pin = Pin;
    // Also a divergence: filter.c counts only the FIRST instance of a pin, while
    // IKsFilter_RemovePin decrements for every one, so the count underflowed.
    *(*This).PinInstanceCount.offset((*Pin).Id as isize) =
        (*(*This).PinInstanceCount.offset((*Pin).Id as isize)).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_RemovePin(
    Filter: PKSFILTER,
    Pin: PKSPIN,
) {
    let mut NextPin: PKSPIN = core::ptr::null_mut();
    let mut CurPin: PKSPIN = core::ptr::null_mut();
    let mut LastPin: PKSPIN = core::ptr::null_mut();
    let mut BasicHeader: PKSBASIC_HEADER = core::ptr::null_mut();
    let This: *mut IKsFilterImpl = (Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter))
        as *mut IKsFilterImpl;

    CurPin = *(*This).FirstPin.offset((*Pin).Id as isize);
    LastPin = core::ptr::null_mut();
    loop {
        NextPin = KsPinGetNextSiblingPin(CurPin);
        if CurPin == Pin {
            if !LastPin.is_null() {
                BasicHeader = (LastPin as usize)
                    .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
                    as PKSBASIC_HEADER;
                (*BasicHeader).Next.Pin = NextPin;
            } else {
                *(*This).FirstPin.offset((*Pin).Id as isize) = NextPin;
            }
            let count = &mut *(*This).PinInstanceCount.offset((*Pin).Id as isize);
            *count = count.wrapping_sub(1);
            return;
        }
        if NextPin.is_null() {
            break;
        }
        LastPin = CurPin;
        // filter.c writes `NextPin = CurPin;` here, the same non-advancing walk as
        // IKsFilter_AddPin's; removing any pin but the first spun forever.
        CurPin = NextPin;
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_DispatchCreatePin(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut This: *mut IKsFilterImpl;
    let mut CreateItem: PKSOBJECT_CREATE_ITEM;
    let mut Connect: PKSPIN_CONNECT = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    CreateItem = *((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext.as_mut_ptr() as *mut PKSOBJECT_CREATE_ITEM);
    This = (*CreateItem).Context as *mut IKsFilterImpl;

    KeWaitForSingleObject(
        (*This).Header.ControlMutex as PVOID,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );

    Status = KspValidateConnectRequest(
        Irp,
        (*(*This).Filter.Descriptor).PinDescriptorsCount,
        (*(*This).Filter.Descriptor).PinDescriptors as PVOID,
        (*(*This).Filter.Descriptor).PinDescriptorSize,
        &mut Connect as *mut PKSPIN_CONNECT,
    );

    if NT_SUCCESS(Status) {
        if (*(*This).PinInstanceCount.offset((*Connect).PinId as isize))
            < (*(*(*This).Filter.Descriptor).PinDescriptors.offset((*Connect).PinId as isize)).InstancesPossible
        {
            Status = KspCreatePin(
                DeviceObject,
                Irp,
                (*This).Header.KsDevice,
                (*This).FilterFactory,
                &mut (*This).Header.OuterUnknown as *mut _ as *mut IKsFilter,
                Connect,
                (*(*This).Filter.Descriptor).PinDescriptors.offset((*Connect).PinId as isize) as *mut KSPIN_DESCRIPTOR_EX,
            );
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    }

    KeReleaseMutex((*This).Header.ControlMutex, 0u8);

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_DispatchCreateNode(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    CompleteRequest(Irp, IO_NO_INCREMENT);
    STATUS_UNSUCCESSFUL
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_AttachFilterToFilterFactory(
    This: *mut IKsFilterImpl,
    FilterFactory: PKSFILTERFACTORY,
) {
    let mut BasicHeader: PKSBASIC_HEADER;
    let mut Filter: PKSFILTER;

    BasicHeader = (FilterFactory as usize)
        .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
        as PKSBASIC_HEADER;

    if (*BasicHeader).FirstChild.FilterFactory.is_null() {
        (*BasicHeader).FirstChild.Filter = &mut (*This).Filter as PKSFILTER;
        return;
    }

    Filter = (*BasicHeader).FirstChild.Filter;

    loop {
        BasicHeader = (Filter as usize)
            .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
            as PKSBASIC_HEADER;

        if !(*BasicHeader).Next.Filter.is_null() {
            Filter = (*BasicHeader).Next.Filter;
        } else {
            break;
        }
    }

    (*BasicHeader).Next.Filter = &mut (*This).Filter as PKSFILTER;
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilter_RemoveFilterFromFilterFactory(
    This: *mut IKsFilterImpl,
    FilterFactory: PKSFILTERFACTORY,
) {
    let mut BasicHeader: PKSBASIC_HEADER;
    let mut Filter: PKSFILTER;
    let mut LastFilter: PKSFILTER;

    BasicHeader = (FilterFactory as usize)
        .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
        as PKSBASIC_HEADER;

    ntkrnl_sys::ASSERT!((*BasicHeader).Type as u32 == KsObjectTypeFilterFactory as u32);
    ntkrnl_sys::ASSERT!(!(*BasicHeader).FirstChild.Filter.is_null());

    Filter = (*BasicHeader).FirstChild.Filter;
    LastFilter = core::ptr::null_mut();

    loop {
        if Filter == &mut (*This).Filter {
            if !LastFilter.is_null() {
                BasicHeader = (LastFilter as usize)
                    .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
                    as PKSBASIC_HEADER;
                (*BasicHeader).Next.Filter = (*This).Header.Next.Filter;
                break;
            } else {
                (*BasicHeader).FirstChild.Filter = (*This).Header.Next.Filter;
                break;
            }
        }

        BasicHeader = (Filter as usize)
            .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
            as PKSBASIC_HEADER;

        ntkrnl_sys::ASSERT!((*BasicHeader).Type as u32 == KsObjectTypeFilter as u32);

        LastFilter = Filter;
        if !(*BasicHeader).Next.Filter.is_null() {
            Filter = (*BasicHeader).Next.Filter;
        } else {
            ntkrnl_sys::ASSERT!(false);
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilter_FilterCentricWorker(Ctx: PVOID) {
    let Object: *mut IKsProcessingObject = Ctx as *mut IKsProcessingObject;
    (*(*Object).lpVtbl).ProcessingObjectWork.expect("non-null function pointer")(Object);
}

#[no_mangle]
pub unsafe extern "system" fn KspCreateFilter(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    iface: *mut IKsFilterFactory,
) -> NTSTATUS {
    let mut This: *mut IKsFilterImpl = core::ptr::null_mut();
    let mut KsDevice: *mut IKsDevice = core::ptr::null_mut();
    let mut Factory: PKSFILTERFACTORY = core::ptr::null_mut();
    let mut IoStack: PIO_STACK_LOCATION = core::ptr::null_mut();
    let mut DeviceExtension: PDEVICE_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;
    let mut CreateItem: PKSOBJECT_CREATE_ITEM = core::ptr::null_mut();

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    Factory = (*(*iface).lpVtbl).GetStruct.expect("non-null function pointer")(iface) as PKSFILTERFACTORY;

    if Factory.is_null() || (*Factory).FilterDescriptor.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    if (*(*Factory).FilterDescriptor).Flags & KSFILTER_FLAG_DENY_USERMODE_ACCESS != 0 {
        if (*Irp).RequestorMode == 1i8 {
            return STATUS_UNSUCCESSFUL;
        }
    }

    This = AllocateItem(NonPagedPool, core::mem::size_of::<IKsFilterImpl>()) as *mut IKsFilterImpl;
    if This.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*This).Filter.Bag = AllocateItem(NonPagedPool, core::mem::size_of::<KSIOBJECT_BAG>()) as KSOBJECT_BAG;
    if (*This).Filter.Bag.is_null() {
        FreeItem(This as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    KsDevice = &raw mut (*(*DeviceExtension).DeviceHeader).BasicHeader.OuterUnknown as *mut IKsDevice;
    (*(*KsDevice).lpVtbl).InitializeObjectBag.expect("non-null function pointer")(
        KsDevice,
        (*This).Filter.Bag as PKSIOBJECT_BAG,
        core::ptr::null_mut(),
    );

    Status = IKsFilter_CopyFilterDescriptor(This, (*Factory).FilterDescriptor);
    if !NT_SUCCESS(Status) {
        FreeItem((*This).Filter.Bag as PVOID);
        FreeItem(This as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    CreateItem = AllocateItem(NonPagedPool, core::mem::size_of::<KSOBJECT_CREATE_ITEM>() * 2) as PKSOBJECT_CREATE_ITEM;
    if CreateItem.is_null() {
        FreeItem((*This).Filter.Bag as PVOID);
        FreeItem(This as PVOID);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*CreateItem.offset(0)).Create = core::mem::transmute(IKsFilter_DispatchCreatePin as *const ());
    (*CreateItem.offset(0)).Context = This as PVOID;
    (*CreateItem.offset(0)).Flags = KSCREATE_ITEM_FREEONSTOP as ULONG;
    RtlInitUnicodeString(&mut (*CreateItem.offset(0)).ObjectClass, KSSTRING_Pin.as_ptr() as PCWSTR);

    (*CreateItem.offset(1)).Create = core::mem::transmute(IKsFilter_DispatchCreateNode as *const ());
    (*CreateItem.offset(1)).Context = This as PVOID;
    (*CreateItem.offset(1)).Flags = KSCREATE_ITEM_FREEONSTOP as ULONG;
    RtlInitUnicodeString(&mut (*CreateItem.offset(1)).ObjectClass, KSSTRING_TopologyNode.as_ptr() as PCWSTR);

    (*This).ref_ = 1;
    (*This).Header.OuterUnknown = &raw mut vt_IKsFilter as PUNKNOWN;
    (*This).lpVtblKsControl = &raw mut vt_IKsControl;
    (*This).lpVtblKsProcessingObject = &raw mut vt_IKsProcessingObject;

    (*This).Factory = Factory;
    (*This).FilterFactory = iface;
    (*This).FileObject = (*IoStack).FileObject;
    KeInitializeMutex(&mut (*This).ProcessingMutex, 0);

    (*This).Header.KsDevice = &mut (*(*DeviceExtension).DeviceHeader).KsDevice as PKSDEVICE;
    (*This).Header.Parent.KsFilterFactory = (*(*iface).lpVtbl).GetStruct.expect("non-null function pointer")(iface) as PKSFILTERFACTORY;
    (*This).Header.Type = KsObjectTypeFilter;
    (*This).Header.ControlMutex = &mut (*This).ControlMutex as PRKMUTEX;
    KeInitializeMutex((*This).Header.ControlMutex, 0);
    InitializeListHead(&mut (*This).Header.EventList);
    KeInitializeSpinLock(&mut (*This).Header.EventListLock);

    KsGateInitializeAnd(&mut (*This).Gate, core::ptr::null_mut());

    ExInitializeWorkItem(&mut (*This).WorkItem as *mut _ as *mut _, crate::cb!(IKsFilter_FilterCentricWorker), (*This).lpVtblKsProcessingObject as PVOID);

    Status = KsRegisterCountedWorker(HyperCriticalWorkQueue, &mut (*This).WorkItem, &mut (*This).Worker);
    if !NT_SUCCESS(Status) {
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return Status;
    }

    Status = IKsFilter_CreateDescriptors(This, (*Factory).FilterDescriptor as *mut KSFILTER_DESCRIPTOR);
    if !NT_SUCCESS(Status) {
        KsUnregisterWorker((*This).Worker);
        FreeItem(This as PVOID);
        FreeItem(CreateItem as PVOID);
        return Status;
    }

    if !(*(*Factory).FilterDescriptor).Dispatch.is_null() {
        if (*(*(*Factory).FilterDescriptor).Dispatch).Create.is_some() {
            Status = (*(*(*Factory).FilterDescriptor).Dispatch).Create.expect("non-null function pointer")(&mut (*This).Filter, Irp);
            if !NT_SUCCESS(Status) && Status != STATUS_PENDING {
                KsUnregisterWorker((*This).Worker);
                FreeItem(This as PVOID);
                FreeItem(CreateItem as PVOID);
                return Status;
            }
        }
    }

    Status = KsAllocateObjectHeader(&raw mut (*This).ObjectHeader as *mut KSOBJECT_HEADER, 2, CreateItem, Irp, &raw mut DispatchTable);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*(*This).ObjectHeader).Type = KsObjectTypeFilter;
    (*(*This).ObjectHeader).Unknown = &raw mut (*This).Header.OuterUnknown as PUNKNOWN;
    (*(*This).ObjectHeader).ObjectType = &raw mut (*This).Filter as PVOID;

    IKsFilter_AttachFilterToFilterFactory(This, (*This).Header.Parent.KsFilterFactory);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterAcquireProcessingMutex(
    Filter: PKSFILTER,
) {
    let This: *mut IKsFilterImpl = (Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter))
        as *mut IKsFilterImpl;

    KeWaitForSingleObject(
        core::ptr::addr_of_mut!((*This).ProcessingMutex) as *mut _,
        _KWAIT_REASON_Executive,
        0i8,
        0u8,
        core::ptr::null_mut(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterReleaseProcessingMutex(
    Filter: PKSFILTER,
) {
    let This: *mut IKsFilterImpl = core::ptr::with_exposed_provenance_mut::<IKsFilterImpl>((Filter as usize)
            .wrapping_sub(
                core::mem::offset_of!(IKsFilterImpl, Filter),
            ),
    );
    KeReleaseMutex(&raw mut (*This).ProcessingMutex, 0u8);
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterAddTopologyConnections(
    Filter: PKSFILTER,
    NewConnectionsCount: ULONG,
    NewTopologyConnections: *const KSTOPOLOGY_CONNECTION,
) -> NTSTATUS {
    let Count: ULONG;
    let Status: NTSTATUS;
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);

    Count = (*((*This).Filter.Descriptor)).ConnectionsCount.wrapping_add(NewConnectionsCount);

    Status = _KsEdit(
        (*This).Filter.Bag,
        &raw const (*((*This).Filter.Descriptor)).Connections as *mut PVOID,
        Count.wrapping_mul(core::mem::size_of::<KSTOPOLOGY_CONNECTION>() as ULONG),
        (*((*This).Filter.Descriptor)).ConnectionsCount.wrapping_mul(core::mem::size_of::<KSTOPOLOGY_CONNECTION>() as ULONG),
        0,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    core::ptr::copy_nonoverlapping(
        NewTopologyConnections,
        (*((*This).Filter.Descriptor)).Connections.offset((*((*This).Filter.Descriptor)).ConnectionsCount as isize) as *mut _,
        NewConnectionsCount as usize,
    );

    (*This).Topology.TopologyConnectionsCount = (*This).Topology.TopologyConnectionsCount.wrapping_add(NewConnectionsCount);
    (*((*This).Filter.Descriptor as PKSFILTER_DESCRIPTOR)).ConnectionsCount = (*((*This).Filter.Descriptor as PKSFILTER_DESCRIPTOR)).ConnectionsCount.wrapping_add(NewConnectionsCount);
    (*This).Topology.TopologyConnections = (*((*This).Filter.Descriptor)).Connections;

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterAttemptProcessing(
    Filter: PKSFILTER,
    Asynchronous: BOOLEAN,
) {
    let Gate: PKSGATE;
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);

    Gate = (*(*This).lpVtblKsProcessingObject).GetAndGate.expect("non-null function pointer")(
        (*This).lpVtblKsProcessingObject as *mut IKsProcessingObject,
    );

    if KsGateCaptureThreshold(Gate) == 0 {
        return;
    }

    (*(*This).lpVtblKsProcessingObject).Process.expect("non-null function pointer")(
        (*This).lpVtblKsProcessingObject as *mut IKsProcessingObject,
        Asynchronous,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterCreateNode(
    Filter: PKSFILTER,
    NodeDescriptor: *const KSNODE_DESCRIPTOR,
    NodeID: PULONG,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterCreatePinFactory(
    Filter: PKSFILTER,
    InPinDescriptor: *const KSPIN_DESCRIPTOR_EX,
    PinID: PULONG,
) -> NTSTATUS {
    let This = (Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter))
        as *mut IKsFilterImpl;

    let Count = ((*(*This).Filter.Descriptor).PinDescriptorsCount as ULONG).wrapping_add(1);

    let mut Status = _KsEdit(
        (*This).Filter.Bag,
        core::ptr::addr_of!((*(*This).Filter.Descriptor).PinDescriptors) as *mut PVOID,
        Count.wrapping_mul((*(*This).Filter.Descriptor).PinDescriptorSize),
        (*(*This).Filter.Descriptor).PinDescriptorsCount.wrapping_mul((*(*This).Filter.Descriptor).PinDescriptorSize),
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = _KsEdit(
        (*This).Filter.Bag,
        core::ptr::addr_of_mut!((*This).PinInstanceCount) as *mut PVOID,
        (core::mem::size_of::<ULONG>() as ULONG).wrapping_mul(Count),
        (core::mem::size_of::<ULONG>() as ULONG).wrapping_mul((*(*This).Filter.Descriptor).PinDescriptorsCount),
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = _KsEdit(
        (*This).Filter.Bag,
        core::ptr::addr_of_mut!((*This).FirstPin) as *mut PVOID,
        (core::mem::size_of::<PVOID>() as ULONG).wrapping_mul(Count),
        (core::mem::size_of::<PVOID>() as ULONG).wrapping_mul((*(*This).Filter.Descriptor).PinDescriptorsCount),
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    core::ptr::copy_nonoverlapping(
        InPinDescriptor,
        (*(*This).Filter.Descriptor).PinDescriptors.add((*(*This).Filter.Descriptor).PinDescriptorsCount as usize) as *mut _,
        1,
    );

    Status = _KsEdit(
        (*This).Filter.Bag,
        core::ptr::addr_of_mut!((*This).ProcessPinIndex) as *mut PVOID,
        (core::mem::size_of::<KSPROCESSPIN_INDEXENTRY>() as ULONG).wrapping_mul(Count),
        (core::mem::size_of::<KSPROCESSPIN_INDEXENTRY>() as ULONG).wrapping_mul((*(*This).Filter.Descriptor).PinDescriptorsCount),
        0,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    *PinID = (*(*This).Filter.Descriptor).PinDescriptorsCount;

    let desc = (*This).Filter.Descriptor as PKSFILTER_DESCRIPTOR;
    (*desc).PinDescriptorsCount = (*desc).PinDescriptorsCount.wrapping_add(1);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterGetAndGate(
    Filter: PKSFILTER,
) -> PKSGATE {
    let This = ((Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter)))
        as *mut IKsFilterImpl;
    core::ptr::addr_of_mut!((*This).Gate)
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterGetChildPinCount(
    Filter: PKSFILTER,
    PinId: ULONG,
) -> ULONG {
    let This = CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter) as *mut IKsFilterImpl;

    if PinId >= (*((*This).Filter.Descriptor)).PinDescriptorsCount {
        return 0;
    }

    return *(*This).PinInstanceCount.offset(PinId as isize);
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterGetFirstChildPin(
    Filter: PKSFILTER,
    PinId: ULONG,
) -> PKSPIN {
    let This = (Filter as usize)
        .wrapping_sub(core::mem::offset_of!(IKsFilterImpl, Filter))
        as *mut IKsFilterImpl;

    if PinId >= (*((*This).Filter.Descriptor)).PinDescriptorsCount {
        return core::ptr::null_mut();
    }

    *(*This).FirstPin.add(PinId as usize)
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterRegisterPowerCallbacks(
    Filter: PKSFILTER,
    Sleep: PFNKSFILTERPOWER,
    Wake: PFNKSFILTERPOWER,
) {
    let This: *mut IKsFilterImpl = CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
    (*This).Sleep = Sleep;
    (*This).Wake = Wake;
}

#[no_mangle]
pub unsafe extern "system" fn KsGetFilterFromIrp(
    Irp: PIRP,
) -> PKSFILTER {
    let IoStack: PIO_STACK_LOCATION;
    let ObjectHeader: PKSIOBJECT_HEADER;

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    ObjectHeader = (*(*IoStack).FileObject).FsContext2 as PKSIOBJECT_HEADER;

    if (*ObjectHeader).Type == KsObjectTypeFilter {
        return (*ObjectHeader).ObjectType as PKSFILTER;
    } else if (*ObjectHeader).Type == KsObjectTypePin {
        return KsPinGetParentFilter((*ObjectHeader).ObjectType as PKSPIN);
    } else {
        return core::ptr::null_mut();
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/filter.c
//  * PURPOSE:         KS IKsFilter interface functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef struct
// {
//     KSBASIC_HEADER Header;
//     KSFILTER Filter;
//
//     IKsControlVtbl *lpVtblKsControl;
//     IKsFilterFactory * FilterFactory;
//     IKsProcessingObjectVtbl * lpVtblKsProcessingObject;
//     LONG ref;
//
//     PKSIOBJECT_HEADER ObjectHeader;
//     KSTOPOLOGY Topology;
//     PKSFILTERFACTORY Factory;
//     PFILE_OBJECT FileObject;
//     KMUTEX ControlMutex;
//     KMUTEX ProcessingMutex;
//
//     PKSWORKER Worker;
//     WORK_QUEUE_ITEM WorkItem;
//     KSGATE Gate;
//
//     PFNKSFILTERPOWER Sleep;
//     PFNKSFILTERPOWER Wake;
//
//     ULONG *PinInstanceCount;
//     PKSPIN * FirstPin;
//     PKSPROCESSPIN_INDEXENTRY ProcessPinIndex;
//
// }IKsFilterImpl;
//
// const GUID IID_IKsControl = {0x28F54685L, 0x06FD, 0x11D2, {0xB2, 0x7A, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96}};
// const GUID IID_IKsFilter  = {0x3ef6ee44L, 0x0D41, 0x11d2, {0xbe, 0xDA, 0x00, 0xc0, 0x4f, 0x8e, 0xF4, 0x57}};
// const GUID KSPROPSETID_Topology                = {0x720D4AC0L, 0x7533, 0x11D0, {0xA5, 0xD6, 0x28, 0xDB, 0x04, 0xC1, 0x00, 0x00}};
// const GUID KSPROPSETID_Pin                     = {0x8C134960L, 0x51AD, 0x11CF, {0x87, 0x8A, 0x94, 0xF8, 0x01, 0xC1, 0x00, 0x00}};
// const GUID KSPROPSETID_General =                 {0x1464EDA5L, 0x6A8F, 0x11D1, {0x9A, 0xA7, 0x00, 0xA0, 0xC9, 0x22, 0x31, 0x96}};
//
// VOID
// IKsFilter_RemoveFilterFromFilterFactory(
//     IKsFilterImpl * This,
//     PKSFILTERFACTORY FilterFactory);
//
// NTSTATUS NTAPI FilterTopologyPropertyHandler(IN PIRP Irp, IN PKSIDENTIFIER  Request, IN OUT PVOID  Data);
// NTSTATUS NTAPI FilterPinPropertyHandler(IN PIRP Irp, IN PKSIDENTIFIER  Request, IN OUT PVOID  Data);
// NTSTATUS NTAPI FilterGeneralComponentIdHandler(IN PIRP Irp, IN PKSIDENTIFIER  Request, IN OUT PVOID  Data);
//
// DEFINE_KSPROPERTY_TOPOLOGYSET(IKsFilterTopologySet, FilterTopologyPropertyHandler);
// DEFINE_KSPROPERTY_PINPROPOSEDATAFORMAT(IKsFilterPinSet, FilterPinPropertyHandler, FilterPinPropertyHandler, FilterPinPropertyHandler);
// DEFINE_KSPROPERTY_GENEREAL_COMPONENTID(IKsFilterGeneralSet, FilterGeneralComponentIdHandler);
//
// KSPROPERTY_SET FilterPropertySet[] =
// {
//     {
//         &KSPROPSETID_Topology,
//         sizeof(IKsFilterTopologySet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&IKsFilterTopologySet,
//         0,
//         NULL
//     },
//     {
//         &KSPROPSETID_Pin,
//         sizeof(IKsFilterPinSet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&IKsFilterPinSet,
//         0,
//         NULL
//     },
//     {
//         &KSPROPSETID_General,
//         sizeof(IKsFilterGeneralSet) / sizeof(KSPROPERTY_ITEM),
//         (const KSPROPERTY_ITEM*)&IKsFilterGeneralSet,
//         0,
//         NULL
//     }
// };
//
// NTSTATUS
// NTAPI
// IKsProcessingObject_fnQueryInterface(
//     IKsProcessingObject * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->Header.OuterUnknown;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//     return STATUS_UNSUCCESSFUL;
// }
//
// ULONG
// NTAPI
// IKsProcessingObject_fnAddRef(
//     IKsProcessingObject * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsProcessingObject_fnRelease(
//     IKsProcessingObject * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     InterlockedDecrement(&This->ref);
//
//     /* Return new reference count */
//     return This->ref;
// }
//
// VOID
// NTAPI
// IKsProcessingObject_fnProcessingObjectWork(
//     IKsProcessingObject * iface)
// {
//     NTSTATUS Status;
//     LARGE_INTEGER TimeOut;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     DPRINT1("processing object\n");
//     /* first check if running at passive level */
//     if (KeGetCurrentIrql() == PASSIVE_LEVEL)
//     {
//         /* acquire processing mutex */
//         KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
//     }
//     else
//     {
//         /* dispatch level processing */
//         if (KeReadStateMutex(&This->ProcessingMutex) == 0)
//         {
//             /* some thread was faster */
//             DPRINT1("processing object too slow\n");
//             return;
//         }
//
//         /* acquire processing mutex */
//         TimeOut.QuadPart = 0LL;
//         Status = KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, &TimeOut);
//
//         if (Status == STATUS_TIMEOUT)
//         {
//             /* some thread was faster */
//             DPRINT1("processing object too slow\n");
//             return;
//         }
//     }
//
//     do
//     {
//
//         /* check if the and-gate has been enabled again */
//         if (This->Gate.Count != 0)
//         {
//             /* gate is open */
//             DPRINT1("processing object gate open\n");
//             break;
//         }
//
//         DPRINT1("IKsProcessingObject_fnProcessingObjectWork not implemented\n");
//         ASSERT(0);
//
//     }while(TRUE);
//
//     /* release process mutex */
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
// }
//
// PKSGATE
// NTAPI
// IKsProcessingObject_fnGetAndGate(
//     IKsProcessingObject * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     /* return and gate */
//     return &This->Gate;
// }
//
// VOID
// NTAPI
// IKsProcessingObject_fnProcess(
//     IKsProcessingObject * iface,
//     IN BOOLEAN Asynchronous)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     /* should the action be asynchronous */
//     if (Asynchronous)
//     {
//         /* queue work item */
//         KsQueueWorkItem(This->Worker, &This->WorkItem);
// DPRINT1("queueing\n");
//         /* done */
//         return;
//     }
//
//     /* does the filter require explicit deferred processing */
//     if ((This->Filter.Descriptor->Flags & (KSFILTER_FLAG_DISPATCH_LEVEL_PROCESSING | KSFILTER_FLAG_CRITICAL_PROCESSING | KSFILTER_FLAG_HYPERCRITICAL_PROCESSING)) &&
//          KeGetCurrentIrql() > PASSIVE_LEVEL)
//     {
//         /* queue work item */
//         KsQueueWorkItem(This->Worker, &This->WorkItem);
// DPRINT1("queueing\n");
//         /* done */
//         return;
//     }
// DPRINT1("invoke\n");
//     /* call worker routine directly */
//     iface->lpVtbl->ProcessingObjectWork(iface);
// }
//
// VOID
// NTAPI
// IKsProcessingObject_fnReset(
//     IKsProcessingObject * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsProcessingObject);
//
//     /* acquire processing mutex */
//     KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
//
//     /* check if the filter supports dispatch routines */
//     if (This->Filter.Descriptor->Dispatch)
//     {
//         /* has the filter a reset routine */
//         if (This->Filter.Descriptor->Dispatch->Reset)
//         {
//             /* reset filter */
//             This->Filter.Descriptor->Dispatch->Reset(&This->Filter);
//         }
//     }
//
//     /* release process mutex */
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
// }
//
// VOID
// NTAPI
// IKsProcessingObject_fnTriggerNotification(
//     IKsProcessingObject * iface)
// {
//
// }
//
// static IKsProcessingObjectVtbl vt_IKsProcessingObject =
// {
//     IKsProcessingObject_fnQueryInterface,
//     IKsProcessingObject_fnAddRef,
//     IKsProcessingObject_fnRelease,
//     IKsProcessingObject_fnProcessingObjectWork,
//     IKsProcessingObject_fnGetAndGate,
//     IKsProcessingObject_fnProcess,
//     IKsProcessingObject_fnReset,
//     IKsProcessingObject_fnTriggerNotification
// };
//
//
// //---------------------------------------------------------------------------------------------------------
// NTSTATUS
// NTAPI
// IKsControl_fnQueryInterface(
//     IKsControl * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->Header.OuterUnknown;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//     return STATUS_UNSUCCESSFUL;
// }
//
// ULONG
// NTAPI
// IKsControl_fnAddRef(
//     IKsControl * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsControl_fnRelease(
//     IKsControl * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     InterlockedDecrement(&This->ref);
//
//     /* Return new reference count */
//     return This->ref;
// }
//
// NTSTATUS
// NTAPI
// IKsControl_fnKsProperty(
//     IKsControl * iface,
//     IN PKSPROPERTY Property,
//     IN ULONG PropertyLength,
//     IN OUT PVOID PropertyData,
//     IN ULONG DataLength,
//     OUT ULONG* BytesReturned)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     return KsSynchronousIoControlDevice(This->FileObject, KernelMode, IOCTL_KS_PROPERTY, Property, PropertyLength, PropertyData, DataLength, BytesReturned);
// }
//
//
// NTSTATUS
// NTAPI
// IKsControl_fnKsMethod(
//     IKsControl * iface,
//     IN PKSMETHOD Method,
//     IN ULONG MethodLength,
//     IN OUT PVOID MethodData,
//     IN ULONG DataLength,
//     OUT ULONG* BytesReturned)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     return KsSynchronousIoControlDevice(This->FileObject, KernelMode, IOCTL_KS_METHOD, Method, MethodLength, MethodData, DataLength, BytesReturned);
// }
//
//
// NTSTATUS
// NTAPI
// IKsControl_fnKsEvent(
//     IKsControl * iface,
//     IN PKSEVENT Event OPTIONAL,
//     IN ULONG EventLength,
//     IN OUT PVOID EventData,
//     IN ULONG DataLength,
//     OUT ULONG* BytesReturned)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, lpVtblKsControl);
//
//     if (Event)
//     {
//        return KsSynchronousIoControlDevice(This->FileObject, KernelMode, IOCTL_KS_ENABLE_EVENT, Event, EventLength, EventData, DataLength, BytesReturned);
//     }
//     else
//     {
//        return KsSynchronousIoControlDevice(This->FileObject, KernelMode, IOCTL_KS_DISABLE_EVENT, EventData, DataLength, NULL, 0, BytesReturned);
//     }
//
// }
//
// static IKsControlVtbl vt_IKsControl =
// {
//     IKsControl_fnQueryInterface,
//     IKsControl_fnAddRef,
//     IKsControl_fnRelease,
//     IKsControl_fnKsProperty,
//     IKsControl_fnKsMethod,
//     IKsControl_fnKsEvent
// };
//
//
// NTSTATUS
// NTAPI
// IKsFilter_fnQueryInterface(
//     IKsFilter * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     NTSTATUS Status;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown) ||
//         IsEqualGUIDAligned(refiid, &IID_IKsFilter))
//     {
//         *Output = &This->Header.OuterUnknown;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//     else if (IsEqualGUIDAligned(refiid, &IID_IKsControl))
//     {
//         *Output = &This->lpVtblKsControl;
//         _InterlockedIncrement(&This->ref);
//         return STATUS_SUCCESS;
//     }
//
//     if (This->Header.ClientAggregate)
//     {
//          /* using client aggregate */
//          Status = This->Header.ClientAggregate->lpVtbl->QueryInterface(This->Header.ClientAggregate, refiid, Output);
//
//          if (NT_SUCCESS(Status))
//          {
//              /* client aggregate supports interface */
//              return Status;
//          }
//     }
//
//     DPRINT("IKsFilter_fnQueryInterface no interface\n");
//     return STATUS_NOT_SUPPORTED;
// }
//
// ULONG
// NTAPI
// IKsFilter_fnAddRef(
//     IKsFilter * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsFilter_fnRelease(
//     IKsFilter * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
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
//
// }
//
// PKSFILTER
// NTAPI
// IKsFilter_fnGetStruct(
//     IKsFilter * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     return &This->Filter;
// }
//
// BOOL
// NTAPI
// IKsFilter_fnDoAllNecessaryPinsExist(
//     IKsFilter * iface)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_fnCreateNode(
//     IKsFilter * iface,
//     IN PIRP Irp,
//     IN IKsPin * Pin,
//     IN PLIST_ENTRY ListEntry)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_fnBindProcessPinsToPipeSection(
//     IKsFilter * iface,
//     IN struct KSPROCESSPIPESECTION *Section,
//     IN PVOID Create,
//     IN PKSPIN KsPin,
//     OUT IKsPin **Pin,
//     OUT PKSGATE *OutGate)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_fnUnbindProcessPinsFromPipeSection(
//     IKsFilter * iface,
//     IN struct KSPROCESSPIPESECTION *Section)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_fnAddProcessPin(
//     IKsFilter * iface,
//     IN PKSPROCESSPIN ProcessPin)
// {
//     NTSTATUS Status;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     /* first acquire processing mutex */
//     KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
//
//     /* sanity check */
//     ASSERT(This->Filter.Descriptor->PinDescriptorsCount > ProcessPin->Pin->Id);
//
//     /* allocate new process pin array */
//     Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->ProcessPinIndex[ProcessPin->Pin->Id].Pins,
//                      (This->Filter.Descriptor->PinDescriptorsCount + 1) * sizeof(PVOID),
//                      This->Filter.Descriptor->PinDescriptorsCount * sizeof(PVOID),
//                      0);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* store process pin */
//         This->ProcessPinIndex[ProcessPin->Pin->Id].Pins[This->ProcessPinIndex[ProcessPin->Pin->Id].Count] = ProcessPin;
//         This->ProcessPinIndex[ProcessPin->Pin->Id].Count++;
//     }
//
//     /* release process mutex */
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_fnRemoveProcessPin(
//     IKsFilter * iface,
//     IN PKSPROCESSPIN ProcessPin)
// {
//     ULONG Index;
//     ULONG Count;
//     PKSPROCESSPIN * Pins;
//
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     /* first acquire processing mutex */
//     KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
//
//     /* sanity check */
//     ASSERT(ProcessPin->Pin);
//     ASSERT(ProcessPin->Pin->Id);
//
//     Count = This->ProcessPinIndex[ProcessPin->Pin->Id].Count;
//     Pins =  This->ProcessPinIndex[ProcessPin->Pin->Id].Pins;
//
//     /* search for current process pin */
//     for(Index = 0; Index < Count; Index++)
//     {
//         if (Pins[Index] == ProcessPin)
//         {
//             RtlMoveMemory(&Pins[Index], &Pins[Index + 1], (Count - (Index + 1)) * sizeof(PVOID));
//             break;
//         }
//
//     }
//
//     /* decrement pin count */
//     This->ProcessPinIndex[ProcessPin->Pin->Id].Count--;
//
//     if (!This->ProcessPinIndex[ProcessPin->Pin->Id].Count)
//     {
//         /* clear entry object bag will delete it */
//        This->ProcessPinIndex[ProcessPin->Pin->Id].Pins = NULL;
//     }
//
//     /* release process mutex */
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
//
//     /* done */
//     return STATUS_SUCCESS;
// }
//
// BOOL
// NTAPI
// IKsFilter_fnReprepareProcessPipeSection(
//     IKsFilter * iface,
//     IN struct KSPROCESSPIPESECTION *PipeSection,
//     IN PULONG Data)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// VOID
// NTAPI
// IKsFilter_fnDeliverResetState(
//     IKsFilter * iface,
//     IN struct KSPROCESSPIPESECTION *PipeSection,
//     IN KSRESET ResetState)
// {
//     UNIMPLEMENTED;
// }
//
// BOOL
// NTAPI
// IKsFilter_fnIsFrameHolding(
//     IKsFilter * iface)
// {
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
// VOID
// NTAPI
// IKsFilter_fnRegisterForCopyCallbacks(
//     IKsFilter * iface,
//     IKsQueue *Queue,
//     BOOL Register)
// {
//     UNIMPLEMENTED;
// }
//
// PKSPROCESSPIN_INDEXENTRY
// NTAPI
// IKsFilter_fnGetProcessDispatch(
//     IKsFilter * iface)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(iface, IKsFilterImpl, Header.OuterUnknown);
//
//     return This->ProcessPinIndex;
// }
//
// static IKsFilterVtbl vt_IKsFilter =
// {
//     IKsFilter_fnQueryInterface,
//     IKsFilter_fnAddRef,
//     IKsFilter_fnRelease,
//     IKsFilter_fnGetStruct,
//     IKsFilter_fnDoAllNecessaryPinsExist,
//     IKsFilter_fnCreateNode,
//     IKsFilter_fnBindProcessPinsToPipeSection,
//     IKsFilter_fnUnbindProcessPinsFromPipeSection,
//     IKsFilter_fnAddProcessPin,
//     IKsFilter_fnRemoveProcessPin,
//     IKsFilter_fnReprepareProcessPipeSection,
//     IKsFilter_fnDeliverResetState,
//     IKsFilter_fnIsFrameHolding,
//     IKsFilter_fnRegisterForCopyCallbacks,
//     IKsFilter_fnGetProcessDispatch
// };
//
// NTSTATUS
// IKsFilter_GetFilterFromIrp(
//     IN PIRP Irp,
//     OUT IKsFilter **Filter)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//     NTSTATUS Status;
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject != NULL);
//
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     /* sanity is important */
//     ASSERT(ObjectHeader != NULL);
//     ASSERT(ObjectHeader->Type == KsObjectTypeFilter);
//     ASSERT(ObjectHeader->Unknown != NULL);
//
//     /* get our private interface */
//     Status = ObjectHeader->Unknown->lpVtbl->QueryInterface(ObjectHeader->Unknown, &IID_IKsFilter, (PVOID*)Filter);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* something is wrong here */
//         DPRINT1("KS: Misbehaving filter %p\n", ObjectHeader->Unknown);
//         Irp->IoStatus.Status = Status;
//
//         /* complete and forget irp */
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//         return Status;
//     }
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// IKsFilter_DispatchClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     IKsFilter * Filter;
//     IKsFilterImpl * This;
//     NTSTATUS Status;
//
//     /* obtain filter from object header */
//     Status = IKsFilter_GetFilterFromIrp(Irp, &Filter);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* get our real implementation */
//     This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Header.OuterUnknown);
//
//     /* does the driver support notifications */
//     if (This->Filter.Descriptor && This->Filter.Descriptor->Dispatch && This->Filter.Descriptor->Dispatch->Close)
//     {
//         /* call driver's filter close function */
//         Status = This->Filter.Descriptor->Dispatch->Close(&This->Filter, Irp);
//     }
//
//     if (NT_SUCCESS(Status) && Status != STATUS_PENDING)
//     {
//         /* save the result */
//         Irp->IoStatus.Status = Status;
//         /* complete irp */
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//
//         /* remove our instance from the filter factory */
//         IKsFilter_RemoveFilterFromFilterFactory(This, This->Factory);
//
//         /* free object header */
//         KsFreeObjectHeader(This->ObjectHeader);
//     }
//     else
//     {
//         /* complete and forget */
//         Irp->IoStatus.Status = Status;
//         /* complete irp */
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     /* done */
//     return Status;
// }
//
// NTSTATUS
// KspHandlePropertyInstances(
//     IN PIO_STATUS_BLOCK IoStatus,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data,
//     IN IKsFilterImpl * This,
//     IN BOOL Global)
// {
//     KSPIN_CINSTANCES * Instances;
//     KSP_PIN * Pin = (KSP_PIN*)Request;
//
//     if (!This->Filter.Descriptor || !This->Filter.Descriptor->PinDescriptorsCount)
//     {
//         /* no filter / pin descriptor */
//         IoStatus->Status = STATUS_NOT_IMPLEMENTED;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* ignore custom structs for now */
//     ASSERT(This->Filter.Descriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//     ASSERT(This->Filter.Descriptor->PinDescriptorsCount > Pin->PinId);
//
//     Instances = (KSPIN_CINSTANCES*)Data;
//     /* max instance count */
//     Instances->PossibleCount = This->Filter.Descriptor->PinDescriptors[Pin->PinId].InstancesPossible;
//     /* current instance count */
//     Instances->CurrentCount = This->PinInstanceCount[Pin->PinId];
//
//     IoStatus->Information = sizeof(KSPIN_CINSTANCES);
//     IoStatus->Status = STATUS_SUCCESS;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspHandleNecessaryPropertyInstances(
//     IN PIO_STATUS_BLOCK IoStatus,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data,
//     IN IKsFilterImpl * This)
// {
//     PULONG Result;
//     KSP_PIN * Pin = (KSP_PIN*)Request;
//
//     if (!This->Filter.Descriptor || !This->Filter.Descriptor->PinDescriptorsCount)
//     {
//         /* no filter / pin descriptor */
//         IoStatus->Status = STATUS_NOT_IMPLEMENTED;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* ignore custom structs for now */
//     ASSERT(This->Filter.Descriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//     ASSERT(This->Filter.Descriptor->PinDescriptorsCount > Pin->PinId);
//
//     Result = (PULONG)Data;
//     *Result = This->Filter.Descriptor->PinDescriptors[Pin->PinId].InstancesNecessary;
//
//     IoStatus->Information = sizeof(ULONG);
//     IoStatus->Status = STATUS_SUCCESS;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspHandleDataIntersection(
//     IN PIRP Irp,
//     IN PIO_STATUS_BLOCK IoStatus,
//     IN PKSIDENTIFIER Request,
//     IN OUT PVOID  Data,
//     IN ULONG DataLength,
//     IN IKsFilterImpl * This)
// {
//     PKSMULTIPLE_ITEM MultipleItem;
//     PKSDATARANGE DataRange;
//     NTSTATUS Status = STATUS_NO_MATCH;
//     ULONG Index, Length;
//     PIO_STACK_LOCATION IoStack;
//     KSP_PIN * Pin = (KSP_PIN*)Request;
//
//     /* get stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(DataLength == IoStack->Parameters.DeviceIoControl.OutputBufferLength);
//
//     /* Access parameters */
//     MultipleItem = (PKSMULTIPLE_ITEM)(Pin + 1);
//     DataRange = (PKSDATARANGE)(MultipleItem + 1);
//
//     /* FIXME make sure its 64 bit aligned */
//     ASSERT(((ULONG_PTR)DataRange & 0x7) == 0);
//
//     if (!This->Filter.Descriptor || !This->Filter.Descriptor->PinDescriptorsCount)
//     {
//         /* no filter / pin descriptor */
//         IoStatus->Status = STATUS_NOT_IMPLEMENTED;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     /* ignore custom structs for now */
//     ASSERT(This->Filter.Descriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//     ASSERT(This->Filter.Descriptor->PinDescriptorsCount > Pin->PinId);
//
//     if (This->Filter.Descriptor->PinDescriptors[Pin->PinId].IntersectHandler == NULL ||
//         This->Filter.Descriptor->PinDescriptors[Pin->PinId].PinDescriptor.DataRanges == NULL ||
//         This->Filter.Descriptor->PinDescriptors[Pin->PinId].PinDescriptor.DataRangesCount == 0)
//     {
//         /* no driver supported intersect handler / no provided data ranges */
//         IoStatus->Status = STATUS_NOT_IMPLEMENTED;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     for(Index = 0; Index < MultipleItem->Count; Index++)
//     {
//         UNICODE_STRING MajorFormat, SubFormat, Specifier;
//         /* convert the guid to string */
//         RtlStringFromGUID(&DataRange->MajorFormat, &MajorFormat);
//         RtlStringFromGUID(&DataRange->SubFormat, &SubFormat);
//         RtlStringFromGUID(&DataRange->Specifier, &Specifier);
//
//         DPRINT("KspHandleDataIntersection Index %lu PinId %lu MajorFormat %S SubFormat %S Specifier %S FormatSize %lu SampleSize %lu Align %lu Flags %lx Reserved %lx DataLength %lu\n", Index, Pin->PinId, MajorFormat.Buffer, SubFormat.Buffer, Specifier.Buffer,
//                DataRange->FormatSize, DataRange->SampleSize, DataRange->Alignment, DataRange->Flags, DataRange->Reserved, DataLength);
//
//         /* FIXME implement KsPinDataIntersectionEx */
//         /* Call miniport's proprietary handler */
//         Status = This->Filter.Descriptor->PinDescriptors[Pin->PinId].IntersectHandler(&This->Filter,
//                                                                                       Irp,
//                                                                                       Pin,
//                                                                                       DataRange,
//                                                                                       This->Filter.Descriptor->PinDescriptors[Pin->PinId].PinDescriptor.DataRanges[0], /* HACK */
//                                                                                       DataLength,
//                                                                                       Data,
//                                                                                       &Length);
//         DPRINT("KspHandleDataIntersection Status %lx\n", Status);
//
//         if (Status == STATUS_SUCCESS || Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ASSERT(Length);
//             IoStatus->Information = Length;
//             break;
//         }
//
//         DataRange = (PKSDATARANGE)((PUCHAR)DataRange + DataRange->FormatSize);
//         /* FIXME make sure its 64 bit aligned */
//         ASSERT(((ULONG_PTR)DataRange & 0x7) == 0);
//     }
//     IoStatus->Status = Status;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// FilterTopologyPropertyHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data)
// {
//     IKsFilterImpl * This;
//
//     /* get filter implementation */
//     This = (IKsFilterImpl*)KSPROPERTY_ITEM_IRP_STORAGE(Irp);
//
//     /* sanity check */
//     ASSERT(This);
//
//     return KsTopologyPropertyHandler(Irp, Request, Data, &This->Topology);
//
// }
//
// NTSTATUS
// NTAPI
// FilterGeneralComponentIdHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     IKsFilterImpl * This;
//
//     /* get filter implementation */
//     This = (IKsFilterImpl*)KSPROPERTY_ITEM_IRP_STORAGE(Irp);
//
//     /* sanity check */
//     ASSERT(This);
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//     ASSERT(IoStack->Parameters.DeviceIoControl.OutputBufferLength >= sizeof(KSCOMPONENTID));
//
//     if (This->Filter.Descriptor->ComponentId != NULL)
//     {
//         RtlMoveMemory(Data, This->Filter.Descriptor->ComponentId, sizeof(KSCOMPONENTID));
//         Irp->IoStatus.Information = sizeof(KSCOMPONENTID);
//         return STATUS_SUCCESS;
//     }
//     else
//     {
//         /* not valid */
//         return STATUS_NOT_FOUND;
//     }
//
// }
//
// NTSTATUS
// NTAPI
// FilterPinPropertyHandler(
//     IN PIRP Irp,
//     IN PKSIDENTIFIER  Request,
//     IN OUT PVOID  Data)
// {
//     PIO_STACK_LOCATION IoStack;
//     IKsFilterImpl * This;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//
//     /* get filter implementation */
//     This = (IKsFilterImpl*)KSPROPERTY_ITEM_IRP_STORAGE(Irp);
//
//     /* sanity check */
//     ASSERT(This);
//
//     /* get current stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     switch(Request->Id)
//     {
//         case KSPROPERTY_PIN_CTYPES:
//         case KSPROPERTY_PIN_DATAFLOW:
//         case KSPROPERTY_PIN_DATARANGES:
//         case KSPROPERTY_PIN_INTERFACES:
//         case KSPROPERTY_PIN_MEDIUMS:
//         case KSPROPERTY_PIN_COMMUNICATION:
//         case KSPROPERTY_PIN_CATEGORY:
//         case KSPROPERTY_PIN_NAME:
//         case KSPROPERTY_PIN_CONSTRAINEDDATARANGES:
//             Status = KspPinPropertyHandler(Irp, Request, Data, This->Filter.Descriptor->PinDescriptorsCount, (const KSPIN_DESCRIPTOR*)This->Filter.Descriptor->PinDescriptors, This->Filter.Descriptor->PinDescriptorSize);
//             break;
//         case KSPROPERTY_PIN_GLOBALCINSTANCES:
//             Status = KspHandlePropertyInstances(&Irp->IoStatus, Request, Data, This, TRUE);
//             break;
//         case KSPROPERTY_PIN_CINSTANCES:
//             Status = KspHandlePropertyInstances(&Irp->IoStatus, Request, Data, This, FALSE);
//             break;
//         case KSPROPERTY_PIN_NECESSARYINSTANCES:
//             Status = KspHandleNecessaryPropertyInstances(&Irp->IoStatus, Request, Data, This);
//             break;
//
//         case KSPROPERTY_PIN_DATAINTERSECTION:
//             Status = KspHandleDataIntersection(Irp, &Irp->IoStatus, Request, Data, IoStack->Parameters.DeviceIoControl.OutputBufferLength, This);
//             break;
//         default:
//             UNIMPLEMENTED;
//             Status = STATUS_NOT_FOUND;
//     }
//     DPRINT("KspPinPropertyHandler Pins %lu Request->Id %lu Status %lx\n", This->Filter.Descriptor->PinDescriptorsCount, Request->Id, Status);
//
//
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_DispatchDeviceIoControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     IKsFilter * Filter;
//     IKsFilterImpl * This;
//     NTSTATUS Status;
//     PKSFILTER FilterInstance;
//     UNICODE_STRING GuidString;
//     PKSPROPERTY Property;
//     ULONG SetCount = 0;
//     PKSP_NODE NodeProperty;
//     PKSNODE_DESCRIPTOR NodeDescriptor;
//
//     /* obtain filter from object header */
//     Status = IKsFilter_GetFilterFromIrp(Irp, &Filter);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* get our real implementation */
//     This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Header.OuterUnknown);
//
//     /* current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* get property from input buffer */
//     Property = (PKSPROPERTY)IoStack->Parameters.DeviceIoControl.Type3InputBuffer;
//
//     /* get filter instance */
//     FilterInstance = Filter->lpVtbl->GetStruct(Filter);
//
//     /* sanity check */
//     ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(KSIDENTIFIER));
//     ASSERT(FilterInstance);
//     ASSERT(FilterInstance->Descriptor);
//     ASSERT(FilterInstance->Descriptor->AutomationTable);
//
//     /* acquire control mutex */
//     KeWaitForSingleObject(This->Header.ControlMutex, Executive, KernelMode, FALSE, NULL);
//
//     if (IoStack->Parameters.DeviceIoControl.IoControlCode == IOCTL_KS_METHOD)
//     {
//         const KSMETHOD_SET *MethodSet = NULL;
//         ULONG MethodItemSize = 0;
//
//         /* check if the driver supports method sets */
//         if (FilterInstance->Descriptor->AutomationTable->MethodSetsCount)
//         {
//             SetCount = FilterInstance->Descriptor->AutomationTable->MethodSetsCount;
//             MethodSet = FilterInstance->Descriptor->AutomationTable->MethodSets;
//             MethodItemSize = FilterInstance->Descriptor->AutomationTable->MethodItemSize;
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
//         if (Property->Flags & KSPROPERTY_TYPE_TOPOLOGY)
//         {
//             ASSERT(IoStack->Parameters.DeviceIoControl.InputBufferLength >= sizeof(KSP_NODE));
//             NodeProperty = (PKSP_NODE)Property;
//             NodeDescriptor = (PKSNODE_DESCRIPTOR)((ULONG_PTR)FilterInstance->Descriptor->NodeDescriptors + FilterInstance->Descriptor->NodeDescriptorSize * NodeProperty->NodeId);
//             if (NodeDescriptor->AutomationTable != NULL)
//             {
//                 SetCount = NodeDescriptor->AutomationTable->PropertySetsCount;
//                 PropertySet = NodeDescriptor->AutomationTable->PropertySets;
//                 PropertyItemSize = 0;
//             }
//         }
//         else if (FilterInstance->Descriptor->AutomationTable->PropertySetsCount)
//         {
//             SetCount = FilterInstance->Descriptor->AutomationTable->PropertySetsCount;
//             PropertySet = FilterInstance->Descriptor->AutomationTable->PropertySets;
//             PropertyItemSize = FilterInstance->Descriptor->AutomationTable->PropertyItemSize;
//             // FIXME: handle variable sized property items
//             ASSERT(PropertyItemSize == sizeof(KSPROPERTY_ITEM));
//             PropertyItemSize = 0;
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
//                                     FilterInstance->Descriptor->AutomationTable->EventSetsCount,
//                                     (PKSEVENT_SET)FilterInstance->Descriptor->AutomationTable->EventSets,
//                                     &This->Header.EventList,
//                                     KSEVENTS_SPINLOCK,
//                                     (PVOID)&This->Header.EventListLock,
//                                     NULL,
//                                     FilterInstance->Descriptor->AutomationTable->EventItemSize);
//         }
//         else
//         {
//             /* disable event handler */
//             Status = KsDisableEvent(Irp, &This->Header.EventList, KSEVENTS_SPINLOCK, &This->Header.EventListLock);
//         }
//     }
//
//     RtlStringFromGUID(&Property->Set, &GuidString);
//     DPRINT("IKsFilter_DispatchDeviceIoControl property PinCount %x\n", FilterInstance->Descriptor->PinDescriptorsCount);
//     DPRINT("IKsFilter_DispatchDeviceIoControl property Set |%S| Id %u Flags %x Status %lx ResultLength %lu\n", GuidString.Buffer, Property->Id, Property->Flags, Status, Irp->IoStatus.Information);
//     RtlFreeUnicodeString(&GuidString);
//
//     /* release filter */
//     Filter->lpVtbl->Release(Filter);
//
//     /* release control mutex */
//     KeReleaseMutex(This->Header.ControlMutex, FALSE);
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
// static KSDISPATCH_TABLE DispatchTable =
// {
//     IKsFilter_DispatchDeviceIoControl,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     KsDispatchInvalidDeviceRequest,
//     IKsFilter_DispatchClose,
//     KsDispatchQuerySecurity,
//     KsDispatchSetSecurity,
//     KsDispatchFastIoDeviceControlFailure,
//     KsDispatchFastReadFailure,
//     KsDispatchFastReadFailure,
// };
//
//
// NTSTATUS
// IKsFilter_CreateDescriptors(
//     IKsFilterImpl * This,
//     KSFILTER_DESCRIPTOR* FilterDescriptor)
// {
//     ULONG Index = 0;
//     NTSTATUS Status;
//     PKSNODE_DESCRIPTOR NodeDescriptor;
//
//     /* initialize pin descriptors */
//     This->FirstPin = NULL;
//     This->PinInstanceCount = NULL;
//     This->ProcessPinIndex = NULL;
//
//     /* initialize topology descriptor */
//     This->Topology.CategoriesCount = FilterDescriptor->CategoriesCount;
//     This->Topology.Categories = FilterDescriptor->Categories;
//     This->Topology.TopologyNodesCount = FilterDescriptor->NodeDescriptorsCount;
//     This->Topology.TopologyConnectionsCount = FilterDescriptor->ConnectionsCount;
//     This->Topology.TopologyConnections = FilterDescriptor->Connections;
//
//     /* are there any templates */
//     if (FilterDescriptor->PinDescriptorsCount)
//     {
//         /* sanity check */
//         ASSERT(FilterDescriptor->PinDescriptors);
//
//         /* FIXME handle variable sized pin descriptors */
//         ASSERT(FilterDescriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//
//         /* store pin descriptors ex */
//         Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->Filter.Descriptor->PinDescriptors, FilterDescriptor->PinDescriptorSize * FilterDescriptor->PinDescriptorsCount,
//                          FilterDescriptor->PinDescriptorSize * FilterDescriptor->PinDescriptorsCount, 0);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("IKsFilter_CreateDescriptors _KsEdit failed %lx\n", Status);
//             return Status;
//         }
//
//         /* store pin instance count */
//         Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->PinInstanceCount, sizeof(ULONG) * FilterDescriptor->PinDescriptorsCount,
//                          sizeof(ULONG) * FilterDescriptor->PinDescriptorsCount, 0);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("IKsFilter_CreateDescriptors _KsEdit failed %lx\n", Status);
//             return Status;
//         }
//
//         /* store instantiated pin arrays */
//         Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->FirstPin, sizeof(PVOID) * FilterDescriptor->PinDescriptorsCount,
//                          sizeof(PVOID) * FilterDescriptor->PinDescriptorsCount, 0);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("IKsFilter_CreateDescriptors _KsEdit failed %lx\n", Status);
//             return Status;
//         }
//
//         /* add new pin factory */
//         RtlMoveMemory((PVOID)This->Filter.Descriptor->PinDescriptors, FilterDescriptor->PinDescriptors, FilterDescriptor->PinDescriptorSize * FilterDescriptor->PinDescriptorsCount);
//
//         /* allocate process pin index */
//         Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->ProcessPinIndex, sizeof(KSPROCESSPIN_INDEXENTRY) * FilterDescriptor->PinDescriptorsCount,
//                          sizeof(KSPROCESSPIN_INDEXENTRY) * FilterDescriptor->PinDescriptorsCount, 0);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT("IKsFilter_CreateDescriptors _KsEdit failed %lx\n", Status);
//             return Status;
//         }
//
//     }
//
//
//     if (FilterDescriptor->ConnectionsCount)
//     {
//         /* modify connections array */
//         Status = _KsEdit(This->Filter.Bag,
//                         (PVOID*)&This->Filter.Descriptor->Connections,
//                          FilterDescriptor->ConnectionsCount * sizeof(KSTOPOLOGY_CONNECTION),
//                          FilterDescriptor->ConnectionsCount * sizeof(KSTOPOLOGY_CONNECTION),
//                          0);
//
//        This->Topology.TopologyConnections = This->Filter.Descriptor->Connections;
//        This->Topology.TopologyConnectionsCount = ((PKSFILTER_DESCRIPTOR)This->Filter.Descriptor)->ConnectionsCount = FilterDescriptor->ConnectionsCount;
//     }
//
//     if (FilterDescriptor->NodeDescriptorsCount)
//     {
//         /* sanity check */
//         ASSERT(FilterDescriptor->NodeDescriptors);
//
//         /* sanity check */
//         ASSERT(FilterDescriptor->NodeDescriptorSize >= sizeof(KSNODE_DESCRIPTOR));
//
//         This->Topology.TopologyNodes = AllocateItem(NonPagedPool, sizeof(GUID) * FilterDescriptor->NodeDescriptorsCount);
//         /* allocate topology node types array */
//         if (!This->Topology.TopologyNodes)
//         {
//             DPRINT("IKsFilter_CreateDescriptors OutOfMemory TopologyNodesCount %lu\n", FilterDescriptor->NodeDescriptorsCount);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         This->Topology.TopologyNodesNames = AllocateItem(NonPagedPool, sizeof(GUID) * FilterDescriptor->NodeDescriptorsCount);
//         /* allocate topology names array */
//         if (!This->Topology.TopologyNodesNames)
//         {
//             FreeItem((PVOID)This->Topology.TopologyNodes);
//             DPRINT("IKsFilter_CreateDescriptors OutOfMemory TopologyNodesCount %lu\n", FilterDescriptor->NodeDescriptorsCount);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         DPRINT("NodeDescriptorCount %lu\n", FilterDescriptor->NodeDescriptorsCount);
//         NodeDescriptor = (PKSNODE_DESCRIPTOR)FilterDescriptor->NodeDescriptors;
//         for(Index = 0; Index < FilterDescriptor->NodeDescriptorsCount; Index++)
//         {
//             DPRINT("Index %lu Type %p Name %p\n", Index, NodeDescriptor->Type, NodeDescriptor->Name);
//
//             /* copy topology type */
//             if (NodeDescriptor->Type)
//                 RtlMoveMemory((PVOID)&This->Topology.TopologyNodes[Index], NodeDescriptor->Type, sizeof(GUID));
//
//             /* copy topology name */
//             if (NodeDescriptor->Name)
//                 RtlMoveMemory((PVOID)&This->Topology.TopologyNodesNames[Index], NodeDescriptor->Name, sizeof(GUID));
//
//             // next node descriptor
//             NodeDescriptor = (PKSNODE_DESCRIPTOR)((ULONG_PTR)NodeDescriptor + FilterDescriptor->NodeDescriptorSize);
//         }
//     }
//     /* done! */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IKsFilter_CopyFilterDescriptor(
//     IKsFilterImpl * This,
//     const KSFILTER_DESCRIPTOR* FilterDescriptor)
// {
//     NTSTATUS Status;
//     KSAUTOMATION_TABLE AutomationTable;
//
//     This->Filter.Descriptor = AllocateItem(NonPagedPool, sizeof(KSFILTER_DESCRIPTOR));
//     if (!This->Filter.Descriptor)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Status = KsAddItemToObjectBag(This->Filter.Bag, (PVOID)This->Filter.Descriptor, NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         FreeItem((PVOID)This->Filter.Descriptor);
//         This->Filter.Descriptor = NULL;
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* copy filter descriptor fields */
//     RtlMoveMemory((PVOID)This->Filter.Descriptor, FilterDescriptor, sizeof(KSFILTER_DESCRIPTOR));
//
//     /* zero automation table */
//     RtlZeroMemory(&AutomationTable, sizeof(KSAUTOMATION_TABLE));
//
//     /* setup filter property sets */
//     AutomationTable.PropertyItemSize = sizeof(KSPROPERTY_ITEM);
//     AutomationTable.PropertySetsCount = 3;
//     AutomationTable.PropertySets = FilterPropertySet;
//
//     /* merge filter automation table */
//     Status = KsMergeAutomationTables((PKSAUTOMATION_TABLE*)&This->Filter.Descriptor->AutomationTable, (PKSAUTOMATION_TABLE)FilterDescriptor->AutomationTable, &AutomationTable, This->Filter.Bag);
//
//     return Status;
// }
//
//
// VOID
// IKsFilter_AddPin(
//     PKSFILTER Filter,
//     PKSPIN Pin)
// {
//     PKSPIN NextPin, CurPin;
//     PKSBASIC_HEADER BasicHeader;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     /* sanity check */
//     ASSERT(Pin->Id < This->Filter.Descriptor->PinDescriptorsCount);
//
//     if (This->FirstPin[Pin->Id] == NULL)
//     {
//         /* welcome first pin */
//         This->FirstPin[Pin->Id] = Pin;
//         This->PinInstanceCount[Pin->Id]++;
//         return;
//     }
//
//     /* get first pin */
//     CurPin = This->FirstPin[Pin->Id];
//
//     do
//     {
//         /* get next instantiated pin */
//         NextPin = KsPinGetNextSiblingPin(CurPin);
//         if (!NextPin)
//             break;
//
//         NextPin = CurPin;
//
//     }while(NextPin != NULL);
//
//     /* get basic header */
//     BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)CurPin - sizeof(KSBASIC_HEADER));
//
//     /* store pin */
//     BasicHeader->Next.Pin = Pin;
// }
//
// VOID
// IKsFilter_RemovePin(
//     PKSFILTER Filter,
//     PKSPIN Pin)
// {
//     PKSPIN NextPin, CurPin, LastPin;
//     PKSBASIC_HEADER BasicHeader;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     /* sanity check */
//     ASSERT(Pin->Id < This->Filter.Descriptor->PinDescriptorsCount);
//
//     /* get first pin */
//     CurPin = This->FirstPin[Pin->Id];
//
//     LastPin = NULL;
//     do
//     {
//         /* get next instantiated pin */
//         NextPin = KsPinGetNextSiblingPin(CurPin);
//
//         if (CurPin == Pin)
//         {
//             if (LastPin)
//             {
//                 /* get basic header of last pin */
//                 BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)LastPin - sizeof(KSBASIC_HEADER));
//
//                 BasicHeader->Next.Pin = NextPin;
//             }
//             else
//             {
//                 /* erase last pin */
//                 This->FirstPin[Pin->Id] = NextPin;
//             }
//             /* decrement pin instance count */
//             This->PinInstanceCount[Pin->Id]--;
//             return;
//         }
//
//         if (!NextPin)
//             break;
//
//         LastPin = CurPin;
//         NextPin = CurPin;
//
//     }while(NextPin != NULL);
//
//     /* pin not found */
//     ASSERT(0);
// }
//
//
// NTSTATUS
// NTAPI
// IKsFilter_DispatchCreatePin(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     IKsFilterImpl * This;
//     PKSOBJECT_CREATE_ITEM CreateItem;
//     PKSPIN_CONNECT Connect;
//     NTSTATUS Status;
//
//     DPRINT("IKsFilter_DispatchCreatePin\n");
//
//     /* get the create item */
//     CreateItem = KSCREATE_ITEM_IRP_STORAGE(Irp);
//
//     /* get the filter object */
//     This = (IKsFilterImpl*)CreateItem->Context;
//
//     /* sanity check */
//     ASSERT(This->Header.Type == KsObjectTypeFilter);
//
//     /* acquire control mutex */
//     KeWaitForSingleObject(This->Header.ControlMutex, Executive, KernelMode, FALSE, NULL);
//
//     /* now validate the connect request */
//     Status = KspValidateConnectRequest(Irp, This->Filter.Descriptor->PinDescriptorsCount, (PVOID)This->Filter.Descriptor->PinDescriptors, This->Filter.Descriptor->PinDescriptorSize, &Connect);
//
//     DPRINT("IKsFilter_DispatchCreatePin KsValidateConnectRequest %lx\n", Status);
//
//     if (NT_SUCCESS(Status))
//     {
//         /* sanity check */
//         ASSERT(Connect->PinId < This->Filter.Descriptor->PinDescriptorsCount);
//
//         DPRINT("IKsFilter_DispatchCreatePin KsValidateConnectRequest PinId %lu CurrentInstanceCount %lu MaxPossible %lu\n", Connect->PinId,
//                This->PinInstanceCount[Connect->PinId],
//                This->Filter.Descriptor->PinDescriptors[Connect->PinId].InstancesPossible);
//
//         if (This->PinInstanceCount[Connect->PinId] < This->Filter.Descriptor->PinDescriptors[Connect->PinId].InstancesPossible)
//         {
//             /* create the pin */
//             Status = KspCreatePin(DeviceObject, Irp, This->Header.KsDevice, This->FilterFactory, (IKsFilter*)&This->Header.OuterUnknown, Connect, (KSPIN_DESCRIPTOR_EX*)&This->Filter.Descriptor->PinDescriptors[Connect->PinId]);
//
//             DPRINT("IKsFilter_DispatchCreatePin  KspCreatePin %lx\n", Status);
//         }
//         else
//         {
//             /* maximum instance count reached, bye-bye */
//             Status = STATUS_UNSUCCESSFUL;
//             DPRINT("IKsFilter_DispatchCreatePin  MaxInstance %lu CurInstance %lu %lx\n", This->Filter.Descriptor->PinDescriptors[Connect->PinId].InstancesPossible, This->PinInstanceCount[Connect->PinId]);
//         }
//     }
//
//     /* release control mutex */
//     KeReleaseMutex(This->Header.ControlMutex, FALSE);
//
//     if (Status != STATUS_PENDING)
//     {
//         /* complete request */
//         Irp->IoStatus.Status = Status;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     /* done */
//     DPRINT("IKsFilter_DispatchCreatePin Result %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// IKsFilter_DispatchCreateNode(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     UNIMPLEMENTED;
//     Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//     CompleteRequest(Irp, IO_NO_INCREMENT);
//     return STATUS_UNSUCCESSFUL;
// }
//
//
// VOID
// IKsFilter_AttachFilterToFilterFactory(
//     IKsFilterImpl * This,
//     PKSFILTERFACTORY FilterFactory)
// {
//     PKSBASIC_HEADER BasicHeader;
//     PKSFILTER Filter;
//
//
//     /* get filter factory basic header */
//     BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)FilterFactory - sizeof(KSBASIC_HEADER));
//
//     /* sanity check */
//     ASSERT(BasicHeader->Type == KsObjectTypeFilterFactory);
//
//     if (BasicHeader->FirstChild.FilterFactory == NULL)
//     {
//         /* welcome first instantiated filter */
//         BasicHeader->FirstChild.Filter = &This->Filter;
//         return;
//     }
//
//     /* set to first entry */
//     Filter = BasicHeader->FirstChild.Filter;
//
//     do
//     {
//         /* get basic header */
//         BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)Filter - sizeof(KSBASIC_HEADER));
//         /* sanity check */
//         ASSERT(BasicHeader->Type == KsObjectTypeFilter);
//
//         if (BasicHeader->Next.Filter)
//         {
//             /* iterate to next filter factory */
//             Filter = BasicHeader->Next.Filter;
//         }
//         else
//         {
//             /* found last entry */
//             break;
//         }
//     }while(TRUE);
//
//     /* attach filter factory */
//     BasicHeader->Next.Filter = &This->Filter;
// }
//
// VOID
// IKsFilter_RemoveFilterFromFilterFactory(
//     IKsFilterImpl * This,
//     PKSFILTERFACTORY FilterFactory)
// {
//     PKSBASIC_HEADER BasicHeader;
//     PKSFILTER Filter, LastFilter;
//
//     /* get filter factory basic header */
//     BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)FilterFactory - sizeof(KSBASIC_HEADER));
//
//     /* sanity check */
//     ASSERT(BasicHeader->Type == KsObjectTypeFilterFactory);
//     ASSERT(BasicHeader->FirstChild.Filter != NULL);
//
//
//     /* set to first entry */
//     Filter = BasicHeader->FirstChild.Filter;
//     LastFilter = NULL;
//
//     do
//     {
//          if (Filter == &This->Filter)
//          {
//              if (LastFilter)
//              {
//                  /* get basic header */
//                  BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)LastFilter - sizeof(KSBASIC_HEADER));
//                  /* remove filter instance */
//                  BasicHeader->Next.Filter = This->Header.Next.Filter;
//                  break;
//              }
//              else
//              {
//                  /* remove filter instance */
//                  BasicHeader->FirstChild.Filter = This->Header.Next.Filter;
//                  break;
//              }
//          }
//
//         /* get basic header */
//         BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)Filter - sizeof(KSBASIC_HEADER));
//         /* sanity check */
//         ASSERT(BasicHeader->Type == KsObjectTypeFilter);
//
//         LastFilter = Filter;
//         if (BasicHeader->Next.Filter)
//         {
//             /* iterate to next filter factory */
//             Filter = BasicHeader->Next.Filter;
//         }
//         else
//         {
//             /* filter is not in list */
//             ASSERT(0);
//             break;
//         }
//     }while(TRUE);
// }
//
// VOID
// NTAPI
// IKsFilter_FilterCentricWorker(
//     IN PVOID Ctx)
// {
//     IKsProcessingObject * Object = (IKsProcessingObject*)Ctx;
//
//     /* sanity check */
//     ASSERT(Object);
//
//     /* perform work */
//     Object->lpVtbl->ProcessingObjectWork(Object);
// }
//
// NTSTATUS
// NTAPI
// KspCreateFilter(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN IKsFilterFactory *iface)
// {
//     IKsFilterImpl * This;
//     IKsDevice *KsDevice;
//     PKSFILTERFACTORY Factory;
//     PIO_STACK_LOCATION IoStack;
//     PDEVICE_EXTENSION DeviceExtension;
//     NTSTATUS Status;
//     PKSOBJECT_CREATE_ITEM CreateItem;
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* get the filter factory */
//     Factory = iface->lpVtbl->GetStruct(iface);
//
//     if (!Factory || !Factory->FilterDescriptor)
//     {
//         /* Sorry it just will not work */
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     if (Factory->FilterDescriptor->Flags & KSFILTER_FLAG_DENY_USERMODE_ACCESS)
//     {
//         if (Irp->RequestorMode == UserMode)
//         {
//             /* filter not accessible from user mode */
//             DPRINT1("Access denied\n");
//             return STATUS_UNSUCCESSFUL;
//         }
//     }
//
//     /* allocate filter instance */
//     This = AllocateItem(NonPagedPool, sizeof(IKsFilterImpl));
//     if (!This)
//     {
//         DPRINT1("KspCreateFilter OutOfMemory\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize object bag */
//     This->Filter.Bag = AllocateItem(NonPagedPool, sizeof(KSIOBJECT_BAG));
//     if (!This->Filter.Bag)
//     {
//         /* no memory */
//         FreeItem(This);
//         DPRINT1("KspCreateFilter OutOfMemory\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     KsDevice = (IKsDevice*)&DeviceExtension->DeviceHeader->BasicHeader.OuterUnknown;
//     KsDevice->lpVtbl->InitializeObjectBag(KsDevice, (PKSIOBJECT_BAG)This->Filter.Bag, NULL);
//
//     /* copy filter descriptor */
//     Status = IKsFilter_CopyFilterDescriptor(This, Factory->FilterDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         /* not enough memory */
//         FreeItem(This->Filter.Bag);
//         FreeItem(This);
//         DPRINT("KspCreateFilter IKsFilter_CopyFilterDescriptor failed %lx\n", Status);
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* get current irp stack */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* allocate create items */
//     CreateItem = AllocateItem(NonPagedPool, sizeof(KSOBJECT_CREATE_ITEM) * 2);
//     if (!CreateItem)
//     {
//         /* no memory */
//         FreeItem(This->Filter.Bag);
//         FreeItem(This);
//         DPRINT1("KspCreateFilter OutOfMemory\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize pin create item */
//     CreateItem[0].Create = IKsFilter_DispatchCreatePin;
//     CreateItem[0].Context = (PVOID)This;
//     CreateItem[0].Flags = KSCREATE_ITEM_FREEONSTOP;
//     RtlInitUnicodeString(&CreateItem[0].ObjectClass, KSSTRING_Pin);
//     /* initialize node create item */
//     CreateItem[1].Create = IKsFilter_DispatchCreateNode;
//     CreateItem[1].Context = (PVOID)This;
//     CreateItem[1].Flags = KSCREATE_ITEM_FREEONSTOP;
//     RtlInitUnicodeString(&CreateItem[1].ObjectClass, KSSTRING_TopologyNode);
//
//
//     /* initialize filter instance */
//     This->ref = 1;
//     This->Header.OuterUnknown = (PUNKNOWN)&vt_IKsFilter;
//     This->lpVtblKsControl = &vt_IKsControl;
//     This->lpVtblKsProcessingObject = &vt_IKsProcessingObject;
//
//     This->Factory = Factory;
//     This->FilterFactory = iface;
//     This->FileObject = IoStack->FileObject;
//     KeInitializeMutex(&This->ProcessingMutex, 0);
//
//     /* initialize basic header */
//     This->Header.KsDevice = &DeviceExtension->DeviceHeader->KsDevice;
//     This->Header.Parent.KsFilterFactory = iface->lpVtbl->GetStruct(iface);
//     This->Header.Type = KsObjectTypeFilter;
//     This->Header.ControlMutex = &This->ControlMutex;
//     KeInitializeMutex(This->Header.ControlMutex, 0);
//     InitializeListHead(&This->Header.EventList);
//     KeInitializeSpinLock(&This->Header.EventListLock);
//
//     /* initialize and gate */
//     KsGateInitializeAnd(&This->Gate, NULL);
//
//     /* FIXME initialize and gate based on pin flags */
//
//     /* initialize work item */
//     ExInitializeWorkItem(&This->WorkItem, IKsFilter_FilterCentricWorker, (PVOID)This->lpVtblKsProcessingObject);
//
//     /* allocate counted work item */
//     Status = KsRegisterCountedWorker(HyperCriticalWorkQueue, &This->WorkItem, &This->Worker);
//     if (!NT_SUCCESS(Status))
//     {
//         /* what can go wrong, goes wrong */
//         DPRINT1("KsRegisterCountedWorker failed with %lx\n", Status);
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return Status;
//     }
//
//     /* allocate the stream descriptors */
//     Status = IKsFilter_CreateDescriptors(This, (PKSFILTER_DESCRIPTOR)Factory->FilterDescriptor);
//     if (!NT_SUCCESS(Status))
//     {
//         /* what can go wrong, goes wrong */
//         DPRINT1("IKsFilter_CreateDescriptors failed with %lx\n", Status);
//         KsUnregisterWorker(This->Worker);
//         FreeItem(This);
//         FreeItem(CreateItem);
//         return Status;
//     }
//
//
//
//     /* does the filter have a filter dispatch */
//     if (Factory->FilterDescriptor->Dispatch)
//     {
//         /* does it have a create routine */
//         if (Factory->FilterDescriptor->Dispatch->Create)
//         {
//             /* now let driver initialize the filter instance */
//
//             ASSERT(This->Header.KsDevice);
//             ASSERT(This->Header.KsDevice->Started);
//             Status = Factory->FilterDescriptor->Dispatch->Create(&This->Filter, Irp);
//
//             if (!NT_SUCCESS(Status) && Status != STATUS_PENDING)
//             {
//                 /* driver failed to initialize */
//                 DPRINT1("Driver: Status %x\n", Status);
//
//                 /* free filter instance */
//                 KsUnregisterWorker(This->Worker);
//                 FreeItem(This);
//                 FreeItem(CreateItem);
//                 return Status;
//             }
//         }
//     }
//
//     /* now allocate the object header */
//     Status = KsAllocateObjectHeader((PVOID*)&This->ObjectHeader, 2, CreateItem, Irp, &DispatchTable);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed to allocate object header */
//         DPRINT1("Failed to allocate object header %x\n", Status);
//
//         return Status;
//     }
//
//     /* initialize object header extra fields */
//     This->ObjectHeader->Type = KsObjectTypeFilter;
//     This->ObjectHeader->Unknown = (PUNKNOWN)&This->Header.OuterUnknown;
//     This->ObjectHeader->ObjectType = (PVOID)&This->Filter;
//
//     /* attach filter to filter factory */
//     IKsFilter_AttachFilterToFilterFactory(This, This->Header.Parent.KsFilterFactory);
//
//     /* completed initialization */
//     DPRINT1("KspCreateFilter done %lx KsDevice %p\n", Status, This->Header.KsDevice);
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFilterAcquireProcessingMutex(
//     IN PKSFILTER Filter)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     KeWaitForSingleObject(&This->ProcessingMutex, Executive, KernelMode, FALSE, NULL);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFilterReleaseProcessingMutex(
//     IN PKSFILTER Filter)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     KeReleaseMutex(&This->ProcessingMutex, FALSE);
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterAddTopologyConnections (
//     IN PKSFILTER Filter,
//     IN ULONG NewConnectionsCount,
//     IN const KSTOPOLOGY_CONNECTION *const NewTopologyConnections)
// {
//     ULONG Count;
//     NTSTATUS Status;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     DPRINT("KsFilterAddTopologyConnections\n");
//
//     ASSERT(This->Filter.Descriptor);
//     Count = This->Filter.Descriptor->ConnectionsCount + NewConnectionsCount;
//
//
//     /* modify connections array */
//     Status = _KsEdit(This->Filter.Bag,
//                     (PVOID*)&This->Filter.Descriptor->Connections,
//                      Count * sizeof(KSTOPOLOGY_CONNECTION),
//                      This->Filter.Descriptor->ConnectionsCount * sizeof(KSTOPOLOGY_CONNECTION),
//                      0);
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         DPRINT("KsFilterAddTopologyConnections KsEdit failed with %lx\n", Status);
//         return Status;
//     }
//
//     /* FIXME verify connections */
//
//     /* copy new connections */
//     RtlMoveMemory((PVOID)&This->Filter.Descriptor->Connections[This->Filter.Descriptor->ConnectionsCount],
//                   NewTopologyConnections,
//                   NewConnectionsCount * sizeof(KSTOPOLOGY_CONNECTION));
//
//     /* update topology */
//     This->Topology.TopologyConnectionsCount += NewConnectionsCount;
//     ((PKSFILTER_DESCRIPTOR)This->Filter.Descriptor)->ConnectionsCount += NewConnectionsCount;
//     This->Topology.TopologyConnections = This->Filter.Descriptor->Connections;
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFilterAttemptProcessing(
//     IN PKSFILTER Filter,
//     IN BOOLEAN Asynchronous)
// {
//     PKSGATE Gate;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     /* get gate */
//     Gate = This->lpVtblKsProcessingObject->GetAndGate((IKsProcessingObject*)This->lpVtblKsProcessingObject);
//
//     if (!KsGateCaptureThreshold(Gate))
//     {
//         /* filter control gate is closed */
//         return;
//     }
// DPRINT1("processing\n");
//     /* try initiate processing */
//     This->lpVtblKsProcessingObject->Process((IKsProcessingObject*)This->lpVtblKsProcessingObject, Asynchronous);
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterCreateNode (
//     IN PKSFILTER Filter,
//     IN const KSNODE_DESCRIPTOR *const NodeDescriptor,
//     OUT PULONG NodeID)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterCreatePinFactory (
//     IN PKSFILTER Filter,
//     IN const KSPIN_DESCRIPTOR_EX *const InPinDescriptor,
//     OUT PULONG PinID)
// {
//     ULONG Count;
//     NTSTATUS Status;
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     DPRINT("KsFilterCreatePinFactory\n");
//
//     /* calculate new count */
//     Count = This->Filter.Descriptor->PinDescriptorsCount + 1;
//
//     /* sanity check */
//     ASSERT(This->Filter.Descriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//
//     /* modify pin descriptors ex array */
//     Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->Filter.Descriptor->PinDescriptors, Count * This->Filter.Descriptor->PinDescriptorSize, This->Filter.Descriptor->PinDescriptorsCount * This->Filter.Descriptor->PinDescriptorSize, 0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         DPRINT("KsFilterCreatePinFactory _KsEdit failed with %lx\n", Status);
//         return Status;
//     }
//
//     /* modify pin instance count array */
//     Status = _KsEdit(This->Filter.Bag,(PVOID*)&This->PinInstanceCount, sizeof(ULONG) * Count, sizeof(ULONG) * This->Filter.Descriptor->PinDescriptorsCount, 0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         DPRINT("KsFilterCreatePinFactory _KsEdit failed with %lx\n", Status);
//         return Status;
//     }
//
//     /* modify first pin array */
//     Status = _KsEdit(This->Filter.Bag,(PVOID*)&This->FirstPin, sizeof(PVOID) * Count, sizeof(PVOID) * This->Filter.Descriptor->PinDescriptorsCount, 0);
//     if (!NT_SUCCESS(Status))
//     {
//         /* failed */
//         DPRINT("KsFilterCreatePinFactory _KsEdit failed with %lx\n", Status);
//         return Status;
//     }
//
//     /* add new pin factory */
//     RtlMoveMemory((PVOID)&This->Filter.Descriptor->PinDescriptors[This->Filter.Descriptor->PinDescriptorsCount], InPinDescriptor, sizeof(KSPIN_DESCRIPTOR_EX));
//
//     /* allocate process pin index */
//     Status = _KsEdit(This->Filter.Bag, (PVOID*)&This->ProcessPinIndex, sizeof(KSPROCESSPIN_INDEXENTRY) * Count,
//                      sizeof(KSPROCESSPIN_INDEXENTRY) * This->Filter.Descriptor->PinDescriptorsCount, 0);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT("KsFilterCreatePinFactory _KsEdit failed %lx\n", Status);
//         return Status;
//     }
//
//     /* store new pin id */
//     *PinID = This->Filter.Descriptor->PinDescriptorsCount;
//
//     /* increment pin descriptor count */
//     ((PKSFILTER_DESCRIPTOR)This->Filter.Descriptor)->PinDescriptorsCount++;
//
//
//     DPRINT("KsFilterCreatePinFactory done\n");
//     return STATUS_SUCCESS;
//
// }
//
// /*
//     @unimplemented
// */
// KSDDKAPI
// PKSGATE
// NTAPI
// KsFilterGetAndGate(
//     IN PKSFILTER Filter)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     /* return and-gate */
//     return &This->Gate;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// ULONG
// NTAPI
// KsFilterGetChildPinCount(
//     IN PKSFILTER Filter,
//     IN ULONG PinId)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     if (PinId >= This->Filter.Descriptor->PinDescriptorsCount)
//     {
//         /* index is out of bounds */
//         return 0;
//     }
//     /* return pin instance count */
//     return This->PinInstanceCount[PinId];
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSPIN
// NTAPI
// KsFilterGetFirstChildPin(
//     IN PKSFILTER Filter,
//     IN ULONG PinId)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     if (PinId >= This->Filter.Descriptor->PinDescriptorsCount)
//     {
//         /* index is out of bounds */
//         return NULL;
//     }
//
//     /* return first pin index */
//     return This->FirstPin[PinId];
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// VOID
// NTAPI
// KsFilterRegisterPowerCallbacks(
//     IN PKSFILTER Filter,
//     IN PFNKSFILTERPOWER Sleep OPTIONAL,
//     IN PFNKSFILTERPOWER Wake OPTIONAL)
// {
//     IKsFilterImpl * This = (IKsFilterImpl*)CONTAINING_RECORD!(Filter, IKsFilterImpl, Filter);
//
//     This->Sleep = Sleep;
//     This->Wake = Wake;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// PKSFILTER
// NTAPI
// KsGetFilterFromIrp(
//     IN PIRP Irp)
// {
//     PIO_STACK_LOCATION IoStack;
//     PKSIOBJECT_HEADER ObjectHeader;
//
//     DPRINT("KsGetFilterFromIrp\n");
//
//     /* get current irp stack location */
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     /* sanity check */
//     ASSERT(IoStack->FileObject);
//
//     /* get object header */
//     ObjectHeader = (PKSIOBJECT_HEADER)IoStack->FileObject->FsContext2;
//
//     if (ObjectHeader->Type == KsObjectTypeFilter)
//     {
//         /* irp is targeted at the filter */
//         return (PKSFILTER)ObjectHeader->ObjectType;
//     }
//     else if (ObjectHeader->Type == KsObjectTypePin)
//     {
//         /* irp is for a pin */
//         return KsPinGetParentFilter((PKSPIN)ObjectHeader->ObjectType);
//     }
//     else
//     {
//         /* irp is unappropiate to retrieve a filter */
//         return NULL;
//     }
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
