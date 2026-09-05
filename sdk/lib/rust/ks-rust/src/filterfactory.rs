use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CLSID, GUID, IoOpenDeviceInterfaceRegistryKey, KeInitializeMutex, KeInitializeSpinLock, PDEVICE_OBJECT, PIRP, PSECURITY_DESCRIPTOR, PUNICODE_STRING, RtlCompareMemory, RtlFreeUnicodeString, RtlInitUnicodeString, RtlStringFromGUID, ZwClose, ZwSetValueKey};
use core::ffi::c_void;

// ---- ks-rust integration glue (filterfactory.rs-local): imports, aliases, helper shims ----
// Shared allocator/completion helpers (defined #[no_mangle] in misc.rs).
use crate::misc::{AllocateItem, FreeItem, CompleteRequest};
// Cross-module Ks* handlers (defined #[no_mangle] in sibling ks modules).
use crate::deviceinterface::{KspSetDeviceInterfacesState, KspFreeDeviceInterfaces, KspRegisterDeviceInterfaces};
use crate::filter::KspCreateFilter;
// header-sourced constants (drivers/ksfilter/ks/precomp.h) — used only in this module.
const REG_PINFLAG_B_MANY: ULONG = 0x4;
const MERIT_DO_NOT_USE: ULONG = 0x200000;

// IsEqualGUIDAligned: guiddef inline GUID equality.
#[inline(always)]
unsafe fn IsEqualGUIDAligned(a: *const GUID, b: *const GUID) -> bool {
    (*a).Data1 == (*b).Data1
        && (*a).Data2 == (*b).Data2
        && (*a).Data3 == (*b).Data3
        && (*a).Data4 == (*b).Data4
}

// RTL_CONSTANT_STRING(L!("...")) helper: wide static -> _UNICODE_STRING (lengths in bytes).
#[inline(always)]
unsafe fn rtl_const_string(w: &'static [u16]) -> _UNICODE_STRING {
    _UNICODE_STRING {
        Length: ((w.len() - 1) * 2) as u16,
        MaximumLength: (w.len() * 2) as u16,
        Buffer: w.as_ptr() as *mut u16,
    }
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// The bindgen vtbl slots are `extern "stdcall"` while our handlers are `extern "system"`
// (identical ABI on i386, distinct Rust types). `cb!` would `as usize`, which const-eval
// forbids; instead coerce each fn item to its `extern "system"` pointer (valid coercion)
// and transmute the (pointer-sized) value into the slot's Option<fn> — const-eval safe.
static mut vt_IKsFilterFactoryVtbl: IKsFilterFactoryVtbl = unsafe {
    IKsFilterFactoryVtbl {
        QueryInterface: core::mem::transmute(
            IKsFilterFactory_fnQueryInterface
                as unsafe extern "system" fn(*mut IKsFilterFactory, *const IID, *mut PVOID) -> NTSTATUS,
        ),
        AddRef: core::mem::transmute(
            IKsFilterFactory_fnAddRef as unsafe extern "system" fn(*mut IKsFilterFactory) -> ULONG,
        ),
        Release: core::mem::transmute(
            IKsFilterFactory_fnRelease as unsafe extern "system" fn(*mut IKsFilterFactory) -> ULONG,
        ),
        GetStruct: core::mem::transmute(
            IKsFilterFactory_fnGetStruct
                as unsafe extern "system" fn(*mut IKsFilterFactory) -> *mut KSFILTERFACTORY,
        ),
        SetDeviceClassesState: core::mem::transmute(
            IKsFilterFactory_fnSetDeviceClassesState
                as unsafe extern "system" fn(*mut IKsFilterFactory, BOOLEAN) -> NTSTATUS,
        ),
        Initialize: core::mem::transmute(
            IKsFilterFactory_fnInitialize
                as unsafe extern "system" fn(
                    *mut IKsFilterFactory,
                    PDEVICE_OBJECT,
                    *const KSFILTER_DESCRIPTOR,
                    PWSTR,
                    PSECURITY_DESCRIPTOR,
                    ULONG,
                    PFNKSFILTERFACTORYPOWER,
                    PFNKSFILTERFACTORYPOWER,
                    *mut PKSFILTERFACTORY,
                ) -> NTSTATUS,
        ),
    }
};

#[repr(C)]
pub struct IKsFilterFactoryImpl {
    pub Header: KSBASIC_HEADER,
    pub FilterFactory: KSFILTERFACTORY,
    pub ref_: LONG,
    pub DeviceHeader: PKSIDEVICE_HEADER,
    pub SleepCallback: PFNKSFILTERFACTORYPOWER,
    pub WakeCallback: PFNKSFILTERFACTORYPOWER,
    pub SymbolicLinkList: LIST_ENTRY,
    pub ControlMutex: KMUTEX,
}

pub type PIKsFilterFactoryImpl = *mut IKsFilterFactoryImpl;

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_ItemFreeCb(
    CreateItem: PKSOBJECT_CREATE_ITEM,
) {
    let iface = CONTAINING_RECORD!(
        (*CreateItem).Context,
        IKsFilterFactoryImpl,
        FilterFactory,
    ) as *mut IKsFilterFactory;
    (*(*iface).lpVtbl).Release.expect("non-null function pointer")(iface);
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_Create(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let CreateItem: PKSOBJECT_CREATE_ITEM;
    let Factory: *mut IKsFilterFactoryImpl;
    let iface: *mut IKsFilterFactory;
    let Status: NTSTATUS;

    CreateItem = *((*Irp).Tail.Overlay.__bindgen_anon_1.__bindgen_anon_1.DriverContext.as_mut_ptr() as *mut PKSOBJECT_CREATE_ITEM);
    if CreateItem.is_null() {
        return STATUS_UNSUCCESSFUL;
    }

    Factory = ((*CreateItem).Context as usize).wrapping_sub(
        core::mem::offset_of!(IKsFilterFactoryImpl, FilterFactory),
    ) as *mut IKsFilterFactoryImpl;

    iface = &raw mut (*Factory).Header.OuterUnknown as *mut IKsFilterFactory;

    Status = KspCreateFilter(DeviceObject, Irp, iface);

    if Status != STATUS_PENDING {
        (*Irp).IoStatus.Information = 0;
        (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
        CompleteRequest(Irp, IO_NO_INCREMENT as i8);
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnQueryInterface(
    iface: *mut IKsFilterFactory,
    refiid: *const IID,
    Output: *mut *mut core::ffi::c_void,
) -> NTSTATUS {
    let This = CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
    
    if IsEqualGUIDAligned(refiid, &IID_IUnknown) {
        *Output = &mut (*This).Header.OuterUnknown as *mut _ as *mut core::ffi::c_void;
        InterlockedIncrement(&mut (*This).ref_);
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
    
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnAddRef(
    iface: *mut IKsFilterFactory,
) -> ULONG {
    let This = CONTAINING_RECORD!(
        iface,
        IKsFilterFactoryImpl,
        Header.OuterUnknown,
    ) as *mut IKsFilterFactoryImpl;
    InterlockedIncrement(&mut (*This).ref_) as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnRelease(
    iface: *mut IKsFilterFactory,
) -> ULONG {
    let This: *mut IKsFilterFactoryImpl = CONTAINING_RECORD!(
        iface,
        IKsFilterFactoryImpl,
        Header.OuterUnknown,
    );

    InterlockedDecrement(&mut (*This).ref_);

    if (*This).ref_ == 0 {
        if !IsListEmpty(&mut (*This).SymbolicLinkList) {
            KspSetDeviceInterfacesState(&mut (*This).SymbolicLinkList, FALSE as i32);
            KspFreeDeviceInterfaces(&mut (*This).SymbolicLinkList);
        }

        FreeItem(This as *mut core::ffi::c_void);
        return 0;
    }

    (*This).ref_ as ULONG
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnGetStruct(
    iface: *mut IKsFilterFactory,
) -> *mut KSFILTERFACTORY {
    let This: *mut IKsFilterFactoryImpl = CONTAINING_RECORD!(
        iface,
        IKsFilterFactoryImpl,
        Header.OuterUnknown,
    );
    &mut (*This).FilterFactory
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnSetDeviceClassesState(
    iface: *mut IKsFilterFactory,
    Enable: u8,
) -> i32 {
    let This = ((iface as usize)
        .wrapping_sub(
            core::mem::offset_of!(IKsFilterFactoryImpl, Header.OuterUnknown),
        )) as *mut IKsFilterFactoryImpl;
    return KspSetDeviceInterfacesState(
        core::ptr::addr_of_mut!((*This).SymbolicLinkList),
        Enable as i32,
    );
}
#[no_mangle]
pub unsafe extern "C" fn IKsFilterFactory_AttachFilterFactoryToDeviceHeader(
    This: *mut IKsFilterFactoryImpl,
    DeviceHeader: PKSIDEVICE_HEADER,
) {
    let mut BasicHeader: PKSBASIC_HEADER = core::ptr::null_mut();
    let mut FilterFactory: PKSFILTERFACTORY = core::ptr::null_mut();

    if (*DeviceHeader).BasicHeader.FirstChild.FilterFactory.is_null() {
        (*DeviceHeader).BasicHeader.FirstChild.FilterFactory = &mut (*This).FilterFactory;
        return;
    }

    FilterFactory = (*DeviceHeader).BasicHeader.FirstChild.FilterFactory;

    loop {
        BasicHeader = (FilterFactory as usize)
            .wrapping_sub(core::mem::size_of::<KSBASIC_HEADER>())
            as PKSBASIC_HEADER;

        if (*BasicHeader).Next.FilterFactory.is_null() {
            break;
        }
        FilterFactory = (*BasicHeader).Next.FilterFactory;
        if FilterFactory.is_null() {
            break;
        }
    }

    (*BasicHeader).Next.FilterFactory = &mut (*This).FilterFactory;
}

#[no_mangle]
pub unsafe extern "system" fn IKsFilterFactory_fnInitialize(
    iface: *mut IKsFilterFactory,
    DeviceObject: PDEVICE_OBJECT,
    Descriptor: *const KSFILTER_DESCRIPTOR,
    RefString: PWSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CreateItemFlags: ULONG,
    SleepCallback: PFNKSFILTERFACTORYPOWER,
    WakeCallback: PFNKSFILTERFACTORYPOWER,
    FilterFactory: *mut PKSFILTERFACTORY,
) -> NTSTATUS {
    let mut ReferenceString: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let DeviceExtension: PDEVICE_EXTENSION;
    let mut CreateItem: KSOBJECT_CREATE_ITEM = core::mem::zeroed();
    let mut FreeString: BOOLEAN = 0u8;
    let mut KsDevice: *mut IKsDevice;

    let This: *mut IKsFilterFactoryImpl = CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);

    DeviceExtension = (*DeviceObject).DeviceExtension as PDEVICE_EXTENSION;

    (*This).SleepCallback = SleepCallback;
    (*This).WakeCallback = WakeCallback;
    (*This).Header.KsDevice = &mut (*(*DeviceExtension).DeviceHeader).KsDevice;
    (*This).Header.Type = KsObjectTypeFilterFactory;
    (*This).Header.Parent.KsDevice = &mut (*(*DeviceExtension).DeviceHeader).KsDevice;
    (*This).DeviceHeader = (*DeviceExtension).DeviceHeader;

    (*This).FilterFactory.FilterDescriptor = AllocateItem(NonPagedPool, core::mem::size_of::<KSFILTER_DESCRIPTOR>()) as *const KSFILTER_DESCRIPTOR;
    if (*This).FilterFactory.FilterDescriptor.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    core::ptr::copy_nonoverlapping(
        Descriptor as *const u8,
        (*This).FilterFactory.FilterDescriptor as *mut u8,
        core::mem::size_of::<KSFILTER_DESCRIPTOR>(),
    );

    (*This).Header.ControlMutex = &mut (*This).ControlMutex;
    KeInitializeMutex((*This).Header.ControlMutex, 0);

    InitializeListHead(&mut (*This).Header.EventList);
    KeInitializeSpinLock(&mut (*This).Header.EventListLock);

    InitializeListHead(&mut (*This).SymbolicLinkList);

    if !RefString.is_null() || (*Descriptor).ReferenceGuid.is_null() {
        RtlInitUnicodeString(&mut ReferenceString, RefString);
    } else {
        Status = RtlStringFromGUID((*Descriptor).ReferenceGuid, &mut ReferenceString);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        FreeString = 1u8;
    }

    Status = KspRegisterDeviceInterfaces(
        (*(*DeviceExtension).DeviceHeader).KsDevice.PhysicalDeviceObject,
        (*Descriptor).CategoriesCount,
        (*Descriptor).Categories,
        &mut ReferenceString,
        &mut (*This).SymbolicLinkList,
    );
    if !NT_SUCCESS(Status) {
        if FreeString != 0 {
            RtlFreeUnicodeString(&mut ReferenceString);
        }
        return Status;
    }

    CreateItem.SecurityDescriptor = SecurityDescriptor;
    CreateItem.Flags = CreateItemFlags;
    CreateItem.Create = cb!(IKsFilterFactory_Create);
    CreateItem.Context = &mut (*This).FilterFactory as *mut _ as PVOID;
    RtlInitUnicodeString(&mut CreateItem.ObjectClass, ReferenceString.Buffer);

    Status = KsAllocateObjectCreateItem(
        (*DeviceExtension).DeviceHeader as KSDEVICE_HEADER,
        &mut CreateItem,
        1u8,
        cb!(IKsFilterFactory_ItemFreeCb),
    );

    if FreeString != 0 {
        RtlFreeUnicodeString(&mut ReferenceString);
    }

    (*This).FilterFactory.Bag = AllocateItem(NonPagedPool, core::mem::size_of::<KSIOBJECT_BAG>()) as KSOBJECT_BAG;
    if !(*This).FilterFactory.Bag.is_null() {
        KsDevice = core::ptr::addr_of_mut!((*(*DeviceExtension).DeviceHeader).BasicHeader.OuterUnknown) as *mut IKsDevice;
        (*(*KsDevice).lpVtbl).InitializeObjectBag.expect("non-null function pointer")(
            KsDevice,
            (*This).FilterFactory.Bag as PKSIOBJECT_BAG,
            core::ptr::null_mut(),
        );
    }

    if !FilterFactory.is_null() {
        *FilterFactory = &mut (*This).FilterFactory;
    }

    IKsFilterFactory_AttachFilterFactoryToDeviceHeader(This, (*DeviceExtension).DeviceHeader);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KspCreateFilterFactory(
    DeviceObject: PDEVICE_OBJECT,
    Descriptor: *const KSFILTER_DESCRIPTOR,
    RefString: PWSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CreateItemFlags: ULONG,
    SleepCallback: PFNKSFILTERFACTORYPOWER,
    WakeCallback: PFNKSFILTERFACTORYPOWER,
    FilterFactory: *mut PKSFILTERFACTORY,
) -> NTSTATUS {
    let This: *mut IKsFilterFactoryImpl;
    let Filter: *mut IKsFilterFactory;
    let Status: NTSTATUS;

    This = AllocateItem(NonPagedPool, core::mem::size_of::<IKsFilterFactoryImpl>()) as *mut IKsFilterFactoryImpl;
    if This.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*This).ref_ = 1;
    (*This).Header.OuterUnknown = core::ptr::addr_of_mut!(vt_IKsFilterFactoryVtbl) as PUNKNOWN;

    Filter = core::ptr::addr_of_mut!((*This).Header.OuterUnknown) as *mut IKsFilterFactory;

    Status = (*(*Filter).lpVtbl).Initialize.unwrap()(
        Filter,
        DeviceObject,
        Descriptor,
        RefString,
        SecurityDescriptor,
        CreateItemFlags,
        SleepCallback,
        WakeCallback,
        FilterFactory,
    );

    if !NT_SUCCESS(Status) {
        (*(*Filter).lpVtbl).Release.unwrap()(Filter);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateFilterFactory(
    DeviceObject: PDEVICE_OBJECT,
    Descriptor: *const KSFILTER_DESCRIPTOR,
    RefString: PWSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CreateItemFlags: ULONG,
    SleepCallback: PFNKSFILTERFACTORYPOWER,
    WakeCallback: PFNKSFILTERFACTORYPOWER,
    FilterFactory: *mut PKSFILTERFACTORY,
) -> NTSTATUS {
    return KspCreateFilterFactory(
        DeviceObject,
        Descriptor,
        RefString,
        SecurityDescriptor,
        CreateItemFlags,
        SleepCallback,
        WakeCallback,
        FilterFactory,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterFactorySetDeviceClassesState(
    FilterFactory: PKSFILTERFACTORY,
    NewState: BOOLEAN,
) -> NTSTATUS {
    let This = CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory) as *mut IKsFilterFactoryImpl;
    let Factory = core::ptr::addr_of_mut!((*This).Header.OuterUnknown) as *mut IKsFilterFactory;
    (*(*Factory).lpVtbl).SetDeviceClassesState.unwrap()(Factory, NewState)
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterFactoryGetSymbolicLink(
    FilterFactory: PKSFILTERFACTORY,
) -> PUNICODE_STRING {
    let Factory = CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory) as *mut IKsFilterFactoryImpl;

    if IsListEmpty(&(*Factory).SymbolicLinkList) {
        return core::ptr::null_mut();
    }

    let LinkEntry = CONTAINING_RECORD!((*Factory).SymbolicLinkList.Flink, SYMBOLIC_LINK_ENTRY, Entry) as PSYMBOLIC_LINK_ENTRY;

    return &mut (*LinkEntry).SymbolicLink;
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterFactoryAddCreateItem(
    FilterFactory: PKSFILTERFACTORY,
    RefString: PWSTR,
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    CreateItemFlags: ULONG,
) -> NTSTATUS {
    let mut CreateItem: KSOBJECT_CREATE_ITEM = core::mem::zeroed();
    let Factory: *mut IKsFilterFactoryImpl = CONTAINING_RECORD!(
        FilterFactory,
        IKsFilterFactoryImpl,
        FilterFactory
    );

    CreateItem.Context = &mut (*Factory).FilterFactory as *mut _ as PVOID;
    CreateItem.Create = cb!(IKsFilterFactory_Create);
    CreateItem.Flags = CreateItemFlags;
    CreateItem.SecurityDescriptor = SecurityDescriptor;
    RtlInitUnicodeString(&mut CreateItem.ObjectClass, RefString);

    KsAllocateObjectCreateItem(
        (*Factory).DeviceHeader as KSDEVICE_HEADER,
        &mut CreateItem,
        1u8,
        cb!(IKsFilterFactory_ItemFreeCb),
    )
}
#[no_mangle]
pub unsafe extern "C" fn KspCacheAddData(
    Descriptor: PKSPCACHE_DESCRIPTOR,
    Data: LPCVOID,
    Length: ULONG,
) -> ULONG {
    let mut Index: ULONG = 0;

    while Index < (*Descriptor).DataOffset {
        if RtlCompareMemory(
            (*Descriptor).DataCache as *const core::ffi::c_void,
            Data as *const core::ffi::c_void,
            Length as _,
        ) == Length as _
        {
            if Index.wrapping_add(Length) > (*Descriptor).DataOffset {
                (*Descriptor).DataOffset = Index.wrapping_add(Length);
                return (*Descriptor).DataLength.wrapping_add(Index);
            }
        }
        Index = Index.wrapping_add(1);
    }

    /* sanity check */
    ntkrnl_sys::ASSERT!((*Descriptor).DataOffset.wrapping_add(Length) < (*Descriptor).DataLength);

    /* copy to data blob */
    core::ptr::copy_nonoverlapping(
        Data as *const u8,
        (*Descriptor).DataCache.offset((*Descriptor).DataOffset as isize),
        Length as usize,
    );

    /* backup offset */
    Index = (*Descriptor).DataOffset;

    /* adjust used space */
    (*Descriptor).DataOffset = (*Descriptor).DataOffset.wrapping_add(Length);

    /* return absolute offset */
    (*Descriptor).DataLength.wrapping_add(Index)
}

#[no_mangle]
pub unsafe extern "system" fn KsFilterFactoryUpdateCacheData(
    FilterFactory: PKSFILTERFACTORY,
    FilterDescriptor: *const KSFILTER_DESCRIPTOR,
) -> NTSTATUS {
    let mut Descriptor: KSPCACHE_DESCRIPTOR = core::mem::zeroed();
    let mut FilterHeader: PKSPCACHE_FILTER_HEADER = core::ptr::null_mut();
    let mut FilterData: _UNICODE_STRING = rtl_const_string(crate::L!("FilterData"));
    let mut PinHeader: PKSPCACHE_PIN_HEADER = core::ptr::null_mut();
    let mut Index: ULONG = 0;
    let mut SubIndex: ULONG = 0;
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut SymEntry: PSYMBOLIC_LINK_ENTRY = core::ptr::null_mut();
    let mut Found: BOOLEAN = 0;
    let mut hKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    let Factory: *mut IKsFilterFactoryImpl = CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory);

    let mut FilterDescriptor = FilterDescriptor;
    if FilterDescriptor.is_null() {
        FilterDescriptor = (*Factory).FilterFactory.FilterDescriptor;
    }

    RtlZeroMemory(&mut Descriptor as *mut KSPCACHE_DESCRIPTOR as *mut c_void, core::mem::size_of::<KSPCACHE_DESCRIPTOR>());

    Descriptor.FilterLength = core::mem::size_of::<KSPCACHE_FILTER_HEADER>() as ULONG;

    Index = 0;
    while Index < (*FilterDescriptor).PinDescriptorsCount {
        Descriptor.FilterLength += core::mem::size_of::<KSPCACHE_PIN_HEADER>() as ULONG;

        if !(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.Category.is_null() {
            Descriptor.FilterLength += core::mem::size_of::<ULONG>() as ULONG;
            Descriptor.DataLength += core::mem::size_of::<CLSID>() as ULONG;
        }

        Descriptor.FilterLength += (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRangesCount as ULONG * core::mem::size_of::<KSPCACHE_DATARANGE>() as ULONG;
        Descriptor.DataLength += (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRangesCount as ULONG * core::mem::size_of::<CLSID>() as ULONG * 2;
        Descriptor.FilterLength += (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.MediumsCount as ULONG * core::mem::size_of::<ULONG>() as ULONG;
        Descriptor.DataLength += (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.MediumsCount as ULONG * core::mem::size_of::<KSPCACHE_MEDIUM>() as ULONG;

        Index += 1;
    }

    Descriptor.FilterData = AllocateItem(NonPagedPool, (Descriptor.DataLength + Descriptor.FilterLength) as SIZE_T) as PUCHAR;
    if Descriptor.FilterData.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Descriptor.DataCache = Descriptor.FilterData.offset(Descriptor.FilterLength as isize);

    FilterHeader = Descriptor.FilterData as PKSPCACHE_FILTER_HEADER;
    (*FilterHeader).dwVersion = 2;
    (*FilterHeader).dwMerit = MERIT_DO_NOT_USE;
    (*FilterHeader).dwUnused = 0;
    (*FilterHeader).dwPins = (*FilterDescriptor).PinDescriptorsCount;

    Descriptor.FilterOffset = core::mem::size_of::<KSPCACHE_FILTER_HEADER>() as ULONG;

    Index = 0;
    while Index < (*FilterDescriptor).PinDescriptorsCount {
        PinHeader = Descriptor.FilterData.offset(Descriptor.FilterOffset as isize) as PKSPCACHE_PIN_HEADER;

        (*PinHeader).Signature = 0x33697030 + Index;
        (*PinHeader).Flags = 0;
        (*PinHeader).Instances = (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).InstancesPossible;
        if (*PinHeader).Instances > 1 {
            (*PinHeader).Flags |= REG_PINFLAG_B_MANY;
        }

        (*PinHeader).MediaTypes = (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRangesCount;
        (*PinHeader).Mediums = (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.MediumsCount;
        (*PinHeader).Category = if !(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.Category.is_null() { 1 } else { 0 };

        Descriptor.FilterOffset += core::mem::size_of::<KSPCACHE_PIN_HEADER>() as ULONG;

        if (*PinHeader).Category != 0 {
            let Category: PULONG = PinHeader.add(1) as PULONG;
            *Category = KspCacheAddData(
                &mut Descriptor,
                (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.Category as LPCVOID,
                core::mem::size_of::<CLSID>() as ULONG,
            );
            Descriptor.FilterOffset += core::mem::size_of::<ULONG>() as ULONG;
        }

        SubIndex = 0;
        while SubIndex < (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRangesCount {
            let DataRange: PKSPCACHE_DATARANGE = Descriptor.FilterData.offset(Descriptor.FilterOffset as isize) as PKSPCACHE_DATARANGE;
            (*DataRange).Signature = 0x33797430 + SubIndex;
            (*DataRange).dwUnused = 0;
            (*DataRange).OffsetMajor = KspCacheAddData(
                &mut Descriptor,
                core::ptr::addr_of!((*(*(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRanges.offset(SubIndex as isize))).__bindgen_anon_1.as_ref().MajorFormat) as LPCVOID,
                core::mem::size_of::<CLSID>() as ULONG,
            );
            (*DataRange).OffsetMinor = KspCacheAddData(
                &mut Descriptor,
                core::ptr::addr_of!((*(*(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.DataRanges.offset(SubIndex as isize))).__bindgen_anon_1.as_ref().SubFormat) as LPCVOID,
                core::mem::size_of::<CLSID>() as ULONG,
            );
            Descriptor.FilterOffset += core::mem::size_of::<KSPCACHE_DATARANGE>() as ULONG;
            SubIndex += 1;
        }

        SubIndex = 0;
        while SubIndex < (*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.MediumsCount {
            let mut Medium: KSPCACHE_MEDIUM = core::mem::zeroed();
            let MediumOffset: PULONG = Descriptor.FilterData.offset(Descriptor.FilterOffset as isize) as PULONG;

            core::ptr::copy_nonoverlapping(
                &(*(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.Mediums.offset(SubIndex as isize)).Set as *const _ as *const u8,
                &mut Medium.Medium as *mut _ as *mut u8,
                core::mem::size_of::<GUID>(),
            );
            Medium.dw1 = (*(*(*FilterDescriptor).PinDescriptors.offset(Index as isize)).PinDescriptor.Mediums.offset(SubIndex as isize)).Id;
            Medium.dw2 = 0;

            *MediumOffset = KspCacheAddData(
                &mut Descriptor,
                core::ptr::addr_of!(Medium) as LPCVOID,
                core::mem::size_of::<KSPCACHE_MEDIUM>() as ULONG,
            );
            Descriptor.FilterOffset += core::mem::size_of::<ULONG>() as ULONG;
            SubIndex += 1;
        }

        Index += 1;
    }

    Index = 0;
    while Index < (*FilterDescriptor).CategoriesCount {
        Entry = (*Factory).SymbolicLinkList.Flink;
        Found = 0;

        while Entry != &mut (*Factory).SymbolicLinkList {
            SymEntry = CONTAINING_RECORD!(Entry, SYMBOLIC_LINK_ENTRY, Entry);

            if IsEqualGUIDAligned(&(*SymEntry).DeviceInterfaceClass, (*FilterDescriptor).Categories.offset(Index as isize)) {
                Found = 1;
                break;
            }

            Entry = (*Entry).Flink;
        }

        if Found == 0 {
            Status = STATUS_INVALID_PARAMETER;
            break;
        }

        Status = IoOpenDeviceInterfaceRegistryKey(&mut (*SymEntry).SymbolicLink, KEY_WRITE, &mut hKey);
        if !NT_SUCCESS(Status) {
            break;
        }

        Status = ZwSetValueKey(hKey, &mut FilterData, 0, REG_BINARY, Descriptor.FilterData as PVOID, Descriptor.FilterLength + Descriptor.DataOffset);
        ZwClose(hKey);

        if !NT_SUCCESS(Status) {
            break;
        }

        Index += 1;
    }

    FreeItem(Descriptor.FilterData as PVOID);
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/filterfactory.c
//  * PURPOSE:         KS IKsFilterFactory interface functions
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
//     KSFILTERFACTORY FilterFactory;
//
//     LONG ref;
//     PKSIDEVICE_HEADER DeviceHeader;
//     PFNKSFILTERFACTORYPOWER SleepCallback;
//     PFNKSFILTERFACTORYPOWER WakeCallback;
//
//     LIST_ENTRY SymbolicLinkList;
//     KMUTEX ControlMutex;
//
// }IKsFilterFactoryImpl;
//
// VOID
// NTAPI
// IKsFilterFactory_ItemFreeCb(
//     IN PKSOBJECT_CREATE_ITEM  CreateItem)
// {
//     /* callback when create item is freed in the device header */
//     IKsFilterFactory * iface = (IKsFilterFactory*)CONTAINING_RECORD!(CreateItem->Context, IKsFilterFactoryImpl, FilterFactory);
//
//     iface->lpVtbl->Release(iface);
// }
//
// NTSTATUS
// NTAPI
// IKsFilterFactory_Create(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PKSOBJECT_CREATE_ITEM CreateItem;
//     IKsFilterFactoryImpl * Factory;
//     IKsFilterFactory * iface;
//     NTSTATUS Status;
//
//     /* access the create item */
//     CreateItem = KSCREATE_ITEM_IRP_STORAGE(Irp);
//     if (!CreateItem)
//     {
//         DPRINT1("IKsFilterFactory_Create no CreateItem\n");
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* get filter factory interface */
//     Factory = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(CreateItem->Context, IKsFilterFactoryImpl, FilterFactory);
//
//     /* get interface */
//     iface = (IKsFilterFactory*)&Factory->Header.OuterUnknown;
//
//     /* create a filter instance */
//     Status = KspCreateFilter(DeviceObject, Irp, iface);
//
//     DPRINT("KspCreateFilter Status %x\n", Status);
//
//     if (Status != STATUS_PENDING)
//     {
//         Irp->IoStatus.Information = 0;
//         Irp->IoStatus.Status = Status;
//         CompleteRequest(Irp, IO_NO_INCREMENT);
//     }
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// IKsFilterFactory_fnQueryInterface(
//     IKsFilterFactory * iface,
//     IN  REFIID refiid,
//     OUT PVOID* Output)
// {
//     NTSTATUS Status;
//
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     if (IsEqualGUIDAligned(refiid, &IID_IUnknown))
//     {
//         *Output = &This->Header.OuterUnknown;
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
//     DPRINT("IKsFilterFactory_fnQueryInterface no interface\n");
//     return STATUS_NOT_SUPPORTED;
// }
//
// ULONG
// NTAPI
// IKsFilterFactory_fnAddRef(
//     IKsFilterFactory * iface)
// {
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     return InterlockedIncrement(&This->ref);
// }
//
// ULONG
// NTAPI
// IKsFilterFactory_fnRelease(
//     IKsFilterFactory * iface)
// {
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     InterlockedDecrement(&This->ref);
//
//     if (This->ref == 0)
//     {
//         if (!IsListEmpty(&This->SymbolicLinkList))
//         {
//             /* disable device interfaces */
//             KspSetDeviceInterfacesState(&This->SymbolicLinkList, FALSE);
//             /* free device interface strings */
//             KspFreeDeviceInterfaces(&This->SymbolicLinkList);
//         }
//
//         FreeItem(This);
//         return 0;
//     }
//     /* Return new reference count */
//     return This->ref;
// }
//
// KSFILTERFACTORY*
// NTAPI
// IKsFilterFactory_fnGetStruct(
//     IKsFilterFactory * iface)
// {
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     return &This->FilterFactory;
// }
//
// NTSTATUS
// NTAPI
// IKsFilterFactory_fnSetDeviceClassesState(
//     IKsFilterFactory * iface,
//     IN BOOLEAN Enable)
// {
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     return KspSetDeviceInterfacesState(&This->SymbolicLinkList, Enable);
// }
//
// VOID
// IKsFilterFactory_AttachFilterFactoryToDeviceHeader(
//     IKsFilterFactoryImpl * This,
//     PKSIDEVICE_HEADER DeviceHeader)
// {
//     PKSBASIC_HEADER BasicHeader;
//     PKSFILTERFACTORY FilterFactory;
//
//     if (DeviceHeader->BasicHeader.FirstChild.FilterFactory == NULL)
//     {
//         /* first attached filter factory */
//         DeviceHeader->BasicHeader.FirstChild.FilterFactory = &This->FilterFactory;
//         return;
//     }
//
//     /* set to first entry */
//     FilterFactory = DeviceHeader->BasicHeader.FirstChild.FilterFactory;
//
//     do
//     {
//         /* get basic header */
//         BasicHeader = (PKSBASIC_HEADER)((ULONG_PTR)FilterFactory - sizeof(KSBASIC_HEADER));
//         /* sanity check */
//         ASSERT(BasicHeader->Type == KsObjectTypeFilterFactory);
//
//         if (BasicHeader->Next.FilterFactory)
//         {
//             /* iterate to next filter factory */
//             FilterFactory = BasicHeader->Next.FilterFactory;
//         }
//         else
//         {
//             /* found last entry */
//             break;
//         }
//     }while(FilterFactory);
//
//     /* attach filter factory */
//     BasicHeader->Next.FilterFactory = &This->FilterFactory;
// }
//
// NTSTATUS
// NTAPI
// IKsFilterFactory_fnInitialize(
//     IKsFilterFactory * iface,
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN const KSFILTER_DESCRIPTOR  *Descriptor,
//     IN PWSTR  RefString OPTIONAL,
//     IN PSECURITY_DESCRIPTOR  SecurityDescriptor OPTIONAL,
//     IN ULONG  CreateItemFlags,
//     IN PFNKSFILTERFACTORYPOWER  SleepCallback OPTIONAL,
//     IN PFNKSFILTERFACTORYPOWER  WakeCallback OPTIONAL,
//     OUT PKSFILTERFACTORY *FilterFactory OPTIONAL)
// {
//     UNICODE_STRING ReferenceString;
//     NTSTATUS Status;
//     PDEVICE_EXTENSION DeviceExtension;
//     KSOBJECT_CREATE_ITEM CreateItem;
//     BOOL FreeString = FALSE;
//     IKsDevice * KsDevice;
//
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(iface, IKsFilterFactoryImpl, Header.OuterUnknown);
//
//     /* get device extension */
//     DeviceExtension = (PDEVICE_EXTENSION)DeviceObject->DeviceExtension;
//
//     /* initialize filterfactory */
//     This->SleepCallback = SleepCallback;
//     This->WakeCallback = WakeCallback;
//     This->Header.KsDevice = &DeviceExtension->DeviceHeader->KsDevice;
//     This->Header.Type = KsObjectTypeFilterFactory;
//     This->Header.Parent.KsDevice = &DeviceExtension->DeviceHeader->KsDevice;
//     This->DeviceHeader = DeviceExtension->DeviceHeader;
//
//     /* copy descriptor */
//     This->FilterFactory.FilterDescriptor = AllocateItem(NonPagedPool, sizeof(KSFILTER_DESCRIPTOR));
//     if (!This->FilterFactory.FilterDescriptor)
//     {
//         DPRINT1("Out of memory\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//     RtlMoveMemory((PVOID)This->FilterFactory.FilterDescriptor, (PVOID)Descriptor, sizeof(KSFILTER_DESCRIPTOR));
//
//     /* initialize filter factory control mutex */
//     This->Header.ControlMutex = &This->ControlMutex;
//     KeInitializeMutex(This->Header.ControlMutex, 0);
//
//     /* unused fields */
//     InitializeListHead(&This->Header.EventList);
//     KeInitializeSpinLock(&This->Header.EventListLock);
//
//     InitializeListHead(&This->SymbolicLinkList);
//
//     /* does the device use a reference string */
//     if (RefString || !Descriptor->ReferenceGuid)
//     {
//         /* use device reference string */
//         RtlInitUnicodeString(&ReferenceString, RefString);
//     }
//     else
//     {
//         /* create reference string from descriptor guid */
//         Status = RtlStringFromGUID(Descriptor->ReferenceGuid, &ReferenceString);
//
//         /* check for success */
//         if (!NT_SUCCESS(Status))
//         {
//             /* omg, we failed */
//             return Status;
//         }
//
//         FreeString = TRUE;
//     }
//
//     DPRINT("IKsFilterFactory_fnInitialize CategoriesCount %u ReferenceString '%S'\n", Descriptor->CategoriesCount,ReferenceString.Buffer);
//
//     /* now register the device interface */
//     Status = KspRegisterDeviceInterfaces(DeviceExtension->DeviceHeader->KsDevice.PhysicalDeviceObject,
//                                          Descriptor->CategoriesCount,
//                                          Descriptor->Categories,
//                                          &ReferenceString,
//                                          &This->SymbolicLinkList);
//     /* check for success */
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("KspRegisterDeviceInterfaces failed with %x\n", Status);
//
//         if (FreeString)
//         {
//             /* free unicode string */
//             RtlFreeUnicodeString(&ReferenceString);
//         }
//
//         return Status;
//     }
//
//     /* now setup the create item */
//     CreateItem.SecurityDescriptor = SecurityDescriptor;
//     CreateItem.Flags = CreateItemFlags;
//     CreateItem.Create = IKsFilterFactory_Create;
//     CreateItem.Context = (PVOID)&This->FilterFactory;
//     RtlInitUnicodeString(&CreateItem.ObjectClass, ReferenceString.Buffer);
//
//     /* insert create item to device header */
//     Status = KsAllocateObjectCreateItem((KSDEVICE_HEADER)DeviceExtension->DeviceHeader, &CreateItem, TRUE, IKsFilterFactory_ItemFreeCb);
//
//     if (FreeString)
//     {
//         /* free unicode string */
//         RtlFreeUnicodeString(&ReferenceString);
//     }
//
//     /* create a object bag for the filter factory */
//     This->FilterFactory.Bag = AllocateItem(NonPagedPool, sizeof(KSIOBJECT_BAG));
//     if (This->FilterFactory.Bag)
//     {
//         /* initialize object bag */
//         KsDevice = (IKsDevice*)&DeviceExtension->DeviceHeader->BasicHeader.OuterUnknown;
//         KsDevice->lpVtbl->InitializeObjectBag(KsDevice, (PKSIOBJECT_BAG)This->FilterFactory.Bag, NULL);
//     }
//
//     if (FilterFactory)
//     {
//         /* return filterfactory */
//         *FilterFactory = &This->FilterFactory;
//     }
//
//     /* attach filterfactory to device header */
//     IKsFilterFactory_AttachFilterFactoryToDeviceHeader(This, DeviceExtension->DeviceHeader);
//
//     /* return result */
//     return Status;
// }
//
// static IKsFilterFactoryVtbl vt_IKsFilterFactoryVtbl =
// {
//     IKsFilterFactory_fnQueryInterface,
//     IKsFilterFactory_fnAddRef,
//     IKsFilterFactory_fnRelease,
//     IKsFilterFactory_fnGetStruct,
//     IKsFilterFactory_fnSetDeviceClassesState,
//     IKsFilterFactory_fnInitialize
// };
//
//
// NTSTATUS
// NTAPI
// KspCreateFilterFactory(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN const KSFILTER_DESCRIPTOR  *Descriptor,
//     IN PWSTR  RefString OPTIONAL,
//     IN PSECURITY_DESCRIPTOR  SecurityDescriptor OPTIONAL,
//     IN ULONG  CreateItemFlags,
//     IN PFNKSFILTERFACTORYPOWER  SleepCallback OPTIONAL,
//     IN PFNKSFILTERFACTORYPOWER  WakeCallback OPTIONAL,
//     OUT PKSFILTERFACTORY *FilterFactory OPTIONAL)
// {
//     IKsFilterFactoryImpl * This;
//     IKsFilterFactory * Filter;
//     NTSTATUS Status;
//
//     DPRINT("KsCreateFilterFactory\n");
//
//     /* Lets allocate a filterfactory */
//     This = AllocateItem(NonPagedPool, sizeof(IKsFilterFactoryImpl));
//     if (!This)
//     {
//         /* not enough memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize struct */
//     This->ref = 1;
//     This->Header.OuterUnknown = (PUNKNOWN)&vt_IKsFilterFactoryVtbl;
//
//     /* map to com object */
//     Filter = (IKsFilterFactory*)&This->Header.OuterUnknown;
//
//     /* initialize filter */
//     Status = Filter->lpVtbl->Initialize(Filter, DeviceObject, Descriptor, RefString, SecurityDescriptor, CreateItemFlags, SleepCallback, WakeCallback, FilterFactory);
//     /* did we succeed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* destroy filterfactory */
//         Filter->lpVtbl->Release(Filter);
//     }
//
//     /* return result */
//     DPRINT("KsCreateFilterFactory %x\n", Status);
//     /* sanity check */
//     ASSERT(Status == STATUS_SUCCESS);
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsCreateFilterFactory(
//     IN PDEVICE_OBJECT  DeviceObject,
//     IN const KSFILTER_DESCRIPTOR  *Descriptor,
//     IN PWSTR  RefString OPTIONAL,
//     IN PSECURITY_DESCRIPTOR  SecurityDescriptor OPTIONAL,
//     IN ULONG  CreateItemFlags,
//     IN PFNKSFILTERFACTORYPOWER  SleepCallback OPTIONAL,
//     IN PFNKSFILTERFACTORYPOWER  WakeCallback OPTIONAL,
//     OUT PKSFILTERFACTORY  *FilterFactory OPTIONAL)
// {
//     return KspCreateFilterFactory(DeviceObject, Descriptor, RefString, SecurityDescriptor, CreateItemFlags, SleepCallback, WakeCallback, FilterFactory);
//
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterFactorySetDeviceClassesState(
//     IN PKSFILTERFACTORY  FilterFactory,
//     IN BOOLEAN  NewState)
// {
//     IKsFilterFactory * Factory;
//     IKsFilterFactoryImpl * This = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory);
//
//     Factory = (IKsFilterFactory*)&This->Header.OuterUnknown;
//     return Factory->lpVtbl->SetDeviceClassesState(Factory, NewState);
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI
// PUNICODE_STRING
// NTAPI
// KsFilterFactoryGetSymbolicLink(
//     IN PKSFILTERFACTORY  FilterFactory)
// {
//     PSYMBOLIC_LINK_ENTRY LinkEntry;
//     IKsFilterFactoryImpl * Factory = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory);
//
//     if (IsListEmpty(&Factory->SymbolicLinkList))
//     {
//         /* device has not registered any interfaces */
//         return NULL;
//     }
//
//     /* get first entry */
//     LinkEntry = (PSYMBOLIC_LINK_ENTRY)CONTAINING_RECORD!(Factory->SymbolicLinkList.Flink, SYMBOLIC_LINK_ENTRY, Entry);
//
//     /* return first link */
//     return &LinkEntry->SymbolicLink;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterFactoryAddCreateItem(
//     IN PKSFILTERFACTORY  FilterFactory,
//     IN PWSTR  RefString,
//     IN PSECURITY_DESCRIPTOR  SecurityDescriptor OPTIONAL,
//     IN ULONG  CreateItemFlags)
// {
//     KSOBJECT_CREATE_ITEM CreateItem;
//
//     IKsFilterFactoryImpl * Factory = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory);
//
//     /* initialize create item */
//     CreateItem.Context = (PVOID)&Factory->FilterFactory;
//     CreateItem.Create = IKsFilterFactory_Create;
//     CreateItem.Flags = CreateItemFlags;
//     CreateItem.SecurityDescriptor = SecurityDescriptor;
//     RtlInitUnicodeString(&CreateItem.ObjectClass, RefString);
//
//     /* insert create item to device header */
//     return KsAllocateObjectCreateItem((KSDEVICE_HEADER)Factory->DeviceHeader, &CreateItem, TRUE, IKsFilterFactory_ItemFreeCb);
// }
//
// ULONG
// KspCacheAddData(
//     PKSPCACHE_DESCRIPTOR Descriptor,
//     LPCVOID Data,
//     ULONG Length)
// {
//     ULONG Index;
//
//     for(Index = 0; Index < Descriptor->DataOffset; Index++)
//     {
//         if (RtlCompareMemory(Descriptor->DataCache, Data, Length) == Length)
//         {
//             if (Index + Length > Descriptor->DataOffset)
//             {
//                 /* adjust used space */
//                 Descriptor->DataOffset = Index + Length;
//                 /* return absolute offset */
//                 return Descriptor->DataLength + Index;
//             }
//         }
//     }
//
//     /* sanity check */
//     ASSERT(Descriptor->DataOffset + Length < Descriptor->DataLength);
//
//     /* copy to data blob */
//     RtlMoveMemory((Descriptor->DataCache + Descriptor->DataOffset), Data, Length);
//
//     /* backup offset */
//     Index = Descriptor->DataOffset;
//
//     /* adjust used space */
//     Descriptor->DataOffset += Length;
//
//     /* return absolute offset */
//     return Descriptor->DataLength + Index;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsFilterFactoryUpdateCacheData(
//     IN PKSFILTERFACTORY  FilterFactory,
//     IN const KSFILTER_DESCRIPTOR*  FilterDescriptor OPTIONAL)
// {
//     KSPCACHE_DESCRIPTOR Descriptor;
//     PKSPCACHE_FILTER_HEADER FilterHeader;
//     UNICODE_STRING FilterData = RTL_CONSTANT_STRING(L!("FilterData"));
//     PKSPCACHE_PIN_HEADER PinHeader;
//     ULONG Index, SubIndex;
//     PLIST_ENTRY Entry;
//     PSYMBOLIC_LINK_ENTRY SymEntry;
//     BOOLEAN Found;
//     HANDLE hKey;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     IKsFilterFactoryImpl * Factory = (IKsFilterFactoryImpl*)CONTAINING_RECORD!(FilterFactory, IKsFilterFactoryImpl, FilterFactory);
//
//     DPRINT("KsFilterFactoryUpdateCacheData %p\n", FilterDescriptor);
//
//     if (!FilterDescriptor)
//         FilterDescriptor = Factory->FilterFactory.FilterDescriptor;
//
//     ASSERT(FilterDescriptor);
//
//     /* initialize cache descriptor */
//     RtlZeroMemory(&Descriptor, sizeof(KSPCACHE_DESCRIPTOR));
//
//     /* calculate filter data size */
//     Descriptor.FilterLength = sizeof(KSPCACHE_FILTER_HEADER);
//
//     /* FIXME support variable size pin descriptors */
//     ASSERT(FilterDescriptor->PinDescriptorSize == sizeof(KSPIN_DESCRIPTOR_EX));
//
//     for(Index = 0; Index < FilterDescriptor->PinDescriptorsCount; Index++)
//     {
//         /* add filter descriptor */
//         Descriptor.FilterLength += sizeof(KSPCACHE_PIN_HEADER);
//
//         if (FilterDescriptor->PinDescriptors[Index].PinDescriptor.Category)
//         {
//             /* add extra ULONG for offset to category */
//             Descriptor.FilterLength += sizeof(ULONG);
//
//             /* add size for clsid */
//             Descriptor.DataLength += sizeof(CLSID);
//         }
//
//         /* add space for formats */
//         Descriptor.FilterLength += FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRangesCount * sizeof(KSPCACHE_DATARANGE);
//
//         /* add space for MajorFormat / MinorFormat */
//         Descriptor.DataLength += FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRangesCount * sizeof(CLSID) * 2;
//
//         /* add space for mediums */
//         Descriptor.FilterLength += FilterDescriptor->PinDescriptors[Index].PinDescriptor.MediumsCount * sizeof(ULONG);
//
//         /* add space for the data */
//         Descriptor.DataLength += FilterDescriptor->PinDescriptors[Index].PinDescriptor.MediumsCount * sizeof(KSPCACHE_MEDIUM);
//     }
//
//     /* now allocate the space */
//     Descriptor.FilterData = (PUCHAR)AllocateItem(NonPagedPool, Descriptor.DataLength + Descriptor.FilterLength);
//     if (!Descriptor.FilterData)
//     {
//         /* no memory */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* initialize data cache */
//     Descriptor.DataCache = (PUCHAR)((ULONG_PTR)Descriptor.FilterData + Descriptor.FilterLength);
//
//     /* setup filter header */
//     FilterHeader = (PKSPCACHE_FILTER_HEADER)Descriptor.FilterData;
//
//     FilterHeader->dwVersion = 2;
//     FilterHeader->dwMerit = MERIT_DO_NOT_USE;
//     FilterHeader->dwUnused = 0;
//     FilterHeader->dwPins = FilterDescriptor->PinDescriptorsCount;
//
//     Descriptor.FilterOffset = sizeof(KSPCACHE_FILTER_HEADER);
//
//     /* write pin headers */
//     for(Index = 0; Index < FilterDescriptor->PinDescriptorsCount; Index++)
//     {
//         /* get offset to pin */
//         PinHeader = (PKSPCACHE_PIN_HEADER)((ULONG_PTR)Descriptor.FilterData + Descriptor.FilterOffset);
//
//         /* write pin header */
//         PinHeader->Signature = 0x33697030 + Index;
//         PinHeader->Flags = 0;
//         PinHeader->Instances = FilterDescriptor->PinDescriptors[Index].InstancesPossible;
//         if (PinHeader->Instances > 1)
//             PinHeader->Flags |= REG_PINFLAG_B_MANY;
//
//
//         PinHeader->MediaTypes = FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRangesCount;
//         PinHeader->Mediums = FilterDescriptor->PinDescriptors[Index].PinDescriptor.MediumsCount;
//         PinHeader->Category = (FilterDescriptor->PinDescriptors[Index].PinDescriptor.Category ? TRUE : FALSE);
//
//         Descriptor.FilterOffset += sizeof(KSPCACHE_PIN_HEADER);
//
//         if (PinHeader->Category)
//         {
//             /* get category offset */
//             PULONG Category = (PULONG)(PinHeader + 1);
//
//             /* write category offset */
//             *Category = KspCacheAddData(&Descriptor, FilterDescriptor->PinDescriptors[Index].PinDescriptor.Category, sizeof(CLSID));
//
//             /* adjust offset */
//             Descriptor.FilterOffset += sizeof(ULONG);
//         }
//
//         /* add dataranges */
//         for(SubIndex = 0; SubIndex < FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRangesCount; SubIndex++)
//         {
//             /* get datarange offset */
//             PKSPCACHE_DATARANGE DataRange = (PKSPCACHE_DATARANGE)((ULONG_PTR)Descriptor.FilterData + Descriptor.FilterOffset);
//
//             /* initialize data range */
//             DataRange->Signature = 0x33797430 + SubIndex;
//             DataRange->dwUnused = 0;
//             DataRange->OffsetMajor = KspCacheAddData(&Descriptor, &FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRanges[SubIndex]->MajorFormat, sizeof(CLSID));
//             DataRange->OffsetMinor = KspCacheAddData(&Descriptor, &FilterDescriptor->PinDescriptors[Index].PinDescriptor.DataRanges[SubIndex]->SubFormat, sizeof(CLSID));
//
//             /* adjust offset */
//             Descriptor.FilterOffset += sizeof(KSPCACHE_DATARANGE);
//         }
//
//         /* add mediums */
//         for(SubIndex = 0; SubIndex < FilterDescriptor->PinDescriptors[Index].PinDescriptor.MediumsCount; SubIndex++)
//         {
//             KSPCACHE_MEDIUM Medium;
//             PULONG MediumOffset;
//
//             /* get pin medium offset */
//             MediumOffset = (PULONG)((ULONG_PTR)Descriptor.FilterData + Descriptor.FilterOffset);
//
//             /* copy medium guid */
//             RtlMoveMemory(&Medium.Medium, &FilterDescriptor->PinDescriptors[Index].PinDescriptor.Mediums[SubIndex].Set, sizeof(GUID));
//             Medium.dw1 = FilterDescriptor->PinDescriptors[Index].PinDescriptor.Mediums[SubIndex].Id; /* FIXME verify */
//             Medium.dw2 = 0;
//
//             *MediumOffset = KspCacheAddData(&Descriptor, &Medium, sizeof(KSPCACHE_MEDIUM));
//
//             /* adjust offset */
//             Descriptor.FilterOffset += sizeof(ULONG);
//         }
//     }
//
//     /* sanity checks */
//     ASSERT(Descriptor.FilterOffset == Descriptor.FilterLength);
//     ASSERT(Descriptor.DataOffset <= Descriptor.DataLength);
//
//
//     /* now go through all entries and update 'FilterData' key */
//     for(Index = 0; Index < FilterDescriptor->CategoriesCount; Index++)
//     {
//         /* get first entry */
//         Entry = Factory->SymbolicLinkList.Flink;
//
//         /* set status to not found */
//         Found = FALSE;
//         /* loop list until the the current category is found */
//         while(Entry != &Factory->SymbolicLinkList)
//         {
//             /* fetch symbolic link entry */
//             SymEntry = (PSYMBOLIC_LINK_ENTRY)CONTAINING_RECORD!(Entry, SYMBOLIC_LINK_ENTRY, Entry);
//
//             if (IsEqualGUIDAligned(&SymEntry->DeviceInterfaceClass, &FilterDescriptor->Categories[Index]))
//             {
//                 /* found category */
//                 Found = TRUE;
//                 break;
//             }
//
//             /* move to next entry */
//             Entry = Entry->Flink;
//         }
//
//         if (!Found)
//         {
//             /* filter category is not present */
//             Status = STATUS_INVALID_PARAMETER;
//             break;
//         }
//
//         /* now open device interface */
//         Status = IoOpenDeviceInterfaceRegistryKey(&SymEntry->SymbolicLink, KEY_WRITE, &hKey);
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to open interface key */
//             break;
//         }
//
//         /* update filterdata key */
//         Status = ZwSetValueKey(hKey, &FilterData, 0, REG_BINARY, Descriptor.FilterData, Descriptor.FilterLength + Descriptor.DataOffset);
//
//         /* close filterdata key */
//         ZwClose(hKey);
//
//         if (!NT_SUCCESS(Status))
//         {
//             /* failed to set key value */
//             break;
//         }
//     }
//     /* free filter data */
//     FreeItem(Descriptor.FilterData);
//
//     /* done */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
