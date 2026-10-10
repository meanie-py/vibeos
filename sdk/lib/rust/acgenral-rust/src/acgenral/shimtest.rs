use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// WIN32_NO_STATUS: empty define, nothing to emit.
// SHIM_NS ShimTest: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(ShimTest);

pub type tGetComputerNameA = unsafe extern "system" fn(lpBuffer: LPSTR, lpnSize: LPDWORD) -> BOOL;
pub type tSHStringFromGUIDA = unsafe extern "system" fn(guid: REFGUID, lpszDest: LPSTR, cchMax: INT) -> INT;


pub unsafe extern "system" fn GetComputerNameA(lpBuffer: LPSTR, lpnSize: LPDWORD) -> BOOL {
    let mut Buffer: [CHAR; 100] = [0; 100];
    let mut dwSize: DWORD = (core::mem::size_of::<[CHAR; 100]>() - 1) as DWORD;
    let mut cchLen: usize = 0;

    if CALL_SHIM!(0, tGetComputerNameA)(Buffer.as_mut_ptr(), &mut dwSize) != 0 {
        SHIM_INFO!("Original function returned: '%s'\n", Buffer.as_ptr());
    } else {
        SHIM_INFO!("Original function failed\n");
        Buffer[0] = 0;
    }

    if lpnSize.is_null() {
        SHIM_WARN!("No lpnSize\n");
        return FALSE as BOOL;
    }

    StringCchCopyA(lpBuffer, *lpnSize as usize, b"ShimTest:\0".as_ptr() as *const core::ffi::c_char);
    StringCchCatA(lpBuffer, *lpnSize as usize, Buffer.as_ptr());
    StringCchLengthA(lpBuffer, *lpnSize as usize, &mut cchLen);
    *lpnSize = cchLen as DWORD;
    return TRUE as BOOL;
}

pub unsafe extern "system" fn SHStringFromGUIDA(guid: REFGUID, lpszDest: LPSTR, cchMax: INT) -> INT {
    let mut Buffer: [CHAR; 100] = [0; 100];

    if CALL_SHIM!(1, tSHStringFromGUIDA)(
        guid,
        Buffer.as_mut_ptr(),
        (core::mem::size_of::<[CHAR; 100]>() - 1) as INT,
    ) != 0
    {
        SHIM_INFO!("Original function returned: '%s'\n", Buffer.as_ptr());
    } else {
        SHIM_INFO!("Original function failed\n");
        Buffer[0] = 0;
    }

    StringCchCopyA(lpszDest, cchMax as usize, b"ShimTest:\0".as_ptr() as *const core::ffi::c_char);
    StringCchCatA(lpszDest, cchMax as usize, Buffer.as_ptr());
    return 0;
}

pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    if fdwReason == SHIM_REASON_INIT {
        SHIM_MSG!("SHIM_REASON_INIT\n");
    }
    return TRUE as BOOL;
}

// SHIM_NOTIFY_FN = SHIM_OBJ_NAME(Notify); SHIM_NUM_HOOKS 2; slot 1 hooks SHLWAPI ordinal 23.
implement_shim!(ShimTest, 2usize, Some(Notify), [
    (0, "KERNEL32.DLL", "GetComputerNameA", GetComputerNameA),
    (1, "SHLWAPI.DLL", [23], SHStringFromGUIDA),
]);
