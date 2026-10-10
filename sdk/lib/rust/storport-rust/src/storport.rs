use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoAllocateDriverObjectExtension, IoAttachDeviceToDeviceStackSafe, IoCreateDevice, IoDeleteDevice, IoGetDriverObjectExtension, KLOCK_QUEUE_HANDLE, KeAcquireInStackQueuedSpinLock, KeAcquireInterruptSpinLock, KeInitializeDpc, KeReleaseInStackQueuedSpinLock, KeReleaseInterruptSpinLock, KeStallExecutionProcessor, MmAllocateContiguousMemorySpecifyCache, MmGetPhysicalAddress, MmMapIoSpace, PBUS_INTERFACE_STANDARD, PDEVICE_OBJECT, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PIRP, PLARGE_INTEGER, PRKDPC, PUNICODE_STRING, RtlInitUnicodeString, va_list};
use ntkrnl_sys::inline::ke::KeInitializeSpinLock;
use ntkrnl_sys::ddk::{PCSTR, PCCH, CCHAR, INT, INTERFACE_TYPE, KINTERRUPT_MODE, KPROCESSOR_MODE, CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PINTERFACE, PPHYSICAL_ADDRESS, MEMORY_CACHING_TYPE, SIZE_T, _UNICODE_STRING, BUS_INTERFACE_STANDARD};
use core::ffi::c_void;
// The C-variadic entry points (StorPortDebugPrint / StorPortExtendedFunction /
// StorPortNotification) live in drivers/storage/port/storport/rust_varargs_shim.c —
// stable Rust cannot DEFINE one (c_variadic, rust-lang#44930). Each is a 3-line
// va_start trampoline into the fixed-arity `*_v` twin below, which holds ALL the
// logic. `VaArgs` reads the va_list the shim hands over.
use varargs_rust::VaArgs;

extern "system" {
    fn vDbgPrintExWithPrefix( Prefix: PCCH, ComponentId: ULONG, Level: ULONG, Format: PCCH, arglist: va_list, ) -> ULONG;
}
extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> u32;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}
unsafe extern "system" {
    fn PortFdoScsi(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
unsafe extern "system" {
    fn PortPdoScsi(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}
extern "system" {
    // crate-internal NTAPI (defs in misc.rs, also extern "system")
    fn TranslateResourceListAddress( DeviceExtension: PFDO_DEVICE_EXTENSION, BusType: INTERFACE_TYPE, SystemIoBusNumber: ULONG, IoAddress: STOR_PHYSICAL_ADDRESS, NumberOfBytes: ULONG, InIoSpace: BOOLEAN, TranslatedAddress: PPHYSICAL_ADDRESS, ) -> BOOLEAN;
    fn AllocateAddressMapping( MappedAddressList: PLIST_ENTRY, IoAddress: STOR_PHYSICAL_ADDRESS, MappedAddress: PVOID, NumberOfBytes: ULONG, SystemIoBusNumber: ULONG, ) -> NTSTATUS;
}
// IofCompleteRequest is FASTCALL on i386 per ntoskrnl.spec
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn IofCompleteRequest(Irp: PIRP, PriorityBoost: i8);
}
// CRT — cdecl on all arches
extern "C" {
    fn memmove( dest: *mut ::core::ffi::c_void, src: *const ::core::ffi::c_void, n: usize, ) -> *mut ::core::ffi::c_void;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PortNumber: ULONG = 0 as ULONG;

unsafe fn PortAddDriverInitData(
    mut DriverExtension: PDRIVER_OBJECT_EXTENSION,
    mut HwInitializationData: PHW_INITIALIZATION_DATA,
) -> NTSTATUS {
    let mut InitData: PDRIVER_INIT_DATA;

    InitData = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<DRIVER_INIT_DATA>(),
        TAG_INIT_DATA as ULONG,
    ) as PDRIVER_INIT_DATA;
    if InitData.is_null() {
        return STATUS_NO_MEMORY;
    }

    RtlCopyMemory(
        core::ptr::addr_of_mut!((*InitData).HwInitData) as *mut core::ffi::c_void,
        HwInitializationData as *const core::ffi::c_void,
        core::mem::size_of::<HW_INITIALIZATION_DATA>(),
    );

    InsertHeadList(
        core::ptr::addr_of_mut!((*DriverExtension).InitDataListHead),
        core::ptr::addr_of_mut!((*InitData).Entry),
    );

    return STATUS_SUCCESS;
}
unsafe fn PortDeleteDriverInitData(DriverExtension: PDRIVER_OBJECT_EXTENSION) {
    let mut InitData: PDRIVER_INIT_DATA;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = (*DriverExtension).InitDataListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!((*DriverExtension).InitDataListHead) {
        InitData = (ListEntry as usize)
            .wrapping_sub(core::mem::offset_of!(DRIVER_INIT_DATA, Entry))
            as *mut DRIVER_INIT_DATA;

        RemoveEntryList(core::ptr::addr_of_mut!((*InitData).Entry));

        ExFreePoolWithTag(InitData as PVOID, TAG_INIT_DATA as ULONG);

        ListEntry = (*DriverExtension).InitDataListHead.Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn PortGetDriverInitData(
    mut DriverExtension: PDRIVER_OBJECT_EXTENSION,
    mut InterfaceType: INTERFACE_TYPE,
) -> PHW_INITIALIZATION_DATA {
    let mut InitData: PDRIVER_INIT_DATA;
    let mut ListEntry: PLIST_ENTRY;

    ListEntry = (*DriverExtension).InitDataListHead.Flink as PLIST_ENTRY;
    while ListEntry != core::ptr::addr_of_mut!((*DriverExtension).InitDataListHead) {
        InitData = (ListEntry as usize).wrapping_sub(
            core::mem::offset_of!(DRIVER_INIT_DATA, Entry),
        ) as *mut DRIVER_INIT_DATA as PDRIVER_INIT_DATA;
        if (*InitData).HwInitData.AdapterInterfaceType as i32 == InterfaceType as i32 {
            return core::ptr::addr_of_mut!((*InitData).HwInitData);
        }
        ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
    }

    return core::ptr::null_mut();
}
unsafe fn PortAcquireSpinLock(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut SpinLock: STOR_SPINLOCK,
    mut LockContext: PVOID,
    mut LockHandle: PSTOR_LOCK_HANDLE,
) {
    if DbgPrint(
        b"(%s:%d) PortAcquireSpinLock(%p %lu %p %p)\n\0" as *const u8 as PCSTR,
        b"storport.c\0" as *const u8 as *const core::ffi::c_char,
        112 as core::ffi::c_int,
        DeviceExtension,
        SpinLock as core::ffi::c_uint,
        LockContext,
        LockHandle,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"storport.c\0" as *const u8 as *const core::ffi::c_char,
            112 as core::ffi::c_int,
        );
    }
    (*LockHandle).Lock = SpinLock;
    match SpinLock as core::ffi::c_uint {
        1 => {
            if DbgPrint(
                b"(%s:%d) DpcLock\n\0" as *const u8 as PCSTR,
                b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                119 as core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                    119 as core::ffi::c_int,
                );
            }
        }
        2 => {
            if DbgPrint(
                b"(%s:%d) StartIoLock\n\0" as *const u8 as PCSTR,
                b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                123 as core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                    123 as core::ffi::c_int,
                );
            }
        }
        3 => {
            if DbgPrint(
                b"(%s:%d) InterruptLock\n\0" as *const u8 as PCSTR,
                b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                127 as core::ffi::c_int,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
                    b"storport.c\0" as *const u8 as *const core::ffi::c_char,
                    127 as core::ffi::c_int,
                );
            }
            if (*DeviceExtension).Interrupt.is_null() {
                (*LockHandle).Context.OldIrql = 0 as KIRQL;
            } else {
                (*LockHandle).Context.OldIrql =
                    KeAcquireInterruptSpinLock((*DeviceExtension).Interrupt);
            }
        }
        _ => {}
    };
}
unsafe fn PortReleaseSpinLock(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut LockHandle: PSTOR_LOCK_HANDLE,
) {
    if DbgPrint(
        b"(%s:%d) PortReleaseSpinLock(%p %p)\n\0" as *const u8 as *const i8,
        b"storport.c\0" as *const u8 as *const i8,
        144 as i32,
        DeviceExtension,
        LockHandle,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"storport.c\0" as *const u8 as *const i8,
            144 as i32,
        );
    }
    match (*LockHandle).Lock as u32 {
        1 => {
            if DbgPrint(
                b"(%s:%d) DpcLock\n\0" as *const u8 as *const i8,
                b"storport.c\0" as *const u8 as *const i8,
                149 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"storport.c\0" as *const u8 as *const i8,
                    149 as i32,
                );
            }
        }
        2 => {
            if DbgPrint(
                b"(%s:%d) StartIoLock\n\0" as *const u8 as *const i8,
                b"storport.c\0" as *const u8 as *const i8,
                153 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"storport.c\0" as *const u8 as *const i8,
                    153 as i32,
                );
            }
        }
        3 => {
            if DbgPrint(
                b"(%s:%d) InterruptLock\n\0" as *const u8 as *const i8,
                b"storport.c\0" as *const u8 as *const i8,
                157 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
                    b"storport.c\0" as *const u8 as *const i8,
                    157 as i32,
                );
            }
            if !(*DeviceExtension).Interrupt.is_null() {
                KeReleaseInterruptSpinLock(
                    (*DeviceExtension).Interrupt,
                    (*LockHandle).Context.OldIrql,
                );
            }
        }
        _ => {}
    };
}
unsafe extern "system" fn PortAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {

    const FILE_DEVICE_CONTROLLER: u32 = 0x00000004;

    let mut DriverObjectExtension: PDRIVER_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut NameBuffer: [WCHAR; 80] = [0; 80];
    let mut DeviceName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut LockHandle: KLOCK_QUEUE_HANDLE = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if DbgPrint(
        b"(%s:%d) PortAddDevice(%p %p)\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        182,
        DriverObject,
        PhysicalDeviceObject,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            182,
        );
    }

    if DriverObject.is_null() {
        RtlAssert(
            b"DriverObject\0".as_ptr() as PVOID,
            b"storport.c\0".as_ptr() as PVOID,
            184,
            core::ptr::null_mut(),
        );
    }
    if PhysicalDeviceObject.is_null() {
        RtlAssert(
            b"PhysicalDeviceObject\0".as_ptr() as PVOID,
            b"storport.c\0".as_ptr() as PVOID,
            185,
            core::ptr::null_mut(),
        );
    }

    let format: [u16; 20] = [
        0x005C, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0052, 0x0061, 0x0069,
        0x0064, 0x0050, 0x006F, 0x0072, 0x0074, 0x0025, 0x006C, 0x0075, 0x0000,
    ];
    swprintf(
        core::ptr::addr_of_mut!(NameBuffer) as *mut u16,
        format.as_ptr(),
        PortNumber,
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(DeviceName),
        core::ptr::addr_of_mut!(NameBuffer) as *mut WCHAR,
    );
    PortNumber = PortNumber.wrapping_add(1);

    if DbgPrint(
        b"(%s:%d) Creating device: %wZ\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        193,
        core::ptr::addr_of_mut!(DeviceName),
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            193,
        );
    }

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>() as u32,
        core::ptr::addr_of_mut!(DeviceName),
        FILE_DEVICE_CONTROLLER,
        FILE_DEVICE_SECURE_OPEN as u32,
        0,
        core::ptr::addr_of_mut!(Fdo),
    );
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) IoCreateDevice() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            205,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"storport.c\0".as_ptr() as *const i8,
                205,
            );
        }
        return Status;
    }

    if DbgPrint(
        b"(%s:%d) Created device: %wZ (%p)\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        209,
        core::ptr::addr_of_mut!(DeviceName),
        Fdo,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            209,
        );
    }

    (*Fdo).Flags |= DO_DIRECT_IO as u32;
    (*Fdo).Flags |= DO_POWER_PAGABLE as u32;

    DeviceExtension = (*Fdo).DeviceExtension as PFDO_DEVICE_EXTENSION;
    core::ptr::write_bytes(
        DeviceExtension as *mut u8,
        0,
        core::mem::size_of::<FDO_DEVICE_EXTENSION>(),
    );

    (*DeviceExtension).ExtensionType = EXTENSION_TYPE_FdoExtension;

    (*DeviceExtension).Device = Fdo;
    (*DeviceExtension).PhysicalDevice = PhysicalDeviceObject;

    (*DeviceExtension).PnpState = dsStopped;

    KeInitializeSpinLock(core::ptr::addr_of_mut!((*DeviceExtension).PdoListLock) as *mut usize);
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceExtension).PdoListHead));

    Status = IoAttachDeviceToDeviceStackSafe(
        Fdo,
        PhysicalDeviceObject,
        core::ptr::addr_of_mut!((*DeviceExtension).LowerDevice),
    );
    if !NT_SUCCESS(Status) {
        if DbgPrint(
            b"(%s:%d) IoAttachDeviceToDeviceStackSafe() failed (Status 0x%08lx)\n\0".as_ptr()
                as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            235,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"storport.c\0".as_ptr() as *const i8,
                235,
            );
        }
        IoDeleteDevice(Fdo);
        return Status;
    }

    DriverObjectExtension = IoGetDriverObjectExtension(
        DriverObject,
        DriverEntry as PVOID,
    ) as PDRIVER_OBJECT_EXTENSION;
    if (*DriverObjectExtension).ExtensionType as u32 != EXTENSION_TYPE_DriverExtension as u32 {
        RtlAssert(
            b"DriverObjectExtension->ExtensionType == DriverExtension\0".as_ptr() as PVOID,
            b"storport.c\0".as_ptr() as PVOID,
            243,
            core::ptr::null_mut(),
        );
    }

    (*DeviceExtension).DriverExtension = DriverObjectExtension;

    KeAcquireInStackQueuedSpinLock(
        core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListLock),
        core::ptr::addr_of_mut!(LockHandle),
    );

    InsertHeadList(
        core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListHead),
        core::ptr::addr_of_mut!((*DeviceExtension).AdapterListEntry),
    );
    (*DriverObjectExtension).AdapterCount =
        (*DriverObjectExtension).AdapterCount.wrapping_add(1);

    KeReleaseInStackQueuedSpinLock(core::ptr::addr_of_mut!(LockHandle));

    (*Fdo).Flags &= !(DO_DEVICE_INITIALIZING as u32);

    if DbgPrint(
        b"(%s:%d) PortAddDevice() done (Status 0x%08lx)\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        259,
        Status,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            259,
        );
    }

    return Status;
}
unsafe extern "system" fn PortUnload(DriverObject: PDRIVER_OBJECT) {
    let mut DriverExtension: PDRIVER_OBJECT_EXTENSION = core::ptr::null_mut();

    DriverExtension = IoGetDriverObjectExtension(
        DriverObject,
        DriverEntry as PVOID,
    ) as PDRIVER_OBJECT_EXTENSION;
    if !DriverExtension.is_null() {
        PortDeleteDriverInitData(DriverExtension);
    }
}
unsafe extern "system" fn PortDispatchCreate(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    const FILE_OPENED: u32 = 0x00000001;
    if DbgPrint(
        b"(%s:%d) PortDispatchCreate(%p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"storport.c\0" as *const u8 as *const ::core::ffi::c_char,
        293 as ::core::ffi::c_int,
        DeviceObject,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"storport.c\0" as *const u8 as *const ::core::ffi::c_char,
            293 as ::core::ffi::c_int,
        );
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = FILE_OPENED as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}
unsafe extern "system" fn PortDispatchClose(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) PortDispatchClose(%p %p)\n\0" as *const u8 as *const i8,
        b"storport.c\0" as *const u8 as *const i8,
        312 as i32,
        DeviceObject,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"storport.c\0" as *const u8 as *const i8,
            312 as i32,
        );
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0 as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}
unsafe extern "system" fn PortDispatchDeviceControl(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0 as _;

    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    return STATUS_SUCCESS;
}
unsafe extern "system" fn PortDispatchScsi(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {

    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    if DbgPrint(
        b"(%s:%d) PortDispatchScsi(%p %p)\n\0".as_ptr() as PCSTR,
        b"storport.c\0".as_ptr() as *const core::ffi::c_char,
        352 as core::ffi::c_int,
        DeviceObject,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"storport.c\0".as_ptr() as *const core::ffi::c_char,
            352 as core::ffi::c_int,
        );
    }

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    if DbgPrint(
        b"(%s:%d) ExtensionType: %u\n\0".as_ptr() as PCSTR,
        b"storport.c\0".as_ptr() as *const core::ffi::c_char,
        355 as core::ffi::c_int,
        (*DeviceExtension).ExtensionType as core::ffi::c_uint,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"storport.c\0".as_ptr() as *const core::ffi::c_char,
            355 as core::ffi::c_int,
        );
    }

    match (*DeviceExtension).ExtensionType as core::ffi::c_uint {
        2 => return PortFdoScsi(DeviceObject, Irp),
        3 => return PortPdoScsi(DeviceObject, Irp),
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*Irp).IoStatus.Information = 0 as _;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as CCHAR);
            return STATUS_UNSUCCESSFUL;
        }
    }
}
unsafe extern "system" fn PortDispatchSystemControl(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) PortDispatchSystemControl(%p %p)\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        386 as i32,
        DeviceObject,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            386 as i32,
        );
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0 as _;
    IofCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}

unsafe extern "system" fn PortDispatchPnp(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();

    DeviceExtension = (*DeviceObject).DeviceExtension as PFDO_DEVICE_EXTENSION;

    match (*DeviceExtension).ExtensionType as u32 {
        2 => {
            return PortFdoPnp(DeviceObject, Irp);
        }
        3 => {
            return PortPdoPnp(DeviceObject, Irp);
        }
        _ => {
            (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
            (*Irp).IoStatus.Information = 0 as _;
            IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
            return STATUS_UNSUCCESSFUL;
        }
    }
}
unsafe extern "system" fn PortDispatchPower(
    mut DeviceObject: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) PortDispatchPower(%p %p)\n\0" as *const u8 as *const i8,
        b"storport.c\0" as *const u8 as *const i8,
        439 as i32,
        DeviceObject,
        Irp,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"storport.c\0" as *const u8 as *const i8,
            439 as i32,
        );
    }
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0 as _;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    mut DriverObject: PDRIVER_OBJECT,
    mut RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    if DbgPrint(
        b"(%s:%d) DriverEntry(%p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        461 as ::core::ffi::c_int,
        DriverObject,
        RegistryPath,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            461 as ::core::ffi::c_int,
        );
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortAllocateRegistryBuffer(
    HwDeviceExtension: PVOID,
    Length: PULONG,
) -> PUCHAR {
    if DbgPrint(
        b"(%s:%d) StorPortAllocateRegistryBuffer()\n\0".as_ptr() as PCSTR,
        b"storport.c\0".as_ptr() as *const ::core::ffi::c_char,
        476 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"storport.c\0".as_ptr() as *const ::core::ffi::c_char,
            476 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"StorPortAllocateRegistryBuffer\0".as_ptr() as *const ::core::ffi::c_char,
        b"storport.c\0".as_ptr() as *const ::core::ffi::c_char,
        477 as ::core::ffi::c_int,
    );
    return ::core::ptr::null_mut::<::core::ffi::c_uchar>();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortBusy(
    mut HwDeviceExtension: PVOID,
    mut RequestsToComplete: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortBuzy()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        492 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            492 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortBusy\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        493 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortCompleteRequest(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut SrbStatus: UCHAR,
) {
    if DbgPrint(
        b"(%s:%d) StorPortCompleteRequest()\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        511 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const core::ffi::c_char,
            511 as core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"StorPortCompleteRequest\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        512 as core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StorPortConvertPhysicalAddressToUlong(
    mut Address: STOR_PHYSICAL_ADDRESS,
) -> ULONG {
    if DbgPrint(
        b"(%s:%d) StorPortConvertPhysicalAddressToUlong()\n\0" as *const u8
            as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        525 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            525 as ::core::ffi::c_int,
        );
    }
    return Address.u.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortConvertUlongToPhysicalAddress(
    mut UlongAddress: ULONG_PTR,
) -> STOR_PHYSICAL_ADDRESS {
    let mut Address: STOR_PHYSICAL_ADDRESS = core::mem::zeroed();
    if DbgPrint(
        b"(%s:%d) StorPortConvertUlongToPhysicalAddress()\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        542 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const core::ffi::c_char,
            542 as core::ffi::c_int,
        );
    }
    Address.QuadPart = UlongAddress as LONGLONG;
    return Address;
}

/// Fixed-arity twin of the `StorPortDebugPrint` C-variadic entry point in
/// `drivers/storage/port/storport/rust_varargs_shim.c`. The C side exists only to do
/// the `va_start` that stable Rust cannot (`c_variadic`, rust-lang#44930). This body
/// never `va_arg`s — it forwards the whole list to `vDbgPrintExWithPrefix` — so the
/// shim hands over the `va_list` BY VALUE (same arrangement as the shipping
/// sdk/lib/rtl/dbgprint_variadic.c ↔ rtl-diag pair).
#[no_mangle]
pub unsafe extern "C" fn StorPortDebugPrint_v(
    mut DebugPrintLevel: ULONG,
    mut DebugMessage: PCHAR,
    ap: va_list,
) {
    vDbgPrintExWithPrefix(
        b"STORMINI: \0".as_ptr() as PCCH,
        0x58 as ULONG,
        DebugPrintLevel,
        DebugMessage as PCCH,
        ap,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StorPortDeviceBusy(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut RequestsToComplete: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortDeviceBusy()\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        580 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const core::ffi::c_char,
            580 as core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"StorPortDeviceBusy\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        581 as core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortDeviceReady(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortDeviceReady()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        598 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            598 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortDeviceReady\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        599 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

/// Fixed-arity twin of the `StorPortExtendedFunction` C-variadic entry point in
/// `drivers/storage/port/storport/rust_varargs_shim.c` (see `StorPortDebugPrint_v`).
/// `_ap` is a pointer to the shim frame's `va_list`; this body is the C's unimplemented
/// stub and never pulled an argument, so it stays unread.
#[no_mangle]
pub unsafe extern "C" fn StorPortExtendedFunction_v(
    mut FunctionCode: STORPORT_FUNCTION_CODE,
    mut HwDeviceExtension: PVOID,
    _ap: *mut core::ffi::c_void,
) -> ULONG {
    if DbgPrint(
        b"(%s:%d) StorPortExtendedFunction(%d %p ...)\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        615 as core::ffi::c_int,
        FunctionCode as core::ffi::c_uint,
        HwDeviceExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const core::ffi::c_char,
            615 as core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"StorPortExtendedFunction\0" as *const u8 as *const core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        616 as core::ffi::c_int,
    );
    return STATUS_NOT_IMPLEMENTED as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortFreeDeviceBase(
    mut HwDeviceExtension: PVOID,
    mut MappedAddress: PVOID,
) {
    if DbgPrint(
        b"(%s:%d) StorPortFreeDeviceBase(%p %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        632 as ::core::ffi::c_int,
        HwDeviceExtension,
        MappedAddress,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            632 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn StorPortFreeRegistryBuffer(
    mut HwDeviceExtension: PVOID,
    mut Buffer: PUCHAR,
) {
    if DbgPrint(
        b"(%s:%d) StorPortFreeRegistryBuffer()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        646 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            646 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortFreeRegistryBuffer\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        647 as ::core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetBusData(
    mut DeviceExtension: PVOID,
    mut BusDataType: ULONG,
    mut SystemIoBusNumber: ULONG,
    mut SlotNumber: ULONG,
    mut Buffer: PVOID,
    mut Length: ULONG,
) -> ULONG {

    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION =
        core::ptr::null_mut::<MINIPORT_DEVICE_EXTENSION>();
    let mut Interface: PBUS_INTERFACE_STANDARD =
        core::ptr::null_mut::<BUS_INTERFACE_STANDARD>();
    let mut ReturnLength: ULONG = 0;

    if DbgPrint(
        b"(%s:%d) StorPortGetBusData(%p %lu %lu %lu %p %lu)\n\0" as *const u8 as PCSTR,
        b"storport.c\0" as *const u8 as *const core::ffi::c_char,
        670 as core::ffi::c_int,
        DeviceExtension,
        BusDataType,
        SystemIoBusNumber,
        SlotNumber,
        Buffer,
        Length,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"storport.c\0" as *const u8 as *const core::ffi::c_char,
            670 as core::ffi::c_int,
        );
    }

    MiniportExtension = (DeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as *mut MINIPORT_DEVICE_EXTENSION as PMINIPORT_DEVICE_EXTENSION;

    if DbgPrint(
        b"(%s:%d) DeviceExtension %p  MiniportExtension %p\n\0" as *const u8 as PCSTR,
        b"storport.c\0" as *const u8 as *const core::ffi::c_char,
        677 as core::ffi::c_int,
        DeviceExtension,
        MiniportExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"storport.c\0" as *const u8 as *const core::ffi::c_char,
            677 as core::ffi::c_int,
        );
    }

    Interface = core::ptr::addr_of_mut!(
        (*(*(*MiniportExtension).Miniport).DeviceExtension).BusInterface
    ) as PBUS_INTERFACE_STANDARD;

    if BusDataType == 4 as core::ffi::c_ulong {
        BusDataType = 0 as ULONG;
    }

    ReturnLength = ((*Interface).GetBusData.expect("non-null function pointer"))(
        (*Interface).Context,
        BusDataType,
        Buffer,
        0 as ULONG,
        Length,
    );

    if DbgPrint(
        b"(%s:%d) ReturnLength: %lu\n\0" as *const u8 as PCSTR,
        b"storport.c\0" as *const u8 as *const core::ffi::c_char,
        689 as core::ffi::c_int,
        ReturnLength,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"storport.c\0" as *const u8 as *const core::ffi::c_char,
            689 as core::ffi::c_int,
        );
    }

    return ReturnLength;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetDeviceBase(
    HwDeviceExtension: PVOID,
    BusType: INTERFACE_TYPE,
    SystemIoBusNumber: ULONG,
    IoAddress: STOR_PHYSICAL_ADDRESS,
    NumberOfBytes: ULONG,
    InIoSpace: BOOLEAN,
) -> PVOID {

    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION =
        core::ptr::null_mut::<MINIPORT_DEVICE_EXTENSION>();
    let mut TranslatedAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut MappedAddress: PVOID;
    let Status: NTSTATUS;

    /* Get the miniport extension */
    MiniportExtension = ((HwDeviceExtension as usize)
        - (core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension)))
        as *mut MINIPORT_DEVICE_EXTENSION;

    if TranslateResourceListAddress(
        (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION,
        BusType,
        SystemIoBusNumber,
        IoAddress,
        NumberOfBytes,
        InIoSpace,
        core::ptr::addr_of_mut!(TranslatedAddress),
    ) == 0
    {
        return core::ptr::null_mut();
    }

    /* In I/O space */
    if InIoSpace != 0 {
        return TranslatedAddress.QuadPart as usize as PVOID;
    }

    /* In memory space */
    MappedAddress = MmMapIoSpace(TranslatedAddress, NumberOfBytes as SIZE_T, 0 /* FALSE */ as MEMORY_CACHING_TYPE);

    Status = AllocateAddressMapping(
        core::ptr::addr_of_mut!(
            (*(*(*MiniportExtension).Miniport).DeviceExtension).MappedAddressList
        ) as PLIST_ENTRY,
        IoAddress,
        MappedAddress,
        NumberOfBytes,
        SystemIoBusNumber,
    );
    if !NT_SUCCESS(Status) {
        MappedAddress = core::ptr::null_mut();
    }

    return MappedAddress;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetLogicalUnit(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
) -> PVOID {
    if DbgPrint(
        b"(%s:%d) StorPortGetLogicalUnit()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        779 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            779 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortGetLogicalUnit\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        780 as ::core::ffi::c_int,
    );
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetPhysicalAddress(
    HwDeviceExtension: PVOID,
    Srb: PSCSI_REQUEST_BLOCK,
    VirtualAddress: PVOID,
    Length: *mut ULONG,
) -> STOR_PHYSICAL_ADDRESS {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut PhysicalAddress: STOR_PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Offset: ULONG_PTR;

    /* Get the miniport extension */
    MiniportExtension = (HwDeviceExtension as ULONG_PTR).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension) as ULONG_PTR,
    ) as PMINIPORT_DEVICE_EXTENSION;

    DeviceExtension = (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Inside of the uncached extension? */
    if (VirtualAddress as ULONG_PTR) >= ((*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR)
        && (VirtualAddress as ULONG_PTR)
            <= ((*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR)
                .wrapping_add((*DeviceExtension).UncachedExtensionSize as ULONG_PTR)
    {
        Offset = (VirtualAddress as ULONG_PTR)
            .wrapping_sub((*DeviceExtension).UncachedExtensionVirtualBase as ULONG_PTR);

        PhysicalAddress.QuadPart = ((*DeviceExtension).UncachedExtensionPhysicalBase.QuadPart
            as ULONG_PTR)
            .wrapping_add(Offset) as LONGLONG;
        *Length =
            ((*DeviceExtension).UncachedExtensionSize as ULONG_PTR).wrapping_sub(Offset) as ULONG;

        return PhysicalAddress;
    }

    // FIXME

    PhysicalAddress = MmGetPhysicalAddress(VirtualAddress);
    *Length = 1 as ULONG;

    return PhysicalAddress;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetScatterGatherList(
    mut DeviceExtension: PVOID,
    mut Srb: PSCSI_REQUEST_BLOCK,
) -> PSTOR_SCATTER_GATHER_LIST {
    if DbgPrint(
        b"(%s:%d) StorPortGetScatterGatherList()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        850 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            850 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortGetScatterGatherList\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        851 as ::core::ffi::c_int,
    );
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetSrb(
    mut DeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut QueueTag: LONG,
) -> PSCSI_REQUEST_BLOCK {
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetUncachedExtension(
    HwDeviceExtension: PVOID,
    ConfigInfo: PPORT_CONFIGURATION_INFORMATION,
    NumberOfBytes: ULONG,
) -> PVOID {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut LowestAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut HighestAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Alignment: PHYSICAL_ADDRESS = core::mem::zeroed();

    /* Get the miniport extension */
    MiniportExtension = (HwDeviceExtension as usize).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension),
    ) as *mut MINIPORT_DEVICE_EXTENSION;

    DeviceExtension = (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;

    /* Return the uncached extension base address if we already have one */
    if !(*DeviceExtension).UncachedExtensionVirtualBase.is_null() {
        return (*DeviceExtension).UncachedExtensionVirtualBase;
    }

    // FIXME: Set DMA stuff here?

    /* Allocate the uncached extension */
    Alignment.QuadPart = 0;
    LowestAddress.QuadPart = 0;
    HighestAddress.QuadPart = 0x00000000FFFFFFFF;
    (*DeviceExtension).UncachedExtensionVirtualBase = MmAllocateContiguousMemorySpecifyCache(
        NumberOfBytes as SIZE_T,
        LowestAddress,
        HighestAddress,
        Alignment,
        MmCached,
    );
    if (*DeviceExtension).UncachedExtensionVirtualBase.is_null() {
        return core::ptr::null_mut();
    }

    (*DeviceExtension).UncachedExtensionPhysicalBase =
        MmGetPhysicalAddress((*DeviceExtension).UncachedExtensionVirtualBase);
    (*DeviceExtension).UncachedExtensionSize = NumberOfBytes;

    return (*DeviceExtension).UncachedExtensionVirtualBase;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortGetVirtualAddress(
    mut HwDeviceExtension: PVOID,
    mut PhysicalAddress: STOR_PHYSICAL_ADDRESS,
) -> PVOID {
    if DbgPrint(
        b"(%s:%d) StorPortGetVirtualAddress(%p %I64x)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        937 as ::core::ffi::c_int,
        HwDeviceExtension,
        PhysicalAddress.QuadPart,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            937 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortGetVirtualAddress\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        938 as ::core::ffi::c_int,
    );
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn StorPortInitialize(
    Argument1: PVOID,
    Argument2: PVOID,
    HwInitializationData: *mut _HW_INITIALIZATION_DATA,
    HwContext: PVOID,
) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = Argument1 as PDRIVER_OBJECT;
    let RegistryPath: PUNICODE_STRING = Argument2 as PUNICODE_STRING;
    let mut DriverObjectExtension: PDRIVER_OBJECT_EXTENSION = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DbgPrint(
        b"StorPortInitialize(%p %p %p %p)\n\0".as_ptr() as *const i8,
        Argument1,
        Argument2,
        HwInitializationData,
        HwContext,
    );

    DbgPrint(
        b"HwInitializationDataSize: %lu\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwInitializationDataSize,
    );
    DbgPrint(
        b"AdapterInterfaceType: %u\n\0".as_ptr() as *const i8,
        (*HwInitializationData).AdapterInterfaceType as i32,
    );
    DbgPrint(
        b"HwInitialize: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwInitialize,
    );
    DbgPrint(
        b"HwStartIo: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwStartIo,
    );
    DbgPrint(
        b"HwInterrupt: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwInterrupt,
    );
    DbgPrint(
        b"HwFindAdapter: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwFindAdapter,
    );
    DbgPrint(
        b"HwResetBus: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwResetBus,
    );
    DbgPrint(
        b"HwDmaStarted: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwDmaStarted,
    );
    DbgPrint(
        b"HwAdapterState: %p\n\0".as_ptr() as *const i8,
        (*HwInitializationData).HwAdapterState,
    );
    DbgPrint(
        b"DeviceExtensionSize: %lu\n\0".as_ptr() as *const i8,
        (*HwInitializationData).DeviceExtensionSize,
    );
    DbgPrint(
        b"SpecificLuExtensionSize: %lu\n\0".as_ptr() as *const i8,
        (*HwInitializationData).SpecificLuExtensionSize,
    );
    DbgPrint(
        b"SrbExtensionSize: %lu\n\0".as_ptr() as *const i8,
        (*HwInitializationData).SrbExtensionSize,
    );
    DbgPrint(
        b"NumberOfAccessRanges: %lu\n\0".as_ptr() as *const i8,
        (*HwInitializationData).NumberOfAccessRanges,
    );

    /* Check parameters */
    if DriverObject.is_null() || RegistryPath.is_null() || HwInitializationData.is_null() {
        DbgPrint(b"Invalid parameter!\n\0".as_ptr() as *const i8);
        return STATUS_INVALID_PARAMETER;
    }

    /* Check initialization data */
    if ((*HwInitializationData).HwInitializationDataSize as usize)
        < core::mem::size_of::<HW_INITIALIZATION_DATA>()
        || (*HwInitializationData).HwInitialize.is_none()
        || (*HwInitializationData).HwStartIo.is_none()
        || (*HwInitializationData).HwFindAdapter.is_none()
        || (*HwInitializationData).HwResetBus.is_none()
    {
        DbgPrint(b"Revision mismatch!\n\0".as_ptr() as *const i8);
        return STATUS_REVISION_MISMATCH;
    }

    DriverObjectExtension = IoGetDriverObjectExtension(
        DriverObject,
        core::mem::transmute::<
            Option<unsafe extern "system" fn(PDRIVER_OBJECT, PUNICODE_STRING) -> NTSTATUS>,
            PVOID,
        >(Some(
            DriverEntry as unsafe extern "system" fn(PDRIVER_OBJECT, PUNICODE_STRING) -> NTSTATUS,
        )),
    ) as PDRIVER_OBJECT_EXTENSION;
    if DriverObjectExtension.is_null() {
        DbgPrint(b"No driver object extension!\n\0".as_ptr() as *const i8);

        Status = IoAllocateDriverObjectExtension(
            DriverObject,
            DriverEntry as unsafe extern "system" fn(PDRIVER_OBJECT, PUNICODE_STRING) -> NTSTATUS
                as PVOID,
            core::mem::size_of::<DRIVER_OBJECT_EXTENSION>() as u32,
            core::ptr::addr_of_mut!(DriverObjectExtension) as *mut PVOID,
        );
        if !NT_SUCCESS(Status) {
            DbgPrint(
                b"IoAllocateDriverObjectExtension() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const i8,
                Status,
            );
            return Status;
        }

        DbgPrint(b"Driver object extension created!\n\0".as_ptr() as *const i8);

        /* Initialize the driver object extension */
        RtlZeroMemory(
            DriverObjectExtension as *mut core::ffi::c_void,
            core::mem::size_of::<DRIVER_OBJECT_EXTENSION>(),
        );

        (*DriverObjectExtension).ExtensionType = EXTENSION_TYPE_DriverExtension;
        (*DriverObjectExtension).DriverObject = DriverObject;

        InitializeListHead(core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListHead));
        KeInitializeSpinLock(core::ptr::addr_of_mut!((*DriverObjectExtension).AdapterListLock) as *mut usize);

        InitializeListHead(core::ptr::addr_of_mut!((*DriverObjectExtension).InitDataListHead));

        /* Set handlers */
        (*(*DriverObject).DriverExtension).AddDevice =
            core::mem::transmute(PortAddDevice as unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS);
        (*DriverObject).DriverUnload =
            core::mem::transmute(PortUnload as unsafe extern "system" fn(PDRIVER_OBJECT));
        (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
            core::mem::transmute(PortDispatchCreate as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
            core::mem::transmute(PortDispatchClose as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
            core::mem::transmute(PortDispatchDeviceControl as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_SCSI as usize] =
            core::mem::transmute(PortDispatchScsi as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
            core::mem::transmute(PortDispatchPower as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] =
            core::mem::transmute(PortDispatchSystemControl as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
        (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
            core::mem::transmute(PortDispatchPnp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS);
    }

    /* Add the initialzation data to the driver extension */
    Status = PortAddDriverInitData(
        DriverObjectExtension,
        HwInitializationData as PHW_INITIALIZATION_DATA,
    );

    DbgPrint(
        b"StorPortInitialize() done (Status 0x%08lx)\n\0".as_ptr() as *const i8,
        Status,
    );

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortLogError(
    mut HwDeviceExtension: PVOID,
    mut Srb: PSCSI_REQUEST_BLOCK,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut ErrorCode: ULONG,
    mut UniqueId: ULONG,
) {
    if DbgPrint(
        b"(%s:%d) ScsiPortLogError() called\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1065 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1065 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) PathId: 0x%02x  TargetId: 0x%02x  Lun: 0x%02x  ErrorCode: 0x%08lx  UniqueId: 0x%08lx\n\0"
            as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1067 as ::core::ffi::c_int,
        PathId as ::core::ffi::c_int,
        TargetId as ::core::ffi::c_int,
        Lun as ::core::ffi::c_int,
        ErrorCode,
        UniqueId,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1067 as ::core::ffi::c_int,
        );
    }
    if DbgPrint(
        b"(%s:%d) ScsiPortLogError() done\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1069 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1069 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn StorPortMoveMemory(
    mut Destination: PVOID,
    mut Source: PVOID,
    mut Length: ULONG,
) {
    memmove(
        Destination as *mut ::core::ffi::c_void,
        Source as *const ::core::ffi::c_void,
        Length as usize,
    );
}

/// Fixed-arity twin of the `StorPortNotification` C-variadic entry point in
/// `drivers/storage/port/storport/rust_varargs_shim.c` (see `StorPortDebugPrint_v`).
/// `ap_list` is a pointer to the shim frame's `va_list`; `VaArgs::arg::<T>()` mirrors
/// nightly's `VaListImpl::arg::<T>()`, so the dispatch below is byte-for-byte the
/// translation it was. All the logic stays here.
#[no_mangle]
pub unsafe extern "C" fn StorPortNotification_v(
    mut NotificationType: SCSI_NOTIFICATION_TYPE,
    mut HwDeviceExtension: PVOID,
    ap_list: *mut core::ffi::c_void,
) {
    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PFDO_DEVICE_EXTENSION = core::ptr::null_mut();
    let mut HwPassiveInitRoutine: PHW_PASSIVE_INITIALIZE_ROUTINE;
    let mut ppExtendedFunctions: *mut PSTORPORT_EXTENDED_FUNCTIONS;
    let mut Result: PBOOLEAN;
    let mut Dpc: PSTOR_DPC;
    let mut HwDpcRoutine: PHW_DPC_ROUTINE;

    let mut SpinLock: STOR_SPINLOCK;
    let mut LockContext: PVOID;
    let mut LockHandle: PSTOR_LOCK_HANDLE;
    let mut Srb: PSCSI_REQUEST_BLOCK;

    /* Get the miniport extension */
    if !HwDeviceExtension.is_null() {
        MiniportExtension = (HwDeviceExtension as usize).wrapping_sub(
            core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension),
        ) as PMINIPORT_DEVICE_EXTENSION;

        DeviceExtension =
            (*(*MiniportExtension).Miniport).DeviceExtension as PFDO_DEVICE_EXTENSION;
    }

    let mut ap = VaArgs::from_ptr(ap_list);

    match NotificationType as u32 {
        0 => {
            // RequestComplete
            Srb = ap.arg::<PSCSI_REQUEST_BLOCK>();
            if !(*Srb).OriginalRequest.is_null() {
                /* Need to complete the IRP! */
            }
        }
        15 => {
            // GetExtendedFunctionTable
            ppExtendedFunctions = ap.arg::<*mut PSTORPORT_EXTENDED_FUNCTIONS>();
            if !ppExtendedFunctions.is_null() {
                *ppExtendedFunctions = core::ptr::null_mut(); /* FIXME */
            }
        }
        4096 => {
            // EnablePassiveInitialization
            HwPassiveInitRoutine = core::mem::transmute(
                ap.arg::<*mut core::ffi::c_void>(),
            );
            Result = ap.arg::<PBOOLEAN>();

            *Result = 0;

            if !DeviceExtension.is_null()
                && (*DeviceExtension).HwPassiveInitRoutine.is_none()
            {
                (*DeviceExtension).HwPassiveInitRoutine = HwPassiveInitRoutine;
                *Result = 1;
            }
        }
        4097 => {
            // InitializeDpc
            Dpc = ap.arg::<PSTOR_DPC>();
            HwDpcRoutine = core::mem::transmute(ap.arg::<*mut core::ffi::c_void>());

            KeInitializeDpc(
                core::ptr::addr_of_mut!((*Dpc).Dpc) as PRKDPC,
                { let p: *const () = core::mem::transmute(HwDpcRoutine); core::mem::transmute::<*const (), ntkrnl_sys::ddk::PKDEFERRED_ROUTINE>(p) },
                DeviceExtension as PVOID,
            );
            KeInitializeSpinLock(core::ptr::addr_of_mut!((*Dpc).Lock) as *mut usize);
        }
        4099 => {
            // AcquireSpinLock
            SpinLock = ap.arg::<STOR_SPINLOCK>();
            LockContext = ap.arg::<PVOID>();
            LockHandle = ap.arg::<PSTOR_LOCK_HANDLE>();
            PortAcquireSpinLock(DeviceExtension, SpinLock, LockContext, LockHandle);
        }
        4100 => {
            // ReleaseSpinLock
            LockHandle = ap.arg::<PSTOR_LOCK_HANDLE>();
            PortReleaseSpinLock(DeviceExtension, LockHandle);
        }
        _ => {
            // Unsupported Notification
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn StorPortPause(
    mut HwDeviceExtension: PVOID,
    mut TimeOut: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortPause()\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1219 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1219 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"StorPortPause\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1220 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortPauseDevice(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut TimeOut: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortPauseDevice()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1238 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1238 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortPauseDevice\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1239 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

// storport.c: `#if defined(_M_AMD64)` -- the export exists on amd64 only (storport.spec).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn StorPortQuerySystemTime(mut CurrentTime: PLARGE_INTEGER) {
    if DbgPrint(
        b"(%s:%d) StorPortQuerySystemTime(%p)\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1256 as ::core::ffi::c_int,
        CurrentTime,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1256 as ::core::ffi::c_int,
        );
    }
    // The C calls KeQuerySystemTime. This read `SharedSystemTime` = 0x7FFE0008, which is
    // KUSER_SHARED_DATA.InterruptTime at its USER-mode address: it returned the uptime as the
    // system time (and only worked while the current process maps the user page).
    ntkrnl_sys::inline::ke::KeQuerySystemTime(CurrentTime as *mut _);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortReady(mut HwDeviceExtension: PVOID) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortReady()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1272 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1272 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortReady\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1273 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortRegistryRead(
    mut HwDeviceExtension: PVOID,
    mut ValueName: PUCHAR,
    mut Global: ULONG,
    mut Type: ULONG,
    mut Buffer: PUCHAR,
    mut BufferLength: PULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortRegistryRead()\n\0" as *const u8 as *const i8,
        b"storport.c\0" as *const u8 as *const i8,
        1292 as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const i8,
            b"storport.c\0" as *const u8 as *const i8,
            1292 as i32,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const i8,
        b"StorPortRegistryRead\0" as *const u8 as *const i8,
        b"storport.c\0" as *const u8 as *const i8,
        1293 as i32,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortRegistryWrite(
    mut HwDeviceExtension: PVOID,
    mut ValueName: PUCHAR,
    mut Global: ULONG,
    mut Type: ULONG,
    mut Buffer: PUCHAR,
    mut BufferLength: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortRegistryWrite()\n\0".as_ptr() as *const i8,
        b"drivers/storage/port/storport/storport.c\0".as_ptr() as *const i8,
        1312 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"drivers/storage/port/storport/storport.c\0".as_ptr() as *const i8,
            1312 as core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const i8,
        b"StorPortRegistryWrite\0".as_ptr() as *const i8,
        b"drivers/storage/port/storport/storport.c\0".as_ptr() as *const i8,
        1313 as core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortResume(mut HwDeviceExtension: PVOID) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortResume()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1327 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1327 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortResume\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1328 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortResumeDevice(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortResumeDevice()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1345 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1345 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortResumeDevice\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1346 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSetBusDataByOffset(
    mut DeviceExtension: PVOID,
    mut BusDataType: ULONG,
    mut SystemIoBusNumber: ULONG,
    mut SlotNumber: ULONG,
    mut Buffer: PVOID,
    mut Offset: ULONG,
    mut Length: ULONG,
) -> ULONG {

    let mut MiniportExtension: PMINIPORT_DEVICE_EXTENSION =
        core::ptr::null_mut::<MINIPORT_DEVICE_EXTENSION>();
    let mut Interface: PBUS_INTERFACE_STANDARD =
        core::ptr::null_mut::<BUS_INTERFACE_STANDARD>();
    let mut ReturnLength: ULONG = 0;

    if DbgPrint(
        b"(%s:%d) StorPortSetBusData(%p %lu %lu %lu %p %lu %lu)\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        1371 as i32,
        DeviceExtension,
        BusDataType,
        SystemIoBusNumber,
        SlotNumber,
        Buffer,
        Offset,
        Length,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            1371 as i32,
        );
    }

    MiniportExtension = (DeviceExtension as usize).wrapping_sub(
        core::mem::offset_of!(MINIPORT_DEVICE_EXTENSION, HwDeviceExtension),
    ) as *mut MINIPORT_DEVICE_EXTENSION;

    if DbgPrint(
        b"(%s:%d) DeviceExtension %p  MiniportExtension %p\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        1377 as i32,
        DeviceExtension,
        MiniportExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            1377 as i32,
        );
    }

    Interface = core::ptr::addr_of_mut!(
        (*(*(*MiniportExtension).Miniport).DeviceExtension).BusInterface
    ) as PBUS_INTERFACE_STANDARD;

    ReturnLength = ((*Interface).SetBusData).expect("non-null function pointer")(
        (*Interface).Context,
        BusDataType,
        Buffer,
        Offset,
        Length,
    );

    if DbgPrint(
        b"(%s:%d) ReturnLength: %lu\n\0".as_ptr() as *const i8,
        b"storport.c\0".as_ptr() as *const i8,
        1386 as i32,
        ReturnLength,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"storport.c\0".as_ptr() as *const i8,
            1386 as i32,
        );
    }

    return ReturnLength;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSetDeviceQueueDepth(
    mut HwDeviceExtension: PVOID,
    mut PathId: UCHAR,
    mut TargetId: UCHAR,
    mut Lun: UCHAR,
    mut Depth: ULONG,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortSetDeviceQueueDepth()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1405 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1405 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortSetDeviceQueueDepth\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1406 as ::core::ffi::c_int,
    );
    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "system" fn StorPortStallExecution(Delay: ULONG) {
    KeStallExecutionProcessor(Delay);
}

#[no_mangle]
pub unsafe extern "system" fn StorPortSynchronizeAccess(
    mut HwDeviceExtension: PVOID,
    mut SynchronizedAccessRoutine: PSTOR_SYNCHRONIZED_ACCESS,
    mut Context: PVOID,
) {
    if DbgPrint(
        b"(%s:%d) StorPortSynchronizeAccess()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1435 as ::core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            1435 as ::core::ffi::c_int,
        );
    }
    DbgPrint(
        b"WARNING:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"StorPortSynchronizeAccess\0" as *const u8 as *const ::core::ffi::c_char,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        1436 as ::core::ffi::c_int,
    );
}

#[no_mangle]
pub unsafe extern "system" fn StorPortValidateRange(
    mut HwDeviceExtension: PVOID,
    mut BusType: INTERFACE_TYPE,
    mut SystemIoBusNumber: ULONG,
    mut IoAddress: STOR_PHYSICAL_ADDRESS,
    mut NumberOfBytes: ULONG,
    mut InIoSpace: BOOLEAN,
) -> BOOLEAN {
    if DbgPrint(
        b"(%s:%d) StorPortValidateRange()\n\0" as *const u8 as PCSTR,
        b"drivers/storage/port/storport/storport.c\0"
            as *const u8 as *const core::ffi::c_char,
        1454 as core::ffi::c_int,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"drivers/storage/port/storport/storport.c\0"
                as *const u8 as *const core::ffi::c_char,
            1454 as core::ffi::c_int,
        );
    }
    return 1 as BOOLEAN;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Storport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Storport driver main file
//  * COPYRIGHT:   Copyright 2017 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ********************************************************************/
//
// ULONG PortNumber = 0;
//
//
// /* FUNCTIONS ******************************************************************/
//
// static
// NTSTATUS
// PortAddDriverInitData(
//     PDRIVER_OBJECT_EXTENSION DriverExtension,
//     PHW_INITIALIZATION_DATA HwInitializationData)
// {
//     PDRIVER_INIT_DATA InitData;
//
//     DPRINT1("PortAddDriverInitData()\n");
//
//     InitData = ExAllocatePoolWithTag(NonPagedPool,
//                                      sizeof(DRIVER_INIT_DATA),
//                                      TAG_INIT_DATA);
//     if (InitData == NULL)
//         return STATUS_NO_MEMORY;
//
//     RtlCopyMemory(&InitData->HwInitData,
//                   HwInitializationData,
//                   sizeof(HW_INITIALIZATION_DATA));
//
//     InsertHeadList(&DriverExtension->InitDataListHead,
//                    &InitData->Entry);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// VOID
// PortDeleteDriverInitData(
//     PDRIVER_OBJECT_EXTENSION DriverExtension)
// {
//     PDRIVER_INIT_DATA InitData;
//     PLIST_ENTRY ListEntry;
//
//     DPRINT1("PortDeleteDriverInitData()\n");
//
//     ListEntry = DriverExtension->InitDataListHead.Flink;
//     while (ListEntry != &DriverExtension->InitDataListHead)
//     {
//         InitData = CONTAINING_RECORD(ListEntry,
//                                      DRIVER_INIT_DATA,
//                                      Entry);
//
//         RemoveEntryList(&InitData->Entry);
//
//         ExFreePoolWithTag(InitData,
//                           TAG_INIT_DATA);
//
//         ListEntry = DriverExtension->InitDataListHead.Flink;
//     }
// }
//
//
// PHW_INITIALIZATION_DATA
// PortGetDriverInitData(
//     PDRIVER_OBJECT_EXTENSION DriverExtension,
//     INTERFACE_TYPE InterfaceType)
// {
//     PDRIVER_INIT_DATA InitData;
//     PLIST_ENTRY ListEntry;
//
//     DPRINT1("PortGetDriverInitData()\n");
//
//     ListEntry = DriverExtension->InitDataListHead.Flink;
//     while (ListEntry != &DriverExtension->InitDataListHead)
//     {
//         InitData = CONTAINING_RECORD(ListEntry,
//                                      DRIVER_INIT_DATA,
//                                      Entry);
//         if (InitData->HwInitData.AdapterInterfaceType == InterfaceType)
//             return &InitData->HwInitData;
//
//         ListEntry = ListEntry->Flink;
//     }
//
//     return NULL;
// }
//
//
// static
// VOID
// PortAcquireSpinLock(
//     PFDO_DEVICE_EXTENSION DeviceExtension,
//     STOR_SPINLOCK SpinLock,
//     PVOID LockContext,
//     PSTOR_LOCK_HANDLE LockHandle)
// {
//     DPRINT1("PortAcquireSpinLock(%p %lu %p %p)\n",
//             DeviceExtension, SpinLock, LockContext, LockHandle);
//
//     LockHandle->Lock = SpinLock;
//
//     switch (SpinLock)
//     {
//         case DpcLock: /* 1, */
//             DPRINT1("DpcLock\n");
//             break;
//
//         case StartIoLock: /* 2 */
//             DPRINT1("StartIoLock\n");
//             break;
//
//         case InterruptLock: /* 3 */
//             DPRINT1("InterruptLock\n");
//             if (DeviceExtension->Interrupt == NULL)
//                 LockHandle->Context.OldIrql = 0;
//             else
//                 LockHandle->Context.OldIrql = KeAcquireInterruptSpinLock(DeviceExtension->Interrupt);
//             break;
//     }
// }
//
//
// static
// VOID
// PortReleaseSpinLock(
//     PFDO_DEVICE_EXTENSION DeviceExtension,
//     PSTOR_LOCK_HANDLE LockHandle)
// {
//     DPRINT1("PortReleaseSpinLock(%p %p)\n",
//             DeviceExtension, LockHandle);
//
//     switch (LockHandle->Lock)
//     {
//         case DpcLock: /* 1, */
//             DPRINT1("DpcLock\n");
//             break;
//
//         case StartIoLock: /* 2 */
//             DPRINT1("StartIoLock\n");
//             break;
//
//         case InterruptLock: /* 3 */
//             DPRINT1("InterruptLock\n");
//             if (DeviceExtension->Interrupt != NULL)
//                 KeReleaseInterruptSpinLock(DeviceExtension->Interrupt,
//                                            LockHandle->Context.OldIrql);
//             break;
//     }
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortAddDevice(
//      PDRIVER_OBJECT DriverObject,
//      PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     PDRIVER_OBJECT_EXTENSION DriverObjectExtension;
//     PFDO_DEVICE_EXTENSION DeviceExtension = NULL;
//     WCHAR NameBuffer[80];
//     UNICODE_STRING DeviceName;
//     PDEVICE_OBJECT Fdo = NULL;
//     KLOCK_QUEUE_HANDLE LockHandle;
//     NTSTATUS Status;
//
//     DPRINT1("PortAddDevice(%p %p)\n",
//             DriverObject, PhysicalDeviceObject);
//
//     ASSERT(DriverObject);
//     ASSERT(PhysicalDeviceObject);
//
//     swprintf(NameBuffer,
//              L"\\Device\\RaidPort%lu",
//              PortNumber);
//     RtlInitUnicodeString(&DeviceName, NameBuffer);
//     PortNumber++;
//
//     DPRINT1("Creating device: %wZ\n", &DeviceName);
//
//     /* Create the port device */
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(FDO_DEVICE_EXTENSION),
//                             &DeviceName,
//                             FILE_DEVICE_CONTROLLER,
//                             FILE_DEVICE_SECURE_OPEN,
//                             FALSE,
//                             &Fdo);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoCreateDevice() failed (Status 0x%08lx)\n", Status);
//         return Status;
//     }
//
//     DPRINT1("Created device: %wZ (%p)\n", &DeviceName, Fdo);
//
//     /* Initialize the device */
//     Fdo->Flags |= DO_DIRECT_IO;
//     Fdo->Flags |= DO_POWER_PAGABLE;
//
//     /* Initialize the device extension */
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)Fdo->DeviceExtension;
//     RtlZeroMemory(DeviceExtension, sizeof(FDO_DEVICE_EXTENSION));
//
//     DeviceExtension->ExtensionType = EXTENSION_TYPE_FdoExtension;
//
//     DeviceExtension->Device = Fdo;
//     DeviceExtension->PhysicalDevice = PhysicalDeviceObject;
//
//     DeviceExtension->PnpState = dsStopped;
//
//     KeInitializeSpinLock(&DeviceExtension->PdoListLock);
//     InitializeListHead(&DeviceExtension->PdoListHead);
//
//     /* Attach the FDO to the device stack */
//     Status = IoAttachDeviceToDeviceStackSafe(Fdo,
//                                              PhysicalDeviceObject,
//                                              &DeviceExtension->LowerDevice);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("IoAttachDeviceToDeviceStackSafe() failed (Status 0x%08lx)\n", Status);
//         IoDeleteDevice(Fdo);
//         return Status;
//     }
//
//     /* Insert the FDO to the drivers FDO list */
//     DriverObjectExtension = IoGetDriverObjectExtension(DriverObject,
//                                                        (PVOID)DriverEntry);
//     ASSERT(DriverObjectExtension->ExtensionType == DriverExtension);
//
//     DeviceExtension->DriverExtension = DriverObjectExtension;
//
//     KeAcquireInStackQueuedSpinLock(&DriverObjectExtension->AdapterListLock,
//                                    &LockHandle);
//
//     InsertHeadList(&DriverObjectExtension->AdapterListHead,
//                    &DeviceExtension->AdapterListEntry);
//     DriverObjectExtension->AdapterCount++;
//
//     KeReleaseInStackQueuedSpinLock(&LockHandle);
//
//     /* The device has been initialized */
//     Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     DPRINT1("PortAddDevice() done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// static
// VOID
// NTAPI
// PortUnload(
//      PDRIVER_OBJECT DriverObject)
// {
//     PDRIVER_OBJECT_EXTENSION DriverExtension;
//
//     DPRINT1("PortUnload(%p)\n",
//             DriverObject);
//
//     DriverExtension = IoGetDriverObjectExtension(DriverObject,
//                                                  (PVOID)DriverEntry);
//     if (DriverExtension != NULL)
//     {
//         PortDeleteDriverInitData(DriverExtension);
//     }
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchCreate(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     DPRINT1("PortDispatchCreate(%p %p)\n",
//             DeviceObject, Irp);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = FILE_OPENED;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchClose(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     DPRINT1("PortDispatchClose(%p %p)\n",
//             DeviceObject, Irp);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchDeviceControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     DPRINT1("PortDispatchDeviceControl(%p %p)\n",
//             DeviceObject, Irp);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchScsi(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//
//     DPRINT1("PortDispatchScsi(%p %p)\n",
//             DeviceObject, Irp);
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     DPRINT1("ExtensionType: %u\n", DeviceExtension->ExtensionType);
//
//     switch (DeviceExtension->ExtensionType)
//     {
//         case FdoExtension:
//             return PortFdoScsi(DeviceObject,
//                                Irp);
//
//         case PdoExtension:
//             return PortPdoScsi(DeviceObject,
//                                Irp);
//
//         default:
//             Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//             Irp->IoStatus.Information = 0;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_UNSUCCESSFUL;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchSystemControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     DPRINT1("PortDispatchSystemControl(%p %p)\n",
//             DeviceObject, Irp);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchPnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//
//     DPRINT1("PortDispatchPnp(%p %p)\n",
//             DeviceObject, Irp);
//
//     DeviceExtension = (PFDO_DEVICE_EXTENSION)DeviceObject->DeviceExtension;
//     DPRINT1("ExtensionType: %u\n", DeviceExtension->ExtensionType);
//
//     switch (DeviceExtension->ExtensionType)
//     {
//         case FdoExtension:
//             return PortFdoPnp(DeviceObject,
//                               Irp);
//
//         case PdoExtension:
//             return PortPdoPnp(DeviceObject,
//                               Irp);
//
//         default:
//             Irp->IoStatus.Status = STATUS_UNSUCCESSFUL;
//             Irp->IoStatus.Information = 0;
//             IoCompleteRequest(Irp, IO_NO_INCREMENT);
//             return STATUS_UNSUCCESSFUL;
//     }
// }
//
//
// static
// NTSTATUS
// NTAPI
// PortDispatchPower(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     DPRINT1("PortDispatchPower(%p %p)\n",
//             DeviceObject, Irp);
//
//     Irp->IoStatus.Status = STATUS_SUCCESS;
//     Irp->IoStatus.Information = 0;
//
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return STATUS_SUCCESS;
// }
//
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// DriverEntry(
//      PDRIVER_OBJECT DriverObject,
//      PUNICODE_STRING RegistryPath)
// {
//     DPRINT1("DriverEntry(%p %p)\n", DriverObject, RegistryPath);
//     return STATUS_SUCCESS;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// PUCHAR
// NTAPI
// StorPortAllocateRegistryBuffer(
//      PVOID HwDeviceExtension,
//      PULONG Length)
// {
//     DPRINT1("StorPortAllocateRegistryBuffer()\n");
//     UNIMPLEMENTED;
//     return NULL;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortBusy(
//      PVOID HwDeviceExtension,
//      ULONG RequestsToComplete)
// {
//     DPRINT1("StorPortBuzy()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortCompleteRequest(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      UCHAR SrbStatus)
// {
//     DPRINT1("StorPortCompleteRequest()\n");
//     UNIMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// ULONG
// NTAPI
// StorPortConvertPhysicalAddressToUlong(
//      STOR_PHYSICAL_ADDRESS Address)
// {
//     DPRINT1("StorPortConvertPhysicalAddressToUlong()\n");
//
//     return Address.u.LowPart;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// STOR_PHYSICAL_ADDRESS
// NTAPI
// StorPortConvertUlongToPhysicalAddress(
//      ULONG_PTR UlongAddress)
// {
//     STOR_PHYSICAL_ADDRESS Address;
//
//     DPRINT1("StorPortConvertUlongToPhysicalAddress()\n");
//
//     Address.QuadPart = UlongAddress;
//     return Address;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// VOID
// StorPortDebugPrint(
//      ULONG DebugPrintLevel,
//      PCHAR DebugMessage,
//     ...)
// {
//     va_list ap;
//
//     va_start(ap, DebugMessage);
//     vDbgPrintExWithPrefix("STORMINI: ", 0x58, DebugPrintLevel, DebugMessage, ap);
//     va_end(ap);
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortDeviceBusy(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      ULONG RequestsToComplete)
// {
//     DPRINT1("StorPortDeviceBusy()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortDeviceReady(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun)
// {
//     DPRINT1("StorPortDeviceReady()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// ULONG
// StorPortExtendedFunction(
//      STORPORT_FUNCTION_CODE FunctionCode,
//      PVOID HwDeviceExtension,
//     ...)
// {
//     DPRINT1("StorPortExtendedFunction(%d %p ...)\n",
//             FunctionCode, HwDeviceExtension);
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortFreeDeviceBase(
//      PVOID HwDeviceExtension,
//      PVOID MappedAddress)
// {
//     DPRINT1("StorPortFreeDeviceBase(%p %p)\n",
//             HwDeviceExtension, MappedAddress);
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortFreeRegistryBuffer(
//      PVOID HwDeviceExtension,
//      PUCHAR Buffer)
// {
//     DPRINT1("StorPortFreeRegistryBuffer()\n");
//     UNIMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// ULONG
// NTAPI
// StorPortGetBusData(
//      PVOID DeviceExtension,
//      ULONG BusDataType,
//      ULONG SystemIoBusNumber,
//      ULONG SlotNumber,
//      (Length != 0, ) PVOID Buffer,
//      ULONG Length)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     PBUS_INTERFACE_STANDARD Interface;
//     ULONG ReturnLength;
//
//     DPRINT1("StorPortGetBusData(%p %lu %lu %lu %p %lu)\n",
//             DeviceExtension, BusDataType, SystemIoBusNumber, SlotNumber, Buffer, Length);
//
//     /* Get the miniport extension */
//     MiniportExtension = CONTAINING_RECORD(DeviceExtension,
//                                           MINIPORT_DEVICE_EXTENSION,
//                                           HwDeviceExtension);
//     DPRINT1("DeviceExtension %p  MiniportExtension %p\n",
//             DeviceExtension, MiniportExtension);
//
//     Interface = &MiniportExtension->Miniport->DeviceExtension->BusInterface;
//
//     if (BusDataType == 4)
//         BusDataType = 0;
//
//     ReturnLength = Interface->GetBusData(Interface->Context,
//                                          BusDataType,
//                                          Buffer,
//                                          0,
//                                          Length);
//     DPRINT1("ReturnLength: %lu\n", ReturnLength);
//
//     return ReturnLength;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// PVOID
// NTAPI
// StorPortGetDeviceBase(
//      PVOID HwDeviceExtension,
//      INTERFACE_TYPE BusType,
//      ULONG SystemIoBusNumber,
//      STOR_PHYSICAL_ADDRESS IoAddress,
//      ULONG NumberOfBytes,
//      BOOLEAN InIoSpace)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     PHYSICAL_ADDRESS TranslatedAddress;
//     PVOID MappedAddress;
//     NTSTATUS Status;
//
//     DPRINT1("StorPortGetDeviceBase(%p %lu %lu 0x%I64x %lu %u)\n",
//             HwDeviceExtension, BusType, SystemIoBusNumber, IoAddress.QuadPart, NumberOfBytes, InIoSpace);
//
//     /* Get the miniport extension */
//     MiniportExtension = CONTAINING_RECORD(HwDeviceExtension,
//                                           MINIPORT_DEVICE_EXTENSION,
//                                           HwDeviceExtension);
//     DPRINT1("HwDeviceExtension %p  MiniportExtension %p\n",
//             HwDeviceExtension, MiniportExtension);
//
//     if (!TranslateResourceListAddress(MiniportExtension->Miniport->DeviceExtension,
//                                       BusType,
//                                       SystemIoBusNumber,
//                                       IoAddress,
//                                       NumberOfBytes,
//                                       InIoSpace,
//                                       &TranslatedAddress))
//     {
//         DPRINT1("Checkpoint!\n");
//         return NULL;
//     }
//
//     DPRINT1("Translated Address: 0x%I64x\n", TranslatedAddress.QuadPart);
//
//     /* In I/O space */
//     if (InIoSpace)
//     {
//         DPRINT1("Translated Address: %p\n", (PVOID)(ULONG_PTR)TranslatedAddress.QuadPart);
//         return (PVOID)(ULONG_PTR)TranslatedAddress.QuadPart;
//     }
//
//     /* In memory space */
//     MappedAddress = MmMapIoSpace(TranslatedAddress,
//                                  NumberOfBytes,
//                                  FALSE);
//     DPRINT1("Mapped Address: %p\n", MappedAddress);
//
//     Status = AllocateAddressMapping(&MiniportExtension->Miniport->DeviceExtension->MappedAddressList,
//                                     IoAddress,
//                                     MappedAddress,
//                                     NumberOfBytes,
//                                     SystemIoBusNumber);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Checkpoint!\n");
//         MappedAddress = NULL;
//     }
//
//     DPRINT1("Mapped Address: %p\n", MappedAddress);
//     return MappedAddress;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// PVOID
// NTAPI
// StorPortGetLogicalUnit(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun)
// {
//     DPRINT1("StorPortGetLogicalUnit()\n");
//     UNIMPLEMENTED;
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// STOR_PHYSICAL_ADDRESS
// NTAPI
// StorPortGetPhysicalAddress(
//      PVOID HwDeviceExtension,
//      PSCSI_REQUEST_BLOCK Srb,
//      PVOID VirtualAddress,
//      ULONG *Length)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     STOR_PHYSICAL_ADDRESS PhysicalAddress;
//     ULONG_PTR Offset;
//
//     DPRINT1("StorPortGetPhysicalAddress(%p %p %p %p)\n",
//             HwDeviceExtension, Srb, VirtualAddress, Length);
//
//     /* Get the miniport extension */
//     MiniportExtension = CONTAINING_RECORD(HwDeviceExtension,
//                                           MINIPORT_DEVICE_EXTENSION,
//                                           HwDeviceExtension);
//     DPRINT1("HwDeviceExtension %p  MiniportExtension %p\n",
//             HwDeviceExtension, MiniportExtension);
//
//     DeviceExtension = MiniportExtension->Miniport->DeviceExtension;
//
//     /* Inside of the uncached extension? */
//     if (((ULONG_PTR)VirtualAddress >= (ULONG_PTR)DeviceExtension->UncachedExtensionVirtualBase) &&
//         ((ULONG_PTR)VirtualAddress <= (ULONG_PTR)DeviceExtension->UncachedExtensionVirtualBase + DeviceExtension->UncachedExtensionSize))
//     {
//         Offset = (ULONG_PTR)VirtualAddress - (ULONG_PTR)DeviceExtension->UncachedExtensionVirtualBase;
//
//         PhysicalAddress.QuadPart = DeviceExtension->UncachedExtensionPhysicalBase.QuadPart + Offset;
//         *Length = DeviceExtension->UncachedExtensionSize - Offset;
//
//         return PhysicalAddress;
//     }
//
//     // FIXME
//
//
//     PhysicalAddress = MmGetPhysicalAddress(VirtualAddress);
//     *Length = 1;
// //    UNIMPLEMENTED;
//
// //    *Length = 0;
// //    PhysicalAddress.QuadPart = (LONGLONG)0;
//
//     return PhysicalAddress;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// PSTOR_SCATTER_GATHER_LIST
// NTAPI
// StorPortGetScatterGatherList(
//      PVOID DeviceExtension,
//      PSCSI_REQUEST_BLOCK Srb)
// {
//     DPRINT1("StorPortGetScatterGatherList()\n");
//     UNIMPLEMENTED;
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// PSCSI_REQUEST_BLOCK
// NTAPI
// StorPortGetSrb(
//      PVOID DeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      LONG QueueTag)
// {
//     DPRINT("StorPortGetSrb()\n");
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// PVOID
// NTAPI
// StorPortGetUncachedExtension(
//      PVOID HwDeviceExtension,
//      PPORT_CONFIGURATION_INFORMATION ConfigInfo,
//      ULONG NumberOfBytes)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     PFDO_DEVICE_EXTENSION DeviceExtension;
//     PHYSICAL_ADDRESS LowestAddress, HighestAddress, Alignment;
//
//     DPRINT1("StorPortGetUncachedExtension(%p %p %lu)\n",
//             HwDeviceExtension, ConfigInfo, NumberOfBytes);
//
//     /* Get the miniport extension */
//     MiniportExtension = CONTAINING_RECORD(HwDeviceExtension,
//                                           MINIPORT_DEVICE_EXTENSION,
//                                           HwDeviceExtension);
//     DPRINT1("HwDeviceExtension %p  MiniportExtension %p\n",
//             HwDeviceExtension, MiniportExtension);
//
//     DeviceExtension = MiniportExtension->Miniport->DeviceExtension;
//
//     /* Return the uncached extension base address if we already have one */
//     if (DeviceExtension->UncachedExtensionVirtualBase != NULL)
//         return DeviceExtension->UncachedExtensionVirtualBase;
//
//     // FIXME: Set DMA stuff here?
//
//     /* Allocate the uncached extension */
//     Alignment.QuadPart = 0;
//     LowestAddress.QuadPart = 0;
//     HighestAddress.QuadPart = 0x00000000FFFFFFFF;
//     DeviceExtension->UncachedExtensionVirtualBase = MmAllocateContiguousMemorySpecifyCache(NumberOfBytes,
//                                                                                            LowestAddress,
//                                                                                            HighestAddress,
//                                                                                            Alignment,
//                                                                                            MmCached);
//     if (DeviceExtension->UncachedExtensionVirtualBase == NULL)
//         return NULL;
//
//     DeviceExtension->UncachedExtensionPhysicalBase = MmGetPhysicalAddress(DeviceExtension->UncachedExtensionVirtualBase);
//     DeviceExtension->UncachedExtensionSize = NumberOfBytes;
//
//     return DeviceExtension->UncachedExtensionVirtualBase;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// PVOID
// NTAPI
// StorPortGetVirtualAddress(
//      PVOID HwDeviceExtension,
//      STOR_PHYSICAL_ADDRESS PhysicalAddress)
// {
//     DPRINT1("StorPortGetVirtualAddress(%p %I64x)\n",
//             HwDeviceExtension, PhysicalAddress.QuadPart);
//     UNIMPLEMENTED;
//     return NULL;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// ULONG
// NTAPI
// StorPortInitialize(
//      PVOID Argument1,
//      PVOID Argument2,
//      struct _HW_INITIALIZATION_DATA *HwInitializationData,
//      PVOID HwContext)
// {
//     PDRIVER_OBJECT DriverObject = (PDRIVER_OBJECT)Argument1;
//     PUNICODE_STRING RegistryPath = (PUNICODE_STRING)Argument2;
//     PDRIVER_OBJECT_EXTENSION DriverObjectExtension;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     DPRINT1("StorPortInitialize(%p %p %p %p)\n",
//             Argument1, Argument2, HwInitializationData, HwContext);
//
//     DPRINT1("HwInitializationDataSize: %lu\n", HwInitializationData->HwInitializationDataSize);
//     DPRINT1("AdapterInterfaceType: %u\n", HwInitializationData->AdapterInterfaceType);
//     DPRINT1("HwInitialize: %p\n", HwInitializationData->HwInitialize);
//     DPRINT1("HwStartIo: %p\n", HwInitializationData->HwStartIo);
//     DPRINT1("HwInterrupt: %p\n", HwInitializationData->HwInterrupt);
//     DPRINT1("HwFindAdapter: %p\n", HwInitializationData->HwFindAdapter);
//     DPRINT1("HwResetBus: %p\n", HwInitializationData->HwResetBus);
//     DPRINT1("HwDmaStarted: %p\n", HwInitializationData->HwDmaStarted);
//     DPRINT1("HwAdapterState: %p\n", HwInitializationData->HwAdapterState);
//     DPRINT1("DeviceExtensionSize: %lu\n", HwInitializationData->DeviceExtensionSize);
//     DPRINT1("SpecificLuExtensionSize: %lu\n", HwInitializationData->SpecificLuExtensionSize);
//     DPRINT1("SrbExtensionSize: %lu\n", HwInitializationData->SrbExtensionSize);
//     DPRINT1("NumberOfAccessRanges: %lu\n", HwInitializationData->NumberOfAccessRanges);
//
//     /* Check parameters */
//     if ((DriverObject == NULL) ||
//         (RegistryPath == NULL) ||
//         (HwInitializationData == NULL))
//     {
//         DPRINT1("Invalid parameter!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Check initialization data */
//     if ((HwInitializationData->HwInitializationDataSize < sizeof(HW_INITIALIZATION_DATA)) ||
//         (HwInitializationData->HwInitialize == NULL) ||
//         (HwInitializationData->HwStartIo == NULL) ||
//         (HwInitializationData->HwFindAdapter == NULL) ||
//         (HwInitializationData->HwResetBus == NULL))
//     {
//         DPRINT1("Revision mismatch!\n");
//         return STATUS_REVISION_MISMATCH;
//     }
//
//     DriverObjectExtension = IoGetDriverObjectExtension(DriverObject,
//                                                        (PVOID)DriverEntry);
//     if (DriverObjectExtension == NULL)
//     {
//         DPRINT1("No driver object extension!\n");
//
//         Status = IoAllocateDriverObjectExtension(DriverObject,
//                                                  (PVOID)DriverEntry,
//                                                  sizeof(DRIVER_OBJECT_EXTENSION),
//                                                  (PVOID *)&DriverObjectExtension);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IoAllocateDriverObjectExtension() failed (Status 0x%08lx)\n", Status);
//             return Status;
//         }
//
//         DPRINT1("Driver object extension created!\n");
//
//         /* Initialize the driver object extension */
//         RtlZeroMemory(DriverObjectExtension,
//                       sizeof(DRIVER_OBJECT_EXTENSION));
//
//         DriverObjectExtension->ExtensionType = EXTENSION_TYPE_DriverExtension;
//         DriverObjectExtension->DriverObject = DriverObject;
//
//         InitializeListHead(&DriverObjectExtension->AdapterListHead);
//         KeInitializeSpinLock(&DriverObjectExtension->AdapterListLock);
//
//         InitializeListHead(&DriverObjectExtension->InitDataListHead);
//
//         /* Set handlers */
//         DriverObject->DriverExtension->AddDevice = PortAddDevice;
// //        DriverObject->DriverStartIo = PortStartIo;
//         DriverObject->DriverUnload = PortUnload;
//         DriverObject->MajorFunction[IRP_MJ_CREATE] = PortDispatchCreate;
//         DriverObject->MajorFunction[IRP_MJ_CLOSE] = PortDispatchClose;
//         DriverObject->MajorFunction[IRP_MJ_DEVICE_CONTROL] = PortDispatchDeviceControl;
//         DriverObject->MajorFunction[IRP_MJ_SCSI] = PortDispatchScsi;
//         DriverObject->MajorFunction[IRP_MJ_POWER] = PortDispatchPower;
//         DriverObject->MajorFunction[IRP_MJ_SYSTEM_CONTROL] = PortDispatchSystemControl;
//         DriverObject->MajorFunction[IRP_MJ_PNP] = PortDispatchPnp;
//     }
//
//     /* Add the initialzation data to the driver extension */
//     Status = PortAddDriverInitData(DriverObjectExtension,
//                                    HwInitializationData);
//
//     DPRINT1("StorPortInitialize() done (Status 0x%08lx)\n", Status);
//
//     return Status;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortLogError(
//      PVOID HwDeviceExtension,
//      PSCSI_REQUEST_BLOCK Srb,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      ULONG ErrorCode,
//      ULONG UniqueId)
// {
//     DPRINT1("ScsiPortLogError() called\n");
//     DPRINT1("PathId: 0x%02x  TargetId: 0x%02x  Lun: 0x%02x  ErrorCode: 0x%08lx  UniqueId: 0x%08lx\n",
//             PathId, TargetId, Lun, ErrorCode, UniqueId);
//
//     DPRINT1("ScsiPortLogError() done\n");
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortMoveMemory(
//      PVOID Destination,
//      PVOID Source,
//      ULONG Length)
// {
//     RtlMoveMemory(Destination, Source, Length);
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// VOID
// StorPortNotification(
//      SCSI_NOTIFICATION_TYPE NotificationType,
//      PVOID HwDeviceExtension,
//     ...)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension = NULL;
//     PFDO_DEVICE_EXTENSION DeviceExtension = NULL;
//     PHW_PASSIVE_INITIALIZE_ROUTINE HwPassiveInitRoutine;
//     PSTORPORT_EXTENDED_FUNCTIONS *ppExtendedFunctions;
//     PBOOLEAN Result;
//     PSTOR_DPC Dpc;
//     PHW_DPC_ROUTINE HwDpcRoutine;
//     va_list ap;
//
//     STOR_SPINLOCK SpinLock;
//     PVOID LockContext;
//     PSTOR_LOCK_HANDLE LockHandle;
//     PSCSI_REQUEST_BLOCK Srb;
//
//     DPRINT1("StorPortNotification(%x %p)\n",
//             NotificationType, HwDeviceExtension);
//
//     /* Get the miniport extension */
//     if (HwDeviceExtension != NULL)
//     {
//         MiniportExtension = CONTAINING_RECORD(HwDeviceExtension,
//                                               MINIPORT_DEVICE_EXTENSION,
//                                               HwDeviceExtension);
//         DPRINT1("HwDeviceExtension %p  MiniportExtension %p\n",
//                 HwDeviceExtension, MiniportExtension);
//
//         DeviceExtension = MiniportExtension->Miniport->DeviceExtension;
//     }
//
//     va_start(ap, HwDeviceExtension);
//
//     switch (NotificationType)
//     {
//         case RequestComplete:
//             DPRINT1("RequestComplete\n");
//             Srb = (PSCSI_REQUEST_BLOCK)va_arg(ap, PSCSI_REQUEST_BLOCK);
//             DPRINT1("Srb %p\n", Srb);
//             if (Srb->OriginalRequest != NULL)
//             {
//                 DPRINT1("Need to complete the IRP!\n");
//
//             }
//             break;
//
//         case GetExtendedFunctionTable:
//             DPRINT1("GetExtendedFunctionTable\n");
//             ppExtendedFunctions = (PSTORPORT_EXTENDED_FUNCTIONS*)va_arg(ap, PSTORPORT_EXTENDED_FUNCTIONS*);
//             if (ppExtendedFunctions != NULL)
//                 *ppExtendedFunctions = NULL; /* FIXME */
//             break;
//
//         case EnablePassiveInitialization:
//             DPRINT1("EnablePassiveInitialization\n");
//             HwPassiveInitRoutine = (PHW_PASSIVE_INITIALIZE_ROUTINE)va_arg(ap, PHW_PASSIVE_INITIALIZE_ROUTINE);
//             DPRINT1("HwPassiveInitRoutine %p\n", HwPassiveInitRoutine);
//             Result = (PBOOLEAN)va_arg(ap, PBOOLEAN);
//
//             *Result = FALSE;
//
//             if ((DeviceExtension != NULL) &&
//                 (DeviceExtension->HwPassiveInitRoutine == NULL))
//             {
//                 DeviceExtension->HwPassiveInitRoutine = HwPassiveInitRoutine;
//                 *Result = TRUE;
//             }
//             break;
//
//         case InitializeDpc:
//             DPRINT1("InitializeDpc\n");
//             Dpc = (PSTOR_DPC)va_arg(ap, PSTOR_DPC);
//             DPRINT1("Dpc %p\n", Dpc);
//             HwDpcRoutine = (PHW_DPC_ROUTINE)va_arg(ap, PHW_DPC_ROUTINE);
//             DPRINT1("HwDpcRoutine %p\n", HwDpcRoutine);
//
//             KeInitializeDpc((PRKDPC)&Dpc->Dpc,
//                             (PKDEFERRED_ROUTINE)HwDpcRoutine,
//                             (PVOID)DeviceExtension);
//             KeInitializeSpinLock(&Dpc->Lock);
//             break;
//
//         case AcquireSpinLock:
//             DPRINT1("AcquireSpinLock\n");
//             SpinLock = (STOR_SPINLOCK)va_arg(ap, STOR_SPINLOCK);
//             DPRINT1("SpinLock %lu\n", SpinLock);
//             LockContext = (PVOID)va_arg(ap, PVOID);
//             DPRINT1("LockContext %p\n", LockContext);
//             LockHandle = (PSTOR_LOCK_HANDLE)va_arg(ap, PSTOR_LOCK_HANDLE);
//             DPRINT1("LockHandle %p\n", LockHandle);
//             PortAcquireSpinLock(DeviceExtension,
//                                 SpinLock,
//                                 LockContext,
//                                 LockHandle);
//             break;
//
//         case ReleaseSpinLock:
//             DPRINT1("ReleaseSpinLock\n");
//             LockHandle = (PSTOR_LOCK_HANDLE)va_arg(ap, PSTOR_LOCK_HANDLE);
//             DPRINT1("LockHandle %p\n", LockHandle);
//             PortReleaseSpinLock(DeviceExtension,
//                                 LockHandle);
//             break;
//
//         default:
//             DPRINT1("Unsupported Notification %lx\n", NotificationType);
//             break;
//     }
//
//     va_end(ap);
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortPause(
//      PVOID HwDeviceExtension,
//      ULONG TimeOut)
// {
//     DPRINT1("StorPortPause()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortPauseDevice(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      ULONG TimeOut)
// {
//     DPRINT1("StorPortPauseDevice()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// #if defined(_M_AMD64)
// /*
//  * @implemented
//  */
// /* KeQuerySystemTime is an inline function,
//    so we cannot forward the export to ntoskrnl */
// STORPORT_API
// VOID
// NTAPI
// StorPortQuerySystemTime(
//      PLARGE_INTEGER CurrentTime)
// {
//     DPRINT1("StorPortQuerySystemTime(%p)\n", CurrentTime);
//
//     KeQuerySystemTime(CurrentTime);
// }
// #endif /* defined(_M_AMD64) */
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortReady(
//      PVOID HwDeviceExtension)
// {
//     DPRINT1("StorPortReady()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortRegistryRead(
//      PVOID HwDeviceExtension,
//      PUCHAR ValueName,
//      ULONG Global,
//      ULONG Type,
//      PUCHAR Buffer,
//      PULONG BufferLength)
// {
//     DPRINT1("StorPortRegistryRead()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortRegistryWrite(
//      PVOID HwDeviceExtension,
//      PUCHAR ValueName,
//      ULONG Global,
//      ULONG Type,
//      PUCHAR Buffer,
//      ULONG BufferLength)
// {
//     DPRINT1("StorPortRegistryWrite()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortResume(
//      PVOID HwDeviceExtension)
// {
//     DPRINT1("StorPortResume()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortResumeDevice(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun)
// {
//     DPRINT1("StorPortResumeDevice()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// ULONG
// NTAPI
// StorPortSetBusDataByOffset(
//      PVOID DeviceExtension,
//      ULONG BusDataType,
//      ULONG SystemIoBusNumber,
//      ULONG SlotNumber,
//      PVOID Buffer,
//      ULONG Offset,
//      ULONG Length)
// {
//     PMINIPORT_DEVICE_EXTENSION MiniportExtension;
//     PBUS_INTERFACE_STANDARD Interface;
//     ULONG ReturnLength;
//
//     DPRINT1("StorPortSetBusData(%p %lu %lu %lu %p %lu %lu)\n",
//             DeviceExtension, BusDataType, SystemIoBusNumber, SlotNumber, Buffer, Offset, Length);
//
//     MiniportExtension = CONTAINING_RECORD(DeviceExtension,
//                                           MINIPORT_DEVICE_EXTENSION,
//                                           HwDeviceExtension);
//     DPRINT1("DeviceExtension %p  MiniportExtension %p\n",
//             DeviceExtension, MiniportExtension);
//
//     Interface = &MiniportExtension->Miniport->DeviceExtension->BusInterface;
//
//     ReturnLength = Interface->SetBusData(Interface->Context,
//                                          BusDataType,
//                                          Buffer,
//                                          Offset,
//                                          Length);
//     DPRINT1("ReturnLength: %lu\n", ReturnLength);
//
//     return ReturnLength;
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortSetDeviceQueueDepth(
//      PVOID HwDeviceExtension,
//      UCHAR PathId,
//      UCHAR TargetId,
//      UCHAR Lun,
//      ULONG Depth)
// {
//     DPRINT1("StorPortSetDeviceQueueDepth()\n");
//     UNIMPLEMENTED;
//     return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortStallExecution(
//      ULONG Delay)
// {
//     KeStallExecutionProcessor(Delay);
// }
//
//
// /*
//  * @unimplemented
//  */
// STORPORT_API
// VOID
// NTAPI
// StorPortSynchronizeAccess(
//      PVOID HwDeviceExtension,
//      PSTOR_SYNCHRONIZED_ACCESS SynchronizedAccessRoutine,
//      PVOID Context)
// {
//     DPRINT1("StorPortSynchronizeAccess()\n");
//     UNIMPLEMENTED;
// }
//
//
// /*
//  * @implemented
//  */
// STORPORT_API
// BOOLEAN
// NTAPI
// StorPortValidateRange(
//      PVOID HwDeviceExtension,
//      INTERFACE_TYPE BusType,
//      ULONG SystemIoBusNumber,
//      STOR_PHYSICAL_ADDRESS IoAddress,
//      ULONG NumberOfBytes,
//      BOOLEAN InIoSpace)
// {
//     DPRINT1("StorPortValidateRange()\n");
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
