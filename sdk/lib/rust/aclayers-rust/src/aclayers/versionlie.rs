use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut g_Win95: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xc3b60004 as DWORD,
    dwMajorVersion: 4 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 950 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_WINDOWS as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinNT4SP5: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x5650004 as DWORD,
    dwMajorVersion: 4 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 1381 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 5 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win98: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xc0000a04 as DWORD,
    dwMajorVersion: 4 as WORD,
    dwMinorVersion: 10 as WORD,
    dwBuildNumber: 0x40a08ae as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_WINDOWS as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2000: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x8930005 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 2195 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2000SP1: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x8930005 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 2195 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 1 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2000SP2: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x8930005 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 2195 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 2 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2000SP3: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x8930005 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 2195 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 3 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinXP: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xa280105 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 2600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinXPSP1: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xa280105 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 2600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 1 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinXPSP2: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xa280105 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 2600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 2 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinXPSP3: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xa280105 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 2600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 3 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2k3RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xece0205 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 2 as WORD,
    dwBuildNumber: 3790 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2k3SP1: VersionLieInfo = VersionLieInfo {
    FullVersion: 0xece0205 as DWORD,
    dwMajorVersion: 5 as WORD,
    dwMinorVersion: 2 as WORD,
    dwBuildNumber: 3790 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 1 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinVistaRTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x17700006 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 6000 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinVistaSP1: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x17710006 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 6001 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 1 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_WinVistaSP2: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x17720006 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 6002 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 2 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win7RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x1db00106 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 7600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win7SP1: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x1db10106 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 1 as WORD,
    dwBuildNumber: 7601 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 1 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win8RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x23f00206 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 2 as WORD,
    dwBuildNumber: 9200 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win81RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x25800306 as DWORD,
    dwMajorVersion: 6 as WORD,
    dwMinorVersion: 3 as WORD,
    dwBuildNumber: 9600 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win10RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x47ba000a as DWORD,
    dwMajorVersion: 10 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 18362 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2k16RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x3fab000a as DWORD,
    dwMajorVersion: 10 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 16299 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

#[no_mangle]
pub static mut g_Win2k19RTM: VersionLieInfo = VersionLieInfo {
    FullVersion: 0x4563000a as DWORD,
    dwMajorVersion: 10 as WORD,
    dwMinorVersion: 0 as WORD,
    dwBuildNumber: 17763 as DWORD,
    dwPlatformId: VER_PLATFORM_WIN32_NT as WORD,
    wServicePackMajor: 0 as WORD,
    wServicePackMinor: 0 as WORD,
    szCSDVersionA: [0; 20],
    szCSDVersionW: [0; 20],
};

// #define WIN32_NO_STATUS: preprocessor flag, no Rust output.

#[repr(C)]
pub struct VersionLieInfo {
    pub FullVersion: u32,
    pub dwMajorVersion: u16,
    pub dwMinorVersion: u16,
    pub dwBuildNumber: u32,
    pub dwPlatformId: u16,
    pub wServicePackMajor: u16,
    pub wServicePackMinor: u16,
    pub szCSDVersionA: [i8; 20],
    pub szCSDVersionW: [u16; 20],
}

pub type GETVERSIONEXAPROC = unsafe extern "system" fn(LPOSVERSIONINFOEXA) -> BOOL;

// SHIM_NS / VERSION_INFO / SHIM_SETUP_HOOKS: framework macros, see the per-namespace modules below.

#[no_mangle]
pub unsafe extern "C" fn FakeVersion(mut pResult: LPOSVERSIONINFOEXA, mut pFake: *mut VersionLieInfo) -> BOOL {
    if (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOEXA>() as usize
        || (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOA>() as usize
        || (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOEXW>() as usize
        || (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOW>() as usize
    {
        (*pResult).dwMajorVersion = (*pFake).dwMajorVersion as DWORD;
        (*pResult).dwMinorVersion = (*pFake).dwMinorVersion as DWORD;
        (*pResult).dwBuildNumber = (*pFake).dwBuildNumber;
        (*pResult).dwPlatformId = (*pFake).dwPlatformId as DWORD;
        if (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOEXA>() as usize
            || (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOA>() as usize
        {
            if StringCbCopyA(
                &raw mut (*pResult).szCSDVersion as *mut core::ffi::c_char,
                core::mem::size_of::<[CHAR; 128]>() as size_t,
                &raw mut (*pFake).szCSDVersionA as *mut core::ffi::c_char as *const core::ffi::c_char,
            ) < 0 as HRESULT
            {
                return FALSE as BOOL;
            }
            if (*pResult).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOEXA>() as usize
                && (*pFake).dwPlatformId as DWORD != VER_PLATFORM_WIN32_WINDOWS
            {
                (*pResult).wServicePackMajor = (*pFake).wServicePackMajor;
                (*pResult).wServicePackMinor = (*pFake).wServicePackMinor;
            }
        } else {
            let mut pResultW: LPOSVERSIONINFOEXW = pResult as LPOSVERSIONINFOEXW;
            if StringCbCopyW(
                &raw mut (*pResultW).szCSDVersion as STRSAFE_LPWSTR,
                core::mem::size_of::<[WCHAR; 128]>() as size_t,
                &raw mut (*pFake).szCSDVersionW as *mut WCHAR as STRSAFE_LPCWSTR,
            ) < 0 as HRESULT
            {
                return FALSE as BOOL;
            }
            if (*pResultW).dwOSVersionInfoSize as usize == core::mem::size_of::<OSVERSIONINFOEXW>() as usize
                && (*pFake).dwPlatformId as DWORD != VER_PLATFORM_WIN32_WINDOWS
            {
                (*pResultW).wServicePackMajor = (*pFake).wServicePackMajor;
                (*pResultW).wServicePackMinor = (*pFake).wServicePackMinor;
            }
        }
        return TRUE as BOOL;
    }
    return FALSE as BOOL;
}

// ======== SHIM_NS Win95VersionLie ============================================================
pub mod Win95VersionLie {
    use super::*;
    setup_shim!(Win95VersionLie);

    pub unsafe extern "system" fn APIHook_GetVersion() -> DWORD {
        return g_Win95.FullVersion;
    }

    pub unsafe extern "system" fn APIHook_GetVersionExA(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
        if CALL_SHIM!(1, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
            return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!(g_Win95));
        }
        return FALSE as BOOL;
    }

    /* We do not call about the actual type, FakeVersion will correctly handle it either way */
    pub unsafe extern "system" fn APIHook_GetVersionExW(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
        if CALL_SHIM!(2, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
            return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!(g_Win95));
        }
        return FALSE as BOOL;
    }

    implement_shim!(Win95VersionLie, 3usize, None, [
        (0, "KERNEL32.DLL", "GetVersion", APIHook_GetVersion),
        (1, "KERNEL32.DLL", "GetVersionExA", APIHook_GetVersionExA),
        (2, "KERNEL32.DLL", "GetVersionExW", APIHook_GetVersionExW),
    ]);
}

// ======== SHIM_NS Win98VersionLie ============================================================
pub mod Win98VersionLie {
    use super::*;
    setup_shim!(Win98VersionLie);

    pub unsafe extern "system" fn APIHook_GetVersion() -> DWORD {
        return g_Win98.FullVersion;
    }

    pub unsafe extern "system" fn APIHook_GetVersionExA(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
        if CALL_SHIM!(0, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
            return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!(g_Win98));
        }
        return FALSE as BOOL;
    }

    /* We do not call about the actual type, FakeVersion will correctly handle it either way */
    pub unsafe extern "system" fn APIHook_GetVersionExW(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
        if CALL_SHIM!(1, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
            return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!(g_Win98));
        }
        return FALSE as BOOL;
    }

    // Hook slots are deliberately 2,0,1 here (the C lists GetVersion at index 2).
    implement_shim!(Win98VersionLie, 3usize, None, [
        (2, "KERNEL32.DLL", "GetVersion", APIHook_GetVersion),
        (0, "KERNEL32.DLL", "GetVersionExA", APIHook_GetVersionExA),
        (1, "KERNEL32.DLL", "GetVersionExW", APIHook_GetVersionExW),
    ]);
}

// ======== versionlie.inl, instantiated once per SHIM_NS / VERSION_INFO pair ====================
macro_rules! versionlie_inl {
    ($ns:ident, $info:ident) => {
        pub mod $ns {
            use super::*;
            setup_shim!($ns);

            // versionlie.inl: `#ifndef STATUS_INVALID_PARAMETER` fallbacks (WIN32_NO_STATUS tree).
            const STATUS_INVALID_PARAMETER: DWORD = 0xC000000D;
            const STATUS_SUCCESS: DWORD = 0;

            pub unsafe extern "system" fn APIHook_GetVersion() -> DWORD {
                return $info.FullVersion;
            }

            pub unsafe extern "system" fn APIHook_GetVersionExA(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
                if CALL_SHIM!(0, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
                    return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!($info));
                }
                return FALSE as BOOL;
            }

            /* We do not care about the actual type, FakeVersion will correctly handle it either way */
            pub unsafe extern "system" fn APIHook_GetVersionExW(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> BOOL {
                if CALL_SHIM!(1, GETVERSIONEXAPROC)(lpOsVersionInfo) != 0 {
                    return FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!($info));
                }
                return FALSE as BOOL;
            }

            /* We do not care about the actual type, FakeVersion will correctly handle it either way */
            pub unsafe extern "system" fn APIHook_RtlGetVersion(lpOsVersionInfo: LPOSVERSIONINFOEXA) -> DWORD {
                // Slot 3 is ntdll's RtlGetVersion (NTSTATUS), called through the BOOL-returning
                // proc type exactly as the C does; both are a 32-bit integer return.
                if CALL_SHIM!(3, GETVERSIONEXAPROC)(lpOsVersionInfo) == STATUS_SUCCESS as BOOL {
                    return if FakeVersion(lpOsVersionInfo, core::ptr::addr_of_mut!($info)) != 0 {
                        STATUS_SUCCESS
                    } else {
                        STATUS_INVALID_PARAMETER
                    };
                }
                return STATUS_INVALID_PARAMETER;
            }

            pub unsafe extern "system" fn Notify(fdwReason: DWORD, ptr: PVOID) -> BOOL {
                if fdwReason == SHIM_NOTIFY_ATTACH && $info.wServicePackMajor != 0 {
                    static szServicePack: [WCHAR; 16] = [
                        'S' as WCHAR, 'e' as WCHAR, 'r' as WCHAR, 'v' as WCHAR, 'i' as WCHAR, 'c' as WCHAR,
                        'e' as WCHAR, ' ' as WCHAR, 'P' as WCHAR, 'a' as WCHAR, 'c' as WCHAR, 'k' as WCHAR,
                        ' ' as WCHAR, '%' as WCHAR, 'u' as WCHAR, 0,
                    ];
                    // `Service Pack %u` with a WORD promoted to int, as the C varargs do.
                    let mut hr: HRESULT = StringCbPrintfA!(
                        $info.szCSDVersionA.as_mut_ptr(),
                        core::mem::size_of::<[core::ffi::c_char; 20]>(),
                        "Service Pack %u\0".as_ptr(),
                        $info.wServicePackMajor as core::ffi::c_int
                    );
                    if hr < 0 {
                        return FALSE as BOOL;
                    }
                    hr = StringCbPrintfW!(
                        $info.szCSDVersionW.as_mut_ptr(),
                        core::mem::size_of::<[WCHAR; 20]>(),
                        szServicePack.as_ptr(),
                        $info.wServicePackMajor as core::ffi::c_int
                    );
                    if hr < 0 {
                        return FALSE as BOOL;
                    }
                }
                return TRUE as BOOL;
            }

            implement_shim!($ns, 4usize, Some(Notify), [
                (0, "KERNEL32.DLL", "GetVersionExA", APIHook_GetVersionExA),
                (1, "KERNEL32.DLL", "GetVersionExW", APIHook_GetVersionExW),
                (2, "KERNEL32.DLL", "GetVersion", APIHook_GetVersion),
                (3, "NTDLL.DLL", "RtlGetVersion", APIHook_RtlGetVersion),
            ]);
        }
    };
}

versionlie_inl!(WinNT4SP5VersionLie, g_WinNT4SP5);
versionlie_inl!(Win2000VersionLie, g_Win2000);
versionlie_inl!(Win2000SP1VersionLie, g_Win2000SP1);
versionlie_inl!(Win2000SP2VersionLie, g_Win2000SP2);
versionlie_inl!(Win2000SP3VersionLie, g_Win2000SP3);
versionlie_inl!(WinXPVersionLie, g_WinXP);
versionlie_inl!(WinXPSP1VersionLie, g_WinXPSP1);
versionlie_inl!(WinXPSP2VersionLie, g_WinXPSP2);
versionlie_inl!(WinXPSP3VersionLie, g_WinXPSP3);
versionlie_inl!(Win2k3RTMVersionLie, g_Win2k3RTM);
versionlie_inl!(Win2k3SP1VersionLie, g_Win2k3SP1);
versionlie_inl!(VistaRTMVersionLie, g_WinVistaRTM);
versionlie_inl!(VistaSP1VersionLie, g_WinVistaSP1);
versionlie_inl!(VistaSP2VersionLie, g_WinVistaSP2);
versionlie_inl!(Win7RTMVersionLie, g_Win7RTM);
versionlie_inl!(Win7SP1VersionLie, g_Win7SP1);
versionlie_inl!(Win8RTMVersionLie, g_Win8RTM);
versionlie_inl!(Win81RTMVersionLie, g_Win81RTM);
versionlie_inl!(Win10RTMVersionLie, g_Win10RTM);
versionlie_inl!(Win2k16RTMVersionLie, g_Win2k16RTM);
versionlie_inl!(Win2k19RTMVersionLie, g_Win2k19RTM);
