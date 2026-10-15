use crate::prelude::*;
use crate::rapps::geninst_cpp_h::ExtractArchiveForExecution;
use crate::rapps::integrity_cpp_h::VerifyInteg;
use crate::rapps::misc_cpp_h::{GetSilentInstallParameters, GuessInstallerType};
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_1B4F33C91A81: [WCHAR; 10] = [0x0052, 0x0041, 0x0070, 0x0070, 0x0073, 0x002F, 0x0031, 0x002E, 0x0031, 0x0000];
pub static CPP_COM_WSTR_1B9334FEECE6: [WCHAR; 2] = [0x0022, 0x0000];
pub static CPP_COM_WSTR_3A7F15C7DA56: [WCHAR; 9] = [0x002A, 0x003C, 0x003D, 0x003E, 0x003A, 0x002F, 0x005C, 0x003F, 0x0000];
pub static CPP_COM_WSTR_5853A555D865: [WCHAR; 4] = [0x0020, 0x002F, 0x0053, 0x0000];
pub static CPP_COM_WSTR_632A8A3E5C84: [WCHAR; 3] = [0x0020, 0x0022, 0x0000];
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000];
pub static CPP_COM_WSTR_85D5870D46C2: [WCHAR; 4] = [0x0022, 0x0020, 0x0022, 0x0000];
pub static CPP_COM_WSTR_8BE96111068E: [WCHAR; 19] = [0x0025, 0x0075, 0x0025, 0x0025, 0x0020, 0x0020, 0x0031, 0x0034, 0x0020, 0x0025, 0x006C, 0x0073, 0x0020, 0x002F, 0x0020, 0x0025, 0x006C, 0x0073, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_9BBDA5970E3F: [WCHAR; 7] = [0x0025, 0x006C, 0x0073, 0x002E, 0x002E, 0x002E, 0x0000];
pub static CPP_COM_WSTR_BDC14411E891: [WCHAR; 12] = [0x0077, 0x0069, 0x006E, 0x0069, 0x006E, 0x0065, 0x0074, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static CPP_COM_WSTR_C2B264989196: [WCHAR; 9] = [0x002F, 0x0047, 0x0045, 0x004E, 0x0049, 0x004E, 0x0053, 0x0054, 0x0000];
pub static CPP_COM_WSTR_D217C45C6798: [WCHAR; 4] = [0x0025, 0x006C, 0x0073, 0x0000];
pub static CPP_COM_WSTR_F1C8A68A6FAF: [WCHAR; 12] = [0x006D, 0x0073, 0x0069, 0x0065, 0x0078, 0x0065, 0x0063, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut g_hDownloadWnd: HWND = core::ptr::null_mut();

static mut CertGetSubjectAndIssuer_dummy: i32 = 0;































unsafe extern "C" fn IsTrustedPinnedCert(Subject: LPCSTR, Issuer: LPCSTR) -> bool {
    extern "C" {
        fn strcmp(a: *const core::ffi::c_char, b: *const core::ffi::c_char) -> core::ffi::c_int;
    }
    // CERT_SUBJECT_INFO
    if strcmp(Subject, b"rapps.reactos.org\0".as_ptr() as LPCSTR) != 0 {
        return false;
    }
    // CERT_ISSUER_INFO_PREFIX is defined, so the C++ takes the StrStrA prefix branch (not the
    // CERT_ISSUER_INFO_OLD/NEW strcmp branch of the #else).
    return Issuer == StrStrA(Issuer, b"US\r\nLet's Encrypt\r\nR\0".as_ptr() as LPCSTR) as LPCSTR
        || Issuer == StrStrA(Issuer, b"US\r\nLet's Encrypt\r\nYR\0".as_ptr() as LPCSTR) as LPCSTR;
}
#[no_mangle]
pub unsafe extern "C" fn LoadStatusString(StatusParam: DownloadStatus) -> CStringW {
    let mut szString = CStringW::new();
    szString.LoadStringW(StatusParam as UINT);
    return szString;
}
#[no_mangle]
pub unsafe extern "C" fn UrlUnescapeAndMakeFileNameValid(str: &mut CStringW) {
    let mut szPath = [0u16; (MAX_PATH) as _];
    let mut cchPath: DWORD = _countof!(szPath) as DWORD;
    UrlUnescapeW(str.GetString() as *mut WCHAR, szPath.as_mut_ptr(), core::ptr::addr_of_mut!(cchPath), 0);

    let mut pch: PWCHAR = szPath.as_mut_ptr();
    while *pch != 0 {
        if PathIsValidCharW(*pch, (FILENAME_VALID_CHAR) as DWORD) == 0 {
            *pch = 0x005Fu16;
        }
        pch = pch.add(1);
    }

    *str = CStringW::from_ptr(szPath.as_ptr());
}

unsafe extern "C" fn SetFriendlyUrl(hWnd: HWND, mut pszUrl: LPCWSTR) {
    let mut buf = CStringW::new();
    let mut cch: DWORD = (wcslen(pszUrl) as DWORD + 1);
    if InternetCanonicalizeUrlW(pszUrl, buf.GetBuffer((cch) as i32), core::ptr::addr_of_mut!(cch), ICU_DECODE | ICU_NO_ENCODE) != 0
    {
        buf.ReleaseBuffer();
        pszUrl = buf.GetString();
    }
    SetWindowTextW(hWnd, pszUrl);
}

unsafe extern "C" fn CertGetSubjectAndIssuer(hFile: HINTERNET, subjectInfo: &mut CLocalPtr_char, issuerInfo: &mut CLocalPtr_char) -> BOOL {
    let mut certInfoLength: DWORD;
    let mut certInfo: INTERNET_CERTIFICATE_INFOA = core::mem::zeroed();
    let mut size: DWORD;
    let mut flags: DWORD = 0;

    size = core::mem::size_of::<DWORD>() as DWORD;
    if !(InternetQueryOptionA(hFile, INTERNET_OPTION_SECURITY_FLAGS, core::ptr::addr_of_mut!(flags) as *mut core::ffi::c_void, core::ptr::addr_of_mut!(size)) != 0) {
        return (FALSE) as BOOL;
    }

    if (((flags == 0) as DWORD) & SECURITY_FLAG_SECURE) != 0 {
        return (FALSE) as BOOL;
    }

    /* Despite what the header indicates, the implementation of INTERNET_CERTIFICATE_INFO is not Unicode-aware. */
    certInfoLength = core::mem::size_of::<INTERNET_CERTIFICATE_INFOA>() as DWORD;
    if !(InternetQueryOptionA(hFile, INTERNET_OPTION_SECURITY_CERTIFICATE_STRUCT, core::ptr::addr_of_mut!(certInfo) as *mut core::ffi::c_void, core::ptr::addr_of_mut!(certInfoLength)) != 0) {
        return (FALSE) as BOOL;
    }

    (*subjectInfo).__base0.Attach(certInfo.lpszSubjectInfo);
    (*issuerInfo).__base0.Attach(certInfo.lpszIssuerInfo);

    if !certInfo.lpszProtocolName.is_null() {
        LocalFree(certInfo.lpszProtocolName as _);
    }
    if !certInfo.lpszSignatureAlgName.is_null() {
        LocalFree(certInfo.lpszSignatureAlgName as _);
    }
    if !certInfo.lpszEncryptionAlgName.is_null() {
        LocalFree(certInfo.lpszEncryptionAlgName as _);
    }

    (!certInfo.lpszSubjectInfo.is_null() && !certInfo.lpszIssuerInfo.is_null()) as BOOL
}

unsafe extern "C" fn MessageBox_LoadString(hOwnerWnd: HWND, StringID: INT) {
    let mut szMsgText = CStringW::new();
    if szMsgText.LoadStringW(StringID as UINT) != 0 {
        MessageBoxW(hOwnerWnd, szMsgText.GetString(), core::ptr::null_mut(), MB_OK | MB_ICONERROR);
    }
}

unsafe extern "C" fn ShowLastError(mut hWndOwner: HWND, bInetError: BOOL, dwLastError: DWORD) -> BOOL {
    let mut lpMsg = CLocalPtr_wchar_t__make_0();

    if FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_IGNORE_INSERTS |
                (if bInetError != 0 { FORMAT_MESSAGE_FROM_HMODULE } else { FORMAT_MESSAGE_FROM_SYSTEM }),
            (if bInetError != 0 { GetModuleHandleW(CPP_COM_WSTR_BDC14411E891.as_ptr()) as *const core::ffi::c_void } else { core::ptr::null() }),
            dwLastError,
            LANG_USER_DEFAULT,
            core::ptr::addr_of_mut!(lpMsg.__base0) as LPWSTR,
            0,
            core::ptr::null_mut()) == 0
    {
        DPRINT1!("FormatMessageW unexpected failure (err %d)\n", GetLastError());
        return (FALSE) as BOOL;
    }

    if !hWndOwner.is_null() && IsWindowVisible(hWndOwner) == 0 {
        hWndOwner = core::ptr::null_mut();
    }
    MessageBoxW(hWndOwner, lpMsg.__base0.m_pData, core::ptr::null_mut(), MB_OK | MB_ICONERROR);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DownloadListOfApplications(AppsList: &CAtlList<*mut CAppInfo>, Flags: UINT) -> BOOL {
    if AppsList.IsEmpty() {
        return (FALSE) as BOOL;
    }

    let mut pDM: CComPtr<CDownloadManager> = CComPtr::new();
    if ((CDownloadManager::CreateInstance((Flags) as i32, &mut pDM)) == 0) {
        return (FALSE) as BOOL;
    }

    let mut it: POSITION = AppsList.GetHeadPosition();
    while !it.is_null() {
        let Info: *const CAppInfo = *AppsList.GetNext(&mut it);
        (*pDM.Get()).Add(&DownloadInfo__make_1(&*Info, Flags));
    }
    (*pDM.Get()).Show();
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn DownloadApplication(pAppInfo: *mut CAppInfo) -> BOOL {
    if pAppInfo.is_null() {
        return (FALSE) as BOOL;
    }

    let mut list: CAtlList<*mut CAppInfo> = CAtlList::new();
    list.AddTail(pAppInfo);
    return DownloadListOfApplications(&list, 0);
}
#[no_mangle]
pub unsafe extern "C" fn DownloadApplicationsDB(lpUrl: LPCWSTR, IsOfficial: BOOL) {
    let mut pDM: CComPtr<CDownloadManager> = CComPtr::new();
    if CDownloadManager::CreateInstance((DAF_MODAL) as i32, &mut pDM) == 0 {
        return;
    }

    let mut DatabaseDLInfo: DownloadInfo = DownloadInfo__make_0();
    DatabaseDLInfo.szUrl = CStringW::from_ptr(lpUrl);
    DatabaseDLInfo.szName.LoadStringW(IDS_DL_DIALOG_DB_DISP);
    DatabaseDLInfo.DLType = if IsOfficial != 0 { DLTYPE_DBUPDATE } else { DLTYPE_DBUPDATE_UNOFFICIAL };

    (*pDM.Get()).Add(&DatabaseDLInfo);
    (*pDM.Get()).Show();
}

impl DownloadInfo {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).DLType = DLTYPE_APPLICATION;
        (*this_).IType = INSTALLER_UNKNOWN;
        (*this_).IExecType = INSTALLER_UNKNOWN;
        (*this_).Flags = 0;
        (*this_).SizeInBytes = 0;
        {
        }
    }
    pub unsafe fn __ctor_1(&mut self, AppInfo: &CAppInfo, DAF: UINT) {
        let this_: *mut Self = self;
        // The C++ parameter is `const CAppInfo &` and calls its (non-const here) methods.
        let AppInfo: *mut CAppInfo = AppInfo as *const CAppInfo as *mut CAppInfo;
        (*this_).DLType = DLTYPE_APPLICATION;
        (*this_).IExecType = INSTALLER_UNKNOWN;
        (*this_).Flags = DAF;
        {
            (*AppInfo).GetDownloadInfo(&mut (*this_).szUrl, &mut (*this_).szSHA1, &mut (*this_).SizeInBytes);
            (*this_).szName = (*AppInfo).szDisplayName.clone();
            (*this_).IType = (*AppInfo).GetInstallerType(false);
            (*this_).szPackageName = (*AppInfo).szIdentifier.clone();

            if ((*this_).Flags & DAF_SILENT) != 0 {
                (*this_).IExecType = (*AppInfo).GetInstallerInfo(&mut (*this_).szSilentInstallArgs);
            }

            let cfg: *mut CConfigParser = (*(AppInfo as *mut CAvailableApplicationInfo)).GetConfigParser();
            if !cfg.is_null() {
                (*cfg).GetString(&CStringW::from_ptr(DB_SAVEAS.as_ptr()), &mut (*this_).szFileName);
                (*cfg).GetString(&CStringW::from_ptr(DB_DEPENDENCIES.as_ptr()), &mut (*this_).szDependencies);
            }
        }
    }
    pub unsafe fn Equal(&self, other: &DownloadInfo) -> bool {
        let this_: *const Self = self;
        return (*this_).DLType == other.DLType && lstrcmpW((*this_).szUrl.GetString(), other.szUrl.GetString()) == 0;
    }
}

impl CDownloaderProgress {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        {
        }
    }
    pub unsafe fn SetMarquee(&mut self, Enable: BOOL) {
        let this_: *mut Self = self;
        if Enable != 0 {
            (*this_).__base0.ModifyStyle(0, PBS_MARQUEE, 0);
        } else {
            (*this_).__base0.ModifyStyle(PBS_MARQUEE, 0, 0);
        }

        (*this_).__base0.SendMessageW__0(PBM_SETMARQUEE, Enable as WPARAM, 0);
    }
    pub unsafe fn SetProgress(&mut self, ulProgress: ULONG, ulProgressMax: ULONG) {
        let this_: *mut Self = self;
        let mut szProgress: [WCHAR; 100] = [0; 100];

        /* format the bits and bytes into pretty and accessible units... */
        StrFormatByteSizeW((ulProgress) as LONGLONG, szProgress.as_mut_ptr(), _countof!(szProgress));

        /* use our subclassed progress bar text subroutine */
        let mut ProgressText = CStringW::new();

        if ulProgressMax != 0 {
            /* total size is known */
            let mut szProgressMax: [WCHAR; 100] = [0; 100];
            let uiPercentage: UINT = (((ulProgress as ULONGLONG) * 100) / (ulProgressMax as ULONGLONG)) as UINT;

            /* send the current progress to the progress bar */
            if (((*this_).__base0.IsWindow()) == 0) {
                return;
            }
            (*this_).__base0.SendMessageW__0(PBM_SETPOS, uiPercentage as WPARAM, 0);

            /* format total download size */
            StrFormatByteSizeW((ulProgressMax) as LONGLONG, szProgressMax.as_mut_ptr(), _countof!(szProgressMax));

            /* generate the text on progress bar */
            atl_format!(ProgressText, CPP_COM_WSTR_8BE96111068E.as_ptr(), uiPercentage, szProgress.as_ptr(), szProgressMax.as_ptr());
        } else {
            /* send the current progress to the progress bar */
            if (((*this_).__base0.IsWindow()) == 0) {
                return;
            }
            (*this_).__base0.SendMessageW__0(PBM_SETPOS, 0, 0);

            /* total size is not known, display only current size */
            atl_format!(ProgressText, CPP_COM_WSTR_9BBDA5970E3F.as_ptr(), szProgress.as_ptr());
        }

        /* and finally display it */
        if (((*this_).__base0.IsWindow()) == 0) {
            return;
        }
        (*this_).__base0.SetWindowTextW(ProgressText.GetString());
    }
    pub unsafe fn OnEraseBkgnd(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
            return (TRUE) as LRESULT;
        }
    pub unsafe fn OnPaint(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut ps: PAINTSTRUCT = core::mem::zeroed();
        let mut hDC: HDC = (*this_).__base0.BeginPaint(core::ptr::addr_of_mut!(ps));
        let mut hdcMem: HDC;
        let mut hbmMem: HBITMAP;
        let mut hOld: HANDLE;
        let mut myRect: RECT = core::mem::zeroed();
        let mut win_width: UINT;
        let mut win_height: UINT;

        (*this_).__base0.GetClientRect(core::ptr::addr_of_mut!(myRect));

        /* grab the progress bar rect size */
        win_width = (myRect.right - myRect.left) as UINT;
        win_height = (myRect.bottom - myRect.top) as UINT;

        /* create an off-screen DC for double-buffering */
        hdcMem = CreateCompatibleDC(hDC);
        hbmMem = CreateCompatibleBitmap(hDC, (win_width) as INT, (win_height) as INT);

        hOld = SelectObject(hdcMem, (hbmMem).cast());

        /* call the original draw code and redirect it to our memory buffer */
        (*this_).DefWindowProc(uMsg, hdcMem as WPARAM, lParam);

        /* draw our nifty progress text over it */
        SelectFont!(hdcMem, GetStockFont!(DEFAULT_GUI_FONT));
        DrawShadowText(
            hdcMem, (*this_).m_szProgressText.GetString(), (*this_).m_szProgressText.GetLength() as UINT, core::ptr::addr_of_mut!(myRect),
            DT_CENTER | DT_VCENTER | DT_NOPREFIX | DT_SINGLELINE, GetSysColor((COLOR_CAPTIONTEXT as i32)),
            GetSysColor((COLOR_3DDKSHADOW as i32)), 1, 1);

        /* transfer the off-screen DC to the screen */
        BitBlt(hDC, 0, 0, (win_width) as INT, (win_height) as INT, hdcMem, 0, 0, SRCCOPY);

        /* free the off-screen DC */
        SelectObject(hdcMem, hOld);
        DeleteObject((hbmMem).cast());
        DeleteDC(hdcMem);

        (*this_).__base0.EndPaint(core::ptr::addr_of_mut!(ps));
        return 0;
    }
    pub unsafe fn OnSetText(&mut self, uMsg: UINT, wParam: WPARAM, lParam: LPARAM, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let pszText: PCWSTR = lParam as PCWSTR;
        if !pszText.is_null() {
            if (*this_).m_szProgressText != CStringW::from_ptr(pszText) {
                (*this_).m_szProgressText = CStringW::from_ptr(pszText);
                (*this_).__base0.InvalidateRect(core::ptr::null_mut(), (TRUE as i32));
            }
        } else {
            if !(*this_).m_szProgressText.IsEmpty() {
                (*this_).m_szProgressText.Empty();
                (*this_).__base0.InvalidateRect(core::ptr::null_mut(), (TRUE as i32));
            }
        }
        return (TRUE) as LRESULT;
    }
}

impl CDowloadingAppsListView {
    pub unsafe fn Create(&mut self, hwndParent: HWND) -> HWND {
        let this_: *mut Self = self;
        let mut r: RECT = core::mem::zeroed();
        GetClientRect(hwndParent, core::ptr::addr_of_mut!(r));
        r.top = (2 * r.top + 1 * r.bottom) / 3; /* The vertical position at ratio 1 : 2 */
        let MARGIN: INT = 10;
        InflateRect(core::ptr::addr_of_mut!(r), -MARGIN, -MARGIN);

        let style: DWORD = WS_CHILD | WS_VISIBLE | LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS | LVS_NOSORTHEADER |
            LVS_NOCOLUMNHEADER;

        let hwnd: HWND = (*this_).__base0.Create(hwndParent, ATL__U_RECT { m_lpRect: core::ptr::addr_of_mut!(r) }, core::ptr::null(), style, WS_EX_CLIENTEDGE, ATL__U_MENUorID { m_hMenu: core::ptr::null_mut() }, core::ptr::null_mut());

        (*this_).AddColumn(0, 150, (LVCFMT_LEFT as i32));
        (*this_).AddColumn(1, 120, (LVCFMT_LEFT as i32));

        return hwnd;
    }
    // C++ takes the array BY VALUE (a deep copy); the body only reads it, and atl_rust's CSimpleArray
    // has no copy constructor, so it is borrowed (no observable difference).
    pub unsafe fn LoadList(&mut self, arrInfo: &CSimpleArray<DownloadInfo>, Start: UINT) {
        let this_: *mut Self = self;
        let base: INT = (*this_).__base0.GetItemCount();
        let mut i: INT = Start as INT;
        while i < arrInfo.GetSize() {
            (*this_).AddRow(base + i - (Start as INT), (*arrInfo.GetAt(i)).szName.GetString(), DLSTATUS_WAITING);
            i += 1;
        }
    }
    pub unsafe fn SetDownloadStatus(&mut self, ItemIndex: INT, Status: DownloadStatus) {
        let this_: *mut Self = self;
        let mut szBuffer = LoadStatusString(Status);
        (*this_).__base0.SetItemText(ItemIndex, 1, szBuffer.GetString());
    }
    pub unsafe fn AddItem(&mut self, ItemIndex: INT, lpText: LPWSTR) -> BOOL {
        let this_: *mut Self = self;
        let mut Item: LVITEMW = core::mem::zeroed();

        ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut _, core::mem::size_of::<LVITEMW>());

        Item.mask = LVIF_TEXT | LVIF_STATE;
        Item.pszText = lpText;
        Item.iItem = ItemIndex;

        return (*this_).__base0.InsertItem(core::ptr::addr_of_mut!(Item));
    }
    pub unsafe fn AddRow(&mut self, RowIndex: INT, szAppName: LPCWSTR, Status: DownloadStatus) {
        let this_: *mut Self = self;
        let mut szStatus = LoadStatusString(Status);
        (*this_).AddItem(RowIndex, szAppName as LPWSTR);
        (*this_).SetDownloadStatus(RowIndex, Status);
    }
    pub unsafe fn AddColumn(&mut self, Index: INT, Width: INT, Format: INT) -> BOOL {
        let this_: *mut Self = self;
            let mut Column: LVCOLUMNW = core::mem::zeroed();
            ZeroMemory!(core::ptr::addr_of_mut!(Column) as *mut core::ffi::c_void, core::mem::size_of::<LVCOLUMNW>());

            Column.mask = LVCF_FMT | LVCF_WIDTH | LVCF_SUBITEM;
            Column.iSubItem = Index;
            Column.cx = Width;
            Column.fmt = Format;

            return if (*this_).__base0.InsertColumn__0(Index, core::ptr::addr_of_mut!(Column)) == -1 { (FALSE as i32) } else { (TRUE as i32) };
        }
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0();
    }
}

impl CDownloadManager {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_hDlg = core::ptr::null_mut();
        (*this_).m_Threads = 0;
        (*this_).m_Index = 0;
        (*this_).m_bCancelled = (FALSE as i32);
        (*this_).m_fDaf = 0;
        {}
    }
    pub unsafe fn CreateInstanceHelper(Flags: UINT) -> *mut CDownloadManager {
        if (Flags & DAF_MODAL) == 0 {
            let mut pExisting: *mut CDownloadManager = CDownloadManager::FindInstance();
            if !pExisting.is_null() {
                (*pExisting).AddRef();
                return pExisting;
            }
        }
        let mut obj: CComPtr<CDownloadManager> = CComPtr::new();
        if (ShellObjectCreator(&mut obj) as HRESULT) < (0 as HRESULT) {
            return core::ptr::null_mut();
        }
        (*obj.Get()).m_fDaf = Flags;
        (*obj.Get()).m_bModal = (((Flags & DAF_MODAL) != 0) as BOOL);
        return obj.Detach();
    }
    pub unsafe fn CreateInstance(Modal: BOOL, Obj: &mut CComPtr<CDownloadManager>) -> BOOL {
        let p: *mut CDownloadManager = CDownloadManager::CreateInstanceHelper((Modal) as UINT);
        if p.is_null() {
            return (FALSE) as BOOL;
        }
        Obj.Attach(p);
        return (TRUE) as BOOL;
    }
    pub unsafe fn FindInstance() -> *mut CDownloadManager {
        if (g_hDownloadWnd as usize) != 0 && IsWindowVisible(g_hDownloadWnd) != 0 {
            return SendMessageW(g_hDownloadWnd, WM_GETINSTANCE, 0, 0) as *mut CDownloadManager;
        }
        return core::ptr::null_mut();
    }
    pub unsafe fn IsCancelled(&mut self) -> BOOL {
        let this_: *mut Self = self;
        return ((IsWindow((*this_).m_hDlg) == 0) || (SendMessageW((*this_).m_hDlg, WM_ISCANCELLED, 0, 0) != 0)) as BOOL;
    }
    pub unsafe fn IsSilentDialog(&mut self) -> BOOL {
        let this_: *mut Self = self;
            return ((*this_).m_fDaf & DAF_SILENT) as BOOL;
        }
    pub unsafe fn StartWorkerThread(&mut self) {
        let this_: *mut Self = self;
        (*this_).AddRef(); // To keep m_List alive in thread
        let mut ThreadId: u32 = 0;
        let Thread: HANDLE = _beginthreadex(
            core::ptr::null_mut(),
            0,
            Some(CDownloadManager::ThreadFunc),
            this_ as *mut core::ffi::c_void,
            0,
            core::ptr::addr_of_mut!(ThreadId),
        ) as usize as HANDLE;
        if Thread != core::ptr::null_mut() {
            CloseHandle(Thread);
        } else {
            (*this_).Release();
        }
    }
    pub unsafe fn Add(&mut self, Info: &DownloadInfo) {
        let this_: *mut Self = self;
        let count: UINT = ((*this_).m_List.GetSize()) as u32;
        let start: UINT = count;
        let mut i: UINT = 0;
        while i < count {
            if Info.Equal(&*(*this_).m_List.GetAt(i as i32)) {
                return; // Already in the list
            }
            i += 1;
        }

        if !Info.szDependencies.IsEmpty() {
            let mut deps: CStringW = CStringW::from_ptr(Info.szDependencies.GetString());
            let mut pos: i32 = 1;
            loop {
                if !(pos > 0) {
                    break;
                }
                'body: {
                    pos = deps.Find(0x007Cu16, 0);
                    let mut pkg: CStringW = if pos > 0 {
                        deps.Left(pos)
                    } else {
                        CStringW::from_ptr(deps.GetString())
                    };
                    pkg.Trim();

                    // In the future a package might need to specify dependency version or WoW64
                    // information ("vcredist*x64" etc), for now, just remove possible modifiers.
                    // CStringW::FindOneOf (wcspbrk): atl_rust has none, so scan for the first
                    // character of pkg that is in the set.
                    let suffix: i32 = {
                        let set: *const WCHAR = CPP_COM_WSTR_3A7F15C7DA56.as_ptr();
                        let mut found: i32 = -1;
                        let mut k: i32 = 0;
                        while k < pkg.GetLength() && found < 0 {
                            let ch: WCHAR = pkg.GetAt(k);
                            let mut q: *const WCHAR = set;
                            while *q != 0 {
                                if *q == ch {
                                    found = k;
                                    break;
                                }
                                q = q.add(1);
                            }
                            k += 1;
                        }
                        found
                    };
                    if suffix > 0 {
                        pkg = pkg.Left(suffix);
                    }

                    let pApp: *mut CAvailableApplicationInfo =
                        CAppDB::CreateAvailableAppInstance(&pkg, core::ptr::null_mut());
                    let _del = Deleter_CAvailableApplicationInfo__make_0(pApp);
                    if pApp.is_null() || (*pApp).IsInstalled(core::ptr::null_mut()) {
                        break 'body;
                    }
                    // Add the dependency before the application in the list
                    (*this_).Add(&DownloadInfo__make_1(
                        &(*pApp).__base0,
                        DAF_SILENT,
                    ));
                }
                // for-loop update: deps = deps.Mid(pos + FACT_0003)
                deps = deps.Mid(pos + 1);
            }
        }

        // m_List.Add(Info): CSimpleArray copy-constructs the element (member-wise copy).
        let mut InfoCopy = DownloadInfo__make_0();
        InfoCopy.DLType = Info.DLType;
        InfoCopy.IType = Info.IType;
        InfoCopy.IExecType = Info.IExecType;
        InfoCopy.Flags = Info.Flags;
        InfoCopy.szUrl = Info.szUrl.clone();
        InfoCopy.szName = Info.szName.clone();
        InfoCopy.szSHA1 = Info.szSHA1.clone();
        InfoCopy.szPackageName = Info.szPackageName.clone();
        InfoCopy.szFileName = Info.szFileName.clone();
        InfoCopy.szSilentInstallArgs = Info.szSilentInstallArgs.clone();
        InfoCopy.szDependencies = Info.szDependencies.clone();
        InfoCopy.SizeInBytes = Info.SizeInBytes;
        (*this_).m_List.Add(InfoCopy);
        if !(*this_).m_hDlg.is_null() {
            (*this_).m_ListView.LoadList(&(*this_).m_List, start);
        }
    }
    pub unsafe fn Show(&mut self) {
        let this_: *mut Self = self;
        let mut hDlg: HWND = core::ptr::null_mut();
        let bSilent: BOOL = (*this_).IsSilentDialog();
        let bModal: BOOL = (*this_).m_bModal;
        if bModal != 0 && bSilent == 0 {
            DialogBoxParamW(hInst, MAKEINTRESOURCEW!(IDD_DOWNLOAD_DIALOG), hMainWnd, core::mem::transmute(CDownloadManager::DlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR), this_ as LPARAM);
        } else if (*this_).m_hDlg.is_null() || IsWindow((*this_).m_hDlg) == 0 {
            hDlg = CreateDialogParamW(hInst, MAKEINTRESOURCEW!(IDD_DOWNLOAD_DIALOG), hMainWnd, core::mem::transmute(CDownloadManager::DlgProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR), this_ as LPARAM);
        }

        // A DialogBox dialog cannot be invisible, it is forced visible after WM_INITDIALOG returns.
        // We therefore use a modeless dialog when we are both modal and silent.
        let mut msg: MSG = core::mem::zeroed();
        while bModal != 0 && bSilent != 0 && !hDlg.is_null() && GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
            DispatchMessage(core::ptr::addr_of_mut!(msg));
        }
    }
    pub unsafe extern "system" fn DlgProc(hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
        let mut pThis: *mut CDownloadManager = GetWindowLongPtrW(hDlg, DWLP_USER) as *mut CDownloadManager;
        if pThis.is_null() {
            if uMsg != WM_INITDIALOG {
                return (FALSE) as INT_PTR;
            }
            SetWindowLongPtrW(hDlg, DWLP_USER, lParam);
            pThis = lParam as *mut CDownloadManager;
        }
        return (*pThis).RealDlgProc(hDlg, uMsg, wParam, lParam);
    }
    pub unsafe fn RealDlgProc(&mut self, hDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
        let this_: *mut Self = self;
        match uMsg {
            WM_INITDIALOG => {
                g_Busy += 1;
                (*this_).AddRef();
                (*this_).m_hDlg = hDlg;
                if (*this_).m_bModal == 0 {
                    g_hDownloadWnd = hDlg;
                }

                let mut hIconSm: HICON = core::ptr::null_mut();
                let mut hIconBg: HICON = core::ptr::null_mut();
                if !hMainWnd.is_null() {
                    hIconBg = GetClassLongPtrW(hMainWnd, GCLP_HICON) as HICON;
                    hIconSm = GetClassLongPtrW(hMainWnd, GCLP_HICONSM) as HICON;
                }
                if hMainWnd.is_null() || (hIconBg.is_null() || hIconSm.is_null()) {
                    hIconBg = LoadIconW(hInst, MAKEINTRESOURCEW!(IDI_MAIN)) as HICON;
                    hIconSm = hIconBg;
                }
                SendMessageW(hDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIconBg as LPARAM);
                SendMessageW(hDlg, WM_SETICON, (ICON_SMALL) as WPARAM, hIconSm as LPARAM);

                GetWindowTextW(hDlg, (*this_).m_szCaptionFmt.as_mut_ptr(), (_countof!((*this_).m_szCaptionFmt)) as i32);
                let mut buf: CStringW = CStringW::new();
                buf = CStringW::from_ptr((*this_).m_szCaptionFmt.as_ptr());
                // buf.Replace(L"%ls", L""): atl_rust's CStringW only has the one-character
                // Replace, so rebuild the string with every occurrence of the pattern replaced.
                {
                    let pszOld: *const WCHAR = CPP_COM_WSTR_D217C45C6798.as_ptr();
                    let pszNew: *const WCHAR = CPP_COM_WSTR_96A296D224F2.as_ptr();
                    let mut nOld: usize = 0;
                    while *pszOld.add(nOld) != 0 {
                        nOld += 1;
                    }
                    let mut out = CStringW::new();
                    let nLen: usize = buf.GetLength() as usize;
                    let mut i: usize = 0;
                    while i < nLen {
                        let mut m: usize = 0;
                        while m < nOld && i + m < nLen && buf.GetAt((i + m) as i32) == *pszOld.add(m) {
                            m += 1;
                        }
                        if nOld != 0 && m == nOld {
                            out += pszNew;
                            i += nOld;
                        } else {
                            out.AppendChar(buf.GetAt(i as i32));
                            i += 1;
                        }
                    }
                    buf = out;
                }
                SetWindowTextW(hDlg, buf.GetString()); // "Downloading..."

                let hItem: HWND = GetDlgItem(hDlg, (IDC_DOWNLOAD_PROGRESS as i32));
                if !hItem.is_null() {
                    // initialize the default values for our nifty progress bar
                    // and subclass it so that it learns to print a status text
                    (*this_).m_ProgressBar.SubclassWindow(hItem);
                    (*this_).m_ProgressBar.__base0.SendMessageW__0(PBM_SETRANGE, 0, MAKELPARAM!(0, 100));
                    (*this_).m_ProgressBar.__base0.SendMessageW__0(PBM_SETPOS, 0, 0);
                    if (*this_).m_List.GetSize() > 0 {
                        (*this_).m_ProgressBar.SetProgress(0, (*(*this_).m_List.GetAt(0)).SizeInBytes);
                    }
                }

                if (*this_).m_ListView.Create(hDlg) == core::ptr::null_mut() {
                    return (FALSE) as INT_PTR;
                }
                (*this_).m_ListView.LoadList(&(*this_).m_List, 0);

                ShowWindow(hDlg, if (*this_).IsSilentDialog() != 0 { (SW_HIDE as i32) } else { (SW_SHOW as i32) });
                (*this_).StartWorkerThread();
                return (TRUE) as INT_PTR;
            }

            WM_COMMAND => {
                if LOWORD!(wParam) == (IDCANCEL as u16) {
                    (*this_).m_bCancelled = (TRUE as i32);
                    PostMessageW(hDlg, WM_CLOSE, 0, 0);
                }
                return (FALSE) as INT_PTR;
            }

            WM_CLOSE => {
                (*this_).m_bCancelled = (TRUE as i32);
                if !(*this_).m_ProgressBar.op_to_HWND().is_null() {
                    // UnsubclassWindow(TRUE): atl_rust's win module has no unsubclass, so undo
                    // subclass_window by hand (restore the saved wndproc, drop its two props).
                    extern "system" {
                        fn GetPropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
                        fn RemovePropW(hWnd: HWND, lpString: LPCWSTR) -> HANDLE;
                    }
                    let prop_this: [WCHAR; 12] = [0x41, 0x74, 0x6C, 0x52, 0x75, 0x73, 0x74, 0x54, 0x68, 0x69, 0x73, 0];
                    let prop_super: [WCHAR; 13] = [0x41, 0x74, 0x6C, 0x52, 0x75, 0x73, 0x74, 0x53, 0x75, 0x70, 0x65, 0x72, 0];
                    let hwndOld: HWND = (*this_).m_ProgressBar.__base0.m_hWnd;
                    let sup: HANDLE = GetPropW(hwndOld, prop_super.as_ptr());
                    if !sup.is_null() {
                        SetWindowLongPtrW(hwndOld, GWLP_WNDPROC, sup as usize as _);
                    }
                    RemovePropW(hwndOld, prop_super.as_ptr());
                    RemovePropW(hwndOld, prop_this.as_ptr());
                    (*this_).m_ProgressBar.__base0.m_hWnd = core::ptr::null_mut();
                }
                return if (*this_).m_bModal != 0 && (*this_).IsSilentDialog() == 0 {
                    (EndDialog(hDlg, 0)) as _
                } else {
                    (DestroyWindow(hDlg)) as _
                };
            }

            WM_DESTROY => {
                if g_hDownloadWnd == hDlg {
                    g_hDownloadWnd = core::ptr::null_mut();
                }
                g_Busy -= 1;
                if !hMainWnd.is_null() {
                    PostMessage(hMainWnd, WM_NOTIFY_OPERATIONCOMPLETED, 0, 0);
                }
                if (*this_).m_bModal != 0 && (*this_).IsSilentDialog() != 0 {
                    PostQuitMessage(0);
                }
                (*this_).Release();
                // break
            }

            WM_ISCANCELLED => {
                // SetDlgMsgResult(hDlg, uMsg, r): uMsg is not one of the messages windowsx.h returns r
                // directly for, so r goes to DWLP_MSGRESULT and TRUE is returned.
                SetWindowLongPtr(hDlg, DWLP_MSGRESULT, ((*this_).m_bCancelled as LRESULT) as LPARAM);
                return (TRUE) as INT_PTR;
            }

            WM_SETSTATUS => {
                (*this_).m_ListView.SetDownloadStatus(((*this_).m_Index - 1) as INT, wParam as DownloadStatus);
                // break
            }

            WM_GETINSTANCE => {
                SetWindowLongPtr(hDlg, DWLP_MSGRESULT, ((this_ as INT_PTR) as LRESULT) as LPARAM);
                return (TRUE) as INT_PTR;
            }

            WM_GETNEXT => {
                let mut pItem: *mut DownloadInfo = core::ptr::null_mut();
                if (*this_).m_bCancelled == 0 && ((*this_).m_Index as usize) < ((*this_).m_List.GetSize() as usize) {
                    let idx: usize = (*this_).m_Index as usize;
                    (*this_).m_Index += 1;
                    pItem = (*this_).m_List.GetAt(idx as i32);
                }
                SetWindowLongPtr(hDlg, DWLP_MSGRESULT, ((pItem as INT_PTR) as LRESULT) as LPARAM);
                return (TRUE) as INT_PTR;
            }

            _ => {}
        }
        return FALSE as INT_PTR;
    }
    pub unsafe fn UpdateProgress(&mut self, ulProgress: ULONG, ulProgressMax: ULONG) {
        let this_: *mut Self = self;
        (*this_).m_ProgressBar.SetProgress(ulProgress, ulProgressMax);
    }
    pub unsafe extern "system" fn ThreadFunc(ThreadParam: *mut core::ffi::c_void) -> UINT {
        let mut pThis: *mut CDownloadManager = ThreadParam as *mut CDownloadManager;
        let mut hDlg: HWND = (*pThis).m_hDlg;
        loop {
            let mut pItem: *mut DownloadInfo = SendMessageW(hDlg, WM_GETNEXT, 0, 0) as *mut DownloadInfo;
            if pItem.is_null() {
                break;
            }
            (*pThis).PerformDownloadAndInstall(&*pItem);
        }
        SendMessageW(hDlg, WM_CLOSE, 0, 0);
        return (*pThis).Release();
    }
    pub unsafe fn PerformDownloadAndInstall(&mut self, Info: &DownloadInfo) {
        let this_: *mut Self = self;
        let hDlg: HWND = (*this_).m_hDlg;
        let hStatus: HWND = GetDlgItem((*this_).m_hDlg, (IDC_DOWNLOAD_STATUS as i32));
        SetFriendlyUrl(hStatus, Info.szUrl.GetString());

        (*this_).m_ProgressBar.SetMarquee((FALSE) as BOOL);
        (*this_).m_ProgressBar.__base0.SendMessageW__0(PBM_SETPOS, 0, 0);
        (*this_).m_ProgressBar.SetProgress(0, Info.SizeInBytes);

        let mut str = CStringW::new();
        let mut Path = CPathW::new();
        let mut p: PCWSTR = core::ptr::null();

        let mut dwContentLen: ULONG = 0;
        let mut dwBytesWritten: ULONG = 0;
        let mut dwBytesRead: ULONG = 0;
        let mut dwStatus: ULONG = 0;
        let mut dwStatusLen: ULONG = 0;
        let mut dwCurrentBytesRead: ULONG = 0;
        let mut bTempfile: BOOL = (FALSE as i32);
        let mut bCancelled: BOOL = (FALSE as i32);

        let mut hOpen: HINTERNET = core::ptr::null_mut();
        let mut hFile: HINTERNET = core::ptr::null_mut();
        let mut hOut: HANDLE = INVALID_HANDLE_VALUE;

        let lpszAgent: LPCWSTR = CPP_COM_WSTR_1B4F33C91A81.as_ptr();
        let dwUrlConnectFlags: DWORD = INTERNET_FLAG_DONT_CACHE | INTERNET_FLAG_PRAGMA_NOCACHE | INTERNET_FLAG_KEEP_CONNECTION;
        let mut urlComponents: URL_COMPONENTSW = core::mem::zeroed();
        let mut urlLength: usize = 0;
        let mut lpBuffer: [u8; 4096 as usize] = [0; 4096 as usize];

        // Change caption to show the currently downloaded app
        match Info.DLType {
            DLTYPE_APPLICATION => {
                atl_format!(str, (*this_).m_szCaptionFmt.as_ptr(), Info.szName.GetString());
            }
            DLTYPE_DBUPDATE => {
                str.LoadStringW(IDS_DL_DIALOG_DB_DOWNLOAD_DISP);
            }
            DLTYPE_DBUPDATE_UNOFFICIAL => {
                str.LoadStringW(IDS_DL_DIALOG_DB_UNOFFICIAL_DOWNLOAD_DISP);
            }
            _ => {}
        }
        SetWindowTextW(hDlg, str.GetString());

        'end: {
            'run: {
                // is this URL an update package for RAPPS? if so store it in a different place
                if Info.DLType != DLTYPE_APPLICATION {
                    if GetStorageDirectory(core::ptr::addr_of_mut!(Path.m_strPath)) == 0 {
                        ShowLastError(hMainWnd, (FALSE) as BOOL, GetLastError());
                        break 'end;
                    }
                } else {
                    Path = CPathW::from_ptr(SettingsInfo.szDownloadDir.as_ptr());
                }

                // build the path for the download
                p = wcsrchr(Info.szUrl.GetString(), 0x002Fu16);

                // do we have a final slash separator?
                if p.is_null() {
                    MessageBox_LoadString(hMainWnd, (IDS_UNABLE_PATH) as INT);
                    break 'end;
                }

                // is the path valid? can we access it?
                if GetFileAttributesW(Path.GetString()) == INVALID_FILE_ATTRIBUTES {
                    if CreateDirectoryW(Path.GetString(), core::ptr::null_mut()) == 0 {
                        ShowLastError(hMainWnd, (FALSE) as BOOL, GetLastError());
                        break 'end;
                    }
                }

                match Info.DLType {
                    DLTYPE_DBUPDATE | DLTYPE_DBUPDATE_UNOFFICIAL => {
                        Path += APPLICATION_DATABASE_NAME.as_ptr();
                    }
                    DLTYPE_APPLICATION => {
                        let mut name = CStringW::from_ptr(Info.szFileName.GetString());
                        if name.IsEmpty() {
                            name = CStringW::from_ptr(p.offset(1 as isize)); // use the filename retrieved from URL
                        }
                        UrlUnescapeAndMakeFileNameValid(&mut name);
                        Path += name.GetString();
                    }
                    _ => {}
                }

                if (Info.DLType == DLTYPE_APPLICATION)
                    && Info.szSHA1.GetAt(0 as i32) != 0
                    && GetFileAttributesW(Path.GetString()) != INVALID_FILE_ATTRIBUTES
                {
                    // only open it in case of total correctness
                    if VerifyInteg(Info.szSHA1.GetString(), Path.GetString()) != 0 {
                        break 'run;
                    }
                }

                // Download it
                SendMessageW(hDlg, WM_SETSTATUS, (DLSTATUS_DOWNLOADING) as WPARAM, 0);
                /* FIXME: this should just be using the system-wide proxy settings */
                match SettingsInfo.Proxy {
                    1 => {
                        // direct (no proxy)
                        hOpen = InternetOpenW(lpszAgent, INTERNET_OPEN_TYPE_DIRECT, core::ptr::null_mut(), core::ptr::null_mut(), 0);
                    }
                    2 => {
                        // use proxy
                        hOpen = InternetOpenW(
                            lpszAgent,
                            INTERNET_OPEN_TYPE_PROXY,
                            (SettingsInfo.szProxyServer).as_ptr(),
                            (SettingsInfo.szNoProxyFor).as_ptr(),
                            0,
                        );
                    }
                    0 | _ => {
                        // preconfig / default
                        hOpen = InternetOpenW(lpszAgent, INTERNET_OPEN_TYPE_PRECONFIG, core::ptr::null_mut(), core::ptr::null_mut(), 0);
                    }
                }

                if hOpen.is_null() {
                    ShowLastError(hMainWnd, (TRUE) as BOOL, GetLastError());
                    break 'end;
                }

                bTempfile = (TRUE as i32);
                dwContentLen = 0;
                dwStatusLen = core::mem::size_of::<ULONG>() as ULONG;
                ZeroMemory!(core::ptr::addr_of_mut!(urlComponents) as *mut core::ffi::c_void, core::mem::size_of::<URL_COMPONENTSW>());
                urlComponents.dwStructSize = core::mem::size_of::<URL_COMPONENTSW>() as DWORD;

                urlLength = Info.szUrl.GetLength() as usize;
                urlComponents.dwSchemeLength = (urlLength + 1 as usize) as DWORD;
                urlComponents.lpszScheme = malloc(urlComponents.dwSchemeLength as usize * core::mem::size_of::<WCHAR>()) as LPWSTR;

                if InternetCrackUrlW(
                    Info.szUrl.GetString(),
                    (urlLength + 1) as DWORD,
                    ICU_DECODE | ICU_ESCAPE,
                    core::ptr::addr_of_mut!(urlComponents),
                ) == 0
                {
                    ShowLastError(hMainWnd, (TRUE) as BOOL, GetLastError());
                    break 'end;
                }

                if urlComponents.nScheme == INTERNET_SCHEME_HTTP || urlComponents.nScheme == INTERNET_SCHEME_HTTPS {
                    hFile = InternetOpenUrlW(hOpen, Info.szUrl.GetString(), core::ptr::null_mut(), 0, dwUrlConnectFlags, 0);
                    if hFile.is_null() {
                        if ShowLastError(hMainWnd, (TRUE) as BOOL, GetLastError()) == 0 {
                            /* Workaround for CORE-17377 */
                            MessageBox_LoadString(hMainWnd, (IDS_UNABLE_TO_DOWNLOAD2) as INT);
                        }
                        break 'end;
                    }

                    // query connection
                    if HttpQueryInfoW(
                        hFile,
                        HTTP_QUERY_STATUS_CODE | HTTP_QUERY_FLAG_NUMBER,
                        (core::ptr::addr_of_mut!(dwStatus)).cast(),
                        core::ptr::addr_of_mut!(dwStatusLen),
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        ShowLastError(hMainWnd, (TRUE) as BOOL, GetLastError());
                        break 'end;
                    }

                    if dwStatus != HTTP_STATUS_OK {
                        MessageBox_LoadString(hMainWnd, (IDS_UNABLE_TO_DOWNLOAD) as INT);
                        break 'end;
                    }

                    // query content length
                    HttpQueryInfoW(
                        hFile,
                        HTTP_QUERY_CONTENT_LENGTH | HTTP_QUERY_FLAG_NUMBER,
                        (core::ptr::addr_of_mut!(dwContentLen)).cast(),
                        core::ptr::addr_of_mut!(dwStatusLen),
                        core::ptr::null_mut(),
                    );
                } else if urlComponents.nScheme == INTERNET_SCHEME_FTP {
                    // force passive mode on FTP
                    hFile = InternetOpenUrlW(
                        hOpen,
                        Info.szUrl.GetString(),
                        core::ptr::null_mut(),
                        0,
                        dwUrlConnectFlags | INTERNET_FLAG_PASSIVE,
                        0,
                    );
                    if hFile.is_null() {
                        if ShowLastError(hMainWnd, (TRUE) as BOOL, GetLastError()) == 0 {
                            /* Workaround for CORE-17377 */
                            MessageBox_LoadString(hMainWnd, (IDS_UNABLE_TO_DOWNLOAD2) as INT);
                        }
                        break 'end;
                    }

                    dwContentLen = FtpGetFileSize(hFile, core::ptr::addr_of_mut!(dwStatus));
                } else if urlComponents.nScheme == INTERNET_SCHEME_FILE {
                    // Add support for the file scheme so testing locally is simpler
                    let mut LocalFilePath: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
                    let mut cchPath: DWORD = _countof!(LocalFilePath) as DWORD;
                    // Ideally we would use PathCreateFromUrlAlloc here, but that is not exported (yet)
                    let hr: HRESULT = PathCreateFromUrlW(Info.szUrl.GetString(), LocalFilePath.as_mut_ptr(), core::ptr::addr_of_mut!(cchPath), 0);
                    if hr >= 0 {
                        if CopyFileW(LocalFilePath.as_mut_ptr(), Path.GetString(), (FALSE) as BOOL) != 0 {
                            // Remove the readonly flag in case we are copying from a read-only medium
                            let attr: DWORD = GetFileAttributesW(Path.GetString());
                            if attr != INVALID_FILE_ATTRIBUTES && (attr & FILE_ATTRIBUTE_READONLY) != 0 {
                                SetFileAttributesW(Path.GetString(), attr & !FILE_ATTRIBUTE_READONLY);
                            }

                            break 'run;
                        } else {
                            ShowLastError(hMainWnd, (FALSE) as BOOL, GetLastError());
                            break 'end;
                        }
                    } else {
                        ShowLastError(hMainWnd, (FALSE) as BOOL, (hr) as DWORD);
                        break 'end;
                    }
                }

                if dwContentLen == 0 {
                    // Someone was nice enough to add this, let's use it
                    if Info.SizeInBytes != 0 {
                        dwContentLen = Info.SizeInBytes;
                    } else {
                        // content-length is not known, enable marquee mode
                        (*this_).m_ProgressBar.SetMarquee((TRUE) as BOOL);
                    }
                }

                free((urlComponents.lpszScheme).cast());

                hOut = CreateFileW(Path.GetString(), GENERIC_WRITE, FILE_SHARE_READ | FILE_SHARE_WRITE, core::ptr::null_mut(), CREATE_ALWAYS, 0, core::ptr::null_mut());
                if hOut == INVALID_HANDLE_VALUE {
                    ShowLastError(hDlg, (FALSE) as BOOL, GetLastError());
                    break 'end;
                }

                dwCurrentBytesRead = 0;
                loop {
                    bCancelled = (*this_).IsCancelled();
                    if bCancelled != 0 {
                        break;
                    }

                    if InternetReadFile(hFile, (lpBuffer.as_mut_ptr()).cast(), _countof!(lpBuffer) as DWORD, core::ptr::addr_of_mut!(dwBytesRead)) == 0 {
                        ShowLastError(hDlg, (TRUE) as BOOL, GetLastError());
                        break 'end;
                    }

                    if WriteFile(
                        hOut,
                        lpBuffer.as_ptr() as *const core::ffi::c_void,
                        dwBytesRead,
                        core::ptr::addr_of_mut!(dwBytesWritten),
                        core::ptr::null_mut(),
                    ) == 0
                    {
                        ShowLastError(hDlg, (FALSE) as BOOL, GetLastError());
                        break 'end;
                    }

                    dwCurrentBytesRead += dwBytesRead;
                    (*this_).UpdateProgress(dwCurrentBytesRead, dwContentLen);

                    if dwBytesRead == 0 {
                        break;
                    }
                }

                CloseHandle(hOut);
                hOut = INVALID_HANDLE_VALUE;

                if bCancelled != 0 {
                    DPRINT1!("Operation cancelled\n");
                    break 'end;
                }

                if dwContentLen == 0 {
                    // set progress bar to 100%
                    (*this_).m_ProgressBar.SetMarquee((FALSE) as BOOL);

                    dwContentLen = dwCurrentBytesRead;
                    (*this_).UpdateProgress(dwCurrentBytesRead, dwContentLen);
                }

                /* if this thing isn't a RAPPS update and it has a SHA-1 checksum
                verify its integrity by using the native advapi32.A_SHA1 functions */
                if (Info.DLType == DLTYPE_APPLICATION) && Info.szSHA1.GetAt(0 as i32) != 0 as WCHAR {
                    let mut szMsgText = CStringW::new();

                    // change a few strings in the download dialog to reflect the verification process
                    if szMsgText.LoadStringW(IDS_INTEG_CHECK_TITLE) == 0 {
                        DPRINT1!("Unable to load string\n");
                        break 'end;
                    }

                    SetWindowTextW(hDlg, szMsgText.GetString());
                    SetWindowTextW(hStatus, Path.GetString());

                    // this may take a while, depending on the file size
                    if VerifyInteg(Info.szSHA1.GetString(), Path.GetString()) == 0 {
                        if szMsgText.LoadStringW(IDS_INTEG_CHECK_FAIL) == 0 {
                            DPRINT1!("Unable to load string\n");
                            break 'end;
                        }

                        MessageBoxW(hDlg, szMsgText.GetString(), core::ptr::null_mut(), MB_OK | MB_ICONERROR);
                        break 'end;
                    }
                }
            }

            // run:
            SendMessageW(hDlg, WM_SETSTATUS, (DLSTATUS_WAITING_INSTALL) as WPARAM, 0);

            // run it
            if Info.DLType == DLTYPE_APPLICATION {
                let bSilentInstall: BOOL = (Info.Flags & DAF_SILENT) as BOOL;
                let mut app = CStringW::new();
                let mut params = CStringW::new();
                let mut tempdir = CStringW::new();
                let mut shExInfo: SHELLEXECUTEINFOW = SHELLEXECUTEINFOW {
                    cbSize: core::mem::size_of::<SHELLEXECUTEINFOW>() as DWORD,
                    fMask: SEE_MASK_NOCLOSEPROCESS,
                    hwnd: hDlg,
                    ..core::mem::zeroed()
                };
                shExInfo.lpVerb = CPP_COM_WSTR_82DB442E7A60.as_ptr();
                shExInfo.lpFile = Path.GetString();
                shExInfo.lpParameters = CPP_COM_WSTR_96A296D224F2.as_ptr();
                shExInfo.nShow = SW_SHOW as i32;

                if Info.IType == INSTALLER_GENERATE {
                    params = CStringW::from_ptr(CPP_COM_WSTR_C2B264989196.as_ptr());
                    params += (if bSilentInstall != 0 {
                        CPP_COM_WSTR_5853A555D865.as_ptr()
                    } else {
                        CPP_COM_WSTR_96A296D224F2.as_ptr()
                    });
                    params += CPP_COM_WSTR_632A8A3E5C84.as_ptr();
                    params += Info.szPackageName.GetString();
                    params += CPP_COM_WSTR_85D5870D46C2.as_ptr();
                    params += shExInfo.lpFile;
                    params += CPP_COM_WSTR_1B9334FEECE6.as_ptr();
                    shExInfo.lpParameters = params.GetString();
                    shExInfo.lpFile = app.GetBuffer((MAX_PATH as i32));
                    GetModuleFileNameW(core::ptr::null_mut(), shExInfo.lpFile as *mut WCHAR, MAX_PATH);
                    app.ReleaseBuffer();
                } else if Info.IType == INSTALLER_EXEINZIP {
                    let hr: HRESULT = ExtractArchiveForExecution(Path.GetString(), &Info.szPackageName, &mut tempdir, &mut app);
                    if hr < 0 {
                        ShowLastError(hDlg, (FALSE) as BOOL, (hr) as DWORD);
                        break 'end;
                    }
                    shExInfo.lpFile = app.GetString();
                }

                if bSilentInstall != 0 {
                    if !Info.szSilentInstallArgs.IsEmpty() {
                        // The package wants to force specific parameters
                        shExInfo.lpParameters = Info.szSilentInstallArgs.GetString();
                    } else {
                        let mut extrainfo: UINT = 0;
                        let mut it: InstallerType = Info.IExecType;

                        if it == INSTALLER_UNKNOWN || it == INSTALLER_EXEINZIP {
                            it = GuessInstallerType(shExInfo.lpFile, &mut extrainfo);
                        }

                        if GetSilentInstallParameters(it, extrainfo, shExInfo.lpFile, &mut params) != 0 {
                            shExInfo.lpParameters = params.GetString();
                            if it == INSTALLER_MSI {
                                shExInfo.lpFile = CPP_COM_WSTR_F1C8A68A6FAF.as_ptr(); // params contains the .msi path
                            }
                        }
                    }
                }

                /* FIXME: Do we want to log installer status? */
                WriteLogMessage((EVENTLOG_SUCCESS) as WORD, MSG_SUCCESS_INSTALL, Info.szName.GetString());

                if ShellExecuteExW(core::ptr::addr_of_mut!(shExInfo)) != 0 {
                    // reflect installation progress in the titlebar
                    // TODO: make a separate string with a placeholder to include app name?
                    let mut szMsgText: CStringW = LoadStatusString(DLSTATUS_INSTALLING);
                    SetWindowTextW(hDlg, szMsgText.GetString());

                    SendMessageW(hDlg, WM_SETSTATUS, (DLSTATUS_INSTALLING) as WPARAM, 0);

                    // TODO: issue an install operation separately so that the apps could be downloaded in the background
                    if !shExInfo.hProcess.is_null() {
                        WaitForSingleObject(shExInfo.hProcess, INFINITE);
                        CloseHandle(shExInfo.hProcess);
                        SendMessageW(
                            hMainWnd,
                            WM_NOTIFY_INSTALLERFINISHED,
                            0,
                            (Info.szPackageName.GetString() as usize) as LPARAM,
                        );
                    }
                } else {
                    ShowLastError(hMainWnd, (FALSE) as BOOL, GetLastError());
                }

                if !tempdir.IsEmpty() {
                    DeleteDirectoryTree(tempdir.GetString(), hDlg);
                }
            }
        }

        // end:
        if hOut != INVALID_HANDLE_VALUE {
            CloseHandle(hOut);
        }

        if !hFile.is_null() {
            InternetCloseHandle(hFile);
        }
        InternetCloseHandle(hOpen);

        if bTempfile != 0 {
            if bCancelled != 0 || (SettingsInfo.bDelInstaller != 0 && Info.DLType == DLTYPE_APPLICATION) {
                // Don't delete .zip/.cab files so the user can extract from them
                if bCancelled != 0
                    || Info.IType == INSTALLER_GENERATE
                    || Info.IType == INSTALLER_EXEINZIP
                    || OpensWithExplorer(Path.GetString()) == 0
                    || (HIBYTE!(ClassifyFile(Path.GetString()))) as i32 != PERCEIVED_TYPE_COMPRESSED
                {
                    DeleteFileW(Path.GetString());
                }
            }
        }

        SendMessageW(hDlg, WM_SETSTATUS, (DLSTATUS_FINISHED) as WPARAM, 0);
    }
}
