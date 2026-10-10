use crate::cmd::cmd::DoDelayedExpansion;
use crate::cmd::misc::StripQuotes;
use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn RtlAssert(FailedAssertion: PVOID, FileName: PVOID, LineNumber: ULONG, Message: PCHAR);
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut StdStreams: [PCON_STREAM; 3] = unsafe {
    [
        (&raw const csStdIn as *mut CON_STREAM).cast(),
        (&raw const csStdOut as *mut CON_STREAM).cast(),
        (&raw const csStdErr as *mut CON_STREAM).cast(),
    ]
};

static mut ExtraHandles: [HANDLE; 7] =
    [::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void; 7];

// FEATURE_REDIRECTION is assumed enabled; the block is emitted unconditionally.

// CMD allows redirection of handles numbered 3-9 even though
// these don't correspond to any STD_ constant.

/* Some parameters used for read, write, and append, respectively */
#[repr(C)]
pub struct REDIR_PARAMS {
    pub dwDesiredAccess: DWORD,
    pub dwShareMode: DWORD,
    pub dwCreationDisposition: DWORD,
}

pub static RedirParams: [REDIR_PARAMS; 3] = [
    REDIR_PARAMS { dwDesiredAccess: GENERIC_READ, dwShareMode: FILE_SHARE_READ | FILE_SHARE_WRITE, dwCreationDisposition: OPEN_EXISTING }, // REDIR_READ
    REDIR_PARAMS { dwDesiredAccess: GENERIC_WRITE, dwShareMode: FILE_SHARE_READ, dwCreationDisposition: CREATE_ALWAYS }, // REDIR_WRITE
    REDIR_PARAMS { dwDesiredAccess: GENERIC_WRITE, dwShareMode: FILE_SHARE_READ, dwCreationDisposition: OPEN_ALWAYS }, // REDIR_APPEND
];
#[no_mangle]
pub unsafe extern "C" fn GetHandle(Number: UINT) -> HANDLE {
    if Number < 3 {
        return ConStreamGetOSHandle((StdStreams[Number as usize]).cast());
        // return GetStdHandle(STD_INPUT_HANDLE - Number);
    } else if Number < (ARRAYSIZE!(ExtraHandles) as UINT) + 3 {
        return ExtraHandles[(Number - 3) as usize];
    } else {
        return INVALID_HANDLE_VALUE;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetHandle(Number: UINT, Handle: HANDLE) {
    if Number < 3 {
        ConStreamSetOSHandle((StdStreams[Number as usize]).cast(), Handle);
        /* Synchronize the associated Win32 handle */
        SetStdHandle(STD_INPUT_HANDLE - Number, Handle);
    } else if Number < ARRAYSIZE!(ExtraHandles) + 3 {
        ExtraHandles[(Number - 3) as usize] = Handle;
    }
}

#[no_mangle]
pub unsafe extern "C" fn PerformRedirection(RedirList: *mut REDIRECTION) -> BOOL {
    let mut Redir: *mut REDIRECTION = core::ptr::null_mut();
    let mut Filename: LPTSTR = core::ptr::null_mut();
    let mut hNew: HANDLE = core::ptr::null_mut();
    let mut DupNumber: UINT = 0;

    static mut SecAttr: SECURITY_ATTRIBUTES = SECURITY_ATTRIBUTES {
        nLength: core::mem::size_of::<SECURITY_ATTRIBUTES>() as DWORD,
        lpSecurityDescriptor: core::ptr::null_mut(),
        bInheritHandle: (TRUE) as i32,
    };

    /* Some parameters used for read, write, and append, respectively */
    static RedirParams: [REDIR_PARAMS; 3] = [
        REDIR_PARAMS {
            dwDesiredAccess: GENERIC_READ as DWORD,
            dwShareMode: (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
            dwCreationDisposition: OPEN_EXISTING as DWORD,
        },
        REDIR_PARAMS {
            dwDesiredAccess: GENERIC_WRITE as DWORD,
            dwShareMode: FILE_SHARE_READ as DWORD,
            dwCreationDisposition: CREATE_ALWAYS as DWORD,
        },
        REDIR_PARAMS {
            dwDesiredAccess: GENERIC_WRITE as DWORD,
            dwShareMode: FILE_SHARE_READ as DWORD,
            dwCreationDisposition: OPEN_ALWAYS as DWORD,
        },
    ];

    Redir = RedirList;
    while !Redir.is_null() {
        let mut redir_error_flag: bool = false;
        'iter: {
            Filename = DoDelayedExpansion(core::ptr::addr_of!((*Redir).Filename) as *const u16) as LPTSTR;
            if Filename.is_null() {
                redir_error_flag = true;
                break 'iter;
            }
            StripQuotes(Filename);

            if *Filename == '&' as u16 {
                DupNumber = ((*Filename.offset(1 as isize)) as i32
                    - '0' as i32) as UINT;
                if DupNumber >= 10 as UINT
                    || DuplicateHandle(
                        GetCurrentProcess(),
                        GetHandle(DupNumber),
                        GetCurrentProcess(),
                        core::ptr::addr_of_mut!(hNew),
                        0 as DWORD,
                        (TRUE) as BOOL,
                        DUPLICATE_SAME_ACCESS as DWORD,
                    ) == 0
                {
                    hNew = INVALID_HANDLE_VALUE;
                }
            } else {
                hNew = CreateFile(
                    Filename,
                    RedirParams[(*Redir).Mode as usize].dwDesiredAccess,
                    RedirParams[(*Redir).Mode as usize].dwShareMode,
                    core::ptr::addr_of_mut!(SecAttr),
                    RedirParams[(*Redir).Mode as usize].dwCreationDisposition,
                    0 as DWORD,
                    core::ptr::null_mut(),
                );
            }

            if hNew == INVALID_HANDLE_VALUE {
                /* TODO: Print a more detailed message */
                ConErrResPrintf!(
                    if (*Redir).Mode == REDIR_READ as UINT {
                        STRING_CMD_ERROR1 as UINT
                    } else {
                        STRING_CMD_ERROR3 as UINT
                    },
                    Filename,
                );
                cmd_free!(Filename);
                redir_error_flag = true;
                break 'iter;
            }

            if (*Redir).Mode == REDIR_APPEND as UINT {
                SetFilePointer(
                    hNew,
                    0 as LONG,
                    core::ptr::null_mut(),
                    FILE_END as DWORD,
                );
            }
            (*Redir).OldHandle = GetHandle((*Redir).Number as UINT);
            SetHandle((*Redir).Number as UINT, hNew);

            cmd_free!(Filename);
        }
        if redir_error_flag {
            /* Undo all the redirections before this one */
            UndoRedirection(RedirList, Redir);
            return (FALSE) as BOOL;
        }
        Redir = (*Redir).Next;
    }
    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn UndoRedirection(mut Redir: *mut REDIRECTION, mut End: *mut REDIRECTION) {
    while Redir != End {
        CloseHandle(GetHandle(((*Redir).Number) as UINT));
        SetHandle(((*Redir).Number) as UINT, (*Redir).OldHandle);
        (*Redir).OldHandle = INVALID_HANDLE_VALUE;
        Redir = (*Redir).Next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn FreeRedirection(mut Redir: *mut REDIRECTION) {
    let mut Next: *mut REDIRECTION = core::ptr::null_mut::<REDIRECTION>();
    while !Redir.is_null() {
        Next = (*Redir).Next as *mut REDIRECTION;
        if !((*Redir).OldHandle == INVALID_HANDLE_VALUE) {
            RtlAssert(
                b"Redir->OldHandle == INVALID_HANDLE_VALUE\0" as *const u8
                    as *const core::ffi::c_char as PVOID,
                b"base/shell/cmd/redir.c\0"
                    as *const u8 as *const core::ffi::c_char as PVOID,
                159 as ULONG,
                core::ptr::null_mut::<CHAR>(),
            );
        } else {
        }
        cmd_free!(Redir as *mut core::ffi::c_void);
        Redir = Next;
    }
}
