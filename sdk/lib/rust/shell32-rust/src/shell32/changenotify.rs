//! `changenotify.cpp`, flipped. The client half of shell change notification: it packs a
//! notification into a shared-memory "delivery ticket", hands it to the notification server
//! window living in explorer, and unpacks it again on receipt via a "handbag".
//!
//! What did NOT come across is `CChangeNotifyBroker` and the `CreateNotificationBroker` that
//! constructs it — an ATL `CWindowImpl<>` whose per-instance thunk and message map are C++
//! templates. Both stayed in `rust_changenotify_broker_stub.cpp` until 2026-09-24, when they
//! moved onto the crate's atlwin substrate (crate::atl_stub_windows) together, since the class
//! is inseparable from its factory. This flip needed NO `EXTERN_C` work:
//! `InitChangeNotifications`/`FreeChangeNotifications` are declared inside the `extern "C"`
//! block of `wine/shell32_main.h`, the exported entry points inside shlobj.h's, and the
//! remaining eight have no header declaration at all (only .spec ordinals), so no C++ caller
//! could have mangled them.
//!
//! Two functions here are HAND-WRITTEN rather than bought, because the seal envelope for each
//! is unsatisfiable rather than merely hard — see their comments and the field manual entry
//! "a sealed token the Rust translation has nowhere to put".

use core::ffi::c_void;
use crate::prelude::*;
use crate::shell32::sdk_consts::*;
// ONLY what the island genuinely lacks. `win32.rs` also declares SHAllocShared, SHBindToObject,
// SHAnsiToUnicode, PathGetDriveNumberW, LocalAlloc/LocalFree and SHUnlockShared — importing
// those from there is a TYPE-WORLD error, not a convenience: `win32.rs` is written against
// `crate::types` (`ITEMIDLIST`) while this module is island-world (`_ITEMIDLIST`). Same layout,
// two distinct Rust types, and the mismatch surfaces as E0308 at every pidl argument.
use crate::win32::wine_dbg_sprintf;
use crate::wine::pidl::SHSimpleIDListFromPathW;

extern "system" {
    /// The island has no `ILGetSize`/`ILClone`, so they are declared here in ISLAND types
    /// rather than imported from `win32.rs` in types-world — that keeps every call site free of
    /// casts that would otherwise paper over a genuine pointer-type question.
    fn ILGetSize(pidl: LPCITEMIDLIST) -> UINT;
    fn ILClone(pidl: LPCITEMIDLIST) -> LPITEMIDLIST;
}

// The ATL broker window is Rust now (crate::atl_stub_windows).
use crate::atl_stub_windows::CreateNotificationBroker;

extern "system" {
    /// Neither the island nor `win32.rs` carries this one; three sibling modules
    /// (`shell_dispatch.rs`, `control_panel_folder.rs`, `recycle_bin_cleaner.rs`) each declare
    /// it locally with exactly this signature, which is the crate's normal FFI practice — an
    /// `extern` declaration is cheap to repeat, unlike a `Drop`.
    fn SHGetSpecialFolderLocation(owner: HWND, csidl: INT, pidl: *mut LPITEMIDLIST) -> HRESULT;
}

/// `CChangeNotifyServer.h:50` `#define DWORD_ALIGNMENT(offset)`. A macro, not a function, so
/// there is no symbol to import — and it must stay a macro here too: as an out-of-line `fn` it
/// would be an optimization barrier on a hot packing path for no benefit.
macro_rules! DWORD_ALIGNMENT {
    ($offset:expr) => {
        ((($offset) + core::mem::size_of::<DWORD>() as DWORD - 1)
            / core::mem::size_of::<DWORD>() as DWORD)
            * core::mem::size_of::<DWORD>() as DWORD
    };
}

/// `LMEM_FIXED` — LocalAlloc returns a pointer, not a movable handle. winbase.h. `crate::atl`
/// has its own private copy; this is a second one rather than a shared export because the two
/// modules are otherwise unrelated and the value is a documented SDK constant, not a choice.
const LMEM_FIXED: UINT = 0x0000;

// ---- file-scope preamble (translated) ----
pub static mut SHELL32_ChangenotifyCS: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };

#[repr(C)]
pub struct ALIAS_PIDL {
    pub csidl1: INT,
    pub csidl2: INT,
    pub pidl1: LPITEMIDLIST,
    pub pidl2: LPITEMIDLIST,
    pub szPath1: [WCHAR; MAX_PATH as usize],
    pub szPath2: [WCHAR; MAX_PATH as usize],
}

pub static mut AliasPIDLs: [ALIAS_PIDL; 3] = [
    ALIAS_PIDL {
        csidl1: CSIDL_PERSONAL as INT,
        csidl2: CSIDL_PERSONAL as INT,
        pidl1: core::ptr::null_mut(),
        pidl2: core::ptr::null_mut(),
        szPath1: [0; MAX_PATH as usize],
        szPath2: [0; MAX_PATH as usize],
    },
    ALIAS_PIDL {
        csidl1: CSIDL_DESKTOP as INT,
        csidl2: CSIDL_COMMON_DESKTOPDIRECTORY as INT,
        pidl1: core::ptr::null_mut(),
        pidl2: core::ptr::null_mut(),
        szPath1: [0; MAX_PATH as usize],
        szPath2: [0; MAX_PATH as usize],
    },
    ALIAS_PIDL {
        csidl1: CSIDL_DESKTOP as INT,
        csidl2: CSIDL_DESKTOPDIRECTORY as INT,
        pidl1: core::ptr::null_mut(),
        pidl2: core::ptr::null_mut(),
        szPath1: [0; MAX_PATH as usize],
        szPath2: [0; MAX_PATH as usize],
    },
];
unsafe extern "C" fn GetNotificationServer(bCreate: BOOL) -> HWND {
    static mut s_hwndServer: HWND = core::ptr::null_mut();

    if !s_hwndServer.is_null() && IsWindow(s_hwndServer) != 0 {
        return s_hwndServer;
    }

    let hwndShell = GetShellWindow();
    if hwndShell.is_null() {
        TRACE!("GetShellWindow() returned NULL\n");
        return core::ptr::null_mut();
    }

    let hwndServer = SendMessageW(hwndShell, WM_DESKTOP_GET_CNOTIFY_SERVER as UINT, bCreate as WPARAM, 0) as HWND;
    let hwndServer = if IsWindow(hwndServer) == 0 {
        ERR!("Unable to get server window\n");
        core::ptr::null_mut()
    } else {
        hwndServer
    };

    s_hwndServer = hwndServer;
    hwndServer
}
#[no_mangle]
pub unsafe extern "C" fn InitChangeNotifications() {
    InitializeCriticalSection(core::ptr::addr_of_mut!(SHELL32_ChangenotifyCS));
}

#[no_mangle]
pub unsafe extern "C" fn FreeChangeNotifications() {
    let hwndServer = GetNotificationServer(FALSE);
    if !hwndServer.is_null() {
        SendMessageW(hwndServer, CN_UNREGISTER_PROCESS, GetCurrentProcessId() as WPARAM, 0);
    }
    DeleteCriticalSection(core::ptr::addr_of_mut!(SHELL32_ChangenotifyCS));
}
unsafe extern "C" fn Shell_ParsePrinterName(
    pszName: *const u16,
    ppidl: *mut LPITEMIDLIST,
    pBindCtx: *mut IBindCtx,
) -> i32 {
    *ppidl = core::ptr::null_mut();

    // The C++ holds pidlPrinters/pidlTemp in `CComHeapPtr<ITEMIDLIST>` and pFolder in a
    // `CComPtr<IShellFolder>`; the paid body unwound all three into explicit cleanup on every
    // exit. Kept that way rather than re-introducing the ATL wrappers, having checked each of
    // the four exits against what the C++ destructors would have run — and note the first
    // early return leaks nothing precisely because a FAILED SHGetSpecialFolderLocation leaves
    // pidlPrinters NULL. CComHeapPtr frees with CoTaskMemFree, which is what these are.
    let mut pidlPrinters: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut hr = SHGetSpecialFolderLocation(
        core::ptr::null_mut(),
        CSIDL_PRINTERS as INT,
        &mut pidlPrinters,
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        return hr;
    }

    let mut pFolder: *mut IShellFolder = core::ptr::null_mut();
    hr = SHBindToObject(
        core::ptr::null_mut(),
        pidlPrinters as *const ITEMIDLIST,
        core::ptr::addr_of!(IID_IShellFolder), core::ptr::addr_of_mut!(pFolder).cast(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        CoTaskMemFree(pidlPrinters as *mut _);
        return hr;
    }

    // `crate::shellcom` renders COM interfaces as a bare `lpVtbl`, so a C++ `p->Method(...)`
    // is spelled through the vtable with the interface pointer passed back as `this`.
    let mut pidlTemp: *mut ITEMIDLIST = core::ptr::null_mut();
    hr = ((*(*pFolder).lpVtbl).ParseDisplayName)(
        pFolder,
        core::ptr::null_mut(),
        pBindCtx,
        pszName as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(pidlTemp).cast(),
        core::ptr::null_mut(),
    );
    if FAILED_UNEXPECTEDLY!(hr) {
        ((*(*pFolder).lpVtbl).Release)(pFolder);
        CoTaskMemFree(pidlPrinters as *mut _);
        return hr;
    }

    hr = SHILCombine(pidlPrinters, pidlTemp, ppidl);
    CoTaskMemFree(pidlTemp as *mut _);
    ((*(*pFolder).lpVtbl).Release)(pFolder);
    CoTaskMemFree(pidlPrinters as *mut _);
    hr
}
unsafe extern "C" fn CreateNotificationParam(
    wEventId: LONG,
    uFlags: UINT,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
    dwOwnerPID: DWORD,
    dwTick: DWORD,
) -> HANDLE {
    let mut cbPidl1 = 0;
    let mut cbPidl2 = 0;
    let mut ibOffset1 = 0;
    let mut ibOffset2 = 0;

    if !pidl1.is_null() {
        cbPidl1 = ILGetSize(pidl1);
        ibOffset1 = DWORD_ALIGNMENT!(core::mem::size_of::<DELITICKET>() as u32);
    }
    if !pidl2.is_null() {
        cbPidl2 = ILGetSize(pidl2);
        ibOffset2 = DWORD_ALIGNMENT!(ibOffset1 + cbPidl1);
    }

    let cbSize = ibOffset2 + cbPidl2;
    let hTicket = SHAllocShared(core::ptr::null_mut(), cbSize, dwOwnerPID);
    if hTicket.is_null() {
        ERR!("Out of memory\n");
        return core::ptr::null_mut();
    }

    let pTicket = SHLockSharedEx(hTicket, dwOwnerPID, TRUE) as LPDELITICKET;
    if pTicket.is_null() {
        ERR!("SHLockSharedEx failed\n");
        SHFreeShared(hTicket, dwOwnerPID);
        return core::ptr::null_mut();
    }

    (*pTicket).dwMagic = DELITICKET_MAGIC;
    (*pTicket).wEventId = wEventId;
    (*pTicket).uFlags = uFlags;
    (*pTicket).ibOffset1 = ibOffset1;
    (*pTicket).ibOffset2 = ibOffset2;
    if !pidl1.is_null() {
        memcpy((pTicket as *mut u8).add(ibOffset1 as usize) as *mut c_void, pidl1 as *const c_void, cbPidl1 as usize);
    }
    if !pidl2.is_null() {
        memcpy((pTicket as *mut u8).add(ibOffset2 as usize) as *mut c_void, pidl2 as *const c_void, cbPidl2 as usize);
    }

    SHUnlockShared(pTicket as *mut c_void);
    hTicket
}
unsafe extern "C" fn DoGetHandbagFromTicket(hTicket: HANDLE, dwOwnerPID: DWORD) -> LPHANDBAG {
    let pTicket = SHLockSharedEx(hTicket, dwOwnerPID, FALSE) as LPDELITICKET;
    if pTicket.is_null() || (*pTicket).dwMagic != DELITICKET_MAGIC {
        ERR!("pTicket is invalid\n");
        SHUnlockShared(pTicket.cast());
        return core::ptr::null_mut();
    }

    let pHandbag = LocalAlloc(LMEM_FIXED, core::mem::size_of::<HANDBAG>() as SIZE_T) as LPHANDBAG;
    if pHandbag.is_null() {
        ERR!("Out of memory\n");
        SHUnlockShared(pTicket.cast());
        return core::ptr::null_mut();
    }

    (*pHandbag).dwMagic = HANDBAG_MAGIC;
    (*pHandbag).pTicket = pTicket;

    (*pHandbag).pidls[0] = core::ptr::null_mut();
    (*pHandbag).pidls[1] = core::ptr::null_mut();
    if (*pTicket).ibOffset1 != 0 {
        (*pHandbag).pidls[0] = (pTicket as *mut u8).add((*pTicket).ibOffset1 as usize) as LPITEMIDLIST;
    }
    if (*pTicket).ibOffset2 != 0 {
        (*pHandbag).pidls[1] = (pTicket as *mut u8).add((*pTicket).ibOffset2 as usize) as LPITEMIDLIST;
    }

    pHandbag
}
unsafe extern "C" fn CreateRegistrationParam(
    nRegID: ULONG,
    hwnd: HWND,
    wMsg: UINT,
    fSources: INT,
    fEvents: LONG,
    fRecursive: LONG,
    pidl: LPCITEMIDLIST,
    dwOwnerPID: DWORD,
    hwndBroker: HWND,
) -> HANDLE {
    let cbPidl = ILGetSize(pidl);
    let ibPidl = DWORD_ALIGNMENT!(core::mem::size_of::<REGENTRY>() as DWORD);
    let cbSize = ibPidl + cbPidl;

    let hRegEntry = SHAllocShared(core::ptr::null_mut(), cbSize, dwOwnerPID);
    if hRegEntry == core::ptr::null_mut() {
        ERR!("Out of memory\n");
        return core::ptr::null_mut();
    }
    let pRegEntry = SHLockSharedEx(hRegEntry, dwOwnerPID, TRUE) as LPREGENTRY;
    if pRegEntry == core::ptr::null_mut() {
        ERR!("SHLockSharedEx failed\n");
        SHFreeShared(hRegEntry, dwOwnerPID);
        return core::ptr::null_mut();
    }

    (*pRegEntry).dwMagic = REGENTRY_MAGIC;
    (*pRegEntry).cbSize = cbSize;
    (*pRegEntry).nRegID = nRegID;
    (*pRegEntry).hwnd = hwnd;
    (*pRegEntry).uMsg = wMsg;
    (*pRegEntry).fSources = fSources;
    (*pRegEntry).fEvents = fEvents;
    (*pRegEntry).fRecursive = fRecursive;
    (*pRegEntry).hwndBroker = hwndBroker;
    (*pRegEntry).ibPidl = 0;
    if pidl != core::ptr::null_mut() {
        (*pRegEntry).ibPidl = ibPidl;
        memcpy((pRegEntry as *mut u8).add(ibPidl as usize).cast(), pidl as *const c_void, cbPidl as usize);
    }

    SHUnlockShared(pRegEntry as *mut c_void);
    return hRegEntry;
}
unsafe extern "C" fn CreateNotificationParamAndSend(
    wEventId: LONG,
    uFlags: UINT,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
    dwTick: DWORD,
) {
    let hwndServer = GetNotificationServer(FALSE);
    if hwndServer.is_null() {
        return;
    }

    let mut pid: DWORD = 0;
    GetWindowThreadProcessId(hwndServer, &mut pid);

    let hTicket = CreateNotificationParam(wEventId, uFlags, pidl1, pidl2, pid, dwTick);
    if hTicket.is_null() {
        return;
    }

    let mut pidl1Alias: *mut ITEMIDLIST = core::ptr::null_mut();
    let mut pidl2Alias: *mut ITEMIDLIST = core::ptr::null_mut();
    if !pidl1.is_null() {
        SHELL32_AliasTranslatePidl(pidl1, &mut pidl1Alias, ALIAS_ANY);
    }
    if !pidl2.is_null() {
        SHELL32_AliasTranslatePidl(pidl2, &mut pidl2Alias, ALIAS_ANY);
    }

    let mut hTicket2: HANDLE = core::ptr::null_mut();
    if (!pidl1.is_null() && !pidl1Alias.is_null() && ILIsEqual(pidl1, pidl1Alias) == 0)
        || (!pidl2.is_null() && !pidl2Alias.is_null() && ILIsEqual(pidl2, pidl2Alias) == 0)
    {
        hTicket2 = CreateNotificationParam(wEventId, uFlags, pidl1Alias, pidl2Alias, pid, dwTick);
        if hTicket2.is_null() {
            SHFreeShared(hTicket, pid);
            CoTaskMemFree(pidl1Alias as *mut _);
            CoTaskMemFree(pidl2Alias as *mut _);
            return;
        }
    }

    if pid != GetCurrentProcessId()
        || (uFlags & (SHCNF_FLUSH | SHCNF_FLUSHNOWAIT)) == SHCNF_FLUSH
    {
        SendMessageW(hwndServer, CN_DELIVER_NOTIFICATION, hTicket as WPARAM, pid as LPARAM);
        if !hTicket2.is_null() {
            SendMessageW(hwndServer, CN_DELIVER_NOTIFICATION, hTicket2 as WPARAM, pid as LPARAM);
        }
    } else {
        SendNotifyMessageW(hwndServer, CN_DELIVER_NOTIFICATION, hTicket as WPARAM, pid as LPARAM);
        if !hTicket2.is_null() {
            SendNotifyMessageW(hwndServer, CN_DELIVER_NOTIFICATION, hTicket2 as WPARAM, pid as LPARAM);
        }
    }

    CoTaskMemFree(pidl1Alias as *mut _);
    CoTaskMemFree(pidl2Alias as *mut _);
}
unsafe extern "C" fn DoInitAliasPIDLs() {
    static mut s_bInit: BOOL = FALSE;
    if s_bInit == 0 {
        // Raw pointer rather than `&mut AliasPIDLs[i]`: a reference to a `static mut` is the
        // `static_mut_refs` lint, and this table is genuinely shared mutable process state.
        let base = core::ptr::addr_of_mut!(AliasPIDLs) as *mut ALIAS_PIDL;
        let count = (*core::ptr::addr_of!(AliasPIDLs)).len();
        let mut i: usize = 0;
        while i < count {
            let alias = base.add(i);

            SHGetSpecialFolderLocation(
                core::ptr::null_mut(),
                (*alias).csidl1,
                core::ptr::addr_of_mut!((*alias).pidl1),
            );
            SHGetPathFromIDListW((*alias).pidl1, (*alias).szPath1.as_mut_ptr());

            SHGetSpecialFolderLocation(
                core::ptr::null_mut(),
                (*alias).csidl2,
                core::ptr::addr_of_mut!((*alias).pidl2),
            );
            SHGetPathFromIDListW((*alias).pidl2, (*alias).szPath2.as_mut_ptr());

            i += 1;
        }
        s_bInit = TRUE;
    }
}

/// HAND-WRITTEN, not bought. The seal envelope for this function is unsatisfiable: `--seal-facts`
/// tokenised the `2` in the ARRAY-PARAMETER BOUND of `LPITEMIDLIST apidls[2]`, and a C array
/// parameter decays to a pointer, so the Rust signature has nowhere to put that token. Four paid
/// attempts each failed the audit for the same missing token before I read the prompt. The
/// tokeniser now skips parameter array bounds; see the field manual.
unsafe extern "C" fn DoGetAliasPIDLs(apidls: *mut LPITEMIDLIST, pidl: PCIDLIST_ABSOLUTE) -> BOOL {
    DoInitAliasPIDLs();

    // C's `apidls[0] = apidls[1] = NULL;` — a chained assignment Rust does not have.
    *apidls.add(0) = core::ptr::null_mut();
    *apidls.add(1) = core::ptr::null_mut();

    let base = core::ptr::addr_of_mut!(AliasPIDLs) as *mut ALIAS_PIDL;
    let count = (*core::ptr::addr_of!(AliasPIDLs)).len();

    let mut k: INT = 0;
    let mut i: usize = 0;
    while i < count {
        let alias = base.add(i);
        if ILIsEqual(pidl, (*alias).pidl1) != 0 {
            // csidl1 == csidl2 means "same folder, different path spelling", so the alias is
            // rebuilt from the recorded PATH rather than cloned from the pidl.
            let p = if (*alias).csidl1 == (*alias).csidl2 {
                ILCreateFromPathW((*alias).szPath2.as_ptr())
            } else {
                ILClone((*alias).pidl2)
            };
            *apidls.add(k as usize) = p;
            k += 1;
            if k >= 2 {
                break;
            }
        }
        i += 1;
    }

    (k > 0) as BOOL
}

/// `DUMPEV(x)` from the C body: contribute `"NAME "` when the bit is set and `""` when it is not.
/// In C this was a function-like macro DEFINED INSIDE the function and `##`-pasting the
/// `SHCNE_` prefix onto its argument.
macro_rules! DUMPEV {
    ($event:expr, $bit:expr, $name:literal) => {
        if ($event as DWORD & $bit as DWORD) != 0 {
            concat!($name, " \0").as_ptr() as LPCSTR
        } else {
            "\0".as_ptr() as LPCSTR
        }
    };
}

/// HAND-WRITTEN, not bought — and the reason is a sealer bug worth knowing about. `--seal-calls`
/// tokenised the `#define DUMPEV(x)` NAME as if it were a runtime callee, tokenised the `#undef`
/// name as a fact, and tokenised the `SHCNE_` half of the `SHCNE_##x` PASTE as a value. The
/// resulting envelope demanded 23 call tokens be preserved "as the called function identifier in
/// executable Rust" when none of them names a function at all, so no correct translation could
/// ever pass the audit. Four paid attempts, all rejected. The tokeniser now leaves preprocessor
/// directives and paste operands alone.
///
/// DEAD BY DESIGN: its only C call site was `TRACE("notifying event %s(%x)\n", DumpEvent(...))`,
/// and this crate's `TRACE!` discards its arguments unexpanded (lib.rs — kept as a macro on
/// purpose, since an out-of-line no-op `fn` is an ABI barrier). Translated and kept anyway so
/// the diagnostic is here, intact, the day `TRACE!` grows a body — dropping it would be a silent
/// deferral.
#[allow(dead_code)]
unsafe extern "C" fn DumpEvent(event: LONG) -> LPCSTR {
    if event == SHCNE_ALLEVENTS as LONG {
        return "SHCNE_ALLEVENTS\0".as_ptr() as LPCSTR;
    }

    wine_dbg_sprintf(
        "%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s\0".as_ptr() as *const c_char,
        DUMPEV!(event, SHCNE_RENAMEITEM, "RENAMEITEM"),
        DUMPEV!(event, SHCNE_CREATE, "CREATE"),
        DUMPEV!(event, SHCNE_DELETE, "DELETE"),
        DUMPEV!(event, SHCNE_MKDIR, "MKDIR"),
        DUMPEV!(event, SHCNE_RMDIR, "RMDIR"),
        DUMPEV!(event, SHCNE_MEDIAINSERTED, "MEDIAINSERTED"),
        DUMPEV!(event, SHCNE_MEDIAREMOVED, "MEDIAREMOVED"),
        DUMPEV!(event, SHCNE_DRIVEREMOVED, "DRIVEREMOVED"),
        DUMPEV!(event, SHCNE_DRIVEADD, "DRIVEADD"),
        DUMPEV!(event, SHCNE_NETSHARE, "NETSHARE"),
        DUMPEV!(event, SHCNE_NETUNSHARE, "NETUNSHARE"),
        DUMPEV!(event, SHCNE_ATTRIBUTES, "ATTRIBUTES"),
        DUMPEV!(event, SHCNE_UPDATEDIR, "UPDATEDIR"),
        DUMPEV!(event, SHCNE_UPDATEITEM, "UPDATEITEM"),
        DUMPEV!(event, SHCNE_SERVERDISCONNECT, "SERVERDISCONNECT"),
        DUMPEV!(event, SHCNE_UPDATEIMAGE, "UPDATEIMAGE"),
        DUMPEV!(event, SHCNE_DRIVEADDGUI, "DRIVEADDGUI"),
        DUMPEV!(event, SHCNE_RENAMEFOLDER, "RENAMEFOLDER"),
        DUMPEV!(event, SHCNE_FREESPACE, "FREESPACE"),
        DUMPEV!(event, SHCNE_EXTENDED_EVENT, "EXTENDED_EVENT"),
        DUMPEV!(event, SHCNE_ASSOCCHANGED, "ASSOCCHANGED"),
        DUMPEV!(event, SHCNE_INTERRUPT, "INTERRUPT"),
    )
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyRegister(
    hwnd: HWND,
    fSources: INT,
    wEventMask: LONG,
    uMsg: UINT,
    cItems: INT,
    lpItems: *mut SHChangeNotifyEntry,
) -> ULONG {
    let mut hwndServer: HWND;
    let mut hwndBroker: HWND = core::ptr::null_mut();
    let mut hRegEntry: HANDLE;
    let mut iItem: INT;
    let mut nRegID: ULONG = INVALID_REG_ID;
    let mut dwOwnerPID: DWORD = 0;
    let mut pRegEntry: LPREGENTRY;

    TRACE!("(%p,0x%08x,0x%08x,0x%08x,%d,%p)\n", hwnd, fSources, wEventMask, uMsg, cItems, lpItems);

    if wEventMask == 0
        || cItems <= 0
        || cItems > 0x7FFF
        || lpItems.is_null()
        || hwnd.is_null()
        || IsWindow(hwnd) == 0
    {
        return INVALID_REG_ID;
    }

    hwndServer = GetNotificationServer(TRUE);
    if hwndServer.is_null() {
        return INVALID_REG_ID;
    }

    let mut fSources = fSources;
    if (fSources & SHCNRF_RecursiveInterrupt as INT) != 0 && (fSources & SHCNRF_InterruptLevel as INT) == 0 {
        fSources &= !(SHCNRF_RecursiveInterrupt as INT);
    }

    let mut hwnd = hwnd;
    let mut uMsg = uMsg;
    if (fSources & SHCNRF_NewDelivery as INT) == 0 {
        hwndBroker = CreateNotificationBroker(hwnd, uMsg);
        hwnd = hwndBroker;
        uMsg = WM_BROKER_NOTIFICATION;
    }

    GetWindowThreadProcessId(hwndServer, &mut dwOwnerPID);

    EnterCriticalSection(core::ptr::addr_of_mut!(SHELL32_ChangenotifyCS));
    iItem = 0;
    while iItem < cItems {
        hRegEntry = CreateRegistrationParam(
            nRegID,
            hwnd,
            uMsg,
            fSources,
            wEventMask,
            (*lpItems.offset(iItem as isize)).fRecursive,
            (*lpItems.offset(iItem as isize)).pidl,
            dwOwnerPID,
            hwndBroker,
        );
        if !hRegEntry.is_null() {
            SendMessageW(hwndServer, CN_REGISTER, hRegEntry as WPARAM, dwOwnerPID as LPARAM);

            pRegEntry = SHLockSharedEx(hRegEntry, dwOwnerPID, FALSE) as LPREGENTRY;
            if !pRegEntry.is_null() {
                nRegID = (*pRegEntry).nRegID;
                SHUnlockShared(pRegEntry as *mut _);
            }

            SHFreeShared(hRegEntry, dwOwnerPID);
        }

        if nRegID == INVALID_REG_ID {
            ERR!("Delivery failed\n");

            if !hwndBroker.is_null() {
                DestroyWindow(hwndBroker);
            }
            break;
        }

        let mut apidlAlias: [LPITEMIDLIST; 2] = [core::ptr::null_mut(); 2];
        if DoGetAliasPIDLs(apidlAlias.as_mut_ptr(), (*lpItems.offset(iItem as isize)).pidl) != 0 {
            if !apidlAlias[0].is_null() {
                hRegEntry = CreateRegistrationParam(
                    nRegID,
                    hwnd,
                    uMsg,
                    fSources,
                    wEventMask,
                    (*lpItems.offset(iItem as isize)).fRecursive,
                    apidlAlias[0],
                    dwOwnerPID,
                    hwndBroker,
                );
                if !hRegEntry.is_null() {
                    SendMessageW(hwndServer, CN_REGISTER, hRegEntry as WPARAM, dwOwnerPID as LPARAM);

                    SHFreeShared(hRegEntry, dwOwnerPID);
                }
                ILFree(apidlAlias[0]);
            }

            if !apidlAlias[1].is_null() {
                hRegEntry = CreateRegistrationParam(
                    nRegID,
                    hwnd,
                    uMsg,
                    fSources,
                    wEventMask,
                    (*lpItems.offset(iItem as isize)).fRecursive,
                    apidlAlias[1],
                    dwOwnerPID,
                    hwndBroker,
                );
                if !hRegEntry.is_null() {
                    SendMessageW(hwndServer, CN_REGISTER, hRegEntry as WPARAM, dwOwnerPID as LPARAM);

                    SHFreeShared(hRegEntry, dwOwnerPID);
                }
                ILFree(apidlAlias[1]);
            }
        }

        iItem += 1;
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(SHELL32_ChangenotifyCS));

    return nRegID;
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyDeregister(hNotify: ULONG) -> BOOL {
    let hwndServer: HWND = GetNotificationServer(FALSE);
    if hwndServer.is_null() {
        return FALSE;
    }

    let ret: BOOL = SendMessageW(hwndServer, CN_UNREGISTER, hNotify as WPARAM, 0) as BOOL;
    if ret == 0 {
        ERR!("CN_UNREGISTER failed\n");
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyUpdateEntryList(
    unknown1: DWORD,
    unknown2: DWORD,
    unknown3: DWORD,
    unknown4: DWORD,
) -> BOOL {
    FIXME!("(0x%08x, 0x%08x, 0x%08x, 0x%08x)\n", unknown1, unknown2, unknown3, unknown4);
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeRegistrationReceive(
    lpUnknown1: LPVOID,
    dwUnknown2: DWORD,
) -> BOOL {
    TRACE!("\n");
    FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyReceiveEx(
    lEvent: LONG,
    uFlags: UINT,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
    dwTick: DWORD,
) {
    CreateNotificationParamAndSend(lEvent, uFlags, pidl1, pidl2, dwTick);
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyReceive(
    lEvent: LONG,
    uFlags: UINT,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
) {
    SHChangeNotifyReceiveEx(lEvent, uFlags, pidl1, pidl2, GetTickCount());
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifyTransmit(
    lEvent: LONG,
    uFlags: UINT,
    pidl1: LPCITEMIDLIST,
    pidl2: LPCITEMIDLIST,
    dwTick: DWORD,
) {
    SHChangeNotifyReceiveEx(lEvent, uFlags, pidl1, pidl2, dwTick);
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotify(
    wEventId: LONG,
    uFlags: UINT,
    dwItem1: LPCVOID,
    dwItem2: LPCVOID,
) {
    TRACE!("(0x%08x,0x%08x,%p,%p)\n", wEventId, uFlags, dwItem1, dwItem2);

    // C: `if (!GetNotificationServer(FALSE)) return;` — `!` on a POINTER. The paid body kept the
    // `!` verbatim, which in Rust is bitwise-NOT and does not even apply to a raw pointer.
    if GetNotificationServer(FALSE).is_null() {
        return;
    }

    let dwTick = GetTickCount();

    let mut szPath1: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut szPath2: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut pidl1: LPITEMIDLIST = core::ptr::null_mut();
    let mut pidl2: LPITEMIDLIST = core::ptr::null_mut();
    let mut psz1: LPWSTR;
    let mut psz2: LPWSTR;
    let mut dwidl: SHChangeDWORDAsIDList = core::mem::zeroed();
    let mut dwType: DWORD;

    let mut uFlags = uFlags;
    let mut dwItem1 = dwItem1;
    let mut dwItem2 = dwItem2;

    'Retry: loop {
        dwType = uFlags & SHCNF_TYPE;
        match dwType {
            SHCNF_IDLIST => {
                if wEventId == SHCNE_FREESPACE as LONG {
                    szPath1[0] = UNICODE_NULL;
                    szPath2[0] = UNICODE_NULL;
                    if !dwItem1.is_null() {
                        SHGetPathFromIDListW(dwItem1 as LPCITEMIDLIST, szPath1.as_mut_ptr());
                    }
                    if !dwItem2.is_null() {
                        SHGetPathFromIDListW(dwItem2 as LPCITEMIDLIST, szPath2.as_mut_ptr());
                    }

                    uFlags = SHCNF_PATHW;
                    dwItem1 = if szPath1[0] != 0 {
                        szPath1.as_ptr() as LPCVOID
                    } else {
                        core::ptr::null()
                    };
                    dwItem2 = if szPath2[0] != 0 {
                        szPath2.as_ptr() as LPCVOID
                    } else {
                        core::ptr::null()
                    };
                    continue 'Retry;
                }

                pidl1 = dwItem1 as LPITEMIDLIST;
                pidl2 = dwItem2 as LPITEMIDLIST;
            }
            SHCNF_PATHA | SHCNF_PRINTERA => {
                psz1 = core::ptr::null_mut();
                psz2 = core::ptr::null_mut();
                szPath1[0] = UNICODE_NULL;
                szPath2[0] = UNICODE_NULL;
                if !dwItem1.is_null() {
                    SHAnsiToUnicode(dwItem1 as LPCSTR, szPath1.as_mut_ptr(), _countof!(szPath1) as INT);
                    psz1 = szPath1.as_mut_ptr();
                }
                if !dwItem2.is_null() {
                    SHAnsiToUnicode(dwItem2 as LPCSTR, szPath2.as_mut_ptr(), _countof!(szPath2) as INT);
                    psz2 = szPath2.as_mut_ptr();
                }

                uFlags = if dwType == SHCNF_PRINTERA {
                    SHCNF_PRINTERW
                } else {
                    SHCNF_PATHW
                };
                dwItem1 = psz1 as LPCVOID;
                dwItem2 = psz2 as LPCVOID;
                continue 'Retry;
            }
            SHCNF_PATHW => {
                if wEventId == SHCNE_FREESPACE as LONG {
                    let mut iDrive1 = -(1 as INT);
                    let mut iDrive2 = -(1 as INT);
                    if !dwItem1.is_null() {
                        iDrive1 = PathGetDriveNumberW(dwItem1 as LPCWSTR);
                    }
                    if !dwItem2.is_null() {
                        iDrive2 = PathGetDriveNumberW(dwItem2 as LPCWSTR);
                    }

                    let mut dwValue = 0;
                    if iDrive1 >= 0 as INT {
                        dwValue |= 1 << iDrive1;
                    }
                    if iDrive2 >= 0 as INT {
                        dwValue |= 1 << iDrive2;
                    }

                    if dwValue == 0 {
                        return;
                    }

                    uFlags = SHCNF_DWORD;
                    // winnt.h `UlongToPtr` — zero-extend, never sign-extend. Going through
                    // `usize` is what keeps the drive bitmask from becoming 0xFFFFFFFF_80000000
                    // on 64-bit when bit 31 (drive Z) is set.
                    dwItem1 = dwValue as usize as LPCVOID;
                    dwItem2 = core::ptr::null();
                    continue 'Retry;
                }

                if !dwItem1.is_null() {
                    pidl1 = SHSimpleIDListFromPathW(dwItem1 as LPCWSTR).cast();
                }
                if !dwItem2.is_null() {
                    pidl2 = SHSimpleIDListFromPathW(dwItem2 as LPCWSTR).cast();
                }
            }
            SHCNF_PRINTERW => {
                if !dwItem1.is_null() {
                    let hr = Shell_ParsePrinterName(
                        dwItem1 as LPCWSTR,
                        &mut pidl1,
                        core::ptr::null_mut(),
                    );
                    if FAILED_UNEXPECTEDLY!(hr) {
                        return;
                    }
                }

                if !dwItem2.is_null() {
                    let hr = Shell_ParsePrinterName(
                        dwItem2 as LPCWSTR,
                        &mut pidl2,
                        core::ptr::null_mut(),
                    );
                    if FAILED_UNEXPECTEDLY!(hr) {
                        ILFree(pidl1);
                        return;
                    }
                }
            }
            SHCNF_DWORD => {
                dwidl.cb = core::mem::offset_of!(SHChangeDWORDAsIDList, cbZero) as u16;
                // winnt.h `PtrToUlong` — deliberate truncation to 32 bits, both directions of
                // this pair being the documented SHCNF_DWORD packing.
                dwidl.dwItem1 = dwItem1 as usize as ULONG;
                dwidl.dwItem2 = dwItem2 as usize as ULONG;
                dwidl.cbZero = 0;
                pidl1 = &mut dwidl as *mut _ as LPITEMIDLIST;
            }
            _ => {
                FIXME!("Unknown type: 0x%X\n", dwType);
                return;
            }
        }
        break;
    }

    if !pidl1.is_null() || wEventId == 0 || (wEventId & SHCNE_ASSOCCHANGED as LONG) != 0 {
        // DumpEvent() renders the event bitmask for this line; see its definition for why the
        // call stays even though TRACE! currently discards its arguments.
        TRACE!("notifying event %s(%x)\n", DumpEvent(wEventId), wEventId);
        SHChangeNotifyTransmit(wEventId, uFlags, pidl1, pidl2, dwTick);
    }

    if dwType == SHCNF_PATHW || dwType == SHCNF_PRINTERW {
        ILFree(pidl1);
        ILFree(pidl2);
    }
}

#[no_mangle]
pub unsafe extern "system" fn NTSHChangeNotifyRegister(
    hwnd: HWND,
    fSources: INT,
    fEvents: LONG,
    msg: UINT,
    count: INT,
    idlist: *mut SHChangeNotifyEntry,
) -> ULONG {
    return SHChangeNotifyRegister(
        hwnd,
        fSources | SHCNRF_NewDelivery as INT,
        fEvents,
        msg,
        count,
        idlist,
    );
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotification_Lock(
    hTicket: HANDLE,
    dwOwnerPID: DWORD,
    lppidls: *mut *mut LPITEMIDLIST,
    lpwEventId: *mut LONG,
) -> HANDLE {
    TRACE!("%p %08x %p %p\n", hTicket, dwOwnerPID, lppidls, lpwEventId);

    let pHandbag = DoGetHandbagFromTicket(hTicket, dwOwnerPID);
    if pHandbag.is_null() || (*pHandbag).dwMagic != HANDBAG_MAGIC {
        ERR!("pHandbag is invalid\n");
        return core::ptr::null_mut();
    }

    if !lppidls.is_null() {
        // C's `*lppidls = pHandbag->pidls;` is array-to-pointer decay; Rust needs it spelled.
        *lppidls = (*pHandbag).pidls.as_mut_ptr();
    }
    if !lpwEventId.is_null() {
        *lpwEventId = ((*(*pHandbag).pTicket).wEventId & !(SHCNE_INTERRUPT as LONG)) as LONG;
    }

    pHandbag as HANDLE
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotification_Unlock(hLock: HANDLE) -> BOOL {
    TRACE!("%p\n", hLock);

    let pHandbag = hLock as LPHANDBAG;
    if pHandbag.is_null() || (*pHandbag).dwMagic != HANDBAG_MAGIC {
        ERR!("pHandbag is invalid\n");
        return FALSE;
    }

    let ret = SHUnlockShared((*pHandbag).pTicket.cast());
    LocalFree(hLock);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn NTSHChangeNotifyDeregister(hNotify: ULONG) -> DWORD {
    FIXME!("(0x%08x):semi stub.\n", hNotify);
    // C widens the BOOL result to DWORD implicitly; this entry point returns DWORD, not BOOL.
    SHChangeNotifyDeregister(hNotify) as DWORD
}

#[no_mangle]
pub unsafe extern "system" fn SHChangeNotifySuspendResume(
    bSuspend: BOOL,
    pidl: LPITEMIDLIST,
    bRecursive: BOOL,
    dwReserved: DWORD,
) -> BOOL {
    FIXME!("SHChangeNotifySuspendResume() stub\n");
    FALSE
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     Shell change notification
//  * COPYRIGHT:   Copyright 2020 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shcn);
//
// CRITICAL_SECTION SHELL32_ChangenotifyCS;
//
// // This function requests creation of the server window if it doesn't exist yet
// static HWND
// GetNotificationServer(BOOL bCreate)
// {
//     static HWND s_hwndServer = NULL;
//
//     // use cache if any
//     if (s_hwndServer && IsWindow(s_hwndServer))
//         return s_hwndServer;
//
//     // get the shell window
//     HWND hwndShell = GetShellWindow();
//     if (hwndShell == NULL)
//     {
//         TRACE("GetShellWindow() returned NULL\n");
//         return NULL;
//     }
//
//     // Get the window of the notification server that runs in explorer
//     HWND hwndServer = (HWND)SendMessageW(hwndShell, WM_DESKTOP_GET_CNOTIFY_SERVER as UINT, bCreate as WPARAM, 0);
//     if (!IsWindow(hwndServer))
//     {
//         ERR("Unable to get server window\n");
//         hwndServer = NULL;
//     }
//
//     // save and return
//     s_hwndServer = hwndServer;
//     return hwndServer;
// }
//
// // This function will be called from DllMain.DLL_PROCESS_ATTACH.
// extern "C" void InitChangeNotifications(void)
// {
//     InitializeCriticalSection(&SHELL32_ChangenotifyCS);
// }
//
// // This function will be called from DllMain.DLL_PROCESS_DETACH.
// extern "C" void FreeChangeNotifications(void)
// {
//     HWND hwndServer = GetNotificationServer(FALSE);
//     if (hwndServer)
//         SendMessageW(hwndServer, CN_UNREGISTER_PROCESS, GetCurrentProcessId(), 0);
//     DeleteCriticalSection(&SHELL32_ChangenotifyCS);
// }
//
// static HRESULT
// Shell_ParsePrinterName(
//      LPCWSTR pszName,
//      LPITEMIDLIST *ppidl,
//      IBindCtx *pBindCtx)
// {
//     *ppidl = NULL;
//
//     CComHeapPtr<ITEMIDLIST> pidlPrinters;
//     HRESULT hr = SHGetSpecialFolderLocation(NULL, CSIDL_PRINTERS, &pidlPrinters);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComPtr<IShellFolder> pFolder;
//     hr = SHBindToObject(NULL, pidlPrinters, IID_PPV_ARG(IShellFolder, &pFolder));
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     CComHeapPtr<ITEMIDLIST> pidlTemp;
//     hr = pFolder->ParseDisplayName(NULL, pBindCtx, (LPWSTR)pszName, NULL, &pidlTemp, NULL);
//     if (FAILED_UNEXPECTEDLY(hr))
//         return hr;
//
//     return SHILCombine(pidlPrinters, pidlTemp, ppidl);
// }
//
// //////////////////////////////////////////////////////////////////////////////////////
// // There are two delivery methods: "old delivery method" and "new delivery method".
// //
// // The old delivery method creates a broker window in the caller process
// // for message trampoline. The old delivery method is slow and deprecated.
// //
// // The new delivery method is enabled by SHCNRF_NewDelivery flag.
// // With the new delivery method the server directly sends the delivery message.
//
// // This class brokers all notifications that don't have the SHCNRF_NewDelivery flag
// class CChangeNotifyBroker :
//     public CWindowImpl<CChangeNotifyBroker, CWindow, CWorkerTraits>
// {
// public:
//     CChangeNotifyBroker(HWND hwndClient, UINT uMsg) :
//         m_hwndClient(hwndClient), m_uMsg(uMsg)
//     {
//     }
//
//     // Message handlers
//     LRESULT OnBrokerNotification(UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL& bHandled)
//     {
//         return BrokerNotification((HANDLE)wParam, (DWORD)lParam);
//     }
//
//     void OnFinalMessage(HWND)
//     {
//         // The server will destroy this window.
//         // After the window gets destroyed we can delete this broker here.
//         delete this;
//     }
//
//     DECLARE_WND_CLASS_EX(L"WorkerW", 0, 0)
//
//     BEGIN_MSG_MAP(CChangeNotifyBroker)
//         MESSAGE_HANDLER(WM_BROKER_NOTIFICATION, OnBrokerNotification)
//     END_MSG_MAP()
//
// private:
//     HWND m_hwndClient;
//     UINT m_uMsg;
//
//     BOOL BrokerNotification(HANDLE hTicket, DWORD dwOwnerPID)
//     {
//         // lock the ticket
//         PIDLIST_ABSOLUTE *ppidl = NULL;
//         LONG lEvent;
//         HANDLE hLock = SHChangeNotification_Lock(hTicket, dwOwnerPID, &ppidl, &lEvent);
//         if (hLock == NULL)
//         {
//             ERR("hLock is NULL\n");
//             return FALSE;
//         }
//
//         // perform the delivery
//         TRACE("broker notifying: %p, 0x%x, %p, 0x%lx\n",
//               m_hwndClient, m_uMsg, ppidl, lEvent);
//         SendMessageW(m_hwndClient, m_uMsg, (WPARAM)ppidl, lEvent);
//
//         // unlock the ticket
//         SHChangeNotification_Unlock(hLock);
//         return TRUE;
//     }
// };
//
// // This function creates a notification broker for old method. Used in SHChangeNotifyRegister.
// static HWND
// CreateNotificationBroker(HWND hwnd, UINT wMsg)
// {
//     // Create a new broker. It will be freed when the window gets destroyed
//     CChangeNotifyBroker* pBroker = new CChangeNotifyBroker(hwnd, wMsg);
//     if (pBroker == NULL)
//     {
//         ERR("Out of memory\n");
//         return NULL;
//     }
//
//     HWND hwndBroker = SHCreateDefaultWorkerWindow();
//     if (hwndBroker == NULL)
//     {
//         ERR("hwndBroker == NULL\n");
//         delete pBroker;
//         return NULL;
//     }
//
//     pBroker->SubclassWindow(hwndBroker);
//     return hwndBroker;
// }
//
// // This function creates a delivery ticket for shell change nofitication.
// // Used in SHChangeNotify.
// static HANDLE
// CreateNotificationParam(LONG wEventId, UINT uFlags, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2,
//                         DWORD dwOwnerPID, DWORD dwTick)
// {
//     // pidl1 and pidl2 have variable length. To store them into the delivery ticket,
//     // we have to consider the offsets and the sizes of pidl1 and pidl2.
//     DWORD cbPidl1 = 0, cbPidl2 = 0, ibOffset1 = 0, ibOffset2 = 0;
//     if (pidl1)
//     {
//         cbPidl1 = ILGetSize(pidl1);
//         ibOffset1 = DWORD_ALIGNMENT(sizeof(DELITICKET));
//     }
//     if (pidl2)
//     {
//         cbPidl2 = ILGetSize(pidl2);
//         ibOffset2 = DWORD_ALIGNMENT(ibOffset1 + cbPidl1);
//     }
//
//     // allocate the delivery ticket
//     DWORD cbSize = ibOffset2 + cbPidl2;
//     HANDLE hTicket = SHAllocShared(NULL, cbSize, dwOwnerPID);
//     if (hTicket == NULL)
//     {
//         ERR("Out of memory\n");
//         return NULL;
//     }
//
//     // lock the ticket
//     LPDELITICKET pTicket = (LPDELITICKET)SHLockSharedEx(hTicket, dwOwnerPID, TRUE);
//     if (pTicket == NULL)
//     {
//         ERR("SHLockSharedEx failed\n");
//         SHFreeShared(hTicket, dwOwnerPID);
//         return NULL;
//     }
//
//     // populate the ticket
//     pTicket->dwMagic  = DELITICKET_MAGIC;
//     pTicket->wEventId = wEventId;
//     pTicket->uFlags = uFlags;
//     pTicket->ibOffset1 = ibOffset1;
//     pTicket->ibOffset2 = ibOffset2;
//     if (pidl1)
//         memcpy((LPBYTE)pTicket + ibOffset1, pidl1, cbPidl1);
//     if (pidl2)
//         memcpy((LPBYTE)pTicket + ibOffset2, pidl2, cbPidl2);
//
//     // unlock the ticket and return
//     SHUnlockShared(pTicket);
//     return hTicket;
// }
//
// // This function creates a "handbag" by using a delivery ticket.
// // The handbag is created in SHChangeNotification_Lock and used in OnBrokerNotification.
// // hTicket is a ticket handle of a shared memory block and dwOwnerPID is
// // the owner PID of the ticket.
// static LPHANDBAG
// DoGetHandbagFromTicket(HANDLE hTicket, DWORD dwOwnerPID)
// {
//     // lock and validate the delivery ticket
//     LPDELITICKET pTicket = (LPDELITICKET)SHLockSharedEx(hTicket, dwOwnerPID, FALSE);
//     if (pTicket == NULL || pTicket->dwMagic != DELITICKET_MAGIC)
//     {
//         ERR("pTicket is invalid\n");
//         SHUnlockShared(pTicket);
//         return NULL;
//     }
//
//     // allocate the handbag
//     LPHANDBAG pHandbag = (LPHANDBAG)LocalAlloc(LMEM_FIXED, sizeof(HANDBAG));
//     if (pHandbag == NULL)
//     {
//         ERR("Out of memory\n");
//         SHUnlockShared(pTicket);
//         return NULL;
//     }
//
//     // populate the handbag
//     pHandbag->dwMagic = HANDBAG_MAGIC;
//     pHandbag->pTicket = pTicket;
//
//     pHandbag->pidls[0] = pHandbag->pidls[1] = NULL;
//     if (pTicket->ibOffset1)
//         pHandbag->pidls[0] = (LPITEMIDLIST)((LPBYTE)pTicket + pTicket->ibOffset1);
//     if (pTicket->ibOffset2)
//         pHandbag->pidls[1] = (LPITEMIDLIST)((LPBYTE)pTicket + pTicket->ibOffset2);
//
//     return pHandbag;
// }
//
// // This function creates a registration entry in SHChangeNotifyRegister function.
// static HANDLE
// CreateRegistrationParam(ULONG nRegID, HWND hwnd, UINT wMsg, INT fSources, LONG fEvents,
//                         LONG fRecursive, LPCITEMIDLIST pidl, DWORD dwOwnerPID,
//                         HWND hwndBroker)
// {
//     // pidl has variable length. To store it into the registration entry,
//     // we have to consider the length of pidl.
//     DWORD cbPidl = ILGetSize(pidl);
//     DWORD ibPidl = DWORD_ALIGNMENT(sizeof(REGENTRY));
//     DWORD cbSize = ibPidl + cbPidl;
//
//     // create the registration entry and lock it
//     HANDLE hRegEntry = SHAllocShared(NULL, cbSize, dwOwnerPID);
//     if (hRegEntry == NULL)
//     {
//         ERR("Out of memory\n");
//         return NULL;
//     }
//     LPREGENTRY pRegEntry = (LPREGENTRY)SHLockSharedEx(hRegEntry, dwOwnerPID, TRUE);
//     if (pRegEntry == NULL)
//     {
//         ERR("SHLockSharedEx failed\n");
//         SHFreeShared(hRegEntry, dwOwnerPID);
//         return NULL;
//     }
//
//     // populate the registration entry
//     pRegEntry->dwMagic = REGENTRY_MAGIC;
//     pRegEntry->cbSize = cbSize;
//     pRegEntry->nRegID = nRegID;
//     pRegEntry->hwnd = hwnd;
//     pRegEntry->uMsg = wMsg;
//     pRegEntry->fSources = fSources;
//     pRegEntry->fEvents = fEvents;
//     pRegEntry->fRecursive = fRecursive;
//     pRegEntry->hwndBroker = hwndBroker;
//     pRegEntry->ibPidl = 0;
//     if (pidl)
//     {
//         pRegEntry->ibPidl = ibPidl;
//         memcpy((LPBYTE)pRegEntry + ibPidl, pidl, cbPidl);
//     }
//
//     // unlock and return
//     SHUnlockShared(pRegEntry);
//     return hRegEntry;
// }
//
// // This function is the body of SHChangeNotify function.
// // It creates a delivery ticket and send CN_DELIVER_NOTIFICATION message to
// // transport the change.
// static void
// CreateNotificationParamAndSend(LONG wEventId, UINT uFlags, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2,
//                                DWORD dwTick)
// {
//     // get server window
//     HWND hwndServer = GetNotificationServer(FALSE);
//     if (hwndServer == NULL)
//         return;
//
//     // the ticket owner is the process of the notification server
//     DWORD pid;
//     GetWindowThreadProcessId(hwndServer, &pid);
//
//     // create a delivery ticket
//     HANDLE hTicket = CreateNotificationParam(wEventId, uFlags, pidl1, pidl2, pid, dwTick);
//     if (hTicket == NULL)
//         return;
//
//     // Create alias PIDLs
//     CComHeapPtr<ITEMIDLIST> pidl1Alias, pidl2Alias;
//     if (pidl1)
//         SHELL32_AliasTranslatePidl(pidl1, &pidl1Alias, ALIAS_ANY);
//     if (pidl2)
//         SHELL32_AliasTranslatePidl(pidl2, &pidl2Alias, ALIAS_ANY);
//
//     HANDLE hTicket2 = NULL;
//     if (((pidl1 && pidl1Alias && !ILIsEqual(pidl1, pidl1Alias)) ||
//          (pidl2 && pidl2Alias && !ILIsEqual(pidl2, pidl2Alias))))
//     {
//         hTicket2 = CreateNotificationParam(wEventId, uFlags, pidl1Alias, pidl2Alias,
//                                            pid, dwTick);
//         if (!hTicket2)
//         {
//             SHFreeShared(hTicket, pid);
//             return;
//         }
//     }
//
//     TRACE("hTicket:%p, hTicket2:%p, pid:0x%lx\n", hTicket, hTicket2, pid);
//
//     // send the ticket by using CN_DELIVER_NOTIFICATION
//     if (pid != GetCurrentProcessId() ||
//         (uFlags & (SHCNF_FLUSH | SHCNF_FLUSHNOWAIT)) == SHCNF_FLUSH)
//     {
//         SendMessageW(hwndServer, CN_DELIVER_NOTIFICATION, (WPARAM)hTicket, pid);
//         if (hTicket2)
//             SendMessageW(hwndServer, CN_DELIVER_NOTIFICATION, (WPARAM)hTicket2, pid);
//     }
//     else
//     {
//         SendNotifyMessageW(hwndServer, CN_DELIVER_NOTIFICATION, (WPARAM)hTicket, pid);
//         if (hTicket2)
//             SendNotifyMessageW(hwndServer, CN_DELIVER_NOTIFICATION, (WPARAM)hTicket2, pid);
//     }
// }
//
// struct ALIAS_PIDL
// {
//     INT csidl1; // from
//     INT csidl2; // to
//     LPITEMIDLIST pidl1; // from
//     LPITEMIDLIST pidl2; // to
//     WCHAR szPath1[MAX_PATH]; // from
//     WCHAR szPath2[MAX_PATH]; // to
// };
//
// static ALIAS_PIDL AliasPIDLs[] =
// {
//     { CSIDL_PERSONAL, CSIDL_PERSONAL },
//     { CSIDL_DESKTOP, CSIDL_COMMON_DESKTOPDIRECTORY, },
//     { CSIDL_DESKTOP, CSIDL_DESKTOPDIRECTORY },
// };
//
// static VOID DoInitAliasPIDLs(void)
// {
//     static BOOL s_bInit = FALSE;
//     if (!s_bInit)
//     {
//         for (SIZE_T i = 0; i < _countof(AliasPIDLs); ++i)
//         {
//             ALIAS_PIDL *alias = &AliasPIDLs[i];
//
//             SHGetSpecialFolderLocation(NULL, alias->csidl1, &alias->pidl1);
//             SHGetPathFromIDListW(alias->pidl1, alias->szPath1);
//
//             SHGetSpecialFolderLocation(NULL, alias->csidl2, &alias->pidl2);
//             SHGetPathFromIDListW(alias->pidl2, alias->szPath2);
//         }
//         s_bInit = TRUE;
//     }
// }
//
// static BOOL DoGetAliasPIDLs(LPITEMIDLIST apidls[2], PCIDLIST_ABSOLUTE pidl)
// {
//     DoInitAliasPIDLs();
//
//     apidls[0] = apidls[1] = NULL;
//
//     INT k = 0;
//     for (SIZE_T i = 0; i < _countof(AliasPIDLs); ++i)
//     {
//         const ALIAS_PIDL *alias = &AliasPIDLs[i];
//         if (ILIsEqual(pidl, alias->pidl1))
//         {
//             if (alias->csidl1 == alias->csidl2)
//             {
//                 apidls[k++] = ILCreateFromPathW(alias->szPath2);
//             }
//             else
//             {
//                 apidls[k++] = ILClone(alias->pidl2);
//             }
//             if (k >= 2)
//                 break;
//         }
//     }
//
//     return k > 0;
// }
//
// /*************************************************************************
//  * SHChangeNotifyRegister           [SHELL32.2]
//  */
// extern "C" ULONG WINAPI
// SHChangeNotifyRegister(HWND hwnd, INT fSources, LONG wEventMask, UINT uMsg,
//                        INT cItems, SHChangeNotifyEntry *lpItems)
// {
//     HWND hwndServer, hwndBroker = NULL;
//     HANDLE hRegEntry;
//     INT iItem;
//     ULONG nRegID = INVALID_REG_ID;
//     DWORD dwOwnerPID;
//     LPREGENTRY pRegEntry;
//
//     TRACE("(%p,0x%08x,0x%08x,0x%08x,%d,%p)\n",
//           hwnd, fSources, wEventMask, uMsg, cItems, lpItems);
//
//     // sanity check
//     if (wEventMask == 0 || cItems <= 0 || cItems > 0x7FFF || lpItems == NULL ||
//         hwnd == NULL || !IsWindow(hwnd))
//     {
//         return INVALID_REG_ID;
//     }
//
//     // request the window of the server
//     hwndServer = GetNotificationServer(TRUE);
//     if (hwndServer == NULL)
//         return INVALID_REG_ID;
//
//     // disable recursive interrupt in specific condition
//     if ((fSources & SHCNRF_RecursiveInterrupt) && !(fSources & SHCNRF_InterruptLevel))
//     {
//         fSources &= ~SHCNRF_RecursiveInterrupt;
//     }
//
//     // if it is old delivery method, then create a broker window
//     if ((fSources & SHCNRF_NewDelivery) == 0)
//     {
//         hwndBroker = hwnd = CreateNotificationBroker(hwnd, uMsg);
//         uMsg = WM_BROKER_NOTIFICATION;
//     }
//
//     // The owner PID is the process ID of the server
//     GetWindowThreadProcessId(hwndServer, &dwOwnerPID);
//
//     EnterCriticalSection(&SHELL32_ChangenotifyCS);
//     for (iItem = 0; iItem < cItems; ++iItem)
//     {
//         // create a registration entry
//         hRegEntry = CreateRegistrationParam(nRegID, hwnd, uMsg, fSources, wEventMask,
//                                             lpItems[iItem].fRecursive, lpItems[iItem].pidl,
//                                             dwOwnerPID, hwndBroker);
//         if (hRegEntry)
//         {
//             TRACE("CN_REGISTER: hwnd:%p, hRegEntry:%p, pid:0x%lx\n",
//                   hwndServer, hRegEntry, dwOwnerPID);
//
//             // send CN_REGISTER to the server
//             SendMessageW(hwndServer, CN_REGISTER, (WPARAM)hRegEntry, dwOwnerPID);
//
//             // update nRegID
//             pRegEntry = (LPREGENTRY)SHLockSharedEx(hRegEntry, dwOwnerPID, FALSE);
//             if (pRegEntry)
//             {
//                 nRegID = pRegEntry->nRegID;
//                 SHUnlockShared(pRegEntry);
//             }
//
//             // free registration entry
//             SHFreeShared(hRegEntry, dwOwnerPID);
//         }
//
//         if (nRegID == INVALID_REG_ID)
//         {
//             ERR("Delivery failed\n");
//
//             if (hwndBroker)
//             {
//                 // destroy the broker
//                 DestroyWindow(hwndBroker);
//             }
//             break;
//         }
//
//         // PIDL alias
//         LPITEMIDLIST apidlAlias[2];
//         if (DoGetAliasPIDLs(apidlAlias, lpItems[iItem].pidl))
//         {
//             if (apidlAlias[0])
//             {
//                 // create another registration entry
//                 hRegEntry = CreateRegistrationParam(nRegID, hwnd, uMsg, fSources, wEventMask,
//                                                     lpItems[iItem].fRecursive, apidlAlias[0],
//                                                     dwOwnerPID, hwndBroker);
//                 if (hRegEntry)
//                 {
//                     TRACE("CN_REGISTER: hwnd:%p, hRegEntry:%p, pid:0x%lx\n",
//                           hwndServer, hRegEntry, dwOwnerPID);
//
//                     // send CN_REGISTER to the server
//                     SendMessageW(hwndServer, CN_REGISTER, (WPARAM)hRegEntry, dwOwnerPID);
//
//                     // free registration entry
//                     SHFreeShared(hRegEntry, dwOwnerPID);
//                 }
//                 ILFree(apidlAlias[0]);
//             }
//
//             if (apidlAlias[1])
//             {
//                 // create another registration entry
//                 hRegEntry = CreateRegistrationParam(nRegID, hwnd, uMsg, fSources, wEventMask,
//                                                     lpItems[iItem].fRecursive, apidlAlias[1],
//                                                     dwOwnerPID, hwndBroker);
//                 if (hRegEntry)
//                 {
//                     TRACE("CN_REGISTER: hwnd:%p, hRegEntry:%p, pid:0x%lx\n",
//                           hwndServer, hRegEntry, dwOwnerPID);
//
//                     // send CN_REGISTER to the server
//                     SendMessageW(hwndServer, CN_REGISTER, (WPARAM)hRegEntry, dwOwnerPID);
//
//                     // free registration entry
//                     SHFreeShared(hRegEntry, dwOwnerPID);
//                 }
//                 ILFree(apidlAlias[1]);
//             }
//         }
//     }
//     LeaveCriticalSection(&SHELL32_ChangenotifyCS);
//
//     return nRegID;
// }
//
// /*************************************************************************
//  * SHChangeNotifyDeregister         [SHELL32.4]
//  */
// extern "C" BOOL WINAPI
// SHChangeNotifyDeregister(ULONG hNotify)
// {
//     TRACE("(0x%08x)\n", hNotify);
//
//     // get the server window
//     HWND hwndServer = GetNotificationServer(FALSE);
//     if (hwndServer == NULL)
//         return FALSE;
//
//     // send CN_UNREGISTER message and try to unregister
//     BOOL ret = (BOOL)SendMessageW(hwndServer, CN_UNREGISTER, hNotify, 0);
//     if (!ret)
//         ERR("CN_UNREGISTER failed\n");
//
//     return ret;
// }
//
// /*************************************************************************
//  * SHChangeNotifyUpdateEntryList            [SHELL32.5]
//  */
// extern "C" BOOL WINAPI
// SHChangeNotifyUpdateEntryList(DWORD unknown1, DWORD unknown2,
//                               DWORD unknown3, DWORD unknown4)
// {
//     FIXME("(0x%08x, 0x%08x, 0x%08x, 0x%08x)\n",
//           unknown1, unknown2, unknown3, unknown4);
//     return TRUE;
// }
//
// /* for dumping events */
// static LPCSTR DumpEvent(LONG event)
// {
//     if (event == SHCNE_ALLEVENTS)
//         return "SHCNE_ALLEVENTS";
// #define DUMPEV(x)  ,( event & SHCNE_##x )? #x " " : ""
//     return wine_dbg_sprintf( "%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s%s"
//     DUMPEV(RENAMEITEM)
//     DUMPEV(CREATE)
//     DUMPEV(DELETE)
//     DUMPEV(MKDIR)
//     DUMPEV(RMDIR)
//     DUMPEV(MEDIAINSERTED)
//     DUMPEV(MEDIAREMOVED)
//     DUMPEV(DRIVEREMOVED)
//     DUMPEV(DRIVEADD)
//     DUMPEV(NETSHARE)
//     DUMPEV(NETUNSHARE)
//     DUMPEV(ATTRIBUTES)
//     DUMPEV(UPDATEDIR)
//     DUMPEV(UPDATEITEM)
//     DUMPEV(SERVERDISCONNECT)
//     DUMPEV(UPDATEIMAGE)
//     DUMPEV(DRIVEADDGUI)
//     DUMPEV(RENAMEFOLDER)
//     DUMPEV(FREESPACE)
//     DUMPEV(EXTENDED_EVENT)
//     DUMPEV(ASSOCCHANGED)
//     DUMPEV(INTERRUPT)
//     );
// #undef DUMPEV
// }
//
// /*************************************************************************
//  * SHChangeRegistrationReceive      [SHELL32.646]
//  */
// extern "C" BOOL WINAPI
// SHChangeRegistrationReceive(LPVOID lpUnknown1, DWORD dwUnknown2)
// {
//     TRACE("\n");
//     return FALSE; /* Just return FALSE */
// }
//
// extern "C" VOID WINAPI
// SHChangeNotifyReceiveEx(LONG lEvent, UINT uFlags,
//                         LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2, DWORD dwTick)
// {
//     // TODO: Queueing notifications
//     CreateNotificationParamAndSend(lEvent, uFlags, pidl1, pidl2, dwTick);
// }
//
// /*************************************************************************
//  * SHChangeNotifyReceive        [SHELL32.643]
//  */
// extern "C" VOID WINAPI
// SHChangeNotifyReceive(LONG lEvent, UINT uFlags, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2)
// {
//     SHChangeNotifyReceiveEx(lEvent, uFlags, pidl1, pidl2, GetTickCount());
// }
//
// extern "C" VOID WINAPI
// SHChangeNotifyTransmit(LONG lEvent, UINT uFlags, LPCITEMIDLIST pidl1, LPCITEMIDLIST pidl2, DWORD dwTick)
// {
//     SHChangeNotifyReceiveEx(lEvent, uFlags, pidl1, pidl2, dwTick);
// }
//
// /*************************************************************************
//  * SHChangeNotify               [SHELL32.@]
//  */
// extern "C" void WINAPI
// SHChangeNotify(LONG wEventId, UINT uFlags, LPCVOID dwItem1, LPCVOID dwItem2)
// {
//     TRACE("(0x%08x,0x%08x,%p,%p)\n", wEventId, uFlags, dwItem1, dwItem2);
//
//     if (!GetNotificationServer(FALSE))
//         return;
//
//     DWORD dwTick = GetTickCount();
//
//     WCHAR szPath1[MAX_PATH], szPath2[MAX_PATH];
//     LPITEMIDLIST pidl1 = NULL, pidl2 = NULL;
//     LPWSTR psz1, psz2;
//     SHChangeDWORDAsIDList dwidl;
//     DWORD dwType;
//
// Retry:
//     dwType = (uFlags & SHCNF_TYPE);
//     switch (dwType)
//     {
//         case SHCNF_IDLIST:
//         {
//             if (wEventId == SHCNE_FREESPACE)
//             {
//                 szPath1[0] = szPath2[0] = UNICODE_NULL;
//                 if (dwItem1)
//                     SHGetPathFromIDList((LPCITEMIDLIST)dwItem1, szPath1);
//                 if (dwItem2)
//                     SHGetPathFromIDList((LPCITEMIDLIST)dwItem2, szPath2);
//
//                 uFlags = SHCNF_PATHW;
//                 dwItem1 = (szPath1[0] ? szPath1 : NULL);
//                 dwItem2 = (szPath2[0] ? szPath2 : NULL);
//                 goto Retry;
//             }
//
//             pidl1 = (LPITEMIDLIST)dwItem1;
//             pidl2 = (LPITEMIDLIST)dwItem2;
//             break;
//         }
//         case SHCNF_PATHA:
//         case SHCNF_PRINTERA:
//         {
//             psz1 = psz2 = NULL;
//             szPath1[0] = szPath2[0] = UNICODE_NULL;
//             if (dwItem1)
//             {
//                 SHAnsiToUnicode((LPCSTR)dwItem1, szPath1, _countof(szPath1));
//                 psz1 = szPath1;
//             }
//             if (dwItem2)
//             {
//                 SHAnsiToUnicode((LPCSTR)dwItem2, szPath2, _countof(szPath2));
//                 psz2 = szPath2;
//             }
//
//             uFlags = ((dwType == SHCNF_PRINTERA) ? SHCNF_PRINTERW : SHCNF_PATHW);
//             dwItem1 = psz1;
//             dwItem2 = psz2;
//             goto Retry;
//         }
//         case SHCNF_PATHW:
//         {
//             if (wEventId == SHCNE_FREESPACE)
//             {
//                 INT iDrive1 = -1, iDrive2 = -1;
//                 if (dwItem1)
//                     iDrive1 = PathGetDriveNumberW((LPCWSTR)dwItem1);
//                 if (dwItem2)
//                     iDrive2 = PathGetDriveNumberW((LPCWSTR)dwItem2);
//
//                 DWORD dwValue = 0;
//                 if (iDrive1 >= 0)
//                     dwValue |= (1 << iDrive1);
//                 if (iDrive2 >= 0)
//                     dwValue |= (1 << iDrive2);
//
//                 if (!dwValue)
//                     return;
//
//                 uFlags = SHCNF_DWORD;
//                 dwItem1 = UlongToPtr(dwValue);
//                 dwItem2 = NULL;
//                 goto Retry;
//             }
//
//             if (dwItem1)
//                 pidl1 = SHSimpleIDListFromPathW((LPCWSTR)dwItem1);
//             if (dwItem2)
//                 pidl2 = SHSimpleIDListFromPathW((LPCWSTR)dwItem2);
//             break;
//         }
//         case SHCNF_PRINTERW:
//         {
//             if (dwItem1)
//             {
//                 HRESULT hr = Shell_ParsePrinterName((LPCWSTR)dwItem1, &pidl1, NULL);
//                 if (FAILED_UNEXPECTEDLY(hr))
//                     return;
//             }
//
//             if (dwItem2)
//             {
//                 HRESULT hr = Shell_ParsePrinterName((LPCWSTR)dwItem2, &pidl2, NULL);
//                 if (FAILED_UNEXPECTEDLY(hr))
//                 {
//                     ILFree(pidl1);
//                     return;
//                 }
//             }
//             break;
//         }
//         case SHCNF_DWORD:
//         {
//             dwidl.cb = offsetof(SHChangeDWORDAsIDList, cbZero);
//             dwidl.dwItem1 = PtrToUlong(dwItem1);
//             dwidl.dwItem2 = PtrToUlong(dwItem2);
//             dwidl.cbZero = 0;
//             pidl1 = (LPITEMIDLIST)&dwidl;
//             break;
//         }
//         default:
//         {
//             FIXME("Unknown type: 0x%X\n", dwType);
//             return;
//         }
//     }
//
//     if (pidl1 || !wEventId || (wEventId & SHCNE_ASSOCCHANGED))
//     {
//         TRACE("notifying event %s(%x)\n", DumpEvent(wEventId), wEventId);
//         SHChangeNotifyTransmit(wEventId, uFlags, pidl1, pidl2, dwTick);
//     }
//
//     if ((dwType == SHCNF_PATHW) || (dwType == SHCNF_PRINTERW))
//     {
//         ILFree(pidl1);
//         ILFree(pidl2);
//     }
// }
//
// /*************************************************************************
//  * NTSHChangeNotifyRegister            [SHELL32.640]
//  */
// extern "C" ULONG WINAPI
// NTSHChangeNotifyRegister(HWND hwnd, INT fSources, LONG fEvents, UINT msg,
//                          INT count, SHChangeNotifyEntry *idlist)
// {
//     return SHChangeNotifyRegister(hwnd, fSources | SHCNRF_NewDelivery,
//                                   fEvents, msg, count, idlist);
// }
//
// /*************************************************************************
//  * SHChangeNotification_Lock            [SHELL32.644]
//  */
// extern "C" HANDLE WINAPI
// SHChangeNotification_Lock(HANDLE hTicket, DWORD dwOwnerPID, LPITEMIDLIST **lppidls,
//                           LPLONG lpwEventId)
// {
//     TRACE("%p %08x %p %p\n", hTicket, dwOwnerPID, lppidls, lpwEventId);
//
//     // create a handbag from the ticket
//     LPHANDBAG pHandbag = DoGetHandbagFromTicket(hTicket, dwOwnerPID);
//     if (pHandbag == NULL || pHandbag->dwMagic != HANDBAG_MAGIC)
//     {
//         ERR("pHandbag is invalid\n");
//         return NULL;
//     }
//
//     // populate parameters from the handbag
//     if (lppidls)
//         *lppidls = pHandbag->pidls;
//     if (lpwEventId)
//         *lpwEventId = (pHandbag->pTicket->wEventId & ~SHCNE_INTERRUPT);
//
//     // return the handbag
//     return pHandbag;
// }
//
// /*************************************************************************
//  * SHChangeNotification_Unlock          [SHELL32.645]
//  */
// extern "C" BOOL WINAPI
// SHChangeNotification_Unlock(HANDLE hLock)
// {
//     TRACE("%p\n", hLock);
//
//     // validate the handbag
//     LPHANDBAG pHandbag = (LPHANDBAG)hLock;
//     if (pHandbag == NULL || pHandbag->dwMagic != HANDBAG_MAGIC)
//     {
//         ERR("pHandbag is invalid\n");
//         return FALSE;
//     }
//
//     // free the handbag
//     BOOL ret = SHUnlockShared(pHandbag->pTicket);
//     LocalFree(hLock);
//     return ret;
// }
//
// /*************************************************************************
//  * NTSHChangeNotifyDeregister           [SHELL32.641]
//  */
// extern "C" DWORD WINAPI
// NTSHChangeNotifyDeregister(ULONG hNotify)
// {
//     FIXME("(0x%08x):semi stub.\n", hNotify);
//     return SHChangeNotifyDeregister(hNotify);
// }
//
// /*************************************************************************
//  * SHChangeNotifySuspendResume          [SHELL32.277]
//  */
// extern "C" BOOL
// WINAPI
// SHChangeNotifySuspendResume(BOOL bSuspend,
//                             LPITEMIDLIST pidl,
//                             BOOL bRecursive,
//                             DWORD dwReserved)
// {
//     FIXME("SHChangeNotifySuspendResume() stub\n");
//     return FALSE;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
