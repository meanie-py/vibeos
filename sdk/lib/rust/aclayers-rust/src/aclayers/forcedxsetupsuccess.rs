// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_119A63D9A682: [u16; 13] = [
    0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_5A1C5569C3F1: [u16; 9] = [0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_9A6EFE4D86A5: [u16; 3] = [0x005Cu16, 0x002F, 0x0000];
pub static RUSTCONV_WSTR_C38D1FD1F897: [u16; 7] = [0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000];
pub static RUSTCONV_WSTR_C56F0B57C11C: [u16; 11] = [
    0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_119A63D9A682: [u16; 13] = [
    0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_C2R_WSTR_5A1C5569C3F1: [u16; 9] = [0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_C2R_WSTR_9A6EFE4D86A5: [u16; 3] = [0x005Cu16, 0x002F, 0x0000];
pub static RUSTCONV_C2R_WSTR_C38D1FD1F897: [u16; 7] = [0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x0000];
pub static RUSTCONV_C2R_WSTR_C56F0B57C11C: [u16; 11] = [
    0x0064u16, 0x0073, 0x0065, 0x0074, 0x0075, 0x0070, 0x002E, 0x0064, 0x006C, 0x006C,
    0x0000,
];

static mut DxSetupDlls: [_UNICODE_STRING; 4] = [
        _UNICODE_STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 11]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 11]>() as USHORT,
            Buffer: RUSTCONV_C2R_WSTR_C56F0B57C11C.as_ptr() as PWSTR,
        },
        _UNICODE_STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 13]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 13]>() as USHORT,
            Buffer: RUSTCONV_C2R_WSTR_119A63D9A682.as_ptr() as PWSTR,
        },
        _UNICODE_STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 7]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 7]>() as USHORT,
            Buffer: RUSTCONV_C2R_WSTR_C38D1FD1F897.as_ptr() as PWSTR,
        },
        _UNICODE_STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 9]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 9]>() as USHORT,
            Buffer: RUSTCONV_C2R_WSTR_5A1C5569C3F1.as_ptr() as PWSTR,
        },
    ];

static mut PathDividerFind: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 3]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 3]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_9A6EFE4D86A5.as_ptr() as PWSTR,
    };

static mut DxSetupFunctions: [STRING; 4] = [
        _STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_char; 13]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_char>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_char; 13]>() as USHORT,
            Buffer: b"DirectXSetup\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        },
        _STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_char; 14]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_char>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_char; 14]>() as USHORT,
            Buffer: b"DirectXSetupA\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        },
        _STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_char; 14]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_char>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_char; 14]>() as USHORT,
            Buffer: b"DirectXSetupW\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        },
        _STRING {
            Length: (::core::mem::size_of::<[::core::ffi::c_char; 23]>() as usize)
                .wrapping_sub(::core::mem::size_of::<::core::ffi::c_char>() as usize)
                as USHORT,
            MaximumLength: ::core::mem::size_of::<[::core::ffi::c_char; 23]>() as USHORT,
            Buffer: b"DirectXSetupGetVersion\0" as *const u8 as *const ::core::ffi::c_char as PCHAR,
        },
    ];

pub type LOADLIBRARYAPROC = unsafe extern "system" fn(lpLibFileName: LPCSTR) -> HMODULE;
pub type LOADLIBRARYWPROC = unsafe extern "system" fn(lpLibFileName: LPCWSTR) -> HMODULE;
pub type LOADLIBRARYEXAPROC = unsafe extern "system" fn(lpLibFileName: LPCSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
pub type LOADLIBRARYEXWPROC = unsafe extern "system" fn(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
pub type GETPROCADDRESSPROC = unsafe extern "system" fn(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
pub type FREELIBRARYPROC = unsafe extern "system" fn(hLibModule: HINSTANCE) -> BOOL;

pub const DSETUPERR_SUCCESS: i32 = 0;

// SHIM_NS ForceDXSetupSuccess: the per-namespace statics (setup_shim.inl) live at file scope.
setup_shim!(ForceDXSetupSuccess);

#[no_mangle]
pub unsafe extern "system" fn DirectXSetup(hWnd: HWND, lpszRootPath: LPSTR, dwFlags: DWORD) -> INT_PTR {
    SHIM_MSG!("Returning DSETUPERR_SUCCESS\n");
    return DSETUPERR_SUCCESS as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn DirectXSetupA(hWnd: HWND, lpszRootPath: LPSTR, dwFlags: DWORD) -> INT_PTR {
    SHIM_MSG!("Returning DSETUPERR_SUCCESS\n");
    return DSETUPERR_SUCCESS as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn DirectXSetupW(hWnd: HWND, lpszRootPath: LPWSTR, dwFlags: DWORD) -> INT_PTR {
    SHIM_MSG!("Returning DSETUPERR_SUCCESS\n");
    return DSETUPERR_SUCCESS as INT_PTR;
}

#[no_mangle]
pub unsafe extern "system" fn DirectXSetupGetVersion(lpdwVersion: *mut DWORD, lpdwMinorVersion: *mut DWORD) -> INT_PTR {
    if !lpdwVersion.is_null() {
        *lpdwVersion = MAKELONG!(7, 4) as DWORD; // DirectX 7.0
    }
    if !lpdwMinorVersion.is_null() {
        *lpdwMinorVersion = MAKELONG!(1792, 0) as DWORD;
    }

    return TRUE as INT_PTR;
}

unsafe extern "C" fn IsDxSetupW(LibraryPath: PCUNICODE_STRING) -> BOOL {
    let mut LibraryName: _UNICODE_STRING = core::mem::zeroed();
    let mut PathDivider: USHORT = 0;
    let mut n: DWORD = 0;

    if !NT_SUCCESS!(RtlFindCharInUnicodeString(
        RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END,
        LibraryPath,
        core::ptr::addr_of!(PathDividerFind) as PCUNICODE_STRING,
        core::ptr::addr_of_mut!(PathDivider),
    )) {
        PathDivider = 0;
    }

    if PathDivider != 0 {
        PathDivider = PathDivider.wrapping_add(core::mem::size_of::<WCHAR>() as USHORT);
    }

    LibraryName.Buffer = (*LibraryPath)
        .Buffer
        .offset((PathDivider as usize / core::mem::size_of::<WCHAR>()) as isize);
    LibraryName.Length =
        ((*LibraryPath).Length as core::ffi::c_int - PathDivider as core::ffi::c_int) as USHORT;
    LibraryName.MaximumLength =
        ((*LibraryPath).MaximumLength as core::ffi::c_int - PathDivider as core::ffi::c_int)
            as USHORT;

    n = 0;
    while (n as usize) < (ARRAYSIZE!(DxSetupDlls) as usize) {
        if RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(LibraryName) as PCUNICODE_STRING,
            (core::ptr::addr_of!(DxSetupDlls) as *const _UNICODE_STRING).offset(n as isize),
            TRUE as BOOLEAN,
        ) != 0
        {
            SHIM_MSG!(
                "Found %wZ\n",
                (core::ptr::addr_of!(DxSetupDlls) as *const _UNICODE_STRING).offset(n as isize)
            );
            return TRUE as BOOL;
        }
        n = n.wrapping_add(1);
    }
    return FALSE as BOOL;
}

unsafe extern "C" fn IsDxSetupA(LibraryPath: PCANSI_STRING) -> BOOL {
    let mut bIsDxSetup: BOOL = 0;
    let mut LibraryPathW: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    if NT_SUCCESS!(RtlAnsiStringToUnicodeString(
        core::ptr::addr_of_mut!(LibraryPathW),
        LibraryPath,
        TRUE as BOOLEAN,
    )) {
        bIsDxSetup = IsDxSetupW(core::ptr::addr_of_mut!(LibraryPathW) as PCUNICODE_STRING);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(LibraryPathW));
        return bIsDxSetup;
    }
    return FALSE as BOOL;
}

// ---- hooks (SHIM_OBJ_NAME(APIHook_*)) ----
pub unsafe extern "system" fn APIHook_LoadLibraryA(lpLibFileName: LPCSTR) -> HMODULE {
    let mut Library: ANSI_STRING = core::mem::zeroed();

    RtlInitAnsiString(&mut Library, lpLibFileName);
    if IsDxSetupA(&mut Library) != 0 {
        return ShimLib_Instance();
    }

    return CALL_SHIM!(0, LOADLIBRARYAPROC)(lpLibFileName);
}

pub unsafe extern "system" fn APIHook_LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE {
    let mut Library: UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Library, lpLibFileName);
    if IsDxSetupW(&mut Library) != 0 {
        return ShimLib_Instance();
    }

    return CALL_SHIM!(1, LOADLIBRARYWPROC)(lpLibFileName);
}

pub unsafe extern "system" fn APIHook_LoadLibraryExA(lpLibFileName: LPCSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE {
    let mut Library: ANSI_STRING = core::mem::zeroed();

    RtlInitAnsiString(&mut Library, lpLibFileName);
    if IsDxSetupA(&mut Library) != 0 {
        return ShimLib_Instance();
    }

    return CALL_SHIM!(2, LOADLIBRARYEXAPROC)(lpLibFileName, hFile, dwFlags);
}

pub unsafe extern "system" fn APIHook_LoadLibraryExW(lpLibFileName: LPCWSTR, hFile: HANDLE, dwFlags: DWORD) -> HMODULE {
    let mut Library: UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(&mut Library, lpLibFileName);
    if IsDxSetupW(&mut Library) != 0 {
        return ShimLib_Instance();
    }

    return CALL_SHIM!(3, LOADLIBRARYEXWPROC)(lpLibFileName, hFile, dwFlags);
}

pub unsafe extern "system" fn APIHook_GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC {
    // static const FARPROC DxSetupRedirections[]; DxSetupFunctions is the file-scope static above.
    let DxSetupRedirections: [FARPROC; 4] = [
        core::mem::transmute::<usize, FARPROC>(DirectXSetup as usize),
        core::mem::transmute::<usize, FARPROC>(DirectXSetupA as usize),
        core::mem::transmute::<usize, FARPROC>(DirectXSetupW as usize),
        core::mem::transmute::<usize, FARPROC>(DirectXSetupGetVersion as usize),
    ];
    let mut n: DWORD;

    if hModule == ShimLib_Instance() && (lpProcName as usize) > 0xFFFF {
        let mut ProcName: STRING = core::mem::zeroed();
        RtlInitAnsiString(&mut ProcName, lpProcName);
        n = 0;
        while (n as usize) < ARRAYSIZE!(DxSetupFunctions) as usize {
            if RtlEqualString(
                &mut ProcName,
                (core::ptr::addr_of!(DxSetupFunctions) as *const STRING).offset(n as isize),
                TRUE as BOOLEAN,
            ) != 0
            {
                SHIM_MSG!(
                    "Intercepted %Z\n",
                    (core::ptr::addr_of!(DxSetupFunctions) as *const STRING).offset(n as isize)
                );
                return DxSetupRedirections[n as usize];
            }
            n = n.wrapping_add(1);
        }
    }
    return CALL_SHIM!(4, GETPROCADDRESSPROC)(hModule, lpProcName);
}

pub unsafe extern "system" fn APIHook_FreeLibrary(hLibModule: HINSTANCE) -> BOOL {
    if hLibModule == ShimLib_Instance() {
        SHIM_MSG!("Intercepted\n");
        return TRUE as BOOL;
    }

    return CALL_SHIM!(5, FREELIBRARYPROC)(hLibModule);
}

// SHIM_NUM_HOOKS 6, no SHIM_NOTIFY_FN
implement_shim!(ForceDXSetupSuccess, 6usize, None, [
    (0, "KERNEL32.DLL", "LoadLibraryA", APIHook_LoadLibraryA),
    (1, "KERNEL32.DLL", "LoadLibraryW", APIHook_LoadLibraryW),
    (2, "KERNEL32.DLL", "LoadLibraryExA", APIHook_LoadLibraryExA),
    (3, "KERNEL32.DLL", "LoadLibraryExW", APIHook_LoadLibraryExW),
    (4, "KERNEL32.DLL", "GetProcAddress", APIHook_GetProcAddress),
    (5, "KERNEL32.DLL", "FreeLibrary", APIHook_FreeLibrary),
]);
