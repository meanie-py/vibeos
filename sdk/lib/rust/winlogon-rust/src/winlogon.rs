//! winlogon.c — the winlogon main: WinMain bootstrap (window station/desktop
//! setup, services.exe + lsass.exe spawn, GINA init) + the WLSession/hAppInstance
//! globals + a handful of helpers. Second of the GINA/SAS/RPC cores flipped.
//!
//! This file is heavily port-customized: the live build always has
//! `_USE_DUMMY_PSEH` (so the stock non-DUMMY `#else` branches are preprocessed
//! out — NOT translated), and the `#ifdef _M_ARM64` branches become
//! `#[cfg(target_arch = "aarch64")]`. The customized boot path (the
//! `#if defined(_M_ARM64) || defined(_USE_DUMMY_PSEH)` block: accept the boot
//! control set, create the minimal HKCU keys, spawn explorer.exe, load fonts on
//! a background thread, then block forever) is live on every arch.
//!
//! WLSession + hAppInstance + WlStrDup are owned here (#[no_mangle]) and
//! re-exported through preamble for the leaf modules; the still-C cores resolve
//! them by symbol. GinaInit/CreateWindowStationAndDesktops/InitDialogListHead
//! (wlx.c) and InitializeSAS/HandleShutdown (sas.c) are still C → local extern.

use crate::preamble::{
    swprintf, wcscat, wcscpy, wcslen, RtlGetProcessHeap, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
};
use crate::{wstr, wstr_arr};
use core::mem::{size_of, zeroed};
use core::ptr::{null, null_mut, write_bytes};
use winlogon_sys::nt::*;

// --- globals (winlogon.c owns these; preamble re-exports) -------------------
#[no_mangle]
pub static mut hAppInstance: HINSTANCE = null_mut();
#[no_mangle]
pub static mut WLSession: PWLSESSION = null_mut();

// --- local consts (not in the winlogon-sys glob) ----------------------------
const STATUS_SYSTEM_PROCESS_TERMINATED: NTSTATUS = 0xC000_021Au32 as NTSTATUS;
const IDS_REACTOSISSTARTINGUP: UINT = 1690;

// --- cross-file externs -----------------------------------------------------
// wlx.c (still C) — all cdecl (no WINAPI in the prototypes). The live (DUMMY_PSEH)
// boot path calls only these three; InitializeSAS/HandleShutdown live in the
// stock non-DUMMY `#else` (preprocessed out), so they are not referenced here.
extern "C" {
    fn GinaInit(Session: PWLSESSION) -> BOOL; // wlx.c
    fn CreateWindowStationAndDesktops(Session: PWLSESSION) -> BOOL; // wlx.c
    fn InitDialogListHead(); // wlx.c
}

// ntdll NTAPI (stdcall) not covered by the bindgen glob.
extern "system" {
    fn NtInitializeRegistry(BootCondition: u32) -> NTSTATUS;
    fn NtWaitForSingleObject(Handle: HANDLE, Alertable: BOOLEAN, Timeout: PLARGE_INTEGER) -> NTSTATUS;
    #[cfg(target_arch = "aarch64")]
    fn NtCreateDirectoryObject(
        DirectoryHandle: PHANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: POBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
}

/// Duplicate a wide string into a heap buffer. cdecl (`PWSTR WlStrDup(PCWSTR)`).
#[no_mangle]
pub unsafe extern "C" fn WlStrDup(String: PCWSTR) -> PWSTR {
    if String.is_null() {
        return null_mut();
    }

    let ptr = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        ((wcslen(String) + 1) * size_of::<WCHAR>()) as SIZE_T,
    ) as PWSTR;
    if !ptr.is_null() {
        wcscpy(ptr, String);
    }
    ptr
}

unsafe fn StartServicesManager() -> BOOL {
    let mut StartupInfo: STARTUPINFOW = zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = zeroed();
    StartupInfo.cb = size_of::<STARTUPINFOW>() as DWORD;

    DbgPrint(
        b"[WINLOGON] StartServicesManager: calling CreateProcessW(services.exe)\n\0".as_ptr()
            as *const i8,
    );

    let res = CreateProcessW(
        wstr!("services.exe").as_ptr(),
        null_mut(),
        null_mut(),
        null_mut(),
        0, // FALSE
        DETACHED_PROCESS,
        null_mut(),
        null(),
        &mut StartupInfo,
        &mut ProcessInformation,
    );
    if res == 0 {
        DbgPrint(
            b"[WINLOGON] StartServicesManager: CreateProcessW FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
        return 0;
    }

    DbgPrint(
        b"[WINLOGON] StartServicesManager: CreateProcessW OK (PID=%lu)\n\0".as_ptr() as *const i8,
        ProcessInformation.dwProcessId,
    );

    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    1 // TRUE
}

unsafe fn StartLsass() -> BOOL {
    let mut StartupInfo: STARTUPINFOW = zeroed();
    let mut ProcessInformation: PROCESS_INFORMATION = zeroed();
    StartupInfo.cb = size_of::<STARTUPINFOW>() as DWORD;

    DbgPrint(b"[WINLOGON] StartLsass: calling CreateProcessW(lsass.exe)\n\0".as_ptr() as *const i8);

    let res = CreateProcessW(
        wstr!("lsass.exe").as_ptr(),
        null_mut(),
        null_mut(),
        null_mut(),
        0, // FALSE
        DETACHED_PROCESS,
        null_mut(),
        null(),
        &mut StartupInfo,
        &mut ProcessInformation,
    );

    if res != 0 {
        DbgPrint(
            b"[WINLOGON] StartLsass: CreateProcessW OK (PID=%lu)\n\0".as_ptr() as *const i8,
            ProcessInformation.dwProcessId,
        );
    } else {
        DbgPrint(
            b"[WINLOGON] StartLsass: CreateProcessW FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }

    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    res
}

unsafe fn WaitForLsass() {
    let mut hEvent: HANDLE = CreateEventW(null_mut(), 1, 0, wstr!("LSA_RPC_SERVER_ACTIVE").as_ptr());
    if hEvent.is_null() {
        let dwError = GetLastError();
        if dwError == ERROR_ALREADY_EXISTS {
            hEvent = OpenEventW(SYNCHRONIZE, 0, wstr!("LSA_RPC_SERVER_ACTIVE").as_ptr());
            if hEvent.is_null() {
                return;
            }
        }
    }

    WaitForSingleObject(hEvent, INFINITE);
    CloseHandle(hEvent);
}

unsafe fn UpdateTcpIpInformation() {
    let mut hKey: HKEY = null_mut();
    let mut dwType: DWORD = 0;
    let mut dwSize: DWORD;
    let mut szBuffer: [u16; 128] = [0; 128];
    let mut pszBuffer: PWSTR = szBuffer.as_mut_ptr();

    let mut lError = RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters").as_ptr(),
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE,
        &mut hKey,
    );
    if lError != ERROR_SUCCESS as LONG {
        return;
    }

    // Read the "NV Hostname" value and copy it into the "Hostname" value.
    dwSize = szBuffer.len() as DWORD; // ARRAYSIZE (in WCHARs, as the C does)
    lError = RegQueryValueExW(
        hKey,
        wstr!("NV Hostname").as_ptr(),
        null_mut(),
        &mut dwType,
        pszBuffer as LPBYTE,
        &mut dwSize,
    );
    if (lError == ERROR_INSUFFICIENT_BUFFER as LONG || lError == ERROR_MORE_DATA as LONG)
        && dwType == REG_SZ
    {
        pszBuffer = HeapAlloc(GetProcessHeap(), 0, dwSize as _) as PWSTR;
        if !pszBuffer.is_null() {
            lError = RegQueryValueExW(
                hKey,
                wstr!("NV Hostname").as_ptr(),
                null_mut(),
                &mut dwType,
                pszBuffer as LPBYTE,
                &mut dwSize,
            );
        } else {
            RegCloseKey(hKey);
            return;
        }
    }
    if lError == ERROR_SUCCESS as LONG && dwType == REG_SZ {
        lError = RegSetValueExW(
            hKey,
            wstr!("Hostname").as_ptr(),
            0,
            REG_SZ,
            pszBuffer as LPBYTE,
            dwSize,
        );
        let _ = lError;
    }

    // Read the "NV Domain" value and copy it into the "Domain" value.
    lError = RegQueryValueExW(
        hKey,
        wstr!("NV Domain").as_ptr(),
        null_mut(),
        &mut dwType,
        pszBuffer as LPBYTE,
        &mut dwSize,
    );
    if (lError == ERROR_INSUFFICIENT_BUFFER as LONG || lError == ERROR_MORE_DATA as LONG)
        && dwType == REG_SZ
    {
        if pszBuffer != szBuffer.as_mut_ptr() {
            let pszNewBuffer = HeapReAlloc(GetProcessHeap(), 0, pszBuffer as _, dwSize as _) as PWSTR;
            if !pszNewBuffer.is_null() {
                pszBuffer = pszNewBuffer;
            } else {
                HeapFree(GetProcessHeap(), 0, pszBuffer as _);
                pszBuffer = null_mut();
            }
        } else {
            pszBuffer = HeapAlloc(GetProcessHeap(), 0, dwSize as _) as PWSTR;
        }
        if !pszBuffer.is_null() {
            lError = RegQueryValueExW(
                hKey,
                wstr!("NV Domain").as_ptr(),
                null_mut(),
                &mut dwType,
                pszBuffer as LPBYTE,
                &mut dwSize,
            );
        } else {
            RegCloseKey(hKey);
            return;
        }
    }
    if lError == ERROR_SUCCESS as LONG && dwType == REG_SZ {
        lError = RegSetValueExW(
            hKey,
            wstr!("Domain").as_ptr(),
            0,
            REG_SZ,
            pszBuffer as LPBYTE,
            dwSize,
        );
        let _ = lError;
    }

    if pszBuffer != szBuffer.as_mut_ptr() {
        HeapFree(GetProcessHeap(), 0, pszBuffer as _);
    }

    RegCloseKey(hKey);
}

/// True on a net-test boot. netstack.sys writes
/// `HKLM\SYSTEM\CurrentControlSet\Services\netstack\NicPresent` = 1 in its
/// DriverEntry iff a virtio-net NIC exists (the amd64-uefi-net / arm64-net
/// configs). Plain smokes have no NIC → the value is 0/absent → this returns
/// false and nettest.exe is never spawned, keeping its process creation off
/// explorer's timing-critical csrss/win32k handoff (the `nettest_explorer_race`
/// flake). netstack is a system-start driver, so the value is committed long
/// before winlogon runs here.
unsafe fn NetTestBootEnabled() -> bool {
    let mut hKey: HKEY = null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SYSTEM\\CurrentControlSet\\Services\\netstack").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != ERROR_SUCCESS as LONG
    {
        return false;
    }
    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut dwSize: DWORD = size_of::<DWORD>() as DWORD;
    let lError = RegQueryValueExW(
        hKey,
        wstr!("NicPresent").as_ptr(),
        null_mut(),
        &mut dwType,
        &mut dwValue as *mut DWORD as LPBYTE,
        &mut dwSize,
    );
    RegCloseKey(hKey);
    lError == ERROR_SUCCESS as LONG && dwType == REG_DWORD && dwValue == 1
}

/// True when a USB mass-storage device was enumerated this boot: PnP creates
/// `HKLM\SYSTEM\CurrentControlSet\Enum\USBSTOR` the moment usbstor reports a
/// child PDO, which happens long before winlogon runs. Plain smokes (no USB
/// disk attached) never have the key, so stortest.exe is never spawned there —
/// same explorer-race rationale as NetTestBootEnabled above.
unsafe fn StorTestBootEnabled() -> bool {
    let mut hKey: HKEY = null_mut();
    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        wstr!("SYSTEM\\CurrentControlSet\\Enum\\USBSTOR").as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != ERROR_SUCCESS as LONG
    {
        return false;
    }
    RegCloseKey(hKey);
    true
}

// Loads the user's preloaded keyboard layouts. Called on all arches now —
// the x86 `#if 0` (Rust-MM SEC_IMAGE deadlock) was obsoleted by the 2026-07-05
// Mm fixes (field manual §9.13).
unsafe fn InitKeyboardLayouts() -> BOOL {
    let mut wszKeyName: [u16; 12] = [0; 12];
    let mut wszKLID: [u16; 10] = [0; 10];
    let mut dwSize: DWORD = size_of::<[u16; 10]>() as DWORD;
    let mut dwType: DWORD = 0;
    let mut i: DWORD = 1;
    let mut hKey: HKEY = null_mut();
    let mut bRet: BOOL = 0; // FALSE

    if RegOpenKeyExW(
        HKEY_CURRENT_USER,
        wstr!("Keyboard Layout\\Preload").as_ptr(),
        0,
        KEY_READ,
        &mut hKey,
    ) == ERROR_SUCCESS as LONG
    {
        loop {
            // Read values with integer names only (swprintf "%d", i++).
            swprintf(wszKeyName.as_mut_ptr(), wstr!("%d").as_ptr(), i);
            i += 1;
            if RegQueryValueExW(
                hKey,
                wszKeyName.as_ptr(),
                null_mut(),
                &mut dwType,
                wszKLID.as_mut_ptr() as LPBYTE,
                &mut dwSize,
            ) != ERROR_SUCCESS as LONG
            {
                break;
            }

            if dwType != REG_SZ {
                continue;
            }

            let mut Flags: UINT = KLF_SUBSTITUTE_OK;
            if i > 1 {
                Flags |= KLF_NOTELLSHELL | KLF_REPLACELANG;
            } else {
                Flags |= KLF_ACTIVATE;
            }
            if LoadKeyboardLayoutW(wszKLID.as_ptr(), Flags).is_null() {
                continue;
            } else {
                bRet = 1; // TRUE
            }
        }

        RegCloseKey(hKey);
    }

    if bRet == 0 {
        if !LoadKeyboardLayoutW(
            wstr!("00000409").as_ptr(),
            KLF_ACTIVATE | KLF_SUBSTITUTE_OK | KLF_REPLACELANG | KLF_SETFORPROCESS,
        )
        .is_null()
        {
            bRet = 1;
        }
    }

    bRet
}

/// Display a GINA status message on the given desktop. cdecl; sas.c calls it.
#[no_mangle]
pub unsafe extern "C" fn DisplayStatusMessage(
    Session: PWLSESSION,
    hDesktop: HDESK,
    ResourceId: UINT,
) -> BOOL {
    let mut StatusMsg: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    if (*Session).Gina.Version < WLX_VERSION_1_3 {
        return 1; // TRUE
    }

    if (*Session).SuppressStatus != 0 {
        return 1;
    }

    if LoadStringW(
        hAppInstance,
        ResourceId,
        StatusMsg.as_mut_ptr(),
        MAX_PATH as i32,
    ) == 0
    {
        return 0; // FALSE
    }

    (*Session).Gina.Functions.WlxDisplayStatusMessage.unwrap()(
        (*Session).Gina.Context,
        hDesktop,
        0,
        null_mut(),
        StatusMsg.as_mut_ptr(),
    )
}

/// Remove the current GINA status message. cdecl; sas.c calls it.
#[no_mangle]
pub unsafe extern "C" fn RemoveStatusMessage(Session: PWLSESSION) -> BOOL {
    if (*Session).Gina.Version < WLX_VERSION_1_3 {
        return 1; // TRUE
    }

    (*Session).Gina.Functions.WlxRemoveStatusMessage.unwrap()((*Session).Gina.Context)
}

// FontLoadThread — WINAPI thread routine (CreateThread's LPTHREAD_START_ROUTINE
// is `extern "stdcall"` on i386 / `extern "C"` on amd64+arm64). Split into a
// shared impl + two cfg trampolines (same pattern as shutdown.rs).
unsafe fn FontLoadThreadImpl(_lpParam: LPVOID) -> DWORD {
    const FONT_NAMES: [&[u16]; 8] = [
        wstr!("\\Fonts\\tahoma.ttf"),
        wstr!("\\Fonts\\tahomabd.ttf"),
        wstr!("\\Fonts\\Marlett.ttf"),
        wstr!("\\Fonts\\arial.ttf"),
        wstr!("\\Fonts\\arialbd.ttf"),
        wstr!("\\Fonts\\cour.ttf"),
        wstr!("\\Fonts\\times.ttf"),
        wstr!("\\Fonts\\symbol.ttf"),
    ];
    let mut WinDir: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut FontPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    let Len = GetWindowsDirectoryW(WinDir.as_mut_ptr(), MAX_PATH);
    DbgPrint(
        b"[WINLOGON] FontLoadThread: WindowsDirectory=%S len=%lu\n\0".as_ptr() as *const i8,
        WinDir.as_ptr(),
        Len,
    );
    if Len == 0 || Len >= MAX_PATH - 40 {
        DbgPrint(
            b"[WINLOGON] FontLoadThread: GetWindowsDirectoryW failed\n\0".as_ptr() as *const i8,
        );
        return 1;
    }
    for name in FONT_NAMES.iter() {
        wcscpy(FontPath.as_mut_ptr(), WinDir.as_ptr());
        wcscat(FontPath.as_mut_ptr(), name.as_ptr());
        if AddFontResourceW(FontPath.as_ptr()) != 0 {
            DbgPrint(
                b"[WINLOGON] Loaded font: %S\n\0".as_ptr() as *const i8,
                FontPath.as_ptr(),
            );
        } else {
            DbgPrint(
                b"[WINLOGON] Failed to load font: %S (err %lu)\n\0".as_ptr() as *const i8,
                FontPath.as_ptr(),
                GetLastError(),
            );
        }
    }
    DbgPrint(
        b"[WINLOGON] Font loading complete (%d fonts)\n\0".as_ptr() as *const i8,
        FONT_NAMES.len() as i32,
    );
    0
}

#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn FontLoadThread(lpParam: LPVOID) -> DWORD {
    FontLoadThreadImpl(lpParam)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn FontLoadThread(lpParam: LPVOID) -> DWORD {
    FontLoadThreadImpl(lpParam)
}

/* DOOM (opt-in demo, VIBEOS_DOOM=1 at image-build time). doomgeneric's
 * Win32/GDI backend — a genuinely foreign binary — is launched ONLY if
 * doom.exe was staged into the image (GetFileAttributesW succeeds): arm64
 * ramdisk via mkramdisk.sh, x86 livecd via strip_cd_media.py. A normal build
 * has no doom.exe, so this is a silent no-op: nothing is spawned, no serial
 * noise. Drive-letter agnostic (GetWindowsDirectoryW: X:\reactos on the arm64
 * ramdisk, D:\reactos on the x86 livecd); the WAD is passed by absolute path
 * (doomgeneric's cwd is not guaranteed to be system32). Bring-up story +
 * smoke suite: docs/engineering/doom-on-vibeos.md, tests/qemu/test_doom.py. */
/* True iff doom.exe was staged into this image (VIBEOS_DOOM=1 build). Also
 * used to suppress the arm64 notepad demo spawn: notepad's window materializes
 * AFTER doom's (slower startup) and — with no foreground-activation-on-show in
 * win32k yet — lands on top of it in the z-order, occluding the game and
 * stealing the input-test click (2026-07-07 screenshot: doom buried under
 * "Untitled - Notepad"). */
#[cfg(target_arch = "aarch64")] // only the arm64 notepad gate consults it
unsafe fn DoomStaged() -> bool {
    let mut DoomPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(DoomPath.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return false;
    }
    wcscat(DoomPath.as_mut_ptr(), wstr!("\\system32\\doom.exe").as_ptr());
    // INVALID_FILE_ATTRIBUTES == 0xFFFFFFFF (not in the bindgen glob).
    GetFileAttributesW(DoomPath.as_ptr()) != 0xFFFF_FFFF
}

unsafe fn TryStartDoom() {
    let mut DoomPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(DoomPath.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(DoomPath.as_mut_ptr(), wstr!("\\system32\\doom.exe").as_ptr());
    // INVALID_FILE_ATTRIBUTES == 0xFFFFFFFF (not in the bindgen glob).
    if GetFileAttributesW(DoomPath.as_ptr()) == 0xFFFF_FFFF {
        return;
    }

    /* Command line: "<windir>\system32\doom.exe" -iwad <windir>\system32\doom1.wad */
    let mut DoomLine: [u16; 2 * MAX_PATH as usize] = [0; 2 * MAX_PATH as usize];
    DoomLine[0] = b'"' as u16;
    wcscat(DoomLine.as_mut_ptr(), DoomPath.as_ptr());
    wcscat(DoomLine.as_mut_ptr(), wstr!("\" -iwad ").as_ptr());
    let mut WadPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    GetWindowsDirectoryW(WadPath.as_mut_ptr(), MAX_PATH - 32);
    wcscat(WadPath.as_mut_ptr(), wstr!("\\system32\\doom1.wad").as_ptr());
    wcscat(DoomLine.as_mut_ptr(), WadPath.as_ptr());

    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;
    let bDoom = CreateProcessW(
        DoomPath.as_ptr(),
        DoomLine.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if bDoom != 0 {
        DbgPrint(
            b"[WINLOGON] doom.exe created (PID %lu, TID %lu)\n\0".as_ptr() as *const i8,
            Pi.dwProcessId,
            Pi.dwThreadId,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(doom.exe) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn WinMain(
    hInstance: HINSTANCE,
    _hPrevInstance: HINSTANCE,
    _lpCmdLine: LPSTR,
    _nShowCmd: i32,
) -> i32 {
    let mut HardErrorResponse: ULONG = 0;

    hAppInstance = hInstance;

    DbgPrint(
        b"[WINLOGON] WinMain entered (PID=%lu)\n\0".as_ptr() as *const i8,
        GetCurrentProcessId(),
    );

    /* Make us critical */
    RtlSetProcessIsCritical(1, null_mut(), 0);
    RtlSetThreadIsCritical(1, null_mut(), 0);

    /* Update the cached TCP/IP Information in the registry */
    UpdateTcpIpInformation();

    DbgPrint(
        b"[WINLOGON] Calling RegisterLogonProcess(%lu)\n\0".as_ptr() as *const i8,
        GetCurrentProcessId(),
    );
    if RegisterLogonProcess(GetCurrentProcessId(), 1) == 0 {
        // _USE_DUMMY_PSEH: log and continue (the stock path raises a hard error).
        DbgPrint(
            b"[WINLOGON] RegisterLogonProcess failed (error %lu), continuing\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }

    WLSession = HeapAlloc(GetProcessHeap(), 0, size_of::<WLSESSION>() as _) as PWLSESSION;
    if WLSession.is_null() {
        NtRaiseHardError(
            STATUS_SYSTEM_PROCESS_TERMINATED,
            0,
            0,
            null_mut(),
            OptionOk as u32,
            &mut HardErrorResponse,
        );
        ExitProcess(1);
    }

    write_bytes(WLSession as *mut u8, 0, size_of::<WLSESSION>());
    (*WLSession).DialogTimeout = 120; /* 2 minutes */

    /* Initialize the dialog tracking list */
    InitDialogListHead();

    #[cfg(target_arch = "aarch64")]
    {
        // Create \Windows\WindowStations directory if it doesn't exist (normally
        // created by NtUserInitialize via winsrv in csrss, which ARM64 skips).
        let mut DirName: UNICODE_STRING = zeroed();
        RtlInitUnicodeString(&mut DirName, wstr!("\\Windows\\WindowStations").as_ptr());
        let mut DirOa: OBJECT_ATTRIBUTES = zeroed();
        DirOa.Length = size_of::<OBJECT_ATTRIBUTES>() as u32;
        DirOa.ObjectName = &mut DirName;
        DirOa.Attributes = OBJ_CASE_INSENSITIVE;
        let mut hWinStaDir: HANDLE = null_mut();
        let DirStatus = NtCreateDirectoryObject(&mut hWinStaDir, DIRECTORY_ALL_ACCESS, &mut DirOa);
        DbgPrint(
            b"[WINLOGON] ARM64: NtCreateDirectoryObject(\\Windows\\WindowStations) = 0x%lx\n\0"
                .as_ptr() as *const i8,
            DirStatus,
        );
        /* Keep hWinStaDir open — directory must persist */
        let _ = hWinStaDir;
    }

    DbgPrint(b"[WINLOGON] Calling CreateWindowStationAndDesktops\n\0".as_ptr() as *const i8);
    if CreateWindowStationAndDesktops(WLSession) == 0 {
        DbgPrint(
            b"[WINLOGON] CreateWindowStationAndDesktops FAILED (error %lu), continuing\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }

    /* LockWorkstation(WLSession) — empty macro */

    /* arm64 + amd64: load a keyboard layout. Without one, win32k's key dispatch
     * drops EVERY key at "NO keyboard layout" (UserProcessKeyboardInput:
     * pti->KeyboardLayout NULL and gspklBaseLayout NULL) — keys reach kbdclass
     * and the RIT and then silently vanish, so no app ever sees WM_KEYDOWN. A
     * normal logon loads layouts via profile loading/userinit, which this
     * customized boot path skips. Was #[cfg(aarch64)]-only ("required for
     * RIT→WM_KEYDOWN delivery") — amd64 needed it for the same reason (found
     * 2026-07-07 by the Doom playable gate: keys traced to [WIN32K] KEYDROP:
     * NO keyboard layout; amd64 now green + playable).
     *
     * i386 EXCLUDED: LoadKeyboardLayoutW -> UserLoadKbdDll ->
     * NtCreateSection(SEC_IMAGE) for kbdus.dll DEADLOCKS in the i386 Rust MM
     * (the "#if 0" the old code cited) — it wedges winlogon before explorer
     * spawns, so i386 never reaches the desktop. Until that MM deadlock is
     * fixed, i386 keeps the pre-existing behavior (no layout -> app keys dead,
     * tracked); the boot-to-desktop North Star wins over interactive input. */
    #[cfg(not(target_arch = "x86"))]
    {
        if InitKeyboardLayouts() == 0 {
            DbgPrint(
                b"[WINLOGON] InitKeyboardLayouts failed (non-fatal)\n\0".as_ptr()
                    as *const i8,
            );
        } else {
            DbgPrint(b"[WINLOGON] Keyboard layout loaded\n\0".as_ptr() as *const i8);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        DbgPrint(
            b"[WINLOGON] ARM64: Skipping RPC, services, lsass, notifications, GINA\n\0".as_ptr()
                as *const i8,
        );
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        // _USE_DUMMY_PSEH path (x86/amd64). InitKeyboardLayouts was `#if 0`
        // in the C for a NtCreateSection SEC_IMAGE deadlock in Rust MM; that Mm
        // family was fixed 2026-07-05 (field manual §9.13), and without a layout
        // win32k drops every keystroke (gspklBaseLayout null). Re-enabled.
        if InitKeyboardLayouts() == 0 {
            DbgPrint(b"[WINLOGON] InitKeyboardLayouts failed (non-fatal)\n\0".as_ptr() as *const i8);
        } else {
            DbgPrint(b"[WINLOGON] InitKeyboardLayouts OK\n\0".as_ptr() as *const i8);
        }

        /* Load system fonts SYNCHRONOUSLY now — right after the window station and
         * desktops exist (CreateWindowStationAndDesktops above) and BEFORE any text
         * is rendered: the GINA status message (DisplayStatusMessage), the desktop
         * watermark, and explorer's taskbar all need g_FontListHead populated. The
         * kernel defers system-font loading to user space on x86 (IntLoadSystemFonts
         * is a no-op there), so until winlogon calls AddFontResourceW the global font
         * list is empty and every GreExtTextOutW hits IntRealizeFont NOMATCH. The old
         * code did this on a background thread AFTER spawning explorer, which raced
         * the shell's first paint and lost on i386 -> permanently blank desktop.
         * Doing it here, before the first text render, makes it deterministic. */
        DbgPrint(b"[WINLOGON] Loading system fonts (synchronous, pre-shell)...\n\0".as_ptr() as *const i8);
        FontLoadThreadImpl(null_mut());

        DbgPrint(b"[WINLOGON] Calling StartRpcServer\n\0".as_ptr() as *const i8);
        if crate::rpcserver::StartRpcServer() == 0 {
            DbgPrint(b"[WINLOGON] StartRpcServer failed (non-fatal)\n\0".as_ptr() as *const i8);
        } else {
            DbgPrint(b"[WINLOGON] StartRpcServer OK\n\0".as_ptr() as *const i8);
        }

        DbgPrint(b"[WINLOGON] Calling StartServicesManager\n\0".as_ptr() as *const i8);
        if StartServicesManager() == 0 {
            DbgPrint(
                b"[WINLOGON] StartServicesManager failed (non-fatal)\n\0".as_ptr() as *const i8,
            );
        } else {
            DbgPrint(b"[WINLOGON] StartServicesManager OK\n\0".as_ptr() as *const i8);
        }

        DbgPrint(b"[WINLOGON] Calling StartLsass\n\0".as_ptr() as *const i8);
        if StartLsass() == 0 {
            DbgPrint(
                b"[WINLOGON] StartLsass failed (error %lu, non-fatal)\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
        } else {
            DbgPrint(
                b"[WINLOGON] StartLsass OK, calling WaitForLsass\n\0".as_ptr() as *const i8,
            );
            /* Wait for the LSA server */
            WaitForLsass();
            DbgPrint(b"[WINLOGON] WaitForLsass returned\n\0".as_ptr() as *const i8);
        }

        /* Init Notifications */
        DbgPrint(b"[WINLOGON] Calling InitNotifications\n\0".as_ptr() as *const i8);
        crate::notify::InitNotifications();
        DbgPrint(
            b"[WINLOGON] InitNotifications done, calling GinaInit\n\0".as_ptr() as *const i8,
        );

        /* Load and initialize gina */
        if GinaInit(WLSession) == 0 {
            DbgPrint(b"[WINLOGON] GinaInit failed (non-fatal)\n\0".as_ptr() as *const i8);
        }

        DisplayStatusMessage(WLSession, (*WLSession).WinlogonDesktop, IDS_REACTOSISSTARTINGUP);
    }

    // --- customized boot path (#if _M_ARM64 || _USE_DUMMY_PSEH: every arch) ---

    /* Tell kernel that CurrentControlSet is good */
    NtInitializeRegistry(CM_BOOT_FLAG_ACCEPTED | 1);

    /* Create minimal HKCU registry keys needed by explorer.exe (normally done by
     * userinit/profile loading, which we skip). */
    {
        let mut hKey: HKEY = null_mut();
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wstr!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer").as_ptr(),
            0,
            null_mut(),
            0,
            KEY_WRITE,
            null_mut(),
            &mut hKey,
            null_mut(),
        ) == ERROR_SUCCESS as LONG
        {
            RegCloseKey(hKey);
            DbgPrint(b"[WINLOGON] Created HKCU\\...\\Explorer key\n\0".as_ptr() as *const i8);
        }
        /* Shell Folders — explorer checks this for desktop/start menu paths */
        if RegCreateKeyExW(
            HKEY_CURRENT_USER,
            wstr!("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders").as_ptr(),
            0,
            null_mut(),
            0,
            KEY_WRITE,
            null_mut(),
            &mut hKey,
            null_mut(),
        ) == ERROR_SUCCESS as LONG
        {
            RegCloseKey(hKey);
        }
    }

    /* Check for pending setup */
    if crate::setup::GetSetupType() != 0 {
        DbgPrint(b"[WINLOGON] Setup mode detected, launching setup...\n\0".as_ptr() as *const i8);
        crate::setup::RunSetup();
    }

    /* crash-value: fire-and-forget the user-mode SEH boot self-test
     * (base/system/sehtest, the userland sibling of ntoskrnl/seh_poc.c).
     * Lives HERE because this boot path skips userinit entirely. Its
     * [SEH-USER] PASS/FAIL serial lines prove hardware-fault ->
     * KiUserExceptionDispatcher -> ntdll dispatch -> __except delivery
     * end-to-end. amd64/arm64 only (native clang SEH does not lower on
     * i386), matching sehtest's cmake gate. */
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        let mut Si: STARTUPINFOW = zeroed();
        let mut Pi: PROCESS_INFORMATION = zeroed();
        let mut SehTestPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut SehTestLine: [u16; 12] = wstr_arr!("sehtest.exe");

        GetWindowsDirectoryW(SehTestPath.as_mut_ptr(), MAX_PATH);
        wcscat(SehTestPath.as_mut_ptr(), wstr!("\\system32\\sehtest.exe").as_ptr());

        Si.cb = size_of::<STARTUPINFOW>() as DWORD;
        Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;

        let ok = CreateProcessW(
            SehTestPath.as_ptr(),
            SehTestLine.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
            0x00000008, /* DETACHED_PROCESS — no console: a win32cui child
                         * spawned this early wedges in console allocation
                         * (observed: WAIT_TIMEOUT + STILL_ACTIVE, no output).
                         * DbgPrint needs no console. */
            null_mut(),
            null(),
            &mut Si,
            &mut Pi,
        );
        if ok != 0 {
            DbgPrint(b"[WINLOGON] sehtest.exe spawned\n\0".as_ptr() as *const i8);
            // Wait + report the exit code: a silent death classifies itself
            // (0xC0000135 DLL-not-found, 0xC0000139 entrypoint-not-found,
            // 0xC0000005 unhandled AV, 0/1 = the test's own verdict).
            let wait = WaitForSingleObject(Pi.hProcess, 10_000);
            let mut code: DWORD = 0;
            GetExitCodeProcess(Pi.hProcess, &mut code);
            DbgPrint(
                b"[WINLOGON] sehtest.exe wait=%lu exit=0x%lx\n\0".as_ptr() as *const i8,
                wait,
                code,
            );
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        } else {
            DbgPrint(
                b"[WINLOGON] sehtest.exe spawn FAILED (error %lu)\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
        }
    }

    DbgPrint(b"[WINLOGON] Init complete. Spawning explorer.exe...\n\0".as_ptr() as *const i8);

    /* Spawn explorer.exe (shell — creates desktop, taskbar) */
    {
        let mut Si: STARTUPINFOW = zeroed();
        let mut Pi: PROCESS_INFORMATION = zeroed();
        let mut ExplorerPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut ExplorerLine: [u16; 13] = wstr_arr!("explorer.exe");

        /* Build full path: %SystemRoot%\explorer.exe */
        GetWindowsDirectoryW(ExplorerPath.as_mut_ptr(), MAX_PATH);
        wcscat(ExplorerPath.as_mut_ptr(), wstr!("\\explorer.exe").as_ptr());
        DbgPrint(
            b"[WINLOGON] Trying explorer path: %S\n\0".as_ptr() as *const i8,
            ExplorerPath.as_ptr(),
        );

        Si.cb = size_of::<STARTUPINFOW>() as DWORD;
        Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;

        let bResult = CreateProcessW(
            ExplorerPath.as_ptr(),
            ExplorerLine.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
            0,
            null_mut(),
            null(),
            &mut Si,
            &mut Pi,
        );

        if bResult != 0 {
            DbgPrint(
                b"[WINLOGON] explorer.exe created (PID %lu, TID %lu)\n\0".as_ptr() as *const i8,
                Pi.dwProcessId,
                Pi.dwThreadId,
            );
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        } else {
            DbgPrint(
                b"[WINLOGON] CreateProcessW(explorer.exe) FAILED (error %lu)\n\0".as_ptr()
                    as *const i8,
                GetLastError(),
            );
        }
    }

    /* ALL ARCHES: make the shell's desktop (WinSta0\Default, where explorer's
     * tray/taskbar windows live) the ACTIVE INPUT DESKTOP. A normal logon
     * reaches this via userinit -> WlxSwitchDesktopToUser, but the customized
     * boot path (every arch under USE_DUMMY_PSEH) skips GINA/logon entirely
     * and RunSetup's async thread can leave the input desktop stranded on
     * "Winlogon". When gpdeskInputDesktop != the shell's desktop:
     *  - win32k's IntGetPaintMessage walks the wrong desktop tree and never
     *    finds explorer's dirty windows -> nothing repaints (the arm64 §9.21
     *    freeze, root-caused 2026-07-06 via a live gpdeskInputDesktop poke);
     *  - keyboard/mouse routing dead-ends: keys reach kbdclass and win32k but
     *    the foreground lookup happens on the stranded desktop, so no app
     *    ever sees them (found on amd64 2026-07-06 by the Doom playable gate:
     *    fresh boots beat RunSetup's async switch and worked, snapshot-restore
     *    runs captured the post-steal state and F1 never reached the game).
     * Was #[cfg(aarch64)] on the §9.22 fix; x86 needs it for the same reason.
     * SwitchDesktop also drives co_IntShowDesktop (desktop mapped to display). */
    if !(*WLSession).ApplicationDesktop.is_null() {
        if SwitchDesktop((*WLSession).ApplicationDesktop) != 0 {
            DbgPrint(
                b"[WINLOGON] switched active input desktop to Default (shell)\n\0".as_ptr()
                    as *const i8,
            );
        } else {
            DbgPrint(
                b"[WINLOGON] SwitchDesktop(Default) FAILED (error %lu)\n\0".as_ptr()
                    as *const i8,
                GetLastError(),
            );
        }
    }

    /* netstack (smoltcp) usermode winsock proof: spawn nettest.exe directly here
     * rather than via explorer's HKCU..\Run / Startup-folder processing, which on
     * amd64 doesn't reliably reach ProcessStartupItems() (the shell startup is
     * slow/fragile — unrelated to the network stack). winlogon is the earliest
     * guaranteed usermode launch point. nettest does a real ws2_32 -> msafd ->
     * AFD -> netstack TCP+UDP round-trip and emits [NETTEST] markers to the debug
     * serial; tests/qemu/test_net.py asserts on them. It retries briefly then
     * exits, so it is harmless on boots with no NIC. We do not wait on it.
     *
     * Gated on NicPresent: even the process CREATION (loader + csrss handoff)
     * contends with explorer's own csrss/win32k handoff, and on plain smoke
     * explorer sometimes lost that race and never reached _tWinMain (the
     * `nettest_explorer_race` flake). netstack.sys publishes NicPresent=1 only
     * when a virtio-net NIC exists (net-test configs), so plain smokes — which
     * have no NIC and no use for nettest anyway — never spawn it. */
    if NetTestBootEnabled() {
        let mut Si: STARTUPINFOW = zeroed();
        let mut Pi: PROCESS_INFORMATION = zeroed();
        let mut NetTestPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut NetTestLine: [u16; 12] = wstr_arr!("nettest.exe");

        GetWindowsDirectoryW(NetTestPath.as_mut_ptr(), MAX_PATH);
        wcscat(NetTestPath.as_mut_ptr(), wstr!("\\system32\\nettest.exe").as_ptr());

        Si.cb = size_of::<STARTUPINFOW>() as DWORD;
        Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;

        // DETACHED_PROCESS (0x8): nettest is a console (win32cui) app, but winlogon
        // has no console to inherit and the desktop console server isn't a sane
        // parent here — without this, kernel32's PROCESS_ATTACH hangs connecting to
        // CSRSS to allocate a console (observed: process stalls before main). The
        // same flag is used for services.exe/lsass.exe. nettest emits only via
        // OutputDebugStringA, so it needs no console.
        const DETACHED_PROCESS: DWORD = 0x0000_0008;
        let bResult = CreateProcessW(
            NetTestPath.as_ptr(),
            NetTestLine.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
            DETACHED_PROCESS,
            null_mut(),
            null(),
            &mut Si,
            &mut Pi,
        );

        if bResult != 0 {
            DbgPrint(
                b"[WINLOGON] nettest.exe created (PID %lu)\n\0".as_ptr() as *const i8,
                Pi.dwProcessId,
            );
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        } else {
            DbgPrint(
                b"[WINLOGON] CreateProcessW(nettest.exe) FAILED (error %lu)\n\0".as_ptr()
                    as *const i8,
                GetLastError(),
            );
        }
    } else {
        DbgPrint(
            b"[WINLOGON] no NIC (NicPresent!=1); skipping nettest.exe spawn\n\0".as_ptr()
                as *const i8,
        );
    }

    /* USB mass-storage usermode proof: spawn stortest.exe (same pattern and
     * rationale as nettest above). Gated on the PnP Enum key that only exists
     * when a USBSTOR device was enumerated this boot, so plain smokes never
     * pay the process-creation race against explorer's handoff. stortest finds
     * the VIBEUSB FAT volume by label, reads MARKER.TXT, writes + reads back a
     * file, and emits [STORTEST] markers to the debug serial. */
    if StorTestBootEnabled() {
        let mut Si: STARTUPINFOW = zeroed();
        let mut Pi: PROCESS_INFORMATION = zeroed();
        let mut StorTestPath: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let mut StorTestLine: [u16; 13] = wstr_arr!("stortest.exe");

        GetWindowsDirectoryW(StorTestPath.as_mut_ptr(), MAX_PATH);
        wcscat(StorTestPath.as_mut_ptr(), wstr!("\\system32\\stortest.exe").as_ptr());

        Si.cb = size_of::<STARTUPINFOW>() as DWORD;
        Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;

        // DETACHED_PROCESS: same console-less-parent reasoning as nettest above.
        const DETACHED_PROCESS: DWORD = 0x0000_0008;
        let bResult = CreateProcessW(
            StorTestPath.as_ptr(),
            StorTestLine.as_mut_ptr(),
            null_mut(),
            null_mut(),
            0,
            DETACHED_PROCESS,
            null_mut(),
            null(),
            &mut Si,
            &mut Pi,
        );

        if bResult != 0 {
            DbgPrint(
                b"[WINLOGON] stortest.exe created (PID %lu)\n\0".as_ptr() as *const i8,
                Pi.dwProcessId,
            );
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        } else {
            DbgPrint(
                b"[WINLOGON] CreateProcessW(stortest.exe) FAILED (error %lu)\n\0".as_ptr()
                    as *const i8,
                GetLastError(),
            );
        }
    }

    /* Fonts are now loaded synchronously BEFORE explorer.exe (see above), so the
     * shell's first paint already has g_FontListHead populated. The old async
     * CreateThread(FontLoadThread) here raced the shell and lost on i386. */

    /* Spawn notepad.exe LAST among GUI apps so it gets keyboard focus.
     * arm64-only: on amd64/i386 notepad hits a separate pre-existing
     * desktop-heap handle-validation AV (pci->pDeskInfo bogus in user32
     * ValidateHandleNoErr) and is reaped by csrss — tracked follow-up in
     * docs/engineering/ui-render-completeness-investigation.md. The NC-paint
     * fix itself is proven on x86 by the tray Start button/clock + dialog
     * chrome, so notepad stays arm64-only until that AV is fixed. */
    #[cfg(target_arch = "aarch64")]
    {
        // When doom.exe is staged (VIBEOS_DOOM=1) it IS the demo GUI app —
        // notepad would materialize on top of it and steal focus (see
        // DoomStaged doc comment). Only the notepad spawn is gated; doom and
        // cmd.exe below always run.
        if !DoomStaged() {
            let mut Si: STARTUPINFOW = zeroed();
            let mut Pi: PROCESS_INFORMATION = zeroed();
            let mut NoteLine: [u16; 12] = wstr_arr!("notepad.exe");
            Si.cb = size_of::<STARTUPINFOW>() as DWORD;
            Si.lpDesktop = wstr!("WinSta0\\Default").as_ptr() as PWSTR;
            let bResult = CreateProcessW(
                wstr!("notepad.exe").as_ptr(),
                NoteLine.as_mut_ptr(),
                null_mut(),
                null_mut(),
                0,
                0,
                null_mut(),
                null(),
                &mut Si,
                &mut Pi,
            );
            if bResult != 0 {
                DbgPrint(
                    b"[WINLOGON] notepad.exe created (PID %lu, TID %lu)\n\0".as_ptr()
                        as *const i8,
                    Pi.dwProcessId,
                    Pi.dwThreadId,
                );
                CloseHandle(Pi.hThread);
                CloseHandle(Pi.hProcess);
            } else {
                DbgPrint(
                    b"[WINLOGON] CreateProcessW(notepad.exe) FAILED (error %lu)\n\0".as_ptr()
                        as *const i8,
                    GetLastError(),
                );
            }
        }

        /* DOOM (opt-in, VIBEOS_DOOM=1): spawned LAST among GUI apps for
         * foreground focus, and BEFORE the cmd.exe INFINITE wait below. */
        TryStartDoom();

        /* Spawn interactive cmd.exe with console (CONSRV available) */
        {
            let mut Si: STARTUPINFOW = zeroed();
            let mut Pi: PROCESS_INFORMATION = zeroed();
            let mut CmdLine: [u16; 8] = wstr_arr!("cmd.exe");
            Si.cb = size_of::<STARTUPINFOW>() as DWORD;
            let bResult = CreateProcessW(
                wstr!("cmd.exe").as_ptr(),
                CmdLine.as_mut_ptr(),
                null_mut(),
                null_mut(),
                0,
                0,
                null_mut(),
                null(),
                &mut Si,
                &mut Pi,
            );
            if bResult != 0 {
                DbgPrint(
                    b"[WINLOGON] ARM64: cmd.exe created (PID %lu, TID %lu). Waiting...\n\0".as_ptr()
                        as *const i8,
                    Pi.dwProcessId,
                    Pi.dwThreadId,
                );
                WaitForSingleObject(Pi.hProcess, INFINITE);
                let mut ExitCode: DWORD = 0xDEAD;
                GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
                DbgPrint(
                    b"[WINLOGON] ARM64: cmd.exe exited, exit code = 0x%lx\n\0".as_ptr() as *const i8,
                    ExitCode,
                );
                CloseHandle(Pi.hThread);
                CloseHandle(Pi.hProcess);
            } else {
                DbgPrint(
                    b"[WINLOGON] ARM64: CreateProcessW(cmd.exe) FAILED (error %lu)\n\0".as_ptr()
                        as *const i8,
                    GetLastError(),
                );
            }
        }

        DbgPrint(b"[WINLOGON] ARM64: cmd.exe exited. Sleeping.\n\0".as_ptr() as *const i8);
    }

    /* DOOM (opt-in, VIBEOS_DOOM=1) — x86 call site. On arm64 doom is spawned
     * inside the block above (before the cmd.exe INFINITE wait, which never
     * returns); this common-path point is only reached on i386/amd64. cfg'd so
     * an arm64 cmd.exe exit can't double-launch. */
    #[cfg(not(target_arch = "aarch64"))]
    TryStartDoom();

    /* No SAS window, no GINA — just sleep forever. Block on a never-signaled
     * event to properly enter WAIT state and leave the CPU for other threads. */
    {
        let mut oa: OBJECT_ATTRIBUTES = zeroed();
        oa.Length = size_of::<OBJECT_ATTRIBUTES>() as u32;
        let mut hSleepEvent: HANDLE = null_mut();
        NtCreateEvent(
            &mut hSleepEvent,
            EVENT_ALL_ACCESS,
            &mut oa,
            NotificationEvent,
            0,
        );
        NtWaitForSingleObject(hSleepEvent, 0, null_mut());
        /* Never reaches here */
    }

    /* We never go there */
    #[allow(unreachable_code)]
    {
        SleepEx(INFINITE, 0);
        0
    }
}
