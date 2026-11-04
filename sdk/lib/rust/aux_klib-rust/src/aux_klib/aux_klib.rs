// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_298476BC0C7B: [u16; 26] = [
    0x0052u16, 0x0074, 0x006C, 0x0051, 0x0075, 0x0065, 0x0072, 0x0079, 0x004D, 0x006F,
    0x0064, 0x0075, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x0072, 0x006D,
    0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithQuotaTag, MmGetSystemRoutineAddress, PIMAGE_EXPORT_DIRECTORY, PRTL_PROCESS_MODULES, RTL_OSVERSIONINFOW, RTL_PROCESS_MODULES, RtlGetVersion, RtlImageDirectoryEntryToData, UINT32, ZwQuerySystemInformation};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut pfnRtlQueryModuleInformation: PFN_RTLQUERYMODULEINFORMATION = None;

#[no_mangle]
pub static mut gKlibInitialized: LONG = 0 as LONG;

pub const TAG_AUXK: u32 = 0x4175_784B; // 'AuxK' (MSVC multichar: 'A' is most significant byte)

pub type PFN_RTLQUERYMODULEINFORMATION =
    Option<unsafe extern "system" fn(PULONG, ULONG, PVOID) -> NTSTATUS>;

#[no_mangle]
pub unsafe extern "system" fn AuxKlibInitialize() -> NTSTATUS {
    let mut osVersion: RTL_OSVERSIONINFOW = core::mem::zeroed();
    let mut strRtlQueryModuleInformation: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_298476BC0C7B.as_ptr());

    PAGED_CODE!();

    if gKlibInitialized == 0 {
        RtlGetVersion(core::ptr::addr_of_mut!(osVersion));
        if osVersion.dwMajorVersion >= 5 {
            pfnRtlQueryModuleInformation = core::mem::transmute::<PVOID, PFN_RTLQUERYMODULEINFORMATION>(
                MmGetSystemRoutineAddress(core::ptr::addr_of_mut!(strRtlQueryModuleInformation)),
            );
            InterlockedExchange(
                core::ptr::addr_of_mut!(gKlibInitialized) as *mut LONG,
                1,
            );
        } else {
            return STATUS_NOT_SUPPORTED;
        }
    }

    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn AuxKlibQueryModuleInformation(
    InformationLength: PULONG,
    SizePerModule: ULONG,
    ModuleInfo: PAUX_MODULE_EXTENDED_INFO,
) -> NTSTATUS {
    let mut status: NTSTATUS = 0;

    PAGED_CODE!();

    if gKlibInitialized != 1 {
        return STATUS_UNSUCCESSFUL;
    }

    // if we have the function exported from the kernel, use it
    if pfnRtlQueryModuleInformation.is_some() {
        return (pfnRtlQueryModuleInformation.unwrap_unchecked())(InformationLength, SizePerModule, ModuleInfo as PVOID);
    }

    if SizePerModule as usize != core::mem::size_of::<AUX_MODULE_BASIC_INFO>() as usize
        && SizePerModule as usize != core::mem::size_of::<AUX_MODULE_EXTENDED_INFO>() as usize
    {
        return STATUS_INVALID_PARAMETER_2;
    }

    if (ModuleInfo as ULONG_PTR
        & (core::mem::align_of::<AUX_MODULE_EXTENDED_INFO>() as ULONG_PTR
            - 1 as ULONG_PTR))
        != 0
    {
        return STATUS_INVALID_PARAMETER_3;
    }

    // first call the function with a place for only 1 module
    let mut processModulesMinimal: RTL_PROCESS_MODULES = core::mem::zeroed();
    let mut processModules: PRTL_PROCESS_MODULES = core::ptr::addr_of_mut!(processModulesMinimal);
    let mut sysInfoLength: ULONG = core::mem::size_of::<RTL_PROCESS_MODULES>() as ULONG;
    let mut resultLength: ULONG = 0;

    // loop until we have a large-enough buffer for all modules
    loop {
        status = ZwQuerySystemInformation(
            SystemModuleInformation,
            processModules as PVOID,
            sysInfoLength,
            core::ptr::addr_of_mut!(resultLength),
        );

        if status == STATUS_INFO_LENGTH_MISMATCH {
            // free the old buffer if it's not the first one
            if processModules != core::ptr::addr_of_mut!(processModulesMinimal) {
                ExFreePoolWithTag(processModules as PVOID, TAG_AUXK as ULONG);
            }

            // allocate the new one. ExAllocatePoolWithQuotaTag RAISES on a quota failure
            // (it does not return NULL), which the C's _SEH2_TRY catches and returns.
            let mut allocated: PRTL_PROCESS_MODULES = core::ptr::null_mut();
            if let Err(code) = ntkrnl_sys::seh::try_except(
                &mut allocated,
                |p| {
                    *p = ExAllocatePoolWithQuotaTag(
                        PagedPool,
                        resultLength as SIZE_T,
                        TAG_AUXK as ULONG,
                    ) as PRTL_PROCESS_MODULES;
                },
                |_, _| ntkrnl_sys::seh::EXCEPTION_EXECUTE_HANDLER,
            ) {
                return code;
            }
            processModules = allocated;

            if processModules.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            sysInfoLength = resultLength;
        }

        if !(status == STATUS_INFO_LENGTH_MISMATCH) {
            break;
        }
    }

    'cleanup: {
        if !NT_SUCCESS!(status) {
            break 'cleanup;
        }

        let mut modulesSize: ULONG = 0;
        status = RtlULongMult(
            SizePerModule,
            (*processModules).NumberOfModules,
            core::ptr::addr_of_mut!(modulesSize),
        );
        if !NT_SUCCESS!(status) {
            break 'cleanup;
        }

        if ModuleInfo.is_null() {
            ASSERT!(status == STATUS_SUCCESS);
            *InformationLength = modulesSize;
            break 'cleanup;
        }

        if *InformationLength < modulesSize {
            status = STATUS_BUFFER_TOO_SMALL;
            *InformationLength = modulesSize;
            break 'cleanup;
        }

        // copy the information to the input array
        let mut i: UINT32 = 0;
        while (i as ULONG) < (*processModules).NumberOfModules {
            (*ModuleInfo.add(i as usize)).BasicInfo.ImageBase =
                (*core::ptr::addr_of!((*processModules).Modules[0]).add(i as usize)).ImageBase;

            if SizePerModule as usize == core::mem::size_of::<AUX_MODULE_EXTENDED_INFO>() as usize {
                (*ModuleInfo.add(i as usize)).ImageSize =
                    (*core::ptr::addr_of!((*processModules).Modules[0]).add(i as usize)).ImageSize;
                (*ModuleInfo.add(i as usize)).FileNameOffset =
                    (*core::ptr::addr_of!((*processModules).Modules[0]).add(i as usize)).OffsetToFileName;
                RtlCopyMemory!(
                    core::ptr::addr_of_mut!((*ModuleInfo.add(i as usize)).FullPathName)
                        as PVOID,
                    core::ptr::addr_of!((*core::ptr::addr_of!((*processModules).Modules[0]).add(i as usize)).FullPathName)
                        as *const core::ffi::c_void,
                    core::mem::size_of::<[CHAR; 256]>() as SIZE_T,
                );
            }
            i = i.wrapping_add(1);
        }
    }

    // Cleanup:
    // don't accidentally free the stack buffer
    if !processModules.is_null() && processModules != core::ptr::addr_of_mut!(processModulesMinimal)
    {
        ExFreePoolWithTag(processModules as PVOID, TAG_AUXK as ULONG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn AuxKlibGetBugCheckData(BugCheckData: PKBUGCHECK_DATA) -> NTSTATUS {
    // `KiBugCheckData` is an NTSYSAPI (dllimport) array of ntoskrnl: C reads it through the
    // import library's `__imp_` slot, which holds its address. The island's plain `extern
    // static` asked the link for the bare symbol, which no driver defines (wdf01000/wdfldr
    // failed to link). Same spelling as bootvid's KeLoaderBlock: i386's rustc prepends a `_`.
    extern "C" {
        #[cfg_attr(target_arch = "x86", link_name = "_imp__KiBugCheckData")]
        #[cfg_attr(not(target_arch = "x86"), link_name = "__imp_KiBugCheckData")]
        static __imp_KiBugCheckData: *mut ULONG_PTR;
    }
    #[inline]
    unsafe fn KiBugCheckData_get() -> *mut ULONG_PTR {
        __imp_KiBugCheckData
    }

    if (*BugCheckData).BugCheckDataSize as usize != core::mem::size_of::<_KBUGCHECK_DATA>() {
        return STATUS_INFO_LENGTH_MISMATCH;
    }

    (*BugCheckData).BugCheckCode = *KiBugCheckData_get()
        .offset(0 as isize) as ULONG;
    (*BugCheckData).Parameter1 = *KiBugCheckData_get()
        .offset(1 as isize);
    (*BugCheckData).Parameter2 = *KiBugCheckData_get()
        .offset(2 as isize);
    (*BugCheckData).Parameter3 = *KiBugCheckData_get()
        .offset(3 as isize);
    (*BugCheckData).Parameter4 = *KiBugCheckData_get()
        .offset(4 as isize);

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn AuxKlibGetImageExportDirectory(ImageBase: PVOID) -> PIMAGE_EXPORT_DIRECTORY {
    let mut size: ULONG = 0;
    return RtlImageDirectoryEntryToData(ImageBase, TRUE as BOOLEAN, IMAGE_DIRECTORY_ENTRY_EXPORT as USHORT, core::ptr::addr_of_mut!(size)) as PIMAGE_EXPORT_DIRECTORY;
}
#[no_mangle]
pub unsafe extern "system" fn AuxKlibEnumerateSystemFirmwareTables(
    FirmwareTableProviderSignature: ULONG,
    FirmwareTableBuffer: PVOID,
    BufferLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    return STATUS_NOT_IMPLEMENTED;
}

#[no_mangle]
pub unsafe extern "system" fn AuxKlibGetSystemFirmwareTable(
    FirmwareTableProviderSignature: ULONG,
    FirmwareTableID: ULONG,
    FirmwareTableBuffer: PVOID,
    BufferLength: ULONG,
    ReturnLength: PULONG,
) -> NTSTATUS {
    return STATUS_NOT_IMPLEMENTED;
}

/// ntintsafe.h's FORCEINLINE `RtlULongMult`: the product, or `ULONG_ERROR` and
/// STATUS_INTEGER_OVERFLOW when it does not fit in 32 bits.
#[inline]
unsafe fn RtlULongMult(uMultiplicand: ULONG, uMultiplier: ULONG, pulResult: *mut ULONG) -> NTSTATUS {
    match uMultiplicand.checked_mul(uMultiplier) {
        Some(v) => {
            *pulResult = v;
            STATUS_SUCCESS
        }
        None => {
            *pulResult = ULONG::MAX;
            0xC0000095_u32 as NTSTATUS // STATUS_INTEGER_OVERFLOW
        }
    }
}
