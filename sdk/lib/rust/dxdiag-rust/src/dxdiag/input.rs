// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6E233CC6DE78: [u16; 7] = [0x0030u16, 0x0078, 0x0025, 0x0030, 0x0034, 0x0058, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut IID_IDirectInput8W: GUID = _GUID {
    Data1: 0xbf798031 as ::core::ffi::c_uint,
    Data2: 0x483a as ::core::ffi::c_ushort,
    Data3: 0x4da2 as ::core::ffi::c_ushort,
    Data4: [
        0xaa as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x99 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x5d as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x64 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xed as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x36 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x97 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

#[repr(C)]
pub struct INPUT_DIALOG_CONTEXT {
    pub hwndDlg: HWND,
    pub pObj: *mut IDirectInput8W,
    pub hDevList: HWND,
    pub hPortTree: HWND,
    pub Count: i32,
}

pub type PINPUT_DIALOG_CONTEXT = *mut INPUT_DIALOG_CONTEXT;
#[no_mangle]
pub unsafe extern "system" fn DirectInputEnumDevCb(
    lpddi: LPCDIDEVICEINSTANCEW,
    pvRef: LPVOID,
) -> BOOL {
    let mut hResult: HRESULT;
    let mut szText: [WCHAR; 100] = [0; 100];
    let mut pDev: *mut IDirectInputDevice8W = core::ptr::null_mut();
    let mut VendorID: DIPROPDWORD = core::mem::zeroed();
    let mut DevCaps: DIDEVCAPS = core::mem::zeroed();
    let mut dwProductID: DWORD;
    let mut dwManufacturerID: DWORD;
    let mut Item: LVITEMW = core::mem::zeroed();
    let mut lResult: LRESULT;

    let pContext: PINPUT_DIALOG_CONTEXT = pvRef as PINPUT_DIALOG_CONTEXT;

    if pContext.is_null() {
        return (DIENUM_STOP) as BOOL;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(Item) as *mut core::ffi::c_void, core::mem::size_of::<LVITEMW>());
    Item.mask = LVIF_TEXT;
    Item.pszText = &raw const (*lpddi).tszProductName as *const WCHAR as LPWSTR;
    Item.iItem = (*pContext).Count;
    /* insert device item */
    lResult = SendMessageW(
        (*pContext).hDevList,
        (LVM_INSERTITEM) as UINT,
        0,
        core::ptr::addr_of_mut!(Item) as LPARAM,
    );
    if lResult == -1 {
        return (DIENUM_CONTINUE) as BOOL;
    }

    /* is the device attached */
    szText[0] = 0x0000u16;
    hResult = ((*(*(*pContext).pObj).lpVtbl).GetDeviceStatus)
        .unwrap_unchecked()(
        (*pContext).pObj,
        &raw const (*lpddi).guidInstance,
    );
    if hResult == DI_OK {
        LoadStringW(
            hInst,
            IDS_DEVICE_STATUS_ATTACHED,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
        );
    } else if hResult == DI_NOTATTACHED {
        LoadStringW(
            hInst,
            IDS_DEVICE_STATUS_MISSING,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
        );
    } else {
        LoadStringW(
            hInst,
            IDS_DEVICE_STATUS_UNKNOWN,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
        );
    }

    if szText[0] != 0 {
        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) - 1] = 0x0000u16;
        Item.iSubItem = 1;
        Item.pszText = core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPWSTR;
        SendMessageW(
            (*pContext).hDevList,
            (LVM_SETITEM) as UINT,
            lResult as WPARAM,
            core::ptr::addr_of_mut!(Item) as LPARAM,
        );
    }

    hResult = ((*(*(*pContext).pObj).lpVtbl).CreateDevice)
        .unwrap_unchecked()(
        (*pContext).pObj,
        &raw const (*lpddi).guidInstance,
        core::ptr::addr_of_mut!(pDev),
        core::ptr::null_mut(),
    );

    if hResult != DI_OK {
        return (DIENUM_STOP) as BOOL;
    }

    ZeroMemory!(core::ptr::addr_of_mut!(VendorID) as *mut core::ffi::c_void, core::mem::size_of::<DIPROPDWORD>());
    VendorID.diph.dwSize = (core::mem::size_of::<DIPROPDWORD>()) as _;
    VendorID.diph.dwHeaderSize = (core::mem::size_of::<DIPROPHEADER>()) as _;

    hResult = ((*(*pDev).lpVtbl).GetProperty)
        .unwrap_unchecked()(
        pDev,
        DIPROP_VIDPID as usize as _,
        core::ptr::addr_of_mut!(VendorID) as LPDIPROPHEADER,
    );
    if hResult == DI_OK {
        /* set manufacturer id */
        dwManufacturerID = LOWORD!(VendorID.dwData) as DWORD;
        wsprintfW(
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            RUSTCONV_WSTR_6E233CC6DE78.as_ptr(),
            dwManufacturerID,
        );
        Item.iSubItem = 3;
        SendMessageW(
            (*pContext).hDevList,
            (LVM_SETITEM) as UINT,
            lResult as WPARAM,
            core::ptr::addr_of_mut!(Item) as LPARAM,
        );
        /* set product id */
        dwProductID = HIWORD!(VendorID.dwData) as DWORD;
        wsprintfW(
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            RUSTCONV_WSTR_6E233CC6DE78.as_ptr(),
            dwProductID,
        );
        Item.iSubItem = 4;
        SendMessageW(
            (*pContext).hDevList,
            (LVM_SETITEM) as UINT,
            lResult as WPARAM,
            core::ptr::addr_of_mut!(Item) as LPARAM,
        );
    } else {
        szText[0] = 0x0000u16;
        LoadStringW(
            hInst,
            IDS_NOT_APPLICABLE,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
        );
        szText[(core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) - 1] = 0x0000u16;
        /* set manufacturer id */
        Item.iSubItem = 3;
        SendMessageW(
            (*pContext).hDevList,
            (LVM_SETITEM) as UINT,
            lResult as WPARAM,
            core::ptr::addr_of_mut!(Item) as LPARAM,
        );
        /* set product id */
        Item.iSubItem = 4;
        SendMessageW(
            (*pContext).hDevList,
            (LVM_SETITEM) as UINT,
            lResult as WPARAM,
            core::ptr::addr_of_mut!(Item) as LPARAM,
        );
    }

    /* check for force feedback support */
    DevCaps.dwSize = (core::mem::size_of::<DIDEVCAPS>()) as _; // use DIDEVCAPS_DX3 for DX3 support
    hResult = ((*(*pDev).lpVtbl).GetCapabilities)
        .unwrap_unchecked()(
        pDev,
        core::ptr::addr_of_mut!(DevCaps),
    );
    szText[0] = 0x0000u16;
    if hResult == DI_OK {
        if DevCaps.dwFlags & DIDC_FORCEFEEDBACK != 0 {
            LoadStringW(
                hInst,
                IDS_OPTION_YES,
                core::ptr::addr_of_mut!(szText) as LPWSTR,
                (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
            );
        } else {
            LoadStringW(
                hInst,
                IDS_NOT_APPLICABLE,
                core::ptr::addr_of_mut!(szText) as LPWSTR,
                (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
            );
        }
    } else {
        LoadStringW(
            hInst,
            IDS_NOT_APPLICABLE,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of_val(&szText) / core::mem::size_of::<WCHAR>()) as INT,
        );
    }

    Item.iSubItem = 5;
    SendMessageW(
        (*pContext).hDevList,
        (LVM_SETITEM) as UINT,
        lResult as WPARAM,
        core::ptr::addr_of_mut!(Item) as LPARAM,
    );

    (*(*pDev).lpVtbl).Release.unwrap_unchecked()(pDev);
    (*pContext).Count += 1;

    return (DIENUM_CONTINUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InputPageInitListViewColumns(pContext: PINPUT_DIALOG_CONTEXT) {
    let mut szText: [WCHAR; 256] = [0; 256];
    let mut lvcolumn: LVCOLUMNW = core::mem::zeroed();
    let mut Index: INT = 0;

    (*pContext).hDevList = GetDlgItem((*pContext).hwndDlg, IDC_LIST_DEVICE as INT as _);

    /* Highlights the entire row instead of just the selected item in the first column.
       This increases readability on the selected item in the list. */
    SendMessage(
        (*pContext).hDevList,
        LVM_SETEXTENDEDLISTVIEWSTYLE as UINT,
        0 as WPARAM,
        LVS_EX_FULLROWSELECT as LPARAM,
    );

    ZeroMemory!(
        core::ptr::addr_of_mut!(lvcolumn) as *mut core::ffi::c_void,
        core::mem::size_of::<LVCOLUMNW>(),
    );
    lvcolumn.pszText = core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPWSTR;
    lvcolumn.mask = (LVCF_FMT | LVCF_TEXT | LVCF_SUBITEM | LVCF_WIDTH) as UINT;
    lvcolumn.fmt = (LVCFMT_LEFT as i32);
    lvcolumn.cx = 100 as core::ffi::c_int;

    Index = 0 as INT;
    while Index < 6 as INT {
        szText[0 as usize] = 0x0000u16;
        LoadStringW(
            hInst,
            (IDS_DEVICE_NAME + (Index) as u32) as UINT,
            core::ptr::addr_of_mut!(szText) as LPWSTR,
            (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int,
        );
        szText[(core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>())
            - 1 as usize] = 0x0000u16;
        if SendMessageW(
            (*pContext).hDevList,
            LVM_INSERTCOLUMNW as UINT,
            Index as WPARAM,
            core::ptr::addr_of_mut!(lvcolumn) as LPARAM,
        ) == -(1 as LRESULT)
        {
            return;
        }
        Index += 1;
    }
}

unsafe extern "C" fn InitializeDirectInputDialog(hwndDlg: HWND) {
    let mut Context: INPUT_DIALOG_CONTEXT = core::mem::zeroed();
    let mut hResult: HRESULT;
    let mut pObj: *mut IDirectInput8W = core::ptr::null_mut::<IDirectInput8W>();

    hResult = DirectInput8Create(
        hInst,
        DIRECTINPUT_VERSION as DWORD,
        core::ptr::addr_of!(IID_IDirectInput8W),
        core::ptr::addr_of_mut!(pObj) as *mut LPVOID,
        core::ptr::null_mut::<IUnknown>(),
    );
    if hResult != DI_OK {
        return;
    }

    ZeroMemory!(
        core::ptr::addr_of_mut!(Context) as *mut core::ffi::c_void,
        core::mem::size_of::<INPUT_DIALOG_CONTEXT>() as size_t,
    );
    Context.pObj = pObj;
    Context.hwndDlg = hwndDlg;
    InputPageInitListViewColumns(core::ptr::addr_of_mut!(Context));
    (*(*pObj).lpVtbl).EnumDevices.unwrap_unchecked()(
        pObj,
        DI8DEVCLASS_ALL as DWORD,
        Some(DirectInputEnumDevCb),
        core::ptr::addr_of_mut!(Context) as LPVOID,
        DIEDFL_ALLDEVICES as DWORD,
    );

    (*(*pObj).lpVtbl).Release.unwrap_unchecked()(pObj);
}

#[no_mangle]
pub unsafe extern "system" fn InputPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    UNREFERENCED_PARAMETER!(lParam);
    UNREFERENCED_PARAMETER!(wParam);
    match message {
        WM_INITDIALOG => {
            SetWindowPos(
                hDlg,
                core::ptr::null_mut(),
                10,
                32,
                0,
                0,
                (SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER) as UINT,
            );
            InitializeDirectInputDialog(hDlg);
            return TRUE as INT_PTR;
        }
        _ => {}
    }

    return FALSE as INT_PTR;
}
