use crate::cmd::misc::IsExistingFile;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const ENV_BUFFER_SIZE: usize = 1024;
#[no_mangle]
pub unsafe extern "C" fn SearchForExecutableSingle(pFileName: LPCTSTR, pFullName: LPTSTR, pPathExt: LPTSTR, pDirectory: LPTSTR) -> BOOL {
    let mut szPathBuffer: [TCHAR; CMDLINE_LENGTH] = [0; CMDLINE_LENGTH];
    let mut pszPathEnd: *mut TCHAR;
    let mut s: LPTSTR;
    let mut f: LPTSTR;
    /* initialize full name buffer */
    *pFullName = _T('\0');

    pszPathEnd = szPathBuffer.as_mut_ptr();
    if !pDirectory.is_null() {
        _tcscpy(szPathBuffer.as_mut_ptr(), pDirectory);
        pszPathEnd = pszPathEnd.add(_tcslen(pszPathEnd) as usize);
        let fresh = pszPathEnd;
        pszPathEnd = pszPathEnd.add(1);
        *fresh = _T('\\');
    }
    _tcscpy(pszPathEnd, pFileName);
    pszPathEnd = pszPathEnd.add(_tcslen(pszPathEnd) as usize);

    if IsExistingFile(szPathBuffer.as_mut_ptr()) != 0 {
        GetFullPathName(szPathBuffer.as_mut_ptr(), MAX_PATH, pFullName, core::ptr::null_mut());
        return TRUE;
    }

    s = pPathExt;
    while !s.is_null() && *s != 0 {
        f = _tcschr(s, _T(';'));

        if !f.is_null() {
            _tcsncpy(pszPathEnd, s, f.offset_from(s) as usize);
            *pszPathEnd.offset(f.offset_from(s)) = _T('\0');
            s = f.add(1 as usize);
        } else {
            _tcscpy(pszPathEnd, s);
            s = core::ptr::null_mut();
        }

        if IsExistingFile(szPathBuffer.as_mut_ptr()) != 0 {
            GetFullPathName(szPathBuffer.as_mut_ptr(), MAX_PATH, pFullName, core::ptr::null_mut());
            return TRUE;
        }
    }
    return FALSE;
}
#[no_mangle]
pub unsafe extern "C" fn SearchForExecutable(pFileName: LPCTSTR, pFullName: LPTSTR) -> BOOL {
    static mut pszDefaultPathExt: [TCHAR; 20] = [
        '.' as TCHAR, 'c' as TCHAR, 'o' as TCHAR, 'm' as TCHAR, ';' as TCHAR,
        '.' as TCHAR, 'e' as TCHAR, 'x' as TCHAR, 'e' as TCHAR, ';' as TCHAR,
        '.' as TCHAR, 'b' as TCHAR, 'a' as TCHAR, 't' as TCHAR, ';' as TCHAR,
        '.' as TCHAR, 'c' as TCHAR, 'm' as TCHAR, 'd' as TCHAR, 0,
    ];
    let mut pszPathExt: LPTSTR;
    let mut pszPath: LPTSTR;
    let mut pCh: LPTSTR;
    let mut dwBuffer: DWORD;

    let pathext_name: [TCHAR; 8] = [
        'P' as TCHAR, 'A' as TCHAR, 'T' as TCHAR, 'H' as TCHAR,
        'E' as TCHAR, 'X' as TCHAR, 'T' as TCHAR, 0,
    ];
    let path_name: [TCHAR; 5] = ['P' as TCHAR, 'A' as TCHAR, 'T' as TCHAR, 'H' as TCHAR, 0];
    let semi: [TCHAR; 2] = [';' as TCHAR, 0];

    /* load environment variable PATHEXT */
    pszPathExt = cmd_alloc(ENV_BUFFER_SIZE * core::mem::size_of::<TCHAR>()) as LPTSTR;
    if pszPathExt.is_null()
    {
        return FALSE;
    }

    dwBuffer = GetEnvironmentVariable(pathext_name.as_ptr() as LPCTSTR, pszPathExt, ENV_BUFFER_SIZE);
    if dwBuffer > ENV_BUFFER_SIZE
    {
        let mut pszOldPathExt: LPTSTR = pszPathExt;
        pszPathExt = cmd_realloc(pszPathExt as *mut core::ffi::c_void, (dwBuffer as usize) * core::mem::size_of::<TCHAR>()) as LPTSTR;
        if pszPathExt.is_null()
        {
            cmd_free(pszOldPathExt as *mut core::ffi::c_void);
            return FALSE;
        }
        GetEnvironmentVariable(pathext_name.as_ptr() as LPCTSTR, pszPathExt, dwBuffer);
        _tcslwr(pszPathExt);
    }
    else if 0 == dwBuffer
    {
        _tcscpy(pszPathExt, core::ptr::addr_of_mut!(pszDefaultPathExt) as LPTSTR);
    }
    else
    {
        _tcslwr(pszPathExt);
    }

    /* Check if valid directly on specified path */
    if SearchForExecutableSingle(pFileName, pFullName, pszPathExt, core::ptr::null_mut()) != 0
    {
        cmd_free(pszPathExt as *mut core::ffi::c_void);
        return TRUE;
    }

    /* If an explicit directory was given, stop here - no need to search PATH. */
    if *pFileName.add(1) == ':' as TCHAR || !_tcschr(pFileName, '\\' as TCHAR).is_null()
    {
        cmd_free(pszPathExt as *mut core::ffi::c_void);
        return FALSE;
    }

    /* load environment variable PATH into buffer */
    pszPath = cmd_alloc(ENV_BUFFER_SIZE * core::mem::size_of::<TCHAR>()) as LPTSTR;
    if pszPath.is_null()
    {
        return FALSE;
    }

    dwBuffer = GetEnvironmentVariable(path_name.as_ptr() as LPCTSTR, pszPath, ENV_BUFFER_SIZE);
    if dwBuffer > ENV_BUFFER_SIZE
    {
        let mut pszOldPath: LPTSTR = pszPath;
        pszPath = cmd_realloc(pszPath as *mut core::ffi::c_void, (dwBuffer as usize) * core::mem::size_of::<TCHAR>()) as LPTSTR;
        if pszPath.is_null()
        {
            cmd_free(pszOldPath as *mut core::ffi::c_void);
            cmd_free(pszPathExt as *mut core::ffi::c_void);
            return FALSE;
        }
        GetEnvironmentVariable(path_name.as_ptr() as LPCTSTR, pszPath, dwBuffer);
    }

    /* search in PATH */
    pCh = _tcstok(pszPath, semi.as_ptr() as LPCTSTR) as LPTSTR;
    while !pCh.is_null()
    {
        if SearchForExecutableSingle(pFileName, pFullName, pszPathExt, pCh) != 0
        {
            cmd_free(pszPath as *mut core::ffi::c_void);
            cmd_free(pszPathExt as *mut core::ffi::c_void);
            return TRUE;
        }
        pCh = _tcstok(core::ptr::null_mut(), semi.as_ptr() as LPCTSTR) as LPTSTR;
    }

    cmd_free(pszPath as *mut core::ffi::c_void);
    cmd_free(pszPathExt as *mut core::ffi::c_void);
    return FALSE;
}
