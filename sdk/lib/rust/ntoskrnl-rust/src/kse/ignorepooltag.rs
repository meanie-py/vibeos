#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK__bindgen_ty_1};
use super::kse::KseRegisterShim;
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK, _KSE_HOOK_COLLECTION, _KSE_SHIM, _KSE_CALLBACK_ROUTINES, PKSE_HOOK, PKSE_HOOK_COLLECTION, GUID, _GUID};
use super::defs::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{KSE_HOOK, KSE_HOOK_COLLECTION, KSE_SHIM};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_B27E7E773645: [u16; 18] = [
    0x0049u16, 0x0067, 0x006E, 0x006F, 0x0072, 0x0065, 0x0046, 0x0072, 0x0065, 0x0065,
    0x0050, 0x006F, 0x006F, 0x006C, 0x0054, 0x0061, 0x0067, 0x0000,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_234 {
    pub FunctionName: PCHAR,
    pub CallbackId: ULONG,
}

#[no_mangle]
pub static mut KseIgnoreFreePoolTagGuid: GUID = _GUID {
    Data1: 0x8d0ed225 as ::core::ffi::c_uint,
    Data2: 0x892 as ::core::ffi::c_ushort,
    Data3: 0x4efc as ::core::ffi::c_ushort,
    Data4: [
        0x9e as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x72 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x89 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x12 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x42 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x74 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x53 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut KseIgnoreFreePoolTagHooksNT: [KSE_HOOK; 2] = [
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
            Type: KseHookInvalid as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: ::core::ptr::null_mut::<CHAR>(),
            },
            HookFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
    ];

static mut KseIgnoreFreePoolTagCollections: [KSE_HOOK_COLLECTION; 2] = unsafe {
    [
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionNtExport as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: &raw const KseIgnoreFreePoolTagHooksNT as PKSE_HOOK,
        },
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionInvalid as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: ::core::ptr::null::<_KSE_HOOK>() as *mut _KSE_HOOK,
        },
    ]
};

#[no_mangle]
pub static mut KseIgnoreFreePoolTagShim: KSE_SHIM = unsafe {
    _KSE_SHIM {
        Size: ::core::mem::size_of::<KSE_SHIM>() as ULONG,
        ShimGuid: &raw const KseIgnoreFreePoolTagGuid,
        ShimName: RUSTCONV_C2R_WSTR_B27E7E773645.as_ptr() as PWCHAR,
        KseCallbackRoutines: ::core::ptr::null::<_KSE_CALLBACK_ROUTINES>()
            as *mut _KSE_CALLBACK_ROUTINES,
        ShimmedDriverTargetedNotification: None,
        ShimmedDriverUntargetedNotification: None,
        HookCollectionsArray: &raw const KseIgnoreFreePoolTagCollections as PKSE_HOOK_COLLECTION,
    }
};
unsafe extern "system" fn ShimExFreePoolWithTag(P: PVOID, mut Tag: ULONG) {
    let mut OriginalFunction: Option<unsafe extern "system" fn(PVOID, ULONG)>;
    OriginalFunction = core::mem::transmute::<PVOID, Option<unsafe extern "system" fn(PVOID, ULONG)>>(
        (*(*KseIgnoreFreePoolTagShim.HookCollectionsArray)
            .HookArray
            .add(0))
        .OriginalFunction,
    );

    if Tag & (PROTECTED_POOL as ULONG) == 0 {
        Tag = 0;
    }

    (OriginalFunction.unwrap_unchecked())(P, Tag);
}
#[no_mangle]
pub unsafe extern "system" fn KseIgnoreFreePoolTagInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS;
    Status = KseRegisterShim(&raw mut KseIgnoreFreePoolTagShim, core::ptr::null_mut(), 0);
    if !NT_SUCCESS(Status) {
        DPRINT1!(b"KSE: Unable to register IgnoreFreePoolTag shim (0x%lx)\n\0".as_ptr() as PCHAR, Status);
    }

    return Status;
}
