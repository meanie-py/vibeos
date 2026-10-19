use crate::prelude::*;
use crate::ntobjshex::cpp_classes::ObjectTypeNames;
use crate::ntobjshex::ntobjshex_cpp_h::g_hInstance;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_606312B79D54: [WCHAR; 15] = [0x0053, 0x0079, 0x006D, 0x006C, 0x0069, 0x006E, 0x006B, 0x0020, 0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x0000];
pub static CPP_COM_WSTR_93B23D21E5F8: [WCHAR; 82] = [0x003A, 0x003A, 0x007B, 0x0032, 0x0030, 0x0044, 0x0030, 0x0034, 0x0046, 0x0045, 0x0030, 0x002D, 0x0033, 0x0041, 0x0045, 0x0041, 0x002D, 0x0031, 0x0030, 0x0036, 0x0039, 0x002D, 0x0041, 0x0032, 0x0044, 0x0038, 0x002D, 0x0030, 0x0038, 0x0030, 0x0030, 0x0032, 0x0042, 0x0033, 0x0030, 0x0033, 0x0030, 0x0039, 0x0044, 0x007D, 0x005C, 0x003A, 0x003A, 0x007B, 0x0038, 0x0034, 0x0035, 0x0042, 0x0030, 0x0046, 0x0042, 0x0032, 0x002D, 0x0036, 0x0036, 0x0045, 0x0030, 0x002D, 0x0034, 0x0031, 0x0036, 0x0042, 0x002D, 0x0038, 0x0046, 0x0039, 0x0031, 0x002D, 0x0033, 0x0031, 0x0034, 0x0045, 0x0032, 0x0033, 0x0046, 0x0037, 0x0043, 0x0031, 0x0032, 0x0044, 0x007D, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_B93EC80933D9: [WCHAR; 12] = [0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000];
pub static CPP_COM_WSTR_F1B73E8B3335: [WCHAR; 12] = [0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0020, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_F3CDEC9800CB: [WCHAR; 8] = [0x0055, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static CLSID_NtObjectFolder: GUID = GUID {
    Data1: 0x845b0fb2,
    Data2: 0x66e0,
    Data3: 0x416b,
    Data4: [0x8f, 0x91, 0x31, 0x4e, 0x23, 0xf7, 0xc1, 0x2d],
};

pub static GUID_NtObjectColumns: GUID = GUID {
    Data1: 0xf4c430c3,
    Data2: 0x3a8d,
    Data3: 0x4b56,
    Data4: [0xa0, 0x18, 0xe5, 0x98, 0xda, 0x60, 0xc2, 0xe0],
};





















impl CNtObjectFolderExtractIcon {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_NtPath = core::ptr::null_mut();
        (*this_).m_pcidlChild = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_pcidlChild.is_null() {
            ILFree((*this_).m_pcidlChild as LPITEMIDLIST);
        }
    }
    pub unsafe fn Initialize(&mut self, ntPath: LPCWSTR, parent: PCIDLIST_ABSOLUTE, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_NtPath = ntPath;
        if cidl != 1 {
            return E_INVALIDARG;
        }
        (*this_).m_pcidlChild = ILClone(*apidl.add(0 as usize));
        return S_OK;
    }
    pub unsafe fn GetIconLocation(&mut self, uFlags: UINT, szIconFile: LPWSTR, cchMax: UINT, piIndex: *mut INT, pwFlags: *mut UINT) -> HRESULT {
        let this_: *mut Self = self;
        let entry: *const NtPidlEntry = (*this_).m_pcidlChild as *const NtPidlEntry;

        if (((*entry).cb as usize) < core::mem::size_of::<NtPidlEntry>()) || ((*entry).magic != NT_OBJECT_PIDL_MAGIC) {
            return E_INVALIDARG;
        }

        let flags: UINT = 0;

        let objectType = (*entry).objectType;
        if objectType == DIRECTORY_OBJECT || objectType == SYMBOLICLINK_OBJECT {
            GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
            *piIndex = -((if (uFlags & GIL_OPENICON) != 0 { IDI_NTOBJECTDIROPEN } else { IDI_NTOBJECTDIR }) as INT);
            *pwFlags = flags;
            return S_OK;
        } else if objectType == DEVICE_OBJECT {
            GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
            *piIndex = -(IDI_NTOBJECTDEVICE as INT);
            *pwFlags = flags;
            return S_OK;
        } else if objectType == PORT_OBJECT {
            GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
            *piIndex = -(IDI_NTOBJECTPORT as INT);
            *pwFlags = flags;
            return S_OK;
        } else if objectType == KEY_OBJECT {
            GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
            *piIndex = -(IDI_REGISTRYKEY as INT);
            *pwFlags = flags;
            return S_OK;
        } else {
            GetModuleFileNameW(g_hInstance, szIconFile, cchMax);
            *piIndex = -(IDI_NTOBJECTITEM as INT);
            *pwFlags = flags;
            return S_OK;
        }
    }
    pub unsafe fn Extract(&mut self, pszFile: LPCWSTR, nIconIndex: UINT, phiconLarge: *mut HICON, phiconSmall: *mut HICON, nIconSize: UINT) -> HRESULT {
        let this_: *mut Self = self;
        return SHDefExtractIconW(pszFile, (nIconIndex) as i32, 0, phiconLarge, phiconSmall, nIconSize);
    }
}

impl CNtObjectFolder {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base1.__ctor_0();
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, grfFlags: SHCONTF, ppenumIDList: *mut *mut IEnumIDList) -> HRESULT {
        let this_: *mut Self = self;
        return GetEnumNTDirectory(((*this_).__base1.m_NtPath).as_ptr(), ppenumIDList);
    }
    pub unsafe fn InternalBindToObject__body(&mut self, path: PWSTR, info: *const NtPidlEntry, first: LPITEMIDLIST, rest: LPCITEMIDLIST, fullPidl: LPITEMIDLIST, pbcReserved: LPBC, ppsfChild: *mut *mut IShellFolder) -> HRESULT {
        let this_: *mut Self = self;

        if (*info).objectType == KEY_OBJECT
        {
            return ShellObjectCreatorInit!(CRegistryFolder, Initialize__1, [fullPidl, path, core::ptr::null_mut()], core::ptr::addr_of!(IID_IShellFolder), ppsfChild as *mut *mut core::ffi::c_void);
        }

        return ShellObjectCreatorInit!(CNtObjectFolder, Initialize__1, [fullPidl, path], core::ptr::addr_of!(IID_IShellFolder), ppsfChild as *mut *mut core::ffi::c_void);
    }
    pub unsafe fn ResolveSymLink__body(&mut self, info: *const NtPidlEntry, fullPidl: *mut LPITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut wbLink: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut link: _UNICODE_STRING = core::mem::zeroed();
        RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(link), wbLink.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>() as u16);

        *fullPidl = core::ptr::null_mut();

        let mut hr: HRESULT = GetNTObjectSymbolicLinkTarget(((*this_).__base1.m_NtPath).as_ptr(), core::ptr::addr_of!((*info).entryName).cast(), core::ptr::addr_of_mut!(link));
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        let mut path: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if link.Length == 0 {
            return E_UNEXPECTED;
        }

        if *link.Buffer.add(1) == 0x003Au16 && ((isalphaW!(*link.Buffer.add(0))) != 0) {
            StringCbCopyNW(path.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), link.Buffer, link.Length as usize);

            let mut psfDesktop: CComPtr<IShellFolder> = CComPtr::new();
            hr = SHGetDesktopFolder(psfDesktop.GetAddressOf());
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            return (*psfDesktop.Get()).ParseDisplayName(core::ptr::null_mut(), core::ptr::null_mut(), path.as_mut_ptr(), core::ptr::null_mut(), fullPidl, core::ptr::null_mut());
        }

        StringCbCopyW(path.as_mut_ptr(), core::mem::size_of::<[u16; (MAX_PATH) as _]>(), CPP_COM_WSTR_93B23D21E5F8.as_ptr());
        PathAppend(path.as_mut_ptr(), link.Buffer);

        let mut psfDesktop: CComPtr<IShellFolder> = CComPtr::new();
        hr = SHGetDesktopFolder(psfDesktop.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        let mut pidl: LPITEMIDLIST = core::ptr::null_mut();

        hr = (*psfDesktop.Get()).ParseDisplayName(core::ptr::null_mut(), core::ptr::null_mut(), path.as_mut_ptr(), core::ptr::null_mut(), core::ptr::addr_of_mut!(pidl), core::ptr::null_mut());
        if hr < 0 {
            return hr;
        }

        *fullPidl = pidl;

        DumpIdList(pidl);

        return S_OK;
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
        let this_: *mut Self = self;
        match (iColumn) as i32 {
            NTOBJECT_COLUMN_NAME => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
                return S_OK;
            }

            NTOBJECT_COLUMN_TYPE => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT;
                return S_OK;
            }

            NTOBJECT_COLUMN_LINKTARGET => {
                *pcsFlags = SHCOLSTATE_TYPE_STR | SHCOLSTATE_ONBYDEFAULT | SHCOLSTATE_SLOW;
                return S_OK;
            }

            _ => {}
        }

        return E_INVALIDARG;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: LPCITEMIDLIST, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        if !pidl.is_null() {
            let hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            static storage: GUID = PSGUID_STORAGE;
            if IsEqualGUID!(core::ptr::addr_of!((*pscid).fmtid), &storage) {
                if (*pscid).pid == PID_STG_NAME {
                    return MakeVariantString(pv, core::ptr::addr_of!((*info).entryName).cast::<WCHAR>());
                } else if (*pscid).pid == PID_STG_STORAGETYPE {
                    if (*info).objectType < 0 {
                        let td: *const NtPidlTypeData = (info as PBYTE)
                            .add(
                                (FIELD_OFFSET!(NtPidlEntry, entryName)
                                    + ((*info).entryNameLength as usize) as i32
                                    + (core::mem::size_of::<WCHAR>()) as i32) as usize,
                            ) as *const NtPidlTypeData;

                        if (*td).typeNameLength > 0 {
                            return MakeVariantString(pv, core::ptr::addr_of!((*td).typeName).cast::<WCHAR>() as *mut WCHAR);
                        } else {
                            return MakeVariantString(pv, CPP_COM_WSTR_F3CDEC9800CB.as_ptr());
                        }
                    } else {
                        return MakeVariantString(pv, ObjectTypeNames[(*info).objectType as usize]);
                    }
                }
            } else if IsEqualGUID!(core::ptr::addr_of!((*pscid).fmtid), &GUID_NtObjectColumns) {
                if (*pscid).pid == (NTOBJECT_COLUMN_LINKTARGET as u32) && (*info).objectType == SYMBOLICLINK_OBJECT {
                    let mut wbLink: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                    let mut link: _UNICODE_STRING = core::mem::zeroed();
                    RtlInitEmptyUnicodeString(
                        core::ptr::addr_of_mut!(link),
                        wbLink.as_mut_ptr(),
                        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as USHORT,
                    );

                    let hr: HRESULT = GetNTObjectSymbolicLinkTarget(
                        ((*this_).__base1.m_NtPath).as_ptr(),
                        core::ptr::addr_of!((*info).entryName).cast::<WCHAR>() as *mut WCHAR,
                        core::ptr::addr_of_mut!(link),
                    );

                    if !FAILED_UNEXPECTEDLY!(hr) && link.Length > 0 {
                        return MakeVariantString(pv, link.Buffer);
                    }
                }

                V_VT!(pv) = VT_EMPTY;
                return S_OK;
            }
        }

        return E_INVALIDARG;
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: LPCITEMIDLIST, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        if !pidl.is_null() {
            let hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            match (iColumn) as i32 {
                NTOBJECT_COLUMN_NAME => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    MakeStrRetFromString__0(core::ptr::addr_of!((*info).entryName).cast(), ((*info).entryNameLength) as DWORD, core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                NTOBJECT_COLUMN_TYPE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    if (*info).objectType < 0 {
                        let td: *mut NtPidlTypeData = ((info as *const u8).add(
                            core::mem::offset_of!(NtPidlEntry, entryName)
                                + (*info).entryNameLength as usize
                                + core::mem::size_of::<WCHAR>(),
                        )) as *mut NtPidlTypeData;

                        if (*td).typeNameLength > 0 {
                            MakeStrRetFromString__0(core::ptr::addr_of!((*td).typeName).cast(), ((*td).typeNameLength) as DWORD, core::ptr::addr_of_mut!((*psd).str_));
                        } else {
                            MakeStrRetFromString__1(CPP_COM_WSTR_F3CDEC9800CB.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                        }
                    } else {
                        MakeStrRetFromString__1(ObjectTypeNames[(*info).objectType as usize], core::ptr::addr_of_mut!((*psd).str_));
                    }
                    return S_OK;
                }

                NTOBJECT_COLUMN_LINKTARGET => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);

                    if (*info).objectType == SYMBOLICLINK_OBJECT {
                        let mut wbLink: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
                        let mut link: _UNICODE_STRING = core::mem::zeroed();
                        RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!(link), wbLink.as_mut_ptr(), core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as _);

                        let hr: HRESULT = GetNTObjectSymbolicLinkTarget(((*this_).__base1.m_NtPath).as_ptr(), core::ptr::addr_of!((*info).entryName).cast(), core::ptr::addr_of_mut!(link));

                        if !FAILED_UNEXPECTEDLY!(hr) && link.Length > 0 {
                            MakeStrRetFromString__0(link.Buffer, (link.Length) as DWORD, core::ptr::addr_of_mut!((*psd).str_));
                            return S_OK;
                        }
                    }

                    MakeStrRetFromString__1(CPP_COM_WSTR_96A296D224F2.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                _ => {}
            }
        } else {
            match (iColumn) as i32 {
                NTOBJECT_COLUMN_NAME => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 30;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_F1B73E8B3335.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                NTOBJECT_COLUMN_TYPE => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 20;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_B93EC80933D9.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
                    return S_OK;
                }

                NTOBJECT_COLUMN_LINKTARGET => {
                    (*psd).fmt = (LVCFMT_LEFT as i32);
                    (*psd).cxChar = 30;

                    // TODO: Make localizable
                    MakeStrRetFromString__1(CPP_COM_WSTR_606312B79D54.as_ptr(), core::ptr::addr_of_mut!((*psd).str_));
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
            NTOBJECT_COLUMN_NAME => {
                (*pscid).fmtid = storage;
                (*pscid).pid = PID_STG_NAME;
                return S_OK;
            }

            NTOBJECT_COLUMN_TYPE => {
                (*pscid).fmtid = storage;
                (*pscid).pid = PID_STG_STORAGETYPE;
                return S_OK;
            }

            NTOBJECT_COLUMN_LINKTARGET => {
                (*pscid).fmtid = GUID_NtObjectColumns;
                (*pscid).pid = (NTOBJECT_COLUMN_LINKTARGET as u32);
                return S_OK;
            }

            _ => {}
        }
        return E_INVALIDARG;
    }
    pub unsafe fn Initialize__0(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base1.m_shellPidl = ILClone(pidl);

        StringCbCopyW(core::ptr::addr_of_mut!((*this_).__base1.m_NtPath) as PWSTR, core::mem::size_of_val(&(*this_).__base1.m_NtPath), CPP_COM_WSTR_B7FC02907B1F.as_ptr());

        return S_OK;
    }
    pub unsafe fn Initialize__1(&mut self, pidl: PCIDLIST_ABSOLUTE, ntPath: PCWSTR) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).__base1.m_shellPidl = ILClone(pidl);

        StringCbCopyW((*this_).__base1.m_NtPath.as_mut_ptr(), core::mem::size_of_val(&(*this_).__base1.m_NtPath), ntPath);

        return S_OK;
    }
    pub unsafe fn CompareIDs__body(&mut self, lParam: LPARAM, first: *const NtPidlEntry, second: *const NtPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;

        let sortMode: DWORD = (lParam as DWORD) & 0xFFFF0000;
        let column: DWORD = (lParam as DWORD) & 0x0000FFFF;

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

        if column == (NTOBJECT_COLUMN_NAME as DWORD) {
            return (*this_).__base1.CompareName(lParam, first, second);
        }

        if column == (NTOBJECT_COLUMN_TYPE as DWORD) {
            return MAKE_COMPARE_HRESULT!((*second).objectType as i32 - (*first).objectType as i32);
        }

        if column == (NTOBJECT_COLUMN_LINKTARGET as DWORD) {
            if (*first).objectType != SYMBOLICLINK_OBJECT && (*second).objectType != SYMBOLICLINK_OBJECT {
                return (*this_).__base1.CompareName(lParam, first, second);
            }

            if (*first).objectType != SYMBOLICLINK_OBJECT || (*second).objectType != SYMBOLICLINK_OBJECT {
                return if (*first).objectType != SYMBOLICLINK_OBJECT {
                    S_GREATERTHAN
                } else {
                    (S_LESSTHAN as i32)
                };
            }

            let mut wbLink1: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut wbLink2: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut firstLink: _UNICODE_STRING = core::mem::zeroed();
            let mut secondLink: _UNICODE_STRING = core::mem::zeroed();
            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!(firstLink),
                wbLink1.as_mut_ptr(),
                core::mem::size_of::<[u16; (MAX_PATH) as _]>() as _,
            );

            if FAILED_UNEXPECTEDLY!(GetNTObjectSymbolicLinkTarget(
                ((*this_).__base1.m_NtPath).as_ptr(),
                core::ptr::addr_of!((*first).entryName).cast(),
                core::ptr::addr_of_mut!(firstLink),
            )) {
                return E_INVALIDARG;
            }

            RtlInitEmptyUnicodeString(
                core::ptr::addr_of_mut!(secondLink),
                wbLink2.as_mut_ptr(),
                core::mem::size_of::<[u16; (MAX_PATH) as _]>() as _,
            );

            if FAILED_UNEXPECTEDLY!(GetNTObjectSymbolicLinkTarget(
                ((*this_).__base1.m_NtPath).as_ptr(),
                core::ptr::addr_of!((*second).entryName).cast(),
                core::ptr::addr_of_mut!(secondLink),
            )) {
                return E_INVALIDARG;
            }

            return MAKE_COMPARE_HRESULT!(RtlCompareUnicodeString(
                core::ptr::addr_of_mut!(firstLink),
                core::ptr::addr_of_mut!(secondLink),
                (TRUE) as BOOLEAN,
            ));
        }

        DbgPrint(b"Unsupported sorting mode.\n\0".as_ptr() as *const i8);
        return E_INVALIDARG;
    }
    pub unsafe fn ConvertAttributes__body(&mut self, entry: *const NtPidlEntry, inMask: PULONG) -> ULONG {
        let this_: *mut Self = self;
        let mut mask: ULONG = if !inMask.is_null() { *inMask } else { 0xFFFFFFFF };
        let mut flags: ULONG = SFGAO_HASPROPSHEET | SFGAO_CANLINK;

        if (*entry).objectType == DIRECTORY_OBJECT {
            flags |= SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
        }

        if (*entry).objectType == SYMBOLICLINK_OBJECT {
            flags |= SFGAO_LINK | SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
        }

        if (*entry).objectType == KEY_OBJECT {
            flags |= SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
        }

        return flags & mask;
    }
    pub unsafe fn IsFolder__body(&mut self, info: *const NtPidlEntry) -> BOOL {
        let this_: *mut Self = self;
        return if ((*info).objectType == DIRECTORY_OBJECT) ||
            ((*info).objectType == SYMBOLICLINK_OBJECT) ||
            ((*info).objectType == KEY_OBJECT) { 1 } else { 0 };
    }
    pub unsafe fn IsSymLink__body(&mut self, info: *const NtPidlEntry) -> BOOL {
        let this_: *mut Self = self;
        return ((*info).objectType == SYMBOLICLINK_OBJECT) as BOOL;
    }
    pub unsafe fn GetInfoFromPidl__body(&mut self, pcidl: LPCITEMIDLIST, pentry: *mut *const NtPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        if pcidl.is_null() {
            DbgPrint(b"PCIDL is NULL\n\0".as_ptr() as *const i8);
            return E_INVALIDARG;
        }

        let mut entry: *mut NtPidlEntry = core::ptr::addr_of!((*pcidl).mkid) as *mut NtPidlEntry;
        if ((*entry).cb as usize) < core::mem::size_of::<NtPidlEntry>() {
            DbgPrint(
                b"PCIDL too small %l (required %l)\n\0".as_ptr() as *const i8,
                (*entry).cb as u32,
                core::mem::size_of::<NtPidlEntry>() as u32,
            );
            return E_INVALIDARG;
        }

        if (*entry).magic != NT_OBJECT_PIDL_MAGIC {
            DbgPrint(
                b"PCIDL magic mismatch %04x (expected %04x)\n\0".as_ptr() as *const i8,
                (*entry).magic as u32,
                NT_OBJECT_PIDL_MAGIC as u32,
            );
            return E_INVALIDARG;
        }

        *pentry = entry as *const NtPidlEntry;
        return S_OK;
    }
}
