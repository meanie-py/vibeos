/*
 *  COPYRIGHT:        See COPYING in the top level directory
 *  PROJECT:          ReactOS Win32k subsystem
 *  PURPOSE:          Window stations
 *  FILE:             win32ss/user/ntuser/winsta.c
 *  PROGRAMER:        Casper S. Hornstrup (chorns@users.sourceforge.net)
 *  TODO:             The process window station is created on
 *                    the first USER32/GDI32 call not related
 *                    to window station/desktop handling
 */

#include <win32k.h>
#include <safe_user_io.h>
DBG_DEFAULT_CHANNEL(UserWinsta);

/* GLOBALS *******************************************************************/

/*
 * The currently active window station. This is the
 * only one interactive window station on the system.
 */
PWINSTATION_OBJECT InputWindowStation = NULL;

/* Winlogon SAS window */
HWND hwndSAS = NULL;

/* Whether cursor/icon loading has been completed (may be deferred if the
 * first GUI process didn't have user32's KernelCallbackTable ready) */
BOOL gbDesktopCursorsAndIconsLoaded = FALSE;

/* Full path to WindowStations directory */
UNICODE_STRING gustrWindowStationsDir;

/* INITIALIZATION FUNCTIONS ****************************************************/

CODE_SEG("INIT")
NTSTATUS
NTAPI
InitWindowStationImpl(VOID)
{
    GENERIC_MAPPING IntWindowStationMapping = { WINSTA_READ,
                                                WINSTA_WRITE,
                                                WINSTA_EXECUTE,
                                                WINSTA_ACCESS_ALL};

    /* Set Winsta Object Attributes */
    ExWindowStationObjectType->TypeInfo.DefaultNonPagedPoolCharge = sizeof(WINSTATION_OBJECT);
    ExWindowStationObjectType->TypeInfo.GenericMapping = IntWindowStationMapping;
    ExWindowStationObjectType->TypeInfo.ValidAccessMask = WINSTA_ACCESS_ALL;

    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
UserCreateWinstaDirectory(VOID)
{
    NTSTATUS Status;
    PPEB Peb;
    OBJECT_ATTRIBUTES ObjectAttributes;
    HANDLE hWinstaDir;
    WCHAR wstrWindowStationsDir[MAX_PATH];

    /* Create the WindowStations directory and cache its path for later use */
    Peb = NtCurrentPeb();
    if(Peb->SessionId == 0)
    {
        if (!RtlCreateUnicodeString(&gustrWindowStationsDir, WINSTA_OBJ_DIR))
        {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }
    else
    {
        Status = RtlStringCbPrintfW(wstrWindowStationsDir,
                                    sizeof(wstrWindowStationsDir),
                                    L"%ws\\%lu%ws",
                                    SESSION_DIR,
                                    Peb->SessionId,
                                    WINSTA_OBJ_DIR);
        if (!NT_SUCCESS(Status))
            return Status;

        if (!RtlCreateUnicodeString(&gustrWindowStationsDir, wstrWindowStationsDir))
        {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
    }

    InitializeObjectAttributes(&ObjectAttributes,
                               &gustrWindowStationsDir,
                               OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);
    Status = ZwCreateDirectoryObject(&hWinstaDir, DIRECTORY_CREATE_OBJECT, &ObjectAttributes);
    if (!NT_SUCCESS(Status))
    {
        ERR("Could not create %wZ directory (Status 0x%X)\n", &gustrWindowStationsDir,  Status);
        return Status;
    }

    TRACE("Created directory %wZ for session %lu\n", &gustrWindowStationsDir, Peb->SessionId);

    return Status;
}

/* OBJECT CALLBACKS ***********************************************************/

NTSTATUS
NTAPI
IntWinStaObjectDelete(
    _In_ PVOID Parameters)
{
    PWIN32_DELETEMETHOD_PARAMETERS DeleteParameters = Parameters;
    PWINSTATION_OBJECT WinSta = (PWINSTATION_OBJECT)DeleteParameters->Object;

    TRACE("Deleting window station 0x%p\n", WinSta);

    if (WinSta == InputWindowStation)
    {
        ERR("WARNING: Deleting the interactive window station '%wZ'!\n",
            &(OBJECT_HEADER_TO_NAME_INFO(OBJECT_TO_OBJECT_HEADER(InputWindowStation))->Name));

        /* Only Winlogon can close and delete the interactive window station */
        ASSERT(gpidLogon == PsGetCurrentProcessId());

        InputWindowStation = NULL;
    }

    WinSta->Flags |= WSS_DYING;

    UserEmptyClipboardData(WinSta);

    RtlDestroyAtomTable(WinSta->AtomTable);

    UserAssignmentUnlock((PVOID*)&WinSta->spklList);

    return STATUS_SUCCESS;
}

NTSTATUS
NTAPI
IntWinStaObjectParse(
    _In_ PVOID Parameters)
{
    PWIN32_PARSEMETHOD_PARAMETERS ParseParameters = Parameters;
    PUNICODE_STRING RemainingName = ParseParameters->RemainingName;

    /* Assume we don't find anything */
    *ParseParameters->Object = NULL;

    /* Check for an empty name */
    if (!RemainingName->Length)
    {
        /* Make sure this is a window station, can't parse a desktop now */
        if (ParseParameters->ObjectType != ExWindowStationObjectType)
        {
            /* Fail */
            return STATUS_OBJECT_TYPE_MISMATCH;
        }

        /* Reference the window station and return */
        ObReferenceObject(ParseParameters->ParseObject);
        *ParseParameters->Object = ParseParameters->ParseObject;
        return STATUS_SUCCESS;
    }

    /* Check for leading slash */
    if (RemainingName->Buffer[0] == OBJ_NAME_PATH_SEPARATOR)
    {
        /* Skip it */
        RemainingName->Buffer++;
        RemainingName->Length -= sizeof(WCHAR);
        RemainingName->MaximumLength -= sizeof(WCHAR);
    }

    /* Check if there is still a slash */
    if (wcschr(RemainingName->Buffer, OBJ_NAME_PATH_SEPARATOR))
    {
        /* In this case, fail */
        return STATUS_OBJECT_PATH_INVALID;
    }

    /*
     * Check if we are parsing a desktop.
     */
    if (ParseParameters->ObjectType == ExDesktopObjectType)
    {
        /* Then call the desktop parse routine */
        return IntDesktopObjectParse(ParseParameters->ParseObject,
                                     ParseParameters->ObjectType,
                                     ParseParameters->AccessState,
                                     ParseParameters->AccessMode,
                                     ParseParameters->Attributes,
                                     ParseParameters->CompleteName,
                                     RemainingName,
                                     ParseParameters->Context,
                                     ParseParameters->SecurityQos,
                                     ParseParameters->Object);
    }

    /* Should hopefully never get here */
    return STATUS_OBJECT_TYPE_MISMATCH;
}

NTSTATUS
NTAPI
IntWinStaOkToClose(
    _In_ PVOID Parameters)
{
    PWIN32_OKAYTOCLOSEMETHOD_PARAMETERS OkToCloseParameters = Parameters;
    PPROCESSINFO ppi;

    ppi = PsGetCurrentProcessWin32Process();

    if (ppi && (OkToCloseParameters->Handle == ppi->hwinsta))
    {
        return STATUS_ACCESS_DENIED;
    }

    return STATUS_SUCCESS;
}

/* PRIVATE FUNCTIONS **********************************************************/

/*
 * IntValidateWindowStationHandle
 *
 * Validates the window station handle.
 *
 * Remarks
 *    If the function succeeds, the handle remains referenced. If the
 *    fucntion fails, last error is set.
 */

NTSTATUS FASTCALL
IntValidateWindowStationHandle(
    HWINSTA WindowStation,
    KPROCESSOR_MODE AccessMode,
    ACCESS_MASK DesiredAccess,
    PWINSTATION_OBJECT *Object,
    POBJECT_HANDLE_INFORMATION pObjectHandleInfo)
{
    NTSTATUS Status;

    if (WindowStation == NULL)
    {
        ERR("Invalid window station handle\n");
        EngSetLastError(ERROR_INVALID_HANDLE);
        return STATUS_INVALID_HANDLE;
    }

    Status = ObReferenceObjectByHandle(WindowStation,
                                       DesiredAccess,
                                       ExWindowStationObjectType,
                                       AccessMode,
                                       (PVOID*)Object,
                                       pObjectHandleInfo);

    if (!NT_SUCCESS(Status))
        SetLastNtError(Status);

    return Status;
}

/*
 * Try to load system cursors and icons if not yet loaded.
 * Called from co_AddGuiApp for each new GUI process until loading succeeds.
 * Requires the calling process to have user32's KernelCallbackTable set up.
 */
VOID FASTCALL
co_IntDeferredCursorsAndIcons(VOID)
{
    PEPROCESS Process;

    if (gbDesktopCursorsAndIconsLoaded)
        return;

    /* Don't attempt user-mode callbacks on the desktop thread —
     * callback functions assert PsGetCurrentThreadWin32Thread() != gptiDesktopThread */
    if (PsGetCurrentThreadWin32Thread() == gptiDesktopThread)
        return;

    Process = PsGetCurrentProcess();
    if (!Process || !Process->Peb || !Process->Peb->KernelCallbackTable)
        return;

    co_IntLoadDefaultCursors();
    co_IntSetWndIcons();
    gbDesktopCursorsAndIconsLoaded = TRUE;
}

BOOL FASTCALL
co_IntInitializeDesktopGraphics(VOID)
{
    LONG lRet;
    UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"DISPLAY");
    PDESKTOP pdesk;
#ifndef _M_ARM64
    TEXTMETRICW tmw;
#endif

    /* If PDEV already created (e.g. early init from UserInitialize), skip re-creation */
    if (gpmdev)
    {
        /* PDEV already exists — just need to do ScreenDC + UI init below */
    }
    else
    {
        lRet = PDEVOBJ_lChangeDisplaySettings(NULL, NULL, NULL, &gpmdev, TRUE);
        if (lRet != DISP_CHANGE_SUCCESSFUL && !gbBaseVideo)
        {
            gbBaseVideo = TRUE;
            EngpUpdateGraphicsDeviceList();
            lRet = PDEVOBJ_lChangeDisplaySettings(NULL, NULL, NULL, &gpmdev, TRUE);
            gbBaseVideo = FALSE;
            EngpUpdateGraphicsDeviceList();
        }
        if (lRet != DISP_CHANGE_SUCCESSFUL)
        {
            return FALSE;
        }
    }

#ifdef _M_ARM64
    /* ARM64: Create screen DC and primary surface. DC_InitHack (dclife.c)
     * skips TextIntRealizeFont on ARM64, so IntGdiCreateDC is safe.
     * Skip font measurement, monitor setup, cursors, icons. */
    ScreenDeviceContext = IntGdiCreateDC(&DriverName, NULL, NULL, NULL, FALSE);
    if (NULL == ScreenDeviceContext)
    {
        IntDestroyPrimarySurface();
        return FALSE;
    }
    GreSetDCOwner(ScreenDeviceContext, GDI_OBJ_HMGR_PUBLIC);

    /* Cache PDEV surface reference + populate GdiHandleTable->DevCaps */
    if (!IntCreatePrimarySurface())
    {
        return FALSE;
    }

    /* Set metrics from PDEV's gdiinfo (skip font measurement) */
    gpsi->aiSysMet[SM_CXSCREEN] = gpmdev->ppdevGlobal->gdiinfo.ulHorzRes;
    gpsi->aiSysMet[SM_CYSCREEN] = gpmdev->ppdevGlobal->gdiinfo.ulVertRes;
    gpsi->Planes        = 1;
    gpsi->BitsPixel     = gpmdev->ppdevGlobal->gdiinfo.cBitsPixel;
    gpsi->BitCount      = gpsi->Planes * gpsi->BitsPixel;
    gpsi->dmLogPixels   = 96;
    gpsi->PUSIFlags    &= ~PUSIF_PALETTEDISPLAY;
    gpsi->cxSysFontChar = 8;
    gpsi->cySysFontChar = 16;
    gpsi->ptCursor.x    = gpsi->aiSysMet[SM_CXSCREEN] / 2;
    gpsi->ptCursor.y    = gpsi->aiSysMet[SM_CYSCREEN] / 2;

    /* Refresh metrics with actual screen resolution. InitMetrics was first
     * called from SpiUpdatePerUserSystemParameters with gpmdev=NULL which
     * defaulted to 640x480. Now that PDEV is live, recalculate all
     * screen-size-dependent metrics (SM_CXMAXIMIZED, SM_CXFULLSCREEN, etc.) */
    InitMetrics();

    MenuInit();

    /* Show the desktop if one already exists (it won't on first call from
     * NtUserInitialize since desktops are created later by winlogon) */
    pdesk = IntGetActiveDesktop();
    if (pdesk)
    {
        co_IntShowDesktop(pdesk,
                          gpsi->aiSysMet[SM_CXSCREEN],
                          gpsi->aiSysMet[SM_CYSCREEN],
                          TRUE);
    }

    return TRUE;
#else
    ScreenDeviceContext = IntGdiCreateDC(&DriverName, NULL, NULL, NULL, FALSE);
    if (NULL == ScreenDeviceContext)
    {
        IntDestroyPrimarySurface();
        return FALSE;
    }
    GreSetDCOwner(ScreenDeviceContext, GDI_OBJ_HMGR_PUBLIC);

    if (!IntCreatePrimarySurface())
    {
        return FALSE;
    }

    hSystemBM = NtGdiCreateCompatibleDC(ScreenDeviceContext);

    NtGdiSelectFont(hSystemBM, NtGdiGetStockObject(SYSTEM_FONT));
    GreSetDCOwner(hSystemBM, GDI_OBJ_HMGR_PUBLIC);

    /* Update the system metrics */
    InitMetrics();

    /* Set new size of the monitor */
    UserUpdateMonitorSize((HDEV)gpmdev->ppdevGlobal);

    /* Update the SERVERINFO */
    gpsi->aiSysMet[SM_CXSCREEN] = gpmdev->ppdevGlobal->gdiinfo.ulHorzRes;
    gpsi->aiSysMet[SM_CYSCREEN] = gpmdev->ppdevGlobal->gdiinfo.ulVertRes;
    gpsi->Planes        = NtGdiGetDeviceCaps(ScreenDeviceContext, PLANES);
    gpsi->BitsPixel     = NtGdiGetDeviceCaps(ScreenDeviceContext, BITSPIXEL);
    gpsi->BitCount      = gpsi->Planes * gpsi->BitsPixel;
    gpsi->dmLogPixels   = NtGdiGetDeviceCaps(ScreenDeviceContext, LOGPIXELSY);
    if (NtGdiGetDeviceCaps(ScreenDeviceContext, RASTERCAPS) & RC_PALETTE)
    {
        gpsi->PUSIFlags |= PUSIF_PALETTEDISPLAY;
    }
    else
    {
        gpsi->PUSIFlags &= ~PUSIF_PALETTEDISPLAY;
    }
    // Font is realized and this dc was previously set to internal DC_ATTR.
    gpsi->cxSysFontChar = IntGetCharDimensions(hSystemBM, &tmw, (DWORD*)&gpsi->cySysFontChar);
    gpsi->tmSysFont     = tmw;

    /* Put the pointer in the center of the screen */
    gpsi->ptCursor.x = gpsi->aiSysMet[SM_CXSCREEN] / 2;
    gpsi->ptCursor.y = gpsi->aiSysMet[SM_CYSCREEN] / 2;

    /* Attach monitor */
    UserAttachMonitor((HDEV)gpmdev->ppdevGlobal);

    /* HACK: Brief yield to let concurrent DLL loading I/O complete.
     * Without this, a timing-dependent deadlock in the cache manager
     * Rust code can freeze the system when the first user-mode callback
     * races with concurrent NtCreateSection calls from other processes. */
    {
        LARGE_INTEGER Delay;
        Delay.QuadPart = -500000; /* 50ms */
        KeDelayExecutionThread(KernelMode, FALSE, &Delay);
    }

    /* Setup the cursor and icons.
     * On DUMMY_PSEH builds (amd64), the KeUserModeCallbacks for icon/cursor
     * loading can crash in user-mode code (LoadImageW → RtlVirtualUnwind AV),
     * corrupting the calling process state and ultimately killing winlogon.
     * Skip on amd64/ARM64 — system will work with NULL cursors/icons.
     * On i386 (PSEH_CLANG), callbacks work because exceptions are caught. */
#if !defined(_USE_DUMMY_PSEH)
    co_IntLoadDefaultCursors();
    co_IntSetWndIcons();
    gbDesktopCursorsAndIconsLoaded = TRUE;
#else
    /* DUMMY_PSEH: skip cursor/icon loading — KeUserModeCallbacks can crash */
#endif

    /* Setup Menu */
    MenuInit();

    /* Show the desktop if one already exists (it won't on first call from
     * NtUserInitialize since desktops are created later by winlogon) */
    pdesk = IntGetActiveDesktop();
    if (pdesk)
    {
        co_IntShowDesktop(pdesk, gpsi->aiSysMet[SM_CXSCREEN], gpsi->aiSysMet[SM_CYSCREEN], TRUE);
    }

    /* HACK: display wallpaper on all secondary displays */
    {
        PGRAPHICS_DEVICE pGraphicsDevice;
        UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"DISPLAY");
        UNICODE_STRING DisplayName;
        HDC hdc;
        ULONG iDevNum;

        for (iDevNum = 1; (pGraphicsDevice = EngpFindGraphicsDevice(NULL, iDevNum)) != NULL; iDevNum++)
        {
            RtlInitUnicodeString(&DisplayName, pGraphicsDevice->szWinDeviceName);
            hdc = IntGdiCreateDC(&DriverName, &DisplayName, NULL, NULL, FALSE);
            IntPaintDesktop(hdc);
        }
    }

    return TRUE;
#endif
}

VOID FASTCALL
IntEndDesktopGraphics(VOID)
{
    if (NULL != ScreenDeviceContext)
    {  // No need to allocate a new dcattr.
        GreSetDCOwner(ScreenDeviceContext, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject(ScreenDeviceContext);
        ScreenDeviceContext = NULL;
    }
    IntHideDesktop(IntGetActiveDesktop());
    IntDestroyPrimarySurface();
}

HDC FASTCALL
IntGetScreenDC(VOID)
{
    return ScreenDeviceContext;
}

BOOL FASTCALL
CheckWinstaAttributeAccess(ACCESS_MASK DesiredAccess)
{
    PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();
    if ( gpidLogon != PsGetCurrentProcessId() )
    {
        if (!(ppi->W32PF_flags & W32PF_IOWINSTA))
        {
            ERR("Requires Interactive Window Station\n");
            EngSetLastError(ERROR_REQUIRES_INTERACTIVE_WINDOWSTATION);
            return FALSE;
        }
        if (!RtlAreAllAccessesGranted(ppi->amwinsta, DesiredAccess))
        {
            ERR("Access Denied\n");
            EngSetLastError(ERROR_ACCESS_DENIED);
            return FALSE;
        }
    }
    return TRUE;
}

// Win: _GetProcessWindowStation
PWINSTATION_OBJECT FASTCALL
IntGetProcessWindowStation(HWINSTA *phWinSta OPTIONAL)
{
    PWINSTATION_OBJECT pWinSta;
    PPROCESSINFO ppi = GetW32ProcessInfo();
    HWINSTA hWinSta = ppi->hwinsta;
    if (phWinSta)
        *phWinSta = hWinSta;
    IntValidateWindowStationHandle(hWinSta, UserMode, 0, &pWinSta, 0);
    return pWinSta;
}

/* PUBLIC FUNCTIONS ***********************************************************/

/*
 * NtUserCreateWindowStation
 *
 * Creates a new window station.
 *
 * Parameters
 *    lpszWindowStationName
 *       Pointer to a null-terminated string specifying the name of the
 *       window station to be created. Window station names are
 *       case-insensitive and cannot contain backslash characters (\).
 *       Only members of the Administrators group are allowed to specify a
 *       name.
 *
 *    dwDesiredAccess
 *       Requested type of access
 *
 *    lpSecurity
 *       Security descriptor
 *
 *    Unknown3, Unknown4, Unknown5, Unknown6
 *       Unused
 *
 * Return Value
 *    If the function succeeds, the return value is a handle to the newly
 *    created window station. If the specified window station already
 *    exists, the function succeeds and returns a handle to the existing
 *    window station. If the function fails, the return value is NULL.
 *
 * Status
 *    @implemented
 */

NTSTATUS
FASTCALL
IntCreateWindowStation(
    OUT HWINSTA* phWinSta,
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    IN KPROCESSOR_MODE AccessMode,
    IN KPROCESSOR_MODE OwnerMode,
    IN ACCESS_MASK dwDesiredAccess,
    DWORD Unknown2,
    DWORD Unknown3,
    DWORD Unknown4,
    DWORD Unknown5,
    DWORD Unknown6)
{
    NTSTATUS Status;
    HWINSTA hWinSta;
    PWINSTATION_OBJECT WindowStation;

    TRACE("IntCreateWindowStation called\n");

    ASSERT(phWinSta);
    *phWinSta = NULL;

    Status = ObOpenObjectByName(ObjectAttributes,
                                ExWindowStationObjectType,
                                AccessMode,
                                NULL,
                                dwDesiredAccess,
                                NULL,
                                (PVOID*)&hWinSta);
    if (NT_SUCCESS(Status))
    {
        TRACE("IntCreateWindowStation opened window station '%wZ'\n",
              ObjectAttributes->ObjectName);
        *phWinSta = hWinSta;
        return Status;
    }

    /*
     * No existing window station found, try to create a new one.
     */

    /* Create the window station object */
    Status = ObCreateObject(AccessMode,
                            ExWindowStationObjectType,
                            ObjectAttributes,
                            OwnerMode,
                            NULL,
                            sizeof(WINSTATION_OBJECT),
                            0,
                            0,
                            (PVOID*)&WindowStation);
    if (!NT_SUCCESS(Status))
    {
        ERR("ObCreateObject failed for window station '%wZ', Status 0x%08lx\n",
            ObjectAttributes->ObjectName, Status);
        SetLastNtError(Status);
        return Status;
    }

    /* Initialize the window station */
    RtlZeroMemory(WindowStation, sizeof(WINSTATION_OBJECT));

    InitializeListHead(&WindowStation->DesktopListHead);
    WindowStation->dwSessionId = NtCurrentPeb()->SessionId;
    Status = RtlCreateAtomTable(37, &WindowStation->AtomTable);
    if (!NT_SUCCESS(Status))
    {
        ERR("RtlCreateAtomTable failed for window station '%wZ', Status 0x%08lx\n",
            ObjectAttributes->ObjectName, Status);
        ObDereferenceObject(WindowStation);
        SetLastNtError(Status);
        return Status;
    }

    Status = ObInsertObject(WindowStation,
                            NULL,
                            dwDesiredAccess,
                            0,
                            NULL,
                            (PVOID*)&hWinSta);
    if (!NT_SUCCESS(Status))
    {
        ERR("ObInsertObject failed for window station, Status 0x%08lx\n", Status);
        SetLastNtError(Status);
        return Status;
    }

    // FIXME! TODO: Add this new window station to a linked list

    if (InputWindowStation == NULL)
    {
        ERR("Initializing input window station\n");

        /* Only Winlogon can create the interactive window station */
        ASSERT(gpidLogon == PsGetCurrentProcessId());

        InputWindowStation = WindowStation;
        WindowStation->Flags &= ~WSS_NOIO;

        InitCursorImpl();

        UserCreateSystemThread(ST_DESKTOP_THREAD);
        UserCreateSystemThread(ST_RIT);

        /* Desktop functions require the desktop thread running so wait for it to initialize */
        UserLeaveCo();
        KeWaitForSingleObject(gpDesktopThreadStartedEvent,
                              UserRequest,
                              UserMode,
                              FALSE,
                              NULL);
        UserEnterCo();
    }
    else
    {
        WindowStation->Flags |= WSS_NOIO;
    }

    TRACE("IntCreateWindowStation created window station '%wZ' object 0x%p handle 0x%p\n",
          ObjectAttributes->ObjectName, WindowStation, hWinSta);

    *phWinSta = hWinSta;
    EngSetLastError(ERROR_SUCCESS);

    return STATUS_SUCCESS;
}

static VOID
FreeUserModeWindowStationName(
    IN OUT PUNICODE_STRING WindowStationName,
    IN PUNICODE_STRING TebStaticUnicodeString,
    IN OUT POBJECT_ATTRIBUTES UserModeObjectAttributes OPTIONAL,
    IN POBJECT_ATTRIBUTES LocalObjectAttributes OPTIONAL)
{
    SIZE_T MemSize = 0;

    /* Try to restore the user's UserModeObjectAttributes */
    if (UserModeObjectAttributes && LocalObjectAttributes)
    {
        /* Best-effort restore; ignore a bad user pointer (was: _SEH2 swallow) */
        if (NT_SUCCESS(SafeProbeForWrite(UserModeObjectAttributes,
                                         sizeof(OBJECT_ATTRIBUTES),
                                         sizeof(ULONG))))
        {
            *UserModeObjectAttributes = *LocalObjectAttributes;
        }
    }

    /* Free the user-mode memory */
    if (WindowStationName && (WindowStationName != TebStaticUnicodeString))
    {
        ZwFreeVirtualMemory(ZwCurrentProcess(),
                            (PVOID*)&WindowStationName,
                            &MemSize,
                            MEM_RELEASE);
    }
}

static NTSTATUS
BuildUserModeWindowStationName(
    IN OUT POBJECT_ATTRIBUTES UserModeObjectAttributes,
    IN OUT POBJECT_ATTRIBUTES LocalObjectAttributes,
    OUT PUNICODE_STRING* WindowStationName,
    OUT PUNICODE_STRING* TebStaticUnicodeString)
{
    NTSTATUS Status;
    SIZE_T MemSize;

    LUID CallerLuid;
    PTEB Teb;
    USHORT StrSize;

    *WindowStationName = NULL;
    *TebStaticUnicodeString = NULL;

    /* Retrieve the current process LUID */
    Status = GetProcessLuid(NULL, NULL, &CallerLuid);
    if (!NT_SUCCESS(Status))
    {
        ERR("Failed to retrieve the caller LUID, Status 0x%08lx\n", Status);
        return Status;
    }

    /* Compute the needed string size */
    MemSize = _scwprintf(L"%wZ\\Service-0x%x-%x$",
                         &gustrWindowStationsDir,
                         CallerLuid.HighPart,
                         CallerLuid.LowPart);
    MemSize = MemSize * sizeof(WCHAR) + sizeof(UNICODE_NULL);
    if (MemSize > MAXUSHORT)
    {
        ERR("Window station name length is too long.\n");
        return STATUS_NAME_TOO_LONG;
    }
    StrSize = (USHORT)MemSize;

    /*
     * Check whether it's short enough so that we can use the static buffer
     * in the TEB. Otherwise continue with virtual memory allocation.
     */
    Teb = NtCurrentTeb();
    if (Teb && (StrSize <= sizeof(Teb->StaticUnicodeBuffer)))
    {
        /* We can use the TEB's static unicode string */
        ASSERT(Teb->StaticUnicodeString.Buffer == Teb->StaticUnicodeBuffer);
        ASSERT(Teb->StaticUnicodeString.MaximumLength == sizeof(Teb->StaticUnicodeBuffer));

        /* Remember the TEB's static unicode string address for later */
        *TebStaticUnicodeString = &Teb->StaticUnicodeString;

        *WindowStationName = *TebStaticUnicodeString;
        (*WindowStationName)->Length = 0;
    }
    else
    {
        /* The TEB's static unicode string is too small, allocate some user-mode virtual memory */
        MemSize += ALIGN_UP(sizeof(UNICODE_STRING), sizeof(PVOID));

        /* Allocate the memory in user-mode */
        Status = ZwAllocateVirtualMemory(ZwCurrentProcess(),
                                         (PVOID*)WindowStationName,
                                         0,
                                         &MemSize,
                                         MEM_COMMIT,
                                         PAGE_READWRITE);
        if (!NT_SUCCESS(Status))
        {
            ERR("ZwAllocateVirtualMemory() failed, Status 0x%08lx\n", Status);
            return Status;
        }

        RtlInitEmptyUnicodeString(*WindowStationName,
                                  (PWCHAR)((ULONG_PTR)*WindowStationName +
                                      ALIGN_UP(sizeof(UNICODE_STRING), sizeof(PVOID))),
                                  StrSize);
    }

    /* Build a valid window station name from the LUID */
    Status = RtlStringCbPrintfW((*WindowStationName)->Buffer,
                                (*WindowStationName)->MaximumLength,
                                L"%wZ\\Service-0x%x-%x$",
                                &gustrWindowStationsDir,
                                CallerLuid.HighPart,
                                CallerLuid.LowPart);
    if (!NT_SUCCESS(Status))
    {
        ERR("Impossible to build a valid window station name, Status 0x%08lx\n", Status);
        goto Quit;
    }
    (*WindowStationName)->Length = (USHORT)(wcslen((*WindowStationName)->Buffer) * sizeof(WCHAR));

    /* Try to update the user's UserModeObjectAttributes */
    Status = SafeProbeForWrite(UserModeObjectAttributes,
                               sizeof(OBJECT_ATTRIBUTES),
                               sizeof(ULONG));
    if (NT_SUCCESS(Status))
    {
        *LocalObjectAttributes = *UserModeObjectAttributes;

        UserModeObjectAttributes->ObjectName = *WindowStationName;
        UserModeObjectAttributes->RootDirectory = NULL;

        Status = STATUS_SUCCESS;
    }

Quit:
    if (!NT_SUCCESS(Status))
    {
        /* Release the window station name */
        FreeUserModeWindowStationName(*WindowStationName,
                                      *TebStaticUnicodeString,
                                      NULL, NULL);
    }

    return Status;
}

HWINSTA
APIENTRY
NtUserCreateWindowStation(
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    IN ACCESS_MASK dwDesiredAccess,
    DWORD Unknown2,
    DWORD Unknown3,
    DWORD Unknown4,
    DWORD Unknown5,
    DWORD Unknown6)
{
    NTSTATUS Status = STATUS_SUCCESS;
    HWINSTA hWinSta = NULL;
    OBJECT_ATTRIBUTES LocalObjectAttributes;
    PUNICODE_STRING WindowStationName = NULL;
    PUNICODE_STRING TebStaticUnicodeString = NULL;
    KPROCESSOR_MODE OwnerMode = UserMode;

    TRACE("NtUserCreateWindowStation called\n");
    /* Capture the object attributes and the window station name */
    Status = SafeProbeForRead(ObjectAttributes, sizeof(OBJECT_ATTRIBUTES), sizeof(ULONG));
    if (!NT_SUCCESS(Status))
    {
        ERR("ObjectAttributes capture failed, Status 0x%08lx\n", Status);
        goto CaptureDone;
    }
    LocalObjectAttributes = *ObjectAttributes;
    if (LocalObjectAttributes.Length != sizeof(OBJECT_ATTRIBUTES))
    {
        ERR("Invalid ObjectAttributes length!\n");
        Status = STATUS_INVALID_PARAMETER;
        goto CaptureDone;
    }

    /*
     * Check whether the caller provided a window station name together
     * with a RootDirectory handle.
     *
     * If the caller did not provide a window station name, build a new one
     * based on the logon session identifier for the calling process.
     * The new name is allocated in user-mode, as the rest of ObjectAttributes
     * already is, so that the validation performed by the Object Manager
     * can be done adequately.
     */
    if ((LocalObjectAttributes.ObjectName == NULL ||
         LocalObjectAttributes.ObjectName->Buffer == NULL ||
         LocalObjectAttributes.ObjectName->Length == 0 ||
         LocalObjectAttributes.ObjectName->Buffer[0] == UNICODE_NULL)
        /* &&
        LocalObjectAttributes.RootDirectory == NULL */)
    {
        /* No, build the new window station name */
        Status = BuildUserModeWindowStationName(ObjectAttributes,
                                                &LocalObjectAttributes,
                                                &WindowStationName,
                                                &TebStaticUnicodeString);
        if (!NT_SUCCESS(Status))
        {
            ERR("BuildUserModeWindowStationName() failed, Status 0x%08lx\n", Status);
            goto CaptureDone;
        }
        OwnerMode = KernelMode;
    }

CaptureDone:
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return NULL;
    }

    UserEnterExclusive();

    /* Create the window station */
    Status = IntCreateWindowStation(&hWinSta,
                                    ObjectAttributes,
                                    UserMode,
                                    OwnerMode,
                                    dwDesiredAccess,
                                    Unknown2,
                                    Unknown3,
                                    Unknown4,
                                    Unknown5,
                                    Unknown6);
    UserLeave();

    if (NT_SUCCESS(Status))
    {
        TRACE("NtUserCreateWindowStation created window station '%wZ' with handle 0x%p\n",
              ObjectAttributes->ObjectName, hWinSta);
        /* success */
    }
    else
    {
        ASSERT(hWinSta == NULL);
        ERR("NtUserCreateWindowStation failed to create window station '%wZ', Status 0x%08lx\n",
            ObjectAttributes->ObjectName, Status);
        /* failed */
    }

    /* Try to restore the user's ObjectAttributes and release the window station name */
    FreeUserModeWindowStationName(WindowStationName,
                                  TebStaticUnicodeString,
                                  (OwnerMode == KernelMode ? ObjectAttributes : NULL),
                                  &LocalObjectAttributes);

    if (!NT_SUCCESS(Status))
    {
        ASSERT(hWinSta == NULL);
        SetLastNtError(Status);
    }

    return hWinSta;
}

/*
 * NtUserOpenWindowStation
 *
 * Opens an existing window station.
 *
 * Parameters
 *    lpszWindowStationName
 *       Name of the existing window station.
 *
 *    dwDesiredAccess
 *       Requested type of access.
 *
 * Return Value
 *    If the function succeeds, the return value is the handle to the
 *    specified window station. If the function fails, the return value
 *    is NULL.
 *
 * Remarks
 *    The returned handle can be closed with NtUserCloseWindowStation.
 *
 * Status
 *    @implemented
 */

HWINSTA
APIENTRY
NtUserOpenWindowStation(
    IN POBJECT_ATTRIBUTES ObjectAttributes,
    IN ACCESS_MASK dwDesiredAccess)
{
    NTSTATUS Status = STATUS_SUCCESS;
    HWINSTA hWinSta = NULL;
    OBJECT_ATTRIBUTES LocalObjectAttributes;
    PUNICODE_STRING WindowStationName = NULL;
    PUNICODE_STRING TebStaticUnicodeString = NULL;
    KPROCESSOR_MODE OwnerMode = UserMode;

    TRACE("NtUserOpenWindowStation called\n");

    /* Capture the object attributes and the window station name */
    Status = SafeProbeForRead(ObjectAttributes, sizeof(OBJECT_ATTRIBUTES), sizeof(ULONG));
    if (!NT_SUCCESS(Status))
    {
        ERR("ObjectAttributes capture failed, Status 0x%08lx\n", Status);
        goto CaptureDone;
    }
    LocalObjectAttributes = *ObjectAttributes;
    if (LocalObjectAttributes.Length != sizeof(OBJECT_ATTRIBUTES))
    {
        ERR("Invalid ObjectAttributes length!\n");
        Status = STATUS_INVALID_PARAMETER;
        goto CaptureDone;
    }

    /*
     * Check whether the caller did not provide a window station name,
     * or provided the special "Service-0x00000000-00000000$" name.
     *
     * NOTE: On Windows, the special "Service-0x00000000-00000000$" string
     * is used instead of an empty name (observed when API-monitoring
     * OpenWindowStation() called with an empty window station name).
     */
    if ((LocalObjectAttributes.ObjectName == NULL ||
         LocalObjectAttributes.ObjectName->Buffer == NULL ||
         LocalObjectAttributes.ObjectName->Length == 0 ||
         LocalObjectAttributes.ObjectName->Buffer[0] == UNICODE_NULL)
        /* &&
        LocalObjectAttributes.RootDirectory == NULL */)
    {
        /* No, remember that for later */
        LocalObjectAttributes.ObjectName = NULL;
    }
    if (LocalObjectAttributes.ObjectName &&
        LocalObjectAttributes.ObjectName->Length ==
            sizeof(L"Service-0x00000000-00000000$") - sizeof(UNICODE_NULL) &&
        _wcsnicmp(LocalObjectAttributes.ObjectName->Buffer,
                  L"Service-0x00000000-00000000$",
                  LocalObjectAttributes.ObjectName->Length / sizeof(WCHAR)) == 0)
    {
        /* No, remember that for later */
        LocalObjectAttributes.ObjectName = NULL;
    }

    /*
     * If the caller did not provide a window station name, build a new one
     * based on the logon session identifier for the calling process.
     * The new name is allocated in user-mode, as the rest of ObjectAttributes
     * already is, so that the validation performed by the Object Manager
     * can be done adequately.
     */
    if (!LocalObjectAttributes.ObjectName)
    {
        /* No, build the new window station name */
        Status = BuildUserModeWindowStationName(ObjectAttributes,
                                                &LocalObjectAttributes,
                                                &WindowStationName,
                                                &TebStaticUnicodeString);
        if (!NT_SUCCESS(Status))
        {
            ERR("BuildUserModeWindowStationName() failed, Status 0x%08lx\n", Status);
            goto CaptureDone;
        }
        OwnerMode = KernelMode;
    }

CaptureDone:;

    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return NULL;
    }

    /* Open the window station */
    Status = ObOpenObjectByName(ObjectAttributes,
                                ExWindowStationObjectType,
                                UserMode,
                                NULL,
                                dwDesiredAccess,
                                NULL,
                                (PVOID*)&hWinSta);
    if (NT_SUCCESS(Status))
    {
        TRACE("NtUserOpenWindowStation opened window station '%wZ' with handle 0x%p\n",
              ObjectAttributes->ObjectName, hWinSta);
    }
    else
    {
        ASSERT(hWinSta == NULL);
        ERR("NtUserOpenWindowStation failed to open window station '%wZ', Status 0x%08lx\n",
            ObjectAttributes->ObjectName, Status);
    }

    /* Try to restore the user's ObjectAttributes and release the window station name */
    FreeUserModeWindowStationName(WindowStationName,
                                  TebStaticUnicodeString,
                                  (OwnerMode == KernelMode ? ObjectAttributes : NULL),
                                  &LocalObjectAttributes);

    if (!NT_SUCCESS(Status))
    {
        ASSERT(hWinSta == NULL);
        SetLastNtError(Status);
    }

    return hWinSta;
}
/* --- pretranslation split: the remaining function groups live in winsta.split2..2.c and are #included below so this stays ONE translation unit (zero CMake/linkage change). Reassembled verbatim by cpp. --- */
#include "winsta.split2.c"
