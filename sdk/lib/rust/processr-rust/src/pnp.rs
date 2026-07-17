use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{
    _KWAIT_REASON_Executive, IO_STACK_LOCATION, _IO_STATUS_BLOCK,
    IoAttachDeviceToDeviceStackSafe, IoBuildSynchronousFsdRequest, IoCreateDevice, IoDeleteDevice,
    IoForwardIrpSynchronously, IoGetAttachedDeviceReference, KEVENT,
    KeInitializeEvent, KeWaitForSingleObject, _EVENT_TYPE_NotificationEvent,
    PCM_RESOURCE_LIST, PDEVICE_OBJECT, PDRIVER_OBJECT, PIO_STACK_LOCATION, PIRP,
    PWSTR, BUS_QUERY_ID_TYPE,
    _DEVICE_RELATION_TYPE_BusRelations, _DEVICE_RELATION_TYPE_RemovalRelations,
};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoGetNextIrpStackLocation};
use crate::shared::{DEVICE_EXTENSION, _DEVICE_EXTENSION};

extern "C" {
    fn DbgPrint(format: *const i8, ...) -> i32;
    fn wcslen(s: *const u16) -> usize;
    fn swprintf(buffer: *mut u16, format: *const u16, ...) -> i32;
}
// KeGetCurrentIrql is inline on amd64/arm64 (not an export) — use the cross-arch
// ke wrapper rather than an extern decl.
use ntkrnl_sys::ke::KeGetCurrentIrql;
// NTAPI (stdcall on i386) externals.
extern "system" {
    fn RtlAssert(assertion: *const core::ffi::c_void, source: *const i8, line: u32, message: *const i8);
    fn ForwardIrpAndForget(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 1;

const KEY_WRITE: u32 = 0x20006;
// IRP_MN_QUERY_ID (0x13) and IRP_MN_FILTER_RESOURCE_REQUIREMENTS (0x0d) are not
// in driver_ddk — define locally (wdm.h values).
const IRP_MN_QUERY_ID: u8 = 0x13;
const IRP_MN_FILTER_RESOURCE_REQUIREMENTS: u8 = 0x0d;

const APC_LEVEL: u32 = 1;
unsafe fn GetDeviceId(
    DeviceObject: PDEVICE_OBJECT,
    IdType: BUS_QUERY_ID_TYPE,
    DeviceId: *mut PWSTR,
) -> NTSTATUS {
    let IrpStack: PIO_STACK_LOCATION;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let TargetObject: PDEVICE_OBJECT;
    let mut Event: KEVENT = core::mem::zeroed();
    let Irp: PIRP;
    let mut Status: NTSTATUS;

    // PAGED_CODE();
    if (KeGetCurrentIrql() as u32) > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if (0 as i32) == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *const core::ffi::c_void,
                b"drivers/processor/processr/pnp.c\0"
                    .as_ptr() as *const i8,
                33,
                core::ptr::null_mut(),
            );
        }
    }

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0);
    TargetObject = IoGetAttachedDeviceReference(DeviceObject);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        TargetObject,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatus),
    );
    if Irp.is_null() {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

        IrpStack = IoGetNextIrpStackLocation(Irp);
        RtlZeroMemory(IrpStack as *mut core::ffi::c_void, core::mem::size_of::<IO_STACK_LOCATION>());
        (*IrpStack).MajorFunction = IRP_MJ_PNP as u8;
        (*IrpStack).MinorFunction = IRP_MN_QUERY_ID as u8;
        (*IrpStack).Parameters.QueryId.IdType = IdType;

        Status = IoCallDriver(TargetObject, Irp);
        if Status == STATUS_PENDING {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!(Event) as *mut core::ffi::c_void,
                _KWAIT_REASON_Executive,
                0i8,
                0,
                core::ptr::null_mut(),
            );
            Status = IoStatus.__bindgen_anon_1.Status;
        }

        if NT_SUCCESS(Status) {
            *DeviceId = IoStatus.Information as PWSTR;
        }
    }

    ObDereferenceObject(TargetObject as *mut core::ffi::c_void);

    Status
}

unsafe fn ProcessorSetFriendlyName(DeviceObject: ntkrnl_sys::ddk::PDEVICE_OBJECT) {
    let mut Buffer: *mut ntkrnl_sys::ddk::KEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut ObjectAttributes: ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES = ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut HardwareKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = ntkrnl_sys::ddk::_UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ValueName: ntkrnl_sys::ddk::_UNICODE_STRING = ntkrnl_sys::ddk::_UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut EnumKeyName: ntkrnl_sys::ddk::_UNICODE_STRING = ntkrnl_sys::ddk::_UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyHandle: ntkrnl_sys::ddk::HANDLE = core::ptr::null_mut();
    let mut DataLength: u32 = 0;
    let mut BufferLength: u32 = 0;
    let mut Status: ntkrnl_sys::base::NTSTATUS = 0;
    let mut KeyNameBuffer: *mut u16 = core::ptr::null_mut();
    let mut DeviceId: *mut u16 = core::ptr::null_mut();
    let mut InstanceId: *mut u16 = core::ptr::null_mut();
    let pszPrefix: *const u16 = &core::mem::transmute::<[u8; 96], [u16; 48]>(
        *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0E\0n\0u\0m\0\0\0",
    ) as *const u16;

    ntkrnl_sys::ddk::RtlInitUnicodeString(
        core::ptr::addr_of_mut!(HardwareKeyName),
        &core::mem::transmute::<[u8; 130], [u16; 65]>(
            *b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0y\0s\0t\0e\0m\0\\\0C\0e\0n\0t\0r\0a\0l\0P\0r\0o\0c\0e\0s\0s\0o\0r\0\\\x000\0\0\0",
        ) as *const u16,
    );

    ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(HardwareKeyName);
    ObjectAttributes.Attributes = ntkrnl_sys::constants::OBJ_CASE_INSENSITIVE as u32 | ntkrnl_sys::constants::OBJ_KERNEL_HANDLE as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ntkrnl_sys::ddk::ZwOpenKey(
        core::ptr::addr_of_mut!(KeyHandle),
        ntkrnl_sys::constants::KEY_READ,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if !(Status >= 0) {
        if DbgPrint(
            b"(%s:%d) ZwOpenKey() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
            122 as i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                122 as i32,
            );
        }
        return;
    }

    ntkrnl_sys::ddk::RtlInitUnicodeString(
        core::ptr::addr_of_mut!(ValueName),
        &core::mem::transmute::<[u8; 40], [u16; 20]>(
            *b"P\0r\0o\0c\0e\0s\0s\0o\0r\0N\0a\0m\0e\0S\0t\0r\0i\0n\0g\0\0\0",
        ) as *const u16,
    );

    Status = ntkrnl_sys::ddk::ZwQueryValueKey(
        KeyHandle,
        core::ptr::addr_of_mut!(ValueName) as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
        ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
        core::ptr::null_mut(),
        0,
        core::ptr::addr_of_mut!(DataLength),
    );
    if Status != ntkrnl_sys::constants::STATUS_BUFFER_OVERFLOW
        && Status != ntkrnl_sys::constants::STATUS_BUFFER_TOO_SMALL
        && Status != ntkrnl_sys::base::STATUS_SUCCESS
    {
        if DbgPrint(
            b"(%s:%d) ZwQueryValueKey() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
            136 as i32,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                136 as i32,
            );
        }
    } else {
        Buffer = ntkrnl_sys::ddk::ExAllocatePool(
            ntkrnl_sys::mm::PagedPool,
            (DataLength as usize + core::mem::size_of::<ntkrnl_sys::ddk::KEY_VALUE_PARTIAL_INFORMATION>()) as ntkrnl_sys::ddk::SIZE_T,
        ) as *mut ntkrnl_sys::ddk::KEY_VALUE_PARTIAL_INFORMATION;
        if Buffer.is_null() {
            if DbgPrint(
                b"(%s:%d) ExAllocatePool() failed\n\0".as_ptr() as *const i8,
                b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                144 as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                    144 as i32,
                );
            }
        } else {
            Status = ntkrnl_sys::ddk::ZwQueryValueKey(
                KeyHandle,
                core::ptr::addr_of_mut!(ValueName) as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
                ntkrnl_sys::ddk::_KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation,
                Buffer as *mut core::ffi::c_void,
                (DataLength as usize + core::mem::size_of::<ntkrnl_sys::ddk::KEY_VALUE_PARTIAL_INFORMATION>() as usize) as u32,
                core::ptr::addr_of_mut!(DataLength),
            );
            if !(Status >= 0) {
                if DbgPrint(
                    b"(%s:%d) ZwQueryValueKey() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                    b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                    156 as i32,
                    Status,
                ) != 0
                {
                    DbgPrint(
                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                        b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                        156 as i32,
                    );
                }
            } else {
                ntkrnl_sys::ddk::ZwClose(KeyHandle);
                KeyHandle = core::ptr::null_mut();

                Status = GetDeviceId(DeviceObject, ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryDeviceID, core::ptr::addr_of_mut!(DeviceId));
                if !(Status >= 0) {
                    if DbgPrint(
                        b"(%s:%d) GetDeviceId() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                        b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                        170 as i32,
                        Status,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                            b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                            170 as i32,
                        );
                    }
                } else {
                    Status = GetDeviceId(DeviceObject, ntkrnl_sys::ddk::_BUS_QUERY_ID_TYPE_BusQueryInstanceID, core::ptr::addr_of_mut!(InstanceId));
                    if !(Status >= 0) {
                        if DbgPrint(
                            b"(%s:%d) GetDeviceId() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                            b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                            181 as i32,
                            Status,
                        ) != 0
                        {
                            DbgPrint(
                                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                181 as i32,
                            );
                        }
                    } else {
                        BufferLength = (wcslen(pszPrefix) as u32)
                            .wrapping_add(1)
                            .wrapping_add(wcslen(DeviceId) as u32)
                            .wrapping_add(1)
                            .wrapping_add(wcslen(InstanceId) as u32)
                            .wrapping_add(1);
                        KeyNameBuffer = ntkrnl_sys::ddk::ExAllocatePool(
                            ntkrnl_sys::mm::PagedPool,
                            (BufferLength as usize).wrapping_mul(core::mem::size_of::<u16>()) as ntkrnl_sys::ddk::SIZE_T,
                        ) as *mut u16;
                        if KeyNameBuffer.is_null() {
                            if DbgPrint(
                                b"(%s:%d) ExAllocatePool() failed\n\0".as_ptr() as *const i8,
                                b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                192 as i32,
                            ) != 0
                            {
                                DbgPrint(
                                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                    b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                    192 as i32,
                                );
                            }
                        } else {
                            swprintf(
                                KeyNameBuffer,
                                &core::mem::transmute::<[u8; 18], [u16; 9]>(
                                    *b"%\0s\0\\\0%\0s\0\\\0%\0s\0\0\0"
                                ) as *const u16,
                                pszPrefix,
                                DeviceId,
                                InstanceId,
                            );
                            ntkrnl_sys::ddk::RtlInitUnicodeString(
                                core::ptr::addr_of_mut!(EnumKeyName),
                                KeyNameBuffer as *const u16,
                            );
                            ObjectAttributes.Length = core::mem::size_of::<ntkrnl_sys::ddk::_OBJECT_ATTRIBUTES>() as u32;
                            ObjectAttributes.RootDirectory = core::ptr::null_mut();
                            ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(EnumKeyName);
                            ObjectAttributes.Attributes = ntkrnl_sys::constants::OBJ_CASE_INSENSITIVE as u32 | ntkrnl_sys::constants::OBJ_KERNEL_HANDLE as u32;
                            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                            Status = ntkrnl_sys::ddk::ZwOpenKey(
                                core::ptr::addr_of_mut!(KeyHandle),
                                KEY_WRITE,
                                core::ptr::addr_of_mut!(ObjectAttributes),
                            );
                            if !(Status >= 0) {
                                if DbgPrint(
                                    b"(%s:%d) ZwOpenKey() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                                    b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                    209 as i32,
                                    Status,
                                ) != 0
                                {
                                    DbgPrint(
                                        b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                        b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                        209 as i32,
                                    );
                                }
                            } else {
                                ntkrnl_sys::ddk::RtlInitUnicodeString(
                                    core::ptr::addr_of_mut!(ValueName),
                                    &core::mem::transmute::<[u8; 26], [u16; 13]>(
                                        *b"F\0r\0i\0e\0n\0d\0l\0y\0N\0a\0m\0e\0\0\0",
                                    ) as *const u16,
                                );
                                Status = ntkrnl_sys::ddk::ZwSetValueKey(
                                    KeyHandle,
                                    core::ptr::addr_of_mut!(ValueName) as *mut ntkrnl_sys::ddk::_UNICODE_STRING,
                                    0,
                                    ntkrnl_sys::constants::REG_SZ,
                                    (&raw mut (*Buffer).Data as *mut u8) as *mut core::ffi::c_void,
                                    (*Buffer).DataLength,
                                );
                                if !(Status >= 0) {
                                    if DbgPrint(
                                        b"(%s:%d) ZwSetValueKey() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                                        b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                        223 as i32,
                                        Status,
                                    ) != 0
                                    {
                                        DbgPrint(
                                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                                            b"drivers/processor/processr/pnp.c\0".as_ptr() as *const i8,
                                            223 as i32,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !KeyHandle.is_null() {
        ntkrnl_sys::ddk::ZwClose(KeyHandle);
    }
    if !KeyNameBuffer.is_null() {
        ntkrnl_sys::ddk::ExFreePool(KeyNameBuffer as *mut core::ffi::c_void);
    }
    if !InstanceId.is_null() {
        ntkrnl_sys::ddk::ExFreePool(InstanceId as *mut core::ffi::c_void);
    }
    if !DeviceId.is_null() {
        ntkrnl_sys::ddk::ExFreePool(DeviceId as *mut core::ffi::c_void);
    }
    if !Buffer.is_null() {
        ntkrnl_sys::ddk::ExFreePool(Buffer as *mut core::ffi::c_void);
    }
}
unsafe fn ProcessorStartDevice(
    DeviceObject: PDEVICE_OBJECT,
    _ResourceList: PCM_RESOURCE_LIST,
    _ResourceListTranslated: PCM_RESOURCE_LIST,
) -> NTSTATUS {
    ProcessorSetFriendlyName(DeviceObject);
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn ProcessorPnp(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let mut DeviceExtension: *mut _DEVICE_EXTENSION = core::ptr::null_mut();
    let mut IrpSp: PIO_STACK_LOCATION = core::ptr::null_mut();
    let Information: usize = 0;
    let mut Status: NTSTATUS = STATUS_NOT_SUPPORTED;

    IrpSp = IoGetCurrentIrpStackLocation(Irp);

    match (*IrpSp).MinorFunction {
        IRP_MN_START_DEVICE => {
            DeviceExtension = (*DeviceObject).DeviceExtension as *mut _DEVICE_EXTENSION;
            Status = STATUS_UNSUCCESSFUL;
            if IoForwardIrpSynchronously((*DeviceExtension).LowerDevice, Irp) != 0 {
                Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
                if NT_SUCCESS(Status) {
                    Status = ProcessorStartDevice(
                        DeviceObject,
                        (*IrpSp).Parameters.StartDevice.AllocatedResources,
                        (*IrpSp).Parameters.StartDevice.AllocatedResourcesTranslated,
                    );
                }
            }
        }
        IRP_MN_QUERY_REMOVE_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_REMOVE_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_CANCEL_REMOVE_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_STOP_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_QUERY_STOP_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_CANCEL_STOP_DEVICE => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_QUERY_DEVICE_RELATIONS => {
            match (*IrpSp).Parameters.QueryDeviceRelations.Type as u32 {
                _DEVICE_RELATION_TYPE_BusRelations => return ForwardIrpAndForget(DeviceObject, Irp),
                _DEVICE_RELATION_TYPE_RemovalRelations => return ForwardIrpAndForget(DeviceObject, Irp),
                _ => return ForwardIrpAndForget(DeviceObject, Irp),
            }
        }
        IRP_MN_SURPRISE_REMOVAL => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        IRP_MN_FILTER_RESOURCE_REQUIREMENTS => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
        _ => {
            return ForwardIrpAndForget(DeviceObject, Irp);
        }
    }

    (*Irp).IoStatus.Information = Information as ntkrnl_sys::ddk::ULONG_PTR;
    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    IoCompleteRequest(Irp, IO_NO_INCREMENT as i8);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn ProcessorAddDevice(
    DriverObject: PDRIVER_OBJECT,
    Pdo: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut DeviceExtension: *mut DEVICE_EXTENSION = core::ptr::null_mut();
    let mut Fdo: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut Status: NTSTATUS = 0;

    if DriverObject.is_null() {
        RtlAssert(
            b"DriverObject\0".as_ptr() as *const core::ffi::c_void,
            b"drivers/processor/processr/pnp.c\0"
                .as_ptr() as *const i8,
            374u32,
            core::ptr::null(),
        );
    }
    if Pdo.is_null() {
        RtlAssert(
            b"Pdo\0".as_ptr() as *const core::ffi::c_void,
            b"drivers/processor/processr/pnp.c\0"
                .as_ptr() as *const i8,
            375u32,
            core::ptr::null(),
        );
    }

    Status = IoCreateDevice(
        DriverObject,
        core::mem::size_of::<DEVICE_EXTENSION>() as u32,
        core::ptr::null_mut::<ntkrnl_sys::ddk::_UNICODE_STRING>(),
        FILE_DEVICE_UNKNOWN as u32,
        FILE_DEVICE_SECURE_OPEN as u32,
        0i32 as u8,
        core::ptr::addr_of_mut!(Fdo),
    );

    if Status >= 0i32 {
        DeviceExtension = (*Fdo).DeviceExtension as *mut DEVICE_EXTENSION;
        core::ptr::write_bytes(
            DeviceExtension as *mut u8,
            0u8,
            core::mem::size_of::<DEVICE_EXTENSION>(),
        );
        (*DeviceExtension).DeviceObject = Fdo;
        Status = IoAttachDeviceToDeviceStackSafe(
            Fdo,
            Pdo,
            core::ptr::addr_of_mut!((*DeviceExtension).LowerDevice),
        );
        if !(Status >= 0i32) {
            if DbgPrint(
                b"(%s:%d) IoAttachDeviceToDeviceStackSafe() failed with status 0x%08lx\n\0"
                    .as_ptr() as *const i8,
                b"drivers/processor/processr/pnp.c\0"
                    .as_ptr() as *const i8,
                395i32,
                Status,
            ) != 0i32
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                    b"drivers/processor/processr/pnp.c\0"
                        .as_ptr() as *const i8,
                    395i32,
                );
            }
            IoDeleteDevice(Fdo);
            return Status;
        }
        (*Fdo).Flags |= DO_DIRECT_IO as u32;
        (*Fdo).Flags |= DO_POWER_PAGABLE as u32;
        (*Fdo).Flags &= !DO_DEVICE_INITIALIZING as u32;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:        ReactOS Generic CPU Driver
//  * LICENSE:        GNU GPLv2 only as published by the Free Software Foundation
//  * FILE:           drivers/processor/processr/pnp.c
//  * PURPOSE:        Plug N Play routines
//  * PROGRAMMERS:    Eric Kohl <eric.kohl@reactos.org>
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "processr.h"
//
// #include <stdio.h>
// #define NDEBUG
// #include <debug.h>
//
// /* FUNCTIONS ******************************************************************/
//
// static
// NTSTATUS
// GetDeviceId(
//     PDEVICE_OBJECT DeviceObject,
//     BUS_QUERY_ID_TYPE IdType,
//     PWSTR *DeviceId)
// {
//     PIO_STACK_LOCATION IrpStack;
//     IO_STATUS_BLOCK IoStatus;
//     PDEVICE_OBJECT TargetObject;
//     KEVENT Event;
//     PIRP Irp;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Initialize the event */
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     TargetObject = IoGetAttachedDeviceReference(DeviceObject);
//
//     /* Build the IRP */
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        TargetObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatus);
//     if (Irp == NULL)
//     {
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//         goto done;
//     }
//
//     /* PNP IRPs all begin life as STATUS_NOT_SUPPORTED */
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     /* Get the top of stack */
//     IrpStack = IoGetNextIrpStackLocation(Irp);
//
//     /* Set the top of stack */
//     RtlZeroMemory(IrpStack, sizeof(IO_STACK_LOCATION));
//     IrpStack->MajorFunction = IRP_MJ_PNP;
//     IrpStack->MinorFunction = IRP_MN_QUERY_ID;
//     IrpStack->Parameters.QueryId.IdType = IdType;
//
//     /* Call the driver */
//     Status = IoCallDriver(TargetObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event,
//                               Executive,
//                               KernelMode,
//                               FALSE,
//                               NULL);
//
//         Status = IoStatus.Status;
//     }
//
//     if (NT_SUCCESS(Status))
//     {
//         *DeviceId = (PWSTR)IoStatus.Information;
//     }
//
// done:
//     /* Dereference the target device object */
//     ObDereferenceObject(TargetObject);
//
//     return Status;
// }
//
//
//
// static
// VOID
// ProcessorSetFriendlyName(
//     PDEVICE_OBJECT DeviceObject)
// {
//     KEY_VALUE_PARTIAL_INFORMATION *Buffer = NULL;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING HardwareKeyName, ValueName, EnumKeyName;
//     HANDLE KeyHandle = NULL;
//     ULONG DataLength = 0;
//     ULONG BufferLength = 0;
//     NTSTATUS Status;
//     PWSTR KeyNameBuffer = NULL;
//     PWSTR DeviceId = NULL;
//     PWSTR InstanceId = NULL;
//     PWSTR pszPrefix = L"\\Registry\\Machine\\System\\CurrentControlSet\\Enum";
//
//     RtlInitUnicodeString(&HardwareKeyName,
//                          L"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &HardwareKeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle,
//                        KEY_READ,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey() failed (Status 0x%08lx)\n", Status);
//         return;
//     }
//
//     RtlInitUnicodeString(&ValueName,
//                          L"ProcessorNameString");
//     Status = ZwQueryValueKey(KeyHandle,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              NULL,
//                              0,
//                              &DataLength);
//     if (Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL && Status != STATUS_SUCCESS)
//     {
//         DPRINT1("ZwQueryValueKey() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     Buffer = ExAllocatePool(PagedPool,
//                             DataLength + sizeof(KEY_VALUE_PARTIAL_INFORMATION));
//     if (Buffer == NULL)
//     {
//         DPRINT1("ExAllocatePool() failed\n");
//         goto done;
//     }
//
//     Status = ZwQueryValueKey(KeyHandle,
//                              &ValueName,
//                              KeyValuePartialInformation,
//                              Buffer,
//                              DataLength + sizeof(KEY_VALUE_PARTIAL_INFORMATION),
//                              &DataLength);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwQueryValueKey() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     DPRINT("ProcessorNameString: %S\n", (PWSTR)&Buffer->Data[0]);
//
//     ZwClose(KeyHandle);
//     KeyHandle = NULL;
//
//     Status = GetDeviceId(DeviceObject,
//                          BusQueryDeviceID,
//                          &DeviceId);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("GetDeviceId() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     DPRINT("DeviceId: %S\n", DeviceId);
//
//     Status = GetDeviceId(DeviceObject,
//                          BusQueryInstanceID,
//                          &InstanceId);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("GetDeviceId() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     DPRINT("InstanceId: %S\n", InstanceId);
//
//     BufferLength = wcslen(pszPrefix) + 1 + wcslen(DeviceId) + 1 + wcslen(InstanceId) + 1;
//
//     KeyNameBuffer = ExAllocatePool(PagedPool, BufferLength * sizeof(WCHAR));
//     if (KeyNameBuffer == NULL)
//     {
//         DPRINT1("ExAllocatePool() failed\n");
//         goto done;
//     }
//
//     swprintf(KeyNameBuffer, L"%s\\%s\\%s", pszPrefix, DeviceId, InstanceId);
//
//     RtlInitUnicodeString(&EnumKeyName, KeyNameBuffer);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &EnumKeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenKey(&KeyHandle,
//                        KEY_WRITE,
//                        &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwOpenKey() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
//     RtlInitUnicodeString(&ValueName,
//                          L"FriendlyName");
//     Status = ZwSetValueKey(KeyHandle,
//                            &ValueName,
//                            0,
//                            REG_SZ,
//                            (PVOID)&Buffer->Data[0],
//                            Buffer->DataLength);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("ZwSetValueKey() failed (Status 0x%08lx)\n", Status);
//         goto done;
//     }
//
// done:
//     if (KeyHandle != NULL)
//         ZwClose(KeyHandle);
//
//     if (KeyNameBuffer != NULL)
//         ExFreePool(KeyNameBuffer);
//
//     if (InstanceId != NULL)
//         ExFreePool(InstanceId);
//
//     if (DeviceId != NULL)
//         ExFreePool(DeviceId);
//
//     if (Buffer != NULL)
//         ExFreePool(Buffer);
// }
//
//
// static
// NTSTATUS
// ProcessorStartDevice(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PCM_RESOURCE_LIST ResourceList,
//     IN PCM_RESOURCE_LIST ResourceListTranslated)
// {
//     DPRINT("ProcessorStartDevice()\n");
//
//     ProcessorSetFriendlyName(DeviceObject);
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// NTAPI
// ProcessorPnp(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp)
// {
//     PDEVICE_EXTENSION DeviceExtension;
//     PIO_STACK_LOCATION IrpSp;
//     ULONG_PTR Information = 0;
//     NTSTATUS Status = STATUS_NOT_SUPPORTED;
//
//     DPRINT("ProcessorPnp()\n");
//
//     IrpSp = IoGetCurrentIrpStackLocation(Irp);
//
//     switch (IrpSp->MinorFunction)
//     {
//         case IRP_MN_START_DEVICE:
//             DPRINT("  IRP_MN_START_DEVICE received\n");
//
//             /* Call lower driver */
//             DeviceExtension = DeviceObject->DeviceExtension;
//             Status = STATUS_UNSUCCESSFUL;
//
//             if (IoForwardIrpSynchronously(DeviceExtension->LowerDevice, Irp))
//             {
//                 Status = Irp->IoStatus.Status;
//                 if (NT_SUCCESS(Status))
//                 {
//                     Status = ProcessorStartDevice(DeviceObject,
//                         IrpSp->Parameters.StartDevice.AllocatedResources,
//                         IrpSp->Parameters.StartDevice.AllocatedResourcesTranslated);
//                 }
//             }
//             break;
//
//         case IRP_MN_QUERY_REMOVE_DEVICE:
//             DPRINT("  IRP_MN_QUERY_REMOVE_DEVICE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_REMOVE_DEVICE:
//             DPRINT("  IRP_MN_REMOVE_DEVICE received\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_CANCEL_REMOVE_DEVICE:
//             DPRINT("  IRP_MN_CANCEL_REMOVE_DEVICE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_STOP_DEVICE:
//             DPRINT("  IRP_MN_STOP_DEVICE received\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_QUERY_STOP_DEVICE:
//             DPRINT("  IRP_MN_QUERY_STOP_DEVICE received\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_CANCEL_STOP_DEVICE:
//             DPRINT("  IRP_MN_CANCEL_STOP_DEVICE\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_QUERY_DEVICE_RELATIONS:
//             DPRINT("  IRP_MN_QUERY_DEVICE_RELATIONS\n");
//
//             switch (IrpSp->Parameters.QueryDeviceRelations.Type)
//             {
//                 case BusRelations:
//                     DPRINT("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / BusRelations\n");
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//                     break;
//
//                 case RemovalRelations:
//                     DPRINT("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / RemovalRelations\n");
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//
//                 default:
//                     DPRINT("    IRP_MJ_PNP / IRP_MN_QUERY_DEVICE_RELATIONS / Unknown type 0x%lx\n",
//                             IrpSp->Parameters.QueryDeviceRelations.Type);
//                     return ForwardIrpAndForget(DeviceObject, Irp);
//             }
//             break;
//
//         case IRP_MN_SURPRISE_REMOVAL:
//             DPRINT("  IRP_MN_SURPRISE_REMOVAL received\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         case IRP_MN_FILTER_RESOURCE_REQUIREMENTS: /* (optional) 0xd */
//             DPRINT("  IRP_MN_FILTER_RESOURCE_REQUIREMENTS\n");
//             return ForwardIrpAndForget(DeviceObject, Irp);
//
//         default:
//             DPRINT("  Unknown IOCTL 0x%lx\n", IrpSp->MinorFunction);
//             return ForwardIrpAndForget(DeviceObject, Irp);
//     }
//
//     Irp->IoStatus.Information = Information;
//     Irp->IoStatus.Status = Status;
//     IoCompleteRequest(Irp, IO_NO_INCREMENT);
//
//     return Status;
// }
//
//
// NTSTATUS
// NTAPI
// ProcessorAddDevice(
//     IN PDRIVER_OBJECT DriverObject,
//     IN PDEVICE_OBJECT Pdo)
// {
//     PDEVICE_EXTENSION DeviceExtension = NULL;
//     PDEVICE_OBJECT Fdo = NULL;
//     NTSTATUS Status;
//
//     DPRINT("ProcessorAddDevice()\n");
//
//     ASSERT(DriverObject);
//     ASSERT(Pdo);
//
//     /* Create functional device object */
//     Status = IoCreateDevice(DriverObject,
//                             sizeof(DEVICE_EXTENSION),
//                             NULL,
//                             FILE_DEVICE_UNKNOWN,
//                             FILE_DEVICE_SECURE_OPEN,
//                             FALSE,
//                             &Fdo);
//     if (NT_SUCCESS(Status))
//     {
//         DeviceExtension = (PDEVICE_EXTENSION)Fdo->DeviceExtension;
//         RtlZeroMemory(DeviceExtension, sizeof(DEVICE_EXTENSION));
//
//         DeviceExtension->DeviceObject = Fdo;
//
//         Status = IoAttachDeviceToDeviceStackSafe(Fdo, Pdo, &DeviceExtension->LowerDevice);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("IoAttachDeviceToDeviceStackSafe() failed with status 0x%08lx\n", Status);
//             IoDeleteDevice(Fdo);
//             return Status;
//         }
//
//         Fdo->Flags |= DO_DIRECT_IO;
//         Fdo->Flags |= DO_POWER_PAGABLE;
//
//         Fdo->Flags &= ~DO_DEVICE_INITIALIZING;
//     }
//
//     return Status;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
