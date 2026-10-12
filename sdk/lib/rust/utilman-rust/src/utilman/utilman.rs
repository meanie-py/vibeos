// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_91759CD7CCE7: [u16; 12] = [
    0x0055u16, 0x004D, 0x0061, 0x006E, 0x0044, 0x006C, 0x0067, 0x002E, 0x0064, 0x006C,
    0x006C, 0x0000,
];

use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInstance: HINSTANCE,
    hPrevInstance: HINSTANCE,
    pCmdLine: LPWSTR,
    nCmdShow: INT,
) -> INT {
    let mut hModule: HMODULE;
    let mut szFormat: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut szFailLoad: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut szTitle: [WCHAR; (MAX_BUFFER) as _] = [0; (MAX_BUFFER) as _];
    let mut UManStartDlg: EXECDLGROUTINE;

    UNREFERENCED_PARAMETER!(hPrevInstance);
    UNREFERENCED_PARAMETER!(pCmdLine);
    UNREFERENCED_PARAMETER!(nCmdShow);

    /* Load the main resources module of Utility Manager */
    hModule = LoadLibraryW(RUSTCONV_WSTR_91759CD7CCE7.as_ptr()) as HMODULE;
    if hModule.is_null() {
        LoadStringW(
            hInstance,
            IDS_FAIL_INIT as UINT,
            szFormat.as_mut_ptr(),
            (_countof!(szFormat)) as i32,
        );
        LoadStringW(
            hInstance,
            IDS_FAIL_INIT_TITLE as UINT,
            szTitle.as_mut_ptr(),
            (_countof!(szTitle)) as i32,
        );

        StringCchPrintfW!(
            szFailLoad.as_mut_ptr(),
            _countof!(szFailLoad),
            szFormat.as_mut_ptr(),
            GetLastError(),
        );
        MessageBoxW(
            GetDesktopWindow(),
            szFailLoad.as_mut_ptr(),
            szTitle.as_mut_ptr(),
            MB_ICONERROR | MB_OK,
        );
        return -(1);
    }

    /* Get the function address and launch Utility Manager */
    UManStartDlg = core::mem::transmute::<FARPROC, EXECDLGROUTINE>(
        GetProcAddress(hModule as HINSTANCE, b"UManStartDlg\0".as_ptr() as LPCSTR),
    );
    (UManStartDlg.unwrap_unchecked())();

    FreeLibrary(hModule);
    return 0;
}
