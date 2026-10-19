use crate::prelude::*;
use crate::ntobjshex::cpp_classes::RegistryTypeNames;
use crate::ntobjshex::ntobjshex_cpp_h::g_hInstance;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_039844F90C51: [WCHAR; 4] = [0x004B, 0x0065, 0x0079, 0x0000];
pub static CPP_COM_WSTR_1EAD41622E51: [WCHAR; 30] = [0x003C, 0x0055, 0x006E, 0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x0020, 0x0076, 0x0061, 0x006C, 0x0075, 0x0065, 0x0020, 0x0074, 0x0079, 0x0070, 0x0065, 0x0020, 0x0025, 0x0064, 0x003E, 0x0000];
pub static CPP_COM_WSTR_54E2098886DA: [WCHAR; 13] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x006E, 0x0074, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_814C47FAAE3A: [WCHAR; 10] = [0x0028, 0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0029, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_8E274E0C964D: [WCHAR; 22] = [0x0028, 0x0045, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0072, 0x0065, 0x0061, 0x0064, 0x0069, 0x006E, 0x0067, 0x0020, 0x0076, 0x0061, 0x006C, 0x0075, 0x0065, 0x0029, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A75A9106944B: [WCHAR; 6] = [0x0025, 0x0030, 0x0032, 0x0078, 0x0020, 0x0000];
pub static CPP_COM_WSTR_AC75CDD1960F: [WCHAR; 12] = [0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x0078, 0x0020, 0x0028, 0x0025, 0x0064, 0x0029, 0x0000];
pub static CPP_COM_WSTR_C69083A52F11: [WCHAR; 8] = [0x0028, 0x0045, 0x006D, 0x0070, 0x0074, 0x0079, 0x0029, 0x0000];
pub static CPP_COM_WSTR_CE58CFCB03A6: [WCHAR; 6] = [0x0056, 0x0061, 0x006C, 0x0075, 0x0065, 0x0000];
pub static CPP_COM_WSTR_E86E769FA884: [WCHAR; 17] = [0x0030, 0x0078, 0x0025, 0x0030, 0x0031, 0x0036, 0x006C, 0x006C, 0x0078, 0x0020, 0x0028, 0x0025, 0x006C, 0x006C, 0x0064, 0x0029, 0x0000];
pub static CPP_COM_WSTR_F1B73E8B3335: [WCHAR; 12] = [0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static CLSID_RegistryFolder: GUID = GUID {
    Data1: 0x1c6d6e08,
    Data2: 0x2332,
    Data3: 0x4a7b,
    Data4: [0xa9, 0x4d, 0x64, 0x32, 0xdb, 0x2b, 0x5a, 0xe6],
};





















impl CRegistryFolderExtractIcon {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_pcidlFolder = core::ptr::null_mut();
        (*this_).m_pcidlChild = core::ptr::null_mut();
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pcidlFolder.is_null() {
            ILFree((*this_).m_pcidlFolder as LPITEMIDLIST);
        }
        if !(*this_).m_pcidlChild.is_null() {
            ILFree((*this_).m_pcidlChild as LPITEMIDLIST);
        }
    }
    pub unsafe fn Initialize(&mut self, ntPath: LPCWSTR, parent: PCIDLIST_ABSOLUTE, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_pcidlFolder = ILClone(parent);
        if cidl != 1 {
            return E_INVALIDARG;
        }
        (*this_).m_pcidlChild = ILClone(*apidl.add(0 as usize));
        return S_OK;
    }
    pub unsafe fn GetIconLocation(&mut self, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        let entry: *const RegPidlEntry = (*this_).m_pcidlChild as *const RegPidlEntry;

        if (((*entry).cb as usize) < core::mem::size_of::<RegPidlEntry>()) || ((*entry).magic != REGISTRY_PIDL_MAGIC as USHORT) {
            return E_INVALIDARG as HRESULT;
        }

        let flags: UINT = 0 as UINT;

        match (*entry).entryType {
            REG_ENTRY_KEY | REG_ENTRY_ROOT => {
                GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
                *piIndex = -(IDI_REGISTRYKEY as INT);
                *pwFlags = flags;
                return S_OK as HRESULT;
            }
            REG_ENTRY_VALUE | REG_ENTRY_VALUE_WITH_CONTENT => {
                GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
                *piIndex = -(IDI_REGISTRYVALUE as INT);
                *pwFlags = flags;
                return S_OK as HRESULT;
            }
            _ => {
                GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
                *piIndex = -(IDI_NTOBJECTITEM as INT);
                *pwFlags = flags;
                return S_OK as HRESULT;
            }
        }
    }
    pub unsafe fn Extract(&mut self, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
        let this_: *mut Self = self;
        return SHDefExtractIconW(pszFile, (nIconIndex) as i32, 0, phiconLarge, phiconSmall, nIconSize);
    }
}

impl CRegistryFolder {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
        let this_: *mut Self = self;
        if (*this_).__base1.m_NtPath[0] == 0 && (*this_).m_hRoot.is_null() {
            return GetEnumRegistryRoot(ppenumIDList);
        } else {
            return GetEnumRegistryKey((*this_).__base1.m_NtPath.as_mut_ptr(), (*this_).m_hRoot, ppenumIDList);
        }
    }
    pub unsafe fn InternalBindToObject__body(&mut self, path: PWSTR, info: *const RegPidlEntry, first: LPITEMIDLIST, rest: LPCITEMIDLIST, fullPidl: LPITEMIDLIST, pbcReserved: LPBC, ppsfChild: *mut *mut IShellFolder) -> HRESULT {
        let this_: *mut Self = self;
        if wcslen(((*this_).__base1.m_NtPath).as_ptr()) == 0 && (*this_).m_hRoot == core::ptr::null_mut() {
            return ShellObjectCreatorInit!(CRegistryFolder, Initialize__1, [fullPidl, CPP_COM_WSTR_96A296D224F2.as_ptr() as PWSTR, (*info).__bindgen_anon_1.rootKey], core::ptr::addr_of!(IID_IShellFolder), ppsfChild as *mut *mut core::ffi::c_void);
        }

        return ShellObjectCreatorInit!(CRegistryFolder, Initialize__1, [fullPidl, path, (*this_).m_hRoot], core::ptr::addr_of!(IID_IShellFolder), ppsfChild as *mut *mut core::ffi::c_void);
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
        let this_: *mut Self = self;
        match (iColumn) as i32 {
            REGISTRY_COLUMN_NAME => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
                return S_OK;
            }

            REGISTRY_COLUMN_TYPE => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
                return S_OK;
            }

            REGISTRY_COLUMN_VALUE => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT | SHCOLSTATE_SLOW;
                return S_OK;
            }

            _ => {}
        }

        return E_INVALIDARG;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: LPCITEMIDLIST, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::ptr::null();

        if !pidl.is_null() {
            let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            static storage: GUID = PSGUID_STORAGE;
            if IsEqualGUID!(core::ptr::addr_of!((*pscid).fmtid), core::ptr::addr_of!(storage)) != false {
                if (*pscid).pid == PID_STG_NAME {
                    if (*info).entryNameLength > 0 {
                        return MakeVariantString(pv, core::ptr::addr_of!((*info).entryName).cast() as PCWSTR);
                    }
                    return MakeVariantString(pv, CPP_COM_WSTR_814C47FAAE3A.as_ptr());
                }
                else if (*pscid).pid == PID_STG_STORAGETYPE {
                    if (*info).entryType == REG_ENTRY_ROOT {
                        return MakeVariantString(pv, CPP_COM_WSTR_039844F90C51.as_ptr());
                    }

                    if (*info).entryType == REG_ENTRY_KEY {
                        if (*info).__bindgen_anon_1.__bindgen_anon_1.contentsLength > 0 {
                            let mut td: PWSTR = ((info as PBYTE).add(
                                core::mem::offset_of!(RegPidlEntry, entryName)
                                    + (*info).entryNameLength as usize
                                    + core::mem::size_of::<WCHAR>()
                            )) as PWSTR;

                            return MakeVariantString(pv, td);
                        }
                        return MakeVariantString(pv, CPP_COM_WSTR_039844F90C51.as_ptr());
                    }

                    return MakeVariantString(pv, registry_type_name((*info).__bindgen_anon_1.__bindgen_anon_1.contentType));
                }
                else if (*pscid).pid == PID_STG_CONTENTS {
                    let mut strValueContents: PCWSTR = core::ptr::null();

                    hr = (*this_).FormatContentsForDisplay(
                        info,
                        (*this_).m_hRoot,
                        ((*this_).__base1.m_NtPath).as_ptr(),
                        core::ptr::addr_of_mut!(strValueContents),
                    );
                    if FAILED_UNEXPECTEDLY!(hr) != false {
                        return hr;
                    }

                    if hr == S_FALSE {
                        V_VT!(pv) = VT_EMPTY;
                        return S_OK;
                    }

                    hr = MakeVariantString(pv, strValueContents);

                    CoTaskMemFree(strValueContents as PVOID);

                    return hr;
                }
            }
        }

        return E_INVALIDARG;
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: LPCITEMIDLIST, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::mem::zeroed();

        if !pidl.is_null() {
            let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            match (iColumn) as i32 {
                REGISTRY_COLUMN_NAME => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    if (*info).entryNameLength as usize > 0 {
                        return MakeStrRetFromString__0(core::ptr::addr_of!((*info).entryName).cast(), ((*info).entryNameLength) as DWORD, core::ptr::addr_of_mut!((*psd).str_));
                    }
                    return MakeStrRetFromString__1(CPP_COM_WSTR_814C47FAAE3A.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                }

                REGISTRY_COLUMN_TYPE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    if (*info).entryType == REG_ENTRY_ROOT {
                        return MakeStrRetFromString__1(CPP_COM_WSTR_039844F90C51.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    }

                    if (*info).entryType == REG_ENTRY_KEY {
                        if (*info).__bindgen_anon_1.__bindgen_anon_1.contentsLength as usize > 0 {
                            let td: PWSTR = ((info as PBYTE).add(
                                (FIELD_OFFSET!(RegPidlEntry, entryName)
                                    + ((*info).entryNameLength as usize) as i32
                                    + (core::mem::size_of::<WCHAR>()) as i32) as usize
                            )) as PWSTR;

                            return MakeStrRetFromString__0(td, ((*info).__bindgen_anon_1.__bindgen_anon_1.contentsLength) as DWORD, core::ptr::addr_of_mut!((*psd).str_));
                        }

                        return MakeStrRetFromString__1(CPP_COM_WSTR_039844F90C51.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    }

                    return MakeStrRetFromString__1(registry_type_name((*info).__bindgen_anon_1.__bindgen_anon_1.contentType), core::ptr::addr_of_mut!((*psd).str_));
                }

                REGISTRY_COLUMN_VALUE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    let mut strValueContents: PCWSTR = core::ptr::null();

                    hr = (*this_).FormatContentsForDisplay(info, (*this_).m_hRoot, ((*this_).__base1.m_NtPath).as_ptr(), core::ptr::addr_of_mut!(strValueContents));
                    if FAILED_UNEXPECTEDLY!(hr) {
                        return hr;
                    }

                    if hr == S_FALSE {
                        return MakeStrRetFromString__1(CPP_COM_WSTR_C69083A52F11.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    }

                    hr = MakeStrRetFromString__1(strValueContents, core::ptr::addr_of_mut!((*psd).str_));

                    CoTaskMemFree(strValueContents as PVOID);

                    return hr;
                }

                _ => {}
            }
        } else {
            match (iColumn) as i32 {
                REGISTRY_COLUMN_NAME => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 30;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_F1B73E8B3335.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                REGISTRY_COLUMN_TYPE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 20;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_54E2098886DA.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                REGISTRY_COLUMN_VALUE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 20;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_CE58CFCB03A6.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                _ => {}
            }
        }

        return E_INVALIDARG;
    }
    pub unsafe fn MapColumnToSCID(&mut self, iColumn: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
        let this_: *mut Self = self;
        let storage: GUID = PSGUID_STORAGE;
        match (iColumn) as i32 {
            REGISTRY_COLUMN_NAME => {
                (*pscid).fmtid = storage;
                (*pscid).pid = PID_STG_NAME;
                return S_OK;
            }
            REGISTRY_COLUMN_TYPE => {
                (*pscid).fmtid = storage;
                (*pscid).pid = PID_STG_STORAGETYPE;
                return S_OK;
            }
            REGISTRY_COLUMN_VALUE => {
                (*pscid).fmtid = storage;
                (*pscid).pid = PID_STG_CONTENTS;
                return S_OK;
            }
            _ => {}
        }
        return E_INVALIDARG;
    }
    pub unsafe fn Initialize__0(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base1.m_shellPidl = ILClone(pidl);
        (*this_).m_hRoot = core::ptr::null_mut();

        StringCbCopyW(core::ptr::addr_of_mut!((*this_).__base1.m_NtPath) as PWSTR, core::mem::size_of_val(&(*this_).__base1.m_NtPath), CPP_COM_WSTR_96A296D224F2.as_ptr());
        return S_OK;
    }
    pub unsafe fn Initialize__1(&mut self, pidl: PCIDLIST_ABSOLUTE, ntPath: PCWSTR, hRoot: HKEY) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base1.m_shellPidl = ILClone(pidl);
        (*this_).m_hRoot = hRoot;

        StringCbCopyW((*this_).__base1.m_NtPath.as_mut_ptr(), core::mem::size_of_val(&*core::ptr::addr_of!((*this_).__base1.m_NtPath)), ntPath);
        return S_OK;
    }
    pub unsafe fn CompareIDs__body(&mut self, lParam: LPARAM, first: *const RegPidlEntry, second: *const RegPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = 0;

        let mut sortMode: DWORD = (lParam as DWORD) & 0xFFFF0000;
        let mut column: DWORD = (lParam as DWORD) & 0x0000FFFF;

        if sortMode == SHCIDS_ALLFIELDS {
            if column != 0 {
                return E_INVALIDARG;
            }

            let minsize: i32 = min!((*first).cb as i32, (*second).cb as i32);
            hr = MAKE_COMPARE_HRESULT!(memcmp(
                second as *const core::ffi::c_void,
                first as *const core::ffi::c_void,
                minsize as usize,
            ));
            if hr != S_EQUAL {
                return hr;
            }

            return MAKE_COMPARE_HRESULT!((*second).cb as i32 - (*first).cb as i32);
        }

        // switch (column): every case returns, so an if/else chain is the faithful lowering
        if column == (REGISTRY_COLUMN_NAME as DWORD) {
            return (*this_).__base1.CompareName(lParam, first, second);
        } else if column == (REGISTRY_COLUMN_TYPE as DWORD) {
            {
                if (*first).entryType != (*second).entryType {
                    // REG_ENTRY_TYPE is an int enum in C++: the difference is signed, never a wrap.
                    return MAKE_COMPARE_HRESULT!((*second).entryType as i32 - (*first).entryType as i32);
                }

                if (*first).entryType == REG_ENTRY_KEY {
                    if (*first).__bindgen_anon_1.__bindgen_anon_1.contentsLength == 0 || (*second).__bindgen_anon_1.__bindgen_anon_1.contentsLength == 0 {
                        return if (*first).__bindgen_anon_1.__bindgen_anon_1.contentsLength == 0 {
                            S_GREATERTHAN
                        } else {
                            (S_LESSTHAN as i32)
                        };
                    }

                    let mut firstKey: PWSTR = ((first as PBYTE).add(
                        core::mem::offset_of!(RegPidlEntry, entryName) + (*first).entryNameLength as usize + core::mem::size_of::<WCHAR>(),
                    )) as PWSTR;
                    let mut secondKey: PWSTR = ((second as PBYTE).add(
                        core::mem::offset_of!(RegPidlEntry, entryName) + (*second).entryNameLength as usize + core::mem::size_of::<WCHAR>(),
                    )) as PWSTR;
                    return MAKE_COMPARE_HRESULT!(lstrcmpW(firstKey, secondKey));
                }

                return (*this_).__base1.CompareName(lParam, first, second);
            }
        } else if column == (REGISTRY_COLUMN_VALUE as DWORD) {
            {
                let mut firstContent: PCWSTR = core::ptr::null();
                let mut secondContent: PCWSTR = core::ptr::null();

                if FAILED_UNEXPECTEDLY!((*this_).FormatContentsForDisplay(
                    first,
                    (*this_).m_hRoot,
                    ((*this_).__base1.m_NtPath).as_ptr(),
                    core::ptr::addr_of_mut!(firstContent),
                )) {
                    return E_INVALIDARG;
                }

                if FAILED_UNEXPECTEDLY!((*this_).FormatContentsForDisplay(
                    second,
                    (*this_).m_hRoot,
                    ((*this_).__base1.m_NtPath).as_ptr(),
                    core::ptr::addr_of_mut!(secondContent),
                )) {
                    return E_INVALIDARG;
                }

                hr = MAKE_COMPARE_HRESULT!(lstrcmpW(firstContent, secondContent));

                CoTaskMemFree(firstContent as LPVOID);
                CoTaskMemFree(secondContent as LPVOID);

                return hr;
            }
        }

        DbgPrint(b"Unsupported sorting mode.\n\0".as_ptr() as *const i8);
        return E_INVALIDARG;
    }
    pub unsafe fn ConvertAttributes__body(&mut self, entry: *const RegPidlEntry, inMask: PULONG) -> ULONG {
        let this_: *mut Self = self;
        let mut mask: ULONG = if !inMask.is_null() { *inMask } else { 0xFFFFFFFF };
        let mut flags: ULONG = 0;

        if ((*entry).entryType == REG_ENTRY_KEY) ||
            ((*entry).entryType == REG_ENTRY_ROOT) {
            flags |= SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
        }

        return flags & mask;
    }
    pub unsafe fn IsFolder__body(&mut self, info: *const RegPidlEntry) -> BOOL {
        let this_: *mut Self = self;
        return (((*info).entryType == REG_ENTRY_KEY) || ((((*info).entryType == REG_ENTRY_ROOT) as BOOL) != 0)) as i32;
    }
    pub unsafe fn GetInfoFromPidl__body(&mut self, pcidl: LPCITEMIDLIST, pentry: *mut *const RegPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        if pcidl.is_null() {
            DbgPrint(b"PCIDL is NULL\n\0".as_ptr() as *const i8);
            return E_INVALIDARG;
        }

        let entry: *mut RegPidlEntry = core::ptr::addr_of!((*pcidl).mkid) as *mut RegPidlEntry;
        if (*entry).cb < core::mem::size_of::<RegPidlEntry>() as _ {
            DbgPrint(
                b"PCIDL too small %l (required %l)\n\0".as_ptr() as *const i8,
                (*entry).cb as u32,
                core::mem::size_of::<RegPidlEntry>() as u32,
            );
            return E_INVALIDARG;
        }

        if (*entry).magic != REGISTRY_PIDL_MAGIC {
            DbgPrint(
                b"PCIDL magic mismatch %04x (expected %04x)\n\0".as_ptr() as *const i8,
                (*entry).magic as u32,
                REGISTRY_PIDL_MAGIC as u32,
            );
            return E_INVALIDARG;
        }

        *pentry = entry as *const RegPidlEntry;
        return S_OK;
    }
    pub unsafe fn FormatValueData(&mut self, contentType: DWORD, td: PVOID, contentsLength: DWORD, strContents: *mut PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        match contentType {
            REG_NONE => {
                let strTodo: PCWSTR = CPP_COM_WSTR_96A296D224F2.as_ptr();
                let bufferLength: DWORD = ((wcslen(strTodo) as DWORD) + 1) * (core::mem::size_of::<WCHAR>() as DWORD);
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                StringCbCopyW(strValue, (bufferLength) as _, strTodo);
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            REG_SZ | REG_EXPAND_SZ => {
                let strValue: PWSTR = CoTaskMemAlloc((contentsLength + (core::mem::size_of::<WCHAR>() as DWORD)) as SIZE_T) as PWSTR;
                StringCbCopyNW(strValue, (contentsLength + (core::mem::size_of::<WCHAR>() as DWORD)) as _, td as LPCWSTR, (contentsLength) as _);
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            REG_MULTI_SZ => {
                let separator: PCWSTR = CPP_COM_WSTR_8D71B3FAAB82.as_ptr(); // To match regedit
                let sepChars: usize = wcslen(separator) as usize;
                let mut strings: i32 = 0;
                let mut stringChars: i32 = 0;

                let mut strData: PCWSTR = td as PCWSTR;
                while *strData != 0 {
                    let len: usize = wcslen(strData) as usize;
                    stringChars += len as i32;
                    strData = strData.add(len + 1 as usize); // Skips null-terminator
                    strings += 1;
                }

                let cch: i32 = stringChars + (strings - 1) * (sepChars as i32) + 1;

                let strValue: PWSTR = CoTaskMemAlloc(((cch as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T) as PWSTR;

                *strValue.add(0 as usize) = 0;

                strData = td as PCWSTR;
                while *strData != 0 {
                    StrCatW(strValue, strData);
                    strData = strData.add((wcslen(strData) as usize) + 1 as usize);
                    if *strData != 0 {
                        StrCatW(strValue, separator);
                    }
                }

                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            REG_DWORD => {
                let bufferLength: DWORD = 64 * (core::mem::size_of::<WCHAR>() as DWORD);
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                StringCbPrintfW!(strValue, bufferLength, CPP_COM_WSTR_AC75CDD1960F.as_ptr(),
                    *(td as *const DWORD), *(td as *const DWORD));
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            REG_QWORD => {
                let bufferLength: DWORD = 64 * (core::mem::size_of::<WCHAR>() as DWORD);
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                StringCbPrintfW!(strValue, bufferLength, CPP_COM_WSTR_E86E769FA884.as_ptr(),
                    *(td as *const LARGE_INTEGER), (*(td as *const LARGE_INTEGER)).QuadPart);
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            REG_BINARY => {
                let mut bufferLength: DWORD = (contentsLength * 3 + 1) * (core::mem::size_of::<WCHAR>() as DWORD);
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                let mut strTemp: PWSTR = strValue;
                let data: PBYTE = td as PBYTE;
                let mut i: DWORD = 0;
                while i < contentsLength {
                    StringCbPrintfW!(strTemp, bufferLength, CPP_COM_WSTR_A75A9106944B.as_ptr(), *data.add(i as usize) as core::ffi::c_uint);
                    strTemp = strTemp.add(3 as usize);
                    bufferLength -= 3;
                    i += 1;
                }
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            _ => {
                let strFormat: PCWSTR = CPP_COM_WSTR_1EAD41622E51.as_ptr();
                let bufferLength: DWORD = ((wcslen(strFormat) as DWORD) + 15) * (core::mem::size_of::<WCHAR>() as DWORD);
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                StringCbPrintfW!(strValue, bufferLength, strFormat, contentType);
                *strContents = strValue as PCWSTR;
                return S_OK;
            }
        }
    }
    pub unsafe fn FormatContentsForDisplay(&mut self, info: *const RegPidlEntry, rootKey: HKEY, ntPath: LPCWSTR, strContents: *mut PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        let td: PVOID = (info as PBYTE).add(core::mem::offset_of!(RegPidlEntry, entryName) + (*info).entryNameLength as usize + core::mem::size_of::<WCHAR>()) as PVOID;

        if (*info).entryType == REG_ENTRY_VALUE_WITH_CONTENT {
            if (*info).__bindgen_anon_1.__bindgen_anon_1.contentsLength > 0 {
                return (*this_).FormatValueData((*info).__bindgen_anon_1.__bindgen_anon_1.contentType, td, ((*info).__bindgen_anon_1.__bindgen_anon_1.contentsLength) as DWORD, strContents);
            }
        } else if (*info).entryType == REG_ENTRY_VALUE {
            let mut valueData: PVOID = core::ptr::null_mut();
            let mut valueLength: DWORD = 0;
            let mut hr: HRESULT = ReadRegistryValue(rootKey, ntPath, core::ptr::addr_of!((*info).entryName).cast(), core::ptr::addr_of_mut!(valueData), core::ptr::addr_of_mut!(valueLength));
            if FAILED_UNEXPECTEDLY!(hr) {
                let strEmpty: PCWSTR = CPP_COM_WSTR_8E274E0C964D.as_ptr();
                let bufferLength: DWORD = ((wcslen(strEmpty) + 1) * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as _;
                let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
                StringCbCopyW(strValue, (bufferLength) as _, strEmpty);
                *strContents = strValue as PCWSTR;
                return S_OK;
            }

            if valueLength > 0 {
                hr = (*this_).FormatValueData((*info).__bindgen_anon_1.__bindgen_anon_1.contentType, valueData, valueLength, strContents);

                CoTaskMemFree(valueData);

                return hr;
            }
        } else {
            let strEmpty: PCWSTR = CPP_COM_WSTR_96A296D224F2.as_ptr();
            let bufferLength: DWORD = ((wcslen(strEmpty) + 1) * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as _;
            let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
            StringCbCopyW(strValue, (bufferLength) as _, strEmpty);
            *strContents = strValue as PCWSTR;
            return S_OK;
        }

        let strEmpty: PCWSTR = CPP_COM_WSTR_C69083A52F11.as_ptr();
        let bufferLength: DWORD = ((wcslen(strEmpty) + 1) * (core::mem::size_of::<WCHAR>() as DWORD) as usize) as _;
        let strValue: PWSTR = CoTaskMemAlloc((bufferLength) as SIZE_T) as PWSTR;
        StringCbCopyW(strValue, (bufferLength) as _, strEmpty);
        *strContents = strValue as PCWSTR;
        return S_OK;
    }
}

/// `RegistryTypeNames[contentType]`, held to the table. The type is whatever RegEnumValueW
/// returned, and anything past REG_QWORD is an out-of-range read in the C++ and a panic in
/// explorer here; such a value shows an empty type name instead.
/// Exceeds the C: regfolder.cpp indexes the table unchecked.
unsafe fn registry_type_name(content_type: DWORD) -> LPCWSTR {
    static EMPTY: [u16; 1] = [0];
    let names = &*core::ptr::addr_of!(RegistryTypeNames);
    names.get(content_type as usize).copied().unwrap_or(EMPTY.as_ptr())
}
