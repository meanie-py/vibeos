use crate::prelude::*;
use core::ffi::c_void;
#[no_mangle]
pub unsafe extern "C" fn CheckAccountName(hwndDlg: HWND, nIdDlgItem: INT, lpAccountName: LPTSTR) -> BOOL {
    let mut szAccountName: [TCHAR; 256] = [0; 256];
    let mut uLen: UINT;

    if !lpAccountName.is_null() {
        uLen = _tcslen(lpAccountName as *const TCHAR) as UINT;
    } else {
        uLen = GetDlgItemText(
            hwndDlg,
            nIdDlgItem,
            szAccountName.as_mut_ptr(),
            256,
        ) as UINT;
    }

    let specials: [u16; 16] = [
        '"' as u16, '*' as u16, '+' as u16, ',' as u16, '/' as u16, '\\' as u16,
        ':' as u16, ';' as u16, '<' as u16, '=' as u16, '>' as u16, '?' as u16,
        '[' as u16, ']' as u16, '|' as u16, 0,
    ];
    let title = ['E' as u16, 'R' as u16, 'R' as u16, 'O' as u16, 'R' as u16, 0];
    let msg = [
        'T' as u16, 'h' as u16, 'e' as u16, ' ' as u16, 'a' as u16, 'c' as u16,
        'c' as u16, 'o' as u16, 'u' as u16, 'n' as u16, 't' as u16, ' ' as u16,
        'n' as u16, 'a' as u16, 'm' as u16, 'e' as u16, ' ' as u16, 'y' as u16,
        'o' as u16, 'u' as u16, ' ' as u16, 'e' as u16, 'n' as u16, 't' as u16,
        'e' as u16, 'r' as u16, 'e' as u16, 'd' as u16, ' ' as u16, 'i' as u16,
        's' as u16, ' ' as u16, 'i' as u16, 'n' as u16, 'v' as u16, 'a' as u16,
        'l' as u16, 'i' as u16, 'd' as u16, '!' as u16, ' ' as u16, 'A' as u16,
        'n' as u16, ' ' as u16, 'a' as u16, 'c' as u16, 'c' as u16, 'o' as u16,
        'u' as u16, 'n' as u16, 't' as u16, ' ' as u16, 'n' as u16, 'a' as u16,
        'm' as u16, 'e' as u16, ' ' as u16, 'm' as u16, 'u' as u16, 's' as u16,
        't' as u16, ' ' as u16, 'n' as u16, 'o' as u16, 't' as u16, ' ' as u16,
        'c' as u16, 'o' as u16, 'n' as u16, 't' as u16, 'a' as u16, 'i' as u16,
        'n' as u16, ' ' as u16, 't' as u16, 'h' as u16, 'e' as u16, ' ' as u16,
        'f' as u16, 'o' as u16, 'l' as u16, 'l' as u16, 'o' as u16, 'w' as u16,
        'i' as u16, 'n' as u16, 'g' as u16, ' ' as u16, 'c' as u16, 'h' as u16,
        'a' as u16, 'r' as u16, 'a' as u16, 'c' as u16, 't' as u16, 'e' as u16,
        'r' as u16, 's' as u16, ':' as u16, ' ' as u16, '*' as u16, '+' as u16,
        ',' as u16, '/' as u16, ':' as u16, ';' as u16, '<' as u16, '=' as u16,
        '>' as u16, '?' as u16, '[' as u16, '\\' as u16, ']' as u16, '|' as u16,
        0,
    ];

    /* Check the account name */
    if uLen > 0 &&
        !_tcspbrk(
            if !lpAccountName.is_null() { lpAccountName } else { szAccountName.as_mut_ptr() },
            specials.as_ptr(),
        )
        .is_null()
    {
        MessageBox(
            hwndDlg,
            msg.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONERROR,
        );
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}
