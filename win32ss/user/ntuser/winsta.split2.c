/* winsta.split2.c — #include fragment of winsta.c, carved for pretranslation so rustconv gets a small file. NOT a standalone TU; NOT in CMake; do not compile directly. The preprocessor reassembles the original winsta.c TU verbatim. See docs/engineering/win32k-pretranslation-audit.md */


/*
 * NtUserCloseWindowStation
 *
 * Closes a window station handle.
 *
 * Parameters
 *    hWinSta
 *       Handle to the window station.
 *
 * Return Value
 *    Status
 *
 * Remarks
 *    The window station handle can be created with NtUserCreateWindowStation
 *    or NtUserOpenWindowStation. Attempts to close a handle to the window
 *    station assigned to the calling process will fail.
 *
 * Status
 *    @implemented
 */

BOOL
APIENTRY
NtUserCloseWindowStation(
    HWINSTA hWinSta)
{
    PWINSTATION_OBJECT Object;
    NTSTATUS Status;

    TRACE("NtUserCloseWindowStation called (%p)\n", hWinSta);

    if (hWinSta == UserGetProcessWindowStation())
    {
        ERR("Attempted to close process window station\n");
        return FALSE;
    }

    Status = IntValidateWindowStationHandle(hWinSta,
                                            UserMode,
                                            0,
                                            &Object,
                                            NULL);
    if (!NT_SUCCESS(Status))
    {
        ERR("Validation of window station handle (%p) failed\n", hWinSta);
        return FALSE;
    }

    ObDereferenceObject(Object);

    TRACE("Closing window station handle (%p)\n", hWinSta);

    Status = ObCloseHandle(hWinSta, UserMode);
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return FALSE;
    }

    return TRUE;
}

/*
 * NtUserGetObjectInformation
 *
 * The NtUserGetObjectInformation function retrieves information about a
 * window station or desktop object.
 *
 * Parameters
 *    hObj
 *       Handle to the window station or desktop object for which to
 *       return information. This can be a handle of type HDESK or HWINSTA
 *       (for example, a handle returned by NtUserCreateWindowStation,
 *       NtUserOpenWindowStation, NtUserCreateDesktop, or NtUserOpenDesktop).
 *
 *    nIndex
 *       Specifies the object information to be retrieved.
 *
 *    pvInfo
 *       Pointer to a buffer to receive the object information.
 *
 *    nLength
 *       Specifies the size, in bytes, of the buffer pointed to by the
 *       pvInfo parameter.
 *
 *    lpnLengthNeeded
 *       Pointer to a variable receiving the number of bytes required to
 *       store the requested information. If this variable's value is
 *       greater than the value of the nLength parameter when the function
 *       returns, the function returns FALSE, and none of the information
 *       is copied to the pvInfo buffer. If the value of the variable pointed
 *       to by lpnLengthNeeded is less than or equal to the value of nLength,
 *       the entire information block is copied.
 *
 * Return Value
 *    If the function succeeds, the return value is nonzero. If the function
 *    fails, the return value is zero.
 *
 * Status
 *    @unimplemented
 */

BOOL APIENTRY
NtUserGetObjectInformation(
    HANDLE hObject,
    DWORD nIndex,
    PVOID pvInformation,
    DWORD nLength,
    PDWORD nLengthNeeded)
{
    NTSTATUS Status;
    PWINSTATION_OBJECT WinStaObject = NULL;
    PDESKTOP DesktopObject = NULL;
    POBJECT_HEADER ObjectHeader;
    POBJECT_HEADER_NAME_INFO NameInfo;
    OBJECT_HANDLE_INFORMATION HandleInfo;
    USEROBJECTFLAGS ObjectFlags;
    PUNICODE_STRING pStrNameU = NULL;
    PVOID pvData = NULL;
    SIZE_T nDataSize = 0;

    if (nLengthNeeded)
    {
        Status = SafeProbeForWrite(nLengthNeeded, sizeof(*nLengthNeeded), 1);
        if (!NT_SUCCESS(Status))
        {
            SetLastNtError(Status);
            return FALSE;
        }
    }
    Status = SafeProbeForWrite(pvInformation, nLength, 1);
    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return FALSE;
    }

    /* Try window station */
    TRACE("Trying to open window station 0x%p\n", hObject);
    Status = ObReferenceObjectByHandle(hObject,
                                       0,
                                       ExWindowStationObjectType,
                                       UserMode,
                                       (PVOID*)&WinStaObject,
                                       &HandleInfo);

    if (Status == STATUS_OBJECT_TYPE_MISMATCH)
    {
        /* Try desktop */
        TRACE("Trying to open desktop %p\n", hObject);
        WinStaObject = NULL;
        Status = IntValidateDesktopHandle(hObject,
                                          UserMode,
                                          0,
                                          &DesktopObject);
    }

    if (!NT_SUCCESS(Status))
    {
        ERR("Failed: 0x%x\n", Status);
        goto Exit;
    }

    TRACE("WinSta or Desktop opened!\n");

    /* Get data */
    switch (nIndex)
    {
        case UOI_FLAGS:
        {
            ObjectFlags.fReserved = FALSE;
            ObjectFlags.fInherit = !!(HandleInfo.HandleAttributes & OBJ_INHERIT);

            ObjectFlags.dwFlags = 0;
            if (WinStaObject != NULL)
            {
                if (!(WinStaObject->Flags & WSS_NOIO))
                    ObjectFlags.dwFlags |= WSF_VISIBLE;
            }
            else if (DesktopObject != NULL)
            {
                FIXME("Setting DF_ALLOWOTHERACCOUNTHOOK is unimplemented.\n");
            }
            else
            {
                ERR("No associated WinStaObject nor DesktopObject!\n");
            }

            pvData = &ObjectFlags;
            nDataSize = sizeof(ObjectFlags);
            Status = STATUS_SUCCESS;
            break;
        }

        case UOI_NAME:
        {
            if (WinStaObject != NULL)
            {
                ObjectHeader = OBJECT_TO_OBJECT_HEADER(WinStaObject);
                NameInfo = OBJECT_HEADER_TO_NAME_INFO(ObjectHeader);

                if (NameInfo && (NameInfo->Name.Length > 0))
                {
                    /* Named window station */
                    pStrNameU = &NameInfo->Name;
                    nDataSize = pStrNameU->Length + sizeof(UNICODE_NULL);
                }
                else
                {
                    /* Unnamed window station (should never happen!) */
                    ASSERT(FALSE);
                    pStrNameU = NULL;
                    nDataSize = sizeof(UNICODE_NULL);
                }
                Status = STATUS_SUCCESS;
            }
            else if (DesktopObject != NULL)
            {
                pvData = DesktopObject->pDeskInfo->szDesktopName;
                nDataSize = (wcslen(DesktopObject->pDeskInfo->szDesktopName) + 1) * sizeof(WCHAR);
                Status = STATUS_SUCCESS;
            }
            else
            {
                Status = STATUS_INVALID_PARAMETER;
            }
            break;
        }

        case UOI_TYPE:
        {
            if (WinStaObject != NULL)
            {
                ObjectHeader = OBJECT_TO_OBJECT_HEADER(WinStaObject);
                pStrNameU = &ObjectHeader->Type->Name;
                nDataSize = pStrNameU->Length + sizeof(UNICODE_NULL);
                Status = STATUS_SUCCESS;
            }
            else if (DesktopObject != NULL)
            {
                ObjectHeader = OBJECT_TO_OBJECT_HEADER(DesktopObject);
                pStrNameU = &ObjectHeader->Type->Name;
                nDataSize = pStrNameU->Length + sizeof(UNICODE_NULL);
                Status = STATUS_SUCCESS;
            }
            else
            {
                Status = STATUS_INVALID_PARAMETER;
            }
            break;
        }

        case UOI_USER_SID:
            Status = STATUS_NOT_IMPLEMENTED;
            ERR("UOI_USER_SID unimplemented!\n");
            break;

        default:
            Status = STATUS_INVALID_PARAMETER;
            break;
    }

Exit:
    if ((Status == STATUS_SUCCESS) && (nLength < nDataSize))
        Status = STATUS_BUFFER_TOO_SMALL;

    /*
     * Write results back to the caller. pvInformation/nLengthNeeded were
     * already validated by SafeProbeForWrite at function entry, so this
     * block needs no SEH of its own (it was belt-and-suspenders SEH on an
     * already-probed buffer; class (d) in the pretranslation audit).
     */
    if (nLengthNeeded)
        *nLengthNeeded = nDataSize;

    /* Try to copy data to caller */
    if (Status == STATUS_SUCCESS && (nDataSize > 0))
    {
        TRACE("Trying to copy data to caller (len = %lu, len needed = %lu)\n", nLength, nDataSize);
        if (pvData)
        {
            /* Copy the data */
            RtlCopyMemory(pvInformation, pvData, nDataSize);
        }
        else if (pStrNameU)
        {
            /* Copy and NULL-terminate the string */
            RtlCopyMemory(pvInformation, pStrNameU->Buffer, pStrNameU->Length);
            ((PWCHAR)pvInformation)[pStrNameU->Length / sizeof(WCHAR)] = UNICODE_NULL;
        }
        else
        {
            /* Zero the memory */
            RtlZeroMemory(pvInformation, nDataSize);
        }
    }

    /* Release objects */
    if (DesktopObject != NULL)
        ObDereferenceObject(DesktopObject);
    if (WinStaObject != NULL)
        ObDereferenceObject(WinStaObject);

    if (!NT_SUCCESS(Status))
    {
        SetLastNtError(Status);
        return FALSE;
    }

    return TRUE;
}

/*
 * NtUserSetObjectInformation
 *
 * The NtUserSetObjectInformation function sets information about a
 * window station or desktop object.
 *
 * Parameters
 *    hObj
 *       Handle to the window station or desktop object for which to set
 *       object information. This value can be a handle of type HDESK or
 *       HWINSTA.
 *
 *    nIndex
 *       Specifies the object information to be set.
 *
 *    pvInfo
 *       Pointer to a buffer containing the object information.
 *
 *    nLength
 *       Specifies the size, in bytes, of the information contained in the
 *       buffer pointed to by pvInfo.
 *
 * Return Value
 *    If the function succeeds, the return value is nonzero. If the function
 *    fails the return value is zero.
 *
 * Status
 *    @unimplemented
 */

BOOL
APIENTRY
NtUserSetObjectInformation(
    HANDLE hObject,
    DWORD nIndex,
    PVOID pvInformation,
    DWORD nLength)
{
    /* FIXME: ZwQueryObject */
    /* FIXME: ZwSetInformationObject */
    SetLastNtError(STATUS_UNSUCCESSFUL);
    return FALSE;
}


HWINSTA FASTCALL
UserGetProcessWindowStation(VOID)
{
    PPROCESSINFO ppi = PsGetCurrentProcessWin32Process();

    return ppi->hwinsta;
}


/*
 * NtUserGetProcessWindowStation
 *
 * Returns a handle to the current process window station.
 *
 * Return Value
 *    If the function succeeds, the return value is handle to the window
 *    station assigned to the current process. If the function fails, the
 *    return value is NULL.
 *
 * Status
 *    @implemented
 */

HWINSTA APIENTRY
NtUserGetProcessWindowStation(VOID)
{
    return UserGetProcessWindowStation();
}

BOOL FASTCALL
UserSetProcessWindowStation(HWINSTA hWindowStation)
{
    NTSTATUS Status;
    PPROCESSINFO ppi;
    OBJECT_HANDLE_INFORMATION ObjectHandleInfo;
    PWINSTATION_OBJECT NewWinSta = NULL, OldWinSta;
    HWINSTA hCacheWinSta;

    ppi = PsGetCurrentProcessWin32Process();

    /* Reference the new window station */
    if (hWindowStation != NULL)
    {
        Status = IntValidateWindowStationHandle(hWindowStation,
                                                UserMode,
                                                0,
                                                &NewWinSta,
                                                &ObjectHandleInfo);
        if (!NT_SUCCESS(Status))
        {
            TRACE("Validation of window station handle 0x%p failed\n", hWindowStation);
            SetLastNtError(Status);
            return FALSE;
        }
    }

    OldWinSta = ppi->prpwinsta;
    hCacheWinSta = PsGetProcessWin32WindowStation(ppi->peProcess);

    /* Dereference the previous window station */
    if (OldWinSta != NULL)
    {
        ObDereferenceObject(OldWinSta);
    }

    /*
     * FIXME: Don't allow changing the window station if there are threads that are attached to desktops and own GUI objects?
     */

    /* Close the cached EPROCESS window station handle if needed */
    if (hCacheWinSta != NULL)
    {
        /* Reference the window station */
        Status = ObReferenceObjectByHandle(hCacheWinSta,
                                           0,
                                           ExWindowStationObjectType,
                                           UserMode,
                                           (PVOID*)&OldWinSta,
                                           NULL);
        if (!NT_SUCCESS(Status))
        {
            ERR("Failed to reference the inherited window station, Status 0x%08lx\n", Status);
            /* We failed, reset the cache */
            hCacheWinSta = NULL;
            PsSetProcessWindowStation(ppi->peProcess, hCacheWinSta);
        }
        else
        {
            /*
             * Close the old handle and reset the cache only
             * if we are setting a different window station.
             */
            if (NewWinSta != OldWinSta)
            {
                ObCloseHandle(hCacheWinSta, UserMode);
                hCacheWinSta = NULL;
                PsSetProcessWindowStation(ppi->peProcess, hCacheWinSta);
            }

            /* Dereference the window station */
            ObDereferenceObject(OldWinSta);
        }
    }

    /* Duplicate and save a new cached EPROCESS window station handle */
    if ((hCacheWinSta == NULL) && (hWindowStation != NULL))
    {
        Status = ZwDuplicateObject(ZwCurrentProcess(),
                                   hWindowStation,
                                   ZwCurrentProcess(),
                                   (PHANDLE)&hCacheWinSta,
                                   0,
                                   0,
                                   DUPLICATE_SAME_ACCESS);
        if (!NT_SUCCESS(Status))
        {
            ERR("UserSetProcessWindowStation: Failed to duplicate the window station handle, Status 0x%08lx\n", Status);
        }
        else
        {
            PsSetProcessWindowStation(ppi->peProcess, hCacheWinSta);
        }
    }

    ppi->prpwinsta = NewWinSta;
    ppi->hwinsta = hWindowStation;
    ppi->amwinsta = hWindowStation != NULL ? ObjectHandleInfo.GrantedAccess : 0;
    TRACE("WS : Granted Access 0x%08lx\n",ppi->amwinsta);

    if (RtlAreAllAccessesGranted(ppi->amwinsta, WINSTA_READSCREEN))
    {
        ppi->W32PF_flags |= W32PF_READSCREENACCESSGRANTED;
    }
    else
    {
        ppi->W32PF_flags &= ~W32PF_READSCREENACCESSGRANTED;
    }

    if (NewWinSta && !(NewWinSta->Flags & WSS_NOIO))
    {
        ppi->W32PF_flags |= W32PF_IOWINSTA;
    }
    else /* Might be closed if the handle is NULL */
    {
        ppi->W32PF_flags &= ~W32PF_IOWINSTA;
    }
    return TRUE;
}

/*
 * NtUserSetProcessWindowStation
 *
 * Assigns a window station to the current process.
 *
 * Parameters
 *    hWinSta
 *       Handle to the window station.
 *
 * Return Value
 *    Status
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserSetProcessWindowStation(HWINSTA hWindowStation)
{
    BOOL ret;

    UserEnterExclusive();

    ret = UserSetProcessWindowStation(hWindowStation);

    UserLeave();

    return ret;
}

/*
 * NtUserLockWindowStation
 *
 * Locks switching desktops. Only the logon application is allowed to call this function.
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserLockWindowStation(HWINSTA hWindowStation)
{
    PWINSTATION_OBJECT Object;
    NTSTATUS Status;

    TRACE("About to set process window station with handle (%p)\n",
          hWindowStation);

    if (gpidLogon != PsGetCurrentProcessId())
    {
        ERR("Unauthorized process attempted to lock the window station!\n");
        EngSetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }

    Status = IntValidateWindowStationHandle(hWindowStation,
                                            UserMode,
                                            0,
                                            &Object,
                                            NULL);
    if (!NT_SUCCESS(Status))
    {
        TRACE("Validation of window station handle (%p) failed\n",
              hWindowStation);
        SetLastNtError(Status);
        return FALSE;
    }

    Object->Flags |= WSS_LOCKED;

    ObDereferenceObject(Object);
    return TRUE;
}

/*
 * NtUserUnlockWindowStation
 *
 * Unlocks switching desktops. Only the logon application is allowed to call this function.
 *
 * Status
 *    @implemented
 */

BOOL APIENTRY
NtUserUnlockWindowStation(HWINSTA hWindowStation)
{
    PWINSTATION_OBJECT Object;
    NTSTATUS Status;
    BOOL Ret;

    TRACE("About to set process window station with handle (%p)\n",
          hWindowStation);

    if (gpidLogon != PsGetCurrentProcessId())
    {
        ERR("Unauthorized process attempted to unlock the window station!\n");
        EngSetLastError(ERROR_ACCESS_DENIED);
        return FALSE;
    }

    Status = IntValidateWindowStationHandle(hWindowStation,
                                            UserMode,
                                            0,
                                            &Object,
                                            NULL);
    if (!NT_SUCCESS(Status))
    {
        TRACE("Validation of window station handle (%p) failed\n",
              hWindowStation);
        SetLastNtError(Status);
        return FALSE;
    }

    Ret = (Object->Flags & WSS_LOCKED) == WSS_LOCKED;
    Object->Flags &= ~WSS_LOCKED;

    ObDereferenceObject(Object);
    return Ret;
}

static NTSTATUS FASTCALL
BuildWindowStationNameList(
    ULONG dwSize,
    PVOID lpBuffer,
    PULONG pRequiredSize)
{
    OBJECT_ATTRIBUTES ObjectAttributes;
    NTSTATUS Status;
    HANDLE DirectoryHandle;
    char InitialBuffer[256], *Buffer;
    ULONG Context, ReturnLength, BufferSize;
    DWORD EntryCount;
    POBJECT_DIRECTORY_INFORMATION DirEntry;
    WCHAR NullWchar;

    //
    // FIXME: Fully wrong! Since, by calling NtUserCreateWindowStation
    // with judicious parameters one can create window stations elsewhere
    // than in Windows\WindowStations directory, Win32k definitely MUST
    // maintain a list of window stations it has created, and not rely
    // on the enumeration of Windows\WindowStations !!!
    //

    /*
     * Try to open the directory.
     */
    InitializeObjectAttributes(&ObjectAttributes,
                               &gustrWindowStationsDir,
                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
                               NULL,
                               NULL);

    Status = ZwOpenDirectoryObject(&DirectoryHandle,
                                   DIRECTORY_QUERY,
                                   &ObjectAttributes);

    if (!NT_SUCCESS(Status))
    {
        return Status;
    }

    /* First try to query the directory using a fixed-size buffer */
    Context = 0;
    Buffer = NULL;
    Status = ZwQueryDirectoryObject(DirectoryHandle,
                                    InitialBuffer,
                                    sizeof(InitialBuffer),
                                    FALSE,
                                    TRUE,
                                    &Context,
                                    &ReturnLength);
    if (NT_SUCCESS(Status))
    {
        if (STATUS_NO_MORE_ENTRIES == ZwQueryDirectoryObject(DirectoryHandle, NULL, 0, FALSE,
                                                             FALSE, &Context, NULL))
        {
            /* Our fixed-size buffer is large enough */
            Buffer = InitialBuffer;
        }
    }

    if (NULL == Buffer)
    {
        /* Need a larger buffer, check how large exactly */
        Status = ZwQueryDirectoryObject(DirectoryHandle, NULL, 0, FALSE, TRUE, &Context,
                                        &ReturnLength);
        if (!NT_SUCCESS(Status))
        {
            ERR("ZwQueryDirectoryObject failed\n");
            ZwClose(DirectoryHandle);
            return Status;
        }

        BufferSize = ReturnLength;
        Buffer = ExAllocatePoolWithTag(PagedPool, BufferSize, TAG_WINSTA);
        if (NULL == Buffer)
        {
            ZwClose(DirectoryHandle);
            return STATUS_NO_MEMORY;
        }

        /* We should have a sufficiently large buffer now */
        Context = 0;
        Status = ZwQueryDirectoryObject(DirectoryHandle, Buffer, BufferSize,
                                        FALSE, TRUE, &Context, &ReturnLength);
        if (! NT_SUCCESS(Status) ||
              STATUS_NO_MORE_ENTRIES != ZwQueryDirectoryObject(DirectoryHandle, NULL, 0, FALSE,
                                                               FALSE, &Context, NULL))
        {
            /* Something went wrong, maybe someone added a directory entry? Just give up. */
            ExFreePoolWithTag(Buffer, TAG_WINSTA);
            ZwClose(DirectoryHandle);
            return NT_SUCCESS(Status) ? STATUS_INTERNAL_ERROR : Status;
        }
    }

    ZwClose(DirectoryHandle);

    /*
     * Count the required size of buffer.
     */
    ReturnLength = sizeof(DWORD);
    EntryCount = 0;
    for (DirEntry = (POBJECT_DIRECTORY_INFORMATION) Buffer;
         0 != DirEntry->Name.Length;
         DirEntry++)
    {
        ReturnLength += DirEntry->Name.Length + sizeof(WCHAR);
        EntryCount++;
    }
    TRACE("Required size: %lu Entry count: %lu\n", ReturnLength, EntryCount);
    if (NULL != pRequiredSize)
    {
        Status = MmCopyToCaller(pRequiredSize, &ReturnLength, sizeof(ULONG));
        if (! NT_SUCCESS(Status))
        {
            if (Buffer != InitialBuffer)
            {
                ExFreePoolWithTag(Buffer, TAG_WINSTA);
            }
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    /*
     * Check if the supplied buffer is large enough.
     */
    if (dwSize < ReturnLength)
    {
        if (Buffer != InitialBuffer)
        {
            ExFreePoolWithTag(Buffer, TAG_WINSTA);
        }
        return STATUS_BUFFER_TOO_SMALL;
    }

    /*
     * Generate the resulting buffer contents.
     */
    Status = MmCopyToCaller(lpBuffer, &EntryCount, sizeof(DWORD));
    if (! NT_SUCCESS(Status))
    {
        if (Buffer != InitialBuffer)
        {
            ExFreePoolWithTag(Buffer, TAG_WINSTA);
        }
        return Status;
    }
    lpBuffer = (PVOID) ((PCHAR) lpBuffer + sizeof(DWORD));

    NullWchar = L'\0';
    for (DirEntry = (POBJECT_DIRECTORY_INFORMATION) Buffer;
         0 != DirEntry->Name.Length;
         DirEntry++)
    {
        Status = MmCopyToCaller(lpBuffer, DirEntry->Name.Buffer, DirEntry->Name.Length);
        if (! NT_SUCCESS(Status))
        {
            if (Buffer != InitialBuffer)
            {
                ExFreePoolWithTag(Buffer, TAG_WINSTA);
            }
            return Status;
        }
        lpBuffer = (PVOID) ((PCHAR) lpBuffer + DirEntry->Name.Length);
        Status = MmCopyToCaller(lpBuffer, &NullWchar, sizeof(WCHAR));
        if (! NT_SUCCESS(Status))
        {
            if (Buffer != InitialBuffer)
            {
                ExFreePoolWithTag(Buffer, TAG_WINSTA);
            }
            return Status;
        }
        lpBuffer = (PVOID) ((PCHAR) lpBuffer + sizeof(WCHAR));
    }

    /*
     * Clean up
     */
    if (Buffer != InitialBuffer)
    {
        ExFreePoolWithTag(Buffer, TAG_WINSTA);
    }

    return STATUS_SUCCESS;
}

static NTSTATUS FASTCALL
BuildDesktopNameList(
    HWINSTA hWindowStation,
    ULONG dwSize,
    PVOID lpBuffer,
    PULONG pRequiredSize)
{
    NTSTATUS Status;
    PWINSTATION_OBJECT WindowStation;
    PLIST_ENTRY DesktopEntry;
    PDESKTOP DesktopObject;
    DWORD EntryCount;
    ULONG ReturnLength;
    WCHAR NullWchar;
    UNICODE_STRING DesktopName;

    Status = IntValidateWindowStationHandle(hWindowStation,
                                            UserMode,
                                            0,
                                            &WindowStation,
                                            NULL);
    if (! NT_SUCCESS(Status))
    {
        return Status;
    }

    /*
     * Count the required size of buffer.
     */
    ReturnLength = sizeof(DWORD);
    EntryCount = 0;
    for (DesktopEntry = WindowStation->DesktopListHead.Flink;
         DesktopEntry != &WindowStation->DesktopListHead;
         DesktopEntry = DesktopEntry->Flink)
    {
        DesktopObject = CONTAINING_RECORD(DesktopEntry, DESKTOP, ListEntry);
        RtlInitUnicodeString(&DesktopName, DesktopObject->pDeskInfo->szDesktopName);
        ReturnLength += DesktopName.Length + sizeof(WCHAR);
        EntryCount++;
    }
    TRACE("Required size: %lu Entry count: %lu\n", ReturnLength, EntryCount);
    if (NULL != pRequiredSize)
    {
        Status = MmCopyToCaller(pRequiredSize, &ReturnLength, sizeof(ULONG));
        if (! NT_SUCCESS(Status))
        {
            ObDereferenceObject(WindowStation);
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    /*
     * Check if the supplied buffer is large enough.
     */
    if (dwSize < ReturnLength)
    {
        ObDereferenceObject(WindowStation);
        return STATUS_BUFFER_TOO_SMALL;
    }

    /*
     * Generate the resulting buffer contents.
     */
    Status = MmCopyToCaller(lpBuffer, &EntryCount, sizeof(DWORD));
    if (! NT_SUCCESS(Status))
    {
        ObDereferenceObject(WindowStation);
        return Status;
    }
    lpBuffer = (PVOID) ((PCHAR) lpBuffer + sizeof(DWORD));

    NullWchar = L'\0';
    for (DesktopEntry = WindowStation->DesktopListHead.Flink;
         DesktopEntry != &WindowStation->DesktopListHead;
         DesktopEntry = DesktopEntry->Flink)
    {
        DesktopObject = CONTAINING_RECORD(DesktopEntry, DESKTOP, ListEntry);
        RtlInitUnicodeString(&DesktopName, DesktopObject->pDeskInfo->szDesktopName);
        Status = MmCopyToCaller(lpBuffer, DesktopName.Buffer, DesktopName.Length);
        if (! NT_SUCCESS(Status))
        {
            ObDereferenceObject(WindowStation);
            return Status;
        }
        lpBuffer = (PVOID) ((PCHAR)lpBuffer + DesktopName.Length);
        Status = MmCopyToCaller(lpBuffer, &NullWchar, sizeof(WCHAR));
        if (! NT_SUCCESS(Status))
        {
            ObDereferenceObject(WindowStation);
            return Status;
        }
        lpBuffer = (PVOID) ((PCHAR) lpBuffer + sizeof(WCHAR));
    }

    /*
     * Clean up and return
     */
    ObDereferenceObject(WindowStation);
    return STATUS_SUCCESS;
}

/*
 * NtUserBuildNameList
 *
 * Function used for enumeration of desktops or window stations.
 *
 * Parameters
 *    hWinSta
 *       For enumeration of window stations this parameter must be set to
 *       zero. Otherwise it's handle for window station.
 *
 *    dwSize
 *       Size of buffer passed by caller.
 *
 *    lpBuffer
 *       Buffer passed by caller. If the function succeeds, the buffer is
 *       filled with window station/desktop count (in first DWORD) and
 *       NULL-terminated window station/desktop names.
 *
 *    pRequiredSize
 *       If the function succeeds, this is the number of bytes copied.
 *       Otherwise it's size of buffer needed for function to succeed.
 *
 * Status
 *    @implemented
 */

NTSTATUS APIENTRY
NtUserBuildNameList(
    HWINSTA hWindowStation,
    ULONG dwSize,
    PVOID lpBuffer,
    PULONG pRequiredSize)
{
    /* The WindowStation name list and desktop name list are build in completely
       different ways. Call the appropriate function */
    return NULL == hWindowStation ? BuildWindowStationNameList(dwSize, lpBuffer, pRequiredSize) :
           BuildDesktopNameList(hWindowStation, dwSize, lpBuffer, pRequiredSize);
}

/*
 * @implemented
 */
BOOL APIENTRY
NtUserSetLogonNotifyWindow(HWND hWnd)
{
    BOOL Ret = FALSE;

    UserEnterExclusive();

    if (!IntIsWindow(hWnd))
        goto Leave;

    if (gpidLogon != PsGetCurrentProcessId())
    {
        EngSetLastError(ERROR_ACCESS_DENIED);
        goto Leave;
    }

    hwndSAS = hWnd;
    Ret = TRUE;

Leave:
    UserLeave();
    return Ret;
}

BOOL
APIENTRY
NtUserLockWorkStation(VOID)
{
    BOOL ret;
    PTHREADINFO pti = PsGetCurrentThreadWin32Thread();

    UserEnterExclusive();

    if (pti->rpdesk == IntGetActiveDesktop())
    {
        ret = UserPostMessage(hwndSAS, WM_LOGONNOTIFY, LN_LOCK_WORKSTATION, 0);
    }
    else
    {
        ret = FALSE;
    }

    UserLeave();

    return ret;
}

BOOL
NTAPI
NtUserSetWindowStationUser(
    IN HWINSTA hWindowStation,
    IN PLUID pluid,
    IN PSID psid OPTIONAL,
    IN DWORD size)
{
    BOOL Ret = FALSE;
    NTSTATUS Status;
    PWINSTATION_OBJECT WindowStation = NULL;
    LUID luidUser;

    UserEnterExclusive();

    if (gpidLogon != PsGetCurrentProcessId())
    {
        EngSetLastError(ERROR_ACCESS_DENIED);
        goto Leave;
    }

    /* Validate the window station */
    Status = IntValidateWindowStationHandle(hWindowStation,
                                            UserMode,
                                            0,
                                            &WindowStation,
                                            NULL);
    if (!NT_SUCCESS(Status))
    {
        goto Leave;
    }

    /* Capture the user LUID */
    Status = SafeProbeForRead(pluid, sizeof(LUID), 1);
    if (!NT_SUCCESS(Status))
        goto Leave;
    luidUser = *pluid;

    /* Reset the window station user LUID */
    RtlZeroMemory(&WindowStation->luidUser, sizeof(LUID));

    /* Reset the window station user SID */
    if (WindowStation->psidUser)
    {
        ExFreePoolWithTag(WindowStation->psidUser, USERTAG_SECURITY);
        WindowStation->psidUser = NULL;
    }

    /* Copy the new user SID if one has been provided */
    if (psid)
    {
        WindowStation->psidUser = ExAllocatePoolWithTag(PagedPool, size, USERTAG_SECURITY);
        if (WindowStation->psidUser == NULL)
        {
            EngSetLastError(ERROR_OUTOFMEMORY);
            goto Leave;
        }

        Status = STATUS_SUCCESS;
        Status = SafeProbeForRead(psid, size, 1);
        if (NT_SUCCESS(Status))
            RtlCopyMemory(WindowStation->psidUser, psid, size);

        if (!NT_SUCCESS(Status))
        {
            ExFreePoolWithTag(WindowStation->psidUser, USERTAG_SECURITY);
            WindowStation->psidUser = NULL;
            goto Leave;
        }
    }

    /* Copy the new user LUID */
    WindowStation->luidUser = luidUser;

    Ret = TRUE;

Leave:
    if (WindowStation)
        ObDereferenceObject(WindowStation);

    UserLeave();
    return Ret;
}

/* EOF */
