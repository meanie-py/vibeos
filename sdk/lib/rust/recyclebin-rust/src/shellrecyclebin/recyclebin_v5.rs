// rustconv-assembly: whole-tu
//! ORPHAN-BY-DESIGN: superseded by the class-atomic modules (RecycleBinGeneric.rs,
//! RecycleBinGenericEnum.rs, RecycleBin5.rs, RecycleBin5File.rs, RecycleBin5Enum.rs).
//! This is the older `whole-tu` assembly, produced before `--cpp-com` could run for a
//! non-shell32 target: it has no vtable, no `_new`/`_delete` and never stores `lpVtbl`,
//! so an object built from it null-derefs on its first method call. Kept only until its
//! removal is approved; the paid bodies live in .responses/ and regenerate for $0.
use crate::prelude::*;
use crate::shellrecyclebin::recyclebin_support::{
    RECYCLE_BIN_DIRECTORY_WITHOUT_ACL, RECYCLE_BIN_DIRECTORY_WITH_ACL,
    RECYCLE_BIN_FILE_NAME, WC_BACKSLASH, W_BACKSLASH, W_DESKTOP_INI, W_STAR,
};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct IRecycleBin5 {
    pub lpVtbl: *const core::ffi::c_void,
}

#[repr(C)]
pub struct CStringW(pub *mut u16);

#[repr(C)]
pub struct CZZWStr {
    pub m_sz: *mut u16,
}

#[repr(C)]
pub struct RecycleBin5 {
    pub base: IRecycleBin5,
    pub m_ref: i32,
    pub m_hInfo: *mut core::ffi::c_void,
    pub m_hInfoMapped: *mut core::ffi::c_void,
    pub m_EnumeratorCount: u32,
    pub m_VolumePath: CStringW,
    pub m_Folder: CStringW,
}
unsafe extern "C" fn SHELL_SingleFileOperation(
    hWnd: HWND,
    Op: UINT,
    pszFrom: LPCWSTR,
    pszTo: LPCWSTR,
    Flags: FILEOP_FLAGS,
) -> i32 {
    let mut szzFrom = CZZWStr::new();
    let mut szzTo = CZZWStr::new();
    if !szzFrom.Initialize(pszFrom) || !szzTo.Initialize(pszTo) {
        return (ERROR_OUTOFMEMORY) as i32;
    }
    let mut fos = SHFILEOPSTRUCTW {
        hwnd: hWnd,
        wFunc: Op,
        pFrom: szzFrom.c_str(),
        pTo: szzTo.c_str(),
        fFlags: Flags,
        fAnyOperationsAborted: 0,
        hNameMappings: core::ptr::null_mut(),
        lpszProgressTitle: core::ptr::null(),
    };
    SHFileOperationW(&mut fos)
}
unsafe extern "C" fn IntDeleteRecursive(FullName: LPCWSTR) -> BOOL {
    let RemovableAttributes: DWORD = FILE_ATTRIBUTE_READONLY;
    let mut FindData: WIN32_FIND_DATAW = core::mem::zeroed();
    let mut hSearch: HANDLE = INVALID_HANDLE_VALUE;
    let mut FullPath: LPWSTR = core::ptr::null_mut();
    let mut pFilePart: LPWSTR;
    let FileAttributes: DWORD;
    let dwLength: SIZE_T;
    let mut ret: BOOL = (FALSE) as i32;

    'cleanup: {
        FileAttributes = GetFileAttributesW(FullName);
        if FileAttributes == INVALID_FILE_ATTRIBUTES {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                ret = (TRUE) as i32;
            }
            break 'cleanup;
        }
        if (FileAttributes & RemovableAttributes) != 0 {
            if SetFileAttributesW(FullName, FileAttributes & !RemovableAttributes) == 0 {
                break 'cleanup;
            }
        }
        if (FileAttributes & FILE_ATTRIBUTE_DIRECTORY) == 0 {
            if DeleteFileW(FullName) == 0 {
                break 'cleanup;
            }
        } else {
            dwLength = wcslen(FullName) as SIZE_T;
            FullPath = HeapAlloc(
                GetProcessHeap(),
                0,
                ((dwLength as usize + 1 as usize + MAX_PATH as usize + 1 as usize) * core::mem::size_of::<u16>()) as SIZE_T,
            ) as LPWSTR;
            if FullPath.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            wcscpy(FullPath, FullName);
            if FullPath.offset((dwLength as usize - 1 as usize) as isize).read() != 0x5Cu16 {
                FullPath.offset(dwLength as isize).write(0x5Cu16);
                dwLength += 1;
            }
            pFilePart = FullPath.offset(dwLength as isize);
            wcscpy(pFilePart, W_STAR.as_ptr());
            hSearch = FindFirstFileW(FullPath, core::ptr::addr_of_mut!(FindData));
            if hSearch == INVALID_HANDLE_VALUE {
                break 'cleanup;
            }
            loop {
                if !(FindData.cFileName[0] == 0x002Eu16
                    && (FindData.cFileName[1] == 0u16
                        || (FindData.cFileName[1] == 0x002Eu16 && FindData.cFileName[2] == 0u16)))
                {
                    wcscpy(pFilePart, FindData.cFileName.as_ptr());
                    if IntDeleteRecursive(FullPath) == 0 {
                        FindClose(hSearch);
                        break 'cleanup;
                    }
                }
                if FindNextFileW(hSearch, core::ptr::addr_of_mut!(FindData)) == 0 {
                    break;
                }
            }
            FindClose(hSearch);
            if GetLastError() != ERROR_NO_MORE_FILES {
                break 'cleanup;
            }
            if RemoveDirectoryW(FullName) == 0 {
                break 'cleanup;
            }
        }
        ret = (TRUE) as i32;
    }

    HeapFree(
        GetProcessHeap(),
        0,
        (FullPath).cast(),
    );

    ret
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_GetDirectory(
    this: *mut RecycleBin5,
    szPath: LPWSTR,
) -> HRESULT {
    let folder = &(*this).m_Folder;
    let str_ptr: LPWSTR = *(folder as *const CStringW as *const LPWSTR);
    if *str_ptr.offset(0) == 0 {
        return E_UNEXPECTED;
    }
    lstrcpynW(szPath, str_ptr, (MAX_PATH) as i32);
    S_OK
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_QueryInterface(
    this: *mut RecycleBin5,
    riid: *const _GUID,
    ppvObject: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppvObject.is_null() {
        return E_POINTER;
    }

    if IsEqualIID(riid, &IID_IUnknown) != 0 || IsEqualIID(riid, &IID_IRecycleBin) != 0 {
        *ppvObject = this as *mut core::ffi::c_void;
    } else if IsEqualIID(riid, &IID_IRecycleBin5) != 0 {
        *ppvObject = this as *mut core::ffi::c_void;
    } else {
        *ppvObject = core::ptr::null_mut();
        return E_NOINTERFACE;
    }

    AddRef(this);
    S_OK
}

#[no_mangle] pub unsafe extern "C" fn RecycleBin5_AddRef(this: *mut RecycleBin5) -> u32 {
    let result = InterlockedIncrement(core::ptr::addr_of_mut!((*this).m_ref));
    result as u32
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5_dtor(this: *mut RecycleBin5) {
    if !(*this).m_hInfo.is_null() && (*this).m_hInfo != INVALID_HANDLE_VALUE {
        CloseHandle((*this).m_hInfo);
    }
    if !(*this).m_hInfoMapped.is_null() {
        CloseHandle((*this).m_hInfoMapped);
    }
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_Release(this: *mut RecycleBin5) -> u32 {
    let refCount = InterlockedDecrement(core::ptr::addr_of_mut!((*this).m_ref));
    if refCount == 0 {
        RecycleBin5_destructor(this);
    }
    (refCount) as u32
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_DeleteFile(
    this: *mut RecycleBin5,
    szFileName: LPCWSTR,
) -> HRESULT {
    let mut szFullName: LPWSTR = core::ptr::null_mut();
    let mut dwBufferLength: DWORD = 0;
    let mut lpFilePart: LPWSTR = core::ptr::null_mut();
    let mut Extension: LPCWSTR;
    let mut DeletedFileName: CStringW = CStringW::new();
    let mut szUniqueId: [WCHAR; 64 as usize] =
        [0u16; 64 as usize];
    let mut len: DWORD;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut pHeader: *mut INFO2_HEADER = core::ptr::null_mut();
    let mut pDeletedFile: *mut DELETED_FILE_RECORD;
    let mut FileSize: ULARGE_INTEGER = ULARGE_INTEGER { u: { core::mem::zeroed() } };
    let mut dwAttributes: DWORD;
    let mut dwEntries: DWORD;
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed();
    let mut ClusterSize: DWORD;
    let mut BytesPerSector: DWORD;
    let mut SectorsPerCluster: DWORD;
    let mut hr: HRESULT;
    let mut wfd: WIN32_FIND_DATAW = core::mem::zeroed();

    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32((ERROR_SHARING_VIOLATION) as LONG);
    }

    // Get full file name
    loop {
        len = GetFullPathNameW(
            szFileName,
            dwBufferLength,
            szFullName,
            core::ptr::addr_of_mut!(lpFilePart),
        );
        if len == 0 {
            if !szFullName.is_null() {
                CoTaskMemFree((szFullName).cast());
            }
            return HResultFromWin32(GetLastError());
        } else if len < dwBufferLength {
            break;
        }
        if !szFullName.is_null() {
            CoTaskMemFree((szFullName).cast());
        }
        dwBufferLength = len;
        szFullName = CoTaskMemAlloc((dwBufferLength as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
        if szFullName.is_null() {
            return HRESULT_FROM_WIN32((ERROR_NOT_ENOUGH_MEMORY) as LONG);
        }
    }

    // Check if file exists
    dwAttributes = GetFileAttributesW(szFullName);
    if dwAttributes == INVALID_FILE_ATTRIBUTES {
        CoTaskMemFree((szFullName).cast());
        return HResultFromWin32(GetLastError());
    }

    if dwBufferLength < 2 || *szFullName.add(1 as usize) != 0x003A
    {
        // Not a local file
        CoTaskMemFree((szFullName).cast());
        return HRESULT_FROM_WIN32((ERROR_INVALID_NAME) as LONG);
    }

    // C `goto cleanup` x9 -> a labeled block, the tree's lowering for a single-exit C
    // function (shell32-rust/src/vista.rs:151 does the same for the same idiom). Every
    // `hr = ...` stays exactly where the C has it and the block yields unit, so this is a
    // 1:1 control-flow mapping and not a rewrite: the cleanup below still runs on all 10
    // paths, which is the whole point of the C idiom (UnmapViewOfFile / CloseHandle /
    // CoTaskMemFree). The early returns ABOVE this block predate hFile and pHeader and do
    // their own CoTaskMemFree, exactly as in the C.
    'cleanup: {
        hFile = CreateFileW(
            szFullName,
            0,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            if (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) != 0 {
                FILE_FLAG_BACKUP_SEMANTICS
            } else {
                0
            },
            core::ptr::null_mut(),
        );
        if hFile == INVALID_HANDLE_VALUE {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }

        // Increase INFO2 file size
        CloseHandle((*this).m_hInfoMapped);
        SetFilePointer(
            (*this).m_hInfo,
            core::mem::size_of::<DELETED_FILE_RECORD>() as LONG,
            core::ptr::null_mut(),
            FILE_END,
        );
        SetEndOfFile((*this).m_hInfo);
        (*this).m_hInfoMapped = CreateFileMappingW(
            (*this).m_hInfo,
            core::ptr::null_mut(),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            0,
            core::ptr::null_mut(),
        );
        if (*this).m_hInfoMapped.is_null() {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }

        // Open INFO2 file
        pHeader = MapViewOfFile(
            (*this).m_hInfoMapped,
            FILE_MAP_WRITE,
            0,
            0,
            0,
        ) as *mut INFO2_HEADER;
        if pHeader.is_null() {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }

        // Get number of entries
        {
            let mut high_part: DWORD = 0;
            FileSize.u.__bindgen_anon_1.LowPart = GetFileSize((*this).m_hInfo, &mut high_part);
            FileSize.u.__bindgen_anon_1.HighPart = high_part;
        }
        if FileSize.u.__bindgen_anon_1.LowPart < (core::mem::size_of::<INFO2_HEADER>() as DWORD) {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }
        dwEntries = ((FileSize.QuadPart - core::mem::size_of::<INFO2_HEADER>() as u64)
            / core::mem::size_of::<DELETED_FILE_RECORD>() as u64) as DWORD
            - 1;
        pDeletedFile =
            // C: ((PDELETED_FILE_RECORD)(pHeader + 1)) + dwEntries -- the records follow the
            // whole INFO2_HEADER, as DeleteFile's walk (below, size_of) already has it.
            (pHeader.add(1) as *mut DELETED_FILE_RECORD).add(dwEntries as usize);

        // Get file size
        {
            let mut high_part: DWORD = 0;
            FileSize.u.__bindgen_anon_1.LowPart = GetFileSize(hFile, &mut high_part);
            FileSize.u.__bindgen_anon_1.HighPart = high_part;
        }
        if FileSize.u.__bindgen_anon_1.LowPart == INVALID_FILE_SIZE && GetLastError() != NO_ERROR {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }
        // Check if file size is > 4Gb
        if FileSize.u.__bindgen_anon_1.HighPart != 0 {
            hr = HRESULT_FROM_WIN32((ERROR_NOT_SUPPORTED) as LONG);
            break 'cleanup;
        }
        (*pHeader).dwTotalLogicalSize += FileSize.u.__bindgen_anon_1.LowPart;

        // Generate new name
        Extension = PathFindExtensionW(szFullName);
        ZeroMemory!(
            pDeletedFile as *mut c_void,
            core::mem::size_of::<DELETED_FILE_RECORD>(),
        );
        if dwEntries == 0 {
            (*pDeletedFile).dwRecordUniqueId = 0;
        } else {
            let pLastDeleted =
                (pHeader.add(1) as *mut DELETED_FILE_RECORD).add(dwEntries as usize - 1);
            (*pDeletedFile).dwRecordUniqueId = (*pLastDeleted).dwRecordUniqueId + 1;
        }

        (*pDeletedFile).dwDriveNumber =
            tolower((*szFullName.add(0 as usize)) as i32) as DWORD
                - 0x0061u16 as DWORD;
        _ultow(
            (*pDeletedFile).dwRecordUniqueId,
            szUniqueId.as_mut_ptr(),
            10,
        );

        DeletedFileName.AssignStr(&(*this).m_Folder);
        {
            let w_d: [WCHAR; 3] = [0x005C, 0x0044, 0]; // L"\\D"
            DeletedFileName.Append(w_d.as_ptr());
        }
        DeletedFileName.AppendChar((0x0061u16 + (*pDeletedFile).dwDriveNumber as u16) as WCHAR);
        DeletedFileName.Append(szUniqueId.as_ptr());
        DeletedFileName.Append(Extension);

        // Get cluster size
        if GetDiskFreeSpaceW(
            (*this).m_VolumePath.as_ptr(),
            &mut SectorsPerCluster,
            &mut BytesPerSector,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }
        ClusterSize = BytesPerSector * SectorsPerCluster;

        // Get current time
        GetSystemTime(&mut SystemTime);
        if SystemTimeToFileTime(&SystemTime, &mut (*pDeletedFile).DeletionTime) == 0 {
            hr = HResultFromWin32(GetLastError());
            break 'cleanup;
        }
        (*pDeletedFile).dwPhysicalFileSize = ROUND_UP(FileSize.u.__bindgen_anon_1.LowPart, ClusterSize);

        // Set name
        wcscpy((*pDeletedFile).FileNameW.as_mut_ptr(), szFullName);
        if WideCharToMultiByte(
            CP_ACP,
            0,
            (*pDeletedFile).FileNameW.as_ptr(),
            -(1 as i32),
            (*pDeletedFile).FileNameA.as_mut_ptr(),
            (MAX_PATH) as i32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32((ERROR_INVALID_NAME) as LONG);
            SetLastError(ERROR_INVALID_NAME);
            break 'cleanup;
        }

        wfd.dwFileAttributes = dwAttributes;
        wfd.nFileSizeLow = FileSize.u.__bindgen_anon_1.LowPart;
        GetFileTime(
            hFile,
            &mut wfd.ftCreationTime,
            &mut wfd.ftLastAccessTime,
            &mut wfd.ftLastWriteTime,
        );

        // Move file
        if MoveFileW(szFullName, DeletedFileName.as_ptr()) != 0 {
            hr = S_OK;
        } else {
            hr = HResultFromWin32(GetLastError());
        }

        if hr >= 0 {
            let ident = RECYCLEBINFILEIDENTITY {
                DeletionTime: (*pDeletedFile).DeletionTime,
                FileName: DeletedFileName.as_ptr(),
            };
            CRecycleBin_NotifyRecycled(szFullName, &wfd, &ident);
        }
    }

    // cleanup:
    if !pHeader.is_null() {
        UnmapViewOfFile(pHeader);
    }
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }
    CoTaskMemFree((szFullName).cast());
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_EmptyRecycleBin(this: *mut RecycleBin5) -> HRESULT {
    loop {
        let mut prbel: *mut IRecycleBinEnumList = core::ptr::null_mut();
        let hr = EnumObjects(this, &mut prbel);
        if !(hr >= 0) {
            return hr;
        }

        let mut prbf: *mut IRecycleBinFile = core::ptr::null_mut();
        let next = (*(*prbel).lpVtbl).Next;
        let hr = next(prbel, 1, &mut prbf, core::ptr::null_mut());

        let release_enum = (*(*prbel).lpVtbl).Release;
        release_enum(prbel);

        if hr == S_FALSE {
            return S_OK;
        }

        let delete = (*(*prbf).lpVtbl).Delete;
        let hr = delete(prbf);

        let release_file = (*(*prbf).lpVtbl).Release;
        release_file(prbf);

        if !(hr >= 0) {
            return hr;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_EnumObjects(
    this: *mut RecycleBin5,
    ppEnumList: *mut *mut IRecycleBinEnumList,
) -> HRESULT {
    let mut pUnk: *mut c_void = core::ptr::null_mut();
    let hr = RecycleBin5Enum_Constructor(
        this,
        (*this).m_hInfo,
        (*this).m_hInfoMapped,
        (*this).m_Folder,
        &mut pUnk,
    );
    if !(hr >= 0) {
        return hr;
    }

    let pUnk_unknown: *mut IUnknown = pUnk as *mut IUnknown;
    let mut prbel: *mut IRecycleBinEnumList = core::ptr::null_mut();
    let hr2 = ((*(*pUnk_unknown).lpVtbl).QueryInterface)(
        pUnk_unknown,
        &IID_IRecycleBinEnumList as *const GUID,
        &mut prbel as *mut *mut IRecycleBinEnumList as *mut *mut c_void,
    );
    if hr2 >= 0 {
        (*this).m_EnumeratorCount += 1;
        *ppEnumList = prbel;
    }

    ((*(*pUnk_unknown).lpVtbl).Release)(pUnk_unknown);

    hr2
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_Delete(
    this: *mut RecycleBin5,
    pDeletedFileName: LPCWSTR,
    pDeletedFile: *mut DELETED_FILE_RECORD,
) -> HRESULT {
    let res: i32 = IntDeleteRecursive(pDeletedFileName);
    if res == 0 {
        return HResultFromWin32(GetLastError());
    }
    let res: HRESULT = RemoveFromDatabase(this, pDeletedFileName, pDeletedFile);
    if res == 0 {
        SHUpdateRecycleBinIcon();
    }
    res
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_Restore(
    this: *mut RecycleBin5,
    pDeletedFileName: LPCWSTR,
    pDeletedFile: *mut DELETED_FILE_RECORD,
) -> HRESULT {
    let res = SHELL_SingleFileOperation(
        core::ptr::null_mut(),
        FO_MOVE,
        pDeletedFileName,
        (*pDeletedFile).FileNameW,
        0,
    );
    if res != 0 {
        return E_FAIL;
    }
    let res = RemoveFromDatabase(this, pDeletedFileName, pDeletedFile);
    if res == 0 {
        SHUpdateRecycleBinIcon(); // Full --> Empty
    }
    res
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_RemoveFromDatabase(
    this: *mut RecycleBin5,
    pDeletedFileName: *const u16,
    pDeletedFile: *mut DELETED_FILE_RECORD,
) -> HRESULT {
    if (*this).m_EnumeratorCount != 0 {
        return HRESULT_FROM_WIN32((ERROR_SHARING_VIOLATION) as LONG);
    }

    let pHeader = MapViewOfFile(
        (*this).m_hInfoMapped,
        FILE_MAP_WRITE,
        0,
        0,
        0,
    ) as *mut INFO2_HEADER;

    if pHeader.is_null() {
        return HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }

    let mut FileSize = ULARGE_INTEGER {
        __bindgen_anon_1: core::mem::zeroed(),
    };
    FileSize.__bindgen_anon_1.LowPart = GetFileSize(
        (*this).m_hInfo,
        &mut FileSize.__bindgen_anon_1.HighPart,
    );
    if FileSize.__bindgen_anon_1.LowPart == 0 {
        UnmapViewOfFile(pHeader);
        return HRESULT_FROM_WIN32((GetLastError()) as LONG);
    }

    let dwEntries = ((FileSize.QuadPart - core::mem::size_of::<INFO2_HEADER>() as u64)
        / core::mem::size_of::<DELETED_FILE_RECORD>() as u64) as u32;

    let mut pRecord = (pHeader as *mut u8).add(core::mem::size_of::<INFO2_HEADER>()) as *mut DELETED_FILE_RECORD;
    let mut i = 0;
    while i < dwEntries {
        if (*pRecord).dwRecordUniqueId == (*pDeletedFile).dwRecordUniqueId {
            MoveMemory!(
                pRecord,
                pRecord.add(1 as usize),
                ((dwEntries - i - 1) as usize)
                    * core::mem::size_of::<DELETED_FILE_RECORD>(),
            );
            let pLast = pRecord.add((dwEntries - i - 1) as usize);
            ZeroMemory!(pLast, core::mem::size_of::<DELETED_FILE_RECORD>());
            UnmapViewOfFile(pHeader);

            CloseHandle((*this).m_hInfoMapped);
            SetFilePointer(
                (*this).m_hInfo,
                -(core::mem::size_of::<DELETED_FILE_RECORD>() as i32),
                core::ptr::null_mut(),
                FILE_END,
            );
            SetEndOfFile((*this).m_hInfo);

            (*this).m_hInfoMapped = CreateFileMappingW(
                (*this).m_hInfo,
                core::ptr::null_mut(),
                PAGE_READWRITE | SEC_COMMIT,
                0,
                0,
                core::ptr::null_mut(),
            );
            if (*this).m_hInfoMapped.is_null() {
                return HResultFromWin32(GetLastError());
            }

            let dwEntries = dwEntries - 1;
            let result = if (dwEntries as i32) < core::mem::transmute::<u32, i32>(0) {
                INT_MAX
            } else {
                dwEntries as HRESULT
            };
            return result;
        }
        pRecord = pRecord.add(1);
        i += 1;
    }

    UnmapViewOfFile(pHeader);
    HRESULT_FROM_WIN32((ERROR_FILE_NOT_FOUND) as LONG)
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin5_OnClosing(this: *mut RecycleBin5, prbel: *mut IRecycleBinEnumList) -> HRESULT {
    (*this).m_EnumeratorCount -= 1;
    S_OK
}
// C file-scope `static` = internal linkage, which is Rust's default for a non-`pub` item.
unsafe extern "C" fn RecycleBin5_Create(Folder: LPCWSTR, OwnerSid: PSID) -> HRESULT {
    let mut BufferName: LPWSTR = core::ptr::null_mut();
    let mut Separator: LPWSTR;
    let mut FileName: LPWSTR;
    let DesktopIniContents: *const u8 =
        b"[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n\0".as_ptr();
    let info2_contents = [INFO2_HEADER(
        5,
        0,
        0,
        0x320,
        0,
    )];
    let mut BytesToWrite: DWORD;
    let mut BytesWritten: DWORD = 0;
    let mut Needed: SIZE_T;
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE as HANDLE;
    let mut hr: HRESULT;

    'cleanup: loop {
        Needed = (wcslen(Folder)
            + 1
            + max(
                wcslen(RECYCLE_BIN_FILE_NAME),
                wcslen(W_DESKTOP_INI.as_ptr()),
            )
            + 1)
            * core::mem::size_of::<u16>();
        BufferName = HeapAlloc(
            GetProcessHeap(),
            0,
            Needed,
        ) as LPWSTR;
        if BufferName.is_null() {
            hr = (ERROR_NOT_ENOUGH_MEMORY) as i32;
            break 'cleanup;
        }

        wcscpy(BufferName, Folder);
        Separator = wcsstr(BufferName.add(3), W_BACKSLASH.as_ptr()) as LPWSTR;
        if !Separator.is_null() {
            *Separator = UNICODE_NULL as u16;
        }

        if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0
            && GetLastError() != ERROR_ALREADY_EXISTS
        {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);

        if !Separator.is_null() {
            *Separator = WC_BACKSLASH;
            if CreateDirectoryW(BufferName, core::ptr::null_mut()) == 0
                && GetLastError() != ERROR_ALREADY_EXISTS
            {
                hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
                break 'cleanup;
            }
        }

        /* FIXME: ACL handling not yet implemented */

        wcscat(BufferName, W_BACKSLASH.as_ptr());
        FileName = BufferName.add(wcslen(BufferName));

        /* Create desktop.ini */
        wcscpy(FileName, W_DESKTOP_INI.as_ptr());
        hFile = CreateFileW(
            BufferName,
            GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        ) as HANDLE;
        if hFile == INVALID_HANDLE_VALUE as HANDLE {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        BytesToWrite = strlen(DesktopIniContents as LPCSTR) as DWORD;
        if WriteFile(
            hFile,
            DesktopIniContents as *const core::ffi::c_void,
            BytesToWrite,
            core::ptr::addr_of_mut!(BytesWritten),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }
        if BytesWritten != BytesToWrite {
            hr = E_FAIL;
            break 'cleanup;
        }
        CloseHandle(hFile);
        hFile = INVALID_HANDLE_VALUE as HANDLE;

        /* Create empty INFO2 file */
        wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
        hFile = CreateFileW(
            BufferName,
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_HIDDEN,
            core::ptr::null_mut(),
        ) as HANDLE;
        if hFile == INVALID_HANDLE_VALUE as HANDLE {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        BytesToWrite = core::mem::size_of::<INFO2_HEADER>() as DWORD;
        if WriteFile(
            hFile,
            info2_contents.as_ptr() as *const core::ffi::c_void,
            BytesToWrite,
            core::ptr::addr_of_mut!(BytesWritten),
            core::ptr::null_mut(),
        ) == 0
        {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }
        if BytesWritten == BytesToWrite {
            hr = S_OK;
        } else {
            hr = E_FAIL;
        }
        break 'cleanup;
    }

    // cleanup
    HeapFree(GetProcessHeap(), 0, (BufferName).cast());
    if hFile != INVALID_HANDLE_VALUE as HANDLE {
        CloseHandle(hFile);
    }
    hr
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5_ctor(this: *mut RecycleBin5) {
    (*this).m_ref = 1;
    (*this).m_hInfo = core::ptr::null_mut();
    (*this).m_hInfoMapped = core::ptr::null_mut();
    (*this).m_EnumeratorCount = 0;
}

#[no_mangle]
pub unsafe extern "C" fn RecycleBin5_Init(this: *mut RecycleBin5, VolumePath: LPCWSTR) -> HRESULT {
    let mut hr: HRESULT;
    let mut FileSystemFlags: DWORD = 0;
    let RecycleBinDirectory: LPCWSTR;
    let mut tokenHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut TokenUserInfo: PTOKEN_USER = core::ptr::null_mut();
    let mut StringSid: LPWSTR = core::ptr::null_mut();
    let mut Needed: DWORD;
    let mut len: INT;

    'cleanup: {
        // m_VolumePath = VolumePath
        CStringW::assign(&mut (*this).m_VolumePath, VolumePath);

        // GetVolumeInformationW
        if GetVolumeInformationW(
            VolumePath,
            core::ptr::null_mut(),
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut FileSystemFlags,
            core::ptr::null_mut(),
            0,
        ) == 0
        {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        // Check file system flags
        if (FileSystemFlags & FILE_PERSISTENT_ACLS) == 0 {
            RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITHOUT_ACL;
        } else {
            RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITH_ACL;

            // OpenProcessToken
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut tokenHandle,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
                break 'cleanup;
            }

            // First GetTokenInformation to get required size
            if GetTokenInformation(
                tokenHandle,
                TokenUser,
                core::ptr::null_mut(),
                0,
                &mut Needed,
            ) != 0
            {
                hr = E_FAIL;
                break 'cleanup;
            }
            if GetLastError() != ERROR_INSUFFICIENT_BUFFER {
                hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
                break 'cleanup;
            }

            // Allocate memory for token user
            TokenUserInfo = HeapAlloc(
                GetProcessHeap(),
                0,
                (Needed) as SIZE_T,
            ) as PTOKEN_USER;
            if TokenUserInfo.is_null() {
                hr = E_OUTOFMEMORY;
                break 'cleanup;
            }

            // Second GetTokenInformation to retrieve actual data
            if GetTokenInformation(
                tokenHandle,
                TokenUser,
                (TokenUserInfo).cast(),
                Needed as DWORD,
                &mut Needed,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
                break 'cleanup;
            }

            // Convert SID to string
            if ConvertSidToStringSidW(
                (*TokenUserInfo).User.Sid,
                &mut StringSid,
            ) == 0
            {
                hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
                break 'cleanup;
            }
        }

        // Build m_Folder path
        CStringW::assign(&mut (*this).m_Folder, VolumePath);
        CStringW::append_str(&mut (*this).m_Folder, RecycleBinDirectory);

        if !StringSid.is_null() {
            CStringW::append_char(&mut (*this).m_Folder, 0x005Cu16); // '\\'
            CStringW::append_str(&mut (*this).m_Folder, StringSid);
        }

        len = CStringW::len(&(*this).m_Folder);
        CStringW::append_str(&mut (*this).m_Folder, RECYCLE_BIN_FILE_NAME);

        // First attempt to open/create the info file
        (*this).m_hInfo = CreateFileW(
            CStringW::as_ptr(&(*this).m_Folder),
            GENERIC_READ | GENERIC_WRITE,
            0,
            core::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            core::ptr::null_mut(),
        );

        // If file doesn't exist or path not found, try to create the recycle bin directory
        if (*this).m_hInfo == INVALID_HANDLE_VALUE
            && (GetLastError() == ERROR_PATH_NOT_FOUND
                || GetLastError() == ERROR_FILE_NOT_FOUND)
        {
            // Trim m_Folder back to the directory path
            CStringW::assign_cstringw(
                &mut (*this).m_Folder,
                &CStringW::left(&(*this).m_Folder, len),
            );

            hr = RecycleBin5_Create(
                CStringW::as_ptr(&(*this).m_Folder),
                if !TokenUserInfo.is_null() {
                    (*TokenUserInfo).User.Sid
                } else {
                    core::ptr::null_mut()
                },
            );

            CStringW::append_str(&mut (*this).m_Folder, RECYCLE_BIN_FILE_NAME);

            if hr < 0 {
                break 'cleanup;
            }

            // Retry opening the info file with TRUNCATE_EXISTING semantics
            (*this).m_hInfo = CreateFileW(
                CStringW::as_ptr(&(*this).m_Folder),
                GENERIC_READ | GENERIC_WRITE,
                0,
                core::ptr::null_mut(),
                OPEN_EXISTING,
                0,
                core::ptr::null_mut(),
            );
        }

        // Still invalid handle => error
        if (*this).m_hInfo == INVALID_HANDLE_VALUE {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        // Create file mapping
        (*this).m_hInfoMapped = CreateFileMappingW(
            (*this).m_hInfo,
            core::ptr::null_mut(),
            PAGE_READWRITE | SEC_COMMIT,
            0,
            0,
            core::ptr::null_mut(),
        );

        if (*this).m_hInfoMapped.is_null() {
            hr = HRESULT_FROM_WIN32((GetLastError()) as LONG);
            break 'cleanup;
        }

        // Restore folder path to directory only
        CStringW::assign_cstringw(
            &mut (*this).m_Folder,
            &CStringW::left(&(*this).m_Folder, len),
        );

        hr = S_OK; // S_OK
    }

    // Cleanup
    if tokenHandle != INVALID_HANDLE_VALUE {
        CloseHandle(tokenHandle);
    }
    HeapFree(
        GetProcessHeap(),
        0,
        TokenUserInfo as *mut core::ffi::c_void,
    );
    if !StringSid.is_null() {
        LocalFree((StringSid).cast());
    }

    hr
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBin5_Constructor(
    VolumePath: *const u16,
    ppUnknown: *mut *mut core::ffi::c_void,
) -> i32 {
    if ppUnknown.is_null() {
        return E_POINTER;
    }
    *ppUnknown = core::ptr::null_mut();
    let pThis: *mut RecycleBin5 = RecycleBin5();
    if pThis.is_null() {
        return E_OUTOFMEMORY;
    }
    let hr = unsafe { RecycleBin5::Init(pThis, VolumePath) };
    if hr < 0 {
        unsafe { RecycleBin5::delete(pThis) };
        return hr;
    }
    *ppUnknown = pThis as *mut core::ffi::c_void;
    S_OK
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     Recycle bin management
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     Deals with recycle bins of Windows 2000/XP/2003
//  * COPYRIGHT:   Copyright 2006-2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              Copyright 2024 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "recyclebin_private.h"
// #include <atlstr.h>
// #include <shlwapi.h>
// #include "sddl.h"
//
// extern "C" HRESULT WINAPI SHUpdateRecycleBinIcon(void);
//
// class CZZWStr
// {
//     LPWSTR m_sz;
//
// public:
//     ~CZZWStr() { SHFree(m_sz); }
//     CZZWStr() : m_sz(NULL) {}
//     CZZWStr(const CZZWStr&) = delete;
//     CZZWStr& operator=(const CZZWStr&) = delete;
//
//     bool Initialize(LPCWSTR Str)
//     {
//         SIZE_T cch = wcslen(Str) + 1;
//         m_sz = (LPWSTR)SHAlloc((cch + 1) * sizeof(*Str));
//         if (!m_sz)
//             return false;
//         CopyMemory(m_sz, Str, cch * sizeof(*Str));
//         m_sz[cch] = UNICODE_NULL; // Double-null terminate
//         return true;
//     }
//     inline LPWSTR c_str() { return m_sz; }
// };
//
// static int SHELL_SingleFileOperation(HWND hWnd, UINT Op, LPCWSTR pszFrom, LPCWSTR pszTo, FILEOP_FLAGS Flags)
// {
//     CZZWStr szzFrom, szzTo;
//     if (!szzFrom.Initialize(pszFrom) || !szzTo.Initialize(pszTo))
//         return ERROR_OUTOFMEMORY; // Note: Not one of the DE errors but also not in the DE range
//     SHFILEOPSTRUCTW fos = { hWnd, Op, szzFrom.c_str(), szzTo.c_str(), Flags };
//     return SHFileOperationW(&fos);
// }
//
// static BOOL
// IntDeleteRecursive(
//      LPCWSTR FullName)
// {
//     DWORD RemovableAttributes = FILE_ATTRIBUTE_READONLY;
//     WIN32_FIND_DATAW FindData;
//     HANDLE hSearch = INVALID_HANDLE_VALUE;
//     LPWSTR FullPath = NULL, pFilePart;
//     DWORD FileAttributes;
//     SIZE_T dwLength;
//     BOOL ret = FALSE;
//
//     FileAttributes = GetFileAttributesW(FullName);
//     if (FileAttributes == INVALID_FILE_ATTRIBUTES)
//     {
//         if (GetLastError() == ERROR_FILE_NOT_FOUND)
//             ret = TRUE;
//         goto cleanup;
//     }
//     if (FileAttributes & RemovableAttributes)
//     {
//         if (!SetFileAttributesW(FullName, FileAttributes & ~RemovableAttributes))
//             goto cleanup;
//     }
//     if (FileAttributes & FILE_ATTRIBUTE_DIRECTORY)
//     {
//         /* Prepare file specification */
//         dwLength = wcslen(FullName);
//         FullPath = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, (dwLength + 1 + MAX_PATH + 1) * sizeof(WCHAR));
//         if (!FullPath)
//         {
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//         wcscpy(FullPath, FullName);
//         if (FullPath[dwLength - 1] != '\\')
//         {
//             FullPath[dwLength] = '\\';
//             dwLength++;
//         }
//         pFilePart = &FullPath[dwLength];
//         wcscpy(pFilePart, W_STAR.as_ptr());
//
//         /* Enumerate contents, and delete it */
//         hSearch = FindFirstFileW(FullPath, &FindData);
//         if (hSearch == INVALID_HANDLE_VALUE)
//             goto cleanup;
//         do
//         {
//             if (!(FindData.cFileName[0] == '.' &&
//                 (FindData.cFileName[1] == '\0' || (FindData.cFileName[1] == '.' && FindData.cFileName[2] == '\0'))))
//             {
//                 wcscpy(pFilePart, FindData.cFileName);
//                 if (!IntDeleteRecursive(FullPath))
//                 {
//                     FindClose(hSearch);
//                     goto cleanup;
//                 }
//             }
//         }
//         while (FindNextFileW(hSearch, &FindData));
//         FindClose(hSearch);
//         if (GetLastError() != ERROR_NO_MORE_FILES)
//             goto cleanup;
//
//         /* Remove (now empty) directory */
//         if (!RemoveDirectoryW(FullName))
//             goto cleanup;
//     }
//     else
//     {
//         if (!DeleteFileW(FullName))
//             goto cleanup;
//     }
//     ret = TRUE;
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, FullPath);
//     return ret;
// }
//
// class RecycleBin5 : public IRecycleBin5
// {
// public:
//     RecycleBin5();
//     virtual ~RecycleBin5();
//
//     HRESULT Init( LPCWSTR VolumePath);
//
//     /* IUnknown interface */
//     HRESULT STDMETHODCALLTYPE QueryInterface( REFIID riid,  void **ppvObject) override;
//     STDMETHODIMP_(ULONG) AddRef() override;
//     STDMETHODIMP_(ULONG) Release() override;
//
//     /* IRecycleBin interface */
//     HRESULT STDMETHODCALLTYPE DeleteFile( LPCWSTR szFileName) override;
//     HRESULT STDMETHODCALLTYPE EmptyRecycleBin() override;
//     HRESULT STDMETHODCALLTYPE EnumObjects( IRecycleBinEnumList **ppEnumList) override;
//     HRESULT STDMETHODCALLTYPE GetDirectory(LPWSTR szPath) override
//     {
//         if (!m_Folder[0])
//             return E_UNEXPECTED;
//         lstrcpynW(szPath, m_Folder, MAX_PATH);
//         return S_OK;
//     }
//
//     /* IRecycleBin5 interface */
//     HRESULT STDMETHODCALLTYPE Delete(
//          LPCWSTR pDeletedFileName,
//          DELETED_FILE_RECORD *pDeletedFile) override;
//     HRESULT STDMETHODCALLTYPE Restore(
//          LPCWSTR pDeletedFileName,
//          DELETED_FILE_RECORD *pDeletedFile) override;
//     HRESULT STDMETHODCALLTYPE RemoveFromDatabase(
//          LPCWSTR pDeletedFileName,
//          DELETED_FILE_RECORD *pDeletedFile) override;
//     HRESULT STDMETHODCALLTYPE OnClosing( IRecycleBinEnumList *prbel) override;
//
// protected:
//     LONG m_ref;
//     HANDLE m_hInfo;
//     HANDLE m_hInfoMapped;
//     DWORD m_EnumeratorCount;
//     CStringW m_VolumePath;
//     CStringW m_Folder; /* [drive]:\[RECYCLE_BIN_DIRECTORY]\{SID} */
// };
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::QueryInterface( REFIID riid,  void **ppvObject)
// {
//     TRACE("(%p, %s, %p)\n", this, debugstr_guid(&riid), ppvObject);
//
//     if (!ppvObject)
//         return E_POINTER;
//
//     if (IsEqualIID(riid, IID_IUnknown) || IsEqualIID(riid, IID_IRecycleBin))
//         *ppvObject = static_cast<IRecycleBin5 *>(this);
//     else if (IsEqualIID(riid, IID_IRecycleBin5))
//         *ppvObject = static_cast<IRecycleBin5 *>(this);
//     else
//     {
//         *ppvObject = NULL;
//         return E_NOINTERFACE;
//     }
//
//     AddRef();
//     return S_OK;
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5::AddRef()
// {
//     TRACE("(%p)\n", this);
//     return InterlockedIncrement(&m_ref);
// }
//
// RecycleBin5::~RecycleBin5()
// {
//     TRACE("(%p)\n", this);
//
//     if (m_hInfo && m_hInfo != INVALID_HANDLE_VALUE)
//         CloseHandle(m_hInfo);
//     if (m_hInfoMapped)
//         CloseHandle(m_hInfoMapped);
// }
//
// STDMETHODIMP_(ULONG) RecycleBin5::Release()
// {
//     TRACE("(%p)\n", this);
//
//     ULONG refCount = InterlockedDecrement(&m_ref);
//     if (refCount == 0)
//         delete this;
//     return refCount;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::DeleteFile( LPCWSTR szFileName)
// {
//     LPWSTR szFullName = NULL;
//     DWORD dwBufferLength = 0;
//     LPWSTR lpFilePart;
//     LPCWSTR Extension;
//     CStringW DeletedFileName;
//     WCHAR szUniqueId[64];
//     DWORD len;
//     HANDLE hFile = INVALID_HANDLE_VALUE;
//     PINFO2_HEADER pHeader = NULL;
//     PDELETED_FILE_RECORD pDeletedFile;
//     ULARGE_INTEGER FileSize;
//     DWORD dwAttributes, dwEntries;
//     SYSTEMTIME SystemTime;
//     DWORD ClusterSize, BytesPerSector, SectorsPerCluster;
//     HRESULT hr;
//     WIN32_FIND_DATAW wfd = {};
//
//     TRACE("(%p, %s)\n", this, debugstr_w(szFileName));
//
//     if (m_EnumeratorCount != 0)
//         return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION);
//
//     /* Get full file name */
//     while (TRUE)
//     {
//         len = GetFullPathNameW(szFileName, dwBufferLength, szFullName, &lpFilePart);
//         if (len == 0)
//         {
//             if (szFullName)
//                 CoTaskMemFree(szFullName);
//             return HResultFromWin32(GetLastError());
//         }
//         else if (len < dwBufferLength)
//             break;
//         if (szFullName)
//             CoTaskMemFree(szFullName);
//         dwBufferLength = len;
//         szFullName = (LPWSTR)CoTaskMemAlloc(dwBufferLength * sizeof(WCHAR));
//         if (!szFullName)
//             return HRESULT_FROM_WIN32(ERROR_NOT_ENOUGH_MEMORY);
//     }
//
//     /* Check if file exists */
//     dwAttributes = GetFileAttributesW(szFullName);
//     if (dwAttributes == INVALID_FILE_ATTRIBUTES)
//     {
//         CoTaskMemFree(szFullName);
//         return HResultFromWin32(GetLastError());
//     }
//
//     if (dwBufferLength < 2 || szFullName[1] != ':')
//     {
//         /* Not a local file */
//         CoTaskMemFree(szFullName);
//         return HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
//     }
//
//     hFile = CreateFileW(szFullName, 0, 0, NULL, OPEN_EXISTING, (dwAttributes & FILE_ATTRIBUTE_DIRECTORY) ? FILE_FLAG_BACKUP_SEMANTICS : 0, NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//
//     /* Increase INFO2 file size */
//     CloseHandle(m_hInfoMapped);
//     SetFilePointer(m_hInfo, sizeof(DELETED_FILE_RECORD), NULL, FILE_END);
//     SetEndOfFile(m_hInfo);
//     m_hInfoMapped = CreateFileMappingW(m_hInfo, NULL, PAGE_READWRITE | SEC_COMMIT, 0, 0, NULL);
//     if (!m_hInfoMapped)
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//
//     /* Open INFO2 file */
//     pHeader = (PINFO2_HEADER)MapViewOfFile(m_hInfoMapped, FILE_MAP_WRITE, 0, 0, 0);
//     if (!pHeader)
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//
//     /* Get number of entries */
//     FileSize.u.LowPart = GetFileSize(m_hInfo, &FileSize.u.HighPart);
//     if (FileSize.u.LowPart < sizeof(INFO2_HEADER))
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//     dwEntries = (DWORD)((FileSize.QuadPart - sizeof(INFO2_HEADER)) / sizeof(DELETED_FILE_RECORD)) - 1;
//     pDeletedFile = ((PDELETED_FILE_RECORD)(pHeader + 1)) + dwEntries;
//
//     /* Get file size */
//     FileSize.u.LowPart = GetFileSize(hFile, &FileSize.u.HighPart);
//     if (FileSize.u.LowPart == INVALID_FILE_SIZE && GetLastError() != NO_ERROR)
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//     /* Check if file size is > 4Gb */
//     if (FileSize.u.HighPart != 0)
//     {
//         /* Yes, this recyclebin can't support this file */
//         hr = HRESULT_FROM_WIN32(ERROR_NOT_SUPPORTED);
//         goto cleanup;
//     }
//     pHeader->dwTotalLogicalSize += FileSize.u.LowPart;
//
//     /* Generate new name */
//     Extension = PathFindExtensionW(szFullName);
//     ZeroMemory(pDeletedFile, sizeof(DELETED_FILE_RECORD));
//     if (dwEntries == 0)
//         pDeletedFile->dwRecordUniqueId = 0;
//     else
//     {
//         PDELETED_FILE_RECORD pLastDeleted = ((PDELETED_FILE_RECORD)(pHeader + 1)) + dwEntries - 1;
//         pDeletedFile->dwRecordUniqueId = pLastDeleted->dwRecordUniqueId + 1;
//     }
//
//     pDeletedFile->dwDriveNumber = tolower(szFullName[0]) - 'a';
//     _ultow(pDeletedFile->dwRecordUniqueId, szUniqueId, 10);
//
//     DeletedFileName = m_Folder;
//     DeletedFileName += L"\\D";
//     DeletedFileName += (WCHAR)(L'a' + pDeletedFile->dwDriveNumber);
//     DeletedFileName += szUniqueId;
//     DeletedFileName += Extension;
//
//     /* Get cluster size */
//     if (!GetDiskFreeSpaceW(m_VolumePath, &SectorsPerCluster, &BytesPerSector, NULL, NULL))
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//     ClusterSize = BytesPerSector * SectorsPerCluster;
//
//     /* Get current time */
//     GetSystemTime(&SystemTime);
//     if (!SystemTimeToFileTime(&SystemTime, &pDeletedFile->DeletionTime))
//     {
//         hr = HResultFromWin32(GetLastError());
//         goto cleanup;
//     }
//     pDeletedFile->dwPhysicalFileSize = ROUND_UP(FileSize.u.LowPart, ClusterSize);
//
//     /* Set name */
//     wcscpy(pDeletedFile->FileNameW, szFullName);
//     if (WideCharToMultiByte(CP_ACP, 0, pDeletedFile->FileNameW, -1, pDeletedFile->FileNameA, MAX_PATH, NULL, NULL) == 0)
//     {
//         hr = HRESULT_FROM_WIN32(ERROR_INVALID_NAME);
//         SetLastError(ERROR_INVALID_NAME);
//         goto cleanup;
//     }
//
//     wfd.dwFileAttributes = dwAttributes;
//     wfd.nFileSizeLow = FileSize.u.LowPart;
//     GetFileTime(hFile, &wfd.ftCreationTime, &wfd.ftLastAccessTime, &wfd.ftLastWriteTime);
//
//     /* Move file */
//     if (MoveFileW(szFullName, DeletedFileName))
//         hr = S_OK;
//     else
//         hr = HResultFromWin32(GetLastError());
//
//     if (SUCCEEDED(hr))
//     {
//         RECYCLEBINFILEIDENTITY ident = { pDeletedFile->DeletionTime, DeletedFileName };
//         CRecycleBin_NotifyRecycled(szFullName, &wfd, &ident);
//     }
//
// cleanup:
//     if (pHeader)
//         UnmapViewOfFile(pHeader);
//     if (hFile != INVALID_HANDLE_VALUE)
//         CloseHandle(hFile);
//     CoTaskMemFree(szFullName);
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::EmptyRecycleBin()
// {
//     TRACE("(%p)\n", this);
//
//     while (TRUE)
//     {
//         IRecycleBinEnumList *prbel;
//         HRESULT hr = EnumObjects(&prbel);
//         if (!SUCCEEDED(hr))
//             return hr;
//
//         IRecycleBinFile *prbf;
//         hr = prbel->Next(1, &prbf, NULL);
//         prbel->Release();
//         if (hr == S_FALSE)
//             return S_OK;
//         hr = prbf->Delete();
//         prbf->Release();
//         if (!SUCCEEDED(hr))
//             return hr;
//     }
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::EnumObjects( IRecycleBinEnumList **ppEnumList)
// {
//     TRACE("(%p, %p)\n", this, ppEnumList);
//
//     IUnknown *pUnk;
//     HRESULT hr = RecycleBin5Enum_Constructor(this, m_hInfo, m_hInfoMapped, m_Folder, &pUnk);
//     if (!SUCCEEDED(hr))
//         return hr;
//
//     IRecycleBinEnumList *prbel;
//     hr = pUnk->QueryInterface(IID_IRecycleBinEnumList, (void **)&prbel);
//     if (SUCCEEDED(hr))
//     {
//         m_EnumeratorCount++;
//         *ppEnumList = prbel;
//     }
//
//     pUnk->Release();
//     return hr;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::Delete(
//      LPCWSTR pDeletedFileName,
//      DELETED_FILE_RECORD *pDeletedFile)
// {
//
//     TRACE("(%p, %s, %p)\n", this, debugstr_w(pDeletedFileName), pDeletedFile);
//
//     int res = IntDeleteRecursive(pDeletedFileName);
//     if (!res)
//         return HResultFromWin32(GetLastError());
//     res = RemoveFromDatabase(pDeletedFileName, pDeletedFile);
//     if (res == 0)
//         SHUpdateRecycleBinIcon(); // Full --> Empty
//     return res;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::Restore(
//      LPCWSTR pDeletedFileName,
//      DELETED_FILE_RECORD *pDeletedFile)
// {
//
//     TRACE("(%p, %s, %p)\n", this, debugstr_w(pDeletedFileName), pDeletedFile);
//
//     int res = SHELL_SingleFileOperation(NULL, FO_MOVE, pDeletedFileName, pDeletedFile->FileNameW, 0);
//     if (res)
//     {
//         ERR("SHFileOperationW failed with 0x%x\n", res);
//         return E_FAIL;
//     }
//     res = RemoveFromDatabase(pDeletedFileName, pDeletedFile);
//     if (res == 0)
//         SHUpdateRecycleBinIcon(); // Full --> Empty
//     return res;
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::RemoveFromDatabase(
//      LPCWSTR pDeletedFileName,
//      DELETED_FILE_RECORD *pDeletedFile)
// {
//     ULARGE_INTEGER FileSize;
//     PINFO2_HEADER pHeader;
//     DELETED_FILE_RECORD *pRecord, *pLast;
//     DWORD dwEntries, i;
//
//     TRACE("(%p, %s, %p)\n", this, debugstr_w(pDeletedFileName), pDeletedFile);
//
//     if (m_EnumeratorCount != 0)
//         return HRESULT_FROM_WIN32(ERROR_SHARING_VIOLATION);
//
//     pHeader = (PINFO2_HEADER)MapViewOfFile(m_hInfoMapped, FILE_MAP_WRITE, 0, 0, 0);
//     if (!pHeader)
//         return HRESULT_FROM_WIN32(GetLastError());
//
//     FileSize.u.LowPart = GetFileSize(m_hInfo, &FileSize.u.HighPart);
//     if (FileSize.u.LowPart == 0)
//     {
//         UnmapViewOfFile(pHeader);
//         return HRESULT_FROM_WIN32(GetLastError());
//     }
//     dwEntries = (DWORD)((FileSize.QuadPart - sizeof(INFO2_HEADER)) / sizeof(DELETED_FILE_RECORD));
//
//     pRecord = (DELETED_FILE_RECORD *)(pHeader + 1);
//     for (i = 0; i < dwEntries; i++)
//     {
//         if (pRecord->dwRecordUniqueId == pDeletedFile->dwRecordUniqueId)
//         {
//             /* Clear last entry in the file */
//             MoveMemory(pRecord, pRecord + 1, (dwEntries - i - 1) * sizeof(DELETED_FILE_RECORD));
//             pLast = pRecord + (dwEntries - i - 1);
//             ZeroMemory(pLast, sizeof(DELETED_FILE_RECORD));
//             UnmapViewOfFile(pHeader);
//
//             /* Resize file */
//             CloseHandle(m_hInfoMapped);
//             SetFilePointer(m_hInfo, -(LONG)sizeof(DELETED_FILE_RECORD), NULL, FILE_END);
//             SetEndOfFile(m_hInfo);
//             m_hInfoMapped = CreateFileMappingW(m_hInfo, NULL, PAGE_READWRITE | SEC_COMMIT, 0, 0, NULL);
//             if (!m_hInfoMapped)
//                 return HResultFromWin32(GetLastError());
//             dwEntries--;
//             return FAILED((int)dwEntries) ? INT_MAX : dwEntries;
//         }
//         pRecord++;
//     }
//
//     UnmapViewOfFile(pHeader);
//     return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
// }
//
// HRESULT STDMETHODCALLTYPE RecycleBin5::OnClosing( IRecycleBinEnumList *prbel)
// {
//     TRACE("(%p, %p)\n", this, prbel);
//     m_EnumeratorCount--;
//     return S_OK;
// }
//
// static HRESULT
// RecycleBin5_Create(
//      LPCWSTR Folder,
//      PSID OwnerSid )
// {
//     LPWSTR BufferName = NULL;
//     LPWSTR Separator; /* Pointer into BufferName buffer */
//     LPWSTR FileName; /* Pointer into BufferName buffer */
//     LPCSTR DesktopIniContents = "[.ShellClassInfo]\r\nCLSID={645FF040-5081-101B-9F08-00AA002F954E}\r\n";
//     INFO2_HEADER Info2Contents[] = { { 5, 0, 0, 0x320, 0 } };
//     DWORD BytesToWrite, BytesWritten;
//     SIZE_T Needed;
//     HANDLE hFile = INVALID_HANDLE_VALUE;
//     HRESULT hr;
//
//     Needed = (wcslen(Folder) + 1 + max(wcslen(RECYCLE_BIN_FILE_NAME), wcslen(L"desktop.ini")) + 1) * sizeof(WCHAR);
//     BufferName = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, Needed);
//     if (!BufferName)
//     {
//         hr = ERROR_NOT_ENOUGH_MEMORY;
//         goto cleanup;
//     }
//
//     wcscpy(BufferName, Folder);
//     Separator = wcsstr(&BufferName[3], L"\\");
//     if (Separator)
//         *Separator = UNICODE_NULL;
//     if (!CreateDirectoryW(BufferName, NULL) && GetLastError() != ERROR_ALREADY_EXISTS)
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//     SetFileAttributesW(BufferName, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN);
//     if (Separator)
//     {
//         *Separator = WC_BACKSLASH;
//         if (!CreateDirectoryW(BufferName, NULL) && GetLastError() != ERROR_ALREADY_EXISTS)
//         {
//             hr = HRESULT_FROM_WIN32(GetLastError());
//             goto cleanup;
//         }
//     }
//
//     if (OwnerSid)
//     {
//         //DWORD rc;
//
//         /* Add ACL to allow only user/SYSTEM to open it */
//         /* FIXME: rc = SetNamedSecurityInfo(
//             BufferName,
//             SE_FILE_OBJECT,
//             ???,
//             OwnerSid,
//             NULL,
//             ???,
//             ???);
//         if (rc != ERROR_SUCCESS)
//         {
//             hr = HRESULT_FROM_WIN32(rc);
//             goto cleanup;
//         }
//         */
//     }
//
//     wcscat(BufferName, W_BACKSLASH.as_ptr());
//     FileName = &BufferName[wcslen(BufferName)];
//
//     /* Create desktop.ini */
//     wcscpy(FileName, W_DESKTOP_INI.as_ptr());
//     hFile = CreateFileW(BufferName, GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_SYSTEM | FILE_ATTRIBUTE_HIDDEN, NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//     BytesToWrite = (UINT)strlen(DesktopIniContents);
//     if (!WriteFile(hFile, DesktopIniContents, (DWORD)BytesToWrite, &BytesWritten, NULL))
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//     if (BytesWritten != BytesToWrite)
//     {
//         hr = E_FAIL;
//         goto cleanup;
//     }
//     CloseHandle(hFile);
//     hFile = INVALID_HANDLE_VALUE;
//
//     /* Create empty INFO2 file */
//     wcscpy(FileName, RECYCLE_BIN_FILE_NAME);
//     hFile = CreateFileW(BufferName, GENERIC_READ | GENERIC_WRITE, 0, NULL, CREATE_ALWAYS, FILE_ATTRIBUTE_HIDDEN, NULL);
//     if (hFile == INVALID_HANDLE_VALUE)
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//     BytesToWrite = sizeof(Info2Contents);
//     if (!WriteFile(hFile, Info2Contents, (DWORD)BytesToWrite, &BytesWritten, NULL))
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//     if (BytesWritten == BytesToWrite)
//         hr = S_OK;
//     else
//         hr = E_FAIL;
//
// cleanup:
//     HeapFree(GetProcessHeap(), 0, BufferName);
//     if (hFile != INVALID_HANDLE_VALUE)
//         CloseHandle(hFile);
//     return hr;
// }
//
// RecycleBin5::RecycleBin5()
//     : m_ref(1)
//     , m_hInfo(NULL)
//     , m_hInfoMapped(NULL)
//     , m_EnumeratorCount(0)
// {
// }
//
// HRESULT RecycleBin5::Init( LPCWSTR VolumePath)
// {
//     DWORD FileSystemFlags;
//     LPCWSTR RecycleBinDirectory;
//     HANDLE tokenHandle = INVALID_HANDLE_VALUE;
//     PTOKEN_USER TokenUserInfo = NULL;
//     LPWSTR StringSid = NULL;
//     DWORD Needed;
//     INT len;
//     HRESULT hr;
//
//     m_VolumePath = VolumePath;
//
//     /* Get information about file system */
//     if (!GetVolumeInformationW(VolumePath, NULL, 0, NULL, NULL, &FileSystemFlags, NULL, 0))
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//
//     if (!(FileSystemFlags & FILE_PERSISTENT_ACLS))
//     {
//         RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITHOUT_ACL;
//     }
//     else
//     {
//         RecycleBinDirectory = RECYCLE_BIN_DIRECTORY_WITH_ACL;
//
//         /* Get user SID */
//         if (!OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &tokenHandle))
//         {
//             hr = HRESULT_FROM_WIN32(GetLastError());
//             goto cleanup;
//         }
//         if (GetTokenInformation(tokenHandle, TokenUser, NULL, 0, &Needed))
//         {
//             hr = E_FAIL;
//             goto cleanup;
//         }
//         if (GetLastError() != ERROR_INSUFFICIENT_BUFFER)
//         {
//             hr = HRESULT_FROM_WIN32(GetLastError());
//             goto cleanup;
//         }
//         TokenUserInfo = (PTOKEN_USER)HeapAlloc(GetProcessHeap(), 0, Needed);
//         if (!TokenUserInfo)
//         {
//             hr = E_OUTOFMEMORY;
//             goto cleanup;
//         }
//         if (!GetTokenInformation(tokenHandle, TokenUser, TokenUserInfo, (DWORD)Needed, &Needed))
//         {
//             hr = HRESULT_FROM_WIN32(GetLastError());
//             goto cleanup;
//         }
//         if (!ConvertSidToStringSidW(TokenUserInfo->User.Sid, &StringSid))
//         {
//             hr = HRESULT_FROM_WIN32(GetLastError());
//             goto cleanup;
//         }
//     }
//
//     m_Folder = VolumePath;
//     m_Folder += RecycleBinDirectory;
//     if (StringSid)
//     {
//         m_Folder += L'\\';
//         m_Folder += StringSid;
//     }
//     len = m_Folder.GetLength();
//     m_Folder += L"\\" RECYCLE_BIN_FILE_NAME;
//
//     m_hInfo = CreateFileW(m_Folder, GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING, 0, NULL);
//     if (m_hInfo == INVALID_HANDLE_VALUE &&
//         (GetLastError() == ERROR_PATH_NOT_FOUND || GetLastError() == ERROR_FILE_NOT_FOUND))
//     {
//         m_Folder = m_Folder.Left(len);
//         hr = RecycleBin5_Create(m_Folder, TokenUserInfo ? TokenUserInfo->User.Sid : NULL);
//         m_Folder += L"\\" RECYCLE_BIN_FILE_NAME;
//         if (!SUCCEEDED(hr))
//             goto cleanup;
//         m_hInfo = CreateFileW(m_Folder, GENERIC_READ | GENERIC_WRITE, 0, NULL, OPEN_EXISTING, 0, NULL);
//     }
//
//     if (m_hInfo == INVALID_HANDLE_VALUE)
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//
//     m_hInfoMapped = CreateFileMappingW(m_hInfo, NULL, PAGE_READWRITE | SEC_COMMIT, 0, 0, NULL);
//     if (!m_hInfoMapped)
//     {
//         hr = HRESULT_FROM_WIN32(GetLastError());
//         goto cleanup;
//     }
//
//     m_Folder = m_Folder.Left(len);
//     hr = S_OK;
//
// cleanup:
//     if (tokenHandle != INVALID_HANDLE_VALUE)
//         CloseHandle(tokenHandle);
//     HeapFree(GetProcessHeap(), 0, TokenUserInfo);
//     if (StringSid)
//         LocalFree(StringSid);
//     return hr;
// }
//
// extern "C"
// HRESULT RecycleBin5_Constructor( LPCWSTR VolumePath,  IUnknown **ppUnknown)
// {
//     if (!ppUnknown)
//         return E_POINTER;
//
//     *ppUnknown = NULL;
//
//     RecycleBin5 *pThis = new RecycleBin5();
//     if (!pThis)
//         return E_OUTOFMEMORY;
//
//     HRESULT hr = pThis->Init(VolumePath);
//     if (FAILED(hr))
//     {
//         delete pThis;
//         return hr;
//     }
//
//     *ppUnknown = static_cast<IRecycleBin5 *>(pThis);
//     return S_OK;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
