use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_97829A80D968: [WCHAR; 8] = [0x0053, 0x0048, 0x0045, 0x004C, 0x004C, 0x0033, 0x0032, 0x0000];

// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
// --local-raii CDataObjectHIDA: hand-verified lowering (spliced verbatim; workers received this API as a sealed contract)
//
// THE canonical copy. Pass it as
//   rustconv translate <tu> --free-functions --local-raii CDataObjectHIDA=tools/rustconv/data/local_raii/CDataObjectHIDA.rs
// Extracted from shell32/shldataobject.rs, which is one of the three already-flipped TUs that
// each had to carry an identical copy (the others: shell32/shlfolder.rs, dialogs/item_prop.rs).
// A fourth, COpenWithMenu.cpp, stalled unpaid for want of it — the refusal is `translate`
// declining to guess drop semantics, and the fix was always a file, never a retry.
// `hida_clipformat()` comes from the crate prelude (shellutils.rs:68); this file assumes it.
// Hand-verified Rust lowering of the C++ RAII struct `CDataObjectHIDA`
// (sdk/include/reactos/shellutils.h:738). Consumed by
// `rustconv translate --free-functions --local-raii CDataObjectHIDA=<this file>`:
// spliced verbatim into the assembled module, and its API sealed into each
// affected worker prompt so the model maps C++ usage onto this fixed surface
// instead of inventing the drop semantics.
//
// It borrows the HIDA out of an IDataObject WITHOUT copying it, so the medium
// must stay locked for exactly the lifetime of the guard — which is the whole
// reason it is RAII and the whole reason a hand lowering is required.
//
// C++ semantics preserved exactly:
//   ctor        — CreateCIDA(pDataObject, &m_cida, m_medium); m_hr = result
//   dtor        — DestroyCIDA(m_cida, m_medium)                → impl Drop
//   hr()        — the stored ctor HRESULT
//   operator bool()        → is_valid()      (m_cida != nullptr)
//   operator const CIDA*() → as_ptr()        (the implicit conversion at a call
//                                             site such as `f(x, cida)`)
//   operator->()           → (*cida.as_ptr()).field
//
// Two details that look like bugs and are not, so do not "fix" them:
//   * DestroyCIDA calls ReleaseStgMedium unconditionally. On the failure paths
//     CreateCIDA has already set tymed = TYMED_NULL, which makes that call a
//     documented no-op — including after CreateCIDA itself released the medium.
//   * C++ leaves m_medium uninitialized and only assigns pUnkForRelease. Every
//     path assigns tymed before anything reads it (GetData on success,
//     TYMED_NULL otherwise). Zeroing here is strictly safer and changes no path.
struct CDataObjectHIDA {
    m_medium: STGMEDIUM,
    m_cida: *const CIDA,
    m_hr: HRESULT,
}

impl CDataObjectHIDA {
    unsafe fn new(pDataObject: *mut IDataObject) -> CDataObjectHIDA {
        let mut medium: STGMEDIUM = core::mem::zeroed();
        let mut cida: *const CIDA = core::ptr::null();
        let hr = CDataObjectHIDA::CreateCIDA(pDataObject, &mut cida, &mut medium);
        CDataObjectHIDA { m_medium: medium, m_cida: cida, m_hr: hr }
    }

    unsafe fn CreateCIDA(
        pDataObject: *mut IDataObject,
        ppcida: *mut *const CIDA,
        medium: *mut STGMEDIUM,
    ) -> HRESULT {
        *ppcida = core::ptr::null();
        (*medium).pUnkForRelease = core::ptr::null_mut();
        // g_cfHIDA in C++ is a DECLSPEC_SELECTANY cache of the registered format.
        let mut format = FORMATETC {
            cfFormat: hida_clipformat(),
            ptd: core::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT,
            lindex: -1,
            tymed: TYMED_HGLOBAL,
        };
        let hr = (((*(*pDataObject).lpVtbl).GetData).unwrap_unchecked())(
            pDataObject,
            (&mut format as *mut FORMATETC).cast(),
            medium.cast(),
        );
        if hr >= 0 {
            *ppcida = GlobalLock((*medium).__bindgen_anon_1.hGlobal) as *const CIDA;
            if !(*ppcida).is_null() {
                return S_OK;
            }
            ReleaseStgMedium(medium);
            (*medium).tymed = TYMED_NULL;
            return E_UNEXPECTED;
        }
        (*medium).tymed = TYMED_NULL;
        hr
    }

    /// Mirrors the C++ public static. It is not only the destructor's body:
    /// dll/shellext/cabview/folder.cpp calls CreateCIDA/DestroyCIDA directly to
    /// hold a CIDA across a scope the guard cannot span, so it must stay callable.
    /// shell32-rust's prelude re-exports it and move_to_menu.rs reaches it through
    /// CDataObjectHIDA_DestroyCIDA_RustBridge — a lowering without it links short.
    unsafe fn DestroyCIDA(pcida: *const CIDA, medium: *mut STGMEDIUM) {
        if !pcida.is_null() {
            GlobalUnlock((*medium).__bindgen_anon_1.hGlobal);
        }
        ReleaseStgMedium(medium);
    }

    unsafe fn hr(&self) -> HRESULT {
        self.m_hr
    }

    unsafe fn is_valid(&self) -> bool {
        !self.m_cida.is_null()
    }

    unsafe fn as_ptr(&self) -> *const CIDA {
        self.m_cida
    }
}

impl Drop for CDataObjectHIDA {
    fn drop(&mut self) {
        unsafe { CDataObjectHIDA::DestroyCIDA(self.m_cida, &mut self.m_medium) }
    }
}

use core::ffi::c_void;




































unsafe extern "C" fn CreateItem__0(Path: LPCWSTR, Attrib: UINT, Size: UINT, DateTime: UINT) -> *mut CABITEM {
    let len: usize = lstrlenW(Path) as usize;
    let cb: usize = (FIELD_OFFSET!(CABITEM, Path) + ((len + 1) * core::mem::size_of::<WCHAR>()) as i32) as usize;
    if cb > 0xffff {
        return core::ptr::null_mut();
    }
    let p: *mut CABITEM = SHAlloc((cb + core::mem::size_of::<USHORT>()) as SIZE_T) as *mut CABITEM;
    if !p.is_null() {
        (*p).cb = cb as u16;
        (*p).Unknown = 0;
        (*p).Size = Size;
        (*p).Attrib = Attrib as u16;
        (*p).Date = HIWORD!(DateTime);
        (*p).Time = LOWORD!(DateTime);
        (*p).NameOffset = 0;
        let mut i: UINT = 0;
        loop {
            let mut c: WCHAR = *Path.add(i as usize);
            if c == 0x003Au16 { // Don't allow absolute paths
                c = 0x005Fu16;
            }
            if c == 0x002Fu16 { // Normalize
                c = 0x005Cu16;
            }
            if c == 0x005Cu16 { // '\\'
                (*p).NameOffset = (i + 1) as u16;
            }
            *core::ptr::addr_of_mut!((*p).Path).cast::<WCHAR>().add(i as usize) = c;
            if c == 0 {
                break;
            }
            i += 1;
        }
        (*((((p as *mut u8).add(cb)) as *mut SHITEMID))).cb = 0;
    }
    return p as *mut CABITEM;
}
unsafe extern "C" fn CreateItem__1(Path: LPCSTR, Attrib: UINT, Size: UINT, DateTime: UINT) -> *mut CABITEM {
    let mut buf = [0u16; (MAX_PATH * 2) as usize];
    let codepage: UINT = if (Attrib & _A_NAME_IS_UTF) != 0 { CP_UTF8 } else { CP_ACP };
    if MultiByteToWideChar(codepage, 0, Path, -(1 as i32), buf.as_mut_ptr(), _countof!(buf) as i32) != 0 {
        return CreateItem__0(buf.as_ptr(), Attrib, Size, DateTime);
    }
    core::ptr::null_mut()
}

unsafe extern "system" fn ItemMenuCallback(psf: *mut IShellFolder, hwnd: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    // `enum { IDC_EXTRACT, IDC_EXTRACTALL };`: the menu ids, compared with the WPARAM.
    const IDC_EXTRACT: WPARAM = 0;
    const IDC_EXTRACTALL: WPARAM = 1;
    let mut hr: HRESULT = E_NOTIMPL;
    let Background: BOOL = (pdtobj.is_null()) as BOOL;

    'sw: {
        match uMsg {
            DFM_MODIFYQCMFLAGS => {
                *(lParam as *mut UINT) = (wParam as UINT) | CMF_NOVERBS | (if Background != 0 { 0 } else { CMF_VERBSONLY });
                return S_OK;
            }

            DFM_MERGECONTEXTMENU => {
                let qcmi: &mut QCMINFO = &mut *(lParam as *mut QCMINFO);
                let mut pos: UINT = qcmi.indexMenu;
                let mut id: UINT = 0;
                if Background != 0 || { hr = InsertMenuItem(&mut *qcmi, &mut pos, &mut id, IDC_EXTRACT as UINT, IDS_EXTRACT, MFS_DEFAULT as i32); hr >= 0 } {
                    hr = InsertMenuItem(&mut *qcmi, &mut pos, &mut id, IDC_EXTRACTALL as UINT, IDS_EXTRACTALL, 0);
                    if hr >= 0 && Background == 0 {
                        pos -= 1;
                        InsertMenuItem(&mut *qcmi, &mut pos, &mut id, 0, (-1i32) as UINT, 0); // Separator
                    }
                }
                if hr >= 0 {
                    qcmi.idCmdFirst = id + 1;
                    hr = S_FALSE; // Don't add verbs
                }
                break 'sw;
            }

            DFM_INVOKECOMMAND => {
                hr = S_FALSE;
                let pCabFolder: *mut CCabFolder = psf as *mut CCabFolder;
                'sw2: {
                    match wParam {
                        IDC_EXTRACT | IDC_EXTRACTALL => {
                            hr = (*pCabFolder).ExtractFilesUI(hwnd, if wParam == IDC_EXTRACT { pdtobj } else { core::ptr::null_mut() });
                            break 'sw2;
                        }
                        _ => {}
                    }
                }
                break 'sw;
            }

            _ => {}
        }
    }
    return hr;
}

unsafe extern "system" fn FolderBackgroundMenuCallback(psf: *mut IShellFolder, hwnd: HWND, pdtobj: *mut IDataObject, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
    return ItemMenuCallback(psf, hwnd, core::ptr::null_mut(), uMsg, wParam, lParam);
}

unsafe extern "system" fn EnumFillCallback(msg: EXTRACTCALLBACKMSG, ecd: *const EXTRACTCALLBACKDATA, cookie: LPVOID) -> HRESULT {
    let ecd: &EXTRACTCALLBACKDATA = &*ecd; // `const EXTRACTCALLBACKDATA &`, as EXTRACTCALLBACK passes it
    let data: &mut FILLCALLBACKDATA = &mut *(cookie as *mut FILLCALLBACKDATA);

    match msg as UINT {
        ECM_FILE => {
            let fdin: &FDINOTIFICATION = &*ecd.pfdin;
            let mut hr: HRESULT = S_FALSE;
            let attr: SFGAOF = MapFSToSFAttributes((fdin.attribs & (CABITEM::FSATTS) as u16) as UINT);
            if IncludeInEnumIDList(data.ContF, attr) != false {
                let datetime: UINT = MAKELONG!(fdin.time, fdin.date) as UINT;
                let item: *mut CABITEM = CreateItem__1(fdin.psz1, (fdin.attribs) as UINT, (fdin.cb) as UINT, datetime);
                if item.is_null() {
                    return E_OUTOFMEMORY;
                }
                hr = (*data.pEIDL).Append(item as LPCITEMIDLIST);
                if hr < 0 {
                    SHFree((item).cast());
                }
            }
            return if hr >= 0 { S_FALSE } else { hr }; // Never extract
        }
        _ => {}
    }
    return E_NOTIMPL;
}

unsafe extern "system" fn ParseDisplayNameCallback(msg: EXTRACTCALLBACKMSG, ecd: *const EXTRACTCALLBACKDATA, cookie: LPVOID) -> HRESULT {
    let ecd: &EXTRACTCALLBACKDATA = &*ecd; // `const EXTRACTCALLBACKDATA &`, as EXTRACTCALLBACK passes it
    let data: &mut PDNCALLBACKDATA = &mut *(cookie as *mut PDNCALLBACKDATA);

    if (msg as UINT) == ECM_FILE {
        let fdin: &FDINOTIFICATION = &*ecd.pfdin;
        let hr: HRESULT = S_FALSE;
        let datetime: UINT = (MAKELONG!(fdin.time, fdin.date)) as u32;
        let item: *mut CABITEM = CreateItem__1(fdin.psz1, (fdin.attribs) as UINT, (fdin.cb) as UINT, datetime);
        if item.is_null() {
            return E_OUTOFMEMORY;
        }
        if lstrcmpiW(core::ptr::addr_of!((*item).Path).cast(), data.pszName) == 0 {
            data.pItem = item;
        } else {
            SHFree(item as LPVOID);
        }
        return if (hr) >= 0 { S_FALSE } else { hr }; // Never extract
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn GetFsPathFromIDList(pidl: PCIDLIST_ABSOLUTE, pszPath: PWSTR) -> HRESULT {
    let mut ret: BOOL = SHGetPathFromIDListW(pidl, pszPath);
    if ret == 0 && ILIsEmpty(pidl) != 0 {
        ret = SHGetSpecialFolderPathW(core::ptr::null_mut(), pszPath, (CSIDL_DESKTOPDIRECTORY as i32), (TRUE) as BOOL);
    }
    return if ret != 0 { S_OK } else { HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND) };
}

unsafe extern "system" fn FolderBrowseCallback(hwnd: HWND, uMsg: UINT, lParam: LPARAM, lpData: LPARAM) -> i32 {
    let mut buf = [0u16; (MAX_PATH) as _];
    match uMsg {
        BFFM_INITIALIZED => {
            if LoadStringW(_AtlBaseModule.GetResourceInstance(), IDS_EXTRACT, buf.as_mut_ptr(), (_countof!(buf)) as i32) != 0 {
                // Remove leading and trailing dots
                let mut s: *mut u16 = buf.as_mut_ptr();
                let mut e: *mut u16 = s.add(lstrlenW(s) as usize);
                while *s == '.' as u16 {
                    s = s.add(1);
                }
                while e > s && *e.offset(-(1 as isize)) == '.' as u16 {
                    e = e.offset(-1);
                    *e = UNICODE_NULL as u16;
                }
                SendMessageW(hwnd, WM_SETTEXT, 0, s as LPARAM);
                SendMessageW(GetDlgItem(hwnd, (IDOK as i32)), WM_SETTEXT, 0, s as LPARAM);
            }
            if lpData != 0 {
                SendMessageW(hwnd, BFFM_SETEXPANDED, (FALSE) as WPARAM, lpData);
                SendMessageW(hwnd, BFFM_SETSELECTION, (FALSE) as WPARAM, lpData);
            }
            // break;
        }

        BFFM_SELCHANGED => {
            let mut wanted: SFGAOF = SFGAO_FILESYSTEM | SFGAO_FOLDER;
            let mut query: SFGAOF = wanted | SFGAO_STREAM;
            let pidl: PCIDLIST_ABSOLUTE = lParam as PCIDLIST_ABSOLUTE;
            let mut enable: BOOL = ILIsEmpty(pidl); // Allow the desktop
            let mut child: PCUITEMID_CHILD = core::ptr::null_mut();
            let mut pSF: *mut IShellFolder = core::ptr::null_mut();
            if SHBindToParent(pidl, core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(pSF) as *mut *mut core::ffi::c_void, core::ptr::addr_of_mut!(child)) >= 0 {
                let mut attrib: SFGAOF = query;
                if (*pSF).GetAttributesOf(1, core::ptr::addr_of_mut!(child), core::ptr::addr_of_mut!(attrib)) >= 0 {
                    enable = (((attrib & query) == wanted) as BOOL);
                }
                (*pSF).Release();
            }
            if enable != 0 {
                // We don't trust .zip folders, check the file-system to make sure
                let attrib: UINT = if GetFsPathFromIDList(pidl, buf.as_mut_ptr()) >= 0 {
                    GetFileAttributesW(buf.as_mut_ptr())
                } else {
                    0
                };
                enable = (((attrib & FILE_ATTRIBUTE_DIRECTORY) != 0 && attrib != INVALID_FILE_ATTRIBUTES) as BOOL);
            }
            PostMessageW(hwnd, BFFM_ENABLEOK, 0, enable as LPARAM);
            // break;
        }
        _ => {}
    }
    return 0;
}

unsafe extern "C" fn GetUiOwner(data: &EXTRACTFILESDATA) -> HWND {
    let mut hWnd: HWND = core::ptr::null_mut();
    if (IUnknown_GetWindow((data.pPD).cast(), core::ptr::addr_of_mut!(hWnd)) >= 0) && IsWindowVisible(hWnd) != 0 {
        return hWnd;
    }
    if IsWindowVisible(data.hWndOwner) != 0 {
        data.hWndOwner
    } else {
        core::ptr::null_mut()
    }
}

unsafe extern "system" fn ExtractFilesCallback(msg: EXTRACTCALLBACKMSG, ecd: *const EXTRACTCALLBACKDATA, cookie: LPVOID) -> HRESULT {
    let ecd: &EXTRACTCALLBACKDATA = &*ecd; // `const EXTRACTCALLBACKDATA &`, as EXTRACTCALLBACK passes it
    let data: &mut EXTRACTFILESDATA = &mut *(cookie as *mut EXTRACTFILESDATA);
    match (msg as UINT) {
        ECM_BEGIN => {
            data.cabfiles = (*ecd.pfdin).hf as SIZE_T as UINT;
            return S_OK;
        }

        ECM_FILE => {
            if !data.pPD.is_null() && (((*data.pPD).HasUserCancelled()) != 0) {
                return HRESULT_FROM_WIN32!(ERROR_CANCELLED);
            }
            let mut hr: HRESULT = if !data.pCIDA.is_null() { S_FALSE } else { S_OK }; // Filtering or all items?
            if hr != S_OK {
                let needle: *mut CABITEM = CreateItem__1((*ecd.pfdin).psz1, ((*ecd.pfdin).attribs) as UINT, 0, 0);
                if needle.is_null() {
                    return E_OUTOFMEMORY;
                }
                let mut i: UINT = 0;
                while i < (*data.pCIDA).cidl && hr == S_FALSE {
                    C_ASSERT!(FLATFOLDER);
                    let pidlChild: LPCITEMIDLIST = ILFindLastID(HIDA_GetPIDLItem(data.pCIDA, i));
                    let haystack: *mut CABITEM = CABITEM::Validate(pidlChild);
                    if haystack.is_null() && ({ hr = E_FAIL; FAILED_UNEXPECTEDLY!(hr) }) {
                        break;
                    }
                    if lstrcmpiW(core::ptr::addr_of!((*needle).Path).cast(), core::ptr::addr_of!((*haystack).Path).cast()) == 0 {
                        if !data.pPD.is_null() {
                            (*data.pPD).SetLine(1, core::ptr::addr_of!((*needle).Path).cast(), (TRUE) as BOOL, core::ptr::null_mut());
                        }
                        hr = S_OK; // Found it in the list of files to extract
                    }
                    i += 1;
                }
                SHFree((needle).cast());
            }
            if !data.pPD.is_null() {
                (*data.pPD).SetProgress({ let old = data.completed; data.completed = old.wrapping_add(1); old }, data.cabfiles);
            }
            return hr;
        }

        ECM_PREPAREPATH => {
            let flags: UINT = SHPPFW_DIRCREATE | SHPPFW_IGNOREFILENAME;
            return SHPathPrepareForWriteW(GetUiOwner(data), core::ptr::null_mut(), ecd.Path, flags);
        }

        ECM_ERROR => {
            return ErrorBox__T0(GetUiOwner(data), ecd.hr);
        }

        _ => {}
    }
    return E_NOTIMPL;
}

unsafe extern "C" fn Free(data: &mut EXTRACTFILESDATA) {
    if !data.pPD.is_null() {
        (((*(*data.pPD).lpVtbl).StopProgressDialog).unwrap_unchecked())(data.pPD);
        (((*(*data.pPD).lpVtbl).Release).unwrap_unchecked())(data.pPD);
    }
    CDataObjectHIDA::DestroyCIDA(
        data.pCIDA as *const CIDA,
        core::ptr::addr_of_mut!(data.cidamedium),
    );
    IUnknown_Set(
        core::ptr::addr_of_mut!(data.pDO) as *mut *mut IUnknown,
        core::ptr::null_mut(),
    );
    IUnknown_Set(
        core::ptr::addr_of_mut!(data.pMarshalDO) as *mut *mut IUnknown,
        core::ptr::null_mut(),
    );
    IUnknown_Set(
        core::ptr::addr_of_mut!(data.pLifetimeCF) as *mut *mut IUnknown,
        core::ptr::null_mut(),
    );
    SHFree(data as *mut EXTRACTFILESDATA as LPVOID);
}

unsafe extern "system" fn ExtractFilesThread(pParam: LPVOID) -> DWORD {
    let data: &mut EXTRACTFILESDATA = &mut *(pParam as *mut EXTRACTFILESDATA);
    let mut hr: HRESULT = S_OK;
    if (SHCoCreateInstance(core::ptr::null_mut(), &CLSID_ProgressDialog, core::ptr::null_mut(), &IID_IProgressDialog, core::ptr::addr_of_mut!(data.pPD) as *mut LPVOID) >= 0)
    {
        // TODO: IActionProgress SPACTION_COPYING
        if ((*data.pPD).StartProgressDialog(data.hWndOwner, core::ptr::null_mut(), PROGDLG_NOTIME, core::ptr::null_mut()) >= 0)
        {
            (*data.pPD).SetTitle(data.cab.as_ptr());
            (*data.pPD).SetLine(2, data.path.as_ptr(), (TRUE) as BOOL, core::ptr::null_mut());
            (*data.pPD).SetAnimation(GetModuleHandleW(CPP_COM_WSTR_97829A80D968.as_ptr()), 161);
            (*data.pPD).SetProgress(0, 0);
        }
    }
    if !data.pMarshalDO.is_null()
    {
        hr = CoGetInterfaceAndReleaseStream(data.pMarshalDO, &IID_IDataObject, core::ptr::addr_of_mut!(data.pDO) as *mut LPVOID);
        data.pMarshalDO = core::ptr::null_mut();
        if hr >= 0
        {
            hr = CDataObjectHIDA::CreateCIDA(data.pDO, core::ptr::addr_of_mut!(data.pCIDA) as *mut *const CIDA, core::ptr::addr_of_mut!(data.cidamedium));
        }
    }
    if hr >= 0
    {
        ExtractCabinet(data.cab.as_ptr(), data.path.as_ptr(), Some(ExtractFilesCallback), (data as *mut EXTRACTFILESDATA).cast());
    }
    Free(data);
    return 0;
}

impl CEnumIDList {
    pub unsafe fn FindNamedItem(&mut self, pidl: PCUITEMID_CHILD) -> i32 {
        let this_: *mut Self = self;
        let needle: *mut CABITEM = pidl as *mut CABITEM;
        let mut i: ULONG = 0;
        let c: UINT = (*this_).GetCount();
        while i < c {
            let item: *mut CABITEM = DPA_FastGetPtr!((*this_).m_Items, i) as *mut CABITEM;
            if lstrcmpiW(core::ptr::addr_of!((*needle).Path) as *const WCHAR, core::ptr::addr_of!((*item).Path) as *const WCHAR) == 0 {
                return i as i32;
            }
            i += 1;
        }
        return -1;
    }
    pub unsafe fn Enumerate(pidl: PCIDLIST_ABSOLUTE, hwnd: HWND, Callback: EXTRACTCALLBACK, cookie: LPVOID) -> HRESULT {
        let mut path: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        if SHGetPathFromIDListW(pidl, path.as_mut_ptr()) != 0 {
            return ExtractCabinet(path.as_mut_ptr(), core::ptr::null_mut(), Callback, cookie);
        }
        return HRESULT_FROM_WIN32!(ERROR_OPEN_FAILED);
    }
    pub unsafe fn Fill(&mut self, pidl: PCIDLIST_ABSOLUTE, hwnd: HWND, contf: SHCONTF) -> HRESULT {
        let this_: *mut Self = self;
        let mut data = FILLCALLBACKDATA { pEIDL: this_, ContF: contf };
        return CEnumIDList::Enumerate(pidl, core::ptr::null_mut(), Some(EnumFillCallback), (core::ptr::addr_of_mut!(data)).cast());
    }
}

impl CCabFolder {
    pub unsafe fn ExtractFilesUI(&mut self, mut hWnd: HWND, pDO: *mut IDataObject) -> HRESULT {
        let this_: *mut Self = self;
        if IsWindowVisible(hWnd) == 0 && IsWindowVisible((*this_).m_ShellViewWindow) != 0 {
            hWnd = (*this_).m_ShellViewWindow;
        }

        let pData: *mut EXTRACTFILESDATA = SHAlloc((core::mem::size_of::<EXTRACTFILESDATA>()) as SIZE_T) as *mut EXTRACTFILESDATA;
        if pData.is_null() {
            return E_OUTOFMEMORY;
        }
        ZeroMemory!(pData as *mut core::ffi::c_void, core::mem::size_of::<EXTRACTFILESDATA>());
        (*pData).hWndOwner = hWnd;
        (*pData).pLifetimeCF = this_;
        (*(*pData).pLifetimeCF).AddRef();

        let mut hr: HRESULT = GetFsPathFromIDList((*this_).m_CurDir.m_pData, (*pData).cab.as_mut_ptr());
        if hr >= 0 && !pDO.is_null() {
            hr = CoMarshalInterThreadInterfaceInStream(&IID_IDataObject, (pDO).cast(), core::ptr::addr_of_mut!((*pData).pMarshalDO));
        }
        if hr >= 0 {
            hr = HRESULT_FROM_WIN32!(ERROR_CANCELLED);
            let pidlInitial: LPITEMIDLIST = ILClone((*this_).m_CurDir.m_pData);
            ILRemoveLastID(pidlInitial); // Remove the "name.cab" part (we can't extract into ourselves)
            let bif: UINT = BIF_RETURNONLYFSDIRS | BIF_USENEWUI;
            let mut bi = BROWSEINFO {
                hwndOwner: hWnd,
                pidlRoot: core::ptr::null_mut(),
                pszDisplayName: core::ptr::null_mut(),
                lpszTitle: (*pData).cab.as_mut_ptr(),
                ulFlags: bif,
                lpfn: Some(FolderBrowseCallback),
                lParam: pidlInitial as LPARAM,
                iImage: 0,
            };
            let folder: PIDLIST_ABSOLUTE = SHBrowseForFolderW(core::ptr::addr_of_mut!(bi));
            if !folder.is_null() {
                hr = GetFsPathFromIDList(folder, (*pData).path.as_mut_ptr());
                ILFree(folder);
                if hr >= 0 {
                    let ctf: UINT = CTF_COINIT | CTF_PROCESS_REF | CTF_THREAD_REF | CTF_FREELIBANDEXIT;
                    hr = if SHCreateThread(Some(ExtractFilesThread), pData as *mut c_void, ctf, None) != 0 {
                        S_OK
                    } else {
                        E_OUTOFMEMORY
                    };
                }
            }
            ILFree(pidlInitial);
        }
        if hr != S_OK {
            Free(&mut *pData);
        }
        return if hr == HRESULT_FROM_WIN32!(ERROR_CANCELLED) { S_OK } else { hr };
    }
    pub unsafe fn GetItemDetails(&mut self, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS, pv: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        let mut hr: HRESULT = E_FAIL;
        let psr: *mut STRRET = core::ptr::addr_of_mut!((*psd).str_);
        let mut srvar: STRRET = core::mem::zeroed();
        let item: *mut CABITEM = CABITEM::Validate(pidl);
        if item.is_null() {
            return E_INVALIDARG;
        }

        match (iColumn) as i32 {
            COL_NAME => {
                hr = (*this_).GetDisplayNameOf(pidl, SHGDN_NORMAL | SHGDN_INFOLDER, if !pv.is_null() { core::ptr::addr_of_mut!(srvar) } else { psr });
                return if ((hr) >= 0) && !pv.is_null() { StrRetToVariantBSTR(core::ptr::addr_of_mut!(srvar), pv) } else { hr };
            }

            COL_SIZE => {
                let data: UINT = (*item).Size;
                if !pv.is_null() {
                    V_VT!(pv) = (VT_UI4 as u16);
                    V_UI4!(pv) = data;
                } else {
                    (*psr).uType = STRRET_CSTR;
                    StrFormatByteSizeA(data, (*psr).__bindgen_anon_1.cStr.as_mut_ptr(), _countof!((*psr).__bindgen_anon_1.cStr));
                }
                return S_OK;
            }

            COL_TYPE => {
                let mut fi: SHFILEINFO = core::mem::zeroed();
                let data: LPCWSTR = (core::ptr::addr_of!(fi.szTypeName) as *const WCHAR);
                let attr: DWORD = (*item).GetFSAttributes() as DWORD;
                let flags: UINT = SHGFI_TYPENAME | SHGFI_USEFILEATTRIBUTES;
                if SHGetFileInfo((*item).GetName(), attr, core::ptr::addr_of_mut!(fi), core::mem::size_of::<SHFILEINFO>() as UINT, flags) != 0 {
                    return if !pv.is_null() { StrTo__3(data, pv) } else { StrTo__1(data, psr) };
                }
            }

            COL_MDATE => {
                if !pv.is_null() {
                    if DosDateTimeToVariantTime((*item).Date, (*item).Time, core::ptr::addr_of_mut!(V_DATE!(pv))) != 0 {
                        V_VT!(pv) = (VT_DATE as u16);
                        return S_OK;
                    }
                } else {
                    let mut utc: FILETIME = core::mem::zeroed();
                    let mut loc: FILETIME = core::mem::zeroed();
                    if (DosDateTimeToFileTime((*item).Date, (*item).Time, core::ptr::addr_of_mut!(utc)) != 0) && (FileTimeToLocalFileTime(core::ptr::addr_of_mut!(utc), core::ptr::addr_of_mut!(loc)) != 0) {
                        (*psr).uType = STRRET_CSTR;
                        if SHFormatDateTimeA(core::ptr::addr_of_mut!(loc), core::ptr::null_mut(), (*psr).__bindgen_anon_1.cStr.as_mut_ptr(), _countof!((*psr).__bindgen_anon_1.cStr)) != 0 {
                            return S_OK;
                        }
                    }
                }
            }

            COL_PATH => {
                let len: UINT = if (*item).NameOffset != 0 { ((*item).NameOffset as UINT) - (1 as UINT) } else { 0 as UINT };
                return if !pv.is_null() { StrTo__2(core::ptr::addr_of!((*item).Path) as *const WCHAR, len, pv) } else { StrTo__0(core::ptr::addr_of!((*item).Path) as *const WCHAR, len, psr) };
            }

            COL_ATT => {
                let data: UINT = (*item).GetFSAttributes() as UINT;
                if !pv.is_null() {
                    V_VT!(pv) = (VT_UI4 as u16);
                    V_UI4!(pv) = data;
                } else {
                    let mut i: UINT = 0;
                    (*psr).uType = STRRET_CSTR;
                    if (data & FILE_ATTRIBUTE_READONLY) != 0 {
                        (*psr).__bindgen_anon_1.cStr[i as usize] = 'R' as CHAR;
                        i += 1;
                    }
                    if (data & FILE_ATTRIBUTE_HIDDEN) != 0 {
                        (*psr).__bindgen_anon_1.cStr[i as usize] = 'H' as CHAR;
                        i += 1;
                    }
                    if (data & FILE_ATTRIBUTE_SYSTEM) != 0 {
                        (*psr).__bindgen_anon_1.cStr[i as usize] = 'S' as CHAR;
                        i += 1;
                    }
                    if (data & FILE_ATTRIBUTE_ARCHIVE) != 0 {
                        (*psr).__bindgen_anon_1.cStr[i as usize] = 'A' as CHAR;
                        i += 1;
                    }
                    (*psr).__bindgen_anon_1.cStr[i as usize] = '\0' as CHAR;
                    i += 1;
                }
                return S_OK;
            }

            _ => {}
        }
        return hr;
    }
    pub unsafe fn MapSCIDToColumn(&mut self, scid: &SHCOLUMNID) -> i32 {
        let this_: *mut Self = self;
        let mut i: UINT = 0;
        while (i as usize) < (_countof!(g_Columns)) as usize {
            if !g_Columns[i as usize].pkg.is_null()
                && IsEqual(scid, g_Columns[i as usize].pkg, (g_Columns[i as usize].pki) as UINT) != false
            {
                return i as i32;
            }
            i += 1;
        }
        return -1;
    }
    pub unsafe fn CompareID(&mut self, lParam: LPARAM, pidl1: PCUITEMID_CHILD, pidl2: PCUITEMID_CHILD) -> HRESULT {
        let this_: *mut Self = self;
        let mut p1: *mut CABITEM = pidl1 as *mut CABITEM;
        let mut p2: *mut CABITEM = pidl2 as *mut CABITEM;
        let mut hr: HRESULT = S_OK;
        let mut ret: i32 = 0;

        if (lParam & ((SHCIDS_ALLFIELDS | SHCIDS_CANONICALONLY)) as LPARAM) != 0
        {
            ret = lstrcmpiW(core::ptr::addr_of!((*p1).Path) as LPCWSTR, core::ptr::addr_of!((*p2).Path) as LPCWSTR);
            if ret != 0 && (lParam & (SHCIDS_ALLFIELDS as LPARAM)) != 0
            {
                let mut i: UINT = 0;
                while ret != 0 && (hr >= 0) && i < (COLCOUNT as u32)
                {
                    hr = if i == (COL_NAME as u32) { 0 } else { (*this_).CompareID(i as LPARAM, pidl1, pidl2) };
                    ret = (HRESULT_CODE!(hr) as i16) as i32;
                    i += 1;
                }
            }
        }
        else
        {
            let mut col: UINT = (lParam & SHCIDS_COLUMNMASK as LPARAM) as UINT;
            match (col) as i32
            {
                COL_NAME => {
                    ret = StrCmpLogicalW((*p1).GetName(), (*p2).GetName()) as i32;
                }

                COL_SIZE => {
                    ret = (*p1).Size.wrapping_sub((*p2).Size) as i32;
                }

                COL_MDATE => {
                    ret = (MAKELONG!((*p1).Time, (*p1).Date) as i32).wrapping_sub(MAKELONG!((*p2).Time, (*p2).Date) as i32);
                }

                _ => {
                    if col < (COLCOUNT as u32)
                    {
                        let mut str1: PWSTR = core::ptr::null_mut();
                        let mut str2: PWSTR = core::ptr::null_mut();
                        hr = GetDetailsOf(&mut (*this_).__base2, pidl1, col, core::ptr::addr_of_mut!(str1));
                        if hr >= 0
                        {
                            hr = GetDetailsOf(&mut (*this_).__base2, pidl2, col, core::ptr::addr_of_mut!(str2));
                            if hr >= 0
                            {
                                ret = StrCmpLogicalW(str1, str2);
                                SHFree((str2).cast());
                            }
                            SHFree((str1).cast());
                        }
                    }
                    else
                    {
                        hr = E_INVALIDARG;
                    }
                }
            }
        }
        if hr >= 0 { MAKE_COMPARE_HRESULT!(ret) } else { hr }
    }
    pub unsafe fn GetDefaultColumn(&mut self, dwRes: DWORD, pSort: *mut ULONG, pDisplay: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        if !pSort.is_null() {
            *pSort = (COL_NAME as u32);
        }
        if !pDisplay.is_null() {
            *pDisplay = (COL_NAME as u32);
        }
        return S_OK;
    }
    pub unsafe fn GetDefaultColumnState(&mut self, iColumn: UINT, pcsFlags: *mut SHCOLSTATEF) -> HRESULT {
        let this_: *mut Self = self;
        if pcsFlags.is_null() || iColumn >= _countof!(g_Columns) {
            return E_INVALIDARG;
        }
        *pcsFlags = (g_Columns[iColumn as usize].ColFlags) as _;
        return S_OK;
    }
    pub unsafe fn GetDetailsEx(&mut self, pidl: PCUITEMID_CHILD, pscid: *const SHCOLUMNID, pv: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if pscid.is_null() || pv.is_null() {
            return E_INVALIDARG;
        }

        let mut item: *mut CABITEM;
        let col: i32 = (*this_).MapSCIDToColumn(&*pscid);
        if col >= 0 {
            return (*this_).GetItemDetails(pidl, col as UINT, core::ptr::null_mut(), pv);
        } else if {
            item = CABITEM::Validate(pidl);
            item.is_null()
        } {
            return E_INVALIDARG;
        } else if IsEqual(pscid, core::ptr::addr_of!(FMTID_ShellDetails), PID_FINDDATA) != false {
            let mut wfd: WIN32_FIND_DATA = core::mem::zeroed();
            ZeroMemory!(
                core::ptr::addr_of_mut!(wfd) as *mut core::ffi::c_void,
                core::mem::size_of::<WIN32_FIND_DATA>(),
            );
            wfd.dwFileAttributes = (*item).GetFSAttributes() as DWORD;
            wfd.nFileSizeLow = (*item).Size as DWORD;
            DosDateTimeToFileTime(
                (*item).Date,
                (*item).Time,
                core::ptr::addr_of_mut!(wfd.ftLastWriteTime),
            );
            lstrcpyn(
                core::ptr::addr_of_mut!(wfd.cFileName) as LPWSTR,
                (*item).GetName(),
                (MAX_PATH as i32),
            );
            return InitVariantFromBuffer(
                core::ptr::addr_of!(wfd) as *const core::ffi::c_void,
                (core::mem::size_of::<WIN32_FIND_DATA>()) as UINT,
                pv,
            );
        }
        E_FAIL
    }
    pub unsafe fn GetDetailsOf(&mut self, pidl: PCUITEMID_CHILD, iColumn: UINT, psd: *mut SHELLDETAILS) -> HRESULT {
        let this_: *mut Self = self;
        if psd.is_null() || (iColumn as usize) >= (_countof!(g_Columns)) as usize {
            return E_INVALIDARG;
        } else if pidl.is_null() {
            (*psd).fmt = (g_Columns[iColumn as usize].LvcFmt) as _;
            (*psd).cxChar = (g_Columns[iColumn as usize].LvcChars) as _;
            let mut buf: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            if LoadStringW(_AtlBaseModule.GetResourceInstance(), (g_Columns[iColumn as usize].TextId) as UINT, buf.as_mut_ptr(), _countof!(buf) as i32) != 0 {
                return StrTo__1(buf.as_mut_ptr(), core::ptr::addr_of_mut!((*psd).str_));
            }
            return E_FAIL;
        }
        return (*this_).GetItemDetails(pidl, iColumn, psd, core::ptr::null_mut());
    }
    pub unsafe fn MapColumnToSCID(&mut self, column: UINT, pscid: *mut SHCOLUMNID) -> HRESULT {
        let this_: *mut Self = self;
        if (column as usize) < (_countof!(g_Columns)) as usize && !g_Columns[column as usize].pkg.is_null() {
            (*pscid).fmtid = *g_Columns[column as usize].pkg;
            (*pscid).pid = (g_Columns[column as usize].pki) as _;
            return S_OK;
        }
        return E_FAIL;
    }
    pub unsafe fn ParseDisplayName(&mut self, hwndOwner: HWND, pbc: LPBC, lpszDisplayName: LPOLESTR, pchEaten: *mut ULONG, ppidl: *mut PIDLIST_RELATIVE, pdwAttributes: *mut ULONG) -> HRESULT {
        let this_: *mut Self = self;
        *ppidl = core::ptr::null_mut();
        if !pchEaten.is_null() {
            *pchEaten = 0;
        }

        let mut hr: HRESULT = 0;
        let mut data: PDNCALLBACKDATA = PDNCALLBACKDATA { pszName: lpszDisplayName, pItem: core::ptr::null_mut() };
        hr = CEnumIDList::Enumerate((*this_).m_CurDir.as_ptr(), hwndOwner, Some(ParseDisplayNameCallback), (core::ptr::addr_of_mut!(data)).cast());
        if hr < 0 {
            return hr;
        }
        if data.pItem.is_null() {
            return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
        }

        *ppidl = data.pItem as PIDLIST_RELATIVE;
        if !pchEaten.is_null() {
            *pchEaten = lstrlenW(lpszDisplayName) as ULONG;
        }
        if !pdwAttributes.is_null() {
            (*this_).GetAttributesOf(1, ppidl as _, pdwAttributes as *mut SFGAOF);
        }
        return S_OK;
    }
    pub unsafe fn EnumObjects(&mut self, hwndOwner: HWND, dwFlags: DWORD, ppEnumIDList: *mut LPENUMIDLIST) -> HRESULT {
        let this_: *mut Self = self;
        let p: *mut CEnumIDList = CEnumIDList::CreateInstance();
        *ppEnumIDList = p as LPENUMIDLIST;
        return if !p.is_null() { (*p).Fill((*this_).m_CurDir.m_pData, hwndOwner, dwFlags) } else { E_OUTOFMEMORY };
    }
    pub unsafe fn BindToObject(&mut self, pidl: PCUIDLIST_RELATIVE, pbcReserved: LPBC, riid: &IID, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        C_ASSERT!(FLATFOLDER); // static_assert(FLATFOLDER, "Support Bind if we display the archive as a tree")
        UNIMPLEMENTED!();
        return E_NOTIMPL;
    }
    pub unsafe fn CompareIDs(&mut self, lParam: LPARAM, pidl1: PCUIDLIST_RELATIVE, pidl2: PCUIDLIST_RELATIVE) -> HRESULT {
        let this_: *mut Self = self;
        C_ASSERT!(FLATFOLDER);
        if pidl1.is_null() || ILIsSingle(pidl1) == 0 || pidl2.is_null() || ILIsSingle(pidl2) == 0 {
            return E_UNEXPECTED;
        }

        return (*this_).CompareID(lParam, pidl1, pidl2);
    }
    pub unsafe fn CreateViewObject(&mut self, hwndOwner: HWND, riid: &IID, ppv: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IShellView)) {
            let mut sfvc: SFV_CREATE = SFV_CREATE {
                cbSize: core::mem::size_of::<SFV_CREATE>() as UINT,
                pshf: this_ as *mut IShellFolder,
                psvOuter: core::ptr::null_mut(),
                psfvcb: core::ptr::null_mut(),
            };
            return SHCreateShellFolderView(core::ptr::addr_of_mut!(sfvc), ppv as *mut *mut IShellView);
        }
        if IsEqualGUID!(&(*riid), &(IID_IContextMenu)) {
            let func: LPFNDFMCALLBACK = Some(FolderBackgroundMenuCallback);
            let ppCM: *mut *mut IContextMenu = ppv as *mut *mut IContextMenu;
            return CDefFolderMenu_Create2((*this_).m_CurDir.m_pData, hwndOwner, 0, core::ptr::null_mut(), (this_ as *mut core::ffi::c_void).cast(), func, 0, core::ptr::null_mut(), ppCM);
        }
        return E_NOINTERFACE;
    }
    pub unsafe fn GetAttributesOf(&mut self, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, rgfInOut: *mut SFGAOF) -> HRESULT {
        let this_: *mut Self = self;
        if cidl == 0 {
            let ThisFolder: SFGAOF = (SFGAO_FOLDER | SFGAO_BROWSABLE | SFGAO_CANLINK);
            *rgfInOut = *rgfInOut & ThisFolder;
            return S_OK;
        } else if apidl.is_null() {
            return E_INVALIDARG;
        }
        let mut hr: HRESULT = S_OK;
        let impliedmask: SFGAOF = SFGAO_ISSLOW | SFGAO_CANCOPY;
        let filemask: SFGAOF = SFGAO_READONLY | SFGAO_HIDDEN | SFGAO_SYSTEM | impliedmask;
        let mut remain: SFGAOF = *rgfInOut & filemask;
        let mut validate: SFGAOF = *rgfInOut & SFGAO_VALIDATE;
        let mut list: CComPtr<CEnumIDList> = CComPtr::new();
        let mut i: UINT = 0;
        while i < cidl && (remain != 0 || validate != 0) {
            let item: *mut CABITEM = CABITEM::Validate(*apidl.add(i as usize));
            if item.is_null() {
                hr = E_INVALIDARG;
                break;
            } else if validate != 0 {
                if list.is_null() && FAILED_UNEXPECTEDLY!({ hr = (*this_).CreateEnum(list.GetAddressOf()); hr }) {
                    return hr;
                }
                if (*list.Get()).FindNamedItem(item as PCUITEMID_CHILD) == -(1) {
                    return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
                }
            }
            let att: SFGAOF = MapFSToSFAttributes(((*item).GetFSAttributes()) as UINT) | impliedmask;
            remain &= att & !(if (FLATFOLDER) != 0 { SFGAO_FOLDER } else { 0 });
            i += 1;
        }
        *rgfInOut = remain;
        return hr;
    }
    pub unsafe fn GetUIObjectOf(&mut self, hwndOwner: HWND, cidl: UINT, apidl: PCUITEMID_CHILD_ARRAY, riid: &IID, prgfInOut: *mut UINT, ppvOut: *mut LPVOID) -> HRESULT {
        let this_: *mut Self = self;
        let hr: HRESULT = E_NOINTERFACE;
        if IsEqualGUID!(&(*riid), &(IID_IExtractIconA)) || IsEqualGUID!(&(*riid), &(IID_IExtractIconW)) {
            if cidl != 1 {
                return E_INVALIDARG;
            }
            let item: *mut CABITEM = CABITEM::Validate(*apidl.add(0 as usize));
            if item.is_null() {
                return E_INVALIDARG;
            }

            let attr: DWORD = (*item).GetFSAttributes() as DWORD;
            return SHCreateFileExtractIconW((*item).GetName(), attr, riid, ppvOut);
        }
        else if IsEqualGUID!(&(*riid), &(IID_IContextMenu)) && cidl != 0 {
            let func: LPFNDFMCALLBACK = Some(ItemMenuCallback);
            let ppCM: *mut *mut IContextMenu = ppvOut as *mut *mut IContextMenu;
            return CDefFolderMenu_Create2(core::ptr::null_mut(), hwndOwner, cidl, apidl, core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder, func, 0, core::ptr::null_mut(), ppCM);
        }
        else if IsEqualGUID!(&(*riid), &(IID_IDataObject)) && cidl != 0 {
            // Note: This IDataObject is only compatible with IContextMenu, it cannot handle drag&drop of virtual items!
            return CIDLData_CreateFromIDArray((*this_).m_CurDir.m_pData, cidl, apidl, ppvOut as *mut *mut IDataObject);
        }
        return hr;
    }
    pub unsafe fn GetDisplayNameOf(&mut self, pidl: PCUITEMID_CHILD, dwFlags: DWORD, pName: LPSTRRET) -> HRESULT {
        let this_: *mut Self = self;
        let item: *mut CABITEM = CABITEM::Validate(pidl);
        if item.is_null() || pName.is_null() {
            return E_INVALIDARG;
        }

        if dwFlags & SHGDN_FORPARSING != 0 {
            if dwFlags & SHGDN_INFOLDER != 0 {
                return StrTo__1(
                    if ((FLATFOLDER) != 0) { (core::ptr::addr_of!((*item).Path) as *const WCHAR) } else { (*item).GetName() },
                    pName,
                );
            }

            let mut parent = [0 as WCHAR; (MAX_PATH) as _];
            if SHGetPathFromIDListW((*this_).m_CurDir.m_pData, parent.as_mut_ptr()) == 0 {
                return E_FAIL;
            }
            let mut cch: UINT = (lstrlenW(parent.as_ptr())
                + 1
                + lstrlenW((core::ptr::addr_of!((*item).Path) as *const WCHAR))
                + 1) as u32;
            (*pName).uType = STRRET_WSTR;
            (*pName).__bindgen_anon_1.pOleStr = SHAlloc(
                ((cch as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
            ) as LPWSTR;
            if (*pName).__bindgen_anon_1.pOleStr.is_null() {
                return E_OUTOFMEMORY;
            }
            lstrcpyW((*pName).__bindgen_anon_1.pOleStr, parent.as_ptr());
            PathAppendW((*pName).__bindgen_anon_1.pOleStr, (core::ptr::addr_of!((*item).Path) as *const WCHAR));
            return S_OK;
        }

        let mut fi: SHFILEINFO = core::mem::zeroed();
        let attr: DWORD = if (*item).IsFolder() { FILE_ATTRIBUTE_DIRECTORY } else { 0 };
        let flags: UINT = SHGFI_DISPLAYNAME | SHGFI_USEFILEATTRIBUTES;
        if SHGetFileInfo(
            (*item).GetName(),
            attr,
            core::ptr::addr_of_mut!(fi),
            (core::mem::size_of::<SHFILEINFO>()) as _,
            flags,
        ) != 0
        {
            return StrTo__1((core::ptr::addr_of!(fi.szDisplayName) as *const WCHAR), pName);
        }
        return StrTo__1((*item).GetName(), pName);
    }
    pub unsafe fn MessageSFVCB(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> HRESULT {
        let this_: *mut Self = self;
        match uMsg {
            SFVM_WINDOWCREATED => {
                (*this_).m_ShellViewWindow = wParam as HWND;
                return S_OK;
            }
            SFVM_WINDOWCLOSING => {
                (*this_).m_ShellViewWindow = core::ptr::null_mut();
                return S_OK;
            }
            _ => {}
        }
        return E_NOTIMPL;
    }
    pub unsafe fn GetIconOf(&mut self, pidl: PCUITEMID_CHILD, flags: UINT, pIconIndex: *mut i32) -> HRESULT {
        let this_: *mut Self = self;
        let item: *mut CABITEM = CABITEM::Validate(pidl);
        if !item.is_null() {
            let mut index: i32 = MapPIDLToSystemImageListIndex(core::ptr::addr_of_mut!((*this_).__base2) as *mut IShellFolder, pidl, flags);
            if index == -1 && (*item).IsFolder() {
                index = if (flags & GIL_OPENICON) != 0 { (SIID_FOLDEROPEN as i32) } else { (SIID_FOLDER as i32) };
            }
            if index != -1 {
                *pIconIndex = index;
                return S_OK;
            }
        }
        return S_FALSE;
    }
}

impl FOLDERCOLUMN {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CABITEM {
    pub unsafe fn IsFolder(&mut self) -> bool {
        let this_: *mut Self = self;
        let _ = this_;
        return false;
    }
    pub unsafe fn GetFSAttributes(&mut self) -> WORD {
        let this_: *mut Self = self;
        return (*this_).Attrib & (FSATTS as u16);
    }
    pub unsafe fn GetName(&mut self) -> LPCWSTR {
        let this_: *mut Self = self;
        return (core::ptr::addr_of_mut!((*this_).Path).cast::<WCHAR>()).add((*this_).NameOffset as usize) as LPCWSTR;
    }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl FILLCALLBACKDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl PDNCALLBACKDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl EXTRACTFILESDATA {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CABITEM {
    /// folder.cpp's member TEMPLATE `template<class PIDL> static CABITEM* Validate(PIDL pidl)`,
    /// hand-ported (cppapp does not generate templates): `pidl` is ours when it is long enough
    /// to hold a name (`cb > FIELD_OFFSET(CABITEM, Path[1])`) and carries our 0 in `Unknown`.
    pub unsafe fn Validate<T>(pidl: *const T) -> *mut CABITEM {
        let p = pidl as *mut CABITEM;
        let path1 = core::mem::offset_of!(CABITEM, Path) + core::mem::size_of::<WCHAR>();
        if !p.is_null() && (*p).cb as usize > path1 && (*p).Unknown == 0 {
            p
        } else {
            core::ptr::null_mut()
        }
    }
}
