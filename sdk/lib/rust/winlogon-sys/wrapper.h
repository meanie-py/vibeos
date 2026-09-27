/* winlogon-sys bindgen wrapper -- the FFI substrate for the winlogon.exe
 * C->Rust leaf-file translation (security/environment/shutdown/setup/
 * screensaver). User-mode analog of ntdll-sys, scoped to what winlogon
 * needs: the Win32 GUI/registry/security API surface + the GINA
 * (winwlx.h) types + winlogon's OWN private structs (WLSESSION,
 * GINAINSTANCE, GINAFUNCTIONS, LOGON_STATE, NOTIFICATION_TYPE).
 *
 * WLSESSION is the crux: the still-C winlogon core owns the `WLSession`
 * global, so every flipped leaf must mirror its layout EXACTLY. The
 * structs are copied verbatim from base/system/winlogon/winlogon.h
 * (which can't be #included wholesale here -- it pulls pseh2/wine-debug/
 * resource.h). The size_check_*.rs asserts (verify_sizes.py) fail the
 * build if any copy drifts from the C layout.
 *
 * USE_GETLASTINPUTINFO is NOT defined in the winlogon build (verified
 * against the build-<arch> build.ninja DEFINES), so KeyboardHook/MouseHook/
 * LastActivity fields ARE present in WLSESSION -- the copy below keeps
 * them (matching the source default).
 *
 * Uses -fms-extensions but NEVER -fms-compatibility (mingw _mingw.h
 * takes the MSVC branch under _MSC_VER and breaks math.h).
 */
#define __REACTOS__
#define _USE_DUMMY_PSEH 1
/* Skip compiler intrinsics -- not available in libclang. */
#define _INTRIN_H_
#define _INTRIN_X86_H_
#ifndef __int64
#define __int64 long long
#endif
#undef FORCEINLINE
#define FORCEINLINE

/* vcruntime/vadefs.h (pulled by <stdio.h>) references __gnuc_va_list
   without including stdarg itself; the lifted -nostdinc flags + the
   ROS sdk/include/crt <stdarg.h> shadowing clang's drop the implicit
   setup. __builtin_va_list is a true clang builtin, always available. */
typedef __builtin_va_list __gnuc_va_list;

#include <stdio.h>

/* Win32 base headers -- registry/process/desktop/window/security API
   that the leaves call, plus the base types (HWND/HDESK/HWINSTA/LUID/
   HANDLE/LARGE_INTEGER) that WLSESSION embeds. */
#define WIN32_NO_STATUS
#define _INC_WINDOWS
#define COM_NO_WINDOWS_H
#include <windef.h>
#include <winbase.h>
#include <wingdi.h>
#include <winuser.h>
#include <winreg.h>

/* GINA interface: WLX_PROFILE_V2_0, WLX_MPR_NOTIFY_INFO, WLX_VERSION_*,
   the PWLX_* pointer aliases the PFWLX* fn-ptr typedefs reference. */
#include <winwlx.h>

/* NT-internal umbrella in USER mode -- Rtl/Nt/Zw + native types for
   the security.c and sas.c leaves (LSA, NtRaiseHardError, etc.). */
#define NTOS_MODE_USER
#include <ndk/rtlfuncs.h>
#include <ndk/exfuncs.h>
#include <ndk/kefuncs.h>

/* Undocumented user32/mpr surface winlogon uses. */
#include <undocuser.h>
#include <undocmpr.h>

/* --- winlogon.exe private structs, copied verbatim from
 * base/system/winlogon/winlogon.h (the C core owns `WLSession`; the
 * flipped leaves read it through these). size_check_*.rs validates. --- */

typedef BOOL (WINAPI * PFWLXNEGOTIATE)  (DWORD, DWORD *);
typedef BOOL (WINAPI * PFWLXINITIALIZE) (LPWSTR, HANDLE, PVOID, PVOID, PVOID *);
typedef VOID (WINAPI * PFWLXDISPLAYSASNOTICE) (PVOID);
typedef int  (WINAPI * PFWLXLOGGEDOUTSAS) (PVOID, DWORD, PLUID, PSID, PDWORD,
                                           PHANDLE, PWLX_MPR_NOTIFY_INFO,
                                           PVOID *);
typedef BOOL (WINAPI * PFWLXACTIVATEUSERSHELL) (PVOID, PWSTR, PWSTR, PVOID);
typedef int  (WINAPI * PFWLXLOGGEDONSAS) (PVOID, DWORD, PVOID);
typedef VOID (WINAPI * PFWLXDISPLAYLOCKEDNOTICE) (PVOID);
typedef int  (WINAPI * PFWLXWKSTALOCKEDSAS) (PVOID, DWORD);
typedef BOOL (WINAPI * PFWLXISLOCKOK) (PVOID);
typedef BOOL (WINAPI * PFWLXISLOGOFFOK) (PVOID);
typedef VOID (WINAPI * PFWLXLOGOFF) (PVOID);
typedef VOID (WINAPI * PFWLXSHUTDOWN) (PVOID, DWORD);
typedef BOOL (WINAPI * PFWLXSCREENSAVERNOTIFY) (PVOID, BOOL *);
typedef BOOL (WINAPI * PFWLXSTARTAPPLICATION) (PVOID, PWSTR, PVOID, PWSTR);
typedef BOOL (WINAPI * PFWLXNETWORKPROVIDERLOAD) (PVOID, PWLX_MPR_NOTIFY_INFO);
typedef BOOL (WINAPI * PFWLXDISPLAYSTATUSMESSAGE) (PVOID, HDESK, DWORD, PWSTR, PWSTR);
typedef BOOL (WINAPI * PFWLXGETSTATUSMESSAGE) (PVOID, DWORD *, PWSTR, DWORD);
typedef BOOL (WINAPI * PFWLXREMOVESTATUSMESSAGE) (PVOID);

typedef struct _GINAFUNCTIONS
{
    PFWLXNEGOTIATE            WlxNegotiate;
    PFWLXINITIALIZE           WlxInitialize;
    PFWLXDISPLAYSASNOTICE     WlxDisplaySASNotice;
    PFWLXLOGGEDOUTSAS         WlxLoggedOutSAS;
    PFWLXACTIVATEUSERSHELL    WlxActivateUserShell;
    PFWLXLOGGEDONSAS          WlxLoggedOnSAS;
    PFWLXDISPLAYLOCKEDNOTICE  WlxDisplayLockedNotice;
    PFWLXWKSTALOCKEDSAS       WlxWkstaLockedSAS;
    PFWLXISLOCKOK             WlxIsLockOk;
    PFWLXISLOGOFFOK           WlxIsLogoffOk;
    PFWLXLOGOFF               WlxLogoff;
    PFWLXSHUTDOWN             WlxShutdown;
    PFWLXSCREENSAVERNOTIFY    WlxScreenSaverNotify;
    PFWLXSTARTAPPLICATION     WlxStartApplication;
    PFWLXNETWORKPROVIDERLOAD  WlxNetworkProviderLoad;
    PFWLXDISPLAYSTATUSMESSAGE WlxDisplayStatusMessage;
    PFWLXGETSTATUSMESSAGE     WlxGetStatusMessage;
    PFWLXREMOVESTATUSMESSAGE  WlxRemoveStatusMessage;
} GINAFUNCTIONS, *PGINAFUNCTIONS;

typedef struct _GINAINSTANCE
{
    HMODULE hDllInstance;
    GINAFUNCTIONS Functions;
    PVOID Context;
    DWORD Version;
    BOOL UseCtrlAltDelete;
} GINAINSTANCE, *PGINAINSTANCE;

typedef enum _LOGON_STATE
{
    STATE_INIT,
    STATE_LOGGED_OFF,
    STATE_LOGGED_OFF_SAS,
    STATE_LOGGED_ON,
    STATE_LOGGED_ON_SAS,
    STATE_LOCKED,
    STATE_LOCKED_SAS,
    STATE_LOGGING_OFF,
    STATE_SHUTTING_DOWN,
    STATE_SHUT_DOWN,
    STATE_SCREENSAVER
} LOGON_STATE, *PLOGON_STATE;

typedef struct _WLSESSION
{
    GINAINSTANCE Gina;
    DWORD SASAction;
    BOOL SuppressStatus;
    BOOL TaskManHotkey;
    BOOL LockWkStaHotkey;
    BOOL UtilManHotkey;
    HWND SASWindow;
    HWINSTA InteractiveWindowStation;
    PWSTR InteractiveWindowStationName;
    PWSTR UserName;
    PWSTR Domain;
    HDESK ApplicationDesktop;
    HDESK WinlogonDesktop;
    HDESK ScreenSaverDesktop;
    LUID LogonId;
    HANDLE UserToken;
    HANDLE hProfileInfo;
    LOGON_STATE LogonState;
    DWORD DialogTimeout;
    LARGE_INTEGER LastLogon;
    HHOOK KeyboardHook;
    HHOOK MouseHook;
    HANDLE hEndOfScreenSaverThread;
    HANDLE hScreenSaverParametersChanged;
    HANDLE hUserActivity;
    HANDLE hEndOfScreenSaver;
    DWORD LastActivity;
    DWORD Options;
    WLX_MPR_NOTIFY_INFO MprNotifyInfo;
    PWLX_PROFILE_V2_0 Profile;
} WLSESSION, *PWLSESSION;

typedef enum _NOTIFICATION_TYPE
{
    LogonHandler,
    LogoffHandler,
    LockHandler,
    UnlockHandler,
    StartupHandler,
    ShutdownHandler,
    StartScreenSaverHandler,
    StopScreenSaverHandler,
    DisconnectHandler,
    ReconnectHandler,
    StartShellHandler,
    PostShellHandler,
    LastHandler
} NOTIFICATION_TYPE, *PNOTIFICATION_TYPE;
