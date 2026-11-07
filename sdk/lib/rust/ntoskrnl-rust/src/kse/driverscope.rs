#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK__bindgen_ty_1};
use super::kse::KseRegisterShim;
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK, _KSE_HOOK_COLLECTION, _KSE_SHIM, _KSE_CALLBACK_ROUTINES, PKSE_HOOK, PKSE_HOOK_COLLECTION, PCSTR, GUID, _GUID};
use super::defs::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use crate::shim_exports::IoGetCurrentIrpStackLocation;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::inline::tier1::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{KSE_HOOK, KSE_HOOK_COLLECTION, KSE_SHIM, PDEVICE_OBJECT, PDRIVER_ADD_DEVICE, PDRIVER_DISPATCH, PDRIVER_INITIALIZE, PDRIVER_OBJECT, PDRIVER_STARTIO, PDRIVER_UNLOAD, PIRP, POWER_STATE, PREQUEST_POWER_COMPLETE, PUNICODE_STRING};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_D0A17C8231AD: [u16; 12] = [
    0x0044u16, 0x0072, 0x0069, 0x0076, 0x0065, 0x0072, 0x0053, 0x0063, 0x006F, 0x0070,
    0x0065, 0x0000,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_234 {
    pub FunctionName: PCHAR,
    pub CallbackId: ULONG,
}

// KseCallbackRoutines.KseGetIoCallbacksRoutine / KseSetCompletionHookRoutine are
// filled by KseInitialize before any shim can register (KseRegisterShimEx refuses
// until the engine is INITIALIZED), and the C calls them unconditionally; hence
// the unwrap_unchecked() on them below.
static mut KsepDebugFlags: ULONG = !(0 as ::core::ffi::c_int) as ULONG;

static mut KmDsShimHooksNT: [KSE_HOOK; 7] = [
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"IoCreateDevice\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((ShimIoCreateDevice
                    as unsafe extern "system" fn(
                        PDRIVER_OBJECT,
                        ULONG,
                        PUNICODE_STRING,
                        ULONG,
                        ULONG,
                        BOOLEAN,
                        *mut PDEVICE_OBJECT,
                    ) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"PoRequestPowerIrp\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((ShimPoRequestPowerIrp
                    as unsafe extern "system" fn(
                        PDEVICE_OBJECT,
                        UCHAR,
                        POWER_STATE,
                        PREQUEST_POWER_COMPLETE,
                        PVOID,
                        *mut PIRP,
                    ) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"ExAllocatePoolWithTag\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((ShimExAllocatePoolWithTag
                    as unsafe extern "system" fn(POOL_TYPE, SIZE_T, ULONG) -> PVOID) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"ExFreePoolWithTag\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((ShimExFreePoolWithTag as unsafe extern "system" fn(PVOID, ULONG) -> ()) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"ExAllocatePool\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((ShimExAllocatePool as unsafe extern "system" fn(POOL_TYPE, SIZE_T) -> PVOID) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"ExFreePool\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
            },
            HookFunction: ((ShimExFreePool as unsafe extern "system" fn(PVOID) -> ()) as *const c_void)
                as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookInvalid as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: ::core::ptr::null_mut::<CHAR>(),
            },
            HookFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
    ];

static mut KmDsShimHooksCB: [KSE_HOOK; 10] = [
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: KseHookCallbackDriverInit as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimDriverInit as unsafe extern "system" fn(PDRIVER_OBJECT, PUNICODE_STRING) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: KseHookCallbackDriverStartIo as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimDriverStartIo as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> ()) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: KseHookCallbackDriverUnload as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimDriverUnload as unsafe extern "system" fn(PDRIVER_OBJECT) -> ()) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: KseHookCallbackAddDevice as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimDriverAddDevice
                    as unsafe extern "system" fn(PDRIVER_OBJECT, PDEVICE_OBJECT) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: (KseHookCallbackMajorFunction + (IRP_MJ_CREATE) as u32) as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimIrp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: (KseHookCallbackMajorFunction + (IRP_MJ_CLOSE) as u32) as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimIrp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: (KseHookCallbackMajorFunction + (IRP_MJ_POWER) as u32) as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimIrp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: (KseHookCallbackMajorFunction + (IRP_MJ_PNP) as u32) as ULONG_PTR as PCHAR,
            },
            HookFunction: ((ShimIrpPnp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookIRPCallback as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: (KseHookCallbackMajorFunction + (IRP_MJ_DEVICE_CONTROL) as u32) as ULONG_PTR
                    as PCHAR,
            },
            HookFunction: ((ShimIrp as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookInvalid as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: ::core::ptr::null_mut::<CHAR>(),
            },
            HookFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
    ];

static mut KmDsShimCollections: [KSE_HOOK_COLLECTION; 3] = unsafe {
    [
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionNtExport as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: &raw const KmDsShimHooksNT as PKSE_HOOK,
        },
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionCallback as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: &raw const KmDsShimHooksCB as PKSE_HOOK,
        },
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionInvalid as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: ::core::ptr::null::<_KSE_HOOK>() as *mut _KSE_HOOK,
        },
    ]
};

#[no_mangle]
pub static mut KmDsShimGuid: GUID = _GUID {
    Data1: 0xbc04ab45 as ::core::ffi::c_uint,
    Data2: 0xea7e as ::core::ffi::c_ushort,
    Data3: 0x4a11 as ::core::ffi::c_ushort,
    Data4: [
        0xa7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xbb as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x76 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x15 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xf4 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xca as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xae as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[no_mangle]
pub static mut KmDsShim: KSE_SHIM = unsafe {
    _KSE_SHIM {
        Size: ::core::mem::size_of::<KSE_SHIM>() as ULONG,
        ShimGuid: &raw const KmDsShimGuid,
        ShimName: RUSTCONV_C2R_WSTR_D0A17C8231AD.as_ptr() as PWCHAR,
        KseCallbackRoutines: ::core::ptr::null::<_KSE_CALLBACK_ROUTINES>()
            as *mut _KSE_CALLBACK_ROUTINES,
        ShimmedDriverTargetedNotification: None,
        ShimmedDriverUntargetedNotification: None,
        HookCollectionsArray: &raw const KmDsShimCollections as PKSE_HOOK_COLLECTION,
    }
};

pub const KSEP_DEBUG_EVENTS: ULONG = 0x0001;
pub const KSEP_DEBUG_MESSAGES: ULONG = 0x0002;
pub const KSEP_DEBUG_GENERAL: ULONG = 0x0100;
pub const KSEP_DEBUG_PNP: ULONG = 0x0200;
pub const KSEP_DEBUG_POWER: ULONG = 0x0400;
pub const KSEP_DEBUG_IRP: ULONG = 0x0800;
pub const KSEP_DEBUG_POOL: ULONG = 0x1000;
pub const KSEP_DEBUG_UNLOAD: ULONG = 0x2000;
unsafe extern "system" fn ShimDriverInit(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut OriginalFunction: PDRIVER_INITIALIZE = None;
    let mut Status: NTSTATUS;
    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines)
        .KseGetIoCallbacksRoutine
        .unwrap_unchecked()(DriverObject))
    .DriverInit;
    let _ = OriginalFunction;
    Status = (OriginalFunction.unwrap_unchecked())(DriverObject, RegistryPath);

    if KsepDebugFlags & KSEP_DEBUG_GENERAL != 0 {
        DPRINT1!(
            b"[DriverScopeGeneral] DriverObject=%p [%wZ] RegistryPath=%wZ Status=0x%08x\n\0"
                .as_ptr() as PCSTR,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            RegistryPath,
            Status,
        );
    }
    return Status;
}
unsafe extern "system" fn ShimDriverStartIo(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) {
    let DriverObject: PDRIVER_OBJECT = (*DeviceObject).DriverObject as PDRIVER_OBJECT;
    let mut OriginalFunction: PDRIVER_STARTIO = None;
    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines)
        .KseGetIoCallbacksRoutine
        .unwrap_unchecked()(DriverObject))
    .DriverStartIo;
    if (KsepDebugFlags & KSEP_DEBUG_GENERAL) != 0 {
        DbgPrint(
            b"(%s:%d) [DriverScopeGeneral] DriverObject=%p [%wZ] Fdo=%p Irp=%p\n\0" as *const u8
                as PCSTR,
            b"ntoskrnl/kse/driverscope.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            60 as ::core::ffi::c_int,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            DeviceObject,
            Irp,
        );
    }
    (OriginalFunction.unwrap_unchecked())(DeviceObject, Irp);
}
unsafe extern "system" fn ShimDriverUnload(DriverObject: PDRIVER_OBJECT) {
    let mut OriginalFunction: PDRIVER_UNLOAD;
    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines)
        .KseGetIoCallbacksRoutine
        .unwrap_unchecked()(DriverObject))
    .DriverUnload;
    if KsepDebugFlags & KSEP_DEBUG_GENERAL != 0 {
        DPRINT1!(
            b"[DriverScopeGeneral] DriverObject=%p [%wZ]\n\0".as_ptr() as PCHAR,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
        );
    }
    (OriginalFunction.unwrap_unchecked())(DriverObject);
}
unsafe extern "system" fn ShimDriverAddDevice(
    DriverObject: PDRIVER_OBJECT,
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> NTSTATUS {
    let mut OriginalFunction: PDRIVER_ADD_DEVICE;
    let mut Status: NTSTATUS;
    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines).KseGetIoCallbacksRoutine.unwrap_unchecked()(DriverObject)).AddDevice;
    let _ = OriginalFunction;
    Status = (OriginalFunction.unwrap_unchecked())(DriverObject, PhysicalDeviceObject);

    if (KsepDebugFlags & (KSEP_DEBUG_GENERAL as ULONG)) != 0 {
        DPRINT1!(
            b"[DriverScopeGeneral] DriverObject=%p [%wZ] Pdo=%p Status=0x%08x\n\0".as_ptr() as PCHAR,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            PhysicalDeviceObject,
            Status,
        );
    }
    return Status;
}
unsafe extern "system" fn ShimIrpComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = (*DeviceObject).DriverObject as PDRIVER_OBJECT;
    let MajorFunction: ULONG = (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction as ULONG;
    let Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if KsepDebugFlags & KSEP_DEBUG_IRP != 0 {
        DPRINT1!(
            b"[DriverScopeIrps] DriverObject=%p [%wZ] Fdo=%p Irp=%p MajorCode=0x%02x Status=0x%08x\n\0"
                .as_ptr() as PCSTR,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            DeviceObject,
            Irp,
            MajorFunction,
            Status,
        );
    }

    return STATUS_SUCCESS;
}
unsafe extern "system" fn ShimIrp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = (*DeviceObject).DriverObject;
    let mut OriginalFunction: PDRIVER_DISPATCH;
    let MajorFunction: ULONG = (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction as ULONG;
    let mut Status: NTSTATUS;

    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines)
        .KseGetIoCallbacksRoutine
        .unwrap_unchecked()(DriverObject))
    .MajorFunction[MajorFunction as usize];
    (*KmDsShim.KseCallbackRoutines)
        .KseSetCompletionHookRoutine
        .unwrap_unchecked()(
        DeviceObject,
        Irp,
        Some(ShimIrpComplete as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS),
        core::ptr::null_mut(),
    );
    Status = (OriginalFunction.unwrap_unchecked())(DeviceObject, Irp);

    if KsepDebugFlags & KSEP_DEBUG_IRP as ULONG != 0 {
        if Status == STATUS_PENDING {
            DPRINT1!(
                b"[DriverScopeIrps] DriverObject=%p [%wZ] Fdo=%p Irp=%p\n\0".as_ptr() as PCSTR,
                DriverObject,
                core::ptr::addr_of_mut!((*DriverObject).DriverName),
                DeviceObject,
                Irp,
            );
        } else {
            DPRINT1!(
                b"[DriverScopeIrps] DriverObject=%p [%wZ] Fdo=%p Irp=%p MajorCode=0x%02x Status=0x%08x\n\0"
                    .as_ptr() as PCSTR,
                DriverObject,
                core::ptr::addr_of_mut!((*DriverObject).DriverName),
                DeviceObject,
                Irp,
                MajorFunction,
                Status,
            );
        }
    }
    return Status;
}
unsafe extern "system" fn ShimIrpPnpComplete(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = (*DeviceObject).DriverObject as PDRIVER_OBJECT;
    let Status: NTSTATUS = (*Irp).IoStatus.__bindgen_anon_1.Status;

    if KsepDebugFlags & KSEP_DEBUG_PNP as ULONG != 0 {
        if DbgPrint(
            b"(%s:%d) [DriverScopePnp] DriverObject=%p [%wZ] Fdo=%p Irp=%p Status=0x%08x\n\0".as_ptr() as PCSTR,
            b"ntoskrnl/kse/driverscope.c\0".as_ptr() as *const ::core::ffi::c_char,
            281 as ::core::ffi::c_int,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            DeviceObject,
            Irp,
            Status,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
                b"ntoskrnl/kse/driverscope.c\0".as_ptr() as *const ::core::ffi::c_char,
                281 as ::core::ffi::c_int,
            );
        }
    }

    return STATUS_SUCCESS;
}
unsafe extern "system" fn ShimIrpPnp(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let DriverObject: PDRIVER_OBJECT = (*DeviceObject).DriverObject;
    let OriginalFunction: PDRIVER_DISPATCH;
    let MajorFunction: ULONG = (*IoGetCurrentIrpStackLocation(Irp)).MajorFunction as ULONG;
    let MinorFunction: ULONG = (*IoGetCurrentIrpStackLocation(Irp)).MinorFunction as ULONG;
    let Status: NTSTATUS;

    OriginalFunction = (*(*KmDsShim.KseCallbackRoutines)
        .KseGetIoCallbacksRoutine
        .unwrap_unchecked()(DriverObject))
    .MajorFunction[MajorFunction as usize];
    (*KmDsShim.KseCallbackRoutines)
        .KseSetCompletionHookRoutine
        .unwrap_unchecked()(
        DeviceObject,
        Irp,
        Some(ShimIrpPnpComplete as unsafe extern "system" fn(PDEVICE_OBJECT, PIRP, PVOID) -> NTSTATUS),
        core::ptr::null_mut(),
    );
    Status = (OriginalFunction.unwrap_unchecked())(DeviceObject, Irp);

    if KsepDebugFlags & KSEP_DEBUG_PNP != 0 {
        if Status == STATUS_PENDING {
            DPRINT1!(
                b"[DriverScopePnp] DriverObject=%p [%wZ] Fdo=%p Irp=%p\n\0".as_ptr() as PCSTR,
                DriverObject,
                core::ptr::addr_of_mut!((*DriverObject).DriverName),
                DeviceObject,
                Irp,
            );
        } else {
            DPRINT1!(
                b"[DriverScopePnp] DriverObject=%p [%wZ] Fdo=%p Irp=%p MinorCode=0x%02x Status=0x%08x\n\0"
                    .as_ptr() as PCSTR,
                DriverObject,
                core::ptr::addr_of_mut!((*DriverObject).DriverName),
                DeviceObject,
                Irp,
                MinorFunction,
                Status,
            );
        }
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn KseDriverScopeInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;

    Status = KseRegisterShim(
        core::ptr::addr_of_mut!(KmDsShim),
        core::ptr::null_mut(),
        0 as ULONG,
    );
    if NT_SUCCESS(Status) {
        DPRINT!(b"KSE: driver scope shim registered.\n\0".as_ptr() as PCHAR);
    }

    return Status;
}

// ---------------------------------------------------------------------------
// Hand-written: c2rust cannot lower `_ReturnAddress()` (the four pool shims),
// and the model kept damaging the call envelope of the two call-through
// wrappers. Each fetches the original from its slot in KmDsShimHooksNT, as
// the C does. Stable Rust has no _ReturnAddress either: the pool shims print
// the caller only under KSEP_DEBUG_POOL, so they print NULL for it there
// (trace-only divergence; KseRegisterShimEx, where the caller decides
// correctness, gets it from an asm trampoline instead).
// ---------------------------------------------------------------------------

#[inline]
unsafe fn ds_original(index: usize) -> PVOID {
    (*(*KmDsShim.HookCollectionsArray).HookArray.add(index)).OriginalFunction
}

unsafe extern "system" fn ShimIoCreateDevice(
    DriverObject: PDRIVER_OBJECT,
    DeviceExtensionSize: ULONG,
    DeviceName: PUNICODE_STRING,
    DeviceType: ULONG,
    DeviceCharacteristics: ULONG,
    Exclusive: BOOLEAN,
    DeviceObject: *mut PDEVICE_OBJECT,
) -> NTSTATUS {
    let original: unsafe extern "system" fn(
        PDRIVER_OBJECT, ULONG, PUNICODE_STRING, ULONG, ULONG, BOOLEAN, *mut PDEVICE_OBJECT,
    ) -> NTSTATUS = core::mem::transmute(ds_original(0));
    let Status = original(DriverObject, DeviceExtensionSize, DeviceName, DeviceType, DeviceCharacteristics, Exclusive, DeviceObject);
    if KsepDebugFlags & KSEP_DEBUG_GENERAL != 0 {
        DPRINT1!(
            b"[DriverScopeGeneral] DriverObject=%p [%wZ] Fdo=%p DeviceType=%d DeviceCharacteristics=0x%x Exclusive=%d Status=0x%08x\n\0".as_ptr() as PCSTR,
            DriverObject,
            core::ptr::addr_of_mut!((*DriverObject).DriverName),
            *DeviceObject,
            DeviceType,
            DeviceCharacteristics,
            Exclusive as ULONG,
            Status,
        );
    }
    Status
}

unsafe extern "system" fn ShimPoRequestPowerIrp(
    DeviceObject: PDEVICE_OBJECT,
    MinorFunction: UCHAR,
    PowerState: POWER_STATE,
    CompletionFunction: PREQUEST_POWER_COMPLETE,
    Context: PVOID,
    Irp: *mut PIRP,
) -> NTSTATUS {
    let DriverObject = (*DeviceObject).DriverObject;
    let original: unsafe extern "system" fn(
        PDEVICE_OBJECT, UCHAR, POWER_STATE, PREQUEST_POWER_COMPLETE, PVOID, *mut PIRP,
    ) -> NTSTATUS = core::mem::transmute(ds_original(1));
    let Status = original(DeviceObject, MinorFunction, PowerState, CompletionFunction, Context, Irp);
    if KsepDebugFlags & KSEP_DEBUG_POWER != 0 {
        DPRINT1!(
            b"[DriverScopePower] DriverObject=%p Fdo=%p Irp=%p MinorCode=0x%02x PowerState=%u Status=0x%08x\n\0".as_ptr() as PCSTR,
            DriverObject,
            DeviceObject,
            *Irp,
            MinorFunction as ULONG,
            *(core::ptr::addr_of!(PowerState) as *const ULONG),
            Status,
        );
    }
    Status
}

unsafe extern "system" fn ShimExAllocatePoolWithTag(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T, Tag: ULONG) -> PVOID {
    let original: unsafe extern "system" fn(POOL_TYPE, SIZE_T, ULONG) -> PVOID = core::mem::transmute(ds_original(2));
    let p = original(PoolType, NumberOfBytes, Tag);
    if KsepDebugFlags & KSEP_DEBUG_POOL != 0 {
        DPRINT1!(
            b"[DriverScopePool] ExAllocatePool: Address=%p Caller=%p Type=%d Size=%u Tag=0x%08x\n\0".as_ptr() as PCSTR,
            p,
            core::ptr::null_mut::<c_void>(),
            PoolType as ULONG,
            NumberOfBytes as ULONG,
            Tag,
        );
    }
    p
}

unsafe extern "system" fn ShimExFreePoolWithTag(p: PVOID, Tag: ULONG) {
    let original: unsafe extern "system" fn(PVOID, ULONG) = core::mem::transmute(ds_original(3));
    if KsepDebugFlags & KSEP_DEBUG_POOL != 0 {
        DPRINT1!(
            b"[DriverScopePool] ExFreePool: Address=%p Caller=%p Tag=0x%08x\n\0".as_ptr() as PCSTR,
            p,
            core::ptr::null_mut::<c_void>(),
            Tag,
        );
    }
    original(p, Tag);
}

unsafe extern "system" fn ShimExAllocatePool(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T) -> PVOID {
    let original: unsafe extern "system" fn(POOL_TYPE, SIZE_T) -> PVOID = core::mem::transmute(ds_original(4));
    let p = original(PoolType, NumberOfBytes);
    if KsepDebugFlags & KSEP_DEBUG_POOL != 0 {
        DPRINT1!(
            b"[DriverScopePool] ExAllocatePool: Address=%p Caller=%p Type=%d Size=%u Tag=0x%08x\n\0".as_ptr() as PCSTR,
            p,
            core::ptr::null_mut::<c_void>(),
            PoolType as ULONG,
            NumberOfBytes as ULONG,
            0 as ULONG,
        );
    }
    p
}

unsafe extern "system" fn ShimExFreePool(p: PVOID) {
    let original: unsafe extern "system" fn(PVOID) = core::mem::transmute(ds_original(5));
    if KsepDebugFlags & KSEP_DEBUG_POOL != 0 {
        DPRINT1!(
            b"[DriverScopePool] ExFreePool: Address=%p Caller=%p Tag=0x%08x\n\0".as_ptr() as PCSTR,
            p,
            core::ptr::null_mut::<c_void>(),
            0 as ULONG,
        );
    }
    original(p);
}
