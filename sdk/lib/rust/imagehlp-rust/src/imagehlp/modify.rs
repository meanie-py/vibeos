use crate::imagehlp::access::MapAndLoad;
use crate::imagehlp::access::UnMapAndLoad;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WINE_DEFAULT_DEBUG_CHANNEL(imagehlp): Wine debug-channel macro, expands to a
// per-module `__wine_dbch_imagehlp` channel object and a default-channel pointer.
// Not translated here; the debug channel is provided by the crate's DPRINT/WARN/ERR macros.
//
// static WORD CalcCheckSum(DWORD StartValue, LPVOID BaseAddress, DWORD WordCount);
// Forward prototype for a file-local function; its definition is emitted with its body.

#[no_mangle]
pub unsafe extern "system" fn BindImage(
    ImageName: PCSTR,
    DllPath: PCSTR,
    SymbolPath: PCSTR,
) -> BOOL {
    return BindImageEx(
        0,
        ImageName as *const ::core::ffi::c_char,
        DllPath as *const ::core::ffi::c_char,
        SymbolPath as *const ::core::ffi::c_char,
        None,
    );
}

#[no_mangle]
pub unsafe extern "system" fn BindImageEx(
    flags: DWORD,
    module: *const ::core::ffi::c_char,
    dll_path: *const ::core::ffi::c_char,
    symbol_path: *const ::core::ffi::c_char,
    cb: PIMAGEHLP_STATUS_ROUTINE,
) -> BOOL {
    let mut import: *const IMAGE_IMPORT_DESCRIPTOR = core::ptr::null();
    let mut image: LOADED_IMAGE = core::mem::zeroed();
    let mut size: ULONG = 0;

    if (flags & BIND_NO_UPDATE) == 0 {
        FIXME!("Image modification is not implemented.\n");
    }
    if (flags & !BIND_NO_UPDATE) != 0 {
        FIXME!("Ignoring flags %#lx.\n", flags);
    }

    if MapAndLoad(
        module as PCSTR,
        dll_path as PCSTR,
        core::ptr::addr_of_mut!(image),
        (TRUE) as BOOL,
        (TRUE) as BOOL,
    ) == 0
    {
        return (FALSE) as BOOL;
    }

    import = ImageDirectoryEntryToData(
        image.MappedAddress as PVOID,
        (FALSE) as BOOLEAN,
        (IMAGE_DIRECTORY_ENTRY_IMPORT) as USHORT,
        core::ptr::addr_of_mut!(size),
    ) as *const IMAGE_IMPORT_DESCRIPTOR;
    if import.is_null() {
        UnMapAndLoad(core::ptr::addr_of_mut!(image));
        return (TRUE) as BOOL;
    }

    if ((*image.FileHeader).OptionalHeader.Magic as ::core::ffi::c_int) as u32 != IMAGE_NT_OPTIONAL_HDR_MAGIC {
        FIXME!(
            "Unhandled architecture %#x.\n",
            (*image.FileHeader).OptionalHeader.Magic as ::core::ffi::c_int
        );
        UnMapAndLoad(core::ptr::addr_of_mut!(image));
        return (TRUE) as BOOL;
    }

    loop {
        if !((*import).Name != 0 && (*import).FirstThunk != 0) {
            break;
        }
        'outer: {
            let mut full_path: [::core::ffi::c_char; (MAX_PATH) as _] =
                [0; (MAX_PATH) as _];
            let mut thunk: *mut IMAGE_THUNK_DATA = core::ptr::null_mut();
            let mut dll_name: *const ::core::ffi::c_char = core::ptr::null();
            let mut thunk_rva: DWORD = 0;

            dll_name = ImageRvaToVa(
                image.FileHeader as PIMAGE_NT_HEADERS,
                image.MappedAddress as PVOID,
                (*import).Name as ULONG,
                core::ptr::null_mut(),
            ) as *const ::core::ffi::c_char;
            if dll_name.is_null() {
                ERR!("Failed to get VA for import name RVA %#lx.\n", (*import).Name);
                break 'outer;
            }

            if cb.is_some() {
                (cb.unwrap_unchecked())(
                    BindImportModule,
                    module as PCSTR,
                    dll_name as PCSTR,
                    0,
                    0,
                );
            }

            if SearchPathA(
                dll_path as LPCSTR,
                dll_name as LPCSTR,
                core::ptr::null(),
                core::mem::size_of::<[::core::ffi::c_char; (MAX_PATH) as _]>() as DWORD,
                core::ptr::addr_of_mut!(full_path) as LPSTR,
                core::ptr::null_mut(),
            ) == 0
            {
                ERR!(
                    "Import %s was not found.\n",
                    debugstr_a(dll_path)
                );
                break 'outer;
            }

            thunk_rva = if (*import).__bindgen_anon_1.OriginalFirstThunk != 0 {
                (*import).__bindgen_anon_1.OriginalFirstThunk
            } else {
                (*import).FirstThunk
            };
            thunk = ImageRvaToVa(
                image.FileHeader as PIMAGE_NT_HEADERS,
                image.MappedAddress as PVOID,
                thunk_rva as ULONG,
                core::ptr::null_mut(),
            ) as *mut IMAGE_THUNK_DATA;
            if thunk.is_null() {
                ERR!("Failed to get VA for import thunk RVA %#lx.\n", thunk_rva);
                break 'outer;
            }

            loop {
                if (*thunk).u1.Ordinal == 0 {
                    break;
                }
                'thunk_body: {
                    if IMAGE_SNAP_BY_ORDINAL((*thunk).u1.Ordinal) {
                        FIXME!("Ordinal imports are not implemented.\n");
                    } else {
                        let mut name: *mut IMAGE_IMPORT_BY_NAME = core::ptr::null_mut();

                        name = ImageRvaToVa(
                            image.FileHeader as PIMAGE_NT_HEADERS,
                            image.MappedAddress as PVOID,
                            (*thunk).u1.AddressOfData as ULONG,
                            core::ptr::null_mut(),
                        ) as *mut IMAGE_IMPORT_BY_NAME;
                        if name.is_null() {
                            ERR!(
                                "Failed to get VA for name RVA %#Ix.\n",
                                (*thunk).u1.AddressOfData as usize
                            );
                            break 'thunk_body;
                        }

                        if cb.is_some() {
                            (cb.unwrap_unchecked())(
                                BindImportProcedure,
                                module as PCSTR,
                                core::ptr::addr_of_mut!(full_path) as PCSTR,
                                0,
                                core::ptr::addr_of_mut!((*name).Name) as *mut BYTE as ULONG_PTR,
                            );
                        }
                    }
                }
                thunk = thunk.offset(1);
            }
        }
        import = import.offset(1);
    }

    UnMapAndLoad(core::ptr::addr_of_mut!(image));
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CalcCheckSum(
    mut StartValue: DWORD,
    mut BaseAddress: LPVOID,
    mut WordCount: DWORD,
) -> WORD {
    let mut Ptr: LPWORD;
    let mut Sum: DWORD;
    let mut i: DWORD;

    Sum = StartValue;
    Ptr = BaseAddress as LPWORD;
    i = 0;
    while i < WordCount {
        Sum = Sum.wrapping_add(*Ptr as DWORD);
        if HIWORD!(Sum) != 0 {
            Sum = (LOWORD!(Sum) as DWORD).wrapping_add(HIWORD!(Sum) as DWORD);
        }
        Ptr = Ptr.add(1);
        i = i.wrapping_add(1);
    }

    return (LOWORD!(Sum) as DWORD).wrapping_add(HIWORD!(Sum) as DWORD) as WORD;
}

#[no_mangle]
pub unsafe extern "system" fn CheckSumMappedFile(
    BaseAddress: LPVOID,
    FileLength: DWORD,
    HeaderSum: LPDWORD,
    CheckSum: LPDWORD,
) -> PIMAGE_NT_HEADERS {
    let mut header: PIMAGE_NT_HEADERS;
    let mut CalcSum: DWORD;
    let mut HdrSum: DWORD;

    TRACE!("(%p, %ld, %p, %p)\n", BaseAddress, FileLength, HeaderSum, CheckSum);

    CalcSum = (CalcCheckSum(
        0 as DWORD,
        BaseAddress,
        (FileLength + 1 as DWORD) / (core::mem::size_of::<WORD>() as DWORD),
    )) as _;
    header = RtlImageNtHeader(BaseAddress);

    if header.is_null() {
        return core::ptr::null_mut();
    }

    HdrSum = (*header).OptionalHeader.CheckSum;
    *HeaderSum = HdrSum;

    /* Subtract image checksum from calculated checksum. */
    /* fix low word of checksum */
    if LOWORD!(CalcSum) >= LOWORD!(HdrSum) {
        CalcSum = CalcSum.wrapping_sub(LOWORD!(HdrSum) as DWORD);
    } else {
        CalcSum = ((((LOWORD!(CalcSum) as i32) - (LOWORD!(HdrSum) as i32))
            & (0xFFFF as i32))
            - (1 as i32)) as DWORD;
    }

    /* fix high word of checksum */
    if LOWORD!(CalcSum) >= HIWORD!(HdrSum) {
        CalcSum = CalcSum.wrapping_sub(HIWORD!(HdrSum) as DWORD);
    } else {
        CalcSum = ((((LOWORD!(CalcSum) as i32) - (HIWORD!(HdrSum) as i32))
            & (0xFFFF as i32))
            - (1 as i32)) as DWORD;
    }

    /* add file length */
    CalcSum = CalcSum.wrapping_add(FileLength);

    *CheckSum = CalcSum;

    return header;
}

#[no_mangle]
pub unsafe extern "system" fn MapFileAndCheckSumA(
    Filename: PCSTR,
    HeaderSum: PDWORD,
    CheckSum: PDWORD,
) -> DWORD {
    let mut hFile: HANDLE = core::ptr::null_mut();
    let mut hMapping: HANDLE = core::ptr::null_mut();
    let mut BaseAddress: LPVOID = core::ptr::null_mut();
    let mut FileLength: DWORD = 0;

    hFile = CreateFileA(Filename,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut());
    if hFile == INVALID_HANDLE_VALUE
    {
        return CHECKSUM_OPEN_FAILURE;
    }

    hMapping = CreateFileMappingW(hFile,
        core::ptr::null_mut(),
        PAGE_READONLY,
        0,
        0,
        core::ptr::null_mut());
    if hMapping == core::ptr::null_mut()
    {
        CloseHandle(hFile);
        return CHECKSUM_MAP_FAILURE;
    }

    BaseAddress = MapViewOfFile(hMapping,
        FILE_MAP_READ,
        0,
        0,
        0);
    if BaseAddress == core::ptr::null_mut()
    {
        CloseHandle(hMapping);
        CloseHandle(hFile);
        return CHECKSUM_MAPVIEW_FAILURE;
    }

    FileLength = GetFileSize(hFile,
        core::ptr::null_mut());

    CheckSumMappedFile(BaseAddress,
        FileLength,
        HeaderSum,
        CheckSum);

    UnmapViewOfFile(BaseAddress);
    CloseHandle(hMapping);
    CloseHandle(hFile);

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn MapFileAndCheckSumW(
    Filename: PCWSTR,
    HeaderSum: PDWORD,
    CheckSum: PDWORD,
) -> DWORD {
    let mut hFile: HANDLE;
    let mut hMapping: HANDLE;
    let mut BaseAddress: LPVOID;
    let mut FileLength: DWORD;

    hFile = CreateFileW(
        Filename,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        core::ptr::null_mut(),
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        core::ptr::null_mut(),
    );
    if hFile == INVALID_HANDLE_VALUE {
        return CHECKSUM_OPEN_FAILURE;
    }

    hMapping = CreateFileMappingW(
        hFile,
        core::ptr::null_mut(),
        PAGE_READONLY,
        0,
        0,
        core::ptr::null_mut(),
    );
    if hMapping == core::ptr::null_mut() {
        CloseHandle(hFile);
        return CHECKSUM_MAP_FAILURE;
    }

    BaseAddress = MapViewOfFile(
        hMapping,
        FILE_MAP_READ,
        0,
        0,
        0,
    );
    if BaseAddress == core::ptr::null_mut() {
        CloseHandle(hMapping);
        CloseHandle(hFile);
        return CHECKSUM_MAPVIEW_FAILURE;
    }

    FileLength = GetFileSize(
        hFile,
        core::ptr::null_mut(),
    );

    CheckSumMappedFile(
        BaseAddress,
        FileLength,
        HeaderSum,
        CheckSum,
    );

    UnmapViewOfFile(BaseAddress);
    CloseHandle(hMapping);
    CloseHandle(hFile);

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ReBaseImage(
    CurrentImageName: PCSTR,
    SymbolPath: PCSTR,
    fReBase: BOOL,
    fRebaseSysfileOk: BOOL,
    fGoingDown: BOOL,
    CheckImageSize: ULONG,
    OldImageSize: *mut ULONG,
    OldImageBase: *mut ULONG_PTR,
    NewImageSize: *mut ULONG,
    NewImageBase: *mut ULONG_PTR,
    TimeStamp: ULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RemovePrivateCvSymbolic(
    DebugData: PCHAR,
    NewDebugData: *mut PCHAR,
    NewDebugSize: *mut ULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn RemoveRelocations(ImageName: PCHAR) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
}

#[no_mangle]
pub unsafe extern "system" fn SplitSymbols(
    mut ImageName: PSTR,
    mut SymbolsPath: PCSTR,
    mut SymbolFilePath: PSTR,
    mut Flags: ULONG,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn UpdateDebugInfoFile(
    ImageFileName: PCSTR,
    SymbolPath: PCSTR,
    DebugFilePath: PSTR,
    NtHeaders: PIMAGE_NT_HEADERS32,
) -> i32 {
    FIXME!(
        "(%s, %s, %s, %p): stub\n",
        debugstr_a(ImageFileName as *const i8),
        debugstr_a(SymbolPath as *const i8),
        debugstr_a(DebugFilePath as *const i8),
        NtHeaders
    );
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE as i32);
}

#[no_mangle]
pub unsafe extern "system" fn UpdateDebugInfoFileEx(
    ImageFileName: PCSTR,
    SymbolPath: PCSTR,
    DebugFilePath: PSTR,
    NtHeaders: PIMAGE_NT_HEADERS32,
    OldChecksum: DWORD,
) -> BOOL {
    if (*__wine_dbch___default).flags as i32
        & (1 as i32) << __WINE_DBCL_FIXME as i32
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/imagehlp/modify.c\0"
                as *const u8 as *const i8,
            b"UpdateDebugInfoFileEx\0" as *const u8 as *const i8,
            424 as i32,
            b"(%s, %s, %s, %p, %ld): stub\n\0" as *const u8 as *const i8,
            debugstr_a(ImageFileName as *const i8),
            debugstr_a(SymbolPath as *const i8),
            debugstr_a(DebugFilePath as *const i8),
            NtHeaders,
            OldChecksum,
        );
    }
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    return (FALSE) as BOOL;
}
