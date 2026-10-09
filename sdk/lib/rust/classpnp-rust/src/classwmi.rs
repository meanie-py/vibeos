use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, IoWMIWriteEvent, KeQueryTimeIncrement, LARGE_INTEGER, LPGUID, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, RtlInitUnicodeString, RtlTimeToTimeFields, TIME_FIELDS};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{STATUS_DEVICE_DOES_NOT_EXIST, TRACE_LEVEL_WARNING};
use crate::types::{GUIDREGINFO, PCLASS_WMI_INFO, PCOMMON_DEVICE_EXTENSION, PFUNCTIONAL_DEVICE_EXTENSION, _CLASS_WMI_INFO};
use ntkrnl_sys::ddk::{CCHAR, GUID, PCCHAR, _UNICODE_STRING};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{IRP_MN_CHANGE_SINGLE_INSTANCE, IRP_MN_CHANGE_SINGLE_ITEM, IRP_MN_EXECUTE_METHOD, IRP_MN_QUERY_SINGLE_INSTANCE, IRP_MN_REGINFO, STATUS_INVALID_BUFFER_SIZE, STATUS_WMI_GUID_NOT_FOUND, STATUS_WMI_INSTANCE_NOT_FOUND, TRACE_LEVEL_VERBOSE, WMIREG_FLAG_INSTANCE_LIST, WMIREG_FLAG_INSTANCE_PDO, WNODE_FLAG_EVENT_ITEM, WNODE_FLAG_FIXED_INSTANCE_SIZE, WNODE_FLAG_SINGLE_INSTANCE, WNODE_FLAG_STATIC_INSTANCE_NAMES, WNODE_FLAG_TOO_SMALL};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{CLASS_TAG_WMI};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{TRACE_FLAG_WMI};
// resolve_imports.py: explicit kernel/private-type imports
use crate::glue::{NonPagedPoolNx};

// WMI IRP minor function codes (sdk/include/xdk/iotypes.h). They are only in
// ntkrnl_sys::hal_ddk, which is x86-only, and this crate also builds arm64 — so
// define them here rather than take an arch-conditional dependency. Typed i32 to
// match the `minorFunction as i32` scrutinee below; without these in scope each
// dispatch arm parsed as a catch-all binding and every WMI request took the
// IRP_MN_QUERY_ALL_DATA path.
const IRP_MN_QUERY_ALL_DATA: i32 = 0x00;
const IRP_MN_ENABLE_EVENTS: i32 = 0x04;
const IRP_MN_DISABLE_EVENTS: i32 = 0x05;
const IRP_MN_ENABLE_COLLECTION: i32 = 0x06;
const IRP_MN_DISABLE_COLLECTION: i32 = 0x07;
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{_GUID};
use crate::shims::*;
use crate::types::{CLASSENABLEDISABLEFUNCTION_EventGeneration as EventGeneration, CLASSENABLEDISABLEFUNCTION_DataBlockCollection as DataBlockCollection};
// resolve_imports.py: explicit kernel/private-type imports
use ntkrnl_sys::ddk::{ExFreePool, KeGetCurrentIrql};
#[cfg(target_arch = "x86")]
use ntkrnl_sys::ddk::KeQuerySystemTime;
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::inline::ke::KeQuerySystemTime;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{__BindgenUnionField, PWMIREGINFO, PWMIREGINFOW, PWNODE_ALL_DATA, PWNODE_HEADER, PWNODE_METHOD_ITEM, PWNODE_SINGLE_INSTANCE, PWNODE_SINGLE_ITEM, WMIREGGUIDW, WMIREGGUIDW__bindgen_ty_1, WMIREGINFO, WNODE_ALL_DATA, WNODE_METHOD_ITEM, WNODE_SINGLE_INSTANCE, WNODE_TOO_SMALL};

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


extern "C" {
    static SharedSystemTime: LARGE_INTEGER;
}
extern "C" {
}
unsafe extern "system" {
    fn IsEqualGUID(Guid1: *const _GUID, Guid2: *const _GUID) -> bool;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const MOFRESOURCENAME: [::core::ffi::c_ushort; 16] = unsafe {
    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_ushort; 16]>(
        *b"M\0o\0f\0R\0e\0s\0o\0u\0r\0c\0e\0N\0a\0m\0e\0\0\0",
    )
};

pub static mut wmiClassGuids: [GUIDREGINFO; 1] = [GUIDREGINFO {
    Guid: _GUID {
        Data1: 0xd5a9a51e as ::core::ffi::c_uint,
        Data2: 0x3f9 as ::core::ffi::c_ushort,
        Data3: 0x404d as ::core::ffi::c_ushort,
        Data4: [
            0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x22 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xf9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xe as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0xb0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x70 as ::core::ffi::c_int as ::core::ffi::c_uchar,
            0x38 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        ],
    },
    InstanceCount: 1 as ULONG,
    Flags: 0 as ULONG,
}];

pub const NUM_CLASS_WMI_GUIDS: usize = (::core::mem::size_of::<[GUIDREGINFO; 1]>() as usize)
    .wrapping_div(::core::mem::size_of::<GUIDREGINFO>() as usize);

pub const TIME_STRING_LENGTH: usize = 25;

pub const MSStorageDriver_ClassErrorLogGuid: GUID = GUID {
    Data1: 0xD5A9A51Eu32,
    Data2: 0x03F9u16,
    Data3: 0x404du16,
    Data4: [0x97u8, 0x22u8, 0x15u8, 0xF9u8, 0x0Eu8, 0xB0u8, 0x70u8, 0x38u8],
};

pub const MSStorageDriver_ClassErrorLogGuid_Index: u32 = 0;
#[no_mangle]
pub unsafe extern "C" fn ClassFindGuid(
    GuidList: *const GUIDREGINFO,
    GuidCount: u32,
    Guid: *const GUID,
    GuidIndex: *mut u32,
) -> u8 {
    if KeGetCurrentIrql() as i32 > (APC_LEVEL) as i32 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"classwmi.c\0".as_ptr() as *const i8,
                142,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    let mut i: u32 = 0;
    while i < GuidCount {
        let guid2 = core::ptr::addr_of!((*GuidList.add((i) as usize)).Guid);
        if IsEqualGUID(Guid, guid2) {
            *GuidIndex = i;
            return 1u8;
        }
        i = i.wrapping_add(1);
    }
    return 0u8;
}
#[no_mangle]
pub unsafe extern "C" fn ClassFindInternalGuid(Guid: *const ::core::ffi::c_void, GuidIndex: *mut u32) -> u8 {
    let mut i: usize = 0;

    if KeGetCurrentIrql() as i32 > 1 {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0".as_ptr() as *const i8,
            KeGetCurrentIrql() as i32,
        );
        if 0 == 0 {
            DbgPrint(
                b"Assertion failed at %s(%d): %s\n\0".as_ptr() as *const i8,
                b"drivers/storage/class/classpnp/classwmi.c\0".as_ptr() as *const i8,
                183i32,
                b"0\0".as_ptr() as *const i8,
            );
            __int2c();
        }
    }

    while i < wmiClassGuids.len() {
        if IsEqualGUID(
            Guid as *const _GUID,
            &(*(wmiClassGuids.as_mut_ptr().add(i))).Guid as *const _,
        ) {
            *GuidIndex = i as u32;
            return 1u8;
        }
        i += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn ClassSystemControl(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let commonExtension = (*DeviceObject).DeviceExtension as PCOMMON_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);

    // PAGED_CODE()
    if KeGetCurrentIrql() > APC_LEVEL {
        DbgPrint(
            b"NTDDK: Pageable code called at IRQL > APC_LEVEL (%d)\n\0" as *const u8 as *const i8,
            (KeGetCurrentIrql()) as i32,
        );
        DbgPrint(
            b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
            b"drivers/storage/class/classpnp/classwmi.c\0" as *const u8 as *const i8,
            238i32,
            b"0\0" as *const u8 as *const i8,
        );
        __int2c();
    }

    let isRemoved = crate::types::ClassAcquireRemoveLockEx(
        DeviceObject, Irp as PVOID,
        b"classwmi.c\0".as_ptr() as *const i8,
        238,
    );
    if isRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    let minorFunction = (*irpStack).MinorFunction;

    if (minorFunction as i32 > (IRP_MN_EXECUTE_METHOD) as i32)
        || ((*irpStack).Parameters.WMI.ProviderId != (((DeviceObject as ULONG_PTR) as u32) as u64) as _)
        || (minorFunction as i32 != (IRP_MN_REGINFO) as i32 && (*commonExtension).GuidCount == 0)
    {
        IoSkipCurrentIrpStackLocation(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        return IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    }

    let buffer = (*irpStack).Parameters.WMI.Buffer as PUCHAR;
    let bufferSize = (*irpStack).Parameters.WMI.BufferSize;
    let mut guidIndex: ULONG = -1i32 as ULONG;
    let mut isInternalGuid: BOOLEAN = 0u8;
    let mut status: NTSTATUS = STATUS_SUCCESS;

    if minorFunction as i32 != (IRP_MN_REGINFO) as i32 {
        if !(*commonExtension).GuidRegInfo.is_null()
            && ClassFindGuid(
                (*commonExtension).GuidRegInfo,
                (*commonExtension).GuidCount,
                (*irpStack).Parameters.WMI.DataPath as LPGUID,
                &mut guidIndex,
            ) != 0
        {
            isInternalGuid = 0u8;
            status = STATUS_SUCCESS;
        } else if ClassFindInternalGuid(
            (*irpStack).Parameters.WMI.DataPath as *const core::ffi::c_void,
            &mut guidIndex,
        ) != 0
        {
            isInternalGuid = 1u8;
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_WMI_GUID_NOT_FOUND;
            TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, b"WMI GUID not found!\0" as *const u8 as PCCHAR);
        }

        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_WMI,
            b"WMI Find Guid = %x, isInternalGuid = %x\0" as *const u8 as PCCHAR,
            status,
            isInternalGuid as i32,
        );

        if NT_SUCCESS(status)
            && (minorFunction as i32 == (IRP_MN_QUERY_SINGLE_INSTANCE) as i32
                || minorFunction as i32 == (IRP_MN_CHANGE_SINGLE_INSTANCE) as i32
                || minorFunction as i32 == (IRP_MN_CHANGE_SINGLE_ITEM) as i32
                || minorFunction as i32 == (IRP_MN_EXECUTE_METHOD) as i32)
        {
            if (*(buffer as PWNODE_HEADER)).Flags & WNODE_FLAG_STATIC_INSTANCE_NAMES != 0 {
                if (*(buffer as PWNODE_SINGLE_INSTANCE)).InstanceIndex != 0 {
                    status = STATUS_WMI_INSTANCE_NOT_FOUND;
                }
            } else {
                status = STATUS_WMI_INSTANCE_NOT_FOUND;
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, b"WMI Instance not found!\0" as *const u8 as PCCHAR);
            }
        }

        if !NT_SUCCESS(status) {
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
            return status;
        }
    }

    let driverExtension = (*commonExtension).DriverExtension;
    let classWmiInfo = if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
        &mut (*driverExtension).InitData.FdoData.ClassWmiInfo as *mut _CLASS_WMI_INFO as PCLASS_WMI_INFO
    } else {
        &mut (*driverExtension).InitData.PdoData.ClassWmiInfo as *mut _CLASS_WMI_INFO as PCLASS_WMI_INFO
    };

    match minorFunction as i32 {
        x if x == IRP_MN_REGINFO as i32 => {
            let mut name = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            let mut mofName = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            let mut nameFlags: ULONG = 0;

            let ClassQueryWmiRegInfoEx = if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
                (*driverExtension).ClassFdoQueryWmiRegInfoEx
            } else {
                (*driverExtension).ClassPdoQueryWmiRegInfoEx
            };

            if !(*classWmiInfo).GuidRegInfo.is_null()
                && (*classWmiInfo).ClassQueryWmiRegInfo.is_some()
                && ClassQueryWmiRegInfoEx.is_none()
            {
                status = ((*classWmiInfo).ClassQueryWmiRegInfo.unwrap())(
                    DeviceObject,
                    &mut nameFlags,
                    &mut name,
                );
                RtlInitUnicodeString(&mut mofName, MOFRESOURCENAME.as_ptr() as PCWSTR);
            } else if !(*classWmiInfo).GuidRegInfo.is_null() && ClassQueryWmiRegInfoEx.is_some() {
                RtlInitUnicodeString(&mut mofName, w!("").as_ptr() as PCWSTR);
                status = ClassQueryWmiRegInfoEx.unwrap()(
                    DeviceObject,
                    &mut nameFlags,
                    &mut name,
                    &mut mofName,
                );
            } else {
                RtlInitUnicodeString(&mut mofName, w!("").as_ptr() as PCWSTR);
                nameFlags = WMIREG_FLAG_INSTANCE_PDO as ULONG;
                status = STATUS_SUCCESS;
            }

            if NT_SUCCESS(status)
                && (nameFlags & WMIREG_FLAG_INSTANCE_PDO == 0 && name.Buffer.is_null())
            {
                status = STATUS_INVALID_DEVICE_REQUEST;
                TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, b"Invalid Device Request!\0" as *const u8 as PCCHAR);
            }

            let mut retSize: ULONG = 0;
            if NT_SUCCESS(status) {
                let guidList = (*classWmiInfo).GuidRegInfo;
                let guidCount = if guidList.is_null() { 0 } else { (*classWmiInfo).GuidCount } as usize
                    + wmiClassGuids.len();
                let nameOffset =
                    (core::mem::size_of::<WMIREGINFO>() + guidCount * core::mem::size_of::<WMIREGGUIDW>()) as ULONG;

                let nameSize: ULONG;
                let nameInfo: ULONG_PTR;
                if nameFlags & WMIREG_FLAG_INSTANCE_PDO as ULONG != 0 {
                    nameSize = 0;
                    nameInfo = if (*commonExtension).__bindgen_anon_1.IsFdo() != 0 {
                        (*(commonExtension as PFUNCTIONAL_DEVICE_EXTENSION)).LowerPdo as ULONG_PTR
                    } else {
                        DeviceObject as ULONG_PTR
                    };
                } else {
                    nameFlags |= WMIREG_FLAG_INSTANCE_LIST as ULONG;
                    nameSize = name.Length as ULONG + core::mem::size_of::<USHORT>() as ULONG;
                    nameInfo = nameOffset as ULONG_PTR;
                }

                let mofResourceOffset = nameOffset + nameSize;
                let registryPathOffset = mofResourceOffset + mofName.Length as ULONG + core::mem::size_of::<USHORT>() as ULONG;
                let regPath = &(*driverExtension).RegistryPath as *const _UNICODE_STRING as PUNICODE_STRING;
                let mut bufferNeeded = registryPathOffset + (*regPath).Length as ULONG;
                bufferNeeded += core::mem::size_of::<USHORT>() as ULONG;

                if bufferNeeded <= bufferSize {
                    retSize = bufferNeeded;
                    (*commonExtension).GuidCount = guidCount as ULONG;
                    (*commonExtension).GuidRegInfo = guidList;

                    let wmiRegInfo = buffer as PWMIREGINFO as PWMIREGINFOW;
                    (*wmiRegInfo).BufferSize = bufferNeeded;
                    (*wmiRegInfo).NextWmiRegInfo = 0;
                    (*wmiRegInfo).MofResourceName = mofResourceOffset;
                    (*wmiRegInfo).RegistryPath = registryPathOffset;
                    (*wmiRegInfo).GuidCount = guidCount as ULONG;

                    // Fill class-specific guids
                    let mut i = 0u32;
                    while i < (*classWmiInfo).GuidCount {
                        let wmiRegGuid = (*wmiRegInfo).WmiRegGuid.as_mut_ptr();
                        wmiRegGuid.offset(i as isize).write(WMIREGGUIDW {
                            Guid: (*guidList.offset(i as isize)).Guid,
                            Flags: (*guidList.offset(i as isize)).Flags | nameFlags,
                            __bindgen_anon_1: WMIREGGUIDW__bindgen_ty_1 {
                                InstanceNameList: __BindgenUnionField::new(),
                                BaseNameOffset: __BindgenUnionField::new(),
                                Pdo: __BindgenUnionField::new(),
                                InstanceInfo: __BindgenUnionField::new(),
                                bindgen_union_field: ((nameInfo as u32) as u64) as _,
                            },
                            InstanceCount: 1,
                        });
                        i += 1;
                    }
                    // Fill internal class guids
                    i = 0;
                    while i < wmiClassGuids.len() as u32 {
                        let wmiRegGuid = (*wmiRegInfo).WmiRegGuid.as_mut_ptr();
                        wmiRegGuid.offset((i + (*classWmiInfo).GuidCount) as isize).write(WMIREGGUIDW {
                            Guid: wmiClassGuids[i as usize].Guid,
                            Flags: wmiClassGuids[i as usize].Flags | nameFlags,
                            __bindgen_anon_1: WMIREGGUIDW__bindgen_ty_1 {
                                InstanceNameList: __BindgenUnionField::new(),
                                BaseNameOffset: __BindgenUnionField::new(),
                                Pdo: __BindgenUnionField::new(),
                                InstanceInfo: __BindgenUnionField::new(),
                                bindgen_union_field: ((nameInfo as u32) as u64) as _,
                            },
                            InstanceCount: 1,
                        });
                        i += 1;
                    }

                    if nameFlags & WMIREG_FLAG_INSTANCE_LIST as ULONG != 0 {
                        bufferNeeded = nameOffset + core::mem::size_of::<WCHAR>() as ULONG;
                        bufferNeeded += name.Length as ULONG;
                        if bufferSize >= bufferNeeded {
                            let stringPtr = buffer.offset(nameOffset as isize) as PWCHAR;
                            *stringPtr = name.Length as WCHAR;
                            stringPtr.offset(1).write_bytes(0, name.Length as usize / 2);
                            core::ptr::copy_nonoverlapping(
                                name.Buffer as *const u8,
                                stringPtr.offset(1) as *mut u8,
                                name.Length as usize,
                            );
                        } else {
                            if !(bufferSize >= bufferNeeded) {
                                DbgPrint(
                                    b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                                    b"drivers/storage/class/classpnp/classwmi.c\0" as *const u8 as *const i8,
                                    463i32,
                                    b"bufferSize >= bufferNeeded\0" as *const u8 as *const i8,
                                );
                                __int2c();
                            }
                            status = STATUS_INVALID_BUFFER_SIZE;
                        }
                    }

                    bufferNeeded = mofResourceOffset + core::mem::size_of::<WCHAR>() as ULONG;
                    bufferNeeded += mofName.Length as ULONG;
                    if bufferSize >= bufferNeeded {
                        let stringPtr = buffer.offset(mofResourceOffset as isize) as PWCHAR;
                        *stringPtr = mofName.Length as WCHAR;
                        core::ptr::copy_nonoverlapping(
                            mofName.Buffer as *const u8,
                            stringPtr.offset(1) as *mut u8,
                            mofName.Length as usize,
                        );
                    } else {
                        if !(bufferSize >= bufferNeeded) {
                            DbgPrint(
                                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                                b"drivers/storage/class/classpnp/classwmi.c\0" as *const u8 as *const i8,
                                477i32,
                                b"bufferSize >= bufferNeeded\0" as *const u8 as *const i8,
                            );
                            __int2c();
                        }
                        status = STATUS_INVALID_BUFFER_SIZE;
                    }

                    bufferNeeded = registryPathOffset + core::mem::size_of::<WCHAR>() as ULONG;
                    bufferNeeded += (*regPath).Length as ULONG;
                    if bufferSize >= bufferNeeded {
                        let stringPtr = buffer.offset(registryPathOffset as isize) as PWCHAR;
                        *stringPtr = (*regPath).Length as WCHAR;
                        core::ptr::copy_nonoverlapping(
                            (*regPath).Buffer as *const u8,
                            stringPtr.offset(1) as *mut u8,
                            (*regPath).Length as usize,
                        );
                    } else {
                        if !(bufferSize >= bufferNeeded) {
                            DbgPrint(
                                b"Assertion failed at %s(%d): %s\n\0" as *const u8 as *const i8,
                                b"drivers/storage/class/classpnp/classwmi.c\0" as *const u8 as *const i8,
                                493i32,
                                b"bufferSize >= bufferNeeded\0" as *const u8 as *const i8,
                            );
                            __int2c();
                        }
                        TracePrint(TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, b"Invalid Buffer Size!\0" as *const u8 as PCCHAR);
                        status = STATUS_INVALID_BUFFER_SIZE;
                    }
                } else {
                    *(buffer as *mut ULONG) = bufferNeeded;
                    retSize = core::mem::size_of::<ULONG>() as ULONG;
                }
            }

            if !name.Buffer.is_null() {
                ExFreePool(name.Buffer as *mut core::ffi::c_void);
            }

            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            (*Irp).IoStatus.Information = (((retSize as ULONG_PTR) as u32) as u64) as _;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, (IO_NO_INCREMENT as CCHAR) as _);
            return status;
        }
        IRP_MN_QUERY_ALL_DATA => {
            let wnode = buffer as PWNODE_ALL_DATA;
            let bufferAvail = if (bufferSize as usize) < core::mem::size_of::<WNODE_ALL_DATA>() { 0 } else { bufferSize - core::mem::size_of::<WNODE_ALL_DATA>() as ULONG };
            (*wnode).DataBlockOffset = core::mem::size_of::<WNODE_ALL_DATA>() as ULONG;
            if isInternalGuid != 0 {
                status = ClassQueryInternalDataBlock(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferAvail,
                    buffer.offset(core::mem::size_of::<WNODE_ALL_DATA>() as isize),
                );
            } else {
                status = ((*classWmiInfo).ClassQueryWmiDataBlock.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferAvail,
                    buffer.offset(core::mem::size_of::<WNODE_ALL_DATA>() as isize),
                );
            }
        }
        x if x == IRP_MN_QUERY_SINGLE_INSTANCE as i32 => {
            let wnode = buffer as PWNODE_SINGLE_INSTANCE;
            let dataBlockOffset = (*wnode).DataBlockOffset;
            if isInternalGuid != 0 {
                status = ClassQueryInternalDataBlock(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferSize - dataBlockOffset,
                    (wnode as PUCHAR).offset(dataBlockOffset as isize),
                );
            } else {
                status = ((*classWmiInfo).ClassQueryWmiDataBlock.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferSize - dataBlockOffset,
                    (wnode as PUCHAR).offset(dataBlockOffset as isize),
                );
            }
        }
        x if x == IRP_MN_CHANGE_SINGLE_INSTANCE as i32 => {
            let wnode = buffer as PWNODE_SINGLE_INSTANCE;
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassSetWmiDataBlock.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).SizeDataBlock,
                    (wnode as PUCHAR).offset((*wnode).DataBlockOffset as isize),
                );
            }
        }
        x if x == IRP_MN_CHANGE_SINGLE_ITEM as i32 => {
            let wnode = buffer as PWNODE_SINGLE_ITEM;
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassSetWmiDataItem.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).ItemId,
                    (*wnode).SizeDataItem,
                    (wnode as PUCHAR).offset((*wnode).DataBlockOffset as isize),
                );
            }
        }
        x if x == IRP_MN_EXECUTE_METHOD as i32 => {
            let wnode = buffer as PWNODE_METHOD_ITEM;
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassExecuteWmiMethod.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).MethodId,
                    (*wnode).SizeDataBlock,
                    bufferSize - (*wnode).DataBlockOffset,
                    buffer.offset((*wnode).DataBlockOffset as isize),
                );
            }
        }
        IRP_MN_ENABLE_EVENTS => {
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassWmiFunctionControl.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    EventGeneration,
                    1u8,
                );
            }
        }
        IRP_MN_DISABLE_EVENTS => {
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassWmiFunctionControl.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    EventGeneration,
                    0u8,
                );
            }
        }
        IRP_MN_ENABLE_COLLECTION => {
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassWmiFunctionControl.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    DataBlockCollection,
                    1u8,
                );
            }
        }
        IRP_MN_DISABLE_COLLECTION => {
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT as CCHAR,
                );
            } else {
                status = ((*classWmiInfo).ClassWmiFunctionControl.unwrap())(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    DataBlockCollection,
                    0u8,
                );
            }
        }
        _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueryInternalDataBlock(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    GuidIndex: ULONG,
    BufferAvail: ULONG,
    Buffer: PUCHAR,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;
    let sizeNeeded: ULONG = 0;
    status = STATUS_WMI_GUID_NOT_FOUND;
    status = ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        sizeNeeded,
        IO_NO_INCREMENT as CCHAR,
    );
    return status;
}
#[no_mangle]
pub unsafe extern "C" fn ConvertTickToDateTime(
    Tick: LARGE_INTEGER,
    String: PWCHAR,
) -> PWCHAR {
    let mut nowTick: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut nowTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut time: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut maxInc: u32 = 0;
    let mut timeFields: TIME_FIELDS = core::mem::zeroed();
    let mut outDateTime: [u16; 26] = [0u16; 26];

    nowTick.QuadPart = 0;
    nowTime.QuadPart = 0;

    // KeQueryTickCount expansion (provided by another module)
    KeQueryTickCount(&mut nowTick.QuadPart);

    maxInc = KeQueryTimeIncrement();

    // KeQuerySystemTime expansion: read the global SharedSystemTime
    nowTime.QuadPart = SharedSystemTime.QuadPart;

    time.QuadPart = nowTime.QuadPart
        - ((nowTick.QuadPart as i64 - Tick.QuadPart as i64) * (maxInc as i64));

    RtlTimeToTimeFields(
        &raw mut time as *mut LARGE_INTEGER,
        &raw mut timeFields as *mut TIME_FIELDS,
    );

    // Format string: L"%04d%02d%02d%02d%02d%02d.%03d***+000"
    let fmt_bytes: [u8; 74] = *b"%\x000\x004\0d\0%\x000\x002\0d\0%\x000\x002\0d\0%\x000\x002\0d\0%\x000\x002\0d\0%\x000\x002\0d\0.\0%\x000\x003\0d\0*\0*\0*\0+\x000\x000\x000\0\0\0";
    let fmt_arr: [u16; 37] = core::mem::transmute::<[u8; 74], [u16; 37]>(fmt_bytes);
    let fmt_pcwstr: *const u16 = fmt_arr.as_ptr();

    strsafe_rust::RtlStringCbPrintfW!(
        outDateTime.as_mut_ptr(),
        core::mem::size_of::<[u16; 26]>(),
        fmt_pcwstr,
        timeFields.Year as i32,
        timeFields.Month as i32,
        timeFields.Day as i32,
        timeFields.Hour as i32,
        timeFields.Minute as i32,
        timeFields.Second as i32,
        timeFields.Milliseconds as i32,
    );

    // RtlMoveMemory expansion: copy 25 WCHARs
    core::ptr::copy_nonoverlapping::<u16>(outDateTime.as_ptr(), String, 25);

    String
}

#[no_mangle]
pub unsafe extern "system" fn ClassWmiCompleteRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    mut Status: NTSTATUS,
    BufferUsed: ULONG,
    PriorityBoost: CCHAR,
) -> NTSTATUS {
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let buffer = (*irpStack).Parameters.WMI.Buffer as PUCHAR;
    let mut retSize: ULONG;
    let minorFunction = (*irpStack).MinorFunction;

    match minorFunction as i32 {
        IRP_MN_QUERY_ALL_DATA => {
            let wnode = buffer as *mut WNODE_ALL_DATA;
            let bufferNeeded = (core::mem::size_of::<WNODE_ALL_DATA>() as ULONG) + BufferUsed;

            if Status >= 0 {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime((&raw mut (*wnode).WnodeHeader.__bindgen_anon_2.TimeStamp).cast());
                (*wnode).WnodeHeader.Flags |= WNODE_FLAG_FIXED_INSTANCE_SIZE as ULONG;
                (*wnode).__bindgen_anon_1.bindgen_union_field = BufferUsed;
                (*wnode).InstanceCount = 1;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                let wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize =
                    core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL as ULONG;
                (*wnodeTooSmall).SizeNeeded =
                    (core::mem::size_of::<WNODE_ALL_DATA>() as ULONG) + BufferUsed;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        }
        x if x == IRP_MN_QUERY_SINGLE_INSTANCE as i32 => {
            let wnode = buffer as *mut WNODE_SINGLE_INSTANCE;
            let bufferNeeded = (*wnode).DataBlockOffset + BufferUsed;

            if Status >= 0 {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime((&raw mut (*wnode).WnodeHeader.__bindgen_anon_2.TimeStamp).cast());
                (*wnode).SizeDataBlock = BufferUsed;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                let wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize =
                    core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL as ULONG;
                (*wnodeTooSmall).SizeNeeded = bufferNeeded;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        }
        x if x == IRP_MN_EXECUTE_METHOD as i32 => {
            let wnode = buffer as *mut WNODE_METHOD_ITEM;
            let bufferNeeded = (*wnode).DataBlockOffset + BufferUsed;

            if Status >= 0 {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime((&raw mut (*wnode).WnodeHeader.__bindgen_anon_2.TimeStamp).cast());
                (*wnode).SizeDataBlock = BufferUsed;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                let wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize =
                    core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL as ULONG;
                (*wnodeTooSmall).SizeNeeded = bufferNeeded;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        }
        _ => {
            retSize = 0;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = (((retSize as ULONG_PTR) as u32) as u64) as _;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, (PriorityBoost) as _);
    Status
}
#[no_mangle]
pub unsafe extern "system" fn ClassWmiFireEvent(
    DeviceObject: PDEVICE_OBJECT,
    Guid: LPGUID,
    InstanceIndex: u32,
    EventDataSize: u32,
    EventData: PVOID,
) -> NTSTATUS {
    let mut size_needed: u32;
    let mut event: PWNODE_SINGLE_INSTANCE;
    let mut status: NTSTATUS;

    let mut data_size = EventDataSize;
    if EventData.is_null() {
        data_size = 0;
    }

    size_needed = core::mem::size_of::<WNODE_SINGLE_INSTANCE>() as u32 + data_size;

    event = ExAllocatePoolWithTag(NonPagedPoolNx, size_needed as usize, CLASS_TAG_WMI) as PWNODE_SINGLE_INSTANCE;

    if !event.is_null() {
        // Zero memory
        core::ptr::write_bytes(event as *mut u8, 0u8, size_needed as usize);

        // Fill WnodeHeader fields
        (*event).WnodeHeader.Guid = *Guid;
        (*event).WnodeHeader.ProviderId = IoWMIDeviceObjectToProviderId(DeviceObject);
        (*event).WnodeHeader.BufferSize = size_needed;
        (*event).WnodeHeader.Flags = (WNODE_FLAG_SINGLE_INSTANCE
            | WNODE_FLAG_EVENT_ITEM
            | WNODE_FLAG_STATIC_INSTANCE_NAMES) as u32;

        // Expand KeQuerySystemTime: directly copy the shared system time
        (*event).WnodeHeader.__bindgen_anon_2.bindgen_union_field = SharedSystemTime.QuadPart as u64;

        (*event).InstanceIndex = InstanceIndex;
        (*event).SizeDataBlock = data_size;
        (*event).DataBlockOffset = core::mem::size_of::<WNODE_SINGLE_INSTANCE>() as u32;

        if !EventData.is_null() {
            core::ptr::copy_nonoverlapping(
                EventData as *const u8,
                &raw mut (*event).VariableData as *mut u8,
                data_size as usize,
            );
        }

        status = IoWMIWriteEvent(event as *mut core::ffi::c_void);

        if status < 0 {
            ExFreePool(event as *mut core::ffi::c_void);
            event = core::ptr::null_mut();
        }
    } else {
        status = STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}



// ---- ABI-corrected extern decls (CC matched to in-crate definition) ----
extern "C" {
    pub fn IoWMIDeviceObjectToProviderId(DeviceObject: PDEVICE_OBJECT) -> ULONG;
    
}


// ---- export decls forced to stdcall (real kernel export CC) ----
extern "system" {
    pub fn KeQueryTickCount(TickCount: *mut i64);
}
