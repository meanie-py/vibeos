/*
 * COPYRIGHT:       See COPYING in the top level directory
 * PROJECT:         ReactOS Winlogon
 * FILE:            base/system/winlogon/winlogon.c
 * PURPOSE:         Logon
 * PROGRAMMERS:     Thomas Weidenmueller (w3seek@users.sourceforge.net)
 *                  Filip Navara
 *                  Hervé Poussineau (hpoussin@reactos.org)
 */

/* INCLUDES *****************************************************************/

#include "winlogon.h"

#include <ndk/cmfuncs.h>
#include <ndk/obfuncs.h>

/* GLOBALS ******************************************************************/

HINSTANCE hAppInstance;
PWLSESSION WLSession = NULL;

/* FUNCTIONS *****************************************************************/

/**
 * @brief
 * Duplicates the given string, allocating a buffer on the heap.
 **/
PWSTR
WlStrDup(
    _In_opt_ PCWSTR String)
{
    PWSTR ptr;

    if (!String)
        return NULL;

    ptr = RtlAllocateHeap(RtlGetProcessHeap(), 0,
                          (wcslen(String) + 1) * sizeof(WCHAR));
    if (ptr)
        wcscpy(ptr, String);
    return ptr;
}


static
BOOL
StartServicesManager(VOID)
{
    STARTUPINFOW StartupInfo = { sizeof(StartupInfo) };
    PROCESS_INFORMATION ProcessInformation;
    LPCWSTR ServiceString = L"services.exe";
    BOOL res;

    /* Start the service control manager (services.exe) */
    ZeroMemory(&StartupInfo, sizeof(STARTUPINFOW));
    StartupInfo.cb = sizeof(StartupInfo);
    StartupInfo.lpReserved = NULL;
    StartupInfo.lpDesktop = NULL;
    StartupInfo.lpTitle = NULL;
    StartupInfo.dwFlags = 0;
    StartupInfo.cbReserved2 = 0;
    StartupInfo.lpReserved2 = 0;

    DbgPrint("[WINLOGON] StartServicesManager: calling CreateProcessW(services.exe)\n");

    res = CreateProcessW(ServiceString,
                         NULL,
                         NULL,
                         NULL,
                         FALSE,
                         DETACHED_PROCESS,
                         NULL,
                         NULL,
                         &StartupInfo,
                         &ProcessInformation);
    if (!res)
    {
        DbgPrint("[WINLOGON] StartServicesManager: CreateProcessW FAILED (error %lu)\n", GetLastError());
        return FALSE;
    }

    DbgPrint("[WINLOGON] StartServicesManager: CreateProcessW OK (PID=%lu)\n",
             ProcessInformation.dwProcessId);

    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    TRACE("WL: StartServicesManager() done.\n");

    return TRUE;
}


static
BOOL
StartLsass(VOID)
{
    STARTUPINFOW StartupInfo = { sizeof(StartupInfo) };
    PROCESS_INFORMATION ProcessInformation;
    LPCWSTR ServiceString = L"lsass.exe";
    BOOL res;

    /* Start the local security authority subsystem (lsass.exe) */
    ZeroMemory(&StartupInfo, sizeof(STARTUPINFOW));
    StartupInfo.cb = sizeof(StartupInfo);
    StartupInfo.lpReserved = NULL;
    StartupInfo.lpDesktop = NULL;
    StartupInfo.lpTitle = NULL;
    StartupInfo.dwFlags = 0;
    StartupInfo.cbReserved2 = 0;
    StartupInfo.lpReserved2 = 0;

    DbgPrint("[WINLOGON] StartLsass: calling CreateProcessW(lsass.exe)\n");

    res = CreateProcessW(ServiceString,
                         NULL,
                         NULL,
                         NULL,
                         FALSE,
                         DETACHED_PROCESS,
                         NULL,
                         NULL,
                         &StartupInfo,
                         &ProcessInformation);

    if (res)
        DbgPrint("[WINLOGON] StartLsass: CreateProcessW OK (PID=%lu)\n",
                 ProcessInformation.dwProcessId);
    else
        DbgPrint("[WINLOGON] StartLsass: CreateProcessW FAILED (error %lu)\n",
                 GetLastError());

    CloseHandle(ProcessInformation.hThread);
    CloseHandle(ProcessInformation.hProcess);

    return res;
}


static
VOID
WaitForLsass(VOID)
{
    HANDLE hEvent;
    DWORD dwError;

    hEvent = CreateEventW(NULL,
                          TRUE,
                          FALSE,
                          L"LSA_RPC_SERVER_ACTIVE");
    if (hEvent == NULL)
    {
        dwError = GetLastError();
        TRACE("WL: Failed to create the notification event (Error %lu)\n", dwError);

        if (dwError == ERROR_ALREADY_EXISTS)
        {
            hEvent = OpenEventW(SYNCHRONIZE,
                                FALSE,
                                L"LSA_RPC_SERVER_ACTIVE");
            if (hEvent == NULL)
            {
               ERR("WL: Could not open the notification event (Error %lu)\n", GetLastError());
               return;
            }
        }
    }

    TRACE("WL: Wait for the LSA server!\n");
    WaitForSingleObject(hEvent, INFINITE);
    TRACE("WL: LSA server running!\n");

    CloseHandle(hEvent);
}


static
VOID
UpdateTcpIpInformation(VOID)
{
    LONG lError;
    HKEY hKey = NULL;
    DWORD dwType, dwSize;
    PWSTR pszBuffer;
    WCHAR szBuffer[128] = L"";

    lError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
                           L"SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters",
                           0,
                           KEY_QUERY_VALUE | KEY_SET_VALUE,
                           &hKey);
    if (lError != ERROR_SUCCESS)
    {
        ERR("WL: RegOpenKeyExW(\"HKLM\\System\\CurrentControlSet\\Services\\Tcpip\\Parameters\") failed (error %lu)\n", lError);
        return;
    }

    /*
     * Read the "NV Hostname" value and copy it into the "Hostname" value.
     */

    pszBuffer = szBuffer;
    dwSize = ARRAYSIZE(szBuffer);

    lError = RegQueryValueExW(hKey,
                              L"NV Hostname",
                              NULL,
                              &dwType,
                              (LPBYTE)pszBuffer,
                              &dwSize);
    if (((lError == ERROR_INSUFFICIENT_BUFFER) || (lError == ERROR_MORE_DATA)) && (dwType == REG_SZ))
    {
        pszBuffer = HeapAlloc(GetProcessHeap(), 0, dwSize);
        if (pszBuffer)
        {
            lError = RegQueryValueExW(hKey,
                                      L"NV Hostname",
                                      NULL,
                                      &dwType,
                                      (LPBYTE)pszBuffer,
                                      &dwSize);
        }
        else
        {
            ERR("WL: Could not reallocate memory for pszBuffer\n");
            goto Quit;
        }
    }
    if ((lError == ERROR_SUCCESS) && (dwType == REG_SZ))
    {
        TRACE("NV Hostname is '%S'.\n", pszBuffer);

        lError = RegSetValueExW(hKey,
                                L"Hostname",
                                0,
                                REG_SZ,
                                (LPBYTE)pszBuffer,
                                dwSize);
        if (lError != ERROR_SUCCESS)
            ERR("WL: RegSetValueExW(\"Hostname\") failed (error %lu)\n", lError);
    }

    /*
     * Read the "NV Domain" value and copy it into the "Domain" value.
     */

    // pszBuffer = szBuffer;
    // dwSize = ARRAYSIZE(szBuffer);

    lError = RegQueryValueExW(hKey,
                              L"NV Domain",
                              NULL,
                              &dwType,
                              (LPBYTE)pszBuffer,
                              &dwSize);
    if (((lError == ERROR_INSUFFICIENT_BUFFER) || (lError == ERROR_MORE_DATA)) && (dwType == REG_SZ))
    {
        if (pszBuffer != szBuffer)
        {
            PWSTR pszNewBuffer;
            pszNewBuffer = HeapReAlloc(GetProcessHeap(), 0, pszBuffer, dwSize);
            if (pszNewBuffer)
            {
                pszBuffer = pszNewBuffer;
            }
            else
            {
                HeapFree(GetProcessHeap(), 0, pszBuffer);
                pszBuffer = NULL;
            }
        }
        else
        {
            pszBuffer = HeapAlloc(GetProcessHeap(), 0, dwSize);
        }
        if (pszBuffer)
        {
            lError = RegQueryValueExW(hKey,
                                      L"NV Domain",
                                      NULL,
                                      &dwType,
                                      (LPBYTE)pszBuffer,
                                      &dwSize);
        }
        else
        {
            ERR("WL: Could not reallocate memory for pszBuffer\n");
            goto Quit;
        }
    }
    if ((lError == ERROR_SUCCESS) && (dwType == REG_SZ))
    {
        TRACE("NV Domain is '%S'.\n", pszBuffer);

        lError = RegSetValueExW(hKey,
                                L"Domain",
                                0,
                                REG_SZ,
                                (LPBYTE)pszBuffer,
                                dwSize);
        if (lError != ERROR_SUCCESS)
            ERR("WL: RegSetValueExW(\"Domain\") failed (error %lu)\n", lError);
    }

    if (pszBuffer != szBuffer)
        HeapFree(GetProcessHeap(), 0, pszBuffer);

Quit:
    RegCloseKey(hKey);
}


static
BOOL
InitKeyboardLayouts(VOID)
{
    WCHAR wszKeyName[12], wszKLID[10];
    DWORD dwSize = sizeof(wszKLID), dwType, i = 1;
    HKEY hKey;
    UINT Flags;
    BOOL bRet = FALSE;

    /* Open registry key with preloaded layouts */
    if (RegOpenKeyExW(HKEY_CURRENT_USER, L"Keyboard Layout\\Preload", 0, KEY_READ, &hKey) == ERROR_SUCCESS)
    {
        while (TRUE)
        {
            /* Read values with integer names only */
            _swprintf(wszKeyName, L"%d", i++);
            if (RegQueryValueExW(hKey, wszKeyName, NULL, &dwType, (LPBYTE)wszKLID, &dwSize) != ERROR_SUCCESS)
            {
                /* There is no more entries */
                break;
            }

            /* Only REG_SZ values are valid */
            if (dwType != REG_SZ)
            {
                ERR("Wrong type: %ws!\n", wszKLID);
                continue;
            }

            /* Load keyboard layout with given locale id */
            Flags = KLF_SUBSTITUTE_OK;
            if (i > 1)
                Flags |= KLF_NOTELLSHELL|KLF_REPLACELANG;
            else // First layout
                Flags |= KLF_ACTIVATE; // |0x40000000
            if (!LoadKeyboardLayoutW(wszKLID, Flags))
            {
                ERR("LoadKeyboardLayoutW(%ws) failed!\n", wszKLID);
                continue;
            }
            else
            {
                /* We loaded at least one layout - success */
                bRet = TRUE;
            }
        }

        /* Close the key now */
        RegCloseKey(hKey);
    }
    else
        WARN("RegOpenKeyExW(Keyboard Layout\\Preload) failed!\n");

    if (!bRet)
    {
        /* If we failed, load US keyboard layout */
        if (LoadKeyboardLayoutW(L"00000409", KLF_ACTIVATE | KLF_SUBSTITUTE_OK | KLF_REPLACELANG | KLF_SETFORPROCESS))
            bRet = TRUE;
    }

    return bRet;
}


BOOL
DisplayStatusMessage(
     IN PWLSESSION Session,
     IN HDESK hDesktop,
     IN UINT ResourceId)
{
    WCHAR StatusMsg[MAX_PATH];

    if (Session->Gina.Version < WLX_VERSION_1_3)
        return TRUE;

    if (Session->SuppressStatus)
        return TRUE;

    if (LoadStringW(hAppInstance, ResourceId, StatusMsg, MAX_PATH) == 0)
        return FALSE;

    return Session->Gina.Functions.WlxDisplayStatusMessage(Session->Gina.Context, hDesktop, 0, NULL, StatusMsg);
}


BOOL
RemoveStatusMessage(
    IN PWLSESSION Session)
{
    if (Session->Gina.Version < WLX_VERSION_1_3)
        return TRUE;

    return Session->Gina.Functions.WlxRemoveStatusMessage(Session->Gina.Context);
}


static
INT_PTR
CALLBACK
GinaLoadFailedWindowProc(
    IN HWND hwndDlg,
    IN UINT uMsg,
    IN WPARAM wParam,
    IN LPARAM lParam)
{
    switch (uMsg)
    {
        case WM_COMMAND:
        {
            switch (LOWORD(wParam))
            {
                case IDOK:
                    EndDialog(hwndDlg, IDOK);
                    return TRUE;
            }
            break;
        }

        case WM_INITDIALOG:
        {
            int len;
            WCHAR templateText[MAX_PATH], text[MAX_PATH];

            len = GetDlgItemTextW(hwndDlg, IDC_GINALOADFAILED, templateText, MAX_PATH);
            if (len)
            {
                wsprintfW(text, templateText, (LPWSTR)lParam);
                SetDlgItemTextW(hwndDlg, IDC_GINALOADFAILED, text);
            }
            return TRUE;
        }

        case WM_CLOSE:
        {
            EndDialog(hwndDlg, IDCANCEL);
            return TRUE;
        }
    }

    return FALSE;
}


static DWORD WINAPI
FontLoadThread(LPVOID lpParam)
{
    static LPCWSTR FontNames[] = {
        L"\\Fonts\\tahoma.ttf",
        L"\\Fonts\\tahomabd.ttf",
        L"\\Fonts\\Marlett.ttf",
        L"\\Fonts\\arial.ttf",
        L"\\Fonts\\arialbd.ttf",
        L"\\Fonts\\cour.ttf",
        L"\\Fonts\\times.ttf",
        L"\\Fonts\\symbol.ttf",
    };
    WCHAR WinDir[MAX_PATH];
    WCHAR FontPath[MAX_PATH];
    DWORD i, Len;
    UNREFERENCED_PARAMETER(lpParam);

    Len = GetWindowsDirectoryW(WinDir, MAX_PATH);
    DbgPrint("[WINLOGON] FontLoadThread: WindowsDirectory=%S len=%lu\n", WinDir, Len);
    if (Len == 0 || Len >= MAX_PATH - 40)
    {
        DbgPrint("[WINLOGON] FontLoadThread: GetWindowsDirectoryW failed\n");
        return 1;
    }
    for (i = 0; i < sizeof(FontNames)/sizeof(FontNames[0]); i++)
    {
        wcscpy(FontPath, WinDir);
        wcscat(FontPath, FontNames[i]);
        if (AddFontResourceW(FontPath))
            DbgPrint("[WINLOGON] Loaded font: %S\n", FontPath);
        else
            DbgPrint("[WINLOGON] Failed to load font: %S (err %lu)\n", FontPath, GetLastError());
    }
    DbgPrint("[WINLOGON] Font loading complete (%d fonts)\n",
             (int)(sizeof(FontNames)/sizeof(FontNames[0])));
    return 0;
}

int
WINAPI
WinMain(
    IN HINSTANCE hInstance,
    IN HINSTANCE hPrevInstance,
    IN LPSTR lpCmdLine,
    IN int nShowCmd)
{
#if 0
    LSA_STRING ProcessName, PackageName;
    HANDLE LsaHandle;
    LSA_OPERATIONAL_MODE Mode;
    BOOLEAN Old;
    ULONG AuthenticationPackage;
    NTSTATUS Status;
#endif
    ULONG HardErrorResponse;
#ifndef _M_ARM64
    MSG Msg;
#endif

    UNREFERENCED_PARAMETER(hPrevInstance);
    UNREFERENCED_PARAMETER(lpCmdLine);
    UNREFERENCED_PARAMETER(nShowCmd);

    hAppInstance = hInstance;

#ifdef _M_ARM64
    DbgPrint("[WINLOGON] ARM64: WinMain entered\n");
#endif
    DbgPrint("[WINLOGON] WinMain entered (PID=%lu)\n", GetCurrentProcessId());

    /* Make us critical */
    RtlSetProcessIsCritical(TRUE, NULL, FALSE);
    RtlSetThreadIsCritical(TRUE, NULL, FALSE);

    /* Update the cached TCP/IP Information in the registry */
    UpdateTcpIpInformation();

    DbgPrint("[WINLOGON] Calling RegisterLogonProcess(%lu)\n", GetCurrentProcessId());
    if (!RegisterLogonProcess(GetCurrentProcessId(), TRUE))
    {
#ifdef _M_ARM64
        DbgPrint("[WINLOGON] ARM64: RegisterLogonProcess failed (error %lu), continuing\n", GetLastError());
#elif defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
        DbgPrint("[WINLOGON] RegisterLogonProcess failed (error %lu), continuing\n", GetLastError());
#else
        ERR("WL: Could not register logon process (error %lu)\n", GetLastError());
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
#endif
    }

    WLSession = (PWLSESSION)HeapAlloc(GetProcessHeap(), 0, sizeof(WLSESSION));
    if (!WLSession)
    {
#ifdef _M_ARM64
        DbgPrint("[WINLOGON] ARM64: HeapAlloc failed\n");
#endif
        ERR("WL: Could not allocate memory for winlogon instance\n");
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
    }

    ZeroMemory(WLSession, sizeof(WLSESSION));
    WLSession->DialogTimeout = 120; /* 2 minutes */

    /* Initialize the dialog tracking list */
    InitDialogListHead();

#ifdef _M_ARM64
    /*
     * Create \Windows\WindowStations directory if it doesn't exist.
     * Normally created by NtUserInitialize (via winsrv in csrss),
     * but we skip winsrv on ARM64. CreateWindowStationW needs this
     * directory to exist before it can call NtUserCreateWindowStation.
     */
    {
        OBJECT_ATTRIBUTES DirOa;
        UNICODE_STRING DirName;
        HANDLE hWinStaDir;
        NTSTATUS DirStatus;

        RtlInitUnicodeString(&DirName, L"\\Windows\\WindowStations");
        InitializeObjectAttributes(&DirOa, &DirName, OBJ_CASE_INSENSITIVE, NULL, NULL);
        DirStatus = NtCreateDirectoryObject(&hWinStaDir, DIRECTORY_ALL_ACCESS, &DirOa);
        DbgPrint("[WINLOGON] ARM64: NtCreateDirectoryObject(\\Windows\\WindowStations) = 0x%lx\n", DirStatus);
        /* Keep hWinStaDir open — directory must persist */
    }

    DbgPrint("[WINLOGON] ARM64: calling CreateWindowStationAndDesktops\n");
#endif

    DbgPrint("[WINLOGON] Calling CreateWindowStationAndDesktops\n");
    if (!CreateWindowStationAndDesktops(WLSession))
    {
#ifdef _M_ARM64
        DbgPrint("[WINLOGON] ARM64: CreateWindowStationAndDesktops FAILED (error %lu), continuing without desktops\n", GetLastError());
#elif defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
        DbgPrint("[WINLOGON] CreateWindowStationAndDesktops FAILED (error %lu), continuing\n", GetLastError());
#else
        ERR("WL: Could not create window station and desktops (error %lu)\n", GetLastError());
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
#endif
    }

    LockWorkstation(WLSession);

#ifdef _M_ARM64
    /* Load keyboard layout on ARM64 (required for RIT→WM_KEYDOWN delivery) */
    if (!InitKeyboardLayouts())
    {
        DbgPrint("[WINLOGON] ARM64: InitKeyboardLayouts failed (non-fatal)\n");
    }
    else
    {
        DbgPrint("[WINLOGON] ARM64: Keyboard layout loaded\n");
    }
    DbgPrint("[WINLOGON] ARM64: Skipping RPC, services, lsass, notifications, GINA\n");
#else
#if defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
    /* Load default keyboard layouts — DISABLED: NtCreateSection deadlock in Rust MM
     * for file-backed SEC_IMAGE sections causes winlogon to hang then BUGCHECK 0xF4.
     * Keyboard layout loading calls NtUserLoadKeyboardLayoutEx which does NtCreateSection
     * on the layout DLL. The desktop still works without keyboard layouts. */
#if 0
    DbgPrint("[WINLOGON] Calling InitKeyboardLayouts\n");
    if (!InitKeyboardLayouts())
    {
        DbgPrint("[WINLOGON] InitKeyboardLayouts failed (non-fatal)\n");
    }
    else
    {
        DbgPrint("[WINLOGON] InitKeyboardLayouts OK\n");
    }
#endif

    DbgPrint("[WINLOGON] Calling StartRpcServer\n");
    if (!StartRpcServer())
    {
        DbgPrint("[WINLOGON] StartRpcServer failed (non-fatal)\n");
    }
    else
    {
        DbgPrint("[WINLOGON] StartRpcServer OK\n");
    }

    DbgPrint("[WINLOGON] Calling StartServicesManager\n");
    if (!StartServicesManager())
    {
        DbgPrint("[WINLOGON] StartServicesManager failed (non-fatal)\n");
    }
    else
    {
        DbgPrint("[WINLOGON] StartServicesManager OK\n");
    }

    DbgPrint("[WINLOGON] Calling StartLsass\n");
    if (!StartLsass())
    {
        DbgPrint("[WINLOGON] StartLsass failed (error %lu, non-fatal)\n", GetLastError());
    }
    else
    {
        DbgPrint("[WINLOGON] StartLsass OK, calling WaitForLsass\n");
        /* Wait for the LSA server */
        WaitForLsass();
        DbgPrint("[WINLOGON] WaitForLsass returned\n");
    }
#else
    /* Load default keyboard layouts */
    if (!InitKeyboardLayouts())
    {
        ERR("WL: Could not preload keyboard layouts\n");
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
    }

    if (!StartRpcServer())
    {
        ERR("WL: Could not start the RPC server\n");
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
    }

    if (!StartServicesManager())
    {
        ERR("WL: Could not start services.exe\n");
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
    }

    if (!StartLsass())
    {
        ERR("WL: Failed to start lsass.exe service (error %lu)\n", GetLastError());
        NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED, 0, 0, NULL, OptionOk, &HardErrorResponse);
        ExitProcess(1);
    }

    /* Wait for the LSA server */
    WaitForLsass();
#endif

    /* Init Notifications */
    DbgPrint("[WINLOGON] Calling InitNotifications\n");
    InitNotifications();
    DbgPrint("[WINLOGON] InitNotifications done, calling GinaInit\n");

    /* Load and initialize gina */
    if (!GinaInit(WLSession))
    {
#if defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
        DbgPrint("[WINLOGON] GinaInit failed (non-fatal)\n");
#else
        ERR("WL: Failed to initialize Gina\n");
        // FIXME: Retrieve the real name of the GINA DLL we were trying to load.
        // It is known only inside the GinaInit function...
        DialogBoxParam(hAppInstance, MAKEINTRESOURCE(IDD_GINALOADFAILED), GetDesktopWindow(), GinaLoadFailedWindowProc, (LPARAM)L"msgina.dll");
        HandleShutdown(WLSession, WLX_SAS_ACTION_SHUTDOWN_REBOOT);
        ExitProcess(1);
#endif
    }

    DisplayStatusMessage(WLSession, WLSession->WinlogonDesktop, IDS_REACTOSISSTARTINGUP);
#endif

#if 0
    /* Connect to NetLogon service (lsass.exe) */
    /* Real winlogon uses "Winlogon" */
    RtlInitUnicodeString((PUNICODE_STRING)&ProcessName, L"Winlogon");
    Status = LsaRegisterLogonProcess(&ProcessName, &LsaHandle, &Mode);
    if (Status == STATUS_PORT_CONNECTION_REFUSED)
    {
        /* Add the 'SeTcbPrivilege' privilege and try again */
        Status = RtlAdjustPrivilege(SE_TCB_PRIVILEGE, TRUE, TRUE, &Old);
        if (!NT_SUCCESS(Status))
        {
            ERR("RtlAdjustPrivilege() failed with error %lu\n", LsaNtStatusToWinError(Status));
            return 1;
        }

        Status = LsaRegisterLogonProcess(&ProcessName, &LsaHandle, &Mode);
    }

    if (!NT_SUCCESS(Status))
    {
        ERR("LsaRegisterLogonProcess() failed with error %lu\n", LsaNtStatusToWinError(Status));
        return 1;
    }

    RtlInitUnicodeString((PUNICODE_STRING)&PackageName, MICROSOFT_KERBEROS_NAME_W);
    Status = LsaLookupAuthenticationPackage(LsaHandle, &PackageName, &AuthenticationPackage);
    if (!NT_SUCCESS(Status))
    {
        ERR("LsaLookupAuthenticationPackage() failed with error %lu\n", LsaNtStatusToWinError(Status));
        LsaDeregisterLogonProcess(LsaHandle);
        return 1;
    }
#endif

#if defined(_M_ARM64) || defined(_USE_DUMMY_PSEH) || defined(_USE_PSEH_CLANG)
    /* Tell kernel that CurrentControlSet is good */
    NtInitializeRegistry(CM_BOOT_FLAG_ACCEPTED | 1);

    /* Create minimal HKCU registry keys needed by explorer.exe.
     * Normally done by userinit/profile loading, which we skip. */
    {
        HKEY hKey;
        if (RegCreateKeyExW(HKEY_CURRENT_USER,
                            L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer",
                            0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
        {
            RegCloseKey(hKey);
            DbgPrint("[WINLOGON] Created HKCU\\...\\Explorer key\n");
        }
        /* Shell Folders — explorer checks this for desktop/start menu paths */
        if (RegCreateKeyExW(HKEY_CURRENT_USER,
                            L"Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Shell Folders",
                            0, NULL, 0, KEY_WRITE, NULL, &hKey, NULL) == ERROR_SUCCESS)
        {
            RegCloseKey(hKey);
        }
    }

    /* Check for pending setup */
    if (GetSetupType() != 0)
    {
        DbgPrint("[WINLOGON] Setup mode detected, launching setup...\n");
        RunSetup();
    }

    DbgPrint("[WINLOGON] Init complete. Spawning explorer.exe...\n");

    /* Spawn explorer.exe (shell — creates desktop, taskbar) */
    {
        STARTUPINFOW Si;
        PROCESS_INFORMATION Pi;
        BOOL bResult;
        WCHAR ExplorerPath[MAX_PATH];
        WCHAR ExplorerLine[] = L"explorer.exe";

        /* Build full path: %SystemRoot%\explorer.exe */
        GetWindowsDirectoryW(ExplorerPath, MAX_PATH);
        wcscat(ExplorerPath, L"\\explorer.exe");
        DbgPrint("[WINLOGON] Trying explorer path: %S\n", ExplorerPath);

        ZeroMemory(&Si, sizeof(Si));
        Si.cb = sizeof(Si);
        Si.lpDesktop = L"WinSta0\\Default";

        bResult = CreateProcessW(
            ExplorerPath,
            ExplorerLine,
            NULL,
            NULL,
            FALSE,
            0,
            NULL,
            NULL,
            &Si,
            &Pi);

        if (bResult)
        {
            DbgPrint("[WINLOGON] explorer.exe created (PID %lu, TID %lu)\n",
                     Pi.dwProcessId, Pi.dwThreadId);
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        }
        else
        {
            DbgPrint("[WINLOGON] CreateProcessW(explorer.exe) FAILED (error %lu)\n",
                     GetLastError());
        }
    }

    /* Load system fonts on a background thread to avoid blocking winlogon
     * while CD I/O completes for each font file. */
    {
        HANDLE hThread;
        hThread = CreateThread(NULL, 0, FontLoadThread, NULL, 0, NULL);
        if (hThread)
        {
            DbgPrint("[WINLOGON] Font loading thread started\n");
            CloseHandle(hThread);
        }
        else
        {
            DbgPrint("[WINLOGON] Failed to create font loading thread\n");
        }
    }

#ifdef _M_ARM64
    /* Spawn notepad.exe LAST among GUI apps so it gets keyboard focus */
    {
        STARTUPINFOW Si;
        PROCESS_INFORMATION Pi;
        BOOL bResult;
        WCHAR NoteLine[] = L"notepad.exe";

        ZeroMemory(&Si, sizeof(Si));
        Si.cb = sizeof(Si);
        Si.lpDesktop = L"WinSta0\\Default";

        bResult = CreateProcessW(
            L"notepad.exe",
            NoteLine,
            NULL,
            NULL,
            FALSE,
            0,
            NULL,
            NULL,
            &Si,
            &Pi);

        if (bResult)
        {
            DbgPrint("[WINLOGON] ARM64: notepad.exe created (PID %lu, TID %lu)\n",
                     Pi.dwProcessId, Pi.dwThreadId);
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        }
        else
        {
            DbgPrint("[WINLOGON] ARM64: CreateProcessW(notepad.exe) FAILED (error %lu)\n",
                     GetLastError());
        }
    }

    /* Spawn interactive cmd.exe with console (CONSRV available) */
    {
        STARTUPINFOW Si;
        PROCESS_INFORMATION Pi;
        BOOL bResult;
        DWORD ExitCode;
        WCHAR CmdLine[] = L"cmd.exe";

        ZeroMemory(&Si, sizeof(Si));
        Si.cb = sizeof(Si);

        bResult = CreateProcessW(
            L"cmd.exe",
            CmdLine,
            NULL,
            NULL,
            FALSE,
            0,
            NULL,
            NULL,
            &Si,
            &Pi);

        if (bResult)
        {
            DbgPrint("[WINLOGON] ARM64: cmd.exe created (PID %lu, TID %lu). Waiting...\n",
                     Pi.dwProcessId, Pi.dwThreadId);
            WaitForSingleObject(Pi.hProcess, INFINITE);
            ExitCode = 0xDEAD;
            GetExitCodeProcess(Pi.hProcess, &ExitCode);
            DbgPrint("[WINLOGON] ARM64: cmd.exe exited, exit code = 0x%lx\n", ExitCode);
            CloseHandle(Pi.hThread);
            CloseHandle(Pi.hProcess);
        }
        else
        {
            DbgPrint("[WINLOGON] ARM64: CreateProcessW(cmd.exe) FAILED (error %lu)\n",
                     GetLastError());
        }
    }

    DbgPrint("[WINLOGON] ARM64: cmd.exe exited. Sleeping.\n");
#endif /* _M_ARM64 */

    /* No SAS window, no GINA — just sleep forever.
     * Block on a never-signaled event to properly enter WAIT state
     * and leave the CPU for other threads. */
    {
        HANDLE hSleepEvent;
        OBJECT_ATTRIBUTES oa;
        InitializeObjectAttributes(&oa, NULL, 0, NULL, NULL);
        NtCreateEvent(&hSleepEvent, EVENT_ALL_ACCESS, &oa, NotificationEvent, FALSE);
        NtWaitForSingleObject(hSleepEvent, FALSE, NULL);
        /* Never reaches here */
    }
#else
    CallNotificationDlls(WLSession, StartupHandler);

    /* Create a hidden window to get SAS notifications */
    if (!InitializeSAS(WLSession))
    {
        ERR("WL: Failed to initialize SAS\n");
        ExitProcess(2);
    }

    // DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_PREPARENETWORKCONNECTIONS);
    // DisplayStatusMessage(Session, Session->WinlogonDesktop, IDS_APPLYINGCOMPUTERSETTINGS);

    /* Display logged out screen */
    WLSession->LogonState = STATE_INIT;
    RemoveStatusMessage(WLSession);

    /* Check for pending setup */
    if (GetSetupType() != 0)
    {
        /* Run setup and reboot when done */
        TRACE("WL: Setup mode detected\n");
        RunSetup();
    }
    else
    {
        PostMessageW(WLSession->SASWindow, WLX_WM_SAS, WLX_SAS_TYPE_CTRL_ALT_DEL, 0);
    }

    /* Tell kernel that CurrentControlSet is good (needed
     * to support Last good known configuration boot) */
    NtInitializeRegistry(CM_BOOT_FLAG_ACCEPTED | 1);

    /* Message loop for the SAS window */
    while (GetMessageW(&Msg, WLSession->SASWindow, 0, 0))
    {
        TranslateMessage(&Msg);
        DispatchMessageW(&Msg);
    }

    CleanupNotifications();
#endif

    /* We never go there */
    // TODO: Shutdown if we are in session 0, otherwise let the process terminate.
    SleepEx(INFINITE, FALSE);
    return 0;
}
