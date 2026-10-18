use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct SXS_GUID_INFORMATION_CLR {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub pcwszRuntimeVersion: *const u16,
    pub pcwszTypeName: *const u16,
    pub pcwszAssemblyIdentity: *const u16,
}
pub type PSXS_GUID_INFORMATION_CLR = *mut SXS_GUID_INFORMATION_CLR;

pub const SXS_GUID_INFORMATION_CLR_FLAG_IS_SURROGATE: u32 = 0x1;

pub const SXS_GUID_INFORMATION_CLR_FLAG_IS_CLASS: u32 = 0x2;

pub const SXS_LOOKUP_CLR_GUID_USE_ACTCTX: u32 = 0x00000001;

pub const SXS_LOOKUP_CLR_GUID_FIND_SURROGATE: u32 = 0x00010000;

pub const SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS: u32 = 0x00020000;

pub const SXS_LOOKUP_CLR_GUID_FIND_ANY: u32 =
    SXS_LOOKUP_CLR_GUID_FIND_SURROGATE | SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS;

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct comclassredirect_data {
    pub size: u32,
    pub flags: u32,
    pub model: u32,
    pub clsid: GUID,
    pub alias: GUID,
    pub clsid2: GUID,
    pub tlbid: GUID,
    pub name_len: u32,
    pub name_offset: u32,
    pub progid_len: u32,
    pub progid_offset: u32,
    pub clrdata_len: u32,
    pub clrdata_offset: u32,
    pub miscstatus: u32,
    pub miscstatuscontent: u32,
    pub miscstatusthumbnail: u32,
    pub miscstatusicon: u32,
    pub miscstatusdocprint: u32,
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct clrclass_data {
    pub size: u32,
    pub res: [u32; 2],
    pub module_len: u32,
    pub module_offset: u32,
    pub name_len: u32,
    pub name_offset: u32,
    pub version_len: u32,
    pub version_offset: u32,
    pub res2: [u32; 2],
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct clrsurrogate_data {
    pub size: u32,
    pub res: u32,
    pub clsid: GUID,
    pub version_offset: u32,
    pub version_len: u32,
    pub name_offset: u32,
    pub name_len: u32,
}

#[no_mangle]
pub unsafe extern "system" fn SxsLookupClrGuid(
    mut flags: DWORD,
    mut clsid: *mut GUID,
    mut actctx: HANDLE,
    mut buffer: *mut core::ffi::c_void,
    mut buffer_len: SIZE_T,
    mut buffer_len_required: *mut SIZE_T,
) -> BOOL {
    let mut guid_info: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    guid_info.cbSize = core::mem::size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    let mut assembly_info: *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION =
        core::ptr::null_mut();
    let mut bytes_assembly_info: SIZE_T = 0;
    let mut len_version: core::ffi::c_uint = 0 as core::ffi::c_uint;
    let mut len_name: core::ffi::c_uint = 0;
    let mut len_identity: core::ffi::c_uint = 0;
    let mut ptr_name: *const core::ffi::c_void = core::ptr::null();
    let mut ptr_version: *const core::ffi::c_void = core::ptr::null();
    let mut ptr_identity: *const core::ffi::c_void = core::ptr::null();
    let mut ret: *mut SXS_GUID_INFORMATION_CLR = buffer as *mut SXS_GUID_INFORMATION_CLR;
    let mut retval: BOOL = FALSE as BOOL;
    let mut ret_strings: *mut core::ffi::c_char = core::ptr::null_mut();
    let mut cookie: ULONG_PTR = 0;

    TRACE!(
        "%#lx, %s, %p, %p, %Ix, %p.\n",
        flags,
        wine_dbgstr_guid(clsid),
        actctx,
        buffer,
        buffer_len,
        buffer_len_required
    );

    if (flags & (SXS_LOOKUP_CLR_GUID_USE_ACTCTX as DWORD)) != 0 {
        if ActivateActCtx(actctx, core::ptr::addr_of_mut!(cookie)) == 0 {
            WARN!("Failed to activate context.\n");
            return FALSE as BOOL;
        }
    }

    if (flags & (SXS_LOOKUP_CLR_GUID_FIND_SURROGATE as DWORD)) != 0 {
        retval = FindActCtxSectionGuid(
            FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX as DWORD,
            core::ptr::null_mut(),
            ACTIVATION_CONTEXT_SECTION_CLR_SURROGATES as ULONG,
            clsid,
            core::ptr::addr_of_mut!(guid_info),
        );
        if retval != 0 {
            flags &= !(SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS as DWORD);
        }
    }

    if retval == 0 && (flags & (SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS as DWORD)) != 0 {
        retval = FindActCtxSectionGuid(
            FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX as DWORD,
            core::ptr::null_mut(),
            ACTIVATION_CONTEXT_SECTION_COM_SERVER_REDIRECTION as ULONG,
            clsid,
            core::ptr::addr_of_mut!(guid_info),
        );
        if retval != 0 {
            flags &= !(SXS_LOOKUP_CLR_GUID_FIND_SURROGATE as DWORD);
        }
    }

    'out: {
        if retval == 0 {
            SetLastError(ERROR_NOT_FOUND as DWORD);
            break 'out;
        }

        retval = QueryActCtxW(
            0 as DWORD,
            guid_info.hActCtx,
            core::ptr::addr_of_mut!(guid_info.ulAssemblyRosterIndex) as PVOID,
            AssemblyDetailedInformationInActivationContext as ULONG,
            core::ptr::null_mut(),
            0 as SIZE_T,
            core::ptr::addr_of_mut!(bytes_assembly_info),
        );
        if retval == 0 && GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD {
            break 'out;
        }

        assembly_info = malloc(bytes_assembly_info as size_t)
            as *mut ACTIVATION_CONTEXT_ASSEMBLY_DETAILED_INFORMATION;
        retval = QueryActCtxW(
            0 as DWORD,
            guid_info.hActCtx,
            core::ptr::addr_of_mut!(guid_info.ulAssemblyRosterIndex) as PVOID,
            AssemblyDetailedInformationInActivationContext as ULONG,
            assembly_info as PVOID,
            bytes_assembly_info,
            core::ptr::addr_of_mut!(bytes_assembly_info),
        );
        if retval == 0 {
            break 'out;
        }

        if (flags & (SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS as DWORD)) != 0 {
            let redirect_data: *const comclassredirect_data =
                guid_info.lpData as *const comclassredirect_data;
            let class_data: *const clrclass_data;

            class_data = (redirect_data as *const u8)
                .add((*redirect_data).clrdata_offset as usize)
                as *const clrclass_data;
            ptr_name = (class_data as *const u8).add((*class_data).name_offset as usize)
                as *const core::ffi::c_void;
            ptr_version = (class_data as *const u8).add((*class_data).version_offset as usize)
                as *const core::ffi::c_void;
            len_name = ((*class_data).name_len as usize + core::mem::size_of::<WCHAR>())
                as core::ffi::c_uint;
            if (*class_data).version_len != 0 {
                len_version = ((*class_data).version_len as usize
                    + core::mem::size_of::<WCHAR>()) as core::ffi::c_uint;
            }
        } else {
            let surrogate: *const clrsurrogate_data = guid_info.lpData as *const clrsurrogate_data;
            ptr_name = (surrogate as *const u8).add((*surrogate).name_offset as usize)
                as *const core::ffi::c_void;
            ptr_version = (surrogate as *const u8).add((*surrogate).version_offset as usize)
                as *const core::ffi::c_void;
            len_name = ((*surrogate).name_len as usize + core::mem::size_of::<WCHAR>())
                as core::ffi::c_uint;
            if (*surrogate).version_len != 0 {
                len_version = ((*surrogate).version_len as usize
                    + core::mem::size_of::<WCHAR>()) as core::ffi::c_uint;
            }
        }

        ptr_identity = (*assembly_info).lpAssemblyEncodedAssemblyIdentity as *const core::ffi::c_void;
        len_identity = ((*assembly_info).ulEncodedAssemblyIdentityLength as usize
            + core::mem::size_of::<WCHAR>()) as core::ffi::c_uint;

        *buffer_len_required = (core::mem::size_of::<SXS_GUID_INFORMATION_CLR>()
            + len_identity as usize
            + len_version as usize
            + len_name as usize) as SIZE_T;
        if buffer.is_null() || buffer_len < *buffer_len_required {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            retval = FALSE as BOOL;
            break 'out;
        }

        (*ret).cbSize = core::mem::size_of::<SXS_GUID_INFORMATION_CLR>() as DWORD;
        (*ret).dwFlags = if (flags & (SXS_LOOKUP_CLR_GUID_FIND_CLR_CLASS as DWORD)) != 0 {
            SXS_GUID_INFORMATION_CLR_FLAG_IS_CLASS as DWORD
        } else {
            SXS_GUID_INFORMATION_CLR_FLAG_IS_SURROGATE as DWORD
        };

        /* Copy strings into buffer */
        ret_strings = (ret as *mut core::ffi::c_char)
            .add(core::mem::size_of::<SXS_GUID_INFORMATION_CLR>());

        memcpy(
            ret_strings as *mut core::ffi::c_void,
            ptr_identity,
            len_identity as size_t,
        );
        (*ret).pcwszAssemblyIdentity = ret_strings as *mut WCHAR as PCWSTR;
        ret_strings = ret_strings.add(len_identity as usize);

        memcpy(
            ret_strings as *mut core::ffi::c_void,
            ptr_name,
            len_name as size_t,
        );
        (*ret).pcwszTypeName = ret_strings as *mut WCHAR as PCWSTR;
        ret_strings = ret_strings.add(len_name as usize);

        if len_version > (0 as core::ffi::c_uint) {
            memcpy(
                ret_strings as *mut core::ffi::c_void,
                ptr_version,
                len_version as size_t,
            );
            (*ret).pcwszRuntimeVersion = ret_strings as *mut WCHAR as PCWSTR;
        } else {
            (*ret).pcwszRuntimeVersion = core::ptr::null();
        }

        SetLastError(0 as DWORD);
    }

    // out:
    ReleaseActCtx(guid_info.hActCtx);

    if (flags & (SXS_LOOKUP_CLR_GUID_USE_ACTCTX as DWORD)) != 0 {
        DeactivateActCtx(0 as DWORD, cookie);
    }

    free(assembly_info as *mut core::ffi::c_void);
    return retval;
}
