// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_2366E523D254: [u16; 17] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047,
    0x005F, 0x004C, 0x0045, 0x0056, 0x0045, 0x004C, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_2366E523D254: [u16; 17] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x005F, 0x0044, 0x0045, 0x0042, 0x0055, 0x0047,
    0x005F, 0x004C, 0x0045, 0x0056, 0x0045, 0x004C, 0x0000,
];

static mut DebugKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 17]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 17]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_2366E523D254.as_ptr() as PWSTR,
    };

#[no_mangle]
pub static mut g_ShimEngDebugLevel: ULONG = 0xffffffff as ULONG;

static mut g_ShimLib_hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut g_ShimLib_Heap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut g_UsedShims: PSLIST_HEADER = ::core::ptr::null::<_SLIST_HEADER>() as *mut _SLIST_HEADER;

pub static mut _shim_start: SHIMREG = tagSHIMREG {
    GetHookAPIs: None,
    Notify: None,
    ShimName: ::core::ptr::null::<CHAR>(),
};

pub static mut _shim_end: SHIMREG = tagSHIMREG {
    GetHookAPIs: None,
    Notify: None,
    ShimName: ::core::ptr::null::<CHAR>(),
};

pub type pUsedShim = *mut UsedShim;

#[repr(C)]
pub struct UsedShim {
    pub Entry: SLIST_ENTRY,
    pub pShim: PSHIMREG,
}

// #pragma section(".shm", long, read) / ".shm$AAA" / ".shm$ZZZ" and
// #pragma comment(linker, "/merge:.shm=.rdata") have no Rust equivalent;
// the section names are applied via link_section below.
#[no_mangle]
pub unsafe extern "C" fn ShimLib_Init(hInstance: HINSTANCE) {
    g_ShimLib_hInstance = hInstance;
    g_ShimLib_Heap = HeapCreate(0, 0x10000, 0);

    g_UsedShims = ShimLib_ShimMalloc(core::mem::size_of::<SLIST_HEADER>() as SIZE_T) as PSLIST_HEADER;
    RtlInitializeSListHead(g_UsedShims);
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_Deinit() {
    // Is this a good idea?
    HeapDestroy(g_ShimLib_Heap);
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_ShimMalloc(dwSize: SIZE_T) -> PVOID {
    return HeapAlloc(g_ShimLib_Heap, 0, dwSize);
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_ShimFree(mut pData: PVOID) {
    HeapFree(g_ShimLib_Heap, 0, pData);
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_Instance() -> HINSTANCE {
    return g_ShimLib_hInstance;
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_StringNDuplicateA(szString: PCSTR, stringLengthIncludingNullTerm: SIZE_T) -> PCSTR {
    let NewString: PSTR = ShimLib_ShimMalloc(stringLengthIncludingNullTerm) as PSTR;
    StringCchCopyA(
        NewString as STRSAFE_LPSTR,
        stringLengthIncludingNullTerm as size_t,
        szString as STRSAFE_LPCSTR,
    );
    return NewString as PCSTR;
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_StringDuplicateA(mut szString: PCSTR) -> PCSTR {
    return ShimLib_StringNDuplicateA(szString, (lstrlenA(szString as LPCSTR) + 1) as SIZE_T);
}
#[no_mangle]
pub unsafe extern "C" fn ShimLib_StrAEqualsWNC(mut szString: PCSTR, mut wszString: PCWSTR) -> BOOL {
    while toupper(*szString as ::core::ffi::c_int) == towupper(*wszString) as ::core::ffi::c_int {
        if *szString == 0 {
            return (TRUE) as BOOL;
        }
        szString = szString.offset(1);
        wszString = wszString.offset(1);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn ShimLib_GetHookAPIs(
    szCommandLine: LPCSTR,
    wszShimName: LPCWSTR,
    pdwHookCount: PDWORD,
) -> PHOOKAPI {
    let mut ps: PSHIMREG = core::ptr::addr_of_mut!(_shim_start);
    ps = ps.offset(1);
    while ps < core::ptr::addr_of_mut!(_shim_end) {
        if (*ps).GetHookAPIs.is_some() && !(*ps).ShimName.is_null() {
            if ShimLib_StrAEqualsWNC((*ps).ShimName, wszShimName as PCWSTR) != 0 {
                let shim: pUsedShim =
                    ShimLib_ShimMalloc(core::mem::size_of::<UsedShim>() as SIZE_T) as pUsedShim;
                (*shim).pShim = ps;
                RtlInterlockedPushEntrySList(g_UsedShims, core::ptr::addr_of_mut!((*shim).Entry));

                return (*ps).GetHookAPIs.unwrap_unchecked()(
                    SHIM_NOTIFY_ATTACH as DWORD,
                    szCommandLine as PCSTR,
                    pdwHookCount,
                );
            }
        }
        ps = ps.offset(1);
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn ShimLib_NotifyShims(fdwReason: DWORD, ptr: PVOID) -> BOOL {
    let mut fdwReason = fdwReason;
    let mut pEntry: PSLIST_ENTRY = RtlFirstEntrySList(g_UsedShims as *const SLIST_HEADER);

    if fdwReason < SHIM_REASON_INIT {
        fdwReason = fdwReason.wrapping_add((SHIM_REASON_INIT - SHIM_NOTIFY_ATTACH) as DWORD);
    }

    while !pEntry.is_null() {
        let mut pUsed: pUsedShim = ((pEntry as usize)
            .wrapping_sub(core::mem::offset_of!(UsedShim, Entry)))
            as pUsedShim;
        let mut Notify: _PVNotify = (*(*pUsed).pShim).Notify;
        if Notify.is_some() {
            (Notify.unwrap_unchecked())(fdwReason, ptr);
        }

        pEntry = (*pEntry).Next as PSLIST_ENTRY;
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SeiInitDebugSupport() {
    // RTL_CONSTANT_STRING(L"SHIM_DEBUG_LEVEL"), spelled out: the shared macro's WideSource
    // calls are not const, and this is a static.
    static mut DebugKey: _UNICODE_STRING = _UNICODE_STRING {
        Length: ((RUSTCONV_WSTR_2366E523D254.len() - 1) * 2) as u16,
        MaximumLength: (RUSTCONV_WSTR_2366E523D254.len() * 2) as u16,
        Buffer: RUSTCONV_WSTR_2366E523D254.as_ptr() as *mut u16,
    };
    let mut DebugValue: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: i32;
    let mut NewLevel: u32 = SEI_MSG as u32;
    let mut Buffer: [u16; 40] = [0; 40];

    RtlInitEmptyUnicodeString(
        &raw mut DebugValue,
        &raw mut Buffer as *mut u16,
        core::mem::size_of::<[u16; 40]>() as u16,
    );

    Status = RtlQueryEnvironmentVariable_U(
        core::ptr::null_mut(),
        &raw const DebugKey,
        &raw mut DebugValue,
    );

    if NT_SUCCESS!(Status) {
        if !NT_SUCCESS!(RtlUnicodeStringToInteger(
            &raw mut DebugValue as *const _UNICODE_STRING,
            10 as u32,
            &raw mut NewLevel,
        )) {
            NewLevel = 0 as u32;
        }
    }
    g_ShimEngDebugLevel = NewLevel;
}
#[no_mangle]
pub unsafe extern "C" fn SeiDbgPrint_v(mut Level: SEI_LOG_LEVEL, mut Function: PCSTR, mut Format: PCSTR, ap_list: *mut ::core::ffi::c_void) -> BOOL {
    let mut Buffer: [::core::ffi::c_char; 512] = [0 as ::core::ffi::c_char; 512];
    let mut Current: *mut ::core::ffi::c_char = Buffer.as_mut_ptr();
    let mut LevelStr: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut Length: size_t = ::core::mem::size_of::<[::core::ffi::c_char; 512]>() as size_t;
    let mut ArgList: va_list = *(ap_list as *mut va_list);
    let mut hr: HRESULT = 0;

    if g_ShimEngDebugLevel == 0xffffffff {
        SeiInitDebugSupport();
    }

    if Level as ULONG > g_ShimEngDebugLevel {
        return (FALSE) as BOOL;
    }

    if Level as ULONG == SEI_MSG {
        LevelStr = b"MSG \0".as_ptr() as *const ::core::ffi::c_char;
    } else if Level as ULONG == SEI_FAIL {
        LevelStr = b"FAIL\0".as_ptr() as *const ::core::ffi::c_char;
    } else if Level as ULONG == SEI_WARN {
        LevelStr = b"WARN\0".as_ptr() as *const ::core::ffi::c_char;
    } else if Level as ULONG == SEI_INFO {
        LevelStr = b"INFO\0".as_ptr() as *const ::core::ffi::c_char;
    } else {
        LevelStr = b"USER\0".as_ptr() as *const ::core::ffi::c_char;
    }

    if !Function.is_null() {
        hr = StringCchPrintfExA!(
            Current as STRSAFE_LPSTR,
            Length,
            core::ptr::addr_of_mut!(Current),
            core::ptr::addr_of_mut!(Length),
            STRSAFE_NULL_ON_FAILURE as STRSAFE_DWORD,
            b"[%s] [%s] \0".as_ptr() as STRSAFE_LPCSTR,
            LevelStr,
            Function,
        );
    } else {
        hr = StringCchPrintfExA!(
            Current as STRSAFE_LPSTR,
            Length,
            core::ptr::addr_of_mut!(Current),
            core::ptr::addr_of_mut!(Length),
            STRSAFE_NULL_ON_FAILURE as STRSAFE_DWORD,
            b"[%s] \0".as_ptr() as STRSAFE_LPCSTR,
            LevelStr,
        );
    }

    if !(hr >= 0) {
        return (FALSE) as BOOL;
    }


    hr = StringCchVPrintfExA!(
        Current as STRSAFE_LPSTR,
        Length,
        core::ptr::addr_of_mut!(Current),
        core::ptr::addr_of_mut!(Length),
        STRSAFE_NULL_ON_FAILURE as STRSAFE_DWORD,
        Format as STRSAFE_LPCSTR,
        ArgList,
    );

    if !(hr >= 0) {
        return (FALSE) as BOOL;
    }

    DbgPrint(
        b"%s\0".as_ptr() as PCSTR,
        Buffer.as_mut_ptr(),
    );
    return (TRUE) as BOOL;
}
