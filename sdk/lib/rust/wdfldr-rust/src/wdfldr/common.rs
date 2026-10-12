// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_34516DFFE4A4: [u16; 10] = [0x0049u16, 0x006D, 0x0061, 0x0067, 0x0065, 0x0050, 0x0061, 0x0074, 0x0068, 0x0000];

use crate::prelude::*;
use ntkrnl_sys::ddk::{ANSI_STRING, ExAcquireResourceExclusiveLite, ExReleaseResourceLite, KeEnterCriticalRegion, KeLeaveCriticalRegion, _OBJECT_ATTRIBUTES, PCUNICODE_STRING, PKEY_VALUE_PARTIAL_INFORMATION, PUNICODE_STRING, PWCH, RtlCopyUnicodeString, RtlFreeAnsiString, RtlUnicodeStringToAnsiString, ZwClose, ZwOpenKey};
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn FxLdrAcquireLoadedModuleLock() {
    KeEnterCriticalRegion();
    ExAcquireResourceExclusiveLite(
        core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesListLock),
        TRUE as BOOLEAN,
    );
}
#[no_mangle]
pub unsafe extern "C" fn FxLdrReleaseLoadedModuleLock() {
    ExReleaseResourceLite(core::ptr::addr_of_mut!(WdfLdrGlobals.LoadedModulesListLock));
    KeLeaveCriticalRegion();
}
#[no_mangle]
pub unsafe extern "C" fn GetNameFromPath(mut Path: PCUNICODE_STRING, mut Name: PUNICODE_STRING) {
    let mut current_block: u64;
    let mut pNextSym: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut pCurrSym: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    if ((*Path).Length as usize) < ::core::mem::size_of::<WCHAR>() as usize {
        (*Name).Length = 0 as USHORT;
        (*Name).Buffer = ::core::ptr::null_mut::<WCHAR>();
        return;
    }
    (*Name).Buffer = (*Path)
        .Buffer
        .offset(((*Path).Length as ::core::ffi::c_int / 2 as ::core::ffi::c_int) as isize)
        .offset(-(1 as ::core::ffi::c_int as isize));
    (*Name).Length = ::core::mem::size_of::<WCHAR>() as USHORT;
    pNextSym = (*Name).Buffer as PWCHAR;
    loop {
        if pNextSym < (*Path).Buffer {
            (*Name).Length = ((*Name).Length as ::core::ffi::c_ulonglong)
                .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong)
                as USHORT as USHORT;
            (*Name).Buffer = (*Name).Buffer.offset(1);
            current_block = 5124827879235607937;
            break;
        } else {
            pCurrSym = (*Name).Buffer as PWCHAR;
            if *pCurrSym as ::core::ffi::c_int == '\\' as i32 {
                current_block = 12800627514080957624;
                break;
            }
            pNextSym = pCurrSym.offset(-(1 as ::core::ffi::c_int as isize));
            (*Name).Length = ((*Name).Length as ::core::ffi::c_ulonglong)
                .wrapping_add(::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong)
                as USHORT as USHORT;
            (*Name).Buffer = pNextSym as PWCH;
        }
    }
    match current_block {
        12800627514080957624 => {
            (*Name).Buffer = (*Name).Buffer.offset(1);
            (*Name).Length = ((*Name).Length as ::core::ffi::c_ulonglong)
                .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize as ::core::ffi::c_ulonglong)
                as USHORT as USHORT;
            if (*Name).Length as usize == ::core::mem::size_of::<WCHAR>() as usize {
                (*Name).Buffer = ::core::ptr::null_mut::<WCHAR>();
                (*Name).Length = 0 as USHORT;
            }
        }
        _ => {}
    }
    (*Name).MaximumLength = (*Name).Length;
}
#[no_mangle]
pub unsafe extern "C" fn GetImageName(ServicePath: PCUNICODE_STRING, ImageName: PUNICODE_STRING) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut objectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut name: _UNICODE_STRING = core::mem::zeroed();
    let mut path: _UNICODE_STRING = core::mem::zeroed();
    let mut pKeyValPartial: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut keyHandle: HANDLE = core::ptr::null_mut();
    let mut valueName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_34516DFFE4A4.as_ptr());

    // InitializeObjectAttributes expanded inline
    objectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    objectAttributes.RootDirectory = core::ptr::null_mut();
    objectAttributes.ObjectName = ServicePath as PUNICODE_STRING;
    objectAttributes.Attributes = OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
    objectAttributes.SecurityDescriptor = core::ptr::null_mut();
    objectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    status = ZwOpenKey(
        core::ptr::addr_of_mut!(keyHandle),
        KEY_READ,
        core::ptr::addr_of_mut!(objectAttributes),
    );

    if !NT_SUCCESS!(status) {
        __DBGPRINT!(("ERROR: GetImageName failed with status 0x%x\n", status));
        return status;
    }

    status = FxLdrQueryData(
        keyHandle,
        core::ptr::addr_of_mut!(valueName),
        WDFLDR_TAG,
        core::ptr::addr_of_mut!(pKeyValPartial),
    );
    if !NT_SUCCESS!(status) {
        __DBGPRINT!(("ERROR: GetImageName failed with status 0x%x\n", status));
        return status;
    }

    'clean: {
        if (*pKeyValPartial).Type != REG_SZ
            && (*pKeyValPartial).Type != REG_EXPAND_SZ
        {
            status = STATUS_OBJECT_TYPE_MISMATCH;
            __DBGPRINT!(("ERROR: GetImageName failed with status 0x%x\n", status));
        }

        if (*pKeyValPartial).DataLength == 0
            || (*pKeyValPartial).DataLength > 0xFFFF
        {
            status = STATUS_INVALID_PARAMETER;
            __DBGPRINT!(("ERROR: GetImageName failed with status 0x%x\n", status));
        }

        path.Buffer = core::ptr::addr_of_mut!((*pKeyValPartial).Data) as PWCH;
        path.Length = (*pKeyValPartial).DataLength as USHORT;
        path.MaximumLength = (*pKeyValPartial).DataLength as USHORT;

        if (*pKeyValPartial).DataLength as usize >= core::mem::size_of::<WCHAR>()
            && *(core::ptr::addr_of_mut!((*pKeyValPartial).Data) as *mut WCHAR).add(
                (*pKeyValPartial).DataLength as usize / core::mem::size_of::<WCHAR>(),
            ) == 0
        {
            path.Length = ((*pKeyValPartial).DataLength as USHORT as usize)
                .wrapping_sub(core::mem::size_of::<WCHAR>()) as USHORT;
        }

        GetNameFromPath(
            core::ptr::addr_of_mut!(path),
            core::ptr::addr_of_mut!(name),
        );

        if name.Length == 0 {
            status = STATUS_INVALID_PARAMETER;
            __DBGPRINT!(("ERROR: GetNameFromPathW could not find a name, status 0x%x\n", status));
            break 'clean;
        }

        status = RtlUShortAdd(
            name.Length,
            2,
            core::ptr::addr_of_mut!((*ImageName).Length),
        );

        if !NT_SUCCESS!(status) {
            status = STATUS_INTEGER_OVERFLOW;
            __DBGPRINT!(("ERROR: size computation failed with Status 0x%x\n", status));
            break 'clean;
        }

        (*ImageName).Buffer = ExAllocatePoolWithTag(
            PagedPool,
            ((*ImageName).Length) as SIZE_T,
            WDFLDR_TAG,
        ) as PWCH;

        if !(*ImageName).Buffer.is_null() {
            core::ptr::write_bytes(
                (*ImageName).Buffer as *mut u8,
                0,
                (*ImageName).Length as usize,
            );
            (*ImageName).MaximumLength = (*ImageName).Length;
            (*ImageName).Length = 0;
            RtlCopyUnicodeString(ImageName, core::ptr::addr_of_mut!(name));

            __DBGPRINT!(("Version Image Name \"%wZ\"\n", ImageName));
        } else {
            status = STATUS_INSUFFICIENT_RESOURCES;
            __DBGPRINT!(("ERROR: ExAllocatePoolWithTag failed with Status 0x%x\n", status));
        }
    }

    // clean:
    if !keyHandle.is_null() {
        ZwClose(keyHandle);
    }
    if !pKeyValPartial.is_null() {
        ExFreePoolWithTag(pKeyValPartial as PVOID, WDFLDR_TAG);
    }

    return status;
}
#[no_mangle]
pub unsafe extern "C" fn GetImageInfo(ImageName: PCUNICODE_STRING, ImageBase: *mut PVOID, ImageSize: PULONG) -> NTSTATUS {
    let mut ansiImageName: ANSI_STRING = core::mem::zeroed();
    let mut status: NTSTATUS;

    status = RtlUnicodeStringToAnsiString(
        core::ptr::addr_of_mut!(ansiImageName),
        ImageName,
        TRUE,
    );

    if !NT_SUCCESS!(status) {
        __DBGPRINT!("ERROR: RtlUnicodeStringToAnsiString failed with Status 0x%x\n", status);
        return status;
    }

    let mut modulesBuffer: PAUX_MODULE_EXTENDED_INFO = core::ptr::null_mut();
    let mut modulesSize: ULONG = 0;

    'clean: {
        loop {
            status = AuxKlibQueryModuleInformation(
                core::ptr::addr_of_mut!(modulesSize),
                core::mem::size_of::<AUX_MODULE_EXTENDED_INFO>() as ULONG,
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS!(status) {
                break 'clean;
            }

            modulesBuffer = ExAllocatePoolZero(
                PagedPool,
                (modulesSize) as _,
                WDFLDR_TAG,
            ) as PAUX_MODULE_EXTENDED_INFO;
            if modulesBuffer.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                __DBGPRINT!("ERROR: ExAllocatePoolWithTag failed\n");

                break 'clean;
            }

            status = AuxKlibQueryModuleInformation(
                core::ptr::addr_of_mut!(modulesSize),
                core::mem::size_of::<AUX_MODULE_EXTENDED_INFO>() as ULONG,
                modulesBuffer,
            );
            if status == STATUS_BUFFER_TOO_SMALL {
                ExFreePoolWithTag(modulesBuffer as PVOID, WDFLDR_TAG);
                continue;
            } else {
                break;
            }
        }

        if NT_SUCCESS!(status) {
            let mut i: SIZE_T = 0 as SIZE_T;
            while i < (modulesSize as SIZE_T) / (core::mem::size_of::<AUX_MODULE_EXTENDED_INFO>() as SIZE_T) {
                if ((*modulesBuffer.offset(i as isize)).FileNameOffset as core::ffi::c_int)
                    < (AUX_KLIB_MODULE_PATH_LEN as core::ffi::c_int)
                    && _strnicmp(
                        (&raw mut (*modulesBuffer.offset(i as isize)).FullPathName as *mut CHAR)
                            .offset((*modulesBuffer.offset(i as isize)).FileNameOffset as isize)
                            as *mut CHAR,
                        ansiImageName.Buffer as *const core::ffi::c_char,
                        ansiImageName.Length as size_t,
                    ) == (0 as core::ffi::c_int)
                {
                    *ImageBase = (*modulesBuffer.offset(i as isize)).BasicInfo.ImageBase;
                    *ImageSize = (*modulesBuffer.offset(i as isize)).ImageSize;
                    break;
                }
                i = i.wrapping_add(1);
            }
        } else {
            __DBGPRINT!("ERROR: AuxKlibQueryModuleInformation failed with Status 0x%x\n", status);
        }

        ExFreePoolWithTag(modulesBuffer as PVOID, WDFLDR_TAG);
    }

    RtlFreeAnsiString(core::ptr::addr_of_mut!(ansiImageName));
    return status;
}
