use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut image_list: LIST_ENTRY = unsafe {
    _LIST_ENTRY {
        Flink: &raw const image_list as *mut _LIST_ENTRY,
        Blink: &raw const image_list as *mut _LIST_ENTRY,
    }
};



#[no_mangle]
pub unsafe extern "system" fn GetImageConfigInformation(
    LoadedImage: PLOADED_IMAGE,
    ImageConfigInformation: PIMAGE_LOAD_CONFIG_DIRECTORY,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetImageUnusedHeaderBytes(
    LoadedImage: PLOADED_IMAGE,
    SizeUnusedHeaderBytes: LPDWORD,
) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/access.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"GetImageUnusedHeaderBytes\0" as *const u8 as *const ::core::ffi::c_char,
            63 as ::core::ffi::c_int,
            b"(%p, %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            LoadedImage,
            SizeUnusedHeaderBytes,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ImageLoad(dll_name: PCSTR, dll_path: PCSTR) -> PLOADED_IMAGE {
    let mut image: *mut LOADED_IMAGE;
    image = HeapAlloc(GetProcessHeap(), 0, core::mem::size_of::<LOADED_IMAGE>() as SIZE_T) as *mut LOADED_IMAGE;
    if image.is_null() {
        return core::ptr::null_mut();
    }

    if MapAndLoad(dll_name, dll_path, image as PLOADED_IMAGE, (TRUE) as BOOL, (TRUE) as BOOL) == 0
    {
        HeapFree(GetProcessHeap(), 0, image as PVOID);
        return core::ptr::null_mut();
    }

    (*image).Links.Flink = image_list.Flink;
    (*image).Links.Blink = core::ptr::addr_of_mut!(image_list) as *mut LIST_ENTRY;
    (*image_list.Flink).Blink = core::ptr::addr_of_mut!((*image).Links) as *mut LIST_ENTRY;
    image_list.Flink = core::ptr::addr_of_mut!((*image).Links) as *mut LIST_ENTRY;

    return image as PLOADED_IMAGE;
}

#[no_mangle]
pub unsafe extern "system" fn ImageUnload(loaded_image: PLOADED_IMAGE) -> BOOL {
    let mut entry: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut mark: *mut LIST_ENTRY = core::ptr::null_mut();
    let mut image: PLOADED_IMAGE = core::ptr::null_mut();

    TRACE!("(%p)\n", loaded_image);

    /* FIXME: do we really need to check this? */
    mark = core::ptr::addr_of_mut!(image_list);
    entry = (*mark).Flink;
    while entry != mark {
        image = CONTAINING_RECORD!(entry, LOADED_IMAGE, Links);
        if image == loaded_image {
            break;
        }
        entry = (*entry).Flink;
    }

    if entry == mark {
        /* Not found */
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    (*(*entry).Blink).Flink = (*entry).Flink;
    (*(*entry).Flink).Blink = (*entry).Blink;

    UnMapAndLoad(loaded_image);
    HeapFree(GetProcessHeap(), 0 as DWORD, loaded_image as PVOID);

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn MapAndLoad(pszImageName: PCSTR, pszDllPath: PCSTR, pLoadedImage: PLOADED_IMAGE, bDotDll: BOOL, bReadOnly: BOOL) -> BOOL {
    let mut szFileName: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut hFileMapping: HANDLE = NULL;
    let mut mapping: PVOID = NULL;
    let mut pNtHeader: PIMAGE_NT_HEADERS = core::ptr::null_mut();

    'err: {
        if SearchPathA(
            pszDllPath as LPCSTR,
            pszImageName as LPCSTR,
            if bDotDll != 0 {
                b".DLL\0".as_ptr() as LPCSTR
            } else {
                b".EXE\0".as_ptr() as LPCSTR
            },
            core::mem::size_of::<[CHAR; (MAX_PATH) as _]>() as DWORD,
            core::ptr::addr_of_mut!(szFileName) as LPSTR,
            core::ptr::null_mut(),
        ) == 0
        {
            SetLastError(ERROR_FILE_NOT_FOUND as DWORD);
            break 'err;
        }

        hFile = CreateFileA(
            core::ptr::addr_of_mut!(szFileName) as LPCSTR,
            (GENERIC_READ
                | ((if bReadOnly != 0 {
                    0
                } else {
                    GENERIC_WRITE
                }) as DWORD)) as DWORD,
            FILE_SHARE_READ as DWORD,
            core::ptr::null_mut(),
            OPEN_EXISTING as DWORD,
            0 as DWORD,
            NULL,
        );
        if hFile == INVALID_HANDLE_VALUE {
            WARN!("CreateFile: Error = %ld\n", GetLastError());
            break 'err;
        }

        hFileMapping = CreateFileMappingA(
            hFile,
            core::ptr::null_mut(),
            ((if bReadOnly != 0 {
                PAGE_READONLY
            } else {
                PAGE_READWRITE
            }) | SEC_COMMIT) as DWORD,
            0 as DWORD,
            0 as DWORD,
            core::ptr::null(),
        );
        if hFileMapping.is_null() {
            WARN!("CreateFileMapping: Error = %ld\n", GetLastError());
            break 'err;
        }

        mapping = MapViewOfFile(
            hFileMapping,
            (if bReadOnly != 0 {
                FILE_MAP_READ
            } else {
                FILE_MAP_WRITE
            }) as DWORD,
            0 as DWORD,
            0 as DWORD,
            0 as SIZE_T,
        );
        CloseHandle(hFileMapping);
        if mapping.is_null() {
            WARN!("MapViewOfFile: Error = %ld\n", GetLastError());
            break 'err;
        }

        pNtHeader = RtlImageNtHeader((mapping as HMODULE).cast());
        if pNtHeader.is_null() {
            WARN!("Not an NT header\n");
            UnmapViewOfFile(mapping as LPCVOID);
            break 'err;
        }

        (*pLoadedImage).ModuleName = HeapAlloc(
            GetProcessHeap(),
            0 as DWORD,
            (strlen(core::ptr::addr_of_mut!(szFileName) as *mut CHAR) as SIZE_T)
                .wrapping_add(1 as SIZE_T),
        ) as PSTR;
        if !(*pLoadedImage).ModuleName.is_null() {
            strcpy(
                (*pLoadedImage).ModuleName as *mut ::core::ffi::c_char,
                core::ptr::addr_of_mut!(szFileName) as *mut CHAR,
            );
        }
        (*pLoadedImage).hFile = hFile;
        (*pLoadedImage).MappedAddress = mapping as PUCHAR;
        (*pLoadedImage).FileHeader = pNtHeader;
        (*pLoadedImage).Sections = IMAGE_FIRST_SECTION(pNtHeader);
        (*pLoadedImage).NumberOfSections = ((*pNtHeader).FileHeader.NumberOfSections) as _;
        (*pLoadedImage).SizeOfImage = GetFileSize(hFile, core::ptr::null_mut()) as ULONG;
        (*pLoadedImage).Characteristics = ((*pNtHeader).FileHeader.Characteristics) as _;
        (*pLoadedImage).LastRvaSection = (*pLoadedImage).Sections;

        (*pLoadedImage).fSystemImage = FALSE as BOOLEAN; /* FIXME */
        (*pLoadedImage).fDOSImage = FALSE as BOOLEAN; /* FIXME */

        (*pLoadedImage).Links.Flink = core::ptr::addr_of_mut!((*pLoadedImage).Links) as *mut _LIST_ENTRY;
        (*pLoadedImage).Links.Blink = core::ptr::addr_of_mut!((*pLoadedImage).Links) as *mut _LIST_ENTRY;

        return (TRUE) as BOOL;
    }

    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetImageConfigInformation(
    mut LoadedImage: PLOADED_IMAGE,
    mut ImageConfigInformation: PIMAGE_LOAD_CONFIG_DIRECTORY,
) -> BOOL {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/access.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetImageConfigInformation\0" as *const u8 as *const ::core::ffi::c_char,
            219 as ::core::ffi::c_int,
            b"(%p, %p): stub\n\0" as *const u8 as *const ::core::ffi::c_char,
            LoadedImage,
            ImageConfigInformation,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UnMapAndLoad(pLoadedImage: PLOADED_IMAGE) -> BOOL {
    HeapFree(GetProcessHeap(), 0, (*pLoadedImage).ModuleName as PVOID);
    /* FIXME: MSDN states that a new checksum is computed and stored into the file */
    if !(*pLoadedImage).MappedAddress.is_null() { UnmapViewOfFile((*pLoadedImage).MappedAddress as LPCVOID); }
    if (*pLoadedImage).hFile != INVALID_HANDLE_VALUE { CloseHandle((*pLoadedImage).hFile); }
    return (TRUE) as BOOL;
}
