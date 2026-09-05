//! main.c → Rust (ACPI Phase 3b glue leaf). Cohab: #[no_mangle] symbols
//! replace the C file in acpi.sys at link time. ACPICA stays C (libacpica.a).
//!
//! Top-level ACPI driver entry: DriverEntry, Bus_AddDevice (FDO setup),
//! ACPIDispatchCreateClose, ACPIDispatchDeviceControl (button/eval IOCTLs),
//! ButtonWaitThread, and the INIT-time GetProcessorInformation registry probe
//! that builds the Processor* HardwareIds globals.
//!
//! The `#ifndef NDEBUG` deviceName block in Bus_AddDevice is NOT compiled under
//! this driver's NDEBUG build → omitted entirely (both the IoGetDeviceProperty
//! logic and the End: free).
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, unused)]

use crate::shared::*; // crate::types::* (P*_DEVICE_DATA, acpi_bus_event, ACPI_STATUS, AE_OK, ...)
use core::ffi::{c_char, c_int, c_void};

// Kernel base/constants/inline + ddk types.
use ntkrnl_sys::base::UNICODE_STRING; // == buspdo.rs's _UNICODE_STRING alias
use ntkrnl_sys::base::{STATUS_INSUFFICIENT_RESOURCES, STATUS_NO_MEMORY, STATUS_SUCCESS};
use ntkrnl_sys::constants::{
    IO_NO_INCREMENT, IRP_MJ_CLOSE, IRP_MJ_CREATE, KEY_READ, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE,
    REG_EXPAND_SZ, REG_MULTI_SZ, REG_SZ, STATUS_BUFFER_OVERFLOW, STATUS_BUFFER_TOO_SMALL,
    STATUS_PENDING, STATUS_UNSUCCESSFUL, STATUS_NOT_SUPPORTED, STATUS_INVALID_PARAMETER,
    THREAD_ALL_ACCESS,
};
use ntkrnl_sys::driver_ddk::{
    DO_DEVICE_INITIALIZING, DO_POWER_PAGABLE, IRP_MJ_DEVICE_CONTROL, IRP_MJ_PNP, IRP_MJ_POWER,
    IoCompleteRequest, wcslen, swprintf,
};

// STATUS_NO_SUCH_DEVICE is not present in the ntkrnl-sys crate consts → define
// locally. Value from ntstatus.h.
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as i32;
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::{NonPagedPool, PagedPool};

use ntkrnl_sys::ddk::{
    ACCESS_MASK, BOOLEAN, HANDLE, KEY_VALUE_PARTIAL_INFORMATION,
    PDEVICE_OBJECT, PDRIVER_OBJECT, PIRP, PIO_STACK_LOCATION,
    PUNICODE_STRING, PWCHAR, ULONG, SIZE_T, NTSTATUS, IoCreateDevice, IoDeleteDevice,
    IoDetachDevice, IoAttachDeviceToDeviceStack, PsCreateSystemThread, ZwClose, ZwOpenKey,
    ZwQueryValueKey, RtlInitUnicodeString, _OBJECT_ATTRIBUTES,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
};
// Device-extension pointer types live in crate::shared (acpisys.h substrate), not ddk.
use crate::shared::{PCOMMON_DEVICE_DATA, PFDO_DEVICE_DATA, PPDO_DEVICE_DATA};
// I/O device-type constants (wdm.h). FILE_DEVICE_ACPI lives only in hal_bindgen; define
// locally to keep the device driver glue self-contained.
const FILE_DEVICE_ACPI: u32 = 0x32;
const FILE_DEVICE_SECURE_OPEN: u32 = 0x100;
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};

// Power-device/system-state enum-variant constants (bindgen prefixes them).
use ntkrnl_sys::ddk::{
    _DEVICE_POWER_STATE_PowerDeviceUnspecified as PowerDeviceUnspecified,
    _SYSTEM_POWER_STATE_PowerSystemWorking as PowerSystemWorking,
};

// INLINE-only DDK fns (no exported symbol).
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoMarkIrpPending};
use ntkrnl_sys::inline::ex::ExInitializeFastMutex;
use ntkrnl_sys::inline::common::InitializeObjectAttributes;

macro_rules! ASSERT {
    ($($t:tt)*) => {};
}

// ---- ACPICA result macro (acexcep.h): SUCCESS(a)=!(a). AE_OK==0. ----
#[inline]
unsafe fn ACPI_SUCCESS(s: c_int) -> bool {
    s == 0
}
// NT_SUCCESS(s) == (s >= 0).
#[inline]
unsafe fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}

// ---- Pool tags. C multichar 'WXYZ' = ('W'<<24)|('X'<<16)|('Y'<<8)|'Z'. ----
const TAG_IpcA: u32 = u32::from_be_bytes([b'I', b'p', b'c', b'A']); // 'IpcA'
const TAG_MpcA: u32 = u32::from_be_bytes([b'M', b'p', b'c', b'A']); // 'MpcA'

// ---- CTL_CODE(d,f,m,a) = (d<<16)|(a<<14)|(f<<2)|m. ----
const fn ctl_code(device_type: u32, function: u32, method: u32, access: u32) -> u32 {
    (device_type << 16) | (access << 14) | (function << 2) | method
}
// FILE_DEVICE_ACPI=0x32, METHOD_BUFFERED=0, FILE_READ_ACCESS|FILE_WRITE_ACCESS=3 (acpiioct.h).
const IOCTL_ACPI_ASYNC_EVAL_METHOD: u32 = ctl_code(0x32, 0, 0, 3);
const IOCTL_ACPI_EVAL_METHOD: u32 = ctl_code(0x32, 1, 0, 3);
// poclass.h: FILE_DEVICE_BATTERY=0x29, METHOD_BUFFERED=0, FILE_READ_ACCESS=1.
const IOCTL_GET_SYS_BUTTON_CAPS: u32 = ctl_code(0x29, 0x50, 0, 1);
const IOCTL_GET_SYS_BUTTON_EVENT: u32 = ctl_code(0x29, 0x51, 0, 1);
// batclass.h: IOCTL_BATTERY_QUERY_TAG = CTL_CODE(FILE_DEVICE_BATTERY,0x10,0,FILE_READ_ACCESS).
const IOCTL_BATTERY_QUERY_TAG: u32 = ctl_code(0x29, 0x10, 0, 1);

// poclass.h SYS_BUTTON_* caps bits.
const SYS_BUTTON_POWER: ULONG = 0x00000001;
const SYS_BUTTON_SLEEP: ULONG = 0x00000002;
const SYS_BUTTON_LID: ULONG = 0x00000004;

// acpi_drivers.h: ACPI_BUTTON_NOTIFY_STATUS.
const ACPI_BUTTON_NOTIFY_STATUS: u32 = 0x80;

// UNICODE_NULL (sdk: 0).
const UNICODE_NULL: u16 = 0;

// ---- INITIALIZE_PNP_STATE(_Data_) macro (acpisys.h):
//   (_Data_).DevicePnPState = NotStarted; (_Data_).PreviousPnPState = NotStarted;
// `NotStarted` comes from crate::shared. ----

// ---- C-runtime wide/ansi helpers (CRT, not kernel exports) → extern "C".
//   swprintf + wcslen come from driver_ddk above. ----
extern "C" {
    fn wcsstr(s1: *const u16, s2: *const u16) -> *mut u16;
    fn strstr(s1: *const c_char, s2: *const c_char) -> *mut c_char;
}

// ---- Sibling acpi-rust functions (other 3b leaves). ----
// acpi_bus_receive_event is Linux-style → extern "C".
extern "C" {
    fn acpi_bus_receive_event(event: *mut acpi_bus_event) -> c_int;
}
// Bus_PnP / Bus_Power / Bus_PDO_EvalMethod are NTAPI siblings → extern "system".
extern "system" {
    fn Bus_PnP(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn Bus_Power(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS;
    fn Bus_PDO_EvalMethod(DeviceData: PPDO_DEVICE_DATA, Irp: PIRP) -> NTSTATUS;
}
// button.c globals (extern "C", Linux-style). main.c only READS them.
extern "C" {
    static mut power_button: *mut acpi_device;
    static mut sleep_button: *mut acpi_device;
}

// ---- C globals with external linkage DEFINED here (buspdo.rs declares them extern). ----
#[no_mangle]
pub static mut ProcessorHardwareIds: UNICODE_STRING = UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut ProcessorIdString: *mut u16 = core::ptr::null_mut(); // LPWSTR
#[no_mangle]
pub static mut ProcessorNameString: *mut u16 = core::ptr::null_mut(); // LPWSTR

// ===========================================================================
// Bus_AddDevice — NTAPI (CODE_SEG("PAGE") dropped).
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn Bus_AddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut deviceObject: PDEVICE_OBJECT = core::ptr::null_mut();
    let mut deviceData: PFDO_DEVICE_DATA = core::ptr::null_mut();

    // PAGED_CODE() — no-op.

    // The `goto End` cleanup is captured by a loop{...break} so the End: block runs once.
    'main: loop {
        status = IoCreateDevice(
            DriverObject,
            core::mem::size_of::<FDO_DEVICE_DATA>() as ULONG,
            core::ptr::null_mut(),
            FILE_DEVICE_ACPI,
            FILE_DEVICE_SECURE_OPEN,
            1, // TRUE (BOOLEAN exclusive)
            &mut deviceObject,
        );
        if !NT_SUCCESS(status) {
            DPRINT1!("IoCreateDevice() failed with status 0x%X\n", status);
            break 'main;
        }

        deviceData = (*deviceObject).DeviceExtension as PFDO_DEVICE_DATA;
        RtlZeroMemory(
            deviceData as *mut c_void,
            core::mem::size_of::<FDO_DEVICE_DATA>() as usize,
        );

        // INITIALIZE_PNP_STATE(deviceData->Common):
        (*deviceData).Common.DevicePnPState = NotStarted;
        (*deviceData).Common.PreviousPnPState = NotStarted;

        (*deviceData).Common.IsFDO = 1; // TRUE

        (*deviceData).Common.Self_ = deviceObject; // bindgen Self → Self_

        ExInitializeFastMutex(&mut (*deviceData).Mutex);

        // InitializeListHead(&deviceData->ListOfPDOs):
        let head = &mut (*deviceData).ListOfPDOs as *mut _;
        ntkrnl_sys::base::InitializeListHead(head);

        // Set the PDO for use with PlugPlay functions.
        (*deviceData).UnderlyingPDO = PhysicalDeviceObject;

        // Initial power state of the FDO.
        (*deviceData).Common.DevicePowerState = PowerDeviceUnspecified;
        (*deviceData).Common.SystemPowerState = PowerSystemWorking;

        (*deviceObject).Flags |= DO_POWER_PAGABLE;

        // Attach our FDO to the device stack.
        (*deviceData).NextLowerDriver =
            IoAttachDeviceToDeviceStack(deviceObject, PhysicalDeviceObject);

        if (*deviceData).NextLowerDriver.is_null() {
            status = STATUS_NO_SUCH_DEVICE;
            break 'main;
        }

        // The `#ifndef NDEBUG` deviceName block is NOT compiled (NDEBUG) → omitted.

        // Done initializing.
        (*deviceObject).Flags &= !DO_DEVICE_INITIALIZING;

        break 'main;
    }

    // End: (the `#ifndef NDEBUG` deviceName free is omitted).
    if !NT_SUCCESS(status) && !deviceObject.is_null() {
        if !deviceData.is_null() && !(*deviceData).NextLowerDriver.is_null() {
            IoDetachDevice((*deviceData).NextLowerDriver);
        }
        IoDeleteDevice(deviceObject);
    }
    status
}

// ===========================================================================
// ACPIDispatchCreateClose — NTAPI.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn ACPIDispatchCreateClose(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    (*Irp).IoStatus.Information = 0;

    IoCompleteRequest(Irp, IO_NO_INCREMENT);

    STATUS_SUCCESS
}

// ===========================================================================
// ButtonWaitThread — NTAPI, Context = PIRP.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn ButtonWaitThread(Context: *mut c_void) {
    let Irp: PIRP = Context as PIRP;
    let mut result: c_int;
    let mut event: acpi_bus_event = core::mem::zeroed();
    let mut ButtonEvent: ULONG;

    // while (ACPI_SUCCESS(result = acpi_bus_receive_event(&event)) &&
    //        event.type != ACPI_BUTTON_NOTIFY_STATUS);
    loop {
        result = acpi_bus_receive_event(&mut event);
        if !ACPI_SUCCESS(result) {
            break;
        }
        if event.type_ != ACPI_BUTTON_NOTIFY_STATUS {
            continue;
        }
        break;
    }

    if !ACPI_SUCCESS(result) {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_UNSUCCESSFUL;
    } else {
        // strstr(event.bus_id, "PWRF") / "SLPF" — bus_id is [c_char; 8].
        if !strstr(event.bus_id.as_ptr(), c"PWRF".as_ptr()).is_null() {
            ButtonEvent = SYS_BUTTON_POWER;
        } else if !strstr(event.bus_id.as_ptr(), c"SLPF".as_ptr()).is_null() {
            ButtonEvent = SYS_BUTTON_SLEEP;
        } else {
            ButtonEvent = 0;
        }

        // RtlCopyMemory(Irp->AssociatedIrp.SystemBuffer, &ButtonEvent, sizeof(ButtonEvent)).
        RtlCopyMemory(
            (*Irp).AssociatedIrp.SystemBuffer,
            &ButtonEvent as *const ULONG as *const c_void,
            core::mem::size_of::<ULONG>() as usize,
        );
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
    }

    IoCompleteRequest(Irp, IO_NO_INCREMENT);
}

// ===========================================================================
// ACPIDispatchDeviceControl — NTAPI.
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn ACPIDispatchDeviceControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let irpStack: PIO_STACK_LOCATION;
    let mut status: NTSTATUS = STATUS_NOT_SUPPORTED;
    let commonData: PCOMMON_DEVICE_DATA;
    let mut Caps: ULONG = 0;
    let mut ThreadHandle: HANDLE = core::ptr::null_mut();

    irpStack = IoGetCurrentIrpStackLocation(Irp);
    ASSERT!(IRP_MJ_DEVICE_CONTROL == (*irpStack).MajorFunction);

    commonData = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_DATA;

    (*Irp).IoStatus.Information = 0;

    if (*commonData).IsFDO == 0 {
        // union path: irpStack->Parameters.DeviceIoControl.IoControlCode
        let ioControlCode = (*irpStack).Parameters.DeviceIoControl.IoControlCode;
        match ioControlCode {
            x if x == IOCTL_ACPI_ASYNC_EVAL_METHOD => {
                // ASSERT(KeGetCurrentIrql() <= DISPATCH_LEVEL) — no-op.
                status = Bus_PDO_EvalMethod(commonData as PPDO_DEVICE_DATA, Irp);
            }
            x if x == IOCTL_ACPI_EVAL_METHOD => {
                // ASSERT(KeGetCurrentIrql() < DISPATCH_LEVEL) — no-op.
                status = Bus_PDO_EvalMethod(commonData as PPDO_DEVICE_DATA, Irp);
            }
            x if x == IOCTL_GET_SYS_BUTTON_CAPS => {
                if (*irpStack).Parameters.DeviceIoControl.OutputBufferLength
                    < core::mem::size_of::<ULONG>() as ULONG
                {
                    status = STATUS_BUFFER_TOO_SMALL;
                } else {
                    let pdo = commonData as PPDO_DEVICE_DATA;
                    let hwids = (*pdo).HardwareIDs as *const u16;
                    if !wcsstr(hwids, ntkrnl_sys::w!("PNP0C0D").as_ptr()).is_null() {
                        DPRINT1!("Lid button reported to power manager\n");
                        Caps |= SYS_BUTTON_LID;
                    } else if (*pdo).AcpiHandle.is_null() {
                        // NULL handle = fixed feature DO; report both at once.
                        if !power_button.is_null() {
                            DPRINT!("Fixed power button reported to power manager\n");
                            Caps |= SYS_BUTTON_POWER;
                        }
                        if !sleep_button.is_null() {
                            DPRINT!("Fixed sleep button reported to power manager\n");
                            Caps |= SYS_BUTTON_SLEEP;
                        }
                    } else if !wcsstr(hwids, ntkrnl_sys::w!("PNP0C0C").as_ptr()).is_null() {
                        DPRINT!("Control method power button reported to power manager\n");
                        Caps |= SYS_BUTTON_POWER;
                    } else if !wcsstr(hwids, ntkrnl_sys::w!("PNP0C0E").as_ptr()).is_null() {
                        DPRINT!("Control method sleep reported to power manager\n");
                        Caps |= SYS_BUTTON_SLEEP;
                    } else {
                        DPRINT1!("IOCTL_GET_SYS_BUTTON_CAPS sent to a non-button device\n");
                        status = STATUS_INVALID_PARAMETER;
                    }

                    if Caps != 0 {
                        RtlCopyMemory(
                            (*Irp).AssociatedIrp.SystemBuffer,
                            &Caps as *const ULONG as *const c_void,
                            core::mem::size_of::<ULONG>() as usize,
                        );
                        (*Irp).IoStatus.Information = core::mem::size_of::<ULONG>() as _;
                        status = STATUS_SUCCESS;
                    }
                }
            }
            x if x == IOCTL_GET_SYS_BUTTON_EVENT => {
                PsCreateSystemThread(
                    &mut ThreadHandle,
                    THREAD_ALL_ACCESS,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    Some(core::mem::transmute(ButtonWaitThread as *const ())),
                    Irp as *mut c_void,
                );
                ZwClose(ThreadHandle);

                status = STATUS_PENDING;
            }
            x if x == IOCTL_BATTERY_QUERY_TAG => {
                DPRINT!("IOCTL_BATTERY_QUERY_TAG is not supported!\n");
            }
            _ => {
                DPRINT1!("Unsupported IOCTL: %x\n", ioControlCode);
            }
        }
    } else {
        DPRINT1!("IOCTL sent to the ACPI FDO! Kill the caller!\n");
    }

    if status != STATUS_PENDING {
        (*Irp).IoStatus.__bindgen_anon_1.Status = status;
        IoCompleteRequest(Irp, IO_NO_INCREMENT);
    } else {
        IoMarkIrpPending(Irp);
    }

    status
}

// ===========================================================================
// AcpiRegOpenKey — static (CODE_SEG("INIT")) → private unsafe fn.
// ===========================================================================
unsafe fn AcpiRegOpenKey(
    ParentKeyHandle: HANDLE,
    KeyName: *const u16, // LPCWSTR
    DesiredAccess: ACCESS_MASK,
    KeyHandle: *mut HANDLE, // OUT HANDLE (C declares OUT HANDLE but passes &handle)
) -> NTSTATUS {
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Name as *mut _ as PUNICODE_STRING, KeyName);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Name as *mut _ as PUNICODE_STRING,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        ParentKeyHandle,
        core::ptr::null_mut(),
    );

    ZwOpenKey(KeyHandle, DesiredAccess, &mut ObjectAttributes as *mut _ as _)
}

// ===========================================================================
// AcpiRegQueryValue — static (CODE_SEG("INIT")) → private unsafe fn.
// ===========================================================================
unsafe fn AcpiRegQueryValue(
    KeyHandle: HANDLE,
    ValueName: *mut u16, // LPWSTR
    Type: *mut ULONG,    // OUT OPTIONAL
    Data: *mut c_void,   // OUT OPTIONAL
    DataLength: *mut ULONG, // IN OUT OPTIONAL
) -> NTSTATUS {
    let mut ValueInfo: *mut KEY_VALUE_PARTIAL_INFORMATION;
    let mut Name: UNICODE_STRING = core::mem::zeroed();
    let mut BufferLength: ULONG = 0;
    let mut Status: NTSTATUS;

    RtlInitUnicodeString(&mut Name as *mut _ as PUNICODE_STRING, ValueName);

    if !DataLength.is_null() {
        BufferLength = *DataLength;
    }

    // Check if the caller provided a valid buffer.
    if !Data.is_null() && BufferLength != 0 {
        // FIELD_OFFSET(KEY_VALUE_PARTIAL_INFORMATION, Data).
        BufferLength += core::mem::offset_of!(KEY_VALUE_PARTIAL_INFORMATION, Data) as ULONG;

        ValueInfo =
            ExAllocatePoolWithTag(PagedPool, BufferLength as usize, TAG_MpcA)
                as *mut KEY_VALUE_PARTIAL_INFORMATION;
        if ValueInfo.is_null() {
            return STATUS_NO_MEMORY;
        }
    } else {
        // Caller wants the size only.
        ValueInfo = core::ptr::null_mut();
        BufferLength = 0;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut Name as *mut _ as PUNICODE_STRING,
        KeyValuePartialInformation,
        ValueInfo as *mut c_void,
        BufferLength,
        &mut BufferLength,
    );

    if !DataLength.is_null() {
        *DataLength = BufferLength;
    }

    if ValueInfo.is_null() {
        // Size only.
        if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
            return Status;
        }
        Status = STATUS_SUCCESS;
    } else if NT_SUCCESS(Status) {
        if !Type.is_null() {
            *Type = (*ValueInfo).Type;
        }

        // RtlMoveMemory(Data, ValueInfo->Data, ValueInfo->DataLength) — memmove.
        let dataLen = (*ValueInfo).DataLength as usize;
        core::ptr::copy(
            (*ValueInfo).Data.as_ptr() as *const u8,
            Data as *mut u8,
            dataLen,
        );

        // REG_SZ/EXPAND_SZ/MULTI_SZ NUL-termination fixup.
        let vtype = (*ValueInfo).Type;
        if (vtype == REG_SZ || vtype == REG_EXPAND_SZ || vtype == REG_MULTI_SZ)
            && (*ValueInfo).DataLength
                <= (*DataLength).wrapping_sub(core::mem::size_of::<u16>() as ULONG)
        {
            // WCHAR *ptr = (WCHAR *)((ULONG_PTR)Data + ValueInfo->DataLength);
            let ptr = (Data as *mut u8).add((*ValueInfo).DataLength as usize) as *mut u16;
            // if ((ptr > Data) && ptr[-1]) *ptr = 0;
            if (ptr as *const u8) > (Data as *const u8) && *ptr.sub(1) != 0 {
                *ptr = 0;
            }
        }
    }

    if !ValueInfo.is_null() {
        ExFreePoolWithTag(ValueInfo as *mut c_void, TAG_MpcA);
    }

    Status
}

// ===========================================================================
// GetProcessorInformation — static (CODE_SEG("INIT")) → private unsafe fn.
// ===========================================================================
unsafe fn GetProcessorInformation() -> NTSTATUS {
    let mut ProcessorIdentifier: *mut u16 = core::ptr::null_mut();
    let mut ProcessorVendorIdentifier: *mut u16 = core::ptr::null_mut();
    let mut HardwareIdsBuffer: *mut u16 = core::ptr::null_mut();
    let mut ProcessorHandle: HANDLE = core::ptr::null_mut();
    let mut Length: ULONG = 0;
    let mut Level1Length: ULONG = 0;
    let mut Level2Length: ULONG = 0;
    let mut Level3Length: ULONG = 0;
    let mut HardwareIdsLength: usize = 0;
    let mut VendorIdentifierLength: usize;
    let mut i: ULONG;
    let mut Ptr: PWCHAR;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    DPRINT!("GetProcessorInformation()\n");

    // The `goto done` cleanup is captured by a loop{...break} so `done:` runs once.
    'done: loop {
        // Open the key for CPU 0.
        Status = AcpiRegOpenKey(
            core::ptr::null_mut(),
            ntkrnl_sys::w!("\\Registry\\Machine\\Hardware\\Description\\System\\CentralProcessor\\0")
                .as_ptr(),
            KEY_READ,
            &mut ProcessorHandle,
        );
        if !NT_SUCCESS(Status) {
            DPRINT1!("Failed to open CentralProcessor registry key: 0x%lx\n", Status);
            break 'done;
        }

        // Query the processor identifier length.
        Status = AcpiRegQueryValue(
            ProcessorHandle,
            ntkrnl_sys::w!("Identifier").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            DPRINT1!("Failed to query Identifier value: 0x%lx\n", Status);
            break 'done;
        }

        // Fallback for level 1-3 length.
        Level1Length = Length;
        Level2Length = Length;
        Level3Length = Length;

        // Allocate large enough to be zero terminated.
        Length += core::mem::size_of::<u16>() as ULONG; // sizeof(UNICODE_NULL)
        ProcessorIdentifier =
            ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_IpcA) as *mut u16;
        if ProcessorIdentifier.is_null() {
            DPRINT1!("Failed to allocate 0x%lx bytes\n", Length);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        // Query the processor identifier string.
        Status = AcpiRegQueryValue(
            ProcessorHandle,
            ntkrnl_sys::w!("Identifier").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            ProcessorIdentifier as *mut c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            DPRINT1!("Failed to query Identifier value: 0x%lx\n", Status);
            break 'done;
        }

        // Query the processor name length.
        Length = 0;
        Status = AcpiRegQueryValue(
            ProcessorHandle,
            ntkrnl_sys::w!("ProcessorNameString").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if NT_SUCCESS(Status) {
            Length += core::mem::size_of::<u16>() as ULONG;
            ProcessorNameString =
                ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_IpcA) as *mut u16;
            if ProcessorNameString.is_null() {
                DPRINT1!("Failed to allocate 0x%lx bytes\n", Length);
                Status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            Status = AcpiRegQueryValue(
                ProcessorHandle,
                ntkrnl_sys::w!("ProcessorNameString").as_ptr() as *mut u16,
                core::ptr::null_mut(),
                ProcessorNameString as *mut c_void,
                &mut Length,
            );
            if !NT_SUCCESS(Status) {
                DPRINT1!("Failed to query ProcessorNameString value: 0x%lx\n", Status);
                break 'done;
            }
        }

        // Query the vendor identifier length.
        Length = 0;
        Status = AcpiRegQueryValue(
            ProcessorHandle,
            ntkrnl_sys::w!("VendorIdentifier").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Length,
        );
        if !NT_SUCCESS(Status) || Length == 0 {
            DPRINT1!("Failed to query VendorIdentifier value: 0x%lx\n", Status);
            break 'done;
        }

        Length += core::mem::size_of::<u16>() as ULONG;
        ProcessorVendorIdentifier =
            ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_IpcA) as *mut u16;
        if ProcessorVendorIdentifier.is_null() {
            DPRINT1!("Failed to allocate 0x%lx bytes\n", Length);
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        // Query the vendor identifier string.
        Status = AcpiRegQueryValue(
            ProcessorHandle,
            ntkrnl_sys::w!("VendorIdentifier").as_ptr() as *mut u16,
            core::ptr::null_mut(),
            ProcessorVendorIdentifier as *mut c_void,
            &mut Length,
        );
        if !NT_SUCCESS(Status) {
            DPRINT1!("Failed to query VendorIdentifier value: 0x%lx\n", Status);
            break 'done;
        }

        // Change spaces to underscores.
        i = 0;
        let idLen = wcslen(ProcessorIdentifier) as ULONG;
        while i < idLen {
            let ch = ProcessorIdentifier.add(i as usize);
            if *ch == b' ' as u16 {
                *ch = b'_' as u16;
            }
            i += 1;
        }

        Ptr = wcsstr(ProcessorIdentifier, ntkrnl_sys::w!("Stepping").as_ptr());
        if !Ptr.is_null() {
            Ptr = Ptr.sub(1);
            Level1Length = Ptr.offset_from(ProcessorIdentifier) as ULONG;
        }

        Ptr = wcsstr(ProcessorIdentifier, ntkrnl_sys::w!("Model").as_ptr());
        if !Ptr.is_null() {
            Ptr = Ptr.sub(1);
            Level2Length = Ptr.offset_from(ProcessorIdentifier) as ULONG;
        }

        Ptr = wcsstr(ProcessorIdentifier, ntkrnl_sys::w!("Family").as_ptr());
        if !Ptr.is_null() {
            Ptr = Ptr.sub(1);
            Level3Length = Ptr.offset_from(ProcessorIdentifier) as ULONG;
        }

        VendorIdentifierLength = wcslen(ProcessorVendorIdentifier);

        // Calculate the size of the full REG_MULTI_SZ data.
        let vil = VendorIdentifierLength as ULONG;
        HardwareIdsLength = ((5 + vil + 3 + Level1Length + 1
            + 1 + vil + 3 + Level1Length + 1
            + 5 + vil + 3 + Level2Length + 1
            + 1 + vil + 3 + Level2Length + 1
            + 5 + vil + 3 + Level3Length + 1
            + 1 + vil + 3 + Level3Length + 1
            + 1) as usize)
            * core::mem::size_of::<u16>() as usize;

        HardwareIdsBuffer =
            ExAllocatePoolWithTag(PagedPool, HardwareIdsLength, TAG_IpcA) as *mut u16;
        if HardwareIdsBuffer.is_null() {
            Status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        // The 7 swprintf calls building the MULTI_SZ. ALL format strings via w!().
        Length = 0;
        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("ACPI\\%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level1Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("*%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level1Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("ACPI\\%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level2Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("*%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level2Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("ACPI\\%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level3Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        Length += swprintf(
            HardwareIdsBuffer.add(Length as usize),
            ntkrnl_sys::w!("*%s_-_%.*s").as_ptr(),
            ProcessorVendorIdentifier,
            Level3Length,
            ProcessorIdentifier,
        ) as ULONG;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;
        *HardwareIdsBuffer.add(Length as usize) = UNICODE_NULL;
        Length += 1;

        // NT_ASSERT(Length * sizeof(WCHAR) == HardwareIdsLength) — no-op assert.

        ProcessorHardwareIds.Length = HardwareIdsLength as u16; // (SHORT)
        ProcessorHardwareIds.MaximumLength = ProcessorHardwareIds.Length;
        ProcessorHardwareIds.Buffer = HardwareIdsBuffer;

        Length = (5 + vil + 3 + Level1Length + 1) * core::mem::size_of::<u16>() as ULONG;
        ProcessorIdString =
            ExAllocatePoolWithTag(PagedPool, Length as usize, TAG_IpcA) as *mut u16;
        if !ProcessorIdString.is_null() {
            Length = swprintf(
                ProcessorIdString,
                ntkrnl_sys::w!("ACPI\\%s_-_%.*s").as_ptr(),
                ProcessorVendorIdentifier,
                Level1Length,
                ProcessorIdentifier,
            ) as ULONG;
            *ProcessorIdString.add(Length as usize) = UNICODE_NULL;
            Length += 1;
            DPRINT!("ProcessorIdString: %S\n", ProcessorIdString);
        }

        break 'done;
    }

    // done:
    if !ProcessorHandle.is_null() {
        ZwClose(ProcessorHandle);
    }

    if !ProcessorIdentifier.is_null() {
        ExFreePoolWithTag(ProcessorIdentifier as *mut c_void, TAG_IpcA);
    }

    if !ProcessorVendorIdentifier.is_null() {
        ExFreePoolWithTag(ProcessorVendorIdentifier as *mut c_void, TAG_IpcA);
    }

    if !NT_SUCCESS(Status) {
        if !HardwareIdsBuffer.is_null() {
            ExFreePoolWithTag(HardwareIdsBuffer as *mut c_void, TAG_IpcA);
        }
    }

    Status
}

// ===========================================================================
// DriverEntry — NTAPI (CODE_SEG("INIT") dropped).
// ===========================================================================
#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    DPRINT!("Driver Entry \n");

    Status = GetProcessorInformation();
    if !NT_SUCCESS(Status) {
        // NT_ASSERT(FALSE) — no-op.
        return Status;
    }

    // Set entry points into the driver. MajorFunction holds PDRIVER_DISPATCH =
    // Option<unsafe extern "stdcall" fn(...)> — the dispatch routines are
    // extern "system" which on i386 == stdcall, amd64/arm64 == cdecl; the
    // PDRIVER_DISPATCH type is `extern "stdcall"` per bindgen, so cast through.
    (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] =
        Some(core::mem::transmute(ACPIDispatchDeviceControl as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] =
        Some(core::mem::transmute(Bus_PnP as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] =
        Some(core::mem::transmute(Bus_Power as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CREATE as usize] =
        Some(core::mem::transmute(ACPIDispatchCreateClose as *const ()));
    (*DriverObject).MajorFunction[IRP_MJ_CLOSE as usize] =
        Some(core::mem::transmute(ACPIDispatchCreateClose as *const ()));

    (*(*DriverObject).DriverExtension).AddDevice =
        Some(core::mem::transmute(Bus_AddDevice as *const ()));

    STATUS_SUCCESS
}
