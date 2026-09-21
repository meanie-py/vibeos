use crate::prelude::*;
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_7289CE32E451: [WCHAR; 8] = [0x0050, 0x0072, 0x006F, 0x0067, 0x006D, 0x0061, 0x006E, 0x0000];
pub static CPP_COM_WSTR_7D196D7F9FE9: [WCHAR; 14] = [0x0041, 0x0070, 0x0070, 0x0050, 0x0072, 0x006F, 0x0070, 0x0065, 0x0072, 0x0074, 0x0069, 0x0065, 0x0073, 0x0000];
pub static CPP_COM_WSTR_96A296D224F2: [WCHAR; 1] = [0x0000];
pub static CPP_COM_WSTR_A1094FC54F9E: [WCHAR; 8] = [0x0046, 0x006F, 0x006C, 0x0064, 0x0065, 0x0072, 0x0073, 0x0000];
pub static CPP_COM_WSTR_C2C6793CDF80: [WCHAR; 6] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub type pfnCommandHandler = core::option::Option<unsafe extern "system" fn(PWSTR, PWSTR, LPITEMIDLIST, INT) -> DWORD>;

#[repr(C)]
pub struct DDECommandHandler {
    pub Command: [u16; 32],
    pub Handler: pfnCommandHandler,
}

static mut dwDDEInst: DWORD = 0;
static mut hszProgmanTopic: HSZ = core::ptr::null_mut();
static mut hszProgmanService: HSZ = core::ptr::null_mut();
static mut hszShell: HSZ = core::ptr::null_mut();
static mut hszAppProperties: HSZ = core::ptr::null_mut();
static mut hszFolders: HSZ = core::ptr::null_mut();
static mut bInitialized: BOOL = 0;

pub static mut HandlerList: [DDECommandHandler; 4] = [
    DDECommandHandler {
        Command: [
            'V' as u16, 'i' as u16, 'e' as u16, 'w' as u16,
            'F' as u16, 'o' as u16, 'l' as u16, 'd' as u16,
            'e' as u16, 'r' as u16, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
        Handler: Some(DDE_OnViewFolder),
    },
    DDECommandHandler {
        Command: [
            'E' as u16, 'x' as u16, 'p' as u16, 'l' as u16,
            'o' as u16, 'r' as u16, 'e' as u16, 'F' as u16,
            'o' as u16, 'l' as u16, 'd' as u16, 'e' as u16,
            'r' as u16, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
        ],
        Handler: Some(DDW_OnExploreFolder),
    },
    DDECommandHandler {
        Command: [
            'F' as u16, 'i' as u16, 'n' as u16, 'd' as u16,
            'F' as u16, 'o' as u16, 'l' as u16, 'd' as u16,
            'e' as u16, 'r' as u16, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
        Handler: Some(DDE_OnFindFolder),
    },
    DDECommandHandler {
        Command: [
            'S' as u16, 'h' as u16, 'e' as u16, 'l' as u16,
            'l' as u16, 'F' as u16, 'i' as u16, 'l' as u16,
            'e' as u16, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ],
        Handler: Some(DDE_OnShellFile),
    },
];

pub const HandlerListLength: i32 = 4;
unsafe extern "C" fn Dde_OnConnect(hszTopic: HSZ, hszService: HSZ) -> BOOL {
    let mut szTopic: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szService: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    DdeQueryStringW(dwDDEInst, hszTopic, szTopic.as_mut_ptr(), szTopic.len() as u32, CP_WINUNICODE as i32);
    DdeQueryStringW(dwDDEInst, hszService, szService.as_mut_ptr(), szService.len() as u32, CP_WINUNICODE as i32);

    TRUE
}
unsafe extern "C" fn Dde_OnConnectConfirm(hconv: HCONV, hszTopic: HSZ, hszService: HSZ) {
    let mut szTopic = [0u16; MAX_PATH as usize];
    let mut szService = [0u16; MAX_PATH as usize];

    unsafe {
        DdeQueryStringW(
            dwDDEInst,
            hszTopic,
            szTopic.as_mut_ptr(),
            szTopic.len() as u32,
            CP_WINUNICODE as i32
        );
        DdeQueryStringW(
            dwDDEInst,
            hszService,
            szService.as_mut_ptr(),
            szService.len() as u32,
            CP_WINUNICODE as i32
        );
    }
}
unsafe extern "C" fn Dde_OnWildConnect(hszTopic: HSZ, hszService: HSZ) -> BOOL {
    let mut szTopic = [0u16; MAX_PATH as usize];
    let mut szService = [0u16; MAX_PATH as usize];

    DdeQueryStringW(dwDDEInst, hszTopic, szTopic.as_mut_ptr(), szTopic.len() as u32, CP_WINUNICODE as i32);
    DdeQueryStringW(dwDDEInst, hszService, szService.as_mut_ptr(), szService.len() as u32, CP_WINUNICODE as i32);

    return FALSE;
}
unsafe extern "C" fn Dde_OnRequest(_uFmt: UINT, _hconv: HCONV, hszTopic: HSZ, hszItem: HSZ) -> HDDEDATA {
    let mut szTopic = [0u16; MAX_PATH as usize];
    let mut szItem = [0u16; MAX_PATH as usize];

    unsafe {
        DdeQueryStringW(dwDDEInst, hszTopic, szTopic.as_mut_ptr(), szTopic.len() as u32, CP_WINUNICODE as i32);
        DdeQueryStringW(dwDDEInst, hszItem, szItem.as_mut_ptr(), szItem.len() as u32, CP_WINUNICODE as i32);
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn _ILReadFromSharedMemory(strField: PCWSTR) -> LPITEMIDLIST {
    let mut ret = core::ptr::null_mut();

    if *strField != 0x003Au16 {
        return ret;
    }

    let hData = UlongToHandle(StrToIntW(strField.offset(1)) as u32);
    let strSecond = StrChrW(strField.offset(1), 0x003Au16);

    if !strSecond.is_null() {
        let pid = StrToIntW(strSecond.offset(1));
        let pvShared = SHLockShared(hData, pid as u32);

        if !pvShared.is_null() {
            ret = ILClone(pvShared as LPCITEMIDLIST);
            SHUnlockShared(pvShared);
            SHFreeShared(hData, pid as u32);
        }
    }

    ret
}
unsafe extern "C" fn Dde_OnExecute(hconv: HCONV, hszTopic: HSZ, hdata: HDDEDATA) -> DWORD {
    let mut szTopic: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut szCommand: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];

    DdeQueryStringW(dwDDEInst, hszTopic, szTopic.as_mut_ptr(), szTopic.len() as u32, CP_WINUNICODE as i32);

    let pszCommand = DdeAccessData(hdata, core::ptr::null_mut()) as *mut WCHAR;
    if pszCommand.is_null() {
        return DDE_FNOTPROCESSED;
    }

    StringCchCopyW(szCommand.as_mut_ptr(), szCommand.len(), pszCommand);
    DdeUnaccessData(hdata);

    let mut Command: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut Path: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();
    let _ = CPP_COM_WSTR_96A296D224F2.as_ptr();

    let mut IdList: LPITEMIDLIST = core::ptr::null_mut();
    let mut UnknownParameter: INT = 0;

    let mut cmd: *mut WCHAR = szCommand.as_mut_ptr();

    if *cmd == 0x005Bu16 {
        cmd = cmd.add(1);
    }

    if *cmd == 0x005Du16 {
        return DDE_FNOTPROCESSED;
    }

    let mut cmdEnd = StrChrW(cmd, 0x0028u16);
    if cmdEnd.is_null() {
        return DDE_FNOTPROCESSED;
    }

    *cmdEnd = 0;
    // dde.cpp:194 spells this `StringCchCopy`, the TCHAR-GENERIC name, while :138 spells
    // the same function `StringCchCopyW`. The TU builds with -DUNICODE (shelldesktop's
    // CMakeLists adds it), so strsafe.h's generic macro IS the W entry point -- resolving
    // it here is what the C preprocessor does, not a substitution.
    StringCchCopyW(Command.as_mut_ptr(), Command.len(), cmd);
    cmd = cmdEnd.add(1);

    if *cmd != 0x0029u16 {
        let mut dst: *mut WCHAR = Path.as_mut_ptr();
        let mut isQuote: BOOL = FALSE;
        let mut arg: *mut WCHAR = cmd;

        while *arg != 0 && (isQuote != 0 || *arg != 0x002Cu16) {
            if *arg == 0x0022u16 {
                isQuote = if isQuote == 0 { 1 } else { 0 };
                if isQuote != 0 && arg != cmd {
                    *dst = 0x0022u16;
                    dst = dst.add(1);
                }
            } else {
                *dst = *arg;
                dst = dst.add(1);
            }
            arg = arg.add(1);
        }

        cmd = arg.add(1);
        while *cmd == 0x0020u16 {
            cmd = cmd.add(1);
        }
    }

    if *cmd != 0x0029u16 {
        if *cmd != 0x003Au16 {
            return DDE_FNOTPROCESSED;
        }

        let mut idlistEnd = StrChrW(cmd, 0x002Cu16);
        if idlistEnd.is_null() {
            idlistEnd = StrChrW(cmd, 0x0029u16);
        }
        if idlistEnd.is_null() {
            return DDE_FNOTPROCESSED;
        }

        IdList = _ILReadFromSharedMemory(cmd as PCWSTR);
        cmd = idlistEnd.add(1);
    }

    if *cmd != 0x0029u16 {
        UnknownParameter = StrToIntW(cmd);
    }

    let mut i = 0;
    while i < HandlerListLength {
        let handler = &HandlerList[i as usize];
        if StrCmpW(handler.Command.as_ptr(), Command.as_ptr()) == 0 {
            return (handler.Handler.unwrap_unchecked())(Command.as_mut_ptr(), Path.as_mut_ptr(), IdList, UnknownParameter);
        }
        i += 1;
    }

    return DDE_FNOTPROCESSED;
}

unsafe extern "C" fn Dde_OnDisconnect(hconv: HCONV) {
    // TRACE("Dde_OnDisconnect: hconv=%p\n", hconv);
}
// dde.cpp declares this `static DWORD CALLBACK`, and CALLBACK is __stdcall. That is
// exactly what `extern "system"` means -- stdcall on i386, the C convention everywhere
// else -- so ONE definition covers all three arches. It arrived as a
// `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` twin
// pair with two byte-identical bodies, which is the documented cfg-twin anti-pattern:
// redundant on i386 and WRONG off it, because the handler table's `pfnCommandHandler`
// is `extern "system"` and Rust types `extern "C"` as a DIFFERENT type even where the
// ABI coincides (E0308 on amd64/arm64).
unsafe extern "system" fn DdeCallback(
    uType: UINT,
    uFmt: UINT,
    hconv: HCONV,
    hsz1: HSZ,
    hsz2: HSZ,
    hdata: HDDEDATA,
    dwData1: ULONG_PTR,
    dwData2: ULONG_PTR,
) -> HDDEDATA {
    match uType {
        XTYP_CONNECT => {
            return Dde_OnConnect(hsz1, hsz2) as DWORD_PTR as HDDEDATA;
        }
        XTYP_CONNECT_CONFIRM => {
            Dde_OnConnectConfirm(hconv, hsz1, hsz2);
            return core::ptr::null_mut();
        }
        XTYP_WILDCONNECT => {
            return Dde_OnWildConnect(hsz1, hsz2) as DWORD_PTR as HDDEDATA;
        }
        XTYP_REQUEST => {
            return Dde_OnRequest(uFmt, hconv, hsz1, hsz2);
        }
        XTYP_EXECUTE => {
            return Dde_OnExecute(hconv, hsz1, hdata) as DWORD_PTR as HDDEDATA;
        }
        XTYP_DISCONNECT => {
            Dde_OnDisconnect(hconv);
            return core::ptr::null_mut();
        }
        XTYP_REGISTER => {
            return core::ptr::null_mut();
        }
        _ => {
            WARN!("DdeCallback: unknown uType=%d\n", uType);
            return core::ptr::null_mut();
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn ShellDDEInit(bInit: i32) {
    if bInit != 0 && bInitialized == 0 {
        DdeInitializeW(
            core::ptr::addr_of_mut!(dwDDEInst),
            Some(DdeCallback),
            CBF_FAIL_ADVISES | CBF_FAIL_POKES,
            0,
        );

        hszProgmanTopic = DdeCreateStringHandleW(
            dwDDEInst,
            CPP_COM_WSTR_7289CE32E451.as_ptr(),
            CP_WINUNICODE as i32,
        );
        hszProgmanService = DdeCreateStringHandleW(
            dwDDEInst,
            CPP_COM_WSTR_7289CE32E451.as_ptr(),
            CP_WINUNICODE as i32,
        );
        hszShell = DdeCreateStringHandleW(
            dwDDEInst,
            CPP_COM_WSTR_C2C6793CDF80.as_ptr(),
            CP_WINUNICODE as i32,
        );
        hszAppProperties = DdeCreateStringHandleW(
            dwDDEInst,
            CPP_COM_WSTR_7D196D7F9FE9.as_ptr(),
            CP_WINUNICODE as i32,
        );
        hszFolders = DdeCreateStringHandleW(
            dwDDEInst,
            CPP_COM_WSTR_A1094FC54F9E.as_ptr(),
            CP_WINUNICODE as i32,
        );

        if hszProgmanTopic as usize != 0
            && hszProgmanService as usize != 0
            && hszShell as usize != 0
            && hszAppProperties as usize != 0
            && hszFolders as usize != 0
            && (DdeNameService(
                dwDDEInst,
                hszFolders,
                core::ptr::null_mut(),
                DNS_REGISTER,
            ) as usize != 0)
            && (DdeNameService(
                dwDDEInst,
                hszProgmanService,
                core::ptr::null_mut(),
                DNS_REGISTER,
            ) as usize != 0)
            && (DdeNameService(
                dwDDEInst,
                hszShell,
                core::ptr::null_mut(),
                DNS_REGISTER,
            ) as usize != 0)
        {
            bInitialized = TRUE;
        }
    } else if bInit == 0 && bInitialized != 0 {
        DdeNameService(
            dwDDEInst,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            DNS_UNREGISTER,
        );

        if hszFolders as usize != 0 {
            DdeFreeStringHandle(dwDDEInst, hszFolders);
        }
        if hszAppProperties as usize != 0 {
            DdeFreeStringHandle(dwDDEInst, hszAppProperties);
        }
        if hszShell as usize != 0 {
            DdeFreeStringHandle(dwDDEInst, hszShell);
        }
        if hszProgmanService as usize != 0 {
            DdeFreeStringHandle(dwDDEInst, hszProgmanService);
        }
        if hszProgmanTopic as usize != 0 {
            DdeFreeStringHandle(dwDDEInst, hszProgmanTopic);
        }

        DdeUninitialize(dwDDEInst);

        bInitialized = FALSE;
    }
}
// dde.cpp declares this `static DWORD CALLBACK`, and CALLBACK is __stdcall. That is
// exactly what `extern "system"` means -- stdcall on i386, the C convention everywhere
// else -- so ONE definition covers all three arches. It arrived as a
// `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` twin
// pair with two byte-identical bodies, which is the documented cfg-twin anti-pattern:
// redundant on i386 and WRONG off it, because the handler table's `pfnCommandHandler`
// is `extern "system"` and Rust types `extern "C"` as a DIFFERENT type even where the
// ABI coincides (E0308 on amd64/arm64).
unsafe extern "system" fn DDE_OnViewFolder(
    strCommand: PWSTR,
    strPath: PWSTR,
    pidl: LPITEMIDLIST,
    unkS: i32,
) -> u32 {
    let mut pidl = pidl;
    if pidl.is_null() {
        pidl = ILCreateFromPathW(strPath);
    }

    if pidl.is_null() {
        return DDE_FNOTPROCESSED;
    }

    if SHOpenNewFrame(
        pidl,
        core::ptr::null_mut(),
        0,
        0,
    ) < 0
    {
        return DDE_FNOTPROCESSED;
    }

    DDE_FACK
}
// dde.cpp declares this `static DWORD CALLBACK`, and CALLBACK is __stdcall. That is
// exactly what `extern "system"` means -- stdcall on i386, the C convention everywhere
// else -- so ONE definition covers all three arches. It arrived as a
// `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` twin
// pair with two byte-identical bodies, which is the documented cfg-twin anti-pattern:
// redundant on i386 and WRONG off it, because the handler table's `pfnCommandHandler`
// is `extern "system"` and Rust types `extern "C"` as a DIFFERENT type even where the
// ABI coincides (E0308 on amd64/arm64).
unsafe extern "system" fn DDW_OnExploreFolder(
    strCommand: PWSTR,
    strPath: PWSTR,
    pidl: LPITEMIDLIST,
    unkS: INT
) -> DWORD {
    let mut pidl = pidl;
    if pidl.is_null() {
        pidl = ILCreateFromPathW(strPath);
    }

    if pidl.is_null() {
        return DDE_FNOTPROCESSED;
    }

    if SHOpenNewFrame(
        pidl,
        core::ptr::null_mut(),
        0,
        SH_EXPLORER_CMDLINE_FLAG_E
    ) < 0
    {
        return DDE_FNOTPROCESSED;
    }

    return DDE_FACK;
}

// dde.cpp declares this `static DWORD CALLBACK`, and CALLBACK is __stdcall. That is
// exactly what `extern "system"` means -- stdcall on i386, the C convention everywhere
// else -- so ONE definition covers all three arches. It arrived as a
// `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` twin
// pair with two byte-identical bodies, which is the documented cfg-twin anti-pattern:
// redundant on i386 and WRONG off it, because the handler table's `pfnCommandHandler`
// is `extern "system"` and Rust types `extern "C"` as a DIFFERENT type even where the
// ABI coincides (E0308 on amd64/arm64).
unsafe extern "system" fn DDE_OnFindFolder(
    strCommand: PWSTR,
    strPath: PWSTR,
    pidl: LPITEMIDLIST,
    unkS: INT,
) -> DWORD {
    // UNIMPLEMENTED;
    DDE_FNOTPROCESSED
}

// dde.cpp declares this `static DWORD CALLBACK`, and CALLBACK is __stdcall. That is
// exactly what `extern "system"` means -- stdcall on i386, the C convention everywhere
// else -- so ONE definition covers all three arches. It arrived as a
// `#[cfg(target_arch = "x86")] extern "stdcall"` / `#[cfg(not(...))] extern "C"` twin
// pair with two byte-identical bodies, which is the documented cfg-twin anti-pattern:
// redundant on i386 and WRONG off it, because the handler table's `pfnCommandHandler`
// is `extern "system"` and Rust types `extern "C"` as a DIFFERENT type even where the
// ABI coincides (E0308 on amd64/arm64).
unsafe extern "system" fn DDE_OnShellFile(
    strCommand: PWSTR,
    strPath: PWSTR,
    pidl: LPITEMIDLIST,
    unkS: INT,
) -> DWORD {
    // UNIMPLEMENTED;
    DDE_FNOTPROCESSED
}
