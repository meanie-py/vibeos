use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type pfnNtGenericOpen = Option<unsafe extern "system" fn(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES) -> NTSTATUS>;
pub type pfnNtOpenFile = Option<unsafe extern "system" fn(PHANDLE, ACCESS_MASK, POBJECT_ATTRIBUTES, PIO_STATUS_BLOCK, ULONG, ULONG) -> NTSTATUS>;





unsafe extern "C" fn NtOpenObject(r#type: OBJECT_TYPE, phandle: PHANDLE, mut access: DWORD, path: LPCWSTR) -> DWORD {
    let mut ustr: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ustr), path);

    let mut open_struct: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    open_struct.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    open_struct.RootDirectory = core::ptr::null_mut();
    open_struct.ObjectName = core::ptr::addr_of_mut!(ustr);
    open_struct.Attributes = 0x40 as ULONG;

    if r#type != FILE_OBJECT {
        access |= STANDARD_RIGHTS_READ;
    }

    let mut ioStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();

    if r#type == DIRECTORY_OBJECT {
        return NtOpenDirectoryObject(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == SYMBOLICLINK_OBJECT {
        return NtOpenSymbolicLinkObject(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == MUTANT_OBJECT {
        return NtOpenMutant(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == SECTION_OBJECT {
        return NtOpenSection(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == EVENT_OBJECT {
        return NtOpenEvent(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == SEMAPHORE_OBJECT {
        return NtOpenSemaphore(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == TIMER_OBJECT {
        return NtOpenTimer(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == KEY_OBJECT {
        return NtOpenKey(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == EVENTPAIR_OBJECT {
        return NtOpenEventPair(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == IOCOMPLETION_OBJECT {
        return NtOpenIoCompletion(phandle, access, core::ptr::addr_of_mut!(open_struct)) as DWORD;
    } else if r#type == FILE_OBJECT {
        return NtOpenFile(
            phandle,
            access,
            core::ptr::addr_of_mut!(open_struct),
            core::ptr::addr_of_mut!(ioStatusBlock),
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            0,
        ) as DWORD;
    } else {
        return ERROR_INVALID_FUNCTION;
    }
}
#[no_mangle]
pub unsafe extern "C" fn MapTypeNameToType(TypeName: LPCWSTR, cbTypeName: DWORD) -> OBJECT_TYPE {
    if TypeName.is_null() {
        return UNKNOWN_OBJECT_TYPE;
    }

    let mut i: UINT = 0;
    while i < _countof!(ObjectTypeNames) as UINT {
        let typeName: LPCWSTR = ObjectTypeNames[i as usize];
        if StrCmpNW(typeName, TypeName, (cbTypeName / core::mem::size_of::<u16>() as DWORD) as INT) == 0 {
            return i as OBJECT_TYPE;
        }
        i += 1;
    }

    return UNKNOWN_OBJECT_TYPE;
}
#[no_mangle]
pub unsafe extern "C" fn ReadRegistryValue(root: HKEY, path: PCWSTR, valueName: PCWSTR, valueData: *mut PVOID, valueLength: PDWORD) -> HRESULT {
    let mut hkey: HKEY = core::ptr::null_mut();

    let mut res: DWORD;
    if !root.is_null() {
        res = (RegOpenKeyExW(root, if *path == '\\' as u16 { path.add(1 as usize) } else { path }, 0, STANDARD_RIGHTS_READ | KEY_QUERY_VALUE, core::ptr::addr_of_mut!(hkey))) as u32;
    } else {
        res = NtOpenObject(KEY_OBJECT, core::ptr::addr_of_mut!(hkey) as PHANDLE, STANDARD_RIGHTS_READ | KEY_QUERY_VALUE, path);
    }
    if !NT_SUCCESS!(res) {
        return HRESULT_FROM_NT!(res);
    }

    res = (RegQueryValueExW(hkey, valueName, core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), valueLength)) as u32;
    if !NT_SUCCESS!(res) {
        return HRESULT_FROM_NT!(res);
    }

    if *valueLength > 0 as u32 {
        let data: PBYTE = CoTaskMemAlloc((*valueLength) as SIZE_T) as PBYTE;
        *valueData = data as PVOID;

        res = (RegQueryValueExW(hkey, valueName, core::ptr::null_mut(), core::ptr::null_mut(), data, valueLength)) as u32;
        if !NT_SUCCESS!(res) {
            CoTaskMemFree(data as PVOID);
            *valueData = core::ptr::null_mut();

            RegCloseKey(hkey);

            return HRESULT_FROM_NT!(res);
        }
    } else {
        *valueData = core::ptr::null_mut();
    }

    RegCloseKey(hkey);

    return S_OK as HRESULT;
}
#[no_mangle]
pub unsafe extern "C" fn GetNTObjectSymbolicLinkTarget(path: LPCWSTR, entryName: LPCWSTR, LinkTarget: PUNICODE_STRING) -> HRESULT {
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut buffer = [0u16; (MAX_PATH) as _];
    let mut pend: LPWSTR = buffer.as_mut_ptr();

    StringCbCopyExW(buffer.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), path, core::ptr::addr_of_mut!(pend), core::ptr::null_mut(), 0);

    if *pend.offset(-(1 as isize)) != '\\' as u16 {
        *pend = '\\' as u16;
        pend = pend.add(1);
        *pend = 0 as u16;
    }

    StringCbCatW(buffer.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), entryName);

    DbgPrint(b"GetNTObjectSymbolicLinkTarget %d\n\0".as_ptr() as *const i8, buffer.as_mut_ptr());

    (*LinkTarget).Length = 0 as u16;

    let mut err: DWORD = NtOpenObject(SYMBOLICLINK_OBJECT, core::ptr::addr_of_mut!(handle), SYMBOLIC_LINK_QUERY, buffer.as_mut_ptr());
    if !NT_SUCCESS!(err) {
        return HRESULT_FROM_NT!(err);
    }

    err = NtQuerySymbolicLinkObject(handle, LinkTarget, core::ptr::null_mut()) as DWORD;
    if !NT_SUCCESS!(err) {
        return HRESULT_FROM_NT!(err);
    }

    NtClose(handle);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn GetEnumRegistryRoot(ppil: *mut *mut IEnumIDList) -> HRESULT {
    return ShellObjectCreatorIid!(CEnumRegRoot, core::ptr::addr_of!(IID_IEnumIDList), ppil as *mut *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn GetEnumRegistryKey(path: LPCWSTR, root: HKEY, ppil: *mut *mut IEnumIDList) -> HRESULT {
    return ShellObjectCreatorInit!(CEnumRegKey, [path, root], &IID_IEnumIDList, ppil as *mut *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn GetEnumNTDirectory(path: LPCWSTR, ppil: *mut *mut IEnumIDList) -> HRESULT {
    return ShellObjectCreatorInit!(CEnumNTDirectory, [path], &IID_IEnumIDList, ppil as *mut *mut core::ffi::c_void);
}

impl RootKeyEntry {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CEnumRegRoot {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_idx = 0;
        {
        }
    }
    pub unsafe fn EnumerateNext(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_idx >= _countof!(RootKeys) as UINT {
            return S_FALSE;
        }

        let idx: usize = (*this_).m_idx as usize;
        (*this_).m_idx += 1;
        let key: *mut RootKeyEntry = core::ptr::addr_of_mut!(RootKeys[idx]);

        let name: PCWSTR = (*key).keyName;
        let cchName: DWORD = wcslen(name) as DWORD;

        let otype: REG_ENTRY_TYPE = REG_ENTRY_ROOT;

        let mut entryBufferLength: DWORD = FIELD_OFFSET!(RegPidlEntry, entryName) as DWORD
            + core::mem::size_of::<WCHAR>() as DWORD
            + cchName * core::mem::size_of::<WCHAR>() as DWORD;

        // allocate space for the terminator
        entryBufferLength += FIELD_OFFSET!(SHITEMID, abID) as DWORD;

        let entry: *mut RegPidlEntry =
            CoTaskMemAlloc((entryBufferLength) as SIZE_T) as *mut RegPidlEntry;
        if entry.is_null() {
            return E_OUTOFMEMORY;
        }

        memset(entry as *mut core::ffi::c_void, 0, entryBufferLength as usize);

        (*entry).cb = FIELD_OFFSET!(RegPidlEntry, entryName) as USHORT;
        (*entry).magic = REGISTRY_PIDL_MAGIC;
        (*entry).entryType = otype;
        (*entry).__bindgen_anon_1.rootKey = (*key).key;

        if cchName > 0 {
            (*entry).entryNameLength = (cchName * core::mem::size_of::<WCHAR>() as DWORD) as USHORT;
            StringCbCopyNW(
                core::ptr::addr_of_mut!((*entry).entryName).cast(),
                (entryBufferLength) as _,
                name,
                ((*entry).entryNameLength) as _,
            );
            (*entry).cb += (*entry).entryNameLength + core::mem::size_of::<WCHAR>() as USHORT;
        } else {
            (*entry).entryNameLength = 0 as USHORT;
            (*core::ptr::addr_of_mut!((*entry).entryName).cast::<WCHAR>().add(0 as usize)) = 0;
            (*entry).cb += core::mem::size_of::<WCHAR>() as USHORT;
        }

        if !ppidl.is_null() {
            *ppidl = entry as LPITEMIDLIST;
        }
        return S_OK;
    }
    pub unsafe fn Next(&mut self, mut celt: ULONG, mut rgelt: *mut LPITEMIDLIST, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        while {
            let __old = celt;
            celt = celt.wrapping_sub(1);
            __old
        } > 0 {
            let hr: HRESULT = (*this_).EnumerateNext(rgelt);
            if hr != S_OK {
                return hr;
            }

            if !pceltFetched.is_null() {
                *pceltFetched = (*pceltFetched).wrapping_add(1);
            }
            if !rgelt.is_null() {
                rgelt = rgelt.add(1);
            }
        }

        return S_OK;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        while celt > 0 {
            let hr: HRESULT = (*this_).EnumerateNext(core::ptr::null_mut());
            if hr < 0 {
                return hr;
            }
            if hr != S_OK {
                break;
            }
        }

        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}

impl CEnumRegKey {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_path = core::ptr::null_mut();
        (*this_).m_hkey = core::ptr::null_mut();
        (*this_).m_values = (FALSE as i32);
        (*this_).m_idx = 0;
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        RegCloseKey((*this_).m_hkey);
    }
    pub unsafe fn Initialize(&mut self, path: PCWSTR, root: HKEY) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_path = path;

        let res: DWORD;
        if root as usize != 0 {
            res = (RegOpenKeyExW(
                root,
                if *path == '\\' as u16 { path.add(1) } else { path },
                0,
                STANDARD_RIGHTS_READ | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS,
                core::ptr::addr_of_mut!((*this_).m_hkey),
            )) as u32;
        } else {
            res = NtOpenObject(
                KEY_OBJECT,
                core::ptr::addr_of_mut!((*this_).m_hkey) as PHANDLE,
                STANDARD_RIGHTS_READ | KEY_QUERY_VALUE | KEY_ENUMERATE_SUB_KEYS,
                path,
            );
        }
        if !NT_SUCCESS!(res) {
            return HRESULT_FROM_NT!(res);
        }

        return S_OK;
    }
    pub unsafe fn NextKey(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut name: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
        let mut cchName: DWORD = _countof!(name) as DWORD;

        let mut className: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
        let mut cchClass: DWORD = _countof!(className) as DWORD;

        if RegEnumKeyExW(
            (*this_).m_hkey,
            {
                let t = (*this_).m_idx;
                (*this_).m_idx += 1;
                t
            } as DWORD,
            name.as_mut_ptr(),
            &mut cchName,
            core::ptr::null_mut(),
            className.as_mut_ptr(),
            &mut cchClass,
            core::ptr::null_mut(),
        ) != 0
        {
            return S_FALSE;
        }

        name[cchName as usize] = 0;
        className[cchClass as usize] = 0;

        let otype: REG_ENTRY_TYPE = REG_ENTRY_KEY;

        let mut entryBufferLength: DWORD = core::mem::offset_of!(RegPidlEntry, entryName) as DWORD
            + core::mem::size_of::<WCHAR>() as DWORD
            + cchName * core::mem::size_of::<WCHAR>() as DWORD;

        if cchClass > 0 {
            entryBufferLength += core::mem::size_of::<WCHAR>() as DWORD
                + cchClass * core::mem::size_of::<WCHAR>() as DWORD;
        }

        // allocate space for the terminator
        entryBufferLength += core::mem::offset_of!(SHITEMID, abID) as DWORD;

        let entry: *mut RegPidlEntry =
            CoTaskMemAlloc((entryBufferLength) as SIZE_T) as *mut RegPidlEntry;
        if entry.is_null() {
            return E_OUTOFMEMORY;
        }

        memset(entry as *mut core::ffi::c_void, 0, entryBufferLength as usize);

        (*entry).cb = core::mem::offset_of!(RegPidlEntry, entryName) as USHORT;
        (*entry).magic = REGISTRY_PIDL_MAGIC;
        (*entry).entryType = otype;

        if cchName > 0 {
            (*entry).entryNameLength = (cchName * core::mem::size_of::<WCHAR>() as DWORD) as USHORT;
            StringCbCopyNW(
                core::ptr::addr_of_mut!((*entry).entryName).cast(),
                (entryBufferLength) as _,
                name.as_ptr(),
                ((*entry).entryNameLength as DWORD) as _,
            );
            (*entry).cb += (*entry).entryNameLength + core::mem::size_of::<WCHAR>() as USHORT;
        } else {
            (*entry).entryNameLength = 0;
            (*core::ptr::addr_of_mut!((*entry).entryName).cast::<WCHAR>().add(0 as usize)) = 0;
            (*entry).cb += core::mem::size_of::<WCHAR>() as USHORT;
        }

        if cchClass != 0 {
            let contentData: PWSTR = (entry as PBYTE).add((*entry).cb as usize) as PWSTR;
            let remainingSpace: DWORD = entryBufferLength - (*entry).cb as DWORD;

            (*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength = (cchClass * core::mem::size_of::<WCHAR>() as DWORD) as USHORT;
            StringCbCopyNW(
                contentData,
                (remainingSpace) as _,
                className.as_ptr(),
                ((*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength as DWORD) as _,
            );

            (*entry).cb += (*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength + core::mem::size_of::<WCHAR>() as USHORT;
        }

        if !ppidl.is_null() {
            *ppidl = entry as LPITEMIDLIST;
        }
        return S_OK;
    }
    pub unsafe fn NextValue(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut name: [WCHAR; MAX_PATH as usize] = core::mem::zeroed();
        let mut cchName: DWORD = _countof!(name) as DWORD;
        let mut type_: DWORD = 0 as DWORD;
        let mut dataSize: DWORD = 0 as DWORD;

        let idx_old: i32 = (*this_).m_idx;
        (*this_).m_idx += 1;
        if RegEnumValueW(
            (*this_).m_hkey,
            idx_old as DWORD,
            name.as_mut_ptr(),
            core::ptr::addr_of_mut!(cchName),
            0 as LPDWORD,
            core::ptr::addr_of_mut!(type_),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dataSize),
        ) != 0
        {
            return S_FALSE;
        }

        let mut otype: REG_ENTRY_TYPE = REG_ENTRY_VALUE as REG_ENTRY_TYPE;

        let mut entryBufferLength: DWORD = FIELD_OFFSET!(RegPidlEntry, entryName) as DWORD
            + core::mem::size_of::<WCHAR>() as DWORD
            + cchName * core::mem::size_of::<WCHAR>() as DWORD;

        // #define MAX_EMBEDDED_DATA 32
        let copyData: BOOL = (dataSize <= MAX_EMBEDDED_DATA as DWORD) as BOOL;
        if copyData != 0 {
            entryBufferLength += dataSize + core::mem::size_of::<WCHAR>() as DWORD;

            otype = REG_ENTRY_VALUE_WITH_CONTENT as REG_ENTRY_TYPE;
        }

        // allocate space for the terminator
        entryBufferLength += FIELD_OFFSET!(SHITEMID, abID) as DWORD;

        let entry: *mut RegPidlEntry =
            CoTaskMemAlloc((entryBufferLength) as SIZE_T) as *mut RegPidlEntry;
        if entry.is_null() {
            return E_OUTOFMEMORY;
        }

        memset(
            entry as *mut core::ffi::c_void,
            0 as i32,
            entryBufferLength as usize,
        );

        (*entry).cb = FIELD_OFFSET!(RegPidlEntry, entryName) as USHORT;
        (*entry).magic = REGISTRY_PIDL_MAGIC as USHORT;
        (*entry).entryType = otype;
        (*entry).__bindgen_anon_1.__bindgen_anon_1.contentType = type_;

        if cchName > 0 as DWORD {
            (*entry).entryNameLength = (cchName * core::mem::size_of::<WCHAR>() as DWORD) as USHORT;
            StringCbCopyNW(
                core::ptr::addr_of_mut!((*entry).entryName).cast(),
                entryBufferLength as usize,
                name.as_ptr(),
                (*entry).entryNameLength as usize,
            );
            (*entry).cb += (*entry).entryNameLength + core::mem::size_of::<WCHAR>() as USHORT;
        } else {
            (*entry).entryNameLength = 0 as USHORT;
            (*core::ptr::addr_of_mut!((*entry).entryName).cast::<WCHAR>().add(0 as usize)) = 0 as WCHAR;
            (*entry).cb += core::mem::size_of::<WCHAR>() as USHORT;
        }

        if copyData != 0 {
            let contentData: PBYTE = ((entry as PBYTE).add((*entry).cb as usize)) as PBYTE;

            (*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength = (dataSize) as u16;

            // In case it's an unterminated string, RegGetValue will add the NULL termination
            dataSize += core::mem::size_of::<WCHAR>() as DWORD;

            if RegQueryValueExW(
                (*this_).m_hkey,
                name.as_ptr(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                contentData,
                core::ptr::addr_of_mut!(dataSize),
            ) == 0
            {
                (*entry).cb += (*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength as USHORT + core::mem::size_of::<WCHAR>() as USHORT;
            } else {
                (*entry).__bindgen_anon_1.__bindgen_anon_1.contentsLength = (0 as DWORD) as u16;
                (*entry).cb += core::mem::size_of::<WCHAR>() as USHORT;
            }
        }

        if !ppidl.is_null() {
            *ppidl = entry as LPITEMIDLIST;
        }
        return S_OK;
    }
    pub unsafe fn EnumerateNext(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).m_values == 0 {
            let hr: HRESULT = (*this_).NextKey(ppidl);
            if hr != S_FALSE {
                return hr;
            }

            // switch to values.
            (*this_).m_values = (TRUE as i32);
            (*this_).m_idx = 0;
        }

        return (*this_).NextValue(ppidl);
    }
    pub unsafe fn Next(&mut self, mut celt: ULONG, mut rgelt: *mut LPITEMIDLIST, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        while {
            let __old = celt;
            celt = celt.wrapping_sub(1);
            __old
        } > 0 {
            let hr: HRESULT = (*this_).EnumerateNext(rgelt);
            if hr != S_OK {
                return hr;
            }

            if !pceltFetched.is_null() {
                (*pceltFetched) += 1;
            }
            if !rgelt.is_null() {
                rgelt = rgelt.add(1);
            }
        }

        return S_OK;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        while celt > 0 {
            let hr: HRESULT = (*this_).EnumerateNext(core::ptr::null_mut());
            if hr < 0 {
                return hr;
            }
            if hr != S_OK {
                break;
            }
        }

        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}

impl CEnumNTDirectory {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_directory = core::ptr::null_mut();
        (*this_).m_first = (TRUE as i32);
        (*this_).m_enumContext = 0;
        (*this_).m_pend = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        NtClose((*this_).m_directory);
    }
    pub unsafe fn Initialize(&mut self, path: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
            StringCbCopyExW((*this_).buffer.as_mut_ptr(), core::mem::size_of::<[WCHAR; 260 as usize]>(), path, core::ptr::addr_of_mut!((*this_).m_pend), core::ptr::null_mut(), 0);

            let err: DWORD = NtOpenObject(DIRECTORY_OBJECT, core::ptr::addr_of_mut!((*this_).m_directory), FILE_LIST_DIRECTORY, (*this_).buffer.as_mut_ptr());
            if !NT_SUCCESS!(err)
            {

                return HRESULT_FROM_NT!(err);
            }

            if *(*this_).m_pend.offset(-(1 as isize)) != '\\' as WCHAR {
                let p: PWSTR = (*this_).m_pend;
                (*this_).m_pend = p.add(1);
                *p = '\\' as WCHAR;
            }

            return S_OK;
        }
    pub unsafe fn EnumerateNext(&mut self, ppidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut dirbuffer = [0u8; 2048 as usize];
        if !NT_SUCCESS!(NtQueryDirectoryObject(
            (*this_).m_directory,
            dirbuffer.as_mut_ptr() as *mut core::ffi::c_void,
            2048,
            (TRUE) as BOOLEAN,
            (*this_).m_first as u8,
            core::ptr::addr_of_mut!((*this_).m_enumContext),
            core::ptr::null_mut(),
        )) {
            return S_FALSE;
        }

        (*this_).m_first = (FALSE as i32);

        // if ppidl is NULL, assume the caller was Skip(),
        // so we don't care about the info
        if ppidl.is_null() {
            return S_OK;
        }

        let info: POBJECT_DIRECTORY_INFORMATION = dirbuffer.as_mut_ptr() as POBJECT_DIRECTORY_INFORMATION;

        if !(*info).Name.Buffer.is_null() {
            StringCbCopyNW(
                (*this_).m_pend,
                core::mem::size_of::<[WCHAR; 260]>(),
                (*info).Name.Buffer,
                (*info).Name.Length as usize,
            );
        }

        let otype: OBJECT_TYPE = MapTypeNameToType((*info).TypeName.Buffer, (*info).TypeName.Length as DWORD);

        let mut entryBufferLength: DWORD = (core::mem::offset_of!(NtPidlEntry, entryName) + core::mem::size_of::<WCHAR>()) as DWORD;
        if !(*info).Name.Buffer.is_null() {
            entryBufferLength += (*info).Name.Length as DWORD;
        }

        if otype < 0 {
            entryBufferLength += (core::mem::offset_of!(NtPidlTypeData, typeName) + core::mem::size_of::<WCHAR>()) as DWORD;

            if !(*info).TypeName.Buffer.is_null() {
                entryBufferLength += (*info).TypeName.Length as DWORD;
            }
        }

        // allocate space for the terminator
        entryBufferLength += core::mem::offset_of!(SHITEMID, abID) as DWORD;

        let entry: *mut NtPidlEntry = CoTaskMemAlloc((entryBufferLength) as SIZE_T) as *mut NtPidlEntry;
        if entry.is_null() {
            return E_OUTOFMEMORY;
        }

        memset(entry as *mut core::ffi::c_void, 0, entryBufferLength as usize);

        (*entry).cb = core::mem::offset_of!(NtPidlEntry, entryName) as USHORT;
        (*entry).magic = NT_OBJECT_PIDL_MAGIC;
        (*entry).objectType = otype;

        if !(*info).Name.Buffer.is_null() {
            (*entry).entryNameLength = (*info).Name.Length;
            StringCbCopyNW(
                core::ptr::addr_of_mut!((*entry).entryName).cast(),
                entryBufferLength as usize,
                (*info).Name.Buffer,
                (*info).Name.Length as usize,
            );
            (*entry).cb = ((*entry).cb as usize
                + (*entry).entryNameLength as usize
                + core::mem::size_of::<WCHAR>()) as USHORT;
        } else {
            (*entry).entryNameLength = 0;
            (*core::ptr::addr_of_mut!((*entry).entryName).cast::<WCHAR>().add(0 as usize)) = 0;
            (*entry).cb = ((*entry).cb as usize + core::mem::size_of::<WCHAR>()) as USHORT;
        }

        if otype < 0 {
            let typedata: *mut NtPidlTypeData = ((entry as *mut u8).add((*entry).cb as usize)) as *mut NtPidlTypeData;
            let remainingSpace: DWORD = entryBufferLength
                - ((core::ptr::addr_of_mut!((*typedata).typeName).cast() as *mut u8 as usize - entry as usize) as DWORD);

            if !(*info).TypeName.Buffer.is_null() {
                (*typedata).typeNameLength = (*info).TypeName.Length;
                StringCbCopyNW(
                    core::ptr::addr_of_mut!((*typedata).typeName).cast(),
                    remainingSpace as usize,
                    (*info).TypeName.Buffer,
                    (*info).TypeName.Length as usize,
                );

                (*entry).cb = ((*entry).cb as usize
                    + (*typedata).typeNameLength as usize
                    + core::mem::size_of::<WCHAR>()) as USHORT;
            } else {
                (*typedata).typeNameLength = 0;
                (*core::ptr::addr_of_mut!((*typedata).typeName).cast::<WCHAR>().add(0 as usize)) = 0;
                (*entry).cb = ((*entry).cb as usize
                    + (*typedata).typeNameLength as usize
                    + core::mem::size_of::<WCHAR>()) as USHORT;
            }
        }

        *ppidl = entry as LPITEMIDLIST;

        return S_OK;
    }
    pub unsafe fn Next(&mut self, mut celt: ULONG, mut rgelt: *mut LPITEMIDLIST, pceltFetched: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pceltFetched.is_null() {
            *pceltFetched = 0;
        }

        while {
            let __old = celt;
            celt = celt.wrapping_sub(1);
            __old
        } > 0
        {
            let hr: HRESULT = (*this_).EnumerateNext(rgelt);
            if hr != S_OK {
                return hr;
            }

            if !pceltFetched.is_null() {
                *pceltFetched = (*pceltFetched).wrapping_add(1);
            }
            if !rgelt.is_null() {
                rgelt = rgelt.add(1);
            }
        }

        return S_OK;
    }
    pub unsafe fn Skip(&mut self, celt: ULONG) -> HRESULT {
        let this_: *mut Self = self;
        let mut celt = celt;
        while celt > 0 {
            let hr: HRESULT = (*this_).EnumerateNext(core::ptr::null_mut());
            if hr < 0 {
                return hr;
            }
            if hr != S_OK {
                break;
            }
        }

        return S_OK;
    }
    pub unsafe fn Reset(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn Clone(&mut self, ppenum: *mut *mut IEnumIDList,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
}
