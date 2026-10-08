// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct VS_VERSION_INFO_STRUCT32 {
    pub wLength: u16,
    pub wValueLength: u16,
    pub wType: u16,
    pub szKey: [u16; 1],
}

pub type PVS_VERSION_INFO_STRUCT32 = *mut VS_VERSION_INFO_STRUCT32;
pub type PCVS_VERSION_INFO_STRUCT32 = *const VS_VERSION_INFO_STRUCT32;

pub const RT_VERSION: usize = 16;
pub const VS_VERSION_INFO: u16 = 1;
pub const VS_FILE_INFO: usize = RT_VERSION;

#[no_mangle]
pub unsafe extern "C" fn NtGetVersionResource(
    BaseAddress: PVOID,
    Resource: *mut PVOID,
    ResourceSize: PULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut ResourceInfo: LDR_RESOURCE_INFO = LDR_RESOURCE_INFO {
        Type: 0,
        Name: 0,
        Language: 0,
    };
    let mut ResourceDataEntry: PIMAGE_RESOURCE_DATA_ENTRY = core::ptr::null_mut();
    let mut Data: PVOID = core::ptr::null_mut();
    let mut Size: u32 = 0;

    ResourceInfo.Type = (RT_VERSION) as _;
    ResourceInfo.Name = (VS_VERSION_INFO) as _;
    ResourceInfo.Language = (MAKELANGID!(LANG_NEUTRAL, SUBLANG_NEUTRAL) as usize) as _;

    Status = LdrFindResource_U(
        BaseAddress,
        core::ptr::addr_of_mut!(ResourceInfo),
        RESOURCE_DATA_LEVEL,
        core::ptr::addr_of_mut!(ResourceDataEntry),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtGetVersionResource: Version resource not found, Status 0x%08lx\n\0".as_ptr()
                as *const i8,
            Status,
        );
        return Status;
    }

    Status = LdrAccessResource(
        BaseAddress,
        ResourceDataEntry,
        core::ptr::addr_of_mut!(Data),
        core::ptr::addr_of_mut!(Size),
    );
    if ((NT_SUCCESS!(Status)) == 0) {
        DPRINT1!(
            b"NtGetVersionResource: Cannot access Version resource, Status 0x%08lx\n\0".as_ptr()
                as *const i8,
            Status,
        );
        return Status;
    }

    *Resource = Data;
    if !ResourceSize.is_null() {
        *ResourceSize = Size;
    }

    return STATUS_SUCCESS;
}

unsafe extern "C" fn VersionInfo32_FindChild(
    info: PCVS_VERSION_INFO_STRUCT32,
    szKey: PCWSTR,
    cchKey: UINT,
) -> PCVS_VERSION_INFO_STRUCT32 {
    let mut child: PCVS_VERSION_INFO_STRUCT32 = VersionInfo32_Children!(info);

    while (child as usize) < (info as usize).wrapping_add((*info).wLength as usize) {
        if _wcsnicmp(
            core::ptr::addr_of!((*child).szKey) as *const u16,
            szKey,
            cchKey as usize,
        ) == 0
            && *(core::ptr::addr_of!((*child).szKey) as *const u16).add(cchKey as usize) == 0
        {
            return child;
        }

        if (*child).wLength as u32 == 0 {
            return core::ptr::null();
        }
        child = VersionInfo32_Next!(child) as PCVS_VERSION_INFO_STRUCT32;
    }

    return core::ptr::null();
}

unsafe extern "C" fn VersionInfo32_QueryValue(
    mut info: PCVS_VERSION_INFO_STRUCT32,
    mut lpSubBlock: PCWSTR,
    mut lplpBuffer: *mut PVOID,
    mut puLen: PUINT,
    mut pbText: *mut BOOL,
) -> NTSTATUS {
    let mut lpNextSlash: PCWSTR = core::ptr::null::<u16>();

    DPRINT!(b"lpSubBlock : (%S)\n\0".as_ptr() as *const i8, lpSubBlock);

    while *lpSubBlock != 0 {
        lpNextSlash = lpSubBlock;
        while *lpNextSlash != 0 {
            if *lpNextSlash == '\\' as u16 {
                break;
            }
            lpNextSlash = lpNextSlash.offset(1);
        }

        if lpNextSlash == lpSubBlock {
            lpSubBlock = lpSubBlock.offset(1);
            continue;
        }

        info = VersionInfo32_FindChild(
            info,
            lpSubBlock,
            lpNextSlash.offset_from(lpSubBlock) as UINT,
        );
        if info.is_null() {
            if !puLen.is_null() {
                *puLen = 0;
            }
            return STATUS_RESOURCE_TYPE_NOT_FOUND;
        }

        lpSubBlock = lpNextSlash;
    }

    *lplpBuffer = VersionInfo32_Value!(info) as PVOID;
    if !puLen.is_null() {
        *puLen = (*info).wValueLength as UINT;
    }
    if !pbText.is_null() {
        *pbText = (*info).wType as BOOL;
    }

    return STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn NtVerQueryValue(
    pBlock: *const core::ffi::c_void,
    mut lpSubBlock: PCWSTR,
    lplpBuffer: *mut PVOID,
    puLen: PUINT,
) -> NTSTATUS {
    let info: PCVS_VERSION_INFO_STRUCT32 = pBlock as PCVS_VERSION_INFO_STRUCT32;

    DPRINT!(
        b"%s (%p, %S, %p, %p)\n\0".as_ptr() as *const i8,
        __FUNCTION__,
        pBlock,
        lpSubBlock,
        lplpBuffer,
        puLen,
    );

    if pBlock.is_null() {
        return (FALSE) as i32;
    }

    if lpSubBlock.is_null() || *lpSubBlock == 0 {
        lpSubBlock = RUSTCONV_WSTR_B7FC02907B1F.as_ptr();
    }

    return VersionInfo32_QueryValue(
        info,
        lpSubBlock,
        lplpBuffer,
        puLen,
        core::ptr::null_mut::<BOOL>(),
    );
}
