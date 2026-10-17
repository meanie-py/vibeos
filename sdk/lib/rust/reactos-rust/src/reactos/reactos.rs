use crate::reactos::drivepage::DriveDlgProc;
use crate::reactos::drivepage::FindVolCreateInTreeByVolume;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_0A4E772A5FB0: [u16; 34] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0070, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0065, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F,
    0x006E, 0x0073, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_123946225403: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0030, 0x005C, 0x0000,
];
pub static RUSTCONV_WSTR_14D3FA313934: [u16; 25] = [
    0x0047u16, 0x006C, 0x006F, 0x0062, 0x0061, 0x006C, 0x005C, 0x005F, 0x005F, 0x0052,
    0x0065, 0x0061, 0x0063, 0x0074, 0x004F, 0x0053, 0x005F, 0x0053, 0x0065, 0x0074,
    0x0075, 0x0070, 0x005F, 0x005F, 0x0000,
];
pub static RUSTCONV_WSTR_151330312B2E: [u16; 23] = [
    0x0047u16, 0x0065, 0x0074, 0x0053, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065, 0x0050,
    0x0061, 0x0074, 0x0068, 0x0073, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065,
    0x0064, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_181895D51D74: [u16; 6] = [0x0025u16, 0x0030, 0x0038, 0x006C, 0x0078, 0x0000];
pub static RUSTCONV_WSTR_18C3046DA6B1: [u16; 32] = [
    0x004Eu16, 0x006F, 0x0020, 0x0064, 0x0069, 0x0073, 0x006B, 0x0020, 0x0064, 0x0065,
    0x0074, 0x0065, 0x0063, 0x0074, 0x0065, 0x0064, 0x0020, 0x0069, 0x006E, 0x0020,
    0x0064, 0x0072, 0x0069, 0x0076, 0x0065, 0x0020, 0x0025, 0x0063, 0x003A, 0x0020,
    0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_18F3B0B9E62F: [u16; 34] = [
    0x0025u16, 0x0063, 0x003A, 0x0020, 0x2014, 0x0020, 0x0048, 0x0061, 0x0072, 0x0064,
    0x0064, 0x0069, 0x0073, 0x006B, 0x0020, 0x0025, 0x006C, 0x0075, 0x002C, 0x0020,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0020,
    0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_2457C191431A: [u16; 28] = [
    0x0048u16, 0x0061, 0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0020, 0x0025,
    0x006C, 0x0075, 0x002C, 0x0020, 0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0020, 0x0025, 0x006C, 0x0075, 0x0000,
];
pub static RUSTCONV_WSTR_4BBE7B81F51A: [u16; 168] = [
    0x0050u16, 0x006C, 0x0065, 0x0061, 0x0073, 0x0065, 0x0020, 0x0069, 0x006E, 0x0073,
    0x0065, 0x0072, 0x0074, 0x0020, 0x0061, 0x0020, 0x0062, 0x006C, 0x0061, 0x006E,
    0x006B, 0x0020, 0x0066, 0x006C, 0x006F, 0x0070, 0x0070, 0x0079, 0x0020, 0x0064,
    0x0069, 0x0073, 0x006B, 0x0020, 0x0069, 0x006E, 0x0020, 0x0064, 0x0072, 0x0069,
    0x0076, 0x0065, 0x0020, 0x0025, 0x0063, 0x003A, 0x0020, 0x002E, 0x000A, 0x0041,
    0x006C, 0x006C, 0x0020, 0x0064, 0x0061, 0x0074, 0x0061, 0x0020, 0x0069, 0x006E,
    0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0066, 0x006C, 0x006F, 0x0070, 0x0070,
    0x0079, 0x0020, 0x0064, 0x0069, 0x0073, 0x006B, 0x0020, 0x0077, 0x0069, 0x006C,
    0x006C, 0x0020, 0x0062, 0x0065, 0x0020, 0x0065, 0x0072, 0x0061, 0x0073, 0x0065,
    0x0064, 0x0021, 0x000A, 0x000A, 0x0043, 0x006C, 0x0069, 0x0063, 0x006B, 0x0020,
    0x006F, 0x006E, 0x0020, 0x004F, 0x004B, 0x0020, 0x0074, 0x006F, 0x0020, 0x0063,
    0x006F, 0x006E, 0x0074, 0x0069, 0x006E, 0x0075, 0x0065, 0x002E, 0x000A, 0x0043,
    0x006C, 0x0069, 0x0063, 0x006B, 0x0020, 0x006F, 0x006E, 0x0020, 0x0043, 0x0041,
    0x004E, 0x0043, 0x0045, 0x004C, 0x0020, 0x0074, 0x006F, 0x0020, 0x0073, 0x006B,
    0x0069, 0x0070, 0x0020, 0x0062, 0x006F, 0x006F, 0x0074, 0x006C, 0x006F, 0x0061,
    0x0064, 0x0065, 0x0072, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C,
    0x006C, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x002E, 0x0000,
];
pub static RUSTCONV_WSTR_546CAB492E94: [u16; 15] = [
    0x0055u16, 0x006E, 0x006B, 0x006E, 0x006F, 0x0077, 0x006E, 0x0020, 0x0065, 0x0072,
    0x0072, 0x006F, 0x0072, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_554CA2A34148: [u16; 35] = [
    0x0055u16, 0x0070, 0x0067, 0x0072, 0x0061, 0x0064, 0x0069, 0x006E, 0x0067, 0x002F,
    0x0052, 0x0065, 0x0070, 0x0061, 0x0069, 0x0072, 0x0069, 0x006E, 0x0067, 0x0020,
    0x0022, 0x0025, 0x0073, 0x0022, 0x0020, 0x0066, 0x0072, 0x006F, 0x006D, 0x0020,
    0x0022, 0x0025, 0x0073, 0x0022, 0x0000,
];
pub static RUSTCONV_WSTR_65DA6EB88F5F: [u16; 37] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0070, 0x0072, 0x0065, 0x0070, 0x0061, 0x0072, 0x0065, 0x0020, 0x0074, 0x0068,
    0x0065, 0x0020, 0x006C, 0x0069, 0x0073, 0x0074, 0x0020, 0x006F, 0x0066, 0x0020,
    0x0066, 0x0069, 0x006C, 0x0065, 0x0073, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_68288B5D5761: [u16; 26] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0063, 0x006F, 0x0070, 0x0079, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0066,
    0x0069, 0x006C, 0x0065, 0x0073, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_6AF5D4B374C5: [u16; 4] = [0x006Eu16, 0x002F, 0x0061, 0x0000];
pub static RUSTCONV_WSTR_75720173C2CE: [u16; 44] = [
    0x0046u16, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020, 0x0074, 0x006F, 0x0020,
    0x0066, 0x0069, 0x006E, 0x0064, 0x0020, 0x006F, 0x0072, 0x0020, 0x0073, 0x0065,
    0x0074, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0073, 0x0079, 0x0073, 0x0074,
    0x0065, 0x006D, 0x0020, 0x0070, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069,
    0x006F, 0x006E, 0x0021, 0x0000,
];
pub static RUSTCONV_WSTR_838DD1C33324: [u16; 23] = [
    0x0053u16, 0x0065, 0x0074, 0x0075, 0x0070, 0x0020, 0x0053, 0x0068, 0x0069, 0x0066,
    0x0074, 0x002B, 0x0046, 0x0031, 0x0030, 0x0020, 0x0048, 0x006F, 0x0074, 0x006B,
    0x0065, 0x0079, 0x0000,
];
pub static RUSTCONV_WSTR_83AE1AA69859: [u16; 35] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0048, 0x0061,
    0x0072, 0x0064, 0x0064, 0x0069, 0x0073, 0x006B, 0x0025, 0x006C, 0x0075, 0x005C,
    0x0050, 0x0061, 0x0072, 0x0074, 0x0069, 0x0074, 0x0069, 0x006F, 0x006E, 0x0025,
    0x006C, 0x0075, 0x0025, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_88A26420BF90: [u16; 13] = [
    0x0063u16, 0x006F, 0x006D, 0x0063, 0x0074, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_934172CB04C3: [u16; 7] = [0x0023u16, 0x0033, 0x0032, 0x0037, 0x0037, 0x0030, 0x0000];
pub static RUSTCONV_WSTR_949D726850F4: [u16; 25] = [
    0x004Eu16, 0x0065, 0x0077, 0x0020, 0x0052, 0x0065, 0x0061, 0x0063, 0x0074, 0x004F,
    0x0053, 0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0061,
    0x0074, 0x0069, 0x006F, 0x006E, 0x0000,
];
pub static RUSTCONV_WSTR_958F461AF768: [u16; 9] = [0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_992FD7539CF7: [u16; 51] = [
    0x0049u16, 0x006E, 0x0069, 0x0074, 0x0044, 0x0065, 0x0073, 0x0074, 0x0069, 0x006E,
    0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0050, 0x0061, 0x0074, 0x0068, 0x0073,
    0x0028, 0x0029, 0x0020, 0x0066, 0x0061, 0x0069, 0x006C, 0x0065, 0x0064, 0x0020,
    0x0077, 0x0069, 0x0074, 0x0068, 0x0020, 0x0073, 0x0074, 0x0061, 0x0074, 0x0075,
    0x0073, 0x0020, 0x0030, 0x0078, 0x0025, 0x0030, 0x0038, 0x006C, 0x0078, 0x000A,
    0x0000,
];
pub static RUSTCONV_WSTR_BCF3913447D5: [u16; 4] = [0x0025u16, 0x0073, 0x005C, 0x0000];
pub static RUSTCONV_WSTR_BF74AC1CFE02: [u16; 6] = [0x0025u16, 0x0063, 0x003A, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_D52B95BF9F39: [u16; 50] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0063, 0x0072, 0x0065,
    0x0061, 0x0074, 0x0065, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0069, 0x006E,
    0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E,
    0x0020, 0x0074, 0x0068, 0x0072, 0x0065, 0x0061, 0x0064, 0x002C, 0x0020, 0x0065,
    0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0025, 0x006C, 0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_D62BC3623F3E: [u16; 10] = [0x005Cu16, 0x0073, 0x0079, 0x0073, 0x0074, 0x0065, 0x006D, 0x0033, 0x0032, 0x0000];
pub static RUSTCONV_WSTR_DB7B28E33CE1: [u16; 44] = [
    0x0043u16, 0x0061, 0x006E, 0x006E, 0x006F, 0x0074, 0x0020, 0x0063, 0x0072, 0x0065,
    0x0061, 0x0074, 0x0065, 0x0020, 0x0074, 0x0068, 0x0065, 0x0020, 0x0069, 0x006E,
    0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0020, 0x0065, 0x0076, 0x0065, 0x006E,
    0x0074, 0x002C, 0x0020, 0x0065, 0x0072, 0x0072, 0x006F, 0x0072, 0x0020, 0x0025,
    0x006C, 0x0075, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_DD8FED8124A3: [u16; 4] = [0x004Du16, 0x0042, 0x0052, 0x0000];
pub static RUSTCONV_WSTR_E2CD87D16D16: [u16; 5] = [0x0025u16, 0x0073, 0x0025, 0x0073, 0x0000];
pub static RUSTCONV_WSTR_E9D409C4F426: [u16; 4] = [0x0025u16, 0x0077, 0x005A, 0x0000];
pub static RUSTCONV_WSTR_F4180D6B4EEA: [u16; 24] = [
    0x0042u16, 0x006F, 0x006F, 0x0074, 0x006C, 0x006F, 0x0061, 0x0064, 0x0065, 0x0072,
    0x0020, 0x0069, 0x006E, 0x0073, 0x0074, 0x0061, 0x006C, 0x006C, 0x0061, 0x0074,
    0x0069, 0x006F, 0x006E, 0x0000,
];

use core::ffi::c_void;

extern "system" {
    fn GetProgmanWindow() -> HWND;
    fn GetShellWindow() -> HWND;
    fn IsWindow(hWnd: HWND) -> BOOL;
    fn IsWindowVisible(hWnd: HWND) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_123946225403: [u16; 17] = [
    0x005Cu16, 0x0044, 0x0065, 0x0076, 0x0069, 0x0063, 0x0065, 0x005C, 0x0046, 0x006C,
    0x006F, 0x0070, 0x0070, 0x0079, 0x0030, 0x005C, 0x0000,
];

#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_21 {
    pub IncludeForUnattended: BOOL,
    pub dwFlags: DWORD,
    pub pszTemplate: PCWSTR,
    pub pszHeaderTitle: PCWSTR,
    pub pszHeaderSubTitle: PCWSTR,
    pub pfnDlgProc: DLGPROC,
}

static mut FloppyDrive: _UNICODE_STRING = _UNICODE_STRING {
        Length: (::core::mem::size_of::<[::core::ffi::c_ushort; 17]>() as usize)
            .wrapping_sub(::core::mem::size_of::<::core::ffi::c_ushort>() as usize)
            as USHORT,
        MaximumLength: ::core::mem::size_of::<[::core::ffi::c_ushort; 17]>() as USHORT,
        Buffer: RUSTCONV_C2R_WSTR_123946225403.as_ptr() as PWSTR,
    };

pub static VENDOR_REACTOS: [::core::ffi::c_ushort; 8] = unsafe {
    ::core::mem::transmute::<[u8; 16], [::core::ffi::c_ushort; 8]>(*b"R\0e\0a\0c\0t\0O\0S\0\0\0")
};

pub static VENDOR_MICROSOFT: [::core::ffi::c_ushort; 10] = unsafe {
    ::core::mem::transmute::<[u8; 20], [::core::ffi::c_ushort; 10]>(
        *b"M\0i\0c\0r\0o\0s\0o\0f\0t\0\0\0",
    )
};

#[no_mangle]
pub static mut ProcessHeap: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

#[no_mangle]
pub static mut SetupData: SETUPDATA = _SETUPDATA {
    hInstance: ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__,
    bUnattend: 0,
    bMustReboot: 0,
    hTitleFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
    hBoldFont: ::core::ptr::null::<HFONT__>() as *mut HFONT__,
    hInstallThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    hHaltInstallEvent: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    __bindgen_anon_1: _SETUPDATA__bindgen_ty_1 { bStopInstall: 0 },
    MappingList: _NT_WIN32_PATH_MAPPING_LIST {
        List: _LIST_ENTRY {
            Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
            Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        },
        MappingsCount: 0,
    },
    USetupData: _USETUP_DATA {
        LastErrorNumber: NOT_AN_ERROR,
        ErrorRoutine: None,
        SetupInf: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SetupFileQueue: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SourceRootPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        SourceRootDir: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        SourcePath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        SystemRootPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        DestinationArcPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        DestinationPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        DestinationRootPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        InstallPath: _UNICODE_STRING {
            Length: 0,
            MaximumLength: 0,
            Buffer: ::core::ptr::null::<WCHAR>() as *mut WCHAR,
        },
        DestinationDiskNumber: 0,
        DestinationPartitionNumber: 0,
        BootLoaderLocation: 0,
        FormatPartition: 0,
        AutoPartition: 0,
        FsType: 0,
        ComputerList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
        DisplayList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
        KeyboardList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
        LayoutList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
        LanguageList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
        ArchType: ARCH_PcAT,
        ComputerType: ::core::ptr::null::<WCHAR>(),
        DisplayType: ::core::ptr::null::<WCHAR>(),
        LayoutId: ::core::ptr::null::<WCHAR>(),
        LocaleID: [0; 9],
        LanguageId: 0,
        RequiredPartitionDiskSpace: 0,
        InstallationDirectory: [0; 260],
    },
    RepairUpdateFlag: 0,
    PartitionList: ::core::ptr::null::<_PARTLIST>() as *mut _PARTLIST,
    CurrentInstallation: ::core::ptr::null::<_NTOS_INSTALLATION>() as *mut _NTOS_INSTALLATION,
    NtOsInstallsList: ::core::ptr::null::<_GENERIC_LIST>() as *mut _GENERIC_LIST,
    DestPartSize: 0,
    SelectedLanguageId: ::core::ptr::null::<WCHAR>(),
    DefaultLanguage: [0; 20],
    DefaultKBLayout: [0; 20],
};

#[no_mangle]
pub static mut InstallPartition: PPARTENTRY = ::core::ptr::null::<_PARTENTRY>() as *mut _PARTENTRY;

#[no_mangle]
pub static mut SystemPartition: PPARTENTRY = ::core::ptr::null::<_PARTENTRY>() as *mut _PARTENTRY;

#[no_mangle]
pub static mut UiContext: UI_CONTEXT = _UI_CONTEXT {
    hPartList: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hwndDlg: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hWndItem: ::core::ptr::null::<HWND__>() as *mut HWND__,
    hWndProgress: ::core::ptr::null::<HWND__>() as *mut HWND__,
    dwPbStyle: 0,
};

#[no_mangle]
pub static mut hWaitCursor: HCURSOR = ::core::ptr::null::<HICON__>() as *mut HICON__;

static mut column_ids: [UINT; 3] = [
    IDS_LIST_COLUMN_FIRST as UINT,
    (IDS_LIST_COLUMN_FIRST + (1 as ::core::ffi::c_int) as u32) as UINT,
    (IDS_LIST_COLUMN_FIRST + (2 as ::core::ffi::c_int) as u32) as UINT,
];

static mut column_widths: [INT; 3] = [
    200 as ::core::ffi::c_int,
    150 as ::core::ffi::c_int,
    100 as ::core::ffi::c_int,
];

static mut column_alignment: [INT; 3] = [(LVCFMT_LEFT as i32), (LVCFMT_LEFT as i32), (LVCFMT_LEFT as i32)];

#[no_mangle]
pub static mut __pfnDliNotifyHook2: PfnDliHook = unsafe {
    Some(setupDelayHook)
};

#[no_mangle]
pub static mut __pfnDliFailureHook2: PfnDliHook = unsafe {
    Some(setupDelayHook)
};

#[no_mangle]
pub static mut wpOrgPrshtProc: WNDPROC = None;

static mut WizardPages: [C2RustUnnamed_21; 9] = unsafe {
    [
        C2RustUnnamed_21 {
            IncludeForUnattended: (FALSE as i32),
            dwFlags: PSP_HIDEHEADER as DWORD,
            pszTemplate: (2000 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: ::core::ptr::null::<WCHAR>(),
            pszHeaderSubTitle: ::core::ptr::null::<WCHAR>(),
            pfnDlgProc: Some(
                StartDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (FALSE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2010 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5001 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5002 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                TypeDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (FALSE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2020 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5003 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5004 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                UpgradeRepairDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2030 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5005 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5006 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                DeviceDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2040 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5007 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5008 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                DriveDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2050 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5009 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5010 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                SummaryDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: (PSP_USEHEADERTITLE | PSP_USEHEADERSUBTITLE) as DWORD,
            pszTemplate: (2060 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: (5011 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderSubTitle: (5012 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR)
                as WORD as ULONG_PTR as LPWSTR as PCWSTR,
            pfnDlgProc: Some(
                ProcessDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: PSP_HIDEHEADER as DWORD,
            pszTemplate: (2070 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: ::core::ptr::null::<WCHAR>(),
            pszHeaderSubTitle: ::core::ptr::null::<WCHAR>(),
            pfnDlgProc: Some(
                FinishDlgProc,
            ),
        },
        C2RustUnnamed_21 {
            IncludeForUnattended: (TRUE as i32),
            dwFlags: PSP_HIDEHEADER as DWORD,
            pszTemplate: (2075 as ::core::ffi::c_int as DWORD_PTR & 0xffff as DWORD_PTR) as WORD
                as ULONG_PTR as LPWSTR as PCWSTR,
            pszHeaderTitle: ::core::ptr::null::<WCHAR>(),
            pszHeaderSubTitle: ::core::ptr::null::<WCHAR>(),
            pfnDlgProc: Some(
                FinishDlgProc,
            ),
        },
    ]
};

pub static REACTOS_SETUP_MUTEX: [::core::ffi::c_ushort; 18] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_ushort; 18]>(
        *b"_\0_\0R\0e\0a\0c\0t\0O\0S\0_\0S\0e\0t\0u\0p\0_\0_\0\0\0",
    )
};

pub type PGET_ENTRY_DESCRIPTION =
    Option<unsafe extern "system" fn(Entry: PGENERIC_LIST_ENTRY, Buffer: PWSTR, cchBufferSize: SIZE_T)>;

pub type PADD_ENTRY_ITEM = Option<unsafe extern "system" fn(
    hWndList: HWND,
    plvItem: *mut LVITEM,
    Entry: PGENERIC_LIST_ENTRY,
    Buffer: PWSTR,
    cchBufferSize: SIZE_T,
)>;

pub const IDS_LIST_COLUMN_FIRST: u32 = IDS_INSTALLATION_NAME;
pub const IDS_LIST_COLUMN_LAST: u32 = IDS_INSTALLATION_VENDOR;
pub const MAX_LIST_COLUMNS: usize = (IDS_LIST_COLUMN_LAST - IDS_LIST_COLUMN_FIRST + 1) as usize;

#[repr(C)]
pub struct _FSVOL_CONTEXT {
    pub pSetupData: PSETUPDATA,
}
pub type FSVOL_CONTEXT = _FSVOL_CONTEXT;
pub type PFSVOL_CONTEXT = *mut FSVOL_CONTEXT;

#[repr(C)]
pub struct _COPYCONTEXT {
    pub pSetupData: PSETUPDATA,
    pub TotalOperations: ULONG,
    pub CompletedOperations: ULONG,
}
pub type COPYCONTEXT = _COPYCONTEXT;
pub type PCOPYCONTEXT = *mut COPYCONTEXT;

pub const PM_INSTALL_START: u32 = WM_APP + 1;
pub const PM_INSTALL_DONE: u32 = WM_APP + 2;

#[repr(C)]
pub struct _CLOSABLE_WND_INFO {
    pub hWndExclude: HWND,
    pub Found: BOOL,
}
pub type CLOSABLE_WND_INFO = _CLOSABLE_WND_INFO;
pub type PCLOSABLE_WND_INFO = *mut CLOSABLE_WND_INFO;

// reactos.c declares it between pshpack1.h and poppack.h: the dialog resource format.
#[repr(C, packed)]
pub struct DLGTEMPLATEEX {
    pub dlgVer: WORD,
    pub signature: WORD,
    pub helpID: DWORD,
    pub exStyle: DWORD,
    pub style: DWORD,
    pub cDlgItems: WORD,
    pub x: i16,
    pub y: i16,
    pub cx: i16,
    pub cy: i16,
}
pub type LPDLGTEMPLATEEX = *mut DLGTEMPLATEEX;

#[repr(C)]
pub struct WizardPagesEntry {
    pub IncludeForUnattended: BOOL,
    pub dwFlags: DWORD,
    pub pszTemplate: PCWSTR,
    pub pszHeaderTitle: PCWSTR,
    pub pszHeaderSubTitle: PCWSTR,
    pub pfnDlgProc: DLGPROC,
}
unsafe extern "C" fn CenterWindow(hWnd: HWND) {
    let mut hWndParent: HWND = core::ptr::null_mut();
    let mut rcParent: RECT = core::mem::zeroed();
    let mut rcWindow: RECT = core::mem::zeroed();

    hWndParent = GetParent(hWnd);
    if hWndParent.is_null() {
        hWndParent = GetDesktopWindow();
    }

    GetWindowRect(hWndParent, core::ptr::addr_of_mut!(rcParent));
    GetWindowRect(hWnd, core::ptr::addr_of_mut!(rcWindow));

    SetWindowPos(
        hWnd,
        HWND_TOP,
        ((rcParent.right - rcParent.left) - (rcWindow.right - rcWindow.left)) / 2,
        ((rcParent.bottom - rcParent.top) - (rcWindow.bottom - rcWindow.top)) / 2,
        0,
        0,
        SWP_NOSIZE,
    );
}

unsafe extern "C" fn CreateBoldFont(hOrigFont: HFONT, PointSize: INT) -> HFONT {
    let mut lf: LOGFONTW = core::mem::zeroed();
    if !hOrigFont.is_null() {
        GetObjectW(
            hOrigFont as HANDLE,
            core::mem::size_of::<LOGFONTW>() as INT,
            &raw mut lf as LPVOID,
        );
    } else {
        let mut ncm: NONCLIENTMETRICSW = core::mem::zeroed();
        ncm.cbSize = core::mem::size_of::<NONCLIENTMETRICSW>() as UINT;
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS as UINT,
            0 as UINT,
            &raw mut ncm as PVOID,
            0 as UINT,
        );
        lf = ncm.lfMessageFont;
    }

    /* Make the font bold, keeping the other attributes */
    lf.lfWeight = FW_BOLD as LONG;

    /* Determine the font height (logical units) if necessary */
    if PointSize != 0 {
        let mut hdc: HDC = GetDC(core::ptr::null_mut::<HWND__>());
        lf.lfHeight = -MulDiv(
            PointSize,
            GetDeviceCaps(hdc, LOGPIXELSY as INT),
            72 as INT,
        ) as LONG;
        // lf.lfWidth = 0;
        ReleaseDC(core::ptr::null_mut::<HWND__>(), hdc);
    }

    return CreateFontIndirect(&raw mut lf);
}

unsafe extern "C" fn CreateTitleFont(hOrigFont: HFONT) -> HFONT {
    return CreateBoldFont(hOrigFont, 12);
}
#[no_mangle]
pub unsafe extern "C" fn LoadAllocStringW(
    hInstance: HINSTANCE,
    uID: UINT,
    pString: *mut PWSTR,
    cchBufferLen: size_t,
) -> size_t {
    let mut pStr: PCWSTR = core::ptr::null::<WCHAR>();
    let mut Length: size_t;

    /* Try to load the string from the resource */
    Length = LoadStringW(hInstance, uID, core::ptr::addr_of_mut!(pStr) as PWSTR, 0) as size_t;
    if Length == 0 {
        /* No resource string was found, return NULL */
        *pString = core::ptr::null_mut();
        return 0;
    }

    /* If the caller gave a pointer to a buffer on input, verify whether it
     * is large enough to contain the string. If not, allocate a new buffer. */
    if (*pString).is_null() || (cchBufferLen < Length + 1) {
        /* Allocate a new buffer, adding a NUL-terminator */
        *pString = HeapAlloc(
            GetProcessHeap(),
            0,
            ((Length + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
        ) as PWSTR;
        if (*pString).is_null() {
            return 0;
        }
    }

    /* Copy the string, NUL-terminated */
    StringCchCopyNW(*pString, Length + 1, pStr, Length);
    return Length;
}
#[no_mangle]
pub unsafe extern "C" fn FormatAllocStringWV(
    pString: *mut PWSTR,
    cchBufferLen: size_t,
    pszFormat: PCWSTR,
    mut args: *mut ::core::ffi::c_void,
) -> size_t {
    let mut Length: size_t = 0;
    Length = _vscwprintf(pszFormat as *const wchar_t, args as *mut u8) as size_t; // Doesn't count the NUL-terminator.
    if (*pString).is_null() || Length >= cchBufferLen {
        *pString = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY as DWORD,
            (Length as SIZE_T)
                .wrapping_add(1 as SIZE_T)
                .wrapping_mul(::core::mem::size_of::<WCHAR>() as SIZE_T),
        ) as PWSTR;
        if (*pString).is_null() {
            return 0 as size_t;
        }
    }
    StringCchVPrintfW(
        *pString,
        Length.wrapping_add(1 as size_t),
        pszFormat as STRSAFE_LPCWSTR,
        args,
    );
    return Length;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayMessageV(
    mut hWnd: HWND,
    mut uType: UINT,
    mut pszTitle: PCWSTR,
    mut pszFormatMessage: PCWSTR,
    mut args: *mut ::core::ffi::c_void,
) -> INT {
    let mut iRes: INT = 0;
    let mut hInstance: HINSTANCE = ::core::ptr::null_mut::<HINSTANCE__>();
    let mut mb: MSGBOXPARAMSW = MSGBOXPARAMSW {
        cbSize: 0 as UINT,
        hwndOwner: ::core::ptr::null_mut::<HWND__>(),
        hInstance: ::core::ptr::null_mut::<HINSTANCE__>(),
        lpszText: ::core::ptr::null::<WCHAR>(),
        lpszCaption: ::core::ptr::null::<WCHAR>(),
        dwStyle: 0,
        lpszIcon: ::core::ptr::null::<WCHAR>(),
        dwContextHelpId: 0,
        lpfnMsgBoxCallback: None,
        dwLanguageId: 0,
    };
    let mut Format: PWSTR = ::core::ptr::null_mut::<WCHAR>();
    let mut StaticBuffer: [WCHAR; 256] = [0; 256];
    let mut Buffer: PWSTR = &raw mut StaticBuffer as PWSTR;

    if (!pszTitle.is_null() && IS_INTRESOURCE!(pszTitle) != false)
        || IS_INTRESOURCE!(pszFormatMessage) != false
    {
        hInstance = GetModuleHandleW(::core::ptr::null::<WCHAR>()) as HINSTANCE;
    }

    if !pszFormatMessage.is_null() && IS_INTRESOURCE!(pszFormatMessage) != false {
        Format = ::core::ptr::null_mut::<WCHAR>();
        LoadAllocStringW(
            hInstance,
            PtrToUlong!(pszFormatMessage as *const ::core::ffi::c_void) as UINT,
            &raw mut Format,
            0,
        );
    } else {
        Format = pszFormatMessage as PWSTR;
    }

    if !Format.is_null() {
        FormatAllocStringWV(
            &raw mut Buffer,
            _countof!(StaticBuffer) as size_t,
            Format as PCWSTR,
            args,
        );
        if Buffer.is_null() {
            Buffer = Format;
        }
    } else {
        Format = pszFormatMessage as PWSTR;
        Buffer = Format;
    }

    mb.cbSize = ::core::mem::size_of::<MSGBOXPARAMSW>() as UINT;
    mb.hwndOwner = hWnd;
    mb.hInstance = hInstance;
    mb.lpszText = Buffer as LPCWSTR;
    mb.lpszCaption = pszTitle as LPCWSTR;
    mb.dwStyle = uType as DWORD;
    mb.dwLanguageId = MAKELANGID!(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD;
    iRes = MessageBoxIndirectW(&raw mut mb) as INT;

    if Buffer != (&raw mut StaticBuffer) as PWSTR && Buffer != Format {
        HeapFree(
            GetProcessHeap(),
            0,
            Buffer as PVOID,
        );
    }

    if !Format.is_null() && Format != pszFormatMessage as PWSTR {
        HeapFree(
            GetProcessHeap(),
            0,
            Format as PVOID,
        );
    }

    return iRes;
}
#[no_mangle]
pub unsafe extern "C" fn DisplayMessage_v(
    mut hWnd: HWND,
    mut uType: UINT,
    mut pszTitle: PCWSTR,
    mut pszFormatMessage: PCWSTR,
    ap_list: *mut ::core::ffi::c_void,
) -> INT {
    let mut iRes: INT = 0;
    let mut args_0: va_list = *(ap_list as *mut va_list);

    iRes = DisplayMessageV(hWnd, uType, pszTitle, pszFormatMessage, (args_0).cast());

    return iRes;
}



#[no_mangle]
pub unsafe extern "C" fn DisplayError_v(
    mut hWnd: HWND,
    mut uIDTitle: UINT,
    mut uIDMessage: UINT,
    ap_list: *mut ::core::ffi::c_void,
) -> INT {
    let mut iRes: INT = 0;
    // The shim va_start'ed the list and hands over `&args`; the va_list itself is a pointer.
    let mut args_0: va_list = *(ap_list as *mut va_list);
    iRes = DisplayMessageV(
        hWnd,
        (MB_ICONERROR | MB_OK) as UINT,
        (uIDTitle as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as PCWSTR,
        (uIDMessage as DWORD_PTR & 0xffff as DWORD_PTR) as WORD as ULONG_PTR as LPWSTR as PCWSTR,
        (args_0).cast(),
    );
    return iRes;
}


#[no_mangle]
pub unsafe extern "C" fn SetWindowResTextW(hWnd: HWND, hInstance: HINSTANCE, uID: UINT) {
    let mut szText: [WCHAR; 256] = [0; 256];
    let mut String: PWSTR = core::ptr::addr_of_mut!(szText) as PWSTR; // Use the static buffer by default.

    /* Try to load the string from the resource */
    let _ = LoadAllocStringW(hInstance, uID, core::ptr::addr_of_mut!(String), (_countof!(szText)) as size_t);
    if String.is_null() {
        return;
    }
    SetWindowTextW(hWnd, String as LPCWSTR);
    if String != core::ptr::addr_of_mut!(szText) as *mut WCHAR {
        HeapFree(GetProcessHeap(), 0 as DWORD, String as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetWindowResPrintfVW(
    mut hWnd: HWND,
    mut hInstance: HINSTANCE,
    mut uID: UINT,
    mut args: *mut ::core::ffi::c_void,
) {
    let mut ResBuffer: [WCHAR; 256] = [0; 256];
    let mut szText: [WCHAR; 256] = [0; 256];
    let mut ResFmt: PWSTR = core::ptr::addr_of_mut!(ResBuffer) as PWSTR;
    let mut String: PWSTR = core::ptr::addr_of_mut!(szText) as PWSTR;

    /* Try to load the string from the resource */
    let _ = LoadAllocStringW(
        hInstance,
        uID,
        core::ptr::addr_of_mut!(ResFmt),
        (_countof!(ResBuffer)) as size_t,
    );
    if ResFmt.is_null() {
        return;
    }

    /* Format the string and retrieve its length. If it is too long,
     * an auxiliary buffer is allocated; otherwise the static buffer
     * is used. The string is built to be NUL-terminated. */
    let _ = FormatAllocStringWV(
        core::ptr::addr_of_mut!(String),
        (_countof!(szText)) as size_t,
        ResFmt as PCWSTR,
        args,
    );
    if String.is_null() {
        /* Allocation failed, use the original format string verbatim */
        String = ResFmt;
    }

    SetWindowTextW(hWnd, String as LPCWSTR);

    /* Free the buffers if needed */
    if (String as *mut WCHAR) != (core::ptr::addr_of_mut!(szText) as *mut WCHAR)
        && String != ResFmt
    {
        HeapFree(
            GetProcessHeap(),
            0,
            String as PVOID,
        );
    }

    if !ResFmt.is_null() && (ResFmt as *mut WCHAR) != (core::ptr::addr_of_mut!(ResBuffer) as *mut WCHAR) {
        HeapFree(
            GetProcessHeap(),
            0,
            ResFmt as PVOID,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn SetWindowResPrintfW_v(
    mut hWnd: HWND,
    mut hInstance: HINSTANCE,
    mut uID: UINT,
    ap_list: *mut ::core::ffi::c_void,
) {
    let mut args_0: va_list = *(ap_list as *mut va_list);


    SetWindowResPrintfVW(hWnd, hInstance, uID, (args_0).cast());

}



unsafe extern "system" fn StartDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    if uMsg == WM_INITDIALOG {
        {
            /* Save pointer to the global setup data */
            pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, (pSetupData as DWORD_PTR) as LONG_PTR);

            /* Set title font */
            SetDlgItemFont!(hwndDlg, (IDC_STARTTITLE as i32), (*pSetupData).hTitleFont, TRUE);

            // TEMPTEMP: Set the ReactOS-Alpha information in bold.
            // TODO: Remove once we reach 0.5/Beta :)
            SetDlgItemFont!(hwndDlg, (IDC_WARNTEXT1 as i32), (*pSetupData).hBoldFont, TRUE);
            SetDlgItemFont!(hwndDlg, (IDC_WARNTEXT2 as i32), (*pSetupData).hBoldFont, TRUE);
            SetDlgItemFont!(hwndDlg, (IDC_WARNTEXT3 as i32), (*pSetupData).hBoldFont, TRUE);

            ///* Center the wizard window */
            //CenterWindow(GetParent(hwndDlg));
            return TRUE as INT_PTR;
        }
    } else if uMsg == WM_NOTIFY {
        {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            if (*lpnm).code == (PSN_SETACTIVE) as u32 {
                {
                    /* Only "Next" and "Cancel" for the first page and hide "Back".
                     * Don't use the PropSheet_SetWizButtons() macro, because its
                     * posted message could interfere with the hidden button. */
                    SendMessageW(GetParent(hwndDlg), PSM_SETWIZBUTTONS, 0, (PSWIZB_NEXT) as LPARAM);
                    // PropSheet_ShowWizButtons(GetParent(hwndDlg), 0, PSWIZB_BACK);
                    ShowDlgItem!(GetParent(hwndDlg), (ID_WIZBACK as i32), (SW_HIDE as i32));
                }
            } else if (*lpnm).code == (PSN_KILLACTIVE) as u32 {
                {
                    /* Show "Back" button */
                    // PropSheet_ShowWizButtons(GetParent(hwndDlg), PSWIZB_BACK, PSWIZB_BACK);
                    ShowDlgItem!(GetParent(hwndDlg), (ID_WIZBACK as i32), (SW_SHOW as i32));
                }
            } else {
            }
        }
    } else {
    }

    return FALSE as INT_PTR;
}
unsafe extern "system" fn TypeDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA;

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, pSetupData as DWORD_PTR as LONG_PTR);

            /* Set the options in bold */
            SetDlgItemFont!(hwndDlg, (IDC_INSTALL as i32), (*pSetupData).hBoldFont, TRUE);
            SetDlgItemFont!(hwndDlg, (IDC_UPDATE as i32), (*pSetupData).hBoldFont, TRUE);

            /*
             * Enable double-click handling with the BS_NOTIFY style for both options.
             * Idea adapted from: https://devblogs.microsoft.com/oldnewthing/20050804-10/?p=34713
             */
            {
                let mut hRadio: HWND = GetDlgItem(hwndDlg, (IDC_INSTALL as i32));
                SetWindowLongPtrW(
                    hRadio,
                    GWL_STYLE,
                    GetWindowLongPtrW(hRadio, GWL_STYLE) | BS_NOTIFY as LONG_PTR,
                );
                hRadio = GetDlgItem(hwndDlg, (IDC_UPDATE as i32));
                SetWindowLongPtrW(
                    hRadio,
                    GWL_STYLE,
                    GetWindowLongPtrW(hRadio, GWL_STYLE) | BS_NOTIFY as LONG_PTR,
                );
            }

            /*
             * Enable the "Update" radio button and text only if we have
             * available NT installations, otherwise disable them.
             */
            if !(*pSetupData).NtOsInstallsList.is_null()
                && GetNumberOfListEntries((*pSetupData).NtOsInstallsList) != 0
            {
                EnableDlgItem!(hwndDlg, (IDC_UPDATE as i32), (TRUE as i32));
                EnableDlgItem!(hwndDlg, (IDC_UPDATETEXT as i32), (TRUE as i32));
            } else {
                EnableDlgItem!(hwndDlg, (IDC_UPDATE as i32), (FALSE as i32));
                EnableDlgItem!(hwndDlg, (IDC_UPDATETEXT as i32), (FALSE as i32));
            }

            /* Check the "Install ReactOS" radio button and ensure it is initially focused */
            CheckRadioButton(hwndDlg, (IDC_INSTALL as i32), (IDC_UPDATE as i32), (IDC_INSTALL as i32));
            SetFocus(GetDlgItem(hwndDlg, (IDC_INSTALL as i32)));
            return FALSE as INT_PTR;
        }

        WM_COMMAND => {
            /*
             * Go to the next page if the user double-clicked one of the options.
             * Idea adapted from: https://devblogs.microsoft.com/oldnewthing/20050804-10/?p=34713
             */
            if HIWORD!(wParam) == (BN_DBLCLK as u16) {
                match (LOWORD!(wParam)) as u32 {
                    IDC_INSTALL | IDC_UPDATE => {
                        PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
                        return TRUE as INT_PTR;
                    }
                    _ => {}
                }
            }
        }

        WM_NOTIFY => {
            let lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* Ensure the "Install ReactOS" radio button is checked if we don't have
                     * a selected installation (default case), which can also happen if the
                     * user clicked on the "Do not upgrade" button on the Upgrade/Repair
                     * selection page, then went back here. */
                    if (*pSetupData).CurrentInstallation.is_null() {
                        CheckRadioButton(hwndDlg, (IDC_INSTALL as i32), (IDC_UPDATE as i32), (IDC_INSTALL as i32));
                    } else {
                        CheckRadioButton(hwndDlg, (IDC_INSTALL as i32), (IDC_UPDATE as i32), (IDC_UPDATE as i32));
                    }

                    PropSheet_SetWizButtons!(
                        GetParent(hwndDlg),
                        PSWIZB_BACK | PSWIZB_NEXT,
                    );
                }

                PSN_QUERYCANCEL => {
                    if DisplayMessage(
                        GetParent(hwndDlg),
                        MB_ICONQUESTION | MB_YESNO | MB_DEFBUTTON2,
                        MAKEINTRESOURCEW!(IDS_ABORTSETUP2),
                        MAKEINTRESOURCEW!(IDS_ABORTSETUP),
                    ) == (IDYES as i32)
                    {
                        /* Go to the Abort page */
                        PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_ABORTPAGE);
                    }

                    /* Do not close the wizard too soon */
                    SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                    return TRUE as INT_PTR;
                }

                PSN_WIZNEXT => {
                    /* Set the selected data */
                    /*
                     * Go update only if we have available NT installations
                     * and we choose to do so.
                     */
                    if !(*pSetupData).NtOsInstallsList.is_null()
                        && GetNumberOfListEntries((*pSetupData).NtOsInstallsList) != 0
                        && IsDlgButtonChecked(hwndDlg, (IDC_UPDATE as i32)) == BST_CHECKED
                    {
                        (*pSetupData).RepairUpdateFlag = TRUE as BOOLEAN;

                        /*
                         * Display the existing NT installations page only
                         * if we have more than one available NT installations.
                         */
                        if GetNumberOfListEntries((*pSetupData).NtOsInstallsList) > 1 {
                            /* pSetupData->CurrentInstallation will be set from within IDD_UPDATEREPAIRPAGE */

                            /* Actually the best would be to dynamically insert the page only when needed */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_UPDATEREPAIRPAGE) as LONG_PTR);
                        } else {
                            /* Retrieve the current installation */
                            (*pSetupData).CurrentInstallation =
                                GetListEntryData(GetCurrentListEntry((*pSetupData).NtOsInstallsList))
                                    as PNTOS_INSTALLATION;
                            InstallPartition = (*(*pSetupData).CurrentInstallation).Volume.read().PartEntry;
                            StringCchCopyW(
                                ((*pSetupData).USetupData.InstallationDirectory).as_mut_ptr(),
                                (_countof!((*pSetupData).USetupData.InstallationDirectory)) as _,
                                (*(*pSetupData).CurrentInstallation).PathComponent,
                            );

                            /* Jump to the Summary page during repair/upgrade */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_SUMMARYPAGE) as LONG_PTR);
                        }
                    } else {
                        (*pSetupData).CurrentInstallation = core::ptr::null_mut();
                        (*pSetupData).RepairUpdateFlag = FALSE as BOOLEAN;
                        SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_DEVICEPAGE) as LONG_PTR);
                    }

                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateListViewColumns(
    hInstance: HINSTANCE,
    hWndListView: HWND,
    pIDs: *const UINT,
    pColsWidth: *const INT,
    pColsAlign: *const INT,
    nNumOfColumns: UINT,
) -> BOOL {
    let mut i: UINT;
    let mut lvC: LVCOLUMN = core::mem::zeroed::<LVCOLUMN>();
    let mut szText: [WCHAR; 50] = [0; 50];

    /* Create the columns */
    lvC.mask = (LVCF_FMT | LVCF_WIDTH | LVCF_TEXT | LVCF_SUBITEM) as UINT;
    lvC.pszText = core::ptr::addr_of_mut!(szText) as *mut WCHAR;

    /* Load the column labels from the resource file */
    i = 0 as UINT;
    while i < nNumOfColumns {
        lvC.iSubItem = i as INT;
        lvC.cx = *pColsWidth.offset(i as isize);
        lvC.fmt = *pColsAlign.offset(i as isize);

        LoadStringW(
            hInstance,
            *pIDs.offset(i as isize),
            core::ptr::addr_of_mut!(szText) as *mut WCHAR,
            (ARRAYSIZE!(szText)) as i32,
        );

        if ListView_InsertColumn!(hWndListView, i as INT, core::ptr::addr_of_mut!(lvC)) == -1 {
            return (FALSE) as BOOL;
        }
        i = i.wrapping_add(1);
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitGenericComboList(
    hWndList: HWND,
    List: PGENERIC_LIST,
    GetEntryDescriptionProc: PGET_ENTRY_DESCRIPTION,
) {
    let mut Index: INT = 0;
    let mut CurrentEntryIndex: INT = 0 as INT;
    let mut ListEntry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut CurrentItemText: [WCHAR; 256] = [0; 256];

    Entry = (*List).ListHead.Flink as PLIST_ENTRY;
    while Entry != core::ptr::addr_of_mut!((*List).ListHead) {
        ListEntry = ((Entry as ULONG_PTR).wrapping_sub(
            core::mem::offset_of!(GENERIC_LIST_ENTRY, Entry) as ULONG_PTR,
        )) as PGENERIC_LIST_ENTRY;

        if GetEntryDescriptionProc.is_some() {
            let GetEntryDescriptionProc = GetEntryDescriptionProc.unwrap_unchecked();
            GetEntryDescriptionProc(
                ListEntry,
                core::ptr::addr_of_mut!(CurrentItemText) as PWSTR,
                (ARRAYSIZE!(CurrentItemText)) as _,
            );
            Index = SendMessageW(
                hWndList,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                core::ptr::addr_of_mut!(CurrentItemText) as *mut WCHAR as LPARAM,
            ) as INT;
        } else {
            Index = SendMessageW(
                hWndList,
                CB_ADDSTRING as UINT,
                0 as WPARAM,
                RUSTCONV_WSTR_6AF5D4B374C5.as_ptr() as LPARAM,
            ) as INT;
        }

        if ListEntry == (*List).CurrentEntry {
            CurrentEntryIndex = Index;
        }

        SendMessageW(
            hWndList,
            CB_SETITEMDATA as UINT,
            Index as WPARAM,
            ListEntry as LPARAM,
        );

        Entry = (*Entry).Flink as PLIST_ENTRY;
    }

    SendMessageW(
        hWndList,
        CB_SETCURSEL as UINT,
        CurrentEntryIndex as WPARAM,
        0 as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedComboListItem(mut hWndList: HWND) -> PVOID {
    let mut Index: INT = 0;

    Index = ComboBox_GetCurSel!(hWndList);
    if Index == CB_ERR {
        return core::ptr::null_mut();
    }

    return ComboBox_GetItemData!(hWndList, Index) as PVOID;
}
#[no_mangle]
pub unsafe extern "C" fn InitGenericListView(
    hWndList: HWND,
    List: PGENERIC_LIST,
    AddEntryItemProc: PADD_ENTRY_ITEM,
) {
    let mut CurrentEntryIndex: INT = 0;
    let mut lvItem: LVITEMW = core::mem::zeroed();
    let mut ListEntry: PGENERIC_LIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PLIST_ENTRY = core::ptr::null_mut();
    let mut CurrentItemText: [WCHAR; 256] = [0; 256];

    Entry = (*List).ListHead.Flink;
    while Entry != core::ptr::addr_of_mut!((*List).ListHead) {
        ListEntry = ((Entry as ULONG_PTR)
            .wrapping_sub(core::mem::offset_of!(GENERIC_LIST_ENTRY, Entry) as ULONG_PTR))
            as PGENERIC_LIST_ENTRY;

        if AddEntryItemProc.is_none() {
            Entry = (*Entry).Flink;
            continue;
        }

        (AddEntryItemProc.unwrap_unchecked())(
            hWndList,
            core::ptr::addr_of_mut!(lvItem),
            ListEntry,
            core::ptr::addr_of_mut!(CurrentItemText) as PWSTR,
            (ARRAYSIZE!(CurrentItemText)) as _,
        );

        if ListEntry == (*List).CurrentEntry {
            CurrentEntryIndex = lvItem.iItem;
        }

        Entry = (*Entry).Flink;
    }

    ListView_EnsureVisible!(hWndList, CurrentEntryIndex, FALSE);
    ListView_SetItemState!(
        hWndList,
        CurrentEntryIndex,
        LVIS_FOCUSED | LVIS_SELECTED,
        LVIS_FOCUSED | LVIS_SELECTED,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GetSelectedListViewItem(hWndList: HWND) -> PVOID {
    let mut Index: INT;
    let mut item: LVITEM = core::mem::zeroed();

    Index = ListView_GetSelectionMark!(hWndList);
    if Index == LB_ERR {
        return core::ptr::null_mut();
    }

    item.mask = LVIF_PARAM;
    item.iItem = Index;
    ListView_GetItem!(hWndList, core::ptr::addr_of_mut!(item));

    return item.lParam as PVOID;
}
unsafe extern "system" fn GetSettingDescription(
    Entry: PGENERIC_LIST_ENTRY,
    Buffer: PWSTR,
    cchBufferSize: SIZE_T,
) {
    StringCchCopyW(
        Buffer,
        (cchBufferSize) as _,
        (*(GetListEntryData(Entry) as PGENENTRY)).Value,
    );
}
unsafe extern "system" fn AddNTOSInstallationItem(
    hWndList: HWND,
    plvItem: *mut LVITEM,
    Entry: PGENERIC_LIST_ENTRY,
    Buffer: PWSTR,
    cchBufferSize: SIZE_T,
) {
    let NtOsInstall: PNTOS_INSTALLATION = GetListEntryData(Entry) as PNTOS_INSTALLATION;
    let VolInfo: PVOLINFO = if !(*NtOsInstall).Volume.is_null() {
        core::ptr::addr_of_mut!((*(*NtOsInstall).Volume).Info)
    } else {
        core::ptr::null_mut()
    };

    if !VolInfo.is_null() && (*VolInfo).DriveLetter != 0 {
        /* We have retrieved a partition that is mounted */
        StringCchPrintfW!(
            Buffer,
            cchBufferSize,
            RUSTCONV_WSTR_BF74AC1CFE02.as_ptr(),
            (*VolInfo).DriveLetter as ::core::ffi::c_int,
            (*NtOsInstall).PathComponent,
        );
    } else {
        /* We failed somewhere, just show the NT path */
        StringCchPrintfW!(
            Buffer,
            cchBufferSize,
            RUSTCONV_WSTR_E9D409C4F426.as_ptr(),
            core::ptr::addr_of_mut!((*NtOsInstall).SystemNtPath),
        );
    }

    (*plvItem).mask = LVIF_IMAGE | LVIF_TEXT | LVIF_PARAM;
    (*plvItem).iItem = 0;
    (*plvItem).iSubItem = 0;
    (*plvItem).lParam = Entry as LPARAM;
    (*plvItem).pszText = core::ptr::addr_of_mut!((*NtOsInstall).InstallationName) as *mut WCHAR as LPWSTR;

    /* Associate vendor icon */
    if !FindSubStrI(
        core::ptr::addr_of_mut!((*NtOsInstall).VendorName) as *mut WCHAR as PCWSTR,
        (VENDOR_REACTOS).as_ptr(),
    )
    .is_null()
    {
        (*plvItem).mask |= LVIF_IMAGE;
        (*plvItem).iImage = 0;
    } else if !FindSubStrI(
        core::ptr::addr_of_mut!((*NtOsInstall).VendorName) as *mut WCHAR as PCWSTR,
        (VENDOR_MICROSOFT).as_ptr(),
    )
    .is_null()
    {
        (*plvItem).mask |= LVIF_IMAGE;
        (*plvItem).iImage = 1;
    }

    (*plvItem).iItem = SendMessageW(
        hWndList,
        LVM_INSERTITEMW,
        0,
        plvItem as LPARAM,
    ) as ::core::ffi::c_int;

    (*plvItem).iSubItem = 1;
    (*plvItem).pszText = Buffer; // SystemRootPath;
    SendMessageW(
        hWndList,
        LVM_SETITEMTEXTW,
        (*plvItem).iItem as WPARAM,
        plvItem as LPARAM,
    );

    (*plvItem).iSubItem = 2;
    (*plvItem).pszText = core::ptr::addr_of_mut!((*NtOsInstall).VendorName) as *mut WCHAR as LPWSTR;
    SendMessageW(
        hWndList,
        LVM_SETITEMTEXTW,
        (*plvItem).iItem as WPARAM,
        plvItem as LPARAM,
    );
}

unsafe extern "system" fn UpgradeRepairDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let mut hList: HWND = core::ptr::null_mut();
    let mut hSmall: HIMAGELIST = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    'outer: {
        match uMsg {
            WM_INITDIALOG => {
                /* Save pointer to the global setup data */
                pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
                SetWindowLongPtrW(hwndDlg, DWLP_USER, (pSetupData as DWORD_PTR) as LONG_PTR);

                hList = GetDlgItem(hwndDlg, (IDC_NTOSLIST as i32));

                ListView_SetExtendedListViewStyleEx!(hList, LVS_EX_FULLROWSELECT, LVS_EX_FULLROWSELECT);

                CreateListViewColumns(
                    (*pSetupData).hInstance,
                    hList,
                    core::ptr::addr_of!(column_ids) as *const UINT,
                    core::ptr::addr_of!(column_widths) as *const INT,
                    core::ptr::addr_of!(column_alignment) as *const INT,
                    (MAX_LIST_COLUMNS) as UINT,
                );

                /* Create the ImageList */
                hSmall = ImageList_Create(
                    GetSystemMetrics((SM_CXSMICON as i32)),
                    GetSystemMetrics((SM_CYSMICON as i32)),
                    ILC_COLOR32 | ILC_MASK, // ILC_COLOR24
                    1,
                    1,
                );

                /* Add event type icons to the ImageList */
                ImageList_AddIcon!(
                    hSmall,
                    LoadIconW((*pSetupData).hInstance, MAKEINTRESOURCEW!(IDI_ROSICON)),
                );
                ImageList_AddIcon!(
                    hSmall,
                    LoadIconW((*pSetupData).hInstance, MAKEINTRESOURCEW!(IDI_WINICON)),
                );

                /* Assign the ImageList to the List View */
                ListView_SetImageList!(hList, hSmall, LVSIL_SMALL);

                InitGenericListView(
                    hList,
                    (*pSetupData).NtOsInstallsList,
                    Some(AddNTOSInstallationItem),
                );
                break 'outer;
            }

            WM_DESTROY => {
                hList = GetDlgItem(hwndDlg, (IDC_NTOSLIST as i32));
                hSmall = ListView_GetImageList!(hList, LVSIL_SMALL);
                ListView_SetImageList!(hList, NULL, LVSIL_SMALL);
                ImageList_Destroy(hSmall);
                return (TRUE) as INT_PTR;
            }

            WM_COMMAND => {
                if HIWORD!(wParam) == (BN_CLICKED as u16) && LOWORD!(wParam) == (IDC_SKIPUPGRADE as u16) {
                    /* Skip the upgrade and do the usual new-installation workflow */
                    (*pSetupData).CurrentInstallation = core::ptr::null_mut();
                    (*pSetupData).RepairUpdateFlag = (FALSE as u8);
                    PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_DEVICEPAGE);
                    return (TRUE) as INT_PTR;
                }
                break 'outer;
            }

            WM_NOTIFY => {
                let lpnm: LPNMHDR = lParam as LPNMHDR;

                if (*lpnm).idFrom == (IDC_NTOSLIST as _) && (*lpnm).code == NM_DBLCLK {
                    /* Go to the next page if the user double-clicked on an installation */
                    let pnmv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;
                    if (*pnmv).iItem != -1 {
                        PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_NEXT);
                    }
                    break 'outer;
                }

                if (*lpnm).idFrom == (IDC_NTOSLIST as _) && (*lpnm).code == (LVN_ITEMCHANGED) as u32 {
                    let pnmv: LPNMLISTVIEW = lParam as LPNMLISTVIEW;

                    /* Check whether the item has been (de)selected */
                    if (*pnmv).uChanged & LVIF_STATE == 0
                        || ((*pnmv).uOldState ^ (*pnmv).uNewState) & LVIS_SELECTED == 0
                    {
                        break 'outer;
                    }

                    /* Enable or disable the "Next" button when the user
                     * selects or deselects an installation to upgrade */
                    if (*pnmv).uNewState & LVIS_SELECTED != 0 {
                        PropSheet_SetWizButtons!(
                            GetParent(hwndDlg),
                            PSWIZB_BACK | PSWIZB_NEXT,
                        );
                    } else {
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                    }
                    break 'outer;
                }

                'inner: {
                    match ((*lpnm).code) as i32 {
                        PSN_SETACTIVE => {
                            /* Keep the "Next" button disabled. It will be enabled only
                             * when the user selects an installation to upgrade. */
                            PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                            break 'inner;
                        }

                        PSN_QUERYINITIALFOCUS => {
                            /* Reselect the currently selected item, so as to refresh the UI buttons */
                            let mut Index: INT;
                            hList = GetDlgItem(hwndDlg, (IDC_NTOSLIST as i32));
                            Index = ListView_GetSelectionMark!(hList);
                            if Index != LB_ERR {
                                /* Deselect first the item before reselecting it, so as to
                                 * invalidate its cached state and have the LVN_ITEMCHANGED
                                 * notification sent. */
                                //ListView_EnsureVisible(hList, Index, FALSE);
                                ListView_SetItemState!(hList, Index, 0, LVIS_SELECTED);
                                ListView_SetItemState!(
                                    hList,
                                    Index,
                                    LVIS_FOCUSED | LVIS_SELECTED,
                                    LVIS_FOCUSED | LVIS_SELECTED,
                                );
                            }

                            /* Focus on the installations list */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, hList as LONG_PTR);
                            return (TRUE) as INT_PTR;
                        }

                        PSN_QUERYCANCEL => {
                            if DisplayMessage(
                                GetParent(hwndDlg),
                                MB_ICONQUESTION | MB_YESNO | MB_DEFBUTTON2,
                                MAKEINTRESOURCEW!(IDS_ABORTSETUP2),
                                MAKEINTRESOURCEW!(IDS_ABORTSETUP),
                            ) == (IDYES as i32)
                            {
                                /* Go to the Abort page */
                                PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_ABORTPAGE);
                            }

                            /* Do not close the wizard too soon */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                            return (TRUE) as INT_PTR;
                        }

                        PSN_WIZNEXT => {
                            /* Set the selected data */
                            /*
                             * Go update only if we have available NT installations
                             * and we choose to do so.
                             */
                            if (*pSetupData).NtOsInstallsList.is_null()
                                || GetNumberOfListEntries((*pSetupData).NtOsInstallsList) == 0
                            {
                                (*pSetupData).CurrentInstallation = core::ptr::null_mut();
                                (*pSetupData).RepairUpdateFlag = (FALSE as u8);
                                break 'inner;
                            }

                            hList = GetDlgItem(hwndDlg, (IDC_NTOSLIST as i32));
                            SetCurrentListEntry(
                                (*pSetupData).NtOsInstallsList,
                                (GetSelectedListViewItem(hList)).cast(),
                            );

                            /* Retrieve the current installation */
                            (*pSetupData).CurrentInstallation = GetListEntryData(
                                GetCurrentListEntry((*pSetupData).NtOsInstallsList),
                            ) as PNTOS_INSTALLATION;
                            InstallPartition = (*(*(*pSetupData).CurrentInstallation).Volume).PartEntry;
                            StringCchCopyW(
                                core::ptr::addr_of_mut!((*pSetupData).USetupData.InstallationDirectory)
                                    as STRSAFE_LPWSTR,
                                (_countof!((*pSetupData).USetupData.InstallationDirectory)) as _,
                                (*(*pSetupData).CurrentInstallation).PathComponent as STRSAFE_LPCWSTR,
                            );

                            /* We perform an upgrade */
                            (*pSetupData).RepairUpdateFlag = (TRUE as u8);
                            /* Jump to the Summary page during repair/upgrade */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_SUMMARYPAGE) as LONG_PTR);
                            return (TRUE) as INT_PTR;
                        }

                        _ => {
                            break 'inner;
                        }
                    }
                }
                break 'outer;
            }

            _ => {
                break 'outer;
            }
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn DeviceDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut::<_SETUPDATA>();
    let mut hList: HWND = core::ptr::null_mut::<HWND__>();

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER as ::core::ffi::c_int) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            /* Save pointer to the global setup data */
            pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
            SetWindowLongPtrW(
                hwndDlg,
                DWLP_USER as ::core::ffi::c_int,
                pSetupData as DWORD_PTR as LONG_PTR,
            );

            hList = GetDlgItem(hwndDlg, (IDC_COMPUTER as i32));
            InitGenericComboList(
                hList,
                (*pSetupData).USetupData.ComputerList,
                Some(GetSettingDescription),
            );

            hList = GetDlgItem(hwndDlg, (IDC_DISPLAY as i32));
            InitGenericComboList(
                hList,
                (*pSetupData).USetupData.DisplayList,
                Some(GetSettingDescription),
            );

            hList = GetDlgItem(hwndDlg, (IDC_KEYBOARD as i32));
            InitGenericComboList(
                hList,
                (*pSetupData).USetupData.KeyboardList,
                Some(GetSettingDescription),
            );

            // hList = GetDlgItem(hwndDlg, IDC_KEYBOARD_LAYOUT);
            // InitGenericComboList(hList, pSetupData->USetupData.LayoutList, GetSettingDescription);

            return TRUE as INT_PTR;
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    /* In unattended mode, don't allow going backwards further, i.e.
                     * back to the Upgrade/Repair, the Install type, or the Start pages,
                     * in case the setup is interrupted and the user manually goes back. */
                    if (*pSetupData).bUnattend != 0 {
                        PropSheet_SetWizButtons!(
                            GetParent(hwndDlg),
                            PSWIZB_NEXT as WPARAM,
                        );
                    } else {
                        PropSheet_SetWizButtons!(
                            GetParent(hwndDlg),
                            (PSWIZB_BACK | PSWIZB_NEXT) as WPARAM,
                        );
                    }

                    /* In unattended mode, switch directly to the next page.
                     * TODO: *UNLESS* there are inconsistencies in the data,
                     * in which case we should stay on the page! */
                    if (*pSetupData).bUnattend != 0 {
                        SetWindowLongPtrW(
                            hwndDlg,
                            DWLP_MSGRESULT as ::core::ffi::c_int,
                            -(1 as LONG_PTR),
                        );
                        return TRUE as INT_PTR;
                    }
                }

                PSN_QUERYCANCEL => {
                    if DisplayMessage(
                        GetParent(hwndDlg),
                        (MB_ICONQUESTION | MB_YESNO | MB_DEFBUTTON2) as UINT,
                        MAKEINTRESOURCEW!(IDS_ABORTSETUP2),
                        MAKEINTRESOURCEW!(IDS_ABORTSETUP),
                    ) == (IDYES as i32)
                    {
                        /* Go to the Abort page */
                        PropSheet_SetCurSelByID!(
                            GetParent(hwndDlg),
                            IDD_ABORTPAGE as WPARAM,
                        );
                    }

                    /* Do not close the wizard too soon */
                    SetWindowLongPtrW(
                        hwndDlg,
                        DWLP_MSGRESULT as ::core::ffi::c_int,
                        TRUE as LONG_PTR,
                    );
                    return TRUE as INT_PTR;
                }

                PSN_WIZNEXT => {
                    /* Set the selected data */
                    hList = GetDlgItem(hwndDlg, (IDC_COMPUTER as i32));
                    SetCurrentListEntry(
                        (*pSetupData).USetupData.ComputerList,
                        GetSelectedComboListItem(hList) as PGENERIC_LIST_ENTRY,
                    );

                    hList = GetDlgItem(hwndDlg, (IDC_DISPLAY as i32));
                    SetCurrentListEntry(
                        (*pSetupData).USetupData.DisplayList,
                        GetSelectedComboListItem(hList) as PGENERIC_LIST_ENTRY,
                    );

                    hList = GetDlgItem(hwndDlg, (IDC_KEYBOARD as i32));
                    SetCurrentListEntry(
                        (*pSetupData).USetupData.KeyboardList,
                        GetSelectedComboListItem(hList) as PGENERIC_LIST_ENTRY,
                    );

                    // hList = GetDlgItem(hwndDlg, IDC_KEYBOARD_LAYOUT);
                    // SetCurrentListEntry(pSetupData->USetupData.LayoutList,
                    //                     GetSelectedComboListItem(hList));

                    return TRUE as INT_PTR;
                }

                PSN_WIZBACK => {
                    /* Return to the Install type selection page instead of the Repair/Upgrade page */
                    SetWindowLongW(
                        hwndDlg,
                        DWLP_MSGRESULT as ::core::ffi::c_int,
                        IDD_TYPEPAGE as LONG,
                    );
                    return TRUE as INT_PTR;
                }

                _ => {}
            }
        }

        _ => {}
    }

    return FALSE as INT_PTR;
}
unsafe extern "system" fn SummaryDlgProc(
    mut hwndDlg: HWND,
    mut uMsg: UINT,
    mut wParam: WPARAM,
    mut lParam: LPARAM,
) -> INT_PTR {
    static mut szOrgWizNextBtnText: [WCHAR; 260] = [0; 260];

    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    'outer: {
        match uMsg {
            WM_INITDIALOG => {
                /* Save pointer to the global setup data */
                pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
                SetWindowLongPtrW(hwndDlg, DWLP_USER, pSetupData as DWORD_PTR as LONG_PTR);
                break 'outer;
            }

            WM_COMMAND => {
                if HIWORD!(wParam) == (BN_CLICKED as u16) && LOWORD!(wParam) == (IDC_CONFIRM_INSTALL as u16) {
                    // Ideally we could add the PSWIZBF_ELEVATIONREQUIRED style.
                    if IsDlgButtonChecked(hwndDlg, (IDC_CONFIRM_INSTALL as i32)) == BST_CHECKED {
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK | PSWIZB_NEXT);
                    } else {
                        PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                    }
                    return (TRUE) as INT_PTR;
                }
                break 'outer;
            }

            WM_NOTIFY => {
                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                'inner: {
                    match ((*lpnm).code) as i32 {
                        PSN_SETACTIVE => {
                            let mut CurrentItemText: [WCHAR; 256] = [0; 256];

                            ASSERT!(!InstallPartition.is_null());

                            /* Skip the Summary page in unattended setup */
                            if (*pSetupData).bUnattend != 0 {
                                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, -1);
                                return (TRUE) as INT_PTR;
                            }

                            /* Show the current selected settings */

                            // FIXME! Localize
                            if (*pSetupData).RepairUpdateFlag != 0 {
                                StringCchPrintfW!(
                                    addr_of_mut!(CurrentItemText) as *mut WCHAR,
                                    ARRAYSIZE!(CurrentItemText),
                                    RUSTCONV_WSTR_554CA2A34148.as_ptr(),
                                    addr_of_mut!((*(*pSetupData).CurrentInstallation).InstallationName) as *mut WCHAR,
                                    addr_of_mut!((*(*pSetupData).CurrentInstallation).VendorName) as *mut WCHAR,
                                );
                            } else {
                                StringCchCopyW(
                                    addr_of_mut!(CurrentItemText) as *mut WCHAR,
                                    (ARRAYSIZE!(CurrentItemText)) as _,
                                    RUSTCONV_WSTR_949D726850F4.as_ptr(),
                                );
                            }
                            SetDlgItemTextW(hwndDlg, (IDC_INSTALLTYPE as i32), addr_of_mut!(CurrentItemText) as *mut WCHAR as LPCWSTR);

                            SetDlgItemTextW(hwndDlg, (IDC_INSTALLSOURCE as i32), RUSTCONV_WSTR_6AF5D4B374C5.as_ptr());
                            SetDlgItemTextW(hwndDlg, (IDC_ARCHITECTURE as i32), RUSTCONV_WSTR_6AF5D4B374C5.as_ptr());

                            GetSettingDescription(
                                GetCurrentListEntry((*pSetupData).USetupData.ComputerList),
                                addr_of_mut!(CurrentItemText) as *mut WCHAR as PWSTR,
                                (ARRAYSIZE!(CurrentItemText)) as SIZE_T,
                            );
                            SetDlgItemTextW(hwndDlg, (IDC_COMPUTER as i32), addr_of_mut!(CurrentItemText) as *mut WCHAR as LPCWSTR);

                            GetSettingDescription(
                                GetCurrentListEntry((*pSetupData).USetupData.DisplayList),
                                addr_of_mut!(CurrentItemText) as *mut WCHAR as PWSTR,
                                (ARRAYSIZE!(CurrentItemText)) as SIZE_T,
                            );
                            SetDlgItemTextW(hwndDlg, (IDC_DISPLAY as i32), addr_of_mut!(CurrentItemText) as *mut WCHAR as LPCWSTR);

                            GetSettingDescription(
                                GetCurrentListEntry((*pSetupData).USetupData.KeyboardList),
                                addr_of_mut!(CurrentItemText) as *mut WCHAR as PWSTR,
                                (ARRAYSIZE!(CurrentItemText)) as SIZE_T,
                            );
                            SetDlgItemTextW(hwndDlg, (IDC_KEYBOARD as i32), addr_of_mut!(CurrentItemText) as *mut WCHAR as LPCWSTR);

                            if (*(*InstallPartition).Volume).Info.DriveLetter != 0 {
                                StringCchPrintfW!(
                                    addr_of_mut!(CurrentItemText) as *mut WCHAR,
                                    ARRAYSIZE!(CurrentItemText),
                                    RUSTCONV_WSTR_18F3B0B9E62F.as_ptr(),
                                    (*(*InstallPartition).Volume).Info.DriveLetter as core::ffi::c_uint,
                                    (*(*InstallPartition).DiskEntry).DiskNumber,
                                    (*InstallPartition).OnDiskPartitionNumber,
                                );
                            } else {
                                StringCchPrintfW!(
                                    addr_of_mut!(CurrentItemText) as *mut WCHAR,
                                    ARRAYSIZE!(CurrentItemText),
                                    RUSTCONV_WSTR_2457C191431A.as_ptr(),
                                    (*(*InstallPartition).DiskEntry).DiskNumber,
                                    (*InstallPartition).OnDiskPartitionNumber,
                                );
                            }
                            SetDlgItemTextW(hwndDlg, (IDC_DESTDRIVE as i32), addr_of_mut!(CurrentItemText) as *mut WCHAR as LPCWSTR);

                            SetDlgItemTextW(
                                hwndDlg,
                                (IDC_PATH as i32),
                                addr_of_mut!((*pSetupData).USetupData.InstallationDirectory) as *mut WCHAR as LPCWSTR,
                            );

                            /* Change the "Next" button text to "Install" */
                            // PropSheet_SetNextText(GetParent(hwndDlg), ...);
                            GetDlgItemTextW(
                                GetParent(hwndDlg),
                                (ID_WIZNEXT as i32),
                                addr_of_mut!(szOrgWizNextBtnText) as *mut WCHAR,
                                (ARRAYSIZE!(szOrgWizNextBtnText)) as i32,
                            );
                            SetWindowResTextW(
                                GetDlgItem(GetParent(hwndDlg), (ID_WIZNEXT as i32)),
                                (*pSetupData).hInstance,
                                IDS_INSTALLBTN,
                            );

                            /* Keep the "Next" button disabled. It will be enabled only
                             * when the user clicks on the installation approval checkbox. */
                            CheckDlgButton(hwndDlg, (IDC_CONFIRM_INSTALL as i32), BST_UNCHECKED);
                            PropSheet_SetWizButtons!(GetParent(hwndDlg), PSWIZB_BACK);
                            break 'inner;
                        }

                        PSN_QUERYINITIALFOCUS => {
                            /* Focus on the confirmation check-box */
                            SetWindowLongPtrW(
                                hwndDlg,
                                DWLP_MSGRESULT,
                                GetDlgItem(hwndDlg, (IDC_CONFIRM_INSTALL as i32)) as LONG_PTR,
                            );
                            return (TRUE) as INT_PTR;
                        }

                        PSN_KILLACTIVE => {
                            /* Restore the original "Next" button text */
                            SetDlgItemTextW(
                                GetParent(hwndDlg),
                                (ID_WIZNEXT as i32),
                                addr_of_mut!(szOrgWizNextBtnText) as *mut WCHAR as LPCWSTR,
                            );
                            break 'inner;
                        }

                        PSN_QUERYCANCEL => {
                            if DisplayMessage(
                                GetParent(hwndDlg),
                                MB_ICONQUESTION | MB_YESNO | MB_DEFBUTTON2,
                                MAKEINTRESOURCEW!(IDS_ABORTSETUP2),
                                MAKEINTRESOURCEW!(IDS_ABORTSETUP),
                            ) == (IDYES as i32)
                            {
                                /* Go to the Abort page */
                                PropSheet_SetCurSelByID!(GetParent(hwndDlg), IDD_ABORTPAGE);
                            }

                            /* Do not close the wizard too soon */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                            return (TRUE) as INT_PTR;
                        }

                        PSN_WIZBACK => {
                            /* When the user performs a regular installation, go back to the previous page */
                            if (*pSetupData).RepairUpdateFlag == 0 {
                                break 'inner;
                            }

                            if GetNumberOfListEntries((*pSetupData).NtOsInstallsList) > 1 {
                                /* Return to the Upgrade/Repair selection page, when the user is
                                 * upgrading and there are more than one installation available */
                                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_UPDATEREPAIRPAGE) as LONG_PTR);
                            } else {
                                /* Return to the Install type selection page, when the user is
                                 * upgrading and there is at most one installation available */
                                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (IDD_TYPEPAGE) as LONG_PTR);
                            }
                            return (TRUE) as INT_PTR;
                        }

                        _ => {
                            break 'inner;
                        }
                    }
                }
                break 'outer;
            }

            _ => {
                break 'outer;
            }
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "system" fn FormatCallback(
    Command: CALLBACKCOMMAND,
    Modifier: ULONG,
    Argument: PVOID,
) -> BOOLEAN {
    match Command {
        PROGRESS => {
            let Percent: PULONG = Argument as PULONG;
            DPRINT!("%lu percent completed\n", *Percent);
            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETPOS as UINT,
                *Percent as WPARAM,
                0 as LPARAM,
            );
        }

        DONE => {
            DPRINT!("Done\n");
        }

        _ => {
            DPRINT!("Unknown callback %lu\n", Command as ULONG);
        }
    }

    return TRUE as BOOLEAN;
}
unsafe extern "system" fn ChkdskCallback(
    mut Command: CALLBACKCOMMAND,
    mut Modifier: ULONG,
    mut Argument: PVOID,
) -> BOOLEAN {
    match Command as ULONG {
        _ => {
            DPRINT!("Unknown callback %lu\n", Command as ULONG);
        }
    }

    return (TRUE) as BOOLEAN;
}

unsafe extern "system" fn FsVolCallback(
    Context: PVOID,
    FormatStatus: FSVOLNOTIFY,
    Param1: ULONG_PTR,
    Param2: ULONG_PTR,
) -> FSVOL_OP {
    let mut FsVolContext: PFSVOL_CONTEXT = Context as PFSVOL_CONTEXT;

    match FormatStatus {
        // FIXME: Deprecate!
        ChangeSystemPartition => {
            return FSVOL_ABORT;
        }

        FSVOLNOTIFY_PARTITIONERROR => {
            match Param1 {
                __c if __c == STATUS_PARTITION_FAILURE as _ => {
                    DisplayError(
                        core::ptr::null_mut(),
                        0,
                        IDS_ERROR_WRITE_PTABLE,
                    );
                }

                __c if __c == ERROR_SYSTEM_PARTITION_NOT_FOUND as _ => {
                    DisplayError(
                        core::ptr::null_mut(),
                        0,
                        IDS_ERROR_SYSTEM_PARTITION,
                    );
                }

                _ => {}
            }
            return FSVOL_ABORT;
        }

        FSVOLNOTIFY_STARTQUEUE | FSVOLNOTIFY_ENDQUEUE => {
            return FSVOL_DOIT;
        }

        FSVOLNOTIFY_STARTSUBQUEUE => {
            if (Param1 as FSVOL_OP) == FSVOL_FORMAT {
                if (*(*FsVolContext).pSetupData).RepairUpdateFlag != 0 {
                    return FSVOL_SKIP;
                }

                SetDlgItemTextW(
                    UiContext.hwndDlg,
                    (IDC_ITEM as i32),
                    RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                );
            } else if (Param1 as FSVOL_OP) == FSVOL_CHECK {
                SetDlgItemTextW(
                    UiContext.hwndDlg,
                    (IDC_ITEM as i32),
                    RUSTCONV_WSTR_96A296D224F2.as_ptr(),
                );

                UiContext.dwPbStyle = GetWindowLongPtrW(
                    UiContext.hWndProgress,
                    GWL_STYLE,
                );
                SetWindowLongPtrW(
                    UiContext.hWndProgress,
                    GWL_STYLE,
                    UiContext.dwPbStyle | PBS_MARQUEE as LONG_PTR,
                );
                SendMessageW(
                    UiContext.hWndProgress,
                    PBM_SETMARQUEE,
                    (TRUE) as WPARAM,
                    0,
                );
            }

            return FSVOL_DOIT;
        }

        FSVOLNOTIFY_ENDSUBQUEUE => {
            if (Param1 as FSVOL_OP) == FSVOL_CHECK {
                SendMessageW(
                    UiContext.hWndProgress,
                    PBM_SETMARQUEE,
                    (FALSE) as WPARAM,
                    0,
                );
                SetWindowLongPtrW(
                    UiContext.hWndProgress,
                    GWL_STYLE,
                    UiContext.dwPbStyle,
                );
            }
            return 0;
        }

        FSVOLNOTIFY_FORMATERROR => {
            let mut FmtInfo: PFORMAT_VOLUME_INFO = Param1 as PFORMAT_VOLUME_INFO;

            if (*FmtInfo).ErrorStatus == STATUS_PARTITION_FAILURE {
                DisplayError(
                    core::ptr::null_mut(),
                    0,
                    IDS_ERROR_WRITE_PTABLE,
                );
                return FSVOL_ABORT;
            } else if (*FmtInfo).ErrorStatus == STATUS_UNRECOGNIZED_VOLUME {
                DisplayError(
                    core::ptr::null_mut(),
                    0,
                    IDS_ERROR_FORMAT_UNRECOGNIZED_VOLUME,
                );
                return FSVOL_ABORT;
            } else if (*FmtInfo).ErrorStatus == STATUS_NOT_SUPPORTED {
                let mut nRet: INT = 0;

                nRet = DisplayMessage(
                    core::ptr::null_mut(),
                    MB_ICONERROR | MB_OKCANCEL,
                    core::ptr::null_mut(),
                    MAKEINTRESOURCEW!(IDS_ERROR_COULD_NOT_FORMAT),
                    (*FmtInfo).FileSystemName,
                );
                if nRet == (IDCANCEL as i32) {
                    return FSVOL_ABORT;
                } else if nRet == (IDOK as i32) {
                    return FSVOL_RETRY;
                }
            } else if !((*FmtInfo).ErrorStatus >= 0 as NTSTATUS) {
                ASSERT!(*(&raw mut (*(*FmtInfo).Volume).Info.DeviceName as *mut WCHAR) != 0);

                DPRINT1!(
                    "FormatPartition() failed with status 0x%08lx\n",
                    (*FmtInfo).ErrorStatus
                );

                DisplayError(
                    core::ptr::null_mut(),
                    0,
                    IDS_ERROR_FORMATTING_PARTITION,
                    &raw mut (*(*FmtInfo).Volume).Info.DeviceName as *mut WCHAR,
                );
                return FSVOL_ABORT;
            }

            return FSVOL_RETRY;
        }

        FSVOLNOTIFY_CHECKERROR => {
            let mut ChkInfo: PCHECK_VOLUME_INFO = Param1 as PCHECK_VOLUME_INFO;

            if (*ChkInfo).ErrorStatus == STATUS_NOT_SUPPORTED {
                let mut nRet_0: INT = 0;

                nRet_0 = DisplayMessage(
                    core::ptr::null_mut(),
                    MB_ICONERROR | MB_OKCANCEL,
                    core::ptr::null_mut(),
                    MAKEINTRESOURCEW!(IDS_ERROR_COULD_NOT_CHECK),
                    &raw mut (*(*ChkInfo).Volume).Info.FileSystem as *mut WCHAR,
                );
                if nRet_0 == (IDCANCEL as i32) {
                    return FSVOL_ABORT;
                } else if nRet_0 == (IDOK as i32) {
                    return FSVOL_SKIP;
                }
            } else if !((*ChkInfo).ErrorStatus >= 0 as NTSTATUS) {
                DPRINT1!(
                    "ChkdskPartition() failed with status 0x%08lx\n",
                    (*ChkInfo).ErrorStatus
                );

                DisplayError(
                    core::ptr::null_mut(),
                    0,
                    IDS_ERROR_CHECKING_PARTITION,
                    (*ChkInfo).ErrorStatus,
                );
                return FSVOL_SKIP;
            }

            return FSVOL_SKIP;
        }

        FSVOLNOTIFY_STARTFORMAT => {
            let mut FmtInfo: PFORMAT_VOLUME_INFO = Param1 as PFORMAT_VOLUME_INFO;
            let mut VolCreate: PVOL_CREATE_INFO;

            ASSERT!((Param2 as FSVOL_OP) == FSVOL_FORMAT);

            VolCreate = FindVolCreateInTreeByVolume(
                UiContext.hPartList,
                (*FmtInfo).Volume,
            );
            if VolCreate.is_null() {
                return FSVOL_SKIP;
            }
            ASSERT!((*VolCreate).Volume == (*FmtInfo).Volume);

            if *(&raw mut (*VolCreate).FileSystemName as *mut WCHAR) == 0 {
                return FSVOL_SKIP;
            }

            ASSERT!(*(&raw mut (*(*FmtInfo).Volume).Info.DeviceName as *mut WCHAR) != 0);

            if (*(*FmtInfo).Volume).Info.DriveLetter != 0 {
                SetWindowResPrintfW(
                    GetDlgItem(UiContext.hwndDlg, (IDC_ITEM as i32)),
                    SetupData.hInstance,
                    IDS_FORMATTING_PROGRESS1,
                    (*(*FmtInfo).Volume).Info.DriveLetter as core::ffi::c_int,
                    &raw mut (*(*FmtInfo).Volume).Info.DeviceName as *mut WCHAR,
                    &raw mut (*VolCreate).FileSystemName as *mut WCHAR,
                );
            } else {
                SetWindowResPrintfW(
                    GetDlgItem(UiContext.hwndDlg, (IDC_ITEM as i32)),
                    SetupData.hInstance,
                    IDS_FORMATTING_PROGRESS2,
                    &raw mut (*(*FmtInfo).Volume).Info.DeviceName as *mut WCHAR,
                    &raw mut (*VolCreate).FileSystemName as *mut WCHAR,
                );
            }

            (*FmtInfo).FileSystemName =
                &raw mut (*VolCreate).FileSystemName as *mut WCHAR as PCWSTR;
            (*FmtInfo).MediaFlag = (*VolCreate).MediaFlag;
            (*FmtInfo).Label = (*VolCreate).Label;
            (*FmtInfo).QuickFormat = (*VolCreate).QuickFormat;
            (*FmtInfo).ClusterSize = (*VolCreate).ClusterSize;
            (*FmtInfo).Callback = Some(FormatCallback);

            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETRANGE,
                0,
                MAKELPARAM!(0, 100),
            );
            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETPOS,
                0,
                0,
            );

            return FSVOL_DOIT;
        }

        FSVOLNOTIFY_ENDFORMAT => {
            let mut FmtInfo: PFORMAT_VOLUME_INFO = Param1 as PFORMAT_VOLUME_INFO;

            if !(*FmtInfo).FileSystemName.is_null() {
                *((*FmtInfo).FileSystemName as PWSTR) = UNICODE_NULL;
            }

            return 0;
        }

        FSVOLNOTIFY_STARTCHECK => {
            let mut ChkInfo: PCHECK_VOLUME_INFO = Param1 as PCHECK_VOLUME_INFO;
            let mut VolCreate: PVOL_CREATE_INFO;

            ASSERT!((Param2 as FSVOL_OP) == FSVOL_CHECK);

            VolCreate = FindVolCreateInTreeByVolume(
                UiContext.hPartList,
                (*ChkInfo).Volume,
            );
            if VolCreate.is_null() {
                return FSVOL_SKIP;
            }
            ASSERT!((*VolCreate).Volume == (*ChkInfo).Volume);

            ASSERT!(*(&raw mut (*(*ChkInfo).Volume).Info.DeviceName as *mut WCHAR) != 0);

            if (*(*ChkInfo).Volume).Info.DriveLetter != 0 {
                SetWindowResPrintfW(
                    GetDlgItem(UiContext.hwndDlg, (IDC_ITEM as i32)),
                    SetupData.hInstance,
                    IDS_CHECKING_PROGRESS1,
                    (*(*ChkInfo).Volume).Info.DriveLetter as core::ffi::c_int,
                    &raw mut (*(*ChkInfo).Volume).Info.DeviceName as *mut WCHAR,
                );
            } else {
                SetWindowResPrintfW(
                    GetDlgItem(UiContext.hwndDlg, (IDC_ITEM as i32)),
                    SetupData.hInstance,
                    IDS_CHECKING_PROGRESS2,
                    &raw mut (*(*ChkInfo).Volume).Info.DeviceName as *mut WCHAR,
                );
            }

            (*ChkInfo).FixErrors = (TRUE as u8);
            (*ChkInfo).Verbose = (FALSE as u8);
            (*ChkInfo).CheckOnlyIfDirty = (TRUE as u8);
            (*ChkInfo).ScanDrive = (FALSE as u8);
            (*ChkInfo).Callback = Some(ChkdskCallback);

            return FSVOL_DOIT;
        }

        FSVOLNOTIFY_ENDCHECK => {
            return 0;
        }

        _ => {}
    }

    return 0;
}

unsafe extern "system" fn FileCopyCallback(
    mut Context: PVOID,
    mut Notification: UINT,
    mut Param1: UINT_PTR,
    mut Param2: UINT_PTR,
) -> UINT {
    let mut CopyContext: PCOPYCONTEXT = Context as PCOPYCONTEXT;
    let mut FilePathInfo: PFILEPATHS_W = core::ptr::null_mut::<_FILEPATHS_W>();
    let mut SrcFileName: PCWSTR = core::ptr::null::<WCHAR>();
    let mut DstFileName: PCWSTR = core::ptr::null::<WCHAR>();
    WaitForSingleObject(
        (*(*CopyContext).pSetupData).hHaltInstallEvent,
        INFINITE,
    );
    if (*(*CopyContext).pSetupData).__bindgen_anon_1.bAbortInstall() != 0 {
        return FILEOP_ABORT;
    }
    match (Notification) as i32 {
        SPFILENOTIFY_STARTSUBQUEUE => {
            (*CopyContext).TotalOperations = Param2 as ULONG;
            (*CopyContext).CompletedOperations = 0 as ULONG;

            /* Set up the progress bar */
            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETRANGE as UINT,
                0 as WPARAM,
                MAKELPARAM!(0 as WORD, (*CopyContext).TotalOperations as WORD) as LPARAM,
            );
            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETSTEP as UINT,
                1 as WPARAM,
                0 as LPARAM,
            );
            SendMessageW(
                UiContext.hWndProgress,
                PBM_SETPOS as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }

        SPFILENOTIFY_STARTDELETE | SPFILENOTIFY_STARTRENAME | SPFILENOTIFY_STARTCOPY => {
            FilePathInfo = Param1 as PFILEPATHS_W;

            if Notification == SPFILENOTIFY_STARTDELETE as UINT {
                /* Display delete message */
                ASSERT!(Param2 == FILEOP_DELETE as UINT_PTR);

                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16) as PCWSTR;
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.offset(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                SetWindowResPrintfW(
                    UiContext.hWndItem,
                    SetupData.hInstance,
                    IDS_DELETING as UINT, // STRING_DELETING
                    DstFileName,
                );
            } else if Notification == SPFILENOTIFY_STARTRENAME as UINT {
                let mut uMsgID: UINT = 0;

                /* Display move/rename message */
                ASSERT!(Param2 == FILEOP_RENAME as UINT_PTR);

                SrcFileName = wcsrchr((*FilePathInfo).Source, 0x005Cu16) as PCWSTR;
                if !SrcFileName.is_null() {
                    SrcFileName = SrcFileName.offset(1);
                } else {
                    SrcFileName = (*FilePathInfo).Source;
                }

                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16) as PCWSTR;
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.offset(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                if _wcsicmp(SrcFileName, DstFileName) != 0 {
                    uMsgID = IDS_MOVING as UINT; // STRING_MOVING
                } else {
                    uMsgID = IDS_RENAMING as UINT; // STRING_RENAMING
                }
                SetWindowResPrintfW(
                    UiContext.hWndItem,
                    SetupData.hInstance,
                    uMsgID,
                    SrcFileName,
                    DstFileName,
                );
            } else if Notification == SPFILENOTIFY_STARTCOPY as UINT {
                /* Display copy message */
                ASSERT!(Param2 == FILEOP_COPY as UINT_PTR);

                DstFileName = wcsrchr((*FilePathInfo).Target, 0x005Cu16) as PCWSTR;
                if !DstFileName.is_null() {
                    DstFileName = DstFileName.offset(1);
                } else {
                    DstFileName = (*FilePathInfo).Target;
                }

                /* Whereas the repair/upgrade procedure may move, rename, or
                 * delete files, the regular installation only copies files.
                 * Therefore, display only the file name instead of the full
                 * "Copying..." message in this case. */
                if SetupData.RepairUpdateFlag != 0 {
                    SetWindowResPrintfW(
                        UiContext.hWndItem,
                        SetupData.hInstance,
                        IDS_COPYING as UINT, // STRING_COPYING
                        DstFileName,
                    );
                } else {
                    SetWindowTextW(UiContext.hWndItem, DstFileName as LPCWSTR);
                }
            }
        }

        SPFILENOTIFY_COPYERROR => {
            FilePathInfo = Param1 as PFILEPATHS_W;

            DPRINT1!(
                "An error happened while trying to copy file '%S' (error 0x%08lx), skipping it...\n",
                (*FilePathInfo).Target,
                (*FilePathInfo).Win32Error,
            );
            return FILEOP_SKIP;
        }

        SPFILENOTIFY_ENDDELETE | SPFILENOTIFY_ENDRENAME | SPFILENOTIFY_ENDCOPY => {
            (*CopyContext).CompletedOperations += 1;

            /* SYSREG checkpoint */
            if (*CopyContext).TotalOperations >> 1 == (*CopyContext).CompletedOperations {
                DPRINT1!("CHECKPOINT:HALF_COPIED\n");
            }

            SendMessageW(
                UiContext.hWndProgress,
                PBM_STEPIT as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }

        _ => {}
    }

    return FILEOP_DOIT;
}

#[no_mangle]
unsafe extern "C" fn RegistryStatus_v(RegStatus: REGISTRY_STATUS, ap_list: *mut ::core::ffi::c_void) {
    static StringIDs: [UINT; 8] = [
        IDS_REG_DONE as UINT,
        IDS_REG_REGHIVEUPDATE as UINT,
        IDS_REG_IMPORTFILE as UINT,
        IDS_REG_DISPLAYSETTINGSUPDATE as UINT,
        IDS_REG_LOCALESETTINGSUPDATE as UINT,
        IDS_REG_ADDKBLAYOUTS as UINT,
        IDS_REG_KEYBOARDSETTINGSUPDATE as UINT,
        IDS_REG_CODEPAGEINFOUPDATE as UINT,
    ];

    if (RegStatus as usize) < _countof!(StringIDs) as usize {
        // va_start/va_end are the shim's (rust_varargs_shim.c); it hands over `&args`.
        let mut args_0: va_list = *(ap_list as *mut va_list);
        SetWindowResPrintfVW(
            UiContext.hWndItem,
            SetupData.hInstance,
            StringIDs[RegStatus as usize],
            (args_0).cast(),
        );
    } else {
        SetWindowResPrintfW(
            UiContext.hWndItem,
            SetupData.hInstance,
            IDS_REG_UNKNOWN as UINT,
            RegStatus as ::core::ffi::c_uint,
        );
    }

    SendMessageW(
        UiContext.hWndProgress,
        PBM_STEPIT as UINT,
        0 as WPARAM,
        0 as LPARAM,
    );
}


#[no_mangle]
pub unsafe extern "C" fn PropSheet_SetCloseCancel(hWndWiz: HWND, Enable: BOOL) {
    EnableDlgItem!(hWndWiz, (IDCANCEL as i32), Enable);
    EnableMenuItem(
        GetSystemMenu(hWndWiz, (FALSE) as BOOL),
        SC_CLOSE,
        MF_BYCOMMAND | (if Enable != 0 { MF_ENABLED } else { MF_GRAYED }),
    );
}

unsafe extern "system" fn PrepareAndDoCopyThread(Param: LPVOID) -> DWORD {
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();
    let hwndDlg: HWND = Param as HWND;
    let hWndParent: HWND = GetParent(hwndDlg);
    let mut hWndProgress: HWND = core::ptr::null_mut();
    let mut dwStyle: LONG_PTR = 0;
    let mut ErrorNumber: ERROR_NUMBER = core::mem::zeroed();
    let mut Success: BOOLEAN = 0;
    let mut Status: NTSTATUS = 0;
    let mut FsVolContext: FSVOL_CONTEXT = core::mem::zeroed();
    let mut CopyContext: COPYCONTEXT = core::mem::zeroed();
    let mut PathBuffer: [WCHAR; RTL_NUMBER_OF_FIELD!(PARTENTRY, DeviceName) + 1] = core::mem::zeroed();

    'quit: {
        /* Retrieve pointer to the global setup data */
        pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

        /* Get the progress handle */
        hWndProgress = GetDlgItem(hwndDlg, (IDC_PROCESSPROGRESS as i32));

        /* Setup global UI context */
        UiContext.hwndDlg = hwndDlg;
        UiContext.hWndItem = GetDlgItem(hwndDlg, (IDC_ITEM as i32));
        UiContext.hWndProgress = hWndProgress;
        UiContext.dwPbStyle = 0;

        /* Disable the Close/Cancel buttons during all partition operations */
        PropSheet_SetCloseCancel(hWndParent, (FALSE) as BOOL);

        /* Create context for the volume/partition operations */
        FsVolContext.pSetupData = pSetupData;

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            IDS_CONFIG_SYSTEM_PARTITION,
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Find or set the active system partition before starting formatting */
        Success = InitSystemPartition(
            (*pSetupData).PartitionList,
            InstallPartition,
            core::ptr::addr_of_mut!(SystemPartition),
            Some(FsVolCallback),
            core::ptr::addr_of_mut!(FsVolContext) as PVOID,
        );
        if Success == 0 {
            /* Display an error if an unexpected failure happened */
            MessageBoxW(hWndParent, RUSTCONV_WSTR_75720173C2CE.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);

            /* Re-enable the Close/Cancel buttons */
            PropSheet_SetCloseCancel(hWndParent, (TRUE) as BOOL);
            break 'quit;
        }

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            IDS_PREPARE_PARTITIONS,
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Apply all pending operations on partitions: formatting and checking */
        Success = FsVolCommitOpsQueue(
            (*pSetupData).PartitionList,
            (*SystemPartition).Volume,
            (*InstallPartition).Volume,
            Some(FsVolCallback),
            core::ptr::addr_of_mut!(FsVolContext) as PVOID,
        );
        if Success == 0 {
            /* Display an error if an unexpected failure happened */
            MessageBoxW(hWndParent, RUSTCONV_WSTR_0A4E772A5FB0.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);

            /* Re-enable the Close/Cancel buttons */
            PropSheet_SetCloseCancel(hWndParent, (TRUE) as BOOL);
            break 'quit;
        }

        /* Re-enable the Close/Cancel buttons */
        PropSheet_SetCloseCancel(hWndParent, (TRUE) as BOOL);

        /* Re-calculate the final destination paths */
        ASSERT!(InstallPartition);
        Status = InitDestinationPaths(
            core::ptr::addr_of_mut!((*pSetupData).USetupData),
            core::ptr::null_mut(), // pSetupData->USetupData.InstallationDirectory,
            (*InstallPartition).Volume,
        );
        Success = NT_SUCCESS!(Status) as BOOLEAN;
        if Success == 0 {
            DisplayMessage(hWndParent, MB_ICONERROR, core::ptr::null_mut(), RUSTCONV_WSTR_992FD7539CF7.as_ptr(), Status);
            break 'quit;
        }

        /*
         * Preparation of the list of files to be copied
         */

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            IDS_PREPARE_FILES,
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Set progress marquee style and start it up */
        dwStyle = GetWindowLongPtrW(hWndProgress, GWL_STYLE);
        SetWindowLongPtrW(hWndProgress, GWL_STYLE, dwStyle | PBS_MARQUEE as LONG_PTR);
        SendMessageW(hWndProgress, PBM_SETMARQUEE, (TRUE) as WPARAM, 0);

        /* Prepare the list of files */
        /* ErrorNumber = */
        Success = PrepareFileCopy(core::ptr::addr_of_mut!((*pSetupData).USetupData), None);

        /* Stop progress and restore its style */
        SendMessageW(hWndProgress, PBM_SETMARQUEE, (FALSE) as WPARAM, 0);
        SetWindowLongPtrW(hWndProgress, GWL_STYLE, dwStyle);

        if /*ErrorNumber != ERROR_SUCCESS*/ Success == 0 {
            /* Display an error only if an unexpected failure happened, and not because the user cancelled the installation */
            if (*pSetupData).__bindgen_anon_1.bAbortInstall() == 0 {
                MessageBoxW(hWndParent, RUSTCONV_WSTR_65DA6EB88F5F.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);
            }
            break 'quit;
        }

        /*
         * Perform the file copy
         */

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            IDS_COPYING_FILES,
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Create context for the copy process */
        CopyContext.pSetupData = pSetupData;
        CopyContext.TotalOperations = 0;
        CopyContext.CompletedOperations = 0;

        /* Do the file copying - The callback handles whether or not we should stop file copying */
        Success = (DoFileCopy(
            core::ptr::addr_of_mut!((*pSetupData).USetupData),
            Some(FileCopyCallback),
            core::ptr::addr_of_mut!(CopyContext) as PVOID,
        )) as u8;
        if Success == 0 {
            /* Display an error only if an unexpected failure happened, and not because the user cancelled the installation */
            if (*pSetupData).__bindgen_anon_1.bAbortInstall() == 0 {
                MessageBoxW(hWndParent, RUSTCONV_WSTR_68288B5D5761.as_ptr(), core::ptr::null_mut(), MB_ICONERROR);
            }
            break 'quit;
        }

        /* Create the $winnt$.inf file */
        InstallSetupInfFile(core::ptr::addr_of_mut!((*pSetupData).USetupData));

        /* Disable the Close/Cancel buttons for the remaining non-cancellable operations */
        PropSheet_SetCloseCancel(hWndParent, (FALSE) as BOOL);

        /*
         * Create or update the registry hives
         */

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            if (*pSetupData).RepairUpdateFlag != 0 { IDS_UPDATE_REGISTRY } else { IDS_CREATE_REGISTRY },
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        /* Set up the progress bar */
        SendMessageW(
            hWndProgress,
            PBM_SETRANGE,
            0,
            MAKELPARAM!(0, 8),
        ); // FIXME: hardcoded number of steps, see StringIDs[] array in RegistryStatus()
        SendMessageW(
            hWndProgress,
            PBM_SETSTEP,
            1,
            0,
        );
        SendMessageW(
            hWndProgress,
            PBM_SETPOS,
            0,
            0,
        );

        ErrorNumber = UpdateRegistry(
            core::ptr::addr_of_mut!((*pSetupData).USetupData),
            (*pSetupData).RepairUpdateFlag,
            (*pSetupData).PartitionList,
            (*(*InstallPartition).Volume).Info.DriveLetter,
            (*pSetupData).SelectedLanguageId,
            Some(RegistryStatus),
            core::ptr::null_mut(), /* SubstSettings */
        );
        DBG_UNREFERENCED_PARAMETER!(ErrorNumber);
        SendMessageW(UiContext.hWndProgress, PBM_SETPOS, 100, 0);

        /*
         * And finally, install the bootloader
         */

        /* Set status text */
        SetWindowResTextW(
            GetDlgItem(hwndDlg, (IDC_ACTIVITY as i32)),
            (*pSetupData).hInstance,
            IDS_INSTALL_BOOTLOADER,
        );
        SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());

        RtlFreeUnicodeString(core::ptr::addr_of_mut!((*pSetupData).USetupData.SystemRootPath));
        StringCchPrintfW!(
            core::ptr::addr_of_mut!(PathBuffer) as *mut WCHAR,
            _countof!(PathBuffer),
            RUSTCONV_WSTR_BCF3913447D5.as_ptr(),
            core::ptr::addr_of_mut!((*SystemPartition).DeviceName) as *mut WCHAR,
        );
        RtlCreateUnicodeString(
            core::ptr::addr_of_mut!((*pSetupData).USetupData.SystemRootPath),
            core::ptr::addr_of_mut!(PathBuffer) as *mut WCHAR,
        );
        DPRINT1!(
            "SystemRootPath: %wZ\n",
            core::ptr::addr_of_mut!((*pSetupData).USetupData.SystemRootPath)
        );

        let sw_val = (*pSetupData).USetupData.BootLoaderLocation;
        'sw: {
            match sw_val {
                /* Install on removable disk */
                _ if sw_val == 1 => {
                    // TODO: So far SETUP only supports the 1st floppy.
                    // Use a simple UI like comdlg32's DlgDirList* to show
                    // a list of drives that the user could select.
                    let FloppyDrive_local: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_123946225403.as_ptr());
                    let DriveLetter: WCHAR = 0x0041u16;

                    let mut nRet: INT = 0;
                    // at 0 = RetryCancel, at 1 = Retry
                    let mut at: u8 = 0;
                    loop {
                        match at {
                            0 => {
                                // RetryCancel:
                                nRet = DisplayMessage(
                                    hWndParent,
                                    MB_ICONINFORMATION | MB_OKCANCEL,
                                    RUSTCONV_WSTR_F4180D6B4EEA.as_ptr(),
                                    RUSTCONV_WSTR_4BBE7B81F51A.as_ptr(),
                                    DriveLetter as ::core::ffi::c_int,
                                );
                                if nRet != (IDOK as i32) {
                                    break 'sw; /* Skip installation */
                                }
                                at = 1;
                            }
                            1 => {
                                // Retry:
                                Status = InstallBootcodeToRemovable(
                                    (*pSetupData).USetupData.ArchType,
                                    core::ptr::addr_of!(FloppyDrive_local),
                                    core::ptr::addr_of_mut!((*pSetupData).USetupData.SourceRootPath),
                                    core::ptr::addr_of_mut!((*pSetupData).USetupData.DestinationArcPath),
                                );
                                if Status == STATUS_SUCCESS {
                                    break 'sw; /* Successful installation */
                                }

                                if Status == STATUS_DEVICE_NOT_READY {
                                    // ERROR_NO_FLOPPY
                                    nRet = DisplayMessage(
                                        hWndParent,
                                        MB_ICONWARNING | MB_RETRYCANCEL,
                                        core::ptr::null_mut(), // Default to "Error"
                                        RUSTCONV_WSTR_18C3046DA6B1.as_ptr(),
                                        DriveLetter as ::core::ffi::c_int,
                                    );
                                    if nRet == (IDRETRY as i32) {
                                        at = 1;
                                        continue; /* goto Retry */
                                    }
                                } else if ((Status) as u32 == ERROR_WRITE_BOOT) || ((Status) as u32 == ERROR_INSTALL_BOOTCODE) {
                                    /* Error when writing the boot code */
                                    DisplayError(
                                        hWndParent,
                                        0, // Default to "Error"
                                        IDS_ERROR_INSTALL_BOOTCODE_REMOVABLE,
                                    );
                                } else if !NT_SUCCESS!(Status) {
                                    /* Any other NTSTATUS failure code */
                                    DPRINT1!("InstallBootcodeToRemovable() failed: Status 0x%lx\n", Status);
                                    DisplayError(
                                        hWndParent,
                                        0, // Default to "Error"
                                        IDS_ERROR_BOOTLDR_FAILED,
                                        Status,
                                    );
                                }
                                at = 0;
                                continue; /* goto RetryCancel */
                            }
                            _ => break,
                        }
                    }
                }

                /* Install on hard-disk */
                _ if sw_val == 2 || sw_val == 3 => {
                    /* Copy FreeLoader to the disk and save the boot entries */
                    Status = InstallBootManagerAndBootEntries(
                        (*pSetupData).USetupData.ArchType,
                        core::ptr::addr_of_mut!((*pSetupData).USetupData.SystemRootPath),
                        core::ptr::addr_of_mut!((*pSetupData).USetupData.SourceRootPath),
                        core::ptr::addr_of_mut!((*pSetupData).USetupData.DestinationArcPath),
                        if (*pSetupData).USetupData.BootLoaderLocation == 2 {
                            1 /* Install MBR and VBR */
                        } else {
                            0 /* Install VBR only */
                        },
                    );
                    if Status == STATUS_SUCCESS {
                        break 'sw; /* Successful installation */
                    }

                    if (Status) as u32 == ERROR_WRITE_BOOT {
                        /* Error when writing the VBR */
                        DisplayError(
                            hWndParent,
                            0, // Default to "Error"
                            IDS_ERROR_WRITE_BOOT,
                            (*(*SystemPartition).Volume).Info.FileSystem,
                        );
                    } else if (Status) as u32 == ERROR_INSTALL_BOOTCODE {
                        /* Error when writing the MBR */
                        DisplayError(
                            hWndParent,
                            0, // Default to "Error"
                            IDS_ERROR_INSTALL_BOOTCODE,
                            RUSTCONV_WSTR_DD8FED8124A3.as_ptr(),
                        );
                    } else if Status == STATUS_NOT_SUPPORTED {
                        DisplayError(
                            hWndParent,
                            0, // Default to "Error"
                            IDS_ERROR_BOOTLDR_ARCH_UNSUPPORTED,
                        );
                    } else if !NT_SUCCESS!(Status) {
                        /* Any other NTSTATUS failure code */
                        DPRINT1!("InstallBootManagerAndBootEntries() failed: Status 0x%lx\n", Status);
                        DisplayError(
                            hWndParent,
                            0, // Default to "Error"
                            IDS_ERROR_BOOTLDR_FAILED,
                            Status,
                        );
                    }
                    Success = (FALSE as u8);
                    break 'quit;
                }

                /* Skip installation */
                _ if sw_val == 0 => {}
                _ => {}
            }
        }
    }

    /* Signal the wizard page that we have succeeded or failed */
    PostMessage(hwndDlg, PM_INSTALL_DONE, (Success) as _, 0); // Status
    return Success as DWORD;
}

unsafe extern "system" fn ProcessDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    'sw: {
        match uMsg {
            WM_INITDIALOG => {
                /* Save pointer to the global setup data */
                pSetupData = (*(lParam as LPPROPSHEETPAGEW)).lParam as PSETUPDATA;
                SetWindowLongPtrW(hwndDlg, DWLP_USER, (pSetupData as DWORD_PTR) as LONG_PTR);

                /* Reset the status text and set the main label in bold */
                SetDlgItemTextW(hwndDlg, (IDC_ACTIVITY as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
                SetDlgItemTextW(hwndDlg, (IDC_ITEM as i32), RUSTCONV_WSTR_96A296D224F2.as_ptr());
                SetDlgItemFont!(hwndDlg, (IDC_ACTIVITY as i32), (*pSetupData).hBoldFont, TRUE);
                break 'sw;
            }

            WM_SETCURSOR => {
                /* Set a page-wide cursor */
                if hWaitCursor.is_null() {
                    break 'sw;
                }
                SetCursor(hWaitCursor);
                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                return (TRUE) as INT_PTR;
            }

            WM_NOTIFY => {
                let mut lpnm: LPNMHDR = lParam as LPNMHDR;

                'nm: {
                    match ((*lpnm).code) as i32 {
                        PSN_SETACTIVE => {
                            let mut hWndParent: HWND = GetParent(hwndDlg);

                            /*
                             * Disable all buttons during installation, and hide "Back" and "Next".
                             */
                            SendMessageW(hWndParent, PSM_SETWIZBUTTONS, 0, 0);
                            // PropSheet_ShowWizButtons(hWndParent, 0, PSWIZB_BACK | PSWIZB_NEXT);
                            ShowDlgItem!(hWndParent, (ID_WIZBACK as i32), (SW_HIDE as i32));
                            ShowDlgItem!(hWndParent, (ID_WIZNEXT as i32), (SW_HIDE as i32));

                            /* Start the installation procedure once the page is fully displayed */
                            PostMessageW(hwndDlg, PM_INSTALL_START, 0, 0);
                            break 'nm;
                        }

                        PSN_KILLACTIVE => {
                            /* Avoid reentrant calls caused by the message dispatching
                             * done below: allow only the first page change, and ignore
                             * all the others. */
                            if InterlockedFlagsTestAndSet8!(
                                core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall),
                                SETUP_PAGE_SWITCHING,
                            ) != false
                            {
                                SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                                return (TRUE) as INT_PTR;
                            }

                            /* If no installation is running, just change the page */
                            if (*pSetupData).hInstallThread.is_null() {
                                break 'nm;
                            }

                            /* Wait for the installation thread to terminate. */
                            let mut dwStatus: DWORD = WaitForSingleObject((*pSetupData).hInstallThread, 0);
                            if dwStatus == WAIT_TIMEOUT {
                                /* Show the hourglass cursor */
                                let mut hOldCursor: HCURSOR = core::ptr::null_mut();
                                hWaitCursor = LoadCursor(core::ptr::null_mut(), IDC_WAIT);
                                hOldCursor = SetCursor(hWaitCursor);
                                ShowCursor((TRUE) as BOOL);

                                loop {
                                    let mut msg: MSG = core::mem::zeroed();
                                    dwStatus = MsgWaitForMultipleObjects(
                                        1,
                                        core::ptr::addr_of_mut!((*pSetupData).hInstallThread),
                                        (FALSE) as BOOL,
                                        INFINITE,
                                        (QS_ALLINPUT | QS_ALLPOSTMESSAGE),
                                    );
                                    if dwStatus != (WAIT_OBJECT_0 + 1) {
                                        break; // Stop if anything else (thread terminated, timeout or failure).
                                    }

                                    /* We still need to process main window messages to avoid freeze */
                                    while PeekMessageW(
                                        core::ptr::addr_of_mut!(msg),
                                        core::ptr::null_mut(),
                                        0,
                                        0,
                                        PM_REMOVE,
                                    ) != 0
                                    {
                                        TranslateMessage(core::ptr::addr_of_mut!(msg));
                                        DispatchMessageW(core::ptr::addr_of_mut!(msg));
                                    }
                                }

                                /* Restore the old cursor */
                                ShowCursor((FALSE) as BOOL);
                                SetCursor(hOldCursor);
                                hWaitCursor = core::ptr::null_mut();
                            }
                            if false {
                                if dwStatus == WAIT_OBJECT_0 {
                                    let mut dwExitCode: DWORD = NO_ERROR;
                                    GetExitCodeThread((*pSetupData).hInstallThread, core::ptr::addr_of_mut!(dwExitCode));
                                    DBG_UNREFERENCED_PARAMETER!(dwExitCode);
                                }
                            }

                            /* Cleanup; we can now change the page */
                            CloseHandle((*pSetupData).hInstallThread);
                            (*pSetupData).hInstallThread = core::ptr::null_mut();
                            CloseHandle((*pSetupData).hHaltInstallEvent);
                            (*pSetupData).hHaltInstallEvent = core::ptr::null_mut();
                            break 'nm;
                        }

                        PSN_QUERYCANCEL => {
                            let mut nRet: INT;

                            if (*pSetupData).__bindgen_anon_1.bStopInstall != 0 {
                                /* The installation is terminating; ignore all new requests. */
                                nRet = (IDYES as i32);
                            } else {
                                /* Halt the on-going file copy */
                                ResetEvent((*pSetupData).hHaltInstallEvent);

                                nRet = DisplayMessage(
                                    GetParent(hwndDlg),
                                    (MB_ICONQUESTION | MB_YESNO | MB_DEFBUTTON2),
                                    MAKEINTRESOURCEW!(IDS_ABORTSETUP2),
                                    MAKEINTRESOURCEW!(IDS_ABORTSETUP),
                                );
                                if nRet == (IDYES as i32) {
                                    /* Signal the file copy thread to stop */
                                    InterlockedOr8(
                                        core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall) as PCHAR,
                                        SETUP_ABORT_INSTALL as c_char,
                                    );
                                }
                                /* else, we don't stop installation, resume file copy */
                                SetEvent((*pSetupData).hHaltInstallEvent);
                            }

                            /* If we are not already in the process of cancelling
                             * the installation or switching to a page, do it now. */
                            if (nRet == (IDYES as i32))
                                && (*pSetupData).__bindgen_anon_1.bPageSwitching() == 0
                                && InterlockedFlagsTestAndSet8!(
                                    core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall),
                                    SETUP_IS_CANCELLING as c_char,
                                ) == false
                            {
                                /* Go to the Abort page */
                                PostMessageW(hwndDlg, PM_INSTALL_DONE, (FALSE) as WPARAM, 0);
                            }

                            /* Do not close the wizard too soon */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, (TRUE) as LONG_PTR);
                            return (TRUE) as INT_PTR;
                        }

                        PSN_WIZBACK => {
                            /* Always disable going back */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, -(1));
                            return (TRUE) as INT_PTR;
                        }

                        PSN_WIZNEXT => {
                            /* Always disable going next */
                            SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, -(1));
                            return (TRUE) as INT_PTR;
                        }

                        _ => {}
                    }
                }
                break 'sw;
            }

            PM_INSTALL_START => {
                let mut hWndParent: HWND = GetParent(hwndDlg);

                ASSERT!((*pSetupData).hInstallThread == core::ptr::null_mut());

                /* Force repainting first to make sure the wizard is visible */
                InvalidateRect(hWndParent, core::ptr::null_mut(), (FALSE) as BOOL);
                UpdateWindow(hWndParent);

                /* Create the file-copy halt (manual-reset) event */
                (*pSetupData).hHaltInstallEvent = CreateEventW(
                    core::ptr::null_mut(),
                    (TRUE) as BOOL,
                    (TRUE) as BOOL,
                    core::ptr::null_mut(),
                );

                let mut do_fail: bool = false;
                if (*pSetupData).hHaltInstallEvent.is_null() {
                    DisplayMessage(
                        hWndParent,
                        MB_ICONERROR,
                        core::ptr::null_mut(),
                        RUSTCONV_WSTR_DB7B28E33CE1.as_ptr(),
                        GetLastError(),
                    );
                    // goto Fail;
                    do_fail = true;
                } else {
                    /* Start the installation thread */
                    (*pSetupData).__bindgen_anon_1.bStopInstall = (FALSE as u8);
                    (*pSetupData).hInstallThread = CreateThread(
                        core::ptr::null_mut(),
                        0,
                        Some(PrepareAndDoCopyThread),
                        hwndDlg as PVOID,
                        0,
                        core::ptr::null_mut(),
                    );
                    if (*pSetupData).hInstallThread.is_null() {
                        DisplayMessage(
                            hWndParent,
                            MB_ICONERROR,
                            core::ptr::null_mut(),
                            RUSTCONV_WSTR_D52B95BF9F39.as_ptr(),
                            GetLastError(),
                        );
                        CloseHandle((*pSetupData).hHaltInstallEvent);
                        (*pSetupData).hHaltInstallEvent = core::ptr::null_mut();
                        do_fail = true;
                    }
                }

                if do_fail {
                    /* Fail: Cancel the installation */
                    InterlockedOr8(
                        core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall) as PCHAR,
                        SETUP_ABORT_INSTALL as c_char,
                    );
                    SendMessageW(hwndDlg, PM_INSTALL_DONE, (FALSE) as WPARAM, 0);
                }
                break 'sw;
            }

            PM_INSTALL_DONE => {
                let mut hWndParent: HWND = GetParent(hwndDlg);
                let mut Success: BOOL = (wParam != 0) as BOOL;
                let mut AutoSwitchPage: BOOL = (TRUE as i32);

                /* If an unexpected error happened, stay on the copy page to allow
                 * the user to view the current state. */
                if Success == 0 && (*pSetupData).__bindgen_anon_1.bStopInstall == 0 {
                    PropSheet_SetCloseCancel(hWndParent, (TRUE) as BOOL);
                    AutoSwitchPage = (FALSE as i32);
                } else {
                    PropSheet_SetCloseCancel(hWndParent, (FALSE) as BOOL);
                }

                if Success == 0 {
                    /* The installation was aborted */
                    InterlockedOr8(
                        core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall) as PCHAR,
                        SETUP_ABORT_INSTALL as c_char,
                    );
                } else if (*pSetupData).__bindgen_anon_1.bAbortInstall() != 0 {
                    /* Override success in case the thread just terminated with some status */
                    Success = (FALSE as i32);
                }

                /* We are done! Switch to the Finish or the Abort page */
                if AutoSwitchPage == 0 {
                    break 'sw;
                }
                // TODO: if (!Success): Unwind installation.
                if Success == 0 {
                    InterlockedOr8(
                        core::ptr::addr_of_mut!((*pSetupData).__bindgen_anon_1.bStopInstall) as PCHAR,
                        SETUP_IS_CANCELLING as c_char,
                    );
                }
                PropSheet_SetCurSelByID!(
                    hWndParent,
                    (if Success != 0 { IDD_FINISHPAGE } else { IDD_ABORTPAGE }),
                );
                break 'sw;
            }

            _ => {}
        }
    }

    return (FALSE) as INT_PTR;
}

unsafe extern "C" fn IsShellActive() -> BOOL {
    let mut hWndProgman: HWND = GetProgmanWindow();
    if !(!hWndProgman.is_null()
        && IsWindow(hWndProgman) != 0
        && IsWindowEnabled(hWndProgman) != 0
        && IsWindowVisible(hWndProgman) != 0)
    {
        hWndProgman = GetShellWindow();
    }
    return (!hWndProgman.is_null()
        && IsWindow(hWndProgman) != 0
        && IsWindowEnabled(hWndProgman) != 0
        && IsWindowVisible(hWndProgman) != 0) as BOOL;
}

unsafe extern "system" fn FindUserClosableWindowProc(hWnd: HWND, lParam: LPARAM) -> BOOL {
    let pInfo: PCLOSABLE_WND_INFO = lParam as PCLOSABLE_WND_INFO;
    let mut hSysMenu: HMENU = core::ptr::null_mut();
    let mut mii: MENUITEMINFOW = core::mem::zeroed();

    /* Skip the window to exclude */
    if hWnd == (*pInfo).hWndExclude {
        return (TRUE) as BOOL;
    }

    /* Skip non-interactive windows */
    if IsWindowEnabled(hWnd) == 0 || IsWindowVisible(hWnd) == 0 {
        return (TRUE) as BOOL;
    }

    hSysMenu = GetSystemMenu(hWnd, (FALSE) as BOOL);
    if hSysMenu.is_null() {
        return (TRUE) as BOOL; /* No menu: skip the window */
    }

    mii.cbSize = core::mem::size_of::<MENUITEMINFOW>() as UINT;
    mii.fMask = MIIM_STATE;
    if GetMenuItemInfoW(hSysMenu, SC_CLOSE, (FALSE) as BOOL, core::ptr::addr_of_mut!(mii)) == 0 {
        return (TRUE) as BOOL; /* No close item: skip the window */
    }

    (*pInfo).Found |= ((mii.fState & MFS_DISABLED) == 0) as BOOL;

    /* Continue enumeration (return TRUE) if no close item is enabled;
     * otherwise stop the enumeration (return FALSE) */
    return ((*pInfo).Found == 0) as BOOL;
}

unsafe extern "C" fn AreThereInteractiveWindows(hWndExclude: HWND) -> BOOL {
    if IsShellActive() != 0 {
        return (TRUE) as BOOL;
    } else {
        let mut Info: CLOSABLE_WND_INFO = CLOSABLE_WND_INFO {
            hWndExclude: hWndExclude,
            Found: (FALSE as i32),
        };
        EnumWindows(
            Some(FindUserClosableWindowProc),
            core::ptr::addr_of_mut!(Info) as LPARAM,
        );
        return Info.Found;
    }
}

unsafe extern "system" fn FinishDlgProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    let mut pSetupData: PSETUPDATA = core::ptr::null_mut();

    /* Retrieve pointer to the global setup data */
    pSetupData = GetWindowLongPtrW(hwndDlg, DWLP_USER) as PSETUPDATA;

    match uMsg {
        WM_INITDIALOG => {
            let mut hWndParent: HWND = GetParent(hwndDlg);
            let mut ppsp: LPPROPSHEETPAGEW = lParam as LPPROPSHEETPAGEW;

            /* Save pointer to the global setup data */
            pSetupData = (*ppsp).lParam as PSETUPDATA;
            SetWindowLongPtrW(hwndDlg, DWLP_USER, (pSetupData as DWORD_PTR) as LONG_PTR);

            /* Set the stop-install flag if the user is aborting
             * the installation: TRUE if Abort, FALSE if Finish. */
            (*pSetupData).__bindgen_anon_1.bStopInstall = ((((*ppsp).__bindgen_anon_1.pszTemplate
                == (MAKEINTRESOURCEW!(IDD_ABORTPAGE) as PCWSTR)) as BOOL) as BOOL) as u8;

            /* Set title font */
            SetDlgItemFont!(hwndDlg, (IDC_FINISHTITLE as i32), (*pSetupData).hTitleFont, TRUE);

            /* We need to reboot at the end of the installation if this is an
             * unattended setup, or if there is no shell active. If there are
             * other user-interactive windows opened, we pause in WM_ACTIVATE
             * the inevitable reboot countdown when the wizard is deactivated,
             * and restart it when the wizard is reactivated. */
            (*pSetupData).bMustReboot = SetupData.bUnattend;
            (*pSetupData).bMustReboot |= (IsShellActive() == 0) as BOOL;

            /* If we must reboot, display the countdown gauge. In any
             * case, let the user restart now or postpone it to later. */
            if (*pSetupData).bMustReboot != 0 {
                /* "Setup will now restart your computer..." is shown */
                EnableDlgItem!(hwndDlg, (IDC_RESTART_PROGRESS as i32), (TRUE as i32));
                ShowDlgItem!(hwndDlg, (IDC_RESTART_PROGRESS as i32), (SW_SHOW as i32));
            } else {
                /* We should not reboot automatically, change the finish
                 * text to "Setup needs to restart your computer..." */
                let mut uMsgID: UINT = if (*pSetupData).__bindgen_anon_1.bStopInstall == 0 {
                    IDS_FINISH_NO_REBOOT
                } else {
                    IDS_ABORT_NO_REBOOT
                } as UINT;
                SetWindowResTextW(
                    GetDlgItem(hwndDlg, (IDC_FINISHTEXT as i32)),
                    (*pSetupData).hInstance,
                    uMsgID,
                );

                /* Hide and disable the countdown gauge */
                EnableDlgItem!(hwndDlg, (IDC_RESTART_PROGRESS as i32), (FALSE as i32));
                ShowDlgItem!(hwndDlg, (IDC_RESTART_PROGRESS as i32), (SW_HIDE as i32));
            }

            /* Ensure that the wizard window is centered, made visible, and focused */
            CenterWindow(hWndParent);
            ShowWindow(hWndParent, (SW_SHOW as i32));
            SwitchToThisWindow(hWndParent, (TRUE) as BOOL);
            return (TRUE) as INT_PTR;
        }

        WM_ACTIVATE => {
            /* Only care about (de)activation only if we must reboot */
            if (*pSetupData).bMustReboot != 0 {
                if LOWORD!(wParam) == (WA_INACTIVE as u16) {
                    /* Wizard window is deactivated, check whether there are
                     * interactive windows. If so, pause the countdown. */
                    if AreThereInteractiveWindows(GetParent(hwndDlg)) != 0 {
                        KillTimer(hwndDlg, 1);
                    }
                } else {
                    /* Wizard window is reactivated, re-enable the countdown */
                    SetTimer(hwndDlg, 1, 50, None);
                }
            }
        }

        WM_TIMER => {
            let mut hWndProgress: HWND = core::ptr::null_mut();
            let mut Position: INT = 0;

            hWndProgress = GetDlgItem(hwndDlg, (IDC_RESTART_PROGRESS as i32));
            Position = SendMessageW(hWndProgress, PBM_GETPOS, 0, 0) as INT;
            if Position == 300 {
                KillTimer(hwndDlg, 1);
                PropSheet_PressButton!(GetParent(hwndDlg), PSBTN_FINISH);
            } else {
                SendMessageW(
                    hWndProgress,
                    PBM_SETPOS,
                    (Position + 1) as WPARAM,
                    0,
                );
            }
            return (TRUE) as INT_PTR;
        }

        WM_NOTIFY => {
            let mut lpnm: LPNMHDR = lParam as LPNMHDR;

            match ((*lpnm).code) as i32 {
                PSN_SETACTIVE => {
                    let mut hWndParent: HWND = GetParent(hwndDlg);

                    /*
                     * Only "Finish" for closing the wizard, and hide "Back" and "Next".
                     * Don't use the PropSheet_SetWizButtons() macro, because its
                     * posted message would be handled after hiding the buttons.
                     * The message would then interfere with the hidden buttons
                     * (when both "Back" and "Next" are hidden, "Next" gets forcefully shown).
                     */
                    SendMessageW(hWndParent, PSM_SETWIZBUTTONS, 0, (PSWIZB_FINISH) as LPARAM);
                    // PropSheet_ShowWizButtons(hWndParent, 0, PSWIZB_BACK | PSWIZB_NEXT | PSWIZB_CANCEL);
                    ShowDlgItem!(hWndParent, (ID_WIZBACK as i32), (SW_HIDE as i32));
                    ShowDlgItem!(hWndParent, (ID_WIZNEXT as i32), (SW_HIDE as i32));

                    /* Change the "Finish" button text to "Restart" */
                    SetWindowResTextW(
                        GetDlgItem(hWndParent, (ID_WIZFINISH as i32)),
                        (*pSetupData).hInstance,
                        IDS_RESTARTBTN,
                    );

                    /* Skip the Finish page in unattended setup */
                    if (*pSetupData).bUnattend != 0 /*&& !pSetupData->bStopInstall*/ {
                        // FIXME: The macro should use PostMessageW, but our
                        // prsht.h is wrong and uses SendMessageW instead...
                        //PropSheet_PressButton(hWndParent, PSBTN_FINISH);
                        PostMessageW(hWndParent, PSM_PRESSBUTTON, (PSBTN_FINISH) as WPARAM, 0);
                        /* We need to "stay" on the page so that we can
                         * receive the PSN_WIZFINISH notification */
                        SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, 0);
                        return (TRUE) as INT_PTR;
                    }

                    /* Re-enable the Close/Cancel buttons if we won't reboot */
                    if (*pSetupData).bMustReboot == 0 {
                        PropSheet_SetCloseCancel(hWndParent, (TRUE) as BOOL);
                    }

                    if (*pSetupData).bMustReboot != 0 {
                        let mut rcBtn1: RECT = core::mem::zeroed();
                        let mut rcBtn2: RECT = core::mem::zeroed();

                        /* Move the "Finish"/"Restart" button to where the "Cancel" button is */
                        GetWindowRect(
                            GetDlgItem(hWndParent, (ID_WIZFINISH as i32)),
                            core::ptr::addr_of_mut!(rcBtn1),
                        );
                        MapWindowPoints(
                            HWND_DESKTOP /*NULL*/,
                            hWndParent,
                            core::ptr::addr_of_mut!(rcBtn1) as LPPOINT,
                            (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>()) as UINT,
                        );
                        GetWindowRect(
                            GetDlgItem(hWndParent, (IDCANCEL as i32)),
                            core::ptr::addr_of_mut!(rcBtn2),
                        );
                        MapWindowPoints(
                            HWND_DESKTOP /*NULL*/,
                            hWndParent,
                            core::ptr::addr_of_mut!(rcBtn2) as LPPOINT,
                            (core::mem::size_of::<RECT>() / core::mem::size_of::<POINT>()) as UINT,
                        );
                        SetWindowPos(
                            GetDlgItem(hWndParent, (ID_WIZFINISH as i32)),
                            HWND_TOP,
                            rcBtn1.left + (rcBtn2.right - rcBtn1.right),
                            rcBtn1.top,
                            0,
                            0,
                            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
                        );

                        /* Hide and disable also the "Cancel" buttons since we can only finish now */
                        ShowDlgItem!(hWndParent, (IDCANCEL as i32), (SW_HIDE as i32));
                        PropSheet_SetCloseCancel(hWndParent, (FALSE) as BOOL);

                        /* Set up the reboot progress bar and countdown timer.
                         * 300 steps at 50 ms each: 15 seconds */
                        SendDlgItemMessage(
                            hwndDlg,
                            (IDC_RESTART_PROGRESS as i32),
                            PBM_SETRANGE,
                            0,
                            MAKELPARAM!(0, 300),
                        );
                        SendDlgItemMessage(hwndDlg, (IDC_RESTART_PROGRESS as i32), PBM_SETPOS, 0, 0);
                        SetTimer(hwndDlg, 1, 50, None);
                    } else if (*pSetupData).__bindgen_anon_1.bStopInstall == 0 {
                        /* Keep the "Cancel" button shown and change its text to "Postpone" */
                        SetWindowResTextW(
                            GetDlgItem(hWndParent, (IDCANCEL as i32)),
                            (*pSetupData).hInstance,
                            IDS_POSTPONEBTN,
                        );
                    } else {
                        /* (!bMustReboot && bStopInstall)
                         * The installation is aborted, change the "Cancel" button text to "Close" */
                        SetWindowResTextW(
                            GetDlgItem(hWndParent, (IDCANCEL as i32)),
                            GetModuleHandleW(RUSTCONV_WSTR_88A26420BF90.as_ptr()),
                            IDS_CLOSE,
                        );
                    }
                }

                PSN_KILLACTIVE => {
                    KillTimer(hwndDlg, 1);
                }

                PSN_WIZBACK => {
                    /* Always disable going back */
                    SetWindowLongPtrW(hwndDlg, DWLP_MSGRESULT, -1);
                    return (TRUE) as INT_PTR;
                }

                PSN_WIZNEXT | PSN_WIZFINISH => {
                    /* Press on "Finish"/"Restart" button */
                    (*pSetupData).bMustReboot = (TRUE as i32);
                    /* fallthrough */
                    /* Press on "Cancel"/"Postpone" button */
                }

                PSN_QUERYCANCEL | _ => {}
            }
        }

        _ => {}
    }

    return (FALSE) as INT_PTR;
}
#[no_mangle]
pub unsafe extern "C" fn LoadSetupData(pSetupData: PSETUPDATA) -> BOOL {
    (*pSetupData).PartitionList = CreatePartitionList();
    if (*pSetupData).PartitionList.is_null() {
        DPRINT1!(b"Could not enumerate available disks; failing installation\n\0".as_ptr() as PCSTR);
        return (FALSE) as BOOL;
    }

    (*pSetupData).NtOsInstallsList = CreateNTOSInstallationsList((*pSetupData).PartitionList);
    if (*pSetupData).NtOsInstallsList.is_null() {
        DPRINT1!(b"Failed to get a list of NTOS installations; continue installation...\n\0".as_ptr() as PCSTR);
    }

    /* Load the hardware, language and keyboard layout lists */

    (*pSetupData).USetupData.ComputerList = CreateComputerTypeList((*pSetupData).USetupData.SetupInf);
    (*pSetupData).USetupData.DisplayList = CreateDisplayDriverList((*pSetupData).USetupData.SetupInf);
    (*pSetupData).USetupData.KeyboardList = CreateKeyboardDriverList((*pSetupData).USetupData.SetupInf);

    (*pSetupData).USetupData.LanguageList = CreateLanguageList(
        (*pSetupData).USetupData.SetupInf,
        &raw mut (*pSetupData).DefaultLanguage as PWSTR,
    );

    /* If not unattended, overwrite language and locale with
     * the current ones of the running ReactOS instance */
    if (*pSetupData).bUnattend == 0 {
        let mut LocaleID: LCID = GetUserDefaultLCID();

        StringCchPrintfW!(
            &raw mut (*pSetupData).DefaultLanguage as PWSTR,
            _countof!((*pSetupData).DefaultLanguage),
            RUSTCONV_WSTR_181895D51D74.as_ptr(),
            LocaleID,
        );

        StringCchPrintfW!(
            &raw mut (*pSetupData).USetupData.LocaleID as PWSTR,
            _countof!((*pSetupData).USetupData.LocaleID),
            RUSTCONV_WSTR_181895D51D74.as_ptr(),
            LocaleID,
        );
    }

    /* new part */
    (*pSetupData).SelectedLanguageId = &raw mut (*pSetupData).DefaultLanguage as *mut WCHAR as PCWSTR;
    wcscpy(
        &raw mut (*pSetupData).DefaultLanguage as *mut WCHAR,
        &raw mut (*pSetupData).USetupData.LocaleID as *mut WCHAR,
    ); // FIXME: In principle, only when unattended.
    (*pSetupData).USetupData.LanguageId = (wcstol(
        (*pSetupData).SelectedLanguageId as *const WCHAR,
        ::core::ptr::null_mut::<*mut WCHAR>(),
        16,
    ) & 0xFFFF) as LANGID;

    (*pSetupData).USetupData.LayoutList = CreateKeyboardLayoutList(
        (*pSetupData).USetupData.SetupInf,
        (*pSetupData).SelectedLanguageId,
        &raw mut (*pSetupData).DefaultKBLayout as PWSTR,
    );

    /* If not unattended, overwrite keyboard layout with
     * the current one of the running ReactOS instance */
    if (*pSetupData).bUnattend == 0 {
        // C_ASSERT(_countof(pSetupData->DefaultKBLayout) >= KL_NAMELENGTH): the element
        // count of the member's array TYPE, measured at compile time as the C does.
        C_ASSERT!({
            const fn __countof<T, const N: usize>(_: *const [T; N]) -> usize { N }
            let __u = core::mem::MaybeUninit::<_SETUPDATA>::uninit();
            __countof(unsafe { core::ptr::addr_of!((*__u.as_ptr()).DefaultKBLayout) })
        } >= KL_NAMELENGTH as usize);
        /* If the call fails, keep the default already stored in the buffer */
        GetKeyboardLayoutNameW(&raw mut (*pSetupData).DefaultKBLayout as LPWSTR);
    }

    /* Change the default entries in the language and keyboard layout lists */
    {
        let mut LanguageList: PGENERIC_LIST = (*pSetupData).USetupData.LanguageList;
        let mut LayoutList: PGENERIC_LIST = (*pSetupData).USetupData.LayoutList;
        let mut ListEntry: PGENERIC_LIST_ENTRY;

        /* Search for default language */
        ListEntry = GetFirstListEntry(LanguageList);
        while !ListEntry.is_null() {
            let mut LocaleId: PCWSTR = (*(GetListEntryData(ListEntry) as PGENENTRY)).Id;
            if _wcsicmp(
                &raw mut (*pSetupData).DefaultLanguage as *mut WCHAR,
                LocaleId as *const WCHAR,
            ) == 0
            {
                DPRINT!(b"found %S in LanguageList\n\0".as_ptr() as PCSTR, LocaleId);
                SetCurrentListEntry(LanguageList, ListEntry);
                break;
            }
            ListEntry = GetNextListEntry(ListEntry);
        }

        /* Search for default layout */
        ListEntry = GetFirstListEntry(LayoutList);
        while !ListEntry.is_null() {
            let mut pszLayoutId: PCWSTR = (*(GetListEntryData(ListEntry) as PGENENTRY)).Id;
            // FIXME: Temporary "fix" to set the best keyboard entry depending on the
            // selected language in unattended setup; see also usetup!SetupStartPage().
            if ((*pSetupData).bUnattend != 0
                && _wcsicmp(
                    &raw mut (*pSetupData).DefaultLanguage as *mut WCHAR,
                    pszLayoutId as *const WCHAR,
                ) == 0)
                || ((*pSetupData).bUnattend == 0
                    && _wcsicmp(
                        &raw mut (*pSetupData).DefaultKBLayout as *mut WCHAR,
                        pszLayoutId as *const WCHAR,
                    ) == 0)
            {
                DPRINT!(b"Found %S in LayoutList\n\0".as_ptr() as PCSTR, pszLayoutId);
                SetCurrentListEntry(LayoutList, ListEntry);
                break;
            }
            ListEntry = GetNextListEntry(ListEntry);
        }
    }

    return (TRUE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn InitNtToWin32PathMappingList(
    MappingList: PNT_WIN32_PATH_MAPPING_LIST,
) {
    InitializeListHead(core::ptr::addr_of_mut!((*MappingList).List));
    (*MappingList).MappingsCount = 0;
}
#[no_mangle]
pub unsafe extern "C" fn FreeNtToWin32PathMappingList(
    mut MappingList: PNT_WIN32_PATH_MAPPING_LIST,
) {
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PVOID = core::ptr::null_mut();
    while IsListEmpty(core::ptr::addr_of_mut!((*MappingList).List)) == false {
        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!((*MappingList).List));
        Entry = CONTAINING_RECORD!(ListEntry, NT_WIN32_PATH_MAPPING, ListEntry) as PVOID;
        HeapFree(ProcessHeap, 0, Entry);
    }

    (*MappingList).MappingsCount = 0;
}
#[no_mangle]
pub unsafe extern "C" fn ConvertNtPathToWin32Path(MappingList: PNT_WIN32_PATH_MAPPING_LIST, pwszPath: PWSTR, cchPathMax: DWORD, pwszNTPath: PCWSTR) -> BOOL {
    let mut FoundDrive: BOOL = (FALSE as i32);
    let mut RetryOnce: BOOL = (FALSE as i32);
    let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
    let mut Entry: PNT_WIN32_PATH_MAPPING = core::ptr::null_mut();
    let mut pwszNtPathToMap: PCWSTR = pwszNTPath;
    let mut pwszRemaining: PCWSTR = core::ptr::null();
    let mut cchDrives: DWORD = 0;
    let mut pwszDrive: PWCHAR = core::ptr::null_mut();
    let mut wszDrives: [WCHAR; 512] = [0; 512];
    let mut wszNTPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut TargetPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut at: u8 = 0;

    *pwszPath = UNICODE_NULL;

    'dispatch: loop {
        match at {
            0 => {
                ListEntry = (*MappingList).List.Flink as PLIST_ENTRY;
                while ListEntry != core::ptr::addr_of_mut!((*MappingList).List) {
                    Entry = CONTAINING_RECORD!(ListEntry, NT_WIN32_PATH_MAPPING, ListEntry) as PNT_WIN32_PATH_MAPPING;

                    DPRINT!("Testing '%S' --> '%S'\n", core::ptr::addr_of!((*Entry).Win32Path), core::ptr::addr_of!((*Entry).NtPath));

                    FoundDrive = (_wcsnicmp(
                        pwszNtPathToMap,
                        core::ptr::addr_of_mut!((*Entry).NtPath) as *mut WCHAR,
                        wcslen(core::ptr::addr_of_mut!((*Entry).NtPath) as *mut WCHAR) as usize,
                    ) == 0) as BOOL;
                    if FoundDrive != 0 {
                        pwszDrive = core::ptr::addr_of_mut!((*Entry).Win32Path) as *mut WCHAR as PWCHAR;
                        pwszRemaining = pwszNTPath.offset(
                            wcslen(core::ptr::addr_of_mut!((*Entry).NtPath) as *mut WCHAR) as isize,
                        );
                        break 'dispatch;
                    }
                    ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
                }

                cchDrives = GetLogicalDriveStringsW(
                    (_countof!(wszDrives) - 1) as DWORD,
                    core::ptr::addr_of_mut!(wszDrives) as *mut WCHAR as LPWSTR,
                );
                if cchDrives == 0 as DWORD || cchDrives as usize >= _countof!(wszDrives) as usize {
                    DPRINT1!("ConvertNtPathToWin32Path: GetLogicalDriveStringsW failed\n");
                    return (FALSE) as BOOL;
                }

                at = 1;
                continue 'dispatch;
            }
            1 => {
                /* Retry: */
                pwszDrive = core::ptr::addr_of_mut!(wszDrives) as *mut WCHAR as PWCHAR;
                while *pwszDrive != 0 {
                    *pwszDrive.offset(2 as isize) = UNICODE_NULL;
                    QueryDosDeviceW(
                        pwszDrive as LPCWSTR,
                        core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR as LPWSTR,
                        _countof!(wszNTPath) as DWORD,
                    );
                    *pwszDrive.offset(2 as isize) = 0x005Cu16;

                    DPRINT!("Testing '%S' --> '%S'\n", pwszDrive, core::ptr::addr_of!(wszNTPath));

                    FoundDrive = (_wcsnicmp(
                        pwszNtPathToMap,
                        core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR,
                        wcslen(core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR) as usize,
                    ) == 0) as BOOL;
                    if FoundDrive == 0 {
                        let mut ptr: PWCHAR = core::ptr::null_mut();
                        let mut ptr2: PWCHAR = core::ptr::null_mut();

                        if _wcsnicmp(
                            core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR,
                            RUSTCONV_WSTR_958F461AF768.as_ptr(),
                            (_countof!(RUSTCONV_WSTR_958F461AF768) as usize - 1),
                        ) == 0
                            && {
                                ptr = wcschr(
                                    (core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR)
                                        .offset((_countof!(RUSTCONV_WSTR_958F461AF768) as usize - 1) as isize),
                                    0x005Cu16,
                                ) as PWCHAR;
                                !ptr.is_null()
                            }
                            && {
                                ptr = ptr.offset(1);
                                (wcslen(ptr as PCWSTR) as usize) >= (3 as usize)
                            }
                            && *ptr.offset(0 as isize) == 0x003Bu16
                            && *ptr.offset(2 as isize) == 0x003Au16
                        {
                            if *pwszDrive.offset(0 as isize)
                                != *ptr.offset(1 as isize)
                            {
                                DPRINT1!(
                                    "Peculiar: expected network share drive letter %C different from actual one %C\n",
                                    *pwszDrive.offset(0 as isize) as core::ffi::c_uint,
                                    *ptr.offset(1 as isize) as core::ffi::c_uint
                                );
                            }

                            ptr2 = ptr.offset(3 as isize);
                            while *ptr2 == 0x005Cu16 {
                                ptr2 = ptr2.offset(1);
                            }

                            memmove(
                                ptr as *mut core::ffi::c_void,
                                ptr2 as *const core::ffi::c_void,
                                (wcslen(ptr2 as PCWSTR) as usize + 1 as usize)
                                    * core::mem::size_of::<WCHAR>(),
                            );

                            FoundDrive = (_wcsnicmp(
                                pwszNtPathToMap,
                                core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR,
                                wcslen(core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR) as usize,
                            ) == 0) as BOOL;
                        }
                    }
                    if FoundDrive != 0 {
                        *pwszDrive.offset(2 as isize) = UNICODE_NULL;

                        if pwszNtPathToMap == pwszNTPath {
                            ASSERT!(
                                RetryOnce == 0
                                    && pwszNTPath != (core::ptr::addr_of_mut!(TargetPath) as *mut WCHAR as PCWSTR)
                            );
                            pwszRemaining = pwszNTPath.offset(
                                wcslen(core::ptr::addr_of_mut!(wszNTPath) as *mut WCHAR) as isize,
                            );
                        }
                        break;
                    }
                    pwszDrive = pwszDrive.offset(
                        (wcslen(pwszDrive as PCWSTR) as usize + 1 as usize) as isize,
                    );
                }

                if FoundDrive != 0 {
                    Entry = HeapAlloc(
                        ProcessHeap,
                        HEAP_ZERO_MEMORY as DWORD,
                        core::mem::size_of::<NT_WIN32_PATH_MAPPING>() as SIZE_T,
                    ) as PNT_WIN32_PATH_MAPPING;
                    if Entry.is_null() {
                        DPRINT1!("ConvertNtPathToWin32Path: Cannot allocate memory\n");
                        return (FALSE) as BOOL;
                    }
                    StringCchCopyNW(
                        core::ptr::addr_of_mut!((*Entry).NtPath) as STRSAFE_LPWSTR,
                        _countof!((*Entry).NtPath) as size_t,
                        pwszNTPath as STRSAFE_LPCWSTR,
                        pwszRemaining.offset_from(pwszNTPath) as usize,
                    );
                    StringCchCopyW(
                        core::ptr::addr_of_mut!((*Entry).Win32Path) as STRSAFE_LPWSTR,
                        _countof!((*Entry).Win32Path) as size_t,
                        pwszDrive as STRSAFE_LPCWSTR,
                    );

                    InsertHeadList(
                        core::ptr::addr_of_mut!((*MappingList).List),
                        core::ptr::addr_of_mut!((*Entry).ListEntry),
                    );
                    (*MappingList).MappingsCount = (*MappingList).MappingsCount.wrapping_add(1);

                    pwszDrive = core::ptr::addr_of_mut!((*Entry).Win32Path) as *mut WCHAR as PWCHAR;
                    break 'dispatch;
                }

                if RetryOnce == 0 && FoundDrive == 0 {
                    let mut DiskNumber: ULONG = 0;
                    let mut PartitionNumber: ULONG = 0;
                    let mut Length: INT = 0;

                    let mut Status: NTSTATUS = 0;
                    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
                    let mut LinkHandle: HANDLE = core::ptr::null_mut();
                    let mut SymLink: _UNICODE_STRING = core::mem::zeroed();
                    let mut Target: _UNICODE_STRING = core::mem::zeroed();

                    if swscanf(
                        pwszNTPath,
                        RUSTCONV_WSTR_83AE1AA69859.as_ptr(),
                        core::ptr::addr_of_mut!(DiskNumber),
                        core::ptr::addr_of_mut!(PartitionNumber),
                        core::ptr::addr_of_mut!(Length),
                    ) != 2
                    {
                        /* Definitively not a recognized path, bail out */
                        return (FALSE) as BOOL;
                    }

                    /* Check whether \Device\HarddiskX\PartitionY is a symlink */
                    RtlInitEmptyUnicodeString(
                        core::ptr::addr_of_mut!(SymLink),
                        pwszNTPath as PWCHAR,
                        (Length as usize * core::mem::size_of::<WCHAR>()) as USHORT,
                    );
                    SymLink.Length = SymLink.MaximumLength;

                    /* InitializeObjectAttributes(&ObjectAttributes, &SymLink, OBJ_CASE_INSENSITIVE, NULL, NULL) expanded inline */
                    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                    ObjectAttributes.RootDirectory = core::ptr::null_mut();
                    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SymLink) as PUNICODE_STRING;
                    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE as ULONG;
                    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
                    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

                    Status = NtOpenSymbolicLinkObject(
                        core::ptr::addr_of_mut!(LinkHandle),
                        SYMBOLIC_LINK_QUERY as ACCESS_MASK,
                        core::ptr::addr_of_mut!(ObjectAttributes),
                    );
                    if !NT_SUCCESS!(Status) {
                        /* Not a symlink, or something else happened: bail out */
                        DPRINT1!(
                            "ConvertNtPathToWin32Path: NtOpenSymbolicLinkObject(%wZ) failed, Status 0x%08lx\n",
                            core::ptr::addr_of!(SymLink),
                            Status
                        );
                        return (FALSE) as BOOL;
                    }

                    TargetPath[0] = UNICODE_NULL;
                    RtlInitEmptyUnicodeString(
                        core::ptr::addr_of_mut!(Target),
                        core::ptr::addr_of_mut!(TargetPath) as *mut WCHAR as PWCHAR,
                        core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as USHORT,
                    );

                    /* Resolve the link and close its handle */
                    Status = NtQuerySymbolicLinkObject(LinkHandle, core::ptr::addr_of_mut!(Target), core::ptr::null_mut());
                    NtClose(LinkHandle);

                    if !NT_SUCCESS!(Status) {
                        /* Not a symlink, or something else happened: bail out */
                        DPRINT1!(
                            "ConvertNtPathToWin32Path: NtQuerySymbolicLinkObject(%wZ) failed, Status 0x%08lx\n",
                            core::ptr::addr_of!(SymLink),
                            Status
                        );
                        return (FALSE) as BOOL;
                    }

                    /* Set the pointers */
                    pwszRemaining = pwszNTPath.offset(Length as isize);
                    pwszNtPathToMap = core::ptr::addr_of_mut!(TargetPath) as *mut WCHAR as PCWSTR;

                    /* Retry once */
                    RetryOnce = (TRUE as i32);
                    at = 1;
                    continue 'dispatch;
                }

                ASSERT!(FoundDrive == 0);
                break 'dispatch;
            }
            _ => break 'dispatch,
        }
    }

    /* Quit: */
    if FoundDrive != 0 {
        StringCchPrintfW!(
            pwszPath as STRSAFE_LPWSTR,
            cchPathMax as usize,
            RUSTCONV_WSTR_E2CD87D16D16.as_ptr(),
            pwszDrive,
            pwszRemaining,
        );
        DPRINT!("ConvertNtPathToWin32Path: %S\n", pwszPath);
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn EnablePrivilege(
    lpszPrivilegeName: LPCWSTR,
    bEnablePrivilege: BOOL,
) -> BOOL {
    let mut Success: BOOL;
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut tp: TOKEN_PRIVILEGES = core::mem::zeroed();

    Success = OpenProcessToken(
        GetCurrentProcess(),
        TOKEN_ADJUST_PRIVILEGES as DWORD,
        &raw mut hToken,
    );
    if Success == 0 {
        return Success;
    }

    'quit: {
        Success = LookupPrivilegeValueW(
            core::ptr::null::<WCHAR>(),
            lpszPrivilegeName,
            &raw mut (*(&raw mut tp.Privileges as *mut LUID_AND_ATTRIBUTES)
                .offset(0 as isize))
            .Luid,
        );
        if Success == 0 {
            break 'quit;
        }

        tp.PrivilegeCount = 1 as DWORD;
        (*(&raw mut tp.Privileges as *mut LUID_AND_ATTRIBUTES)
            .offset(0 as isize))
        .Attributes = (if bEnablePrivilege != 0 {
            SE_PRIVILEGE_ENABLED
        } else {
            0
        }) as DWORD;

        Success = AdjustTokenPrivileges(
            hToken,
            FALSE as BOOL,
            &raw mut tp,
            0 as DWORD,
            core::ptr::null_mut::<_TOKEN_PRIVILEGES>(),
            core::ptr::null_mut::<DWORD>(),
        );
    }

    CloseHandle(hToken);
    return Success;
}

unsafe extern "system" fn HotkeyThread(Parameter: LPVOID) -> DWORD {
    let mut hotkey: ATOM = 0;
    let mut msg: MSG = core::mem::zeroed();

    DPRINT!("HotkeyThread start\n");

    hotkey = GlobalAddAtomW(RUSTCONV_WSTR_838DD1C33324.as_ptr());
    if RegisterHotKey(core::ptr::null_mut(), hotkey as core::ffi::c_int, MOD_SHIFT, VK_F10) == 0 {
        DPRINT1!("RegisterHotKey failed with %lu\n", GetLastError());
    }

    while GetMessageW(core::ptr::addr_of_mut!(msg), core::ptr::null_mut(), 0, 0) != 0 {
        if msg.hwnd.is_null() && msg.message == WM_HOTKEY && msg.wParam == hotkey as WPARAM {
            let mut CmdLine: [WCHAR; 8] = [0x0063u16, 0x006D, 0x0064, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000]; // CreateProcess can modify this buffer.
            let mut si: STARTUPINFOW = core::mem::zeroed();
            si.cb = core::mem::size_of::<STARTUPINFOW>() as DWORD;
            let mut pi: PROCESS_INFORMATION = core::mem::zeroed();

            if CreateProcessW(core::ptr::null(),
                               core::ptr::addr_of_mut!(CmdLine) as LPWSTR,
                               core::ptr::null_mut(),
                               core::ptr::null_mut(),
                               (FALSE) as BOOL,
                               CREATE_NEW_CONSOLE,
                               core::ptr::null_mut(),
                               core::ptr::null(),
                               core::ptr::addr_of_mut!(si),
                               core::ptr::addr_of_mut!(pi)) != 0
            {
                CloseHandle(pi.hProcess);
                CloseHandle(pi.hThread);
            }
            else
            {
                DPRINT1!("Failed to launch command prompt: %lu\n", GetLastError());
            }
        }
    }

    UnregisterHotKey(core::ptr::null_mut(), hotkey as core::ffi::c_int);
    GlobalDeleteAtom(hotkey);

    DPRINT!("HotkeyThread terminate\n");
    return 0;
}

unsafe extern "C" fn GetLocalSetupDllPath() -> PCWSTR {
    static mut SetupDllPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    static mut Init: BOOL = (FALSE as i32);
    let mut Success: BOOL = 0;
    let mut PathSize: DWORD = 0;

    /* Don't rebuild the path if we did it already */
    if Init != 0 {
        return core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR as PCWSTR;
    }
    Init = TRUE as BOOL;

    /*
     * Retrieve the full path of the current running Setup instance.
     * From this we build the suitable path to the Setup DLL.
     */
    PathSize = GetModuleFileNameW(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR,
        _countof!(SetupDllPath) as DWORD,
    );
    *(core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR)
        .add(_countof!(SetupDllPath) as usize - 1) = UNICODE_NULL as WCHAR; // Ensure NUL-termination (see WinXP bug)

    Success = ((PathSize != 0 as DWORD)
        && ((PathSize as usize) < _countof!(SetupDllPath) as usize)
        && (GetLastError() != ERROR_INSUFFICIENT_BUFFER as DWORD)) as BOOL;
    if Success != 0 {
        /* Find the last path separator, remove it as well as the file name */
        let mut pch: PWCHAR = wcsrchr(
            core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR,
            0x005Cu16,
        ) as PWCHAR;
        if pch.is_null() {
            pch = core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR as PWCHAR;
        }

        /* The Setup DLL is inside the System32 sub-directory */
        PathSize = (_countof!(SetupDllPath) as usize
            - (pch.offset_from(core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR) as usize))
            as DWORD;
        Success = ((StringCchCopyW(pch, PathSize as usize, RUSTCONV_WSTR_D62BC3623F3E.as_ptr()))
            >= 0 as i32) as BOOL;
    }
    if Success == 0 {
        /* Failure: invalidate the path; the DLL won't be found and delay-loaded */
        *(core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR) = UNICODE_NULL as WCHAR;
    }

    return core::ptr::addr_of_mut!(SetupDllPath) as *mut WCHAR as PCWSTR;
}

unsafe extern "system" fn setupDelayHook(dliNotify: ::core::ffi::c_uint, pdli: PDelayLoadInfo) -> FARPROC {
    static mut dllPath: [CHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    static mut setupDllPath: PCWSTR = core::ptr::null::<WCHAR>();
    match dliNotify {
        1 => {
            if _stricmp(
                (*pdli).szDll as *const ::core::ffi::c_char,
                b"setuplib.dll\0".as_ptr() as *const ::core::ffi::c_char,
            ) == 0
            {
                if setupDllPath.is_null() {
                    setupDllPath = GetLocalSetupDllPath();
                }
                if !setupDllPath.is_null()
                    && *setupDllPath as ::core::ffi::c_int != 0
                    && (StringCchPrintfA!(
                        core::ptr::addr_of_mut!(dllPath) as STRSAFE_LPSTR,
                        _countof!(dllPath),
                        b"%S\\%s\0".as_ptr() as STRSAFE_LPCSTR,
                        setupDllPath,
                        (*pdli).szDll,
                    )) >= 0
                {
                    (*pdli).szDll = core::ptr::addr_of_mut!(dllPath) as *mut CHAR as LPCSTR;
                }
            }
            // break;
        }
        3 => {
            let mut DllPathA: ANSI_STRING = _STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut::<CHAR>(),
            };
            // DECLARE_UNICODE_STRING_SIZE(DllPathU, COMMENT_TOKEN_ECHO____)
            let mut DllPathU_buffer: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut DllPathU: _UNICODE_STRING = _UNICODE_STRING {
                Length: 0 as USHORT,
                MaximumLength: (MAX_PATH as usize)
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as usize) as USHORT,
                Buffer: core::ptr::addr_of_mut!(DllPathU_buffer) as PWSTR,
            };
            let mut Parameters: [ULONG_PTR; 1] = [core::ptr::addr_of_mut!(DllPathU) as ULONG_PTR];
            let mut Response: ULONG = 0;
            RtlInitAnsiString(
                core::ptr::addr_of_mut!(DllPathA),
                (*pdli).szDll as PCSZ,
            );
            RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(DllPathU),
                core::ptr::addr_of_mut!(DllPathA),
                FALSE as BOOLEAN,
            );
            NtRaiseHardError(
                (STATUS_DLL_NOT_FOUND | (HARDERROR_OVERRIDE_ERRORMODE as i32)) as NTSTATUS,
                _countof!(Parameters) as ULONG,
                0x1 as ULONG,
                core::ptr::addr_of_mut!(Parameters) as PULONG_PTR,
                OptionOk as ULONG,
                core::ptr::addr_of_mut!(Response),
            );
            ExitProcess(-(1 as ::core::ffi::c_int) as UINT);
            // break;
        }
        _ => {}
    }
    None
}

unsafe extern "system" fn PrshtWndProc(hWnd: HWND, uMessage: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT {
    match uMessage {
        DM_REPOSITION => {
            /* Center the wizard window */
            CenterWindow(hWnd);
            // FIXME: HACK: See hack in PropSheetCallback()::PSCB_INITIALIZED
            ShowWindow(hWnd, (SW_SHOWNORMAL as i32));
        }

        WM_SETCURSOR => {
            /* Set a wizard-wide cursor */
            // NOTE: There is a problem, where when hovering over the wizard
            // navigation buttons, the cursor would blink with the arrow.
            // To mitigate this problem, only show the custom cursor when
            // we are on the main wizard window.
            if !hWaitCursor.is_null() && wParam == hWnd as WPARAM {
                SetCursor(hWaitCursor);
                return TRUE as LRESULT;
            }
        }

        WM_DESTROY => {
            /* Restore the original dialog procedure */
            ASSERT!(wpOrgPrshtProc.is_some());
            SetWindowLongPtrW(
                hWnd,
                DWLP_DLGPROC as ::core::ffi::c_int,
                ::core::mem::transmute::<WNDPROC, LONG_PTR>(wpOrgPrshtProc),
            );
        }

        _ => {}
    }

    /* Invoke the original dialog procedure */
    return CallWindowProc(wpOrgPrshtProc, hWnd, uMessage, wParam, lParam);
}

unsafe extern "system" fn PropSheetCallback(
    mut hDlg: HWND,
    mut message: UINT,
    mut lParam: LPARAM,
) -> ::core::ffi::c_int {
    match message {
        PSCB_PRECREATE => {
            let mut dlgTemplate: LPDLGTEMPLATE = lParam as LPDLGTEMPLATE;
            let mut dlgTemplateEx: LPDLGTEMPLATEEX = lParam as LPDLGTEMPLATEEX;
            let mut dwStyle: DWORD = 0 as DWORD;
            let mut dwStyleMask: DWORD = 0 as DWORD;

            // FIXME: HACK: See hack in PropSheetCallback()::PSCB_INITIALIZED
            // Hide the dialog by default; DM_REPOSITION will center it on screen then show it.
            dwStyleMask |= WS_VISIBLE as DWORD;

            dwStyle |= DS_CENTER as DWORD; // Center the dialog -- But propsheet code repositions it afterwards...
            //dwStyleMask |= DS_CONTEXTHELP; // TODO: Enable if you want context help.
            dwStyle |= DS_SETFOREGROUND as DWORD; // Ensure we are initially set to the foreground.
            dwStyleMask |= dwStyle;

            /* Set the property sheet dialog styles */
            if (*dlgTemplateEx).signature as ::core::ffi::c_int
                == 0xFFFF as ::core::ffi::c_int
            {
                (*dlgTemplateEx).style =
                    (*dlgTemplateEx).style & !dwStyleMask | dwStyle & dwStyleMask;
            } else {
                (*dlgTemplate).style = (*dlgTemplate).style & !dwStyleMask | dwStyle & dwStyleMask;
            }
        }

        // NOTE: This callback is needed to set large icon correctly.
        PSCB_INITIALIZED => {
            let mut hIcon: HICON = LoadIconW(
                SetupData.hInstance,
                MAKEINTRESOURCEW!(IDI_MAIN as WORD) as LPCWSTR,
            );
            SendMessageW(
                hDlg,
                WM_SETICON as UINT,
                ICON_BIG as WPARAM,
                hIcon as LPARAM,
            );

            /* Sub-class the property sheet window procedure */
            wpOrgPrshtProc = ::core::mem::transmute::<LONG_PTR, WNDPROC>(SetWindowLongPtrW(
                hDlg,
                DWLP_DLGPROC as ::core::ffi::c_int,
                PrshtWndProc as usize as LONG_PTR,
            ));

            // FIXME: HACK: Wine comctl32 propsheet.c doesn't send DM_REPOSITION
            // after creating, initializing and resizing the property sheet dialog,
            // so we simulate its call there...
            PostMessageW(
                hDlg,
                DM_REPOSITION as UINT,
                0 as WPARAM,
                0 as LPARAM,
            );
        }

        _ => {}
    }

    return FALSE as ::core::ffi::c_int;
}

unsafe extern "C" fn CheckForOtherInstance(hInstance: HINSTANCE) -> HANDLE {
    static mut s_hMutex: HANDLE = core::ptr::null_mut();

    /* If we are already running an instance, just return it */
    if !s_hMutex.is_null() {
        return s_hMutex;
    }

    /* Try to create or open the mutex. Use the Global namespace
     * if it exists; otherwise fall back to the default one. */
    s_hMutex = CreateMutexW(core::ptr::null_mut(), (FALSE) as BOOL, RUSTCONV_WSTR_14D3FA313934.as_ptr());
    if s_hMutex.is_null() && (GetLastError() == ERROR_PATH_NOT_FOUND) {
        s_hMutex = CreateMutexW(core::ptr::null_mut(), (FALSE) as BOOL, (REACTOS_SETUP_MUTEX).as_ptr());
    }
    if s_hMutex.is_null() {
        return core::ptr::null_mut(); /* Failed to create the mutex */
    }

    if GetLastError() == ERROR_ALREADY_EXISTS {
        /* The mutex was created by another instance */
        let mut szCaption: [WCHAR; 128] = [0; 128];
        let mut Title: PWSTR = core::ptr::addr_of_mut!(szCaption) as PWSTR; // Use the static buffer by default.
        let mut hWnd: HWND = core::ptr::null_mut();

        /* Find and activate its window */
        let _ = LoadAllocStringW(hInstance, IDS_CAPTION, core::ptr::addr_of_mut!(Title), (_countof!(szCaption)) as size_t);
        if !Title.is_null() {
            /* Locate the window (PropertySheetW uses the dialog class)
             * and retrieve the last popup it may have open */
            let hWndMain: HWND = FindWindowW(RUSTCONV_WSTR_934172CB04C3.as_ptr(), Title as LPCWSTR);
            hWnd = GetLastActivePopup(hWndMain);
            if hWnd.is_null() {
                hWnd = hWndMain;
            }

            if Title != core::ptr::addr_of_mut!(szCaption) as PWSTR {
                HeapFree(GetProcessHeap(), 0, Title as PVOID);
            }
        }
        if !hWnd.is_null() {
            ShowWindow(hWnd, (SW_SHOWNA as i32));
            SwitchToThisWindow(hWnd, (TRUE) as BOOL);
        }

        CloseHandle(s_hMutex);
        return core::ptr::null_mut();
    }

    /* We are the first instance */
    s_hMutex
}
#[no_mangle]
pub unsafe extern "system" fn wWinMain(
    hInst: HINSTANCE,
    hPrevInstance: HINSTANCE,
    lpszCmdLine: LPTSTR,
    nCmdShow: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut Error: ULONG;
    let mut dwHotkeyThreadId: DWORD = 0;
    let mut iccx: INITCOMMONCONTROLSEX = core::mem::zeroed();
    let mut psh: PROPSHEETHEADERW = core::mem::zeroed();
    let mut psp: PROPSHEETPAGEW = core::mem::zeroed();
    let mut ahpsp: [HPROPSHEETPAGE; 9] = [core::ptr::null_mut(); 9];
    let mut nPages: UINT;
    let mut i: UINT;

    /* Ensure only one instance is running */
    if CheckForOtherInstance(hInst) == core::ptr::null_mut() {
        return 0;
    }

    ProcessHeap = GetProcessHeap();

    SetupData.hInstance = hInst;
    SetupData.hInstallThread = core::ptr::null_mut();
    SetupData.hHaltInstallEvent = core::ptr::null_mut();
    SetupData.__bindgen_anon_1.bStopInstall = (FALSE as u8);

    /* Initialize the NT to Win32 path prefix mapping list */
    InitNtToWin32PathMappingList(core::ptr::addr_of_mut!(SetupData.MappingList));

    /* Initialize Setup */
    Error = InitializeSetup(
        core::ptr::addr_of_mut!(SetupData.USetupData),
        None,
        core::ptr::addr_of_mut!(SpFileExports),
        core::ptr::addr_of_mut!(SpInfExports),
    );
    'quit: {
        if Error != ERROR_SUCCESS {
            //
            // TODO: Write an error mapper (much like the MUIDisplayError of USETUP)
            //
            if Error == ERROR_NO_SOURCE_DRIVE {
                MessageBoxW(
                    core::ptr::null_mut(),
                    RUSTCONV_WSTR_151330312B2E.as_ptr(),
                    core::ptr::null_mut(),
                    MB_ICONERROR,
                );
            } else if Error == ERROR_LOAD_TXTSETUPSIF {
                DisplayError(core::ptr::null_mut(), IDS_CAPTION, IDS_NO_TXTSETUP_SIF);
            } else {
                // FIXME!!
                MessageBoxW(
                    core::ptr::null_mut(),
                    RUSTCONV_WSTR_546CAB492E94.as_ptr(),
                    core::ptr::null_mut(),
                    MB_ICONERROR,
                );
            }

            break 'quit;
        }

        /* Retrieve any supplemental options from the unattend file */
        SetupData.bUnattend = (CheckUnattendedSetup(core::ptr::addr_of_mut!(SetupData.USetupData))) as _;

        /* Load extra setup data (HW lists etc...) */
        if LoadSetupData(core::ptr::addr_of_mut!(SetupData)) == 0 {
            break 'quit;
        }

        /* Start the hotkey thread */
        CloseHandle(CreateThread(
            core::ptr::null_mut(),
            0,
            Some(HotkeyThread),
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(dwHotkeyThreadId),
        ));

        /* Whenever any of the common controls are used in your app,
         * you must call InitCommonControlsEx() to register the classes
         * for those controls. */
        iccx.dwSize = core::mem::size_of::<INITCOMMONCONTROLSEX>() as DWORD;
        iccx.dwICC = ICC_LISTVIEW_CLASSES | ICC_TREEVIEW_CLASSES | ICC_PROGRESS_CLASS;
        InitCommonControlsEx(core::ptr::addr_of_mut!(iccx));

        /* Register the TreeList control */
        // RegisterTreeListClass(hInst);
        TreeListRegister(hInst);

        /* Create the title and bold fonts */
        SetupData.hTitleFont = CreateTitleFont(core::ptr::null_mut());
        SetupData.hBoldFont = CreateBoldFont(core::ptr::null_mut(), 0);

        /* Create each page */
        psp.dwSize = core::mem::size_of::<PROPSHEETPAGEW>() as DWORD;
        psp.hInstance = hInst;
        psp.lParam = core::ptr::addr_of_mut!(SetupData) as LPARAM;
        nPages = 0;
        i = 0;
        while (i as usize) < _countof!(WizardPages) as usize {
            'body: {
                /* Skip pages that don't apply to unattended mode */
                if SetupData.bUnattend != 0 && WizardPages[i as usize].IncludeForUnattended == 0 {
                    break 'body;
                }

                psp.dwFlags = PSP_DEFAULT | WizardPages[i as usize].dwFlags;
                psp.__bindgen_anon_1.pszTemplate = WizardPages[i as usize].pszTemplate;
                // FIXME: HACK: Wine comctl32 propsheet.c doesn't correctly set the wizard
                // dialog title when the pages don't have captions, even if the user sends
                // an initial PSM_SETTITLE message via the callback -- see CORE-20687.
                // To avert this problem, force-set the same title to each page, before
                // creating the wizard.
                psp.dwFlags |= PSP_USETITLE;
                psp.pszTitle = MAKEINTRESOURCEW!(IDS_CAPTION);
                psp.pfnDlgProc = WizardPages[i as usize].pfnDlgProc;
                psp.pszHeaderTitle = WizardPages[i as usize].pszHeaderTitle;
                psp.pszHeaderSubTitle = WizardPages[i as usize].pszHeaderSubTitle;
                let fresh = nPages;
                nPages += 1;
                ahpsp[fresh as usize] = CreatePropertySheetPage(core::ptr::addr_of_mut!(psp));
            }
            i += 1;
        }

        /* Create the property sheet */
        psh.dwSize = core::mem::size_of::<PROPSHEETHEADERW>() as DWORD;
        psh.dwFlags = PSH_WIZARD97
            | PSH_USEICONID
            | PSH_USECALLBACK
            | PSH_WATERMARK
            | PSH_HEADER;
        psh.hInstance = hInst;
        psh.hwndParent = core::ptr::null_mut();
        psh.__bindgen_anon_1.pszIcon = MAKEINTRESOURCEW!(IDI_MAIN);
        psh.nPages = nPages;
        psh.__bindgen_anon_2.nStartPage = 0;
        psh.__bindgen_anon_3.phpage = ahpsp.as_mut_ptr();
        psh.pfnCallback = Some(PropSheetCallback);
        psh.__bindgen_anon_4.pszbmWatermark = MAKEINTRESOURCEW!(IDB_WATERMARK);
        psh.__bindgen_anon_5.pszbmHeader = MAKEINTRESOURCEW!(IDB_HEADER);

        /* Display the wizard */
        PropertySheetW(core::ptr::addr_of_mut!(psh));

        if !SetupData.hBoldFont.is_null() {
            DeleteFont!(SetupData.hBoldFont);
        }
        if !SetupData.hTitleFont.is_null() {
            DeleteFont!(SetupData.hTitleFont);
        }

        /* Unregister the TreeList control */
        // UnregisterTreeListClass(hInst);
        TreeListUnregister(hInst);

        /* Stop the hotkey thread */
        if dwHotkeyThreadId != 0 {
            PostThreadMessageW(
                dwHotkeyThreadId,
                WM_QUIT,
                0,
                0,
            );
        }
    }

    /* Setup has finished */
    FinishSetup(core::ptr::addr_of_mut!(SetupData.USetupData));

    /* Free the NT to Win32 path prefix mapping list */
    FreeNtToWin32PathMappingList(core::ptr::addr_of_mut!(SetupData.MappingList));

    /* Force reboot if there are no other user-interactive windows opened */
    SetupData.bMustReboot |= (AreThereInteractiveWindows(core::ptr::null_mut()) == 0) as BOOL;

    /* System rebooting will be done by Winlogon if necessary */
    if SetupData.bMustReboot != 0 {
        EnablePrivilege(SE_SHUTDOWN_NAME, (TRUE) as BOOL);
        ExitWindowsEx(EWX_REBOOT, 0);
        EnablePrivilege(SE_SHUTDOWN_NAME, (FALSE) as BOOL);
    }
    return 0;
}
