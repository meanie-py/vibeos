use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const DESK_EXT_EXTINTERFACE: *const TCHAR = (&['D' as TCHAR, 'e' as TCHAR, 's' as TCHAR, 'k' as TCHAR, '.' as TCHAR, 'c' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, ' ' as TCHAR, 'e' as TCHAR, 'x' as TCHAR, 't' as TCHAR, 'e' as TCHAR, 'n' as TCHAR, 's' as TCHAR, 'i' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, ' ' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, 't' as TCHAR, 'e' as TCHAR, 'r' as TCHAR, 'f' as TCHAR, 'a' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_PRUNINGMODE: *const TCHAR = (&['P' as TCHAR, 'r' as TCHAR, 'u' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 'n' as TCHAR, 'g' as TCHAR, ' ' as TCHAR, 'M' as TCHAR, 'o' as TCHAR, 'd' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_DISPLAYDEVICE: *const TCHAR = (&['D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'y' as TCHAR, ' ' as TCHAR, 'D' as TCHAR, 'e' as TCHAR, 'v' as TCHAR, 'i' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_DISPLAYNAME: *const TCHAR = (&['D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'y' as TCHAR, ' ' as TCHAR, 'N' as TCHAR, 'a' as TCHAR, 'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_DISPLAYID: *const TCHAR = (&['D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'y' as TCHAR, ' ' as TCHAR, 'I' as TCHAR, 'D' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_DISPLAYKEY: *const TCHAR = (&['D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'y' as TCHAR, ' ' as TCHAR, 'K' as TCHAR, 'e' as TCHAR, 'y' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_DISPLAYSTATEFLAGS: *const TCHAR = (&['D' as TCHAR, 'i' as TCHAR, 's' as TCHAR, 'p' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'y' as TCHAR, ' ' as TCHAR, 'S' as TCHAR, 't' as TCHAR, 'a' as TCHAR, 't' as TCHAR, 'e' as TCHAR, ' ' as TCHAR, 'F' as TCHAR, 'l' as TCHAR, 'a' as TCHAR, 'g' as TCHAR, 's' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_MONITORNAME: *const TCHAR = (&['M' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 't' as TCHAR, 'o' as TCHAR, 'r' as TCHAR, ' ' as TCHAR, 'N' as TCHAR, 'a' as TCHAR, 'm' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();
pub const DESK_EXT_MONITORDEVICE: *const TCHAR = (&['M' as TCHAR, 'o' as TCHAR, 'n' as TCHAR, 'i' as TCHAR, 't' as TCHAR, 'o' as TCHAR, 'r' as TCHAR, ' ' as TCHAR, 'D' as TCHAR, 'e' as TCHAR, 'v' as TCHAR, 'i' as TCHAR, 'c' as TCHAR, 'e' as TCHAR, 0 as TCHAR]).as_ptr();

pub unsafe extern "C" fn QueryDeskCplExtInterface(pdo: *mut IDataObject) -> PDESK_EXT_INTERFACE {
    let mut pRecvBuffer: PDESK_EXT_INTERFACE = core::ptr::null_mut();
    let mut pExtIface: PDESK_EXT_INTERFACE = core::ptr::null_mut();
    let mut fetc: FORMATETC = core::mem::zeroed();
    let mut medium: STGMEDIUM = core::mem::zeroed();

    fetc.cfFormat = RegisterClipboardFormat(DESK_EXT_EXTINTERFACE) as CLIPFORMAT;
    fetc.ptd = core::ptr::null_mut();
    fetc.dwAspect = DVASPECT_CONTENT;
    fetc.lindex = -1;
    fetc.tymed = TYMED_HGLOBAL;

    if (IDataObject_GetData(pdo, core::ptr::addr_of_mut!(fetc), core::ptr::addr_of_mut!(medium)) >= 0)
        && !medium.__bindgen_anon_1.hGlobal.is_null()
    {
        /* We always receive the string in unicode! */
        pRecvBuffer = GlobalLock(medium.__bindgen_anon_1.hGlobal) as PDESK_EXT_INTERFACE;

        if (*pRecvBuffer).cbSize as usize == core::mem::size_of::<DESK_EXT_INTERFACE>() as usize {
            pExtIface = LocalAlloc(
                LMEM_FIXED,
                (core::mem::size_of::<DESK_EXT_INTERFACE>()) as SIZE_T,
            ) as PDESK_EXT_INTERFACE;
            if !pExtIface.is_null() {
                CopyMemory!(
                    pExtIface as *mut core::ffi::c_void,
                    pRecvBuffer as *const core::ffi::c_void,
                    core::mem::size_of::<DESK_EXT_INTERFACE>(),
                );
            }
        }

        GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
        ReleaseStgMedium(core::ptr::addr_of_mut!(medium));
    }

    return pExtIface;
}

pub unsafe extern "C" fn QueryDeskCplString(pdo: *mut IDataObject, cfFormat: UINT) -> LPTSTR {
    const UNICODE_BUILD: bool = true;
    let mut fetc: FORMATETC = core::mem::zeroed();
    let mut medium: STGMEDIUM = core::mem::zeroed();
    let mut BufLen: SIZE_T = 0;
    let mut lpRecvBuffer: LPWSTR = core::ptr::null_mut();
    let mut lpStr: LPTSTR = core::ptr::null_mut();

    fetc.cfFormat = cfFormat as CLIPFORMAT;
    fetc.ptd = core::ptr::null_mut();
    fetc.dwAspect = DVASPECT_CONTENT as DWORD;
    fetc.lindex = -(1 as LONG);
    fetc.tymed = TYMED_HGLOBAL as DWORD;

    if (IDataObject_GetData(pdo, core::ptr::addr_of_mut!(fetc), core::ptr::addr_of_mut!(medium)) >= (0 as HRESULT))
        && !medium.__bindgen_anon_1.hGlobal.is_null()
    {
        /* We always receive the string in unicode! */
        lpRecvBuffer = GlobalLock(medium.__bindgen_anon_1.hGlobal) as LPWSTR;

        BufLen = wcslen(lpRecvBuffer as *const WCHAR)
            .wrapping_add((1 as SIZE_T) as _) as SIZE_T;
        lpStr = LocalAlloc(
            LMEM_FIXED as UINT,
            BufLen.wrapping_mul(core::mem::size_of::<TCHAR>() as SIZE_T),
        ) as LPTSTR;
        if !lpStr.is_null() {
            if UNICODE_BUILD {
                wcscpy(lpStr as *mut WCHAR, lpRecvBuffer as *const WCHAR);
            } else {
                WideCharToMultiByte(
                    CP_ACP as UINT,
                    0 as DWORD,
                    lpRecvBuffer as *const WCHAR,
                    -(1 as i32),
                    lpStr as *mut i8,
                    BufLen as i32,
                    core::ptr::null(),
                    core::ptr::null_mut(),
                );
            }
        }

        GlobalUnlock(medium.__bindgen_anon_1.hGlobal);
        ReleaseStgMedium(core::ptr::addr_of_mut!(medium));
    }

    return lpStr;
}

pub unsafe extern "C" fn DeskCplExtDisplaySaveSettings(
    mut DeskExtInterface: PDESK_EXT_INTERFACE,
    mut hwndDlg: HWND,
) -> LONG {
    type PDISPLAYSAVESETTINGS = unsafe extern "system" fn(PVOID, HWND) -> LONG;
    let mut hModDeskCpl: HMODULE = core::ptr::null_mut();
    let mut lRet: LONG = DISP_CHANGE_BADPARAM as LONG;

    /* We could use GetModuleHandle() instead, but then this routine
       wouldn't work if some other application hosts the shell extension */
    let deskcpl_name: [u16; 9] = ['d' as u16, 'e' as u16, 's' as u16, 'k' as u16, '.' as u16, 'c' as u16, 'p' as u16, 'l' as u16, 0];
    hModDeskCpl = LoadLibrary(deskcpl_name.as_ptr()) as HMODULE;
    if !hModDeskCpl.is_null() {
        let pRawProc: FARPROC = GetProcAddress(
            hModDeskCpl as HINSTANCE,
            b"DisplaySaveSettings\0".as_ptr() as LPCSTR,
        );
        if core::mem::transmute::<FARPROC, usize>(pRawProc) != 0 {
            let pDisplaySaveSettings: PDISPLAYSAVESETTINGS =
                core::mem::transmute::<FARPROC, PDISPLAYSAVESETTINGS>(pRawProc);
            lRet = pDisplaySaveSettings(
                (*DeskExtInterface).Context,
                hwndDlg,
            );
        }

        FreeLibrary(hModDeskCpl);
    }

    return lRet;
}
