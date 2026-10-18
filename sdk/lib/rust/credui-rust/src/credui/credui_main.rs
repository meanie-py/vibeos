// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2F930CE27772: [u16; 6] = [0x0042u16, 0x0061, 0x0073, 0x0069, 0x0063, 0x0000];
pub static RUSTCONV_WSTR_A2F5C72A8C4B: [u16; 5] = [0x004Eu16, 0x0054, 0x004C, 0x004D, 0x0000];
pub static RUSTCONV_WSTR_C057620377F9: [u16; 10] = [0x004Eu16, 0x0065, 0x0067, 0x006F, 0x0074, 0x0069, 0x0061, 0x0074, 0x0065, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_0 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_0,
}

pub static TOOLTIPS_CLASSW: [::core::ffi::c_ushort; 17] = unsafe {
    ::core::mem::transmute::<[u8; 34], [::core::ffi::c_ushort; 17]>(
        *b"t\0o\0o\0l\0t\0i\0p\0s\0_\0c\0l\0a\0s\0s\x003\x002\0\0\0",
    )
};

static mut hinstCredUI: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut pending_credentials_list: list = unsafe {
    list {
        next: &raw const pending_credentials_list as *mut list,
        prev: &raw const pending_credentials_list as *mut list,
    }
};

static mut critsect_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut csPendingCredentials,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut critsect_debug.ProcessLocksList,
            Blink: &raw mut critsect_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut csPendingCredentials: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const critsect_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(credui): debug-channel macro, no Rust declaration emitted.

pub const TOOLID_INCORRECTPASSWORD: i32 = 1;

pub const TOOLID_CAPSLOCKON: i32 = 2;

pub const ID_CAPSLOCKPOP: i32 = 1;

#[repr(C)]
pub struct pending_credentials {
    pub entry: list,
    pub pszTargetName: PWSTR,
    pub pszUsername: PWSTR,
    pub pszPassword: PWSTR,
    pub generic: BOOL,
}

// static struct list pending_credentials_list = LIST_INIT(pending_credentials_list);

// static CRITICAL_SECTION_DEBUG critsect_debug = { 0, 0, &csPendingCredentials,
//     { &critsect_debug.ProcessLocksList, &critsect_debug.ProcessLocksList },
//     0, 0, { (DWORD_PTR)(__FILE__ ": csPendingCredentials") } };
// NOTE: the __FILE__ string address cannot be const-evaluated into a DWORD_PTR;
// the Spare[0] slot is left as 0 here.

#[repr(C)]
pub struct cred_dialog_params {
    pub pszTargetName: PCWSTR,
    pub pszMessageText: PCWSTR,
    pub pszCaptionText: PCWSTR,
    pub hbmBanner: HBITMAP,
    pub pszUsername: PWSTR,
    pub ulUsernameMaxChars: ULONG,
    pub pszPassword: PWSTR,
    pub ulPasswordMaxChars: ULONG,
    pub fSave: BOOL,
    pub dwFlags: DWORD,
    pub hwndBalloonTip: HWND,
    pub fBalloonTipActive: BOOL,
}
#[no_mangle]
pub unsafe extern "system" fn DllMain(hinstDLL: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    let mut entry: *mut pending_credentials = core::ptr::null_mut();
    let mut cursor2: *mut pending_credentials = core::ptr::null_mut();
    match fdwReason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(hinstDLL as HMODULE);
            hinstCredUI = hinstDLL;
            InitCommonControls();
        }
        DLL_PROCESS_DETACH => {
            if !lpvReserved.is_null() {
            } else {
                entry = ((pending_credentials_list.next as *mut core::ffi::c_char).offset(
                    -(core::mem::offset_of!(pending_credentials, entry) as isize),
                )) as *mut pending_credentials;
                cursor2 = (((*entry).entry.next as *mut core::ffi::c_char).offset(
                    -(core::mem::offset_of!(pending_credentials, entry) as isize),
                )) as *mut pending_credentials;
                while &raw mut (*entry).entry != &raw mut pending_credentials_list {
                    list_remove(&raw mut (*entry).entry);

                    HeapFree(
                        GetProcessHeap(),
                        0,
                        (*entry).pszTargetName as PVOID,
                    );
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        (*entry).pszUsername as PVOID,
                    );
                    SecureZeroMemory(
                        (*entry).pszPassword as PVOID,
                        (lstrlenW((*entry).pszPassword as LPCWSTR) as SIZE_T)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                    );
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        (*entry).pszPassword as PVOID,
                    );
                    HeapFree(
                        GetProcessHeap(),
                        0,
                        entry as PVOID,
                    );

                    entry = cursor2;
                    cursor2 = (((*entry).entry.next as *mut core::ffi::c_char).offset(
                        -(core::mem::offset_of!(pending_credentials, entry) as isize),
                    )) as *mut pending_credentials;
                }
                DeleteCriticalSection(&raw mut csPendingCredentials);
            }
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn save_credentials(
    pszTargetName: PCWSTR,
    pszUsername: PCWSTR,
    pszPassword: PCWSTR,
    generic: BOOL,
) -> DWORD {
    let mut cred: CREDENTIALW = core::mem::zeroed();

    cred.Flags = (0) as DWORD;
    cred.Type = (if generic != 0 {
        CRED_TYPE_GENERIC
    } else {
        CRED_TYPE_DOMAIN_PASSWORD
    }) as DWORD;
    cred.TargetName = pszTargetName as LPWSTR;
    cred.Comment = core::ptr::null_mut::<WCHAR>();
    cred.CredentialBlobSize = (lstrlenW(pszPassword) as usize)
        .wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    cred.CredentialBlob = pszPassword as LPBYTE;
    cred.Persist = (CRED_PERSIST_ENTERPRISE) as DWORD;
    cred.AttributeCount = (0) as DWORD;
    cred.Attributes = core::ptr::null_mut::<_CREDENTIAL_ATTRIBUTEW>();
    cred.TargetAlias = core::ptr::null_mut::<WCHAR>();
    cred.UserName = pszUsername as LPWSTR;

    if CredWriteW(core::ptr::addr_of_mut!(cred), (0) as DWORD) != 0 {
        return (ERROR_SUCCESS) as DWORD;
    } else {
        let ret: DWORD = GetLastError();
        ERR!("CredWriteW failed with error %ld\n", ret);
        return ret;
    }
}

unsafe extern "C" fn CredDialogFillUsernameCombo(hwndUsername: HWND, params: *const cred_dialog_params) {
    let mut count: DWORD = 0;
    let mut i: DWORD = 0;
    let mut credentials: *mut PCREDENTIALW = core::ptr::null_mut::<PCREDENTIALW>();

    if CredEnumerateW(
        core::ptr::null::<WCHAR>(),
        0 as DWORD,
        core::ptr::addr_of_mut!(count),
        core::ptr::addr_of_mut!(credentials),
    ) == 0
    {
        return;
    }

    i = 0 as DWORD;
    while i < count {
        'next: {
            let mut comboitem: COMBOBOXEXITEMW = core::mem::zeroed();
            let mut j: DWORD = 0;
            let mut duplicate: BOOL = FALSE as BOOL;

            if (**credentials.offset(i as isize)).UserName.is_null() {
                break 'next;
            }

            if ((*params).dwFlags & (CREDUI_FLAGS_GENERIC_CREDENTIALS as DWORD)) != 0 {
                if (**credentials.offset(i as isize)).Type != (CRED_TYPE_GENERIC as DWORD) {
                    (**credentials.offset(i as isize)).UserName = core::ptr::null_mut::<WCHAR>();
                    break 'next;
                }
            } else if (**credentials.offset(i as isize)).Type == (CRED_TYPE_GENERIC as DWORD) {
                (**credentials.offset(i as isize)).UserName = core::ptr::null_mut::<WCHAR>();
                break 'next;
            }

            /* don't add another item with the same name if we've already added it */
            j = 0 as DWORD;
            while j < i {
                if !(**credentials.offset(j as isize)).UserName.is_null()
                    && lstrcmpW(
                        (**credentials.offset(i as isize)).UserName as LPCWSTR,
                        (**credentials.offset(j as isize)).UserName as LPCWSTR,
                    ) == 0
                {
                    duplicate = TRUE as BOOL;
                    break;
                }
                j = j.wrapping_add(1);
            }

            if duplicate != 0 {
                break 'next;
            }

            comboitem.mask = CBEIF_TEXT as UINT;
            comboitem.iItem = -(1 as ::core::ffi::c_int) as INT_PTR;
            comboitem.pszText = (**credentials.offset(i as isize)).UserName;
            SendMessageW(
                hwndUsername,
                CBEM_INSERTITEMW as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(comboitem) as LPARAM,
            );
        }
        i = i.wrapping_add(1);
    }

    CredFree(credentials as PVOID);
}

unsafe extern "C" fn CredDialogCreateBalloonTip(hwndDlg: HWND, params: *mut cred_dialog_params) {
    let mut toolinfo: TTTOOLINFOW = core::mem::zeroed();
    let mut wszText: [WCHAR; 256] = [0; 256];

    if !(*params).hwndBalloonTip.is_null() {
        return;
    }

    (*params).hwndBalloonTip = CreateWindowExW(
        WS_EX_TOOLWINDOW as DWORD,
        (TOOLTIPS_CLASSW).as_ptr() as LPCWSTR,
        core::ptr::null::<WCHAR>(),
        (WS_POPUP as DWORD) | (TTS_NOPREFIX as DWORD) | (TTS_BALLOON as DWORD),
        CW_USEDEFAULT as ::core::ffi::c_int,
        CW_USEDEFAULT as ::core::ffi::c_int,
        CW_USEDEFAULT as ::core::ffi::c_int,
        CW_USEDEFAULT as ::core::ffi::c_int,
        hwndDlg,
        core::ptr::null_mut(),
        hinstCredUI,
        core::ptr::null_mut(),
    );
    SetWindowPos(
        (*params).hwndBalloonTip,
        HWND_TOPMOST,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        (SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE) as UINT,
    );

    if !(LoadStringW(
        hinstCredUI,
        IDS_INCORRECTPASSWORD as UINT,
        wszText.as_mut_ptr(),
        ARRAY_SIZE!(wszText) as ::core::ffi::c_int,
    ) != 0)
    {
        ERR!("failed to load IDS_INCORRECTPASSWORD\n");
        return;
    }

    toolinfo.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    toolinfo.uFlags = TTF_TRACK as UINT;
    toolinfo.hwnd = hwndDlg;
    toolinfo.uId = TOOLID_INCORRECTPASSWORD as UINT_PTR;
    SetRectEmpty(core::ptr::addr_of_mut!(toolinfo.rect));
    toolinfo.hinst = core::ptr::null_mut();
    toolinfo.lpszText = wszText.as_mut_ptr();
    toolinfo.lParam = 0 as LPARAM;
    toolinfo.lpReserved = core::ptr::null_mut();
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_ADDTOOLW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );

    if !(LoadStringW(
        hinstCredUI,
        IDS_CAPSLOCKON as UINT,
        wszText.as_mut_ptr(),
        ARRAY_SIZE!(wszText) as ::core::ffi::c_int,
    ) != 0)
    {
        ERR!("failed to load IDS_CAPSLOCKON\n");
        return;
    }

    toolinfo.uId = TOOLID_CAPSLOCKON as UINT_PTR;
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_ADDTOOLW as UINT,
        0 as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );
}

unsafe extern "C" fn CredDialogShowIncorrectPasswordBalloon(
    mut hwndDlg: HWND,
    mut params: *mut cred_dialog_params,
) {
    let mut toolinfo: TTTOOLINFOW = core::mem::zeroed();
    let mut rcPassword: RECT = core::mem::zeroed();
    let mut x: INT = 0;
    let mut y: INT = 0;
    let mut wszTitle: [WCHAR; 256] = [0; 256];

    if *(*params).pszUsername.offset(0) == 0 {
        return;
    }

    if (*params).fBalloonTipActive != 0 {
        return;
    }

    if LoadStringW(
        hinstCredUI,
        IDS_INCORRECTPASSWORDTITLE as UINT,
        core::ptr::addr_of_mut!(wszTitle) as LPWSTR,
        ARRAY_SIZE!(wszTitle) as INT,
    ) == 0
    {
        ERR!("failed to load IDS_INCORRECTPASSWORDTITLE\n");
        return;
    }

    CredDialogCreateBalloonTip(hwndDlg, params);

    memset(
        core::ptr::addr_of_mut!(toolinfo) as *mut core::ffi::c_void,
        0 as i32,
        core::mem::size_of::<TTTOOLINFOW>() as usize,
    );
    toolinfo.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    toolinfo.hwnd = hwndDlg;
    toolinfo.uId = TOOLID_INCORRECTPASSWORD as UINT_PTR;

    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_SETTITLEW as UINT,
        TTI_ERROR as WPARAM,
        core::ptr::addr_of_mut!(wszTitle) as LPARAM,
    );

    GetWindowRect(
        GetDlgItem(hwndDlg, (IDC_PASSWORD as i32)),
        core::ptr::addr_of_mut!(rcPassword),
    );
    /* centered vertically and in the right side of the password edit control */
    x = (rcPassword.right - 12 as LONG) as INT;
    y = ((rcPassword.top + rcPassword.bottom) / 2 as LONG) as INT;
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKPOSITION as UINT,
        0 as WPARAM,
        MAKELONG!(x, y) as LPARAM,
    );

    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKACTIVATE as UINT,
        TRUE as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );

    (*params).fBalloonTipActive = TRUE as BOOL;
}

unsafe extern "C" fn CredDialogShowCapsLockBalloon(hwndDlg: HWND, params: *mut cred_dialog_params) {
    let mut toolinfo: TTTOOLINFOW = core::mem::zeroed();
    let mut rcPassword: RECT = core::mem::zeroed();
    let mut x: INT;
    let mut y: INT;
    let mut wszTitle: [WCHAR; 256] = [0; 256];

    /* don't show two balloon tips at once */
    if (*params).fBalloonTipActive != 0 {
        return;
    }

    if LoadStringW(
        hinstCredUI,
        IDS_CAPSLOCKONTITLE as UINT,
        wszTitle.as_mut_ptr(),
        ARRAY_SIZE!(wszTitle) as INT,
    ) == 0
    {
        ERR!("failed to load IDS_IDSCAPSLOCKONTITLE\n");
        return;
    }

    CredDialogCreateBalloonTip(hwndDlg, params);

    memset(
        core::ptr::addr_of_mut!(toolinfo) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<TTTOOLINFOW>() as size_t,
    );
    toolinfo.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    toolinfo.hwnd = hwndDlg;
    toolinfo.uId = TOOLID_CAPSLOCKON as UINT_PTR;

    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_SETTITLEW as UINT,
        TTI_WARNING as WPARAM,
        wszTitle.as_mut_ptr() as LPARAM,
    );

    GetWindowRect(
        GetDlgItem(hwndDlg, IDC_PASSWORD as INT),
        core::ptr::addr_of_mut!(rcPassword),
    );
    /* just inside the left side of the password edit control */
    x = rcPassword.left + 12 as INT;
    y = rcPassword.bottom - 3 as INT;
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKPOSITION as UINT,
        0 as WPARAM,
        MAKELONG!(x as WORD, y as WORD) as LPARAM,
    );

    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKACTIVATE as UINT,
        TRUE as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );

    SetTimer(
        hwndDlg,
        ID_CAPSLOCKPOP as UINT_PTR,
        SendMessageW(
            (*params).hwndBalloonTip,
            TTM_GETDELAYTIME as UINT,
            TTDT_AUTOPOP as WPARAM,
            0 as LPARAM,
        ) as UINT,
        None,
    );

    (*params).fBalloonTipActive = TRUE as BOOL;
}

unsafe extern "C" fn CredDialogHideBalloonTip(hwndDlg: HWND, params: *mut cred_dialog_params) {
    let mut toolinfo: TTTOOLINFOW = core::mem::zeroed();

    if (*params).hwndBalloonTip.is_null() {
        return;
    }

    memset(
        core::ptr::addr_of_mut!(toolinfo) as *mut core::ffi::c_void,
        0 as core::ffi::c_int,
        core::mem::size_of::<TTTOOLINFOW>() as usize,
    );

    toolinfo.cbSize = core::mem::size_of::<TTTOOLINFOW>() as UINT;
    toolinfo.hwnd = hwndDlg;
    toolinfo.uId = 0 as UINT_PTR;
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKACTIVATE as UINT,
        FALSE as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );
    toolinfo.uId = 1 as UINT_PTR;
    SendMessageW(
        (*params).hwndBalloonTip,
        TTM_TRACKACTIVATE as UINT,
        FALSE as WPARAM,
        core::ptr::addr_of_mut!(toolinfo) as LPARAM,
    );

    (*params).fBalloonTipActive = FALSE as BOOL;
}

unsafe extern "C" fn CredDialogCapsLockOn() -> BOOL {
    return ((GetKeyState((VK_CAPITAL as i32)) as i32 & 0x1) != 0) as BOOL;
}

unsafe extern "system" fn CredDialogPasswordSubclassProc(
    hwnd: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    uIdSubclass: UINT_PTR,
    dwRefData: DWORD_PTR,
) -> LRESULT {
    let params: *mut cred_dialog_params = dwRefData as *mut cred_dialog_params;
    match uMsg {
        WM_KEYDOWN => {
            if wParam == VK_CAPITAL as WPARAM {
                let hwndDlg: HWND = GetParent(hwnd);
                if CredDialogCapsLockOn() != 0 {
                    CredDialogShowCapsLockBalloon(hwndDlg, params);
                } else {
                    CredDialogHideBalloonTip(hwndDlg, params);
                }
            }
        }
        WM_DESTROY => {
            RemoveWindowSubclass(
                hwnd,
                Some(
                    CredDialogPasswordSubclassProc
                        as unsafe extern "system" fn(
                            HWND,
                            UINT,
                            WPARAM,
                            LPARAM,
                            UINT_PTR,
                            DWORD_PTR,
                        ) -> LRESULT,
                ),
                uIdSubclass,
            );
        }
        _ => {}
    }
    return DefSubclassProc(hwnd, uMsg, wParam, lParam);
}

unsafe extern "C" fn CredDialogInit(hwndDlg: HWND, params: *mut cred_dialog_params) -> BOOL {
    let hwndUsername: HWND = GetDlgItem(hwndDlg, (IDC_USERNAME as i32));
    let hwndPassword: HWND = GetDlgItem(hwndDlg, (IDC_PASSWORD as i32));

    SetWindowLongPtrW(hwndDlg, DWLP_USER, params as LONG_PTR);

    if !(*params).hbmBanner.is_null() {
        SendMessageW(
            GetDlgItem(hwndDlg, (IDB_BANNER as i32)),
            STM_SETIMAGE as UINT,
            IMAGE_BITMAP as WPARAM,
            (*params).hbmBanner as LPARAM,
        );
    }

    if !(*params).pszMessageText.is_null() {
        SetDlgItemTextW(hwndDlg, (IDC_MESSAGE as i32), (*params).pszMessageText as LPCWSTR);
    } else {
        let mut format: [WCHAR; 256] = [0; 256];
        let mut message: [WCHAR; 256] = [0; 256];
        LoadStringW(
            hinstCredUI,
            IDS_MESSAGEFORMAT as UINT,
            core::ptr::addr_of_mut!(format) as LPWSTR,
            ARRAY_SIZE!(format) as ::core::ffi::c_int,
        );
        swprintf(
            core::ptr::addr_of_mut!(message) as *mut wchar_t,
            ARRAY_SIZE!(message) as size_t,
            core::ptr::addr_of_mut!(format) as *mut WCHAR,
            (*params).pszTargetName,
        );
        SetDlgItemTextW(
            hwndDlg,
            (IDC_MESSAGE as i32),
            core::ptr::addr_of_mut!(message) as *mut WCHAR as LPCWSTR,
        );
    }
    SetWindowTextW(hwndUsername, (*params).pszUsername as LPCWSTR);
    SetWindowTextW(hwndPassword, (*params).pszPassword as LPCWSTR);

    CredDialogFillUsernameCombo(hwndUsername, params);

    if *(*params).pszUsername.offset(0 as isize) != 0 {
        /* prevent showing a balloon tip here */
        (*params).fBalloonTipActive = TRUE as BOOL;
        SetFocus(hwndPassword);
        (*params).fBalloonTipActive = FALSE as BOOL;
    } else {
        SetFocus(hwndUsername);
    }

    if !(*params).pszCaptionText.is_null() {
        SetWindowTextW(hwndDlg, (*params).pszCaptionText as LPCWSTR);
    } else {
        let mut format_0: [WCHAR; 256] = [0; 256];
        let mut title: [WCHAR; 256] = [0; 256];
        LoadStringW(
            hinstCredUI,
            IDS_TITLEFORMAT as UINT,
            core::ptr::addr_of_mut!(format_0) as LPWSTR,
            ARRAY_SIZE!(format_0) as ::core::ffi::c_int,
        );
        swprintf(
            core::ptr::addr_of_mut!(title) as *mut wchar_t,
            ARRAY_SIZE!(title) as size_t,
            core::ptr::addr_of_mut!(format_0) as *mut WCHAR,
            (*params).pszTargetName,
        );
        SetWindowTextW(
            hwndDlg,
            core::ptr::addr_of_mut!(title) as *mut WCHAR as LPCWSTR,
        );
    }

    if ((*params).dwFlags & (CREDUI_FLAGS_PERSIST as DWORD)) != 0
        || ((*params).dwFlags & (CREDUI_FLAGS_DO_NOT_PERSIST as DWORD)) != 0
            && ((*params).dwFlags & (CREDUI_FLAGS_SHOW_SAVE_CHECK_BOX as DWORD)) == 0
    {
        ShowWindow(
            GetDlgItem(hwndDlg, (IDC_SAVE as i32)),
            (SW_HIDE as i32),
        );
    } else if (*params).fSave != 0 {
        CheckDlgButton(hwndDlg, (IDC_SAVE as i32), BST_CHECKED as UINT);
    }

    /* setup subclassing for Caps Lock detection */
    SetWindowSubclass(
        hwndPassword,
        Some(
            CredDialogPasswordSubclassProc,
        ),
        1 as UINT_PTR,
        params as DWORD_PTR,
    );

    if ((*params).dwFlags & (CREDUI_FLAGS_INCORRECT_PASSWORD as DWORD)) != 0 {
        CredDialogShowIncorrectPasswordBalloon(hwndDlg, params);
    } else if (GetFocus() == hwndPassword) && CredDialogCapsLockOn() != 0 {
        CredDialogShowCapsLockBalloon(hwndDlg, params);
    }

    return FALSE as BOOL;
}

unsafe extern "C" fn CredDialogCommandOk(hwndDlg: HWND, params: *mut cred_dialog_params) {
    let hwndUsername: HWND = GetDlgItem(hwndDlg, (IDC_USERNAME as i32));
    let mut user: LPWSTR = core::ptr::null_mut();
    let mut len: INT = 0;
    let mut len2: INT = 0;

    len = GetWindowTextLengthW(hwndUsername) as INT;
    user = HeapAlloc(
        GetProcessHeap(),
        0,
        ((len + 1) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    GetWindowTextW(hwndUsername, user, len + 1);

    if *user.offset(0 as isize) == 0 {
        HeapFree(GetProcessHeap(), 0, user as PVOID);
        return;
    }

    if wcschr(user as *const wchar_t, '\\' as i32 as wchar_t).is_null()
        && wcschr(user as *const wchar_t, '@' as i32 as wchar_t).is_null()
    {
        let len_target: ULONG = lstrlenW((*params).pszTargetName as LPCWSTR) as ULONG;
        memcpy(
            (*params).pszUsername as *mut core::ffi::c_void,
            (*params).pszTargetName as *const core::ffi::c_void,
            (min(len_target, (*params).ulUsernameMaxChars) as usize)
                .wrapping_mul(core::mem::size_of::<WCHAR>()),
        );
        if len_target.wrapping_add(1 as ULONG) < (*params).ulUsernameMaxChars {
            *(*params).pszUsername.offset(len_target as isize) = '\\' as i32 as WCHAR;
        }
        if len_target.wrapping_add(2 as ULONG) < (*params).ulUsernameMaxChars {
            *(*params)
                .pszUsername
                .offset(len_target.wrapping_add(1 as ULONG) as isize) = '\0' as i32 as WCHAR;
        }
    } else if (*params).ulUsernameMaxChars > 0 as ULONG {
        *(*params).pszUsername.offset(0 as isize) = '\0' as i32 as WCHAR;
    }

    len2 = lstrlenW((*params).pszUsername as LPCWSTR) as INT;
    memcpy(
        (*params).pszUsername.offset(len2 as isize) as *mut core::ffi::c_void,
        user as *const core::ffi::c_void,
        (min(
            len as ULONG,
            (*params).ulUsernameMaxChars.wrapping_sub(len2 as ULONG),
        ) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    if (*params).ulUsernameMaxChars != 0 {
        *(*params).pszUsername.offset(
            (len2 as ULONG).wrapping_add(min(
                len as ULONG,
                (*params)
                    .ulUsernameMaxChars
                    .wrapping_sub(len2 as ULONG)
                    .wrapping_sub(1 as ULONG),
            )) as isize,
        ) = '\0' as i32 as WCHAR;
    }

    HeapFree(GetProcessHeap(), 0, user as PVOID);

    GetDlgItemTextW(
        hwndDlg,
        (IDC_PASSWORD as i32),
        (*params).pszPassword as LPWSTR,
        (*params).ulPasswordMaxChars as INT,
    );

    (*params).fSave = (IsDlgButtonChecked(hwndDlg, (IDC_SAVE as i32))
        == BST_CHECKED) as BOOL;

    EndDialog(hwndDlg, IDOK as INT_PTR);
}

unsafe extern "system" fn CredDialogProc(hwndDlg: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            let params: *mut cred_dialog_params = lParam as *mut cred_dialog_params;

            return CredDialogInit(hwndDlg, params) as INT_PTR;
        }
        WM_COMMAND => {
            if wParam == MAKELONG!(IDOK, BN_CLICKED) as WPARAM {
                let params: *mut cred_dialog_params =
                    GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
                CredDialogCommandOk(hwndDlg, params);
                return TRUE as INT_PTR;
            } else if wParam == MAKELONG!(IDCANCEL, BN_CLICKED) as WPARAM {
                EndDialog(hwndDlg, IDCANCEL as INT_PTR);
                return TRUE as INT_PTR;
            } else if wParam == MAKELONG!(IDC_PASSWORD, EN_SETFOCUS) as WPARAM {
                if CredDialogCapsLockOn() != 0 {
                    let params: *mut cred_dialog_params =
                        GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
                    CredDialogShowCapsLockBalloon(hwndDlg, params);
                }
                /* don't allow another window to steal focus while the
                 * user is typing their password */
                LockSetForegroundWindow(LSFW_LOCK as UINT);
                return TRUE as INT_PTR;
            } else if wParam == MAKELONG!(IDC_PASSWORD, EN_KILLFOCUS) as WPARAM {
                let params: *mut cred_dialog_params =
                    GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
                /* the user is no longer typing their password, so allow
                 * other windows to become foreground ones */
                LockSetForegroundWindow(LSFW_UNLOCK as UINT);
                CredDialogHideBalloonTip(hwndDlg, params);
                return TRUE as INT_PTR;
            } else if wParam == MAKELONG!(IDC_PASSWORD, EN_CHANGE) as WPARAM {
                let params: *mut cred_dialog_params =
                    GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
                CredDialogHideBalloonTip(hwndDlg, params);
                return TRUE as INT_PTR;
            }
            return FALSE as INT_PTR;
        }
        WM_TIMER => {
            if wParam == ID_CAPSLOCKPOP as WPARAM {
                let params: *mut cred_dialog_params =
                    GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
                CredDialogHideBalloonTip(hwndDlg, params);
                return TRUE as INT_PTR;
            }
            return FALSE as INT_PTR;
        }
        WM_DESTROY => {
            let params: *mut cred_dialog_params =
                GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as *mut cred_dialog_params;
            if !(*params).hwndBalloonTip.is_null() {
                DestroyWindow((*params).hwndBalloonTip);
            }
            return TRUE as INT_PTR;
        }
        _ => return FALSE as INT_PTR,
    }
}

unsafe extern "C" fn find_existing_credential(
    target: *const WCHAR,
    username: *mut WCHAR,
    len_username: ULONG,
    password: *mut WCHAR,
    len_password: ULONG,
) -> BOOL {
    let mut count: DWORD = 0;
    let mut i: DWORD = 0;
    let mut credentials: *mut *mut CREDENTIALW = core::ptr::null_mut::<*mut CREDENTIALW>();

    if CredEnumerateW(
        target as LPCWSTR,
        0,
        core::ptr::addr_of_mut!(count),
        core::ptr::addr_of_mut!(credentials),
    ) == 0
    {
        return (FALSE) as BOOL;
    }
    i = 0;
    while i < count {
        if (**credentials.offset(i as isize)).Type != CRED_TYPE_DOMAIN_PASSWORD
            && (**credentials.offset(i as isize)).Type != CRED_TYPE_GENERIC
        {
            // FIXME/TRACE diagnostics for unsupported credential types elided in source.
        } else if (*username == 0
            || lstrcmpW(
                username as LPCWSTR,
                (**credentials.offset(i as isize)).UserName as LPCWSTR,
            ) == 0)
            && (lstrlenW((**credentials.offset(i as isize)).UserName as LPCWSTR)
                as ULONG)
                < len_username
            && ((**credentials.offset(i as isize)).CredentialBlobSize as usize)
                .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                < len_password as usize
        {
            // TRACE diagnostic elided in source.
            lstrcpyW(
                username as LPWSTR,
                (**credentials.offset(i as isize)).UserName as LPCWSTR,
            );
            memcpy(
                password as *mut core::ffi::c_void,
                (**credentials.offset(i as isize)).CredentialBlob as *const core::ffi::c_void,
                (**credentials.offset(i as isize)).CredentialBlobSize as usize,
            );
            *password.offset(
                ((**credentials.offset(i as isize)).CredentialBlobSize as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize) as isize,
            ) = 0;

            CredFree(credentials as PVOID);
            return (TRUE) as BOOL;
        }
        i = i.wrapping_add(1);
    }
    CredFree(credentials as PVOID);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIPromptForCredentialsW(
    mut pUIInfo: PCREDUI_INFOW,
    mut pszTargetName: PCWSTR,
    mut Reserved: PCtxtHandle,
    mut dwAuthError: DWORD,
    mut pszUsername: PWSTR,
    mut ulUsernameMaxChars: ULONG,
    mut pszPassword: PWSTR,
    mut ulPasswordMaxChars: ULONG,
    mut pfSave: PBOOL,
    mut dwFlags: DWORD,
) -> DWORD {
    let mut ret: INT_PTR = 0;
    let mut params: cred_dialog_params = cred_dialog_params {
        pszTargetName: core::ptr::null::<WCHAR>(),
        pszMessageText: core::ptr::null::<WCHAR>(),
        pszCaptionText: core::ptr::null::<WCHAR>(),
        hbmBanner: core::ptr::null_mut::<HBITMAP__>(),
        pszUsername: core::ptr::null_mut::<WCHAR>(),
        ulUsernameMaxChars: 0,
        pszPassword: core::ptr::null_mut::<WCHAR>(),
        ulPasswordMaxChars: 0,
        fSave: 0,
        dwFlags: 0,
        hwndBalloonTip: core::ptr::null_mut::<HWND__>(),
        fBalloonTipActive: 0,
    };
    let mut result: DWORD = ERROR_SUCCESS as DWORD;

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/credui/credui_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CredUIPromptForCredentialsW\0" as *const u8 as *const ::core::ffi::c_char,
            609 as ::core::ffi::c_int,
            b"(%p, %s, %p, %ld, %s, %ld, %p, %ld, %p, 0x%08lx)\n\0" as *const u8
                as *const ::core::ffi::c_char,
            pUIInfo,
            debugstr_w(pszTargetName as *const WCHAR),
            Reserved,
            dwAuthError,
            debugstr_w(pszUsername as *const WCHAR),
            ulUsernameMaxChars,
            pszPassword,
            ulPasswordMaxChars,
            pfSave,
            dwFlags,
        );
    }

    if (dwFlags & ((CREDUI_FLAGS_ALWAYS_SHOW_UI | CREDUI_FLAGS_GENERIC_CREDENTIALS) as DWORD))
        == (CREDUI_FLAGS_ALWAYS_SHOW_UI as DWORD)
    {
        return ERROR_INVALID_FLAGS as DWORD;
    }

    if pszTargetName.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    if (dwFlags & (CREDUI_FLAGS_SHOW_SAVE_CHECK_BOX as DWORD)) != 0 && pfSave.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }

    if (dwFlags & (CREDUI_FLAGS_ALWAYS_SHOW_UI as DWORD)) == 0
        && (dwFlags & (CREDUI_FLAGS_INCORRECT_PASSWORD as DWORD)) == 0
        && find_existing_credential(
            pszTargetName as *const WCHAR,
            pszUsername as *mut WCHAR,
            ulUsernameMaxChars,
            pszPassword as *mut WCHAR,
            ulPasswordMaxChars,
        ) != 0
    {
        return ERROR_SUCCESS as DWORD;
    }

    params.pszTargetName = pszTargetName;
    if !pUIInfo.is_null() {
        params.pszMessageText = (*pUIInfo).pszMessageText;
        params.pszCaptionText = (*pUIInfo).pszCaptionText;
        params.hbmBanner = (*pUIInfo).hbmBanner;
    } else {
        params.pszMessageText = core::ptr::null::<WCHAR>();
        params.pszCaptionText = core::ptr::null::<WCHAR>();
        params.hbmBanner = core::ptr::null_mut::<HBITMAP__>();
    }
    params.pszUsername = pszUsername;
    params.ulUsernameMaxChars = ulUsernameMaxChars;
    params.pszPassword = pszPassword;
    params.ulPasswordMaxChars = ulPasswordMaxChars;
    params.fSave = (if !pfSave.is_null() { *pfSave } else { FALSE as BOOL }) as BOOL;
    params.dwFlags = dwFlags;
    params.hwndBalloonTip = core::ptr::null_mut::<HWND__>();
    params.fBalloonTipActive = FALSE as BOOL;

    ret = DialogBoxParamW(
        hinstCredUI,
        MAKEINTRESOURCEW!(IDD_CREDDIALOG),
        if !pUIInfo.is_null() {
            (*pUIInfo).hwndParent
        } else {
            core::ptr::null_mut::<HWND__>()
        },
        Some(CredDialogProc as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> INT_PTR),
        &raw mut params as LPARAM,
    );
    if ret <= (0 as INT_PTR) {
        return GetLastError();
    }

    if ret == (IDCANCEL as INT_PTR) {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int)
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/credui/credui_main.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"CredUIPromptForCredentialsW\0" as *const u8 as *const ::core::ffi::c_char,
                655 as ::core::ffi::c_int,
                b"dialog cancelled\n\0" as *const u8 as *const ::core::ffi::c_char,
            );
        }
        return ERROR_CANCELLED as DWORD;
    }

    if !pfSave.is_null() {
        *pfSave = params.fSave;
    }

    if params.fSave != 0 {
        if (dwFlags & (CREDUI_FLAGS_EXPECT_CONFIRMATION as DWORD)) != 0 {
            let mut found: BOOL = FALSE as BOOL;
            let mut entry: *mut pending_credentials =
                core::ptr::null_mut::<pending_credentials>();
            let mut len: ::core::ffi::c_int = 0;

            EnterCriticalSection(&raw mut csPendingCredentials);

            /* find existing pending credentials for the same target and overwrite */
            /* FIXME: is this correct? */
            entry = (pending_credentials_list.next as *mut ::core::ffi::c_char).offset(
                -(core::mem::offset_of!(pending_credentials, entry) as isize),
            ) as *mut pending_credentials;
            while &raw mut (*entry).entry != &raw mut pending_credentials_list {
                if lstrcmpW(
                    pszTargetName as LPCWSTR,
                    (*entry).pszTargetName as LPCWSTR,
                ) == 0
                {
                    found = TRUE as BOOL;
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*entry).pszUsername as PVOID,
                    );
                    SecureZeroMemory(
                        (*entry).pszPassword as PVOID,
                        (lstrlenW((*entry).pszPassword as LPCWSTR) as SIZE_T)
                            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                    );
                    HeapFree(
                        GetProcessHeap(),
                        0 as DWORD,
                        (*entry).pszPassword as PVOID,
                    );
                }
                entry = ((*entry).entry.next as *mut ::core::ffi::c_char).offset(
                    -(core::mem::offset_of!(pending_credentials, entry) as isize),
                ) as *mut pending_credentials;
            }

            if found == 0 {
                entry = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    core::mem::size_of::<pending_credentials>() as SIZE_T,
                ) as *mut pending_credentials;
                len = lstrlenW(pszTargetName as LPCWSTR);
                (*entry).pszTargetName = HeapAlloc(
                    GetProcessHeap(),
                    0 as DWORD,
                    ((len + 1 as ::core::ffi::c_int) as SIZE_T)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
                ) as PWSTR;
                memcpy(
                    (*entry).pszTargetName as *mut ::core::ffi::c_void,
                    pszTargetName as *const ::core::ffi::c_void,
                    ((len + 1 as ::core::ffi::c_int) as size_t)
                        .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
                );
                list_add_tail(&raw mut pending_credentials_list, &raw mut (*entry).entry);
            }

            len = lstrlenW(params.pszUsername as LPCWSTR);
            (*entry).pszUsername = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                ((len + 1 as ::core::ffi::c_int) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            memcpy(
                (*entry).pszUsername as *mut ::core::ffi::c_void,
                params.pszUsername as *const ::core::ffi::c_void,
                ((len + 1 as ::core::ffi::c_int) as size_t)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            );

            len = lstrlenW(params.pszPassword as LPCWSTR);
            (*entry).pszPassword = HeapAlloc(
                GetProcessHeap(),
                0 as DWORD,
                ((len + 1 as ::core::ffi::c_int) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as PWSTR;
            memcpy(
                (*entry).pszPassword as *mut ::core::ffi::c_void,
                params.pszPassword as *const ::core::ffi::c_void,
                ((len + 1 as ::core::ffi::c_int) as size_t)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as size_t),
            );
            (*entry).generic = ((dwFlags & (CREDUI_FLAGS_GENERIC_CREDENTIALS as DWORD))
                != (0 as DWORD)) as ::core::ffi::c_int as BOOL;

            LeaveCriticalSection(&raw mut csPendingCredentials);
        } else if (dwFlags & (CREDUI_FLAGS_DO_NOT_PERSIST as DWORD)) == 0 {
            result = save_credentials(
                pszTargetName,
                pszUsername as PCWSTR,
                pszPassword as PCWSTR,
                ((dwFlags & (CREDUI_FLAGS_GENERIC_CREDENTIALS as DWORD))
                    != (0 as DWORD)) as ::core::ffi::c_int as BOOL,
            );
        }
    }

    return result;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIConfirmCredentialsW(
    pszTargetName: PCWSTR,
    bConfirm: BOOL,
) -> DWORD {
    let mut entry: *mut pending_credentials = core::ptr::null_mut::<pending_credentials>();
    let mut result: DWORD = ERROR_NOT_FOUND as DWORD;
    if (*__wine_dbch___default).flags as core::ffi::c_int
        & (1 as core::ffi::c_int) << __WINE_DBCL_TRACE as core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/credui/credui_main.c\0"
                as *const u8 as *const core::ffi::c_char,
            b"CredUIConfirmCredentialsW\0" as *const u8 as *const core::ffi::c_char,
            718 as core::ffi::c_int,
            b"(%s, %s)\n\0" as *const u8 as *const core::ffi::c_char,
            debugstr_w(pszTargetName as *const WCHAR),
            if bConfirm != 0 {
                b"TRUE\0" as *const u8 as *const core::ffi::c_char
            } else {
                b"FALSE\0" as *const u8 as *const core::ffi::c_char
            },
        );
    }
    if pszTargetName.is_null() {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    EnterCriticalSection(core::ptr::addr_of_mut!(csPendingCredentials));
    entry = ((*core::ptr::addr_of_mut!(pending_credentials_list)).next as *mut core::ffi::c_char)
        .offset(-(core::mem::offset_of!(pending_credentials, entry) as isize))
        as *mut pending_credentials;
    while core::ptr::addr_of_mut!((*entry).entry) != core::ptr::addr_of_mut!(pending_credentials_list) {
        if lstrcmpW(pszTargetName as LPCWSTR, (*entry).pszTargetName as LPCWSTR) == 0 {
            if bConfirm != 0 {
                result = save_credentials(
                    (*entry).pszTargetName as PCWSTR,
                    (*entry).pszUsername as PCWSTR,
                    (*entry).pszPassword as PCWSTR,
                    (*entry).generic,
                );
            } else {
                result = ERROR_SUCCESS as DWORD;
            }
            list_remove(core::ptr::addr_of_mut!((*entry).entry));
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*entry).pszTargetName as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*entry).pszUsername as PVOID,
            );
            SecureZeroMemory(
                (*entry).pszPassword as PVOID,
                (lstrlenW((*entry).pszPassword as LPCWSTR) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                (*entry).pszPassword as PVOID,
            );
            HeapFree(
                GetProcessHeap(),
                0 as DWORD,
                entry as PVOID,
            );
            break;
        } else {
            entry = ((*entry).entry.next as *mut core::ffi::c_char)
                .offset(-(core::mem::offset_of!(pending_credentials, entry) as isize))
                as *mut pending_credentials;
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(csPendingCredentials));
    return result;
}
#[no_mangle]
pub unsafe extern "system" fn CredUIParseUserNameW(
    mut pszUserName: PCWSTR,
    mut pszUser: PWSTR,
    mut ulMaxUserChars: ULONG,
    mut pszDomain: PWSTR,
    mut ulMaxDomainChars: ULONG,
) -> DWORD {
    let mut p: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/credui/credui_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CredUIParseUserNameW\0" as *const u8 as *const ::core::ffi::c_char,
            762 as ::core::ffi::c_int,
            b"(%s, %p, %ld, %p, %ld)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pszUserName as *const WCHAR),
            pszUser,
            ulMaxUserChars,
            pszDomain,
            ulMaxDomainChars,
        );
    }
    if pszUserName.is_null()
        || pszUser.is_null()
        || ulMaxUserChars == 0
        || pszDomain.is_null()
        || ulMaxDomainChars == 0
    {
        return ERROR_INVALID_PARAMETER as DWORD;
    }
    p = wcschr(pszUserName as *const wchar_t, '\\' as i32 as wchar_t) as PWSTR;
    if !p.is_null() {
        if p.offset_from(pszUserName) as ::core::ffi::c_longlong
            > ulMaxDomainChars.wrapping_sub(1 as ULONG) as ::core::ffi::c_longlong
        {
            return ERROR_INSUFFICIENT_BUFFER as DWORD;
        }
        if lstrlenW(p.offset(1 as isize) as LPCWSTR) as ULONG
            > ulMaxUserChars.wrapping_sub(1 as ULONG)
        {
            return ERROR_INSUFFICIENT_BUFFER as DWORD;
        }
        lstrcpyW(
            pszUser as LPWSTR,
            p.offset(1 as isize) as LPCWSTR,
        );
        memcpy(
            pszDomain as *mut ::core::ffi::c_void,
            pszUserName as *const ::core::ffi::c_void,
            (p.offset_from(pszUserName) as ::core::ffi::c_longlong as size_t)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        );
        *pszDomain.offset(p.offset_from(pszUserName) as ::core::ffi::c_longlong as isize) =
            '\0' as i32 as WCHAR;
        return ERROR_SUCCESS as DWORD;
    }
    p = wcsrchr(pszUserName as *const wchar_t, '@' as i32 as wchar_t) as PWSTR;
    if !p.is_null() {
        if p.offset(1 as isize)
            .offset_from(pszUserName) as ::core::ffi::c_longlong
            > ulMaxUserChars.wrapping_sub(1 as ULONG) as ::core::ffi::c_longlong
        {
            return ERROR_INSUFFICIENT_BUFFER as DWORD;
        }
        if lstrlenW(p.offset(1 as isize) as LPCWSTR) as ULONG
            > ulMaxDomainChars.wrapping_sub(1 as ULONG)
        {
            return ERROR_INSUFFICIENT_BUFFER as DWORD;
        }
        lstrcpyW(
            pszDomain as LPWSTR,
            p.offset(1 as isize) as LPCWSTR,
        );
        memcpy(
            pszUser as *mut ::core::ffi::c_void,
            pszUserName as *const ::core::ffi::c_void,
            (p.offset_from(pszUserName) as ::core::ffi::c_longlong as size_t)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as size_t),
        );
        *pszUser.offset(p.offset_from(pszUserName) as ::core::ffi::c_longlong as isize) =
            '\0' as i32 as WCHAR;
        return ERROR_SUCCESS as DWORD;
    }
    if lstrlenW(pszUserName as LPCWSTR) as ULONG
        > ulMaxUserChars.wrapping_sub(1 as ULONG)
    {
        return ERROR_INSUFFICIENT_BUFFER as DWORD;
    }
    lstrcpyW(pszUser as LPWSTR, pszUserName as LPCWSTR);
    *pszDomain.offset(0 as isize) = '\0' as i32 as WCHAR;
    return ERROR_SUCCESS as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIStoreSSOCredA(
    pszRealm: PCSTR,
    pszUsername: PCSTR,
    pszPassword: PCSTR,
    bPersist: BOOL,
) -> DWORD {
    // FIXME("(%s, %s, %p, %d)\n", debugstr_a(pszRealm), debugstr_a(pszUsername), pszPassword, bPersist);
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CredUIStoreSSOCredW(
    pszRealm: PCWSTR,
    pszUsername: PCWSTR,
    pszPassword: PCWSTR,
    bPersist: BOOL,
) -> DWORD {
    // FIXME("(%s, %s, %p, %d)\n", debugstr_w(pszRealm), debugstr_w(pszUsername), pszPassword, bPersist);
    ERROR_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CredUIReadSSOCredA(pszRealm: PCSTR, ppszUsername: *mut PSTR) -> DWORD {
    if !ppszUsername.is_null() {
        *ppszUsername = core::ptr::null_mut();
    }
    return ERROR_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIReadSSOCredW(
    mut pszRealm: PCWSTR,
    mut ppszUsername: *mut PWSTR,
) -> DWORD {
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_FIXME as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_FIXME;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/credui/credui_main.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"CredUIReadSSOCredW\0" as *const u8 as *const ::core::ffi::c_char,
            844 as ::core::ffi::c_int,
            b"(%s, %p)\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(pszRealm as *const WCHAR),
            ppszUsername,
        );
    }
    if !ppszUsername.is_null() {
        *ppszUsername = ::core::ptr::null_mut::<WCHAR>();
    }
    return ERROR_NOT_FOUND;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIInitControls() -> BOOL {
    // FIXME("() stub\n");
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SspiPromptForCredentialsW(
    target: PCWSTR,
    info: *mut ::core::ffi::c_void,
    error: DWORD,
    package: PCWSTR,
    input_id: PSEC_WINNT_AUTH_IDENTITY_OPAQUE,
    output_id: *mut PSEC_WINNT_AUTH_IDENTITY_OPAQUE,
    save: *mut BOOL,
    sspi_flags: DWORD,
) -> ULONG {
    let mut username: [WCHAR; ((CREDUI_MAX_USERNAME_LENGTH + 1)) as _] =
        [0 as WCHAR; ((CREDUI_MAX_USERNAME_LENGTH + 1)) as _];
    let mut password: [WCHAR; ((CREDUI_MAX_PASSWORD_LENGTH + 1)) as _] =
        [0 as WCHAR; ((CREDUI_MAX_PASSWORD_LENGTH + 1)) as _];
    let mut len_username: DWORD = ARRAY_SIZE!(username) as DWORD;
    let mut len_password: DWORD = ARRAY_SIZE!(password) as DWORD;
    let mut ret: DWORD = 0;
    let mut flags: DWORD = 0;
    let mut cred_info: *mut CREDUI_INFOW = info as *mut CREDUI_INFOW;
    let mut id: *mut SEC_WINNT_AUTH_IDENTITY_W = input_id as *mut SEC_WINNT_AUTH_IDENTITY_W;

    if target.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    if package.is_null()
        || (wcsicmp(package, RUSTCONV_WSTR_2F930CE27772.as_ptr()) != 0
            && wcsicmp(package, RUSTCONV_WSTR_A2F5C72A8C4B.as_ptr()) != 0
            && wcsicmp(package, RUSTCONV_WSTR_C057620377F9.as_ptr()) != 0)
    {
        return ERROR_NO_SUCH_PACKAGE;
    }

    flags = CREDUI_FLAGS_ALWAYS_SHOW_UI | CREDUI_FLAGS_GENERIC_CREDENTIALS;

    if (sspi_flags & SSPIPFC_CREDPROV_DO_NOT_SAVE) != 0 {
        flags |= CREDUI_FLAGS_DO_NOT_PERSIST;
    }

    if (sspi_flags & SSPIPFC_NO_CHECKBOX) == 0 {
        flags |= CREDUI_FLAGS_SHOW_SAVE_CHECK_BOX;
    }

    if id.is_null() {
        find_existing_credential(
            target,
            core::ptr::addr_of_mut!(username) as *mut WCHAR,
            len_username as ULONG,
            core::ptr::addr_of_mut!(password) as *mut WCHAR,
            len_password as ULONG,
        );
    } else {
        if !(*id).User.is_null()
            && (*id).UserLength > 0
            && (*id).UserLength <= CREDUI_MAX_USERNAME_LENGTH
        {
            memcpy(
                core::ptr::addr_of_mut!(username) as *mut WCHAR as *mut ::core::ffi::c_void,
                (*id).User as *const ::core::ffi::c_void,
                ((*id).UserLength as usize) * (core::mem::size_of::<WCHAR>()),
            );
            username[(*id).UserLength as usize] = 0;
        }
        if !(*id).Password.is_null()
            && (*id).PasswordLength > 0
            && (*id).PasswordLength <= CREDUI_MAX_PASSWORD_LENGTH
        {
            memcpy(
                core::ptr::addr_of_mut!(password) as *mut WCHAR as *mut ::core::ffi::c_void,
                (*id).Password as *const ::core::ffi::c_void,
                ((*id).PasswordLength as usize) * (core::mem::size_of::<WCHAR>()),
            );
            password[(*id).PasswordLength as usize] = 0;
        }
    }

    ret = CredUIPromptForCredentialsW(
        cred_info as PCREDUI_INFOW,
        target,
        core::ptr::null_mut(),
        error,
        core::ptr::addr_of_mut!(username) as PWSTR,
        len_username as ULONG,
        core::ptr::addr_of_mut!(password) as PWSTR,
        len_password as ULONG,
        save as PBOOL,
        flags,
    );
    if ret == 0 {
        let mut size: DWORD = core::mem::size_of::<SEC_WINNT_AUTH_IDENTITY_W>() as DWORD;
        let mut len_domain: DWORD = 0;
        let mut ptr: *mut WCHAR = core::ptr::null_mut();
        let mut user: *mut WCHAR = core::ptr::addr_of_mut!(username) as *mut WCHAR;
        let mut domain: *mut WCHAR = core::ptr::null_mut();

        ptr = wcschr(
            core::ptr::addr_of_mut!(username) as *mut WCHAR,
            '\\' as WCHAR,
        ) as *mut WCHAR;
        if !ptr.is_null() {
            user = ptr.offset(1 as isize);
            len_username = lstrlenW(user) as DWORD;
            if wcsicmp(package, RUSTCONV_WSTR_A2F5C72A8C4B.as_ptr()) == 0
                || wcsicmp(package, RUSTCONV_WSTR_C057620377F9.as_ptr()) == 0
            {
                domain = core::ptr::addr_of_mut!(username) as *mut WCHAR;
                len_domain = ptr.offset_from(core::ptr::addr_of_mut!(username) as *mut WCHAR)
                    as DWORD;
            }
            *ptr = 0;
        } else {
            len_username =
                lstrlenW(core::ptr::addr_of_mut!(username) as *mut WCHAR) as DWORD;
        }
        len_password =
            lstrlenW(core::ptr::addr_of_mut!(password) as *mut WCHAR) as DWORD;

        size += (len_username + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        size += (len_domain + 1) * (core::mem::size_of::<WCHAR>() as DWORD);
        size += (len_password + 1)
            * (core::mem::size_of::<WCHAR>() as DWORD);
        id = HeapAlloc(
            GetProcessHeap(),
            0,
            size as SIZE_T,
        ) as *mut SEC_WINNT_AUTH_IDENTITY_W;
        if id.is_null() {
            return ERROR_OUTOFMEMORY;
        }
        ptr = id.offset(1 as isize) as *mut WCHAR;

        memcpy(
            ptr as *mut ::core::ffi::c_void,
            user as *const ::core::ffi::c_void,
            ((len_username + 1) as usize) * (core::mem::size_of::<WCHAR>()),
        );
        (*id).User = ptr as *mut ::core::ffi::c_ushort;
        (*id).UserLength = len_username as ULONG;
        ptr = ptr.offset((len_username + 1) as isize);
        if len_domain != 0 {
            memcpy(
                ptr as *mut ::core::ffi::c_void,
                domain as *const ::core::ffi::c_void,
                ((len_domain + 1) as usize)
                    * (core::mem::size_of::<WCHAR>()),
            );
            (*id).Domain = ptr as *mut ::core::ffi::c_ushort;
            (*id).DomainLength = len_domain as ULONG;
            ptr = ptr.offset((len_domain + 1) as isize);
        } else {
            (*id).Domain = core::ptr::null_mut();
            (*id).DomainLength = 0;
        }
        memcpy(
            ptr as *mut ::core::ffi::c_void,
            core::ptr::addr_of_mut!(password) as *mut WCHAR as *const ::core::ffi::c_void,
            ((len_password + 1) as usize) * (core::mem::size_of::<WCHAR>()),
        );
        (*id).Password = ptr as *mut ::core::ffi::c_ushort;
        (*id).PasswordLength = len_password as ULONG;
        (*id).Flags = 0;

        *output_id = id as PSEC_WINNT_AUTH_IDENTITY_OPAQUE;
    }

    return ret as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn CredUIPromptForWindowsCredentialsW(
    info: *mut CREDUI_INFOW,
    error: DWORD,
    package: *mut ULONG,
    in_buf: *const core::ffi::c_void,
    in_buf_size: ULONG,
    out_buf: *mut *mut core::ffi::c_void,
    out_buf_size: *mut ULONG,
    save: *mut BOOL,
    flags: DWORD,
) -> DWORD {
    // FIXME( "(%p, %lu, %p, %p, %lu, %p, %p, %p, %08lx) stub\n", info, error, package, in_buf, in_buf_size, out_buf, out_buf_size, save, flags );
    ERROR_CALL_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn CredPackAuthenticationBufferW(
    flags: DWORD,
    username: *mut WCHAR,
    password: *mut WCHAR,
    buf: *mut BYTE,
    size: *mut DWORD,
) -> BOOL {
    // FIXME( "(%08lx, %s, %p, %p, %p) stub\n", flags, debugstr_w(username), password, buf, size );
    (ERROR_CALL_NOT_IMPLEMENTED as i32)
}

#[no_mangle]
pub unsafe extern "system" fn CredUnPackAuthenticationBufferW(
    flags: DWORD,
    buf: *mut core::ffi::c_void,
    size: DWORD,
    username: *mut WCHAR,
    len_username: *mut DWORD,
    domain: *mut WCHAR,
    len_domain: *mut DWORD,
    password: *mut WCHAR,
    len_password: *mut DWORD,
) -> BOOL {
    // FIXME( "(%08lx, %p, %lu, %p, %p, %p, %p, %p, %p) stub\n", flags, buf, size, username, len_username, domain, len_domain, password, len_password );
    (ERROR_CALL_NOT_IMPLEMENTED as i32)
}
