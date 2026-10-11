use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut Callbacks: winhelp_callbacks = unsafe {
    winhelp_callbacks {
        GetFSError: Some(WHD_GetFSError),
        HfsOpenSz: Some(WHD_Open),
        RcCloseHfs: Some(WHD_Close),
        HfOpenHfs: Some(WHD_OpenBag),
        RcCloseHf: Some(WHD_CloseBag),
        LcbReadHf: Some(WHD_ReadBag),
        LTellHf: Some(WHD_TellBag),
        LSeekHf: Some(WHD_SeekBag),
        FEofHf: Some(WHD_IsEofBag),
        LcbSizeHf: Some(WHD_SizeBag),
        FAccessHfs: Some(WHD_Access),
        RcLLInfoFromHf: Some(
            WHD_LLInfoFromBag,
        ),
        RcLLInfoFromHfs: Some(
            WHD_LLInfoFromFile,
        ),
        ErrorW: Some(WHD_Error),
        ErrorSz: Some(WHD_ErrorString),
        GetInfo: Some(WHD_GetInfo),
        API: Some(WHD_API),
    }
};

// #define WIN32_LEAN_AND_MEAN (no value; build-configuration flag, no Rust declaration)

// WINE_DEFAULT_DEBUG_CHANNEL(winhelp): debug channel registration; the crate's DPRINT macros replace it.
unsafe extern "system" fn WHD_GetFSError() -> WORD {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}

unsafe extern "system" fn WHD_Open(mut name: LPSTR, mut flags: BYTE) -> HANDLE {
    let mut mode: ::core::ffi::c_uint = 0 as ::core::ffi::c_uint;

    WINE_FIXME!(
        "(%s %x)\n",
        debugstr_a(name as *const ::core::ffi::c_char),
        flags as ::core::ffi::c_int,
    );
    match flags as ::core::ffi::c_int {
        0 => {
            mode = (GENERIC_READ | GENERIC_WRITE) as ::core::ffi::c_uint;
        }
        2 => {
            mode = GENERIC_READ as ::core::ffi::c_uint;
        }
        _ => {
            WINE_FIXME!("Undocumented flags %x\n", flags as ::core::ffi::c_int);
        }
    }
    return CreateFileA(
        name as LPCSTR,
        mode as DWORD,
        (FILE_SHARE_READ | FILE_SHARE_WRITE) as DWORD,
        ::core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        OPEN_EXISTING as DWORD,
        FILE_ATTRIBUTE_NORMAL as DWORD,
        ::core::ptr::null_mut(),
    );
}

unsafe extern "system" fn WHD_Close(fs: HANDLE) -> WORD {
    WINE_FIXME!(b"(%p)\n\0".as_ptr() as *const i8, fs);
    CloseHandle(fs);
    return 0;
}
unsafe extern "system" fn WHD_OpenBag(fs: HANDLE, name: LPSTR, flags: BYTE) -> HANDLE {
    WINE_FIXME!(b"(%p %s %x)\n\0".as_ptr() as *const i8, fs, debugstr_a(name), flags as ::core::ffi::c_int);
    NULL
}
unsafe extern "system" fn WHD_CloseBag(bag: HANDLE) -> HANDLE {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    NULL
}
unsafe extern "system" fn WHD_ReadBag(bag: HANDLE, ptr: *mut BYTE, len: LONG) -> LONG {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_TellBag(bag: HANDLE) -> LONG {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_SeekBag(bag: HANDLE, offset: LONG, whence: WORD) -> LONG {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_IsEofBag(bag: HANDLE) -> BOOL {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    (FALSE as i32)
}
unsafe extern "system" fn WHD_SizeBag(bag: HANDLE) -> LONG {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_Access(fs: HANDLE, name: LPSTR, flags: BYTE) -> BOOL {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    (FALSE as i32)
}
unsafe extern "system" fn WHD_LLInfoFromBag(
    bag: HANDLE,
    opt: WORD,
    p1: LPWORD,
    p2: LPLONG,
    p3: LPLONG,
) -> WORD {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_LLInfoFromFile(
    fs: HANDLE,
    name: LPSTR,
    opt: WORD,
    p1: LPWORD,
    p2: LPLONG,
    p3: LPLONG,
) -> WORD {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
unsafe extern "system" fn WHD_Error(err: i32) {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}
unsafe extern "system" fn WHD_ErrorString(err: LPSTR) {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
}

unsafe extern "system" fn WHD_GetInfo(what: WORD, hnd: HWND) -> ULONG_PTR {
    let mut ret: ULONG_PTR = 0;

    WINE_TRACE!("(%x %p)\n", what as core::ffi::c_uint, hnd);
    match what {
        0 => {}
        1 => {
            /* instance */
            ret = Globals.hInstance as ULONG_PTR;
        }
        3 => {
            /* current window */
            ret = (*Globals.active_win).hMainWnd as ULONG_PTR;
        }
        2
        | 4
        | 5
        | 6
        | 7
        | 8 => {
            /* main window, handle to opened file, foreground color,
               background color, topic number, current opened file name */
            WINE_FIXME!("NIY %u\n", what as core::ffi::c_uint);
        }
        _ => {
            WINE_FIXME!("Undocumented %u\n", what as core::ffi::c_uint);
        }
    }
    return ret;
}
unsafe extern "system" fn WHD_API(x: LPSTR, xx: WORD, xxx: DWORD) -> LONG {
    WINE_FIXME!(b"()\n\0".as_ptr() as *const i8);
    0
}
