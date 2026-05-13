use ntkrnl_sys::hal_ddk::{ARRAY, HAL_BUS_HANDLER, HaliFindBusAddressTranslation, HaliReferenceHandlerForConfigSpace, HaliTranslateBusAddress, HalpAssignSlotResources, HalpContextToBusHandler, HalpInitBusHandler, HalpNoBusData, KEVENT, KIRQL, KSPIN_LOCK, 0i32, LIST_ENTRY, MAXULONG, PARRAY, PBUS_HANDLER, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PHAL_BUS_HANDLER, PINSTALL_BUS_HANDLER, PIO_RESOURCE_REQUIREMENTS_LIST, PLIST_ENTRY, PUNICODE_STRING};
use ntkrnl_sys::ddk::{BusNumber, ExAllocatePoolWithTag, ExFreePoolWithTag, KeBugCheckEx, KeInitializeEvent, KeSetEvent, KeWaitForSingleObject, NonPagedPool, STATUS_INSUFFICIENT_RESOURCES, STATUS_SUCCESS, Type};
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn HalpAllocateArray(ArraySize: ULONG) -> PARRAY {
    let mut array_size = ArraySize;
    let size: ULONG;

    if array_size == MAXULONG {
        array_size = 0;
    }
    size = array_size.wrapping_mul(core::mem::size_of::<PARRAY>() as ULONG)
        .wrapping_add(core::mem::size_of::<ARRAY>() as ULONG);

    let array = ExAllocatePoolWithTag(NonPagedPool, size as usize, 0x48535542u32);
    if array.is_null() {
        KeBugCheckEx(0x2000001, size as usize, 0, 0, 0);
    }

    let array = array as PARRAY;
    (*array).ArraySize = array_size;
    core::ptr::write_bytes(
        (*array).Element.as_mut_ptr(),
        0,
        (array_size.wrapping_add(1)) as usize,
    );
    array
}

#[no_mangle]
pub unsafe extern "system" fn HalpGrowArray(
    CurrentArray: *mut PARRAY,
    NewArray: *mut PARRAY,
) {
    let mut tmp: PVOID;

    if (*CurrentArray).is_null() || ((**NewArray).ArraySize > (**CurrentArray).ArraySize) {
        if !(*CurrentArray).is_null() {
            core::ptr::copy_nonoverlapping(
                core::ptr::addr_of!((**CurrentArray).Element) as *const u8,
                core::ptr::addr_of_mut!((**NewArray).Element) as *mut u8,
                (core::mem::size_of::<PVOID>() * ((**CurrentArray).ArraySize as usize + 1)),
            );
        }

        tmp = *CurrentArray as PVOID;
        *CurrentArray = *NewArray;
        *NewArray = tmp as PARRAY;
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HalpLookupHandler(
    Array: PARRAY,
    Type: ULONG,
    Number: ULONG,
    AddReference: BOOLEAN,
) -> PBUS_HANDLER {
    let mut Handler: PBUS_HANDLER = core::ptr::null_mut();

    unsafe {
        if (*Array).ArraySize >= Type {
            let Array = (*Array).Element[Type as usize] as PARRAY;

            if !Array.is_null() && (*Array).ArraySize >= Number {
                let Bus = (*Array).Element[Number as usize] as PHAL_BUS_HANDLER;
                Handler = &raw mut (*Bus).Handler;

                if AddReference != 0 {
                    (*Bus).ReferenceCount += 1;
                }
            }
        }
    }

    Handler
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HalpLookupHandler(
    Array: PARRAY,
    Type: ULONG,
    Number: ULONG,
    AddReference: BOOLEAN,
) -> PBUS_HANDLER {
    let mut Handler: PBUS_HANDLER = core::ptr::null_mut();

    unsafe {
        if (*Array).ArraySize >= Type {
            let Array = (*Array).Element[Type as usize] as PARRAY;

            if !Array.is_null() && (*Array).ArraySize >= Number {
                let Bus = (*Array).Element[Number as usize] as PHAL_BUS_HANDLER;
                Handler = &raw mut (*Bus).Handler;

                if AddReference != 0 {
                    (*Bus).ReferenceCount += 1;
                }
            }
        }
    }

    Handler
}

#[no_mangle]
pub unsafe extern "system" fn HalpNoBusData(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _SlotNumber: ULONG,
    _Buffer: PVOID,
    _Offset: ULONG,
    _Length: ULONG,
) -> ULONG {
    let _ = _BusHandler;
    let _ = _RootHandler;
    let _ = _SlotNumber;
    let _ = _Buffer;
    let _ = _Offset;
    let _ = _Length;
    0
}

#[no_mangle]
pub unsafe extern "system" fn HalpNoAdjustResourceList(
    _BusHandler: PBUS_HANDLER,
    _RootHandler: PBUS_HANDLER,
    _pResourceList: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    0xC0000001u32 as NTSTATUS
}

#[no_mangle]
pub unsafe extern "system" fn HalpNoAssignSlotResources(
    BusHandler: PBUS_HANDLER,
    RootHandler: PBUS_HANDLER,
    RegistryPath: PUNICODE_STRING,
    DriverClassName: PUNICODE_STRING,
    DriverObject: PDRIVER_OBJECT,
    DeviceObject: PDEVICE_OBJECT,
    SlotNumber: ULONG,
    AllocatedResources: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let _ = BusHandler;
    let _ = RootHandler;
    let _ = RegistryPath;
    let _ = DriverClassName;
    let _ = DriverObject;
    let _ = DeviceObject;
    let _ = SlotNumber;
    let _ = AllocatedResources;
    
    STATUS_NOT_SUPPORTED
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliReferenceBusHandler(Handler: PBUS_HANDLER) {
    let Bus: PHAL_BUS_HANDLER = ((Handler as ULONG_PTR) - 
        core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
    (*Bus).ReferenceCount += 1;
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliReferenceBusHandler(Handler: PBUS_HANDLER) {
    let Bus: PHAL_BUS_HANDLER = ((Handler as ULONG_PTR) - 
        core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
    (*Bus).ReferenceCount += 1;
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliDereferenceBusHandler(Handler: PBUS_HANDLER) {
    let Bus: PHAL_BUS_HANDLER = ((Handler as ULONG_PTR) - 
        core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
    
    (*Bus).ReferenceCount -= 1;
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliDereferenceBusHandler(Handler: PBUS_HANDLER) {
    let Bus: PHAL_BUS_HANDLER = ((Handler as ULONG_PTR) - 
        core::mem::offset_of!(HAL_BUS_HANDLER, Handler) as ULONG_PTR) as *mut HAL_BUS_HANDLER;
    
    (*Bus).ReferenceCount -= 1;
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliHandlerForBus(
    InterfaceType: ULONG,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe 
    
    unsafe extern "C" {
        static mut HalpBusTable: PARRAY;
    }
    
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType, BusNumber, 0) }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliHandlerForBus(
    InterfaceType: ULONG,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe 
    
    unsafe extern "C" {
        static mut HalpBusTable: PARRAY;
    }
    
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType, BusNumber, 0) }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliHandlerForConfigSpace(
    ConfigType: BUS_DATA_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe {
        HalpLookupHandler(HalpConfigTable, ConfigType, BusNumber, 0)
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliHandlerForConfigSpace(
    ConfigType: BUS_DATA_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe {
        HalpLookupHandler(HalpConfigTable, ConfigType, BusNumber, 0)
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliReferenceHandlerForBus(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType, BusNumber, TRUE) }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliReferenceHandlerForBus(
    InterfaceType: INTERFACE_TYPE,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe { HalpLookupHandler(HalpBusTable, InterfaceType, BusNumber, TRUE) }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn HaliReferenceHandlerForConfigSpace(
    ConfigType: ULONG,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe 
    
    unsafe {
        HalpLookupHandler(
            HalpConfigTable,
            ConfigType,
            BusNumber,
            1,
        )
    }
}

#[no_mangle]
#[cfg(target_arch = "x86_64")]
pub unsafe extern "C" fn HaliReferenceHandlerForConfigSpace(
    ConfigType: ULONG,
    BusNumber: ULONG,
) -> PBUS_HANDLER {
    unsafe 
    
    unsafe {
        HalpLookupHandler(
            HalpConfigTable,
            ConfigType,
            BusNumber,
            1,
        )
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpContextToBusHandler(ContextValue: usize) -> PBUS_HANDLER {
    let mut next_entry: PLIST_ENTRY;
    let mut bus_handler: PHAL_BUS_HANDLER;
    let mut this_handler: PHAL_BUS_HANDLER;

    unsafe extern "system" {
        static mut HalpAllBusHandlers: LIST_ENTRY;
    }

    next_entry = unsafe { (*core::ptr::addr_of_mut!(HalpAllBusHandlers)).Flink };
    this_handler = ((next_entry as usize) - core::mem::offset_of!(HAL_BUS_HANDLER, AllHandlers)) as PHAL_BUS_HANDLER;

    if ContextValue != 0 {
        if unsafe { IsListEmpty(core::ptr::addr_of_mut!(HalpAllBusHandlers)) } != 0 {
            return core::ptr::null_mut();
        }

        bus_handler = ((ContextValue as usize) - core::mem::offset_of!(HAL_BUS_HANDLER, Handler)) as PHAL_BUS_HANDLER;
        loop {
            this_handler = ((next_entry as usize) - core::mem::offset_of!(HAL_BUS_HANDLER, AllHandlers)) as PHAL_BUS_HANDLER;
            if this_handler == bus_handler {
                break;
            }

            next_entry = unsafe { (*next_entry).Flink };
            if next_entry == core::ptr::addr_of_mut!(HalpAllBusHandlers) {
                break;
            }
        }
    }

    if next_entry == core::ptr::addr_of_mut!(HalpAllBusHandlers) {
        return core::ptr::null_mut();
    }

    unsafe { core::ptr::addr_of_mut!((*this_handler).Handler) }
}

#[no_mangle]
pub unsafe extern "system" fn HaliRegisterBusHandler(
    InterfaceType: INTERFACE_TYPE,
    ConfigType: BUS_DATA_TYPE,
    BusNumber: ULONG,
    ParentBusType: INTERFACE_TYPE,
    ParentBusNumber: ULONG,
    ExtraData: ULONG,
    InstallCallback: PINSTALL_BUS_HANDLER,
    ReturnedBusHandler: *mut PBUS_HANDLER,
) -> NTSTATUS {
    let mut Bus: PHAL_BUS_HANDLER;
    let mut OldHandler: PHAL_BUS_HANDLER = core::ptr::null_mut();
    let BusEntry: *mut PHAL_BUS_HANDLER;
    let mut InterfaceArray: PARRAY;
    let mut InterfaceBusNumberArray: PARRAY;
    let mut ConfigArray: PARRAY;
    let mut ConfigBusNumberArray: PARRAY;
    let ParentHandler: PBUS_HANDLER;
    let mut OldIrql: KIRQL = 0;
    let mut Status: NTSTATUS;
    let mut bus_number = BusNumber;

    unsafe 

    #[cfg(target_arch = "x86")]
    unsafe 

    #[cfg(target_arch = "x86_64")]
    unsafe 

    #[cfg(target_arch = "x86")]
    unsafe 

    #[cfg(target_arch = "x86_64")]
    unsafe 

    unsafe 

    #[link(name = "ntoskrnl", kind = "raw-dylib")]
    unsafe extern "C" {
        static mut HalpBusDatabaseEvent: KEVENT;
        static mut HalpBusDatabaseSpinLock: KSPIN_LOCK;
        static mut HalpBusTable: PARRAY;
        static mut HalpConfigTable: PARRAY;
        static mut HalpAllBusHandlers: LIST_ENTRY;
    }

    Bus = ExAllocatePoolWithTag(
        NonPagedPool,
        (core::mem::size_of::<HAL_BUS_HANDLER>() + ExtraData as usize) as usize,
        0x48535542u32,
    ) as PHAL_BUS_HANDLER;
    
    if Bus.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    *ReturnedBusHandler = &raw mut (*Bus).Handler;

    KeWaitForSingleObject(
        &raw mut HalpBusDatabaseEvent as *mut _ as PVOID,
        0,
        0i32 as i8,
        0,
        core::ptr::null_mut(),
    );

    if bus_number == MAXULONG {
        bus_number = 0;
        while !HaliHandlerForBus(InterfaceType, bus_number).is_null() {
            bus_number = bus_number.wrapping_add(1);
        }
    }

    InterfaceArray = HalpAllocateArray(InterfaceType);
    InterfaceBusNumberArray = HalpAllocateArray(bus_number);
    ConfigArray = HalpAllocateArray(ConfigType);
    ConfigBusNumberArray = HalpAllocateArray(bus_number);

    if !InterfaceArray.is_null()
        && !InterfaceBusNumberArray.is_null()
        && !ConfigArray.is_null()
        && !ConfigBusNumberArray.is_null()
    {
        ParentHandler = HaliReferenceHandlerForBus(ParentBusType, ParentBusNumber);

        RtlZeroMemory(
            Bus as PVOID,
            (core::mem::size_of::<HAL_BUS_HANDLER>() + ExtraData as usize) as usize,
        );
        (*Bus).ReferenceCount = 1;

        (*Bus).Handler.BusNumber = bus_number;
        (*Bus).Handler.InterfaceType = InterfaceType;
        (*Bus).Handler.ConfigurationType = ConfigType;
        (*Bus).Handler.ParentHandler = ParentHandler;

        (*Bus).Handler.GetBusData = Some(HalpNoBusData);
        (*Bus).Handler.SetBusData = Some(HalpNoBusData);
        (*Bus).Handler.AdjustResourceList = Some(core::mem::transmute(HalpNoAdjustResourceList as usize));
        (*Bus).Handler.AssignSlotResources = Some(core::mem::transmute(HalpNoAssignSlotResources as usize));

        if ExtraData != 0 {
            (*Bus).Handler.BusData = Bus.add(1) as PVOID;
        }

        if !ParentHandler.is_null() {
            (*Bus).Handler.GetBusData = (*ParentHandler).GetBusData;
            (*Bus).Handler.SetBusData = (*ParentHandler).SetBusData;
            (*Bus).Handler.AdjustResourceList = (*ParentHandler).AdjustResourceList;
            (*Bus).Handler.AssignSlotResources = (*ParentHandler).AssignSlotResources;
            (*Bus).Handler.TranslateBusAddress = (*ParentHandler).TranslateBusAddress;
            (*Bus).Handler.GetInterruptVector = (*ParentHandler).GetInterruptVector;
        }

        KeAcquireSpinLock(&raw mut HalpBusDatabaseSpinLock, &raw mut OldIrql);

        HalpGrowArray(&raw mut HalpBusTable, &raw mut InterfaceArray);

        if InterfaceType != InterfaceTypeUndefined {
            HalpGrowArray(
                &raw mut (*HalpBusTable).Element[InterfaceType as usize] as *mut PVOID as *mut PARRAY,
                &raw mut InterfaceBusNumberArray,
            );

            let interface_array = (*HalpBusTable).Element[InterfaceType as usize] as PARRAY;
            let bus_entry_ptr = &raw mut (*interface_array).Element[bus_number as usize] as *mut PVOID as *mut PHAL_BUS_HANDLER;
            
            if !(*bus_entry_ptr).is_null() {
                OldHandler = *bus_entry_ptr;
            }
            *bus_entry_ptr = Bus;
        }

        HalpGrowArray(&raw mut HalpConfigTable, &raw mut ConfigArray);

        if ConfigType != ConfigurationSpaceUndefined {
            HalpGrowArray(
                &raw mut (*HalpConfigTable).Element[ConfigType as usize] as *mut PVOID as *mut PARRAY,
                &raw mut ConfigBusNumberArray,
            );

            let config_array = (*HalpConfigTable).Element[ConfigType as usize] as PARRAY;
            let bus_entry_ptr = &raw mut (*config_array).Element[bus_number as usize] as *mut PVOID as *mut PHAL_BUS_HANDLER;
            
            if !(*bus_entry_ptr).is_null() {
                OldHandler = *bus_entry_ptr;
            }

            *bus_entry_ptr = Bus;
        }

        InsertTailList(&raw mut HalpAllBusHandlers, &raw mut (*Bus).AllHandlers);

        Bus = OldHandler;
        if !Bus.is_null() {
            RemoveEntryList(&raw mut (*Bus).AllHandlers);
        }

        KeReleaseSpinLock(&raw mut HalpBusDatabaseSpinLock, OldIrql);
        Status = STATUS_SUCCESS;
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    KeSetEvent(&raw mut HalpBusDatabaseEvent as *mut _ as PVOID, 0, 0);

    if !Bus.is_null() {
        ExFreePoolWithTag(Bus as PVOID, 0x48535542u32);
    }
    if !InterfaceArray.is_null() {
        ExFreePoolWithTag(InterfaceArray as PVOID, 0x48535542u32);
    }
    if !InterfaceBusNumberArray.is_null() {
        ExFreePoolWithTag(InterfaceBusNumberArray as PVOID, 0x48535542u32);
    }
    if !ConfigArray.is_null() {
        ExFreePoolWithTag(ConfigArray as PVOID, 0x48535542u32);
    }
    if !ConfigBusNumberArray.is_null() {
        ExFreePoolWithTag(ConfigBusNumberArray as PVOID, 0x48535542u32);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitBusHandler() {
    unsafe {
        KeInitializeSpinLock(&raw mut HalpBusDatabaseSpinLock);

        KeInitializeEvent(
            &raw mut HalpBusDatabaseEvent,
            0,
            1,
        );

        HalpBusTable = HalpAllocateArray(0);
        HalpConfigTable = HalpAllocateArray(0);

        InitializeListHead(&raw mut HalpAllBusHandlers);

        HalPciAssignSlotResources = HalpAssignSlotResources;
        HalPciTranslateBusAddress = HaliTranslateBusAddress;
        if HalFindBusAddressTranslation.is_null() {
            HalFindBusAddressTranslation = HaliFindBusAddressTranslation;
        }
    }
}
