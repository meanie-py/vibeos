use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: ::core::ffi::c_int, argv: *mut *mut TCHAR) -> ::core::ffi::c_int {
    UNREFERENCED_PARAMETER!(argc);
    UNREFERENCED_PARAMETER!(argv);

    if !(NotifyBootConfigStatus((TRUE) as BOOL) != 0) {
        let fmt: [u16; 45] = [
            'N' as u16, 'o' as u16, 't' as u16, 'i' as u16, 'f' as u16, 'y' as u16,
            'B' as u16, 'o' as u16, 'o' as u16, 't' as u16, 'C' as u16, 'o' as u16,
            'n' as u16, 'f' as u16, 'i' as u16, 'g' as u16, 'S' as u16, 't' as u16,
            'a' as u16, 't' as u16, 'u' as u16, 's' as u16, ' ' as u16, 'f' as u16,
            'a' as u16, 'i' as u16, 'l' as u16, 'e' as u16, 'd' as u16, '!' as u16,
            ' ' as u16, '(' as u16, 'E' as u16, 'r' as u16, 'r' as u16, 'o' as u16,
            'r' as u16, ':' as u16, ' ' as u16, '%' as u16, 'l' as u16, 'u' as u16,
            ')' as u16, '\n' as u16, 0,
        ];
        _tprintf(
            (fmt.as_ptr()),
            GetLastError(),
        );
    }

    return 0;
}
