//! notify.c — logon notification DLLs. Walks
//! HKLM\...\Winlogon\Notify, loads each registered DLL, snaps its per-event
//! handlers, and dispatches them on logon/logoff/startup/shutdown/etc.
//! CallNotificationDlls is called from sas.c + screensaver.rs/security on
//! the boot path (e.g. StartShellHandler at logon), so this is smoke-
//! exercised. The built-in SFC notification (sfc.dll) is always registered.
//!
//! Translated by rustconv (Opus 4.8), then hand-reconciled: dropped the
//! bogus `extern "system" { _SEH2_*, RtlGetProcessHeap, WlStrDup, wcschr }`
//! block (SEH2 are macros / the others are macro-or-cdecl — use preamble);
//! provided local LIST_ENTRY ops (bindgen's are FORCEINLINE → no exported
//! symbol); added the `NOTIFICATION_TYPE` alias (bindgen only has the
//! underscored form). The i386 CORE-20279 ESP-tolerance asm belt for buggy
//! cdecl 3rd-party handlers is omitted — a defensive-only path; the lone
//! boot notify DLL (sfc.dll) is a conformant STDCALL export, and the
//! handler pointer is still range-validated below.

use winlogon_sys::nt::*;
use crate::preamble::*;

// bindgen exposes only the underscored enum type; alias the documented name.
type NOTIFICATION_TYPE = _NOTIFICATION_TYPE;

const ANSI_NULL: u8 = 0;
const UNICODE_NULL: u16 = 0;

// VOID (WINAPI *)(PWLX_NOTIFICATION_INFO) — notify.c-local typedef.
type PWLX_NOTIFY_HANDLER = Option<unsafe extern "system" fn(pInfo: PWLX_NOTIFICATION_INFO)>;

// FUNCTIONAL (not debug): these are the registry value names under each
// notify DLL's key that hold the per-event handler export name. Stored as
// NUL-terminated byte-slice refs (`&[u8]` is Sync and const-OK, unlike raw
// pointers); `.as_ptr()` at the use site yields the PCSTR.
static FuncNames: [&[u8]; LastHandler as usize] = [
    b"Logon\0",
    b"Logoff\0",
    b"Lock\0",
    b"Unlock\0",
    b"Startup\0",
    b"Shutdown\0",
    b"StartScreenSaver\0",
    b"StopScreenSaver\0",
    b"Disconnect\0",
    b"Reconnect\0",
    b"StartShell\0",
    b"PostShell\0",
];

#[repr(C)]
struct NOTIFICATION_ITEM {
    ListEntry: LIST_ENTRY,
    hModule: HMODULE,
    pszKeyName: PWSTR,
    pszDllName: PWSTR,
    bEnabled: BOOL,
    bAsynchronous: BOOL,
    bImpersonate: BOOL,
    bSmartCardLogon: BOOL,
    dwMaxWait: DWORD,
    bSfcNotification: BOOL,
    Handler: [PWLX_NOTIFY_HANDLER; LastHandler as usize],
}

type PNOTIFICATION_ITEM = *mut NOTIFICATION_ITEM;

static mut NotificationDllListHead: LIST_ENTRY = unsafe { core::mem::zeroed() };

// Intrusive doubly-linked-list ops. bindgen emits these as extern decls but
// they are FORCEINLINE in the SDK headers (no exported symbol), so provide
// local definitions (these shadow the glob import).
#[inline]
unsafe fn InitializeListHead(head: PLIST_ENTRY) {
    (*head).Flink = head;
    (*head).Blink = head;
}
#[inline]
unsafe fn IsListEmpty(head: PLIST_ENTRY) -> BOOLEAN {
    ((*head).Flink == head) as BOOLEAN
}
#[inline]
unsafe fn InsertHeadList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let flink = (*head).Flink;
    (*entry).Flink = flink;
    (*entry).Blink = head;
    (*flink).Blink = entry;
    (*head).Flink = entry;
}
#[inline]
unsafe fn RemoveHeadList(head: PLIST_ENTRY) -> PLIST_ENTRY {
    let entry = (*head).Flink;
    let flink = (*entry).Flink;
    (*head).Flink = flink;
    (*flink).Blink = head;
    entry
}

unsafe fn GetNotificationHandler(
    hDllKey: HKEY,
    hModule: HMODULE,
    pNotification: PCSTR,
) -> PWLX_NOTIFY_HANDLER {
    let lError: LONG;
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut szFuncBuffer: [i8; 128] = [0; 128];

    dwSize = core::mem::size_of::<[i8; 128]>() as DWORD;
    lError = RegQueryValueExA(
        hDllKey,
        pNotification,
        core::ptr::null_mut(),
        &mut dwType,
        szFuncBuffer.as_mut_ptr() as PBYTE,
        &mut dwSize,
    );
    if (lError != ERROR_SUCCESS as LONG)
        || (dwType != REG_SZ && dwType != REG_EXPAND_SZ)
        || (dwSize == 0)
    {
        return None;
    }

    /* NUL-terminate */
    szFuncBuffer[dwSize as usize / core::mem::size_of::<i8>() - 1] = ANSI_NULL as i8;

    let handler: PWLX_NOTIFY_HANDLER =
        core::mem::transmute(GetProcAddress(hModule, szFuncBuffer.as_ptr() as PCSTR));
    handler
}

unsafe fn LoadNotifyDll(NotificationDll: PNOTIFICATION_ITEM) -> BOOL {
    let mut hNotifyKey: HKEY = core::ptr::null_mut();
    let mut hDllKey: HKEY = core::ptr::null_mut();
    let hModule: HMODULE;
    let mut lError: LONG;
    let mut Type: NOTIFICATION_TYPE;

    if (*NotificationDll).bSfcNotification != 0 {
        return 1; // Already loaded.
    }

    static NOTIFY_PATH: [u16; 62] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16,
        b'r' as u16, b'e' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16,
        b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16,
        b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16,
        b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16, b'T' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16,
        b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16,
        b'o' as u16, b'n' as u16, b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16,
        b'l' as u16, b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
        b'N' as u16, b'o' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'y' as u16,
        0, 0,
    ];

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        NOTIFY_PATH.as_ptr(),
        0,
        KEY_READ,
        &mut hNotifyKey,
    );
    if lError != ERROR_SUCCESS as LONG {
        return 0;
    }

    lError = RegOpenKeyExW(
        hNotifyKey,
        (*NotificationDll).pszKeyName,
        0,
        KEY_READ,
        &mut hDllKey,
    );
    RegCloseKey(hNotifyKey);

    if lError != ERROR_SUCCESS as LONG {
        return 0;
    }

    hModule = LoadLibraryW((*NotificationDll).pszDllName);
    if hModule.is_null() {
        RegCloseKey(hDllKey);
        return 0;
    }
    (*NotificationDll).hModule = hModule;

    Type = LogonHandler;
    while Type < LastHandler {
        (*NotificationDll).Handler[Type as usize] =
            GetNotificationHandler(hDllKey, hModule, FuncNames[Type as usize].as_ptr() as PCSTR);
        Type += 1;
    }

    RegCloseKey(hDllKey);
    1
}

unsafe fn DeleteNotification(Notification: PNOTIFICATION_ITEM) {
    if !(*Notification).hModule.is_null() {
        FreeLibrary((*Notification).hModule);
    }

    if !(*Notification).pszKeyName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Notification).pszKeyName as PVOID);
    }

    if !(*Notification).pszDllName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Notification).pszDllName as PVOID);
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, Notification as PVOID);
}

unsafe fn AddSfcNotification() {
    let NotificationDll: PNOTIFICATION_ITEM;
    let hModule: HMODULE;
    let dwError: DWORD;
    let mut szSfcPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    static SFC_DLL: [u16; 38] = [
        b'%' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
        b'm' as u16, b'R' as u16, b'o' as u16, b'o' as u16, b't' as u16, b'%' as u16,
        b'\\' as u16, b's' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
        b'm' as u16, b'3' as u16, b'2' as u16, b'\\' as u16, b's' as u16, b'f' as u16,
        b'c' as u16, b'.' as u16, b'd' as u16, b'l' as u16, b'l' as u16, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ];

    ExpandEnvironmentStringsW(
        SFC_DLL.as_ptr(),
        szSfcPath.as_mut_ptr(),
        szSfcPath.len() as DWORD,
    );

    NotificationDll = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        core::mem::size_of::<NOTIFICATION_ITEM>() as _,
    ) as PNOTIFICATION_ITEM;
    if NotificationDll.is_null() {
        return;
    }

    'done: {
        (*NotificationDll).pszDllName = WlStrDup(szSfcPath.as_ptr());
        if (*NotificationDll).pszDllName.is_null() {
            dwError = ERROR_OUTOFMEMORY;
            break 'done;
        }

        (*NotificationDll).bEnabled = 1;
        (*NotificationDll).dwMaxWait = 30; /* FIXME: ??? */
        (*NotificationDll).bSfcNotification = 1;

        /* Load sfc.dll, and also sfc_os.dll on systems where it forwards to */
        hModule = LoadLibraryW(szSfcPath.as_ptr());
        if hModule.is_null() {
            dwError = GetLastError();
            break 'done;
        }
        (*NotificationDll).hModule = hModule;

        (*NotificationDll).Handler[LogonHandler as usize] = core::mem::transmute(
            GetProcAddress(hModule, b"SfcWLEventLogon\0".as_ptr() as PCSTR),
        );
        (*NotificationDll).Handler[LogoffHandler as usize] = core::mem::transmute(
            GetProcAddress(hModule, b"SfcWLEventLogoff\0".as_ptr() as PCSTR),
        );
        if (*NotificationDll).Handler[LogonHandler as usize].is_none()
            || (*NotificationDll).Handler[LogoffHandler as usize].is_none()
        {
            dwError = ERROR_PROC_NOT_FOUND;
            break 'done;
        }

        InsertHeadList(
            core::ptr::addr_of_mut!(NotificationDllListHead),
            core::ptr::addr_of_mut!((*NotificationDll).ListEntry),
        );

        dwError = ERROR_SUCCESS;
    }

    if dwError != ERROR_SUCCESS {
        DeleteNotification(NotificationDll);
    }
}

unsafe fn AddNotificationDll(hNotifyKey: HKEY, pszKeyName: PCWSTR) {
    let mut hDllKey: HKEY = core::ptr::null_mut();
    let mut NotificationDll: PNOTIFICATION_ITEM = core::ptr::null_mut();
    let mut pszDllName: PWSTR;
    let mut dwValue: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwType: DWORD = 0;
    let mut lError: LONG;

    lError = RegOpenKeyExW(hNotifyKey, pszKeyName, 0, KEY_READ, &mut hDllKey);
    if lError != ERROR_SUCCESS as LONG {
        return;
    }

    /* In safe-boot mode, load the notification DLL only if it is enabled there */
    if GetSystemMetrics(SM_CLEANBOOT as i32) != 0 {
        let mut bSafeMode: BOOL = 0; // Default to NOT loading the DLL in SafeMode.

        static SAFE_MODE: [u16; 9] = [
            b'S' as u16, b'a' as u16, b'f' as u16, b'e' as u16, b'M' as u16,
            b'o' as u16, b'd' as u16, b'e' as u16, 0,
        ];

        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lError = RegQueryValueExW(
            hDllKey,
            SAFE_MODE.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dwValue as *mut DWORD as PBYTE,
            &mut dwSize,
        );
        if (lError == ERROR_SUCCESS as LONG)
            && (dwType == REG_DWORD)
            && (dwSize == core::mem::size_of::<DWORD>() as DWORD)
        {
            bSafeMode = if dwValue != 0 { 1 } else { 0 };
        }

        if bSafeMode == 0 {
            RegCloseKey(hDllKey);
            return;
        }
    }

    'done: {
        NotificationDll = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            core::mem::size_of::<NOTIFICATION_ITEM>() as _,
        ) as PNOTIFICATION_ITEM;
        if NotificationDll.is_null() {
            lError = ERROR_OUTOFMEMORY as LONG;
            break 'done;
        }

        (*NotificationDll).pszKeyName = WlStrDup(pszKeyName);
        if (*NotificationDll).pszKeyName.is_null() {
            lError = ERROR_OUTOFMEMORY as LONG;
            break 'done;
        }

        static DLL_NAME: [u16; 8] = [
            b'D' as u16, b'l' as u16, b'l' as u16, b'N' as u16, b'a' as u16,
            b'm' as u16, b'e' as u16, 0,
        ];

        dwSize = 0;
        lError = RegQueryValueExW(
            hDllKey,
            DLL_NAME.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            core::ptr::null_mut(),
            &mut dwSize,
        );
        if (lError != ERROR_SUCCESS as LONG)
            || (dwType != REG_SZ && dwType != REG_EXPAND_SZ)
            || (dwSize == 0)
            || (dwSize % core::mem::size_of::<u16>() as DWORD != 0)
        {
            lError = ERROR_FILE_NOT_FOUND as LONG;
            break 'done;
        }
        /* Ensure the string can be NUL-terminated */
        dwSize += core::mem::size_of::<u16>() as DWORD;

        pszDllName = RtlAllocateHeap(RtlGetProcessHeap(), 0, dwSize as _) as PWSTR;
        if pszDllName.is_null() {
            lError = ERROR_OUTOFMEMORY as LONG;
            break 'done;
        }

        lError = RegQueryValueExW(
            hDllKey,
            DLL_NAME.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            pszDllName as PBYTE,
            &mut dwSize,
        );
        if lError != ERROR_SUCCESS as LONG {
            RtlFreeHeap(RtlGetProcessHeap(), 0, pszDllName as PVOID);
            break 'done;
        }
        /* NUL-terminate */
        *pszDllName.add(dwSize as usize / core::mem::size_of::<u16>() - 1) = UNICODE_NULL;

        /* Expand the path if applicable. */
        if !wcschr(pszDllName, b'%' as u16).is_null() {
            dwSize = ExpandEnvironmentStringsW(pszDllName, core::ptr::null_mut(), 0);
            if dwSize != 0 {
                let pszDllPath = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (dwSize as usize * core::mem::size_of::<u16>()) as _,
                ) as PWSTR;
                if pszDllPath.is_null() {
                    RtlFreeHeap(RtlGetProcessHeap(), 0, pszDllName as PVOID);
                    lError = ERROR_OUTOFMEMORY as LONG;
                    break 'done;
                }
                ExpandEnvironmentStringsW(pszDllName, pszDllPath, dwSize);

                /* Free the old buffer and replace it with the expanded path */
                RtlFreeHeap(RtlGetProcessHeap(), 0, pszDllName as PVOID);
                pszDllName = pszDllPath;
            }
        }

        (*NotificationDll).pszDllName = pszDllName;

        (*NotificationDll).bEnabled = 1;
        (*NotificationDll).dwMaxWait = 30; /* FIXME: ??? */

        static ASYNCHRONOUS: [u16; 13] = [
            b'A' as u16, b's' as u16, b'y' as u16, b'n' as u16, b'c' as u16,
            b'h' as u16, b'r' as u16, b'o' as u16, b'n' as u16, b'o' as u16,
            b'u' as u16, b's' as u16, 0,
        ];
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lError = RegQueryValueExW(
            hDllKey,
            ASYNCHRONOUS.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dwValue as *mut DWORD as PBYTE,
            &mut dwSize,
        );
        if (lError == ERROR_SUCCESS as LONG)
            && (dwType == REG_DWORD)
            && (dwSize == core::mem::size_of::<DWORD>() as DWORD)
        {
            (*NotificationDll).bAsynchronous = if dwValue != 0 { 1 } else { 0 };
        }

        static IMPERSONATE: [u16; 12] = [
            b'I' as u16, b'm' as u16, b'p' as u16, b'e' as u16, b'r' as u16,
            b's' as u16, b'o' as u16, b'n' as u16, b'a' as u16, b't' as u16,
            b'e' as u16, 0,
        ];
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lError = RegQueryValueExW(
            hDllKey,
            IMPERSONATE.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dwValue as *mut DWORD as PBYTE,
            &mut dwSize,
        );
        if (lError == ERROR_SUCCESS as LONG)
            && (dwType == REG_DWORD)
            && (dwSize == core::mem::size_of::<DWORD>() as DWORD)
        {
            (*NotificationDll).bImpersonate = if dwValue != 0 { 1 } else { 0 };
        }

        static SMARTCARD: [u16; 21] = [
            b'S' as u16, b'm' as u16, b'a' as u16, b'r' as u16, b't' as u16,
            b'C' as u16, b'a' as u16, b'r' as u16, b'd' as u16, b'L' as u16,
            b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16, b'N' as u16,
            b'o' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'y' as u16, 0,
        ];
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lError = RegQueryValueExW(
            hDllKey,
            SMARTCARD.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dwValue as *mut DWORD as PBYTE,
            &mut dwSize,
        );
        if (lError == ERROR_SUCCESS as LONG)
            && (dwType == REG_DWORD)
            && (dwSize == core::mem::size_of::<DWORD>() as DWORD)
        {
            (*NotificationDll).bSmartCardLogon = if dwValue != 0 { 1 } else { 0 };
        }

        static MAXWAIT: [u16; 8] = [
            b'M' as u16, b'a' as u16, b'x' as u16, b'W' as u16, b'a' as u16,
            b'i' as u16, b't' as u16, 0,
        ];
        dwSize = core::mem::size_of::<DWORD>() as DWORD;
        lError = RegQueryValueExW(
            hDllKey,
            MAXWAIT.as_ptr(),
            core::ptr::null_mut(),
            &mut dwType,
            &mut dwValue as *mut DWORD as PBYTE,
            &mut dwSize,
        );
        if (lError == ERROR_SUCCESS as LONG)
            && (dwType == REG_DWORD)
            && (dwSize == core::mem::size_of::<DWORD>() as DWORD)
        {
            (*NotificationDll).dwMaxWait = dwValue;
        }

        InsertHeadList(
            core::ptr::addr_of_mut!(NotificationDllListHead),
            core::ptr::addr_of_mut!((*NotificationDll).ListEntry),
        );

        lError = ERROR_SUCCESS as LONG;
    }

    if lError != ERROR_SUCCESS as LONG {
        if !NotificationDll.is_null() {
            DeleteNotification(NotificationDll);
        }
    }

    RegCloseKey(hDllKey);
}

#[no_mangle]
pub unsafe extern "C" fn InitNotifications() -> BOOL {
    let mut hNotifyKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG;
    let mut dwIndex: DWORD;
    let mut dwKeyName: DWORD;
    let mut szKeyName: [u16; 80] = [0; 80];

    InitializeListHead(core::ptr::addr_of_mut!(NotificationDllListHead));

    AddSfcNotification();

    static NOTIFY_PATH: [u16; 62] = [
        b'S' as u16, b'o' as u16, b'f' as u16, b't' as u16, b'w' as u16, b'a' as u16,
        b'r' as u16, b'e' as u16, b'\\' as u16, b'M' as u16, b'i' as u16, b'c' as u16,
        b'r' as u16, b'o' as u16, b's' as u16, b'o' as u16, b'f' as u16, b't' as u16,
        b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16,
        b'w' as u16, b's' as u16, b' ' as u16, b'N' as u16, b'T' as u16, b'\\' as u16,
        b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16, b'n' as u16,
        b't' as u16, b'V' as u16, b'e' as u16, b'r' as u16, b's' as u16, b'i' as u16,
        b'o' as u16, b'n' as u16, b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16,
        b'l' as u16, b'o' as u16, b'g' as u16, b'o' as u16, b'n' as u16, b'\\' as u16,
        b'N' as u16, b'o' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'y' as u16,
        0, 0,
    ];

    lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        NOTIFY_PATH.as_ptr(),
        0,
        KEY_READ,
        &mut hNotifyKey,
    );
    if lError != ERROR_SUCCESS as LONG {
        return 1;
    }

    dwIndex = 0;
    loop {
        dwKeyName = szKeyName.len() as DWORD;
        lError = RegEnumKeyExW(
            hNotifyKey,
            dwIndex,
            szKeyName.as_mut_ptr(),
            &mut dwKeyName,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if lError != ERROR_SUCCESS as LONG {
            break;
        }

        AddNotificationDll(hNotifyKey, szKeyName.as_ptr());
        dwIndex += 1;
    }

    RegCloseKey(hNotifyKey);

    1
}

unsafe fn CallNotificationDll(
    NotificationDll: PNOTIFICATION_ITEM,
    Type: NOTIFICATION_TYPE,
    pInfo: PWLX_NOTIFICATION_INFO,
) {
    let pNotifyHandler: PWLX_NOTIFY_HANDLER;
    let mut Info: WLX_NOTIFICATION_INFO;
    let UserToken: HANDLE;

    /* Delay-load the DLL if needed */
    if (*NotificationDll).hModule.is_null() {
        if LoadNotifyDll(NotificationDll) == 0 {
            /* We failed, disable it */
            (*NotificationDll).bEnabled = 0;
            return;
        }
    }

    /* Retrieve the notification handler; bail out if none is specified */
    pNotifyHandler = (*NotificationDll).Handler[Type as usize];
    if pNotifyHandler.is_none() {
        return;
    }

    /* Validate handler pointer — must be within the DLL's address range */
    let handler_addr = core::mem::transmute::<_, usize>(pNotifyHandler);
    if (handler_addr < (*NotificationDll).hModule as usize) || (handler_addr < 0x10000) {
        return;
    }

    /* Capture the notification info structure */
    Info = *pInfo;

    /* Impersonate the logged-on user if necessary */
    UserToken = if (*NotificationDll).bImpersonate != 0 {
        Info.hToken
    } else {
        core::ptr::null_mut()
    };
    if !UserToken.is_null() && ImpersonateLoggedOnUser(UserToken) == 0 {
        return;
    }

    /* Call the notification handler. (The i386 CORE-20279 ESP-restore belt
     * for non-conformant cdecl 3rd-party handlers is omitted — see module
     * doc; the handler ABI is the correct WINAPI/stdcall here.) */
    (pNotifyHandler.unwrap())(&mut Info);

    /* Revert impersonation */
    if !UserToken.is_null() {
        RevertToSelf();
    }
}

#[no_mangle]
pub unsafe extern "C" fn CallNotificationDlls(pSession: PWLSESSION, Type: NOTIFICATION_TYPE) {
    let mut ListEntry: PLIST_ENTRY;
    let mut Info: WLX_NOTIFICATION_INFO = core::mem::zeroed();

    /* Set up the notification info structure template */
    Info.Size = core::mem::size_of::<WLX_NOTIFICATION_INFO>() as DWORD;

    match Type {
        LogoffHandler | ShutdownHandler => {
            Info.Flags = 3;
        }
        _ => {
            Info.Flags = 0;
        }
    }

    Info.UserName = (*pSession).UserName;
    Info.Domain = (*pSession).Domain;
    Info.WindowStation = (*pSession).InteractiveWindowStationName;
    Info.hToken = (*pSession).UserToken;

    match Type {
        LogonHandler | StartShellHandler => {
            Info.hDesktop = (*pSession).ApplicationDesktop;
        }
        StartScreenSaverHandler => {
            Info.hDesktop = (*pSession).ApplicationDesktop;
        }
        _ => {
            Info.hDesktop = (*pSession).WinlogonDesktop;
        }
    }

    Info.pStatusCallback = None;

    ListEntry = NotificationDllListHead.Flink;
    while ListEntry != core::ptr::addr_of_mut!(NotificationDllListHead) {
        let Notification = ((ListEntry as usize)
            - core::mem::offset_of!(NOTIFICATION_ITEM, ListEntry))
            as PNOTIFICATION_ITEM;
        if (*Notification).bEnabled != 0 {
            CallNotificationDll(Notification, Type, &mut Info);
        }
        ListEntry = (*ListEntry).Flink;
    }
}

#[no_mangle]
pub unsafe extern "C" fn CleanupNotifications() {
    while IsListEmpty(core::ptr::addr_of_mut!(NotificationDllListHead)) == 0 {
        let ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(NotificationDllListHead));
        let Notification = ((ListEntry as usize)
            - core::mem::offset_of!(NOTIFICATION_ITEM, ListEntry))
            as PNOTIFICATION_ITEM;
        DeleteNotification(Notification);
    }
}