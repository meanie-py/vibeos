// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_074CFF6456D6: [u16; 4] = [0x0053u16, 0x0053, 0x0045, 0x0000];
pub static RUSTCONV_WSTR_1A1DAE0EED51: [u16; 3] = [0x0025u16, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_36EBC2C9A0E2: [u16; 6] = [0x0033u16, 0x0044, 0x004E, 0x004F, 0x0057, 0x0000];
pub static RUSTCONV_WSTR_77D95186A6AE: [u16; 4] = [0x004Du16, 0x004D, 0x0058, 0x0000];
pub static RUSTCONV_WSTR_82294FD79A0D: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];
pub static RUSTCONV_WSTR_879403D34C95: [u16; 5] = [0x0053u16, 0x0053, 0x0045, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_A9EDBA2FC474: [u16; 3] = [0x0025u16, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_C3BE9DAFC52C: [u16; 8] = [0x0025u16, 0x006C, 0x0064, 0x0020, 0x004D, 0x0048, 0x007A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

// #define WIN32_NO_STATUS (flag only, no value)
// #define _INC_WINDOWS (flag only, no value)
// #define COM_NO_WINDOWS_H (flag only, no value)

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstance: HANDLE, dwReason: DWORD, lpReserved: LPVOID) -> BOOL {
    match dwReason {
        DLL_THREAD_ATTACH | DLL_THREAD_DETACH | DLL_PROCESS_ATTACH | DLL_PROCESS_DETACH => {}
        _ => {}
    }

    g_hInstance = hInstance as HINSTANCE;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn PropSheetExtProc(PropPageRequest: PSP_PROPSHEETPAGE_REQUEST, fAddFunc: LPFNADDPROPSHEETPAGE, lParam: LPARAM) -> BOOL {
    let mut PropSheetPage: PROPSHEETPAGEW = core::mem::zeroed();
    let mut hPropSheetPage: HPROPSHEETPAGE = core::ptr::null_mut();

    if (*PropPageRequest).PageRequested != SPPSR_ENUM_ADV_DEVICE_PROPERTIES as DWORD {
        return (FALSE) as BOOL;
    }

    if (*PropPageRequest).DeviceInfoSet.is_null() || (*PropPageRequest).DeviceInfoData.is_null() {
        return (FALSE) as BOOL;
    }

    ZeroMemory!(&raw mut PropSheetPage as *mut core::ffi::c_void, core::mem::size_of::<PROPSHEETPAGE>() as usize);
    PropSheetPage.dwSize = core::mem::size_of::<PROPSHEETPAGE>() as DWORD;
    PropSheetPage.hInstance = g_hInstance;
    PropSheetPage.__bindgen_anon_1.pszTemplate = MAKEINTRESOURCE!(DLG_PROCESSORINFO as WORD) as LPCWSTR;
    PropSheetPage.pfnDlgProc = Some(ProcessorDlgProc);

    hPropSheetPage = CreatePropertySheetPage(&raw mut PropSheetPage as LPCPROPSHEETPAGEW);
    if hPropSheetPage.is_null() {
        return (FALSE) as BOOL;
    }

    if fAddFunc.unwrap_unchecked()(hPropSheetPage, lParam) == 0 {
        DestroyPropertySheetPage(hPropSheetPage);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn AddFeature(szFeatures: *mut WCHAR, cbDest: size_t, Feature: *mut WCHAR, bFirst: *mut BOOL) {
    if *bFirst == 0 {
        StringCbCatW(szFeatures, cbDest, RUSTCONV_WSTR_82294FD79A0D.as_ptr());
    }
    *bFirst = (FALSE as i32);
    StringCbCatW(szFeatures, cbDest, Feature);
}

#[no_mangle]
pub unsafe extern "system" fn ProcessorDlgProc(
    hDlg: HWND,
    uMessage: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMessage {
        WM_INITDIALOG => {
            let mut szFeatures: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut szModel: [WCHAR; 3] = [0; 3];
            let mut szStepping: [WCHAR; 3] = [0; 3];
            let mut szMhz: [WCHAR; 16] = [0; 16];
            let mut szCurrentMhz: [WCHAR; 10] = [0; 10];
            let mut bFirst: BOOL = (TRUE as i32);
            let mut SystemInfo: SYSTEM_INFO = ::core::mem::zeroed::<SYSTEM_INFO>();
            let mut PowerInfo: PROCESSOR_POWER_INFORMATION =
                ::core::mem::zeroed::<PROCESSOR_POWER_INFORMATION>();

            if LoadStringW(
                g_hInstance,
                IDS_MEGAHERTZ as UINT,
                ::core::ptr::addr_of_mut!(szMhz) as LPWSTR,
                ARRAYSIZE!(szMhz) as ::core::ffi::c_int,
            ) == 0
            {
                StringCbCopyW(
                    ::core::ptr::addr_of_mut!(szMhz) as *mut WCHAR,
                    ::core::mem::size_of::<[WCHAR; 16]>() as size_t,
                    RUSTCONV_WSTR_C3BE9DAFC52C.as_ptr(),
                );
            }

            if IsProcessorFeaturePresent(PF_MMX_INSTRUCTIONS_AVAILABLE as DWORD) != 0 {
                AddFeature(
                    ::core::ptr::addr_of_mut!(szFeatures) as *mut WCHAR,
                    ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
                    RUSTCONV_WSTR_77D95186A6AE.as_ptr() as *mut WCHAR,
                    ::core::ptr::addr_of_mut!(bFirst),
                );
            }
            if IsProcessorFeaturePresent(PF_XMMI_INSTRUCTIONS_AVAILABLE as DWORD) != 0 {
                AddFeature(
                    ::core::ptr::addr_of_mut!(szFeatures) as *mut WCHAR,
                    ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
                    RUSTCONV_WSTR_074CFF6456D6.as_ptr() as *mut WCHAR,
                    ::core::ptr::addr_of_mut!(bFirst),
                );
            }
            if IsProcessorFeaturePresent(PF_XMMI64_INSTRUCTIONS_AVAILABLE as DWORD) != 0 {
                AddFeature(
                    ::core::ptr::addr_of_mut!(szFeatures) as *mut WCHAR,
                    ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
                    RUSTCONV_WSTR_879403D34C95.as_ptr() as *mut WCHAR,
                    ::core::ptr::addr_of_mut!(bFirst),
                );
            }
            /*if (IsProcessorFeaturePresent(PF_SSE3_INSTRUCTIONS_AVAILABLE))
                AddFeature(szFeatures, sizeof(szFeatures), L"SSE3", &bFirst); */
            if IsProcessorFeaturePresent(PF_3DNOW_INSTRUCTIONS_AVAILABLE as DWORD) != 0 {
                AddFeature(
                    ::core::ptr::addr_of_mut!(szFeatures) as *mut WCHAR,
                    ::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as size_t,
                    RUSTCONV_WSTR_36EBC2C9A0E2.as_ptr() as *mut WCHAR,
                    ::core::ptr::addr_of_mut!(bFirst),
                );
            }

            SetDlgItemTextW(
                hDlg,
                IDC_FEATURES as ::core::ffi::c_int,
                ::core::ptr::addr_of_mut!(szFeatures) as *mut WCHAR as LPCWSTR,
            );

            GetSystemInfo(::core::ptr::addr_of_mut!(SystemInfo));

            StringCbPrintfW!(
                ::core::ptr::addr_of_mut!(szModel) as *mut WCHAR,
                ::core::mem::size_of::<[WCHAR; 3]>() as size_t,
                RUSTCONV_WSTR_1A1DAE0EED51.as_ptr(),
                HIBYTE!(SystemInfo.wProcessorRevision) as ::core::ffi::c_int,
            );
            StringCbPrintfW!(
                ::core::ptr::addr_of_mut!(szStepping) as *mut WCHAR,
                ::core::mem::size_of::<[WCHAR; 3]>() as size_t,
                RUSTCONV_WSTR_A9EDBA2FC474.as_ptr(),
                LOBYTE!(SystemInfo.wProcessorRevision) as ::core::ffi::c_int,
            );

            SetDlgItemTextW(
                hDlg,
                IDC_MODEL as ::core::ffi::c_int,
                ::core::ptr::addr_of_mut!(szModel) as *mut WCHAR as LPCWSTR,
            );
            SetDlgItemTextW(
                hDlg,
                IDC_STEPPING as ::core::ffi::c_int,
                ::core::ptr::addr_of_mut!(szStepping) as *mut WCHAR as LPCWSTR,
            );

            CallNtPowerInformation(
                11,
                ::core::ptr::null_mut(),
                0 as ULONG,
                ::core::ptr::addr_of_mut!(PowerInfo) as PVOID,
                ::core::mem::size_of::<PROCESSOR_POWER_INFORMATION>() as ULONG,
            );
            StringCbPrintfW!(
                ::core::ptr::addr_of_mut!(szCurrentMhz) as *mut WCHAR,
                ::core::mem::size_of::<[WCHAR; 10]>() as size_t,
                ::core::ptr::addr_of_mut!(szMhz) as *mut WCHAR as LPCWSTR,
                PowerInfo.CurrentMhz,
            );
            SetDlgItemTextW(
                hDlg,
                IDC_CORESPEED as ::core::ffi::c_int,
                ::core::ptr::addr_of_mut!(szCurrentMhz) as *mut WCHAR as LPCWSTR,
            );

            return TRUE as INT_PTR;
        }
        _ => {}
    }
    return FALSE as INT_PTR;
}
