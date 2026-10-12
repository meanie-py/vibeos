// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_09486E12FA3E: [u16; 34] = [
    0x0023u16, 0x0020, 0x0045, 0x006E, 0x0064, 0x0020, 0x006F, 0x0066, 0x0020, 0x0049,
    0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x0020, 0x0043,
    0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0075, 0x0072, 0x0061, 0x0074, 0x0069,
    0x006F, 0x006E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_164B079CD949: [u16; 27] = [
    0x0023u16, 0x0020, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063,
    0x0065, 0x0020, 0x0043, 0x006F, 0x006E, 0x0066, 0x0069, 0x0067, 0x0075, 0x0072,
    0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_1A94C3A3FBF9: [u16; 10] = [0x0069u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_579325F9E47B: [u16; 17] = [
    0x0070u16, 0x0075, 0x0073, 0x0068, 0x0064, 0x0020, 0x0069, 0x006E, 0x0074, 0x0065,
    0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_CF0DF476CD22: [u16; 48] = [
    0x0049u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x0053,
    0x0068, 0x006F, 0x0077, 0x0049, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061,
    0x0063, 0x0065, 0x0028, 0x0029, 0x003A, 0x0020, 0x004E, 0x006F, 0x0074, 0x0020,
    0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_DC3CC836BA3F: [u16; 6] = [0x0070u16, 0x006F, 0x0070, 0x0064, 0x000A, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_1A94C3A3FBF9: [u16; 10] = [0x0069u16, 0x006E, 0x0074, 0x0065, 0x0072, 0x0066, 0x0061, 0x0063, 0x0065, 0x0000];

static mut InterfaceShowCommands: [CMD_ENTRY; 1] = unsafe {
    [(_CMD_ENTRY {
        pwszCmdToken: RUSTCONV_C2R_WSTR_1A94C3A3FBF9.as_ptr() as LPCWSTR,
        pfnCmdHandler: Some(
            InterfaceShowInterface,
        ),
        dwShortCmdHelpToken: IDS_HLP_INTERFACE_SHOW_INTERFACE as DWORD,
        dwCmdHlpToken: IDS_HLP_INTERFACE_SHOW_INTERFACE_EX as DWORD,
        dwFlags: 0 as DWORD,
        pOsVersionCheck: None,
    })]
};

static mut InterfaceGroups: [CMD_GROUP_ENTRY; 1] = [(_CMD_GROUP_ENTRY {
        pwszCmdGroupToken: { static W: [u16; 5] = unsafe { ::core::mem::transmute::<[u8; 10], [u16; 5]>(*b"s\0h\0o\0w\0\0\0") }; W.as_ptr() } as LPCWSTR,
        dwShortCmdHelpToken: IDS_HLP_INTERFACE_SHOW as DWORD,
        ulCmdGroupSize: (::core::mem::size_of::<[CMD_ENTRY; 1]>() as usize)
            .wrapping_div(::core::mem::size_of::<CMD_ENTRY>() as usize)
            as ULONG,
        dwFlags: 0 as DWORD,
        pCmdGroup: &raw mut InterfaceShowCommands as PCMD_ENTRY,
        pOsVersionCheck: None,
    })];

// #define NDEBUG: flag-only macro with no value, so no Rust constant is emitted.

// Forward declaration `static FN_HANDLE_CMD InterfaceShowInterface;` is a function
// declaration only. The function is defined elsewhere and referenced by name below.

// netsh.h's typedef, as the island has it. The preamble had re-declared the struct
// with invented fields.
pub type CMD_ENTRY = _CMD_ENTRY;

pub type CMD_GROUP_ENTRY = _CMD_GROUP_ENTRY;
unsafe extern "system" fn InterfaceShowInterface(
    mut pwszMachine: LPCWSTR,
    mut argv: *mut LPWSTR,
    mut dwCurrentIndex: DWORD,
    mut dwArgCount: DWORD,
    mut dwFlags: DWORD,
    mut pvData: LPCVOID,
    mut pbDone: *mut BOOL,
) -> DWORD {
    PrintMessage(RUSTCONV_WSTR_CF0DF476CD22.as_ptr());

    return ERROR_SUCCESS as DWORD;
}

unsafe extern "system" fn InterfaceDumpFn(
    pwszRouter: LPCWSTR,
    ppwcArguments: *mut LPWSTR,
    dwArgCount: DWORD,
    pvData: LPCVOID,
) -> DWORD {
    DPRINT!("InterfaceDumpFn(%S %p %lu %p)\n", pwszRouter, ppwcArguments, dwArgCount, pvData);

    PrintMessageFromModule((hDllInstance).cast(), IDS_DUMP_HEADERLINE);
    PrintMessage(RUSTCONV_WSTR_164B079CD949.as_ptr());
    PrintMessageFromModule((hDllInstance).cast(), IDS_DUMP_HEADERLINE);
    PrintMessage(RUSTCONV_WSTR_579325F9E47B.as_ptr());
    PrintMessageFromModule((hDllInstance).cast(), IDS_DUMP_NEWLINE);

    PrintMessageFromModule((hDllInstance).cast(), IDS_DUMP_NEWLINE);
    PrintMessage(RUSTCONV_WSTR_DC3CC836BA3F.as_ptr());
    PrintMessage(RUSTCONV_WSTR_09486E12FA3E.as_ptr());
    PrintMessageFromModule((hDllInstance).cast(), IDS_DUMP_NEWLINE);

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn InterfaceStart(pguidParent: *const GUID, dwVersion: DWORD) -> DWORD {
    let mut ContextAttributes: NS_CONTEXT_ATTRIBUTES = core::mem::zeroed();

    DPRINT!("InterfaceStart()\n");

    ZeroMemory!(
        &raw mut ContextAttributes as *mut core::ffi::c_void,
        core::mem::size_of::<NS_CONTEXT_ATTRIBUTES>(),
    );
    ContextAttributes.__bindgen_anon_1.__bindgen_anon_1.dwVersion = 1;
    ContextAttributes.pwszContext = RUSTCONV_WSTR_1A94C3A3FBF9.as_ptr() as LPWSTR;
    ContextAttributes.guidHelper = GUID_IFMON_INTERFACE;

    ContextAttributes.ulNumTopCmds = 0;
    ContextAttributes.pTopCmds = core::ptr::null_mut();

    ContextAttributes.ulNumGroups = (core::mem::size_of::<[CMD_GROUP_ENTRY; 1]>()
        / core::mem::size_of::<CMD_GROUP_ENTRY>()) as ULONG;
    ContextAttributes.pCmdGroups = (&raw mut InterfaceGroups as *mut CMD_GROUP_ENTRY).cast();

    ContextAttributes.pfnDumpFn = Some(InterfaceDumpFn);

    RegisterContext(&raw mut ContextAttributes);

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterInterfaceHelper() -> DWORD {
    let mut HelperAttributes: NS_HELPER_ATTRIBUTES = core::mem::zeroed();

    DPRINT!("RegisterInterfaceHelper()\n");

    ZeroMemory!(
        core::ptr::addr_of_mut!(HelperAttributes) as *mut core::ffi::c_void,
        core::mem::size_of::<NS_HELPER_ATTRIBUTES>(),
    );
    HelperAttributes.__bindgen_anon_1.__bindgen_anon_1.dwVersion = 1;
    HelperAttributes.guidHelper = GUID_IFMON_INTERFACE;
    HelperAttributes.pfnStart =
        Some(InterfaceStart);
    HelperAttributes.pfnStop = None;
    RegisterHelper(core::ptr::null_mut(), core::ptr::addr_of_mut!(HelperAttributes));

    return ERROR_SUCCESS;
}
