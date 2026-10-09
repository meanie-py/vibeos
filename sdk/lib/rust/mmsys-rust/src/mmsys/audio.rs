// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0F138087CAEF: [u16; 19] = [
    0x0073u16, 0x006E, 0x0064, 0x0076, 0x006F, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065,
    0x0078, 0x0065, 0x0020, 0x002D, 0x0064, 0x0020, 0x0025, 0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_33D5CC7E5919: [u16; 22] = [
    0x0073u16, 0x006E, 0x0064, 0x0076, 0x006F, 0x006C, 0x0033, 0x0032, 0x002E, 0x0065,
    0x0078, 0x0065, 0x0020, 0x002D, 0x0072, 0x0020, 0x002D, 0x0064, 0x0020, 0x0025,
    0x0064, 0x0000,
];
pub static RUSTCONV_WSTR_349B5CC15A79: [u16; 43] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x004D,
    0x0075, 0x006C, 0x0074, 0x0069, 0x006D, 0x0065, 0x0064, 0x0069, 0x0061, 0x005C,
    0x0053, 0x006F, 0x0075, 0x006E, 0x0064, 0x0020, 0x004D, 0x0061, 0x0070, 0x0070,
    0x0065, 0x0072, 0x0000,
];
pub static RUSTCONV_WSTR_508DD5557C0E: [u16; 8] = [0x0073u16, 0x007A, 0x0050, 0x006E, 0x0061, 0x006D, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_5714CBC7926F: [u16; 61] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x004D, 0x0075, 0x006C, 0x0074, 0x0069, 0x006D, 0x0065, 0x0064,
    0x0069, 0x0061, 0x005C, 0x004D, 0x0049, 0x0044, 0x0049, 0x004D, 0x0061, 0x0070,
    0x0000,
];
pub static RUSTCONV_WSTR_58FC0BEA9D1C: [u16; 9] = [0x0050u16, 0x006C, 0x0061, 0x0079, 0x0062, 0x0061, 0x0063, 0x006B, 0x0000];
pub static RUSTCONV_WSTR_F12F4C721C10: [u16; 7] = [0x0052u16, 0x0065, 0x0063, 0x006F, 0x0072, 0x0064, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _GLOBAL_DATA {
    pub bNoAudioOut: BOOL,
    pub bNoAudioIn: BOOL,
    pub bNoMIDIOut: BOOL,
    pub bAudioOutChanged: BOOL,
    pub bAudioInChanged: BOOL,
    pub bMIDIOutChanged: BOOL,
}

pub type GLOBAL_DATA = _GLOBAL_DATA;
pub type PGLOBAL_DATA = *mut _GLOBAL_DATA;
#[no_mangle]
pub unsafe extern "C" fn InitAudioDlg(hwnd: HWND, pGlobalData: PGLOBAL_DATA) {
    let mut waveOutputPaps: WAVEOUTCAPSW = core::mem::zeroed();
    let mut waveInputPaps: WAVEINCAPSW = core::mem::zeroed();
    let mut midiOutCaps: MIDIOUTCAPSW = core::mem::zeroed();
    let mut szNoDevices: [WCHAR; 256] = [0; 256];
    let mut DevsNum: UINT = 0;
    let mut uIndex: UINT = 0;
    let mut hCB: HWND = core::ptr::null_mut();
    let mut Res: LRESULT = 0;

    LoadStringW(hApplet, IDS_NO_DEVICES, core::ptr::addr_of_mut!(szNoDevices) as LPWSTR, _countof!(szNoDevices) as ::core::ffi::c_int);

    // Init sound playback devices list
    hCB = GetDlgItem(hwnd, (IDC_DEVICE_PLAY_LIST as i32));

    DevsNum = waveOutGetNumDevs();
    if DevsNum < 1 {
        Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(szNoDevices) as LPARAM);
        SendMessageW(hCB, CB_SETCURSEL, Res as WPARAM, 0);
        (*pGlobalData).bNoAudioOut = (TRUE) as i32;
    } else {
        let mut DefaultDevice: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut dwSize: DWORD = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
        let mut DefaultIndex: UINT = 0;

        if (RegOpenKeyExW(HKEY_CURRENT_USER, RUSTCONV_WSTR_349B5CC15A79.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
            RegQueryValueExW(hKey, RUSTCONV_WSTR_58FC0BEA9D1C.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(DefaultDevice) as LPBYTE, core::ptr::addr_of_mut!(dwSize));
            DefaultDevice[(_countof!(DefaultDevice) - 1) as usize] = UNICODE_NULL;
            RegCloseKey(hKey);
        }

        uIndex = 0;
        while uIndex < DevsNum {
            'body: {
                if waveOutGetDevCapsW(uIndex as UINT_PTR, core::ptr::addr_of_mut!(waveOutputPaps), core::mem::size_of::<WAVEOUTCAPSW>() as UINT) != 0 {
                    break 'body;
                }

                Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(waveOutputPaps.szPname) as *mut WCHAR as LPARAM);

                if (CB_ERR as LRESULT) != Res {
                    SendMessageW(hCB, CB_SETITEMDATA, Res as WPARAM, uIndex as LPARAM);
                    if _wcsicmp(core::ptr::addr_of_mut!(waveOutputPaps.szPname) as *mut WCHAR, core::ptr::addr_of_mut!(DefaultDevice) as *mut WCHAR) == 0 {
                        DefaultIndex = Res as UINT;
                    }
                }
            }
            uIndex = uIndex.wrapping_add(1);
        }
        SendMessageW(hCB, CB_SETCURSEL, DefaultIndex as WPARAM, 0);
    }

    // Init sound recording devices list
    hCB = GetDlgItem(hwnd, (IDC_DEVICE_REC_LIST as i32));

    DevsNum = waveInGetNumDevs();
    if DevsNum < 1 {
        Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(szNoDevices) as LPARAM);
        SendMessageW(hCB, CB_SETCURSEL, Res as WPARAM, 0);
        (*pGlobalData).bNoAudioIn = (TRUE) as i32;
    } else {
        let mut DefaultDevice: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut dwSize: DWORD = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
        let mut DefaultIndex: UINT = 0;

        if (RegOpenKeyExW(HKEY_CURRENT_USER, RUSTCONV_WSTR_349B5CC15A79.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
            RegQueryValueExW(hKey, RUSTCONV_WSTR_F12F4C721C10.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(DefaultDevice) as LPBYTE, core::ptr::addr_of_mut!(dwSize));
            DefaultDevice[(_countof!(DefaultDevice) - 1) as usize] = UNICODE_NULL;
            RegCloseKey(hKey);
        }

        uIndex = 0;
        while uIndex < DevsNum {
            'body: {
                if waveInGetDevCapsW(uIndex as UINT_PTR, core::ptr::addr_of_mut!(waveInputPaps), core::mem::size_of::<WAVEINCAPSW>() as UINT) != 0 {
                    break 'body;
                }

                Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(waveInputPaps.szPname) as *mut WCHAR as LPARAM);

                if (CB_ERR as LRESULT) != Res {
                    SendMessageW(hCB, CB_SETITEMDATA, Res as WPARAM, uIndex as LPARAM);
                    if _wcsicmp(core::ptr::addr_of_mut!(waveInputPaps.szPname) as *mut WCHAR, core::ptr::addr_of_mut!(DefaultDevice) as *mut WCHAR) == 0 {
                        DefaultIndex = Res as UINT;
                    }
                }
            }
            uIndex = uIndex.wrapping_add(1);
        }
        SendMessageW(hCB, CB_SETCURSEL, DefaultIndex as WPARAM, 0);
    }

    // Init MIDI devices list
    hCB = GetDlgItem(hwnd, (IDC_DEVICE_MIDI_LIST as i32));

    DevsNum = midiOutGetNumDevs();
    if DevsNum < 1 {
        Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(szNoDevices) as LPARAM);
        SendMessageW(hCB, CB_SETCURSEL, Res as WPARAM, 0);
        (*pGlobalData).bNoMIDIOut = (TRUE) as i32;
    } else {
        let mut DefaultDevice: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut dwSize: DWORD = core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as DWORD;
        let mut DefaultIndex: UINT = 0;

        if (RegOpenKeyExW(HKEY_CURRENT_USER, RUSTCONV_WSTR_5714CBC7926F.as_ptr(), 0, KEY_READ, core::ptr::addr_of_mut!(hKey))) as u32 == ERROR_SUCCESS {
            RegQueryValueExW(hKey, RUSTCONV_WSTR_508DD5557C0E.as_ptr(), core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(DefaultDevice) as LPBYTE, core::ptr::addr_of_mut!(dwSize));
            DefaultDevice[(_countof!(DefaultDevice) - 1) as usize] = UNICODE_NULL;
            RegCloseKey(hKey);
        }

        uIndex = 0;
        while uIndex < DevsNum {
            'body: {
                if midiOutGetDevCapsW(uIndex as UINT_PTR, core::ptr::addr_of_mut!(midiOutCaps), core::mem::size_of::<MIDIOUTCAPSW>() as UINT) != 0 {
                    break 'body;
                }

                Res = SendMessageW(hCB, CB_ADDSTRING, 0, core::ptr::addr_of_mut!(midiOutCaps.szPname) as *mut WCHAR as LPARAM);

                if (CB_ERR as LRESULT) != Res {
                    SendMessageW(hCB, CB_SETITEMDATA, Res as WPARAM, uIndex as LPARAM);
                    if _wcsicmp(core::ptr::addr_of_mut!(midiOutCaps.szPname) as *mut WCHAR, core::ptr::addr_of_mut!(DefaultDevice) as *mut WCHAR) == 0 {
                        DefaultIndex = Res as UINT;
                    }
                }
            }
            uIndex = uIndex.wrapping_add(1);
        }
        SendMessageW(hCB, CB_SETCURSEL, DefaultIndex as WPARAM, 0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn UpdateRegistryString(hwnd: HWND, ctrl: INT, key: LPCWSTR, value: LPCWSTR) {
    let mut hwndCombo: HWND = GetDlgItem(hwnd, ctrl as ::core::ffi::c_int);
    let mut CurSel: INT = SendMessageW(
        hwndCombo,
        CB_GETCURSEL as UINT,
        0 as WPARAM,
        0 as LPARAM,
    ) as INT;
    let mut TextLen: UINT = 0;
    let mut SelectedDevice: [WCHAR; (MAX_PATH) as _] = [0 as WCHAR; (MAX_PATH) as _];
    let mut hKey: HKEY = ::core::ptr::null_mut::<HKEY__>();

    if CurSel == CB_ERR {
        return;
    }

    TextLen = (SendMessageW(
        hwndCombo,
        CB_GETLBTEXTLEN as UINT,
        CurSel as WPARAM,
        0 as LPARAM,
    ) + 1 as LRESULT) as UINT;

    if TextLen as usize
        > (::core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as usize)
            .wrapping_div(::core::mem::size_of::<WCHAR>() as usize)
    {
        return;
    }

    SendMessageW(
        hwndCombo,
        CB_GETLBTEXT as UINT,
        CurSel as WPARAM,
        core::ptr::addr_of_mut!(SelectedDevice) as *mut WCHAR as LPARAM,
    );

    if RegCreateKeyExW(
        HKEY_CURRENT_USER,
        key,
        0 as DWORD,
        ::core::ptr::null_mut::<WCHAR>(),
        REG_OPTION_NON_VOLATILE as DWORD,
        KEY_WRITE as REGSAM,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        core::ptr::addr_of_mut!(hKey),
        ::core::ptr::null_mut::<DWORD>(),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return;
    }

    RegSetValueExW(
        hKey,
        value,
        0 as DWORD,
        REG_SZ as DWORD,
        core::ptr::addr_of_mut!(SelectedDevice) as *mut WCHAR as *mut BYTE,
        (wcslen(core::ptr::addr_of_mut!(SelectedDevice) as *mut WCHAR) as usize)
            .wrapping_add(1 as usize)
            .wrapping_mul(::core::mem::size_of::<WCHAR>() as usize) as DWORD,
    );
    RegCloseKey(hKey);
}
#[no_mangle]
pub unsafe extern "C" fn SaveAudioDlg(hwnd: HWND, pGlobalData: PGLOBAL_DATA) {
    if (*pGlobalData).bAudioOutChanged != 0 {
        UpdateRegistryString(
            hwnd,
            (IDC_DEVICE_PLAY_LIST) as INT,
            RUSTCONV_WSTR_349B5CC15A79.as_ptr(),
            RUSTCONV_WSTR_58FC0BEA9D1C.as_ptr(),
        );
    }

    if (*pGlobalData).bAudioInChanged != 0 {
        UpdateRegistryString(
            hwnd,
            (IDC_DEVICE_REC_LIST) as INT,
            RUSTCONV_WSTR_349B5CC15A79.as_ptr(),
            RUSTCONV_WSTR_F12F4C721C10.as_ptr(),
        );
    }

    if (*pGlobalData).bMIDIOutChanged != 0 {
        UpdateRegistryString(
            hwnd,
            (IDC_DEVICE_MIDI_LIST) as INT,
            RUSTCONV_WSTR_5714CBC7926F.as_ptr(),
            RUSTCONV_WSTR_508DD5557C0E.as_ptr(),
        );
    }
}

unsafe extern "C" fn GetDevNum(hControl: HWND, Id: DWORD) -> UINT {
    let mut iCurSel: ::core::ffi::c_int = 0;
    let mut DevNum: UINT = 0;
    iCurSel = SendMessageW(hControl, CB_GETCURSEL as UINT, 0 as WPARAM, 0 as LPARAM) as ::core::ffi::c_int;
    if iCurSel == CB_ERR as ::core::ffi::c_int {
        return 0 as UINT;
    }
    DevNum = SendMessageW(hControl, CB_GETITEMDATA as UINT, iCurSel as WPARAM, 0 as LPARAM) as UINT;
    if DevNum == (CB_ERR as UINT) {
        return 0 as UINT;
    }
    if mixerGetID(IntToPtr!(DevNum as ::core::ffi::c_int) as HMIXEROBJ, core::ptr::addr_of_mut!(DevNum), Id) != (MMSYSERR_NOERROR as MMRESULT) {
        return 0 as UINT;
    }
    return DevNum;
}

#[no_mangle]
pub unsafe extern "system" fn AudioDlgProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    let mut pGlobalData: PGLOBAL_DATA = core::ptr::null_mut();

    pGlobalData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PGLOBAL_DATA;

    'sw: {
        if uMsg == WM_INITDIALOG {
            let NumWavOut: UINT = waveOutGetNumDevs();

            pGlobalData = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<GLOBAL_DATA>() as SIZE_T,
            ) as PGLOBAL_DATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pGlobalData as LONG_PTR);

            if pGlobalData.is_null() {
                break 'sw;
            }

            InitAudioDlg(hwndDlg, pGlobalData);

            if NumWavOut == 0 {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_PLAY_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_REC_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_MIDI_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEFAULT_DEV_CHECKBOX as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME1_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV2_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME2_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV1_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME3_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV3_BTN as i32)), (FALSE) as BOOL);
            }

            if (*pGlobalData).bNoAudioOut != 0 {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_PLAY_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME1_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV2_BTN as i32)), (FALSE) as BOOL);
            }

            if (*pGlobalData).bNoAudioIn != 0 {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_REC_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME2_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV1_BTN as i32)), (FALSE) as BOOL);
            }

            if (*pGlobalData).bNoMIDIOut != 0 {
                EnableWindow(GetDlgItem(hwndDlg, (IDC_DEVICE_MIDI_LIST as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_VOLUME3_BTN as i32)), (FALSE) as BOOL);
                EnableWindow(GetDlgItem(hwndDlg, (IDC_ADV3_BTN as i32)), (FALSE) as BOOL);
            }
        } else if uMsg == WM_COMMAND {
            let mut si: STARTUPINFOW = core::mem::zeroed();
            let mut pi: PROCESS_INFORMATION = core::mem::zeroed();
            let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

            if pGlobalData.is_null() {
                break 'sw;
            }

            let mut at: u8 = 9;
            if (LOWORD!(wParam) as UINT) == IDC_VOLUME1_BTN {
                at = 0;
            } else if (LOWORD!(wParam) as UINT) == IDC_ADV2_BTN {
                at = 1;
            } else if (LOWORD!(wParam) as UINT) == IDC_VOLUME2_BTN {
                at = 2;
            } else if (LOWORD!(wParam) as UINT) == IDC_ADV1_BTN {
                at = 3;
            } else if (LOWORD!(wParam) as UINT) == IDC_VOLUME3_BTN {
                at = 4;
            } else if (LOWORD!(wParam) as UINT) == IDC_ADV3_BTN {
                at = 5;
            } else if (LOWORD!(wParam) as UINT) == IDC_DEVICE_PLAY_LIST {
                at = 6;
            } else if (LOWORD!(wParam) as UINT) == IDC_DEVICE_REC_LIST {
                at = 7;
            } else if (LOWORD!(wParam) as UINT) == IDC_DEVICE_MIDI_LIST {
                at = 8;
            }

            loop {
                match at {
                    0 => {
                        StringCchPrintfW!(
                            szPath.as_mut_ptr(),
                            _countof!(szPath),
                            RUSTCONV_WSTR_0F138087CAEF.as_ptr(),
                            GetDevNum(GetDlgItem(hwndDlg, (IDC_DEVICE_PLAY_LIST as i32)), MIXER_OBJECTF_WAVEOUT),
                        );

                        ZeroMemory!(core::ptr::addr_of_mut!(si) as PVOID, core::mem::size_of::<STARTUPINFOW>());
                        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
                        si.dwFlags = STARTF_USESHOWWINDOW;
                        si.wShowWindow = (SW_SHOW) as u16;

                        if CreateProcessW(
                            core::ptr::null(),
                            szPath.as_mut_ptr(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (FALSE) as BOOL,
                            0,
                            core::ptr::null_mut(),
                            core::ptr::null(),
                            core::ptr::addr_of_mut!(si),
                            core::ptr::addr_of_mut!(pi),
                        ) != 0
                        {
                            CloseHandle(pi.hProcess);
                            CloseHandle(pi.hThread);
                        }
                        break;
                    }
                    1 => {
                        break;
                    }
                    2 => {
                        StringCchPrintfW!(
                            szPath.as_mut_ptr(),
                            _countof!(szPath),
                            RUSTCONV_WSTR_33D5CC7E5919.as_ptr(),
                            GetDevNum(GetDlgItem(hwndDlg, (IDC_DEVICE_REC_LIST as i32)), MIXER_OBJECTF_WAVEIN),
                        );

                        ZeroMemory!(core::ptr::addr_of_mut!(si) as PVOID, core::mem::size_of::<STARTUPINFOW>());
                        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
                        si.dwFlags = STARTF_USESHOWWINDOW;
                        si.wShowWindow = (SW_SHOW) as u16;

                        if CreateProcessW(
                            core::ptr::null(),
                            szPath.as_mut_ptr(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (FALSE) as BOOL,
                            0,
                            core::ptr::null_mut(),
                            core::ptr::null(),
                            core::ptr::addr_of_mut!(si),
                            core::ptr::addr_of_mut!(pi),
                        ) != 0
                        {
                            CloseHandle(pi.hProcess);
                            CloseHandle(pi.hThread);
                        }
                        break;
                    }
                    3 => {
                        break;
                    }
                    4 => {
                        StringCchPrintfW!(
                            szPath.as_mut_ptr(),
                            _countof!(szPath),
                            RUSTCONV_WSTR_0F138087CAEF.as_ptr(),
                            GetDevNum(GetDlgItem(hwndDlg, (IDC_DEVICE_MIDI_LIST as i32)), MIXER_OBJECTF_MIDIOUT),
                        );

                        ZeroMemory!(core::ptr::addr_of_mut!(si) as PVOID, core::mem::size_of::<STARTUPINFOW>());
                        si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
                        si.dwFlags = STARTF_USESHOWWINDOW;
                        si.wShowWindow = (SW_SHOW) as u16;

                        if CreateProcessW(
                            core::ptr::null(),
                            szPath.as_mut_ptr(),
                            core::ptr::null_mut(),
                            core::ptr::null_mut(),
                            (FALSE) as BOOL,
                            0,
                            core::ptr::null_mut(),
                            core::ptr::null(),
                            core::ptr::addr_of_mut!(si),
                            core::ptr::addr_of_mut!(pi),
                        ) != 0
                        {
                            CloseHandle(pi.hProcess);
                            CloseHandle(pi.hThread);
                        }
                        break;
                    }
                    5 => {
                        break;
                    }
                    6 => {
                        if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            (*pGlobalData).bAudioOutChanged = (TRUE) as i32;
                        }
                        break;
                    }
                    7 => {
                        if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            (*pGlobalData).bAudioInChanged = (TRUE) as i32;
                        }
                        at = 8;
                        continue;
                    }
                    8 => {
                        if (HIWORD!(wParam)) as u32 == CBN_SELCHANGE {
                            PropSheet_Changed!(GetParent(hwndDlg), hwndDlg);
                            (*pGlobalData).bMIDIOutChanged = (TRUE) as i32;
                        }
                        break;
                    }
                    _ => break,
                }
            }
        } else if uMsg == WM_DESTROY {
            if pGlobalData.is_null() {
                break 'sw;
            }

            HeapFree(GetProcessHeap(), 0, (pGlobalData).cast());
            break 'sw;
        } else if uMsg == WM_NOTIFY {
            if pGlobalData.is_null() {
                break 'sw;
            }

            if (*(lParam as LPNMHDR)).code == (PSN_APPLY as UINT) {
                SaveAudioDlg(hwndDlg, pGlobalData);
            }
            return TRUE as INT_PTR;
        }
    }

    return FALSE as INT_PTR;
}
