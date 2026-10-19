use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_B7FC02907B1F: [WCHAR; 2] = [0x005C, 0x0000];
pub static CPP_COM_WSTR_B94CB911C87A: [WCHAR; 22] = [0x004E, 0x0054, 0x004F, 0x0062, 0x006A, 0x0053, 0x0068, 0x0045, 0x0078, 0x002E, 0x004E, 0x0054, 0x0044, 0x0069, 0x0072, 0x0065, 0x0063, 0x0074, 0x006F, 0x0072, 0x0079, 0x0000];
pub static CPP_COM_WSTR_EB4CE2024A43: [WCHAR; 7] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0000];

use core::ffi::c_void;




























































impl CFolderViewCB {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_View = core::ptr::null_mut();
        {}
    }
    pub unsafe fn MessageSFVCB(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        match uMsg {
            SFVM_DEFVIEWMODE => {
                let pViewMode: *mut FOLDERVIEWMODE = lParam as *mut FOLDERVIEWMODE;
                *pViewMode = (FVM_DETAILS as i32);
                return S_OK;
            }
            SFVM_COLUMNCLICK => {
                return S_FALSE;
            }
            SFVM_BACKGROUNDENUM => {
                return S_OK;
            }
            _ => {}
        }

        DbgPrint(b"MessageSFVCB unimplemented %d %08x %08x\n\0".as_ptr() as *const i8, uMsg, wParam, lParam);
        return E_NOTIMPL;
    }
    pub unsafe fn Initialize__body(&mut self, psv: *mut IShellView) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_View = psv;
        return S_OK;
    }
}

impl CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_shellPidl = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_shellPidl.is_null() {
            ILFree((*this_).m_shellPidl);
        }
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut LPITEMIDLIST, pdwAttributes: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if ppidl.is_null() {
            return E_POINTER;
        }

        if !pchEaten.is_null() {
            *pchEaten = 0;
        }

        if !pdwAttributes.is_null() {
            *pdwAttributes = 0;
        }

        let mut info: *const NtPidlEntry = core::ptr::null();
        let mut it: *mut IEnumIDList = core::ptr::null_mut();
        let mut hr: HRESULT = (*this_).__base1.EnumObjects(hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, core::ptr::addr_of_mut!(it));
        if hr < 0 {
            return hr;
        }

        let mut end: PWSTR = StrChrW(lpszDisplayName, '\\' as u16);
        let mut length: i32 = if !end.is_null() {
            end.offset_from(lpszDisplayName) as i32
        } else {
            wcslen(lpszDisplayName) as i32
        };

        loop {
            hr = (*it).Next(1, ppidl, core::ptr::null_mut());

            if hr < 0 {
                return hr;
            }

            if hr != S_OK {
                return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
            }

            hr = (*this_).GetInfoFromPidl(*ppidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            if StrCmpNW(core::ptr::addr_of!((*info).entryName).cast(), lpszDisplayName, length) == 0 {
                break;
            }
        }

        // if has remaining path to parse (and the path didn't just terminate in a backslash)
        if !end.is_null() && (wcslen(end) as i32) > 1 {
            let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
            hr = (*this_).BindToObject(*ppidl, pbcReserved, &IID_IShellFolder, psfChild.GetAddressOf() as *mut *mut core::ffi::c_void);
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            let mut child: LPITEMIDLIST = core::ptr::null_mut();
            hr = (*psfChild.Get()).ParseDisplayName(hwndOwner, pbcReserved, end.offset(1), pchEaten, core::ptr::addr_of_mut!(child), pdwAttributes);
            if hr < 0 {
                return hr;
            }

            let old: LPITEMIDLIST = *ppidl;
            *ppidl = ILCombine(old, child);
            ILFree(old);

            // Count the path separator
            if !pchEaten.is_null() {
                *pchEaten += 1;
            }
        } else {
            if !pdwAttributes.is_null() {
                *pdwAttributes = (*this_).ConvertAttributes(info, pdwAttributes);
            }
        }

        if !pchEaten.is_null() {
            *pchEaten += wcslen(core::ptr::addr_of!((*info).entryName).cast()) as u32;
        }

        return S_OK;
    }
    pub unsafe fn BindToObject(&mut self, pidl: LPCITEMIDLIST, pbcReserved: LPBC, riid: &IID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        if IsEqualIID!(riid as *const IID, &IID_IShellFolder as *const IID) != false {
            let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            let mut path: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

            StringCbCopyW(path.as_mut_ptr(), core::mem::size_of::<[WCHAR; MAX_PATH as usize]>(), (*this_).m_NtPath.as_ptr());
            PathAppendW(path.as_mut_ptr(), core::ptr::addr_of!((*info).entryName).cast::<WCHAR>() as *mut WCHAR);

            let first: LPITEMIDLIST = ILCloneFirst(pidl);
            let rest: LPCITEMIDLIST = ILGetNext(pidl) as LPCITEMIDLIST;

            let fullPidl: LPITEMIDLIST = ILCombine((*this_).m_shellPidl, first);

            let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
            hr = (*this_).InternalBindToObject(path.as_mut_ptr(), info, first, rest, fullPidl, pbcReserved, psfChild.GetAddressOf());

            ILFree(fullPidl);
            ILFree(first);

            if hr < 0 {
                return hr;
            }

            if hr == S_FALSE {
                return S_OK;
            }

            if (*rest).mkid.cb > 0 {
                return (*psfChild.Get()).BindToObject(rest, pbcReserved, riid, ppvOut);
            }

            return (*psfChild.Get()).QueryInterface(riid, ppvOut);
        }

        return E_NOTIMPL;
    }
    pub unsafe fn ResolveSymLink__body(&mut self, info: *const NtPidlEntry,
        fullPidl: *mut LPITEMIDLIST,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn BindToStorage(&mut self, pidl: LPCITEMIDLIST, pbcReserved: LPBC, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs__0(&mut self, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;

        let mut id1: *const NtPidlEntry = core::ptr::null();
        hr = (*this_).GetInfoFromPidl(pidl1, core::ptr::addr_of_mut!(id1));
        if hr < 0 {
            return E_INVALIDARG;
        }

        let mut id2: *const NtPidlEntry = core::ptr::null();
        hr = (*this_).GetInfoFromPidl(pidl2, core::ptr::addr_of_mut!(id2));
        if hr < 0 {
            return E_INVALIDARG;
        }

        hr = (*this_).CompareIDs__1(lParam, id1, id2);
        if hr != S_EQUAL {
            return hr;
        }

        // The wollowing snipped is basically SHELL32_CompareChildren

        let rest1: PUIDLIST_RELATIVE = ILGetNext(pidl1);
        let rest2: PUIDLIST_RELATIVE = ILGetNext(pidl2);

        let isEmpty1: bool = (*rest1).mkid.cb == 0;
        let isEmpty2: bool = (*rest2).mkid.cb == 0;

        if isEmpty1 || isEmpty2 {
            return MAKE_COMPARE_HRESULT!((isEmpty2 as i32) - (isEmpty1 as i32));
        }

        let first1: LPCITEMIDLIST = ILCloneFirst(pidl1);
        if first1.is_null() {
            return E_OUTOFMEMORY;
        }

        let mut psfNext: CComPtr<IShellFolder> = CComPtr::new();
        hr = (*this_).BindToObject(
            first1,
            core::ptr::null_mut(),
            &IID_IShellFolder,
            psfNext.GetAddressOf() as *mut *mut core::ffi::c_void,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        return (*psfNext.Get()).CompareIDs(lParam, rest1, rest2);
    }
    pub unsafe fn CompareName__body(&mut self, lParam: LPARAM, first: *const NtPidlEntry, second: *const NtPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        let f1: bool = (*this_).IsFolder__1(first) != 0;
        let f2: bool = (*this_).IsFolder__1(second) != 0;

        let mut hr: HRESULT = MAKE_COMPARE_HRESULT!((f2 as i32) - (f1 as i32));
        if hr != S_EQUAL {
            return hr;
        }

        let canonical: bool = (lParam & (0xFFFF0000u32 as LPARAM)) == (SHCIDS_CANONICALONLY as LPARAM);
        if canonical {
            // Shortcut: avoid comparing contents if not necessary when the results are not for display.
            hr = MAKE_COMPARE_HRESULT!(((*second).entryNameLength as i32) - ((*first).entryNameLength as i32));
            if hr != S_EQUAL {
                return hr;
            }

            let minlength: i32 = min!((*first).entryNameLength as i32, (*second).entryNameLength as i32);
            if minlength > 0 {
                hr = MAKE_COMPARE_HRESULT!(memcmp(core::ptr::addr_of!((*first).entryName).cast(), core::ptr::addr_of!((*second).entryName).cast(), (minlength) as usize));
                if hr != S_EQUAL {
                    return hr;
                }
            }

            return S_EQUAL;
        }

        let minlength: i32 = min!((*first).entryNameLength as i32, (*second).entryNameLength as i32);
        if minlength > 0 {
            hr = MAKE_COMPARE_HRESULT!(StrCmpNW((core::ptr::addr_of!((*first).entryName).cast() as *const core::ffi::c_void).cast(), (core::ptr::addr_of!((*second).entryName).cast() as *const core::ffi::c_void).cast(), (minlength / (core::mem::size_of::<WCHAR>() as i32)) as INT));
            if hr != S_EQUAL {
                return hr;
            }
        }

        return MAKE_COMPARE_HRESULT!(((*second).entryNameLength as i32) - ((*first).entryNameLength as i32));
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualIID!(riid, &IID_IShellView) == false {
            return E_NOINTERFACE;
        }

        let mut pcb: *mut CFolderViewCB = core::ptr::null_mut();

        let mut hr: HRESULT = <CFolderViewCB as atl_rust::com::ComObject>::create_instance(&mut pcb);
        if hr < 0 {
            return hr;
        }

        (*pcb).AddRef();

        let mut sfv: SFV_CREATE = core::mem::zeroed();
        sfv.cbSize = core::mem::size_of::<SFV_CREATE>() as UINT;
        sfv.pshf = core::ptr::addr_of_mut!((*this_).__base1) as *mut IShellFolder;
        sfv.psvOuter = core::ptr::null_mut();
        sfv.psfvcb = core::ptr::addr_of_mut!((*pcb).__base1) as *mut IShellFolderViewCB;

        let mut view: *mut IShellView = core::ptr::null_mut();

        hr = SHCreateShellFolderView(core::ptr::addr_of_mut!(sfv), core::ptr::addr_of_mut!(view));
        if hr < 0 {
            return hr;
        }

        (*pcb).Initialize(view);

        (*pcb).Release();

        *ppvOut = view as *mut core::ffi::c_void;

        return S_OK;
    }
    pub unsafe fn GetAttributesOf(&mut self, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        if cidl == 0 {
            *rgfInOut &= SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
            return S_OK;
        }

        let mut i: i32 = 0;
        while i < (cidl as i32) {
            let pidl = *apidl.offset(i as isize);

            let hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            // Update attributes.
            *rgfInOut = (*this_).ConvertAttributes(info, rgfInOut);
            i += 1;
        }

        return S_OK;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let mut res: DWORD = 0;

        if IsEqualIID!(riid, &IID_IContextMenu) ||
            IsEqualIID!(riid, &IID_IContextMenu2) ||
            IsEqualIID!(riid, &IID_IContextMenu3)
        {
            let mut pcm: CComPtr<IContextMenu> = CComPtr::new();

            let mut keys: [HKEY; 1] = core::mem::zeroed();

            let mut parent: LPITEMIDLIST = (*this_).m_shellPidl;

            let mut psfParent: CComPtr<IShellFolder> = CComPtr::new();
            psfParent.assign(core::ptr::addr_of_mut!((*this_).__base1) as *mut IShellFolder);

            let mut child: LPCITEMIDLIST = core::mem::zeroed();

            let mut nkeys: UINT = 0;
            if cidl == 1 && (((*this_).IsSymLink__0(*apidl.add(0))) != 0) {
                let mut info: *const NtPidlEntry = core::mem::zeroed();
                let mut hr: HRESULT = (*this_).GetInfoFromPidl(*apidl.add(0), core::ptr::addr_of_mut!(info));
                if hr < 0 {
                    return hr;
                }

                let mut target: LPITEMIDLIST = core::mem::zeroed();
                hr = (*this_).ResolveSymLink(info, core::ptr::addr_of_mut!(target));
                if hr < 0 {
                    return hr;
                }

                let mut psfTarget: CComPtr<IShellFolder> = CComPtr::new();
                hr = SHBindToParent(target, core::ptr::addr_of!(IID_IShellFolder), psfTarget.GetAddressOf() as *mut *mut core::ffi::c_void, core::ptr::addr_of_mut!(child));
                if hr < 0 {
                    ILFree(target);
                    return hr;
                }

                parent = ILClone(target);
                ILRemoveLastID(parent);
                psfParent.assign(psfTarget.Get());

                apidl = core::ptr::addr_of_mut!(child) as PCUITEMID_CHILD_ARRAY;
            }

            if cidl == 1 && (((*this_).IsFolder__0(*apidl.add(0))) != 0) {
                res = (RegOpenKeyExW(HKEY_CLASSES_ROOT, CPP_COM_WSTR_EB4CE2024A43.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(keys[0]))) as u32;
                if !NT_SUCCESS!(res) {
                    return HRESULT_FROM_WIN32!(res);
                }
                nkeys += 1;
            }

            let hr: HRESULT = CDefFolderMenu_Create2(parent, hwndOwner, cidl, apidl, psfParent.Get(), Some(CCommonFolder_CNtObjectFolder_NtPidlEntry_CNtObjectFolderExtractIcon::DefCtxMenuCallback), nkeys, keys.as_mut_ptr(), pcm.GetAddressOf());
            let mut i: UINT = 0;
            while i < nkeys {
                RegCloseKey(keys[i as usize]);
                i += 1;
            }
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            return (*pcm.Get()).QueryInterface(riid, ppvOut);
        }

        if IsEqualIID!(riid, &IID_IExtractIconW) {
            return ShellObjectCreatorInit!(CNtObjectFolderExtractIcon, [((*this_).m_NtPath).as_ptr(), (*this_).m_shellPidl, cidl, apidl], riid, ppvOut);
        }

        if IsEqualIID!(riid, &IID_IDataObject) {
            return CIDLData_CreateFromIDArray((*this_).m_shellPidl, cidl, apidl, ppvOut as *mut *mut IDataObject);
        }

        if IsEqualIID!(riid, &IID_IQueryAssociations) {
            if cidl == 1 && (((*this_).IsFolder__0(*apidl.add(0))) != 0) {
                let mut pqa: CComPtr<IQueryAssociations> = CComPtr::new();
                let mut hr: HRESULT = AssocCreate(CLSID_QueryAssociations, core::ptr::addr_of!(IID_IQueryAssociations), pqa.GetAddressOf() as *mut *mut core::ffi::c_void);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                hr = (*pqa.Get()).Init(ASSOCF_INIT_DEFAULTTOFOLDER, CPP_COM_WSTR_B94CB911C87A.as_ptr(), core::ptr::null_mut(), hwndOwner);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                return (*pqa.Get()).QueryInterface(riid, ppvOut);
            }
        }

        return E_NOTIMPL;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: LPCITEMIDLIST, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        if ((GET_SHGDN_FOR!(uFlags) & SHGDN_FOREDITING) != 0) {
            hr = MakeStrRetFromString__0(core::ptr::addr_of!((*info).entryName).cast(), ((*info).entryNameLength) as DWORD, lpName);
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }
        }

        let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

        if ((GET_SHGDN_FOR!(uFlags) & SHGDN_FORPARSING) != 0) {
            if GET_SHGDN_RELATION!(uFlags) != SHGDN_INFOLDER {
                hr = GetFullName((*this_).m_shellPidl, uFlags, path.as_mut_ptr(), _countof!(path));
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }
            }
        }

        PathAppendW(path.as_mut_ptr() as PWSTR, core::ptr::addr_of!((*info).entryName).cast::<WCHAR>() as PWSTR);

        let mut pidlNext: LPCITEMIDLIST = ILGetNext(pidl);
        if !pidlNext.is_null() && (*pidlNext).mkid.cb > 0 {
            let mut pidlFirst: LPITEMIDLIST = ILCloneFirst(pidl);

            let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
            hr = (*this_).BindToObject(
                pidlFirst,
                core::ptr::null_mut(),
                &IID_IShellFolder,
                psfChild.GetAddressOf() as *mut *mut core::ffi::c_void,
            );
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            let mut temp: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut childName: STRRET = core::mem::zeroed();

            hr = (*psfChild.Get()).GetDisplayNameOf(
                pidlNext,
                uFlags | SHGDN_INFOLDER,
                core::ptr::addr_of_mut!(childName),
            );
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            hr = StrRetToBufW(
                core::ptr::addr_of_mut!(childName),
                pidlNext,
                temp.as_mut_ptr(),
                _countof!(temp),
            );
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            PathAppendW(path.as_mut_ptr() as PWSTR, temp.as_mut_ptr() as PWSTR);

            ILFree(pidlFirst);
        }

        hr = MakeStrRetFromString__1(path.as_mut_ptr() as PWSTR, lpName);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        return S_OK;
    }
    pub unsafe fn SetNameOf(&mut self, hwnd: HWND,
        pidl: LPCITEMIDLIST,
        lpszName: LPCOLESTR,
        uFlags: SHGDNF,
        ppidlOut: *mut LPITEMIDLIST,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultSearchGUID(&mut self, lpguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pSort.is_null() {
            *pSort = 0;
        }
        if !pDisplay.is_null() {
            *pDisplay = 0;
        }
        return S_OK;
    }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if lpClassId.is_null() {
            return E_POINTER;
        }

        *lpClassId = CLSID_NtObjectFolder;
        return S_OK;
    }
    pub unsafe fn Initialize(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_shellPidl = ILClone(pidl);

        StringCbCopyW(core::ptr::addr_of_mut!((*this_).m_NtPath) as *mut WCHAR, core::mem::size_of::<[WCHAR; 260]>(), CPP_COM_WSTR_B7FC02907B1F.as_ptr());

        return S_OK;
    }
    pub unsafe fn GetCurFolder(&mut self, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        if !pidl.is_null() {
            *pidl = ILClone((*this_).m_shellPidl);
        }
        if (*this_).m_shellPidl.is_null() {
            return S_FALSE;
        }
        return S_OK;
    }
    pub unsafe fn IsFolder__0__body(&mut self, pcidl: LPCITEMIDLIST) -> BOOL {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::mem::zeroed();

        let hr: HRESULT = (*this_).GetInfoFromPidl(pcidl, core::ptr::addr_of_mut!(info));
        if hr < 0 {
            return hr;
        }

        return (*this_).IsFolder__1(info);
    }
    pub unsafe fn IsSymLink__0__body(&mut self, pcidl: LPCITEMIDLIST) -> BOOL {
        let this_: *mut Self = self;
        let mut info: *const NtPidlEntry = core::ptr::null();

        let hr: HRESULT = (*this_).GetInfoFromPidl(pcidl, core::ptr::addr_of_mut!(info));
        if hr < 0 {
            return hr;
        }

        return (*this_).IsSymLink__1(info);
    }
    pub unsafe fn IsSymLink__1__body(&mut self, info: *const NtPidlEntry,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE as i32)
    }
    pub unsafe extern "system" fn DefCtxMenuCallback(arg0: *mut IShellFolder, arg1: HWND, arg2: *mut IDataObject, uMsg: UINT, arg4: WPARAM, arg5: LPARAM) -> HRESULT {
        if uMsg == DFM_MERGECONTEXTMENU {
            return S_OK;
        }
        // case COMMENT_TOKEN_ECHO____: / case COMMENT_TOKEN_ECHO____: / case COMMENT_TOKEN_ECHO____: (shared body)
        if uMsg == DFM_INVOKECOMMAND || uMsg == DFM_INVOKECOMMANDEX || uMsg == DFM_GETDEFSTATICID {
            // Required for Windows 7 to pick a default
            return S_FALSE;
        }
        return E_NOTIMPL;
    }
}

impl CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_shellPidl = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).m_shellPidl.is_null() {
            ILFree((*this_).m_shellPidl);
        }
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbcReserved: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut LPITEMIDLIST, pdwAttributes: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
            if ppidl.is_null() {
                return E_POINTER;
            }

            if !pchEaten.is_null() {
                *pchEaten = 0;
            }

            if !pdwAttributes.is_null() {
                *pdwAttributes = 0;
            }

            let mut info: *const RegPidlEntry = core::ptr::null();
            let mut it: *mut IEnumIDList = core::ptr::null_mut();
            let mut hr: HRESULT = (*this_).__base1.EnumObjects(hwndOwner, SHCONTF_FOLDERS | SHCONTF_NONFOLDERS, core::ptr::addr_of_mut!(it));
            if hr < 0 {
                return hr;
            }

            let mut end: PWSTR = StrChrW(lpszDisplayName, '\\' as u16);
            let mut length: i32 = if !end.is_null() {
                end.offset_from(lpszDisplayName) as i32
            } else {
                wcslen(lpszDisplayName) as i32
            };

            loop {
                hr = (*it).Next(1, ppidl, core::ptr::null_mut());

                if hr < 0 {
                    return hr;
                }

                if hr != S_OK {
                    return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
                }

                hr = (*this_).GetInfoFromPidl(*ppidl, core::ptr::addr_of_mut!(info));
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                if StrCmpNW(core::ptr::addr_of!((*info).entryName).cast(), lpszDisplayName, length) == 0 {
                    break;
                }
            }

            // if has remaining path to parse (and the path didn't just terminate in a backslash)
            if !end.is_null() && wcslen(end) > 1 {
                let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
                hr = (*this_).BindToObject(*ppidl, pbcReserved, &IID_IShellFolder, psfChild.GetAddressOf() as *mut *mut core::ffi::c_void);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                let mut child: LPITEMIDLIST = core::ptr::null_mut();
                hr = (*psfChild.Get()).ParseDisplayName(hwndOwner, pbcReserved, end.offset(1 as isize), pchEaten, core::ptr::addr_of_mut!(child), pdwAttributes);
                if hr < 0 {
                    return hr;
                }

                let mut old: LPITEMIDLIST = *ppidl;
                *ppidl = ILCombine(old, child);
                ILFree(old);

                // Count the path separator
                if !pchEaten.is_null() {
                    *pchEaten += 1;
                }
            } else {
                if !pdwAttributes.is_null() {
                    *pdwAttributes = (*this_).ConvertAttributes(info, pdwAttributes);
                }
            }

            if !pchEaten.is_null() {
                *pchEaten += wcslen(core::ptr::addr_of!((*info).entryName).cast()) as ULONG;
            }

            return S_OK;
        }
    pub unsafe fn BindToObject(&mut self, pidl: LPCITEMIDLIST, pbcReserved: LPBC, riid: &IID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::ptr::null();

        if IsEqualIID!(riid, &IID_IShellFolder) {
            let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            let mut path: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

            StringCbCopyW(path.as_mut_ptr(), core::mem::size_of::<[WCHAR; MAX_PATH as usize]>(), (*this_).m_NtPath.as_ptr());
            PathAppendW(path.as_mut_ptr(), core::ptr::addr_of!((*info).entryName).cast());

            let first: LPITEMIDLIST = ILCloneFirst(pidl);
            let rest: LPCITEMIDLIST = ILGetNext(pidl);

            let fullPidl: LPITEMIDLIST = ILCombine((*this_).m_shellPidl, first);

            let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
            hr = (*this_).InternalBindToObject(path.as_mut_ptr(), info, first, rest, fullPidl, pbcReserved, psfChild.GetAddressOf());

            ILFree(fullPidl);
            ILFree(first);

            if hr < 0 {
                return hr;
            }

            if hr == S_FALSE {
                return S_OK;
            }

            if (*rest).mkid.cb > 0 {
                return (*psfChild.Get()).BindToObject(rest, pbcReserved, riid, ppvOut);
            }

            return (*psfChild.Get()).QueryInterface(riid, ppvOut);
        }

        return E_NOTIMPL;
    }
    pub unsafe fn ResolveSymLink__body(&mut self, info: *const RegPidlEntry,
        fullPidl: *mut LPITEMIDLIST,
    ) -> HRESULT {
        let this_: *mut Self = self;
        E_NOTIMPL
    }
    pub unsafe fn BindToStorage(&mut self, pidl: LPCITEMIDLIST, pbcReserved: LPBC, riid: &IID, ppvObj: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs__0(&mut self, lParam: LPARAM, pidl1: LPCITEMIDLIST, pidl2: LPCITEMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT;

        let mut id1: *const RegPidlEntry = core::ptr::null();
        hr = (*this_).GetInfoFromPidl(pidl1, core::ptr::addr_of_mut!(id1));
        if hr < 0 {
            return E_INVALIDARG;
        }

        let mut id2: *const RegPidlEntry = core::ptr::null();
        hr = (*this_).GetInfoFromPidl(pidl2, core::ptr::addr_of_mut!(id2));
        if hr < 0 {
            return E_INVALIDARG;
        }

        hr = (*this_).CompareIDs__1(lParam, id1, id2);
        if hr != S_EQUAL {
            return hr;
        }

        // The wollowing snipped is basically SHELL32_CompareChildren

        let rest1: PUIDLIST_RELATIVE = ILGetNext(pidl1);
        let rest2: PUIDLIST_RELATIVE = ILGetNext(pidl2);

        let is_empty1: bool = (*rest1).mkid.cb == 0;
        let is_empty2: bool = (*rest2).mkid.cb == 0;

        if is_empty1 || is_empty2 {
            return MAKE_COMPARE_HRESULT!((is_empty2 as i32) - (is_empty1 as i32));
        }

        let first1: LPCITEMIDLIST = ILCloneFirst(pidl1);
        if first1.is_null() {
            return E_OUTOFMEMORY;
        }

        let mut psfNext: CComPtr<IShellFolder> = CComPtr::new();
        hr = (*this_).BindToObject(
            first1,
            core::ptr::null_mut(),
            &IID_IShellFolder,
            psfNext.GetAddressOf() as *mut *mut core::ffi::c_void,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        return (*psfNext.Get()).CompareIDs(lParam, rest1, rest2);
    }
    pub unsafe fn CompareName__body(&mut self, lParam: LPARAM, first: *const RegPidlEntry, second: *const RegPidlEntry) -> HRESULT {
        let this_: *mut Self = self;
        let f1: bool = (*this_).IsFolder__1(first) != 0;
        let f2: bool = (*this_).IsFolder__1(second) != 0;

        let mut hr: HRESULT = MAKE_COMPARE_HRESULT!((f2 as i32) - (f1 as i32));
        if hr != S_EQUAL {
            return hr;
        }

        let canonical: bool = (lParam & (0xFFFF0000u32 as LPARAM)) == (SHCIDS_CANONICALONLY as LPARAM);
        if canonical {
            // Shortcut: avoid comparing contents if not necessary when the results are not for display.
            hr = MAKE_COMPARE_HRESULT!((*second).entryNameLength as i32 - (*first).entryNameLength as i32);
            if hr != S_EQUAL {
                return hr;
            }

            let minlength: i32 = min!((*first).entryNameLength as i32, (*second).entryNameLength as i32);
            if minlength > 0 {
                hr = MAKE_COMPARE_HRESULT!(memcmp(
                    (core::ptr::addr_of!((*first).entryName) as *const WCHAR).cast(),
                    (core::ptr::addr_of!((*second).entryName) as *const WCHAR).cast(),
                    (minlength) as usize,
                ));
                if hr != S_EQUAL {
                    return hr;
                }
            }

            return S_EQUAL;
        }

        let minlength: i32 = min!((*first).entryNameLength as i32, (*second).entryNameLength as i32);
        if minlength > 0 {
            hr = MAKE_COMPARE_HRESULT!(StrCmpNW(
                core::ptr::addr_of!((*first).entryName) as *const WCHAR,
                core::ptr::addr_of!((*second).entryName) as *const WCHAR,
                minlength / (core::mem::size_of::<WCHAR>() as i32),
            ));
            if hr != S_EQUAL {
                return hr;
            }
        }

        return MAKE_COMPARE_HRESULT!((*second).entryNameLength as i32 - (*first).entryNameLength as i32);
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if !IsEqualIID!(riid, &IID_IShellView) {
            return E_NOINTERFACE;
        }

        let mut pcb: *mut CFolderViewCB = core::ptr::null_mut();

        let mut hr: HRESULT = <CFolderViewCB as atl_rust::com::ComObject>::create_instance(&mut pcb);
        if hr < 0 {
            return hr;
        }

        (*pcb).AddRef();

        let mut sfv: SFV_CREATE = core::mem::zeroed();
        sfv.cbSize = core::mem::size_of::<SFV_CREATE>() as u32;
        sfv.pshf = &mut (*this_).__base1 as *mut IShellFolder2 as *mut IShellFolder;
        sfv.psvOuter = core::ptr::null_mut();
        sfv.psfvcb = &mut (*pcb).__base1 as *mut IShellFolderViewCB;

        let mut view: *mut IShellView = core::ptr::null_mut();

        hr = SHCreateShellFolderView(core::ptr::addr_of_mut!(sfv), core::ptr::addr_of_mut!(view));
        if hr < 0 {
            return hr;
        }

        (*pcb).Initialize(view);

        (*pcb).Release();

        *ppvOut = view as *mut core::ffi::c_void;

        return S_OK;
    }
    pub unsafe fn GetAttributesOf(&mut self, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::ptr::null();

        if cidl == 0 {
            *rgfInOut &= SFGAO_FOLDER | SFGAO_HASSUBFOLDER | SFGAO_BROWSABLE;
            return S_OK;
        }

        let mut i: i32 = 0;
        while i < (cidl as i32) {
            let pidl: PCUITEMID_CHILD = *apidl.offset(i as isize);

            let hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
            if FAILED_UNEXPECTEDLY!(hr) != false {
                return hr;
            }

            // Update attributes.
            *rgfInOut = (*this_).ConvertAttributes(info, rgfInOut);

            i += 1;
        }

        return S_OK;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, mut apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        let mut res: DWORD = 0;

        if IsEqualIID!(riid, &IID_IContextMenu) ||
            IsEqualIID!(riid, &IID_IContextMenu2) ||
            IsEqualIID!(riid, &IID_IContextMenu3)
        {
            let mut pcm: CComPtr<IContextMenu> = CComPtr::new();

            let mut keys: [HKEY; 1 as usize] = core::mem::zeroed();

            let mut parent: LPITEMIDLIST = (*this_).m_shellPidl;

            let mut psfParent: CComPtr<IShellFolder> = CComPtr::new();
            psfParent.assign(core::ptr::addr_of_mut!((*this_).__base1) as *mut IShellFolder);

            let mut child: LPCITEMIDLIST = core::mem::zeroed();

            let mut nkeys: UINT = 0;
            if cidl == 1 && (*this_).IsSymLink__0(*apidl.add(0 as usize)) != 0
            {
                let mut info: *const RegPidlEntry = core::ptr::null();
                let mut hr: HRESULT = (*this_).GetInfoFromPidl(*apidl.add(0 as usize), core::ptr::addr_of_mut!(info));
                if hr < 0 {
                    return hr;
                }

                let mut target: LPITEMIDLIST = core::mem::zeroed();
                hr = (*this_).ResolveSymLink(info, core::ptr::addr_of_mut!(target));
                if hr < 0 {
                    return hr;
                }

                let mut psfTarget: CComPtr<IShellFolder> = CComPtr::new();
                hr = SHBindToParent(target, core::ptr::addr_of!(IID_IShellFolder), psfTarget.GetAddressOf() as *mut *mut core::ffi::c_void, core::ptr::addr_of_mut!(child));
                if hr < 0
                {
                    ILFree(target);
                    return hr;
                }

                parent = ILClone(target);
                ILRemoveLastID(parent);
                psfParent.assign(psfTarget.Get());

                apidl = core::ptr::addr_of!(child) as PCUITEMID_CHILD_ARRAY;
            }

            if cidl == 1 && (*this_).IsFolder__0(*apidl.add(0 as usize)) != 0
            {
                res = (RegOpenKeyExW(HKEY_CLASSES_ROOT, CPP_COM_WSTR_EB4CE2024A43.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(keys[0 as usize]))) as u32;
                if !NT_SUCCESS!(res) {
                    return HRESULT_FROM_WIN32!(res);
                }
                nkeys += 1;
            }

            let mut hr: HRESULT = CDefFolderMenu_Create2(parent, hwndOwner, cidl, apidl, psfParent.Get(), Some(CCommonFolder_CRegistryFolder_RegPidlEntry_CRegistryFolderExtractIcon::DefCtxMenuCallback), nkeys, keys.as_ptr(), pcm.GetAddressOf());
            let mut i: UINT = 0;
            while i < nkeys {
                RegCloseKey(keys[i as usize]);
                i += 1;
            }
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            return (*pcm.Get()).QueryInterface(riid, ppvOut);
        }

        if IsEqualIID!(riid, &IID_IExtractIconW)
        {
            return ShellObjectCreatorInit!(CRegistryFolderExtractIcon, [((*this_).m_NtPath).as_ptr(), (*this_).m_shellPidl, cidl, apidl], riid, ppvOut);
        }

        if IsEqualIID!(riid, &IID_IDataObject)
        {
            return CIDLData_CreateFromIDArray((*this_).m_shellPidl, cidl, apidl, ppvOut as *mut *mut IDataObject);
        }

        if IsEqualIID!(riid, &IID_IQueryAssociations)
        {
            if cidl == 1 && (*this_).IsFolder__0(*apidl.add(0 as usize)) != 0
            {
                let mut pqa: CComPtr<IQueryAssociations> = CComPtr::new();
                let mut hr: HRESULT = AssocCreate(CLSID_QueryAssociations, core::ptr::addr_of!(IID_IQueryAssociations), pqa.GetAddressOf() as *mut *mut core::ffi::c_void);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                hr = (*pqa.Get()).Init(ASSOCF_INIT_DEFAULTTOFOLDER, CPP_COM_WSTR_B94CB911C87A.as_ptr(), core::ptr::null_mut(), hwndOwner);
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }

                return (*pqa.Get()).QueryInterface(riid, ppvOut);
            }
        }

        return E_NOTIMPL;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: LPCITEMIDLIST, uFlags: SHGDNF, lpName: *mut STRRET) -> HRESULT {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::ptr::null();

        let mut hr: HRESULT = (*this_).GetInfoFromPidl(pidl, core::ptr::addr_of_mut!(info));
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        if (GET_SHGDN_FOR!(uFlags) & SHGDN_FOREDITING) != 0
        {
            hr = MakeStrRetFromString__0(core::ptr::addr_of!((*info).entryName).cast(), ((*info).entryNameLength) as DWORD, lpName);
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }
        }

        let mut path: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

        if (GET_SHGDN_FOR!(uFlags) & SHGDN_FORPARSING) != 0
        {
            if GET_SHGDN_RELATION!(uFlags) != SHGDN_INFOLDER
            {
                hr = GetFullName((*this_).m_shellPidl, uFlags, path.as_mut_ptr(), _countof!(path));
                if FAILED_UNEXPECTEDLY!(hr) {
                    return hr;
                }
            }
        }

        PathAppendW(path.as_mut_ptr(), core::ptr::addr_of!((*info).entryName).cast());

        let pidlNext: LPCITEMIDLIST = ILGetNext(pidl);
        if !pidlNext.is_null() && (*pidlNext).mkid.cb > 0
        {
            let pidlFirst: LPITEMIDLIST = ILCloneFirst(pidl);

            let mut psfChild: CComPtr<IShellFolder> = CComPtr::new();
            hr = (*this_).BindToObject(pidlFirst, core::ptr::null_mut(), &IID_IShellFolder, psfChild.GetAddressOf() as *mut *mut core::ffi::c_void);
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            let mut temp: [WCHAR; MAX_PATH as usize] = core::mem::zeroed();
            let mut childName: STRRET = core::mem::zeroed();

            hr = (*psfChild.Get()).GetDisplayNameOf(pidlNext, uFlags | SHGDN_INFOLDER, core::ptr::addr_of_mut!(childName));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            hr = StrRetToBufW(core::ptr::addr_of_mut!(childName), pidlNext, temp.as_mut_ptr(), _countof!(temp));
            if FAILED_UNEXPECTEDLY!(hr) {
                return hr;
            }

            PathAppendW(path.as_mut_ptr(), temp.as_ptr());

            ILFree(pidlFirst);
        }

        hr = MakeStrRetFromString__1(path.as_mut_ptr(), lpName);
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        return S_OK;
    }
    pub unsafe fn SetNameOf(&mut self, hwnd: HWND,
        pidl: LPCITEMIDLIST,
        lpszName: LPCOLESTR,
        uFlags: SHGDNF,
        ppidlOut: *mut LPITEMIDLIST,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultSearchGUID(&mut self, lpguid: *mut GUID,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn EnumSearches(&mut self, ppenum: *mut *mut IEnumExtraSearch,
    ) -> HRESULT {
        let this_: *mut Self = self;
        // UNIMPLEMENTED;
        E_NOTIMPL
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwReserved: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
            if !pSort.is_null() {
                *pSort = 0;
            }
            if !pDisplay.is_null() {
                *pDisplay = 0;
            }
            return S_OK;
        }
    pub unsafe fn GetClassID(&mut self, lpClassId: *mut CLSID) -> HRESULT {
        let this_: *mut Self = self;
        if lpClassId.is_null() {
            return E_POINTER;
        }

        *lpClassId = CLSID_NtObjectFolder;
        return S_OK;
    }
    pub unsafe fn Initialize(&mut self, pidl: PCIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        (*this_).m_shellPidl = ILClone(pidl);
        StringCbCopyW((*this_).m_NtPath.as_mut_ptr(), core::mem::size_of::<[WCHAR; 260]>(), CPP_COM_WSTR_B7FC02907B1F.as_ptr());
        return S_OK;
    }
    pub unsafe fn GetCurFolder(&mut self, pidl: *mut PIDLIST_ABSOLUTE) -> HRESULT {
        let this_: *mut Self = self;
        if !pidl.is_null() {
            *pidl = ILClone((*this_).m_shellPidl);
        }
        if (*this_).m_shellPidl.is_null() {
            return S_FALSE;
        }
        return S_OK;
    }
    pub unsafe fn IsFolder__0__body(&mut self, pcidl: LPCITEMIDLIST) -> BOOL {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::ptr::null();

        let hr: HRESULT = (*this_).GetInfoFromPidl(pcidl, core::ptr::addr_of_mut!(info));
        if hr < 0 {
            return hr;
        }

        return (*this_).IsFolder__1(info);
    }
    pub unsafe fn IsSymLink__0__body(&mut self, pcidl: LPCITEMIDLIST) -> BOOL {
        let this_: *mut Self = self;
        let mut info: *const RegPidlEntry = core::mem::zeroed();

        let hr: HRESULT = (*this_).GetInfoFromPidl(pcidl, core::ptr::addr_of_mut!(info));
        if hr < 0 {
            return hr;
        }

        return (*this_).IsSymLink__1(info);
    }
    pub unsafe fn IsSymLink__1__body(&mut self, info: *const RegPidlEntry,
    ) -> BOOL {
        let this_: *mut Self = self;
        (FALSE as i32)
    }
    pub unsafe extern "system" fn DefCtxMenuCallback(arg0: *mut IShellFolder, arg1: HWND, arg2: *mut IDataObject, uMsg: u32, arg4: WPARAM, arg5: LPARAM) -> HRESULT {
        match uMsg {
            DFM_MERGECONTEXTMENU => {
                return S_OK;
            }
            DFM_INVOKECOMMAND | DFM_INVOKECOMMANDEX | DFM_GETDEFSTATICID => {
                // Required for Windows 7 to pick a default
                return S_FALSE;
            }
            _ => {}
        }
        return E_NOTIMPL;
    }
}
