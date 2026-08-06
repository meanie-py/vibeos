// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const PROFILE_NAME_LENGTH: usize = 80;

#[repr(C)]
pub struct _PROFILE {
    pub szFriendlyName: [u16; PROFILE_NAME_LENGTH],
    pub szName: [u16; 5],
    pub dwProfileNumber: u32,
    pub dwPreferenceOrder: u32,
}
pub type PROFILE = _PROFILE;
pub type PPROFILE = *mut _PROFILE;

#[repr(C)]
pub struct _PROFILEDATA {
    pub dwProfileCount: u32,
    pub dwLastProfile: u32,
    pub dwSelectedProfile: u32,
    pub dwSelectedProfileIndex: u32,
    pub pProfiles: PPROFILE,
}
pub type PROFILEDATA = _PROFILEDATA;
pub type PPROFILEDATA = *mut _PROFILEDATA;

#[repr(C)]
pub struct _PROFILENAMES {
    pub szSourceName: [u16; PROFILE_NAME_LENGTH],
    pub szDestinationName: [u16; PROFILE_NAME_LENGTH],
    pub pProfileData: PPROFILEDATA,
}
pub type PROFILENAMES = _PROFILENAMES;
pub type PPROFILENAMES = *mut _PROFILENAMES;
unsafe extern "C" fn UpdateButtons(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
) {
    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFPROP), if (*pProfileData).dwSelectedProfileIndex != (-1 as i32) as DWORD { TRUE } else { FALSE });
    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFCOPY), if (*pProfileData).dwSelectedProfileIndex != (-1 as i32) as DWORD { TRUE } else { FALSE });
    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFRENAME), if (*pProfileData).dwSelectedProfileIndex != (-1 as i32) as DWORD { TRUE } else { FALSE });
    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDEL), if (*pProfileData).dwSelectedProfileIndex != (-1 as i32) as DWORD { TRUE } else { FALSE });

    if (*pProfileData).dwProfileCount < 2 {
        EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFUP), FALSE);
        EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDWN), FALSE);
    } else {
        EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFUP),
                     if (*pProfileData).dwSelectedProfileIndex > 0 { TRUE } else { FALSE });
        EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDWN),
                     if (*pProfileData).dwSelectedProfileIndex < (*pProfileData).dwProfileCount - 1 { TRUE } else { FALSE });
    }
}
unsafe extern "C" fn IsProfileNameInUse(
    pProfileNames: *mut PROFILENAMES,
    bIgnoreCurrent: BOOL,
) -> BOOL {
    let mut i: DWORD;

    i = 0;
    while i < (*(*pProfileNames).pProfileData).dwProfileCount {
        if bIgnoreCurrent == TRUE && i == (*(*pProfileNames).pProfileData).dwSelectedProfileIndex {
            i = i.wrapping_add(1);
            continue;
        }

        if wcscmp(
            (*(*(*pProfileNames).pProfileData).pProfiles.offset(i as isize)).szFriendlyName.as_ptr(),
            (*pProfileNames).szDestinationName.as_ptr(),
        ) == 0
        {
            return TRUE;
        }

        i = i.wrapping_add(1);
    }

    return FALSE;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn CopyProfileDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pProfileNames: *mut PROFILENAMES;

    pProfileNames = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut PROFILENAMES;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            let pProfileNames = lParam as *mut PROFILENAMES;

            SetDlgItemText(hwndDlg, IDC_COPYPROFILEFROM, (*pProfileNames).szSourceName.as_ptr());

            SendDlgItemMessageW(hwndDlg, IDC_COPYPROFILETO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
            SetDlgItemText(hwndDlg, IDC_COPYPROFILETO, (*pProfileNames).szDestinationName.as_ptr());
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK => {
                    GetDlgItemText(hwndDlg,
                                   IDC_COPYPROFILETO,
                                   (*pProfileNames).szDestinationName.as_mut_ptr(),
                                   PROFILE_NAME_LENGTH);
                    if IsProfileNameInUse(pProfileNames, FALSE) != 0 {
                        ResourceMessageBox(hApplet,
                                           hwndDlg,
                                           MB_OK | MB_ICONERROR,
                                           IDS_HWPROFILE_WARNING,
                                           IDS_HWPROFILE_ALREADY_IN_USE);
                    } else {
                        EndDialog(hwndDlg, IDOK);
                    }
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn CopyProfileDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pProfileNames: *mut PROFILENAMES;

    pProfileNames = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut PROFILENAMES;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            let pProfileNames = lParam as *mut PROFILENAMES;

            SetDlgItemText(hwndDlg, IDC_COPYPROFILEFROM, (*pProfileNames).szSourceName.as_ptr());

            SendDlgItemMessageW(hwndDlg, IDC_COPYPROFILETO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
            SetDlgItemText(hwndDlg, IDC_COPYPROFILETO, (*pProfileNames).szDestinationName.as_ptr());
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK => {
                    GetDlgItemText(hwndDlg,
                                   IDC_COPYPROFILETO,
                                   (*pProfileNames).szDestinationName.as_mut_ptr(),
                                   PROFILE_NAME_LENGTH);
                    if IsProfileNameInUse(pProfileNames, FALSE) != 0 {
                        ResourceMessageBox(hApplet,
                                           hwndDlg,
                                           MB_OK | MB_ICONERROR,
                                           IDS_HWPROFILE_WARNING,
                                           IDS_HWPROFILE_ALREADY_IN_USE);
                    } else {
                        EndDialog(hwndDlg, IDOK);
                    }
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}
unsafe extern "C" fn CopyHardwareProfile(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
) {
    let mut ProfileNames: PROFILENAMES = core::mem::zeroed();
    let mut pProfile: *mut PROFILE;
    let mut pNewProfiles: *mut PROFILE;
    let mut pNewProfile: *mut PROFILE;
    let mut szBuffer: [u16; 80] = core::mem::zeroed();

    pProfile = &mut (*pProfileData).pProfiles.offset((*pProfileData).dwSelectedProfileIndex as isize);

    LoadStringW(hApplet, IDS_HWPROFILE_PROFILE, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as u32);

    wcscpy(ProfileNames.szSourceName.as_mut_ptr(), (*pProfile).szFriendlyName.as_ptr());
    swprintf(ProfileNames.szDestinationName.as_mut_ptr(), b"%s %lu\0".as_ptr() as *const u16, szBuffer.as_ptr(), (*pProfileData).dwProfileCount);

    ProfileNames.pProfileData = pProfileData;

    if DialogBoxParam(hApplet,
                       MAKEINTRESOURCE(IDD_COPYPROFILE),
                       hwndDlg,
                       CopyProfileDlgProc,
                       &ProfileNames as *const _ as LPARAM) != IDOK {
        return;
    }

    if wcscmp(ProfileNames.szSourceName.as_ptr(), ProfileNames.szDestinationName.as_ptr()) == 0 {
        return;
    }

    pNewProfiles = HeapReAlloc(GetProcessHeap(),
                               HEAP_ZERO_MEMORY,
                               (*pProfileData).pProfiles as *mut _,
                               ((*pProfileData).dwProfileCount + 1) as usize * core::mem::size_of::<PROFILE>()) as *mut PROFILE;
    if pNewProfiles.is_null() {
        // DPRINT removed
        return;
    }

    (*pProfileData).dwProfileCount += 1;
    (*pProfileData).pProfiles = pNewProfiles;

    pNewProfile = &mut (*pProfileData).pProfiles.offset(((*pProfileData).dwProfileCount - 1) as isize);

    CopyMemory(pNewProfile as *mut _ as *mut _, pProfile as *const _ as *const _, core::mem::size_of::<PROFILE>());

    wcscpy((*pNewProfile).szFriendlyName.as_mut_ptr(), ProfileNames.szDestinationName.as_ptr());

    (*pProfileData).dwLastProfile += 1;
    (*pNewProfile).dwProfileNumber = (*pProfileData).dwLastProfile;
    swprintf((*pNewProfile).szName.as_mut_ptr(), b"%04lu\0".as_ptr() as *const u16, (*pNewProfile).dwProfileNumber);

    (*pNewProfile).dwPreferenceOrder = (*pNewProfile).dwProfileNumber;

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_ADDSTRING, 0, (*pNewProfile).szFriendlyName.as_ptr() as LPARAM);

    UpdateButtons(hwndDlg, pProfileData);
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn RenameProfileDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pProfileNames: *mut PROFILENAMES;

    pProfileNames = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut PROFILENAMES;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            pProfileNames = lParam as *mut PROFILENAMES;

            SetDlgItemText(hwndDlg, IDC_RENPROFEDITFROM, (*pProfileNames).szSourceName.as_ptr());

            SendDlgItemMessageW(hwndDlg, IDC_RENPROFEDITTO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
            SetDlgItemText(hwndDlg, IDC_RENPROFEDITTO, (*pProfileNames).szDestinationName.as_ptr());
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK => {
                    GetDlgItemText(hwndDlg,
                                   IDC_RENPROFEDITTO,
                                   (*pProfileNames).szDestinationName.as_mut_ptr(),
                                   PROFILE_NAME_LENGTH);
                    if IsProfileNameInUse(pProfileNames, TRUE) != 0 {
                        ResourceMessageBox(hApplet,
                                           hwndDlg,
                                           MB_OK | MB_ICONERROR,
                                           IDS_HWPROFILE_WARNING,
                                           IDS_HWPROFILE_ALREADY_IN_USE);
                    } else {
                        EndDialog(hwndDlg, IDOK);
                    }
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn RenameProfileDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pProfileNames: *mut PROFILENAMES;

    pProfileNames = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut PROFILENAMES;

    match uMsg {
        WM_INITDIALOG => {
            SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
            pProfileNames = lParam as *mut PROFILENAMES;

            SetDlgItemText(hwndDlg, IDC_RENPROFEDITFROM, (*pProfileNames).szSourceName.as_ptr());

            SendDlgItemMessageW(hwndDlg, IDC_RENPROFEDITTO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
            SetDlgItemText(hwndDlg, IDC_RENPROFEDITTO, (*pProfileNames).szDestinationName.as_ptr());
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDOK => {
                    GetDlgItemText(hwndDlg,
                                   IDC_RENPROFEDITTO,
                                   (*pProfileNames).szDestinationName.as_mut_ptr(),
                                   PROFILE_NAME_LENGTH);
                    if IsProfileNameInUse(pProfileNames, TRUE) != 0 {
                        ResourceMessageBox(hApplet,
                                           hwndDlg,
                                           MB_OK | MB_ICONERROR,
                                           IDS_HWPROFILE_WARNING,
                                           IDS_HWPROFILE_ALREADY_IN_USE);
                    } else {
                        EndDialog(hwndDlg, IDOK);
                    }
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, IDCANCEL);
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}
unsafe extern "C" fn RenameHardwareProfile(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
) {
    let mut ProfileNames: PROFILENAMES = core::mem::zeroed();
    let mut szBuffer: [u16; 80] = core::mem::zeroed();

    let pProfile = &mut (*(*pProfileData).pProfiles.offset((*pProfileData).dwSelectedProfileIndex as isize));

    LoadStringW(hApplet, IDS_HWPROFILE_PROFILE, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32);

    wcscpy(ProfileNames.szSourceName.as_mut_ptr(), pProfile.szFriendlyName.as_ptr());
    swprintf(ProfileNames.szDestinationName.as_mut_ptr(), b"%s %lu\0".as_ptr() as *const u16, szBuffer.as_ptr(), (*pProfileData).dwProfileCount);

    ProfileNames.pProfileData = pProfileData;

    if DialogBoxParam(hApplet,
                       MAKEINTRESOURCE(IDD_RENAMEPROFILE),
                       hwndDlg,
                       RenameProfileDlgProc,
                       core::ptr::addr_of_mut!(ProfileNames) as LPARAM) != IDOK
    {
        return;
    }

    if wcscmp(pProfile.szFriendlyName.as_ptr(), ProfileNames.szDestinationName.as_ptr()) == 0 {
        return;
    }

    wcscpy(pProfile.szFriendlyName.as_mut_ptr(), ProfileNames.szDestinationName.as_ptr());

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_DELETESTRING, (*pProfileData).dwSelectedProfileIndex as WPARAM, 0);
    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_INSERTSTRING, (*pProfileData).dwSelectedProfileIndex as WPARAM, pProfile.szFriendlyName.as_ptr() as LPARAM);
}
unsafe extern "C" fn DeleteHardwareProfile(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
) {
    let pProfiles: *mut PROFILE;
    let pProfile: *mut PROFILE;

    pProfile = &mut (*pProfileData).pProfiles.offset((*pProfileData).dwSelectedProfileIndex as isize);

    if ResourceMessageBox(hApplet,
                           hwndDlg,
                           MB_YESNO | MB_ICONQUESTION,
                           IDS_HWPROFILE_CONFIRM_DELETE_TITLE,
                           IDS_HWPROFILE_CONFIRM_DELETE,
                           (*pProfile).szFriendlyName.as_ptr()) != IDYES
    {
        return;
    }

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_DELETESTRING, (*pProfileData).dwSelectedProfileIndex, 0);

    if (*pProfileData).dwSelectedProfileIndex != (*pProfileData).dwProfileCount - 1
    {
        core::ptr::copy(
            (*pProfileData).pProfiles.offset(((*pProfileData).dwSelectedProfileIndex + 1) as isize),
            (*pProfileData).pProfiles.offset((*pProfileData).dwSelectedProfileIndex as isize),
            (((*pProfileData).dwProfileCount - (*pProfileData).dwSelectedProfileIndex - 1) * core::mem::size_of::<PROFILE>()) as usize / core::mem::size_of::<PROFILE>()
        );
    }
    else
    {
        (*pProfileData).dwSelectedProfileIndex = (*pProfileData).dwSelectedProfileIndex.wrapping_sub(1);
    }

    pProfiles = HeapReAlloc(GetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            (*pProfileData).pProfiles as *mut core::ffi::c_void,
                            ((*pProfileData).dwProfileCount - 1) * core::mem::size_of::<PROFILE>()) as *mut PROFILE;
    if pProfiles == core::ptr::null_mut()
    {
        // DPRINT removed
        return;
    }

    (*pProfileData).dwProfileCount = (*pProfileData).dwProfileCount.wrapping_sub(1);
    (*pProfileData).pProfiles = pProfiles;

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_SETCURSEL, (*pProfileData).dwSelectedProfileIndex, 0);

    UpdateButtons(hwndDlg, pProfileData);
}
unsafe extern "C" fn MoveHardwareProfile(
    hwndDlg: HWND,
    pProfileData: *mut PROFILEDATA,
    bMoveUp: u8,
) {
    let mut TempProfile: PROFILE = core::mem::zeroed();
    let pSrcProfile: *mut PROFILE;
    let pDstProfile: *mut PROFILE;
    let dwSrcIndex: u32;
    let dwDstIndex: u32;

    dwSrcIndex = (*pProfileData).dwSelectedProfileIndex;
    dwDstIndex = if bMoveUp != 0 {
        dwSrcIndex - 1
    } else {
        dwSrcIndex + 1
    };

    pSrcProfile = &mut (*pProfileData).pProfiles.offset(dwSrcIndex as isize);
    pDstProfile = &mut (*pProfileData).pProfiles.offset(dwDstIndex as isize);

    CopyMemory(
        &mut TempProfile as *mut _ as *mut u8,
        pDstProfile as *const _ as *const u8,
        core::mem::size_of::<PROFILE>(),
    );
    CopyMemory(
        pDstProfile as *mut u8,
        pSrcProfile as *const _ as *const u8,
        core::mem::size_of::<PROFILE>(),
    );
    CopyMemory(
        pSrcProfile as *mut u8,
        &TempProfile as *const _ as *const u8,
        core::mem::size_of::<PROFILE>(),
    );

    SendDlgItemMessageW(
        hwndDlg,
        IDC_HRDPROFLSTBOX,
        LB_DELETESTRING,
        dwSrcIndex as WPARAM,
        0,
    );
    SendDlgItemMessageW(
        hwndDlg,
        IDC_HRDPROFLSTBOX,
        LB_INSERTSTRING,
        dwDstIndex as WPARAM,
        (*pDstProfile).szFriendlyName.as_ptr() as LPARAM,
    );

    (*pProfileData).dwSelectedProfileIndex = dwDstIndex;
    SendDlgItemMessageW(
        hwndDlg,
        IDC_HRDPROFLSTBOX,
        LB_SETCURSEL,
        (*pProfileData).dwSelectedProfileIndex as WPARAM,
        0,
    );

    UpdateButtons(hwndDlg, pProfileData);
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn HardwareProfilePropertiesDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER(hwndDlg);
    UNREFERENCED_PARAMETER(lParam);
    UNREFERENCED_PARAMETER(wParam);

    match uMsg {
        WM_INITDIALOG => {
            return TRUE;
        }
        _ => {}
    }

    return FALSE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn HardwareProfilePropertiesDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    UNREFERENCED_PARAMETER(hwndDlg);
    UNREFERENCED_PARAMETER(lParam);
    UNREFERENCED_PARAMETER(wParam);

    match uMsg {
        WM_INITDIALOG => {
            return TRUE;
        }
        _ => {}
    }

    return FALSE;
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDI_HARDPROF));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn PropSheetProc(hwndDlg: HWND, uMsg: u32, lParam: LPARAM) -> i32 {
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDI_HARDPROF));
            SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, hIcon as LPARAM);
        }
        _ => {}
    }
    0
}
unsafe extern "C" fn GetUserWaitInterval() -> DWORD {
    let mut dwWaitInterval: DWORD = 30;
    let mut dwSize: DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                      b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0I\0D\0C\0o\0n\0f\0i\0g\0D\0B\0\0\0".as_ptr() as *const u16,
                      0,
                      KEY_QUERY_VALUE,
                      &mut hKey) != 0
    {
        return dwWaitInterval;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    RegQueryValueExW(hKey,
                     b"U\0s\0e\0r\0W\0a\0i\0t\0I\0n\0t\0e\0r\0v\0a\0l\0\0\0".as_ptr() as *const u16,
                     core::ptr::null_mut(),
                     core::ptr::null_mut(),
                     &mut dwWaitInterval as *mut DWORD as LPBYTE,
                     &mut dwSize);

    RegCloseKey(hKey);

    return dwWaitInterval;
}
unsafe extern "C" fn SetUserWaitInterval(dwWaitInterval: DWORD) {
    let mut hKey: HKEY = core::ptr::null_mut();

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0I\0D\0C\0o\0n\0f\0i\0g\0D\0B\0\0\0".as_ptr() as LPWSTR,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    ) != 0
    {
        return;
    }

    RegSetValueExW(
        hKey,
        b"U\0s\0e\0r\0W\0a\0i\0t\0I\0n\0t\0e\0r\0v\0a\0l\0\0\0".as_ptr() as LPWSTR,
        0,
        REG_DWORD,
        (&dwWaitInterval as *const DWORD) as LPBYTE,
        core::mem::size_of::<DWORD>() as u32,
    );

    RegCloseKey(hKey);
}
unsafe extern "C" fn GetProfileCount(lpProfileCount: LPDWORD) -> i32 {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: i32;

    *lpProfileCount = 0;

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0I\0D\0C\0o\0n\0f\0i\0g\0D\0B\0\\\0H\0a\0r\0d\0w\0a\0r\0e\0 \0P\0r\0o\0f\0i\0l\0e\0s\0\0\0".as_ptr() as LPWSTR,
        0,
        KEY_READ,
        &mut hKey,
    );
    if lError != ERROR_SUCCESS {
        return FALSE;
    }

    lError = RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        lpProfileCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    RegCloseKey(hKey);

    if lError != ERROR_SUCCESS {
        return FALSE;
    }

    return TRUE;
}
unsafe extern "C" fn GetProfile(
    hwndDlg: HWND,
    hKey: HKEY,
    lpName: LPWSTR,
    dwProfileNumber: DWORD,
    pProfile: PPROFILE,
) {
    let mut hProfileKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut lError: LONG;

    lError = RegOpenKeyExW(hKey,
                           lpName,
                           0,
                           KEY_READ,
                           &mut hProfileKey);
    if lError != ERROR_SUCCESS {
        return;
    }

    dwSize = (PROFILE_NAME_LENGTH * core::mem::size_of::<WCHAR>()) as DWORD;
    lError = RegQueryValueExW(hProfileKey,
                              w!("FriendlyName").as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::null_mut(),
                              (*pProfile).szFriendlyName.as_mut_ptr() as LPBYTE,
                              &mut dwSize);
    if lError == ERROR_SUCCESS {
        // DPRINT removed
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    lError = RegQueryValueExW(hProfileKey,
                              w!("PreferenceOrder").as_ptr(),
                              core::ptr::null_mut(),
                              core::ptr::null_mut(),
                              core::ptr::addr_of_mut!((*pProfile).dwPreferenceOrder) as LPBYTE,
                              &mut dwSize);
    if lError == ERROR_SUCCESS {
        // DPRINT removed
    }

    (*pProfile).dwProfileNumber = dwProfileNumber;

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_ADDSTRING, 0, (*pProfile).szFriendlyName.as_ptr() as LPARAM);

    RegCloseKey(hProfileKey);
}
unsafe extern "C" fn GetProfiles(hwndDlg: HWND) -> i32 {
    let mut pProfileData: PPROFILEDATA;
    let mut szName: [u16; 8] = [0; 8];
    let mut dwNameLength: DWORD;
    let mut dwProfileNumber: DWORD;
    let mut dwIndex: DWORD = 0;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: i32;

    pProfileData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<PROFILEDATA>()) as PPROFILEDATA;
    if pProfileData.is_null() {
        return FALSE;
    }

    (*pProfileData).dwLastProfile = (-1) as DWORD;
    (*pProfileData).dwSelectedProfileIndex = (-1) as DWORD;

    if GetProfileCount(core::ptr::addr_of_mut!((*pProfileData).dwProfileCount)) == 0 {
        HeapFree(GetProcessHeap(), 0, pProfileData as *mut _);
        return FALSE;
    }

    (*pProfileData).pProfiles = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
                                        (*pProfileData).dwProfileCount as usize * core::mem::size_of::<PROFILE>()) as PPROFILE;
    if (*pProfileData).pProfiles.is_null() {
        HeapFree(GetProcessHeap(), 0, pProfileData as *mut _);
        return FALSE;
    }

    SetWindowLongPtr(hwndDlg, DWLP_USER, pProfileData as isize);

    if RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                      b"S\0y\0s\0t\0e\0m\0\\\0C\0u\0r\0r\0e\0n\0t\0C\0o\0n\0t\0r\0o\0l\0S\0e\0t\0\\\0C\0o\0n\0t\0r\0o\0l\0\\\0I\0D\0C\0o\0n\0f\0i\0g\0D\0B\0\\\0H\0a\0r\0d\0w\0a\0r\0e\0 \0P\0r\0o\0f\0i\0l\0e\0s\0\0\0".as_ptr() as LPWSTR,
                      0,
                      KEY_READ,
                      core::ptr::addr_of_mut!(hKey)) != ERROR_SUCCESS {
        return FALSE;
    }

    dwIndex = 0;
    while dwIndex < (*pProfileData).dwProfileCount {
        dwNameLength = 8;
        lError = RegEnumKeyExW(hKey,
                               dwIndex,
                               szName.as_mut_ptr(),
                               core::ptr::addr_of_mut!(dwNameLength),
                               core::ptr::null_mut(),
                               core::ptr::null_mut(),
                               core::ptr::null_mut(),
                               core::ptr::null_mut());
        if lError != ERROR_SUCCESS {
            break;
        }

        dwProfileNumber = wcstoul(szName.as_ptr(), core::ptr::null_mut(), 10);
        // DPRINT removed
        // DPRINT removed

        if ((*pProfileData).dwLastProfile == (-1) as DWORD) ||
            ((*pProfileData).dwLastProfile < dwProfileNumber) {
            (*pProfileData).dwLastProfile = dwProfileNumber;
        }

        // DPRINT removed

        GetProfile(hwndDlg, hKey, szName.as_mut_ptr(), dwProfileNumber, core::ptr::addr_of_mut!((*(*pProfileData).pProfiles.offset(dwIndex as isize))));

        dwIndex += 1;
    }

    RegCloseKey(hKey);

    return TRUE;
}
unsafe extern "C" fn OnInitHardProfDialog(hwndDlg: HWND) -> i32 {
    let mut dwWaitInterval: DWORD;

    // DPRINT removed

    SendMessage(GetDlgItem(hwndDlg, IDC_HRDPROFUP),
                BM_SETIMAGE, IMAGE_ICON as WPARAM,
                LoadIcon(hApplet, MAKEINTRESOURCE(IDI_UP)) as LPARAM);
    SendMessage(GetDlgItem(hwndDlg, IDC_HRDPROFDWN),
                BM_SETIMAGE, IMAGE_ICON as WPARAM,
                LoadIcon(hApplet, MAKEINTRESOURCE(IDI_DOWN)) as LPARAM);

    if GetProfiles(hwndDlg) == 0 {
        return FALSE;
    }

    SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETRANGE, 0 as WPARAM, MAKELONG(500 as i16, 0) as LPARAM);

    dwWaitInterval = GetUserWaitInterval();
    if dwWaitInterval == 1.wrapping_neg() as DWORD {
        CheckDlgButton(hwndDlg, IDC_HRDPROFWAIT, BST_CHECKED);
        SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETPOS, 0, 30);
        EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), FALSE);
    } else {
        CheckDlgButton(hwndDlg, IDC_HRDPROFSELECT, BST_CHECKED);
        SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETPOS, 0, dwWaitInterval);
    }

    TRUE
}
unsafe extern "C" fn OnOk(hwndDlg: HWND) {
    let dwWaitInterval: DWORD;

    if IsDlgButtonChecked(hwndDlg, IDC_HRDPROFWAIT) == BST_CHECKED {
        dwWaitInterval = (-1) as DWORD;
    } else {
        dwWaitInterval = LOWORD(SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_GETPOS, 0, 0));
    }

    SetUserWaitInterval(dwWaitInterval);
}
#[no_mangle]
pub unsafe extern "system" fn HardProfDlgProc(
    hwndDlg: HWND,
    uMsg: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let pProfileData: *mut PROFILEDATA;

    UNREFERENCED_PARAMETER(lParam);
    UNREFERENCED_PARAMETER(wParam);
    UNREFERENCED_PARAMETER(hwndDlg);

    pProfileData = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut PROFILEDATA;

    match uMsg {
        WM_INITDIALOG => {
            return OnInitHardProfDialog(hwndDlg) as INT_PTR;
        }

        WM_DESTROY => {
            if !pProfileData.is_null() {
                if !(*pProfileData).pProfiles.is_null() {
                    HeapFree(GetProcessHeap(), 0, (*pProfileData).pProfiles as *mut _);
                }
                HeapFree(GetProcessHeap(), 0, pProfileData as *mut _);
            }
        }

        WM_COMMAND => {
            match LOWORD(wParam) {
                IDC_HRDPROFPROP => {
                    HardwareProfileProperties(hwndDlg, pProfileData);
                }

                IDC_HRDPROFCOPY => {
                    CopyHardwareProfile(hwndDlg, pProfileData);
                }

                IDC_HRDPROFRENAME => {
                    RenameHardwareProfile(hwndDlg, pProfileData);
                }

                IDC_HRDPROFDEL => {
                    DeleteHardwareProfile(hwndDlg, pProfileData);
                }

                IDC_HRDPROFUP => {
                    MoveHardwareProfile(hwndDlg, pProfileData, TRUE);
                }

                IDC_HRDPROFDWN => {
                    MoveHardwareProfile(hwndDlg, pProfileData, FALSE);
                }

                IDC_HRDPROFWAIT => {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), FALSE);
                    return TRUE;
                }

                IDC_HRDPROFSELECT => {
                    EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), TRUE);
                    return TRUE;
                }

                IDC_HRDPROFLSTBOX => {
                    if HIWORD(wParam) == LBN_SELCHANGE {
                        (*pProfileData).dwSelectedProfileIndex = SendDlgItemMessage(hwndDlg, IDC_HRDPROFLSTBOX, LB_GETCURSEL, 0, 0) as u32;
                        UpdateButtons(hwndDlg, pProfileData);
                    }
                    return TRUE;
                }

                IDOK => {
                    OnOk(hwndDlg);
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }

                IDCANCEL => {
                    EndDialog(hwndDlg, LOWORD(wParam));
                    return TRUE;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS System Control Panel Applet
//  * LICENSE:     GPL - See COPYING in the top level directory
//  * FILE:        dll/cpl/sysdm/hardprof.c
//  * PURPOSE:     Modify hardware profiles
//  * COPYRIGHT:   Copyright 2006 Ged Murphy <gedmurphy@gmail.com>
//  *
//  */
//
// #include "precomp.h"
//
// #include <debug.h>
//
// #define PROFILE_NAME_LENGTH 80
//
// typedef struct _PROFILE
// {
//     WCHAR szFriendlyName[PROFILE_NAME_LENGTH];
//     WCHAR szName[5];
//     DWORD dwProfileNumber;
//     DWORD dwPreferenceOrder;
// } PROFILE, *PPROFILE;
//
// typedef struct _PROFILEDATA
// {
//     DWORD dwProfileCount;
//     DWORD dwLastProfile;
//     DWORD dwSelectedProfile;
//     DWORD dwSelectedProfileIndex;
//     PPROFILE pProfiles;
// } PROFILEDATA, *PPROFILEDATA;
//
// typedef struct _PROFILENAMES
// {
//     WCHAR szSourceName[PROFILE_NAME_LENGTH];
//     WCHAR szDestinationName[PROFILE_NAME_LENGTH];
//     PPROFILEDATA pProfileData;
// } PROFILENAMES, *PPROFILENAMES;
//
//
// static
// VOID
// UpdateButtons(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData)
// {
//     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFPROP), (pProfileData->dwSelectedProfileIndex != (DWORD)-1) ? TRUE : FALSE);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFCOPY), (pProfileData->dwSelectedProfileIndex != (DWORD)-1) ? TRUE : FALSE);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFRENAME), (pProfileData->dwSelectedProfileIndex != (DWORD)-1) ? TRUE : FALSE);
//     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDEL), (pProfileData->dwSelectedProfileIndex != (DWORD)-1) ? TRUE : FALSE);
//
//     if (pProfileData->dwProfileCount < 2)
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFUP), FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDWN), FALSE);
//     }
//     else
//     {
//         EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFUP),
//                      (pProfileData->dwSelectedProfileIndex > 0) ? TRUE : FALSE);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFDWN),
//                      (pProfileData->dwSelectedProfileIndex < pProfileData->dwProfileCount - 1) ? TRUE : FALSE);
//     }
// }
//
//
// static
// BOOL
// IsProfileNameInUse(
//     PPROFILENAMES pProfileNames,
//     BOOL bIgnoreCurrent)
// {
//     DWORD i;
//
//     for (i = 0; i < pProfileNames->pProfileData->dwProfileCount; i++)
//     {
//         if (bIgnoreCurrent == TRUE && i == pProfileNames->pProfileData->dwSelectedProfileIndex)
//             continue;
//
//         if (wcscmp(pProfileNames->pProfileData->pProfiles[i].szFriendlyName, pProfileNames->szDestinationName) == 0)
//             return TRUE;
//     }
//
//     return FALSE;
// }
//
//
// static
// INT_PTR
// __stdcall
// CopyProfileDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PPROFILENAMES pProfileNames;
//
//     pProfileNames = (PPROFILENAMES)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
//             pProfileNames = (PPROFILENAMES)lParam;
//
//             /* Set the old name */
//             SetDlgItemText(hwndDlg, IDC_COPYPROFILEFROM, pProfileNames->szSourceName);
//
//             /* Set the new name */
//             SendDlgItemMessageW(hwndDlg, IDC_COPYPROFILETO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
//             SetDlgItemText(hwndDlg, IDC_COPYPROFILETO, pProfileNames->szDestinationName);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                     GetDlgItemText(hwndDlg,
//                                    IDC_COPYPROFILETO,
//                                    pProfileNames->szDestinationName,
//                                    PROFILE_NAME_LENGTH);
//                     if (IsProfileNameInUse(pProfileNames, FALSE))
//                     {
//                         ResourceMessageBox(hApplet,
//                                            hwndDlg,
//                                            MB_OK | MB_ICONERROR,
//                                            IDS_HWPROFILE_WARNING,
//                                            IDS_HWPROFILE_ALREADY_IN_USE);
//                     }
//                     else
//                     {
//                         EndDialog(hwndDlg, IDOK);
//                     }
//                     return TRUE;
//
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, IDCANCEL);
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static
// VOID
// CopyHardwareProfile(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData)
// {
//     PROFILENAMES ProfileNames;
//     PPROFILE pProfile, pNewProfiles, pNewProfile;
//     WCHAR szBuffer[80];
//
//     pProfile = &pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex];
//
//     LoadStringW(hApplet, IDS_HWPROFILE_PROFILE, szBuffer, sizeof(szBuffer) / sizeof(WCHAR));
//
//     wcscpy(ProfileNames.szSourceName, pProfile->szFriendlyName);
//     swprintf(ProfileNames.szDestinationName, L"%s %lu", szBuffer, pProfileData->dwProfileCount);
//
//     ProfileNames.pProfileData = pProfileData;
//
//     if (DialogBoxParam(hApplet,
//                        MAKEINTRESOURCE(IDD_COPYPROFILE),
//                        hwndDlg,
//                        CopyProfileDlgProc,
//                        (LPARAM)&ProfileNames) != IDOK)
//         return;
//
//     /* Apply new name only if it has been changed */
//     if (wcscmp(ProfileNames.szSourceName, ProfileNames.szDestinationName) == 0)
//         return;
//
//     /* Allocate memory for the new profile */
//     pNewProfiles = HeapReAlloc(GetProcessHeap(),
//                                HEAP_ZERO_MEMORY,
//                                pProfileData->pProfiles,
//                                (pProfileData->dwProfileCount + 1) * sizeof(PROFILE));
//     if (pNewProfiles == NULL)
//     {
//         DPRINT1("HeapReAlloc() failed!\n");
//         return;
//     }
//
//     pProfileData->dwProfileCount++;
//     pProfileData->pProfiles = pNewProfiles;
//
//     pNewProfile = &pProfileData->pProfiles[pProfileData->dwProfileCount - 1];
//
//     CopyMemory(pNewProfile, pProfile, sizeof(PROFILE));
//
//     wcscpy(pNewProfile->szFriendlyName, ProfileNames.szDestinationName);
//
//     pNewProfile->dwProfileNumber = ++pProfileData->dwLastProfile;
//     swprintf(pNewProfile->szName, L"%04lu", pNewProfile->dwProfileNumber);
//
//     pNewProfile->dwPreferenceOrder = pNewProfile->dwProfileNumber;
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_ADDSTRING, 0, (LPARAM)pNewProfile->szFriendlyName);
//
//     UpdateButtons(hwndDlg, pProfileData);
// }
//
//
// static
// INT_PTR
// __stdcall
// RenameProfileDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     PPROFILENAMES pProfileNames;
//
//     pProfileNames = (PPROFILENAMES)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             SetWindowLongPtr(hwndDlg, DWLP_USER, lParam);
//             pProfileNames = (PPROFILENAMES)lParam;
//
//             /* Set the old name */
//             SetDlgItemText(hwndDlg, IDC_RENPROFEDITFROM, pProfileNames->szSourceName);
//
//             /* Set the new name */
//             SendDlgItemMessageW(hwndDlg, IDC_RENPROFEDITTO, EM_SETLIMITTEXT, PROFILE_NAME_LENGTH - 1, 0);
//             SetDlgItemText(hwndDlg, IDC_RENPROFEDITTO, pProfileNames->szDestinationName);
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDOK:
//                     GetDlgItemText(hwndDlg,
//                                    IDC_RENPROFEDITTO,
//                                    pProfileNames->szDestinationName,
//                                    PROFILE_NAME_LENGTH);
//                     if (IsProfileNameInUse(pProfileNames, TRUE))
//                     {
//                         ResourceMessageBox(hApplet,
//                                            hwndDlg,
//                                            MB_OK | MB_ICONERROR,
//                                            IDS_HWPROFILE_WARNING,
//                                            IDS_HWPROFILE_ALREADY_IN_USE);
//                     }
//                     else
//                     {
//                         EndDialog(hwndDlg, IDOK);
//                     }
//                     return TRUE;
//
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, IDCANCEL);
//                     return TRUE;
//             }
//             break;
//     }
//
//     return FALSE;
// }
//
//
// static
// VOID
// RenameHardwareProfile(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData)
// {
//     PROFILENAMES ProfileNames;
//     PPROFILE pProfile;
//     WCHAR szBuffer[80];
//
//     pProfile = &pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex];
//
//     LoadStringW(hApplet, IDS_HWPROFILE_PROFILE, szBuffer, sizeof(szBuffer) / sizeof(WCHAR));
//
//     wcscpy(ProfileNames.szSourceName, pProfile->szFriendlyName);
//     swprintf(ProfileNames.szDestinationName, L"%s %lu", szBuffer, pProfileData->dwProfileCount);
//
//     ProfileNames.pProfileData = pProfileData;
//
//     if (DialogBoxParam(hApplet,
//                        MAKEINTRESOURCE(IDD_RENAMEPROFILE),
//                        hwndDlg,
//                        RenameProfileDlgProc,
//                        (LPARAM)&ProfileNames) != IDOK)
//         return;
//
//     /* Apply new name only if it has been changed */
//     if (wcscmp(pProfile->szFriendlyName, ProfileNames.szDestinationName) == 0)
//         return;
//
//     /* Replace the profile name in the profile list */
//     wcscpy(pProfile->szFriendlyName, ProfileNames.szDestinationName);
//
//     /* Replace the profile name in the listbox */
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_DELETESTRING, pProfileData->dwSelectedProfileIndex, 0);
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_INSERTSTRING, pProfileData->dwSelectedProfileIndex, (LPARAM)pProfile->szFriendlyName);
// }
//
//
// static
// VOID
// DeleteHardwareProfile(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData)
// {
//     PPROFILE pProfiles;
//     PPROFILE pProfile;
//
//     pProfile = &pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex];
//
//     if (ResourceMessageBox(hApplet,
//                            hwndDlg,
//                            MB_YESNO | MB_ICONQUESTION,
//                            IDS_HWPROFILE_CONFIRM_DELETE_TITLE,
//                            IDS_HWPROFILE_CONFIRM_DELETE,
//                            pProfile->szFriendlyName) != IDYES)
//     {
//         return;
//     }
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_DELETESTRING, pProfileData->dwSelectedProfileIndex, 0);
//
//     if (pProfileData->dwSelectedProfileIndex != pProfileData->dwProfileCount - 1)
//     {
//         RtlMoveMemory(&pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex],
//                       &pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex + 1],
//                       (pProfileData->dwProfileCount - pProfileData->dwSelectedProfileIndex - 1) * sizeof(PROFILE));
//     }
//     else
//     {
//         pProfileData->dwSelectedProfileIndex--;
//     }
//
//     pProfiles = HeapReAlloc(GetProcessHeap(),
//                             HEAP_ZERO_MEMORY,
//                             pProfileData->pProfiles,
//                             (pProfileData->dwProfileCount - 1) * sizeof(PROFILE));
//     if (pProfiles == NULL)
//     {
//         DPRINT1("HeapReAlloc() failed!\n");
//         return;
//     }
//
//     pProfileData->dwProfileCount--;
//     pProfileData->pProfiles = pProfiles;
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_SETCURSEL, pProfileData->dwSelectedProfileIndex, 0);
//
//     UpdateButtons(hwndDlg, pProfileData);
// }
//
//
// static
// VOID
// MoveHardwareProfile(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData,
//     BOOL bMoveUp)
// {
//     PROFILE TempProfile;
//     PPROFILE pSrcProfile, pDstProfile;
//     DWORD dwSrcIndex, dwDstIndex;
//
//     dwSrcIndex = pProfileData->dwSelectedProfileIndex;
//     dwDstIndex = bMoveUp ? (dwSrcIndex - 1) : (dwSrcIndex + 1);
//
//     pSrcProfile = &pProfileData->pProfiles[dwSrcIndex];
//     pDstProfile = &pProfileData->pProfiles[dwDstIndex];
//
//     CopyMemory(&TempProfile, pDstProfile, sizeof(PROFILE));
//     CopyMemory(pDstProfile, pSrcProfile, sizeof(PROFILE));
//     CopyMemory(pSrcProfile, &TempProfile, sizeof(PROFILE));
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_DELETESTRING, dwSrcIndex, 0);
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_INSERTSTRING, dwDstIndex, (LPARAM)pDstProfile->szFriendlyName);
//
//     pProfileData->dwSelectedProfileIndex = dwDstIndex;
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_SETCURSEL, pProfileData->dwSelectedProfileIndex, 0);
//
//     UpdateButtons(hwndDlg, pProfileData);
// }
//
//
// static
// INT_PTR
// __stdcall
// HardwareProfilePropertiesDlgProc(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     UNREFERENCED_PARAMETER(hwndDlg);
//     UNREFERENCED_PARAMETER(lParam);
//     UNREFERENCED_PARAMETER(wParam);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return TRUE;
//
//     }
//
//     return FALSE;
// }
//
// static int __stdcall
// PropSheetProc(HWND hwndDlg, UINT uMsg, LPARAM lParam)
// {
//     // NOTE: This callback is needed to set large icon correctly.
//     HICON hIcon;
//     switch (uMsg)
//     {
//         case PSCB_INITIALIZED:
//         {
//             hIcon = LoadIconW(hApplet, MAKEINTRESOURCEW(IDI_HARDPROF));
//             SendMessageW(hwndDlg, WM_SETICON, ICON_BIG, (LPARAM)hIcon);
//             break;
//         }
//     }
//     return 0;
// }
//
// static
// VOID
// HardwareProfileProperties(
//     HWND hwndDlg,
//     PPROFILEDATA pProfileData)
// {
//     HPROPSHEETPAGE hpsp;
//     PROPSHEETHEADER psh;
//     PROPSHEETPAGE psp;
//
//     ZeroMemory(&psp, sizeof(psp));
//     psp.dwSize = sizeof(psp);
//     psp.dwFlags = PSP_DEFAULT;
//     psp.hInstance = hApplet;
//     psp.pszTemplate = MAKEINTRESOURCE(IDD_HARDWAREPROFILE);
//     psp.pfnDlgProc = HardwareProfilePropertiesDlgProc;
//
//     hpsp = CreatePropertySheetPage(&psp);
//     if (hpsp == NULL)
//     {
//         return;
//     }
//
//     ZeroMemory(&psh, sizeof(PROPSHEETHEADER));
//     psh.dwSize = sizeof(PROPSHEETHEADER);
//     psh.dwFlags = PSH_PROPTITLE | PSH_USEICONID | PSH_USECALLBACK;
//     psh.hwndParent = hwndDlg;
//     psh.hInstance = hApplet;
//     psh.pszIcon = MAKEINTRESOURCEW(IDI_HARDPROF);
//     psh.pszCaption = pProfileData->pProfiles[pProfileData->dwSelectedProfileIndex].szFriendlyName;
//     psh.nPages = 1;
//     psh.nStartPage = 0;
//     psh.phpage = &hpsp;
//     psh.pfnCallback = PropSheetProc;
//
//     PropertySheet(&psh);
// }
//
//
// static
// DWORD
// GetUserWaitInterval(VOID)
// {
//     DWORD dwWaitInterval = 30;
//     DWORD dwSize;
//     HKEY hKey;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"System\\CurrentControlSet\\Control\\IDConfigDB",
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hKey))
//         return dwWaitInterval;
//
//     dwSize = sizeof(DWORD);
//     RegQueryValueExW(hKey,
//                      L"UserWaitInterval",
//                      NULL,
//                      NULL,
//                      (LPBYTE)&dwWaitInterval,
//                      &dwSize);
//
//     RegCloseKey(hKey);
//
//     return dwWaitInterval;
// }
//
//
// static
// VOID
// SetUserWaitInterval(DWORD dwWaitInterval)
// {
//     HKEY hKey;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"System\\CurrentControlSet\\Control\\IDConfigDB",
//                       0,
//                       KEY_SET_VALUE,
//                       &hKey))
//         return;
//
//     RegSetValueExW(hKey,
//                    L"UserWaitInterval",
//                    0,
//                    REG_DWORD,
//                    (LPBYTE)&dwWaitInterval,
//                    sizeof(DWORD));
//
//     RegCloseKey(hKey);
// }
//
//
// static
// BOOL
// GetProfileCount(LPDWORD lpProfileCount)
// {
//     HKEY hKey;
//     LONG lError;
//
//     *lpProfileCount = 0;
//
//     lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                            L"System\\CurrentControlSet\\Control\\IDConfigDB\\Hardware Profiles",
//                            0,
//                            KEY_READ,
//                            &hKey);
//     if (lError != ERROR_SUCCESS)
//         return FALSE;
//
//     lError = RegQueryInfoKeyW(hKey, NULL, NULL, NULL, lpProfileCount,
//                               NULL, NULL, NULL, NULL, NULL, NULL, NULL);
//
//     RegCloseKey(hKey);
//
//     if (lError != ERROR_SUCCESS)
//         return FALSE;
//
//     return TRUE;
// }
//
//
// static
// VOID
// GetProfile(
//     HWND hwndDlg,
//     HKEY hKey,
//     LPWSTR lpName,
//     DWORD dwProfileNumber,
//     PPROFILE pProfile)
// {
//     HKEY hProfileKey;
//     DWORD dwSize;
//     LONG lError;
//
//     lError = RegOpenKeyExW(hKey,
//                            lpName,
//                            0,
//                            KEY_READ,
//                            &hProfileKey);
//     if (lError != ERROR_SUCCESS)
//         return;
//
//     dwSize = PROFILE_NAME_LENGTH * sizeof(WCHAR);
//     lError = RegQueryValueExW(hProfileKey,
//                               L"FriendlyName",
//                               NULL,
//                               NULL,
//                               (LPBYTE)pProfile->szFriendlyName,
//                               &dwSize);
//     if (lError == ERROR_SUCCESS)
//     {
//         DPRINT1("Profile: %S\n", pProfile->szFriendlyName);
//     }
//
//     dwSize = sizeof(DWORD);
//     lError = RegQueryValueExW(hProfileKey,
//                               L"PreferenceOrder",
//                               NULL,
//                               NULL,
//                               (LPBYTE)&pProfile->dwPreferenceOrder,
//                               &dwSize);
//     if (lError == ERROR_SUCCESS)
//     {
//         DPRINT1("PreferenceOrder: %lu\n", pProfile->dwPreferenceOrder);
//     }
//
//     pProfile->dwProfileNumber = dwProfileNumber;
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFLSTBOX, LB_ADDSTRING, 0, (LPARAM)pProfile->szFriendlyName);
//
//     RegCloseKey(hProfileKey);
// }
//
//
// static
// BOOL
// GetProfiles(HWND hwndDlg)
// {
//     PPROFILEDATA pProfileData;
//     WCHAR szName[8];
//     DWORD dwNameLength;
//     DWORD dwProfileNumber;
//     DWORD dwIndex = 0;
//     HKEY hKey;
//     LONG lError;
//
//     pProfileData = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(PROFILEDATA));
//     if (pProfileData == NULL)
//         return FALSE;
//
//     pProfileData->dwLastProfile = (DWORD)-1;
//     pProfileData->dwSelectedProfileIndex = (DWORD)-1;
//
//     if (!GetProfileCount(&pProfileData->dwProfileCount))
//     {
//         HeapFree(GetProcessHeap(), 0, pProfileData);
//         return FALSE;
//     }
//
//     pProfileData->pProfiles = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY,
//                                         pProfileData->dwProfileCount * sizeof(PROFILE));
//     if (pProfileData->pProfiles == NULL)
//     {
//         HeapFree(GetProcessHeap(), 0, pProfileData);
//         return FALSE;
//     }
//
//     SetWindowLongPtr(hwndDlg, DWLP_USER, (LONG_PTR)pProfileData);
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       L"System\\CurrentControlSet\\Control\\IDConfigDB\\Hardware Profiles",
//                       0,
//                       KEY_READ,
//                       &hKey) != ERROR_SUCCESS)
//         return FALSE;
//
//     for (dwIndex = 0; dwIndex < pProfileData->dwProfileCount; dwIndex++)
//     {
//         dwNameLength = 8;
//         lError = RegEnumKeyExW(hKey,
//                                dwIndex,
//                                szName,
//                                &dwNameLength,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//         if (lError != ERROR_SUCCESS)
//             break;
//
//         dwProfileNumber = wcstoul(szName, NULL, 10);
//         DPRINT("Profile name: %S\n", szName);
//         DPRINT("Profile number: %lu\n", dwProfileNumber);
//
//         if ((pProfileData->dwLastProfile == (DWORD)-1) ||
//             (pProfileData->dwLastProfile < dwProfileNumber))
//             pProfileData->dwLastProfile = dwProfileNumber;
//
//         DPRINT("Last Profile number: %lu\n", pProfileData->dwLastProfile);
//
//         GetProfile(hwndDlg, hKey, szName, dwProfileNumber, &pProfileData->pProfiles[dwIndex]);
//     }
//
//     RegCloseKey(hKey);
//
//     return TRUE;
// }
//
//
// static
// BOOL
// OnInitHardProfDialog(HWND hwndDlg)
// {
//     DWORD dwWaitInterval;
//
//     DPRINT("OnInitHardProfDialog()\n");
//
//     SendMessage(GetDlgItem(hwndDlg, IDC_HRDPROFUP),
//                 BM_SETIMAGE,(WPARAM)IMAGE_ICON,
//                 (LPARAM)(HANDLE)LoadIcon(hApplet, MAKEINTRESOURCE(IDI_UP)));
//     SendMessage(GetDlgItem(hwndDlg, IDC_HRDPROFDWN),
//                 BM_SETIMAGE,(WPARAM)IMAGE_ICON,
//                 (LPARAM)(HANDLE)LoadIcon(hApplet, MAKEINTRESOURCE(IDI_DOWN)));
//
//     if (!GetProfiles(hwndDlg))
//         return FALSE;
//
//     SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETRANGE, (WPARAM)0, (LPARAM)MAKELONG((SHORT)500, 0));
//
//     dwWaitInterval = GetUserWaitInterval();
//     if (dwWaitInterval == (DWORD)-1)
//     {
//         CheckDlgButton(hwndDlg, IDC_HRDPROFWAIT, BST_CHECKED);
//         SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETPOS, 0, 30);
//         EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), FALSE);
//     }
//     else
//     {
//         CheckDlgButton(hwndDlg, IDC_HRDPROFSELECT, BST_CHECKED);
//         SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_SETPOS, 0, dwWaitInterval);
//     }
//
//     return TRUE;
// }
//
//
// static
// VOID
// OnOk(HWND hwndDlg)
// {
//     DWORD dwWaitInterval;
//
//     if (IsDlgButtonChecked(hwndDlg, IDC_HRDPROFWAIT) == BST_CHECKED)
//     {
//         dwWaitInterval = (DWORD)-1;
//     }
//     else
//     {
//         dwWaitInterval = LOWORD(SendDlgItemMessageW(hwndDlg, IDC_HRDPROFUPDWN, UDM_GETPOS, 0, 0));
//     }
//
//     SetUserWaitInterval(dwWaitInterval);
// }
//
//
// /* Property page dialog callback */
// INT_PTR
// __stdcall
// HardProfDlgProc(HWND hwndDlg,
//                 UINT uMsg,
//                 WPARAM wParam,
//                 LPARAM lParam)
// {
//     PPROFILEDATA pProfileData;
//
//     UNREFERENCED_PARAMETER(lParam);
//     UNREFERENCED_PARAMETER(wParam);
//     UNREFERENCED_PARAMETER(hwndDlg);
//
//     pProfileData = (PPROFILEDATA)GetWindowLongPtr(hwndDlg, DWLP_USER);
//
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             return OnInitHardProfDialog(hwndDlg);
//
//         case WM_DESTROY:
//             if (pProfileData != NULL)
//             {
//                 if (pProfileData->pProfiles != NULL)
//                     HeapFree(GetProcessHeap(), 0, pProfileData->pProfiles);
//                 HeapFree(GetProcessHeap(), 0, pProfileData);
//             }
//             break;
//
//         case WM_COMMAND:
//             switch (LOWORD(wParam))
//             {
//                 case IDC_HRDPROFPROP:
//                     HardwareProfileProperties(hwndDlg, pProfileData);
//                     break;
//
//                 case IDC_HRDPROFCOPY:
//                     CopyHardwareProfile(hwndDlg, pProfileData);
//                     break;
//
//                 case IDC_HRDPROFRENAME:
//                     RenameHardwareProfile(hwndDlg, pProfileData);
//                     break;
//
//                 case IDC_HRDPROFDEL:
//                     DeleteHardwareProfile(hwndDlg, pProfileData);
//                     break;
//
//                 case IDC_HRDPROFUP:
//                     MoveHardwareProfile(hwndDlg, pProfileData, TRUE);
//                     break;
//
//                 case IDC_HRDPROFDWN:
//                     MoveHardwareProfile(hwndDlg, pProfileData, FALSE);
//                     break;
//
//                 case IDC_HRDPROFWAIT:
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), FALSE);
//                     return TRUE;
//
//                 case IDC_HRDPROFSELECT:
//                     EnableWindow(GetDlgItem(hwndDlg, IDC_HRDPROFEDIT), TRUE);
//                     return TRUE;
//
//                 case IDC_HRDPROFLSTBOX:
//                     if (HIWORD(wParam) == LBN_SELCHANGE)
//                     {
//                         pProfileData->dwSelectedProfileIndex = (DWORD)SendDlgItemMessage(hwndDlg, IDC_HRDPROFLSTBOX, LB_GETCURSEL, 0, 0);
//                         UpdateButtons(hwndDlg, pProfileData);
//                     }
//                     return TRUE;
//
//                 case IDOK:
//                     OnOk(hwndDlg);
//
//                 case IDCANCEL:
//                     EndDialog(hwndDlg, LOWORD(wParam));
//                     return TRUE;
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
