use crate::dxdiag::dxdiag::EnableDialogTheme;
use crate::dxdiag::display::EnumerateDrivers;
use crate::dxdiag::dxdiag::InsertTabCtrlItem;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_321CE9DB4CF4: [u16; 4] = [0x0057u16, 0x0044, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_37AD5F74A84E: [u16; 6] = [0x0025u16, 0x0073, 0x0020, 0x0025, 0x0075, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut GUID_DEVCLASS_MEDIA: GUID = _GUID {
    Data1: 0x4d36e96c as ::core::ffi::c_int as ::core::ffi::c_uint,
    Data2: 0xe325 as ::core::ffi::c_ushort,
    Data3: 0x11ce as ::core::ffi::c_ushort,
    Data4: [
        0xbf as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xc1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x8 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x2b as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0xe1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
        0x18 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    ],
};

unsafe extern "C" fn SetDeviceDetails(hwndDlg: HWND, classGUID: LPCGUID, lpcstrDescription: LPCWSTR) {
    let mut hInfo: HDEVINFO = core::ptr::null_mut();
    let mut dwIndex: DWORD = 0;
    let mut InfoData: SP_DEVINFO_DATA = core::mem::zeroed();
    let mut szText: [WCHAR; 30] = [0; 30];
    let mut hDlgCtrls: [HWND; 3] = [core::ptr::null_mut(); 3];
    let mut waveOut: WAVEOUTCAPSW = core::mem::zeroed();
    let mut numDev: UINT = 0;
    let mut errCode: MMRESULT = 0;

    /*  enumerate waveout devices */
    numDev = waveOutGetNumDevs();
    if numDev != 0 {
        loop {
            ZeroMemory!(
                core::ptr::addr_of_mut!(waveOut) as *mut core::ffi::c_void,
                core::mem::size_of::<WAVEOUTCAPSW>(),
            );
            let fresh_idx = dwIndex;
            dwIndex = dwIndex.wrapping_add(1);
            errCode = waveOutGetDevCapsW(
                fresh_idx as UINT_PTR,
                core::ptr::addr_of_mut!(waveOut),
                core::mem::size_of::<WAVEOUTCAPSW>() as UINT,
            );
            if wcsncmp(
                lpcstrDescription,
                core::ptr::addr_of_mut!(waveOut.szPname) as *mut WCHAR,
                min(
                    MAXPNAMELEN as size_t,
                    wcslen(core::ptr::addr_of_mut!(waveOut.szPname) as *mut WCHAR) as size_t,
                ),
            ) == 0
            {
                /* set the product id */
                SetDlgItemInt(
                    hwndDlg,
                    (IDC_STATIC_DSOUND_PRODUCTID as i32),
                    waveOut.wPid as UINT,
                    (FALSE) as BOOL,
                );
                /* set the vendor id */
                SetDlgItemInt(
                    hwndDlg,
                    (IDC_STATIC_DSOUND_VENDORID as i32),
                    waveOut.wMid as UINT,
                    (FALSE) as BOOL,
                );
                /* check if it's a WDM audio driver */
                if waveOut.wPid == (MM_MSFT_WDMAUDIO_WAVEOUT as u16) {
                    SendDlgItemMessageW(
                        hwndDlg,
                        (IDC_STATIC_DSOUND_TYPE as i32),
                        WM_SETTEXT,
                        0,
                        RUSTCONV_WSTR_321CE9DB4CF4.as_ptr() as LPARAM,
                    );
                }

                /* check if device is default device */
                szText[0 as usize] = 0x0000u16;
                if dwIndex.wrapping_sub(1) == 0 {
                    /* FIXME assume default playback device is device 0 */
                    LoadStringW(
                        hInst,
                        IDS_OPTION_YES,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        (core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>()) as i32,
                    );
                } else {
                    LoadStringW(
                        hInst,
                        IDS_OPTION_NO,
                        core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                        (core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>()) as i32,
                    );
                }

                szText[((core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>())
                    - 1 as usize)] = 0x0000u16;
                /* set default device info */
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_STATIC_DSOUND_STANDARD as i32),
                    WM_SETTEXT,
                    0,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
                );
                break;
            }
            if !(errCode == MMSYSERR_NOERROR && dwIndex < numDev) {
                break;
            }
        }
    }

    dwIndex = 0;
    /* create the setup list */
    hInfo = SetupDiGetClassDevsW(
        classGUID,
        core::ptr::null(),
        core::ptr::null_mut(),
        DIGCF_PRESENT | DIGCF_PROFILE,
    );
    if hInfo == INVALID_HANDLE_VALUE {
        return;
    }

    loop {
        ZeroMemory!(
            core::ptr::addr_of_mut!(InfoData) as *mut core::ffi::c_void,
            core::mem::size_of::<SP_DEVINFO_DATA>(),
        );
        InfoData.cbSize = core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD;

        if SetupDiEnumDeviceInfo(hInfo, dwIndex, core::ptr::addr_of_mut!(InfoData)) != 0 {
            /* set device name */
            if SetupDiGetDeviceInstanceId(
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
                core::ptr::addr_of_mut!(szText) as *mut WCHAR,
                (core::mem::size_of::<[WCHAR; 30]>() / core::mem::size_of::<WCHAR>()) as DWORD,
                core::ptr::null_mut(),
            ) != 0
            {
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_STATIC_DSOUND_DEVICEID as i32),
                    WM_SETTEXT,
                    0,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
                );
            }

            /* set the manufacturer name */
            if SetupDiGetDeviceRegistryPropertyW(
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
                SPDRP_MFG,
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(szText) as *mut WCHAR as PBYTE,
                core::mem::size_of::<[WCHAR; 30]>() as DWORD,
                core::ptr::null_mut(),
            ) != 0
            {
                SendDlgItemMessageW(
                    hwndDlg,
                    (IDC_STATIC_ADAPTER_PROVIDER as i32),
                    WM_SETTEXT,
                    0,
                    core::ptr::addr_of_mut!(szText) as *mut WCHAR as LPARAM,
                );
            }

            /* FIXME
             * we currently enumerate only the first adapter
             */
            hDlgCtrls[0 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_DSOUND_DRIVER as i32));
            hDlgCtrls[1 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_DSOUND_VERSION as i32));
            hDlgCtrls[2 as usize] = GetDlgItem(hwndDlg, (IDC_STATIC_DSOUND_DATE as i32));
            EnumerateDrivers(
                core::ptr::addr_of_mut!(hDlgCtrls) as *mut HWND as PVOID,
                hInfo,
                core::ptr::addr_of_mut!(InfoData),
            );
            break;
        }

        if GetLastError() == ERROR_NO_MORE_ITEMS {
            break;
        }

        dwIndex = dwIndex.wrapping_add(1);
        if !(TRUE != 0) {
            break;
        }
    }

    /* destroy the setup list */
    SetupDiDestroyDeviceInfoList(hInfo);
}
#[no_mangle]
pub unsafe extern "system" fn DSEnumCallback(
    lpGuid: LPGUID,
    lpcstrDescription: LPCWSTR,
    lpcstrModule: LPCWSTR,
    lpContext: LPVOID,
) -> BOOL {
    let pContext: PDXDIAG_CONTEXT = lpContext as PDXDIAG_CONTEXT;
    let mut hDlgs: *mut HWND;
    let mut hwndDlg: HWND;
    let mut szSound: [WCHAR; 20] = [0; 20];
    let mut szText: [WCHAR; 30] = [0; 30];
    let mut pObj: *mut IDirectSound8 = core::ptr::null_mut();
    let mut hResult: HRESULT;
    let mut dwCertified: DWORD = 0;

    if lpGuid.is_null() {
        return (TRUE) as BOOL;
    }

    if (*pContext).NumSoundAdapter != 0 {
        hDlgs = HeapReAlloc(
            GetProcessHeap(),
            0,
            (*pContext).hSoundWnd as PVOID,
            ((*pContext).NumSoundAdapter.wrapping_add(1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<HWND>() as SIZE_T),
        ) as *mut HWND;
    } else {
        hDlgs = HeapAlloc(
            GetProcessHeap(),
            0,
            ((*pContext).NumSoundAdapter.wrapping_add(1) as SIZE_T)
                .wrapping_mul(core::mem::size_of::<HWND>() as SIZE_T),
        ) as *mut HWND;
    }

    if hDlgs.is_null() {
        return (FALSE) as BOOL;
    }

    (*pContext).hSoundWnd = hDlgs;
    hwndDlg = CreateDialogParamW(
        hInst,
        MAKEINTRESOURCEW!(IDD_SOUND_DIALOG) as LPCWSTR,
        (*pContext).hMainDialog,
        Some(SoundPageWndProc),
        pContext as LPARAM,
    );
    EnableDialogTheme(hwndDlg);
    if hwndDlg.is_null() {
        return (FALSE) as BOOL;
    }

    hResult = DirectSoundCreate8(
        lpGuid as LPCGUID,
        &raw mut pObj as *mut LPDIRECTSOUND8,
        core::ptr::null_mut(),
    );
    if hResult == DS_OK as HRESULT {
        szText[0 as usize] = 0x0000u16;
        if IDirectSound8_VerifyCertification(pObj, &raw mut dwCertified) == DS_OK as HRESULT {
            if dwCertified == (DS_CERTIFIED as u32) {
                LoadStringW(
                    hInst,
                    IDS_OPTION_YES,
                    szText.as_mut_ptr(),
                    (core::mem::size_of::<[WCHAR; 30]>()
                        / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                );
            } else if dwCertified == (DS_UNCERTIFIED as u32) {
                LoadStringW(
                    hInst,
                    IDS_NOT_APPLICABLE,
                    szText.as_mut_ptr(),
                    (core::mem::size_of::<[WCHAR; 30]>()
                        / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
                );
            }
        } else {
            LoadStringW(
                hInst,
                IDS_OPTION_NO,
                szText.as_mut_ptr(),
                (core::mem::size_of::<[WCHAR; 30]>()
                    / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
            );
        }
        szText[(core::mem::size_of::<[WCHAR; 30]>()
            / core::mem::size_of::<WCHAR>())
            - (1 as usize)] = 0x0000u16;
        SendDlgItemMessageW(
            hwndDlg,
            (IDC_STATIC_DSOUND_LOGO as i32),
            WM_SETTEXT,
            0,
            szText.as_mut_ptr() as LPARAM,
        );
        IDirectSound8_Release(pObj);
    }

    /* set device name */
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_STATIC_DSOUND_NAME as i32),
        WM_SETTEXT,
        0,
        lpcstrDescription as LPARAM,
    );

    /* set range for slider */
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_SLIDER_DSOUND as i32),
        TBM_SETRANGE,
        (TRUE) as WPARAM,
        MAKELONG!(0, 3) as LPARAM,
    );

    /* FIXME set correct position */
    SendDlgItemMessageW(
        hwndDlg,
        (IDC_SLIDER_DSOUND as i32),
        TBM_SETSEL,
        (FALSE) as WPARAM,
        0 as LPARAM,
    );

    /* set further device details */
    SetDeviceDetails(
        hwndDlg,
        &raw const GUID_DEVCLASS_MEDIA,
        lpcstrDescription,
    );

    /* load sound resource string */
    szSound[0 as usize] = 0x0000u16;
    LoadStringW(
        hInst,
        IDS_SOUND_DIALOG,
        szSound.as_mut_ptr(),
        (core::mem::size_of::<[WCHAR; 20]>()
            / core::mem::size_of::<WCHAR>()) as ::core::ffi::c_int,
    );
    szSound[(core::mem::size_of::<[WCHAR; 20]>()
        / core::mem::size_of::<WCHAR>())
        - (1 as usize)] = 0x0000u16;
    /* output the device id */
    wsprintfW(
        szText.as_mut_ptr(),
        RUSTCONV_WSTR_37AD5F74A84E.as_ptr(),
        szSound.as_mut_ptr(),
        (*pContext).NumSoundAdapter.wrapping_add(1),
    );
    /* insert it into general tab */
    InsertTabCtrlItem(
        (*pContext).hTabCtrl,
        (*pContext)
            .NumDisplayAdapter
            .wrapping_add((*pContext).NumSoundAdapter)
            .wrapping_add(1) as INT,
        szText.as_mut_ptr(),
    );
    /* store dialog window */
    *hDlgs.offset((*pContext).NumSoundAdapter as isize) = hwndDlg;
    (*pContext).NumSoundAdapter = (*pContext).NumSoundAdapter.wrapping_add(1);
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitializeDirectSoundPage(mut pContext: PDXDIAG_CONTEXT) {
    let mut hResult: HRESULT;

    /* create DSound object */

//    if (hResult != DS_OK)
//        return;
    hResult = DirectSoundEnumerateW(
        Some(DSEnumCallback),
        pContext as LPVOID,
    );

    /* release the DSound object */
//    pObj->lpVtbl->Release(pObj);
    let _ = hResult;
}

#[no_mangle]
pub unsafe extern "system" fn SoundPageWndProc(hDlg: HWND, message: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match message {
        WM_INITDIALOG => {
            SetWindowPos(hDlg, core::ptr::null_mut(), 10, 32, 0, 0, SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_NOSIZE | SWP_NOZORDER);
            return (TRUE) as INT_PTR;
        }
        WM_COMMAND => {
            if LOWORD!(wParam) == (IDC_BUTTON_TESTDSOUND as u16) {
                return (FALSE) as INT_PTR;
            }
        }
        _ => {}
    }

    return (FALSE) as INT_PTR;
}
