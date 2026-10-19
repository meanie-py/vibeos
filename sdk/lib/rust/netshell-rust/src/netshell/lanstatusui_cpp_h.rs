use crate::prelude::*;
use crate::netshell::connectmanager_cpp_h::CNetConnectionManager_CreateInstance;
use crate::netshell::connectmanager_cpp_h::GetAdapterIndexFromNetCfgInstanceId;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_03971BB4A692: [WCHAR; 12] = [0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x002E, 0x0025, 0x0075, 0x0000];
pub static CPP_COM_WSTR_31C1CCDE73D9: [WCHAR; 30] = [0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x002D, 0x0025, 0x0030, 0x0032, 0x0078, 0x0000];
pub static CPP_COM_WSTR_41594B820641: [WCHAR; 13] = [0x0048, 0x0048, 0x0027, 0x003A, 0x0027, 0x006D, 0x006D, 0x0027, 0x003A, 0x0027, 0x0073, 0x0073, 0x0000];
pub static CPP_COM_WSTR_5FC51169506B: [WCHAR; 32] = [0x0049, 0x004F, 0x006C, 0x0065, 0x0043, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0054, 0x0061, 0x0072, 0x0067, 0x0065, 0x0074, 0x005F, 0x0066, 0x006E, 0x0051, 0x0075, 0x0065, 0x0072, 0x0079, 0x0053, 0x0074, 0x0061, 0x0074, 0x0075, 0x0073, 0x0000];
pub static CPP_COM_WSTR_8AFAAE84FEDA: [WCHAR; 8] = [0x0063, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0000];
pub static CPP_COM_WSTR_8D71B3FAAB82: [WCHAR; 2] = [0x0020, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_CA983B81C2D2: [WCHAR; 15] = [0x006E, 0x0065, 0x0074, 0x0063, 0x006F, 0x006E, 0x006E, 0x0065, 0x0063, 0x0074, 0x0069, 0x006F, 0x006E, 0x0073, 0x0000];

use core::ffi::c_void;

// time.h's `struct tm` and `localtime`, which the island (CRT headers excluded) does not
// carry. `localtime` is msvcrt's export where time_t is 32-bit (i386, _USE_32BIT_TIME_T)
// and an inline for `_localtime64` where it is 64-bit.
#[repr(C)]
pub struct tm {
    pub tm_sec: i32,
    pub tm_min: i32,
    pub tm_hour: i32,
    pub tm_mday: i32,
    pub tm_mon: i32,
    pub tm_year: i32,
    pub tm_wday: i32,
    pub tm_yday: i32,
    pub tm_isdst: i32,
}
extern "C" {
    #[cfg_attr(not(target_arch = "x86"), link_name = "_localtime64")]
    fn localtime(t: *const time_t) -> *mut tm;
}






unsafe extern "C" fn EnumNotificationIconWindows(Func: WNDENUMPROC, lParam: LPARAM) {
    // The current design unfortunately uses one window per icon (that is what NOTIFYICONDATA::uID is for),
    // WNDENUMPROC is responsible for making sure they are dealing with the correct window.
    let mut hPrev: HWND = core::ptr::null_mut();
    let mut hWnd: HWND;
    loop {
        hWnd = FindWindowExW(core::ptr::null_mut(), hPrev, core::ptr::null_mut(), TRAYICONWINDOWCAPTIONW);
        if hWnd.is_null() {
            break;
        }
        if (Func.unwrap_unchecked())(hWnd, lParam) == 0 {
            break;
        }
        hPrev = hWnd;
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateLanStatusUiDlg(hwndDlg: HWND, IfEntry: *mut MIB_IFROW, pContext: *mut LANSTATUSUI_CONTEXT) {
    let mut szFormat: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut TimeConnected: SYSTEMTIME = core::mem::zeroed();
    let mut DurationSeconds: DWORD;
    let mut Buffer: [WCHAR; 100] = [0; 100];
    let mut DayBuffer: [WCHAR; 30] = [0; 30];
    let mut LocBuffer: [WCHAR; 50] = [0; 50];

    let mut Ticks: DWORD;

    if (*IfEntry).dwSpeed < 1000 {
        if LoadStringW(netshell_hInstance, IDS_FORMAT_BIT, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
            _swprintf(szBuffer.as_mut_ptr(), szFormat.as_ptr(), (*IfEntry).dwSpeed);
            SendDlgItemMessageW(hwndDlg, (IDC_SPEED as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
        }
    } else if (*IfEntry).dwSpeed < 1000000 {
        if LoadStringW(netshell_hInstance, IDS_FORMAT_KBIT, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
            _swprintf(szBuffer.as_mut_ptr(), szFormat.as_ptr(), (*IfEntry).dwSpeed / 1000);
            SendDlgItemMessageW(hwndDlg, (IDC_SPEED as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
        }
    } else if (*IfEntry).dwSpeed < 1000000000 {
        if LoadStringW(netshell_hInstance, IDS_FORMAT_MBIT, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
            _swprintf(szBuffer.as_mut_ptr(), szFormat.as_ptr(), (*IfEntry).dwSpeed / 1000000);
            SendDlgItemMessageW(hwndDlg, (IDC_SPEED as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
        }
    } else {
        if LoadStringW(netshell_hInstance, IDS_FORMAT_GBIT, szFormat.as_mut_ptr(), (core::mem::size_of_val(&szFormat) / core::mem::size_of::<u16>()) as i32) != 0 {
            _swprintf(szBuffer.as_mut_ptr(), szFormat.as_ptr(), (*IfEntry).dwSpeed / 1000000000);
            SendDlgItemMessageW(hwndDlg, (IDC_SPEED as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
        }
    }

    if !StrFormatByteSizeW(((*IfEntry).dwInOctets) as LONGLONG, szBuffer.as_mut_ptr(), _countof!(szBuffer)).is_null() {
        SendDlgItemMessageW(hwndDlg, (IDC_RECEIVED as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
    }

    if !StrFormatByteSizeW(((*IfEntry).dwOutOctets) as LONGLONG, szBuffer.as_mut_ptr(), _countof!(szBuffer)).is_null() {
        SendDlgItemMessageW(hwndDlg, (IDC_SEND as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);
    }

    Ticks = GetTickCount();

    DurationSeconds = Ticks / 1000;
    TimeConnected.wSecond = (DurationSeconds % 60) as WORD;
    TimeConnected.wMinute = ((DurationSeconds / 60) % 60) as WORD;
    TimeConnected.wHour = ((DurationSeconds / (60 * 60)) % 24) as WORD;
    TimeConnected.wDay = (DurationSeconds / (60 * 60 * 24)) as WORD;

    if GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(TimeConnected), CPP_COM_WSTR_41594B820641.as_ptr(), LocBuffer.as_mut_ptr(), (core::mem::size_of_val(&LocBuffer) / core::mem::size_of_val(&LocBuffer[0])) as i32) == 0 {
        return;
    }

    if TimeConnected.wDay == 0 {
        SendDlgItemMessageW(hwndDlg, (IDC_DURATION as i32), WM_SETTEXT, 0, LocBuffer.as_mut_ptr() as LPARAM);
    } else {
        if TimeConnected.wDay == 1 {
            if LoadStringW(netshell_hInstance, IDS_DURATION_DAY, DayBuffer.as_mut_ptr(), (core::mem::size_of_val(&DayBuffer) / core::mem::size_of_val(&DayBuffer[0])) as i32) == 0 {
                DayBuffer[0] = 0x0000u16;
            }
        } else {
            if LoadStringW(netshell_hInstance, IDS_DURATION_DAYS, DayBuffer.as_mut_ptr(), (core::mem::size_of_val(&DayBuffer) / core::mem::size_of_val(&DayBuffer[0])) as i32) == 0 {
                DayBuffer[0] = 0x0000u16;
            }
        }
        _swprintf(Buffer.as_mut_ptr(), DayBuffer.as_ptr(), TimeConnected.wDay as u32, LocBuffer.as_ptr());
        SendDlgItemMessageW(hwndDlg, (IDC_DURATION as i32), WM_SETTEXT, 0, Buffer.as_mut_ptr() as LPARAM);
    }
}

unsafe extern "C" fn UpdateNotificationIcon(ctx: &mut LANSTATUSUI_CONTEXT, mut IconRes: UINT) {
    let mut pProps: *mut NETCON_PROPERTIES = core::ptr::null_mut();
    if (((*ctx.pNet).GetProperties(core::ptr::addr_of_mut!(pProps))) < 0) {
        return;
    }
    if !(IconRes != 0) {
        match (*pProps).Status {
            NCS_CONNECTING | NCS_CONNECTED | NCS_AUTHENTICATING | NCS_AUTHENTICATION_SUCCEEDED => {
                IconRes = IDI_NET_IDLE;
            }
            _ => {
                IconRes = IDI_NET_OFF;
            }
        }
    }

    let IconSize: UINT = 32; // FIXME: DPI scaled 16px based on taskbar window or Shell_NotifyIconGetRect
    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
    nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
    nid.uID = ctx.uID;
    nid.hWnd = ctx.hwndStatusDlg;
    nid.__bindgen_anon_1.uVersion = NOTIFYICON_VERSION;
    nid.uFlags = NIF_STATE;
    nid.dwState = 0;
    nid.dwStateMask = NIS_HIDDEN;
    if ((*pProps).dwCharacter & NCCF_SHOW_ICON) != 0 {
        nid.uFlags |= NIF_TIP;
        lstrcpynW(core::ptr::addr_of_mut!(nid.szTip).cast(), (*pProps).pszwName, _countof!({ nid.szTip }) as i32);

        nid.hIcon = LoadImage(netshell_hInstance, MAKEINTRESOURCE!(IconRes), IMAGE_ICON, IconSize as i32, IconSize as i32, LR_SHARED) as HICON;
        if !nid.hIcon.is_null() {
            nid.uFlags |= NIF_ICON;
        }
    } else {
        nid.dwState |= NIS_HIDDEN;
    }
    Shell_NotifyIconW(NIM_MODIFY, core::ptr::addr_of_mut!(nid));
    NcFreeNetconProperties(pProps);
}

unsafe extern "C" fn UpdateLanStatus(hwndDlg: HWND, pContext: *mut LANSTATUSUI_CONTEXT) {
    let mut IfEntry: MIB_IFROW = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(IfEntry) as *mut core::ffi::c_void,
        core::mem::size_of::<MIB_IFROW>(),
    );
    IfEntry.dwIndex = (*pContext).dwAdapterIndex;
    if GetIfEntry(core::ptr::addr_of_mut!(IfEntry)) != NO_ERROR {
        return;
    }

    if (*pContext).Status == (1 as UINT).wrapping_neg() {
        /*
         * On first execution, pContext->dw[In|Out]Octets will be zero while
         * the interface info is already refreshed with non-null data, so a
         * gap is normal and does not correspond to an effective TX or RX packet.
         */
        (*pContext).dwInOctets = IfEntry.dwInOctets;
        (*pContext).dwOutOctets = IfEntry.dwOutOctets;
    }

    let mut IconRes: UINT = 0;
    if IfEntry.dwOperStatus == (MIB_IF_OPER_STATUS_CONNECTED as u32) || IfEntry.dwOperStatus == (MIB_IF_OPER_STATUS_OPERATIONAL as u32) {
        if (*pContext).dwInOctets == IfEntry.dwInOctets
            && (*pContext).dwOutOctets == IfEntry.dwOutOctets
            && (*pContext).Status != 0
        {
            IconRes = IDI_NET_IDLE;
            (*pContext).Status = 0;
        } else if (*pContext).dwInOctets != IfEntry.dwInOctets
            && (*pContext).dwOutOctets != IfEntry.dwOutOctets
            && (*pContext).Status != 1
        {
            IconRes = IDI_NET_TRANSREC;
            (*pContext).Status = 1;
        } else if (*pContext).dwInOctets != IfEntry.dwInOctets
            && (*pContext).Status != 2
        {
            IconRes = IDI_NET_REC;
            (*pContext).Status = 2;
        } else if (*pContext).dwOutOctets != IfEntry.dwOutOctets
            && (*pContext).Status != 3
        {
            IconRes = IDI_NET_TRANS;
            (*pContext).Status = 3;
        }
    } else if IfEntry.dwOperStatus == MIB_IF_OPER_STATUS_UNREACHABLE || IfEntry.dwOperStatus == MIB_IF_OPER_STATUS_DISCONNECTED {
        if (*pContext).Status != 4 {
            IconRes = IDI_NET_OFF;
            (*pContext).Status = 4;
        }
    } else if IfEntry.dwOperStatus == MIB_IF_OPER_STATUS_NON_OPERATIONAL {
        if (*pContext).Status != 5 {
            IconRes = IDI_NET_OFF;
            (*pContext).Status = 5;
        }
    }

    if !hwndDlg.is_null() && IconRes != 0 {
        let mut hBigIcon: HICON = LoadImage(
            netshell_hInstance,
            MAKEINTRESOURCE!(IconRes),
            IMAGE_ICON,
            0,
            0,
            LR_SHARED | LR_DEFAULTSIZE,
        ) as HICON;
        hBigIcon = SendDlgItemMessageW(
            hwndDlg,
            (IDC_NETSTAT as i32),
            STM_SETICON,
            hBigIcon as WPARAM,
            0,
        ) as HICON;
        if !hBigIcon.is_null() {
            DestroyIcon(hBigIcon);
        }
    }

    if IconRes != 0 {
        UpdateNotificationIcon(&mut *pContext, IconRes);
    }

    (*pContext).dwInOctets = IfEntry.dwInOctets;
    (*pContext).dwOutOctets = IfEntry.dwOutOctets;

    if !hwndDlg.is_null() {
        UpdateLanStatusUiDlg(hwndDlg, core::ptr::addr_of_mut!(IfEntry), pContext);
    }
}
#[no_mangle]
pub unsafe extern "C" fn InitializeLANStatusUiDlg(hwndDlg: HWND, pContext: *mut LANSTATUSUI_CONTEXT) {
    let mut szBuffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut pProperties: *mut NETCON_PROPERTIES = core::ptr::null_mut();

    if (*(*pContext).pNet).GetProperties(core::ptr::addr_of_mut!(pProperties)) != S_OK {
        return;
    }

    if (*pProperties).Status == NCS_DISCONNECTED {
        LoadStringW(netshell_hInstance, IDS_STATUS_UNREACHABLE, szBuffer.as_mut_ptr(), (MAX_PATH as i32));
    } else if (*pProperties).Status == NCS_MEDIA_DISCONNECTED {
        LoadStringW(netshell_hInstance, IDS_STATUS_DISCONNECTED, szBuffer.as_mut_ptr(), (MAX_PATH as i32));
    } else if (*pProperties).Status == NCS_CONNECTING {
        LoadStringW(netshell_hInstance, IDS_STATUS_CONNECTING, szBuffer.as_mut_ptr(), (MAX_PATH as i32));
    } else if (*pProperties).Status == NCS_CONNECTED {
        LoadStringW(netshell_hInstance, IDS_STATUS_CONNECTED, szBuffer.as_mut_ptr(), (MAX_PATH as i32));
    }

    SendDlgItemMessageW(hwndDlg, (IDC_STATUS as i32), WM_SETTEXT, 0, szBuffer.as_mut_ptr() as LPARAM);

    (*pContext).dwInOctets = 0;
    (*pContext).dwOutOctets = 0;

    if ((*pProperties).dwCharacter & NCCF_SHOW_ICON) == 0 {
        SetTimer(hwndDlg, (NETTIMERID) as UINT_PTR, (NETTIMERINTERVAL) as UINT, None);
    }

    /* update adapter info */
    (*pContext).Status = (1 as u32).wrapping_neg();
    UpdateLanStatus(hwndDlg, pContext);
    NcFreeNetconProperties(pProperties);
}

unsafe extern "C" fn InsertColumnToListView(hDlgCtrl: HWND, ResId: UINT, SubItem: UINT, Size: UINT) {
    let mut szBuffer: [u16; 200] = [0u16; 200];
    let mut lc: LVCOLUMNW = core::mem::zeroed();

    if LoadStringW(netshell_hInstance, ResId, szBuffer.as_mut_ptr(), 200 as i32) == 0 {
        return;
    }

    memset(core::ptr::addr_of_mut!(lc) as *mut core::ffi::c_void, 0 as i32, core::mem::size_of::<LV_COLUMN>() as usize);
    lc.mask = LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM | LVCF_FMT;
    lc.iSubItem = SubItem as i32;
    lc.fmt = (LVCFMT_FIXED_WIDTH as i32);
    lc.cx = Size as i32;
    lc.cchTextMax = wcslen(szBuffer.as_ptr()) as i32;
    lc.pszText = szBuffer.as_mut_ptr();

    let _ = SendMessageW(hDlgCtrl, LVM_INSERTCOLUMNW, SubItem as WPARAM, core::ptr::addr_of_mut!(lc) as LPARAM);
}

unsafe extern "C" fn AddIPAddressToListView(hDlgCtrl: HWND, pAddr: PIP_ADDR_STRING, mut Index: INT) {
    let mut li: LVITEMW = core::mem::zeroed();
    let mut pCur: PIP_ADDR_STRING;
    let mut szBuffer: [WCHAR; 100] = [0; 100];
    let mut SubIndex: UINT;

    ZeroMemory!(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
    li.mask = LVIF_TEXT;
    li.iItem = Index;
    pCur = pAddr;
    SubIndex = 0;

    loop {
        if SubIndex != 0 {
            ZeroMemory!(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
            li.mask = LVIF_TEXT;
            li.iItem = Index;
            li.iSubItem = 0;
            li.pszText = CPP_COM_WSTR_96A296D224F2.as_ptr() as LPWSTR;
            li.iItem = SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM) as INT;
        }

        if MultiByteToWideChar(CP_ACP, 0, (*pCur).IpAddress.String.as_ptr() as *const i8, -1, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32) != 0
        {
            li.pszText = szBuffer.as_mut_ptr();
            li.iSubItem = 1;
            li.iItem = Index;
            Index += 1;
            SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
        }
        SubIndex += 1;
        pCur = (*pCur).Next;

        if !(!pCur.is_null() && (*pCur).IpAddress.String[0] != 0) {
            break;
        }
    }
}

unsafe extern "C" fn InsertItemToListView(hDlgCtrl: HWND, ResId: UINT) -> INT {
    let mut li: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [WCHAR; 100] = [0u16; 100];

    ZeroMemory!(core::ptr::addr_of_mut!(li) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>() as usize);
    li.mask = LVIF_TEXT;
    li.iItem = ListView_GetItemCount!(hDlgCtrl);
    if LoadStringW(netshell_hInstance, ResId, szBuffer.as_mut_ptr(), (core::mem::size_of::<[WCHAR; 100]>() / core::mem::size_of::<WCHAR>()) as i32) != 0
    {
        li.pszText = szBuffer.as_mut_ptr();
        return SendMessageW(hDlgCtrl, LVM_INSERTITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM) as INT;
    }
    return -1;
}

unsafe extern "C" fn tmToStr(pTM: *mut tm, szBuffer: LPWSTR, nBufferSize: UINT) -> BOOL {
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut strBufferDate = CStringW::new();
    let mut strBufferTime = CStringW::new();
    let mut nCharDate: UINT;
    let mut nCharTime: UINT;
    let mut bResult: BOOL = (FALSE as i32);

    st.wYear = ((*pTM).tm_year + 1900) as WORD;
    st.wMonth = ((*pTM).tm_mon + 1) as WORD;
    st.wDay = (*pTM).tm_mday as WORD;
    st.wHour = (*pTM).tm_hour as WORD;
    st.wMinute = (*pTM).tm_min as WORD;
    st.wSecond = (*pTM).tm_sec as WORD;

    /* Check required size before cpy/cat */
    nCharDate = (GetDateFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null_mut(), core::ptr::null_mut(), 0) + 1) as UINT;
    nCharTime = (GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null_mut(), core::ptr::null_mut(), 0) + 1) as UINT;

    if GetDateFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null_mut(), strBufferDate.GetBuffer((nCharDate) as i32), nCharDate as i32) != 0 &&
        GetTimeFormatW(LOCALE_USER_DEFAULT, 0, core::ptr::addr_of_mut!(st), core::ptr::null_mut(), strBufferTime.GetBuffer((nCharTime) as i32), nCharTime as i32) != 0
    {
        StringCbCopy(szBuffer, nBufferSize as usize, strBufferDate.GetString());
        StringCbCat(szBuffer, nBufferSize as usize, CPP_COM_WSTR_8D71B3FAAB82.as_ptr());
        StringCbCat(szBuffer, nBufferSize as usize, strBufferTime.GetString());
        bResult = (TRUE as i32);
    }
    strBufferDate.ReleaseBuffer();
    strBufferTime.ReleaseBuffer();

    return bResult;
}
#[no_mangle]
pub unsafe extern "system" fn LANStatusUiDetailsDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pContext: *mut LANSTATUSUI_CONTEXT;
    let mut li: LVITEMW = core::mem::zeroed();
    let mut szBuffer: [u16; 100] = [0u16; 100];
    let mut pAdapterInfo: PIP_ADAPTER_INFO;
    let mut pCurAdapter: PIP_ADAPTER_INFO;
    let mut pPerAdapter: PIP_PER_ADAPTER_INFO;
    let mut dwSize: DWORD;
    let mut hDlgCtrl: HWND;
    let mut rect: RECT = core::mem::zeroed();

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                pContext = lParam as *mut LANSTATUSUI_CONTEXT;

                hDlgCtrl = GetDlgItem(hwndDlg, (IDC_DETAILS as i32));
                ListView_SetExtendedListViewStyleEx!(hDlgCtrl, LVS_EX_FULLROWSELECT, LVS_EX_FULLROWSELECT);

                /* Calculate column width */
                GetClientRect(hDlgCtrl, core::ptr::addr_of_mut!(rect));
                dwSize = rect.right as DWORD / 2;

                InsertColumnToListView(hDlgCtrl, IDS_PROPERTY, 0, dwSize);
                InsertColumnToListView(hDlgCtrl, IDS_VALUE, 1, dwSize);

                dwSize = 0;
                pCurAdapter = core::ptr::null_mut();
                pAdapterInfo = core::ptr::null_mut();
                if GetAdaptersInfo(core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize)) == ERROR_BUFFER_OVERFLOW
                {
                    pAdapterInfo = CoTaskMemAlloc((dwSize) as SIZE_T) as PIP_ADAPTER_INFO;
                    if !pAdapterInfo.is_null()
                    {
                        if GetAdaptersInfo(pAdapterInfo, core::ptr::addr_of_mut!(dwSize)) == NO_ERROR
                        {
                            pCurAdapter = pAdapterInfo;
                            while !pCurAdapter.is_null() && (*pCurAdapter).Index != (*pContext).dwAdapterIndex
                            {
                                pCurAdapter = (*pCurAdapter).Next;
                            }

                            if (*pCurAdapter).Index != (*pContext).dwAdapterIndex
                            {
                                pCurAdapter = core::ptr::null_mut();
                            }
                        }
                    }
                }

                ZeroMemory!(core::ptr::addr_of_mut!(li) as *mut c_void, core::mem::size_of::<LVITEMW>());
                li.mask = LVIF_TEXT;
                li.iSubItem = 1;
                li.pszText = szBuffer.as_mut_ptr();

                if !pCurAdapter.is_null()
                {
                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_PHYSICAL_ADDRESS);
                    if li.iItem >= 0
                    {
                        _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_31C1CCDE73D9.as_ptr(), (*pCurAdapter).Address[0] as core::ffi::c_uint, (*pCurAdapter).Address[1] as core::ffi::c_uint,
                                  (*pCurAdapter).Address[2] as core::ffi::c_uint, (*pCurAdapter).Address[3] as core::ffi::c_uint, (*pCurAdapter).Address[4] as core::ffi::c_uint, (*pCurAdapter).Address[5] as core::ffi::c_uint);
                        SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                    }
                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_IP_ADDRESS);
                    if li.iItem >= 0
                        && MultiByteToWideChar(CP_ACP, 0, (*pCurAdapter).IpAddressList.IpAddress.String.as_ptr(), -1, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0
                    {
                        SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                    }

                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_SUBNET_MASK);
                    if li.iItem >= 0
                        && MultiByteToWideChar(CP_ACP, 0, (*pCurAdapter).IpAddressList.IpMask.String.as_ptr(), -1, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0
                    {
                        SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                    }

                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_DEF_GATEWAY);
                    if li.iItem >= 0 && (*pCurAdapter).GatewayList.IpAddress.String[0] != b'0' as i8
                    {
                        if MultiByteToWideChar(CP_ACP, 0, (*pCurAdapter).GatewayList.IpAddress.String.as_ptr(), -1, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0
                        {
                            SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                        }
                    }

                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_DHCP_SERVER);
                    if li.iItem >= 0 && (*pCurAdapter).DhcpServer.IpAddress.String[0] != b'0' as i8
                    {
                        if MultiByteToWideChar(CP_ACP, 0, (*pCurAdapter).DhcpServer.IpAddress.String.as_ptr(), -1, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<u16>()) as i32) != 0
                        {
                            SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                        }
                    }

                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_LEASE_OBTAINED);
                    if li.iItem >= 0 && (*pCurAdapter).LeaseObtained != 0
                    {
                        let mut leaseOptained: *mut tm;

                        leaseOptained = localtime(core::ptr::addr_of_mut!((*pCurAdapter).LeaseObtained));

                        if tmToStr(leaseOptained, szBuffer.as_mut_ptr(), _countof!(szBuffer)) != 0
                        {
                            SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                        }
                    }

                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_LEASE_EXPIRES);
                    if li.iItem >= 0 && (*pCurAdapter).LeaseExpires != 0
                    {
                        let mut leaseExpire: *mut tm;

                        leaseExpire = localtime(core::ptr::addr_of_mut!((*pCurAdapter).LeaseExpires));

                        if tmToStr(leaseExpire, szBuffer.as_mut_ptr(), _countof!(szBuffer)) != 0
                        {
                            SendMessageW(hDlgCtrl, LVM_SETITEMW, 0, core::ptr::addr_of_mut!(li) as LPARAM);
                        }
                    }
                }

                dwSize = 0;
                li.iItem = InsertItemToListView(hDlgCtrl, IDS_DNS_SERVERS);
                if GetPerAdapterInfo((*pContext).dwAdapterIndex, core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize)) == ERROR_BUFFER_OVERFLOW
                {
                    pPerAdapter = CoTaskMemAlloc((dwSize) as SIZE_T) as PIP_PER_ADAPTER_INFO;
                    if !pPerAdapter.is_null()
                    {
                        if GetPerAdapterInfo((*pContext).dwAdapterIndex, pPerAdapter, core::ptr::addr_of_mut!(dwSize)) == ERROR_SUCCESS
                        {
                            if li.iItem >= 0
                            {
                                AddIPAddressToListView(hDlgCtrl, core::ptr::addr_of_mut!((*pPerAdapter).DnsServerList), li.iItem);
                            }
                        }
                        CoTaskMemFree(pPerAdapter as _);
                    }
                }

                if !pCurAdapter.is_null()
                {
                    li.iItem = InsertItemToListView(hDlgCtrl, IDS_WINS_SERVERS);
                    if (*pCurAdapter).HaveWins != 0
                    {
                        AddIPAddressToListView(hDlgCtrl, core::ptr::addr_of_mut!((*pCurAdapter).PrimaryWinsServer), li.iItem);
                        AddIPAddressToListView(hDlgCtrl, core::ptr::addr_of_mut!((*pCurAdapter).SecondaryWinsServer), li.iItem + 1);
                    }
                }

                CoTaskMemFree(pAdapterInfo as _);
                break 'sw;
            }

            WM_COMMAND => {
                if LOWORD!(wParam) == (IDCANCEL as u16)
                {
                    EndDialog(hwndDlg, (FALSE) as INT_PTR);
                    break 'sw;
                }
            }

            _ => {}
        }
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "system" fn LANStatusUiAdvancedDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut szBuffer: [WCHAR; 100] = [0; 100];
    let mut page: *mut PROPSHEETPAGE = core::ptr::null_mut();
    let mut pContext: *mut LANSTATUSUI_CONTEXT = core::ptr::null_mut();
    let mut dwIpAddr: DWORD = 0;

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as *mut PROPSHEETPAGE;
            pContext = (*page).lParam as *mut LANSTATUSUI_CONTEXT;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pContext as LONG_PTR);
            if (*pContext).DHCPEnabled != 0 {
                LoadStringW(netshell_hInstance, IDS_ASSIGNED_DHCP, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32);
            } else {
                LoadStringW(netshell_hInstance, IDS_ASSIGNED_MANUAL, szBuffer.as_mut_ptr(), (core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) as i32);
            }

            szBuffer[(core::mem::size_of_val(&szBuffer) / core::mem::size_of::<WCHAR>()) - 1] = 0x0000u16;
            SendDlgItemMessageW(hwndDlg, (IDC_DETAILSTYPE as i32), WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);

            dwIpAddr = ntohl((*pContext).IpAddress);
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_03971BB4A692.as_ptr(), FIRST_IPADDRESS!(dwIpAddr), SECOND_IPADDRESS!(dwIpAddr),
                      THIRD_IPADDRESS!(dwIpAddr), FOURTH_IPADDRESS!(dwIpAddr));
            SendDlgItemMessageW(hwndDlg, (IDC_DETAILSIP as i32), WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);

            dwIpAddr = ntohl((*pContext).SubnetMask);
            _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_03971BB4A692.as_ptr(), FIRST_IPADDRESS!(dwIpAddr), SECOND_IPADDRESS!(dwIpAddr),
                      THIRD_IPADDRESS!(dwIpAddr), FOURTH_IPADDRESS!(dwIpAddr));
            SendDlgItemMessageW(hwndDlg, (IDC_DETAILSSUBNET as i32), WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);

            dwIpAddr = ntohl((*pContext).Gateway);
            if dwIpAddr != 0 {
                _swprintf(szBuffer.as_mut_ptr(), CPP_COM_WSTR_03971BB4A692.as_ptr(), FIRST_IPADDRESS!(dwIpAddr), SECOND_IPADDRESS!(dwIpAddr),
                          THIRD_IPADDRESS!(dwIpAddr), FOURTH_IPADDRESS!(dwIpAddr));
                SendDlgItemMessageW(hwndDlg, (IDC_DETAILSGATEWAY as i32), WM_SETTEXT, 0, szBuffer.as_ptr() as LPARAM);
            }
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_DETAILS as u16) {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if !pContext.is_null() {
                    DialogBoxParamW(netshell_hInstance, MAKEINTRESOURCEW!(IDD_LAN_NETSTATUSDETAILS), GetParent(hwndDlg),
                                    Some(LANStatusUiDetailsDlg), pContext as LPARAM);
                }
            }
            // break;
        }
        _ => {
            // break;
        }
    }
    (FALSE) as INT_PTR
}
#[no_mangle]
pub unsafe extern "C" fn DisableNetworkAdapter(pNet: *mut INetConnection, pContext: *mut LANSTATUSUI_CONTEXT, hwndDlg: HWND) {
    let hr: HRESULT = (((*(*pNet).lpVtbl).Disconnect).unwrap_unchecked())(pNet);
    if FAILED_UNEXPECTEDLY!(hr) != false {
        return;
    }

    let mut nid: NOTIFYICONDATAW = core::mem::zeroed();

    PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_CANCEL);
    ZeroMemory!(core::ptr::addr_of_mut!(nid) as *mut core::ffi::c_void, core::mem::size_of::<NOTIFYICONDATAW>());
    nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.uID = (*pContext).uID;
    nid.hWnd = (*pContext).hwndDlg;
    nid.uFlags = NIF_STATE;
    nid.dwState = NIS_HIDDEN;
    nid.dwStateMask = NIS_HIDDEN;

    Shell_NotifyIconW(NIM_MODIFY, core::ptr::addr_of_mut!(nid));
}
#[no_mangle]
pub unsafe extern "system" fn LANStatusUiDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut page: *mut PROPSHEETPAGE;
    let mut pContext: *mut LANSTATUSUI_CONTEXT;
    let mut lppsn: LPPSHNOTIFY;

    match uMsg {
        WM_INITDIALOG => {
            page = lParam as *mut PROPSHEETPAGE;
            pContext = (*page).lParam as *mut LANSTATUSUI_CONTEXT;
            (*pContext).hwndDlg = hwndDlg;
            SetWindowLongPtr(hwndDlg, DWLP_USER, pContext as LONG_PTR);
            InitializeLANStatusUiDlg(hwndDlg, pContext);
            return (TRUE) as INT_PTR;
        }
        WM_DESTROY => {
            KillTimer(hwndDlg, (NETTIMERID) as UINT_PTR);
            pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
            if !pContext.is_null() {
                (*pContext).hwndDlg = core::ptr::null_mut();
            }
            // break
        }
        WM_TIMER => {
            pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
            if !pContext.is_null() {
                UpdateLanStatus((*pContext).hwndDlg, pContext);
            }
            // break
        }
        WM_COMMAND => {
            pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
            if LOWORD!(wParam) == (IDC_STATUS_PROPERTIES as u16) {
                if !pContext.is_null() {
                    ShowNetConnectionProperties((*pContext).pNet, GetParent((*pContext).hwndDlg));
                    BringWindowToTop(GetParent((*pContext).hwndDlg));
                }
                // break
            } else if LOWORD!(wParam) == (IDC_ENDISABLE as u16) {
                DisableNetworkAdapter((*pContext).pNet, pContext, hwndDlg);
                // break
            } else {
                // fall through to WM_NOTIFY handling
                lppsn = lParam as LPPSHNOTIFY;
                if (*lppsn).hdr.code == (PSN_APPLY) as u32 || (*lppsn).hdr.code == (PSN_RESET) as u32 {
                    pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                    SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
                    (*pContext).hwndDlg = core::ptr::null_mut();
                    return (TRUE) as INT_PTR;
                }
                // break
            }
        }
        WM_NOTIFY => {
            lppsn = lParam as LPPSHNOTIFY;
            if (*lppsn).hdr.code == (PSN_APPLY) as u32 || (*lppsn).hdr.code == (PSN_RESET) as u32 {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                SetWindowLongPtr(hwndDlg, DWLP_MSGRESULT, PSNRET_NOERROR as LONG_PTR);
                (*pContext).hwndDlg = core::ptr::null_mut();
                return (TRUE) as INT_PTR;
            }
            // break
        }
        _ => {}
    }
    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn InitializePropertyDialog(pContext: *mut LANSTATUSUI_CONTEXT, pProperties: *mut NETCON_PROPERTIES) {
    let mut dwSize: DWORD;
    let mut dwAdapterIndex: DWORD = 0;
    let mut dwResult: DWORD;
    let mut pStr: LPWSTR = core::ptr::null_mut();
    let mut pAdapterInfo: *mut IP_ADAPTER_INFO;
    let mut pCurAdapter: *mut IP_ADAPTER_INFO;

    if ((StringFromCLSID(core::ptr::addr_of!((*pProperties).guidId) as *const CLSID, core::ptr::addr_of_mut!(pStr))) < 0)
    {
        return;
    }

    /* get the IfTable */
    dwSize = 0;
    dwResult = GetAdaptersInfo(core::ptr::null_mut(), core::ptr::addr_of_mut!(dwSize));
    if dwResult != ERROR_BUFFER_OVERFLOW
    {
        CoTaskMemFree(pStr as *mut core::ffi::c_void);
        return;
    }

    pAdapterInfo = CoTaskMemAlloc((dwSize) as SIZE_T) as PIP_ADAPTER_INFO;
    if pAdapterInfo.is_null()
    {
        CoTaskMemFree(pAdapterInfo as *mut core::ffi::c_void);
        CoTaskMemFree(pStr as *mut core::ffi::c_void);
        return;
    }

    if GetAdaptersInfo(pAdapterInfo, core::ptr::addr_of_mut!(dwSize)) != NO_ERROR
    {
        CoTaskMemFree(pAdapterInfo as *mut core::ffi::c_void);
        CoTaskMemFree(pStr as *mut core::ffi::c_void);
        return;
    }

    if ((GetAdapterIndexFromNetCfgInstanceId(pAdapterInfo, pStr, core::ptr::addr_of_mut!(dwAdapterIndex))) == 0)
    {
        CoTaskMemFree(pAdapterInfo as *mut core::ffi::c_void);
        CoTaskMemFree(pStr as *mut core::ffi::c_void);
        return;
    }

    pCurAdapter = pAdapterInfo;
    while (*pCurAdapter).Index != dwAdapterIndex
    {
        pCurAdapter = (*pCurAdapter).Next;
    }

    (*pContext).IpAddress = inet_addr(core::ptr::addr_of!((*pCurAdapter).IpAddressList.IpAddress.String) as *const i8);
    (*pContext).SubnetMask = inet_addr(core::ptr::addr_of!((*pCurAdapter).IpAddressList.IpMask.String) as *const i8);
    (*pContext).Gateway = inet_addr(core::ptr::addr_of!((*pCurAdapter).GatewayList.IpAddress.String) as *const i8);
    (*pContext).DHCPEnabled = (*pCurAdapter).DhcpEnabled;
    CoTaskMemFree(pStr as *mut core::ffi::c_void);
    CoTaskMemFree(pAdapterInfo as *mut core::ffi::c_void);
    (*pContext).dwAdapterIndex = dwAdapterIndex;
}

unsafe extern "system" fn PropSheetProc__lanstatusui(hwndDlg: HWND, uMsg: UINT, lParam: LPARAM) -> i32 {
    // NOTE: This callback is needed to set large icon correctly.
    let mut hIcon: HICON;
    match uMsg {
        PSCB_INITIALIZED => {
            hIcon = LoadIconW(netshell_hInstance, MAKEINTRESOURCEW!(IDI_NET_IDLE));
            SendMessageW(hwndDlg, WM_SETICON, (ICON_BIG) as WPARAM, hIcon as LPARAM);
        }
        _ => {}
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn ShowStatusPropertyDialog(pContext: *mut LANSTATUSUI_CONTEXT, hwndDlg: HWND) {
    let mut hppages: [HPROPSHEETPAGE; 2 as usize] = core::mem::zeroed();
    let mut pinfo: PROPSHEETHEADERW = core::mem::zeroed();
    let mut pProperties: *mut NETCON_PROPERTIES = core::ptr::null_mut();

    ZeroMemory!(core::ptr::addr_of_mut!(pinfo) as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETHEADERW>() as usize);
    ZeroMemory!(hppages.as_mut_ptr() as *mut core::ffi::c_void, core::mem::size_of_val(&hppages));
    pinfo.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as u32;
    pinfo.dwFlags = (PSH_NOCONTEXTHELP | PSH_PROPTITLE | PSH_NOAPPLYNOW |
                    PSH_USEICONID | PSH_USECALLBACK) as u32;
    pinfo.__bindgen_anon_3.phpage = hppages.as_mut_ptr();
    pinfo.hwndParent = hwndDlg;
    pinfo.hInstance = netshell_hInstance;
    pinfo.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_NET_IDLE);
    pinfo.pfnCallback = Some(PropSheetProc__lanstatusui);

    if (*(*pContext).pNet).GetProperties(core::ptr::addr_of_mut!(pProperties)) == S_OK {
        if !(*pProperties).pszwName.is_null() {
            pinfo.pszCaption = (*pProperties).pszwName;
            pinfo.dwFlags |= PSH_PROPTITLE as u32;
        }
        InitializePropertyDialog(pContext, pProperties);
        if (*pProperties).MediaType == NCM_LAN && (*pProperties).Status == NCS_CONNECTED {
            hppages[0 as usize] = InitializePropertySheetPage(
                MAKEINTRESOURCEW!(IDD_LAN_NETSTATUS),
                Some(LANStatusUiDlg),
                pContext as LPARAM,
                core::ptr::null_mut(),
            );
            if !hppages[0 as usize].is_null() {
                pinfo.nPages += 1;
            }

            hppages[pinfo.nPages as usize] = InitializePropertySheetPage(
                MAKEINTRESOURCEW!(IDD_LAN_NETSTATUSADVANCED),
                Some(LANStatusUiAdvancedDlg),
                pContext as LPARAM,
                core::ptr::null_mut(),
            );
            if !hppages[pinfo.nPages as usize].is_null() {
                pinfo.nPages += 1;
            }

            if pinfo.nPages != 0 {
                PropertySheetW(core::ptr::addr_of_mut!(pinfo));
            }
        } else if (*pProperties).Status == NCS_MEDIA_DISCONNECTED || (*pProperties).Status == NCS_DISCONNECTED ||
                 (*pProperties).Status == NCS_HARDWARE_DISABLED {
            ShowNetConnectionProperties((*pContext).pNet, (*pContext).hwndDlg);
        }

        NcFreeNetconProperties(pProperties);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ShowNetworkIconContextMenu(hwndOwner: HWND, pContext: *mut LANSTATUSUI_CONTEXT) {
    #[repr(C)]
    struct MenuItem {
        uID: u32,
        uFlags: u32,
        uIDNewItem: UINT_PTR,
    }

    if pContext.is_null() || (*pContext).pNet.is_null() {
        return;
    }

    let hMenu: HMENU = CreatePopupMenu();
    if hMenu.is_null() {
        return;
    }

    let mut pt: POINT = core::mem::zeroed();
    GetCursorPos(core::ptr::addr_of_mut!(pt));

    SetForegroundWindow(hwndOwner);

    // The context menu items, set to their default values.
    let mut MenuItems: [MenuItem; 6] = [
        MenuItem { uID: IDS_NET_ACTIVATE, uFlags: MF_GRAYED, uIDNewItem: (IDM_NETICON_ENABLE as _) },
        MenuItem { uID: IDS_NET_STATUS, uFlags: MF_GRAYED, uIDNewItem: (IDM_NETICON_STATUS as _) },
        MenuItem { uID: IDS_NET_REPAIR, uFlags: MF_GRAYED, uIDNewItem: (IDM_NETICON_REPAIR as _) },
        MenuItem { uID: UINT_MAX, uFlags: 0, uIDNewItem: 0 }, // Separator
        MenuItem { uID: IDS_NET_OPEN_CONNECTIONS, uFlags: MF_ENABLED, uIDNewItem: (IDM_NETICON_OPEN_CONNECTIONS as _) },
        MenuItem { uID: IDS_NET_PROPERTIES, uFlags: MF_ENABLED | MFS_DEFAULT, uIDNewItem: (IDM_NETICON_PROPERTIES as _) },
    ];

    let mut pProps: *mut NETCON_PROPERTIES = core::ptr::null_mut();
    let hr: HRESULT = (*(*pContext).pNet).GetProperties(core::ptr::addr_of_mut!(pProps));
    if ((hr) >= 0) && !pProps.is_null() {
        if (*pProps).Status == NCS_HARDWARE_DISABLED ||
            (*pProps).Status == NCS_MEDIA_DISCONNECTED ||
            (*pProps).Status == NCS_DISCONNECTED
        {
            MenuItems[0 as usize].uID = IDS_NET_ACTIVATE;
            MenuItems[0 as usize].uFlags = MF_ENABLED | MFS_DEFAULT;
            MenuItems[0 as usize].uIDNewItem = (IDM_NETICON_ENABLE as _);
            MenuItems[5 as usize].uFlags = MF_ENABLED;
        } else {
            MenuItems[0 as usize].uID = IDS_NET_DEACTIVATE;
            MenuItems[0 as usize].uFlags = MF_ENABLED;
            MenuItems[0 as usize].uIDNewItem = (IDM_NETICON_DISABLE as _);
        }

        if (*pProps).Status == NCS_CONNECTED {
            MenuItems[1 as usize].uFlags = MF_ENABLED;
            MenuItems[2 as usize].uFlags = MF_ENABLED;
        } else if (*pProps).Status == NCS_CONNECTING {
            MenuItems[1 as usize].uFlags = MF_ENABLED;
            MenuItems[2 as usize].uFlags = MF_GRAYED;
        } else {
            MenuItems[1 as usize].uFlags = MF_GRAYED;
            MenuItems[2 as usize].uFlags = MF_GRAYED;
        }

        NcFreeNetconProperties(pProps);
        pProps = core::ptr::null_mut();
    } else {
        MenuItems[0 as usize].uFlags = MF_GRAYED;
        MenuItems[1 as usize].uFlags = MF_GRAYED;
        MenuItems[2 as usize].uFlags = MF_GRAYED;
        MenuItems[5 as usize].uFlags = MF_GRAYED;
    }

    // Set the "Properties" item as default, if the Network "Enable/Disable" item isn't.
    if (MenuItems[0 as usize].uFlags & MFS_DEFAULT) == 0 {
        MenuItems[5 as usize].uFlags |= MFS_DEFAULT;
    }

    let mut szMenuItem: [u16; 128 as usize] = [0u16; 128 as usize];

    let mut i: u16 = 0 as u16;
    while i < _countof!(MenuItems) as u16 {
        if MenuItems[i as usize].uID != UINT_MAX {
            if LoadStringW(netshell_hInstance, MenuItems[i as usize].uID, szMenuItem.as_mut_ptr(), _countof!(szMenuItem) as i32) != 0 {
                AppendMenuW(hMenu, MF_STRING | MenuItems[i as usize].uFlags, MenuItems[i as usize].uIDNewItem, szMenuItem.as_mut_ptr());
            }
        } else {
            AppendMenuW(hMenu, MF_SEPARATOR, 0, core::ptr::null());
        }
        i += 1;
    }

    TrackPopupMenuEx(hMenu, TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON, pt.x, pt.y, hwndOwner, core::ptr::null_mut());

    PostMessage(hwndOwner, WM_NULL, 0, 0);

    DestroyMenu(hMenu);
}

pub unsafe extern "C" fn RepairConnection(pNet: *mut INetConnection, hwndOwner: HWND) -> HRESULT {
    SHELL_ErrorBox(hwndOwner, (HRESULT_FROM_WIN32!(ERROR_NOT_SUPPORTED)) as u32);

    return E_NOTIMPL;
}

unsafe extern "C" fn ConfigureNotificationIconTimer(ctx: &mut LANSTATUSUI_CONTEXT) {
    let mut Enable: BOOL = (TRUE as i32);
    let mut pNCP: *mut NETCON_PROPERTIES = core::ptr::null_mut();
    if (((*ctx.pNet).GetProperties(core::ptr::addr_of_mut!(pNCP))) >= 0) {
        Enable = ((*pNCP).dwCharacter & NCCF_SHOW_ICON) as BOOL;
        NcFreeNetconProperties(pNCP);
    }

    if Enable != 0 {
        ctx.nIDEvent = SetTimer(
            ctx.hwndStatusDlg,
            (NETTIMERID) as UINT_PTR,
            (NETTIMERINTERVAL) as UINT,
            None,
        );
    } else if ctx.nIDEvent != 0 {
        KillTimer(ctx.hwndStatusDlg, ctx.nIDEvent);
        ctx.nIDEvent = 0;
    }
    UpdateNotificationIcon(ctx, 0);
}

#[no_mangle]
pub unsafe extern "system" fn LANStatusDlg(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pContext: *mut LANSTATUSUI_CONTEXT = core::ptr::null_mut();

    'outer: {
        match uMsg {
            WM_INITDIALOG => {
                pContext = lParam as *mut LANSTATUSUI_CONTEXT;
                SetWindowLongPtr(hwndDlg, DWLP_USER, lParam as LONG_PTR);
                (*pContext).hwndStatusDlg = hwndDlg;
                PostMessage(hwndDlg, WM_CONFIGURETRAYICON, WPARAM_MAGIC as WPARAM, 0); // Delay timer until after NIM_ADD
                return (TRUE) as INT_PTR;
            }

            WM_DESTROY => {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if !pContext.is_null() && (*pContext).nIDEvent != 0 {
                    KillTimer(hwndDlg, (*pContext).nIDEvent);
                    (*pContext).nIDEvent = 0;
                }
                SetWindowLongPtr(hwndDlg, DWLP_USER, core::ptr::null_mut::<u8>() as LONG_PTR);
                break 'outer;
            }

            WM_TIMER => {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if wParam == (*pContext).nIDEvent as WPARAM {
                    UpdateLanStatus((*pContext).hwndDlg, pContext);
                }
                break 'outer;
            }

            WM_CONFIGURETRAYICON => {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if !pContext.is_null() && wParam == (WPARAM_MAGIC as WPARAM) {
                    ConfigureNotificationIconTimer(&mut *pContext);
                }
                break 'outer;
            }

            WM_SHOWSTATUSDLG => {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if pContext.is_null() {
                    break 'outer;
                }

                'sw: {
                    match (LOWORD!(lParam as DWORD)) as u32 {
                        WM_LBUTTONUP => {
                            if !(*pContext).hwndDlg.is_null() {
                                let hwndSheet: HWND = GetParent((*pContext).hwndDlg);
                                if !hwndSheet.is_null() {
                                    ShowWindow(hwndSheet, (SW_RESTORE as i32));
                                    SetForegroundWindow(hwndSheet);
                                    BringWindowToTop(hwndSheet);
                                }
                            } else {
                                ShowStatusPropertyDialog(pContext, hwndDlg);
                            }
                            break 'sw;
                        }
                        WM_RBUTTONUP | WM_CONTEXTMENU => {
                            ShowNetworkIconContextMenu(hwndDlg, pContext);
                            break 'sw;
                        }
                        _ => {}
                    }
                }
                // Unreachable in C: statements placed after `break;` before the
                // following case labels. Preserved as dead code.
                if false {
                    if !(*pContext).hwndDlg.is_null() {
                        ShowWindow(GetParent((*pContext).hwndDlg), (SW_SHOW as i32));
                        BringWindowToTop(GetParent((*pContext).hwndDlg));
                    } else {
                        ShowStatusPropertyDialog(pContext, hwndDlg);
                    }
                }
                break 'outer;
            }

            WM_COMMAND => {
                pContext = GetWindowLongPtr(hwndDlg, DWLP_USER) as *mut LANSTATUSUI_CONTEXT;
                if pContext.is_null() || (*pContext).pNet.is_null() {
                    break 'outer;
                }

                'sw: {
                    match (LOWORD!(wParam as DWORD)) as u32 {
                        IDM_NETICON_ENABLE => {
                            (*(*pContext).pNet).Connect();
                            UpdateLanStatus(core::ptr::null_mut(), pContext);
                            break 'sw;
                        }

                        IDM_NETICON_DISABLE => {
                            (*(*pContext).pNet).Disconnect();
                            UpdateLanStatus(core::ptr::null_mut(), pContext);
                            break 'sw;
                        }

                        IDM_NETICON_STATUS => {
                            if !(*pContext).hwndDlg.is_null() {
                                let hwndSheet: HWND = GetParent((*pContext).hwndDlg);
                                if !hwndSheet.is_null() {
                                    ShowWindow(hwndSheet, (SW_RESTORE as i32));
                                    SetForegroundWindow(hwndSheet);
                                    BringWindowToTop(hwndSheet);
                                }
                            } else {
                                ShowStatusPropertyDialog(pContext, hwndDlg);
                            }
                            break 'sw;
                        }

                        IDM_NETICON_REPAIR => {
                            RepairConnection((*pContext).pNet, hwndDlg);
                            break 'sw;
                        }

                        IDM_NETICON_PROPERTIES => {
                            ShowNetConnectionProperties((*pContext).pNet, hwndDlg);
                            break 'sw;
                        }

                        IDM_NETICON_OPEN_CONNECTIONS => {
                            ShellExecuteW(hwndDlg, core::ptr::null_mut(), CPP_COM_WSTR_8AFAAE84FEDA.as_ptr(), CPP_COM_WSTR_CA983B81C2D2.as_ptr(), core::ptr::null_mut(), (SW_SHOWNORMAL) as INT);
                            break 'sw;
                        }
                        _ => {}
                    }
                }
                break 'outer;
            }
            _ => {}
        }
    }
    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn RefreshNetTaskbarNotificationIconEnumProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    SendMessageW(hWnd, WM_CONFIGURETRAYICON, (WPARAM_MAGIC) as WPARAM, 0);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn RefreshNetTaskbarNotificationIcon() {
    EnumNotificationIconWindows(Some(RefreshNetTaskbarNotificationIconEnumProc), 0);
}

impl CLanStatus {
    pub unsafe fn __ctor_0(&mut self) {
        let this_: *mut Self = self;
        (*this_).__init_vtables();
        (*this_).m_lpNetMan = CComPtr::null();
        (*this_).m_pHead = core::ptr::null_mut();
        {
        }
    }
    pub unsafe fn QueryStatus(&mut self, pguidCmdGroup: *const GUID, cCmds: ULONG, prgCmds: *mut OLECMD, pCmdText: *mut OLECMDTEXT) -> HRESULT {
        let this_: *mut Self = self;
        MessageBoxW(core::ptr::null_mut(), (*pCmdText).rgwz.as_ptr(), CPP_COM_WSTR_5FC51169506B.as_ptr(), MB_OK);
        return E_NOTIMPL;
    }
    pub unsafe fn Exec(&mut self, pguidCmdGroup: *const GUID, nCmdID: DWORD, nCmdexecopt: DWORD, pvaIn: *mut VARIANT, pvaOut: *mut VARIANT) -> HRESULT {
        let this_: *mut Self = self;
        if !pguidCmdGroup.is_null() {
            if IsEqualGUID!(pguidCmdGroup, &CGID_ShellServiceObject) != false {
                return (*this_).InitializeNetTaskbarNotifications();
            } else {
                /* invoke status dialog */
                return (*this_).ShowStatusDialogByCLSID(pguidCmdGroup);
            }
        }
        return S_OK;
    }
    pub unsafe fn InitializeNetTaskbarNotifications(&mut self) -> HRESULT {
        let this_: *mut Self = self;
        let mut nid: NOTIFYICONDATAW = core::mem::zeroed();
        let mut hwndDlg: HWND;
        let mut pNetConMan: CComPtr<INetConnectionManager> = CComPtr::new();
        let mut pEnumCon: CComPtr<IEnumNetConnection> = CComPtr::new();
        let mut pNetCon: CComPtr<INetConnection> = CComPtr::new();
        let mut pProps: *mut NETCON_PROPERTIES = core::mem::zeroed();
        let mut hr: HRESULT;
        let mut Count: ULONG = core::mem::zeroed();
        let mut Index: ULONG;
        let mut pItem: *mut NOTIFICATION_ITEM;
        let mut pLast: *mut NOTIFICATION_ITEM = core::ptr::null_mut();
        let mut pContext: *mut LANSTATUSUI_CONTEXT;

        if !(*this_).m_pHead.is_null() {
            pItem = (*this_).m_pHead;
            while !pItem.is_null() {
                hr = (*(*pItem).pNet).GetProperties(core::ptr::addr_of_mut!(pProps));
                if hr >= 0 {
                    ZeroMemory!(
                        core::ptr::addr_of_mut!(nid) as *mut core::ffi::c_void,
                        core::mem::size_of::<NOTIFYICONDATAW>(),
                    );
                    nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
                    nid.uID = (*pItem).uID;
                    nid.hWnd = (*pItem).hwndDlg;
                    nid.uFlags = NIF_STATE;
                    if ((*pProps).dwCharacter & NCCF_SHOW_ICON) != 0 {
                        nid.dwState = 0;
                    } else {
                        nid.dwState = NIS_HIDDEN;
                    }

                    nid.dwStateMask = NIS_HIDDEN;
                    Shell_NotifyIconW(NIM_MODIFY, core::ptr::addr_of_mut!(nid));
                    NcFreeNetconProperties((pProps as *mut core::ffi::c_void).cast());
                }
                pItem = (*pItem).pNext;
            }
            return S_OK;
        }
        /* get an instance to of IConnectionManager */
        hr = CNetConnectionManager_CreateInstance(
            &IID_INetConnectionManager,
            pNetConMan.GetAddressOf() as *mut *mut core::ffi::c_void,
        );
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        hr = (*pNetConMan.Get()).EnumConnections(NCME_DEFAULT, pEnumCon.GetAddressOf());
        if FAILED_UNEXPECTEDLY!(hr) {
            return hr;
        }

        Index = 1;
        loop {
            pNetCon.Release();
            hr = (*pEnumCon.Get()).Next(
                1,
                pNetCon.GetAddressOf(),
                core::ptr::addr_of_mut!(Count),
            );
            if hr != S_OK {
                break;
            }

            pItem = CoTaskMemAlloc(
                (core::mem::size_of::<NOTIFICATION_ITEM>() as ULONG) as SIZE_T,
            ) as *mut NOTIFICATION_ITEM;
            if pItem.is_null() {
                break;
            }

            pContext = CoTaskMemAlloc(
                (core::mem::size_of::<LANSTATUSUI_CONTEXT>() as ULONG) as SIZE_T,
            ) as *mut LANSTATUSUI_CONTEXT;
            if pContext.is_null() {
                CoTaskMemFree(pItem as *mut core::ffi::c_void);
                break;
            }

            ZeroMemory!(
                pContext as *mut core::ffi::c_void,
                core::mem::size_of::<LANSTATUSUI_CONTEXT>(),
            );
            (*pContext).uID = Index;
            (*pContext).pNet = pNetCon.Get();
            (*pContext).Status = (1 as u32).wrapping_neg();
            (*pContext).dwAdapterIndex = Index;
            (*pItem).uID = Index;
            (*pItem).pNext = core::ptr::null_mut();
            (*pItem).pNet = pNetCon.Get();
            (*(*pItem).pNet).AddRef();
            hwndDlg = CreateDialogParamW(
                netshell_hInstance,
                MAKEINTRESOURCEW!(IDD_STATUS),
                core::ptr::null_mut(),
                Some(LANStatusDlg),
                pContext as LPARAM,
            );
            if hwndDlg.is_null() {
                /* the dialog could not be created; skip this connection */
                continue;
            }

            ZeroMemory!(
                core::ptr::addr_of_mut!(nid) as *mut core::ffi::c_void,
                core::mem::size_of::<NOTIFYICONDATAW>(),
            );
            nid.cbSize = core::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
            nid.uID = Index;
            Index += 1;
            nid.uFlags = NIF_MESSAGE;
            nid.__bindgen_anon_1.uVersion = NOTIFYICON_VERSION;
            nid.uCallbackMessage = WM_SHOWSTATUSDLG;
            nid.hWnd = hwndDlg;

            hr = (*pNetCon.Get()).GetProperties(core::ptr::addr_of_mut!(pProps));
            if hr >= 0 {
                CopyMemory!(
                    core::ptr::addr_of_mut!((*pItem).guidItem) as *mut core::ffi::c_void,
                    core::ptr::addr_of!((*pProps).guidId) as *const core::ffi::c_void,
                    core::mem::size_of::<GUID>(),
                );
                if ((*pProps).dwCharacter & NCCF_SHOW_ICON) == 0 {
                    nid.dwState = NIS_HIDDEN;
                    nid.dwStateMask = NIS_HIDDEN;
                    nid.uFlags |= NIF_STATE;
                }
                if (*pProps).Status == NCS_MEDIA_DISCONNECTED
                    || (*pProps).Status == NCS_DISCONNECTED
                    || (*pProps).Status == NCS_HARDWARE_DISABLED
                {
                    nid.hIcon = LoadIcon(
                        netshell_hInstance,
                        MAKEINTRESOURCE!(IDI_NET_OFF),
                    );
                } else if (*pProps).Status == NCS_CONNECTED {
                    nid.hIcon = LoadIcon(
                        netshell_hInstance,
                        MAKEINTRESOURCE!(IDI_NET_IDLE),
                    );
                }

                if !nid.hIcon.is_null() {
                    nid.uFlags |= NIF_ICON;
                }

                wcscpy(
                    core::ptr::addr_of_mut!(nid.szTip) as *mut u16,
                    (*pProps).pszwName,
                );
                nid.uFlags |= NIF_TIP;
                NcFreeNetconProperties(pProps);
            }
            (*pContext).hwndStatusDlg = hwndDlg;
            (*pItem).hwndDlg = hwndDlg;

            if Shell_NotifyIconW(NIM_ADD, core::ptr::addr_of_mut!(nid)) != 0 {
                if !pLast.is_null() {
                    (*pLast).pNext = pItem;
                } else {
                    (*this_).m_pHead = pItem;
                }

                pLast = pItem;
                Index += 1;
            } else {
                /* the shell rejected the icon; release the item */
                CoTaskMemFree(pItem as *mut core::ffi::c_void);
            }

            if (nid.uFlags & NIF_ICON) != 0 {
                DestroyIcon(nid.hIcon);
            }
        }

        (*this_).m_lpNetMan.assign(pNetConMan.Get());
        return S_OK;
    }
    pub unsafe fn ShowStatusDialogByCLSID(&mut self, pguidCmdGroup: *const GUID) -> HRESULT {
        let this_: *mut Self = self;
        let mut pItem: *mut NOTIFICATION_ITEM;

        pItem = (*this_).m_pHead;
        while !pItem.is_null() {
            if IsEqualGUID!(core::ptr::addr_of!((*pItem).guidItem), pguidCmdGroup) != false {
                SendMessageW((*pItem).hwndDlg, WM_SHOWSTATUSDLG, 0, (WM_LBUTTONUP) as LPARAM);
                return S_OK;
            }
            pItem = (*pItem).pNext;
        }

        return E_FAIL;
    }
}
