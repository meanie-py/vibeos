// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const MEGABYTE: u32 = 1024 * 1024;

pub const MINIMUM_PAGEFILE_SIZE: u32 = 2;

pub const MAXIMUM_PAGEFILE_SIZE32: u32 = 4 * 1024 - 1;

pub const MAXIMUM_PAGEFILE_SIZE64: u32 = 16 * 1024 * 1024 - 1;

#[cfg(target_arch = "x86")]
pub const MAXIMUM_PAGEFILE_SIZE: u32 = MAXIMUM_PAGEFILE_SIZE32;

#[cfg(target_arch = "x86")]
pub const MAXIMUM_PAGEFILE_SIZE_PAE: u32 = MAXIMUM_PAGEFILE_SIZE64;

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
pub const MAXIMUM_PAGEFILE_SIZE: u32 = MAXIMUM_PAGEFILE_SIZE64;

#[cfg(target_arch = "arm")]
pub const MAXIMUM_PAGEFILE_SIZE: u32 = 2 * 1024 - 1;

#[repr(C)]
pub struct _PAGEFILE {
    pub szDrive: [TCHAR; 3],
    pub pszVolume: LPTSTR,
    pub OldMinSize: INT,
    pub OldMaxSize: INT,
    pub NewMinSize: INT,
    pub NewMaxSize: INT,
    pub FreeSize: UINT,
    pub bIsNotFAT: BOOL,
    pub bUsed: BOOL,
}

pub type PAGEFILE = _PAGEFILE;
pub type PPAGEFILE = *mut _PAGEFILE;

#[repr(C)]
pub struct _VIRTMEM {
    pub hSelf: HWND,
    pub hListBox: HWND,
    pub szPagingFiles: LPTSTR,
    pub Count: UINT,
    pub bModified: BOOL,
    pub PageFile: [PAGEFILE; 26],
}

pub type VIRTMEM = _VIRTMEM;
pub type PVIRTMEM = *mut _VIRTMEM;

// virtmem.c:102 `static LPCTSTR lpKey = _T("…")`. A raw pointer is not `Sync`, so this
// cannot be a `static`; `w!` already yields a pointer into an aligned static array.
macro_rules! lpKey {
    () => {
        w!("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management") as LPCTSTR
    };
}
unsafe extern "C" fn GetMaxPageFileSize(PageFile: *mut PAGEFILE) -> UINT {
    #[cfg(target_arch = "x86")]
    {
        if (*core::ptr::addr_of!((*SharedUserData).ProcessorFeatures))[(PF_PAE_ENABLED) as usize] != 0
            && (*PageFile).bIsNotFAT != 0
        {
            return min!((*PageFile).FreeSize, MAXIMUM_PAGEFILE_SIZE_PAE);
        }
    }
    return min!((*PageFile).FreeSize, MAXIMUM_PAGEFILE_SIZE);
}
unsafe extern "C" fn ReadPageFileSettings(pVirtMem: PVIRTMEM) -> i32 {
    let mut bRet: i32 = FALSE;
    let mut hkey: HKEY = core::ptr::null_mut();
    let mut dwType: u32 = 0;
    let mut dwDataSize: u32 = 0;

    if RegCreateKeyEx(
        HKEY_LOCAL_MACHINE,
        lpKey!(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_QUERY_VALUE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hkey),
        core::ptr::null_mut(),
    ) == (ERROR_SUCCESS) as i32
    {
        if RegQueryValueEx(
            hkey,
            _T(w!("PagingFiles")),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwType),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(dwDataSize),
        ) == (ERROR_SUCCESS) as i32
        {
            (*pVirtMem).szPagingFiles = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (dwDataSize) as SIZE_T,
            ) as LPTSTR;
            if !(*pVirtMem).szPagingFiles.is_null() {
                if RegQueryValueEx(
                    hkey,
                    _T(w!("PagingFiles")),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(dwType),
                    (*pVirtMem).szPagingFiles as PBYTE,
                    core::ptr::addr_of_mut!(dwDataSize),
                ) == (ERROR_SUCCESS) as i32
                {
                    bRet = TRUE;
                }
            }
        }
    }

    if bRet == 0 {
        ShowLastWin32Error((*pVirtMem).hSelf);
    }

    if !hkey.is_null() {
        RegCloseKey(hkey);
    }

    bRet
}
unsafe extern "C" fn GetPageFileSizes(
    mut lpPageFiles: LPTSTR,
    lpInitialSize: LPINT,
    lpMaximumSize: LPINT,
) {
    let mut i: u32 = 0;

    *lpInitialSize = -1;
    *lpMaximumSize = -1;

    while *lpPageFiles != _T('\0' as TCHAR) {
        if *lpPageFiles == _T(' ' as TCHAR) {
            lpPageFiles = lpPageFiles.offset(1);

            match i {
                0 => {
                    *lpInitialSize = _ttoi(lpPageFiles) as i32;
                    i = 1;
                }

                1 => {
                    *lpMaximumSize = _ttoi(lpPageFiles) as i32;
                    return;
                }

                _ => {}
            }
        }

        lpPageFiles = lpPageFiles.offset(1);
    }
}
unsafe extern "C" fn ParseMemSettings(pVirtMem: PVIRTMEM) {
    let mut szDrives: [TCHAR; 1024] = [0; 1024];
    let mut DrivePtr: LPTSTR;
    let mut szDrive: [TCHAR; 3] = [0; 3];
    let mut szVolume: [TCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];
    let mut szFSName: [TCHAR; (MAX_PATH + 1) as usize] = [0; (MAX_PATH + 1) as usize];
    let mut MinSize: INT;
    let mut MaxSize: INT;
    let mut DriveLen: INT;
    let mut Len: INT;
    let mut PgCnt: UINT = 0;
    let mut PageFile: PPAGEFILE;

    DriveLen = (GetLogicalDriveStrings(_countof!(szDrives) - 1, szDrives.as_mut_ptr())) as i32;

    DrivePtr = szDrives.as_mut_ptr();

    while DriveLen != 0 {
        Len = lstrlen(DrivePtr) + 1;
        DriveLen -= Len;

        DrivePtr = _tcsupr(DrivePtr);

        lstrcpyn(szDrive.as_mut_ptr(), DrivePtr, (_countof!(szDrive)) as c_int);

        if GetDriveType(DrivePtr) == DRIVE_FIXED {
            MinSize = -1;
            MaxSize = -1;

            if _tcsnicmp((*pVirtMem).szPagingFiles, szDrive.as_mut_ptr(), 2) == 0 {
                GetPageFileSizes((*pVirtMem).szPagingFiles, core::ptr::addr_of_mut!(MinSize), core::ptr::addr_of_mut!(MaxSize));
            }

            PageFile = core::ptr::addr_of_mut!((*pVirtMem).PageFile[PgCnt as usize]);
            (*PageFile).OldMinSize = MinSize;
            (*PageFile).OldMaxSize = MaxSize;
            (*PageFile).NewMinSize = MinSize;
            (*PageFile).NewMaxSize = MaxSize;
            (*PageFile).bIsNotFAT = TRUE;
            (*PageFile).bUsed = TRUE;
            lstrcpy((*PageFile).szDrive.as_mut_ptr(), szDrive.as_mut_ptr());

            if GetVolumeInformation(DrivePtr, szVolume.as_mut_ptr(), _countof!(szVolume), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::null_mut(), szFSName.as_mut_ptr(), _countof!(szFSName)) != 0 {
                (*PageFile).pszVolume = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ((_tcslen(szVolume.as_mut_ptr()) + 1) * core::mem::size_of::<TCHAR>()) as SIZE_T) as LPTSTR;
                if (*PageFile).pszVolume != core::ptr::null_mut() {
                    _tcscpy((*PageFile).pszVolume, szVolume.as_mut_ptr());
                }

                (*PageFile).bIsNotFAT = (_tcsnicmp(szFSName.as_mut_ptr(), w!("FAT"), 3) != 0) as BOOL;
            }

            PgCnt += 1;
        }

        DrivePtr = DrivePtr.offset(Len as isize);
    }

    (*pVirtMem).Count = PgCnt;
}
unsafe extern "C" fn WritePageFileSettings(pVirtMem: PVIRTMEM) {
    let mut bErr = TRUE;
    let mut hk: HKEY = core::ptr::null_mut();
    let mut szText: [TCHAR; 256] = core::mem::zeroed();
    let mut szPagingFiles: [TCHAR; 2048] = core::mem::zeroed();
    let mut nPos: UINT = 0;
    let mut PageFile: PPAGEFILE;

    let mut i = 0;
    while i < (*pVirtMem).Count {
        PageFile = core::ptr::addr_of_mut!((*pVirtMem).PageFile[i as usize]);

        if (*PageFile).bUsed != 0 &&
            (*PageFile).NewMinSize != -1 &&
            (*PageFile).NewMaxSize != -1
        {
            _stprintf(
                szText.as_mut_ptr(),
                _T(w!("%s\\pagefile.sys %i %i") as LPCTSTR),
                (*PageFile).szDrive.as_ptr(),
                (*PageFile).NewMinSize,
                (*PageFile).NewMaxSize
            );

            lstrcpy(szPagingFiles.as_mut_ptr().add(nPos as usize), szText.as_ptr());

            nPos += lstrlen(szText.as_ptr()) as UINT + 1;

            *szPagingFiles.as_mut_ptr().add(nPos as usize) = _T(w!("") as LPCTSTR) as TCHAR;
            nPos += 1;
        }
        i += 1;
    }

    if RegCreateKeyEx(
        HKEY_LOCAL_MACHINE,
        lpKey!(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_WRITE,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(hk),
        core::ptr::null_mut()
    ) == (ERROR_SUCCESS) as i32
    {
        if RegSetValueEx(
            hk,
            _T(w!("PagingFiles") as LPCTSTR),
            0,
            REG_MULTI_SZ,
            szPagingFiles.as_ptr() as LPBYTE,
            (nPos as u32) * core::mem::size_of::<TCHAR>() as u32
        ) == (ERROR_SUCCESS) as i32
        {
            bErr = FALSE;
        }

        RegCloseKey(hk);
    }

    if bErr == FALSE {
        i = 0;
        while i < _countof!((*pVirtMem).PageFile) {
            if (*pVirtMem).PageFile[i as usize].OldMinSize != -1 &&
                (*pVirtMem).PageFile[i as usize].NewMinSize == -1
            {
                _stprintf(
                    szText.as_mut_ptr(),
                    _T(w!("%s\\pagefile.sys") as LPCTSTR),
                    (*pVirtMem).PageFile[i as usize].szDrive.as_ptr()
                );

                MoveFileEx(szText.as_ptr(), core::ptr::null(), MOVEFILE_DELAY_UNTIL_REBOOT);
            }
            i += 1;
        }
    }

    if bErr != 0 {
        ShowLastWin32Error((*pVirtMem).hSelf);
    }
}
unsafe extern "C" fn SetListBoxColumns(hwndListBox: HWND) {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 103,
        bottom: 0,
    };
    MapDialogRect(hwndListBox, &mut rect);

    SendMessage(
        hwndListBox,
        LB_SETTABSTOPS,
        1 as WPARAM,
        core::ptr::addr_of_mut!(rect.right) as LPARAM,
    );
}
unsafe extern "C" fn OnNoPagingFile(pVirtMem: PVIRTMEM) {
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_INITIALSIZE) as i32), FALSE);
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_MAXSIZE) as i32), FALSE);
}
unsafe extern "C" fn OnSysManSize(pVirtMem: PVIRTMEM) {
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_INITIALSIZE) as i32), FALSE);
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_MAXSIZE) as i32), FALSE);
}
unsafe extern "C" fn OnCustom(pVirtMem: PVIRTMEM) {
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_INITIALSIZE) as i32), TRUE);
    EnableWindow(GetDlgItem((*pVirtMem).hSelf, (IDC_MAXSIZE) as i32), TRUE);
}
unsafe extern "C" fn InitPagefileList(pVirtMem: PVIRTMEM) {
    let mut Index: INT;
    let mut i: UINT;
    let mut PageFile: PPAGEFILE;
    let mut szSize: [TCHAR; 64] = [0; 64];
    let mut szDisplayString: [TCHAR; 256] = [0; 256];

    i = 0;
    while i < _countof!((*pVirtMem).PageFile) {
        PageFile = &mut (*pVirtMem).PageFile[i as usize];

        if (*PageFile).bUsed != 0 {
            if ((*PageFile).NewMinSize == -1) &&
               ((*PageFile).NewMaxSize == -1) {
                LoadString(hApplet,
                           IDS_PAGEFILE_NONE,
                           szSize.as_mut_ptr(),
                           (_countof!(szSize)) as i32);
            } else if ((*PageFile).NewMinSize == 0) &&
                      ((*PageFile).NewMaxSize == 0) {
                LoadString(hApplet,
                           IDS_PAGEFILE_SYSTEM,
                           szSize.as_mut_ptr(),
                           (_countof!(szSize)) as i32);
            } else {
                _stprintf(szSize.as_mut_ptr(), w!("%d - %d"),
                          (*PageFile).NewMinSize,
                          (*PageFile).NewMaxSize);
            }

            _stprintf(szDisplayString.as_mut_ptr(),
                      w!("%s  [%s]\t%s"),
                      (*PageFile).szDrive.as_ptr(),
                      if !(*PageFile).pszVolume.is_null() { (*PageFile).pszVolume } else { w!("") },
                      szSize.as_ptr());

            Index = (SendMessage((*pVirtMem).hListBox, LB_ADDSTRING, 0 as WPARAM, szDisplayString.as_ptr() as LPARAM)) as i32;
            SendMessage((*pVirtMem).hListBox, LB_SETITEMDATA, Index as WPARAM, i as LPARAM);
        }

        i += 1;
    }

    SendMessage((*pVirtMem).hListBox, LB_SETCURSEL, 0 as WPARAM, 0 as LPARAM);

    OnSelChange(pVirtMem);
}
unsafe extern "C" fn UpdatePagefileEntry(
    pVirtMem: PVIRTMEM,
    ListIndex: INT,
    DriveIndex: INT,
) {
    let PageFile: PPAGEFILE = core::ptr::addr_of_mut!((*pVirtMem).PageFile[DriveIndex as usize]);
    let mut szSize: [TCHAR; 64] = core::mem::zeroed();
    let mut szDisplayString: [TCHAR; 256] = core::mem::zeroed();

    if ((*PageFile).NewMinSize == -1) &&
       ((*PageFile).NewMaxSize == -1)
    {
        LoadString(
            hApplet,
            IDS_PAGEFILE_NONE,
            szSize.as_mut_ptr(),
            (_countof!(szSize)) as i32,
        );
    }
    else if ((*PageFile).NewMinSize == 0) &&
            ((*PageFile).NewMaxSize == 0)
    {
        LoadString(
            hApplet,
            IDS_PAGEFILE_SYSTEM,
            szSize.as_mut_ptr(),
            (_countof!(szSize)) as i32,
        );
    }
    else
    {
        _stprintf(
            szSize.as_mut_ptr(),
            _T(w!("%d - %d")),
            (*PageFile).NewMinSize,
            (*PageFile).NewMaxSize,
        );
    }

    _stprintf(
        szDisplayString.as_mut_ptr(),
        _T(w!("%s  [%s]	%s")),
        (*PageFile).szDrive.as_ptr(),
        if !(*PageFile).pszVolume.is_null() { (*PageFile).pszVolume } else { _T(w!("")) },
        szSize.as_ptr(),
    );

    SendMessage((*pVirtMem).hListBox, LB_DELETESTRING, ListIndex as WPARAM, 0);
    SendMessage((*pVirtMem).hListBox, LB_INSERTSTRING, ListIndex as WPARAM, szDisplayString.as_ptr() as LPARAM);
    SendMessage((*pVirtMem).hListBox, LB_SETCURSEL, ListIndex as WPARAM, 0);
}
unsafe extern "C" fn OnSet(pVirtMem: PVIRTMEM) {
    let mut Index: INT;
    let mut MinSize: UINT = (-1i32) as UINT;
    let mut MaxSize: UINT = (-1i32) as UINT;
    let mut bTranslated: BOOL = 0;
    let mut DriveIndex: UINT;
    let mut PageFile: PPAGEFILE;

    Index = SendDlgItemMessage(
        (*pVirtMem).hSelf,
        (IDC_PAGEFILELIST) as i32,
        LB_GETCURSEL,
        0,
        0
    ) as INT;

    if Index >= 0 && Index < (*pVirtMem).Count as INT {
        DriveIndex = SendDlgItemMessage(
            (*pVirtMem).hSelf,
            (IDC_PAGEFILELIST) as i32,
            LB_GETITEMDATA,
            Index as WPARAM,
            0
        ) as UINT;

        PageFile = core::ptr::addr_of_mut!((*pVirtMem).PageFile[DriveIndex as usize]);

        if IsDlgButtonChecked(
            (*pVirtMem).hSelf,
            (IDC_CUSTOM) as i32
        ) == BST_CHECKED {
            let maxPageFileSize: UINT;

            MinSize = GetDlgItemInt(
                (*pVirtMem).hSelf,
                (IDC_INITIALSIZE) as i32,
                core::ptr::addr_of_mut!(bTranslated),
                FALSE
            );
            if bTranslated == 0 {
                ResourceMessageBox(
                    hApplet,
                    core::ptr::null_mut(),
                    MB_ICONWARNING | MB_OK,
                    IDS_MESSAGEBOXTITLE,
                    IDS_WARNINITIALSIZE
                );
                return;
            }

            MaxSize = GetDlgItemInt(
                (*pVirtMem).hSelf,
                (IDC_MAXSIZE) as i32,
                core::ptr::addr_of_mut!(bTranslated),
                FALSE
            );
            if bTranslated == 0 {
                ResourceMessageBox(
                    hApplet,
                    core::ptr::null_mut(),
                    MB_ICONWARNING | MB_OK,
                    IDS_MESSAGEBOXTITLE,
                    IDS_WARNMAXIMUMSIZE
                );
                return;
            }

            maxPageFileSize = GetMaxPageFileSize(PageFile);

            if (MinSize < MINIMUM_PAGEFILE_SIZE) ||
               (MinSize > maxPageFileSize) {
                ResourceMessageBox(
                    hApplet,
                    core::ptr::null_mut(),
                    MB_ICONWARNING | MB_OK,
                    IDS_MESSAGEBOXTITLE,
                    IDS_WARNINITIALRANGE,
                    maxPageFileSize
                );
                return;
            }

            if (MaxSize < MinSize) ||
               (MaxSize > maxPageFileSize) {
                ResourceMessageBox(
                    hApplet,
                    core::ptr::null_mut(),
                    MB_ICONWARNING | MB_OK,
                    IDS_MESSAGEBOXTITLE,
                    IDS_WARNMAXIMUMRANGE,
                    maxPageFileSize
                );
                return;
            }

            (*PageFile).NewMinSize = MinSize as INT;
            (*PageFile).NewMaxSize = MaxSize as INT;
            (*PageFile).bUsed = TRUE;
        } else if IsDlgButtonChecked(
            (*pVirtMem).hSelf,
            (IDC_NOPAGEFILE) as i32
        ) == BST_CHECKED {
            (*PageFile).NewMinSize = -1;
            (*PageFile).NewMaxSize = -1;
            (*PageFile).bUsed = TRUE;
        } else {
            (*PageFile).NewMinSize = 0;
            (*PageFile).NewMaxSize = 0;
            (*PageFile).bUsed = TRUE;
        }

        if ((*PageFile).OldMinSize != (*PageFile).NewMinSize) ||
           ((*PageFile).OldMaxSize != (*PageFile).NewMaxSize) {
            (*pVirtMem).bModified = TRUE;
        }

        UpdatePagefileEntry(pVirtMem, Index, DriveIndex as INT);
    }
}
unsafe extern "C" fn OnSelChange(pVirtMem: PVIRTMEM) -> BOOL {
    let Index: INT;
    let DriveIndex: UINT;
    let PageFile: PPAGEFILE;
    let mut MemoryStatus: MEMORYSTATUSEX = core::mem::zeroed();
    let mut FreeDiskSpace: ULARGE_INTEGER = core::mem::zeroed();
    let i: UINT;
    let PageFileSizeMb: UINT;
    let mut szMegabytes: [TCHAR; 8] = core::mem::zeroed();
    let mut szBuffer: [TCHAR; 64] = core::mem::zeroed();
    let mut szText: [TCHAR; (MAX_PATH) as usize] = core::mem::zeroed();
    let mut fdata: WIN32_FIND_DATA = core::mem::zeroed();
    let mut hFind: HANDLE;
    let mut pfSize: ULARGE_INTEGER = core::mem::zeroed();

    Index = SendDlgItemMessage(
        (*pVirtMem).hSelf,
        (IDC_PAGEFILELIST) as i32,
        LB_GETCURSEL,
        0,
        0
    ) as INT;

    if Index >= 0 && Index < (*pVirtMem).Count as INT {
        DriveIndex = SendDlgItemMessage(
            (*pVirtMem).hSelf,
            (IDC_PAGEFILELIST) as i32,
            LB_GETITEMDATA,
            Index as WPARAM,
            0
        ) as UINT;

        PageFile = core::ptr::addr_of_mut!((*pVirtMem).PageFile[DriveIndex as usize]);

        LoadString(
            hApplet,
            IDS_PAGEFILE_MB,
            szMegabytes.as_mut_ptr(),
            (_countof!(szMegabytes)) as i32
        );

        SetDlgItemText(
            (*pVirtMem).hSelf,
            (IDC_DRIVE) as i32,
            (*PageFile).szDrive.as_ptr()
        );

        if GetDiskFreeSpaceEx(
            (*PageFile).szDrive.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(FreeDiskSpace)
        ) != 0
        {
            (*PageFile).FreeSize = (FreeDiskSpace.QuadPart / (MEGABYTE) as u64) as UINT;
            _stprintf(szBuffer.as_mut_ptr(), szMegabytes.as_ptr(), (*PageFile).FreeSize);
            SetDlgItemText((*pVirtMem).hSelf, (IDC_SPACEAVAIL) as i32, szBuffer.as_ptr());
        }

        if (*PageFile).NewMinSize == -1 &&
           (*PageFile).NewMaxSize == -1
        {
            OnNoPagingFile(pVirtMem);
            CheckDlgButton((*pVirtMem).hSelf, (IDC_NOPAGEFILE) as i32, BST_CHECKED);
        }
        else if (*PageFile).NewMinSize == 0 &&
                (*PageFile).NewMaxSize == 0
        {
            OnSysManSize(pVirtMem);
            CheckDlgButton((*pVirtMem).hSelf, (IDC_SYSMANSIZE) as i32, BST_CHECKED);
        }
        else
        {
            OnCustom(pVirtMem);

            SetDlgItemInt(
                (*pVirtMem).hSelf,
                (IDC_INITIALSIZE) as i32,
                (*PageFile).NewMinSize as UINT,
                FALSE
            );

            SetDlgItemInt(
                (*pVirtMem).hSelf,
                (IDC_MAXSIZE) as i32,
                (*PageFile).NewMaxSize as UINT,
                FALSE
            );

            CheckDlgButton(
                (*pVirtMem).hSelf,
                (IDC_CUSTOM) as i32,
                BST_CHECKED
            );
        }

        _stprintf(szBuffer.as_mut_ptr(), szMegabytes.as_ptr(), MINIMUM_PAGEFILE_SIZE);
        SetDlgItemText((*pVirtMem).hSelf, (IDC_MINIMUM) as i32, szBuffer.as_ptr());

        MemoryStatus.dwLength = core::mem::size_of::<MEMORYSTATUSEX>() as u32;
        if GlobalMemoryStatusEx(core::ptr::addr_of_mut!(MemoryStatus)) != 0 {
            let FreeMemMb: UINT;
            let mut RecoMemMb: UINT;
            let maxPageFileSize: UINT = GetMaxPageFileSize(PageFile);

            FreeMemMb = (MemoryStatus.ullTotalPhys / (MEGABYTE) as u64) as UINT;
            RecoMemMb = FreeMemMb + (FreeMemMb / 2);
            if RecoMemMb > maxPageFileSize {
                RecoMemMb = maxPageFileSize;
            }
            _stprintf(szBuffer.as_mut_ptr(), szMegabytes.as_ptr(), RecoMemMb);
            SetDlgItemText((*pVirtMem).hSelf, (IDC_RECOMMENDED) as i32, szBuffer.as_ptr());
        }

        let mut PageFileSizeMb = 0;

        let mut i = 0;
        while i < (*pVirtMem).Count {
            _stprintf(
                szText.as_mut_ptr(),
                w!("%c:\\pagefile.sys"),
                (*pVirtMem).PageFile[i as usize].szDrive[0 as usize] as UINT
            );

            hFind = FindFirstFile(szText.as_ptr(), core::ptr::addr_of_mut!(fdata));
            if hFind == INVALID_HANDLE_VALUE {
                // DPRINT removed
            } else {
                pfSize.u.LowPart = fdata.nFileSizeLow;
                pfSize.u.HighPart = fdata.nFileSizeHigh;
                PageFileSizeMb += (pfSize.QuadPart / (MEGABYTE) as u64) as UINT;
                FindClose(hFind);
            }
            i += 1;
        }

        _stprintf(szBuffer.as_mut_ptr(), szMegabytes.as_ptr(), PageFileSizeMb);
        SetDlgItemText((*pVirtMem).hSelf, (IDC_CURRENT) as i32, szBuffer.as_ptr());
    }

    TRUE
}
unsafe extern "C" fn OnVirtMemDialogOk(pVirtMem: PVIRTMEM) {
    if (*pVirtMem).bModified != FALSE {
        ResourceMessageBox(
            hApplet,
            core::ptr::null_mut(),
            MB_ICONINFORMATION | MB_OK,
            IDS_MESSAGEBOXTITLE,
            IDS_INFOREBOOT,
        );

        WritePageFileSettings(pVirtMem);
    }
}
unsafe extern "C" fn OnInitVirtMemDialog(hwnd: HWND, pVirtMem: PVIRTMEM) {
    let mut i: u32;
    let mut PageFile: PPAGEFILE;

    (*pVirtMem).hSelf = hwnd;
    (*pVirtMem).hListBox = GetDlgItem(hwnd, (IDC_PAGEFILELIST) as i32);
    (*pVirtMem).bModified = FALSE;

    SetListBoxColumns((*pVirtMem).hListBox);

    i = 0;
    while i < _countof!((*pVirtMem).PageFile) {
        PageFile = &mut (*pVirtMem).PageFile[i as usize];
        (*PageFile).bUsed = FALSE;
        (*PageFile).OldMinSize = -1;
        (*PageFile).OldMaxSize = -1;
        (*PageFile).NewMinSize = -1;
        (*PageFile).NewMaxSize = -1;
        (*PageFile).FreeSize = 0;
        (*PageFile).bIsNotFAT = TRUE;
        i += 1;
    }

    ReadPageFileSettings(pVirtMem);

    ParseMemSettings(pVirtMem);

    InitPagefileList(pVirtMem);
}
unsafe extern "C" fn OnDestroy(pVirtMem: PVIRTMEM) {
    let mut i: UINT;

    i = 0;
    while i < _countof!((*pVirtMem).PageFile) {
        if !(*pVirtMem).PageFile[i as usize].pszVolume.is_null() {
            HeapFree(GetProcessHeap(), 0, (*pVirtMem).PageFile[i as usize].pszVolume as *mut _);
        }
        i = i.wrapping_add(1);
    }

    if !(*pVirtMem).szPagingFiles.is_null() {
        HeapFree(GetProcessHeap(), 0, (*pVirtMem).szPagingFiles as *mut _);
    }

    HeapFree(GetProcessHeap(), 0, pVirtMem as *mut _);
}
#[no_mangle]
pub unsafe extern "system" fn VirtMemDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pVirtMem: PVIRTMEM;

    UNREFERENCED_PARAMETER!(lParam);

    pVirtMem = GetWindowLongPtr(hwndDlg, DWLP_USER) as PVIRTMEM;

    match uMsg {
        WM_INITDIALOG => {
            pVirtMem = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (core::mem::size_of::<VIRTMEM>()) as SIZE_T,
            ) as PVIRTMEM;
            if pVirtMem.is_null() {
                EndDialog(hwndDlg, 0);
                return (FALSE) as INT_PTR;
            }

            SetWindowLongPtr(hwndDlg, DWLP_USER, pVirtMem as LONG_PTR);

            OnInitVirtMemDialog(hwndDlg, pVirtMem);
        }

        WM_DESTROY => {
            OnDestroy(pVirtMem);
        }

        WM_COMMAND => {
            match (LOWORD!(wParam)) as u32 {
                IDCANCEL => {
                    EndDialog(hwndDlg, 0);
                    return (TRUE) as INT_PTR;
                }

                IDOK => {
                    OnVirtMemDialogOk(pVirtMem);
                    EndDialog(hwndDlg, ((*pVirtMem).bModified) as INT_PTR);
                    return (TRUE) as INT_PTR;
                }

                IDC_NOPAGEFILE => {
                    OnNoPagingFile(pVirtMem);
                    return (TRUE) as INT_PTR;
                }

                IDC_SYSMANSIZE => {
                    OnSysManSize(pVirtMem);
                    return (TRUE) as INT_PTR;
                }

                IDC_CUSTOM => {
                    OnCustom(pVirtMem);
                    return (TRUE) as INT_PTR;
                }

                IDC_SET => {
                    OnSet(pVirtMem);
                    return (TRUE) as INT_PTR;
                }

                IDC_PAGEFILELIST => {
                    match (HIWORD!(wParam)) as u32 {
                        LBN_SELCHANGE => {
                            OnSelChange(pVirtMem);
                            return (TRUE) as INT_PTR;
                        }
                        _ => {}
                    }
                }

                _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS system properties, control panel applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/virtmem.c
//  * PURPOSE:     Virtual memory control dialog
//  * COPYRIGHT:   Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "precomp.h"
//
// #ifdef _M_IX86
// /* Used for SharedUserData by GetMaxPageFileSize() in the PAE case */
// #define NTOS_MODE_USER
// #include <ndk/pstypes.h>
// #endif
//
// #define NDEBUG
// #include <debug.h>
//
// // #define MAX_PAGING_FILES     16  // See also ntoskrnl/include/internal/mm.h
// #define MEGABYTE                        (1024 * 1024)
//
// /* Values adapted from smss/pagefile.c, converted in megabytes and rounded-down.
//  * Compare to the more "accurate" values from SMSS (and NTOS) in bytes. */
//
// /* Minimum pagefile size: 2 MB, instead of 256 pages (1 MB) */
// #define MINIMUM_PAGEFILE_SIZE           2
//
// /* Maximum pagefile sizes for different architectures */
// #define MAXIMUM_PAGEFILE_SIZE32         (4UL * 1024 - 1)
// #define MAXIMUM_PAGEFILE_SIZE64         (16UL * 1024 * 1024 - 1)
//
// #if defined(_M_IX86)
// /* 4095 MB */
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
// /* PAE uses the same size as x64 */
//     #define MAXIMUM_PAGEFILE_SIZE_PAE   MAXIMUM_PAGEFILE_SIZE64
// #elif defined (_M_AMD64) || defined(_M_ARM64)
// /* Around 16 TB */
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE64
// #elif defined (_M_IA64)
// /* Around 32 TB */
//     #define MAXIMUM_PAGEFILE_SIZE       (32UL * 1024 * 1024 - 1)
// #elif defined(_M_ARM)
// /* Around 2 GB */
//     #if (NTDDI_VERSION >= NTDDI_WINBLUE) // NTDDI_WIN81
//     #define MAXIMUM_PAGEFILE_SIZE       (2UL * 1024 - 1)
//     #else
// /* Around 4 GB */
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
//     #endif
// #else
// /* On unknown architectures, default to either one of the 32 or 64 bit sizes */
// #pragma message("Unknown architecture")
//     #ifdef _WIN64
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE64
//     #else
//     #define MAXIMUM_PAGEFILE_SIZE       MAXIMUM_PAGEFILE_SIZE32
//     #endif
// #endif
//
// typedef struct _PAGEFILE
// {
//     TCHAR  szDrive[3];
//     LPTSTR pszVolume;
//     INT    OldMinSize;
//     INT    OldMaxSize;
//     INT    NewMinSize;
//     INT    NewMaxSize;
//     UINT   FreeSize;
//     BOOL   bIsNotFAT;
//     BOOL   bUsed;
// } PAGEFILE, *PPAGEFILE;
//
// typedef struct _VIRTMEM
// {
//     HWND   hSelf;
//     HWND   hListBox;
//     LPTSTR szPagingFiles;
//     UINT   Count;
//     BOOL   bModified;
//     PAGEFILE PageFile[26];
// } VIRTMEM, *PVIRTMEM;
//
//
// static __inline
// UINT
// GetMaxPageFileSize(
//      PPAGEFILE PageFile)
// {
// #ifdef _M_IX86
//     /* For x86 PAE-enabled systems, where the maximum pagefile size is
//      * greater than 4 GB, verify also that the drive's filesystem on which
//      * the pagefile is stored can support it. */
//     if (SharedUserData->ProcessorFeatures[PF_PAE_ENABLED] && PageFile->bIsNotFAT)
//         return min(PageFile->FreeSize, MAXIMUM_PAGEFILE_SIZE_PAE);
// #endif
//     return min(PageFile->FreeSize, MAXIMUM_PAGEFILE_SIZE);
// }
//
// static LPCTSTR lpKey = _T("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Memory Management");
//
// static BOOL
// ReadPageFileSettings(PVIRTMEM pVirtMem)
// {
//     BOOL bRet = FALSE;
//     HKEY hkey = NULL;
//     DWORD dwType;
//     DWORD dwDataSize;
//
//     if (RegCreateKeyEx(HKEY_LOCAL_MACHINE,
//                        lpKey,
//                        0,
//                        NULL,
//                        REG_OPTION_NON_VOLATILE,
//                        KEY_QUERY_VALUE,
//                        NULL,
//                        &hkey,
//                        NULL) == ERROR_SUCCESS)
//     {
//         if (RegQueryValueEx(hkey,
//                             _T("PagingFiles"),
//                             NULL,
//                             &dwType,
//                             NULL,
//                             &dwDataSize) == ERROR_SUCCESS)
//         {
//             pVirtMem->szPagingFiles = (LPTSTR)HeapAlloc(GetProcessHeap(),
//                                                         HEAP_ZERO_MEMORY,
//                                                         dwDataSize);
//             if (pVirtMem->szPagingFiles != NULL)
//             {
//                 if (RegQueryValueEx(hkey,
//                                     _T("PagingFiles"),
//                                     NULL,
//                                     &dwType,
//                                     (PBYTE)pVirtMem->szPagingFiles,
//                                     &dwDataSize) == ERROR_SUCCESS)
//                 {
//                     bRet = TRUE;
//                 }
//             }
//         }
//     }
//
//     if (!bRet)
//         ShowLastWin32Error(pVirtMem->hSelf);
//
//     if (hkey != NULL)
//         RegCloseKey(hkey);
//
//     return bRet;
// }
//
//
// static VOID
// GetPageFileSizes(LPTSTR lpPageFiles,
//                  LPINT lpInitialSize,
//                  LPINT lpMaximumSize)
// {
//     UINT i = 0;
//
//     *lpInitialSize = -1;
//     *lpMaximumSize = -1;
//
//     while (*lpPageFiles != _T('\0'))
//     {
//         if (*lpPageFiles == _T(' '))
//         {
//             lpPageFiles++;
//
//             switch (i)
//             {
//                 case 0:
//                     *lpInitialSize = (INT)_ttoi(lpPageFiles);
//                     i = 1;
//                     break;
//
//                 case 1:
//                     *lpMaximumSize = (INT)_ttoi(lpPageFiles);
//                     return;
//             }
//         }
//
//         lpPageFiles++;
//     }
// }
//
//
// static VOID
// ParseMemSettings(PVIRTMEM pVirtMem)
// {
//     TCHAR szDrives[1024];    // All drives
//     LPTSTR DrivePtr = szDrives;
//     TCHAR szDrive[3];        // Single drive
//     TCHAR szVolume[MAX_PATH + 1];
//     TCHAR szFSName[MAX_PATH + 1];
//     INT MinSize;
//     INT MaxSize;
//     INT DriveLen;
//     INT Len;
//     UINT PgCnt = 0;
//     PPAGEFILE PageFile;
//
//     DriveLen = GetLogicalDriveStrings(_countof(szDrives) - 1,
//                                       szDrives);
//
//     while (DriveLen != 0)
//     {
//         Len = lstrlen(DrivePtr) + 1;
//         DriveLen -= Len;
//
//         DrivePtr = _tcsupr(DrivePtr);
//
//         /* Copy the 'X:' portion */
//         lstrcpyn(szDrive, DrivePtr, _countof(szDrive));
//
//         if (GetDriveType(DrivePtr) == DRIVE_FIXED)
//         {
//             MinSize = -1;
//             MaxSize = -1;
//
//             /* Does drive match the one in the registry ? */
//             if (_tcsnicmp(pVirtMem->szPagingFiles, szDrive, 2) == 0)
//             {
//                 GetPageFileSizes(pVirtMem->szPagingFiles,
//                                  &MinSize,
//                                  &MaxSize);
//             }
//
//             PageFile = &pVirtMem->PageFile[PgCnt];
//             PageFile->OldMinSize = MinSize;
//             PageFile->OldMaxSize = MaxSize;
//             PageFile->NewMinSize = MinSize;
//             PageFile->NewMaxSize = MaxSize;
//             PageFile->bIsNotFAT  = TRUE; /* Suppose this is not a FAT volume */
//             PageFile->bUsed = TRUE;
//             lstrcpy(PageFile->szDrive, szDrive);
//
//             /* Get the volume label if there is one */
//             if (GetVolumeInformation(DrivePtr,
//                                      szVolume, _countof(szVolume),
//                                      NULL, NULL, NULL,
//                                      szFSName, _countof(szFSName)))
//             {
//                 PageFile->pszVolume = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//                                                 (_tcslen(szVolume) + 1) * sizeof(TCHAR));
//                 if (PageFile->pszVolume != NULL)
//                     _tcscpy(PageFile->pszVolume, szVolume);
//
//                 /*
//                  * Check whether the volume is FAT, which cannot support files
//                  * larger than 4GB (for FAT32 and FATX, and less for FAT16).
//                  * This will limit the maximum size of the pagefile this volume
//                  * can contain (see GetMaxPageFileSize()).
//                  */
//                 PageFile->bIsNotFAT = (_tcsnicmp(szFSName, _T("FAT"), 3) != 0);
//             }
//
//             PgCnt++;
//         }
//
//         DrivePtr += Len;
//     }
//
//     pVirtMem->Count = PgCnt;
// }
//
//
// static VOID
// WritePageFileSettings(PVIRTMEM pVirtMem)
// {
//     BOOL bErr = TRUE;
//     HKEY hk = NULL;
//     TCHAR szText[256];
//     TCHAR szPagingFiles[2048];
//     UINT i, nPos = 0;
//     PPAGEFILE PageFile;
//
//     for (i = 0; i < pVirtMem->Count; ++i)
//     {
//         PageFile = &pVirtMem->PageFile[i];
//
//         if (PageFile->bUsed &&
//             PageFile->NewMinSize != -1 &&
//             PageFile->NewMaxSize != -1)
//         {
//             _stprintf(szText,
//                       _T("%s\\pagefile.sys %i %i"),
//                       PageFile->szDrive,
//                       PageFile->NewMinSize,
//                       PageFile->NewMaxSize);
//
//             /* Add it to our overall registry string */
//             lstrcpy(szPagingFiles + nPos, szText);
//
//             /* Record the position where the next string will start */
//             nPos += (UINT)lstrlen(szText) + 1;
//
//             /* Add another NULL for REG_MULTI_SZ */
//             szPagingFiles[nPos] = _T('\0');
//             nPos++;
//         }
//     }
//
//     if (RegCreateKeyEx(HKEY_LOCAL_MACHINE,
//                        lpKey,
//                        0,
//                        NULL,
//                        REG_OPTION_NON_VOLATILE,
//                        KEY_WRITE,
//                        NULL,
//                        &hk,
//                        NULL) == ERROR_SUCCESS)
//     {
//         if (RegSetValueEx(hk,
//                           _T("PagingFiles"),
//                           0,
//                           REG_MULTI_SZ,
//                           (LPBYTE)szPagingFiles,
//                           (DWORD)nPos * sizeof(TCHAR)) == ERROR_SUCCESS)
//         {
//             bErr = FALSE;
//         }
//
//         RegCloseKey(hk);
//     }
//
//     if (bErr == FALSE)
//     {
//         /* Delete obsolete paging files on the next boot */
//         for (i = 0; i < _countof(pVirtMem->PageFile); i++)
//         {
//             if (pVirtMem->PageFile[i].OldMinSize != -1 &&
//                 pVirtMem->PageFile[i].NewMinSize == -1)
//             {
//                 _stprintf(szText,
//                           _T("%s\\pagefile.sys"),
//                           pVirtMem->PageFile[i].szDrive);
//
//                 MoveFileEx(szText, NULL, MOVEFILE_DELAY_UNTIL_REBOOT);
//             }
//         }
//     }
//
//     if (bErr)
//         ShowLastWin32Error(pVirtMem->hSelf);
// }
//
//
// static VOID
// SetListBoxColumns(HWND hwndListBox)
// {
//     RECT rect = {0, 0, 103, 0};
//     MapDialogRect(hwndListBox, &rect);
//
//     SendMessage(hwndListBox, LB_SETTABSTOPS, (WPARAM)1, (LPARAM)&rect.right);
// }
//
//
// static VOID
// OnNoPagingFile(PVIRTMEM pVirtMem)
// {
//     /* Disable the page file custom size boxes */
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_INITIALSIZE), FALSE);
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_MAXSIZE), FALSE);
// }
//
//
// static VOID
// OnSysManSize(PVIRTMEM pVirtMem)
// {
//     /* Disable the page file custom size boxes */
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_INITIALSIZE), FALSE);
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_MAXSIZE), FALSE);
// }
//
//
// static VOID
// OnCustom(PVIRTMEM pVirtMem)
// {
//     /* Enable the page file custom size boxes */
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_INITIALSIZE), TRUE);
//     EnableWindow(GetDlgItem(pVirtMem->hSelf, IDC_MAXSIZE), TRUE);
// }
//
//
// static BOOL OnSelChange(PVIRTMEM pVirtMem);
//
// static VOID
// InitPagefileList(PVIRTMEM pVirtMem)
// {
//     INT Index;
//     UINT i;
//     PPAGEFILE PageFile;
//     TCHAR szSize[64];
//     TCHAR szDisplayString[256];
//
//     for (i = 0; i < _countof(pVirtMem->PageFile); i++)
//     {
//         PageFile = &pVirtMem->PageFile[i];
//
//         if (PageFile->bUsed)
//         {
//             if ((PageFile->NewMinSize == -1) &&
//                 (PageFile->NewMaxSize == -1))
//             {
//                 LoadString(hApplet,
//                            IDS_PAGEFILE_NONE,
//                            szSize,
//                            _countof(szSize));
//             }
//             else if ((PageFile->NewMinSize == 0) &&
//                      (PageFile->NewMaxSize == 0))
//             {
//                 LoadString(hApplet,
//                            IDS_PAGEFILE_SYSTEM,
//                            szSize,
//                            _countof(szSize));
//             }
//             else
//             {
//                 _stprintf(szSize, _T("%d - %d"),
//                           PageFile->NewMinSize,
//                           PageFile->NewMaxSize);
//             }
//
//             _stprintf(szDisplayString,
//                       _T("%s  [%s]\t%s"),
//                       PageFile->szDrive,
//                       PageFile->pszVolume ? PageFile->pszVolume : _T(""),
//                       szSize);
//
//             Index = SendMessage(pVirtMem->hListBox, LB_ADDSTRING, (WPARAM)0, (LPARAM)szDisplayString);
//             SendMessage(pVirtMem->hListBox, LB_SETITEMDATA, Index, i);
//         }
//     }
//
//     SendMessage(pVirtMem->hListBox, LB_SETCURSEL, (WPARAM)0, (LPARAM)0);
//
//     OnSelChange(pVirtMem);
// }
//
//
// static VOID
// UpdatePagefileEntry(PVIRTMEM pVirtMem,
//                     INT ListIndex,
//                     INT DriveIndex)
// {
//     PPAGEFILE PageFile = &pVirtMem->PageFile[DriveIndex];
//     TCHAR szSize[64];
//     TCHAR szDisplayString[256];
//
//     if ((PageFile->NewMinSize == -1) &&
//         (PageFile->NewMaxSize == -1))
//     {
//         LoadString(hApplet,
//                    IDS_PAGEFILE_NONE,
//                    szSize,
//                    _countof(szSize));
//     }
//     else if ((PageFile->NewMinSize == 0) &&
//              (PageFile->NewMaxSize == 0))
//     {
//         LoadString(hApplet,
//                    IDS_PAGEFILE_SYSTEM,
//                    szSize,
//                    _countof(szSize));
//     }
//     else
//     {
//         _stprintf(szSize,
//                   _T("%d - %d"),
//                   PageFile->NewMinSize,
//                   PageFile->NewMaxSize);
//     }
//
//     _stprintf(szDisplayString,
//               _T("%s  [%s]\t%s"),
//               PageFile->szDrive,
//               PageFile->pszVolume ? PageFile->pszVolume : _T(""),
//               szSize);
//
//     SendMessage(pVirtMem->hListBox, LB_DELETESTRING, (WPARAM)ListIndex, 0);
//     SendMessage(pVirtMem->hListBox, LB_INSERTSTRING, (WPARAM)ListIndex, (LPARAM)szDisplayString);
//     SendMessage(pVirtMem->hListBox, LB_SETCURSEL, (WPARAM)ListIndex, 0);
// }
//
//
// static VOID
// OnSet(PVIRTMEM pVirtMem)
// {
//     INT Index;
//     UINT MinSize = -1;
//     UINT MaxSize = -1;
//     BOOL bTranslated;
//     UINT DriveIndex;
//     PPAGEFILE PageFile;
//
//     Index = (INT)SendDlgItemMessage(pVirtMem->hSelf,
//                                     IDC_PAGEFILELIST,
//                                     LB_GETCURSEL,
//                                     0,
//                                     0);
//     if (Index >= 0 && Index < pVirtMem->Count)
//     {
//         DriveIndex = SendDlgItemMessage(pVirtMem->hSelf,
//                                         IDC_PAGEFILELIST,
//                                         LB_GETITEMDATA,
//                                         (WPARAM)Index,
//                                         0);
//
//         PageFile = &pVirtMem->PageFile[DriveIndex];
//
//         /* Check if custom settings are checked */
//         if (IsDlgButtonChecked(pVirtMem->hSelf,
//                                IDC_CUSTOM) == BST_CHECKED)
//         {
//             UINT maxPageFileSize;
//
//             MinSize = GetDlgItemInt(pVirtMem->hSelf,
//                                     IDC_INITIALSIZE,
//                                     &bTranslated,
//                                     FALSE);
//             if (!bTranslated)
//             {
//                 ResourceMessageBox(hApplet,
//                                    NULL,
//                                    MB_ICONWARNING | MB_OK,
//                                    IDS_MESSAGEBOXTITLE,
//                                    IDS_WARNINITIALSIZE);
//                 return;
//             }
//
//             MaxSize = GetDlgItemInt(pVirtMem->hSelf,
//                                     IDC_MAXSIZE,
//                                     &bTranslated,
//                                     FALSE);
//             if (!bTranslated)
//             {
//                 ResourceMessageBox(hApplet,
//                                    NULL,
//                                    MB_ICONWARNING | MB_OK,
//                                    IDS_MESSAGEBOXTITLE,
//                                    IDS_WARNMAXIMUMSIZE);
//                 return;
//             }
//
//             maxPageFileSize = GetMaxPageFileSize(PageFile);
//
//             /* Check the valid range of the minimum size */
//             if ((MinSize < MINIMUM_PAGEFILE_SIZE) ||
//                 (MinSize > maxPageFileSize))
//             {
//                 ResourceMessageBox(hApplet,
//                                    NULL,
//                                    MB_ICONWARNING | MB_OK,
//                                    IDS_MESSAGEBOXTITLE,
//                                    IDS_WARNINITIALRANGE,
//                                    maxPageFileSize);
//                 return;
//             }
//
//             /* Check the valid range of the maximum size */
//             if ((MaxSize < MinSize) ||
//                 (MaxSize > maxPageFileSize))
//             {
//                 ResourceMessageBox(hApplet,
//                                    NULL,
//                                    MB_ICONWARNING | MB_OK,
//                                    IDS_MESSAGEBOXTITLE,
//                                    IDS_WARNMAXIMUMRANGE,
//                                    maxPageFileSize);
//                 return;
//             }
//
//             // TODO: Check how much disk space would remain after
//             // storing a pagefile of a certain size. Warn/error out
//             // if less than 5 MB would remain.
//
//             PageFile->NewMinSize = MinSize;
//             PageFile->NewMaxSize = MaxSize;
//             PageFile->bUsed = TRUE;
//         }
//         else if (IsDlgButtonChecked(pVirtMem->hSelf,
//                                     IDC_NOPAGEFILE) == BST_CHECKED)
//         {
//             /* No pagefile */
//             PageFile->NewMinSize = -1;
//             PageFile->NewMaxSize = -1;
//             PageFile->bUsed = TRUE;
//         }
//         else
//         {
//             /* System managed size*/
//             PageFile->NewMinSize = 0;
//             PageFile->NewMaxSize = 0;
//             PageFile->bUsed = TRUE;
//         }
//
//         /* Set the modified flag if min or max size has changed */
//         if ((PageFile->OldMinSize != PageFile->NewMinSize) ||
//             (PageFile->OldMaxSize != PageFile->NewMaxSize))
//         {
//             pVirtMem->bModified = TRUE;
//         }
//
//         UpdatePagefileEntry(pVirtMem, Index, DriveIndex);
//     }
// }
//
//
// static BOOL
// OnSelChange(PVIRTMEM pVirtMem)
// {
//     INT Index;
//     UINT DriveIndex;
//     PPAGEFILE PageFile;
//     MEMORYSTATUSEX MemoryStatus;
//     ULARGE_INTEGER FreeDiskSpace;
//     UINT i, PageFileSizeMb;
//     TCHAR szMegabytes[8];
//     TCHAR szBuffer[64];
//     TCHAR szText[MAX_PATH];
//     WIN32_FIND_DATA fdata = {0};
//     HANDLE hFind;
//     ULARGE_INTEGER pfSize;
//
//     Index = (INT)SendDlgItemMessage(pVirtMem->hSelf,
//                                     IDC_PAGEFILELIST,
//                                     LB_GETCURSEL,
//                                     0,
//                                     0);
//     if (Index >= 0 && Index < pVirtMem->Count)
//     {
//         DriveIndex = SendDlgItemMessage(pVirtMem->hSelf,
//                                         IDC_PAGEFILELIST,
//                                         LB_GETITEMDATA,
//                                         (WPARAM)Index,
//                                         0);
//
//         PageFile = &pVirtMem->PageFile[DriveIndex];
//
//         LoadString(hApplet,
//                    IDS_PAGEFILE_MB,
//                    szMegabytes,
//                    _countof(szMegabytes));
//
//         /* Set drive letter */
//         SetDlgItemText(pVirtMem->hSelf, IDC_DRIVE,
//                        PageFile->szDrive);
//
//         /* Set available disk space */
//         if (GetDiskFreeSpaceEx(PageFile->szDrive,
//                                NULL, NULL, &FreeDiskSpace))
//         {
//             PageFile->FreeSize = (UINT)(FreeDiskSpace.QuadPart / MEGABYTE);
//             _stprintf(szBuffer, szMegabytes, PageFile->FreeSize);
//             SetDlgItemText(pVirtMem->hSelf, IDC_SPACEAVAIL, szBuffer);
//         }
//
//         if (PageFile->NewMinSize == -1 &&
//             PageFile->NewMaxSize == -1)
//         {
//             /* No pagefile */
//             OnNoPagingFile(pVirtMem);
//             CheckDlgButton(pVirtMem->hSelf, IDC_NOPAGEFILE, BST_CHECKED);
//         }
//         else if (PageFile->NewMinSize == 0 &&
//                  PageFile->NewMaxSize == 0)
//         {
//             /* System managed size */
//             OnSysManSize(pVirtMem);
//             CheckDlgButton(pVirtMem->hSelf, IDC_SYSMANSIZE, BST_CHECKED);
//         }
//         else
//         {
//             /* Custom size */
//
//             /* Enable and fill the custom values */
//             OnCustom(pVirtMem);
//
//             SetDlgItemInt(pVirtMem->hSelf,
//                           IDC_INITIALSIZE,
//                           PageFile->NewMinSize,
//                           FALSE);
//
//             SetDlgItemInt(pVirtMem->hSelf,
//                           IDC_MAXSIZE,
//                           PageFile->NewMaxSize,
//                           FALSE);
//
//             CheckDlgButton(pVirtMem->hSelf,
//                            IDC_CUSTOM,
//                            BST_CHECKED);
//         }
//
//         /* Set minimum pagefile size */
//         _stprintf(szBuffer, szMegabytes, MINIMUM_PAGEFILE_SIZE);
//         SetDlgItemText(pVirtMem->hSelf, IDC_MINIMUM, szBuffer);
//
//         /* Set recommended pagefile size */
//         MemoryStatus.dwLength = sizeof(MemoryStatus);
//         if (GlobalMemoryStatusEx(&MemoryStatus))
//         {
//             UINT FreeMemMb, RecoMemMb;
//             UINT maxPageFileSize = GetMaxPageFileSize(PageFile);
//
//             FreeMemMb = (UINT)(MemoryStatus.ullTotalPhys / MEGABYTE);
//             /* The recommended VM size is 150% of free memory */
//             RecoMemMb = FreeMemMb + (FreeMemMb / 2);
//             if (RecoMemMb > maxPageFileSize)
//                 RecoMemMb = maxPageFileSize;
//             _stprintf(szBuffer, szMegabytes, RecoMemMb);
//             SetDlgItemText(pVirtMem->hSelf, IDC_RECOMMENDED, szBuffer);
//         }
//
//         /* Set current pagefile size */
//         PageFileSizeMb = 0;
//
//         for (i = 0; i < pVirtMem->Count; i++)
//         {
//             _stprintf(szText,
//                       _T("%c:\\pagefile.sys"),
//                       pVirtMem->PageFile[i].szDrive[0]);
//
//             hFind = FindFirstFile(szText, &fdata);
//             if (hFind == INVALID_HANDLE_VALUE)
//             {
//                 // FIXME: MsgBox error?
//                 DPRINT1("Unable to read PageFile size: %ls due to error %d\n",
//                         szText, GetLastError());
//             }
//             else
//             {
//                 pfSize.LowPart = fdata.nFileSizeLow;
//                 pfSize.HighPart = fdata.nFileSizeHigh;
//                 PageFileSizeMb += pfSize.QuadPart / MEGABYTE;
//                 FindClose(hFind);
//             }
//         }
//
//         _stprintf(szBuffer, szMegabytes, PageFileSizeMb);
//         SetDlgItemText(pVirtMem->hSelf, IDC_CURRENT, szBuffer);
//     }
//
//     return TRUE;
// }
//
//
// static VOID
// OnVirtMemDialogOk(PVIRTMEM pVirtMem)
// {
//     if (pVirtMem->bModified != FALSE)
//     {
//         ResourceMessageBox(hApplet,
//                            NULL,
//                            MB_ICONINFORMATION | MB_OK,
//                            IDS_MESSAGEBOXTITLE,
//                            IDS_INFOREBOOT);
//
//         WritePageFileSettings(pVirtMem);
//     }
// }
//
//
// static VOID
// OnInitVirtMemDialog(HWND hwnd, PVIRTMEM pVirtMem)
// {
//     UINT i;
//     PPAGEFILE PageFile;
//
//     pVirtMem->hSelf = hwnd;
//     pVirtMem->hListBox = GetDlgItem(hwnd, IDC_PAGEFILELIST);
//     pVirtMem->bModified = FALSE;
//
//     SetListBoxColumns(pVirtMem->hListBox);
//
//     for (i = 0; i < _countof(pVirtMem->PageFile); i++)
//     {
//         PageFile = &pVirtMem->PageFile[i];
//         PageFile->bUsed = FALSE;
//         PageFile->OldMinSize = -1;
//         PageFile->OldMaxSize = -1;
//         PageFile->NewMinSize = -1;
//         PageFile->NewMaxSize = -1;
//         PageFile->FreeSize   = 0;
//         PageFile->bIsNotFAT  = TRUE; /* Suppose this is not a FAT volume */
//     }
//
//     /* Load the pagefile systems from the reg */
//     ReadPageFileSettings(pVirtMem);
//
//     /* Parse our settings and set up dialog */
//     ParseMemSettings(pVirtMem);
//
//     InitPagefileList(pVirtMem);
// }
//
//
// static VOID
// OnDestroy(PVIRTMEM pVirtMem)
// {
//     UINT i;
//
//     for (i = 0; i < _countof(pVirtMem->PageFile); i++)
//     {
//         if (pVirtMem->PageFile[i].pszVolume != NULL)
//             HeapFree(GetProcessHeap(), 0, pVirtMem->PageFile[i].pszVolume);
//     }
//
//     if (pVirtMem->szPagingFiles)
//         HeapFree(GetProcessHeap(), 0, pVirtMem->szPagingFiles);
//
//     HeapFree(GetProcessHeap(), 0, pVirtMem);
// }
//
//
// INT_PTR __stdcall
// VirtMemDlgProc(HWND hwndDlg,
//                UINT uMsg,
//                WPARAM wParam,
//                LPARAM lParam)
// {
//     PVIRTMEM pVirtMem;
//
//     UNREFERENCED_PARAMETER(lParam);
//
//     pVirtMem = (PVIRTMEM)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             pVirtMem = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(VIRTMEM));
//             if (pVirtMem == NULL)
//             {
//                 EndDialog(hwndDlg, 0);
//                 return FALSE;
//             }
//
//             SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pVirtMem);
//
//             OnInitVirtMemDialog(hwndDlg, pVirtMem);
//             break;
//         }
//
//         case WM_DESTROY:
//             OnDestroy(pVirtMem);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, 0);
//                     return TRUE;
//
//                 case IDOK:
//                     OnVirtMemDialogOk(pVirtMem);
//                     EndDialog(hwndDlg, pVirtMem->bModified);
//                     return TRUE;
//
//                 case IDC_NOPAGEFILE:
//                     OnNoPagingFile(pVirtMem);
//                     return TRUE;
//
//                 case IDC_SYSMANSIZE:
//                     OnSysManSize(pVirtMem);
//                     return TRUE;
//
//                 case IDC_CUSTOM:
//                     OnCustom(pVirtMem);
//                     return TRUE;
//
//                 case IDC_SET:
//                     OnSet(pVirtMem);
//                     return TRUE;
//
//                 case IDC_PAGEFILELIST:
//                     switch (HIWORD(wParam))
//                     {
//                         case LBN_SELCHANGE:
//                             OnSelChange(pVirtMem);
//                             return TRUE;
//                     }
//                     break;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
