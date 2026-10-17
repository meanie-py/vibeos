use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0ACAC64D15DA: [WCHAR; 3] = [0x002D, 0x0031, 0x0000];
pub static CPP_COM_WSTR_3C3B5BF3E3E6: [WCHAR; 17] = [0x0053, 0x0057, 0x005F, 0x0046, 0x004F, 0x0052, 0x0043, 0x0045, 0x004D, 0x0049, 0x004E, 0x0049, 0x004D, 0x0049, 0x005A, 0x0045, 0x0000];
pub static CPP_COM_WSTR_404F6E6985E9: [WCHAR; 17] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004D, 0x0049, 0x004E, 0x0049, 0x004D, 0x0049, 0x005A, 0x0045, 0x0044, 0x0000];
pub static CPP_COM_WSTR_49E06D442BC9: [WCHAR; 18] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004E, 0x004F, 0x0041, 0x0043, 0x0054, 0x0049, 0x0056, 0x0041, 0x0054, 0x0045, 0x0000];
pub static CPP_COM_WSTR_4FA8C46AD063: [WCHAR; 8] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x0000];
pub static CPP_COM_WSTR_5FFE2A98CFCF: [WCHAR; 5] = [0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_737145833B40: [WCHAR; 14] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004E, 0x004F, 0x0052, 0x004D, 0x0041, 0x004C, 0x0000];
pub static CPP_COM_WSTR_8329369A55B2: [WCHAR; 11] = [0x0053, 0x0057, 0x005F, 0x0052, 0x0045, 0x0053, 0x0054, 0x004F, 0x0052, 0x0045, 0x0000];
pub static CPP_COM_WSTR_84FEB4D52CDE: [WCHAR; 5] = [0x004C, 0x0069, 0x0073, 0x0074, 0x0000];
pub static CPP_COM_WSTR_884B2A4A85AF: [WCHAR; 15] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x0044, 0x0045, 0x0046, 0x0041, 0x0055, 0x004C, 0x0054, 0x0000];
pub static CPP_COM_WSTR_AE431856164E: [WCHAR; 19] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004D, 0x0049, 0x004E, 0x004E, 0x004F, 0x0041, 0x0043, 0x0054, 0x0049, 0x0056, 0x0045, 0x0000];
pub static CPP_COM_WSTR_B305364C27DB: [WCHAR; 7] = [0x0052, 0x0065, 0x0070, 0x006F, 0x0072, 0x0074, 0x0000];
pub static CPP_COM_WSTR_BF3C1AE071C4: [WCHAR; 10] = [0x0053, 0x006D, 0x0061, 0x006C, 0x006C, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000];
pub static CPP_COM_WSTR_C9022ED4B6EC: [WCHAR; 8] = [0x0053, 0x0057, 0x005F, 0x0048, 0x0049, 0x0044, 0x0045, 0x0000];
pub static CPP_COM_WSTR_D5910B659783: [WCHAR; 17] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004D, 0x0041, 0x0058, 0x0049, 0x004D, 0x0049, 0x005A, 0x0045, 0x0044, 0x0000];
pub static CPP_COM_WSTR_DA160AF7FB55: [WCHAR; 12] = [0x0053, 0x0057, 0x005F, 0x004D, 0x0049, 0x004E, 0x0049, 0x004D, 0x0049, 0x005A, 0x0045, 0x0000];
pub static CPP_COM_WSTR_E26FB2050240: [WCHAR; 10] = [0x0053, 0x0057, 0x005F, 0x0053, 0x0048, 0x004F, 0x0057, 0x004E, 0x0041, 0x0000];

use core::ffi::c_void;

const ListView_SmallIcons: i32 = 0;
const ListView_LargeIcons: i32 = 1;
const ListView_List: i32 = 2;
const ListView_Detail: i32 = 3;
#[no_mangle]
pub unsafe extern "C" fn ViewModeToCmdId(ListViewMode: LISTVIEW_MODE) -> UINT {
    let mut CmdId: UINT;

    match ListViewMode as i32 {
        0 => {
            // ListView_SmallIcons
            CmdId = IDM_VIEW_SMALL_ICONS;
        }

        1 => {
            // ListView_LargeIcons
            CmdId = IDM_VIEW_LARGE_ICONS;
        }

        2 => {
            // ListView_List
            CmdId = IDM_VIEW_LIST;
        }

        // default: falls through to ListView_Detail
        _ => {
            // ListView_Detail
            CmdId = IDM_VIEW_DETAILS;
        }
    }

    return CmdId;
}

#[no_mangle]
pub unsafe extern "C" fn CmdIdToViewMode(CmdId: UINT) -> LISTVIEW_MODE {
    let ListViewMode: LISTVIEW_MODE;

    match CmdId {
        IDM_VIEW_SMALL_ICONS => {
            ListViewMode = (ListView_SmallIcons) as u32;
        }

        IDM_VIEW_LARGE_ICONS => {
            ListViewMode = (ListView_LargeIcons) as u32;
        }

        IDM_VIEW_LIST => {
            ListViewMode = (ListView_List) as u32;
        }

        IDM_VIEW_DETAILS | _ => {
            ListViewMode = (ListView_Detail) as u32;
        }
    }

    return ListViewMode;
}
#[no_mangle]
pub unsafe extern "C" fn ViewModeToString(ListViewMode: i32) -> LPWSTR {
    match ListViewMode {
        ListView_SmallIcons => {
            return CPP_COM_WSTR_BF3C1AE071C4.as_ptr() as LPWSTR;
        }

        ListView_LargeIcons => {
            return CPP_COM_WSTR_5FFE2A98CFCF.as_ptr() as LPWSTR;
        }

        ListView_List => {
            return CPP_COM_WSTR_84FEB4D52CDE.as_ptr() as LPWSTR;
        }

        _ => {
            return CPP_COM_WSTR_B305364C27DB.as_ptr() as LPWSTR;
        }
    }
    #[allow(unreachable_code)]
    {
        return CPP_COM_WSTR_0ACAC64D15DA.as_ptr() as LPWSTR;
    }
}
#[no_mangle]
pub unsafe extern "C" fn StringToViewMode(ViewMode: LPWSTR) -> LISTVIEW_MODE {
    if _wcsicmp(ViewMode, CPP_COM_WSTR_BF3C1AE071C4.as_ptr()) == 0 {
        return (ListView_SmallIcons) as LISTVIEW_MODE;
    } else if _wcsicmp(ViewMode, CPP_COM_WSTR_5FFE2A98CFCF.as_ptr()) == 0 {
        return (ListView_LargeIcons) as LISTVIEW_MODE;
    } else if _wcsicmp(ViewMode, CPP_COM_WSTR_84FEB4D52CDE.as_ptr()) == 0 {
        return (ListView_List) as LISTVIEW_MODE;
    } else {
        return (ListView_Detail) as LISTVIEW_MODE;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ViewModeToStyle(ListViewMode: LISTVIEW_MODE) -> DWORD {
    if ListViewMode == (ListView_SmallIcons) as u32 {
        return LVS_SMALLICON;
    } else if ListViewMode == (ListView_LargeIcons) as u32 {
        return LVS_ICON;
    } else if ListViewMode == (ListView_List) as u32 {
        return LVS_LIST;
    } else {
        // default: and case ListView_Detail: share this return
        return LVS_REPORT;
    }
}
#[no_mangle]
pub unsafe extern "C" fn ShowCmdToString(nShowCmd: i32) -> LPWSTR {
    match (nShowCmd) as u32 {
        SW_HIDE => {
            return CPP_COM_WSTR_C9022ED4B6EC.as_ptr() as LPWSTR;
        }
        SW_SHOWNORMAL => {
            return CPP_COM_WSTR_737145833B40.as_ptr() as LPWSTR;
        }
        SW_SHOWMINIMIZED => {
            return CPP_COM_WSTR_404F6E6985E9.as_ptr() as LPWSTR;
        }
        SW_SHOWMAXIMIZED => {
            return CPP_COM_WSTR_D5910B659783.as_ptr() as LPWSTR;
        }
        SW_SHOWNOACTIVATE => {
            return CPP_COM_WSTR_49E06D442BC9.as_ptr() as LPWSTR;
        }
        SW_SHOW => {
            return CPP_COM_WSTR_4FA8C46AD063.as_ptr() as LPWSTR;
        }
        SW_MINIMIZE => {
            return CPP_COM_WSTR_DA160AF7FB55.as_ptr() as LPWSTR;
        }
        SW_SHOWMINNOACTIVE => {
            return CPP_COM_WSTR_AE431856164E.as_ptr() as LPWSTR;
        }
        SW_SHOWNA => {
            return CPP_COM_WSTR_E26FB2050240.as_ptr() as LPWSTR;
        }
        SW_RESTORE => {
            return CPP_COM_WSTR_8329369A55B2.as_ptr() as LPWSTR;
        }
        SW_SHOWDEFAULT => {
            return CPP_COM_WSTR_884B2A4A85AF.as_ptr() as LPWSTR;
        }
        SW_FORCEMINIMIZE => {
            return CPP_COM_WSTR_3C3B5BF3E3E6.as_ptr() as LPWSTR;
        }
        _ => {}
    }

    return CPP_COM_WSTR_0ACAC64D15DA.as_ptr() as LPWSTR;
}
#[no_mangle]
pub unsafe extern "C" fn ShowCmdFromString(pszString: LPWSTR) -> i32 {
    if _wcsicmp(pszString, CPP_COM_WSTR_C9022ED4B6EC.as_ptr()) == 0 {
        return (SW_HIDE as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_737145833B40.as_ptr()) == 0 {
        return (SW_SHOWNORMAL as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_404F6E6985E9.as_ptr()) == 0 {
        return (SW_SHOWMINIMIZED as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_D5910B659783.as_ptr()) == 0 {
        return (SW_SHOWMAXIMIZED as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_49E06D442BC9.as_ptr()) == 0 {
        return (SW_SHOWNOACTIVATE as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_4FA8C46AD063.as_ptr()) == 0 {
        return (SW_SHOW as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_DA160AF7FB55.as_ptr()) == 0 {
        return (SW_MINIMIZE as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_AE431856164E.as_ptr()) == 0 {
        return (SW_SHOWMINNOACTIVE as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_E26FB2050240.as_ptr()) == 0 {
        return (SW_SHOWNA as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_8329369A55B2.as_ptr()) == 0 {
        return (SW_RESTORE as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_884B2A4A85AF.as_ptr()) == 0 {
        return (SW_SHOWDEFAULT as i32);
    } else if _wcsicmp(pszString, CPP_COM_WSTR_3C3B5BF3E3E6.as_ptr()) == 0 {
        return (SW_FORCEMINIMIZE as i32);
    }
    return 0;
}
