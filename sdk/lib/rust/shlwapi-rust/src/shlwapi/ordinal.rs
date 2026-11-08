// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_20C44FBAF751: [u16; 8] = [0x002Eu16, 0x0062, 0x006F, 0x0074, 0x0074, 0x006F, 0x006D, 0x0000];
pub static RUSTCONV_WSTR_33D489E223A7: [u16; 7] = [0x002Eu16, 0x0072, 0x0069, 0x0067, 0x0068, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_433FA3D0669D: [u16; 3] = [0x002Eu16, 0x0079, 0x0000];
pub static RUSTCONV_WSTR_52D451AA9857: [u16; 9] = [0x0041u16, 0x0064, 0x0076, 0x0061, 0x006E, 0x0063, 0x0065, 0x0064, 0x0000];
pub static RUSTCONV_WSTR_55366762880C: [u16; 5] = [0x002Eu16, 0x0074, 0x006F, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_62D9010A526D: [u16; 3] = [0x002Eu16, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_7E73C65754ED: [u16; 12] = [
    0x0050u16, 0x0072, 0x006F, 0x0070, 0x0044, 0x006C, 0x0067, 0x0046, 0x006F, 0x006E,
    0x0074, 0x0000,
];
pub static RUSTCONV_WSTR_D4E75AE8C989: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0050, 0x006F, 0x006C, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073,
    0x0000,
];
pub static RUSTCONV_WSTR_E744B4C578CD: [u16; 6] = [0x002Eu16, 0x006C, 0x0065, 0x0066, 0x0074, 0x0000];
pub static RUSTCONV_WSTR_F6C54912C188: [u16; 14] = [
    0x0053u16, 0x0065, 0x0072, 0x0076, 0x0065, 0x0072, 0x0041, 0x0064, 0x006D, 0x0069,
    0x006E, 0x0055, 0x0049, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn SendMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    fn PostMessageA(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut objcompat_table: [objcompat_entry; 13] = [
    objcompat_entry {
        name: [
            'C' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_COCREATESHELLFOLDERONLY as DWORD,
    },
    objcompat_entry {
        name: [
            'C' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            '_' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_CTXMENU_LIMITEDQI as DWORD,
    },
    objcompat_entry {
        name: [
            'C' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            '_' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_CTXMENU_LIMITEDQI as DWORD,
    },
    objcompat_entry {
        name: [
            'C' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'U' as i32 as WCHAR,
            '_' as i32 as WCHAR,
            'X' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'Q' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_CTXMENU_XPQCMFLAGS as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NEEDSFILESYSANCESTOR as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NEEDSSTORAGEANCESTOR as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'P' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'R' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NOIPROPERTYSTORE as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'G' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NOLEGACYWEBVIEW as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'Y' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'M' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NOTAFILESYSTEM as DWORD,
    },
    objcompat_entry {
        name: [
            'N' as i32 as WCHAR,
            'O' as i32 as WCHAR,
            '_' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'V' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'W' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_NO_WEBVIEW as DWORD,
    },
    objcompat_entry {
        name: [
            'O' as i32 as WCHAR,
            'T' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'S' as i32 as WCHAR,
            'F' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'C' as i32 as WCHAR,
            'H' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_OTNEEDSSFCACHE as DWORD,
    },
    objcompat_entry {
        name: [
            'P' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_PINDLL as DWORD,
    },
    objcompat_entry {
        name: [
            'U' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'I' as i32 as WCHAR,
            'N' as i32 as WCHAR,
            'D' as i32 as WCHAR,
            'A' as i32 as WCHAR,
            'B' as i32 as WCHAR,
            'L' as i32 as WCHAR,
            'E' as i32 as WCHAR,
            0 as ::core::ffi::c_int as WCHAR,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ],
        value: OBJCOMPAT_UNBINDABLE as DWORD,
    },
];


#[repr(C)]
pub struct enumWndData {
    pub uiMsgId: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub pfnPost: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
}

#[repr(C)]
pub struct FDSA_info {
    pub num_items: DWORD,
    pub mem: *mut core::ffi::c_void,
    pub blocks_alloced: DWORD,
    pub inc: BYTE,
    pub block_size: BYTE,
    pub flags: BYTE,
}

pub const FDSA_FLAG_INTERNAL_ALLOC: u8 = 0x01;

#[repr(C)]
pub struct tagLOGPALETTEMAX {
    pub palVersion: WORD,
    pub palNumEntries: WORD,
    pub palPalEntry: [PALETTEENTRY; 256],
}
pub type LOGPALETTEMAX = tagLOGPALETTEMAX;
pub type PLOGPALETTEMAX = *mut tagLOGPALETTEMAX;

pub type DllGetVersion_func = Option<unsafe extern "system" fn(*mut DLLVERSIONINFO) -> HRESULT>;

#[repr(C)]
pub struct SHELL_USER_SID {
    pub sidAuthority: SID_IDENTIFIER_AUTHORITY,
    pub dwUserGroupID: DWORD,
    pub dwUserID: DWORD,
}
pub type PSHELL_USER_SID = *mut SHELL_USER_SID;

#[repr(C)]
pub struct SHELL_USER_PERMISSION {
    pub susID: SHELL_USER_SID,
    pub dwAccessType: DWORD,
    pub fInherit: BOOL,
    pub dwAccessMask: DWORD,
    pub dwInheritMask: DWORD,
    pub dwInheritAccessMask: DWORD,
}
pub type PSHELL_USER_PERMISSION = *mut SHELL_USER_PERMISSION;

pub const OBJCOMPAT_OTNEEDSSFCACHE: DWORD = 0x00000001;
pub const OBJCOMPAT_NO_WEBVIEW: DWORD = 0x00000002;
pub const OBJCOMPAT_UNBINDABLE: DWORD = 0x00000004;
pub const OBJCOMPAT_PINDLL: DWORD = 0x00000008;
pub const OBJCOMPAT_NEEDSFILESYSANCESTOR: DWORD = 0x00000010;
pub const OBJCOMPAT_NOTAFILESYSTEM: DWORD = 0x00000020;
pub const OBJCOMPAT_CTXMENU_NOVERBS: DWORD = 0x00000040;
pub const OBJCOMPAT_CTXMENU_LIMITEDQI: DWORD = 0x00000080;
pub const OBJCOMPAT_COCREATESHELLFOLDERONLY: DWORD = 0x00000100;
pub const OBJCOMPAT_NEEDSSTORAGEANCESTOR: DWORD = 0x00000200;
pub const OBJCOMPAT_NOLEGACYWEBVIEW: DWORD = 0x00000400;
pub const OBJCOMPAT_CTXMENU_XPQCMFLAGS: DWORD = 0x00001000;
pub const OBJCOMPAT_NOIPROPERTYSTORE: DWORD = 0x00002000;

#[repr(C)]
pub struct objcompat_entry {
    pub name: [WCHAR; 30],
    pub value: DWORD,
}

#[no_mangle]
pub unsafe extern "system" fn SHMapHandle(
    hShared: HANDLE,
    dwSrcProcId: DWORD,
    dwDstProcId: DWORD,
    dwAccess: DWORD,
    dwOptions: DWORD,
) -> HANDLE {
    let hDst: HANDLE;
    let hSrc: HANDLE;
    let dwMyProcId: DWORD = GetCurrentProcessId();
    let mut hRet: HANDLE = core::ptr::null_mut();

    TRACE!(
        "(%p,%d,%d,%08x,%08x)\n",
        hShared,
        dwDstProcId,
        dwSrcProcId,
        dwAccess,
        dwOptions
    );

    if hShared.is_null() {
        TRACE!("Returning handle NULL\n");
        return core::ptr::null_mut();
    }

    /* Get dest process handle */
    if dwDstProcId == dwMyProcId {
        hDst = GetCurrentProcess();
    } else {
        hDst = OpenProcess(
            PROCESS_DUP_HANDLE,
            0,
            dwDstProcId,
        );
    }

    if !hDst.is_null() {
        /* Get src process handle */
        if dwSrcProcId == dwMyProcId {
            hSrc = GetCurrentProcess();
        } else {
            hSrc = OpenProcess(
                PROCESS_DUP_HANDLE,
                0,
                dwSrcProcId,
            );
        }

        if !hSrc.is_null() {
            /* Make handle available to dest process */
            if DuplicateHandle(
                hSrc,
                hShared,
                hDst,
                &raw mut hRet,
                dwAccess,
                0,
                dwOptions | DUPLICATE_SAME_ACCESS,
            ) == 0
            {
                hRet = core::ptr::null_mut();
            }

            if dwSrcProcId != dwMyProcId {
                CloseHandle(hSrc);
            }
        }

        if dwDstProcId != dwMyProcId {
            CloseHandle(hDst);
        }
    }

    TRACE!("Returning handle %p\n", hRet);
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHAllocShared(lpvData: LPCVOID, dwSize: DWORD, dwProcId: DWORD) -> HANDLE {
    let mut hMap: HANDLE = core::ptr::null_mut();
    let mut pMapped: LPVOID = core::ptr::null_mut();
    let mut hRet: HANDLE = core::ptr::null_mut();

    TRACE!("(%p,%d,%d)\n", lpvData, dwSize, dwProcId);

    /* Create file mapping of the correct length */
    hMap = CreateFileMappingA(
        INVALID_HANDLE_VALUE,
        core::ptr::null_mut(),
        FILE_MAP_READ,
        0,
        (dwSize as usize).wrapping_add(core::mem::size_of::<DWORD>()) as DWORD,
        core::ptr::null_mut(),
    );
    if hMap.is_null() {
        return hRet;
    }

    /* Get a view in our process address space */
    pMapped = MapViewOfFile(
        hMap,
        FILE_MAP_READ | FILE_MAP_WRITE,
        0,
        0,
        0,
    );

    if !pMapped.is_null() {
        /* Write size of data, followed by the data, to the view */
        *(pMapped as *mut DWORD) = dwSize;
        if !lpvData.is_null() {
            memcpy(
                (pMapped as *mut core::ffi::c_char).add(core::mem::size_of::<DWORD>()) as *mut core::ffi::c_void,
                lpvData as *const core::ffi::c_void,
                dwSize as usize,
            );
        }

        /* Release view. All further views mapped will be opaque */
        UnmapViewOfFile(pMapped);
        hRet = SHMapHandle(
            hMap,
            GetCurrentProcessId(),
            dwProcId,
            FILE_MAP_ALL_ACCESS,
            DUPLICATE_SAME_ACCESS,
        );
    }

    CloseHandle(hMap);
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHLockSharedEx(hShared: HANDLE, dwProcId: DWORD, bWriteAccess: BOOL) -> LPVOID {
    let mut hDup: HANDLE = core::ptr::null_mut();
    let mut pMapped: LPVOID = core::ptr::null_mut();
    let mut dwAccess: DWORD = 0;

    TRACE!("(%p %d %d)\n", hShared, dwProcId, bWriteAccess);

    /* Get handle to shared memory for current process */
    hDup = SHMapHandle(hShared, dwProcId, GetCurrentProcessId(), FILE_MAP_ALL_ACCESS as DWORD, 0 as DWORD);
    if hDup.is_null() {
        return core::ptr::null_mut();
    }

    /* Get View */
    dwAccess = ((FILE_MAP_READ as DWORD) | (if bWriteAccess != 0 { FILE_MAP_WRITE as DWORD } else { 0 as DWORD })) as DWORD;
    pMapped = MapViewOfFile(hDup, dwAccess, 0 as DWORD, 0 as DWORD, 0 as SIZE_T) as LPVOID;
    CloseHandle(hDup);

    if !pMapped.is_null() {
        return (pMapped as *mut u8).offset(core::mem::size_of::<DWORD>() as isize) as LPVOID; /* Hide size */
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn SHLockShared(hShared: HANDLE, dwProcId: DWORD) -> PVOID {
    return SHLockSharedEx(hShared, dwProcId, (TRUE) as BOOL) as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn SHUnlockShared(lpView: LPVOID) -> BOOL {
    return UnmapViewOfFile((lpView as *mut u8).offset(-(core::mem::size_of::<DWORD>() as isize)) as LPCVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SHFreeShared(mut hShared: HANDLE, mut dwProcId: DWORD) -> BOOL {
    let mut hClose: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    WINE_TRACE!("(%p %d)\n", hShared, dwProcId);
    if hShared.is_null() {
        return (TRUE) as BOOL;
    }
    hClose = SHMapHandle(
        hShared,
        dwProcId,
        GetCurrentProcessId(),
        FILE_MAP_ALL_ACCESS as DWORD,
        DUPLICATE_CLOSE_SOURCE as DWORD,
    );
    return CloseHandle(hClose);
}

#[no_mangle]
pub unsafe extern "system" fn RegisterDefaultAcceptHeaders(lpBC: LPBC, lpUnknown: *mut IUnknown) -> HRESULT {
    static szProperty: [WCHAR; 39] = [
    0x007Bu16, 0x0044, 0x0030, 0x0046, 0x0043, 0x0041, 0x0034, 0x0032, 0x0030, 0x002D,
    0x0044, 0x0033, 0x0046, 0x0035, 0x002D, 0x0031, 0x0031, 0x0043, 0x0046, 0x002D,
    0x0042, 0x0032, 0x0031, 0x0031, 0x002D, 0x0030, 0x0030, 0x0041, 0x0041, 0x0030,
    0x0030, 0x0034, 0x0041, 0x0045, 0x0038, 0x0033, 0x0037, 0x007D, 0x0000,
];
    let mut property: BSTR = core::ptr::null_mut();
    let mut pIEnumFormatEtc: *mut IEnumFORMATETC = core::ptr::null_mut();
    let mut var: VARIANTARG = core::mem::zeroed();
    let mut hr: HRESULT = 0;
    let mut pBrowser: *mut IWebBrowserApp = core::ptr::null_mut();

    'exit: {
        hr = IUnknown_QueryService(
            lpUnknown,
            &raw const IID_IWebBrowserApp,
            &raw const IID_IWebBrowserApp,
            &raw mut pBrowser as *mut LPVOID,
        );
        if hr < 0 {
            return hr;
        }

        var.n1.n2.vt = VT_EMPTY as VARTYPE;

        /* The property we get is the browsers clipboard enumerator */
        property = SysAllocString(&raw const szProperty as *const OLECHAR);
        hr = IWebBrowserApp_GetProperty(pBrowser, property, &raw mut var);
        SysFreeString(property);
        if hr < 0 {
            break 'exit;
        }

        if (var.n1.n2.vt as i32) == VT_EMPTY {
            /* Iterate through accepted documents and RegisterClipBoardFormatA() them */
            let mut szKeyBuff: [core::ffi::c_char; 128] = [0; 128];
            let mut szValueBuff: [core::ffi::c_char; 128] = [0; 128];
            let mut dwKeySize: DWORD = 0;
            let mut dwValueSize: DWORD = 0;
            let mut dwRet: DWORD = 0;
            let mut dwCount: DWORD = 0;
            let mut dwNumValues: DWORD = 0;
            let mut dwType: DWORD = 0;
            let mut formatList: *mut FORMATETC = core::ptr::null_mut();
            let mut format: *mut FORMATETC = core::ptr::null_mut();
            let mut hDocs: HKEY = core::ptr::null_mut();

            if RegOpenKeyA(
                HKEY_LOCAL_MACHINE,
                b"Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings\\Accepted Documents\0"
                    .as_ptr() as LPCSTR,
                &raw mut hDocs,
            ) == 0
            {
                hr = E_FAIL;
                break 'exit;
            }

            /* Get count of values in key */
            while dwRet == 0 {
                dwKeySize = core::mem::size_of::<[core::ffi::c_char; 128]>() as DWORD;
                dwRet = RegEnumValueA(
                    hDocs,
                    dwCount,
                    &raw mut szKeyBuff as LPSTR,
                    &raw mut dwKeySize,
                    core::ptr::null_mut(),
                    &raw mut dwType,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                ) as DWORD;
                dwCount = dwCount.wrapping_add(1);
            }

            dwNumValues = dwCount;

            /* Note: dwCount = number of items + 1; The extra item is the end node */
            formatList = HeapAlloc(
                GetProcessHeap(),
                0,
                (dwCount as SIZE_T).wrapping_mul(core::mem::size_of::<FORMATETC>() as SIZE_T),
            ) as *mut FORMATETC;
            format = formatList;
            if formatList.is_null() {
                RegCloseKey(hDocs);
                hr = E_OUTOFMEMORY;
                break 'exit;
            }

            if dwNumValues > 1 {
                dwRet = 0;
                dwCount = 0;

                dwNumValues = dwNumValues.wrapping_sub(1);

                /* Register clipboard formats for the values and populate format list */
                while dwRet == 0 && dwCount < dwNumValues {
                    dwKeySize = core::mem::size_of::<[core::ffi::c_char; 128]>() as DWORD;
                    dwValueSize = core::mem::size_of::<[core::ffi::c_char; 128]>() as DWORD;
                    dwRet = RegEnumValueA(
                        hDocs,
                        dwCount,
                        &raw mut szKeyBuff as LPSTR,
                        &raw mut dwKeySize,
                        core::ptr::null_mut(),
                        &raw mut dwType,
                        &raw mut szValueBuff as *mut core::ffi::c_char as LPBYTE,
                        &raw mut dwValueSize,
                    ) as DWORD;
                    if dwRet == 0 {
                        HeapFree(GetProcessHeap(), 0, formatList as PVOID);
                        RegCloseKey(hDocs);
                        hr = E_FAIL;
                        break 'exit;
                    }

                    (*format).cfFormat = RegisterClipboardFormatA(
                        &raw mut szValueBuff as *mut core::ffi::c_char as LPCSTR,
                    ) as CLIPFORMAT;
                    (*format).ptd = core::ptr::null_mut();
                    (*format).dwAspect = 1;
                    (*format).lindex = 4;
                    (*format).tymed = -(1 as core::ffi::c_int) as DWORD;

                    format = format.offset(1);
                    dwCount = dwCount.wrapping_add(1);
                }
            }

            RegCloseKey(hDocs);

            /* Terminate the (maybe empty) list, last entry has a cfFormat of 0 */
            (*format).cfFormat = 0;
            (*format).ptd = core::ptr::null_mut();
            (*format).dwAspect = 1;
            (*format).lindex = 4;
            (*format).tymed = -(1 as core::ffi::c_int) as DWORD;

            /* Create a clipboard enumerator */
            hr = CreateFormatEnumerator(dwNumValues as UINT, formatList, &raw mut pIEnumFormatEtc);
            HeapFree(GetProcessHeap(), 0, formatList as PVOID);
            if hr < 0 {
                break 'exit;
            }

            /* Set our enumerator as the browsers property */
            var.n1.n2.vt = VT_UNKNOWN as VARTYPE;
            var.n1.n2.n3.punkVal = pIEnumFormatEtc as *mut IUnknown;

            property = SysAllocString(&raw const szProperty as *const OLECHAR);
            hr = IWebBrowserApp_PutProperty(pBrowser, property, var);
            SysFreeString(property);
            if hr < 0 {
                IEnumFORMATETC_Release(pIEnumFormatEtc);
                break 'exit;
            }
        }

        if ((var.n1.n2.vt as i32)) as u32 == VT_UNKNOWN {
            /* Our variant is holding the clipboard enumerator */
            let mut pIUnknown: *mut IUnknown = var.n1.n2.n3.punkVal;
            let mut pClone: *mut IEnumFORMATETC = core::ptr::null_mut();

            /* Get an IEnumFormatEtc interface from the variants value */
            pIEnumFormatEtc = core::ptr::null_mut();
            hr = IUnknown_QueryInterface(
                pIUnknown,
                &raw const IID_IEnumFORMATETC,
                &raw mut pIEnumFormatEtc as *mut *mut core::ffi::c_void,
            );
            if hr == S_OK && !pIEnumFormatEtc.is_null() {
                /* Clone and register the enumerator */
                hr = IEnumFORMATETC_Clone(pIEnumFormatEtc, &raw mut pClone);
                if hr == S_OK && !pClone.is_null() {
                    RegisterFormatEnumerator(lpBC, pClone, 0);

                    IEnumFORMATETC_Release(pClone);
                }

                IUnknown_Release(pIUnknown);
            }
            IUnknown_Release(var.n1.n2.n3.punkVal);
        }
    }

    IWebBrowserApp_Release(pBrowser);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn GetAcceptLanguagesW(langbuf: LPWSTR, buflen: LPDWORD) -> HRESULT {
    static szkeyW: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0049,
    0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0020, 0x0045, 0x0078,
    0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0049, 0x006E, 0x0074,
    0x0065, 0x0072, 0x006E, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0061, 0x006C,
    0x0000,
];
    static valueW: [u16; 15] = [
    0x0041u16, 0x0063, 0x0063, 0x0065, 0x0070, 0x0074, 0x004C, 0x0061, 0x006E, 0x0067,
    0x0075, 0x0061, 0x0067, 0x0065, 0x0000,
];
    let mut mystrlen: DWORD;
    let mut mytype: DWORD = core::mem::zeroed();
    let mut len: DWORD;
    let mut mykey: HKEY = core::mem::zeroed();
    let mut mylcid: LCID;
    let mut mystr: *mut WCHAR;
    let mut lres: LONG;

    if langbuf.is_null() || buflen.is_null() || *buflen == 0 {
        return E_FAIL;
    }

    mystrlen = if *buflen > 20 {
        *buflen
    } else {
        20
    };
    len = (mystrlen as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    mystr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T) as *mut WCHAR;
    *mystr.add(0 as usize) = 0;
    RegOpenKeyW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of!(szkeyW) as LPCWSTR,
        core::ptr::addr_of_mut!(mykey),
    );
    lres = RegQueryValueExW(
        mykey,
        core::ptr::addr_of!(valueW) as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(mytype),
        mystr as PBYTE,
        core::ptr::addr_of_mut!(len),
    ) as LONG;
    RegCloseKey(mykey);
    len = lstrlenW(mystr as LPCWSTR) as DWORD;

    if lres == 0 && *buflen > len {
        lstrcpyW(langbuf, mystr as LPCWSTR);
        *buflen = len;
        HeapFree(GetProcessHeap(), 0, mystr as LPVOID);
        return S_OK;
    }

    /* Did not find a value in the registry or the user buffer is too small */
    mylcid = GetUserDefaultLCID();
    LcidToRfc1766W(mylcid, mystr as LPWSTR, mystrlen as i32);
    len = lstrlenW(mystr as LPCWSTR) as DWORD;

    memcpy(
        langbuf as *mut core::ffi::c_void,
        mystr as *const core::ffi::c_void,
        (min!(*buflen, len.wrapping_add(1)) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()),
    );
    HeapFree(GetProcessHeap(), 0, mystr as LPVOID);

    if *buflen > len {
        *buflen = len;
        return S_OK;
    }

    *buflen = 0;
    return E_NOT_SUFFICIENT_BUFFER;
}

#[no_mangle]
pub unsafe extern "system" fn GetAcceptLanguagesA(langbuf: LPSTR, buflen: LPDWORD) -> HRESULT {
    let mut langbufW: *mut WCHAR = core::ptr::null_mut();
    let mut buflenW: DWORD = 0;
    let mut convlen: DWORD = 0;
    let mut retval: HRESULT = 0;
    WINE_TRACE!("(%p, %p) *%p: %d\n", langbuf, buflen, buflen, if !buflen.is_null() { *buflen } else { -(1 as i32) as DWORD });
    if langbuf.is_null() || buflen.is_null() || *buflen == 0 {
        return E_FAIL;
    }
    buflenW = *buflen;
    langbufW = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(buflenW as SIZE_T),
    ) as *mut WCHAR;
    retval = GetAcceptLanguagesW(langbufW as LPWSTR, core::ptr::addr_of_mut!(buflenW));

    if retval == S_OK {
        convlen = WideCharToMultiByte(
            CP_ACP,
            0,
            langbufW as LPCWSTR,
            -1,
            langbuf,
            *buflen as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as DWORD;
        convlen = convlen.wrapping_sub(1);
    } else {
        convlen = WideCharToMultiByte(
            CP_ACP,
            0,
            langbufW as LPCWSTR,
            *buflen as i32,
            langbuf,
            *buflen as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        ) as DWORD;
        if convlen < *buflen {
            *langbuf.offset(convlen as isize) = 0;
            convlen = convlen.wrapping_sub(1);
        } else {
            convlen = *buflen;
        }
    }
    *buflen = if buflenW != 0 { convlen } else { 0 };

    HeapFree(
        GetProcessHeap(),
        0,
        langbufW as PVOID,
    );
    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn SHStringFromGUIDA(guid: *const GUID, lpszDest: LPSTR, cchMax: INT) -> INT {
    let mut xguid: [::core::ffi::c_char; 40] = [0; 40];
    let mut iLen: INT = 0;

    sprintf(
        xguid.as_mut_ptr(),
        b"{%08X-%04X-%04X-%02X%02X-%02X%02X%02X%02X%02X%02X}\0".as_ptr() as *const ::core::ffi::c_char,
        (*guid).Data1 as UINT,
        (*guid).Data2 as UINT,
        (*guid).Data3 as UINT,
        (*guid).Data4[0 as usize] as UINT,
        (*guid).Data4[1 as usize] as UINT,
        (*guid).Data4[2 as usize] as UINT,
        (*guid).Data4[3 as usize] as UINT,
        (*guid).Data4[4 as usize] as UINT,
        (*guid).Data4[5 as usize] as UINT,
        (*guid).Data4[6 as usize] as UINT,
        (*guid).Data4[7 as usize] as UINT,
    );

    iLen = (strlen(xguid.as_ptr()) as INT).wrapping_add(1);

    if iLen > cchMax {
        return 0;
    }
    memcpy(
        lpszDest as *mut ::core::ffi::c_void,
        xguid.as_ptr() as *const ::core::ffi::c_void,
        iLen as usize,
    );
    return iLen;
}

#[no_mangle]
pub unsafe extern "system" fn SHStringFromGUIDW(guid: REFGUID, lpszDest: LPWSTR, cchMax: INT) -> INT {
    let mut xguid: [WCHAR; 40] = [0; 40];
    let mut iLen: INT;
    static wszFormat: [WCHAR; 52] = [
    0x007Bu16, 0x0025, 0x0030, 0x0038, 0x006C, 0x0058, 0x002D, 0x0025, 0x0030, 0x0034,
    0x0058, 0x002D, 0x0025, 0x0030, 0x0034, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032,
    0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x002D, 0x0025, 0x0030, 0x0032, 0x0058,
    0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030,
    0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058, 0x0025, 0x0030, 0x0032, 0x0058,
    0x007D, 0x0000,
];

    sprintfW(
        core::ptr::addr_of_mut!(xguid) as *mut WCHAR,
        core::ptr::addr_of!(wszFormat) as *const WCHAR,
        (*guid).Data1,
        (*guid).Data2 as INT,
        (*guid).Data3 as INT,
        (*guid).Data4[0] as INT,
        (*guid).Data4[1] as INT,
        (*guid).Data4[2] as INT,
        (*guid).Data4[3] as INT,
        (*guid).Data4[4] as INT,
        (*guid).Data4[5] as INT,
        (*guid).Data4[6] as INT,
        (*guid).Data4[7] as INT,
    );

    iLen = (strlenW(core::ptr::addr_of!(xguid) as *const WCHAR) as INT) + 1;

    if iLen > cchMax {
        return 0;
    }
    memcpy(
        lpszDest as *mut core::ffi::c_void,
        core::ptr::addr_of!(xguid) as *const WCHAR as *const core::ffi::c_void,
        (iLen as usize) * core::mem::size_of::<WCHAR>(),
    );
    return iLen;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharBlankW(mut wc: WCHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return (GetStringTypeW(
        CT_CTYPE1 as DWORD,
        &raw mut wc as LPCWSTR,
        1 as ::core::ffi::c_int,
        &raw mut CharType,
    ) != 0
        && (CharType as ::core::ffi::c_int & (C1_BLANK as i32)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharPunctW(mut wc: WCHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return (GetStringTypeW(
        CT_CTYPE1 as DWORD,
        core::ptr::addr_of_mut!(wc) as LPCWSTR,
        1 as ::core::ffi::c_int,
        core::ptr::addr_of_mut!(CharType),
    ) != 0
        && (CharType & C1_PUNCT as WORD) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharCntrlW(wc: WCHAR) -> BOOL {
    let mut wc = wc;
    let mut CharType: WORD = 0;
    return (GetStringTypeW(CT_CTYPE1, core::ptr::addr_of_mut!(wc), 1, core::ptr::addr_of_mut!(CharType)) != 0 && (CharType & (C1_CNTRL as WORD)) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharDigitW(mut wc: WCHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return (GetStringTypeW(
        CT_CTYPE1 as DWORD,
        &raw mut wc as LPCWSTR,
        1 as ::core::ffi::c_int,
        &raw mut CharType,
    ) != 0
        && CharType as ::core::ffi::c_int & (C1_DIGIT as i32) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IsCharXDigitW(mut wc: WCHAR) -> BOOL {
    let mut CharType: WORD = 0;
    return (GetStringTypeW(
        CT_CTYPE1 as DWORD,
        &raw mut wc as LPCWSTR,
        1 as ::core::ffi::c_int,
        &raw mut CharType,
    ) != 0
        && CharType as ::core::ffi::c_int & (C1_XDIGIT as i32) != 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GetStringType3ExW(src: LPWSTR, count: INT, type_0: LPWORD) -> BOOL {
    return GetStringTypeW(CT_CTYPE3 as DWORD, src as LPCWSTR, count, type_0);
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNCA(lpszSrc: LPCSTR, lpszCmp: LPCSTR, len: INT) -> DWORD {
    (StrCmpNA(lpszSrc, lpszCmp, len)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNCW(lpszSrc: LPCWSTR, lpszCmp: LPCWSTR, len: INT) -> DWORD {
    (StrCmpNW(lpszSrc, lpszCmp, len)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNICA(
    lpszSrc: LPCSTR,
    lpszCmp: LPCSTR,
    len: DWORD,
) -> DWORD {
    (StrCmpNIA(lpszSrc, lpszCmp, (len) as i32)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpNICW(
    lpszSrc: LPCWSTR,
    lpszCmp: LPCWSTR,
    len: DWORD,
) -> DWORD {
    (StrCmpNIW(lpszSrc, lpszCmp, (len) as i32)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpCA(lpszSrc: LPCSTR, lpszCmp: LPCSTR) -> DWORD {
    (lstrcmpA(lpszSrc, lpszCmp)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpCW(lpszSrc: LPCWSTR, lpszCmp: LPCWSTR) -> DWORD {
    (lstrcmpW(lpszSrc, lpszCmp)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpICA(lpszSrc: LPCSTR, lpszCmp: LPCSTR) -> DWORD {
    (lstrcmpiA(lpszSrc, lpszCmp)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn StrCmpICW(lpszSrc: LPCWSTR, lpszCmp: LPCWSTR) -> DWORD {
    (lstrcmpiW(lpszSrc, lpszCmp)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn SHAboutInfoA(lpszDest: LPSTR, dwDestLen: DWORD) -> BOOL {
    let mut buff: [WCHAR; 2084] = [0; 2084];
    if !lpszDest.is_null() && SHAboutInfoW(core::ptr::addr_of_mut!(buff) as LPWSTR, dwDestLen) != 0 {
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            core::ptr::addr_of_mut!(buff) as *mut WCHAR as LPCWSTR,
            -1 as ::core::ffi::c_int,
            lpszDest,
            dwDestLen as ::core::ffi::c_int,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHAboutInfoW(lpszDest: LPWSTR, dwDestLen: DWORD) -> BOOL {
    static szIEKey: [WCHAR; 37] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0049,
    0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0020, 0x0045, 0x0078,
    0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x0000,
];
    static szWinNtKey: [WCHAR; 45] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x0020, 0x004E, 0x0054, 0x005C,
    0x0043, 0x0075, 0x0072, 0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072,
    0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static szWinKey: [WCHAR; 42] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x0000,
];
    static szRegKey: [WCHAR; 50] = [
    0x0053u16, 0x004F, 0x0046, 0x0054, 0x0057, 0x0041, 0x0052, 0x0045, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0049,
    0x006E, 0x0074, 0x0065, 0x0072, 0x006E, 0x0065, 0x0074, 0x0020, 0x0045, 0x0078,
    0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072, 0x005C, 0x0052, 0x0065, 0x0067,
    0x0069, 0x0073, 0x0074, 0x0072, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static szVersion: [WCHAR; 8] = [0x0056u16, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000];
    static szCustomized: [WCHAR; 18] = [
    0x0043u16, 0x0075, 0x0073, 0x0074, 0x006F, 0x006D, 0x0069, 0x007A, 0x0065, 0x0064,
    0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F, 0x006E, 0x0000,
];
    static szOwner: [WCHAR; 16] = [
    0x0052u16, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0065, 0x0072, 0x0065, 0x0064,
    0x004F, 0x0077, 0x006E, 0x0065, 0x0072, 0x0000,
];
    static szOrg: [WCHAR; 23] = [
    0x0052u16, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0065, 0x0072, 0x0065, 0x0064,
    0x004F, 0x0072, 0x0067, 0x0061, 0x006E, 0x0069, 0x007A, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0000,
];
    static szProduct: [WCHAR; 10] = [0x0050u16, 0x0072, 0x006F, 0x0064, 0x0075, 0x0063, 0x0074, 0x0049, 0x0064, 0x0000];
    static szUpdate: [WCHAR; 14] = [
    0x0049u16, 0x0045, 0x0041, 0x004B, 0x0055, 0x0070, 0x0064, 0x0061, 0x0074, 0x0065,
    0x0055, 0x0072, 0x006C, 0x0000,
];
    static szHelp: [WCHAR; 15] = [
    0x0049u16, 0x0045, 0x0041, 0x004B, 0x0048, 0x0065, 0x006C, 0x0070, 0x0053, 0x0074,
    0x0072, 0x0069, 0x006E, 0x0067, 0x0000,
];
    let mut buff: [WCHAR; 2084] = [0; 2084];
    let mut hReg: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwLen: DWORD = 0;

    if lpszDest.is_null() {
        return (FALSE) as BOOL;
    }

    *lpszDest = '\0' as WCHAR;

    /* Try the NT key first, followed by 95/98 key */
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(szWinNtKey) as LPCWSTR,
        0,
        KEY_READ as REGSAM,
        core::ptr::addr_of_mut!(hReg),
    ) != 0
        && RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of!(szWinKey) as LPCWSTR,
            0,
            KEY_READ as REGSAM,
            core::ptr::addr_of_mut!(hReg),
        ) != 0
    {
        return (FALSE) as BOOL;
    }

    /* OS Version */
    buff[0] = '\0' as WCHAR;
    dwLen = 30;
    if SHGetValueW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(szIEKey) as LPCWSTR,
        core::ptr::addr_of!(szVersion) as LPCWSTR,
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr() as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) == 0
    {
        let mut dwStrLen: DWORD = strlenW(buff.as_mut_ptr() as LPCWSTR) as DWORD;
        dwLen = (30 as DWORD).wrapping_sub(dwStrLen);
        SHGetValueW(
            HKEY_LOCAL_MACHINE,
            core::ptr::addr_of!(szIEKey) as LPCWSTR,
            core::ptr::addr_of!(szCustomized) as LPCWSTR,
            core::ptr::addr_of_mut!(dwType),
            buff.as_mut_ptr().offset(dwStrLen as isize) as LPVOID,
            core::ptr::addr_of_mut!(dwLen),
        );
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* ~Registered Owner */
    buff[0] = '~' as WCHAR;
    dwLen = 256;
    if SHGetValueW(
        hReg,
        core::ptr::addr_of!(szOwner) as LPCWSTR,
        core::ptr::null(),
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr().offset(1) as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) != 0
    {
        buff[1] = '\0' as WCHAR;
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* ~Registered Organization */
    dwLen = 256;
    if SHGetValueW(
        hReg,
        core::ptr::addr_of!(szOrg) as LPCWSTR,
        core::ptr::null(),
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr().offset(1) as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) != 0
    {
        buff[1] = '\0' as WCHAR;
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* FIXME: Not sure where this number comes from  */
    buff[0] = '~' as WCHAR;
    buff[1] = '0' as WCHAR;
    buff[2] = '\0' as WCHAR;
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* ~Product Id */
    dwLen = 256;
    if SHGetValueW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(szRegKey) as LPCWSTR,
        core::ptr::addr_of!(szProduct) as LPCWSTR,
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr().offset(1) as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) != 0
    {
        buff[1] = '\0' as WCHAR;
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* ~IE Update Url */
    dwLen = 2048;
    if SHGetValueW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of!(szWinKey) as LPCWSTR,
        core::ptr::addr_of!(szUpdate) as LPCWSTR,
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr().offset(1) as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) != 0
    {
        buff[1] = '\0' as WCHAR;
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    /* ~IE Help String */
    dwLen = 256;
    if SHGetValueW(
        hReg,
        core::ptr::addr_of!(szHelp) as LPCWSTR,
        core::ptr::null(),
        core::ptr::addr_of_mut!(dwType),
        buff.as_mut_ptr().offset(1) as LPVOID,
        core::ptr::addr_of_mut!(dwLen),
    ) != 0
    {
        buff[1] = '\0' as WCHAR;
    }
    StrCatBuffW(lpszDest, buff.as_mut_ptr() as LPCWSTR, dwDestLen as INT);

    RegCloseKey(hReg);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryStatus(
    lpUnknown: *mut IUnknown,
    pguidCmdGroup: *const GUID,
    cCmds: ULONG,
    prgCmds: *mut OLECMD,
    pCmdText: *mut OLECMDTEXT,
) -> HRESULT {
    let mut hRet: HRESULT = E_FAIL;

    if !lpUnknown.is_null() {
        let mut lpOle: *mut IOleCommandTarget = core::ptr::null_mut();

        hRet = IUnknown_QueryInterface(
            lpUnknown,
            &raw const IID_IOleCommandTarget,
            &raw mut lpOle as *mut *mut core::ffi::c_void,
        );

        if hRet >= 0 && !lpOle.is_null() {
            hRet = IOleCommandTarget_QueryStatus(lpOle, pguidCmdGroup, cCmds, prgCmds, pCmdText);
            IOleCommandTarget_Release(lpOle);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_Exec(
    lpUnknown: *mut IUnknown,
    pguidCmdGroup: *const GUID,
    nCmdID: DWORD,
    nCmdexecopt: DWORD,
    pvaIn: *mut VARIANT,
    pvaOut: *mut VARIANT,
) -> HRESULT {
    let mut hRet: HRESULT = E_FAIL;

    if !lpUnknown.is_null() {
        let mut lpOle: *mut IOleCommandTarget = core::ptr::null_mut::<IOleCommandTarget>();

        hRet = IUnknown_QueryInterface(
            lpUnknown,
            core::ptr::addr_of!(IID_IOleCommandTarget),
            core::ptr::addr_of_mut!(lpOle) as *mut *mut core::ffi::c_void,
        );
        if (hRet >= 0) && !lpOle.is_null() {
            hRet = IOleCommandTarget_Exec(lpOle, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut);
            IOleCommandTarget_Release(lpOle);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetWindowBits(hwnd: HWND, offset: INT, mask: UINT, flags: UINT) -> LONG {
    let mut ret: LONG = GetWindowLongW(hwnd, offset);
    let mut new_flags: LONG = ((flags & mask) | (ret as UINT & !mask)) as LONG;

    // TRACE("%p %d %x %x\n", hwnd, offset, mask, flags) from the anchor: debug-channel logging only, no effect on results.

    if new_flags != ret {
        ret = SetWindowLongW(hwnd, offset, new_flags);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetParentHwnd(mut hWnd: HWND, mut hWndParent: HWND) -> HWND {
    WINE_TRACE!("%p, %p\n", hWnd, hWndParent);
    if GetParent(hWnd) == hWndParent {
        return core::ptr::null_mut();
    }
    if !hWndParent.is_null() {
        SHSetWindowBits(
            hWnd,
            GWL_STYLE,
            WS_CHILD as UINT | WS_POPUP,
            WS_CHILD,
        );
    } else {
        SHSetWindowBits(
            hWnd,
            GWL_STYLE,
            WS_CHILD as UINT | WS_POPUP,
            WS_POPUP,
        );
    }
    return if !hWndParent.is_null() {
        SetParent(hWnd, hWndParent)
    } else {
        core::ptr::null_mut()
    };
}

#[no_mangle]
pub unsafe extern "system" fn ConnectToConnectionPoint(
    lpUnkSink: *mut IUnknown,
    riid: REFIID,
    fConnect: BOOL,
    lpUnknown: *mut IUnknown,
    lpCookie: LPDWORD,
    lppCP: *mut *mut IConnectionPoint,
) -> HRESULT {
    let mut hRet: HRESULT;
    let mut lpContainer: *mut IConnectionPointContainer = core::ptr::null_mut();
    let mut lpCP: *mut IConnectionPoint = core::ptr::null_mut();

    if lpUnknown.is_null() || (fConnect != 0 && lpUnkSink.is_null()) {
        return E_FAIL;
    }

    if !lppCP.is_null() {
        *lppCP = core::ptr::null_mut();
    }

    hRet = IUnknown_QueryInterface(
        lpUnknown,
        core::ptr::addr_of!(IID_IConnectionPointContainer),
        core::ptr::addr_of_mut!(lpContainer) as *mut *mut core::ffi::c_void,
    );
    if hRet >= 0 {
        hRet = IConnectionPointContainer_FindConnectionPoint(lpContainer, riid, core::ptr::addr_of_mut!(lpCP));

        if hRet >= 0 {
            if fConnect == 0 {
                hRet = IConnectionPoint_Unadvise(lpCP, *lpCookie);
            } else {
                hRet = IConnectionPoint_Advise(lpCP, lpUnkSink, lpCookie);
            }

            if hRet < 0 {
                *lpCookie = 0;
            }

            if !lppCP.is_null() && hRet >= 0 {
                *lppCP = lpCP; /* Caller keeps the interface */
            } else {
                IConnectionPoint_Release(lpCP); /* Release it */
            }
        }

        IConnectionPointContainer_Release(lpContainer);
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_AtomicRelease(lpUnknown: *mut *mut IUnknown) {
    if lpUnknown.is_null() || (*lpUnknown).is_null() {
        return;
    }

    IUnknown_Release(*lpUnknown);
    *lpUnknown = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn PathSkipLeadingSlashesA(mut lpszSrc: LPCSTR) -> LPCSTR {
    if !lpszSrc.is_null()
        && *lpszSrc.offset(0 as isize) as i32 == '/' as i32
        && *lpszSrc.offset(1 as isize) as i32 == '/' as i32
    {
        lpszSrc = lpszSrc.offset(2 as isize);
    }
    return lpszSrc;
}

#[no_mangle]
pub unsafe extern "system" fn SHIsSameObject(lpInt1: *mut IUnknown, lpInt2: *mut IUnknown) -> BOOL {
    let mut lpUnknown1: *mut IUnknown = core::ptr::null_mut();
    let mut lpUnknown2: *mut IUnknown = core::ptr::null_mut();
    let mut ret: BOOL;

    if lpInt1.is_null() || lpInt2.is_null() {
        return (FALSE) as BOOL;
    }

    if lpInt1 == lpInt2 {
        return (TRUE) as BOOL;
    }

    if IUnknown_QueryInterface(
        lpInt1,
        &raw const IID_IUnknown,
        &raw mut lpUnknown1 as *mut *mut core::ffi::c_void,
    ) != S_OK
    {
        return (FALSE) as BOOL;
    }

    if IUnknown_QueryInterface(
        lpInt2,
        &raw const IID_IUnknown,
        &raw mut lpUnknown2 as *mut *mut core::ffi::c_void,
    ) != S_OK
    {
        IUnknown_Release(lpUnknown1);
        return (FALSE) as BOOL;
    }

    ret = (lpUnknown1 == lpUnknown2) as i32;

    IUnknown_Release(lpUnknown1);
    IUnknown_Release(lpUnknown2);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_GetWindow(lpUnknown: *mut IUnknown, lphWnd: *mut HWND) -> HRESULT {
    let mut lpOle: *mut IUnknown = core::ptr::null_mut();
    let mut hRet: HRESULT = E_FAIL;

    if lpUnknown.is_null() {
        return hRet;
    }

    hRet = IUnknown_QueryInterface(
        lpUnknown,
        &raw const IID_IOleWindow,
        &raw mut lpOle as *mut *mut core::ffi::c_void,
    );

    if hRet < 0 {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            &raw const IID_IShellView,
            &raw mut lpOle as *mut *mut core::ffi::c_void,
        );

        if hRet < 0 {
            hRet = IUnknown_QueryInterface(
                lpUnknown,
                &raw const IID_IInternetSecurityMgrSite,
                &raw mut lpOle as *mut *mut core::ffi::c_void,
            );
        }
    }

    if hRet >= 0 {
        /* Laziness here - Since GetWindow() is the first method for the above 3
         * interfaces, we use the same call for them all.
         */
        hRet = IOleWindow_GetWindow(lpOle as *mut IOleWindow, lphWnd);
        IUnknown_Release(lpOle);
        if !lphWnd.is_null() {
            TRACE!("Returning HWND=%p\n", *lphWnd);
        }
    }

    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_SetOwner(iface: *mut IUnknown, pUnk: *mut IUnknown) -> HRESULT {
    let mut service: *mut IShellService = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    if iface.is_null() {
        return E_FAIL;
    }

    hr = IUnknown_QueryInterface(
        iface,
        core::ptr::addr_of!(IID_IShellService),
        core::ptr::addr_of_mut!(service) as *mut *mut core::ffi::c_void,
    );
    if hr == S_OK {
        hr = IShellService_SetOwner(service, pUnk);
        IShellService_Release(service);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_SetSite(
    obj: *mut IUnknown,
    site: *mut IUnknown,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut iobjwithsite: *mut IObjectWithSite = core::ptr::null_mut();
    let mut isecmgr: *mut IInternetSecurityManager = core::ptr::null_mut();

    if obj.is_null() {
        return E_FAIL;
    }

    hr = IUnknown_QueryInterface(
        obj,
        core::ptr::addr_of!(IID_IObjectWithSite),
        core::ptr::addr_of_mut!(iobjwithsite) as *mut LPVOID,
    );

    if hr >= 0 {
        hr = IObjectWithSite_SetSite(iobjwithsite, site);
        IObjectWithSite_Release(iobjwithsite);
    } else {
        hr = IUnknown_QueryInterface(
            obj,
            core::ptr::addr_of!(IID_IInternetSecurityManager),
            core::ptr::addr_of_mut!(isecmgr) as *mut LPVOID,
        );

        if hr < 0 {
            return hr;
        }

        hr = IInternetSecurityManager_SetSecuritySite(isecmgr, site as *mut IInternetSecurityMgrSite);
        IInternetSecurityManager_Release(isecmgr);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_GetClassID(lpUnknown: *mut IUnknown, clsid: *mut CLSID) -> HRESULT {
    let mut persist: *mut IPersist = core::ptr::null_mut();
    let mut hr: HRESULT;

    if lpUnknown.is_null() {
        memset(clsid as *mut core::ffi::c_void, 0, core::mem::size_of::<CLSID>());
        return E_FAIL;
    }

    hr = IUnknown_QueryInterface(lpUnknown, core::ptr::addr_of!(IID_IPersist), core::ptr::addr_of_mut!(persist) as *mut *mut core::ffi::c_void);
    if hr != S_OK {
        hr = IUnknown_QueryInterface(lpUnknown, core::ptr::addr_of!(IID_IPersistFolder), core::ptr::addr_of_mut!(persist) as *mut *mut core::ffi::c_void);
        if hr != S_OK {
            return hr;
        }
    }

    hr = IPersist_GetClassID(persist, clsid);
    IPersist_Release(persist);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryService(
    lpUnknown: *mut IUnknown,
    sid: REFGUID,
    riid: REFIID,
    lppOut: *mut LPVOID,
) -> HRESULT {
    let mut pService: *mut IServiceProvider = core::ptr::null_mut();
    let mut hRet: HRESULT;

    if lppOut.is_null() {
        return E_FAIL;
    }

    *lppOut = core::ptr::null_mut();

    if lpUnknown.is_null() {
        return E_FAIL;
    }

    hRet = IUnknown_QueryInterface(
        lpUnknown,
        core::ptr::addr_of!(IID_IServiceProvider),
        &mut pService as *mut *mut IServiceProvider as *mut LPVOID,
    );

    if hRet == S_OK && !pService.is_null() {
        /* Get a Service interface from the object */
        hRet = IServiceProvider_QueryService(pService, sid, riid, lppOut as *mut *mut core::ffi::c_void);

        IServiceProvider_Release(pService);
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryServiceExec(
    lpUnknown: *mut IUnknown,
    service: *const IID,
    group: *const GUID,
    cmdId: DWORD,
    cmdOpt: DWORD,
    pIn: *mut VARIANT,
    pOut: *mut VARIANT,
) -> HRESULT {
    let mut target: *mut IOleCommandTarget = core::ptr::null_mut();
    let mut hr: HRESULT = 0;

    hr = IUnknown_QueryService(
        lpUnknown,
        service as *const GUID,
        core::ptr::addr_of!(IID_IOleCommandTarget),
        core::ptr::addr_of_mut!(target) as *mut LPVOID,
    );
    if hr == S_OK {
        hr = IOleCommandTarget_Exec(target, group, cmdId, cmdOpt, pIn, pOut);
        IOleCommandTarget_Release(target);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_ProfferService(
    lpUnknown: *mut IUnknown,
    service: REFGUID,
    pService: *mut IServiceProvider,
    pCookie: *mut DWORD,
) -> HRESULT {
    let mut proffer: *mut IProfferService = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = IUnknown_QueryService(
        lpUnknown,
        core::ptr::addr_of!(IID_IProfferService),
        core::ptr::addr_of!(IID_IProfferService),
        core::ptr::addr_of_mut!(proffer) as *mut LPVOID,
    );
    if hr == S_OK {
        if !pService.is_null() {
            hr = IProfferService_ProfferService(proffer, service, pService, pCookie);
        } else {
            hr = IProfferService_RevokeService(proffer, *pCookie);
            *pCookie = 0;
        }

        IProfferService_Release(proffer);
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_UIActivateIO(unknown: *mut IUnknown, activate: BOOL, msg: LPMSG) -> HRESULT {
    let mut object: *mut IInputObject = core::ptr::null_mut();
    let mut ret: HRESULT;
    if unknown.is_null() {
        return E_FAIL;
    }
    /* Get an IInputObject interface from the object */
    ret = IUnknown_QueryInterface(unknown, core::ptr::addr_of!(IID_IInputObject), core::ptr::addr_of_mut!(object) as *mut LPVOID);
    if ret == S_OK {
        ret = IInputObject_UIActivateIO(object, activate, msg);
        IInputObject_Release(object);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHLoadMenuPopup(hInst: HINSTANCE, szName: LPCWSTR) -> BOOL {
    let mut hMenu: HMENU = core::ptr::null_mut();
    hMenu = LoadMenuW(hInst, szName);
    if !hMenu.is_null() {
        if !GetSubMenu(hMenu, 0).is_null() {
            RemoveMenu(hMenu, 0, MF_BYPOSITION);
        }
        DestroyMenu(hMenu);
        return (TRUE) as BOOL;
    }
    return (FALSE) as BOOL;
}

unsafe extern "system" fn SHLWAPI_EnumChildProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let mut data: *mut enumWndData = lParam as *mut enumWndData;
    WINE_TRACE!("(%p,%p)\n", hWnd, data);
    (*data).pfnPost.unwrap_unchecked()(
        hWnd,
        (*data).uiMsgId,
        (*data).wParam,
        (*data).lParam,
    );
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropagateMessage(
    hWnd: HWND,
    uiMsgId: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
    bSend: BOOL,
) {

    let mut data: enumWndData = core::mem::zeroed();

    if !hWnd.is_null() {
        data.uiMsgId = uiMsgId;
        data.wParam = wParam;
        data.lParam = lParam;

        if bSend != 0 {
            data.pfnPost = core::mem::transmute(
                (if IsWindowUnicode(hWnd) != 0 {
                    SendMessageW as *mut core::ffi::c_void
                } else {
                    SendMessageA as *mut core::ffi::c_void
                }),
            );
        } else {
            data.pfnPost = core::mem::transmute(
                (if IsWindowUnicode(hWnd) != 0 {
                    PostMessageW as *mut core::ffi::c_void
                } else {
                    PostMessageA as *mut core::ffi::c_void
                }),
            );
        }

        EnumChildWindows(
            hWnd,
            Some(SHLWAPI_EnumChildProc as unsafe extern "system" fn(HWND, LPARAM) -> BOOL),
            core::ptr::addr_of_mut!(data) as LPARAM,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHRemoveAllSubMenus(hMenu: HMENU) -> DWORD {
    let mut iItemCount: i32 = GetMenuItemCount(hMenu) - 1 as i32;

    while iItemCount >= 0 as i32 {
        let mut hSubMenu: HMENU = GetSubMenu(hMenu, iItemCount);
        if !hSubMenu.is_null() {
            RemoveMenu(hMenu, iItemCount as UINT, MF_BYPOSITION as UINT);
        }
        iItemCount -= 1;
    }
    return iItemCount as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHEnableMenuItem(hMenu: HMENU, wItemID: UINT, bEnable: BOOL) -> UINT {
    return EnableMenuItem(hMenu, wItemID, if bEnable != 0 { MF_ENABLED } else { MF_GRAYED }) as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn SHCheckMenuItem(hMenu: HMENU, uID: UINT, bCheck: BOOL) -> DWORD {
    return CheckMenuItem(hMenu, uID, if bCheck != 0 { MF_CHECKED } else { MF_UNCHECKED });
}

#[no_mangle]
pub unsafe extern "system" fn SHRegisterClassA(wndclass: *mut WNDCLASSA) -> DWORD {
    let mut wca: WNDCLASSA = core::mem::zeroed();
    if GetClassInfoA((*wndclass).hInstance, (*wndclass).lpszClassName, core::ptr::addr_of_mut!(wca)) != 0 {
        return TRUE as DWORD;
    }
    return RegisterClassA(wndclass) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHSimulateDrop(
    mut pDrop: *mut IDropTarget,
    mut pDataObj: *mut IDataObject,
    mut grfKeyState: DWORD,
    mut lpPt: PPOINTL,
    mut pdwEffect: *mut DWORD,
) -> BOOL {
    let mut dwEffect: DWORD = (DROPEFFECT_LINK | DROPEFFECT_MOVE | DROPEFFECT_COPY) as DWORD;
    let mut pt: POINTL = POINTL {
        x: 0 as LONG,
        y: 0 as LONG,
    };

    if lpPt.is_null() {
        lpPt = core::ptr::addr_of_mut!(pt);
    }

    if pdwEffect.is_null() {
        pdwEffect = core::ptr::addr_of_mut!(dwEffect);
    }

    IDropTarget_DragEnter(pDrop, pDataObj, grfKeyState, *lpPt, pdwEffect);

    if *pdwEffect != DROPEFFECT_NONE as DWORD {
        return IDropTarget_Drop(pDrop, pDataObj, grfKeyState, *lpPt, pdwEffect) as BOOL;
    }

    IDropTarget_DragLeave(pDrop);
    return TRUE as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHLoadFromPropertyBag(
    lpUnknown: *mut IUnknown,
    lpPropBag: *mut IPropertyBag,
) -> DWORD {
    let mut lpPPBag: *mut IPersistPropertyBag = core::ptr::null_mut();
    let mut hRet: HRESULT = E_FAIL;

    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            core::ptr::addr_of!(IID_IPersistPropertyBag),
            &mut lpPPBag as *mut *mut IPersistPropertyBag as *mut LPVOID,
        );
        if hRet >= 0 && !lpPPBag.is_null() {
            hRet = IPersistPropertyBag_Load(lpPPBag, lpPropBag, core::ptr::null_mut());
            IPersistPropertyBag_Release(lpPPBag);
        }
    }
    return hRet as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_TranslateAcceleratorOCS(lpUnknown: *mut IUnknown, lpMsg: LPMSG, dwModifiers: DWORD) -> HRESULT {
    let mut lpCSite: *mut IOleControlSite = core::ptr::null_mut();
    let mut hRet: HRESULT = E_INVALIDARG;
    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            &raw const IID_IOleControlSite,
            &raw mut lpCSite as *mut *mut core::ffi::c_void,
        );
        if hRet >= 0 && !lpCSite.is_null() {
            hRet = IOleControlSite_TranslateAccelerator(lpCSite, lpMsg, dwModifiers);
            IOleControlSite_Release(lpCSite);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_OnFocusOCS(lpUnknown: *mut IUnknown, fGotFocus: BOOL) -> HRESULT {
    let mut lpCSite: *mut IOleControlSite = core::ptr::null_mut();
    let mut hRet: HRESULT = E_FAIL;

    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            &raw const IID_IOleControlSite,
            &raw mut lpCSite as *mut *mut core::ffi::c_void,
        );
        if hRet >= 0 && !lpCSite.is_null() {
            hRet = IOleControlSite_OnFocus(lpCSite, fGotFocus);
            IOleControlSite_Release(lpCSite);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_HandleIRestrict(lpUnknown: LPUNKNOWN, lpArg1: PVOID, lpArg2: PVOID, lpArg3: PVOID, lpArg4: PVOID) -> HRESULT {
    /* FIXME: {D12F26B2-D90A-11D0-830D-00AA005B4383} - What object does this represent? */
    static service_id: [DWORD; 4] = [0xd12f26b2, 0x11d0d90a, 0xaa000d83, 0x83435b00];
    /* FIXME: {D12F26B1-D90A-11D0-830D-00AA005B4383} - Also Unknown/undocumented */
    static function_id: [DWORD; 4] = [0xd12f26b1, 0x11d0d90a, 0xaa000d83, 0x83435b00];
    let mut hRet: HRESULT = E_INVALIDARG;
    let mut lpUnkInner: LPUNKNOWN = core::ptr::null_mut();

    if !lpUnknown.is_null() && !lpArg4.is_null() {
        hRet = IUnknown_QueryService(
            lpUnknown,
            core::ptr::addr_of!(service_id) as *const DWORD as REFGUID,
            core::ptr::addr_of!(function_id) as *const DWORD as REFGUID,
            core::ptr::addr_of_mut!(lpUnkInner) as *mut LPVOID,
        );

        if (hRet >= 0) && !lpUnkInner.is_null() {
            /* FIXME: The type of service object requested is unknown, however
             * testing shows that its first method is called with 4 parameters.
             * Fake this by using IParseDisplayName_ParseDisplayName since the
             * signature and position in the vtable matches our unknown object type.
             */
            hRet = IParseDisplayName_ParseDisplayName(
                lpUnkInner as LPPARSEDISPLAYNAME,
                (lpArg1).cast(),
                (lpArg2).cast(),
                (lpArg3).cast(),
                (lpArg4).cast(),
            );
            IUnknown_Release(lpUnkInner);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetMenuFromID(hMenu: HMENU, uID: UINT) -> HMENU {
    let mut mi: MENUITEMINFOW = core::mem::zeroed();

    mi.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mi.fMask = MIIM_SUBMENU;

    if GetMenuItemInfoW(hMenu, uID, (FALSE) as BOOL, core::ptr::addr_of_mut!(mi)) == 0 {
        return core::ptr::null_mut();
    }

    return mi.hSubMenu;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetCurColorRes() -> DWORD {
    let mut hdc: HDC;
    let mut ret: DWORD;

    hdc = GetDC(core::ptr::null_mut());
    ret = (GetDeviceCaps(hdc, (BITSPIXEL as i32)) * GetDeviceCaps(hdc, (PLANES as i32))) as DWORD;
    ReleaseDC(core::ptr::null_mut(), hdc);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHWaitForSendMessageThread(mut hand: HANDLE, mut dwTimeout: DWORD) -> DWORD {
    let mut dwEndTicks: DWORD = GetTickCount().wrapping_add(dwTimeout);
    let mut dwRet: DWORD = 0;
    loop {
        dwRet = MsgWaitForMultipleObjectsEx(
            1,
            core::ptr::addr_of_mut!(hand),
            dwTimeout,
            QS_SENDMESSAGE,
            0,
        );
        if !(dwRet == 1) {
            break;
        }
        let mut msg: MSG = core::mem::zeroed();

        PeekMessageW(
            core::ptr::addr_of_mut!(msg),
            core::ptr::null_mut(),
            0,
            0,
            PM_NOREMOVE,
        );

        if dwTimeout != INFINITE {
            dwTimeout = dwEndTicks.wrapping_sub(GetTickCount());
            if (dwTimeout as i32) <= 0 {
                return WAIT_TIMEOUT;
            }
        }
    }

    return dwRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHIsExpandableFolder(mut lpFolder: LPSHELLFOLDER, mut pidl: LPCITEMIDLIST) -> HRESULT {
    let mut hRet: HRESULT = E_INVALIDARG;
    let mut lpInfo: *mut IQueryInfo = core::ptr::null_mut();

    if !lpFolder.is_null() && !pidl.is_null() {
        hRet = IShellFolder_GetUIObjectOf(
            lpFolder as *mut IShellFolder,
            core::ptr::null_mut(),
            1,
            core::ptr::addr_of_mut!(pidl) as PCUITEMID_CHILD_ARRAY,
            core::ptr::addr_of!(IID_IQueryInfo),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(lpInfo) as *mut *mut core::ffi::c_void,
        );
        if hRet < 0 {
            hRet = S_FALSE; /* Doesn't expose IQueryInfo */
        } else {
            let mut dwFlags: DWORD = 0;

            /* MSDN states of IQueryInfo_GetInfoFlags() that "This method is not
             * currently used". Really? You wouldn't be holding out on me would you?
             */
            hRet = IQueryInfo_GetInfoFlags(lpInfo, core::ptr::addr_of_mut!(dwFlags));

            if hRet >= 0 {
                /* 0x2 is an undocumented flag apparently indicating expandability */
                hRet = if dwFlags & 0x2 != 0 {
                    S_OK
                } else {
                    S_FALSE
                };
            }

            IQueryInfo_Release(lpInfo);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHFillRectClr(hDC: HDC, pRect: LPCRECT, cRef: COLORREF) -> DWORD {
    let cOldColor: COLORREF = SetBkColor(hDC, cRef);
    ExtTextOutA(hDC, 0, 0, ETO_OPAQUE, pRect, core::ptr::null(), 0, core::ptr::null());
    SetBkColor(hDC, cOldColor);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SHSearchMapInt(lpKeys: *const i32, lpValues: *const i32, iLen: i32, iKey: i32) -> i32 {
    if !lpKeys.is_null() && !lpValues.is_null() {
        let mut i: i32 = 0;

        while i < iLen {
            if *lpKeys.offset(i as isize) == iKey {
                return *lpValues.offset(i as isize); /* Found */
            }
            i += 1;
        }
    }
    return -1; /* Not found */
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_Set(lppDest: *mut *mut IUnknown, lpUnknown: *mut IUnknown) {
    IUnknown_AtomicRelease(lppDest);

    if !lpUnknown.is_null() {
        IUnknown_AddRef(lpUnknown);
        *lppDest = lpUnknown;
    }
}

#[no_mangle]
pub unsafe extern "system" fn MayQSForward(
    lpUnknown: *mut IUnknown,
    nUnknown: INT,
    riidCmdGrp: REFGUID,
    cCmds: ULONG,
    prgCmds: *mut OLECMD,
    pCmdText: *mut OLECMDTEXT,
) -> HRESULT {
    let hr: HRESULT = IsQSForward(riidCmdGrp, cCmds, prgCmds);
    if hr < 0 || HRESULT_CODE!(hr) == 0 || nUnknown <= 0 {
        return OLECMDERR_E_NOTSUPPORTED;
    }

    return IUnknown_QueryStatus(lpUnknown, riidCmdGrp, cCmds, prgCmds, pCmdText);
}

#[no_mangle]
pub unsafe extern "system" fn MayExecForward(
    lpUnknown: *mut IUnknown,
    nUnknown: INT,
    pguidCmdGroup: *const GUID,
    nCmdID: DWORD,
    nCmdexecopt: DWORD,
    pvaIn: *mut VARIANT,
    pvaOut: *mut VARIANT,
) -> HRESULT {
    let mut cmd: OLECMD = OLECMD {
        cmdID: nCmdID as ULONG,
        cmdf: 0,
    };
    let mut hr: HRESULT = IsQSForward(
        pguidCmdGroup,
        1 as ULONG,
        core::ptr::addr_of_mut!(cmd),
    );
    if hr < 0 || HRESULT_CODE!(hr) == 0 || nUnknown <= 0 {
        return OLECMDERR_E_NOTSUPPORTED;
    }

    return IUnknown_Exec(lpUnknown, pguidCmdGroup, nCmdID, nCmdexecopt, pvaIn, pvaOut);
}

#[no_mangle]
pub unsafe extern "system" fn IsQSForward(
    pguidCmdGroup: *const GUID,
    cCmds: ULONG,
    prgCmds: *mut OLECMD,
) -> HRESULT {
    const CMD_FLAG_SUPPORTED_BASIC: u32 = 0x1;
    const CMD_FLAG_SUPPORTED_ADVANCED: u32 = 0x2;
    const CMD_FLAG_NOT_SUPPORTED: u32 = 0x4;

    let mut cmdFlags: DWORD = 0;
    let mut cmdID: OLECMDID = 0 as OLECMDID;
    let mut iCmd: ULONG = 0;

    TRACE!(
        "(%s, %lu, %p)\n",
        wine_dbgstr_guid(pguidCmdGroup),
        cCmds,
        prgCmds
    );

    if cCmds as LONG <= 0 {
        return OLECMDERR_E_NOTSUPPORTED;
    }

    if pguidCmdGroup.is_null() {
        iCmd = 0;
        while iCmd < cCmds {
            cmdID = (*prgCmds.offset(iCmd as isize)).cmdID as OLECMDID;
            if (cmdID as u32) <= OLECMDID_PROPERTIES {
                cmdFlags |= CMD_FLAG_NOT_SUPPORTED; // Not supported
            } else if (cmdID as u32) <= OLECMDID_PASTE
                || (cmdID as u32) == OLECMDID_SELECTALL
            {
                cmdFlags |= CMD_FLAG_SUPPORTED_BASIC;
            } else if (cmdID as u32) <= OLECMDID_UPDATECOMMANDS
                || (OLECMDID_HIDETOOLBARS <= (cmdID as u32)
                    && (cmdID as u32) != OLECMDID_ENABLE_INTERACTION)
            {
                cmdFlags |= CMD_FLAG_NOT_SUPPORTED; // Not supported
            } else {
                cmdFlags |= CMD_FLAG_SUPPORTED_ADVANCED;
            }
            iCmd = iCmd.wrapping_add(1);
        }
    } else {
        if IsEqualGUID!(&raw const CGID_Explorer, pguidCmdGroup) == false {
            // `#if (_WIN32_WINNT >= _WIN32_WINNT_VISTA)` is dead at the module's 0x502: the
            // #else arm. (The bought body took the VISTA arm.)
            return OLECMDERR_E_NOTSUPPORTED;
        }

        iCmd = 0;
        while iCmd < cCmds {
            cmdID = (*prgCmds.offset(iCmd as isize)).cmdID as OLECMDID;
            if (cmdID as u32) == OLECMDID_SELECTALL
                || (OLECMDID_SHOWFIND <= (cmdID as u32)
                    && (cmdID as u32) <= OLECMDID_SHOWPRINT)
            {
                cmdFlags |= CMD_FLAG_SUPPORTED_BASIC;
                break;
            }
            iCmd = iCmd.wrapping_add(1);
        }
    }

    if cmdFlags == 0 || (cmdFlags & CMD_FLAG_NOT_SUPPORTED) != 0 {
        return OLECMDERR_E_NOTSUPPORTED; // Not supported
    }

    return MAKE_HRESULT!(SEVERITY_SUCCESS, FACILITY_NULL, cmdFlags);
}

#[no_mangle]
pub unsafe extern "system" fn SHIsChildOrSelf(hParent: HWND, hChild: HWND) -> BOOL {
    if hParent.is_null() || hChild.is_null() {
        return (TRUE) as BOOL;
    } else if hParent == hChild {
        return (FALSE) as BOOL;
    }
    return (IsChild(hParent, hChild) == 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FDSA_Initialize(block_size: DWORD, mut inc: DWORD, info: *mut FDSA_info, mem: *mut core::ffi::c_void, init_blocks: DWORD) -> BOOL {

    if inc == 0 {
        inc = 1;
    }

    if !mem.is_null() {
        memset(mem, 0, block_size.wrapping_mul(init_blocks) as usize);
    }

    (*info).num_items = 0;
    (*info).inc = inc as BYTE;
    (*info).mem = mem;
    (*info).blocks_alloced = init_blocks;
    (*info).block_size = block_size as BYTE;
    (*info).flags = 0;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FDSA_Destroy(info: *mut FDSA_info) -> BOOL {
    if (*info).flags as i32 & (FDSA_FLAG_INTERNAL_ALLOC as i32) != 0 {
        HeapFree(GetProcessHeap(), 0, (*info).mem);
        return (FALSE) as BOOL;
    }

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn FDSA_InsertItem(info: *mut FDSA_info, mut where_0: DWORD, block: *const core::ffi::c_void) -> DWORD {
    if where_0 > (*info).num_items {
        where_0 = (*info).num_items;
    }

    if (*info).num_items >= (*info).blocks_alloced {
        let size: DWORD = (*info).blocks_alloced.wrapping_add((*info).inc as DWORD).wrapping_mul((*info).block_size as DWORD);
        if (*info).flags as DWORD & 0x1 != 0 {
            (*info).mem = HeapReAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                (*info).mem,
                size as SIZE_T,
            ) as *mut core::ffi::c_void;
        } else {
            let old_mem: *mut core::ffi::c_void = (*info).mem;
            (*info).mem = HeapAlloc(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                size as SIZE_T,
            ) as *mut core::ffi::c_void;
            memcpy(
                (*info).mem,
                old_mem,
                (*info).blocks_alloced.wrapping_mul((*info).block_size as DWORD) as size_t,
            );
        }
        (*info).blocks_alloced = (*info).blocks_alloced.wrapping_add((*info).inc as DWORD);
        (*info).flags = ((*info).flags as DWORD | 0x1) as BYTE;
    }

    if where_0 < (*info).num_items {
        memmove(
            ((*info).mem as *mut core::ffi::c_char).offset(
                where_0.wrapping_add(1).wrapping_mul((*info).block_size as DWORD) as isize,
            ) as *mut core::ffi::c_void,
            ((*info).mem as *mut core::ffi::c_char).offset(
                where_0.wrapping_mul((*info).block_size as DWORD) as isize,
            ) as *const core::ffi::c_void,
            (*info).num_items.wrapping_sub(where_0).wrapping_mul((*info).block_size as DWORD) as size_t,
        );
    }
    memcpy(
        ((*info).mem as *mut core::ffi::c_char).offset(
            where_0.wrapping_mul((*info).block_size as DWORD) as isize,
        ) as *mut core::ffi::c_void,
        block,
        (*info).block_size as size_t,
    );

    (*info).num_items = (*info).num_items.wrapping_add(1);
    return where_0;
}

#[no_mangle]
pub unsafe extern "system" fn FDSA_DeleteItem(info: *mut FDSA_info, where_0: DWORD) -> BOOL {
    if where_0 >= (*info).num_items {
        return (FALSE) as BOOL;
    }

    if where_0 < (*info).num_items.wrapping_sub(1 as DWORD) {
        memmove(
            ((*info).mem as *mut ::core::ffi::c_char)
                .offset(where_0.wrapping_mul((*info).block_size as DWORD) as isize)
                as *mut ::core::ffi::c_void,
            ((*info).mem as *mut ::core::ffi::c_char).offset(
                where_0
                    .wrapping_add(1 as DWORD)
                    .wrapping_mul((*info).block_size as DWORD) as isize,
            ) as *const ::core::ffi::c_void,
            (*info)
                .num_items
                .wrapping_sub(where_0)
                .wrapping_sub(1 as DWORD)
                .wrapping_mul((*info).block_size as DWORD) as size_t,
        );
    }

    memset(
        ((*info).mem as *mut ::core::ffi::c_char).offset(
            (*info)
                .num_items
                .wrapping_sub(1 as DWORD)
                .wrapping_mul((*info).block_size as DWORD) as isize,
        ) as *mut ::core::ffi::c_void,
        0 as ::core::ffi::c_int,
        (*info).block_size as size_t,
    );
    (*info).num_items = (*info).num_items.wrapping_sub(1);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn QISearch(
    mut base: *mut ::core::ffi::c_void,
    mut table: *const QITAB,
    riid: *const IID,
    mut ppv: *mut *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut ret: HRESULT = 0;
    let mut a_vtbl: *mut IUnknown = ::core::ptr::null_mut::<IUnknown>();
    let mut xmove: *const QITAB = ::core::ptr::null::<QITAB>();
    WINE_TRACE!("(%p %p %s %p)\n", base, table, debugstr_guid(riid as *const _GUID), ppv);
    if !ppv.is_null() {
        xmove = table;
        while !(*xmove).piid.is_null() {
            WINE_TRACE!("trying (offset %d) %s\n", (*xmove).dwOffset, debugstr_guid((*xmove).piid as *const _GUID));
            if IsEqualIID!(riid, (*xmove).piid) != 0 {
                a_vtbl = (base as LPBYTE).offset((*xmove).dwOffset as isize) as *mut IUnknown;
                WINE_TRACE!("matched, returning (%p)\n", a_vtbl);
                *ppv = a_vtbl as *mut ::core::ffi::c_void;
                IUnknown_AddRef(a_vtbl);
                return S_OK;
            }
            xmove = xmove.offset(1);
        }
        if IsEqualIID!(riid, core::ptr::addr_of!(IID_IUnknown)) != 0 {
            a_vtbl = (base as LPBYTE).offset((*table).dwOffset as isize) as *mut IUnknown;
            WINE_TRACE!("returning first for IUnknown (%p)\n", a_vtbl);
            *ppv = a_vtbl as *mut ::core::ffi::c_void;
            IUnknown_AddRef(a_vtbl);
            return S_OK;
        }
        *ppv = core::ptr::null_mut();
        ret = E_NOINTERFACE;
    } else {
        ret = E_POINTER;
    }
    WINE_TRACE!("-- 0x%08x\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHSetDefaultDialogFont(hWnd: HWND, id: INT) {
    let mut hOldFont: HFONT;
    let mut hNewFont: HFONT;
    let mut lfOldFont: LOGFONTW = core::mem::zeroed();
    let mut lfNewFont: LOGFONTW = core::mem::zeroed();
    let mut hwndItem: HWND;

    hOldFont = SendMessageW(hWnd, WM_GETFONT, 0, 0) as HFONT;
    GetObjectW(hOldFont as HANDLE, core::mem::size_of::<LOGFONTW>() as INT, core::ptr::addr_of_mut!(lfOldFont) as LPVOID);
    SystemParametersInfoW(SPI_GETICONTITLELOGFONT, core::mem::size_of::<LOGFONTW>() as UINT, core::ptr::addr_of_mut!(lfNewFont) as PVOID, 0);

    if lfOldFont.lfCharSet == lfNewFont.lfCharSet {
        return;
    }

    hNewFont = GetPropW(hWnd, RUSTCONV_WSTR_7E73C65754ED.as_ptr()) as HFONT;
    if hNewFont.is_null() {
        /* Create the icon-title font of the same height */
        lfNewFont.lfHeight = lfOldFont.lfHeight;
        hNewFont = CreateFontIndirectW(core::ptr::addr_of_mut!(lfNewFont)) as HFONT;

        /* If creating the font is failed, then keep the old font */
        if hNewFont.is_null() {
            hNewFont = hOldFont;
        }

        /* Set "PropDlgFont" property if the font is changed */
        if hOldFont != hNewFont {
            SetPropW(hWnd, RUSTCONV_WSTR_7E73C65754ED.as_ptr(), hNewFont as HANDLE);
        }
    }

    hwndItem = GetDlgItem(hWnd, id) as HWND;
    SendMessageW(hwndItem, WM_SETFONT, hNewFont as WPARAM, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SHRemoveDefaultDialogFont(hWnd: HWND) -> HANDLE {
    let mut hProp: HANDLE;

    hProp = GetPropA(hWnd, b"PropDlgFont\0" as *const u8 as LPCSTR);

    if !hProp.is_null() {
        DeleteObject(hProp as HGDIOBJ);
        hProp = RemovePropA(hWnd, b"PropDlgFont\0" as *const u8 as LPCSTR);
    }
    return hProp;
}

#[no_mangle]
pub unsafe extern "system" fn SHPinDllOfCLSID(refiid: *const IID) -> HMODULE {
    let mut newkey: HKEY = core::ptr::null_mut();
    let mut type_0: DWORD = 0;
    let mut count: DWORD = 0;
    let mut value: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut string: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    strcpy(
        core::ptr::addr_of_mut!(string) as *mut CHAR,
        b"CLSID\\\0".as_ptr() as *const CHAR,
    );
    SHStringFromGUIDA(
        refiid as *const GUID,
        (core::ptr::addr_of_mut!(string) as *mut CHAR).offset(6 as isize),
        ((core::mem::size_of::<[CHAR; (MAX_PATH) as _]>() / core::mem::size_of::<CHAR>()) as usize)
            .wrapping_sub(6 as usize) as INT,
    );
    strcat(
        core::ptr::addr_of_mut!(string) as *mut CHAR,
        b"\\InProcServer32\0".as_ptr() as *const CHAR,
    );

    count = MAX_PATH;
    RegOpenKeyExA(
        HKEY_CLASSES_ROOT,
        core::ptr::addr_of_mut!(string) as *mut CHAR as LPCSTR,
        0,
        1,
        core::ptr::addr_of_mut!(newkey),
    );
    RegQueryValueExA(
        newkey,
        core::ptr::null(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(type_0),
        core::ptr::addr_of_mut!(value) as *mut CHAR as LPBYTE,
        core::ptr::addr_of_mut!(count),
    );
    RegCloseKey(newkey);
    return LoadLibraryExA(
        core::ptr::addr_of_mut!(value) as *mut CHAR as LPCSTR,
        core::ptr::null_mut(),
        0,
    ) as HMODULE;
}

#[no_mangle]
pub unsafe extern "system" fn SHRegisterClassW(lpWndClass: *mut WNDCLASSW) -> DWORD {
    let mut WndClass: WNDCLASSW = core::mem::zeroed();
    WINE_TRACE!("(%p %s)\n", (*lpWndClass).hInstance, debugstr_w((*lpWndClass).lpszClassName as *const WCHAR));
    if GetClassInfoW((*lpWndClass).hInstance, (*lpWndClass).lpszClassName, &raw mut WndClass) != 0
    {
        return TRUE;
    }
    return RegisterClassW(lpWndClass) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHUnregisterClassesA(hInst: HINSTANCE, mut lppClasses: *mut LPCSTR, mut iCount: INT) {
    let mut WndClass: WNDCLASSA = core::mem::zeroed();

    while iCount > 0 {
        if GetClassInfoA(hInst, *lppClasses, core::ptr::addr_of_mut!(WndClass)) != 0 {
            UnregisterClassA(*lppClasses, hInst);
        }
        lppClasses = lppClasses.offset(1);
        iCount -= 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHUnregisterClassesW(mut hInst: HINSTANCE, mut lppClasses: *mut LPCWSTR, mut iCount: INT) {
    let mut WndClass: WNDCLASSW = core::mem::zeroed();

    while iCount > 0 {
        if GetClassInfoW(hInst, *lppClasses, core::ptr::addr_of_mut!(WndClass)) != 0 {
            UnregisterClassW(*lppClasses, hInst);
        }
        lppClasses = lppClasses.offset(1);
        iCount -= 1;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHDefWindowProc(hWnd: HWND, uMessage: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    if IsWindowUnicode(hWnd) != 0 {
        return DefWindowProcW(hWnd, uMessage, wParam, lParam);
    }
    return DefWindowProcA(hWnd, uMessage, wParam, lParam);
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_GetSite(lpUnknown: LPUNKNOWN, iid: REFIID, lppSite: *mut PVOID) -> HRESULT {
    let mut hRet: HRESULT = E_INVALIDARG;
    let mut lpSite: LPOBJECTWITHSITE = core::ptr::null_mut();

    if !lpUnknown.is_null() && !iid.is_null() && !lppSite.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            core::ptr::addr_of!(IID_IObjectWithSite),
            &raw mut lpSite as *mut *mut core::ffi::c_void,
        );
        if (hRet >= 0) && !lpSite.is_null() {
            hRet = IObjectWithSite_GetSite(lpSite, iid, lppSite);
            IObjectWithSite_Release(lpSite);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateWorkerWindowA(
    wndProc: WNDPROC,
    hWndParent: HWND,
    dwExStyle: DWORD,
    dwStyle: DWORD,
    hMenu: HMENU,
    wnd_extra: LONG_PTR,
) -> HWND {
    static mut szClass: [u8; 8] = *b"WorkerA\0";
    let mut wc: WNDCLASSA = core::mem::zeroed();
    let mut hWnd: HWND = core::ptr::null_mut();
    WINE_TRACE!("(%p, %p, 0x%08x, 0x%08x, %p, 0x%08lx)\n", wndProc, hWndParent, dwExStyle, dwStyle, hMenu, wnd_extra);

    /* Create Window class */
    wc.style = 0;
    wc.lpfnWndProc = Some(
        DefWindowProcA as unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT,
    ) as WNDPROC;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = core::mem::size_of::<LONG_PTR>() as ::core::ffi::c_int;
    wc.hInstance = shlwapi_hInstance;
    wc.hIcon = core::ptr::null_mut();
    wc.hCursor = LoadCursorA(
        core::ptr::null_mut(),
        IDC_ARROW as LPSTR as LPCSTR,
    );
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = core::ptr::addr_of!(szClass) as LPCSTR;

    SHRegisterClassA(core::ptr::addr_of_mut!(wc));

    hWnd = CreateWindowExA(
        dwExStyle,
        core::ptr::addr_of!(szClass) as LPCSTR,
        core::ptr::null(),
        dwStyle,
        0,
        0,
        0,
        0,
        hWndParent,
        hMenu,
        shlwapi_hInstance,
        core::ptr::null_mut(),
    );
    if !hWnd.is_null() {
        SetWindowLongPtrW(hWnd, 0, wnd_extra);
        if wndProc.is_some() {
            SetWindowLongPtrA(
                hWnd,
                GWLP_WNDPROC,
                (::core::mem::transmute::<WNDPROC, LONG_PTR>(wndProc)) as _,
            );
        }
    }

    return hWnd;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetRestriction(
    mut lpSubKey: LPCWSTR,
    mut lpSubName: LPCWSTR,
    mut lpValue: LPCWSTR,
) -> DWORD {
    let mut szPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwSize: DWORD = 0;
    let mut dwValue: DWORD = 0;
    WINE_TRACE!("(%s, %s, %s)\n", debugstr_w(lpSubKey as *const WCHAR), debugstr_w(lpSubName as *const WCHAR), debugstr_w(lpValue as *const WCHAR));
    if lpSubKey.is_null() {
        lpSubKey = RUSTCONV_WSTR_D4E75AE8C989.as_ptr();
    }
    PathCombineW(
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPWSTR,
        lpSubKey,
        lpSubName,
    );
    dwSize = ::core::mem::size_of::<DWORD>() as DWORD;
    if SHGetValueW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
        lpValue,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(dwValue) as LPVOID,
        core::ptr::addr_of_mut!(dwSize),
    ) == ERROR_SUCCESS
    {
        return dwValue;
    }
    dwSize = ::core::mem::size_of::<DWORD>() as DWORD;
    SHGetValueW(
        HKEY_CURRENT_USER,
        core::ptr::addr_of_mut!(szPath) as *mut WCHAR as LPCWSTR,
        lpValue,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(dwValue) as LPVOID,
        core::ptr::addr_of_mut!(dwSize),
    );
    return dwValue;
}

#[no_mangle]
pub unsafe extern "system" fn SHRestrictionLookup(
    mut policy: DWORD,
    mut initial: LPCWSTR,
    mut polTable: *const POLICYDATA,
    mut polArr: LPDWORD,
) -> DWORD {
    while (*polTable).policy != 0 {
        if policy == (*polTable).policy {
            /* we have a known policy */

            /* check if this policy has been cached */
            if *polArr == SHELL_NO_POLICY {
                *polArr = SHGetRestriction(initial, (*polTable).appstr, (*polTable).keystr);
            }
            return *polArr;
        }
        polTable = polTable.add(1);
        polArr = polArr.add(1);
    }
    /* we don't know this policy, return 0 */
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SHWeakQueryInterface(
    pUnk: *mut IUnknown,
    pInner: *mut IUnknown,
    riid: *mut IID,
    ppv: *mut LPVOID,
) -> HRESULT {
    let mut hret: HRESULT = E_NOINTERFACE;

    *ppv = core::ptr::null_mut();
    if !pUnk.is_null() && !pInner.is_null() {
        hret = IUnknown_QueryInterface(pInner, riid, ppv);
        if (hret) >= 0 {
            IUnknown_Release(pUnk);
        }
    }
    return hret;
}

#[no_mangle]
pub unsafe extern "system" fn SHWeakReleaseInterface(lpDest: *mut IUnknown, lppUnknown: *mut *mut IUnknown) {
    if !(*lppUnknown).is_null() {
        /* Copy Reference*/
        IUnknown_AddRef(lpDest);
        IUnknown_AtomicRelease(lppUnknown); /* Release existing interface */
    }
}

#[no_mangle]
pub unsafe extern "system" fn GUIDFromStringA(idstr: LPCSTR, id: *mut CLSID) -> BOOL {
    let mut wClsid: [WCHAR; 40] = [0; 40];
    MultiByteToWideChar(CP_ACP as UINT, 0 as DWORD, idstr, -1 as core::ffi::c_int, core::ptr::addr_of_mut!(wClsid) as LPWSTR, (core::mem::size_of::<[WCHAR; 40]>() / core::mem::size_of::<WCHAR>()) as core::ffi::c_int);
    return (CLSIDFromString(core::ptr::addr_of_mut!(wClsid) as *mut WCHAR as LPCOLESTR, id as LPCLSID) >= 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn GUIDFromStringW(idstr: LPCWSTR, id: *mut CLSID) -> BOOL {
    return ((CLSIDFromString(idstr as LPCOLESTR, id)) >= 0) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn WhichPlatform() -> UINT {
    static mut szIntegratedBrowser: [u8; 18] = *b"IntegratedBrowser\0";
    static mut dwState: DWORD = 0 as DWORD;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwRet: DWORD = 0;
    let mut dwData: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut hshell32: HMODULE = core::ptr::null_mut();

    if dwState != 0 {
        return dwState as UINT;
    }

    /* If shell32 exports DllGetVersion(), the browser is integrated */
    dwState = 1 as DWORD;
    hshell32 = LoadLibraryA(b"shell32.dll\0" as *const u8 as LPCSTR) as HMODULE;
    if !hshell32.is_null() {
        let mut pDllGetVersion: FARPROC = None;
        pDllGetVersion = GetProcAddress(
            hshell32 as HINSTANCE,
            b"DllGetVersion\0" as *const u8 as LPCSTR,
        );
        dwState = (if pDllGetVersion.is_some() {
            2 as DWORD
        } else {
            1 as DWORD
        }) as DWORD;
        FreeLibrary(hshell32);
    }

    /* Set or delete the key accordingly */
    dwRet = RegOpenKeyExA(
        HKEY_LOCAL_MACHINE,
        b"Software\\Microsoft\\Internet Explorer\0" as *const u8 as LPCSTR,
        0 as DWORD,
        KEY_ALL_ACCESS as REGSAM,
        &raw mut hKey,
    ) as DWORD;
    if dwRet == 0 {
        dwRet = RegQueryValueExA(
            hKey,
            core::ptr::addr_of!(szIntegratedBrowser) as LPCSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &raw mut dwData as LPBYTE,
            &raw mut dwSize,
        ) as DWORD;

        if dwRet == 0 && dwState == 1 as DWORD {
            /* Value exists but browser is not integrated */
            RegDeleteValueA(hKey, core::ptr::addr_of!(szIntegratedBrowser) as LPCSTR);
        } else if dwRet != 0 && dwState == 2 as DWORD {
            /* Browser is integrated but value does not exist */
            dwData = TRUE as DWORD;
            RegSetValueExA(
                hKey,
                core::ptr::addr_of!(szIntegratedBrowser) as LPCSTR,
                0 as DWORD,
                REG_DWORD as DWORD,
                &raw mut dwData as LPBYTE as *const BYTE,
                core::mem::size_of::<DWORD>() as DWORD,
            );
        }
        RegCloseKey(hKey);
    }
    return dwState as UINT;
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateWorkerWindowW(
    wndProc: WNDPROC,
    hWndParent: HWND,
    dwExStyle: DWORD,
    dwStyle: DWORD,
    hMenu: HMENU,
    wnd_extra: LONG_PTR,
) -> HWND {
    static szClass: [WCHAR; 8] = [0x0057u16, 0x006F, 0x0072, 0x006B, 0x0065, 0x0072, 0x0057, 0x0000];
    let mut wc: WNDCLASSW = core::mem::zeroed();
    let mut hWnd: HWND;

    /* If our OS is natively ANSI, use the ANSI version */
    if (GetVersion() & 0x80000000) != 0
    /* not NT */
    {
        return SHCreateWorkerWindowA(wndProc, hWndParent, dwExStyle, dwStyle, hMenu, wnd_extra);
    }

    /* Create Window class */
    wc.style = 0 as UINT;
    wc.lpfnWndProc = Some(DefWindowProcW);
    wc.cbClsExtra = 0 as i32;
    wc.cbWndExtra = core::mem::size_of::<LONG_PTR>() as i32;
    wc.hInstance = shlwapi_hInstance;
    wc.hIcon = core::ptr::null_mut();
    wc.hCursor = LoadCursorW(
        core::ptr::null_mut(),
        IDC_ARROW as LPWSTR as LPCWSTR,
    );
    wc.hbrBackground = (COLOR_BTNFACE + 1) as HBRUSH;
    wc.lpszMenuName = core::ptr::null();
    wc.lpszClassName = core::ptr::addr_of!(szClass) as *const WCHAR as LPCWSTR;

    SHRegisterClassW(core::ptr::addr_of_mut!(wc));

    hWnd = CreateWindowExW(
        dwExStyle,
        core::ptr::addr_of!(szClass) as LPCWSTR,
        0 as LPCWSTR,
        dwStyle,
        0,
        0,
        0,
        0,
        hWndParent,
        hMenu,
        shlwapi_hInstance,
        0 as LPVOID,
    );
    if !hWnd.is_null() {
        SetWindowLongPtrW(hWnd, 0, wnd_extra);
        if wndProc.is_some() {
            SetWindowLongPtrW(
                hWnd,
                GWLP_WNDPROC,
                core::mem::transmute::<WNDPROC, LONG_PTR>(wndProc),
            );
        }
    }

    return hWnd;
}

#[no_mangle]
pub unsafe extern "system" fn SHInvokeDefaultCommand(
    mut hWnd: HWND,
    mut lpFolder: *mut IShellFolder,
    mut lpApidl: LPCITEMIDLIST,
) -> HRESULT {
    WINE_TRACE!("%p %p %p\n", hWnd, lpFolder, lpApidl);
    return SHInvokeCommand(hWnd, lpFolder, lpApidl, core::ptr::null::<CHAR>());
}

#[no_mangle]
pub unsafe extern "system" fn SHPackDispParamsV(
    mut params: *mut DISPPARAMS,
    mut args: *mut VARIANTARG,
    mut cnt: UINT,
    valist_in: *mut ::core::ffi::c_void,
) -> HRESULT {
    // `va_list` by value (`char *` on every Windows target): VaArgs reads through a pointer
    // to this frame's copy, as `va_arg` does.
    let mut __va = valist_in;
    let mut valist = varargs_rust::VaArgs::from_ptr(core::ptr::addr_of_mut!(__va) as *mut ::core::ffi::c_void);
    let mut iter: *mut VARIANTARG = core::ptr::null_mut::<VARIANTARG>();

    (*params).rgvarg = args;
    (*params).rgdispidNamedArgs = core::ptr::null_mut::<DISPID>();
    (*params).cArgs = cnt;
    (*params).cNamedArgs = 0 as UINT;

    iter = args.offset(cnt as isize);
    loop {
        let fresh15 = iter;
        iter = iter.offset(-1);
        if !(fresh15 > args) {
            break;
        }

        (*iter).n1.n2.vt = valist.arg::<VARENUM>() as VARTYPE;

        if (*iter).n1.n2.vt as ::core::ffi::c_int & VT_BYREF as ::core::ffi::c_int != 0 {
            (*iter).n1.n2.n3.byref = valist.arg::<LPVOID>() as PVOID;
        } else {
            match ((*iter).n1.n2.vt as ::core::ffi::c_int) as u32 {
                VT_I4 => {
                    (*iter).n1.n2.n3.lVal = valist.arg::<LONG>();
                }
                VT_BSTR => {
                    (*iter).n1.n2.n3.bstrVal = valist.arg::<BSTR>();
                }
                VT_DISPATCH => {
                    (*iter).n1.n2.n3.pdispVal = valist.arg::<*mut IDispatch>();
                }
                VT_BOOL => {
                    (*iter).n1.n2.n3.boolVal = valist.arg::<::core::ffi::c_int>() as VARIANT_BOOL;
                }
                VT_UNKNOWN => {
                    (*iter).n1.n2.n3.punkVal = valist.arg::<*mut IUnknown>();
                }
                _ => {
                    (*iter).n1.n2.vt = VT_I4 as VARTYPE;
                    (*iter).n1.n2.n3.lVal = valist.arg::<LONG>();
                }
            }
        }
    }

    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "C" fn SHPackDispParams_v(
    params: *mut DISPPARAMS,
    args: *mut VARIANTARG,
    cnt: UINT,
    ap_list: *mut ::core::ffi::c_void,
) -> HRESULT {
    // The shim already va_start'ed; hand on the started list by value.
    SHPackDispParamsV(params, args, cnt, *(ap_list as *mut *mut ::core::ffi::c_void))
}



unsafe extern "C" fn SHLWAPI_InvokeByIID(
    mut iCP: *mut IConnectionPoint,
    iid: *const IID,
    mut dispId: DISPID,
    mut dispParams: *mut DISPPARAMS,
) -> HRESULT {
    let mut enumerator: *mut IEnumConnections = core::ptr::null_mut::<IEnumConnections>();
    let mut rgcd: CONNECTDATA = CONNECTDATA {
        pUnk: core::ptr::null_mut::<IUnknown>(),
        dwCookie: 0,
    };
    static mut empty: DISPPARAMS = DISPPARAMS {
        rgvarg: core::ptr::null::<VARIANTARG>() as *mut VARIANTARG,
        rgdispidNamedArgs: core::ptr::null::<DISPID>() as *mut DISPID,
        cArgs: 0 as UINT,
        cNamedArgs: 0 as UINT,
    };
    let mut params: *mut DISPPARAMS = dispParams;

    let mut result: HRESULT = IConnectionPoint_EnumConnections(iCP, &raw mut enumerator);
    if result < 0 as HRESULT {
        return result;
    }

    /* Invoke is never happening with an NULL dispParams */
    if params.is_null() {
        params = &raw mut empty;
    }

    while IEnumConnections_Next(
        enumerator,
        1 as ULONG,
        &raw mut rgcd,
        core::ptr::null_mut::<ULONG>(),
    ) == S_OK as HRESULT
    {
        let mut dispIface: *mut IDispatch = core::ptr::null_mut::<IDispatch>();
        if (!iid.is_null()
            && (IUnknown_QueryInterface(
                rgcd.pUnk,
                iid,
                &raw mut dispIface as *mut LPVOID,
            ) >= 0 as HRESULT))
            || (IUnknown_QueryInterface(
                rgcd.pUnk,
                &raw const IID_IDispatch,
                &raw mut dispIface as *mut LPVOID,
            ) >= 0 as HRESULT)
        {
            IDispatch_Invoke(
                dispIface,
                dispId,
                &raw const IID_NULL,
                0 as LCID,
                DISPATCH_METHOD as WORD,
                params,
                core::ptr::null_mut::<VARIANT>(),
                core::ptr::null_mut::<EXCEPINFO>(),
                core::ptr::null_mut::<UINT>(),
            );
            IDispatch_Release(dispIface);
        }
        IUnknown_Release(rgcd.pUnk);
    }

    IEnumConnections_Release(enumerator);

    return S_OK as HRESULT;
}

#[no_mangle]
pub unsafe extern "system" fn IConnectionPoint_InvokeWithCancel(
    iCP: *mut IConnectionPoint,
    dispId: DISPID,
    dispParams: *mut DISPPARAMS,
    unknown1: DWORD,
    unknown2: DWORD,
) -> HRESULT {
    let mut iid: IID = core::mem::zeroed();
    let mut result: HRESULT;

    result = IConnectionPoint_GetConnectionInterface(iCP, core::ptr::addr_of_mut!(iid));
    if result >= 0 {
        result = SHLWAPI_InvokeByIID(iCP, core::ptr::addr_of_mut!(iid), dispId, dispParams);
    } else {
        result = SHLWAPI_InvokeByIID(iCP, core::ptr::null(), dispId, dispParams);
    }

    return result;
}

#[no_mangle]
pub unsafe extern "system" fn IConnectionPoint_SimpleInvoke(
    iCP: *mut IConnectionPoint,
    dispId: DISPID,
    dispParams: *mut DISPPARAMS,
) -> HRESULT {
    let mut iid: IID = core::mem::zeroed();
    let mut result: HRESULT;

    result = IConnectionPoint_GetConnectionInterface(iCP, core::ptr::addr_of_mut!(iid));
    if result >= 0 {
        result = SHLWAPI_InvokeByIID(iCP, core::ptr::addr_of_mut!(iid), dispId, dispParams);
    } else {
        result = SHLWAPI_InvokeByIID(iCP, core::ptr::null_mut::<IID>(), dispId, dispParams);
    }

    return result;
}

#[no_mangle]
pub unsafe extern "system" fn IConnectionPoint_OnChanged(
    lpCP: *mut IConnectionPoint,
    dispID: DISPID,
) -> HRESULT {
    let mut lpEnum: *mut IEnumConnections = core::ptr::null_mut::<IEnumConnections>();
    let mut hRet: HRESULT = E_NOINTERFACE;

    /* Get an enumerator for the connections */
    if !lpCP.is_null() {
        hRet = IConnectionPoint_EnumConnections(lpCP, core::ptr::addr_of_mut!(lpEnum));
    }

    if hRet >= 0 {
        let mut lpSink: *mut IPropertyNotifySink = core::ptr::null_mut::<IPropertyNotifySink>();
        let mut connData: CONNECTDATA = CONNECTDATA {
            pUnk: core::ptr::null_mut::<IUnknown>(),
            dwCookie: 0,
        };
        let mut ulFetched: ULONG = 0;

        /* Call OnChanged() for every notify sink in the connection point */
        while IEnumConnections_Next(
            lpEnum,
            1,
            core::ptr::addr_of_mut!(connData),
            core::ptr::addr_of_mut!(ulFetched),
        ) == S_OK
        {
            if IUnknown_QueryInterface(
                connData.pUnk,
                core::ptr::addr_of!(IID_IPropertyNotifySink),
                core::ptr::addr_of_mut!(lpSink) as *mut *mut core::ffi::c_void,
            ) >= 0
                && !lpSink.is_null()
            {
                IPropertyNotifySink_OnChanged(lpSink, dispID);
                IPropertyNotifySink_Release(lpSink);
            }
            IUnknown_Release(connData.pUnk);
        }

        IEnumConnections_Release(lpEnum);
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "C" fn IUnknown_CPContainerInvokeParam_v(
    mut container: *mut IUnknown,
    riid: *const IID,
    mut dispId: DISPID,
    mut buffer: *mut VARIANTARG,
    mut cParams: DWORD,
    ap_list: *mut ::core::ffi::c_void,
) -> HRESULT {
    let mut result: HRESULT;
    let mut iCP: *mut IConnectionPoint = core::ptr::null_mut::<IConnectionPoint>();
    let mut iCPC: *mut IConnectionPointContainer = core::ptr::null_mut::<IConnectionPointContainer>();
    let mut dispParams: DISPPARAMS = DISPPARAMS {
        rgvarg: buffer,
        rgdispidNamedArgs: core::ptr::null_mut::<DISPID>(),
        cArgs: cParams as UINT,
        cNamedArgs: 0,
    };
    let mut valist: va_list = *(ap_list as *mut va_list);

    if container.is_null() {
        return E_NOINTERFACE;
    }

    result = IUnknown_QueryInterface(
        container,
        &raw const IID_IConnectionPointContainer,
        &raw mut iCPC as *mut *mut core::ffi::c_void,
    );
    if result < 0 {
        return result;
    }

    result = IConnectionPointContainer_FindConnectionPoint(iCPC, riid, &raw mut iCP);
    IConnectionPointContainer_Release(iCPC);
    if result < 0 {
        return result;
    }


    SHPackDispParamsV(
        &raw mut dispParams,
        buffer,
        cParams as UINT,
        (valist).cast(),
    );

    result = SHLWAPI_InvokeByIID(iCP, riid, dispId, &raw mut dispParams);
    IConnectionPoint_Release(iCP);

    return result;
}



#[no_mangle]
pub unsafe extern "system" fn IUnknown_CPContainerOnChanged(lpUnknown: *mut IUnknown, dispID: DISPID) -> HRESULT {
    let mut lpCPC: *mut IConnectionPointContainer = core::ptr::null_mut();
    let mut hRet: HRESULT = E_NOINTERFACE;

    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            &raw const IID_IConnectionPointContainer,
            &raw mut lpCPC as *mut *mut core::ffi::c_void,
        );
    }

    if hRet >= 0 {
        let mut lpCP: *mut IConnectionPoint = core::ptr::null_mut();

        hRet = IConnectionPointContainer_FindConnectionPoint(
            lpCPC,
            &raw const IID_IPropertyNotifySink,
            &raw mut lpCP,
        );
        IConnectionPointContainer_Release(lpCPC);

        hRet = IConnectionPoint_OnChanged(lpCP, dispID);
        IConnectionPoint_Release(lpCP);
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn PlaySoundWrapW(
    pszSound: LPCWSTR,
    hmod: HMODULE,
    fdwSound: DWORD,
) -> BOOL {
    PlaySoundW(pszSound, hmod, fdwSound)
}

#[no_mangle]
pub unsafe extern "system" fn SHGetFileInfoWrapW(
    path: LPCWSTR,
    dwFileAttributes: DWORD,
    psfi: *mut SHFILEINFOW,
    sizeofpsfi: UINT,
    flags: UINT,
) -> DWORD {
    (SHGetFileInfoW(path, dwFileAttributes, psfi, sizeofpsfi, flags)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn DragQueryFileWrapW(
    hDrop: HDROP,
    lFile: UINT,
    lpszFile: LPWSTR,
    lLength: UINT,
) -> UINT {
    DragQueryFileW(hDrop, lFile, lpszFile, lLength)
}

#[no_mangle]
pub unsafe extern "system" fn SHBrowseForFolderWrapW(lpBi: LPBROWSEINFOW) -> LPITEMIDLIST {
    SHBrowseForFolderW(lpBi)
}

#[no_mangle]
pub unsafe extern "system" fn SHGetPathFromIDListWrapW(
    pidl: LPCITEMIDLIST,
    pszPath: LPWSTR,
) -> BOOL {
    SHGetPathFromIDListW(pidl, pszPath)
}

#[no_mangle]
pub unsafe extern "system" fn ShellExecuteExWrapW(lpExecInfo: LPSHELLEXECUTEINFOW) -> BOOL {
    ShellExecuteExW(lpExecInfo)
}

#[no_mangle]
pub unsafe extern "system" fn SHFileOperationWrapW(lpFileOp: LPSHFILEOPSTRUCTW) -> INT {
    SHFileOperationW(lpFileOp)
}

#[no_mangle]
pub unsafe extern "system" fn SHInterlockedCompareExchange(
    dest: *mut PVOID,
    xchg: PVOID,
    compare: PVOID,
) -> PVOID {
    InterlockedCompareExchangePointer(dest, xchg, compare)
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeWrapW(
    filename: LPCWSTR,
    handle: LPDWORD,
) -> DWORD {
    GetFileVersionInfoSizeW(filename, handle)
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoWrapW(
    filename: LPCWSTR,
    handle: DWORD,
    datasize: DWORD,
    data: LPVOID,
) -> BOOL {
    GetFileVersionInfoW(filename, handle, datasize, data)
}

#[no_mangle]
pub unsafe extern "system" fn VerQueryValueWrapW(
    pBlock: LPVOID,
    lpSubBlock: LPCWSTR,
    lplpBuffer: *mut LPVOID,
    puLen: *mut UINT,
) -> WORD {
    (VerQueryValueW(pBlock, lpSubBlock, lplpBuffer, puLen)) as u16
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_EnableModeless(mut lpUnknown: *mut IUnknown, mut bModeless: BOOL) -> HRESULT {
    let mut lpObj: *mut IUnknown = core::ptr::null_mut::<IUnknown>();
    let mut hRet: HRESULT = 0;
    WINE_TRACE!("(%p,%d)\n", lpUnknown, bModeless);
    if lpUnknown.is_null() {
        return E_FAIL;
    }
    hRet = (*(*lpUnknown).lpVtbl)
        .QueryInterface
        .unwrap_unchecked()(
        lpUnknown,
        &raw const IID_IOleInPlaceActiveObject,
        &raw mut lpObj as *mut *mut ::core::ffi::c_void,
    );
    if hRet >= 0 as HRESULT {
        IOleInPlaceActiveObject_EnableModeless(lpObj as *mut IOleInPlaceActiveObject, bModeless);
    } else {
        hRet = (*(*lpUnknown).lpVtbl)
            .QueryInterface
            .unwrap_unchecked()(
            lpUnknown,
            &raw const IID_IOleInPlaceFrame,
            &raw mut lpObj as *mut *mut ::core::ffi::c_void,
        );
        if hRet >= 0 as HRESULT {
            IOleInPlaceFrame_EnableModeless(lpObj as *mut IOleInPlaceFrame, bModeless);
        } else {
            hRet = (*(*lpUnknown).lpVtbl)
                .QueryInterface
                .unwrap_unchecked()(
                lpUnknown,
                &raw const IID_IShellBrowser,
                &raw mut lpObj as *mut *mut ::core::ffi::c_void,
            );
            if hRet >= 0 as HRESULT {
                IShellBrowser_EnableModelessSB(lpObj as *mut IShellBrowser, bModeless);
            } else {
                hRet = (*(*lpUnknown).lpVtbl)
                    .QueryInterface
                    .unwrap_unchecked()(
                    lpUnknown,
                    &raw const IID_IInternetSecurityMgrSite,
                    &raw mut lpObj as *mut *mut ::core::ffi::c_void,
                );
                if hRet >= 0 as HRESULT {
                    IInternetSecurityMgrSite_EnableModeless(
                        lpObj as *mut IInternetSecurityMgrSite,
                        bModeless,
                    );
                } else {
                    hRet = (*(*lpUnknown).lpVtbl)
                        .QueryInterface
                        .unwrap_unchecked()(
                        lpUnknown,
                        &raw const IID_IDocHostUIHandler,
                        &raw mut lpObj as *mut *mut ::core::ffi::c_void,
                    );
                    if hRet >= 0 as HRESULT {
                        IDocHostUIHandler_EnableModeless(
                            lpObj as *mut IDocHostUIHandler,
                            bModeless,
                        );
                    } else {
                        return hRet;
                    }
                }
            }
        }
    }
    (*(*lpObj).lpVtbl)
        .Release
        .unwrap_unchecked()(lpObj);
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetNewLinkInfoWrapW(
    pszLinkTo: LPCWSTR,
    pszDir: LPCWSTR,
    pszName: LPWSTR,
    pfMustCopy: *mut BOOL,
    uFlags: UINT,
) -> BOOL {
    SHGetNewLinkInfoW(pszLinkTo, pszDir, pszName, pfMustCopy, uFlags)
}

#[no_mangle]
pub unsafe extern "system" fn SHDefExtractIconWrapW(
    pszIconFile: LPCWSTR,
    iIndex: i32,
    uFlags: UINT,
    phiconLarge: *mut HICON,
    phiconSmall: *mut HICON,
    nIconSize: UINT,
) -> UINT {
    (SHDefExtractIconW(pszIconFile, iIndex, uFlags, phiconLarge, phiconSmall, nIconSize)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn SHInvokeCommand(hWnd: HWND, lpFolder: *mut IShellFolder, lpApidl: LPCITEMIDLIST, lpVerb: LPCSTR) -> HRESULT {
    return SHInvokeCommandWithFlagsAndSite(hWnd, core::ptr::null_mut::<IUnknown>(), lpFolder, lpApidl, 0, lpVerb);
}

#[no_mangle]
pub unsafe extern "system" fn ExtractIconWrapW(
    hInstance: HINSTANCE,
    lpszExeFileName: LPCWSTR,
    nIconIndex: UINT,
) -> HICON {
    ExtractIconW(hInstance, lpszExeFileName, nIconIndex)
}

#[no_mangle]
pub unsafe extern "system" fn MLLoadLibraryA(
    new_mod: LPCSTR,
    inst_hwnd: HMODULE,
    dwCrossCodePage: DWORD,
) -> HMODULE {
    let mut mod_path: [CHAR; (2 * MAX_PATH) as _] =
        [0; (2 * MAX_PATH) as _];
    let mut ptr: LPSTR = core::ptr::null_mut::<CHAR>();
    let mut len: DWORD = 0;
    WINE_FIXME!("(%s,%p,%d) semi-stub!\n", debugstr_a(new_mod as *const core::ffi::c_char), inst_hwnd, dwCrossCodePage);
    len = GetModuleFileNameA(
        inst_hwnd as HINSTANCE,
        core::ptr::addr_of_mut!(mod_path) as LPSTR,
        core::mem::size_of::<[CHAR; (2 * MAX_PATH) as _]>() as DWORD,
    );
    if len == 0
        || len as usize
            >= core::mem::size_of::<[CHAR; (2 * MAX_PATH) as _]>()
                as usize
    {
        return core::ptr::null_mut::<HINSTANCE__>();
    }
    ptr = strrchr(core::ptr::addr_of_mut!(mod_path) as *mut CHAR, '\\' as i32)
        as LPSTR;
    if !ptr.is_null() {
        strcpy(
            ptr.offset(1 as isize),
            new_mod as *const core::ffi::c_char,
        );
        WINE_TRACE!("loading %s\n", debugstr_a(core::ptr::addr_of_mut!(mod_path) as *mut CHAR));
        return LoadLibraryA(core::ptr::addr_of_mut!(mod_path) as *mut CHAR as LPCSTR)
            as HMODULE;
    }
    return core::ptr::null_mut::<HINSTANCE__>();
}

#[no_mangle]
pub unsafe extern "system" fn MLLoadLibraryW(new_mod: LPCWSTR, inst_hwnd: HMODULE, dwCrossCodePage: DWORD) -> HMODULE {
    let mut mod_path: [WCHAR; (2*MAX_PATH) as _] = [0; (2*MAX_PATH) as _];
    let mut ptr: LPWSTR;
    let mut len: DWORD;

    len = GetModuleFileNameW(
        inst_hwnd as HINSTANCE,
        core::ptr::addr_of_mut!(mod_path) as LPWSTR,
        (core::mem::size_of::<[WCHAR; (2*MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>()) as DWORD,
    );
    if len == 0
        || len as usize
            >= (core::mem::size_of::<[WCHAR; (2*MAX_PATH) as _]>() / core::mem::size_of::<WCHAR>())
    {
        return core::ptr::null_mut();
    }

    ptr = strrchrW!(core::ptr::addr_of_mut!(mod_path) as LPWSTR, '\\' as WCHAR);
    if !ptr.is_null() {
        strcpyW!(ptr.offset(1 as isize), new_mod);

        return LoadLibraryW(core::ptr::addr_of_mut!(mod_path) as LPWSTR as LPCWSTR) as HMODULE;
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ColorAdjustLuma(mut cRGB: COLORREF, mut dwLuma: i32, mut bUnknown: BOOL) -> COLORREF {
    if dwLuma != 0 {
        let mut wH: WORD = 0;
        let mut wL: WORD = 0;
        let mut wS: WORD = 0;

        ColorRGBToHLS(
            cRGB,
            core::ptr::addr_of_mut!(wH),
            core::ptr::addr_of_mut!(wL),
            core::ptr::addr_of_mut!(wS),
        );

        /* FIXME: The adjustment is not linear */

        cRGB = ColorHLSToRGB(wH, wL, wS);
    }
    return cRGB;
}

#[no_mangle]
pub unsafe extern "system" fn GetSaveFileNameWrapW(ofn: LPOPENFILENAMEW) -> BOOL {
    GetSaveFileNameW(ofn)
}

#[no_mangle]
pub unsafe extern "system" fn WNetRestoreConnectionWrapW(
    hwndOwner: HWND,
    lpszDevice: LPWSTR,
) -> DWORD {
    WNetRestoreConnectionW(hwndOwner, lpszDevice)
}

#[no_mangle]
pub unsafe extern "system" fn WNetGetLastErrorWrapW(
    lpError: LPDWORD,
    lpErrorBuf: LPWSTR,
    nErrorBufSize: DWORD,
    lpNameBuf: LPWSTR,
    nNameBufSize: DWORD,
) -> DWORD {
    WNetGetLastErrorW(lpError, lpErrorBuf, nErrorBufSize, lpNameBuf, nNameBufSize)
}

#[no_mangle]
pub unsafe extern "system" fn PageSetupDlgWrapW(pagedlg: LPPAGESETUPDLGW) -> BOOL {
    PageSetupDlgW(pagedlg)
}

#[no_mangle]
pub unsafe extern "system" fn PrintDlgWrapW(printdlg: LPPRINTDLGW) -> BOOL {
    PrintDlgW(printdlg)
}

#[no_mangle]
pub unsafe extern "system" fn GetOpenFileNameWrapW(ofn: LPOPENFILENAMEW) -> BOOL {
    GetOpenFileNameW(ofn)
}

#[no_mangle]
pub unsafe extern "system" fn SHIShellFolder_EnumObjects(
    lpFolder: LPSHELLFOLDER,
    hwnd: HWND,
    flags: SHCONTF,
    ppenum: *mut *mut IEnumIDList,
) -> HRESULT {
    IShellFolder_EnumObjects(lpFolder, hwnd, flags, ppenum)
}

unsafe extern "C" fn ConvertHue(mut wHue: ::core::ffi::c_int, mut wMid1: WORD, mut wMid2: WORD) -> WORD {
    wHue = if wHue > 240 {
        wHue - 240
    } else if wHue < 0 {
        wHue + 240
    } else {
        wHue
    };
    if wHue > 160 {
        return wMid1;
    } else if wHue > 120 {
        wHue = 160 - wHue;
    } else if wHue > 40 {
        return wMid2;
    }
    return ((wHue * (wMid2 as ::core::ffi::c_int - wMid1 as ::core::ffi::c_int)
        + 20)
        / 40
        + wMid1 as ::core::ffi::c_int) as WORD;
}

#[no_mangle]
pub unsafe extern "system" fn ColorHLSToRGB(wHue: WORD, wLuminosity: WORD, wSaturation: WORD) -> COLORREF {
    let mut wRed: WORD = 0;
    if wSaturation != 0 {
        let mut wGreen: WORD = 0;
        let mut wBlue: WORD = 0;
        let mut wMid1: WORD = 0;
        let mut wMid2: WORD = 0;

        if wLuminosity as i32 > 120 as i32 {
            wMid2 = (wSaturation as i32 + wLuminosity as i32
                - (wSaturation as i32 * wLuminosity as i32 + 120 as i32)
                    / 240 as i32) as WORD;
        } else {
            wMid2 = (((wSaturation as i32 + 240 as i32)
                * wLuminosity as i32
                + 120 as i32)
                / 240 as i32) as WORD;
        }

        wMid1 = (wLuminosity as i32 * 2 as i32 - wMid2 as i32) as WORD;

        wRed = ((ConvertHue(wHue as i32 + 80 as i32, wMid1, wMid2) as i32 * 255 + 120) / 240) as WORD;
        wGreen = ((ConvertHue(wHue as i32, wMid1, wMid2) as i32 * 255 + 120) / 240) as WORD;
        wBlue = ((ConvertHue(wHue as i32 - 80 as i32, wMid1, wMid2) as i32 * 255 + 120) / 240) as WORD;

        return RGB!(wRed, wGreen, wBlue);
    }

    wRed = (wLuminosity as i32 * 255 as i32 / 240 as i32) as WORD;
    return RGB!(wRed, wRed, wRed);
}

#[no_mangle]
pub unsafe extern "system" fn SHGetMachineInfo(dwFlags: DWORD) -> DWORD {
    let mut hwInfo: HW_PROFILE_INFOA = core::mem::zeroed();

    GetCurrentHwProfileA(core::ptr::addr_of_mut!(hwInfo));
    let masked: DWORD = hwInfo.dwDockInfo & (DOCKINFO_DOCKED | DOCKINFO_UNDOCKED);
    if masked == DOCKINFO_DOCKED || masked == DOCKINFO_UNDOCKED {
        return hwInfo.dwDockInfo & (DOCKINFO_DOCKED | DOCKINFO_UNDOCKED);
    } else {
        return 0;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHWinHelpOnDemandW(
    hwnd: HWND,
    helpfile: LPCWSTR,
    flags1: DWORD,
    ptr1: LPVOID,
    flags2: DWORD,
) -> DWORD {
    return 0 as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHWinHelpOnDemandA(hwnd: HWND, helpfile: LPCSTR, flags1: DWORD, ptr1: LPVOID, flags2: DWORD) -> DWORD {
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn MLFreeLibrary(hModule: HMODULE) -> BOOL {
    // FIXME("(%p) semi-stub\n", hModule);
    FreeLibrary(hModule)
}

#[no_mangle]
pub unsafe extern "system" fn SHFlushSFCacheWrap() -> BOOL {
    // FIXME(": stub\n");
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn MLIsMLHInstance(hInst: HINSTANCE) -> BOOL {
    // FIXME("(%p) stub\n", hInst);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn MLSetMLHInstance(hInst: HINSTANCE, hHeap: HANDLE) -> DWORD {
    // FIXME("(%p,%p) stub\n", hInst, hHeap);
    (E_FAIL) as u32
}

#[no_mangle]
pub unsafe extern "system" fn MLClearMLHInstance(x: DWORD) -> DWORD {
    // FIXME("(0x%08x)stub\n", x);
    0xabba1247
}

#[no_mangle]
pub unsafe extern "system" fn SHSendMessageBroadcastA(uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> DWORD {
    return SendMessageTimeoutA(HWND_BROADCAST, uMsg, wParam, lParam, SMTO_ABORTIFHUNG, 2000, core::ptr::null_mut()) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SHSendMessageBroadcastW(uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> DWORD {
    return SendMessageTimeoutW(HWND_BROADCAST, uMsg, wParam, lParam, SMTO_ABORTIFHUNG, 2000, core::ptr::null_mut()) as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn CLSIDFromStringWrap(idstr: LPCWSTR, id: *mut CLSID) -> HRESULT {
    return CLSIDFromString(idstr as LPCOLESTR, id);
}

#[no_mangle]
pub unsafe extern "system" fn IsOS(feature: DWORD) -> BOOL {
    let mut osvi: OSVERSIONINFOEXA = core::mem::zeroed();
    let mut platform: DWORD;
    let mut majorv: DWORD;
    let mut minorv: DWORD;

    osvi.dwOSVersionInfoSize = core::mem::size_of::<OSVERSIONINFOEXA>() as DWORD;
    if GetVersionExA(core::ptr::addr_of_mut!(osvi) as *mut OSVERSIONINFOA) == 0 {
        osvi.dwOSVersionInfoSize = (core::mem::size_of::<OSVERSIONINFOA>()) as _;
        if GetVersionExA(core::ptr::addr_of_mut!(osvi) as *mut OSVERSIONINFOA) == 0 {
            ERR!("GetVersionEx failed\n");
            return (FALSE) as BOOL;
        }
        osvi.wProductType = (VER_NT_WORKSTATION as u8);
        osvi.wSuiteMask = 0;
    }
    majorv = osvi.dwMajorVersion;
    minorv = osvi.dwMinorVersion;
    platform = osvi.dwPlatformId;

    match feature {
        OS_WIN32SORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32s
                || platform == VER_PLATFORM_WIN32_WINDOWS) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_NT => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN95ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_NT4ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && majorv >= 4) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000ORGREATER_ALT | OS_WIN2000ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && majorv >= 5) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN98ORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS
                && minorv >= 10) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN98_GOLD => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS
                && minorv == 10) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000PRO => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && majorv >= 5) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000SERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000ADVSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000DATACENTER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WIN2000TERMINAL => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (minorv == 0 || minorv == 1)) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_EMBEDDED => {
            FIXME!("(OS_EMBEDDED) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_TERMINALCLIENT => {
            FIXME!("(OS_TERMINALCLIENT) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_TERMINALREMOTEADMIN => {
            FIXME!("(OS_TERMINALREMOTEADMIN) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_WIN95_GOLD => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS
                && minorv == 0) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_MEORGREATER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_WINDOWS
                && minorv >= 90) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_XPORGREATER => {
            /* "5.1 or later": the Wine test is FALSE at 10.0 (minor 0) */
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (majorv > 5
                    || (majorv == 5 && minorv >= 1))) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_HOME => {
            /* "5.1 or later": the Wine test is FALSE at 10.0 (minor 0) */
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (majorv > 5
                    || (majorv == 5 && minorv >= 1))) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_PROFESSIONAL => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_DATACENTER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_ADVSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && majorv >= 5) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_SERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_TERMINALSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_PERSONALTERMINALSERVER => {
            /* "5.1 or later": the Wine test is FALSE at 10.0 (minor 0) */
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && (majorv > 5
                    || (majorv == 5 && minorv >= 1))) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_FASTUSERSWITCHING => {
            FIXME!("(OS_FASTUSERSWITCHING) What should we return here?\n");
            return (TRUE) as BOOL;
        }
        OS_WELCOMELOGONUI => {
            FIXME!("(OS_WELCOMELOGONUI) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_DOMAINMEMBER => {
            FIXME!("(OS_DOMAINMEMBER) What should we return here?\n");
            return (TRUE) as BOOL;
        }
        OS_ANYSERVER => {
            let ret: BOOL = ((osvi.wProductType as DWORD) > VER_NT_WORKSTATION) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_WOW6432 => {
            let mut is_wow64: BOOL = 0;
            IsWow64Process(GetCurrentProcess(), core::ptr::addr_of_mut!(is_wow64));
            return is_wow64;
        }
        OS_WEBSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_SMALLBUSINESSSERVER => {
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_TABLETPC => {
            FIXME!("(OS_TABLETPC) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_SERVERADMINUI => {
            let mut value: DWORD = FALSE as DWORD;
            let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;
            let mut hKey: HKEY = SHGetShellKey(
                (SHKEY_Root_HKCU | SHKEY_Key_Explorer) as DWORD,
                RUSTCONV_WSTR_52D451AA9857.as_ptr(),
                (FALSE) as BOOL,
            );
            if !hKey.is_null() {
                SHQueryValueExW(
                    hKey,
                    RUSTCONV_WSTR_F6C54912C188.as_ptr(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::addr_of_mut!(value) as LPVOID,
                    core::ptr::addr_of_mut!(size),
                );
                RegCloseKey(hKey);
            }
            let ret: BOOL = value as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        OS_MEDIACENTER => {
            FIXME!("(OS_MEDIACENTER) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        OS_APPLIANCE => {
            FIXME!("(OS_APPLIANCE) What should we return here?\n");
            return (FALSE) as BOOL;
        }
        0x25 => {
            /*OS_VISTAORGREATER*/
            let ret: BOOL = (platform == VER_PLATFORM_WIN32_NT
                && majorv >= 6) as BOOL;
            TRACE!("(0x%x) ret=%d\n", feature, ret);
            return ret;
        }
        _ => {}
    }

    WARN!("(0x%x) unknown parameter\n", feature);

    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHLoadRegUIStringA(hkey: HKEY, value: LPCSTR, buf: LPSTR, size: DWORD) -> HRESULT {
    let mut valueW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut bufferW: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut dwSize: DWORD = (ARRAY_SIZE!(bufferW) as DWORD) * (core::mem::size_of::<CHAR>() as DWORD);
    let mut hr: HRESULT;

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        value,
        -(1 as ::core::ffi::c_int),
        core::ptr::addr_of_mut!(valueW) as LPWSTR,
        ARRAY_SIZE!(valueW) as ::core::ffi::c_int,
    );
    valueW[(ARRAY_SIZE!(valueW) - (1 as usize) as u32) as usize] = UNICODE_NULL; /* Avoid buffer overrun */

    if RegQueryValueExW(
        hkey,
        core::ptr::addr_of_mut!(valueW) as *mut WCHAR as LPCWSTR,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(bufferW) as *mut WCHAR as LPBYTE,
        core::ptr::addr_of_mut!(dwSize),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return E_FAIL;
    }

    hr = SHLoadIndirectString(
        core::ptr::addr_of_mut!(bufferW) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(bufferW) as LPWSTR,
        ARRAY_SIZE!(bufferW) as UINT,
        core::ptr::null_mut::<PVOID>(),
    );
    if hr < 0 as HRESULT {
        return hr;
    }

    WideCharToMultiByte(
        CP_ACP as UINT,
        0 as DWORD,
        core::ptr::addr_of_mut!(bufferW) as *mut WCHAR as LPCWSTR,
        -(1 as ::core::ffi::c_int),
        buf,
        size as ::core::ffi::c_int,
        core::ptr::null::<CHAR>(),
        core::ptr::null_mut::<BOOL>(),
    );
    if size > 0 as DWORD {
        *buf.offset((size - 1 as DWORD) as isize) = ANSI_NULL; /* Avoid buffer overrun */
    }
    return S_OK;
}

#[no_mangle]
pub unsafe extern "system" fn SHLoadRegUIStringW(hkey: HKEY, value: LPCWSTR, buf: LPWSTR, size: DWORD) -> HRESULT {
    let mut type_0: DWORD = 0;
    let mut sz: DWORD = (size as usize).wrapping_mul(core::mem::size_of::<WCHAR>()) as DWORD;
    if RegQueryValueExW(
        hkey,
        value,
        core::ptr::null_mut::<DWORD>(),
        core::ptr::addr_of_mut!(type_0),
        buf as LPBYTE,
        core::ptr::addr_of_mut!(sz),
    ) != ERROR_SUCCESS as LSTATUS
    {
        return E_FAIL as HRESULT;
    }
    return SHLoadIndirectString(
        buf as LPCWSTR,
        buf,
        size as UINT,
        core::ptr::null_mut::<PVOID>(),
    );
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_TranslateAcceleratorIO(lpUnknown: *mut IUnknown, lpMsg: LPMSG) -> HRESULT {
    let mut lpInput: *mut IInputObject = core::ptr::null_mut();
    let mut hRet: HRESULT = E_INVALIDARG;
    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            core::ptr::addr_of!(IID_IInputObject),
            (core::ptr::addr_of_mut!(lpInput) as *mut core::ffi::c_void).cast(),
        );
        if hRet >= 0 && !lpInput.is_null() {
            hRet = IInputObject_TranslateAcceleratorIO(lpInput, lpMsg);
            IInputObject_Release(lpInput);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_HasFocusIO(lpUnknown: *mut IUnknown) -> HRESULT {
    let mut lpInput: *mut IInputObject = core::ptr::null_mut();
    let mut hRet: HRESULT = E_INVALIDARG;
    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(
            lpUnknown,
            core::ptr::addr_of!(IID_IInputObject),
            core::ptr::addr_of_mut!(lpInput) as *mut *mut core::ffi::c_void,
        );
        if hRet >= 0 && !lpInput.is_null() {
            hRet = IInputObject_HasFocusIO(lpInput);
            IInputObject_Release(lpInput);
        }
    }
    hRet
}

#[no_mangle]
pub unsafe extern "system" fn ColorRGBToHLS(
    cRGB: COLORREF,
    pwHue: LPWORD,
    pwLuminance: LPWORD,
    pwSaturation: LPWORD,
) {
    let mut wR: ::core::ffi::c_int = 0;
    let mut wG: ::core::ffi::c_int = 0;
    let mut wB: ::core::ffi::c_int = 0;
    let mut wMax: ::core::ffi::c_int = 0;
    let mut wMin: ::core::ffi::c_int = 0;
    let mut wHue: ::core::ffi::c_int = 0;
    let mut wLuminosity: ::core::ffi::c_int = 0;
    let mut wSaturation: ::core::ffi::c_int = 0;

    wR = GetRValue!(cRGB) as ::core::ffi::c_int;
    wG = GetGValue!(cRGB) as ::core::ffi::c_int;
    wB = GetBValue!(cRGB) as ::core::ffi::c_int;

    wMax = max!(wR, max!(wG, wB));
    wMin = min!(wR, min!(wG, wB));

    /* Luminosity */
    wLuminosity = ((wMax + wMin) * 240 + 255) / 510;

    if wMax == wMin {
        /* Achromatic case */
        wSaturation = 0;
        /* Hue is now unrepresentable, but this is what native returns... */
        wHue = 160;
    } else {
        /* Chromatic case */
        let mut wDelta: ::core::ffi::c_int = wMax - wMin;
        let mut wRNorm: ::core::ffi::c_int = 0;
        let mut wGNorm: ::core::ffi::c_int = 0;
        let mut wBNorm: ::core::ffi::c_int = 0;

        /* Saturation */
        if wLuminosity <= 120 {
            wSaturation = ((wMax + wMin) / 2 + wDelta * 240) / (wMax + wMin);
        } else {
            wSaturation = ((510 - wMax - wMin) / 2 + wDelta * 240) / (510 - wMax - wMin);
        }

        /* Hue */
        wRNorm = (wDelta / 2 + wMax * 40 - wR * 40) / wDelta;
        wGNorm = (wDelta / 2 + wMax * 40 - wG * 40) / wDelta;
        wBNorm = (wDelta / 2 + wMax * 40 - wB * 40) / wDelta;

        if wR == wMax {
            wHue = wBNorm - wGNorm;
        } else if wG == wMax {
            wHue = 80 + wRNorm - wBNorm;
        } else {
            wHue = 160 + wGNorm - wRNorm;
        }
        if wHue < 0 {
            wHue += 240;
        } else if wHue > 240 {
            wHue -= 240;
        }
    }
    if !pwHue.is_null() {
        *pwHue = wHue as WORD;
    }
    if !pwLuminance.is_null() {
        *pwLuminance = wLuminosity as WORD;
    }
    if !pwSaturation.is_null() {
        *pwSaturation = wSaturation as WORD;
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHCreateShellPalette(hdc: HDC) -> HPALETTE {
    let mut hdcMem: HDC;
    let mut hHalftonePalette: HPALETTE;
    let mut data: LOGPALETTEMAX = core::mem::zeroed();
    let nExtractCount: UINT = 10 as UINT;
    let nSecondBlockStart: UINT = _countof!(data.palPalEntry) - nExtractCount;

    /* Get the colors of the halftone palette */
    hHalftonePalette = CreateHalftonePalette(hdc);
    if hHalftonePalette.is_null() {
        return core::ptr::null_mut();
    }
    data.palVersion = 0x300 as WORD;
    data.palNumEntries = GetPaletteEntries(
        hHalftonePalette,
        0 as UINT,
        _countof!(data.palPalEntry) as UINT,
        &raw mut data.palPalEntry as LPPALETTEENTRY,
    ) as WORD;
    DeleteObject(hHalftonePalette as HGDIOBJ);

    hdcMem = if !hdc.is_null() {
        hdc
    } else {
        CreateCompatibleDC(core::ptr::null_mut())
    };

    if !hdcMem.is_null() {
        /* The first 10 and last 10 entries in the system colors are considered important */
        GetSystemPaletteEntries(
            hdcMem,
            0 as UINT,
            nExtractCount,
            &raw mut data.palPalEntry as LPPALETTEENTRY,
        );
        GetSystemPaletteEntries(
            hdcMem,
            nSecondBlockStart,
            nExtractCount,
            (&raw mut data.palPalEntry as *mut PALETTEENTRY).offset(nSecondBlockStart as isize)
                as LPPALETTEENTRY,
        );
    }

    if !hdcMem.is_null() && hdc != hdcMem {
        DeleteDC(hdcMem);
    }

    /* Create a palette from the modified color entries */
    return CreatePalette(&raw mut data as PLOGPALETTE as *const LOGPALETTE);
}

#[no_mangle]
pub unsafe extern "system" fn SHGetInverseCMAP(dest: LPDWORD, dwSize: DWORD) -> HRESULT {
    if dwSize == 4 {
        *dest = 0xabba1249 as DWORD;
        return 0;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn SHIsLowMemoryMachine(dwType: DWORD) -> BOOL {
    let mut status: MEMORYSTATUS = core::mem::zeroed();
    static mut is_low: BOOL = -1;

    if dwType == 0 && is_low == -1 {
        GlobalMemoryStatus(core::ptr::addr_of_mut!(status));
        is_low = (status.dwTotalPhys <= 0x1000000) as BOOL;
    }
    return is_low;
}

#[no_mangle]
pub unsafe extern "system" fn GetMenuPosFromID(hMenu: HMENU, wID: UINT) -> INT {
    let mut mi: MENUITEMINFOW = core::mem::zeroed();
    let mut nCount: INT = GetMenuItemCount(hMenu) as INT;
    let mut nIter: INT = 0 as INT;
    WINE_TRACE!("%p %u\n", hMenu, wID);
    while nIter < nCount {
        mi.cbSize = ::core::mem::size_of::<MENUITEMINFOW>() as UINT;
        mi.fMask = MIIM_ID as UINT;
        if GetMenuItemInfoW(hMenu, nIter as UINT, TRUE as _, &raw mut mi) != 0
            && mi.wID == wID
        {
            WINE_TRACE!("ret %d\n", nIter);
            return nIter;
        }
        nIter += 1;
    }

    return -(1 as INT);
}

#[no_mangle]
pub unsafe extern "system" fn SHMenuIndexFromID(hMenu: HMENU, uID: UINT) -> DWORD {
    // TRACE("%p %u\n", hMenu, uID);
    (GetMenuPosFromID(hMenu, uID)) as u32
}

#[no_mangle]
pub unsafe extern "system" fn FixSlashesAndColonW(lpwstr: LPWSTR) {
    let mut lpwstr = lpwstr;
    while *lpwstr != 0 {
        if *lpwstr == '/' as u16 {
            *lpwstr = '\\' as u16;
        }
        lpwstr = lpwstr.add(1);
    }
}

#[no_mangle]
pub unsafe extern "system" fn SHCoCreateInstanceAC(
    rclsid: REFCLSID,
    pUnkOuter: LPUNKNOWN,
    dwClsContext: DWORD,
    iid: REFIID,
    ppv: *mut LPVOID,
) -> HRESULT {
    CoCreateInstance(rclsid, pUnkOuter, dwClsContext, iid, ppv)
}

#[no_mangle]
pub unsafe extern "system" fn SHSkipJunction(pbc: *mut IBindCtx, pclsid: *const CLSID) -> BOOL {
    static mut szSkipBinding: [u16; 19] = [
    0x0053u16, 0x006B, 0x0069, 0x0070, 0x0020, 0x0042, 0x0069, 0x006E, 0x0064, 0x0069,
    0x006E, 0x0067, 0x0020, 0x0043, 0x004C, 0x0053, 0x0049, 0x0044, 0x0000,
];
    let mut bRet: BOOL = (FALSE as i32);

    if !pbc.is_null() {
        let mut lpUnk: *mut IUnknown = core::ptr::null_mut::<IUnknown>();

        if IBindCtx_GetObjectParam(pbc, core::ptr::addr_of_mut!(szSkipBinding) as LPOLESTR, core::ptr::addr_of_mut!(lpUnk))
            >= 0
        {
            let mut clsid: CLSID = GUID {
                Data1: 0,
                Data2: 0,
                Data3: 0,
                Data4: [0; 8],
            };

            if (IUnknown_GetClassID(lpUnk, core::ptr::addr_of_mut!(clsid)) >= 0)
                && IsEqualGUID!(pclsid, core::ptr::addr_of_mut!(clsid))
            {
                bRet = (TRUE as i32);
            }

            IUnknown_Release(lpUnk);
        }
    }
    return bRet;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetShellKey(mut flags: DWORD, mut sub_key: LPCWSTR, mut create: BOOL) -> HKEY {
    static explorerW: [u16; 51] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0045, 0x0078, 0x0070, 0x006C, 0x006F, 0x0072, 0x0065, 0x0072,
    0x005C,
];
    static shellW: [u16; 33] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0053, 0x0068, 0x0065,
    0x006C, 0x006C, 0x005C,
];
    static shell_no_roamW: [u16; 39] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0053, 0x0068, 0x0065,
    0x006C, 0x006C, 0x004E, 0x006F, 0x0052, 0x006F, 0x0061, 0x006D, 0x005C,
];
    static classesW: [u16; 17] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0043,
    0x006C, 0x0061, 0x0073, 0x0073, 0x0065, 0x0073, 0x005C,
];

    static localized_resource_nameW: [u16; 22] = [
    0x004Cu16, 0x006F, 0x0063, 0x0061, 0x006C, 0x0069, 0x007A, 0x0065, 0x0064, 0x0052,
    0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x004E, 0x0061, 0x006D,
    0x0065, 0x005C,
];
    static handlersW: [u16; 9] = [0x0048u16, 0x0061, 0x006E, 0x0064, 0x006C, 0x0065, 0x0072, 0x0073, 0x005C];
    static associationsW: [u16; 13] = [
    0x0041u16, 0x0073, 0x0073, 0x006F, 0x0063, 0x0069, 0x0061, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0073, 0x005C,
];
    static volatileW: [u16; 9] = [0x0056u16, 0x006F, 0x006C, 0x0061, 0x0074, 0x0069, 0x006C, 0x0065, 0x005C];
    static mui_cacheW: [u16; 9] = [0x004Du16, 0x0055, 0x0049, 0x0043, 0x0061, 0x0063, 0x0068, 0x0065, 0x005C];
    static file_extsW: [u16; 9] = [0x0046u16, 0x0069, 0x006C, 0x0065, 0x0045, 0x0078, 0x0074, 0x0073, 0x005C];

    let mut path: *mut WCHAR = core::ptr::null_mut();
    let mut key: *const WCHAR = core::ptr::null();
    let mut subkey: *const WCHAR = core::ptr::null();
    let mut size_key: i32 = 0;
    let mut size_subkey: i32 = 0;
    let mut size_user: i32 = 0;
    let mut hkey: HKEY = core::ptr::null_mut();

    TRACE!("(0x%08x, %s, %d)\n", flags, debugstr_w(sub_key), create);

    /* For compatibility with Vista+ */
    if flags == 0x1ffff {
        flags = 0x21;
    }

    match flags & 0xff0 {
        0 => {
            // SHKEY_Key_Explorer
            key = &raw const explorerW as *const WCHAR;
            size_key = core::mem::size_of::<[u16; 51]>() as i32;
        }
        16 => {
            // SHKEY_Key_Shell
            key = &raw const shellW as *const WCHAR;
            size_key = core::mem::size_of::<[u16; 33]>() as i32;
        }
        32 => {
            // SHKEY_Key_ShellNoRoam
            key = &raw const shell_no_roamW as *const WCHAR;
            size_key = core::mem::size_of::<[u16; 39]>() as i32;
        }
        48 => {
            // SHKEY_Key_Classes
            key = &raw const classesW as *const WCHAR;
            size_key = core::mem::size_of::<[u16; 17]>() as i32;
        }
        _ => {
            FIXME!("unsupported flags (0x%08x)\n", flags);
            return core::ptr::null_mut();
        }
    }

    match flags & 0xff000 {
        0 => {
            // SHKEY_Subkey_Default
            subkey = core::ptr::null();
            size_subkey = 0 as i32;
        }
        4096 => {
            // SHKEY_Subkey_ResourceName
            subkey = &raw const localized_resource_nameW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 22]>() as i32;
        }
        8192 => {
            // SHKEY_Subkey_Handlers
            subkey = &raw const handlersW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 9]>() as i32;
        }
        12288 => {
            // SHKEY_Subkey_Associations
            subkey = &raw const associationsW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 13]>() as i32;
        }
        16384 => {
            // SHKEY_Subkey_Volatile
            subkey = &raw const volatileW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 9]>() as i32;
        }
        20480 => {
            // SHKEY_Subkey_MUICache
            subkey = &raw const mui_cacheW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 9]>() as i32;
        }
        24576 => {
            // SHKEY_Subkey_FileExts
            subkey = &raw const file_extsW as *const WCHAR;
            size_subkey = core::mem::size_of::<[u16; 9]>() as i32;
        }
        _ => {
            FIXME!("unsupported flags (0x%08x)\n", flags);
            return core::ptr::null_mut();
        }
    }

    if !sub_key.is_null() {
        size_user = (lstrlenW(sub_key) as usize)
            .wrapping_mul(core::mem::size_of::<WCHAR>()) as i32;
    } else {
        size_user = 0 as i32;
    }

    path = HeapAlloc(
        GetProcessHeap(),
        0,
        (((size_key + size_subkey + size_user) as usize).wrapping_add(core::mem::size_of::<WCHAR>())) as SIZE_T,
    ) as *mut WCHAR;
    if path.is_null() {
        ERR!("Out of memory\n");
        return core::ptr::null_mut();
    }

    memcpy(path as *mut core::ffi::c_void, key as *const core::ffi::c_void, size_key as usize);
    if !subkey.is_null() {
        memcpy(
            path.offset((size_key as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize) as *mut core::ffi::c_void,
            subkey as *const core::ffi::c_void,
            size_subkey as usize,
        );
    }
    if !sub_key.is_null() {
        memcpy(
            path.offset(((size_key + size_subkey) as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize)
                as *mut core::ffi::c_void,
            sub_key as *const core::ffi::c_void,
            size_user as usize,
        );
    }
    *path.offset(((size_key + size_subkey + size_user) as usize).wrapping_div(core::mem::size_of::<WCHAR>()) as isize) =
        0 as WCHAR;

    if create != 0 {
        RegCreateKeyExW(
            if (flags & 0xf) == 0x2 as DWORD /* SHKEY_Root_HKLM */ {
                HKEY_LOCAL_MACHINE
            } else {
                HKEY_CURRENT_USER
            },
            path as LPCWSTR,
            0,
            core::ptr::null_mut(),
            0,
            MAXIMUM_ALLOWED,
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(hkey),
            core::ptr::null_mut(),
        );
    } else {
        RegOpenKeyExW(
            if (flags & 0xf) == 0x2 as DWORD /* SHKEY_Root_HKLM */ {
                HKEY_LOCAL_MACHINE
            } else {
                HKEY_CURRENT_USER
            },
            path as LPCWSTR,
            0,
            MAXIMUM_ALLOWED,
            core::ptr::addr_of_mut!(hkey),
        );
    }

    HeapFree(GetProcessHeap(), 0, path as PVOID);
    return hkey;
}

#[no_mangle]
pub unsafe extern "system" fn SHQueueUserWorkItem(
    pfnCallback: LPTHREAD_START_ROUTINE,
    pContext: LPVOID,
    lPriority: LONG,
    dwTag: DWORD_PTR,
    pdwId: *mut DWORD_PTR,
    pszModule: LPCSTR,
    dwFlags: DWORD,
) -> BOOL {
    if lPriority != 0 || dwTag != 0 || !pdwId.is_null() || !pszModule.is_null() || dwFlags != 0 {
        FIXME!("Unsupported arguments\n");
    }

    return QueueUserWorkItem(pfnCallback, pContext, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SHSetTimerQueueTimer(
    hQueue: HANDLE,
    pfnCallback: WAITORTIMERCALLBACK,
    pContext: LPVOID,
    dwDueTime: DWORD,
    dwPeriod: DWORD,
    lpszLibrary: LPCSTR,
    mut dwFlags: DWORD,
) -> HANDLE {
    let mut hNewTimer: HANDLE = core::ptr::null_mut();

    /* SHSetTimerQueueTimer flags -> CreateTimerQueueTimer flags */
    if dwFlags & TPS_LONGEXECTIME as DWORD != 0 {
        dwFlags &= !TPS_LONGEXECTIME as DWORD;
        dwFlags |= WT_EXECUTELONGFUNCTION as DWORD;
    }
    if dwFlags & TPS_EXECUTEIO as DWORD != 0 {
        dwFlags &= !TPS_EXECUTEIO as DWORD;
        dwFlags |= WT_EXECUTEINIOTHREAD as DWORD;
    }

    if CreateTimerQueueTimer(
        core::ptr::addr_of_mut!(hNewTimer),
        hQueue,
        pfnCallback,
        pContext,
        dwDueTime,
        dwPeriod,
        dwFlags,
    ) == 0
    {
        return core::ptr::null_mut();
    }

    return hNewTimer;
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_OnFocusChangeIS(lpUnknown: LPUNKNOWN, pFocusObject: LPUNKNOWN, bFocus: BOOL) -> HRESULT {
    let mut pIOS: *mut IInputObjectSite = core::ptr::null_mut();
    let mut hRet: HRESULT = E_INVALIDARG;

    if !lpUnknown.is_null() {
        hRet = IUnknown_QueryInterface(lpUnknown, core::ptr::addr_of!(IID_IInputObjectSite), core::ptr::addr_of_mut!(pIOS) as *mut LPVOID);
        if (hRet >= 0) && !pIOS.is_null() {
            hRet = IInputObjectSite_OnFocusChangeIS(pIOS, pFocusObject, bFocus);
            IInputObjectSite_Release(pIOS);
        }
    }
    return hRet;
}

#[no_mangle]
pub unsafe extern "system" fn SKAllocValueW(
    flags: DWORD,
    subkey: LPCWSTR,
    value: LPCWSTR,
    type_: *mut DWORD,
    data: *mut LPVOID,
    count: *mut DWORD,
) -> HRESULT {
    let mut ret: DWORD = 0;
    let mut size: DWORD = 0;
    let mut hkey: HKEY = core::ptr::null_mut();
    WINE_TRACE!("(0x%x, %s, %s, %p, %p, %p)\n", flags, debugstr_w(subkey as *const WCHAR), debugstr_w(value as *const WCHAR), type_, data, count);

    hkey = SHGetShellKey(flags, subkey, (FALSE) as BOOL);
    if hkey.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    }

    ret = SHQueryValueExW(
        hkey,
        value,
        core::ptr::null_mut(),
        type_,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(size),
    );
    if ret != 0 {
        RegCloseKey(hkey);
        return HRESULT_FROM_WIN32!(ret);
    }

    size = size.wrapping_add(2);
    *data = LocalAlloc(0, size as SIZE_T) as LPVOID;
    if (*data).is_null() {
        RegCloseKey(hkey);
        return E_OUTOFMEMORY;
    }

    ret = SHQueryValueExW(
        hkey,
        value,
        core::ptr::null_mut(),
        type_,
        *data,
        core::ptr::addr_of_mut!(size),
    );
    if !count.is_null() {
        *count = size;
    }

    RegCloseKey(hkey);
    return HRESULT_FROM_WIN32!(ret);
}

#[no_mangle]
pub unsafe extern "system" fn SKDeleteValueW(flags: DWORD, subkey: LPCWSTR, value: LPCWSTR) -> HRESULT {
    let mut ret: DWORD;
    let mut hkey: HKEY;

    hkey = SHGetShellKey(flags, subkey, (FALSE) as BOOL);
    if hkey.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    }

    ret = RegDeleteValueW(hkey, value) as DWORD;

    RegCloseKey(hkey);
    return HRESULT_FROM_WIN32!(ret);
}

#[no_mangle]
pub unsafe extern "system" fn SKGetValueW(flags: DWORD, subkey: LPCWSTR, value: LPCWSTR, type_: *mut DWORD, data: LPVOID, count: *mut DWORD) -> HRESULT {
    let mut ret: DWORD;
    let mut hkey: HKEY;

    hkey = SHGetShellKey(flags, subkey, (FALSE) as BOOL);
    if hkey.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    }

    ret = SHQueryValueExW(hkey, value, core::ptr::null_mut(), type_, data, count);

    RegCloseKey(hkey);
    return HRESULT_FROM_WIN32!(ret);
}

#[no_mangle]
pub unsafe extern "system" fn SKSetValueW(
    mut flags: DWORD,
    mut subkey: LPCWSTR,
    mut value: LPCWSTR,
    mut type_0: DWORD,
    mut data: *mut ::core::ffi::c_void,
    mut count: DWORD,
) -> HRESULT {
    let mut ret: DWORD = 0;
    let mut hkey: HKEY = ::core::ptr::null_mut::<HKEY__>();
    WINE_TRACE!("(0x%x, %s, %s, %x, %p, %d)\n", flags, debugstr_w(subkey as *const WCHAR), debugstr_w(value as *const WCHAR), type_0, data, count);
    hkey = SHGetShellKey(flags, subkey, (TRUE) as BOOL);
    if hkey.is_null() {
        return HRESULT_FROM_WIN32!(ERROR_FILE_NOT_FOUND);
    }
    ret = RegSetValueExW(
        hkey,
        value,
        0,
        type_0,
        data as *const BYTE,
        count,
    ) as DWORD;
    RegCloseKey(hkey);
    return HRESULT_FROM_WIN32!(ret);
}

#[no_mangle]
pub unsafe extern "system" fn GetUIVersion() -> DWORD {
    static mut version: DWORD = 0;
    if version == 0 {
        let mut pDllGetVersion: DllGetVersion_func = None;
        let mut dll: HMODULE = LoadLibraryA(b"shell32.dll\0".as_ptr() as LPCSTR) as HMODULE;
        if dll.is_null() {
            return 0 as DWORD;
        }
        pDllGetVersion = ::core::mem::transmute::<FARPROC, DllGetVersion_func>(GetProcAddress(
            dll as HINSTANCE,
            b"DllGetVersion\0".as_ptr() as LPCSTR,
        ));
        if pDllGetVersion.is_some() {
            let mut dvi: DLLVERSIONINFO = _DllVersionInfo {
                cbSize: 0,
                dwMajorVersion: 0,
                dwMinorVersion: 0,
                dwBuildNumber: 0,
                dwPlatformID: 0,
            };
            dvi.cbSize = core::mem::size_of::<DLLVERSIONINFO>() as DWORD;
            if (pDllGetVersion.unwrap_unchecked())(&raw mut dvi) == S_OK {
                version = dvi.dwMajorVersion;
            }
        }
        FreeLibrary(dll);
        if version == 0 {
            version = 3 as DWORD;
        }
    }
    return version;
}

#[no_mangle]
pub unsafe extern "C" fn ShellMessageBoxWrapW_v(
    mut hInstance: HINSTANCE,
    mut hWnd: HWND,
    mut lpText: LPCWSTR,
    mut lpCaption: LPCWSTR,
    mut uType: UINT,
    ap_list: *mut ::core::ffi::c_void,
) -> INT {
    let mut szText: *mut WCHAR = core::ptr::null_mut();
    let mut szTitle: [WCHAR; 100] = [0; 100];
    let mut pszText: LPCWSTR;
    let mut pszTitle: LPCWSTR = core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPCWSTR;
    let mut pszTemp: LPWSTR = core::ptr::null_mut();

    let mut ret: INT;

    if IS_INTRESOURCE!(lpCaption) {
        LoadStringW(
            hInstance,
            LOWORD!(lpCaption) as UINT,
            core::ptr::addr_of_mut!(szTitle) as *mut WCHAR as LPWSTR,
            (ARRAY_SIZE!(szTitle)) as i32,
        );
    } else {
        pszTitle = lpCaption;
    }

    if IS_INTRESOURCE!(lpText) {
        let mut ptr: *const WCHAR = core::ptr::null();
        let mut len: UINT = LoadStringW(
            hInstance,
            LOWORD!(lpText) as UINT,
            core::ptr::addr_of_mut!(ptr) as LPWSTR,
            0,
        ) as UINT;

        if len != 0 {
            szText = HeapAlloc(
                GetProcessHeap(),
                0,
                (len.wrapping_add(1) as SIZE_T)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as *mut WCHAR;
            if !szText.is_null() {
                LoadStringW(
                    hInstance,
                    LOWORD!(lpText) as UINT,
                    szText as LPWSTR,
                    len.wrapping_add(1) as INT,
                );
            }
        }
        pszText = szText as LPCWSTR;
        if pszText.is_null() {
            WARN!("Failed to load id %d\n", LOWORD!(lpText) as UINT);
            return 0;
        }
    } else {
        pszText = lpText;
    }

    FormatMessageW(
        (FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_STRING) as DWORD,
        pszText as LPCVOID,
        0,
        0,
        core::ptr::addr_of_mut!(pszTemp) as LPWSTR,
        0,
        (ap_list as *mut va_list),
    );


    uType |= MB_SETFOREGROUND;
    ret = MessageBoxW(hWnd, pszTemp as LPCWSTR, pszTitle, uType);

    HeapFree(
        GetProcessHeap(),
        0,
        szText as PVOID,
    );
    LocalFree(pszTemp as HLOCAL);
    return ret;
}



#[no_mangle]
pub unsafe extern "system" fn ZoneComputePaneSize(hwnd: HWND) -> UINT {
    // FIXME("\n");
    0x95
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyWrap(
    wEventId: LONG,
    uFlags: UINT,
    dwItem1: LPCVOID,
    dwItem2: LPCVOID,
) {
    SHChangeNotify(wEventId, uFlags, dwItem1, dwItem2);
}

#[no_mangle]
pub unsafe extern "system" fn GetShellSecurityDescriptor(apUserPerm: *const PSHELL_USER_PERMISSION, cUserPerm: c_int) -> PSECURITY_DESCRIPTOR {
    let sidlist: *mut PSID;
    let mut cur_user: PSID = core::ptr::null_mut();
    let mut tuUser: [BYTE; 2000] = [0; 2000];
    let mut acl_size: DWORD;
    let mut sid_count: c_int;
    let mut i: c_int;
    let mut psd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    if apUserPerm.is_null() || cUserPerm <= 0 {
        return core::ptr::null_mut();
    }

    sidlist = HeapAlloc(GetProcessHeap(), 0, (cUserPerm as SIZE_T).wrapping_mul(core::mem::size_of::<PSID>() as SIZE_T)) as *mut PSID;
    if sidlist.is_null() {
        return core::ptr::null_mut();
    }

    acl_size = core::mem::size_of::<ACL>() as DWORD;

    sid_count = 0;
    'free_sids: {
        'error: {
            while sid_count < cUserPerm {
                static mut null_sid: SHELL_USER_SID = SHELL_USER_SID {
                    sidAuthority: SECURITY_NULL_SID_AUTHORITY,
                    dwUserGroupID: 0,
                    dwUserID: 0,
                };
                let perm: PSHELL_USER_PERMISSION = *apUserPerm.offset(sid_count as isize);
                let sid: PSHELL_USER_SID = core::ptr::addr_of_mut!((*perm).susID);
                let mut pSid: PSID = core::ptr::null_mut();
                let mut ret: BOOL = (TRUE as i32);

                if memcmp(sid as *const c_void, core::ptr::addr_of_mut!(null_sid) as *const c_void, core::mem::size_of::<SHELL_USER_SID>() as usize) == 0 {
                    if cur_user.is_null() {
                        let mut Token: HANDLE = core::ptr::null_mut();
                        let mut bufsize: DWORD = core::mem::size_of::<[BYTE; 2000]>() as DWORD;

                        ret = OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, core::ptr::addr_of_mut!(Token));
                        if ret != 0 {
                            ret = GetTokenInformation(Token, TokenUser, core::ptr::addr_of_mut!(tuUser) as *mut BYTE as PVOID, bufsize, core::ptr::addr_of_mut!(bufsize));
                            if ret != 0 {
                                cur_user = (*(core::ptr::addr_of_mut!(tuUser) as *mut BYTE as PTOKEN_USER)).User.Sid;
                            }
                            CloseHandle(Token);
                        }
                    }
                    pSid = cur_user;
                } else if (*sid).dwUserID == 0 {
                    ret = AllocateAndInitializeSid(core::ptr::addr_of_mut!((*sid).sidAuthority), 1, (*sid).dwUserGroupID, 0, 0, 0, 0, 0, 0, 0, core::ptr::addr_of_mut!(pSid));
                } else {
                    ret = AllocateAndInitializeSid(core::ptr::addr_of_mut!((*sid).sidAuthority), 2, (*sid).dwUserGroupID, (*sid).dwUserID, 0, 0, 0, 0, 0, 0, core::ptr::addr_of_mut!(pSid));
                }
                if ret == 0 {
                    break 'free_sids;
                }

                *sidlist.offset(sid_count as isize) = pSid;
                acl_size = acl_size.wrapping_add(((core::mem::size_of::<ACCESS_ALLOWED_ACE>() as DWORD).wrapping_sub(core::mem::size_of::<DWORD>() as DWORD).wrapping_add(GetLengthSid(pSid))).wrapping_mul((if (*perm).fInherit != 0 { 2 } else { 1 }) as DWORD));

                sid_count += 1;
            }

            psd = LocalAlloc(0, (core::mem::size_of::<_SECURITY_DESCRIPTOR>() as SIZE_T).wrapping_add(acl_size as SIZE_T)) as PSECURITY_DESCRIPTOR;

            if psd != core::ptr::null_mut() {
                let pAcl: PACL = (psd as *mut BYTE).offset(core::mem::size_of::<_SECURITY_DESCRIPTOR>() as isize) as PACL;

                if InitializeSecurityDescriptor(psd, SECURITY_DESCRIPTOR_REVISION) == 0 {
                    break 'error;
                }

                if InitializeAcl(pAcl, acl_size, ACL_REVISION) == 0 {
                    break 'error;
                }

                i = 0;
                while i < sid_count {
                    let sup: PSHELL_USER_PERMISSION = *apUserPerm.offset(i as isize);
                    let sid: PSID = *sidlist.offset(i as isize);

                    match (*sup).dwAccessType {
                        ACCESS_ALLOWED_ACE_TYPE => {
                            if AddAccessAllowedAce(pAcl, ACL_REVISION, (*sup).dwAccessMask, sid) == 0 {
                                break 'error;
                            }
                            if (*sup).fInherit != 0 && AddAccessAllowedAceEx(pAcl, ACL_REVISION, ((*sup).dwInheritMask as BYTE) as DWORD, (*sup).dwInheritAccessMask, sid) == 0 {
                                break 'error;
                            }
                        }
                        ACCESS_DENIED_ACE_TYPE => {
                            if AddAccessDeniedAce(pAcl, ACL_REVISION, (*sup).dwAccessMask, sid) == 0 {
                                break 'error;
                            }
                            if (*sup).fInherit != 0 && AddAccessDeniedAceEx(pAcl, ACL_REVISION, ((*sup).dwInheritMask as BYTE) as DWORD, (*sup).dwInheritAccessMask, sid) == 0 {
                                break 'error;
                            }
                        }
                        _ => {
                            break 'error;
                        }
                    }
                    i += 1;
                }

                if SetSecurityDescriptorDacl(psd, (TRUE) as BOOL, pAcl, (FALSE) as BOOL) == 0 {
                    break 'error;
                }
            }
            break 'free_sids;
        }
        // error:
        LocalFree(psd);
        psd = core::ptr::null_mut();
    }
    // free_sids:
    i = 0;
    while i < sid_count {
        if cur_user.is_null() || *sidlist.offset(i as isize) != cur_user {
            FreeSid(*sidlist.offset(i as isize));
        }
        i += 1;
    }
    HeapFree(GetProcessHeap(), 0, sidlist as PVOID);

    return psd;
}

#[no_mangle]
pub unsafe extern "system" fn SHFormatDateTimeW(
    fileTime: *const FILETIME,
    flags: *mut DWORD,
    buf: LPWSTR,
    size: UINT,
) -> INT {
    let mut fmt_flags: DWORD = if !flags.is_null() {
        *flags
    } else {
        FDTF_DEFAULT
    };
    let mut st: SYSTEMTIME = core::mem::zeroed();
    let mut ft: FILETIME = core::mem::zeroed();
    let mut ret: INT = 0;

    if buf.is_null() || size == 0 {
        return 0;
    }

    if (fmt_flags & SHFORMATDT_UNSUPPORTED_FLAGS) != 0 {
        FIXME!("ignoring some flags - 0x%08x\n", fmt_flags & SHFORMATDT_UNSUPPORTED_FLAGS);
    }

    FileTimeToLocalFileTime(fileTime, core::ptr::addr_of_mut!(ft));
    FileTimeToSystemTime(core::ptr::addr_of_mut!(ft), core::ptr::addr_of_mut!(st));

    /* first of all date */
    if (fmt_flags & (FDTF_LONGDATE | FDTF_SHORTDATE)) != 0 {
        static sep1: [u16; 3] = [0x002Cu16, 0x0020, 0x0000];
        static sep2: [u16; 2] = [0x0020u16, 0x0000];

        let date: DWORD = if (fmt_flags & FDTF_LONGDATE) != 0 {
            DATE_LONGDATE
        } else {
            DATE_SHORTDATE
        };
        ret = GetDateFormatW(
            LOCALE_USER_DEFAULT,
            date,
            core::ptr::addr_of_mut!(st),
            core::ptr::null_mut(),
            buf,
            size as INT,
        ) as INT;
        if (ret as UINT) >= size {
            return ret;
        }

        /* add separator */
        if (ret as UINT) < size && (fmt_flags & (FDTF_LONGTIME | FDTF_SHORTTIME)) != 0 {
            if (fmt_flags & FDTF_LONGDATE) != 0
                && (ret as UINT) < size.wrapping_add(2)
            {
                lstrcatW(
                    buf.offset((ret - 1) as isize),
                    core::ptr::addr_of!(sep1) as *const u16,
                );
                ret += 2;
            } else {
                lstrcatW(
                    buf.offset((ret - 1) as isize),
                    core::ptr::addr_of!(sep2) as *const u16,
                );
                ret += 1;
            }
        }
    }
    /* time part */
    if (fmt_flags & (FDTF_LONGTIME | FDTF_SHORTTIME)) != 0 {
        let time: DWORD = if (fmt_flags & FDTF_LONGTIME) != 0 {
            0
        } else {
            TIME_NOSECONDS
        };

        if ret != 0 {
            ret -= 1;
        }
        ret += GetTimeFormatW(
            LOCALE_USER_DEFAULT,
            time,
            core::ptr::addr_of_mut!(st),
            core::ptr::null_mut(),
            buf.offset(ret as isize),
            size.wrapping_sub(ret as UINT) as INT,
        ) as INT;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHFormatDateTimeA(
    fileTime: *const FILETIME,
    flags: *mut DWORD,
    buf: LPSTR,
    size: UINT,
) -> INT {
    let mut bufW: *mut WCHAR = core::ptr::null_mut();
    let mut retval: INT = 0;

    if buf.is_null() || size == 0 {
        return 0;
    }

    bufW = HeapAlloc(
        GetProcessHeap(),
        0,
        (core::mem::size_of::<WCHAR>() as SIZE_T).wrapping_mul(size as SIZE_T),
    ) as *mut WCHAR;
    retval = SHFormatDateTimeW(fileTime, flags, bufW as LPWSTR, size);

    if retval != 0 {
        WideCharToMultiByte(
            CP_ACP,
            0,
            bufW as LPCWSTR,
            -(1 as INT),
            buf,
            size as INT,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    HeapFree(
        GetProcessHeap(),
        0,
        bufW as PVOID,
    );
    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn SHVerbExistsNA(
    verb: LPSTR,
    pUnknown: PVOID,
    pUnknown2: PVOID,
    dwUnknown3: DWORD,
) -> INT {
    // FIXME("(%s, %p, %p, %i) STUB\n",verb, pUnknown, pUnknown2, dwUnknown3);
    0
}

#[no_mangle]
pub unsafe extern "system" fn IUnknown_QueryServiceForWebBrowserApp(lpUnknown: *mut IUnknown, riid: REFGUID, lppOut: *mut LPVOID) -> HRESULT {
    return IUnknown_QueryService(lpUnknown, core::ptr::addr_of!(IID_IWebBrowserApp), riid, lppOut);
}
#[no_mangle]
pub unsafe extern "C" fn VariantChangeTypeForRead(
    mut pvarg: *mut VARIANTARG,
    mut vt: VARTYPE,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut vargTemp: VARIANTARG = core::mem::zeroed();
    let mut variTemp: VARIANT = core::mem::zeroed();

    // V_VT(pvarg) == vt || vt == COMMENT_TOKEN_ECHO____
    if (*pvarg).n1.n2.vt as i32 == vt as i32 || vt as i32 == VT_EMPTY as i32 {
        return S_OK as HRESULT;
    }

    vargTemp = *pvarg;

    'DoDefault: {
        // if (V_VT(&vargTemp) != COMMENT_TOKEN_ECHO____ || vt <= COMMENT_TOKEN_ECHO____) goto DoDefault;
        if vargTemp.n1.n2.vt as i32 != VT_BSTR as i32
            || vt as i32 <= VT_NULL as i32
        {
            break 'DoDefault;
        }

        if vt as i32 == VT_I1 as i32
            || vt as i32 == VT_I2 as i32
            || vt as i32 == VT_I4 as i32
        {
            if StrToIntExW(
                vargTemp.n1.n2.n3.bstrVal as LPCWSTR,
                STIF_SUPPORT_HEX as DWORD,
                core::ptr::addr_of_mut!(variTemp.n1.n2.n3.lVal) as *mut i32,
            ) == 0
            {
                break 'DoDefault;
            }

            variTemp.n1.n2.vt = VT_INT as VARTYPE;
            VariantInit(pvarg as *mut VARIANT);
            hr = VariantChangeType(
                pvarg,
                core::ptr::addr_of_mut!(variTemp),
                0 as USHORT,
                vt,
            );
            VariantClear(core::ptr::addr_of_mut!(vargTemp));
            return hr;
        }

        if vt as i32 <= VT_DECIMAL as i32 {
            break 'DoDefault;
        }

        if vt as i32 == VT_UI1 as i32
            || vt as i32 == VT_UI2 as i32
            || vt as i32 == VT_UI4 as i32
        {
            if StrToIntExW(
                vargTemp.n1.n2.n3.bstrVal as LPCWSTR,
                STIF_SUPPORT_HEX as DWORD,
                core::ptr::addr_of_mut!(variTemp.n1.n2.n3.ulVal) as *mut i32,
            ) == 0
            {
                break 'DoDefault;
            }

            variTemp.n1.n2.vt = VT_UINT as VARTYPE;
            VariantInit(pvarg as *mut VARIANT);
            hr = VariantChangeType(
                pvarg,
                core::ptr::addr_of_mut!(variTemp),
                0 as USHORT,
                vt,
            );
            VariantClear(core::ptr::addr_of_mut!(vargTemp));
            return hr;
        }

        if vt as i32 == VT_INT as i32 || vt as i32 == VT_UINT as i32 {
            if StrToIntExW(
                vargTemp.n1.n2.n3.bstrVal as LPCWSTR,
                STIF_SUPPORT_HEX as DWORD,
                core::ptr::addr_of_mut!(variTemp.n1.n2.n3.intVal) as *mut i32,
            ) == 0
            {
                break 'DoDefault;
            }

            variTemp.n1.n2.vt = VT_UINT as VARTYPE;
            VariantInit(pvarg as *mut VARIANT);
            hr = VariantChangeType(
                pvarg,
                core::ptr::addr_of_mut!(variTemp),
                0 as USHORT,
                vt,
            );
            VariantClear(core::ptr::addr_of_mut!(vargTemp));
            return hr;
        }
    }

    // DoDefault:
    VariantInit(pvarg as *mut VARIANT);
    hr = VariantChangeType(
        pvarg,
        core::ptr::addr_of_mut!(vargTemp),
        0 as USHORT,
        vt,
    );
    VariantClear(core::ptr::addr_of_mut!(vargTemp));
    return hr;
}
#[no_mangle]
pub unsafe extern "C" fn VariantArrayToBuffer(
    pvarIn: *const VARIANT,
    pvDest: LPVOID,
    cbSize: SIZE_T,
) -> BOOL {
    let mut pvData: LPVOID = core::ptr::null_mut();
    let mut LowerBound: LONG = 0;
    let mut UpperBound: LONG = 0;
    let mut pArray: LPSAFEARRAY = core::ptr::null_mut();

    /* Only supports byte array */
    if pvarIn.is_null() || (V_VT!(pvarIn)) as i32 != (VT_UI1 | (VT_ARRAY as i32)) {
        return (FALSE) as BOOL;
    }

    /* Boundary check and access */
    pArray = V_ARRAY!(pvarIn);
    if SafeArrayGetDim(pArray) == 1
        && (SafeArrayGetLBound(pArray, 1, core::ptr::addr_of_mut!(LowerBound)))
            >= 0
        && (SafeArrayGetUBound(pArray, 1, core::ptr::addr_of_mut!(UpperBound)))
            >= 0
        && ((cbSize as LONG) <= UpperBound - LowerBound + 1)
        && (SafeArrayAccessData(pArray, core::ptr::addr_of_mut!(pvData))) >= 0
    {
        CopyMemory!(pvDest, pvData, cbSize);
        SafeArrayUnaccessData(pArray);
        return (TRUE) as BOOL; /* Success */
    }

    return (FALSE) as BOOL; /* Failure */
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadType(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pvarg: *mut VARIANTARG, vt: VARTYPE) -> HRESULT {
    let mut hr: HRESULT;

    VariantInit(pvarg);
    (*pvarg).n1.n2.vt = vt;

    hr = IPropertyBag_Read(ppb, pszPropName, pvarg, core::ptr::null_mut());
    if hr < 0 {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        VariantInit(pvarg);
        return hr;
    }

    return VariantChangeTypeForRead(pvarg, vt);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadBOOL(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    pbValue: *mut BOOL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pbValue.is_null() {
        ERR!(
            "%p %s %p\n",
            ppb,
            debugstr_w(pszPropName),
            pbValue
        );
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(
        ppb,
        pszPropName,
        core::ptr::addr_of_mut!(varg),
        (VT_BOOL) as VARTYPE,
    );
    if hr >= 0 {
        *pbValue = (V_BOOL!(core::ptr::addr_of_mut!(varg)) == VARIANT_TRUE) as BOOL;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadBOOLOld(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, bDefValue: BOOL) -> BOOL {
    let mut varg: VARIANTARG = core::mem::zeroed();
    let mut hr: HRESULT;

    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(varg), (VT_BOOL) as VARTYPE);
    if hr < 0 {
        return bDefValue;
    }

    return (V_BOOL!(core::ptr::addr_of_mut!(varg)) == VARIANT_TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadSHORT(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, psValue: *mut SHORT) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || psValue.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), psValue);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(varg), VT_UI2 as VARTYPE);
    if hr >= 0 {
        *psValue = V_UI2!(core::ptr::addr_of_mut!(varg)) as SHORT;
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadLONG(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pValue: LPLONG) -> HRESULT {
    let mut hr: HRESULT;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pValue.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pValue);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, &raw mut varg, (VT_I4) as VARTYPE);
    if hr >= 0 {
        *pValue = V_I4!(&raw mut varg);
    }
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadDWORD(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pdwValue: *mut DWORD) -> HRESULT {
    let mut hr: HRESULT;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pdwValue.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pdwValue);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(varg), (VT_UI4) as VARTYPE);
    if hr >= 0 {
        *pdwValue = V_UI4!(core::ptr::addr_of_mut!(varg));
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadBSTR(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pbstr: *mut BSTR) -> HRESULT {
    let hr: HRESULT;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pbstr.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pbstr);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(varg), (VT_BSTR) as VARTYPE);
    if hr < 0 {
        *pbstr = core::ptr::null_mut();
    } else {
        *pbstr = V_BSTR!(core::ptr::addr_of_mut!(varg));
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadStr(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pszDst: LPWSTR, cchMax: i32) -> HRESULT {
    let mut hr: HRESULT;
    let mut varg: VARIANTARG = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pszDst.is_null() {
        ERR!("%p %s %p %d\n", ppb, debugstr_w(pszPropName), pszDst, cchMax);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(varg), (VT_BSTR) as VARTYPE);
    if hr < 0 {
        return E_FAIL;
    }

    StrCpyNW(pszDst, V_BSTR!(core::ptr::addr_of_mut!(varg)), cchMax);
    VariantClear(core::ptr::addr_of_mut!(varg));
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadPOINTL(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    pptl: *mut POINTL,
) -> HRESULT {
    let mut hr: HRESULT;
    let mut cch: i32;
    let mut cch2: i32;
    let mut pch: *mut WCHAR;
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if ppb.is_null() || pszPropName.is_null() || pptl.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pptl);
        return E_INVALIDARG;
    }

    StrCpyNW(
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPWSTR,
        pszPropName,
        _countof!(szBuff) as i32,
    );

    cch = lstrlenW(core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR);
    cch2 = _countof!(szBuff) as i32 - cch;
    if cch2 < _countof!(RUSTCONV_WSTR_62D9010A526D) as i32 {
        ERR!("%s is too long\n", debugstr_w(pszPropName));
        return E_FAIL;
    }

    pch = (core::ptr::addr_of_mut!(szBuff) as *mut WCHAR).offset(cch as isize);

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_62D9010A526D.as_ptr(), cch2);
    hr = SHPropertyBag_ReadLONG(
        ppb,
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!((*pptl).x),
    );
    if hr < 0 {
        return hr;
    }

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_433FA3D0669D.as_ptr(), cch2);
    return SHPropertyBag_ReadLONG(
        ppb,
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!((*pptl).y),
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadPOINTS(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, ppts: *mut POINTS) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut ptl: POINTL = POINTL { x: 0, y: 0 };
    if ppb.is_null() || pszPropName.is_null() || ppts.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), ppts);
        return E_INVALIDARG;
    }
    hr = SHPropertyBag_ReadPOINTL(ppb, pszPropName, core::ptr::addr_of_mut!(ptl));
    if hr < 0 {
        return hr;
    }
    (*ppts).x = ptl.x as SHORT;
    (*ppts).y = ptl.y as SHORT;
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadRECTL(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, prcl: *mut RECTL) -> HRESULT {
    let mut hr: HRESULT;
    let mut cch: ::core::ffi::c_int;
    let mut cch2: ::core::ffi::c_int;
    let mut pch: *mut WCHAR;
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if ppb.is_null() || pszPropName.is_null() || prcl.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), prcl);
        return E_INVALIDARG;
    }

    StrCpyNW(&raw mut szBuff as LPWSTR, pszPropName, _countof!(szBuff) as ::core::ffi::c_int);

    cch = lstrlenW(&raw mut szBuff as *mut WCHAR as LPCWSTR);
    cch2 = (_countof!(szBuff) as ::core::ffi::c_int) - cch;
    if cch2 < (_countof!(RUSTCONV_WSTR_20C44FBAF751) as ::core::ffi::c_int) {
        ERR!("%s is too long\n", debugstr_w(pszPropName));
        return E_FAIL;
    }

    pch = (&raw mut szBuff as *mut WCHAR).offset(cch as isize) as *mut WCHAR;

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_E744B4C578CD.as_ptr(), cch2);
    hr = SHPropertyBag_ReadLONG(ppb, &raw mut szBuff as *mut WCHAR as LPCWSTR, &raw mut (*prcl).left);
    if hr < 0 {
        return hr;
    }

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_55366762880C.as_ptr(), cch2);
    hr = SHPropertyBag_ReadLONG(ppb, &raw mut szBuff as *mut WCHAR as LPCWSTR, &raw mut (*prcl).top);
    if hr < 0 {
        return hr;
    }

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_33D489E223A7.as_ptr(), cch2);
    hr = SHPropertyBag_ReadLONG(ppb, &raw mut szBuff as *mut WCHAR as LPCWSTR, &raw mut (*prcl).right);
    if hr < 0 {
        return hr;
    }

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_20C44FBAF751.as_ptr(), cch2);
    return SHPropertyBag_ReadLONG(ppb, &raw mut szBuff as *mut WCHAR as LPCWSTR, &raw mut (*prcl).bottom);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadGUID(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pguid: *mut GUID) -> HRESULT {
    let mut hr: HRESULT;
    let mut bRet: BOOL;
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pguid.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName as *const WCHAR), pguid);
        return E_INVALIDARG;
    }

    hr = SHPropertyBag_ReadType(ppb, pszPropName, &raw mut vari, VT_EMPTY as VARTYPE);
    if hr < 0 {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName as *const WCHAR), pguid);
        return hr;
    }

    if V_VT!(&raw mut vari) == (VT_UI1 as VARTYPE | VT_ARRAY as VARTYPE) {
        /* Byte Array */
        bRet = VariantArrayToBuffer(&raw mut vari, pguid as LPVOID, core::mem::size_of::<GUID>() as SIZE_T);
    } else if V_VT!(&raw mut vari) == VT_BSTR as VARTYPE {
        bRet = GUIDFromStringW(V_BSTR!(&raw mut vari) as LPCWSTR, pguid as *mut CLSID);
    } else {
        // `#if (_WIN32_WINNT >= _WIN32_WINNT_VISTA)` is dead at 0x502; the #else arm: "This is
        // by design in WinXP/Win2k3." (The bought body took the VISTA arm's FALSE.)
        bRet = TRUE as BOOL;
    }

    if bRet == 0 {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName as *const WCHAR), pguid);
    }

    VariantClear(&raw mut vari);
    return if bRet != 0 { S_OK } else { E_FAIL };
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_ReadStream(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, ppStream: *mut *mut IStream) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut vari: VARIANT = core::mem::zeroed();
    if ppb.is_null() || pszPropName.is_null() || ppStream.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), ppStream);
        return E_INVALIDARG;
    }
    hr = SHPropertyBag_ReadType(ppb, pszPropName, core::ptr::addr_of_mut!(vari), (VT_UNKNOWN) as VARTYPE);
    if hr < 0 {
        return hr;
    }
    hr = IUnknown_QueryInterface(V_UNKNOWN!(core::ptr::addr_of_mut!(vari)), core::ptr::addr_of!(IID_IStream), ppStream as *mut *mut core::ffi::c_void);
    IUnknown_Release(V_UNKNOWN!(core::ptr::addr_of_mut!(vari)));
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_Delete(ppb: *mut IPropertyBag, pszPropName: LPCWSTR) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        return E_INVALIDARG;
    }

    V_VT!(core::ptr::addr_of_mut!(vari)) = VT_EMPTY as VARTYPE;
    return IPropertyBag_Write(ppb, pszPropName, core::ptr::addr_of_mut!(vari));
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteBOOL(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, bValue: BOOL) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        return E_INVALIDARG;
    }

    V_VT!(core::ptr::addr_of_mut!(vari)) = (VT_BOOL) as u16;
    V_BOOL!(core::ptr::addr_of_mut!(vari)) = if bValue != 0 { VARIANT_TRUE } else { VARIANT_FALSE };
    return IPropertyBag_Write(ppb, pszPropName, core::ptr::addr_of_mut!(vari));
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteSHORT(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, sValue: SHORT) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        return E_INVALIDARG;
    }

    V_VT!(&raw mut vari) = VT_UI2 as VARTYPE;
    V_UI2!(&raw mut vari) = sValue as USHORT;
    return IPropertyBag_Write(ppb, pszPropName as LPCOLESTR, &raw mut vari);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteLONG(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, lValue: LONG) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        return E_INVALIDARG;
    }

    V_VT!(core::ptr::addr_of_mut!(vari)) = VT_I4 as VARTYPE;
    V_I4!(core::ptr::addr_of_mut!(vari)) = lValue;
    return IPropertyBag_Write(ppb, pszPropName, core::ptr::addr_of_mut!(vari));
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteDWORD(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, dwValue: DWORD) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName));
        return E_INVALIDARG;
    }

    V_VT!(&raw mut vari) = VT_UI4 as VARTYPE;
    V_UI4!(&raw mut vari) = dwValue;
    return IPropertyBag_Write(ppb, pszPropName as LPCOLESTR, &raw mut vari);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteStr(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, pszValue: LPCWSTR) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() {
        ERR!("%p %s\n", ppb, debugstr_w(pszPropName as *const WCHAR));
        return E_INVALIDARG;
    }

    vari.n1.n2.n3.bstrVal = SysAllocString(pszValue as *const OLECHAR);
    if vari.n1.n2.n3.bstrVal.is_null() {
        return E_OUTOFMEMORY;
    }

    vari.n1.n2.vt = VT_BSTR as VARTYPE;
    hr = IPropertyBag_Write(ppb, pszPropName as LPCOLESTR, core::ptr::addr_of_mut!(vari));

    SysFreeString(vari.n1.n2.n3.bstrVal);
    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteGUID(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    pguid: *const GUID,
) -> HRESULT {
    let mut szBuff: [WCHAR; 64] = [0; 64];

    if ppb.is_null() || pszPropName.is_null() || pguid.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pguid);
        return E_INVALIDARG;
    }

    SHStringFromGUIDW(pguid, core::ptr::addr_of_mut!(szBuff) as LPWSTR, _countof!(szBuff) as INT);
    return SHPropertyBag_WriteStr(ppb, pszPropName, core::ptr::addr_of_mut!(szBuff) as LPCWSTR);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteStream(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    pStream: *mut IStream,
) -> HRESULT {
    let mut vari: VARIANT = core::mem::zeroed();

    if ppb.is_null() || pszPropName.is_null() || pStream.is_null() {
        ERR!(
            "%p %s %p\n",
            ppb,
            debugstr_w(pszPropName as *const WCHAR),
            pStream
        );
        return E_INVALIDARG;
    }

    V_VT!(&raw mut vari) = VT_UNKNOWN as VARTYPE;
    V_UNKNOWN!(&raw mut vari) = pStream as *mut IUnknown;
    return IPropertyBag_Write(ppb, pszPropName, &raw mut vari);
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WritePOINTL(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    pptl: *const POINTL,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut cch: core::ffi::c_int = 0;
    let mut cch2: core::ffi::c_int = 0;
    let mut pch: *mut WCHAR = core::ptr::null_mut();
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if ppb.is_null() || pszPropName.is_null() || pptl.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName), pptl);
        return E_INVALIDARG;
    }

    StrCpyNW(
        (&raw mut szBuff) as *mut WCHAR as LPWSTR,
        pszPropName,
        _countof!(szBuff) as core::ffi::c_int,
    );

    cch = lstrlenW((&raw mut szBuff) as *mut WCHAR as LPCWSTR);
    cch2 = _countof!(szBuff) as core::ffi::c_int - cch;
    if cch2 < _countof!(RUSTCONV_WSTR_62D9010A526D) as core::ffi::c_int {
        ERR!("%s is too long\n", debugstr_w(pszPropName));
        return E_FAIL;
    }

    pch = ((&raw mut szBuff) as *mut WCHAR).offset(cch as isize);

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_62D9010A526D.as_ptr() as LPCWSTR, cch2);
    hr = SHPropertyBag_WriteLONG(ppb, (&raw mut szBuff) as *mut WCHAR as LPCWSTR, (*pptl).x);
    if hr < 0 {
        return hr;
    }

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_433FA3D0669D.as_ptr() as LPCWSTR, cch2);
    hr = SHPropertyBag_WriteLONG(ppb, (&raw mut szBuff) as *mut WCHAR as LPCWSTR, (*pptl).y);
    if hr < 0 {
        StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_62D9010A526D.as_ptr() as LPCWSTR, cch2);
        return SHPropertyBag_Delete(ppb, (&raw mut szBuff) as *mut WCHAR as LPCWSTR);
    }

    hr
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WritePOINTS(ppb: *mut IPropertyBag, pszPropName: LPCWSTR, ppts: *const POINTS) -> HRESULT {
    let mut pt: POINTL = POINTL { x: 0, y: 0 };
    if ppb.is_null() || pszPropName.is_null() || ppts.is_null() {
        ERR!("%p %s %p\n", ppb, debugstr_w(pszPropName as *const WCHAR), ppts);
        return E_INVALIDARG;
    }
    pt.x = (*ppts).x as LONG;
    pt.y = (*ppts).y as LONG;
    return SHPropertyBag_WritePOINTL(ppb, pszPropName, core::ptr::addr_of_mut!(pt));
}

#[no_mangle]
pub unsafe extern "system" fn SHPropertyBag_WriteRECTL(
    ppb: *mut IPropertyBag,
    pszPropName: LPCWSTR,
    prcl: *const RECTL,
) -> HRESULT {
    let mut hr: HRESULT = 0;
    let mut cch: ::core::ffi::c_int = 0;
    let mut cch2: ::core::ffi::c_int = 0;
    let mut pch: *mut WCHAR = core::ptr::null_mut();
    let mut szBuff: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if ppb.is_null() || pszPropName.is_null() || prcl.is_null() {
        ERR!(
            "%p %s %p\n",
            ppb,
            debugstr_w(pszPropName),
            prcl
        );
        return E_INVALIDARG;
    }

    StrCpyNW(
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPWSTR,
        pszPropName,
        _countof!(szBuff) as ::core::ffi::c_int,
    );

    cch = lstrlenW(core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR);
    cch2 = _countof!(szBuff) as ::core::ffi::c_int - cch;
    if cch2 < _countof!(RUSTCONV_WSTR_20C44FBAF751) as ::core::ffi::c_int {
        ERR!(
            "%s is too long\n",
            debugstr_w(pszPropName)
        );
        return E_FAIL;
    }

    pch = (core::ptr::addr_of_mut!(szBuff) as *mut WCHAR).offset(cch as isize);

    StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_E744B4C578CD.as_ptr(), cch2);
    hr = SHPropertyBag_WriteLONG(
        ppb,
        core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
        (*prcl).left,
    );
    if hr >= 0 {
        StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_55366762880C.as_ptr(), cch2);
        hr = SHPropertyBag_WriteLONG(
            ppb,
            core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
            (*prcl).top,
        );
        if hr >= 0 {
            StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_33D489E223A7.as_ptr(), cch2);
            hr = SHPropertyBag_WriteLONG(
                ppb,
                core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
                (*prcl).right,
            );
            if hr >= 0 {
                StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_20C44FBAF751.as_ptr(), cch2);
                hr = SHPropertyBag_WriteLONG(
                    ppb,
                    core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
                    (*prcl).bottom,
                );
                if hr >= 0 {
                    return hr; /* All successful */
                }

                StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_33D489E223A7.as_ptr(), cch2);
                hr = SHPropertyBag_Delete(
                    ppb,
                    core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
                );
                if hr >= 0 {
                    return hr;
                }
            }

            StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_55366762880C.as_ptr(), cch2);
            hr = SHPropertyBag_Delete(
                ppb,
                core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
            );
            if hr >= 0 {
                return hr;
            }
        }

        StrCpyNW(pch as LPWSTR, RUSTCONV_WSTR_E744B4C578CD.as_ptr(), cch2);
        hr = SHPropertyBag_Delete(
            ppb,
            core::ptr::addr_of_mut!(szBuff) as *mut WCHAR as LPCWSTR,
        );
        if hr >= 0 {
            return hr;
        }
    }

    return hr;
}

#[no_mangle]
pub unsafe extern "system" fn SHGetObjectCompatFlags(pUnk: *mut IUnknown, clsid: *const CLSID) -> DWORD {
    static compatpathW: [u16; 72] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x004D,
    0x0069, 0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0057,
    0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x0073, 0x005C, 0x0043, 0x0075, 0x0072,
    0x0072, 0x0065, 0x006E, 0x0074, 0x0056, 0x0065, 0x0072, 0x0073, 0x0069, 0x006F,
    0x006E, 0x005C, 0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0043, 0x006F, 0x006D,
    0x0070, 0x0061, 0x0074, 0x0069, 0x0062, 0x0069, 0x006C, 0x0069, 0x0074, 0x0079,
    0x005C, 0x004F, 0x0062, 0x006A, 0x0065, 0x0063, 0x0074, 0x0073, 0x005C, 0x0025,
    0x0073, 0x0000,
];
    let mut strW: [WCHAR; 72 + 38] = [0; 72 + 38];
    let mut ret: DWORD = 0;
    let mut length: DWORD = (core::mem::size_of::<[WCHAR; 72 + 38]>()
        / core::mem::size_of::<WCHAR>()) as DWORD;
    let mut clsid_str: *mut OLECHAR = core::ptr::null_mut();
    let mut key: HKEY = core::ptr::null_mut();
    let mut i: INT = 0;

    if pUnk.is_null() && clsid.is_null() {
        return 0;
    }

    if !pUnk.is_null() && clsid.is_null() {
        return 0;
    }

    StringFromCLSID(clsid, core::ptr::addr_of_mut!(clsid_str));
    sprintfW(
        core::ptr::addr_of_mut!(strW) as *mut WCHAR,
        core::ptr::addr_of!(compatpathW) as *const WCHAR,
        clsid_str,
    );
    CoTaskMemFree(clsid_str as LPVOID);

    ret = RegOpenKeyW(
        HKEY_LOCAL_MACHINE,
        core::ptr::addr_of_mut!(strW) as *mut WCHAR as LPCWSTR,
        core::ptr::addr_of_mut!(key),
    ) as DWORD;
    if ret != ERROR_SUCCESS {
        return 0;
    }

    /* now collect flag values */
    ret = 0;
    i = 0;
    while RegEnumValueW(
        key,
        i as DWORD,
        core::ptr::addr_of_mut!(strW) as LPWSTR,
        core::ptr::addr_of_mut!(length),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<DWORD>(),
        core::ptr::null_mut::<BYTE>(),
        core::ptr::null_mut::<DWORD>(),
    ) == (ERROR_SUCCESS as i32)
    {
        let mut left: INT;
        let mut right: INT;
        let mut res: INT;
        let mut x: INT;

        /* search in table */
        left = 0;
        right = (core::mem::size_of::<[objcompat_entry; 13]>()
            / core::mem::size_of::<objcompat_entry>()) as INT - 1;

        while right >= left {
            x = (left + right) / 2;
            res = strcmpW!(
                core::ptr::addr_of_mut!(strW) as *mut WCHAR,
                core::ptr::addr_of!(objcompat_table[x as usize].name) as *const WCHAR,
            );
            if res == 0 {
                ret |= objcompat_table[x as usize].value;
                break;
            } else if res < 0 {
                right = x - 1;
            } else {
                left = x + 1;
            }
        }

        length = (core::mem::size_of::<[WCHAR; 72 + 38]>()
            / core::mem::size_of::<WCHAR>()) as DWORD;
        i += 1;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SHBoolSystemParametersInfo(uiAction: UINT, mut pvParam: PVOID) -> BOOL {
    let mut retval: BOOL = 0;
    let mut pvOrgParam: PVOID = pvParam;
    let mut uiParam: UINT = 0;
    let mut animinfo: ANIMATIONINFO = core::mem::zeroed();

    if uiAction == SPI_GETANIMATION || uiAction == SPI_SETANIMATION {
        animinfo.cbSize = core::mem::size_of::<ANIMATIONINFO>() as UINT;
        uiParam = animinfo.cbSize;
        animinfo.iMinAnimate = *(pvParam as *mut BOOL); /* SPI_SET */
        pvParam = core::ptr::addr_of_mut!(animinfo) as PVOID;
    } else if uiAction == SPI_SETDRAGFULLWINDOWS || uiAction == SPI_SETFONTSMOOTHING {
        uiParam = *(pvParam as *mut BOOL) as UINT;
    } else if uiAction == SPI_GETDRAGFULLWINDOWS || uiAction == SPI_GETFONTSMOOTHING {
        /* pvParam already correct */
    } else {
        if uiAction < 0x1000 {
            return (FALSE) as BOOL;
        } else if (uiAction & 1) != 0 {
            /* SPI_SET */
            pvParam = *(pvParam as *mut BOOL) as SIZE_T as PVOID;
        }
    }

    retval = SystemParametersInfoW(
        uiAction,
        uiParam,
        pvParam,
        (SPIF_SENDCHANGE | SPIF_UPDATEINIFILE) as UINT,
    );
    if uiAction == SPI_GETANIMATION {
        *(pvOrgParam as *mut BOOL) = animinfo.iMinAnimate as BOOL;
    }
    return retval;
}
