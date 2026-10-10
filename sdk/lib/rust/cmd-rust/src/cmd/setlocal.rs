use crate::cmd::error::error_invalid_parameter_format;
use crate::cmd::error::error_out_of_memory;
use crate::cmd::misc::freep;
use crate::cmd::misc::splitspace;
use ntdll_sys::nt::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct SETLOCAL {
    pub Prev: *mut SETLOCAL,
    pub Environment: LPTSTR,
    pub CurDrive: i32,
    pub EnableExtensions: i32,
    pub DelayedExpansion: i32,
}

pub type PSETLOCAL = *mut SETLOCAL;
#[no_mangle]
pub unsafe extern "C" fn DuplicateEnvironment() -> LPTSTR {
    let mut Environ: LPTSTR = GetEnvironmentStrings();
    let mut End: LPTSTR = core::ptr::null_mut();
    let mut EnvironCopy: LPTSTR = core::ptr::null_mut();

    if Environ.is_null() {
        return core::ptr::null_mut();
    }

    End = Environ;
    while *End != 0 {
        End = End.add((_tcslen(End) + 1) as usize);
    }
    EnvironCopy = cmd_alloc(
        ((End.add(1 as usize)).offset_from(Environ) as usize)
            * core::mem::size_of::<TCHAR>(),
    ) as LPTSTR;

    if !EnvironCopy.is_null() {
        memcpy(
            EnvironCopy as *mut core::ffi::c_void,
            Environ as *const core::ffi::c_void,
            ((End.add(1 as usize)).offset_from(Environ) as usize)
                * core::mem::size_of::<TCHAR>(),
        );
    }

    FreeEnvironmentStrings(Environ);
    return EnvironCopy;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_setlocal(mut param: LPTSTR) -> INT {
    let mut Saved: PSETLOCAL = core::ptr::null_mut();
    let mut arg: *mut LPTSTR = core::ptr::null_mut();
    let mut argc: INT = 0;
    let mut i: INT = 0;

    if _tcscmp(param, ['/' as u16, '?' as u16, 0].as_ptr() as LPTSTR) == 0 {
        // FIXME
        let msg = [
            'S' as u16, 'E' as u16, 'T' as u16, 'L' as u16, 'O' as u16, 'C' as u16, 'A' as u16, 'L' as u16,
            ' ' as u16, 'h' as u16, 'e' as u16, 'l' as u16, 'p' as u16, ' ' as u16,
            'n' as u16, 'o' as u16, 't' as u16, ' ' as u16,
            'i' as u16, 'm' as u16, 'p' as u16, 'l' as u16, 'e' as u16, 'm' as u16, 'e' as u16, 'n' as u16,
            't' as u16, 'e' as u16, 'd' as u16, ' ' as u16,
            'y' as u16, 'e' as u16, 't' as u16, '!' as u16, '\n' as u16, 0,
        ];
        ConOutPuts(msg.as_ptr() as LPTSTR);
        return 0;
    }

    /* SETLOCAL only works inside a batch context */
    if bc.is_null() {
        return 0;
    }

    Saved = cmd_alloc(core::mem::size_of::<SETLOCAL>()) as PSETLOCAL;
    if Saved.is_null() {
        error_out_of_memory();
        return 1;
    }

    (*Saved).Environment = DuplicateEnvironment();
    if (*Saved).Environment.is_null() {
        error_out_of_memory();
        cmd_free(Saved as *mut core::ffi::c_void);
        return 1;
    }
    /*
     * Save the current drive; the duplicated environment
     * contains the corresponding current directory.
     */
    (*Saved).CurDrive = _getdrive() as INT;

    (*Saved).EnableExtensions = bEnableExtensions;
    (*Saved).DelayedExpansion = bDelayedExpansion;

    (*Saved).Prev = (*bc).setlocal;
    (*bc).setlocal = Saved;

    nErrorLevel = 0;

    arg = splitspace(param, core::ptr::addr_of_mut!(argc));
    i = 0;
    while i < argc {
        if _tcsicmp(
            *arg.offset(i as isize),
            [
                'E' as u16, 'N' as u16, 'A' as u16, 'B' as u16, 'L' as u16, 'E' as u16,
                'E' as u16, 'X' as u16, 'T' as u16, 'E' as u16, 'N' as u16, 'S' as u16,
                'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, 0,
            ].as_ptr() as LPTSTR,
        ) == 0
        {
            bEnableExtensions = TRUE;
        } else if _tcsicmp(
            *arg.offset(i as isize),
            [
                'D' as u16, 'I' as u16, 'S' as u16, 'A' as u16, 'B' as u16, 'L' as u16,
                'E' as u16, 'E' as u16, 'X' as u16, 'T' as u16, 'E' as u16, 'N' as u16,
                'S' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 'S' as u16, 0,
            ].as_ptr() as LPTSTR,
        ) == 0
        {
            bEnableExtensions = FALSE;
        } else if _tcsicmp(
            *arg.offset(i as isize),
            [
                'E' as u16, 'N' as u16, 'A' as u16, 'B' as u16, 'L' as u16, 'E' as u16,
                'D' as u16, 'E' as u16, 'L' as u16, 'A' as u16, 'Y' as u16, 'E' as u16,
                'D' as u16, 'E' as u16, 'X' as u16, 'P' as u16, 'A' as u16, 'N' as u16,
                'S' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 0,
            ].as_ptr() as LPTSTR,
        ) == 0
        {
            bDelayedExpansion = TRUE;
        } else if _tcsicmp(
            *arg.offset(i as isize),
            [
                'D' as u16, 'I' as u16, 'S' as u16, 'A' as u16, 'B' as u16, 'L' as u16,
                'E' as u16, 'D' as u16, 'E' as u16, 'L' as u16, 'A' as u16, 'Y' as u16,
                'E' as u16, 'D' as u16, 'E' as u16, 'X' as u16, 'P' as u16, 'A' as u16,
                'N' as u16, 'S' as u16, 'I' as u16, 'O' as u16, 'N' as u16, 0,
            ].as_ptr() as LPTSTR,
        ) == 0
        {
            bDelayedExpansion = FALSE;
        } else {
            error_invalid_parameter_format(*arg.offset(i as isize));
            break;
        }
        i += 1;
    }
    freep(arg);

    return nErrorLevel;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_endlocal(param: LPTSTR) -> INT {
    let mut Environ: LPTSTR;
    let mut Name: LPTSTR;
    let mut Value: LPTSTR;
    let mut Saved: PSETLOCAL;
    let mut drvEnvVar: [TCHAR; 4] = ['=' as TCHAR, '?' as TCHAR, ':' as TCHAR, 0 as TCHAR];
    let mut szCurrent: [TCHAR; MAX_PATH] = [0 as TCHAR; MAX_PATH];

    if _tcscmp(param, _T("/?")) == 0 {
        // FIXME
        ConOutPuts(_T("ENDLOCAL help not implemented yet!\n"));
        return 0;
    }

    /* Pop a SETLOCAL struct off of this batch context's stack */
    if bc.is_null() {
        return 0;
    }
    Saved = (*bc).setlocal as PSETLOCAL;
    if Saved.is_null() {
        return 0;
    }
    (*bc).setlocal = (*Saved).Prev;

    bEnableExtensions = (*Saved).EnableExtensions;
    bDelayedExpansion = (*Saved).DelayedExpansion;

    /* First, clear out the environment. Since making any changes to the
     * environment invalidates pointers obtained from GetEnvironmentStrings(),
     * we must make a copy of it and get the variable names from that. */
    Environ = DuplicateEnvironment();
    if !Environ.is_null() {
        Name = Environ;
        while *Name != 0 {
            'body: {
                Value = _tcschr(Name.add(1 as usize), _T('='));
                if Value.is_null() {
                    break 'body;
                }
                let fresh0 = Value;
                Value = Value.add(1);
                *fresh0 = '\0' as TCHAR;
                SetEnvironmentVariable(Name, core::ptr::null_mut());
                Name = Value;
            }
            Name = Name.add((_tcslen(Name) + 1) as usize);
        }
        cmd_free(Environ as *mut core::ffi::c_void);
    }

    /* Now, restore variables from the copy saved by cmd_setlocal() */
    Name = (*Saved).Environment;
    while *Name != 0 {
        'body: {
            Value = _tcschr(Name.add(1 as usize), _T('='));
            if Value.is_null() {
                break 'body;
            }
            let fresh1 = Value;
            Value = Value.add(1);
            *fresh1 = '\0' as TCHAR;
            SetEnvironmentVariable(Name, Value);
            Name = Value;
        }
        Name = Name.add((_tcslen(Name) + 1) as usize);
    }

    /* Restore the current drive and its current directory from the environment */
    drvEnvVar[1 as usize] =
        ('A' as i32 + (*Saved).CurDrive as i32 - 1) as TCHAR;
    if GetEnvironmentVariable(
        core::ptr::addr_of_mut!(drvEnvVar) as *mut TCHAR,
        core::ptr::addr_of_mut!(szCurrent) as *mut TCHAR,
        ARRAYSIZE(szCurrent),
    ) == 0
    {
        _stprintf(
            core::ptr::addr_of_mut!(szCurrent) as *mut TCHAR,
            _T("%C:\\"),
            ('A' as i32 + (*Saved).CurDrive as i32 - 1) as TCHAR,
        );
    }
    _tchdir(core::ptr::addr_of_mut!(szCurrent) as *mut TCHAR); // SetRootPath(NULL, szCurrent);

    cmd_free((*Saved).Environment as *mut core::ffi::c_void);
    cmd_free(Saved as *mut core::ffi::c_void);
    return 0;
}
