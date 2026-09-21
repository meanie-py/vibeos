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

unsafe extern "system" fn FontLoadThread(lpParam: LPVOID) -> DWORD {
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

/* FEX bring-up rung 2 (docs/engineering/fex-bringup-plan.md): spawn the
 * hello-compute harness fexprobe.exe if it was staged into the image, exactly
 * like TryStartDoom — a foreign-payload probe gated on file existence, output
 * via DbgPrint->serial ([FEX-HELLO] lines). A normal build has no fexprobe.exe,
 * so this is a silent no-op. We WaitForSingleObject so its serial lines land
 * deterministically before the desktop-idle cmd.exe wait. */
#[cfg(target_arch = "aarch64")]
unsafe fn TryStartFexProbe() {
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\system32\\fexprobe.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 13] = wstr_arr!("fexprobe.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] fexprobe.exe created (PID %lu) - waiting\n\0".as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 30_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] fexprobe.exe exit = 0x%lx\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(fexprobe.exe) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
}

/* WOW64 Phase B probe (docs/engineering/wow64-plan.md): spawn wow64probe.exe
 * if staged — same gate-on-file-existence pattern as fexprobe above. It drives
 * the real libwow64fex.dll BTCpu contract; verdict lands on serial as
 * [FEX-WOW64B] lines and exit code 0x600D on PASS. */
#[cfg(target_arch = "aarch64")]
unsafe fn TryStartWow64Probe() {
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\system32\\wow64probe.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 15] = wstr_arr!("wow64probe.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] wow64probe.exe created (PID %lu) - waiting\n\0".as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 30_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] wow64probe.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(wow64probe.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* True only when the FEX CPU layer (libwow64fex.dll) is staged in system32 —
 * i.e. a VIBEOS_FEX build. The WoW64 guest exes (hello32/gui32/win32) can
 * persist in a shared ramdisk.img across a plain `mkramdisk --refresh` (dev
 * pollution), but WITHOUT libwow64fex.dll their WoW64 bootstrap runs
 * half-initialized and faults the kernel. Gate every guest spawn on the real
 * support layer, not just the guest exe, so a polluted ramdisk in a non-FEX
 * boot is an inert no-op (kept the standard smoke green — 2026-07-11). */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn Wow64SupportStaged() -> bool {
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return false;
    }
    /* The per-arch CPU layer (the same selection wow64.c makes): FEX JIT on
     * arm64, native heaven's-gate on amd64. */
    #[cfg(target_arch = "aarch64")]
    wcscat(Path.as_mut_ptr(), wstr!("\\system32\\libwow64fex.dll").as_ptr());
    #[cfg(target_arch = "x86_64")]
    wcscat(Path.as_mut_ptr(), wstr!("\\system32\\wow64cpu.dll").as_ptr());
    GetFileAttributesW(Path.as_ptr()) != 0xFFFF_FFFF
}

/* WOW64 Phase C probe: launch a real i386 PE (hello32.exe) through the normal
 * CreateProcessW front door. Unlike wow64probe (a native arm64 harness), this
 * exercises the actual WoW64 process bootstrap: kernel32 machine gate →
 * PspCreateProcess → PEB32/TEB32 → 64-bit ntdll loader wow64 branch →
 * wow64!Wow64LdrpInitialize. Verdict = [FEX-WOW64C] serial lines + 0x600D.
 * amd64 (Phase E): identical probe, but the CPU layer is the native wow64cpu
 * heaven's-gate instead of FEX — same staging contract, same verdict. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Hello() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\hello32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("hello32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] hello32.exe (i386 guest) created (PID %lu) - waiting\n\0".as_ptr()
                as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 30_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] hello32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(hello32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* M10 file-I/O probe: an i386 guest that reads, writes and enumerates real
 * files. It is the oracle for the IO_STATUS_BLOCK 32<->64 marshalling — the
 * byte count ReadFile hands back IS IoStatusBlock.Information, which is 4 bytes
 * wide in the guest and 8 on the host. Silent no-op unless staged;
 * verdict = [file32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64File() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\file32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 11] = wstr_arr!("file32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] file32.exe (i386 file-I/O guest) created (PID %lu) - waiting\n\0".as_ptr()
                as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] file32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(file32.exe) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
}

/* M15b async file-I/O probe: an i386 guest doing Event-based + polled
 * overlapped ReadFile on a FILE_FLAG_OVERLAPPED handle. It is the oracle for
 * kernel-side IO_STATUS_BLOCK32 completion — OVERLAPPED.Internal/InternalHigh
 * ARE the guest IOSB32, and .Offset detects a stray 16-byte (64-bit layout)
 * write. Silent no-op unless staged; verdict = [async32] lines + exit 0x600D. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Async() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\async32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("async32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] async32.exe (i386 overlapped-I/O guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] async32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(async32.exe) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
}

/* M8 window-creation probe: an i386 guest with user32 imports that registers
 * a class, creates a REAL window, and pumps messages (forces user32 client
 * connect + kernel->user callbacks under wow64). Silent no-op unless staged;
 * verdict = [win32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Win32() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\win32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 10] = wstr_arr!("win32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    /* The window must live on the INPUT desktop (Default) or win32k's
     * IntGetPaintMessage never finds it and the pump starves (§9.22 class);
     * winlogon's own threads are on the Winlogon desktop. */
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] win32.exe (i386 window guest) created (PID %lu) - waiting\n\0".as_ptr()
                as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] win32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(win32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* True iff the WoW64 (i386) doom.exe was staged into syswow64. When it is, DOOM
 * is the demo: the other GUI guests would materialise on top of its window and
 * occlude it (doomgeneric creates its window at CW_USEDEFAULT and never calls
 * SetForegroundWindow). Same rationale as DoomStaged() gating the notepad spawn. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn Wow64DoomStaged() -> bool {
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return false;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\doom.exe").as_ptr());
    GetFileAttributesW(Path.as_ptr()) != 0xFFFF_FFFF
}

/* M18 marshaller-verification probe: an i386 guest that self-verifies the
 * generated/manual marshallers no earlier guest touches — TBI against its own
 * TEB32, IOCP sentinel round-trip, monitor enumeration. Silent no-op unless
 * staged; verdict = [marshal32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Marshal() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\marshal32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 14] = wstr_arr!("marshal32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    /* EnumDisplayMonitors needs a user32 connect on the input desktop. */
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] marshal32.exe (i386 marshaller-verify guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] marshal32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(marshal32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* E5 — fault32.exe: guest-SEH verification. A hardware AV in guest code must
 * dispatch into the guest's OWN 32-bit SEH with the LIVE faulting context
 * (BTCpuResetToConsistentState flush + the VEH redirect), a software
 * RaiseException must round-trip through the NtRaiseException marshaller, and
 * OutputDebugStringA's DBG_PRINTEXCEPTION must be consumed, not dispatched.
 *
 * Runs on BOTH arches as of E18. The two get there differently and that is the
 * point: amd64 catches a real compat-mode fault in Wow64pPrepareForExceptionHook,
 * while on arm64 the fault is a HOST fault inside FEX's JIT that the CPU layer
 * hands back in guest terms, and wow64.dll's VEH builds the guest dispatcher
 * frame. Same guest, same verdict, two completely different arrival paths —
 * which is exactly what makes it worth running twice. Silent no-op unless
 * staged; verdict = [FAULT32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Fault() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\fault32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("fault32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] fault32.exe (i386 guest-SEH verify guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] fault32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(fault32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* #64 — thread32.exe: 32-bit guest THREADING. Until the per-thread WoW64 entry
 * existed, NtCreateThread was a loud refusal (and before that a flat thunk that
 * returned SUCCESS while the kernel built a thread with a zeroed trap frame —
 * osk.exe, BUGCHECK 0x1E). This guest creates real threads and checks the
 * pieces that would each fail silently on their own: the thread parameter, TLS
 * (proves the guest loader's per-thread init ran), guest SEH on the new thread
 * (proves TEB32's NtTib stack bounds were set), and four concurrent threads
 * with distinct stacks (proves the CPU area is per-thread).
 *
 * Runs on BOTH arches — the per-thread entry is arch-neutral and arm64 drives
 * it through FEX's BTCpu contract, which is exactly the path that would
 * otherwise have no functional coverage. All nine rungs run on both arches as
 * of E18; arm64 used to pass "noseh" to skip rung 7 while the hardware-fault
 * redirect into guest SEH was still owed there.
 * Silent no-op unless staged; verdict = [THREAD32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Thread() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\thread32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    /* E18 dropped the "noseh" argument: arm64 now redirects a hardware fault in
     * guest code into the guest's own 32-bit SEH, so rung 7 runs on both. */
    let mut Line: [u16; 13] = wstr_arr!("thread32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] thread32.exe (i386 threading guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 90_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] thread32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(thread32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* E17 — selfmod32.exe: RUNTIME CODE GENERATION by a 32-bit guest. wow64.dll
 * wired exactly one of the CPU layer's memory notifications (map-view), which
 * covers images and nothing else. On a JIT host that is a missing capability,
 * not a missing optimisation: FEX refuses to decode any guest address outside
 * the executable set it was told about, and caches translations by guest
 * address. So a guest could not run VirtualAlloc'd code at all — ATL/MFC
 * thunks, packers, self-extractors, any JIT — and where it could, it ran
 * whatever it had generated before. The stale-translation rungs return the
 * PREVIOUS constant rather than crashing, which is why this needs a guest that
 * checks values instead of one that merely survives.
 *
 * amd64 is the control, not a second copy of the test: wow64cpu exports none of
 * these notifications (execution there is native, nothing to invalidate), so
 * every rung must pass with 0/8 wired. arm64 exercises the real contract.
 * Silent no-op unless staged; verdict = [selfmod32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Selfmod() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\selfmod32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 14] = wstr_arr!("selfmod32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] selfmod32.exe (i386 codegen guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 90_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] selfmod32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(selfmod32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* reg32.exe: WOW64 REGISTRY REDIRECTION, checked from inside a 32-bit process.
 *
 * The rest of the ladder cannot see this and neither can a host-side unit test.
 * Redirection keys off "is the CALLING process 32-bit"
 * (CmpWow64IsCurrentProcess32Bit), so the only vantage point that can observe
 * it is a real WoW64 guest — which is exactly why the feature shipped with no
 * runtime test at all.
 *
 * It is also the ladder's first guest whose failures are SILENT. Every other
 * rung here crashes, hangs, or returns a wrong number. A redirection bug
 * returns SUCCESS and the wrong key: the app writes its configuration into the
 * 64-bit view and the next reader gets someone else's answer. Its sharpest
 * rung is the two-view DELETE (RegDeleteKeyExW with an explicit
 * KEY_WOW64_32KEY must not touch the native key) — the one failure here that
 * destroys data rather than merely misplacing it.
 *
 * Runs on both hosts and must pass identically: CM does the redirecting, above
 * the CPU layer, so amd64 and arm64 differ in nothing this guest touches.
 * Silent no-op unless staged; verdict = [reg32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Reg() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\reg32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 10] = wstr_arr!("reg32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] reg32.exe (i386 registry-view guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 90_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] reg32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(reg32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* spawn32.exe — the i386 PROCESS-CREATION guest: can a 32-bit guest start
 * another 32-bit process at all (CreateProcessW, IsWow64Process on the child,
 * ResumeThread across the process boundary, wait, exit code).
 *
 * DIAGNOSTIC, not a parity guest. It measures a gap the x64-primary plan ranks
 * #1 and which is expected to be OPEN, so ladder.py reports its verdict from
 * the DIAGNOSTIC table and never fails a gate on it. Launched here all the
 * same, and BEFORE the optional stress guests: it is fast and bounded, while
 * they are slow third-party code that can push later guests past the drain.
 *
 * The 60s wait is deliberately double the guest's own 30s child timeout, so a
 * guest that hit its internal cap still gets to report a verdict rather than
 * being cut off by winlogon and read as a hang. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Spawn() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\spawn32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("spawn32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] spawn32.exe (i386 process-creation guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] spawn32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr()
                as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(spawn32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* sqlite32.exe — a THIRD-PARTY STRESS guest (tools/stress/sqlite/build.sh).
 *
 * Every other guest here probes one mechanism on purpose, with a body written
 * to make that mechanism observable. This one is the opposite and that is the
 * point: ~250k lines of SQLite, a program that never heard of us, doing what it
 * normally does. It leans on what the deliberate probes barely touch --
 * allocator churn under a real pager, Win32 file I/O with locking, journalling
 * and truncation, and long stretches of pure compute between syscalls. DOOM
 * covers the graphics axis; this covers the storage and allocator axis.
 *
 * Built on demand (it fetches SQLite), so it is normally ABSENT: silent no-op
 * unless staged, like every guest here. Verdict = [sqlite32] serial lines +
 * exit 0x600D, with a distinct 0xBADn per rung so a red run names the rung. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Sqlite() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\sqlite32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 13] = wstr_arr!("sqlite32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] sqlite32.exe (i386 SQLite stress guest) created (PID %lu) - waiting\n\0"
                .as_ptr() as *const i8,
            Pi.dwProcessId,
        );
        /* Longer than the probe guests: this one does real work, and it runs as
         * x86 under FEX under TCG. A timeout here reads as a red rung, so give
         * it room rather than manufacturing a flake. */
        WaitForSingleObject(Pi.hProcess, 300_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] sqlite32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(sqlite32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
    }
}

/* Third-party stress guest #2: the real Lua 5.4 interpreter, i386, under WoW64.
 *
 * sqlite32 stresses the DATA path. This one stresses the CONTROL path, which no
 * other guest reaches: Lua implements error propagation AND coroutine switching
 * with setjmp/longjmp, so each rung drives a non-local jump across frames the
 * emulator has already translated. fault32 does not cover this -- SEH unwinding
 * and C setjmp/longjmp are different machinery, and only the latter is what a
 * block cache has to invalidate correctly.
 *
 * Built on demand (it fetches Lua), so it is normally ABSENT: silent no-op
 * unless staged, like every guest here. Verdict = [lua32] serial lines + exit
 * 0x600D, with a distinct 0xBADn per rung so a red run names the rung. */
#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
unsafe fn TryStartWow64Lua() {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\lua32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 10] = wstr_arr!("lua32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] lua32.exe (i386 Lua stress guest) created (PID %lu) - waiting\n\0".as_ptr()
                as *const i8,
            Pi.dwProcessId,
        );
        /* Same budget as sqlite32 and for the same reason: real third-party work
         * running as x86 under FEX under TCG. A timeout here reads as a red rung,
         * so give it room rather than manufacturing a flake. */
        WaitForSingleObject(Pi.hProcess, 300_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] lua32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(lua32.exe) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            GetLastError(),
        );
    }
}

/* M12 — THE NORTH STAR: x86 DOOM, under WoW64, on arm64.
 *
 * doomgeneric's Win32/GDI backend built for i386 (tools/doom/build.sh i386) —
 * a genuinely foreign binary, compiled by mingw against the real msvcrt, that
 * has never heard of ReactOS. It needs everything the lane has built: the CRT
 * bootstrap and the 53-module SysWOW64 closure (M11), window creation and the
 * message pump (M8), the 32-bit client view of USER/GDI (M9/M6/M7), file I/O to
 * read the 4 MB IWAD (M10), and a GDI blit path for its 320x200 framebuffer.
 *
 * Its import closure is EXACTLY the same 53 modules testgui needs — msvcrt,
 * user32, gdi32, kernel32 — so M11's closure work covers it with nothing added.
 *
 * Like every other guest: silent no-op unless staged (VIBEOS_FEX=1 image).
 * amd64 (E4): the same doom-i386.exe, but the CPU under it is real silicon. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Doom() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\doom.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    /* "<windir>\syswow64\doom.exe" -iwad <windir>\syswow64\doom1.wad
     * The WAD goes by ABSOLUTE path: doomgeneric's cwd is not guaranteed. */
    let mut Line: [u16; 2 * MAX_PATH as usize] = [0; 2 * MAX_PATH as usize];
    Line[0] = b'"' as u16;
    wcscat(Line.as_mut_ptr(), Path.as_ptr());
    wcscat(Line.as_mut_ptr(), wstr!("\" -iwad ").as_ptr());
    let mut Wad: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    GetWindowsDirectoryW(Wad.as_mut_ptr(), MAX_PATH - 32);
    wcscat(Wad.as_mut_ptr(), wstr!("\\syswow64\\doom1.wad").as_ptr());
    wcscat(Line.as_mut_ptr(), Wad.as_ptr());

    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    if CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    ) == 0
    {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(WoW64 doom.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
        return;
    }

    DbgPrint(
        b"[WINLOGON] *** x86 DOOM under WoW64 created (PID %lu) *** - watching\n\0".as_ptr()
            as *const i8,
        Pi.dwProcessId,
    );
    /* Doom never exits on its own. Still running = it is playing. */
    const WAIT_TIMEOUT_D: DWORD = 0x102;
    if WaitForSingleObject(Pi.hProcess, 90_000) == WAIT_TIMEOUT_D {
        DbgPrint(
            b"[WINLOGON] *** x86 DOOM still running after 90s = IT IS RUNNING *** \
              (leaving it up)\n\0"
                .as_ptr() as *const i8,
        );
    } else {
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] x86 DOOM EXITED = 0x%lx (it should never exit)\n\0".as_ptr()
                as *const i8,
            ExitCode,
        );
    }
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* M11 A/B control: the NATIVE arm64 build of testgui.exe.
 *
 * The WoW64 guest and this one are the SAME SOURCE, and both drive the same
 * user32-rust. So when the 32-bit one dies inside a user32 control, this says
 * whether the WoW64 layer is at fault or whether user32 is simply broken there
 * for everyone. Without a control, every user32 crash looks like a WoW64 bug.
 *
 * Only spawned in a VIBEOS_FEX dev boot (gated on Wow64SupportStaged, like the
 * guests), so the normal smoke media never sees it. */
#[cfg(target_arch = "aarch64")]
unsafe fn TryStartNativeTestGui() {
    if !Wow64SupportStaged() {
        return;
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\system32\\testgui.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("testgui.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    if CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    ) == 0
    {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(native testgui.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
        return;
    }
    DbgPrint(
        b"[WINLOGON] NATIVE testgui.exe created (PID %lu) - A/B control for the WoW64 one\n\0"
            .as_ptr() as *const i8,
        Pi.dwProcessId,
    );
    const WAIT_TIMEOUT_N: DWORD = 0x102;
    if WaitForSingleObject(Pi.hProcess, 20_000) == WAIT_TIMEOUT_N {
        DbgPrint(
            b"[WINLOGON] NATIVE testgui.exe still running after 20s (healthy)\n\0".as_ptr()
                as *const i8,
        );
    } else {
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] NATIVE testgui.exe EXITED = 0x%lx <- if this also dies, the bug is \
              in user32, NOT in the WoW64 layer\n\0"
                .as_ptr() as *const i8,
            ExitCode,
        );
    }
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* M11: a STOCK, unmodified i386 binary — base/applications/testgui, built for
 * i386 exactly as the livecd ships it. Every guest before this one (hello32,
 * gui32, win32, file32) was written BY US, which means each could quietly dodge
 * whatever WoW64 didn't support yet. testgui can't dodge anything: it's linked
 * against the real msvcrt CRT (so the entrypoint is msvcrt's startup, not a
 * hand-rolled `_start`), it has a real import table (the full 53-module
 * SysWOW64 closure), and it registers a class, creates a window with a child
 * EDIT control, and pumps messages.
 *
 * It never exits — it's a message loop. So the verdict is inverted from the
 * other guests: STILL RUNNING after the wait means the CRT started, the imports
 * bound, the window came up and the pump is alive = PASS. An exit means it
 * either failed its own checks (1) or died (0xC0000005 &c). It is deliberately
 * left running so its window is on screen for the screenshot.
 *
 * Verdict: [TESTGUI] serial lines (wWinMain -> RegisterClassExW ->
 * CreateWindowExW -> WM_CREATE -> WM_PAINT) + a blue/green window. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64TestGui() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\testgui.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("testgui.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    /* Must live on the INPUT desktop or the pump starves (§9.22 class). */
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok == 0 {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(testgui.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
        return;
    }

    DbgPrint(
        b"[WINLOGON] testgui.exe (STOCK i386 GUI app) created (PID %lu) - watching\n\0".as_ptr()
            as *const i8,
        Pi.dwProcessId,
    );
    const WAIT_TIMEOUT_: DWORD = 0x102;
    if WaitForSingleObject(Pi.hProcess, 20_000) == WAIT_TIMEOUT_ {
        DbgPrint(
            b"[WINLOGON] testgui.exe still running after 20s = PASS (CRT started, imports \
              bound, message pump alive) - leaving it up\n\0"
                .as_ptr() as *const i8,
        );
    } else {
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] testgui.exe EXITED = 0x%lx (a stock GUI app should never exit; \
              it should still be pumping)\n\0"
                .as_ptr() as *const i8,
            ExitCode,
        );
    }
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* E6 — generic STOCK i386 applet launcher for the amd64 robustness ladder.
 * Same inverted verdict as winmine: still running after the wait = PASS (CRT
 * started, imports bound, window up, message pump alive); an early exit means
 * it died (exit code names the failure). Silent no-op unless the named exe is
 * staged in syswow64. The cmdline is the bare exe name in a writable buffer
 * (CreateProcessW mutates it in place). */
#[cfg(target_arch = "x86_64")]
unsafe fn TryStartWow64StockApp(Name: *const u16, Label: *const i8, TimeoutMs: DWORD) {
    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\").as_ptr());
    wcscat(Path.as_mut_ptr(), Name);
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    wcscat(Line.as_mut_ptr(), Name);
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    if CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    ) == 0
    {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(%s) FAILED (error %lu)\n\0".as_ptr() as *const i8,
            Label,
            GetLastError(),
        );
        return;
    }
    DbgPrint(
        b"[WINLOGON] %s (STOCK i386 app) created (PID %lu) - watching\n\0".as_ptr() as *const i8,
        Label,
        Pi.dwProcessId,
    );
    const WAIT_TIMEOUT_: DWORD = 0x102;
    if WaitForSingleObject(Pi.hProcess, TimeoutMs) == WAIT_TIMEOUT_ {
        DbgPrint(
            b"[WINLOGON] %s still running = PASS (window up, pump alive) - leaving it up\n\0"
                .as_ptr() as *const i8,
            Label,
        );
    } else {
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] %s EXITED = 0x%lx (a stock GUI app should still be pumping)\n\0".as_ptr()
                as *const i8,
            Label,
            ExitCode,
        );
    }
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* A Wine conformance suite, run as a WoW64 guest.
 *
 * Every other guest on this ladder reports in one number -- an exit code, or
 * "still running after N seconds". A winetest reports in PROSE on stdout and
 * only summarises in its exit code (= the failure count), so both of those
 * verdicts throw away the part we actually want: WHICH assertion failed and
 * where. Nothing inside the guest can reach the serial port, so this hands the
 * child a pipe for stdout+stderr and re-emits what comes back, line by line,
 * through DbgPrint -- the one channel the smoke harness already reads.
 *
 * `Sub` names ONE subtest. A winetest invoked with no argument runs all of
 * them, which on a suite like user32 is thousands of lines and ends at the
 * first crash with everything after it unrun; pass null only when running the
 * whole suite is the point.
 *
 * The read loop peeks rather than blocking, because a blocking ReadFile on a
 * pipe whose writer is a wedged guest never returns and would hang winlogon
 * itself -- the ladder would stop dead with no verdict for anything after it.
 */
/* x86_64 only for now: the call sites are in the amd64 ladder, and an
 * arm64-compiled-but-never-called copy is just a dead_code warning. arm64 can
 * have it the day it stages a winetest. */
#[cfg(target_arch = "x86_64")]
unsafe fn TryStartWow64Winetest(
    Exe: *const u16,
    Sub: *const u16,
    Label: *const i8,
    TimeoutMs: DWORD,
) {
    /* Serial is the bottleneck for the whole boot, and a looping suite can
     * emit without bound. Past this many bytes the output is dropped (the
     * pipe is still drained, or the child blocks on a full pipe and the
     * timeout below turns a chatty test into a fake hang). */
    const WT_MAX_OUTPUT: u32 = 256 * 1024;

    if !Wow64SupportStaged() {
        return; // CPU layer absent
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\").as_ptr());
    wcscat(Path.as_mut_ptr(), Exe);
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged -- silent no-op, like every other optional guest
    }

    let mut Sa: SECURITY_ATTRIBUTES = zeroed();
    Sa.nLength = size_of::<SECURITY_ATTRIBUTES>() as DWORD;
    Sa.bInheritHandle = 1;
    let mut Rd: HANDLE = null_mut();
    let mut Wr: HANDLE = null_mut();
    if CreatePipe(&mut Rd, &mut Wr, &mut Sa, 64 * 1024) == 0 {
        DbgPrint(
            b"[WT] %s: CreatePipe FAILED (error %lu) - not running\n\0".as_ptr() as *const i8,
            Label,
            GetLastError(),
        );
        return;
    }
    /* The child must NOT inherit the read end: if it did, the last writer
     * would never close and the drain below would never see EOF. */
    SetHandleInformation(Rd, HANDLE_FLAG_INHERIT, 0);

    let mut Line: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    wcscat(Line.as_mut_ptr(), Exe);
    if !Sub.is_null() {
        wcscat(Line.as_mut_ptr(), wstr!(" ").as_ptr());
        wcscat(Line.as_mut_ptr(), Sub);
    }

    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    Si.dwFlags |= STARTF_USESTDHANDLES;
    Si.hStdInput = null_mut();
    Si.hStdOutput = Wr;
    Si.hStdError = Wr;
    /* DETACHED_PROCESS. A winetest is a CUI image, so without this csrss
     * allocates it a console -- and console creation from a WoW64 guest fails
     * here (`co_UserCreateWindowEx(ConsoleWindowClass): NCCREATE message
     * failed`), after which the guest spins forever without reaching main and
     * writes not one byte. Measured: 60s timeout, 0 bytes, 5036 "Sending to
     * Suspended Thread" lines. It needs no console anyway -- its stdout is the
     * pipe above. The console failure is a real bug and is registered
     * separately; this is not a workaround for it, the process genuinely
     * should not have a console. */
    const DETACHED_PROCESS: DWORD = 0x0000_0008;
    if CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        1, /* bInheritHandles: required for the child to get Wr */
        DETACHED_PROCESS,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    ) == 0
    {
        DbgPrint(
            b"[WT] %s: CreateProcessW FAILED (error %lu)\n\0".as_ptr() as *const i8,
            Label,
            GetLastError(),
        );
        CloseHandle(Rd);
        CloseHandle(Wr);
        return;
    }
    /* Drop OUR copy of the write end. The child now holds the only one, so
     * the pipe reports empty-and-closed exactly when the child is gone --
     * without this the drain below can never distinguish "no output yet"
     * from "finished". */
    CloseHandle(Wr);

    DbgPrint(
        b"[WT] %s: started (PID %lu)\n\0".as_ptr() as *const i8,
        Label,
        Pi.dwProcessId,
    );

    let mut Chunk: [u8; 1024] = [0; 1024];
    let mut Out: [u8; 512] = [0; 512];
    let mut OutLen: usize = 0;
    let mut Total: u32 = 0;
    let mut Truncated = false;
    let Start = GetTickCount();
    const WAIT_TIMEOUT_: DWORD = 0x102;
    let mut TimedOut = false;

    loop {
        let mut Avail: DWORD = 0;
        let Peeked = PeekNamedPipe(
            Rd,
            null_mut(),
            0,
            null_mut(),
            &mut Avail,
            null_mut(),
        );
        if Peeked != 0 && Avail > 0 {
            let Want = if Avail as usize > Chunk.len() {
                Chunk.len() as DWORD
            } else {
                Avail
            };
            let mut Got: DWORD = 0;
            if ReadFile(
                Rd,
                Chunk.as_mut_ptr() as *mut _,
                Want,
                &mut Got,
                null_mut(),
            ) == 0
                || Got == 0
            {
                break;
            }
            Total = Total.saturating_add(Got);
            if Total > WT_MAX_OUTPUT {
                if !Truncated {
                    Truncated = true;
                    DbgPrint(
                        b"[WT] %s: output past %lu bytes DROPPED (still draining)\n\0".as_ptr()
                            as *const i8,
                        Label,
                        WT_MAX_OUTPUT,
                    );
                }
            } else {
                for i in 0..Got as usize {
                    let c = Chunk[i];
                    if c == b'\n' || OutLen == Out.len() - 1 {
                        Out[OutLen] = 0;
                        /* "%s" and not the line itself: a test's own output
                         * is full of %d/%s from the assertions it prints. */
                        DbgPrint(
                            b"[WT] %s| %s\n\0".as_ptr() as *const i8,
                            Label,
                            Out.as_ptr(),
                        );
                        OutLen = 0;
                    }
                    if c != b'\n' && c != b'\r' {
                        Out[OutLen] = c;
                        OutLen += 1;
                    }
                }
            }
            continue; /* drain greedily before re-checking the clock */
        }
        /* Nothing buffered: the child is either done or thinking. */
        if WaitForSingleObject(Pi.hProcess, 0) != WAIT_TIMEOUT_ {
            /* Exited. One more peek catches whatever it wrote last. */
            let mut Left: DWORD = 0;
            if PeekNamedPipe(Rd, null_mut(), 0, null_mut(), &mut Left, null_mut()) == 0
                || Left == 0
            {
                break;
            }
            continue;
        }
        if GetTickCount().wrapping_sub(Start) > TimeoutMs {
            TimedOut = true;
            break;
        }
        Sleep(20);
    }

    if OutLen > 0 {
        Out[OutLen] = 0;
        DbgPrint(
            b"[WT] %s| %s\n\0".as_ptr() as *const i8,
            Label,
            Out.as_ptr(),
        );
    }

    if TimedOut {
        DbgPrint(
            b"[WT] %s: TIMEOUT after %lu ms - terminating\n\0".as_ptr() as *const i8,
            Label,
            TimeoutMs,
        );
        TerminateProcess(Pi.hProcess, 0xDEAD);
        WaitForSingleObject(Pi.hProcess, 5_000);
    }
    let mut ExitCode: DWORD = 0xDEAD;
    GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
    /* A winetest's exit code IS its failure count -- but only when it got to
     * choose it. An unhandled exception exits the process with the NTSTATUS,
     * and 0xC0000005 read as "3221225477 failures" is a number that looks like
     * data and is not: the suite died partway and every assertion after the
     * fault went unrun. Say which of the two this is, because they mean
     * opposite things about the run. */
    if ExitCode >= 0xC000_0000 {
        DbgPrint(
            b"[WT] %s: CRASHED, exit = 0x%lx, %lu bytes of output\n\0".as_ptr() as *const i8,
            Label,
            ExitCode,
            Total,
        );
    } else {
        DbgPrint(
            b"[WT] %s: exit = %lu failure(s), %lu bytes of output\n\0".as_ptr() as *const i8,
            Label,
            ExitCode,
            Total,
        );
    }
    CloseHandle(Rd);
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* E4c robustness: winmine.exe — a STOCK i386 OS applet (menus, WM_TIMER,
 * dialog/menu resources, GetSystemMetrics-driven layout) that nobody wrote
 * for the WoW64 harness. Same inverted verdict as testgui: still running
 * after the wait = PASS (window up, pump alive); an exit = it died. Silent
 * no-op unless syswow64\winmine.exe is staged. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64WinMine() {
    if !Wow64SupportStaged() {
        return;
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\winmine.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 12] = wstr_arr!("winmine.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let mut Desk: [u16; 16] = wstr_arr!("WinSta0\\Default");
    Si.lpDesktop = Desk.as_mut_ptr();
    if CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    ) == 0
    {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(winmine.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
            GetLastError(),
        );
        return;
    }
    DbgPrint(
        b"[WINLOGON] winmine.exe (STOCK i386 OS applet) created (PID %lu) - watching\n\0"
            .as_ptr() as *const i8,
        Pi.dwProcessId,
    );
    const WAIT_TIMEOUT_: DWORD = 0x102;
    if WaitForSingleObject(Pi.hProcess, 10_000) == WAIT_TIMEOUT_ {
        DbgPrint(
            b"[WINLOGON] winmine.exe still running after 10s = PASS (window up, pump \
              alive) - leaving it up\n\0"
                .as_ptr() as *const i8,
        );
    } else {
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] winmine.exe EXITED = 0x%lx (a stock applet should still be \
              pumping)\n\0"
                .as_ptr() as *const i8,
            ExitCode,
        );
    }
    CloseHandle(Pi.hThread);
    CloseHandle(Pi.hProcess);
}

/* wow64win probe (M4): a real i386 GUI guest with kernel32+gdi32 imports that
 * draws on the host desktop through the win32k thunk table (direct display
 * DC, no user-mode callbacks). Same contract as hello32: silent no-op unless
 * staged; verdict = [gui32]/[GUEST32] serial lines + exit 0x600D. */
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn TryStartWow64Gui() {
    if !Wow64SupportStaged() {
        return; // FEX CPU layer absent — don't spawn a guest that can't run
    }
    let mut Path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    if GetWindowsDirectoryW(Path.as_mut_ptr(), MAX_PATH - 32) == 0 {
        return;
    }
    wcscat(Path.as_mut_ptr(), wstr!("\\syswow64\\gui32.exe").as_ptr());
    if GetFileAttributesW(Path.as_ptr()) == 0xFFFF_FFFF {
        return; // not staged — silent no-op
    }

    let mut Line: [u16; 10] = wstr_arr!("gui32.exe");
    let mut Si: STARTUPINFOW = zeroed();
    let mut Pi: PROCESS_INFORMATION = zeroed();
    Si.cb = size_of::<STARTUPINFOW>() as DWORD;
    let ok = CreateProcessW(
        Path.as_ptr(),
        Line.as_mut_ptr(),
        null_mut(),
        null_mut(),
        0,
        0,
        null_mut(),
        null(),
        &mut Si,
        &mut Pi,
    );
    if ok != 0 {
        DbgPrint(
            b"[WINLOGON] gui32.exe (i386 GUI guest) created (PID %lu) - waiting\n\0".as_ptr()
                as *const i8,
            Pi.dwProcessId,
        );
        WaitForSingleObject(Pi.hProcess, 60_000);
        let mut ExitCode: DWORD = 0xDEAD;
        GetExitCodeProcess(Pi.hProcess, &mut ExitCode);
        DbgPrint(
            b"[WINLOGON] gui32.exe exit = 0x%lx (0x600D = PASS)\n\0".as_ptr() as *const i8,
            ExitCode,
        );
        CloseHandle(Pi.hThread);
        CloseHandle(Pi.hProcess);
    } else {
        DbgPrint(
            b"[WINLOGON] CreateProcessW(gui32.exe) FAILED (error %lu)\n\0".as_ptr()
                as *const i8,
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
        /* arm64 still skips winlogon's OWN RPC server, notifications and GINA
         * (nothing on arm64 consumes the winlogon RPC interface yet) — but NOT
         * the SCM, and NOT lsass.
         *
         * services.exe is the Service Control Manager, and without it arm64 has
         * no CreateService/StartService at all: that is what blocked the kmtest
         * conformance sweep, where every test is advapi32 -> RPC over
         * \pipe\ntsvcs -> services.exe and so raised RPC_S_SERVER_UNAVAILABLE
         * (0x6BA) before printing a line. Its whole import set and npfs.sys (the
         * pipe transport) are staged on the ramdisk now.
         *
         * lsass MUST come with it, and is not optional: ScmCreateServiceDatabase
         * ends with ScmWaitForLsa(), which is a
         * WaitForSingleObject(LSA_RPC_SERVER_ACTIVE, INFINITE) — the event only
         * lsass.exe ever signals (services-rust/src/services.rs:253). Starting
         * the SCM WITHOUT lsass therefore hangs services.exe inside
         * ScmCreateServiceDatabase forever (measured 2026-07-15: the SCM reached
         * "LastKnownGood OK" and then sat on that INFINITE wait for 420s+, so
         * SC_AutoStartComplete never fired and the kmtest sweep timed out every
         * test). x86 has never hit it because its branch below starts lsass.
         *
         * All three (SCM, lsass, and the RPC/pipe transport under them) are
         * non-fatal exactly like the x86 branch: if any fails we log and boot on,
         * so the desktop North Star cannot regress behind them.
         *
         * DELIBERATE divergence from x86: we do NOT call WaitForLsass() after
         * StartLsass(). services.exe already does its own ScmWaitForLsa, so
         * nothing needs winlogon to block here — and making winlogon wait on
         * lsass would put a brand-new INFINITE wait directly on the arm64
         * boot-to-desktop path (the very failure mode above, moved into the
         * process that OWNS the boot). Let the SCM do the waiting. */
        DbgPrint(
            b"[WINLOGON] ARM64: Skipping RPC, notifications, GINA\n\0".as_ptr() as *const i8,
        );
        DbgPrint(b"[WINLOGON] ARM64: Calling StartServicesManager\n\0".as_ptr() as *const i8);
        if StartServicesManager() == 0 {
            DbgPrint(
                b"[WINLOGON] StartServicesManager failed (non-fatal)\n\0".as_ptr() as *const i8,
            );
        } else {
            DbgPrint(b"[WINLOGON] StartServicesManager OK\n\0".as_ptr() as *const i8);
        }

        DbgPrint(b"[WINLOGON] ARM64: Calling StartLsass\n\0".as_ptr() as *const i8);
        if StartLsass() == 0 {
            DbgPrint(
                b"[WINLOGON] StartLsass failed (error %lu, non-fatal)\n\0".as_ptr() as *const i8,
                GetLastError(),
            );
        } else {
            DbgPrint(b"[WINLOGON] ARM64: StartLsass OK\n\0".as_ptr() as *const i8);
        }
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
        // cmd.exe below always run. The WoW64 (i386) doom lives in syswow64,
        // so it needs its own check.
        #[cfg(target_arch = "aarch64")]
        let DoomOwnsTheDesktop = DoomStaged() || Wow64DoomStaged();
        #[cfg(not(target_arch = "aarch64"))]
        let DoomOwnsTheDesktop = DoomStaged();
        if !DoomOwnsTheDesktop {
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

        /* FEX rung-2 hello-compute probe (no-op unless fexprobe.exe staged) —
         * before the cmd.exe INFINITE wait so its [FEX-HELLO] serial lands. */
        TryStartFexProbe();

        /* WOW64 Phase B probe (no-op unless wow64probe.exe staged). */
        TryStartWow64Probe();

        /* WOW64 Phase C probe: real i386 PE through CreateProcessW
         * (no-op unless syswow64\hello32.exe staged). */
        TryStartWow64Hello();

        /* wow64win probe (M4): i386 GUI guest drawing through the win32k
         * thunk table (no-op unless syswow64\gui32.exe staged). */
        TryStartWow64Gui();

        /* M8: window-creation guest AFTER the direct-DC guest (gui32 is the
         * regression oracle; win32 is the frontier). */
        TryStartWow64Win32();

        /* M10: file-I/O guest — the IO_STATUS_BLOCK marshalling oracle. */
        TryStartWow64File();

        /* M15b: overlapped-I/O guest — kernel-side IOSB32 completion oracle. */
        TryStartWow64Async();

        /* M18: marshaller-verification guest (TBI/IOCP/monitors). */
        TryStartWow64Marshal();

        /* E18: guest-SEH dispatch verify — a hardware AV in guest code must
         * reach the guest's own fs:[0] handler. */
        TryStartWow64Fault();

        /* #64: 32-bit guest threading, driven through FEX's BTCpu contract —
         * the arm64 side of the per-thread WoW64 entry. */
        TryStartWow64Thread();

        /* E17: runtime code generation — the CPU layer's memory-notification
         * contract. Most rungs fail silently wrong, not loudly. */
        TryStartWow64Selfmod();

        /* Registry redirection — the Wow6432Node view, checked from inside a
         * 32-bit process. The only ladder rung whose failure is silent. */
        TryStartWow64Reg();

        /* Guest process creation. MOVED AHEAD of the stress guests 2026-09-19,
         * when it started passing and joined the graded parity set: it used to
         * run dead last because it bugchecked the kernel reliably, and anything
         * after it would never report. That reason is gone, and the ordering
         * rule now cuts the other way -- sqlite32 carries the registered
         * BUGCHECK, so a GRADED guest must run BEFORE it or an sqlite32 crash
         * silently costs spawn32 its verdict, which is exactly what the rule
         * above describes happening to lua32. */
        TryStartWow64Spawn();

        /* Third-party stress: real SQLite on the storage + allocator axis.
         * LAST, after every parity guest: it is slow third-party code under
         * emulation, and running it earlier delays the parity set past the
         * ladder's drain budget (that is exactly how it faked a selfmod32
         * failure the first time). The harness extends the drain itself once it
         * sees this guest launch, so being last costs nothing. */
        /* Third-party stress #2: real Lua on the control-flow axis. BEFORE
         * sqlite32, though both follow the parity set -- sqlite32 is the
         * registered arm64 on-disk instability (roadmap: BUGCHECK 0x1E), and a
         * guest that can take the whole boot down must be LAST among the
         * optional guests or its crash silently costs every guest after it its
         * verdict. Measured: with sqlite32 first, an arm64 ladder run graded
         * 9/9 parity PASS and reported nothing at all about lua32, because the
         * machine was gone. lua32 is purely in-memory, so it is not exposed to
         * that bug. */
        TryStartWow64Lua();

        TryStartWow64Sqlite();


        /* The testgui pair never exits, so it would sit on top of DOOM's window.
         * When the WoW64 doom is staged, DOOM is the demo — skip them. */
        if !Wow64DoomStaged() {
            /* M11 A/B control: the NATIVE arm64 build of the very same app, so a
             * failure can be attributed. If native testgui reaches WM_PAINT and
             * the WoW64 one does not, the bug is in the WoW64 layer; if they fail
             * the same way, it is a plain user32 bug and belongs to another lane. */
            TryStartNativeTestGui();

            /* M11: the first STOCK i386 binary — real CRT, real imports, real
             * window. It never exits: it stays up for the screenshot. */
            TryStartWow64TestGui();
        }

        /* M12 — THE NORTH STAR: x86 Doom under WoW64. Last, and alone on the
         * desktop; it never exits. */
        TryStartWow64Doom();

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

    /* WOW64 Phase E probes (amd64 native wow64cpu). E3/E4 rungs that PASS
     * cleanly and run in every amd64 smoke as regression coverage:
     *   hello32 (loader verdict -> 0x600D)
     *   gui32   (win32k GDI thunk: draws the magenta rect -> 0x600e soft-pass)
     * E4b: the window-creation + kernel->user-callback guests. testgui runs
     * always (its window doubles as regression coverage for the callback
     * marshallers); doom is the demo rung, same mutual-occlusion rule as
     * arm64. See wow64-plan.md Phase E. */
    #[cfg(target_arch = "x86_64")]
    {
        /* Full arm64-parity probe ladder (each is a silent no-op unless its
         * guest is staged): loader → GDI → window/callbacks → file I/O →
         * overlapped I/O → marshaller self-verify. */
        TryStartWow64Hello();
        TryStartWow64Gui();
        TryStartWow64Win32();
        TryStartWow64File();
        TryStartWow64Async();
        TryStartWow64Marshal();
        /* E5: guest-SEH dispatch verify (fault + raise + ODS consumption). */
        TryStartWow64Fault();
        /* #64: 32-bit guest threading — create/join, TLS, per-thread SEH,
         * 4 concurrent threads. */
        TryStartWow64Thread();
        /* E17: the guest generates, rewrites, recycles and hot-patches its
         * own code. */
        TryStartWow64Selfmod();
        /* Wow6432Node redirection, isolation and the two-view delete. */
        TryStartWow64Reg();

        /* Guest process creation. MOVED AHEAD of the stress guests 2026-09-19,
         * when it started passing and joined the graded parity set: it used to
         * run dead last because it bugchecked the kernel reliably, and anything
         * after it would never report. That reason is gone, and the ordering
         * rule now cuts the other way -- sqlite32 carries the registered
         * BUGCHECK, so a GRADED guest must run BEFORE it or an sqlite32 crash
         * silently costs spawn32 its verdict, which is exactly what the rule
         * above describes happening to lua32. */
        TryStartWow64Spawn();
        /* Third-party stress: real SQLite, last — see the note at the other
         * call site on why it must not precede the parity guests. */
        /* Third-party stress #2: real Lua, setjmp/longjmp-heavy. BEFORE
         * sqlite32 — see the note at the other call site on why the guest that
         * can BUGCHECK must go last. */
        TryStartWow64Lua();
        TryStartWow64Sqlite();

        /* Wine's own conformance suites, under WoW64. Every guest above this
         * line is one we wrote or one we chose; these are the ORACLE, and they
         * assert against real Windows behaviour rather than against what our
         * implementation happens to do. Two subtests to start, picked to
         * separate the two things that could be wrong:
         *   wsprintf — almost no win32k surface, so a failure here is the
         *              runner, the pipe or the CRT, not WoW64
         *   class    — RegisterClass/GetClassInfo/GetClassLong, i.e. real
         *              win32k round-trips with 32<->64 handle and LONG_PTR
         *              width conversion in the middle
         * Silent no-op unless the i386 rostests build staged them. */
        TryStartWow64Winetest(
            wstr!("user32_winetest.exe").as_ptr(),
            wstr!("wsprintf").as_ptr(),
            b"user32:wsprintf\0".as_ptr() as *const i8,
            60_000,
        );
        TryStartWow64Winetest(
            wstr!("user32_winetest.exe").as_ptr(),
            wstr!("class").as_ptr(),
            b"user32:class\0".as_ptr() as *const i8,
            60_000,
        );

        /* E4c robustness: a stock i386 OS applet nobody wrote for this test —
         * menus, timers, dialog resources. Launched BEFORE doom so doom's
         * window (created last) stays the demo on top. */
        TryStartWow64WinMine();
        /* E6 robustness ladder: more stock i386 applets (common controls,
         * shell32/comctl32 closure). Each is a silent no-op unless staged. */
        TryStartWow64StockApp(
            wstr!("calc.exe").as_ptr(),
            b"calc.exe\0".as_ptr() as *const i8,
            10_000,
        );
        TryStartWow64StockApp(
            wstr!("notepad.exe").as_ptr(),
            b"notepad.exe\0".as_ptr() as *const i8,
            10_000,
        );
        TryStartWow64StockApp(
            wstr!("sol.exe").as_ptr(),
            b"sol.exe\0".as_ptr() as *const i8,
            10_000,
        );
        /* E8 robustness ladder: more stock i386 applets, each exercising NEW
         * win32k/GDI surface a prior guest never reached:
         *   charmap — EnumFontFamilies font-enumeration arrays + callback
         *   mspaint — heavy GDI (regions/DIBs/brushes) + comdlg32 dialogs
         *   regedit — treeview/listview common controls + registry-enum arrays
         *   winver  — shell32 ShellAbout modal dialog
         * Silent no-op unless staged. 8s watch keeps total ladder time bounded. */
        TryStartWow64StockApp(
            wstr!("charmap.exe").as_ptr(),
            b"charmap.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("mspaint.exe").as_ptr(),
            b"mspaint.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("regedit.exe").as_ptr(),
            b"regedit.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("winver.exe").as_ptr(),
            b"winver.exe\0".as_ptr() as *const i8,
            8_000,
        );
        /* E9 robustness ladder: taskmgr (listview + NtQuerySystemInformation
         * process array), wordpad (richedit control), clipbrd (clipboard). */
        TryStartWow64StockApp(
            wstr!("taskmgr.exe").as_ptr(),
            b"taskmgr.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("wordpad.exe").as_ptr(),
            b"wordpad.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("clipbrd.exe").as_ptr(),
            b"clipbrd.exe\0".as_ptr() as *const i8,
            8_000,
        );
        /* osk (NtUserSendInput path) + magnify (timer-driven screen-capture
         * BitBlt from a NULL/screen DC — a distinct GDI surface). */
        TryStartWow64StockApp(
            wstr!("osk.exe").as_ptr(),
            b"osk.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("magnify.exe").as_ptr(),
            b"magnify.exe\0".as_ptr() as *const i8,
            8_000,
        );
        /* E11 ladder — LIVE again as of E13 (2026-07-24). These six were staged
         * but held back because two of them (spider, eventvwr) BUGCHECKed the
         * box: win32k's g_MsgMemory sized SendMessage pointer-lParam payloads
         * with the HOST's size_of::<T>(), over-reading a 32-bit sender's buffer
         * -> 0x1E/C0000005 at win32k+0x258EC. E13 fixed that in the wow64 layer
         * (Wow64Manual_NtUserMessageCall marshals the payload AND the ResultInfo
         * out-slot), so this block is now exactly its regression test: spider
         * and eventvwr reach the old fault ~6 applets earlier than doom does.
         *
         * spider     — cardlib DIB blitting (also the disproof rung for E10's
         *              NtGdiCreateDIBSection fix)
         * eventvwr   — listview + the first 32-bit guest to drive real RPC to a
         *              64-bit service; owner-draw is what found E13
         * devmgmt    — setupapi device enumeration
         * msconfig   — comctl32 property sheets
         * sndrec32   — winmm
         * kbswitch   — shell tray-icon IPC */
        TryStartWow64StockApp(
            wstr!("spider.exe").as_ptr(),
            b"spider.exe\0".as_ptr() as *const i8,
            8_000,
        );
        TryStartWow64StockApp(
            wstr!("eventvwr.exe").as_ptr(),
            b"eventvwr.exe\0".as_ptr() as *const i8,
            8_000,
        );
        /* devmgmt / msconfig / sndrec32 / kbswitch are STAGED but still held.
         * With E13 fixed the ladder reached devmgmt for the FIRST time ever —
         * and it BUGCHECKs 0x1E/C0000005 there (task #67). That is a NEW bug on
         * newly-reachable surface, not an E13 regression: the old E13 signature
         * (raw_move <- memcpy <- g_MsgMemory <- co_IntDoSendMessage) is now
         * absent from the log entirely, and spider/eventvwr — the two that used
         * to kill the box — both PASS.
         *
         * Same coverage argument as before: devmgmt sits ahead of testgui/doom,
         * so auto-launching it would stop the sixteen rungs that now pass from
         * ever reporting. Run `devmgmt.exe` from reactos\syswow64 by hand to
         * repro. Re-enabling these four is #67's regression test.
         *
         * for app in [devmgmt, msconfig, sndrec32, kbswitch]:
         *     TryStartWow64StockApp(app, label, 8_000);
         */
        /* E4b: same mutual-occlusion rule as the arm64 ladder — when doom is
         * staged it IS the demo GUI guest; testgui would materialise on top
         * of its window. */
        if !Wow64DoomStaged() {
            TryStartWow64TestGui();
        }
        TryStartWow64Doom();
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
