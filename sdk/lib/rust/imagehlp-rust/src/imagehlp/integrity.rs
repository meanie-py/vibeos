use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const HDR_FAIL: i32 = -1;

pub const HDR_NT32: i32 = 0;

pub const HDR_NT64: i32 = 1;

unsafe extern "C" fn IMAGEHLP_GetNTHeaders(
    handle: HANDLE,
    pe_offset: *mut DWORD,
    nt32: *mut IMAGE_NT_HEADERS32,
    nt64: *mut IMAGE_NT_HEADERS64,
) -> i32 {
    let mut dos_hdr: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut count: DWORD = 0;
    let mut r: i32 = 0;

    if nt32.is_null() || nt64.is_null() {
        return HDR_FAIL;
    }

    /* read the DOS header */
    count = SetFilePointer(
        handle,
        0,
        core::ptr::null_mut(),
        FILE_BEGIN,
    );

    if count == INVALID_SET_FILE_POINTER {
        return HDR_FAIL;
    }

    count = 0;

    r = ReadFile(
        handle,
        core::ptr::addr_of_mut!(dos_hdr) as PVOID,
        core::mem::size_of::<IMAGE_DOS_HEADER>() as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );

    if r == 0 {
        return HDR_FAIL;
    }

    if count as usize != core::mem::size_of::<IMAGE_DOS_HEADER>() as usize {
        return HDR_FAIL;
    }

    /* verify magic number of 'MZ' */
    if dos_hdr.e_magic as i32 != (IMAGE_DOS_SIGNATURE as i32) {
        return HDR_FAIL;
    }

    if !pe_offset.is_null() {
        *pe_offset = dos_hdr.e_lfanew as DWORD;
    }

    /* read the PE header */
    count = SetFilePointer(
        handle,
        dos_hdr.e_lfanew,
        core::ptr::null_mut(),
        FILE_BEGIN,
    );

    if count == INVALID_SET_FILE_POINTER {
        return HDR_FAIL;
    }

    count = 0;

    r = ReadFile(
        handle,
        nt32 as PVOID,
        core::mem::size_of::<IMAGE_NT_HEADERS32>() as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );

    if r == 0 {
        return HDR_FAIL;
    }

    if count as usize != core::mem::size_of::<IMAGE_NT_HEADERS32>() as usize {
        return HDR_FAIL;
    }

    /* verify NT signature */
    if (*nt32).Signature != IMAGE_NT_SIGNATURE {
        return HDR_FAIL;
    }

    /* check if we have a 32-bit or 64-bit executable */
    match ((*nt32).OptionalHeader.Magic as i32) as u32 {
        IMAGE_NT_OPTIONAL_HDR32_MAGIC => {
            return HDR_NT32;
        }
        IMAGE_NT_OPTIONAL_HDR64_MAGIC => {
            /* Re-read as 64-bit */

            count = SetFilePointer(
                handle,
                dos_hdr.e_lfanew,
                core::ptr::null_mut(),
                FILE_BEGIN,
            );

            if count == INVALID_SET_FILE_POINTER {
                return HDR_FAIL;
            }

            count = 0;

            r = ReadFile(
                handle,
                nt64 as PVOID,
                core::mem::size_of::<IMAGE_NT_HEADERS64>() as DWORD,
                core::ptr::addr_of_mut!(count),
                core::ptr::null_mut(),
            );

            if r == 0 {
                return HDR_FAIL;
            }

            if count as usize != core::mem::size_of::<IMAGE_NT_HEADERS64>() as usize {
                return HDR_FAIL;
            }

            /* verify NT signature */
            if (*nt64).Signature != IMAGE_NT_SIGNATURE {
                return HDR_FAIL;
            }

            return HDR_NT64;
        }
        _ => {}
    }

    return HDR_FAIL;
}

unsafe extern "C" fn IMAGEHLP_GetSecurityDirOffset(
    mut handle: HANDLE,
    mut pdwOfs: *mut DWORD,
    mut pdwSize: *mut DWORD,
) -> BOOL {
    let mut nt_hdr32: IMAGE_NT_HEADERS32 = core::mem::zeroed();
    let mut nt_hdr64: IMAGE_NT_HEADERS64 = core::mem::zeroed();
    let mut sd: *mut IMAGE_DATA_DIRECTORY = core::ptr::null_mut::<IMAGE_DATA_DIRECTORY>();
    let mut ret: ::core::ffi::c_int = 0;

    ret = IMAGEHLP_GetNTHeaders(
        handle,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(nt_hdr32),
        core::ptr::addr_of_mut!(nt_hdr64),
    );

    if ret == HDR_NT32 {
        sd = (core::ptr::addr_of_mut!(nt_hdr32.OptionalHeader.DataDirectory)
            as *mut IMAGE_DATA_DIRECTORY)
            .offset(IMAGE_FILE_SECURITY_DIRECTORY as isize);
    } else if ret == HDR_NT64 {
        sd = (core::ptr::addr_of_mut!(nt_hdr64.OptionalHeader.DataDirectory)
            as *mut IMAGE_DATA_DIRECTORY)
            .offset(IMAGE_FILE_SECURITY_DIRECTORY as isize);
    } else {
        return (FALSE) as BOOL;
    }

    TRACE!(
        "ret = %d size = %lx addr = %lx\n",
        ret,
        (*sd).Size,
        (*sd).VirtualAddress
    );

    *pdwSize = (*sd).Size;
    *pdwOfs = (*sd).VirtualAddress;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn IMAGEHLP_SetSecurityDirOffset(
    mut handle: HANDLE,
    mut dwOfs: DWORD,
    mut dwSize: DWORD,
) -> BOOL {
    let mut nt_hdr32: IMAGE_NT_HEADERS32 = core::mem::zeroed();
    let mut nt_hdr64: IMAGE_NT_HEADERS64 = core::mem::zeroed();
    let mut sd: *mut IMAGE_DATA_DIRECTORY = core::ptr::null_mut::<IMAGE_DATA_DIRECTORY>();
    let mut ret: ::core::ffi::c_int = 0;
    let mut nt_hdr_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut pe_offset: DWORD = 0;
    let mut nt_hdr: *mut ::core::ffi::c_void = core::ptr::null_mut::<::core::ffi::c_void>();
    let mut count: DWORD = 0;
    let mut r: BOOL = 0;

    ret = IMAGEHLP_GetNTHeaders(
        handle,
        core::ptr::addr_of_mut!(pe_offset),
        core::ptr::addr_of_mut!(nt_hdr32),
        core::ptr::addr_of_mut!(nt_hdr64),
    );

    if ret == HDR_NT32 {
        sd = (core::ptr::addr_of_mut!(nt_hdr32.OptionalHeader.DataDirectory)
            as *mut IMAGE_DATA_DIRECTORY)
            .offset(IMAGE_FILE_SECURITY_DIRECTORY as isize);

        nt_hdr = core::ptr::addr_of_mut!(nt_hdr32) as *mut ::core::ffi::c_void;
        nt_hdr_size = core::mem::size_of::<IMAGE_NT_HEADERS32>() as ::core::ffi::c_int;
    } else if ret == HDR_NT64 {
        sd = (core::ptr::addr_of_mut!(nt_hdr64.OptionalHeader.DataDirectory)
            as *mut IMAGE_DATA_DIRECTORY)
            .offset(IMAGE_FILE_SECURITY_DIRECTORY as isize);

        nt_hdr = core::ptr::addr_of_mut!(nt_hdr64) as *mut ::core::ffi::c_void;
        nt_hdr_size = core::mem::size_of::<IMAGE_NT_HEADERS64>() as ::core::ffi::c_int;
    } else {
        return (FALSE) as BOOL;
    }

    (*sd).Size = dwSize;
    (*sd).VirtualAddress = dwOfs;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/integrity.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IMAGEHLP_SetSecurityDirOffset\0" as *const u8
                as *const ::core::ffi::c_char,
            212 as ::core::ffi::c_int,
            b"size = %lx addr = %lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            (*sd).Size,
            (*sd).VirtualAddress,
        );
    }

    /* write the header back again */
    count = SetFilePointer(
        handle,
        (pe_offset) as LONG,
        core::ptr::null_mut::<LONG>(),
        FILE_BEGIN,
    );

    if count == INVALID_SET_FILE_POINTER {
        return (FALSE) as BOOL;
    }

    count = 0;

    r = WriteFile(
        handle,
        nt_hdr as LPCVOID,
        nt_hdr_size as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut::<_OVERLAPPED>(),
    );

    if r == 0 {
        return (FALSE) as BOOL;
    }

    if count != nt_hdr_size as DWORD {
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

unsafe extern "C" fn IMAGEHLP_GetCertificateOffset(
    mut handle: HANDLE,
    mut num: DWORD,
    mut pdwOfs: *mut DWORD,
    mut pdwSize: *mut DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut len: DWORD = 0;
    let mut sd_VirtualAddr: DWORD = 0;
    let mut r: BOOL = 0;
    r = IMAGEHLP_GetSecurityDirOffset(handle, &raw mut sd_VirtualAddr, &raw mut size);
    if r == 0 {
        return (FALSE) as BOOL;
    }
    offset = 0 as DWORD;
    while 1 != 0 {
        count = SetFilePointer(
            handle,
            sd_VirtualAddr.wrapping_add(offset) as LONG,
            ::core::ptr::null_mut::<LONG>(),
            FILE_BEGIN as DWORD,
        );
        if count == INVALID_SET_FILE_POINTER {
            return (FALSE) as BOOL;
        }
        r = ReadFile(
            handle,
            &raw mut len as PVOID,
            ::core::mem::size_of::<DWORD>() as DWORD,
            &raw mut count,
            ::core::ptr::null_mut::<_OVERLAPPED>(),
        );
        if r == 0 {
            return (FALSE) as BOOL;
        }
        if count as usize != ::core::mem::size_of::<DWORD>() as usize {
            return (FALSE) as BOOL;
        }
        if (len as usize) < ::core::mem::size_of::<DWORD>() as usize {
            return (FALSE) as BOOL;
        }
        if len > size.wrapping_sub(offset) {
            return (FALSE) as BOOL;
        }
        let fresh15 = num;
        num = num.wrapping_sub(1);
        if fresh15 == 0 {
            break;
        }
        offset = offset.wrapping_add(len);
        if len % 8 as DWORD != 0 {
            offset = offset.wrapping_add(
                (8 as DWORD)
                    .wrapping_sub(len % 8 as DWORD),
            );
        }
        if offset >= size {
            return FALSE as BOOL;
        }
    }
    *pdwOfs = sd_VirtualAddr.wrapping_add(offset);
    *pdwSize = len;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/integrity.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"IMAGEHLP_GetCertificateOffset\0" as *const u8
                as *const ::core::ffi::c_char,
            286 as ::core::ffi::c_int,
            b"len = %lx addr = %lx\n\0" as *const u8 as *const ::core::ffi::c_char,
            len,
            sd_VirtualAddr.wrapping_add(offset),
        );
    }
    return TRUE as BOOL;
}

unsafe extern "C" fn IMAGEHLP_RecalculateChecksum(handle: HANDLE) -> BOOL {
    let mut FileLength: DWORD = 0;
    let mut count: DWORD = 0;
    let mut HeaderSum: DWORD = 0;
    let mut pe_offset: DWORD = 0;
    let mut nt_hdr_size: DWORD = 0;
    let mut nt_hdr32: IMAGE_NT_HEADERS32 = core::mem::zeroed();
    let mut nt_hdr64: IMAGE_NT_HEADERS64 = core::mem::zeroed();
    let mut BaseAddress: LPVOID = core::ptr::null_mut::<core::ffi::c_void>();
    let mut hMapping: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut CheckSum: *mut DWORD = core::ptr::null_mut::<DWORD>();
    let mut nt_hdr: *mut core::ffi::c_void = core::ptr::null_mut::<core::ffi::c_void>();
    let mut ret: i32 = 0;
    let mut r: BOOL = 0;

    ret = IMAGEHLP_GetNTHeaders(
        handle,
        core::ptr::addr_of_mut!(pe_offset),
        core::ptr::addr_of_mut!(nt_hdr32),
        core::ptr::addr_of_mut!(nt_hdr64),
    );

    if ret == HDR_NT32 {
        CheckSum = core::ptr::addr_of_mut!(nt_hdr32.OptionalHeader.CheckSum);

        nt_hdr = core::ptr::addr_of_mut!(nt_hdr32) as *mut core::ffi::c_void;
        nt_hdr_size = core::mem::size_of::<IMAGE_NT_HEADERS32>() as DWORD;
    } else if ret == HDR_NT64 {
        CheckSum = core::ptr::addr_of_mut!(nt_hdr64.OptionalHeader.CheckSum);

        nt_hdr = core::ptr::addr_of_mut!(nt_hdr64) as *mut core::ffi::c_void;
        nt_hdr_size = core::mem::size_of::<IMAGE_NT_HEADERS64>() as DWORD;
    } else {
        return (FALSE) as BOOL;
    }

    hMapping = CreateFileMappingW(
        handle,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        PAGE_READONLY as DWORD,
        0 as DWORD,
        0 as DWORD,
        core::ptr::null::<WCHAR>(),
    );

    if hMapping.is_null() {
        return (FALSE) as BOOL;
    }

    BaseAddress = MapViewOfFile(
        hMapping,
        FILE_MAP_READ as DWORD,
        0 as DWORD,
        0 as DWORD,
        0 as SIZE_T,
    ) as LPVOID;

    if BaseAddress.is_null() {
        CloseHandle(hMapping);
        return (FALSE) as BOOL;
    }

    FileLength = GetFileSize(handle, core::ptr::null_mut::<DWORD>());

    *CheckSum = 0 as DWORD;
    CheckSumMappedFile(
        BaseAddress,
        FileLength,
        core::ptr::addr_of_mut!(HeaderSum),
        CheckSum as LPDWORD,
    );

    UnmapViewOfFile(BaseAddress as LPCVOID);
    CloseHandle(hMapping);

    if *CheckSum != 0 {
        /* write the header back again */
        count = SetFilePointer(
            handle,
            pe_offset as LONG,
            core::ptr::null_mut::<LONG>(),
            FILE_BEGIN as DWORD,
        );

        if count == INVALID_SET_FILE_POINTER as DWORD {
            return (FALSE) as BOOL;
        }

        count = 0;

        r = WriteFile(
            handle,
            nt_hdr as LPCVOID,
            nt_hdr_size,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut::<_OVERLAPPED>(),
        );

        if r == 0 {
            return (FALSE) as BOOL;
        }

        if count != nt_hdr_size {
            return (FALSE) as BOOL;
        }

        return TRUE as BOOL;
    }

    return FALSE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImageAddCertificate(
    FileHandle: HANDLE,
    Certificate: LPWIN_CERTIFICATE,
    Index: PDWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut sd_VirtualAddr: DWORD = 0;
    let mut index: DWORD = 0;
    let mut hdr: WIN_CERTIFICATE = core::mem::zeroed();
    let cert_hdr_size: usize = core::mem::size_of::<WIN_CERTIFICATE>()
        - core::mem::size_of::<[BYTE; 1]>();
    let mut r: BOOL;

    r = IMAGEHLP_GetSecurityDirOffset(
        FileHandle,
        core::ptr::addr_of_mut!(sd_VirtualAddr),
        core::ptr::addr_of_mut!(size),
    );

    /* If we've already got a security directory, find the end of it */
    if r != 0 && sd_VirtualAddr != 0 {
        /* Check if the security directory is at the end of the file.
           If not, we should probably relocate it. */
        if GetFileSize(FileHandle, core::ptr::null_mut()) != sd_VirtualAddr.wrapping_add(size) {
            SetLastError(ERROR_NOT_SUPPORTED);
            return (FALSE) as BOOL;
        }

        while offset < size {
            /* read the length of the current certificate */
            count = SetFilePointer(
                FileHandle,
                sd_VirtualAddr.wrapping_add(offset) as LONG,
                core::ptr::null_mut::<LONG>(),
                FILE_BEGIN,
            );

            if count == INVALID_SET_FILE_POINTER {
                return (FALSE) as BOOL;
            }

            r = ReadFile(
                FileHandle,
                core::ptr::addr_of_mut!(hdr) as LPVOID,
                cert_hdr_size as DWORD,
                core::ptr::addr_of_mut!(count),
                core::ptr::null_mut(),
            );

            if r == 0 {
                return (FALSE) as BOOL;
            }

            if count as usize != cert_hdr_size {
                return (FALSE) as BOOL;
            }

            /* check the certificate is not too big or too small */
            if (hdr.dwLength as usize) < cert_hdr_size {
                return (FALSE) as BOOL;
            }

            if hdr.dwLength > size.wrapping_sub(offset) {
                return (FALSE) as BOOL;
            }

            /* next certificate */
            offset = offset.wrapping_add(hdr.dwLength);

            /* padded out to the nearest 8-byte boundary */
            if hdr.dwLength % 8 != 0 {
                offset = offset.wrapping_add(
                    8 - (hdr.dwLength % 8),
                );
            }

            index = index.wrapping_add(1);
        }

        count = SetFilePointer(
            FileHandle,
            sd_VirtualAddr.wrapping_add(offset) as LONG,
            core::ptr::null_mut::<LONG>(),
            FILE_BEGIN,
        );

        if count == INVALID_SET_FILE_POINTER {
            return FALSE as BOOL;
        }
    } else {
        sd_VirtualAddr = SetFilePointer(
            FileHandle,
            0 as LONG,
            core::ptr::null_mut::<LONG>(),
            FILE_END,
        );

        if sd_VirtualAddr == INVALID_SET_FILE_POINTER {
            return FALSE as BOOL;
        }
    }

    /* Write the certificate to the file */
    r = WriteFile(
        FileHandle,
        Certificate as LPCVOID,
        (*Certificate).dwLength,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
    );

    if r == 0 {
        return FALSE as BOOL;
    }

    /* Pad out if necessary */
    if (*Certificate).dwLength % 8 != 0 {
        let mut null: [core::ffi::c_char; 8] = [0; 8];

        ZeroMemory!(null.as_mut_ptr() as *mut core::ffi::c_void, 8 as usize);
        WriteFile(
            FileHandle,
            null.as_mut_ptr() as LPCVOID,
            8 - ((*Certificate).dwLength % 8),
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        );

        size = size.wrapping_add(
            8 - ((*Certificate).dwLength % 8),
        );
    }

    size = size.wrapping_add((*Certificate).dwLength);

    /* Update the security directory offset and size */
    if IMAGEHLP_SetSecurityDirOffset(FileHandle, sd_VirtualAddr, size) == 0 {
        return FALSE as BOOL;
    }

    if IMAGEHLP_RecalculateChecksum(FileHandle) == 0 {
        return FALSE as BOOL;
    }

    if !Index.is_null() {
        *Index = index;
    }
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImageEnumerateCertificates(
    mut handle: HANDLE,
    mut TypeFilter: WORD,
    mut CertificateCount: PDWORD,
    mut Indices: PDWORD,
    mut IndexCount: DWORD,
) -> BOOL {
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut sd_VirtualAddr: DWORD = 0;
    let mut index: DWORD = 0;
    let mut hdr: WIN_CERTIFICATE = core::mem::zeroed();
    let cert_hdr_size: size_t = (core::mem::size_of::<WIN_CERTIFICATE>() as size_t)
        .wrapping_sub(core::mem::size_of::<[BYTE; 1]>() as size_t);
    let mut r: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/integrity.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImageEnumerateCertificates\0" as *const u8 as *const ::core::ffi::c_char,
            497 as ::core::ffi::c_int,
            b"%p %hd %p %p %ld\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            TypeFilter as ::core::ffi::c_int,
            CertificateCount,
            Indices,
            IndexCount,
        );
    }

    r = IMAGEHLP_GetSecurityDirOffset(
        handle,
        core::ptr::addr_of_mut!(sd_VirtualAddr),
        core::ptr::addr_of_mut!(size),
    );
    if r == 0 {
        return (FALSE) as BOOL;
    }

    offset = 0;
    index = 0;
    *CertificateCount = 0;
    while offset < size {
        count = SetFilePointer(
            handle,
            sd_VirtualAddr.wrapping_add(offset) as LONG,
            core::ptr::null_mut(),
            FILE_BEGIN as DWORD,
        );
        if count == INVALID_SET_FILE_POINTER {
            return (FALSE) as BOOL;
        }
        r = ReadFile(
            handle,
            core::ptr::addr_of_mut!(hdr) as PVOID,
            cert_hdr_size as DWORD,
            core::ptr::addr_of_mut!(count),
            core::ptr::null_mut(),
        );
        if r == 0 {
            return (FALSE) as BOOL;
        }
        if count as size_t != cert_hdr_size {
            return (FALSE) as BOOL;
        }
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/imagehlp/integrity.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"ImageEnumerateCertificates\0" as *const u8 as *const ::core::ffi::c_char,
                520 as ::core::ffi::c_int,
                b"Size = %08lx  id = %08hx\n\0" as *const u8 as *const ::core::ffi::c_char,
                hdr.dwLength,
                hdr.wCertificateType as ::core::ffi::c_int,
            );
        }

        /* check the certificate is not too big or too small */
        if (hdr.dwLength as size_t) < cert_hdr_size {
            return (FALSE) as BOOL;
        }
        if hdr.dwLength > size.wrapping_sub(offset) {
            return (FALSE) as BOOL;
        }

        if TypeFilter as ::core::ffi::c_int == (CERT_SECTION_TYPE_ANY as i32)
            || TypeFilter as ::core::ffi::c_int == hdr.wCertificateType as ::core::ffi::c_int
        {
            *CertificateCount = (*CertificateCount).wrapping_add(1);
            if !Indices.is_null() && *CertificateCount <= IndexCount {
                let fresh = Indices;
                Indices = Indices.offset(1);
                *fresh = index;
            }
        }

        /* next certificate */
        offset = offset.wrapping_add(hdr.dwLength);

        /* padded out to the nearest 8-byte boundary */
        if hdr.dwLength.wrapping_rem(8 as DWORD) != 0 {
            offset = offset.wrapping_add(
                (8 as DWORD)
                    .wrapping_sub(hdr.dwLength.wrapping_rem(8 as DWORD)),
            );
        }

        index = index.wrapping_add(1);
    }

    return TRUE as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn ImageGetCertificateData(
                handle: HANDLE, Index: DWORD,
                Certificate: LPWIN_CERTIFICATE, RequiredLength: PDWORD) -> BOOL {
    let mut r: DWORD;
    let mut offset: DWORD;
    let mut ofs: DWORD = 0;
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;

    if RequiredLength.is_null()
    {
        SetLastError( ERROR_INVALID_PARAMETER );
        return (FALSE) as BOOL;
    }

    if IMAGEHLP_GetCertificateOffset( handle, Index, core::ptr::addr_of_mut!(ofs), core::ptr::addr_of_mut!(size) ) == 0
    {
        return (FALSE) as BOOL;
    }

    if *RequiredLength < size
    {
        *RequiredLength = size;
        SetLastError( ERROR_INSUFFICIENT_BUFFER );
        return (FALSE) as BOOL;
    }

    if Certificate.is_null()
    {
        SetLastError( ERROR_INVALID_PARAMETER );
        return (FALSE) as BOOL;
    }

    *RequiredLength = size;

    offset = SetFilePointer( handle, ofs as LONG, core::ptr::null_mut::<LONG>(), FILE_BEGIN );
    if offset == INVALID_SET_FILE_POINTER
    {
        return (FALSE) as BOOL;
    }

    r = ReadFile( handle, Certificate as LPVOID, size, core::ptr::addr_of_mut!(count), core::ptr::null_mut() ) as DWORD;
    if r == 0
    {
        return (FALSE) as BOOL;
    }
    if count != size
    {
        return FALSE as BOOL;
    }

    SetLastError( NO_ERROR );

    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ImageGetCertificateHeader(
    mut handle: HANDLE,
    mut index: DWORD,
    mut pCert: LPWIN_CERTIFICATE,
) -> BOOL {
    let mut r: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut ofs: DWORD = 0;
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;
    let cert_hdr_size: size_t = (core::mem::size_of::<WIN_CERTIFICATE>() as size_t)
        .wrapping_sub(core::mem::size_of::<[BYTE; 1]>() as size_t);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/integrity.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImageGetCertificateHeader\0" as *const u8 as *const ::core::ffi::c_char,
            611 as ::core::ffi::c_int,
            b"%p %ld %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            handle,
            index,
            pCert,
        );
    }
    if IMAGEHLP_GetCertificateOffset(
        handle,
        index,
        core::ptr::addr_of_mut!(ofs),
        core::ptr::addr_of_mut!(size),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    if (size as size_t) < cert_hdr_size {
        return (FALSE) as BOOL;
    }
    offset = SetFilePointer(
        handle,
        ofs as LONG,
        core::ptr::null_mut::<LONG>(),
        FILE_BEGIN as DWORD,
    );
    if offset == INVALID_SET_FILE_POINTER {
        return (FALSE) as BOOL;
    }
    r = ReadFile(
        handle,
        pCert as PVOID,
        cert_hdr_size as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut::<OVERLAPPED>(),
    ) as DWORD;
    if r == 0 {
        return (FALSE) as BOOL;
    }
    if count as size_t != cert_hdr_size {
        return (FALSE) as BOOL;
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/imagehlp/integrity.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"ImageGetCertificateHeader\0" as *const u8 as *const ::core::ffi::c_char,
            629 as ::core::ffi::c_int,
            b"OK\n\0" as *const u8 as *const ::core::ffi::c_char,
        );
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn IMAGEHLP_GetSectionOffset(
    mut hdr: *mut IMAGE_SECTION_HEADER,
    mut num_sections: DWORD,
    mut section: LPCSTR,
    mut size: PDWORD,
    mut base: PDWORD,
) -> DWORD {
    let mut i: DWORD = 0;
    let mut offset: DWORD = 0;
    while offset == 0 && i < num_sections {
        if memcmp(
            core::ptr::addr_of_mut!((*hdr).Name) as *mut BYTE as *const core::ffi::c_void,
            section as *const core::ffi::c_void,
            strlen(section as *const core::ffi::c_char),
        ) == 0
        {
            offset = (*hdr).PointerToRawData;
            if !size.is_null() {
                *size = (*hdr).SizeOfRawData;
            }
            if !base.is_null() {
                *base = (*hdr).VirtualAddress;
            }
        }
        i = i.wrapping_add(1);
        hdr = hdr.offset(1);
    }
    return offset;
}

unsafe extern "C" fn IMAGEHLP_ReportSectionFromOffset(
    mut offset: DWORD,
    mut size: DWORD,
    mut map: *mut BYTE,
    mut fileSize: DWORD,
    mut DigestFunction: DIGEST_FUNCTION,
    mut DigestHandle: DIGEST_HANDLE,
) -> BOOL {
    if offset.wrapping_add(size) > fileSize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }
    return (DigestFunction.unwrap_unchecked())(DigestHandle, map.offset(offset as isize), size);
}

unsafe extern "C" fn IMAGEHLP_ReportSection(section_headers: *mut IMAGE_SECTION_HEADER, num_sections: DWORD, section: LPCSTR, map: *mut BYTE, fileSize: DWORD, DigestFunction: DIGEST_FUNCTION, DigestHandle: DIGEST_HANDLE) -> BOOL {
    let mut offset: DWORD;
    let mut size: DWORD = 0;

    offset = IMAGEHLP_GetSectionOffset(
        section_headers,
        num_sections,
        section,
        core::ptr::addr_of_mut!(size),
        core::ptr::null_mut::<DWORD>(),
    );
    if offset == 0 {
        return (FALSE) as BOOL;
    }
    return IMAGEHLP_ReportSectionFromOffset(
        offset,
        size,
        map,
        fileSize,
        DigestFunction,
        DigestHandle,
    );
}

unsafe extern "C" fn IMAGEHLP_ReportCodeSections(
    mut hdr: *mut IMAGE_SECTION_HEADER,
    mut num_sections: DWORD,
    mut map: *mut BYTE,
    mut fileSize: DWORD,
    mut DigestFunction: DIGEST_FUNCTION,
    mut DigestHandle: DIGEST_HANDLE,
) -> BOOL {
    let mut i: DWORD;
    let mut ret: BOOL = (TRUE as i32);

    i = 0;
    while ret != 0 && i < num_sections {
        if (*hdr).Characteristics & (IMAGE_SCN_CNT_CODE) as DWORD != 0 {
            ret = IMAGEHLP_ReportSectionFromOffset(
                (*hdr).PointerToRawData,
                (*hdr).SizeOfRawData,
                map,
                fileSize,
                DigestFunction,
                DigestHandle,
            );
        }
        i = i.wrapping_add(1);
        hdr = hdr.add(1);
    }
    return ret;
}

unsafe extern "C" fn IMAGEHLP_ReportImportSection(
    hdr: *mut IMAGE_SECTION_HEADER,
    num_sections: DWORD,
    map: *mut BYTE,
    fileSize: DWORD,
    DigestLevel: DWORD,
    DigestFunction: DIGEST_FUNCTION,
    DigestHandle: DIGEST_HANDLE,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    let mut offset: DWORD;
    let mut size: DWORD = 0;
    let mut base: DWORD = 0;

    /* Get import data */
    offset = IMAGEHLP_GetSectionOffset(
        hdr,
        num_sections,
        b".idata\0".as_ptr() as LPCSTR,
        &raw mut size,
        &raw mut base,
    );
    if offset == 0 {
        return (FALSE) as BOOL;
    }

    /* If CERT_PE_IMAGE_DIGEST_ALL_IMPORT_INFO is set, the entire
     * section is reported.  Otherwise, the debug info section is
     * decoded and reported piecemeal.  See tests.  However, I haven't been
     * able to figure out how the native implementation decides which values
     * to report.  Either it's buggy or my understanding is flawed.
     */
    if (DigestLevel & (CERT_PE_IMAGE_DIGEST_ALL_IMPORT_INFO as DWORD)) != 0 {
        ret = IMAGEHLP_ReportSectionFromOffset(
            offset,
            size,
            map,
            fileSize,
            DigestFunction,
            DigestHandle,
        );
    } else {
        FIXME!("not supported except for CERT_PE_IMAGE_DIGEST_ALL_IMPORT_INFO\n");
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        ret = FALSE as BOOL;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImageGetDigestStream(
    FileHandle: HANDLE,
    DigestLevel: DWORD,
    DigestFunction: DIGEST_FUNCTION,
    DigestHandle: DIGEST_HANDLE,
) -> BOOL {
    let mut error: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);
    let mut offset: DWORD;
    let mut size: DWORD;
    let mut num_sections: DWORD;
    let mut fileSize: DWORD;
    let mut hMap: HANDLE = INVALID_HANDLE_VALUE;
    let mut map: *mut BYTE = core::ptr::null_mut();
    let mut dos_hdr: *mut IMAGE_DOS_HEADER;
    let mut nt_hdr: *mut IMAGE_NT_HEADERS;
    let mut section_headers: *mut IMAGE_SECTION_HEADER;

    TRACE!(
        "(%p, %ld, %p, %p)\n",
        FileHandle,
        DigestLevel,
        DigestFunction,
        DigestHandle
    );

    'body: {
        /* Get the file size */
        if FileHandle.is_null() {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        fileSize = GetFileSize(FileHandle, core::ptr::null_mut());
        if fileSize == INVALID_FILE_SIZE {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }

        /* map file */
        hMap = CreateFileMappingW(
            FileHandle,
            core::ptr::null_mut(),
            PAGE_READONLY,
            0,
            0,
            core::ptr::null_mut(),
        );
        if hMap == INVALID_HANDLE_VALUE {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        map = MapViewOfFile(
            hMap,
            FILE_MAP_COPY,
            0,
            0,
            0,
        ) as *mut BYTE;
        if map.is_null() {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }

        /* Read the file header */
        if (fileSize as usize) < core::mem::size_of::<IMAGE_DOS_HEADER>() {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        dos_hdr = map as *mut IMAGE_DOS_HEADER;

        if (*dos_hdr).e_magic != (IMAGE_DOS_SIGNATURE as u16) {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        offset = (*dos_hdr).e_lfanew as DWORD;
        if offset == 0 || offset > fileSize {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        ret = (DigestFunction.unwrap_unchecked())(DigestHandle, map, offset);
        if ret == 0 {
            break 'body;
        }

        /* Read the NT header */
        if (offset as usize) + core::mem::size_of::<IMAGE_NT_HEADERS>() > fileSize as usize {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        nt_hdr = map.add(offset as usize) as *mut IMAGE_NT_HEADERS;
        if (*nt_hdr).Signature != IMAGE_NT_SIGNATURE {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        /* It's clear why the checksum is cleared, but why only these size headers?
         */
        (*nt_hdr).OptionalHeader.SizeOfInitializedData = 0;
        (*nt_hdr).OptionalHeader.SizeOfImage = 0;
        (*nt_hdr).OptionalHeader.CheckSum = 0;
        size = (core::mem::size_of::<DWORD>()
            + core::mem::size_of::<IMAGE_FILE_HEADER>()
            + (*nt_hdr).FileHeader.SizeOfOptionalHeader as usize) as DWORD;
        ret = (DigestFunction.unwrap_unchecked())(DigestHandle, map.add(offset as usize), size);
        if ret == 0 {
            break 'body;
        }

        /* Read the section headers */
        offset += size;
        num_sections = (*nt_hdr).FileHeader.NumberOfSections as DWORD;
        size = (num_sections as usize * core::mem::size_of::<IMAGE_SECTION_HEADER>()) as DWORD;
        if (offset as usize) + (size as usize) > fileSize as usize {
            error = ERROR_INVALID_PARAMETER;
            break 'body;
        }
        ret = (DigestFunction.unwrap_unchecked())(DigestHandle, map.add(offset as usize), size);
        if ret == 0 {
            break 'body;
        }

        section_headers = map.add(offset as usize) as *mut IMAGE_SECTION_HEADER;
        IMAGEHLP_ReportCodeSections(
            section_headers,
            num_sections,
            map,
            fileSize,
            DigestFunction,
            DigestHandle,
        );
        IMAGEHLP_ReportSection(
            section_headers,
            num_sections,
            b".data\0".as_ptr() as LPCSTR,
            map,
            fileSize,
            DigestFunction,
            DigestHandle,
        );
        IMAGEHLP_ReportSection(
            section_headers,
            num_sections,
            b".rdata\0".as_ptr() as LPCSTR,
            map,
            fileSize,
            DigestFunction,
            DigestHandle,
        );
        IMAGEHLP_ReportImportSection(
            section_headers,
            num_sections,
            map,
            fileSize,
            DigestLevel,
            DigestFunction,
            DigestHandle,
        );
        if (DigestLevel & CERT_PE_IMAGE_DIGEST_DEBUG_INFO) != 0 {
            IMAGEHLP_ReportSection(
                section_headers,
                num_sections,
                b".debug\0".as_ptr() as LPCSTR,
                map,
                fileSize,
                DigestFunction,
                DigestHandle,
            );
        }
        if (DigestLevel & CERT_PE_IMAGE_DIGEST_RESOURCES) != 0 {
            IMAGEHLP_ReportSection(
                section_headers,
                num_sections,
                b".rsrc\0".as_ptr() as LPCSTR,
                map,
                fileSize,
                DigestFunction,
                DigestHandle,
            );
        }
    }

    // end:
    if !map.is_null() {
        UnmapViewOfFile(map as LPCVOID);
    }
    if hMap != INVALID_HANDLE_VALUE {
        CloseHandle(hMap);
    }
    if error != 0 {
        SetLastError(error);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn ImageRemoveCertificate(FileHandle: HANDLE, Index: DWORD) -> BOOL {
    let mut size: DWORD = 0;
    let mut count: DWORD = 0;
    let mut sd_VirtualAddr: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut data_size: DWORD = 0;
    let mut cert_size: DWORD = 0;
    let mut cert_size_padded: DWORD = 0;
    let mut ret: DWORD = 0;
    let mut cert_data: LPVOID = core::ptr::null_mut();
    let mut r: BOOL;

    r = ImageEnumerateCertificates(
        FileHandle,
        (CERT_SECTION_TYPE_ANY) as WORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::null_mut(),
        0,
    );

    if r == 0 || count == 0 {
        return (FALSE) as BOOL;
    }

    if IMAGEHLP_GetSecurityDirOffset(
        FileHandle,
        core::ptr::addr_of_mut!(sd_VirtualAddr),
        core::ptr::addr_of_mut!(size),
    ) == 0
        || IMAGEHLP_GetCertificateOffset(
            FileHandle,
            Index,
            core::ptr::addr_of_mut!(offset),
            core::ptr::addr_of_mut!(cert_size),
        ) == 0
    {
        return (FALSE) as BOOL;
    }

    /* Ignore any padding we have, too */
    if cert_size % 8 != 0 {
        cert_size_padded = cert_size.wrapping_add(
            (8 as DWORD).wrapping_sub(cert_size % 8),
        );
    } else {
        cert_size_padded = cert_size;
    }

    data_size = size
        .wrapping_sub(offset.wrapping_sub(sd_VirtualAddr))
        .wrapping_sub(cert_size_padded);

    if data_size == 0 {
        ret = SetFilePointer(
            FileHandle,
            (sd_VirtualAddr) as LONG,
            core::ptr::null_mut(),
            FILE_BEGIN,
        );

        if ret == INVALID_SET_FILE_POINTER {
            return (FALSE) as BOOL;
        }
    } else {
        cert_data = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            data_size as SIZE_T,
        ) as LPVOID;

        if cert_data.is_null() {
            return (FALSE) as BOOL;
        }

        'error: {
            ret = SetFilePointer(
                FileHandle,
                (offset.wrapping_add(cert_size_padded)) as LONG,
                core::ptr::null_mut(),
                FILE_BEGIN,
            );

            if ret == INVALID_SET_FILE_POINTER {
                break 'error;
            }

            /* Read any subsequent certificates */
            r = ReadFile(
                FileHandle,
                cert_data,
                data_size,
                core::ptr::addr_of_mut!(count),
                core::ptr::null_mut(),
            );

            if r == 0 || count != data_size {
                break 'error;
            }

            SetFilePointer(
                FileHandle,
                (offset) as LONG,
                core::ptr::null_mut(),
                FILE_BEGIN,
            );

            /* Write them one index back */
            r = WriteFile(
                FileHandle,
                cert_data,
                data_size,
                core::ptr::addr_of_mut!(count),
                core::ptr::null_mut(),
            );

            if r == 0 || count != data_size {
                break 'error;
            }

            HeapFree(
                GetProcessHeap(),
                0,
                cert_data,
            );
        }

        /* Reached only when the success path above completed; error exits jump to the tail below. */
        if GetFileSize(FileHandle, core::ptr::null_mut())
            == sd_VirtualAddr.wrapping_add(size)
        {
            SetEndOfFile(FileHandle);
        }

        if count == 1 {
            r = IMAGEHLP_SetSecurityDirOffset(FileHandle, 0, 0);
        } else {
            r = IMAGEHLP_SetSecurityDirOffset(
                FileHandle,
                sd_VirtualAddr,
                size.wrapping_sub(cert_size_padded),
            );
        }

        if r == 0 {
            return (FALSE) as BOOL;
        }

        if IMAGEHLP_RecalculateChecksum(FileHandle) == 0 {
            return (FALSE) as BOOL;
        }

        return TRUE as BOOL;
    }

    /* error: */
    HeapFree(GetProcessHeap(), 0, cert_data);
    return FALSE as BOOL;
}
