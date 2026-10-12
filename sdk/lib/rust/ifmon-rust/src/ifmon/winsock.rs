// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_7F7633C5DD1C: [u16; 8] = [0x0077u16, 0x0069, 0x006E, 0x0073, 0x006F, 0x0063, 0x006B, 0x0000];
pub static RUSTCONV_WSTR_CE72071156FB: [u16; 38] = [
    0x0052u16, 0x0065, 0x0073, 0x0065, 0x0074, 0x0043, 0x006F, 0x006D, 0x006D, 0x0061,
    0x006E, 0x0064, 0x0028, 0x0029, 0x003A, 0x0020, 0x004E, 0x006F, 0x0074, 0x0020,
    0x0069, 0x006D, 0x0070, 0x006C, 0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065,
    0x0064, 0x0020, 0x0079, 0x0065, 0x0074, 0x0021, 0x000A, 0x0000,
];
pub static RUSTCONV_WSTR_F6E8604E480E: [u16; 44] = [
    0x0053u16, 0x0068, 0x006F, 0x0077, 0x0043, 0x0061, 0x0074, 0x0061, 0x006C, 0x006F,
    0x0067, 0x0043, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0028, 0x0029,
    0x003A, 0x0020, 0x004E, 0x006F, 0x0074, 0x0020, 0x0069, 0x006D, 0x0070, 0x006C,
    0x0065, 0x006D, 0x0065, 0x006E, 0x0074, 0x0065, 0x0064, 0x0020, 0x0079, 0x0065,
    0x0074, 0x0021, 0x000A, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub static RUSTCONV_C2R_WSTR_228C0BB400D4: [u16; 8] = [0x0063u16, 0x0061, 0x0074, 0x0061, 0x006C, 0x006F, 0x0067, 0x0000];
pub static RUSTCONV_C2R_WSTR_C97C62500E9C: [u16; 6] = [0x0072u16, 0x0065, 0x0073, 0x0065, 0x0074, 0x0000];

static mut g_TestCmdTable: [CMD_ENTRY; 1] = unsafe {
    [(_CMD_ENTRY {
        pwszCmdToken: RUSTCONV_C2R_WSTR_C97C62500E9C.as_ptr() as LPCWSTR,
        pfnCmdHandler: Some(
            ResetCommand,
        ),
        dwShortCmdHelpToken: IDS_HLP_WINSOCK_RESET as DWORD,
        dwCmdHlpToken: IDS_HLP_WINSOCK_RESET_EX as DWORD,
        dwFlags: 0 as DWORD,
        pOsVersionCheck: None,
    })]
};

static mut g_WinsockShowCmdTable: [CMD_ENTRY; 1] = unsafe {
    [(_CMD_ENTRY {
        pwszCmdToken: RUSTCONV_C2R_WSTR_228C0BB400D4.as_ptr() as LPCWSTR,
        pfnCmdHandler: Some(
            ShowCatalogCommand,
        ),
        dwShortCmdHelpToken: IDS_HLP_WINSOCK_SHOW_CATALOG as DWORD,
        dwCmdHlpToken: IDS_HLP_WINSOCK_SHOW_CATALOG_EX as DWORD,
        dwFlags: 0 as DWORD,
        pOsVersionCheck: None,
    })]
};

static mut g_WinsockGroupCmds: [CMD_GROUP_ENTRY; 1] = [(_CMD_GROUP_ENTRY {
        pwszCmdGroupToken: { static W: [u16; 5] = unsafe { ::core::mem::transmute::<[u8; 10], [u16; 5]>(*b"s\0h\0o\0w\0\0\0") }; W.as_ptr() } as LPCWSTR,
        dwShortCmdHelpToken: IDS_HLP_WINSOCK_SHOW as DWORD,
        ulCmdGroupSize: (::core::mem::size_of::<[CMD_ENTRY; 1]>() as usize)
            .wrapping_div(::core::mem::size_of::<CMD_ENTRY>() as usize)
            as ULONG,
        dwFlags: 0 as DWORD,
        pCmdGroup: &raw mut g_WinsockShowCmdTable as PCMD_ENTRY,
        pOsVersionCheck: None,
    })];
unsafe extern "system" fn ResetCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    PrintMessage(RUSTCONV_WSTR_CE72071156FB.as_ptr());
    return ERROR_SUCCESS;
}

unsafe extern "system" fn ShowCatalogCommand(
    pwszMachine: LPCWSTR,
    argv: *mut LPWSTR,
    dwCurrentIndex: DWORD,
    dwArgCount: DWORD,
    dwFlags: DWORD,
    pvData: LPCVOID,
    pbDone: *mut BOOL,
) -> DWORD {
    PrintMessage(RUSTCONV_WSTR_F6E8604E480E.as_ptr());
    return ERROR_SUCCESS;
}
unsafe extern "system" fn WinsockStart(pguidParent: *const GUID, dwVersion: DWORD) -> DWORD {
    let mut ContextAttributes: NS_CONTEXT_ATTRIBUTES = core::mem::zeroed();
    DPRINT1!("WinsockStart()\n");

    ZeroMemory!(core::ptr::addr_of_mut!(ContextAttributes) as *mut core::ffi::c_void, core::mem::size_of::<NS_CONTEXT_ATTRIBUTES>());
    (*core::ptr::addr_of_mut!(ContextAttributes)).__bindgen_anon_1.__bindgen_anon_1.dwVersion = 1;
    (*core::ptr::addr_of_mut!(ContextAttributes)).pwszContext = RUSTCONV_WSTR_7F7633C5DD1C.as_ptr() as LPWSTR;
    (*core::ptr::addr_of_mut!(ContextAttributes)).guidHelper = GUID_IFMON_WINSOCK;

    (*core::ptr::addr_of_mut!(ContextAttributes)).ulNumTopCmds = 1;
    (*core::ptr::addr_of_mut!(ContextAttributes)).pTopCmds = (core::ptr::addr_of_mut!(g_TestCmdTable) as *mut CMD_ENTRY).cast();

    (*core::ptr::addr_of_mut!(ContextAttributes)).ulNumGroups = 1;
    (*core::ptr::addr_of_mut!(ContextAttributes)).pCmdGroups = (core::ptr::addr_of_mut!(g_WinsockGroupCmds) as *mut CMD_GROUP_ENTRY).cast();

    RegisterContext(core::ptr::addr_of_mut!(ContextAttributes));

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWinsockHelper() -> DWORD {
    let mut HelperAttributes: NS_HELPER_ATTRIBUTES = core::mem::zeroed();

    ZeroMemory!(
        core::ptr::addr_of_mut!(HelperAttributes) as *mut core::ffi::c_void,
        core::mem::size_of::<NS_HELPER_ATTRIBUTES>() as usize,
    );
    HelperAttributes.__bindgen_anon_1.__bindgen_anon_1.dwVersion = 1;
    HelperAttributes.guidHelper = GUID_IFMON_WINSOCK;
    HelperAttributes.pfnStart = Some(WinsockStart);
    HelperAttributes.pfnStop = None;
    RegisterHelper(
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(HelperAttributes),
    );

    return ERROR_SUCCESS;
}
