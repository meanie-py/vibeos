use crate::prelude::*;
use crate::zipfldr::CConfirmReplace_cpp_h::_CZipAskReplace;
use crate::zipfldr::zipfldr_cpp_h::g_FFunc;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0022049C7838: [WCHAR; 5] = [0x000D, 0x000A, 0x000D, 0x000A, 0x0000];
pub static CPP_COM_WSTR_31AF3113D116: [WCHAR; 8] = [0x0065, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0000];

use core::ffi::c_void;
































#[no_mangle]
pub unsafe extern "C" fn _CZipExtract_runWizard(Filename: PCWSTR) {
    let mut extractor: CZipExtract = CZipExtract__make_0(Filename);
    extractor.runWizard();
}

impl CZipExtract {
    pub unsafe fn __ctor_0(&mut self, Filename: PCWSTR) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_DirectoryChanged = false;
        (*this_).uf = core::ptr::null_mut();
        {
            (*this_).m_Filename = CStringW::from_ptr(Filename);
            (*this_).m_Directory = CStringW::from_ptr((*this_).m_Filename.GetString());
            let mut Dir: PWSTR = (*this_).m_Directory.GetBuffer(0);
            PathRemoveExtensionW(Dir);
            (*this_).m_Directory.ReleaseBuffer();
        }
    }
    pub unsafe fn __dtor__body(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).uf.is_null() {
            DPRINT1!("WARNING: uf not closed!\n");
            (*this_).Close();
        }
    }
    pub unsafe fn Close(&mut self) {
        let this_: *mut Self = self;
        if !(*this_).uf.is_null() {
            unzClose((*this_).uf);
        }
        (*this_).uf = core::ptr::null_mut();
    }
    pub unsafe fn QueryInterface(&mut self, riid: &IID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
        let this_: *mut Self = self;
        if IsEqualGUID!(&(*riid), &(IID_IUnknown)) {
            *ppvObject = this_ as *mut core::ffi::c_void;
            (*this_).AddRef();
            return S_OK;
        }
        return E_NOINTERFACE;
    }
    pub unsafe fn AddRef(&mut self) -> ULONG {
        let this_: *mut Self = self;
        2
    }
    pub unsafe fn Release(&mut self) -> ULONG {
        let this_: *mut Self = self;
        1
    }
    pub unsafe fn getZip(&mut self) -> unzFile {
        let this_: *mut Self = self;
        (*this_).uf
    }
    pub unsafe extern "system" fn s_PropSheetCallbackProc(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> INT {
        if uMsg == PSCB_INITIALIZED {
            let hIcon: HICON = LoadIconW(_AtlBaseModule.GetResourceInstance(), MAKEINTRESOURCEW!(IDI_ZIPFLDR));
            let mut dlg: CWindow = CWindow__make_0(hwndDlg);
            dlg.SetIcon(hIcon, (TRUE as i32));
        }

        return 0;
    }
    pub unsafe fn runWizard(&mut self) {
        let this_: *mut Self = self;
        let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
        psh.dwFlags = 0;
        psh.dwFlags = PSH_WIZARD97 | PSH_HEADER | PSH_USEICONID | PSH_USECALLBACK;
        psh.hInstance = _AtlBaseModule.GetResourceInstance();

        let mut extractPage: CExtractSettingsPage = CExtractSettingsPage__make_0(this_, core::ptr::addr_of_mut!((*this_).m_Password));
        let mut completePage: CCompleteSettingsPage = CCompleteSettingsPage__make_0(this_);
        let mut hpsp: [HPROPSHEETPAGE; 2] = [
            extractPage.Create(),
            completePage.Create(),
        ];

        psh.__bindgen_anon_3.phpage = hpsp.as_mut_ptr();
        psh.nPages = _countof!(hpsp) as u32;
        psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCE!(IDI_ZIPFLDR);
        psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCE!(IDB_WATERMARK);
        psh.__bindgen_anon_5.pszbmHeader = MAKEINTRESOURCE!(IDB_HEADER);
        psh.pfnCallback = Some(CZipExtract::s_PropSheetCallbackProc);

        PropertySheetW(core::ptr::addr_of_mut!(psh));
    }
    pub unsafe fn ExtractSingle(&mut self, hDlg: HWND, FullPath: PCWSTR, is_dir: bool, Info: *mut unz_file_info64, Name: CStringW, mut Password: CStringA, bOverwriteAll: *mut bool, bCancel: *const bool, ErrorCode: *mut i32) -> eZipExtractError {
        let this_: *mut Self = self;
        let mut err: i32 = 0;
        let mut Buffer: [u8; 2048] = [0u8; 2048];
        let mut dwFlags: DWORD = SHPPFW_DIRCREATE | (if is_dir { SHPPFW_NONE } else { SHPPFW_IGNOREFILENAME });
        let mut hr: HRESULT = SHPathPrepareForWriteW(hDlg, core::ptr::null_mut(), FullPath, dwFlags);
        if FAILED_UNEXPECTEDLY!(hr) {
            *ErrorCode = hr;
            return eZipExtractError_eDirectoryError;
        }
        if is_dir {
            return eZipExtractError_eNoError;
        }

        if ((*Info).flag & MINIZIP_PASSWORD_FLAG) != 0 {
            let mut Response = eZipPasswordResponse_eAccept;
            loop {
                /* If there is a password set, try it */
                if !Password.IsEmpty() {
                    err = unzOpenCurrentFilePassword((*this_).uf, (Password.GetString()).cast());
                    if err == (UNZ_OK as i32) {
                        /* Try to read some bytes, because unzOpenCurrentFilePassword does not return failure */
                        let mut Buf: [i8; 10] = [0i8; 10];
                        err = unzReadCurrentFile((*this_).uf, (Buf.as_mut_ptr()).cast(), core::mem::size_of_val(&Buf) as u32);
                        unzCloseCurrentFile((*this_).uf);
                        if err >= (UNZ_OK as i32) {
                            /* 're'-open the file so that we can begin to extract */
                            err = unzOpenCurrentFilePassword((*this_).uf, (Password.GetString()).cast());
                            break;
                        }
                    }
                }
                Response = _CZipAskPassword(hDlg, Name.GetString(), &mut Password);
                if !(Response == eZipPasswordResponse_eAccept) {
                    break;
                }
            }

            if Response == eZipPasswordResponse_eSkip {
                return eZipExtractError_eNoError;
            }
            else if Response == eZipPasswordResponse_eAbort {
                return eZipExtractError_eExtractAbort;
            }
        }
        else {
            err = unzOpenCurrentFile((*this_).uf);
        }

        if err != (UNZ_OK as i32) {
            DPRINT1!("ERROR, unzOpenCurrentFilePassword: 0x%x\n", err);
            *ErrorCode = err;
            return eZipExtractError_eOpenError;
        }

        let mut hFile: HANDLE = CreateFileW(FullPath, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_NEW, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
        if hFile == INVALID_HANDLE_VALUE {
            let mut dwErr: DWORD = GetLastError();
            if dwErr == ERROR_FILE_EXISTS {
                let mut bOverwrite: bool = *bOverwriteAll;
                if !*bOverwriteAll {
                    let mut Result = _CZipAskReplace(hDlg, FullPath);
                    match Result {
                        eZipConfirmResponse_eYesToAll => {
                            *bOverwriteAll = true;
                            /* fall through */
                            bOverwrite = true;
                        }
                        eZipConfirmResponse_eYes => {
                            bOverwrite = true;
                        }
                        eZipConfirmResponse_eNo => {
                        }
                        eZipConfirmResponse_eCancel => {
                            unzCloseCurrentFile((*this_).uf);
                            return eZipExtractError_eExtractAbort;
                        }
                        _ => {} // the C switch has no default
                    }
                }

                if bOverwrite {
                    hFile = CreateFileW(FullPath, GENERIC_WRITE, 0, core::ptr::null_mut(), CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, core::ptr::null_mut());
                    if hFile == INVALID_HANDLE_VALUE {
                        dwErr = GetLastError();
                    }
                }
                else {
                    unzCloseCurrentFile((*this_).uf);
                    return eZipExtractError_eNoError;
                }
            }
            if hFile == INVALID_HANDLE_VALUE {
                unzCloseCurrentFile((*this_).uf);
                DPRINT1!("ERROR, CreateFile: 0x%x (%s)\n", dwErr, if *bOverwriteAll { "Y" } else { "N" });
                *ErrorCode = dwErr as i32;
                return eZipExtractError_eFileError;
            }
        }

        loop {
            if *bCancel {
                CloseHandle(hFile);
                let mut deleteResult: BOOL = DeleteFileW(FullPath);
                if deleteResult == 0 {
                    DPRINT1!("ERROR, DeleteFile: 0x%x\n", GetLastError());
                }
                return eZipExtractError_eExtractAbort;
            }

            err = unzReadCurrentFile((*this_).uf, (Buffer.as_mut_ptr()).cast(), core::mem::size_of_val(&Buffer) as u32);

            if err < 0 {
                DPRINT1!("ERROR, unzReadCurrentFile: 0x%x\n", err);
                break;
            }
            else if err > 0 {
                let mut dwWritten: DWORD = 0;
                if WriteFile(hFile, Buffer.as_ptr() as *const core::ffi::c_void, err as u32, core::ptr::addr_of_mut!(dwWritten), core::ptr::null_mut()) == 0 {
                    DPRINT1!("ERROR, WriteFile: 0x%x\n", GetLastError());
                    break;
                }
                if dwWritten != (err as DWORD) {
                    DPRINT1!("ERROR, WriteFile: dwWritten:%d err:%d\n", dwWritten, err);
                    break;
                }
            }

            if !(err > 0) {
                break;
            }
        }

        /* Update Filetime */
        let mut LocalFileTime: FILETIME = core::mem::zeroed();
        DosDateTimeToFileTime(((*Info).dosDate >> 16) as WORD, (*Info).dosDate as WORD, core::ptr::addr_of_mut!(LocalFileTime));
        let mut FileTime: FILETIME = core::mem::zeroed();
        LocalFileTimeToFileTime(core::ptr::addr_of!(LocalFileTime), core::ptr::addr_of_mut!(FileTime));
        SetFileTime(hFile, core::ptr::addr_of!(FileTime), core::ptr::addr_of!(FileTime), core::ptr::addr_of!(FileTime));

        /* Done */
        CloseHandle(hFile);

        if err != 0 {
            unzCloseCurrentFile((*this_).uf);
            DPRINT1!("ERROR, unzReadCurrentFile2: 0x%x\n", err);
            *ErrorCode = err;
            return eZipExtractError_eUnpackError;
        }
        else {
            err = unzCloseCurrentFile((*this_).uf);
            if err != (UNZ_OK as i32) {
                DPRINT1!("ERROR(non-fatal), unzCloseCurrentFile: 0x%x\n", err);
            }
        }
        return eZipExtractError_eNoError;
    }
    pub unsafe fn Extract(&mut self, hDlg: HWND, hProgress: HWND, bCancel: *const bool) -> bool {
        let this_: *mut Self = self;
            let mut gi: unz_global_info64 = core::mem::zeroed();
            (*this_).uf = unzOpen2_64(((*this_).m_Filename.GetString()).cast(), core::ptr::addr_of_mut!(g_FFunc));
            let mut err: i32 = unzGetGlobalInfo64((*this_).uf, core::ptr::addr_of_mut!(gi));
            if err != (UNZ_OK as i32)
            {
                DPRINT1!("ERROR, unzGetGlobalInfo64: 0x%x\n", err);
                (*this_).Close();
                return false;
            }

            let mut zipEnum: CZipEnumerator = CZipEnumerator__make_0();
            if zipEnum.Initialize(core::ptr::addr_of_mut!((*this_).__base0)) == 0
            {
                DPRINT1!("ERROR, zipEnum.Initialize\n");
                (*this_).Close();
                return false;
            }

            let mut Progress: CWindow = CWindow__make_0(hProgress);
            Progress.SendMessageW__0(PBM_SETRANGE32, 0, gi.number_entry as LPARAM);
            Progress.SendMessageW__0(PBM_SETPOS, 0, 0 as LPARAM);

            let mut BaseDirectory: CStringW = CStringW::from_ptr((*this_).m_Directory.GetString());
            let mut Name: CStringW = CStringW::new();
            let mut Password: CStringA = CStringA::from_ptr(((*this_).m_Password.GetString()).cast());
            let mut Info: unz_file_info64 = core::mem::zeroed();
            let mut CurrentFile: i32 = 0;
            let mut bOverwriteAll: bool = false;
            'outer: while zipEnum.Next(&mut Name, &mut Info) != 0
            {
                if *bCancel
                {
                    (*this_).Close();
                    return false;
                }

                let is_dir: bool = Name.GetLength() > 0 && Name.GetAt(Name.GetLength() - 1) == '/' as u16;

                // Build a combined path
                let mut FullPath: CPathW = CPathW::from_ptr(BaseDirectory.GetString());
                FullPath += &Name; // CPathT::operator+= is PathAppendW: one separator, `..` canonicalized

                // We use SHPathPrepareForWrite for this path.
                // SHPathPrepareForWrite will prepare the necessary directories.
                // Windows and ReactOS SHPathPrepareForWrite do not support '/'.
                FullPath.m_strPath.Replace(0x002Fu16, 0x005Cu16);

            'retry: loop {
                let Result: eZipExtractError = (*this_).ExtractSingle(
                    hDlg,
                    FullPath.m_strPath.GetString(),
                    is_dir,
                    core::ptr::addr_of_mut!(Info),
                    CStringW::from_ptr(Name.GetString()),
                    CStringA::from_ptr((Password.GetString()).cast()),
                    core::ptr::addr_of_mut!(bOverwriteAll),
                    bCancel,
                    core::ptr::addr_of_mut!(err),
                );
                if Result != eDirectoryError
                    { CurrentFile += 1; }
                'sw: {
                    match Result {
                        eNoError => {
                            break 'sw;
                        }

                        eExtractAbort | eUnpackError => {
                            (*this_).Close();
                            return false;
                        }

                        eDirectoryError => {
                            let mut StrippedPath = [0u16; (MAX_PATH) as _];

                            StrCpyNW(StrippedPath.as_mut_ptr(), FullPath.m_strPath.GetString(), (_countof!(StrippedPath)) as i32);
                            if !is_dir
                                { PathRemoveFileSpecW(StrippedPath.as_mut_ptr()); }
                            PathStripPathW(StrippedPath.as_mut_ptr());
                            if (*this_).ShowExtractError(hDlg, StrippedPath.as_ptr(), err, eDirectoryError) == (IDRETRY as i32)
                                { continue 'retry; }
                            (*this_).Close();
                            return false;
                        }

                        eFileError => {
                            let Result: i32 = (*this_).ShowExtractError(hDlg, FullPath.m_strPath.GetString(), err, eFileError);
                            match (Result) as u32 {
                                IDABORT => {
                                    (*this_).Close();
                                    return false;
                                }
                                IDRETRY => {
                                    CurrentFile -= 1;
                                    continue 'retry;
                                }
                                IDIGNORE => {
                                }
                                _ => {}
                            }
                            break 'sw;
                        }

                        eOpenError => {
                            if err == UNZ_BADZIPFILE &&
                                Info.compression_method != 0 &&
                                Info.compression_method != Z_DEFLATED &&
                                Info.compression_method != Z_BZIP2ED
                            {
                                if (*this_).ShowExtractError(hDlg, FullPath.m_strPath.GetString(), Info.compression_method as i32, eOpenError) == (IDYES as i32)
                                    { break 'sw; }
                            }
                            (*this_).Close();
                            return false;
                        }

                        _ => {}
                    }
                }
                if Result == eNoError && is_dir
                    { continue 'outer; }
                Progress.SendMessageW__0(PBM_SETPOS, CurrentFile as WPARAM, 0 as LPARAM);
                break 'retry;
            }
            }

            (*this_).Close();
            return true;
        }
    pub unsafe fn ShowExtractError(&mut self, hDlg: HWND, path: PCWSTR, Error: i32, ErrorType: eZipExtractError) -> i32 {
        let this_: *mut Self = self;
        let mut strTitle = CStringW::from_resource(IDS_ERRORTITLE);
        let mut strErr = CStringW::new();
        let mut strText = CStringW::new();
        let mut Win32ErrorString: PWSTR = core::ptr::null_mut();

        if ErrorType == eFileError || ErrorType == eOpenError {
            strText.LoadStringW(IDS_CANTEXTRACTFILE);
        } else {
            strText.LoadStringFrom(GetModuleHandleA(b"shell32.dll\0".as_ptr() as *const i8) as _, 128); // IDS_CREATEFOLDER_DENIED
        }

        strText.FormatMessageStr(strText.GetString(), path); // %1, not printf

        if ErrorType == eFileError || HRESULT_FACILITY!(Error) == (FACILITY_WIN32 as i32) {
            if FormatMessageW(
                FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM,
                core::ptr::null_mut(),
                if ErrorType == eFileError { Error as u32 } else { HRESULT_CODE!(Error) as u32 },
                0,
                core::ptr::addr_of_mut!(Win32ErrorString) as PWSTR,
                0,
                core::ptr::null_mut(),
            ) != 0
            {
                strErr = CStringW::from_ptr(Win32ErrorString); // SetString(psz): the whole string
                LocalFree(Win32ErrorString as *mut core::ffi::c_void);
            }
        }

        if ErrorType == eOpenError {
            atl_format_id!(strErr, IDS_DECOMPRESSERROR, Error);
        } else if strErr.GetLength() == 0 {
            atl_format_id!(strErr, IDS_UNKNOWNERROR, Error);
        }

        let mut Concat = CStringW::from_ptr((CPP_COM_WSTR_0022049C7838).as_ptr());
        Concat += strErr.GetString();
        strText.Append(Concat.GetString());

        let mut mbFlags: u32 = MB_ICONWARNING;
        if ErrorType == eDirectoryError {
            mbFlags |= MB_RETRYCANCEL;
        } else if ErrorType == eFileError {
            mbFlags |= MB_ABORTRETRYIGNORE;
        } else if ErrorType == eOpenError {
            mbFlags |= MB_YESNO;
        }

        return MessageBoxW(hDlg, strText.GetString(), strTitle.GetString(), mbFlags) as i32;
    }
}

impl CExtractSettingsPage {
    pub unsafe fn __ctor_0(&mut self, extract: *mut CZipExtract, password: *mut CStringA) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_psp = CExtractSettingsPage::__psp_init_titled(this_, MAKEINTRESOURCE!(IDS_WIZ_TITLE));
        (*this_).m_hExtractionThread = core::ptr::null_mut();
        (*this_).m_bExtractionThreadCancel = false;
        (*this_).m_pExtract = extract;
        (*this_).m_pPassword = password;
        {
            (*this_).m_psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_WIZ_DEST_TITLE);
            (*this_).m_psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_WIZ_DEST_SUBTITLE);
            (*this_).m_psp.dwFlags |= PSP_USETITLE | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
        }
    }
    pub unsafe fn OnSetActive(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).__base0.SetDlgItemTextW((IDC_DIRECTORY as i32), (*(*this_).m_pExtract).m_Directory.GetString());
        (*(*this_).m_pExtract).m_DirectoryChanged = false;
        (*this_).__base0.GetParent().CenterWindow(GetDesktopWindow());
        (*this_).SetWizardButtons(PSWIZB_NEXT);
        return 0;
    }
    pub unsafe fn OnWizardNext(&mut self) -> i32 {
        let this_: *mut Self = self;
        if !(*this_).m_hExtractionThread.is_null() {
            /* We enter here when extraction has finished, and go to next page if it succeeded */
            WaitForSingleObject((*this_).m_hExtractionThread, INFINITE);
            CloseHandle((*this_).m_hExtractionThread);
            (*this_).m_hExtractionThread = core::ptr::null_mut();
            (*(*this_).m_pExtract).Release();
            if !(*this_).m_bExtractionThreadCancel {
                return 0 as i32;
            } else {
                (*this_).__base0.SetWindowLongPtr(DWLP_MSGRESULT, -1);
                return TRUE as i32;
            }
        }

        /* We end up here if the user manually clicks Next: start extraction */
        (*this_).m_bExtractionThreadCancel = false;

        /* Grey out every control during extraction to prevent user interaction */
        EnableWindow((*this_).__base0.GetDlgItem((IDC_BROWSE as i32)), (FALSE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDC_DIRECTORY as i32)), (FALSE) as BOOL);
        EnableWindow((*this_).__base0.GetDlgItem((IDC_PASSWORD as i32)), (FALSE) as BOOL);
        (*this_).SetWizardButtons(0);

        GetWindowTextW(
            (*this_).__base0.GetDlgItem((IDC_STATUSTEXT as i32)),
            (*this_).m_OldStatus.GetBuffer((MAX_PATH as i32)),
            (MAX_PATH as i32),
        );
        (*this_).m_OldStatus.ReleaseBuffer();
        let mut strExtracting: CStringW = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_EXTRACTING));
        (*this_).__base0.SetDlgItemTextW((IDC_STATUSTEXT as i32), strExtracting.GetString());

        if (*(*this_).m_pExtract).m_DirectoryChanged {
            (*this_).UpdateDirectory();
        }

        (*(*this_).m_pExtract).AddRef();

        (*this_).m_hExtractionThread = CreateThread(
            core::ptr::null_mut(),
            0,
            Some(CExtractSettingsPage::ExtractEntry),
            this_ as _,
            0,
            core::ptr::null_mut(),
        );
        if (*this_).m_hExtractionThread.is_null() {
            /* Extraction thread creation failed, do not go to the next page */
            let err: DWORD = GetLastError();
            DPRINT1!("ERROR, m_hExtractionThread: CreateThread failed: 0x%x\n", err);
            (*(*this_).m_pExtract).Release();

            (*this_).__base0.SetWindowLongPtr(DWLP_MSGRESULT, -1);

            EnableWindow((*this_).__base0.GetDlgItem((IDC_BROWSE as i32)), (TRUE) as BOOL);
            EnableWindow((*this_).__base0.GetDlgItem((IDC_DIRECTORY as i32)), (TRUE) as BOOL);
            EnableWindow((*this_).__base0.GetDlgItem((IDC_PASSWORD as i32)), (TRUE) as BOOL);
            (*this_).SetWizardButtons(PSWIZB_NEXT);
        }
        return TRUE as i32;
    }
    pub unsafe fn WizardReset(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.SetDlgItemTextW((IDC_STATUSTEXT as i32), (*this_).m_OldStatus.GetString());
    }
    pub unsafe extern "system" fn ExtractEntry(lpParam: LPVOID) -> DWORD {
        let pPage: *mut CExtractSettingsPage = lpParam as *mut CExtractSettingsPage;
        let res: bool = (*(*pPage).m_pExtract).Extract(
            (*pPage).__base0.m_hWnd,
            (*pPage).__base0.GetDlgItem((IDC_PROGRESS as i32)),
            core::ptr::addr_of_mut!((*pPage).m_bExtractionThreadCancel),
        );
        /* Failing and cancelling extraction both mean we stay on the same property page */
        (*pPage).m_bExtractionThreadCancel = !res;

        (*pPage).SetWizardButtons(PSWIZB_NEXT);
        if !res {
            /* Extraction failed/cancelled: the page becomes interactive again */
            EnableWindow((*pPage).__base0.GetDlgItem((IDC_BROWSE as i32)), (TRUE) as BOOL);
            EnableWindow((*pPage).__base0.GetDlgItem((IDC_DIRECTORY as i32)), (TRUE) as BOOL);
            EnableWindow((*pPage).__base0.GetDlgItem((IDC_PASSWORD as i32)), (TRUE) as BOOL);

            /* Reset the progress bar's appearance */
            let mut Progress: CWindow = CWindow__make_0((*pPage).__base0.GetDlgItem((IDC_PROGRESS as i32)));
            Progress.SendMessageW__0(PBM_SETRANGE32, 0, 1);
            Progress.SendMessageW__0(PBM_SETPOS, 0, 0);
            (*pPage).WizardReset();
        }
        SendMessageCallback(
            (*pPage).__base0.GetParent().m_hWnd,
            PSM_PRESSBUTTON,
            (PSBTN_NEXT as _),
            0,
            None,
            0, // dwData: ULONG_PTR
        );

        return 0;
    }
    pub unsafe fn OnQueryCancel(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).m_hExtractionThread != core::ptr::null_mut() {
            /* Extraction will check the value of m_bExtractionThreadCancel between each file in the archive */
            (*this_).m_bExtractionThreadCancel = true;
            return (TRUE) as BOOL;
        }
        return (FALSE) as BOOL;
    }
    pub unsafe extern "system" fn s_BrowseCallbackProc(hWnd: HWND, uMsg: UINT, lp: LPARAM, pData: LPARAM) -> INT {
        if uMsg == BFFM_INITIALIZED {
            let info: *mut browse_info = pData as *mut browse_info;
            let mut dlg: CWindow = CWindow__make_0(hWnd);
            dlg.SendMessageW__0(BFFM_SETSELECTION, (TRUE) as WPARAM, (*info).Directory as LPARAM);
            dlg.CenterWindow((*info).hWnd);
        }
        return 0;
    }
    pub unsafe fn OnBrowse(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut bi: BROWSEINFOW = core::mem::zeroed();
        bi.hwndOwner = (*this_).__base0.m_hWnd;
        let mut path = [0u16; (MAX_PATH) as _];
        bi.pszDisplayName = path.as_mut_ptr();
        bi.lpfn = Some(CExtractSettingsPage::s_BrowseCallbackProc);
        bi.ulFlags = BIF_NEWDIALOGSTYLE | BIF_RETURNFSANCESTORS | BIF_RETURNONLYFSDIRS;
        let mut title = CStringW::from_ptr(MAKEINTRESOURCEW!(IDS_WIZ_BROWSE_TITLE));
        bi.lpszTitle = title.GetString();

        if (*(*this_).m_pExtract).m_DirectoryChanged {
            (*this_).UpdateDirectory();
        }

        let mut info: browse_info = browse_info {
            hWnd: (*this_).__base0.m_hWnd,
            Directory: (*(*this_).m_pExtract).m_Directory.GetString(),
        };
        bi.lParam = &mut info as *mut browse_info as LPARAM;

        let mut pidl: CComHeapPtr<ITEMIDLIST> = CComHeapPtr::new();
        pidl.Attach(SHBrowseForFolderW(core::ptr::addr_of_mut!(bi)));

        let mut tmpPath = [0u16; (MAX_PATH) as _];
        if !pidl.is_null() && SHGetPathFromIDListW(pidl.as_ptr(), tmpPath.as_mut_ptr()) != 0 {
            (*(*this_).m_pExtract).m_Directory = CStringW::from_ptr(tmpPath.as_ptr());
            (*this_).__base0.SetDlgItemTextW((IDC_DIRECTORY as i32), (*(*this_).m_pExtract).m_Directory.GetString());
            (*(*this_).m_pExtract).m_DirectoryChanged = false;
        }
        return 0;
    }
    pub unsafe fn OnEnChangeDirectory(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        (*(*this_).m_pExtract).m_DirectoryChanged = true;
        return 0;
    }
    pub unsafe fn OnPassword(&mut self, wNotifyCode: WORD, wID: WORD, hWndCtl: HWND, bHandled: &mut BOOL) -> LRESULT {
        let this_: *mut Self = self;
        let mut Password: CStringA = CStringA::new();
        if _CZipAskPassword((*this_).__base0.m_hWnd, core::ptr::null_mut(), &mut Password) == eAccept {
            *(*this_).m_pPassword = Password;
        }
        return 0;
    }
    pub unsafe fn UpdateDirectory(&mut self) {
        let this_: *mut Self = self;
        (*this_).__base0.GetDlgItemTextW__1((IDC_DIRECTORY as i32), &mut (*(*this_).m_pExtract).m_Directory);
        (*(*this_).m_pExtract).m_DirectoryChanged = false;
    }
}

impl browse_info {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;}
}

impl CCompleteSettingsPage {
    pub unsafe fn __ctor_0(&mut self, extract: *mut CZipExtract) {
        let this_: *mut Self = self;
        (*this_).__base0.__ctor_0(core::ptr::null_mut());
        (*this_).__init_vtables();
        (*this_).m_psp = CCompleteSettingsPage::__psp_init_titled(this_, MAKEINTRESOURCE!(IDS_WIZ_TITLE));
        (*this_).m_pExtract = extract;
        {
            (*this_).m_psp.pszHeaderTitle = MAKEINTRESOURCE!(IDS_WIZ_COMPL_TITLE);
            (*this_).m_psp.pszHeaderSubTitle = MAKEINTRESOURCE!(IDS_WIZ_COMPL_SUBTITLE);
            (*this_).m_psp.dwFlags |= PSP_USETITLE | PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE;
        }
    }
    pub unsafe fn OnSetActive(&mut self) -> i32 {
        let this_: *mut Self = self;
        (*this_).SetWizardButtons(PSWIZB_FINISH);
        let mut Path: CStringW = CStringW::from_ptr((*(*this_).m_pExtract).m_Directory.GetString());
        let Ptr: PWSTR = Path.GetBuffer((MAX_PATH as i32));
        let mut rc: RECT = core::mem::zeroed();
        GetWindowRect((*this_).__base0.GetDlgItem((IDC_DESTDIR as i32)), core::ptr::addr_of_mut!(rc));
        let dc: HDC = (*this_).__base0.GetDC();
        PathCompactPathW(dc, Ptr, (rc.right - rc.left) as u32);
        (*this_).__base0.ReleaseDC(dc);
        Path.ReleaseBuffer();
        (*this_).__base0.SetDlgItemTextW((IDC_DESTDIR as i32), Path.GetString());
        (*this_).__base0.CheckDlgButton((IDC_SHOW_EXTRACTED as i32), BST_CHECKED);
        return 0;
    }
    pub unsafe fn OnWizardFinish(&mut self) -> BOOL {
        let this_: *mut Self = self;
        if (*this_).__base0.IsDlgButtonChecked((IDC_SHOW_EXTRACTED as i32)) == BST_CHECKED {
            ShellExecuteW(
                core::ptr::null_mut(),
                CPP_COM_WSTR_31AF3113D116.as_ptr(),
                (*(*this_).m_pExtract).m_Directory.GetString(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                (SW_SHOW) as INT,
            );
        }
        return (FALSE) as BOOL;
    }
}
