//! `dll/win32/shell32/dialogs/recycler_prop.cpp`, translated and FLIPPED — the C is commented
//! out of the CMake source list and this module is what shell32 links. Island-first, like its
//! `general.rs` sibling. A PAID artifact — do not delete it to tidy up.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_4BF412C806D7: [WCHAR; 78] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0042, 0x0069, 0x0074, 0x0042, 0x0075, 0x0063, 0x006B, 0x0065, 0x0074, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x005C, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];
pub static CPP_COM_WSTR_604114ADC8B4: [WCHAR; 13] = [0x004E, 0x0075, 0x006B, 0x0065, 0x004F, 0x006E, 0x0044, 0x0065, 0x006C, 0x0065, 0x0074, 0x0065, 0x0000];
pub static CPP_COM_WSTR_6C7249CE2D15: [WCHAR; 68] = [0x0053, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D, 0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0042, 0x0069, 0x0074, 0x0042, 0x0075, 0x0063, 0x006B, 0x0065, 0x0074, 0x005C, 0x0056, 0x006F, 0x006C, 0x0075, 0x006D, 0x0065, 0x0000];
pub static CPP_COM_WSTR_7F8732C542BA: [WCHAR; 4] = [0x0041, 0x003A, 0x005C, 0x0000];
pub static CPP_COM_WSTR_9618E75FEC6D: [WCHAR; 10] = [0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];
pub static CPP_COM_WSTR_9AEB29CF1D78: [WCHAR; 9] = [0x0025, 0x0073, 0x0020, 0x0028, 0x0025, 0x0063, 0x003A, 0x0029, 0x0000];
pub static CPP_COM_WSTR_C7A5956868FE: [WCHAR; 12] = [0x004D, 0x0061, 0x0078, 0x0043, 0x0061, 0x0070, 0x0061, 0x0063, 0x0069, 0x0074, 0x0079, 0x0000];

use core::ffi::c_void;
use crate::prelude::*;

// ---- file-scope preamble (translated) ----

/// `enum { WM_NEWDRIVESELECTED = WM_APP, WM_UPDATEDRIVESETTINGS };` — a FUNCTION-LOCAL enum in
/// recycler_prop.cpp:282, dropped by the translation (it survives only as a comment further
/// down) and hoisted to module scope here.
///
/// Restoring it is not cosmetic. Both names are `case` labels, and in Rust an unresolved
/// identifier in PATTERN position is not an error — it is a fresh irrefutable BINDING that
/// matches EVERY message, collapsing the whole dialog proc into its first arm (field manual
/// §9.112, the class that shipped a BUGCHECK 0x1E). `rustconv.profile_derive`'s switch-constant
/// audit reports these as SWITCH_CONST_UNOWNED because no header owns them — a TU-local enum
/// cannot come from the island, so a module-scope const is the correct home. Same treatment as
/// `WM_UPDATEDIRSTATS` in dialogs/filedefext.rs, whose C enum is identical in shape.
pub const WM_APP: DWORD = 32768; // winuser.h:1924 `#define WM_APP 32768`
pub const WM_NEWDRIVESELECTED: DWORD = WM_APP;
pub const WM_UPDATEDRIVESETTINGS: DWORD = WM_APP + 1;

#[repr(C)]
pub struct DRIVE_ITEM_CONTEXT {
    pub FreeBytesAvailable: _ULARGE_INTEGER,
    pub dwSerial: u32,
    pub dwMaxCapacity: u32,
    pub dwNukeOnDelete: u32,
}
pub type PDRIVE_ITEM_CONTEXT = *mut DRIVE_ITEM_CONTEXT;
unsafe extern "C" fn toggleNukeOnDeleteOption(hwndDlg: HWND, bEnable: BOOL) {
    if bEnable != 0 {
        SendDlgItemMessage(hwndDlg, 14001, BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
        EnableWindow(GetDlgItem(hwndDlg, 14002), (FALSE) as i32);
        SendDlgItemMessage(hwndDlg, 14003, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
    } else {
        SendDlgItemMessage(hwndDlg, 14001, BM_SETCHECK, BST_CHECKED as WPARAM, 0);
        EnableWindow(GetDlgItem(hwndDlg, 14002), (TRUE) as i32);
        SendDlgItemMessage(hwndDlg, 14003, BM_SETCHECK, BST_UNCHECKED as WPARAM, 0);
    }

    // FIXME: Max capacity not implemented yet, disable for now (CORE-13743)
    EnableWindow(GetDlgItem(hwndDlg, 14002), (FALSE) as i32);
}
unsafe extern "C" fn InitializeRecycleBinDlg(hwndDlg: HWND, DefaultDrive: u16) {
    let szDrive_static: *const u16 = CPP_COM_WSTR_7F8732C542BA.as_ptr();
    let mut szDrive: [u16; 4] = [0; 4];
    core::ptr::copy_nonoverlapping(szDrive_static, szDrive.as_mut_ptr(), 4);

    let mut dwDrives: DWORD;
    let mut szName: [u16; 100 as usize] = [0; 100 as usize];
    let mut szVolume: [u16; 100 as usize] = [0; 100 as usize];
    let mut MaxComponent: DWORD = core::mem::zeroed();
    let mut Flags: DWORD = core::mem::zeroed();
    let mut dwSerial: DWORD = core::mem::zeroed();
    let mut lc: LVCOLUMNW = core::mem::zeroed();
    let mut hDlgCtrl: HWND;
    let mut li: LVITEMW = core::mem::zeroed();
    let mut itemCount: INT;
    let mut TotalNumberOfFreeBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut TotalNumberOfBytes: ULARGE_INTEGER = core::mem::zeroed();
    let mut FreeBytesAvailable: ULARGE_INTEGER = core::mem::zeroed();
    let mut rect: RECT = core::mem::zeroed();
    let mut columnSize: i32;
    let mut defIndex: i32 = 0;
    let mut dwSize: DWORD;
    let mut pItem: *mut DRIVE_ITEM_CONTEXT = core::ptr::null_mut();
    let mut pDefault: *mut DRIVE_ITEM_CONTEXT = core::ptr::null_mut();
    let mut pFirst: *mut DRIVE_ITEM_CONTEXT = core::ptr::null_mut();

    hDlgCtrl = GetDlgItem(hwndDlg, 14000);

    if LoadStringW(shell32_hInstance, IDS_RECYCLEBIN_LOCATION, szVolume.as_mut_ptr(), (_countof!(&szVolume)) as i32) == 0 {
        szVolume[0 as usize] = 0;
    }

    GetClientRect(hDlgCtrl, core::ptr::addr_of_mut!(rect));

    ZeroMemory(core::ptr::addr_of_mut!(lc), (core::mem::size_of_val(&lc) as DWORD) as usize);
    lc.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;

    columnSize = 140;
    lc.iSubItem = 0;
    lc.fmt = (LVCFMT_FIXED_WIDTH) as i32;
    lc.cx = columnSize;
    lc.cchTextMax = (wcslen(szVolume.as_ptr())) as i32;
    lc.pszText = szVolume.as_mut_ptr();
    let _ = SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, 0, core::ptr::addr_of!(lc) as LPARAM);

    if LoadStringW(shell32_hInstance, IDS_RECYCLEBIN_DISKSPACE, szVolume.as_mut_ptr(), (_countof!(&szVolume)) as i32) == 0 {
        szVolume[0 as usize] = 0;
    }

    lc.iSubItem = 1;
    lc.cx = rect.right - rect.left - columnSize;
    lc.cchTextMax = (wcslen(szVolume.as_ptr())) as i32;
    lc.pszText = szVolume.as_mut_ptr();
    let _ = SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, 1, core::ptr::addr_of!(lc) as LPARAM);

    dwDrives = GetLogicalDrives();
    itemCount = 0;
    loop {
        if (dwDrives & 0x1) != 0 {
            let Type = GetDriveTypeW(szDrive.as_ptr());
            if Type == DRIVE_FIXED {
                if GetVolumeInformationW(
                    szDrive.as_ptr(),
                    szName.as_mut_ptr(),
                    _countof!(&szName),
                    core::ptr::addr_of_mut!(dwSerial),
                    core::ptr::addr_of_mut!(MaxComponent),
                    core::ptr::addr_of_mut!(Flags),
                    core::ptr::null_mut(),
                    0,
                ) == 0
                {
                    szName[0 as usize] = 0;
                    dwSerial = (-(1 as i32)) as DWORD;
                }

                swprintf(
                    szVolume.as_mut_ptr(),
                    CPP_COM_WSTR_9AEB29CF1D78.as_ptr(),
                    szName.as_ptr(),
                    szDrive[0 as usize] as ::core::ffi::c_int,
                );
                ZeroMemory(core::ptr::addr_of_mut!(li), (core::mem::size_of_val(&li) as DWORD) as usize);
                li.mask = LVIF_TEXT | LVIF_PARAM;
                li.iSubItem = 0;
                li.pszText = szVolume.as_mut_ptr();
                li.iItem = itemCount;
                SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, core::ptr::addr_of!(li) as LPARAM);
                if GetDiskFreeSpaceExW(
                    szDrive.as_ptr(),
                    core::ptr::addr_of_mut!(FreeBytesAvailable),
                    core::ptr::addr_of_mut!(TotalNumberOfBytes),
                    core::ptr::addr_of_mut!(TotalNumberOfFreeBytes),
                ) != 0
                {
                    if !StrFormatByteSizeW((TotalNumberOfFreeBytes.QuadPart) as i64, szVolume.as_mut_ptr(), _countof!(&szVolume)).is_null() {
                        pItem = HeapAlloc(
                            GetProcessHeap(),
                            HEAP_ZERO_MEMORY,
                            core::mem::size_of::<DRIVE_ITEM_CONTEXT>() as SIZE_T,
                        ) as *mut DRIVE_ITEM_CONTEXT;
                        if pItem.is_null() == false {
                            (*pItem).FreeBytesAvailable = FreeBytesAvailable;
                            (*pItem).dwSerial = dwSerial;

                            swprintf(
                                szName.as_mut_ptr(),
                                CPP_COM_WSTR_4BF412C806D7.as_ptr(),
                                LOWORD(dwSerial) as ::core::ffi::c_int,
                                HIWORD(dwSerial) as ::core::ffi::c_int,
                            );

                            dwSize = core::mem::size_of::<DWORD>() as DWORD;
                            if RegGetValueW(
                                HKEY_CURRENT_USER,
                                szName.as_ptr(),
                                CPP_COM_WSTR_C7A5956868FE.as_ptr(),
                                RRF_RT_DWORD,
                                core::ptr::null_mut(),
                                (core::ptr::addr_of_mut!((*pItem).dwMaxCapacity)).cast(),
                                core::ptr::addr_of_mut!(dwSize),
                            ) != 0
                            {
                                // failed; leave default
                            } else {
                                (*pItem).dwMaxCapacity = !0;
                            }

                            FreeBytesAvailable.QuadPart =
                                (FreeBytesAvailable.QuadPart / (1024 * 1024));
                            (*pItem).dwMaxCapacity = min(
                                (*pItem).dwMaxCapacity,
                                FreeBytesAvailable.__bindgen_anon_1.LowPart,
                            );

                            dwSize = core::mem::size_of::<DWORD>() as DWORD;
                            RegGetValueW(
                                HKEY_CURRENT_USER,
                                szName.as_ptr(),
                                CPP_COM_WSTR_604114ADC8B4.as_ptr(),
                                RRF_RT_DWORD,
                                core::ptr::null_mut(),
                                (core::ptr::addr_of_mut!((*pItem).dwNukeOnDelete)).cast(),
                                core::ptr::addr_of_mut!(dwSize),
                            );

                            li.mask = LVIF_PARAM;
                            li.lParam = pItem as LPARAM;
                            let _ = SendMessageW(
                                hDlgCtrl,
                                LVM_SETITEMW,
                                0,
                                core::ptr::addr_of!(li) as LPARAM,
                            );
                            if szDrive[0 as usize] == DefaultDrive {
                                defIndex = itemCount;
                                pDefault = pItem;
                            }
                        }
                        if pFirst.is_null() {
                            pFirst = pItem;
                        }

                        li.mask = LVIF_TEXT;
                        li.iSubItem = 1;
                        li.pszText = szVolume.as_mut_ptr();
                        li.iItem = itemCount;
                        let _ = SendMessageW(
                            hDlgCtrl,
                            LVM_SETITEMW,
                            0,
                            core::ptr::addr_of!(li) as LPARAM,
                        );
                    }
                }
                itemCount += 1;
            }
        }
        szDrive[0 as usize] = szDrive[0 as usize] + 1;
        dwDrives = dwDrives >> 1;
        if dwDrives == 0 {
            break;
        }
    }

    if pDefault.is_null() {
        pDefault = pFirst;
    }
    if !pDefault.is_null() {
        toggleNukeOnDeleteOption(hwndDlg, ((*pDefault).dwNukeOnDelete) as i32);
        SetDlgItemInt(
            hwndDlg,
            14002,
            (*pDefault).dwMaxCapacity,
            (FALSE) as i32,
        );
    }
    ZeroMemory(core::ptr::addr_of_mut!(li), (core::mem::size_of_val(&li) as DWORD) as usize);
    li.mask = LVIF_STATE;
    li.stateMask = (-(1 as i32)) as u32;
    li.state = LVIS_FOCUSED | LVIS_SELECTED;
    li.iItem = defIndex;
    let _ = SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of!(li) as LPARAM);
}
unsafe extern "C" fn StoreDriveSettings(hwndDlg: HWND) -> BOOL {
    let mut iCount: i32;
    let mut iIndex: i32;
    let hDlgCtrl: HWND = GetDlgItem(hwndDlg, 14000);
    let mut li: LVITEMW = core::mem::zeroed();
    let mut pItem: *mut DRIVE_ITEM_CONTEXT;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hSubKey: HKEY = core::ptr::null_mut();
    let mut szSerial: [u16; 20 as usize] = [0u16; 20 as usize];
    let mut dwSize: DWORD;

    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        CPP_COM_WSTR_6C7249CE2D15.as_ptr(),
        0,
        core::ptr::null_mut(),
        0,
        KEY_WRITE,
        core::ptr::null_mut(),
        &mut hKey,
        core::ptr::null_mut(),
    ) != (ERROR_SUCCESS) as i32
    {
        return (FALSE) as i32;
    }

    iCount = ListView_GetItemCount(hDlgCtrl);

    ZeroMemory(
        core::ptr::addr_of_mut!(li).cast::<core::ffi::c_void>(),
        core::mem::size_of::<LVITEMW>(),
    );
    li.mask = LVIF_PARAM;

    iIndex = 0;
    while iIndex < iCount {
        li.iItem = iIndex;
        if SendMessageW(
            hDlgCtrl,
            LVM_GETITEMW,
            0,
            core::ptr::addr_of_mut!(li) as LPARAM,
        ) != 0
        {
            pItem = li.lParam as *mut DRIVE_ITEM_CONTEXT;
            swprintf(
                szSerial.as_mut_ptr(),
                CPP_COM_WSTR_9618E75FEC6D.as_ptr(),
                LOWORD((*pItem).dwSerial) as ::core::ffi::c_int,
                HIWORD((*pItem).dwSerial) as ::core::ffi::c_int,
            );
            if RegCreateKeyExW(
                hKey,
                szSerial.as_ptr(),
                0,
                core::ptr::null_mut(),
                0,
                KEY_WRITE,
                core::ptr::null_mut(),
                &mut hSubKey,
                core::ptr::null_mut(),
            ) == (ERROR_SUCCESS) as i32
            {
                dwSize = core::mem::size_of::<DWORD>() as DWORD;
                RegSetValueExW(
                    hSubKey,
                    CPP_COM_WSTR_C7A5956868FE.as_ptr(),
                    0,
                    REG_DWORD,
                    ((&raw const (*pItem).dwMaxCapacity) as *const u8),
                    dwSize,
                );
                dwSize = core::mem::size_of::<DWORD>() as DWORD;
                RegSetValueExW(
                    hSubKey,
                    CPP_COM_WSTR_604114ADC8B4.as_ptr(),
                    0,
                    REG_DWORD,
                    ((&raw const (*pItem).dwNukeOnDelete) as *const u8),
                    dwSize,
                );
                RegCloseKey(hSubKey);
            }
        }
        iIndex += 1;
    }

    RegCloseKey(hKey);
    (TRUE) as i32
}
unsafe extern "C" fn FreeDriveItemContext(hwndDlg: HWND) {
    let hDlgCtrl = GetDlgItem(hwndDlg, 14000);
    let iCount = ListView_GetItemCount(hDlgCtrl);
    let mut li: LVITEMW = core::mem::zeroed();
    ZeroMemory(core::ptr::addr_of_mut!(li), core::mem::size_of::<LVITEMW>());
    li.mask = LVIF_PARAM;
    for iIndex in 0..iCount {
        li.iItem = iIndex;
        if SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, core::ptr::addr_of!(li) as LPARAM) != 0 {
            HeapFree(GetProcessHeap(), 0, li.lParam as *mut c_void);
        }
    }
}
unsafe extern "C" fn GetSelectedDriveItem(hwndDlg: HWND, li: *mut LVITEMW) -> i32 {
    let hDlgCtrl = GetDlgItem(hwndDlg, 14000);
    if hDlgCtrl.is_null() {
        return -1;
    }
    let iItemCount = ListView_GetItemCount(hDlgCtrl);
    if iItemCount == 0 {
        return -1;
    }
    ZeroMemory(li as *mut _, core::mem::size_of::<LVITEMW>());
    (*li).mask = LVIF_PARAM | LVIF_STATE;
    (*li).stateMask = (1 as u32).wrapping_neg();
    let mut iIndex = 0 as u32;
    while iIndex < (iItemCount) as u32 {
        (*li).iItem = (iIndex) as i32;
        let result = SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, li as LPARAM);
        if result != 0 {
            if (*li).state & LVIS_SELECTED != 0 {
                return iIndex as i32;
            }
        }
        iIndex += 1;
    }
    -1
}
// DLGPROC is CALLBACK — `extern "system"` is stdcall on i386 and the C convention on
// 64-bit, so ONE copy of this procedure is correct on all three arches.
unsafe extern "system" fn RecycleBinDlg(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let _ = (WM_NEWDRIVESELECTED, WM_APP, WM_UPDATEDRIVESETTINGS);
    let mut lppsn: *mut NMHDR = core::ptr::null_mut();
    let mut lppl: *mut NMLISTVIEW = core::ptr::null_mut();
    let mut li: LVITEMW = core::mem::zeroed();
    let mut pItem: *mut DRIVE_ITEM_CONTEXT = core::ptr::null_mut();
    let mut bSuccess: BOOL = 0;
    let mut uResult: UINT = 0;
    let mut page: *mut PROPSHEETPAGE = core::ptr::null_mut();
    let mut dwStyle: u32 = 0;
    let mut FreeBytesAvailable: ULARGE_INTEGER = core::mem::zeroed();

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as *mut PROPSHEETPAGE;
            InitializeRecycleBinDlg(hwndDlg, (*page).lParam as u16);
            SendDlgItemMessage(
                hwndDlg,
                14004,
                BM_SETCHECK,
                (!SHELL_GetSetting!(SSF_NOCONFIRMRECYCLE, fNoConfirmRecycle)) as WPARAM,
                0,
            );
            dwStyle = SendDlgItemMessage(
                hwndDlg,
                14000,
                LVM_GETEXTENDEDLISTVIEWSTYLE,
                0,
                0,
            ) as u32;
            dwStyle |= LVS_EX_FULLROWSELECT;
            SendDlgItemMessage(
                hwndDlg,
                14000,
                LVM_SETEXTENDEDLISTVIEWSTYLE,
                0,
                dwStyle as LPARAM,
            );
            if GetDlgCtrlID(wParam as HWND) != 14000 {
                SetFocus(GetDlgItem(hwndDlg, 14000));
                return FALSE as INT_PTR;
            }
            return TRUE as INT_PTR;
        }
        WM_COMMAND => {
            match LOWORD((wParam) as u32) {
                14001 => {
                    toggleNukeOnDeleteOption(hwndDlg, FALSE);
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
                14002 => {
                    if HIWORD((wParam) as u32) == (EN_CHANGE) as u16 {
                        PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                    }
                }
                14003 => {
                    toggleNukeOnDeleteOption(hwndDlg, TRUE);
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
                14004 => {
                    PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
                }
                _ => {}
            }
        }
        WM_NOTIFY => {
            lppsn = lParam as *mut NMHDR;
            lppl = lParam as *mut NMLISTVIEW;
            if (*lppsn).code as i32 == PSN_APPLY {
                let mut ss: SHELLSTATE = core::mem::zeroed();
                ss.set_fNoConfirmRecycle(
                    (SendDlgItemMessage(hwndDlg, 14004, BM_GETCHECK, 0, 0)
                        == BST_UNCHECKED as LRESULT) as BOOL,
                );
                SHGetSetSettings(
                    (&mut ss as *mut SHELLSTATE).cast(),
                    SSF_NOCONFIRMRECYCLE,
                    TRUE,
                );

                if GetSelectedDriveItem(hwndDlg, &mut li as *mut LVITEMW) > -1 {
                    SendMessage(
                        hwndDlg,
                        WM_UPDATEDRIVESETTINGS,
                        0,
                        li.lParam as LONG_PTR,
                    );
                }
                if StoreDriveSettings(hwndDlg) != 0 {
                    SetWindowLongPtr(
                        hwndDlg,
                        DWLP_MSGRESULT as ::core::ffi::c_int,
                        PSNRET_NOERROR as LONG_PTR,
                    );
                    return TRUE as INT_PTR;
                }
            } else if (*lppl).hdr.code as i32 == LVN_ITEMCHANGING {
                ZeroMemory(&mut li as *mut LVITEMW, core::mem::size_of::<LVITEMW>());
                li.mask = LVIF_PARAM;
                li.iItem = (*lppl).iItem;
                if SendMessageW((*lppl).hdr.hwndFrom, LVM_GETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM) == 0 {
                    return TRUE as INT_PTR;
                }

                pItem = li.lParam as *mut DRIVE_ITEM_CONTEXT;
                if pItem.is_null() {
                    return TRUE as INT_PTR;
                }

                if ((*lppl).uOldState & LVIS_FOCUSED) == 0 && ((*lppl).uNewState & LVIS_FOCUSED) != 0 {
                    PostMessage(
                        hwndDlg,
                        WM_NEWDRIVESELECTED,
                        0,
                        pItem as LPARAM,
                    );
                } else if ((*lppl).uOldState & LVIS_FOCUSED) != 0 && ((*lppl).uNewState & LVIS_FOCUSED) == 0 {
                    SendMessage(
                        hwndDlg,
                        WM_UPDATEDRIVESETTINGS,
                        0,
                        pItem as LPARAM,
                    );
                }
                return TRUE as INT_PTR;
            }
        }
        WM_NEWDRIVESELECTED => {
            if lParam != 0 {
                pItem = lParam as *mut DRIVE_ITEM_CONTEXT;
                toggleNukeOnDeleteOption(hwndDlg, (*pItem).dwNukeOnDelete as BOOL);
                SetDlgItemInt(
                    hwndDlg,
                    14002,
                    (*pItem).dwMaxCapacity as UINT,
                    FALSE,
                );
            }
        }
        WM_UPDATEDRIVESETTINGS => {
            if lParam != 0 {
                pItem = lParam as *mut DRIVE_ITEM_CONTEXT;
                uResult = GetDlgItemInt(
                    hwndDlg,
                    14002,
                    &mut bSuccess as *mut BOOL,
                    FALSE,
                );
                if bSuccess != 0 {
                    FreeBytesAvailable = (*pItem).FreeBytesAvailable;
                    FreeBytesAvailable.QuadPart = FreeBytesAvailable.QuadPart / (1024 * 1024);
                    (*pItem).dwMaxCapacity = min(
                        uResult,
                        FreeBytesAvailable.__bindgen_anon_1.LowPart,
                    ) as u32;
                    SetDlgItemInt(
                        hwndDlg,
                        14002,
                        (*pItem).dwMaxCapacity as UINT,
                        FALSE,
                    );
                }
                if SendDlgItemMessageW(
                    hwndDlg,
                    14003,
                    BM_GETCHECK,
                    0,
                    0,
                ) == BST_CHECKED as LRESULT {
                    (*pItem).dwNukeOnDelete = TRUE as u32;
                } else {
                    (*pItem).dwNukeOnDelete = FALSE as u32;
                }
            }
        }
        WM_DESTROY => {
            FreeDriveItemContext(hwndDlg);
        }
        _ => {}
    }
    FALSE as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn RecycleBin_AddPropSheetPages(pfnAddPage: LPFNSVADDPROPSHEETPAGE, lParam: LPARAM) -> HRESULT {
    let hpsp = SH_CreatePropertySheetPage((IDD_RECYCLE_BIN_PROPERTIES) as u16, Some(RecycleBinDlg), 0, core::ptr::null_mut());
    AddPropSheetPage((hpsp).cast(), pfnAddPage, lParam)
}

#[no_mangle]
pub unsafe extern "system" fn RecycleBin_AddPropSheetPages_RustBridge(
    pfnAddPage: LPFNSVADDPROPSHEETPAGE,
    lParam: LPARAM,
) -> HRESULT {
    RecycleBin_AddPropSheetPages(pfnAddPage, lParam)
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * Trash virtual folder support. The trashing engine is implemented in trash.c
//  *
//  * Copyright (C) 2006 Mikolaj Zalewski
//  * Copyright (C) 2009 Andrew Hill
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301, USA
//  */
//
// #include <precomp.h>
//
// WINE_DEFAULT_DEBUG_CHANNEL(CRecycleBin);
//
// typedef struct
// {
//     ULARGE_INTEGER FreeBytesAvailable;
//     DWORD dwSerial;
//     DWORD dwMaxCapacity;
//     DWORD dwNukeOnDelete;
// } DRIVE_ITEM_CONTEXT, *PDRIVE_ITEM_CONTEXT;
//
// static void toggleNukeOnDeleteOption(HWND hwndDlg, BOOL bEnable)
// {
//     if (bEnable)
//     {
//         SendDlgItemMessage(hwndDlg, 14001, BM_SETCHECK, BST_UNCHECKED, 0);
//         EnableWindow(GetDlgItem(hwndDlg, 14002), FALSE);
//         SendDlgItemMessage(hwndDlg, 14003, BM_SETCHECK, BST_CHECKED, 0);
//     }
//     else
//     {
//         SendDlgItemMessage(hwndDlg, 14001, BM_SETCHECK, BST_CHECKED, 0);
//         EnableWindow(GetDlgItem(hwndDlg, 14002), TRUE);
//         SendDlgItemMessage(hwndDlg, 14003, BM_SETCHECK, BST_UNCHECKED, 0);
//     }
//
//     // FIXME: Max capacity not implemented yet, disable for now (CORE-13743)
//     EnableWindow(GetDlgItem(hwndDlg, 14002), FALSE);
// }
//
// static VOID
// InitializeRecycleBinDlg(HWND hwndDlg, WCHAR DefaultDrive)
// {
//     WCHAR szDrive[] = L"A:\\";
//     DWORD dwDrives;
//     WCHAR szName[100];
//     WCHAR szVolume[100];
//     DWORD MaxComponent, Flags;
//     DWORD dwSerial;
//     LVCOLUMNW lc;
//     HWND hDlgCtrl;
//     LVITEMW li;
//     INT itemCount;
//     ULARGE_INTEGER TotalNumberOfFreeBytes, TotalNumberOfBytes, FreeBytesAvailable;
//     RECT rect;
//     int columnSize;
//     int defIndex = 0;
//     DWORD dwSize;
//     PDRIVE_ITEM_CONTEXT pItem = NULL, pDefault = NULL, pFirst = NULL;
//
//     hDlgCtrl = GetDlgItem(hwndDlg, 14000);
//
//     if (!LoadStringW(shell32_hInstance, IDS_RECYCLEBIN_LOCATION, szVolume, _countof!(szVolume)))
//         szVolume[0] = 0;
//
//     GetClientRect(hDlgCtrl, &rect);
//
//     ZeroMemory(&lc, sizeof(lc));
//     lc.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;
//
//     columnSize = 140; //FIXME
//     lc.iSubItem   = 0;
//     lc.fmt = LVCFMT_FIXED_WIDTH;
//     lc.cx         = columnSize;
//     lc.cchTextMax = wcslen(szVolume);
//     lc.pszText    = szVolume;
//     (void)SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, 0, (LPARAM)&lc);
//
//     if (!LoadStringW(shell32_hInstance, IDS_RECYCLEBIN_DISKSPACE, szVolume, _countof!(szVolume)))
//         szVolume[0] = 0;
//
//     lc.iSubItem   = 1;
//     lc.cx         = rect.right - rect.left - columnSize;
//     lc.cchTextMax = wcslen(szVolume);
//     lc.pszText    = szVolume;
//     (void)SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, 1, (LPARAM)&lc);
//
//     dwDrives = GetLogicalDrives();
//     itemCount = 0;
//     do
//     {
//         if (dwDrives & 0x1)
//         {
//             UINT Type = GetDriveTypeW(szDrive);
//             if (Type == DRIVE_FIXED) //FIXME
//             {
//                 if (!GetVolumeInformationW(szDrive, szName, _countof!(szName), &dwSerial, &MaxComponent, &Flags, NULL, 0))
//                 {
//                     szName[0] = 0;
//                     dwSerial = -1;
//                 }
//
//                 swprintf(szVolume, L"%s (%c:)", szName, szDrive[0]);
//                 ZeroMemory(&li, sizeof(li));
//                 li.mask = LVIF_TEXT | LVIF_PARAM;
//                 li.iSubItem = 0;
//                 li.pszText = szVolume;
//                 li.iItem = itemCount;
//                 SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, (LPARAM)&li);
//                 if (GetDiskFreeSpaceExW(szDrive, &FreeBytesAvailable, &TotalNumberOfBytes, &TotalNumberOfFreeBytes))
//                 {
//                     if (StrFormatByteSizeW(TotalNumberOfFreeBytes.QuadPart, szVolume, _countof!(szVolume)))
//                     {
//                         pItem = (DRIVE_ITEM_CONTEXT *)HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(DRIVE_ITEM_CONTEXT));
//                         if (pItem)
//                         {
//                             pItem->FreeBytesAvailable = FreeBytesAvailable;
//                             pItem->dwSerial = dwSerial;
//
//                             swprintf(szName, L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\BitBucket\\Volume\\%04X-%04X", LOWORD(dwSerial), HIWORD(dwSerial));
//
//                             dwSize = sizeof(DWORD);
//                             if (RegGetValueW(HKEY_CURRENT_USER, szName, L"MaxCapacity", RRF_RT_DWORD, NULL, &pItem->dwMaxCapacity, &dwSize))
//                                 pItem->dwMaxCapacity = ~0;
//
//                             /* Check if the maximum capacity doesn't exceed the available disk space (in megabytes), and truncate it if needed */
//                             FreeBytesAvailable.QuadPart = (FreeBytesAvailable.QuadPart / (1024 * 1024));
//                             pItem->dwMaxCapacity = min(pItem->dwMaxCapacity, FreeBytesAvailable.LowPart);
//
//                             dwSize = sizeof(DWORD);
//                             RegGetValueW(HKEY_CURRENT_USER, szName, L"NukeOnDelete", RRF_RT_DWORD, NULL, &pItem->dwNukeOnDelete, &dwSize);
//
//                             li.mask = LVIF_PARAM;
//                             li.lParam = (LPARAM)pItem;
//                             (void)SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, (LPARAM)&li);
//                             if (szDrive[0] == DefaultDrive)
//                             {
//                                 defIndex = itemCount;
//                                 pDefault = pItem;
//                             }
//                         }
//                         if (!pFirst)
//                             pFirst = pItem;
//
//                         li.mask = LVIF_TEXT;
//                         li.iSubItem = 1;
//                         li.pszText = szVolume;
//                         li.iItem = itemCount;
//                         (void)SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, (LPARAM)&li);
//                     }
//                 }
//                 itemCount++;
//             }
//         }
//         szDrive[0]++;
//         dwDrives = (dwDrives >> 1);
//     } while (dwDrives);
//
//     if (!pDefault)
//         pDefault = pFirst;
//     if (pDefault)
//     {
//         toggleNukeOnDeleteOption(hwndDlg, pDefault->dwNukeOnDelete);
//         SetDlgItemInt(hwndDlg, 14002, pDefault->dwMaxCapacity, FALSE);
//     }
//     ZeroMemory(&li, sizeof(li));
//     li.mask = LVIF_STATE;
//     li.stateMask = (UINT)-1;
//     li.state = LVIS_FOCUSED | LVIS_SELECTED;
//     li.iItem = defIndex;
//     (void)SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, (LPARAM)&li);
// }
//
// static BOOL StoreDriveSettings(HWND hwndDlg)
// {
//     int iCount, iIndex;
//     HWND hDlgCtrl = GetDlgItem(hwndDlg, 14000);
//     LVITEMW li;
//     PDRIVE_ITEM_CONTEXT pItem;
//     HKEY hKey, hSubKey;
//     WCHAR szSerial[20];
//     DWORD dwSize;
//
//     if (RegCreateKeyExW(HKEY_CURRENT_USER, L"SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Explorer\\BitBucket\\Volume", 0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) != ERROR_SUCCESS)
//         return FALSE;
//
//     iCount = ListView_GetItemCount(hDlgCtrl);
//
//     ZeroMemory(&li, sizeof(li));
//     li.mask = LVIF_PARAM;
//
//     for (iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         li.iItem = iIndex;
//         if (SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         {
//             pItem = (PDRIVE_ITEM_CONTEXT)li.lParam;
//             swprintf(szSerial, L"%04X-%04X", LOWORD(pItem->dwSerial), HIWORD(pItem->dwSerial));
//             if (RegCreateKeyExW(hKey, szSerial, 0, NULL, 0, KEY_WRITE, NULL, &hSubKey, NULL) == ERROR_SUCCESS)
//             {
//                 dwSize = sizeof(DWORD);
//                 RegSetValueExW(hSubKey, L"MaxCapacity", 0, REG_DWORD, (LPBYTE)&pItem->dwMaxCapacity, dwSize);
//                 dwSize = sizeof(DWORD);
//                 RegSetValueExW(hSubKey, L"NukeOnDelete", 0, REG_DWORD, (LPBYTE)&pItem->dwNukeOnDelete, dwSize);
//                 RegCloseKey(hSubKey);
//             }
//         }
//     }
//     RegCloseKey(hKey);
//     return TRUE;
// }
//
// static VOID FreeDriveItemContext(HWND hwndDlg)
// {
//     int iCount, iIndex;
//     HWND hDlgCtrl = GetDlgItem(hwndDlg, 14000);
//     LVITEMW li;
//
//     iCount = ListView_GetItemCount(hDlgCtrl);
//
//     ZeroMemory(&li, sizeof(li));
//     li.mask = LVIF_PARAM;
//
//     for (iIndex = 0; iIndex < iCount; iIndex++)
//     {
//         li.iItem = iIndex;
//         if (SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)&li))
//         {
//             HeapFree(GetProcessHeap(), 0, (PVOID)li.lParam);
//         }
//     }
// }
//
// static INT
// GetSelectedDriveItem(HWND hwndDlg, LVITEMW* li)
// {
//     HWND hDlgCtrl;
//     UINT iItemCount, iIndex;
//
//     hDlgCtrl = GetDlgItem(hwndDlg, 14000);
//     if (!hDlgCtrl)
//         return -1;
//
//     iItemCount = ListView_GetItemCount(hDlgCtrl);
//     if (!iItemCount)
//         return -1;
//
//     ZeroMemory(li, sizeof(LVITEMW));
//     li->mask = LVIF_PARAM | LVIF_STATE;
//     li->stateMask = (UINT)-1;
//     for (iIndex = 0; iIndex < iItemCount; iIndex++)
//     {
//         li->iItem = iIndex;
//         if (SendMessageW(hDlgCtrl, LVM_GETITEMW, 0, (LPARAM)li))
//         {
//             if (li->state & LVIS_SELECTED)
//                 return iIndex;
//         }
//     }
//     return -1;
// }
//
// static INT_PTR __stdcall
// RecycleBinDlg(
//     HWND hwndDlg,
//     UINT uMsg,
//     WPARAM wParam,
//     LPARAM lParam)
// {
//     enum { WM_NEWDRIVESELECTED = WM_APP, WM_UPDATEDRIVESETTINGS };
//     LPPSHNOTIFY lppsn;
//     LPNMLISTVIEW lppl;
//     LVITEMW li;
//     PDRIVE_ITEM_CONTEXT pItem;
//     BOOL bSuccess;
//     UINT uResult;
//     PROPSHEETPAGE * page;
//     DWORD dwStyle;
//     ULARGE_INTEGER FreeBytesAvailable;
//
//     switch(uMsg)
//     {
//         case WM_INITDIALOG:
//             page = (PROPSHEETPAGE*)lParam;
//             InitializeRecycleBinDlg(hwndDlg, (WCHAR)page->lParam);
//             SendDlgItemMessage(hwndDlg, 14004, BM_SETCHECK, !SHELL_GetSetting!(SSF_NOCONFIRMRECYCLE, fNoConfirmRecycle), 0);
//             dwStyle = (DWORD) SendDlgItemMessage(hwndDlg, 14000, LVM_GETEXTENDEDLISTVIEWSTYLE, 0, 0);
//             dwStyle = dwStyle | LVS_EX_FULLROWSELECT;
//             SendDlgItemMessage(hwndDlg, 14000, LVM_SETEXTENDEDLISTVIEWSTYLE, 0, dwStyle);
//             if (GetDlgCtrlID((HWND)wParam) != 14000)
//             {
//                 SetFocus(GetDlgItem(hwndDlg, 14000));
//                 return FALSE;
//             }
//             return TRUE;
//         case WM_COMMAND:
//             switch(LOWORD(wParam))
//             {
//                 case 14001:
//                     toggleNukeOnDeleteOption(hwndDlg, FALSE);
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//                 case 14002:
//                     if (HIWORD(wParam) == EN_CHANGE)
//                         PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//                 case 14003:
//                     toggleNukeOnDeleteOption(hwndDlg, TRUE);
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//                 case 14004:
//                     PropSheet_Changed(GetParent(hwndDlg), hwndDlg);
//                     break;
//             }
//             break;
//         case WM_NOTIFY:
//             lppsn = (LPPSHNOTIFY) lParam;
//             lppl = (LPNMLISTVIEW) lParam;
//             if (lppsn->hdr.code == PSN_APPLY)
//             {
//                 SHELLSTATE ss;
//                 ss.fNoConfirmRecycle = SendDlgItemMessage(hwndDlg, 14004, BM_GETCHECK, 0, 0) == BST_UNCHECKED;
//                 SHGetSetSettings(&ss, SSF_NOCONFIRMRECYCLE, TRUE);
//
//                 if (GetSelectedDriveItem(hwndDlg, &li) > -1)
//                 {
//                     SendMessage(hwndDlg, WM_UPDATEDRIVESETTINGS, 0, li.lParam);
//                 }
//                 if (StoreDriveSettings(hwndDlg))
//                 {
//                     SetWindowLongPtr( hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR );
//                     return TRUE;
//                 }
//             }
//             else if (lppl->hdr.code == LVN_ITEMCHANGING)
//             {
//                 ZeroMemory(&li, sizeof(li));
//                 li.mask = LVIF_PARAM;
//                 li.iItem = lppl->iItem;
//                 if (!SendMessageW(lppl->hdr.hwndFrom, LVM_GETITEMW, 0, (LPARAM)&li))
//                     return TRUE;
//
//                 pItem = (PDRIVE_ITEM_CONTEXT)li.lParam;
//                 if (!pItem)
//                     return TRUE;
//
//                 if (!(lppl->uOldState & LVIS_FOCUSED) && (lppl->uNewState & LVIS_FOCUSED))
//                 {
//                     // New focused item, delay handling until after kill focus has been processed
//                     PostMessage(hwndDlg, WM_NEWDRIVESELECTED, 0, (LPARAM)pItem);
//                 }
//                 else if ((lppl->uOldState & LVIS_FOCUSED) && !(lppl->uNewState & LVIS_FOCUSED))
//                 {
//                     // Kill focus
//                     SendMessage(hwndDlg, WM_UPDATEDRIVESETTINGS, 0, (LPARAM)pItem);
//                 }
//                 return TRUE;
//
//             }
//             break;
//         case WM_NEWDRIVESELECTED:
//             if (lParam)
//             {
//                 pItem = (PDRIVE_ITEM_CONTEXT)lParam;
//                 toggleNukeOnDeleteOption(hwndDlg, pItem->dwNukeOnDelete);
//                 SetDlgItemInt(hwndDlg, 14002, pItem->dwMaxCapacity, FALSE);
//             }
//             break;
//         case WM_UPDATEDRIVESETTINGS:
//             if (lParam)
//             {
//                 pItem = (PDRIVE_ITEM_CONTEXT)lParam;
//                 uResult = GetDlgItemInt(hwndDlg, 14002, &bSuccess, FALSE);
//                 if (bSuccess)
//                 {
//                     /* Check if the maximum capacity doesn't exceed the available disk space (in megabytes), and truncate it if needed */
//                     FreeBytesAvailable = pItem->FreeBytesAvailable;
//                     FreeBytesAvailable.QuadPart = (FreeBytesAvailable.QuadPart / (1024 * 1024));
//                     pItem->dwMaxCapacity = min(uResult, FreeBytesAvailable.LowPart);
//                     SetDlgItemInt(hwndDlg, 14002, pItem->dwMaxCapacity, FALSE);
//                 }
//                 if (SendDlgItemMessageW(hwndDlg, 14003, BM_GETCHECK, 0, 0) == BST_CHECKED)
//                     pItem->dwNukeOnDelete = TRUE;
//                 else
//                     pItem->dwNukeOnDelete = FALSE;
//             }
//             break;
//         case WM_DESTROY:
//             FreeDriveItemContext(hwndDlg);
//             break;
//     }
//     return FALSE;
// }
//
// HRESULT RecycleBin_AddPropSheetPages(LPFNSVADDPROPSHEETPAGE pfnAddPage, LPARAM lParam)
// {
//     HPROPSHEETPAGE hpsp = SH_CreatePropertySheetPage(IDD_RECYCLE_BIN_PROPERTIES, RecycleBinDlg, 0, NULL);
//     return AddPropSheetPage(hpsp, pfnAddPage, lParam);
// }
//
// // Rust bridge — shim for the Rust-owned CRecycleBin (shell32-rust recycle_bin).
// extern "C" HRESULT WINAPI
// RecycleBin_AddPropSheetPages_RustBridge(LPFNSVADDPROPSHEETPAGE pfnAddPage, LPARAM lParam)
// {
//     return RecycleBin_AddPropSheetPages(pfnAddPage, lParam);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
