#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK__bindgen_ty_1, PWCH};
use super::kse::KseRegisterShim;
#[allow(unused_imports)]
use ntkrnl_sys::ddk::{_KSE_HOOK, _KSE_HOOK_COLLECTION, _KSE_SHIM, _KSE_CALLBACK_ROUTINES, PKSE_HOOK, PKSE_HOOK_COLLECTION, PKSE_SHIM, PCSTR, GUID, _GUID};
use super::defs::*;
use ntkrnl_sys::driver_ddk::DbgPrint;
use ntkrnl_sys::ddk::_UNICODE_STRING;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{KSE_HOOK, KSE_HOOK_COLLECTION, KSE_SHIM, PRTL_OSVERSIONINFOEXW, PRTL_OSVERSIONINFOW, PUNICODE_STRING, PsGetVersion, RTL_OSVERSIONINFOEXW, RtlGetVersion};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_234CDC0C83FD: [u16; 19] = [
    0x004Bu16, 0x006D, 0x0057, 0x0069, 0x006E, 0x0032, 0x006B, 0x0033, 0x0056, 0x0065,
    0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x004C, 0x0069, 0x0065, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_CA4AF5AD4472: [u16; 15] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0069, 0x0063, 0x0065, 0x0020, 0x0050, 0x0061,
    0x0063, 0x006B, 0x0020, 0x0032, 0x0000,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_234 {
    pub FunctionName: PCHAR,
    pub CallbackId: ULONG,
}

#[no_mangle]
pub static mut KseWin2k3VersionLieGuid: GUID = _GUID {
    Data1: 0xb120afe6 as ::core::ffi::c_uint,
    Data2: 0x7bdd as ::core::ffi::c_ushort,
    Data3: 0x4672 as ::core::ffi::c_ushort,
    Data4: [
        0xa9 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x90 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xb1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x50 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x78 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

static mut KseWin2k3VersionLie: KSE_VERSION_LIE = _KSE_VERSION_LIE {
        MajorVersion: 5 as ULONG,
        MinorVersion: 2 as ULONG,
        BuildNumber: 3790 as ULONG,
        ServicePackMajor: 2 as USHORT,
        CSDVersion: _UNICODE_STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
                as USHORT,
            MaximumLength: (::core::mem::size_of::<[::core::ffi::c_ushort; 15]>() as usize)
                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>() as usize)
                as USHORT,
            Buffer: RUSTCONV_C2R_WSTR_CA4AF5AD4472.as_ptr() as PWCH,
        },
    };

static mut KseWin2k3VersionLieHooksNT: [KSE_HOOK; 3] = [
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"RtlGetVersion\0" as *const u8 as *const ::core::ffi::c_char
                    as PCHAR,
            },
            HookFunction: ((KseWin2k3RtlGetVersion as unsafe extern "system" fn(PRTL_OSVERSIONINFOW) -> NTSTATUS) as *const c_void) as PVOID,
            OriginalFunction: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        },
        _KSE_HOOK {
            Type: KseHookFunction as ULONG,
            __bindgen_anon_1: _KSE_HOOK__bindgen_ty_1 {
                FunctionName: b"PsGetVersion\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
            },
            HookFunction: ((KseWin2k3PsGetVersion
                    as unsafe extern "system" fn(PULONG, PULONG, PULONG, PUNICODE_STRING) -> BOOLEAN) as *const c_void) as PVOID,
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

static mut KseWin2k3VersionLieShimCollections: [KSE_HOOK_COLLECTION; 2] = unsafe {
    [
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionNtExport as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: &raw const KseWin2k3VersionLieHooksNT as PKSE_HOOK,
        },
        _KSE_HOOK_COLLECTION {
            Type: KseCollectionInvalid as ULONG,
            ExportDriverName: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
            HookArray: ::core::ptr::null::<_KSE_HOOK>() as *mut _KSE_HOOK,
        },
    ]
};

static mut KseVersionLieShims: [KSE_SHIM; 1] = unsafe {
    [_KSE_SHIM {
        Size: ::core::mem::size_of::<KSE_SHIM>() as ULONG,
        ShimGuid: &raw const KseWin2k3VersionLieGuid,
        ShimName: RUSTCONV_C2R_WSTR_234CDC0C83FD.as_ptr() as PWCHAR,
        KseCallbackRoutines: ::core::ptr::null::<_KSE_CALLBACK_ROUTINES>()
            as *mut _KSE_CALLBACK_ROUTINES,
        ShimmedDriverTargetedNotification: None,
        ShimmedDriverUntargetedNotification: None,
        HookCollectionsArray: &raw const KseWin2k3VersionLieShimCollections as PKSE_HOOK_COLLECTION,
    }]
};

#[repr(C)]
pub struct _KSE_VERSION_LIE {
    pub MajorVersion: ULONG,
    pub MinorVersion: ULONG,
    pub BuildNumber: ULONG,
    pub ServicePackMajor: USHORT,
    pub CSDVersion: _UNICODE_STRING,
}
pub type KSE_VERSION_LIE = _KSE_VERSION_LIE;
pub type PKSE_VERSION_LIE = *mut KSE_VERSION_LIE;
unsafe extern "system" fn KseCommonRtlGetVersion(
    lpVersionInformation: PRTL_OSVERSIONINFOW,
    VersionLie: PKSE_VERSION_LIE,
) -> NTSTATUS {
    let mut InfoEx: PRTL_OSVERSIONINFOEXW;
    let mut Status: NTSTATUS;

    Status = RtlGetVersion(lpVersionInformation);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    (*lpVersionInformation).dwMajorVersion = (*VersionLie).MajorVersion;
    (*lpVersionInformation).dwMinorVersion = (*VersionLie).MinorVersion;
    (*lpVersionInformation).dwBuildNumber = (*VersionLie).BuildNumber;
    (*lpVersionInformation).dwPlatformId = VER_PLATFORM_WIN32_NT;

    core::ptr::write_bytes(
        core::ptr::addr_of_mut!((*lpVersionInformation).szCSDVersion) as *mut u8,
        0,
        ::core::mem::size_of::<[WCHAR; 128]>(),
    );
    core::ptr::copy_nonoverlapping(
        (*VersionLie).CSDVersion.Buffer as *const u8,
        core::ptr::addr_of_mut!((*lpVersionInformation).szCSDVersion) as *mut u8,
        core::cmp::min(
            (*VersionLie).CSDVersion.Length as usize,
            ::core::mem::size_of::<[WCHAR; 128]>() - ::core::mem::size_of::<WCHAR>(),
        ),
    );

    if (*lpVersionInformation).dwOSVersionInfoSize as usize
        == ::core::mem::size_of::<RTL_OSVERSIONINFOEXW>()
    {
        InfoEx = lpVersionInformation as PRTL_OSVERSIONINFOEXW;
        (*InfoEx).wServicePackMajor = (*VersionLie).ServicePackMajor;
        (*InfoEx).wServicePackMinor = 0;
    }

    return STATUS_SUCCESS;
}
unsafe extern "system" fn KseCommonPsGetVersion(
    MajorVersion: PULONG,
    MinorVersion: PULONG,
    BuildNumber: PULONG,
    CSDVersion: PUNICODE_STRING,
    VersionLie: PKSE_VERSION_LIE,
) -> BOOLEAN {
    if !MajorVersion.is_null() {
        *MajorVersion = (*VersionLie).MajorVersion;
    }
    if !MinorVersion.is_null() {
        *MinorVersion = (*VersionLie).MinorVersion;
    }
    if !BuildNumber.is_null() {
        *BuildNumber = (*VersionLie).BuildNumber;
    }
    if !CSDVersion.is_null() {
        *CSDVersion = (*VersionLie).CSDVersion;
    }

    PsGetVersion(
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
}
unsafe extern "system" fn KseWin2k3RtlGetVersion(lpVersionInformation: PRTL_OSVERSIONINFOW) -> NTSTATUS {
    return KseCommonRtlGetVersion(lpVersionInformation, core::ptr::addr_of_mut!(KseWin2k3VersionLie));
}
unsafe extern "system" fn KseWin2k3PsGetVersion(
    MajorVersion: PULONG,
    MinorVersion: PULONG,
    BuildNumber: PULONG,
    CSDVersion: PUNICODE_STRING,
) -> BOOLEAN {
    KseCommonPsGetVersion(MajorVersion, MinorVersion, BuildNumber, CSDVersion, &raw mut KseWin2k3VersionLie)
}

#[no_mangle]
pub unsafe extern "system" fn KseVersionLieInitialize() -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut Result: NTSTATUS = STATUS_SUCCESS;
    let mut n: ULONG = 0 as ULONG;
    while (n as usize) < core::mem::size_of_val(&*core::ptr::addr_of!(KseVersionLieShims)) / core::mem::size_of::<KSE_SHIM>() {
        let shim: *mut KSE_SHIM = (&raw mut KseVersionLieShims as *mut KSE_SHIM).offset(n as isize);
        Status = KseRegisterShim(
            shim as PKSE_SHIM,
            core::ptr::null_mut(),
            0 as ULONG,
        );
        if !NT_SUCCESS(Status) {
            DPRINT1!(
                b"KSE: Unable to register %S shim (0x%lx)\n\0".as_ptr() as PCSTR,
                (*shim).ShimName,
                Status,
            );
            Result = Status;
        }
        n = n.wrapping_add(1);
    }
    Result
}
