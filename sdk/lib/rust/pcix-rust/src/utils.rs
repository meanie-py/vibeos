#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use ntkrnl_sys::ke::KeGetCurrentIrql;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use crate::glue::{IRP_MN_QUERY_INTERFACE, IRP_MN_QUERY_CAPABILITIES}; // explicit: disambiguate vs ntkrnl_sys::driver_ddk glob
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, DEVICE_CAPABILITIES, DEVICE_REGISTRY_PROPERTY, _KWAIT_REASON_Executive, IO_STACK_LOCATION, _IO_STATUS_BLOCK, IoBuildDeviceIoControlRequest, IoBuildSynchronousFsdRequest, IoGetAttachedDeviceReference, IoGetDeviceProperty, IoOpenDeviceRegistryKey, KEVENT, KEY_VALUE_PARTIAL_INFORMATION, KeEnterCriticalRegion, KeInitializeEvent, KeLeaveCriticalRegion, KeWaitForSingleObject, _OBJECT_ATTRIBUTES, PCI_CAPABILITIES_HEADER, PCI_COMMON_CONFIG, PCI_COMMON_HEADER, PCUNICODE_STRING, PDEVICE_CAPABILITIES, PDEVICE_OBJECT, PHANDLE, PIO_RESOURCE_DESCRIPTOR, PIO_STACK_LOCATION, PIRP, PKEVENT, PKEY_VALUE_PARTIAL_INFORMATION, PNP_BUS_INFORMATION, PNTSTATUS, PPCI_CAPABILITIES_HEADER, PPCI_COMMON_HEADER, PPNP_BUS_INFORMATION, PUNICODE_STRING, RTL_OSVERSIONINFOEXW, RTL_RANGE_LIST, RtlAddRange, RtlEqualUnicodeString, RtlFreeRangeList, RtlInitUnicodeString, RtlInitializeRangeList, RtlVerifyVersionInfo, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwCreateKey, ZwOpenKey, ZwQueryValueKey, ZwSetValueKey};
use ntkrnl_sys::inline::tier1::{IoGetNextIrpStackLocation, IoSetCompletionRoutine};
use ntkrnl_sys::ddk::{
    DEVICE_REGISTRY_PROPERTY_DevicePropertyUINumber as DevicePropertyUINumber,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
    _INTERFACE_TYPE_PCIBus as PCIBus,
    _DEVICE_POWER_STATE_PowerDeviceD0 as PowerDeviceD0,
    _DEVICE_POWER_STATE_PowerDeviceD1 as PowerDeviceD1,
    _DEVICE_POWER_STATE_PowerDeviceD2 as PowerDeviceD2,
    _DEVICE_POWER_STATE_PowerDeviceD3 as PowerDeviceD3,
    _DEVICE_POWER_STATE_PowerDeviceUnspecified as PowerDeviceUnspecified,
    _SYSTEM_POWER_STATE_PowerSystemHibernate as PowerSystemHibernate,
    _SYSTEM_POWER_STATE_PowerSystemMaximum as PowerSystemMaximum,
    _SYSTEM_POWER_STATE_PowerSystemShutdown as PowerSystemShutdown,
    _SYSTEM_POWER_STATE_PowerSystemSleeping3 as PowerSystemSleeping3,
    _SYSTEM_POWER_STATE_PowerSystemUnspecified as PowerSystemUnspecified,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
};
use core::ffi::c_void;
use ntkrnl_sys::ddk::RtlIoEncodeMemIoResource;
use crate::glue::{PCI_HACK_NEVER_DISCONNECT, PCI_HACK_DONT_DISABLE, PCI_HACK_NEVER_POWER_DOWN, PCI_HACK_KEEP_DECODES_ON_STOP};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{DEVICE_POWER_STATE, IO_RESOURCE_DESCRIPTOR, PCSTR, PRKEVENT, PSTR, RtlAssert, SYSTEM_POWER_STATE, _LIST_ENTRY, _PNP_BUS_INFORMATION, _UNICODE_STRING, wchar_t};

extern "C" {
    fn DbgPrint(format: *const u8, ...) -> i32;
}

// PCI_COMMON_HDR_LENGTH = FIELD_OFFSET(PCI_COMMON_CONFIG, DeviceSpecific) = 0x40
pub const PCI_COMMON_HDR_LENGTH: usize = 0x40;

// CRITICAL_SERVICE_FAILED bugcheck code (mc/bugcodes.mc = 0x5a)
pub const CRITICAL_SERVICE_FAILED: ULONG = 0x5a;

// reactos/drivers/pci/pci.h: SLOT_INFO, *PSLOT_INFO (SLOT_INFO is in crate::types)
pub type PSLOT_INFO = *mut SLOT_INFO;

extern "system" {
    // ntddk.h VerSetConditionMask (not in ntkrnl_sys bindgen)
    fn VerSetConditionMask(ConditionMask: u64, TypeMask: ULONG, Condition: UCHAR) -> u64;
}

// ---- ACPI eval-method types (from sdk/include/ddk/acpiioct.h; not in ntkrnl_sys) ----
pub const ACPI_EVAL_INPUT_BUFFER_SIGNATURE: ULONG = 0x42696541; // 'BieA' (acpiioct.h:7)
pub const ACPI_METHOD_ARGUMENT_INTEGER: USHORT = 0x0;
// IOCTL_ACPI_EVAL_METHOD = CTL_CODE(FILE_DEVICE_ACPI=0x32, 1, METHOD_BUFFERED=0, FILE_READ|WRITE=3)
pub const IOCTL_ACPI_EVAL_METHOD: ULONG = (0x32 << 16) | (3 << 14) | (1 << 2) | 0;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ACPI_EVAL_INPUT_BUFFER {
    pub Signature: ULONG,
    pub MethodName: [UCHAR; 4],
}
pub type PACPI_EVAL_INPUT_BUFFER = *mut ACPI_EVAL_INPUT_BUFFER;

#[repr(C)]
#[derive(Copy, Clone)]
pub union ACPI_METHOD_ARGUMENT_u {
    pub Argument: ULONG,
    pub Data: [UCHAR; 1],
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ACPI_METHOD_ARGUMENT {
    pub Type: USHORT,
    pub DataLength: USHORT,
    pub c2rust_unnamed: ACPI_METHOD_ARGUMENT_u,
}
pub type PACPI_METHOD_ARGUMENT = *mut ACPI_METHOD_ARGUMENT;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ACPI_EVAL_OUTPUT_BUFFER {
    pub Signature: ULONG,
    pub Length: ULONG,
    pub Count: ULONG,
    pub Argument: [ACPI_METHOD_ARGUMENT; 1],
}
pub type PACPI_EVAL_OUTPUT_BUFFER = *mut ACPI_EVAL_OUTPUT_BUFFER;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static mut PciDebugPortsCount: ULONG = 0;

pub static mut PciIsaBitExclusionList: RTL_RANGE_LIST = RTL_RANGE_LIST {
    ListHead: LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    Flags: 0,
    Count: 0,
    Stamp: 0,
};

pub static mut PciVgaAndIsaBitExclusionList: RTL_RANGE_LIST = RTL_RANGE_LIST {
    ListHead: LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    Flags: 0,
    Count: 0,
    Stamp: 0,
};
#[no_mangle]
pub unsafe extern "system" fn PciUnicodeStringStrStr(
    InputString: PUNICODE_STRING,
    EqualString: PCUNICODE_STRING,
    CaseInSensitive: u8,
) -> u8 {
    let mut PartialString: _UNICODE_STRING = core::mem::zeroed();
    let mut EqualChars: i32;
    let mut TotalChars: i32;

    PartialString.Length = (*EqualString).Length;
    PartialString.MaximumLength = (*InputString).MaximumLength;
    PartialString.Buffer = (*InputString).Buffer;

    EqualChars = 0;
    TotalChars = ((*InputString).Length as i32 - (*EqualString).Length as i32)
        / core::mem::size_of::<u16>() as i32;

    if TotalChars < 0 {
        return 0u8;
    }

    while RtlEqualUnicodeString(
        EqualString as *const _UNICODE_STRING,
        &raw mut PartialString,
        CaseInSensitive,
    ) == 0
    {
        PartialString.Buffer = PartialString.Buffer.add(1);
        PartialString.MaximumLength =
            PartialString.MaximumLength.wrapping_sub(core::mem::size_of::<u16>() as u16);
        EqualChars += 1;
        if EqualChars > TotalChars {
            return 0u8;
        }
    }

    1u8
}
#[no_mangle]
pub unsafe extern "system" fn PciStringToUSHORT(String: PWCHAR, Value: PUSHORT) -> BOOLEAN {
    let mut short_val: u16 = 0;
    let mut length: u32 = 0;
    let mut ptr = String;
    loop {
        let ch = *ptr;
        ptr = ptr.add(1);
        let high = (16u16 * short_val) as u32;
        let low: u32;
        if ch >= u16::from(b'0') && ch <= u16::from(b'9') {
            low = (ch - u16::from(b'0')) as u32;
        } else if ch >= u16::from(b'A') && ch <= u16::from(b'F') {
            low = (ch - u16::from(b'7')) as u32; // '7' = 0x37, 'A' - '7' = 10
        } else if ch >= u16::from(b'a') && ch <= u16::from(b'f') {
            low = (ch - u16::from(b'W')) as u32; // 'W' = 0x57, 'a' - 'W' = 10
        } else {
            return 0u8;
        }
        short_val = (high | low) as u16;
        length += 1;
        if length >= 4 {
            *Value = short_val;
            return 1u8;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciIsSuiteVersion(SuiteMask: u16) -> u8 {
    let mut mask: u64 = 0;
    let mut version_info: RTL_OSVERSIONINFOEXW = core::mem::zeroed();
    version_info.dwOSVersionInfoSize = core::mem::size_of::<RTL_OSVERSIONINFOEXW>() as u32;
    version_info.wSuiteMask = SuiteMask;
    mask = VerSetConditionMask(mask, 0x40u32, 6u8);
    (RtlVerifyVersionInfo(&mut version_info, 0x40u32, mask) >= 0) as u8
}
#[no_mangle]
pub unsafe extern "system" fn PciIsDatacenter() -> BOOLEAN {
    let mut Result: BOOLEAN = 0;
    let mut Value: PVOID = core::ptr::null_mut();
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    Result = FALSE;

    let empty_name: [u16; 1] = core::mem::transmute::<[u8; 2], [u16; 1]>(*b"\0\0");
    let setupdd_path: [u16; 60] = core::mem::transmute::<[u8; 120], [u16; 60]>(
        *b"\\\0R\0E\0G\0I\0S\0T\0R\0Y\0\\\0M\0A\0C\0H\0I\0N\0E\0\\\0S\0Y\0S\0T\0E\0M\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0S\0e\0r\0v\0i\0c\0e\0s\0\\\0s\0e\0t\0u\0p\0d\0d\0\0\0",
    );
    Status = PciGetRegistryValue(
        empty_name.as_ptr() as PWCHAR,
        setupdd_path.as_ptr() as PWCHAR,
        core::ptr::null_mut(),
        REG_BINARY as ULONG,
        &raw mut Value,
        &raw mut ResultLength,
    );

    if !(Status >= 0) {
        Result = PciIsSuiteVersion(VER_SUITE_DATACENTER as u16);
    } else {
        DbgPrint(
            (b"UNRECOVERABLE ERROR:  %s at %s:%d is unimplemented!()\n\0" as *const u8 as PCSTR).cast(),
            b"PciIsDatacenter\0" as *const u8 as *const i8,
            b"drivers/bus/pcix/utils.c\0" as *const u8
                as *const i8,
            156 as i32,
        );
        DbgPrint(
            (b"ReactOS doesn't use SetupDD for its installation program. Therefore this scenario must not happen!\n\0"
                as *const u8 as PCSTR).cast(),
        );
        DbgBreakPoint();
        KeBugCheckEx(
            CRITICAL_SERVICE_FAILED,
            0xc0000002u32 as NTSTATUS as ULONG_PTR,
            0,
            0,
            0,
        );
    }

    Result
}
#[no_mangle]
pub unsafe extern "system" fn PciOpenKey(
    KeyName: *mut u16,
    RootKey: *mut core::ffi::c_void,
    DesiredAccess: u32,
    KeyHandle: *mut *mut core::ffi::c_void,
    KeyStatus: *mut i32,
) -> u8 {
    let mut Status: i32;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = unsafe { core::mem::zeroed() };
    let mut KeyString: _UNICODE_STRING = unsafe { core::mem::zeroed() };
    if (KeGetCurrentIrql() as u32) > 1u32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8,
            KeGetCurrentIrql() as u32,
        );
        if true {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as *mut core::ffi::c_void,
                b"drivers/bus/pcix/utils.c\0".as_ptr()
                    as *const i8 as *mut core::ffi::c_void,
                174u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }
    RtlInitUnicodeString(&raw mut KeyString, KeyName as *const u16);
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = RootKey;
    ObjectAttributes.ObjectName = &raw mut KeyString as *mut _UNICODE_STRING;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>();
    Status = ZwOpenKey(KeyHandle, DesiredAccess, &raw mut ObjectAttributes);
    if !KeyStatus.is_null() {
        *KeyStatus = Status;
    }
    (Status >= 0) as u8
}
#[no_mangle]
pub unsafe extern "system" fn PciGetRegistryValue(
    ValueName: *mut u16,
    KeyName: *mut u16,
    RootHandle: HANDLE,
    Type: ULONG,
    OutputBuffer: *mut PVOID,
    OutputLength: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut NeededLength: ULONG = 0;
    let mut ActualLength: ULONG = 0;
    let mut ValueString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let Result: BOOLEAN;

    PartialInfo = core::ptr::null_mut();
    KeyHandle = core::ptr::null_mut();

    Result = PciOpenKey(
        KeyName,
        RootHandle,
        KEY_QUERY_VALUE,
        &mut KeyHandle,
        &mut Status,
    );

    loop {
        if Result == 0 {
            break;
        }

        {
            let mut len: u16 = 0;
            if !ValueName.is_null() {
                while *ValueName.offset(len as isize) != 0 {
                    len += 1;
                }
            }
            ValueString.Buffer = ValueName;
            ValueString.Length = len * 2;
            ValueString.MaximumLength = len * 2;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueString,
            KeyValuePartialInformation,
            core::ptr::null_mut(),
            0,
            &mut NeededLength,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        PartialInfo = ExAllocatePoolWithTag(PagedPool, NeededLength as usize, PCI_POOL_TAG)
            as PKEY_VALUE_PARTIAL_INFORMATION;
        if PartialInfo.is_null() {
            break;
        }

        Status = ZwQueryValueKey(
            KeyHandle,
            &mut ValueString,
            KeyValuePartialInformation,
            PartialInfo as PVOID,
            NeededLength,
            &mut ActualLength,
        );
        if Status < 0 {
            break;
        }

        Status = STATUS_INVALID_PARAMETER;
        if (*PartialInfo).Type != Type {
            break;
        }

        /* The data size is what the key says it is: subtracting
         * sizeof(KEY_VALUE_PARTIAL_INFORMATION) from the query length undercounts by the
         * Data[1] byte plus tail padding, dropping the last bytes of every value. */
        NeededLength = (*PartialInfo).DataLength;

        Status = STATUS_INSUFFICIENT_RESOURCES;
        *OutputBuffer = ExAllocatePoolWithTag(PagedPool, NeededLength as usize, PCI_POOL_TAG);
        if (*OutputBuffer).is_null() {
            break;
        }

        core::ptr::copy_nonoverlapping(
            (*PartialInfo).Data.as_ptr(),
            (*OutputBuffer).cast(),
            NeededLength as usize,
        );

        if !OutputLength.is_null() {
            *OutputLength = NeededLength;
        }
        Status = STATUS_SUCCESS;
        break;
    }

    if !KeyHandle.is_null() {
        ZwClose(KeyHandle);
    }
    if !PartialInfo.is_null() {
        ExFreePoolWithTag(PartialInfo as PVOID, 0);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciBuildDefaultExclusionLists() -> NTSTATUS {
    if PciIsaBitExclusionList.Count != 0 {
        RtlAssert(
            b"PciIsaBitExclusionList.Count == 0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"utils.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            280u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if PciVgaAndIsaBitExclusionList.Count != 0 {
        RtlAssert(
            b"PciVgaAndIsaBitExclusionList.Count == 0\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            b"utils.c\0" as *const u8 as *const i8 as *mut core::ffi::c_void,
            281u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    RtlInitializeRangeList(&raw mut PciIsaBitExclusionList);
    RtlInitializeRangeList(&raw mut PciVgaAndIsaBitExclusionList);
    let mut Start: u32 = 0x100;
    let mut Status: NTSTATUS = 0;
    while Start <= 0xFEFF {
        Status = RtlAddRange(
            &raw mut PciIsaBitExclusionList,
            Start as u64,
            (Start.wrapping_add(0x2FF)) as u64,
            0u8,
            RTL_RANGE_LIST_ADD_IF_CONFLICT as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
        if !(Status >= 0) {
            break;
        }
        Status = RtlAddRange(
            &raw mut PciVgaAndIsaBitExclusionList,
            Start as u64,
            (Start.wrapping_add(0x2AF)) as u64,
            0u8,
            RTL_RANGE_LIST_ADD_IF_CONFLICT as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
        if !(Status >= 0) {
            break;
        }
        Status = RtlAddRange(
            &raw mut PciVgaAndIsaBitExclusionList,
            (Start.wrapping_add(0x2BC)) as u64,
            (Start.wrapping_add(0x2BF)) as u64,
            0u8,
            RTL_RANGE_LIST_ADD_IF_CONFLICT as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
        if !(Status >= 0) {
            break;
        }
        Status = RtlAddRange(
            &raw mut PciVgaAndIsaBitExclusionList,
            (Start.wrapping_add(0x2E0)) as u64,
            (Start.wrapping_add(0x2FF)) as u64,
            0u8,
            RTL_RANGE_LIST_ADD_IF_CONFLICT as u32,
            core::ptr::null_mut::<core::ffi::c_void>(),
            core::ptr::null_mut::<core::ffi::c_void>(),
        );
        if !(Status >= 0) {
            break;
        }
        Start = Start.wrapping_add(0x400);
    }
    RtlFreeRangeList(&raw mut PciIsaBitExclusionList);
    RtlFreeRangeList(&raw mut PciVgaAndIsaBitExclusionList);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciFindParentPciFdoExtension(
    DeviceObject: PDEVICE_OBJECT,
    Lock: PKEVENT,
) -> PPCI_FDO_EXTENSION {
    let SearchExtension: PPCI_PDO_EXTENSION = (*DeviceObject).DeviceExtension as PPCI_PDO_EXTENSION;
    let mut FoundExtension: PPCI_PDO_EXTENSION = core::ptr::null_mut();
    let mut DeviceExtension: PPCI_FDO_EXTENSION;

    if !Lock.is_null() {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            Lock as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }

    DeviceExtension = PciFdoExtensionListHead.Next as PPCI_FDO_EXTENSION;
    while !DeviceExtension.is_null() {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );

        FoundExtension = (*DeviceExtension).ChildPdoList as PPCI_PDO_EXTENSION;
        while !FoundExtension.is_null() && FoundExtension != SearchExtension {
            FoundExtension = (*FoundExtension).Next as PPCI_PDO_EXTENSION;
        }

        ntkrnl_sys::ddk::KeSetEvent(
            core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as PRKEVENT,
            IO_NO_INCREMENT as i32,
            0u8,
        );
        KeLeaveCriticalRegion();

        if !FoundExtension.is_null() {
            break;
        }

        DeviceExtension = (*DeviceExtension).List.Next as PPCI_FDO_EXTENSION;
    }

    if !Lock.is_null() {
        ntkrnl_sys::ddk::KeSetEvent(Lock as PRKEVENT, IO_NO_INCREMENT as i32, 0u8);
        KeLeaveCriticalRegion();
    }

    DeviceExtension
}

#[no_mangle]
pub unsafe extern "system" fn PciInsertEntryAtTail(
    ListHead: PSINGLE_LIST_ENTRY,
    DeviceExtension: PPCI_FDO_EXTENSION,
    Lock: PKEVENT,
) {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as *const i8 as PVOID,
                405 as ULONG,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    if !Lock.is_null() {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            Lock as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }

    let mut NextEntry = ListHead;
    while !(*NextEntry).Next.is_null() {
        NextEntry = (*NextEntry).Next as PSINGLE_LIST_ENTRY;
    }
    (*NextEntry).Next = core::ptr::addr_of_mut!((*DeviceExtension).List) as *mut SINGLE_LIST_ENTRY;

    if !Lock.is_null() {
        ntkrnl_sys::ddk::KeSetEvent(Lock as PRKEVENT, IO_NO_INCREMENT as i32, 0u8);
        KeLeaveCriticalRegion();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PciInsertEntryAtHead(
    ListHead: PSINGLE_LIST_ENTRY,
    Entry: PSINGLE_LIST_ENTRY,
    Lock: PKEVENT,
) {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const u8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr()
                    as PVOID,
                434 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }
    if !Lock.is_null() {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            Lock as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }
    (*Entry).Next = (*ListHead).Next;
    (*ListHead).Next = Entry;
    if !Lock.is_null() {
        ntkrnl_sys::ddk::KeSetEvent(Lock as PRKEVENT, IO_NO_INCREMENT as i32, 0u8);
        KeLeaveCriticalRegion();
    }
}

#[no_mangle]
pub unsafe extern "system" fn PcipLinkSecondaryExtension(
    List: PSINGLE_LIST_ENTRY,
    Lock: PVOID,
    SecondaryExtension: PPCI_SECONDARY_EXTENSION,
    ExtensionType: PCI_SIGNATURE,
    Destructor: PVOID,
) {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr()
                    as PVOID,
                465u32,
                core::ptr::null_mut::<i8>(),
            );
        } else {
        }
    }
    (*SecondaryExtension).ExtensionType = ExtensionType;
    (*SecondaryExtension).Destructor = Destructor;
    PciInsertEntryAtHead(
        List,
        (core::ptr::addr_of_mut!((*SecondaryExtension).List)).cast(),
        Lock as PKEVENT,
    );
}

#[no_mangle]
pub unsafe extern "system" fn PciGetDeviceProperty(
    DeviceObject: PDEVICE_OBJECT,
    DeviceProperty: DEVICE_REGISTRY_PROPERTY,
    OutputBuffer: *mut PVOID,
) -> NTSTATUS {
    let mut BufferLength: ULONG = 0;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS;

    Status = IoGetDeviceProperty(
        DeviceObject,
        DeviceProperty,
        0u32,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(BufferLength),
    );
    if Status != STATUS_BUFFER_TOO_SMALL {
        if DbgPrint(
            (b"(%s:%d) PCI - Unexpected status from GetDeviceProperty, saw %08X, expected %08X.\n\0"
                as *const u8 as PCSTR).cast(),
            b"drivers/bus/pcix/utils.c\0" as *const u8
                as *const i8,
            495i32,
            Status,
            STATUS_BUFFER_TOO_SMALL,
        ) != 0
        {
            DbgPrint(
                (b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/bus/pcix/utils.c\0" as *const u8
                    as *const i8,
                495i32,
            );
        }
        *OutputBuffer = core::ptr::null_mut();
        if 0i32 == 0 {
            RtlAssert(
                b"FALSE\0" as *const u8 as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0" as *const u8
                    as *const i8 as PVOID,
                497u32,
                b"PCI Successfully did the impossible!\n\0" as *const u8 as *const i8 as PSTR,
            );
        }
        return STATUS_UNSUCCESSFUL;
    }

    let Buffer: PVOID = ExAllocatePoolWithTag(PagedPool, BufferLength as SIZE_T, 1114202960u32);
    if Buffer.is_null() {
        if DbgPrint(
            (b"(%s:%d) PCI - Failed to allocate DeviceProperty buffer (%u bytes).\n\0" as *const u8
                as PCSTR).cast(),
            b"drivers/bus/pcix/utils.c\0" as *const u8
                as *const i8,
            506i32,
            BufferLength,
        ) != 0
        {
            DbgPrint(
                (b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR).cast(),
                b"drivers/bus/pcix/utils.c\0" as *const u8
                    as *const i8,
                506i32,
            );
        }
        Status = STATUS_INSUFFICIENT_RESOURCES;
    } else {
        Status = IoGetDeviceProperty(
            DeviceObject,
            DeviceProperty,
            BufferLength,
            Buffer,
            core::ptr::addr_of_mut!(ResultLength),
        );
        if Status >= 0i32 {
            if BufferLength != ResultLength {
                RtlAssert(
                    b"BufferLength == ResultLength\0" as *const u8 as *const i8 as PVOID,
                    b"drivers/bus/pcix/utils.c\0"
                        as *const u8 as *const i8 as PVOID,
                    520u32,
                    core::ptr::null_mut::<i8>(),
                );
            }
            *OutputBuffer = Buffer;
            return STATUS_SUCCESS;
        }
    }

    return STATUS_UNSUCCESSFUL;
}
#[no_mangle]
pub unsafe extern "system" fn PciSendIoctl(
    DeviceObject: PDEVICE_OBJECT,
    IoControlCode: u32,
    InputBuffer: PVOID,
    InputBufferLength: u32,
    OutputBuffer: PVOID,
    OutputBufferLength: u32,
) -> NTSTATUS {
    let mut Irp: PIRP = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut Event: KEVENT = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let AttachedDevice: PDEVICE_OBJECT;

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0i32 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as *const i8 as PVOID,
                543u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    KeInitializeEvent(&raw mut Event, _EVENT_TYPE_SynchronizationEvent, 0u8);
    AttachedDevice = IoGetAttachedDeviceReference(DeviceObject);
    if AttachedDevice.is_null() {
        return STATUS_INVALID_PARAMETER;
    }
    Irp = IoBuildDeviceIoControlRequest(
        IoControlCode,
        AttachedDevice,
        InputBuffer,
        InputBufferLength,
        OutputBuffer,
        OutputBufferLength,
        0u8,
        &raw mut Event,
        &raw mut IoStatusBlock,
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    Status = IoCallDriver(AttachedDevice, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            &raw mut Event as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }
    ObDereferenceObject(AttachedDevice as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciFindNextSecondaryExtension(
    ListHead: PSINGLE_LIST_ENTRY,
    ExtensionType: PCI_SIGNATURE,
) -> PPCI_SECONDARY_EXTENSION {
    let mut NextEntry: PSINGLE_LIST_ENTRY = ListHead;
    let offset = ::core::mem::offset_of!(PCI_SECONDARY_EXTENSION, List);

    while !NextEntry.is_null() {
        let Extension = (NextEntry as usize).wrapping_sub(offset) as PPCI_SECONDARY_EXTENSION;
        if (*Extension).ExtensionType == ExtensionType {
            return Extension;
        }
        NextEntry = (*NextEntry).Next;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn PciGetHackFlags(
    VendorId: USHORT,
    DeviceId: USHORT,
    SubVendorId: USHORT,
    SubSystemId: USHORT,
    RevisionId: UCHAR,
) -> ULONGLONG {
    if PciHackTable.is_null() {
        return 0;
    }

    let mut last_weight: ULONG = 0;
    let mut hack_flags: ULONGLONG = 0;

    // ASSERT(PciHackTable)
    if PciHackTable.is_null() {
        RtlAssert(
            b"PciHackTable\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
            621u32,
            core::ptr::null_mut::<CHAR>(),
        );
    }

    let mut hack_entry: PPCI_HACK_ENTRY = PciHackTable;
    loop {
        if (*hack_entry).VendorID == PCI_INVALID_VENDORID as USHORT {
            break;
        }

        if (*hack_entry).DeviceID == DeviceId && (*hack_entry).VendorID == VendorId {
            let entry_flags = (*hack_entry).Flags as ULONG;
            let mut match_weight: ULONG = 1;

            if (entry_flags & PCI_HACK_HAS_REVISION_INFO as ULONG) != 0 {
                if (*hack_entry).RevisionID != RevisionId as USHORT {
                    hack_entry = hack_entry.offset(1);
                    continue;
                }
                match_weight = 3;
            }

            if (entry_flags & PCI_HACK_HAS_SUBSYSTEM_INFO as ULONG) != 0 {
                if (*hack_entry).SubVendorID != SubVendorId || (*hack_entry).SubSystemID != SubSystemId {
                    hack_entry = hack_entry.offset(1);
                    continue;
                }
                match_weight = match_weight.wrapping_add(4);
            }

            if match_weight > last_weight {
                hack_flags = (*hack_entry).HackFlags;
                last_weight = match_weight;
            }
        }

        hack_entry = hack_entry.offset(1);
    }

    hack_flags
}
#[no_mangle]
pub unsafe extern "system" fn PciIsCriticalDeviceClass(
    BaseClass: u8, // UCHAR
    SubClass: u8,  // UCHAR
) -> u8 /* BOOLEAN */ {
    if BaseClass == PCI_CLASS_BASE_SYSTEM_DEV as u8 {
        (SubClass == PCI_SUBCLASS_SYS_INTERRUPT_CTLR as u8) as u8
    } else if BaseClass == PCI_CLASS_BRIDGE_DEV as u8 {
        (SubClass == PCI_SUBCLASS_BR_ISA as u8) as u8
    } else {
        (BaseClass == PCI_CLASS_DISPLAY_CTLR as u8) as u8
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciFindPdoByFunction(
    DeviceExtension: PPCI_FDO_EXTENSION,
    FunctionNumber: ULONG,
    PciData: PPCI_COMMON_HEADER,
) -> PPCI_PDO_EXTENSION {
    let Irql: KIRQL = KeGetCurrentIrql();
    if Irql < DISPATCH_LEVEL {
        KeEnterCriticalRegion();
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
    }
    let mut PdoExtension: PPCI_PDO_EXTENSION = (*DeviceExtension).ChildPdoList;
    while !PdoExtension.is_null() {
        if (*PdoExtension).ReportedMissing == 0 {
            if FunctionNumber == (*PdoExtension).Slot.u.AsULONG
                && (*PdoExtension).VendorId == (*PciData).VendorID
                && (*PdoExtension).DeviceId == (*PciData).DeviceID
                && (*PdoExtension).RevisionId == (*PciData).RevisionID
            {
                break;
            }
        }
        PdoExtension = ((*PdoExtension).Next).cast();
    }
    if Irql < DISPATCH_LEVEL {
        ntkrnl_sys::ddk::KeSetEvent(
            core::ptr::addr_of_mut!((*DeviceExtension).ChildListLock) as PRKEVENT,
            IO_NO_INCREMENT as i32,
            0u8,
        );
        KeLeaveCriticalRegion();
    }
    PdoExtension
}
#[no_mangle]
pub unsafe extern "system" fn PciIsDeviceOnDebugPath(
    mut DeviceExtension: PPCI_PDO_EXTENSION,
) -> BOOLEAN {
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
                753u32,
                core::ptr::null_mut(),
            );
        }
    }
    if !(PciDebugPortsCount <= 4u32) {
        RtlAssert(
            b"PciDebugPortsCount <= MAX_DEBUGGING_DEVICES_SUPPORTED\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
            758u32,
            core::ptr::null_mut(),
        );
    }
    if PciDebugPortsCount == 0 {
        return 0u8;
    }
    DbgPrint(
        (b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR).cast(),
        b"PciIsDeviceOnDebugPath\0".as_ptr() as *const i8,
        b"drivers/bus/pcix/utils.c\0".as_ptr() as *const i8,
        762i32,
    );
    DbgPrint((b"\0".as_ptr() as PCSTR).cast());
    DbgBreakPoint();
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn PciGetBiosConfig(
    DeviceExtension: PPCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut KeyValue: _UNICODE_STRING = _UNICODE_STRING { Length: 0, MaximumLength: 0, Buffer: core::ptr::null_mut() };
    let mut Buffer: [WCHAR; 32] = [0u16; 32];

    // Buffer size: sizeof(KEY_VALUE_PARTIAL_INFORMATION) + PCI_COMMON_HDR_LENGTH
    let mut DataBuffer: [u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + PCI_COMMON_HDR_LENGTH] =
        [0u8; core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() + PCI_COMMON_HDR_LENGTH];
    let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = DataBuffer.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut Status: NTSTATUS;
    let mut ResultLength: ULONG = 0;

    // PAGED_CODE: check IRQL
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 as i32 == 0 {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0" as *const u8
                    as *const i8 as PVOID,
                779 as ULONG,
                core::ptr::null_mut(),
            );
        }
    }

    Status = IoOpenDeviceRegistryKey(
        (*(*DeviceExtension).ParentFdoExtension).PhysicalDeviceObject,
        1u32, // TRUE as ULONG
        KEY_ALL_ACCESS as ACCESS_MASK,
        core::ptr::addr_of_mut!(KeyHandle),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // RtlInitUnicodeString with L"BiosConfig"
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        core::mem::transmute::<[u8; 22], [u16; 11]>(*b"B\0i\0o\0s\0C\0o\0n\0f\0i\0g\0\0\0").as_ptr() as PCWSTR,
    );

    // InitializeObjectAttributes expanded
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName) as PUNICODE_STRING;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as ULONG;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = ZwCreateKey(
        core::ptr::addr_of_mut!(SubKeyHandle),
        KEY_READ as ACCESS_MASK,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0u32,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        REG_OPTION_VOLATILE as ULONG,
        core::ptr::null_mut(),
    );
    ZwClose(KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // swprintf format string. Bound to a LOCAL, not `.as_ptr()`'d off the transmute: that
    // temporary died at the end of its `let`, so `swprintf` read the format from a dead
    // stack slot and the value name it built -- the one ZwQueryValueKey looks up below --
    // was whatever had overwritten it.
    let format: [u16; 18] = core::mem::transmute::<[u8; 36], [u16; 18]>(
        *b"D\0E\0V\0_\0%\x000\x002\0x\0&\0F\0U\0N\0_\0%\x000\x002\0x\0\0\0",
    );
    swprintf(
        Buffer.as_mut_ptr() as *mut wchar_t,
        format.as_ptr() as *const wchar_t,
        (*DeviceExtension).Slot.u.bits.DeviceNumber() as i32,
        (*DeviceExtension).Slot.u.bits.FunctionNumber() as i32,
    );

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyValue),
        Buffer.as_ptr() as PCWSTR,
    );

    Status = ZwQueryValueKey(
        SubKeyHandle,
        core::ptr::addr_of_mut!(KeyValue),
        KeyValuePartialInformation,
        PartialInfo as PVOID,
        DataBuffer.len() as ULONG,
        core::ptr::addr_of_mut!(ResultLength),
    );
    ZwClose(SubKeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // ASSERT(PartialInfo->DataLength == PCI_COMMON_HDR_LENGTH)
    if (*PartialInfo).DataLength != PCI_COMMON_HDR_LENGTH as ULONG {
        RtlAssert(
            b"PartialInfo->DataLength == PCI_COMMON_HDR_LENGTH\0" as *const u8 as *const i8 as PVOID,
            b"drivers/bus/pcix/utils.c\0" as *const u8
                as *const i8 as PVOID,
            824 as ULONG,
            core::ptr::null_mut(),
        );
    }

    core::ptr::copy_nonoverlapping(
        (*PartialInfo).Data.as_ptr(),
        PciData as *mut u8,
        PCI_COMMON_HDR_LENGTH,
    );

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciSaveBiosConfig(
    DeviceExtension: *mut _PCI_PDO_EXTENSION,
    PciData: PPCI_COMMON_HEADER,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKeyHandle: HANDLE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut KeyValue: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Buffer: [WCHAR; 32] = [0; 32];

    // PAGED_CODE expansion
    if KeGetCurrentIrql() > APC_LEVEL as KIRQL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if true {
            RtlAssert(
                b"0\0" as *const u8 as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0" as *const u8
                    as *const i8 as PVOID,
                839u32,
                core::ptr::null_mut::<i8>(),
            );
        }
    }

    let mut Status = IoOpenDeviceRegistryKey(
        (*(*DeviceExtension).ParentFdoExtension).PhysicalDeviceObject,
        1u32 as ULONG, // TRUE -> a device key type, represented as ULONG
        (KEY_READ | KEY_WRITE) as ACCESS_MASK,
        core::ptr::addr_of_mut!(KeyHandle),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // L"BiosConfig" as wide string
    let mut key_name_buf: [u16; 11] = [
        0x0042u16, // B
        0x0069u16, // i
        0x006Fu16, // o
        0x0073u16, // s
        0x0043u16, // C
        0x006Fu16, // o
        0x006Eu16, // n
        0x0066u16, // f
        0x0069u16, // i
        0x0067u16, // g
        0x0000u16, // \0
    ];
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(KeyName),
        &raw const key_name_buf as PCWSTR,
    );

    // Expand InitializeObjectAttributes macro
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = KeyHandle;
    ObjectAttributes.Attributes = OBJ_KERNEL_HANDLE as ULONG;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(KeyName) as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<core::ffi::c_void>() as PVOID;

    Status = ZwCreateKey(
        core::ptr::addr_of_mut!(SubKeyHandle),
        (KEY_READ | KEY_WRITE) as ACCESS_MASK,
        core::ptr::addr_of_mut!(ObjectAttributes),
        0u32,
        core::ptr::null_mut::<_UNICODE_STRING>(),
        REG_OPTION_VOLATILE as ULONG,
        core::ptr::null_mut::<u32>(),
    );
    ZwClose(KeyHandle);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    // Build "DEV_%02x&FUN_%02x" wide format string
    let mut fmt_buf: [u16; 18] = [
        0x0044u16, // D
        0x0045u16, // E
        0x0056u16, // V
        0x005Fu16, // _
        0x0025u16, // %
        0x0030u16, // 0
        0x0032u16, // 2
        0x0078u16, // x
        0x0026u16, // &
        0x0046u16, // F
        0x0055u16, // U
        0x004Eu16, // N
        0x005Fu16, // _
        0x0025u16, // %
        0x0030u16, // 0
        0x0032u16, // 2
        0x0078u16, // x
        0x0000u16, // \0
    ];
    swprintf(
        Buffer.as_mut_ptr() as *mut wchar_t,
        fmt_buf.as_ptr() as *const wchar_t,
        (*DeviceExtension).Slot.u.bits.DeviceNumber() as i32,
        (*DeviceExtension).Slot.u.bits.FunctionNumber() as i32,
    );

    RtlInitUnicodeString(core::ptr::addr_of_mut!(KeyValue), Buffer.as_mut_ptr() as PCWSTR);

    Status = ZwSetValueKey(
        SubKeyHandle,
        core::ptr::addr_of_mut!(KeyValue),
        0u32,
        REG_BINARY as ULONG,
        PciData as PVOID,
        PCI_COMMON_HDR_LENGTH as ULONG,
    );
    ZwClose(SubKeyHandle);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciReadDeviceCapability(
    DeviceExtension: PPCI_PDO_EXTENSION,
    mut Offset: u8,
    CapabilityId: u32,
    Buffer: PPCI_CAPABILITIES_HEADER,
    Length: u32,
) -> u8 {
    let mut CapabilityCount: u32 = 0u32;
    if Offset == 0 {
        return 0u8;
    }
    if !((*DeviceExtension).ExtensionType == PciPdoExtensionType as u32) {
        RtlAssert(
            b"DeviceExtension->ExtensionType == PciPdoExtensionType\0"
                .as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/utils.c\0"
                .as_ptr() as *const i8 as PVOID,
            898u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if !((*DeviceExtension).CapabilitiesPtr as i32 != 0i32) {
        RtlAssert(
            b"DeviceExtension->CapabilitiesPtr != 0\0"
                .as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/utils.c\0"
                .as_ptr() as *const i8 as PVOID,
            899u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if Buffer.is_null() {
        RtlAssert(
            b"Buffer\0".as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/utils.c\0"
                .as_ptr() as *const i8 as PVOID,
            900u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    if (Length as usize) < core::mem::size_of::<PCI_CAPABILITIES_HEADER>() {
        RtlAssert(
            b"Length >= sizeof(PCI_CAPABILITIES_HEADER)\0"
                .as_ptr() as *const i8 as PVOID,
            b"drivers/bus/pcix/utils.c\0"
                .as_ptr() as *const i8 as PVOID,
            901u32,
            core::ptr::null_mut::<i8>(),
        );
    }
    while Offset != 0 {
        if !(Offset >= 0x40u8 && (Offset & 0x3) == 0) {
            RtlAssert(
                b"(Offset >= PCI_COMMON_HDR_LENGTH) && ((Offset & 0x3) == 0)\0"
                    .as_ptr() as *const i8 as PVOID,
                b"drivers/bus/pcix/utils.c\0"
                    .as_ptr() as *const i8 as PVOID,
                907u32,
                core::ptr::null_mut::<i8>(),
            );
        }
        PciReadDeviceConfig(
            DeviceExtension,
            Buffer as PVOID,
            Offset as u32,
            core::mem::size_of::<PCI_CAPABILITIES_HEADER>() as u32,
        );
        if (*Buffer).CapabilityID as u32 == CapabilityId || CapabilityId == 0 {
            if Offset != 0
                && (Length as usize) > core::mem::size_of::<PCI_CAPABILITIES_HEADER>()
            {
                if !(Length as usize
                    <= (core::mem::size_of::<PCI_COMMON_CONFIG>())
                        .wrapping_sub(Offset as usize))
                {
                    RtlAssert(
                        b"Length <= (sizeof(PCI_COMMON_CONFIG) - Offset)\0"
                            .as_ptr() as *const i8 as PVOID,
                        b"drivers/bus/pcix/utils.c\0"
                            .as_ptr() as *const i8 as PVOID,
                        922u32,
                        core::ptr::null_mut::<i8>(),
                    );
                }
                PciReadDeviceConfig(
                    DeviceExtension,
                    (Buffer as usize)
                        .wrapping_add(core::mem::size_of::<PCI_CAPABILITIES_HEADER>())
                        as PVOID,
                    (Offset as usize + core::mem::size_of::<PCI_CAPABILITIES_HEADER>()) as u32,
                    (Length as usize - core::mem::size_of::<PCI_CAPABILITIES_HEADER>()) as u32,
                );
            }
            return Offset;
        }
        CapabilityCount = CapabilityCount.wrapping_add(1);
        Offset = (*Buffer).Next;
        if CapabilityCount > 48u32 {
            if DbgPrint(
                (b"(%s:%d) PCI device %p capabilities list is broken.\n\0"
                    .as_ptr() as PCSTR).cast(),
                b"drivers/bus/pcix/utils.c\0"
                    .as_ptr() as *const i8,
                944i32,
                DeviceExtension,
            ) != 0
            {
                DbgPrint(
                    (b"(%s:%d) DbgPrint() failed!\n\0"
                        .as_ptr() as PCSTR).cast(),
                    b"drivers/bus/pcix/utils.c\0"
                        .as_ptr() as *const i8,
                    944i32,
                );
            }
            return 0u8;
        }
    }
    0u8
}
#[no_mangle]
pub unsafe extern "system" fn PciCanDisableDecodes(
    DeviceExtension: PPCI_PDO_EXTENSION,
    Config: PPCI_COMMON_HEADER,
    HackFlags: ULONGLONG,
    ForPowerDown: BOOLEAN,
) -> BOOLEAN {
    let mut BaseClass: UCHAR;
    let mut SubClass: UCHAR;
    let mut HackFlags = HackFlags;
    let mut KeepPower: BOOLEAN = FALSE as BOOLEAN;

    if !DeviceExtension.is_null() {
        if (*DeviceExtension).OnDebugPath != 0 {
            return FALSE as BOOLEAN;
        }
        if HackFlags != 0 {
            RtlAssert(
                b"HackFlags == 0\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/bus/pcix/utils.c\0".as_ptr()
                    as *mut core::ffi::c_void,
                970 as ULONG,
                (core::ptr::null_mut::<core::ffi::c_void>()).cast(),
            );
        }
        HackFlags = (*DeviceExtension).HackFlags;
        SubClass = (*DeviceExtension).SubClass as UCHAR;
        BaseClass = (*DeviceExtension).BaseClass as UCHAR;
        KeepPower = (*DeviceExtension).DisablePowerDown;
    } else {
        if Config.is_null() {
            RtlAssert(
                b"Config != NULL\0".as_ptr() as *mut core::ffi::c_void,
                b"drivers/bus/pcix/utils.c\0".as_ptr()
                    as *mut core::ffi::c_void,
                980 as ULONG,
                (core::ptr::null_mut::<core::ffi::c_void>()).cast(),
            );
        }
        BaseClass = (*Config).BaseClass;
        SubClass = (*Config).SubClass;
    }

    if HackFlags
        & ((PCI_HACK_PRESERVE_COMMAND
            | PCI_HACK_NEVER_DISCONNECT
            | PCI_HACK_DONT_DISABLE
            | PCI_HACK_CB_SHARE_CMD_BITS) as u64)
        != 0
    {
        return FALSE as BOOLEAN;
    }

    if ForPowerDown != 0 {
        /* IDE controllers, legacy bridges and flagged functions may stop but not power down */
        if KeepPower != 0 || (HackFlags & PCI_HACK_NEVER_POWER_DOWN) != 0 {
            return FALSE as BOOLEAN;
        }
    } else if (HackFlags & PCI_HACK_KEEP_DECODES_ON_STOP) != 0 {
        /* Flagged functions may power down, but keep decoding while stopped or removed */
        return FALSE as BOOLEAN;
    }

    /* VGA adapters keep decoding unless they are being powered down */
    if (BaseClass as u32 == PCI_CLASS_DISPLAY_CTLR && SubClass as u32 == PCI_SUBCLASS_VID_VGA_CTLR)
        || (BaseClass as u32 == PCI_CLASS_PRE_20 && SubClass as u32 == PCI_SUBCLASS_PRE_20_VGA)
    {
        return ForPowerDown;
    }

    if BaseClass as u32 == PCI_CLASS_BRIDGE_DEV {
        if SubClass as u32 == PCI_SUBCLASS_BR_ISA
            || SubClass as u32 == PCI_SUBCLASS_BR_EISA
            || SubClass as u32 == PCI_SUBCLASS_BR_MCA
            || SubClass as u32 == PCI_SUBCLASS_BR_HOST
            || SubClass as u32 == PCI_SUBCLASS_BR_OTHER
        {
            return FALSE as BOOLEAN;
        } else if SubClass as u32 == PCI_SUBCLASS_BR_PCI_TO_PCI
            || SubClass as u32 == PCI_SUBCLASS_BR_CARDBUS
        {
            let IsVga: BOOLEAN;
            if DeviceExtension.is_null() {
                IsVga = (((*Config).u.type1.BridgeControl as u32 & PCI_ENABLE_BRIDGE_VGA) != 0)
                    as BOOLEAN;
            } else {
                IsVga = ((*DeviceExtension).Dependent.type1.as_ref().VgaBitSet() != 0) as BOOLEAN;
            }
            /* A bridge forwarding VGA follows the same rule as the adapter behind it */
            if IsVga != 0 {
                return ForPowerDown;
            }
        }
    }

    /* Nothing else has to keep decoding */
    TRUE as BOOLEAN
}
#[no_mangle]
pub unsafe extern "system" fn PciClassifyDeviceType(
    PdoExtension: PPCI_PDO_EXTENSION,
) -> PCI_DEVICE_TYPES {
    ntkrnl_sys::ASSERT!((*PdoExtension).ExtensionType == _PCI_SIGNATURE_PciPdoExtensionType as ULONG);
    if (*PdoExtension).BaseClass as u32 != PCI_CLASS_BRIDGE_DEV {
        return _PCI_DEVICE_TYPES_PciTypeDevice;
    }
    if (*PdoExtension).SubClass as u32 == PCI_SUBCLASS_BR_HOST {
        return _PCI_DEVICE_TYPES_PciTypeHostBridge;
    }
    if (*PdoExtension).SubClass as u32 == PCI_SUBCLASS_BR_PCI_TO_PCI {
        return _PCI_DEVICE_TYPES_PciTypePciBridge;
    }
    if (*PdoExtension).SubClass as u32 == PCI_SUBCLASS_BR_CARDBUS {
        return _PCI_DEVICE_TYPES_PciTypeCardbusBridge;
    }
    _PCI_DEVICE_TYPES_PciTypeDevice
}
#[no_mangle]
pub unsafe extern "system" fn PciExecuteCriticalSystemRoutine(IpiContext: ULONG_PTR) -> ULONG_PTR {
    let Context: PPCI_IPI_CONTEXT = IpiContext as PPCI_IPI_CONTEXT;
    if InterlockedDecrement(core::ptr::addr_of_mut!((*Context).RunCount)) == 0 {
        ((*Context).Function).unwrap()((*Context).DeviceExtension, (*Context).Context);
        core::ptr::write_volatile(core::ptr::addr_of_mut!((*Context).Barrier), 0);
    } else {
        // The other CPUs spin here until the one running the routine clears Barrier.
        // pci.h leaves it a plain ULONG, so the C's `while (Context->Barrier);` only
        // works if the compiler happens to re-read it; the Rust load was hoisted out
        // of the loop, leaving an unconditional `b .` (scan_ud2) for every other CPU.
        while core::ptr::read_volatile(core::ptr::addr_of!((*Context).Barrier)) != 0 {
            core::hint::spin_loop();
        }
    }
    0 as ULONG_PTR
}
#[no_mangle]
pub unsafe extern "system" fn PciIsSlotPresentInParentMethod(
    PdoExtension: PPCI_PDO_EXTENSION,
    Method: ULONG,
) -> BOOLEAN {
    let mut FoundSlot: BOOLEAN = 0;
    let mut Argument: PACPI_METHOD_ARGUMENT;
    let mut OutputBuffer: PACPI_EVAL_OUTPUT_BUFFER;
    let Length: ULONG;

    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            (b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as PCSTR).cast(),
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            RtlAssert(
                b"0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
                1103 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        }
    }

    FoundSlot = FALSE as BOOLEAN;
    Length = (core::mem::size_of::<ACPI_EVAL_OUTPUT_BUFFER>() + 2048) as ULONG;
    OutputBuffer = ExAllocatePoolWithTag(PagedPool, Length as usize, 0x42696350u32) as PACPI_EVAL_OUTPUT_BUFFER;
    if !OutputBuffer.is_null() {
        core::ptr::write_bytes(OutputBuffer as *mut u8, 0, Length as usize);

        let mut InputBuffer: ACPI_EVAL_INPUT_BUFFER = core::mem::zeroed();
        InputBuffer.Signature = 0;
        *( (&raw mut InputBuffer.MethodName) as *mut ULONG ) = Method;
        InputBuffer.Signature = ACPI_EVAL_INPUT_BUFFER_SIGNATURE;

        let Status = PciSendIoctl(
            (*(*PdoExtension).ParentFdoExtension).PhysicalDeviceObject,
            IOCTL_ACPI_EVAL_METHOD as u32,
            &raw mut InputBuffer as PVOID,
            core::mem::size_of::<ACPI_EVAL_INPUT_BUFFER>() as u32,
            OutputBuffer as PVOID,
            Length,
        );

        if NT_SUCCESS(Status) {
            let mut i: ULONG = 0;
            while i < (*OutputBuffer).Count {
                // NOT `Argument[i]`: the C over-allocates this buffer (Length =
                // sizeof(hdr) + 2048) and the header declares
                // `_Field_size_bytes_(Length) ACPI_METHOD_ARGUMENT Argument[ANYSIZE_ARRAY]`,
                // which bindgen models as `[T; 1]`. Indexing it bounds-checks and
                // PANICS for every i >= 1 -- i.e. any ACPI _PRT returning more than
                // one slot. Walk the flexible array by pointer instead.
                Argument = (&raw mut (*OutputBuffer).Argument)
                    .cast::<ACPI_METHOD_ARGUMENT>()
                    .add(i as usize) as PACPI_METHOD_ARGUMENT;
                if (*Argument).Type != ACPI_METHOD_ARGUMENT_INTEGER {
                    i = i.wrapping_add(1);
                    continue;
                }
                let slot_val = ((*PdoExtension).Slot.u.bits.DeviceNumber() as ULONG)
                    | (((*PdoExtension).Slot.u.bits.FunctionNumber() as ULONG) << 16);
                if (*Argument).c2rust_unnamed.Argument as ULONG == slot_val {
                    FoundSlot = TRUE as BOOLEAN;
                    break;
                }
                i = i.wrapping_add(1);
            }
        }

        ExFreePoolWithTag(OutputBuffer as PVOID, 0u32);
    }

    FoundSlot
}
/// Builds the requirement of a BAR from the value it reads back after all ones were
/// written to it (upstream 4dabb9e5825 rework: named bounds, 64-bit BARs above 4 GB,
/// a BAR implementing no address bits is absent).
///
/// `Bar` is the probed BAR; `NextBar` the probed BAR after it, holding the high half of
/// a 64-bit BAR (0 when there is none); `Rom` is TRUE for an expansion ROM BAR.
/// Returns TRUE when the BAR is 64-bit, so the BAR after it is not a BAR of its own.
/// (`PciGetLengthFromBar` went away with this rework: the length is now the lowest
/// implemented address bit.)
#[no_mangle]
pub unsafe extern "system" fn PciCreateIoDescriptorFromBarLimit(
    ResourceDescriptor: *mut IO_RESOURCE_DESCRIPTOR,
    Bar: u32,
    NextBar: u32,
    Rom: u8,
) -> u8 {
    let mut Bar: u32 = Bar;
    let mut AddressBits: u64;
    let Length: u64;
    let Alignment: u64;
    let MinimumAddress: u64;
    let MaximumAddress: u64;
    let Type: u8;
    let mut Is64BitBar: u8 = 0;

    // Check if the BAR is neither I/O nor memory
    if (Bar & !PCI_ADDRESS_IO_SPACE) == 0 {
        (*ResourceDescriptor).Type = CmResourceTypeNull as UCHAR;
        return 0u8;
    }

    // Set default flag
    (*ResourceDescriptor).Flags = 0u16;

    // Check for ROM Address
    if Rom != 0 {
        Bar &= PCI_ADDRESS_ROM_ADDRESS_MASK;
        if Bar == 0 {
            (*ResourceDescriptor).Type = CmResourceTypeNull as UCHAR;
            return 0u8;
        }
        // ROM Addresses are always read only
        (*ResourceDescriptor).Flags = CM_RESOURCE_MEMORY_READ_ONLY as USHORT;
    }

    // Check what kind of BAR this is
    if (Bar & PCI_ADDRESS_IO_SPACE) != 0 {
        // Set this as an I/O Port descriptor
        Type = CmResourceTypePort as u8;
        (*ResourceDescriptor).Flags = CM_RESOURCE_PORT_IO as USHORT;
        AddressBits = (Bar & PCI_ADDRESS_IO_ADDRESS_MASK) as u64;
    } else {
        // Set this as a memory descriptor
        Type = CmResourceTypeMemory as u8;
        AddressBits = (Bar & PCI_ADDRESS_MEMORY_ADDRESS_MASK) as u64;

        // A 64-bit BAR takes the high half of its address from the next BAR
        if (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_64BIT {
            AddressBits |= (NextBar as u64) << 32;
            Is64BitBar = 1u8;
        }

        // Check if the BAR is listed as prefetchable memory
        if (Bar & PCI_ADDRESS_MEMORY_PREFETCHABLE) != 0 {
            (*ResourceDescriptor).Flags |= CM_RESOURCE_MEMORY_PREFETCHABLE as USHORT;
        }
    }

    // The probe leaves only the bits the BAR implements, so the lowest one is its length
    Length = AddressBits & (!AddressBits).wrapping_add(1);

    // A BAR that implements no address bits is not there
    if Length == 0 {
        (*ResourceDescriptor).Type = CmResourceTypeNull as UCHAR;
        return 0u8;
    }

    // A legacy memory BAR can only be placed below 1MB
    if Type as u32 == CmResourceTypeMemory
        && (Bar & PCI_ADDRESS_MEMORY_TYPE_MASK) == PCI_TYPE_20BIT
    {
        AddressBits &= 0xFFFFF;
    }

    // A BAR aligns to its own length and decodes nothing above the bits it implements
    Alignment = Length;
    MinimumAddress = 0;
    MaximumAddress = AddressBits | (Length - 1);

    // A length of 4GB or more takes the large memory form
    if RtlIoEncodeMemIoResource(
        ResourceDescriptor,
        Type,
        Length,
        Alignment,
        MinimumAddress,
        MaximumAddress,
    ) < 0
    {
        (*ResourceDescriptor).Type = CmResourceTypeNull as UCHAR;
    }

    // Return if this is a 64-bit BAR, so the loop code knows to skip the next one
    Is64BitBar
}

#[no_mangle]
pub unsafe extern "system" fn PciDecodeEnable(
    PdoExtension: PPCI_PDO_EXTENSION,
    Enable: BOOLEAN,
    mut Command: PUSHORT,
) {
    let mut CommandValue: USHORT = 0;
    if (Enable != 0 || PciCanDisableDecodes(PdoExtension, core::ptr::null_mut(), 0, 0) != 0)
        && ((*PdoExtension).HackFlags & PCI_HACK_PRESERVE_COMMAND as u64) == 0
    {
        if !Command.is_null() {
            CommandValue = *Command;
        } else {
            PciReadDeviceConfig(
                PdoExtension,
                &raw mut Command as *mut _ as PVOID,
                4u32 as ULONG,
                core::mem::size_of::<USHORT>() as ULONG,
            );
        }
        CommandValue &= !((PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as USHORT);
        if Enable != 0 {
            CommandValue |= (*PdoExtension).CommandEnables
                & (PCI_ENABLE_IO_SPACE | PCI_ENABLE_MEMORY_SPACE | PCI_ENABLE_BUS_MASTER) as USHORT;
        }
        PciWriteDeviceConfig(
            PdoExtension,
            &raw mut CommandValue as PVOID,
            4u32 as ULONG,
            core::mem::size_of::<USHORT>() as ULONG,
        );
    }
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryBusInformation(
    PdoExtension: PPCI_PDO_EXTENSION,
    Buffer: *mut PPNP_BUS_INFORMATION,
) -> NTSTATUS {
    let mut BusInfo: PPNP_BUS_INFORMATION = core::ptr::null_mut::<_PNP_BUS_INFORMATION>();
    BusInfo = ExAllocatePoolWithTag(
        PagedPool,
        core::mem::size_of::<PNP_BUS_INFORMATION>() as usize,
        1114202960u32 as ULONG,
    ) as PPNP_BUS_INFORMATION;
    if BusInfo.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*BusInfo).BusTypeGuid = GUID_BUS_TYPE_PCI;
    (*BusInfo).LegacyBusType = PCIBus;
    (*BusInfo).BusNumber = ((*(*PdoExtension).ParentFdoExtension).BaseBus) as ULONG;
    *Buffer = BusInfo;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciDetermineSlotNumber(
    PdoExtension: PPCI_PDO_EXTENSION,
    SlotNumber: PULONG,
) -> NTSTATUS {

    let ParentExtension: PPCI_FDO_EXTENSION = (*PdoExtension).ParentFdoExtension;
    let mut ResultLength: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    if DbgPrint(
        b"(%s:%d) Slot lookup for %d.%u.%u\n\0".as_ptr() as *const u8,
        b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
        1352i32,
        if !ParentExtension.is_null() {
            (*ParentExtension).BaseBus as i32
        } else {
            -(1i32)
        },
        (*PdoExtension).Slot.u.bits.DeviceNumber() as i32,
        (*PdoExtension).Slot.u.bits.FunctionNumber() as i32,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const u8,
            b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
            1352i32,
        );
    }

    if !PciIrqRoutingTable.is_null() && !ParentExtension.is_null() {
        let mut SlotInfo: PSLOT_INFO = &raw mut (*PciIrqRoutingTable).Slot as *mut SLOT_INFO;
        if DbgPrint(
            b"(%s:%d) PIR$ %p is %lx bytes, slot 0 is at: %p\n\0".as_ptr() as *const u8,
            b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
            1358i32,
            PciIrqRoutingTable,
            (*PciIrqRoutingTable).TableSize as i32,
            SlotInfo,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const u8,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
                1358i32,
            );
        }

        let table_end = (PciIrqRoutingTable as usize).wrapping_add((*PciIrqRoutingTable).TableSize as usize) as PSLOT_INFO;
        while SlotInfo < table_end {
            if DbgPrint(
                b"(%s:%d) Slot Info: %u.%u->#%u\n\0".as_ptr() as *const u8,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
                1365i32,
                (*SlotInfo).BusNumber as i32,
                (*SlotInfo).DeviceNumber as i32,
                (*SlotInfo).SlotNumber as i32,
            ) != 0
            {
                DbgPrint(
                    b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const u8,
                    b"drivers/bus/pcix/utils.c\0".as_ptr() as *const u8,
                    1365i32,
                );
            }

            if (*ParentExtension).BaseBus as i32 == (*SlotInfo).BusNumber as i32
                && (*PdoExtension).Slot.u.bits.DeviceNumber() as i32 == ((*SlotInfo).DeviceNumber as i32 >> 3)
                && (*SlotInfo).SlotNumber != 0
            {
                *SlotNumber = (*SlotInfo).SlotNumber as u32;
                return STATUS_SUCCESS;
            }

            SlotInfo = SlotInfo.offset(1);
        }
    }

    if (*ParentExtension).BusRootFdoExtension == ParentExtension {
        Status = STATUS_UNSUCCESSFUL;
    } else {
        Status = IoGetDeviceProperty(
            (*ParentExtension).PhysicalDeviceObject,
            DevicePropertyUINumber,
            core::mem::size_of::<ULONG>() as ULONG,
            SlotNumber as *mut core::ffi::c_void,
            &raw mut ResultLength,
        );
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciGetDeviceCapabilities(
    DeviceObject: PDEVICE_OBJECT,
    DeviceCapability: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    // PAGED_CODE() expansion
    if KeGetCurrentIrql() > APC_LEVEL {
        // In ReactOS this would DbgPrint and ASSERT(FALSE).
        // Preserve the intent with a panic.
        RtlAssert(
            b"FALSE\0".as_ptr() as *mut core::ffi::c_void,
            b"utils.rs\0".as_ptr() as *mut core::ffi::c_void,
            line!(),
            b"Pageable code called at IRQL > APC_LEVEL\0".as_ptr() as *mut i8,
        );
    }

    // Zero out capabilities and set initial values
    core::ptr::write_bytes(
        DeviceCapability as *mut u8,
        0,
        core::mem::size_of::<DEVICE_CAPABILITIES>(),
    );
    (*DeviceCapability).Size = core::mem::size_of::<DEVICE_CAPABILITIES>() as u16;
    (*DeviceCapability).Version = 1u16;
    (*DeviceCapability).Address = !0u32;
    (*DeviceCapability).UINumber = !0u32;

    // In place, never moved: an initialized KEVENT's empty wait list points at itself.
    let mut Event: KEVENT = core::mem::zeroed();
    KeInitializeEvent(&mut Event, _EVENT_TYPE_SynchronizationEvent, 0u8);

    let AttachedDevice = IoGetAttachedDeviceReference(DeviceObject);

    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as u32,
        AttachedDevice,
        core::ptr::null_mut(),
        0u32,
        core::ptr::null_mut(),
        &mut Event as PKEVENT,
        &mut IoStatusBlock,
    );

    if Irp.is_null() {
        ObDereferenceObject(AttachedDevice as *mut _);
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*Irp).IoStatus.Information = 0;
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    let IoStackLocation = IoGetNextIrpStackLocation(Irp);
    ntkrnl_sys::ASSERT!(!IoStackLocation.is_null(), "IoStackLocation is null");

    core::ptr::write_bytes(
        IoStackLocation as *mut u8,
        0,
        core::mem::size_of::<IO_STACK_LOCATION>(),
    );
    (*IoStackLocation).MajorFunction = IRP_MJ_PNP as u8;
    (*IoStackLocation).MinorFunction = IRP_MN_QUERY_CAPABILITIES as u8;
    (*IoStackLocation).Parameters.DeviceCapabilities.Capabilities = DeviceCapability;

    IoSetCompletionRoutine(
        Irp,
        None,
        core::ptr::null_mut(),
        0u8,
        0u8,
        0u8,
    );

    let mut Status = IoCallDriver(AttachedDevice, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i8,
            0u8,
            core::ptr::null_mut(),
        );
        Status = (*Irp).IoStatus.__bindgen_anon_1.Status;
    }

    ObDereferenceObject(AttachedDevice as *mut _);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciQueryPowerCapabilities(
    PdoExtension: PPCI_PDO_EXTENSION,
    DeviceCapability: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let DeviceObject: PDEVICE_OBJECT;
    let mut Status: NTSTATUS;
    let mut AttachedCaps: DEVICE_CAPABILITIES = core::mem::zeroed();
    let mut NewPowerState: DEVICE_POWER_STATE;
    let mut DevicePowerState: DEVICE_POWER_STATE;
    let mut DeviceWakeLevel: DEVICE_POWER_STATE;
    let mut DeviceWakeState: DEVICE_POWER_STATE = PowerDeviceUnspecified;
    let mut SystemWakeState: SYSTEM_POWER_STATE;
    let mut DeepestWakeState: SYSTEM_POWER_STATE = PowerSystemUnspecified;
    let mut CurrentState: SYSTEM_POWER_STATE = PowerSystemUnspecified;

    DeepestWakeState = PowerSystemUnspecified;
    SystemWakeState = DeepestWakeState;
    DeviceWakeState = PowerDeviceUnspecified;

    DeviceObject = (*(*PdoExtension).ParentFdoExtension).PhysicalDeviceObject;
    Status = PciGetDeviceCapabilities(DeviceObject, &raw mut AttachedCaps);
    if !NT_SUCCESS(Status) {
        RtlAssert(
            b"NT_SUCCESS(Status)\0".as_ptr() as PVOID,
            b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
            1494,
            core::ptr::null_mut(),
        );
    }
    if !NT_SUCCESS(Status) {
        return Status;
    }

    if AttachedCaps.DeviceState[PowerSystemWorking as usize] == PowerDeviceUnspecified {
        AttachedCaps.DeviceState[PowerSystemWorking as usize] = PowerDeviceD0;
    }
    if AttachedCaps.DeviceState[PowerSystemShutdown as usize] == PowerDeviceUnspecified {
        AttachedCaps.DeviceState[PowerSystemShutdown as usize] = PowerDeviceD3;
    }

    if ((*PdoExtension).HackFlags & PCI_HACK_NO_PM_CAPS as u64) != 0 {
        (*DeviceCapability).DeviceWake = PowerDeviceUnspecified;
        (*DeviceCapability).SystemWake = PowerSystemUnspecified;
        (*DeviceCapability).set_DeviceD1(0u32);
        (*DeviceCapability).set_DeviceD2(0u32);
        (*DeviceCapability).set_WakeFromD0(0u32);
        (*DeviceCapability).set_WakeFromD1(0u32);
        (*DeviceCapability).set_WakeFromD2(0u32);
        (*DeviceCapability).set_WakeFromD3(0u32);
        core::ptr::copy_nonoverlapping(
            AttachedCaps.DeviceState.as_ptr(),
            (*DeviceCapability).DeviceState.as_mut_ptr(),
            7, // element count — size_of here copied 4x too much (kernel-stack overrun)
        );
        return STATUS_SUCCESS;
    }

    (*DeviceCapability).set_DeviceD1((*PdoExtension).PowerCapabilities.Support.D1() as u32);
    (*DeviceCapability).set_DeviceD2((*PdoExtension).PowerCapabilities.Support.D2() as u32);
    (*DeviceCapability).set_WakeFromD0((*PdoExtension).PowerCapabilities.Support.PMED0() as u32);
    (*DeviceCapability).set_WakeFromD1((*PdoExtension).PowerCapabilities.Support.PMED1() as u32);
    (*DeviceCapability).set_WakeFromD2((*PdoExtension).PowerCapabilities.Support.PMED2() as u32);

    if AttachedCaps.DeviceWake != PowerDeviceD3 {
        (*DeviceCapability).set_WakeFromD3((*PdoExtension).PowerCapabilities.Support.PMED3Hot() as u32);
    } else {
        if (*(*PdoExtension).ParentFdoExtension).BusRootFdoExtension == (*PdoExtension).ParentFdoExtension {
            (*DeviceCapability).set_WakeFromD3((*PdoExtension).PowerCapabilities.Support.PMED3Hot() as u32);
        } else {
            DbgPrint(
                (b"WARNING:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR).cast(),
                b"PciQueryPowerCapabilities\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as PCSTR,
                1559,
            );
        }
    }

    CurrentState = PowerSystemWorking;
    while CurrentState < PowerSystemMaximum {
        DevicePowerState = AttachedCaps.DeviceState[CurrentState as usize];
        NewPowerState = DevicePowerState;

        if NewPowerState == PowerDeviceD1 && (*PdoExtension).PowerCapabilities.Support.D1() == 0 {
            NewPowerState = PowerDeviceD2;
        }
        if NewPowerState == PowerDeviceD2 && (*PdoExtension).PowerCapabilities.Support.D2() == 0 {
            NewPowerState = PowerDeviceD3;
        }

        (*DeviceCapability).DeviceState[CurrentState as usize] = NewPowerState;

        if CurrentState < PowerSystemHibernate && NewPowerState != PowerDeviceUnspecified {
            DeepestWakeState = CurrentState;
        }

        if (CurrentState < AttachedCaps.SystemWake)
            && (NewPowerState >= DevicePowerState)
            && (DevicePowerState != PowerDeviceUnspecified)
            && ((NewPowerState == PowerDeviceD0 && (*DeviceCapability).WakeFromD0() != 0)
                || (NewPowerState == PowerDeviceD1 && (*DeviceCapability).WakeFromD1() != 0)
                || (NewPowerState == PowerDeviceD2 && (*DeviceCapability).WakeFromD2() != 0)
                || (NewPowerState == PowerDeviceD3
                    && (*PdoExtension).PowerCapabilities.Support.PMED3Hot() != 0
                    && (DevicePowerState < PowerDeviceD3
                        || (*PdoExtension).PowerCapabilities.Support.PMED3Cold() != 0)))
        {
            SystemWakeState = CurrentState;
            DeviceWakeState = NewPowerState;
        }

        CurrentState = ((CurrentState as i32) + 1) as SYSTEM_POWER_STATE;
    }

    DeviceWakeLevel = (*PdoExtension).PowerState.DeviceWakeLevel;

    if AttachedCaps.SystemWake != PowerSystemUnspecified
        && AttachedCaps.DeviceWake != PowerDeviceUnspecified
        && DeviceWakeLevel != PowerDeviceUnspecified
        && DeviceWakeLevel >= AttachedCaps.DeviceWake
    {
        (*DeviceCapability).SystemWake = AttachedCaps.SystemWake;
        (*DeviceCapability).DeviceWake = DeviceWakeLevel;

        if (*DeviceCapability).DeviceWake == PowerDeviceD0 && (*DeviceCapability).WakeFromD0() == 0 {
            (*DeviceCapability).DeviceWake = PowerDeviceD1;
        }
        if (*DeviceCapability).DeviceWake == PowerDeviceD1 && (*DeviceCapability).WakeFromD1() == 0 {
            (*DeviceCapability).DeviceWake = PowerDeviceD2;
        }
        if (*DeviceCapability).DeviceWake == PowerDeviceD2 && (*DeviceCapability).WakeFromD2() == 0 {
            (*DeviceCapability).DeviceWake = PowerDeviceD3;
        }
        if (*DeviceCapability).DeviceWake == PowerDeviceD3 && (*DeviceCapability).WakeFromD3() == 0 {
            (*DeviceCapability).DeviceWake = PowerDeviceUnspecified;
            (*DeviceCapability).SystemWake = PowerSystemUnspecified;
        }

        if (*DeviceCapability).DeviceWake == PowerDeviceUnspecified
            || (*DeviceCapability).SystemWake == PowerSystemUnspecified
        {
            if SystemWakeState != PowerSystemUnspecified && DeviceWakeState != PowerDeviceUnspecified {
                (*DeviceCapability).DeviceWake = DeviceWakeState;
                (*DeviceCapability).SystemWake = SystemWakeState;
                if DeviceWakeState == PowerDeviceD3 {
                    (*DeviceCapability).set_WakeFromD3(1u32);
                }
            }
        }

        if (*DeviceCapability).SystemWake > PowerSystemSleeping3
            && ((*DeviceCapability).DeviceWake != PowerDeviceD3
                || (*PdoExtension).PowerCapabilities.Support.PMED3Cold() == 0)
        {
            (*DeviceCapability).SystemWake = DeepestWakeState;
        }

        (*DeviceCapability).D1Latency = 0;
        (*DeviceCapability).D2Latency = 2;
        (*DeviceCapability).D3Latency = 100;

        if (*DeviceCapability).DeviceState[PowerSystemWorking as usize] != PowerDeviceD0 {
            RtlAssert(
                b"DeviceCapability->DeviceState[PowerSystemWorking] == PowerDeviceD0\0".as_ptr() as PVOID,
                b"drivers/bus/pcix/utils.c\0".as_ptr() as PVOID,
                1712,
                core::ptr::null_mut(),
            );
        }
    } else {
        (*DeviceCapability).D1Latency = 0;
        (*DeviceCapability).D2Latency = 0;
        (*DeviceCapability).D3Latency = 0;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciQueryCapabilities(
    PdoExtension: PPCI_PDO_EXTENSION,
    DeviceCapability: PDEVICE_CAPABILITIES,
) -> NTSTATUS {
    let function = (*PdoExtension).Slot.u.bits.FunctionNumber() as u32;
    let device = (*PdoExtension).Slot.u.bits.DeviceNumber() as u32;
    (*DeviceCapability).Address = function | (device << 16);
    (*DeviceCapability).set_UniqueID(0u32);
    if (*PdoExtension).BaseClass as u32 == PCI_CLASS_BRIDGE_DEV
        && (*PdoExtension).SubClass as u32 == PCI_SUBCLASS_BR_HOST
    {
        (*DeviceCapability).set_RawDeviceOK(1u32);
    } else {
        (*DeviceCapability).set_RawDeviceOK(0u32);
    }
    (*DeviceCapability).set_LockSupported(0u32);
    (*DeviceCapability).set_EjectSupported(0u32);
    (*DeviceCapability).set_Removable(0u32);
    (*DeviceCapability).set_DockDevice(0u32);
    PciDetermineSlotNumber(PdoExtension, &raw mut (*DeviceCapability).UINumber);
    let Status = PciQueryPowerCapabilities(PdoExtension, DeviceCapability);
    if Status >= 0 {
        PciDebugDumpQueryCapabilities(DeviceCapability);
    }
    Status
}
