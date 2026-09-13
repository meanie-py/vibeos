// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut szRootDeviceInstanceID: [u16; 14] = [
    72, 84, 82, 69, 69, 92, 82, 79, 79, 84, 92, 48, 0, 0
];

pub static mut LoadDriverPrivilege: LUID = LUID {
    LowPart: (SE_LOAD_DRIVER_PRIVILEGE) as u32,
    HighPart: 0,
};

// precomp.h declares every one of these `extern`: they are module globals with EXTERNAL
// linkage, shared across the four TUs. Without #[no_mangle] each definition is mangled
// while sibling modules reference the plain name, and the crate links against undefined
// symbols — the failure only appears at LINK, never in `cargo check`.
#[no_mangle]
pub static mut NotificationListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

// rpcserver.c:43 is `RTL_RESOURCE NotificationListLock;` — a bare TENTATIVE DEFINITION,
// which C zero-initialises. The translation instead wrote a full initializer with nine
// field names RECALLED rather than read (the real ntdll layout is Lock/SharedWaiters/
// ExclusiveWaiters/NumberActive/OwningThread/TimeoutBoost). Same bug msv1_0 shipped for
// LogonListResource. Zeroed, which is what the C actually says.
#[no_mangle]
pub static mut NotificationListLock: RTL_RESOURCE = unsafe { core::mem::zeroed() };
#[no_mangle]
pub unsafe extern "system" fn RpcServerThread(lpParameter: LPVOID) -> DWORD {
    let mut Status: RPC_STATUS;
    let mut RegisteredProtSeq: BOOLEAN = (FALSE) as u8;

    UNREFERENCED_PARAMETER!(lpParameter);

    // DPRINT removed

    InitializeListHead(&mut NotificationListHead);
    RtlInitializeResource(&mut NotificationListLock);

    Status = RpcServerUseProtseqEpW(
        w!("ncacn_np") as PWSTR,
        20,
        w!("\\pipe\\plugplay") as PWSTR,
        core::ptr::null_mut(),
    );
    if Status == RPC_S_OK {
        RegisteredProtSeq = (TRUE) as u8;
    } else {
        // DPRINT removed
    }

    if RegisteredProtSeq == (FALSE) as u8 {
        return 0;
    }

    Status = RpcServerRegisterIf(
        pnp_v1_0_s_ifspec,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Status != RPC_S_OK {
        // DPRINT removed
        return 0;
    }

    Status = RpcServerListen(
        1,
        20,
        FALSE,
    );
    if Status != RPC_S_OK {
        // DPRINT removed
        return 0;
    }

    // DPRINT removed
    // DPRINT removed

    // DPRINT removed

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}
unsafe extern "system" fn NtStatusToCrError(Status: NTSTATUS) -> CONFIGRET {
    match Status {
        STATUS_NOT_IMPLEMENTED => CR_CALL_NOT_IMPLEMENTED,
        STATUS_INVALID_PARAMETER => CR_INVALID_DATA,
        STATUS_NO_SUCH_DEVICE => CR_NO_SUCH_DEVINST,
        STATUS_ACCESS_DENIED => CR_ACCESS_DENIED,
        STATUS_BUFFER_TOO_SMALL => CR_BUFFER_SMALL,
        STATUS_OBJECT_NAME_NOT_FOUND => CR_NO_SUCH_VALUE,
        _ => CR_FAILURE,
    }
}
unsafe extern "C" fn SplitDeviceInstanceID(
    pszDeviceInstanceID: LPWSTR,
    pszEnumerator: LPWSTR,
    pszDevice: LPWSTR,
    pszInstance: LPWSTR,
) {
    let mut szLocalDeviceInstanceID: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut lpEnumerator: LPWSTR = core::ptr::null_mut();
    let mut lpDevice: LPWSTR = core::ptr::null_mut();
    let mut lpInstance: LPWSTR = core::ptr::null_mut();
    let mut ptr: LPWSTR;

    wcscpy(szLocalDeviceInstanceID.as_mut_ptr(), pszDeviceInstanceID);

    *pszEnumerator = 0;
    *pszDevice = 0;
    *pszInstance = 0;

    lpEnumerator = szLocalDeviceInstanceID.as_mut_ptr();

    ptr = wcschr(lpEnumerator, '\\' as u16);
    if !ptr.is_null() {
        *ptr = 0;
        lpDevice = ptr.offset(1);

        ptr = wcschr(lpDevice, '\\' as u16);
        if !ptr.is_null() {
            *ptr = 0;
            lpInstance = ptr.offset(1);
        }
    }

    if !lpEnumerator.is_null() {
        wcscpy(pszEnumerator, lpEnumerator);
    }

    if !lpDevice.is_null() {
        wcscpy(pszDevice, lpDevice);
    }

    if !lpInstance.is_null() {
        wcscpy(pszInstance, lpInstance);
    }
}
unsafe extern "C" fn ClearDeviceStatus(
    pszDeviceID: LPWSTR,
    ulStatus: DWORD,
    ulProblem: DWORD,
) -> CONFIGRET {
    let mut PlugPlayData: PLUGPLAY_CONTROL_STATUS_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    RtlInitUnicodeString(
        &mut PlugPlayData.DeviceInstance,
        pszDeviceID,
    );
    PlugPlayData.Operation = PNP_CLEAR_DEVICE_STATUS;
    PlugPlayData.DeviceStatus = ulStatus;
    PlugPlayData.DeviceProblem = ulProblem;

    Status = NtPlugPlayControl(
        PlugPlayControlDeviceStatus,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_STATUS_DATA>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
unsafe extern "C" fn GetDeviceStatus(
    pszDeviceID: LPWSTR,
    pulStatus: *mut DWORD,
    pulProblem: *mut DWORD,
) -> CONFIGRET {
    let mut PlugPlayData: PLUGPLAY_CONTROL_STATUS_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );
    PlugPlayData.Operation = PNP_GET_DEVICE_STATUS;

    Status = NtPlugPlayControl(
        PlugPlayControlDeviceStatus,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_STATUS_DATA>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        *pulStatus = PlugPlayData.DeviceStatus;
        *pulProblem = PlugPlayData.DeviceProblem;
    } else {
        ret = NtStatusToCrError(Status);
    }

    ret
}
unsafe extern "C" fn SetDeviceStatus(
    pszDeviceID: LPWSTR,
    ulStatus: DWORD,
    ulProblem: DWORD,
) -> CONFIGRET {
    let mut PlugPlayData: PLUGPLAY_CONTROL_STATUS_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );
    PlugPlayData.Operation = PNP_SET_DEVICE_STATUS;
    PlugPlayData.DeviceStatus = ulStatus;
    PlugPlayData.DeviceProblem = ulProblem;

    Status = NtPlugPlayControl(
        PlugPlayControlDeviceStatus,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_STATUS_DATA>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
unsafe extern "C" fn DisableDeviceInstance(
    pszDeviceInstance: LPWSTR,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: DWORD,
) -> CONFIGRET {
    let mut QueryRemoveData: PLUGPLAY_CONTROL_QUERY_REMOVE_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(QueryRemoveData.DeviceInstance),
        pszDeviceInstance,
    );

    QueryRemoveData.Flags = 0;
    QueryRemoveData.VetoType = 0;
    QueryRemoveData.VetoName = pszVetoName;
    QueryRemoveData.NameLength = ulNameLength;

    Status = NtPlugPlayControl(
        PlugPlayControlQueryAndRemoveDevice,
        core::ptr::addr_of_mut!(QueryRemoveData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_QUERY_REMOVE_DATA>() as ULONG,
    );
    if Status == STATUS_NO_SUCH_DEVICE {
        ret = (CR_INVALID_DEVNODE) as u32;
    } else if Status == STATUS_PLUGPLAY_QUERY_VETOED {
        if !pVetoType.is_null() {
            *pVetoType = QueryRemoveData.VetoType;
        }

        ret = (CR_REMOVE_VETOED) as u32;
    } else if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
unsafe extern "C" fn IsRootDeviceInstanceID(
    pszDeviceInstanceID: PWSTR,
) -> BOOL {
    if _wcsicmp(pszDeviceInstanceID, szRootDeviceInstanceID.as_ptr() as PWSTR) == 0 {
        return (TRUE) as i32;
    }

    return (FALSE) as i32;
}
// HAND-WRITTEN, transcribed from rpcserver.c:342-407.
// `translate --seal-facts` rejected the model's body twice with the same damage (facts 0020 and
// 0022 dropped from the envelope). Recall is stable across samples, so a third sample would not
// have helped; --skip-fn kept it from blocking the other 109 and recorded the omission as an
// audit finding. Every early-out is a `return FALSE`, so a subtly-wrong version of this would
// not fail loudly — it would accept malformed device instance IDs into the registry paths built
// from them, which is why it is transcribed line by line rather than paraphrased.
unsafe extern "C" fn IsValidDeviceInstanceID(
    pszDeviceInstanceID: PWSTR,
) -> BOOL {
    let mut nPartLength: [INT; 3] = [0, 0, 0];
    let mut nLength: INT = 0;
    let mut nParts: INT = 0;
    let mut p: PWCHAR;

    if pszDeviceInstanceID.is_null() {
        return (FALSE) as i32;
    }

    p = pszDeviceInstanceID;
    while *p != UNICODE_NULL {
        if *p == b'\\' as WCHAR {
            nParts += 1;
            if nParts >= 3 {
                return (FALSE) as i32;
            }
        } else {
            nPartLength[nParts as usize] += 1;
        }

        nLength += 1;
        if nLength >= (MAX_DEVICE_ID_LEN) as i32 {
            return (FALSE) as i32;
        }

        p = p.add(1);
    }

    if nParts != 2 {
        return (FALSE) as i32;
    }

    if (nPartLength[0] == 0) || (nPartLength[1] == 0) || (nPartLength[2] == 0) {
        return (FALSE) as i32;
    }

    (TRUE) as i32
}
unsafe extern "C" fn IsPresentDeviceInstanceID(
    pszDeviceInstanceID: LPWSTR,
) -> BOOL {
    let mut ulStatus: DWORD = 0;
    let mut ulProblem: DWORD = 0;

    (GetDeviceStatus(pszDeviceInstanceID, &mut ulStatus, &mut ulProblem) == (CR_SUCCESS) as u32) as BOOL
}
unsafe extern "C" fn OpenConfigurationKey(
    pszDeviceID: LPCWSTR,
    ulLogConfType: DWORD,
    phKey: PHKEY,
) -> CONFIGRET {
    let mut szKeyName: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let pszSubKeyName: PCWSTR;
    let mut hInstanceKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD;

    wcscpy(szKeyName.as_mut_ptr(), w!("System\\CurrentControlSet\\Enum\\") as LPCWSTR);
    wcscat(szKeyName.as_mut_ptr(), pszDeviceID);

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyName.as_ptr(),
        0,
        KEY_QUERY_VALUE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY,
        &mut hInstanceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return (CR_INVALID_DEVINST) as u32;
    }

    match ulLogConfType {
        BOOT_LOG_CONF | BASIC_LOG_CONF => {
            pszSubKeyName = w!("LogConf") as PCWSTR;
        }
        ALLOC_LOG_CONF | FILTERED_LOG_CONF => {
            pszSubKeyName = w!("Control") as PCWSTR;
        }
        _ => {
            // DPRINT removed
            return (CR_FAILURE) as u32;
        }
    }

    dwError = (RegCreateKeyExW(
        hInstanceKey,
        pszSubKeyName,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        phKey,
        core::ptr::null_mut(),
    )) as u32;

    RegCloseKey(hInstanceKey);

    if dwError != ERROR_SUCCESS {
        return (CR_REGISTRY_ERROR) as u32;
    }

    return (CR_SUCCESS) as u32;
}
unsafe extern "C" fn GetConfigurationData(
    hKey: HKEY,
    ulLogConfType: ULONG,
    pulRegDataType: PULONG,
    pulDataSize: PULONG,
    ppBuffer: *mut LPBYTE,
    ulValueNameBufferSize: ULONG,
    pszValueNameBuffer: LPWSTR,
) -> CONFIGRET {
    let pszValueName: LPCWSTR;

    match ulLogConfType {
        BOOT_LOG_CONF => {
            pszValueName = w!("BootConfig") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_LIST) as u32;
        }

        ALLOC_LOG_CONF => {
            pszValueName = w!("AllocConfig") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_LIST) as u32;
        }

        FORCED_LOG_CONF => {
            pszValueName = w!("ForcedConfig") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_LIST) as u32;
        }

        FILTERED_LOG_CONF => {
            pszValueName = w!("FilteredConfigVector") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_REQUIREMENTS_LIST) as u32;
        }

        BASIC_LOG_CONF => {
            pszValueName = w!("BasicConfigVector") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_REQUIREMENTS_LIST) as u32;
        }

        OVERRIDE_LOG_CONF => {
            pszValueName = w!("OverrideConfigVector") as LPCWSTR;
            *pulRegDataType = (REG_RESOURCE_REQUIREMENTS_LIST) as u32;
        }

        _ => {
            // DPRINT removed
            return (CR_FAILURE) as u32;
        }
    }

    if (ulValueNameBufferSize > 0) && !pszValueNameBuffer.is_null() &&
        (wcslen(pszValueName) + 1 <= (ulValueNameBufferSize) as usize) {
        wcscpy(pszValueNameBuffer, pszValueName);
    }

    if RegQueryValueExW(
        hKey,
        pszValueName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pulDataSize,
    ) != (ERROR_SUCCESS) as i32
    {
        return (CR_INVALID_LOG_CONF) as u32;
    }

    *ppBuffer = (HeapAlloc(GetProcessHeap(), 0, *pulDataSize as SIZE_T)).cast();
    if (*ppBuffer).is_null() {
        return (CR_OUT_OF_MEMORY) as u32;
    }

    if RegQueryValueExW(
        hKey,
        pszValueName,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        *ppBuffer as LPBYTE,
        pulDataSize,
    ) != (ERROR_SUCCESS) as i32
    {
        return (CR_INVALID_LOG_CONF) as u32;
    }

    return (CR_SUCCESS) as u32;
}
unsafe extern "C" fn NextResourceDescriptor(
    pDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR,
) -> PCM_FULL_RESOURCE_DESCRIPTOR {
    let mut pNext: PBYTE = core::ptr::null_mut();
    let mut ulLastIndex: ULONG = 0;

    if pDescriptor.is_null() {
        return core::ptr::null_mut();
    }

    pNext = (pDescriptor as LPBYTE).add(core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>());

    pNext = pNext.add(
        (((*pDescriptor).PartialResourceList.Count - 1) as usize)
            * core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
    );

    if (*pDescriptor).PartialResourceList.Count > 0 {
        ulLastIndex = (*pDescriptor).PartialResourceList.Count - 1;

        if (*(&raw mut (*pDescriptor).PartialResourceList.PartialDescriptors[0]).add(ulLastIndex as usize)).Type
            == (CmResourceTypeDeviceSpecific) as u8
        {
            pNext = pNext.add(
                (*(&raw mut (*pDescriptor).PartialResourceList.PartialDescriptors[0]).add(ulLastIndex as usize))
                    .u
                    .DeviceSpecificData
                    .DataSize as usize,
            );
        }
    }

    pNext as PCM_FULL_RESOURCE_DESCRIPTOR
}
unsafe extern "C" fn NextResourceRequirement(
    pResourceList: PIO_RESOURCE_LIST,
) -> PIO_RESOURCE_LIST {
    let mut pNext: LPBYTE = core::ptr::null_mut();

    if pResourceList.is_null() {
        return core::ptr::null_mut();
    }

    pNext = pResourceList as LPBYTE;
    pNext = pNext.add(core::mem::size_of::<IO_RESOURCE_LIST>());

    pNext = pNext.add(
        ((*pResourceList).Count as usize - 1) 
        * core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>()
    );

    pNext as PIO_RESOURCE_LIST
}
unsafe extern "C" fn IsCallerInteractive(hBinding: handle_t) -> BOOL {
    let mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    let mut hToken: HANDLE = core::ptr::null_mut();
    let mut pInteractiveSid: PSID = core::ptr::null_mut();
    let mut bInteractive: BOOL = (FALSE) as i32;
    let RpcStatus: RPC_STATUS;

    // DPRINT removed

    if AllocateAndInitializeSid(
        &mut NtAuthority,
        1,
        SECURITY_INTERACTIVE_RID,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        &mut pInteractiveSid,
    ) == 0
    {
        // DPRINT removed
        return (FALSE) as i32;
    }

    RpcStatus = RpcImpersonateClient(hBinding);
    if RpcStatus != RPC_S_OK {
        // DPRINT removed
    } else {
        if OpenThreadToken(
            GetCurrentThread(),
            TOKEN_QUERY,
            (FALSE) as i32,
            &mut hToken,
        ) != 0
        {
            CheckTokenMembership(hToken, pInteractiveSid, &mut bInteractive);
            CloseHandle(hToken);
        }

        RpcRevertToSelf();
    }

    if !pInteractiveSid.is_null() {
        FreeSid(pInteractiveSid);
    }

    bInteractive
}

#[no_mangle]
pub unsafe extern "system" fn PNP_NOTIFY_HANDLE_rundown(pHandle: PNP_NOTIFY_HANDLE) {
    // DPRINT1!("PNP_NOTIFY_HANDLE_rundown(%p)\n", pHandle);
}

#[no_mangle]
pub unsafe extern "system" fn PNP_Disconnect(
    hBinding: handle_t,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);
    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_Connect(
    hBinding: handle_t,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);
    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetVersion(
    hBinding: handle_t,
    pVersion: *mut WORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    *pVersion = (CONFIGMG_VERSION) as u16;

    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetGlobalState(
    hBinding: handle_t,
    pulState: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    *pulState = (CM_GLOBAL_STATE_CAN_DO_UI | CM_GLOBAL_STATE_SERVICES_AVAILABLE) as u32;

    if g_ShuttingDown != 0 {
        *pulState |= (CM_GLOBAL_STATE_SHUTTING_DOWN) as u32;
    }

    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_InitDetection(
    hBinding: handle_t,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_ReportLogOn(
    hBinding: handle_t,
    Admin: BOOL,
    ProcessId: DWORD,
) -> DWORD {
    let mut ReturnValue: DWORD = (CR_FAILURE) as u32;
    let mut hProcess: HANDLE = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(Admin);

    // DPRINT removed

    'cleanup: loop {
        if (g_IsLiveMedium == (FALSE) as i32) && (IsCallerInteractive(hBinding) == (FALSE) as i32) {
            break 'cleanup;
        }

        hProcess = OpenProcess(PROCESS_ALL_ACCESS, (TRUE) as i32, ProcessId);

        if hProcess.is_null() {
            // DPRINT removed
            break 'cleanup;
        }

        if !hUserToken.is_null() {
            CloseHandle(hUserToken);
            hUserToken = core::ptr::null_mut();
        }

        if OpenProcessToken(
            hProcess,
            TOKEN_ASSIGN_PRIMARY | TOKEN_DUPLICATE | TOKEN_QUERY,
            core::ptr::addr_of_mut!(hUserToken),
        ) == 0
        {
            // DPRINT removed
            break 'cleanup;
        }

        if !hInstallEvent.is_null() {
            SetEvent(hInstallEvent);
        }

        ReturnValue = (CR_SUCCESS) as u32;

        break 'cleanup;
    }

    if !hProcess.is_null() {
        CloseHandle(hProcess);
    }

    ReturnValue
}

#[no_mangle]
pub unsafe extern "system" fn PNP_ValidateDeviceInstance(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut hDeviceKey: HKEY = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if RegOpenKeyExW(
        hEnumKey,
        pDeviceID,
        0,
        KEY_READ,
        &mut hDeviceKey,
    ) != 0
    {
        // DPRINT removed
        ret = (CR_NO_SUCH_DEVNODE) as u32;
    }

    if !hDeviceKey.is_null() {
        RegCloseKey(hDeviceKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetRootDeviceInstance(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    mut ulLength: PNP_RPC_STRING_LEN,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'Done: {
        if pDeviceID.is_null() {
            ret = (CR_INVALID_POINTER) as u32;
            break 'Done;
        }

        if ulLength < (lstrlenW(szRootDeviceInstanceID.as_ptr()) + 1) as u32 {
            ret = (CR_BUFFER_SMALL) as u32;
            break 'Done;
        }

        lstrcpyW(pDeviceID, szRootDeviceInstanceID.as_ptr());
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetRelatedDeviceInstance(
    hBinding: handle_t,
    ulRelationship: DWORD,
    pDeviceID: LPWSTR,
    pRelatedDeviceId: LPWSTR,
    pulLength: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_RELATED_DEVICE_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if ulRelationship == (PNP_GET_PARENT_DEVICE_INSTANCE) as u32 {
        if IsRootDeviceInstanceID(pDeviceID) != 0 {
            return (CR_NO_SUCH_DEVINST) as u32;
        }

        if IsPresentDeviceInstanceID(pDeviceID) == 0 {
            if (wcslen(szRootDeviceInstanceID.as_ptr() as LPWSTR) + 1) > (*pulLength) as usize {
                *pulLength = (wcslen(szRootDeviceInstanceID.as_ptr() as LPWSTR) + 1) as u32;
                return (CR_BUFFER_SMALL) as u32;
            }

            wcscpy(pRelatedDeviceId, szRootDeviceInstanceID.as_ptr() as LPWSTR);
            *pulLength = (wcslen(szRootDeviceInstanceID.as_ptr() as LPWSTR) + 1) as u32;
            return (CR_SUCCESS) as u32;
        }
    } else if ulRelationship == (PNP_GET_SIBLING_DEVICE_INSTANCE) as u32 {
        if IsRootDeviceInstanceID(pDeviceID) != 0 {
            return (CR_NO_SUCH_DEVINST) as u32;
        }
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.TargetDeviceInstance),
        pDeviceID,
    );

    PlugPlayData.Relation = ulRelationship;

    PlugPlayData.RelatedDeviceInstanceLength = *pulLength;
    PlugPlayData.RelatedDeviceInstance = pRelatedDeviceId;

    Status = NtPlugPlayControl(
        PlugPlayControlGetRelatedDevice,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_RELATED_DEVICE_DATA>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    // DPRINT removed
    if ret == (CR_SUCCESS) as u32 {
        // DPRINT removed
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_EnumerateSubKeys(
    hBinding: handle_t,
    ulBranch: DWORD,
    ulIndex: DWORD,
    Buffer: LPWSTR,
    mut ulLength: PNP_RPC_STRING_LEN,
    pulRequiredLen: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let hKey: HKEY;
    let mut dwError: DWORD;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    match ulBranch {
        PNP_ENUMERATOR_SUBKEYS => {
            hKey = hEnumKey;
        }
        PNP_CLASS_SUBKEYS => {
            hKey = hClassKey;
        }
        _ => {
            return (CR_FAILURE) as u32;
        }
    }

    *pulRequiredLen = ulLength;
    dwError = (RegEnumKeyExW(
        hKey,
        ulIndex,
        Buffer,
        pulRequiredLen,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        ret = if dwError == ERROR_NO_MORE_ITEMS {
            (CR_NO_SUCH_VALUE) as u32
        } else {
            (CR_FAILURE) as u32
        };
    } else {
        (*pulRequiredLen) = (*pulRequiredLen).wrapping_add(1);
    }

    // DPRINT removed

    ret
}
unsafe extern "C" fn GetRelationsInstanceList(
    pszDevice: PWSTR,
    ulFlags: DWORD,
    pszBuffer: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut PlugPlayData: PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    RtlInitUnicodeString(
        &mut PlugPlayData.DeviceInstance,
        pszDevice,
    );

    if ulFlags & (CM_GETIDLIST_FILTER_BUSRELATIONS) as u32 != 0 {
        PlugPlayData.Relations = 3;
    } else if ulFlags & (CM_GETIDLIST_FILTER_POWERRELATIONS) as u32 != 0 {
        PlugPlayData.Relations = 2;
    } else if ulFlags & (CM_GETIDLIST_FILTER_REMOVALRELATIONS) as u32 != 0 {
        PlugPlayData.Relations = 1;
    } else if ulFlags & (CM_GETIDLIST_FILTER_EJECTRELATIONS) as u32 != 0 {
        PlugPlayData.Relations = 0;
    }

    PlugPlayData.BufferSize = (*pulLength) * core::mem::size_of::<WCHAR>() as u32;
    PlugPlayData.Buffer = pszBuffer;

    Status = NtPlugPlayControl(
        PlugPlayControlQueryDeviceRelations,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA>() as u32,
    );
    if NT_SUCCESS(Status) {
        *pulLength = PlugPlayData.BufferSize / core::mem::size_of::<WCHAR>() as u32;
    } else {
        ret = NtStatusToCrError(Status);
    }

    ret
}
// The C declares a LOCAL of the same name as a module global here and relies on C's
// shadowing; Rust forbids a let binding shadowing a static. Renamed the LOCAL only, so
// every use in this function still means the local, exactly as the C does.
unsafe extern "C" fn GetServiceInstanceList(
    pszService: PWSTR,
    pszBuffer: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut szPathBuffer: [u16; 512] = [0; 512];
    let mut szName: [u16; 16] = [0; 16];
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut hEnumKeyLocal: HKEY = core::ptr::null_mut();
    let mut dwValues: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwIndex: DWORD;
    let mut dwUsedLength: DWORD = 0;
    let mut dwPathLength: DWORD;
    let mut dwError: DWORD = 0;
    let mut pPtr: PWSTR;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services") as LPCWSTR,
        0,
        KEY_READ,
        &mut hServicesKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwError = (RegOpenKeyExW(
        hServicesKey,
        pszService,
        0,
        KEY_READ,
        &mut hServiceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        ret = (CR_REGISTRY_ERROR) as u32;
        RegCloseKey(hEnumKeyLocal);
        RegCloseKey(hServiceKey);
        RegCloseKey(hServicesKey);
        if ret == (CR_SUCCESS) as u32 {
            *pulLength = dwUsedLength + 1;
        } else {
            *pulLength = 0;
        }
        return ret;
    }

    dwError = (RegOpenKeyExW(
        hServiceKey,
        w!("Enum") as LPCWSTR,
        0,
        KEY_READ,
        &mut hEnumKeyLocal,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        ret = (CR_REGISTRY_ERROR) as u32;
        RegCloseKey(hEnumKeyLocal);
        RegCloseKey(hServiceKey);
        RegCloseKey(hServicesKey);
        if ret == (CR_SUCCESS) as u32 {
            *pulLength = dwUsedLength + 1;
        } else {
            *pulLength = 0;
        }
        return ret;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = (RegQueryValueExW(
        hEnumKeyLocal,
        w!("Count") as LPCWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (&mut dwValues as *mut DWORD) as LPBYTE,
        &mut dwSize,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        dwValues = 1;
    }

    // DPRINT removed

    dwUsedLength = 0;
    pPtr = pszBuffer;

    dwIndex = 0;
    while dwIndex < dwValues {
        wsprintf(szName.as_mut_ptr(), w!("%lu") as LPCWSTR, dwIndex);

        dwSize = core::mem::size_of_val(&szPathBuffer) as DWORD;
        dwError = (RegQueryValueExW(
            hEnumKeyLocal,
            szName.as_ptr(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            szPathBuffer.as_mut_ptr() as LPBYTE,
            &mut dwSize,
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        // DPRINT removed

        dwPathLength = (wcslen(szPathBuffer.as_ptr()) + 1) as u32;
        if dwUsedLength + dwPathLength + 1 > *pulLength {
            ret = (CR_BUFFER_SMALL) as u32;
            break;
        }

        wcscpy(pPtr, szPathBuffer.as_ptr());
        dwUsedLength += dwPathLength;
        pPtr = pPtr.add(dwPathLength as usize);

        *pPtr = UNICODE_NULL;

        dwIndex += 1;
    }

    if !hEnumKeyLocal.is_null() {
        RegCloseKey(hEnumKeyLocal);
    }

    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }

    if !hServicesKey.is_null() {
        RegCloseKey(hServicesKey);
    }

    if ret == (CR_SUCCESS) as u32 {
        *pulLength = dwUsedLength + 1;
    } else {
        *pulLength = 0;
    }

    ret
}
unsafe extern "C" fn GetDeviceInstanceList(
    pszDevice: PWSTR,
    pszBuffer: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut szInstanceBuffer: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szPathBuffer: [u16; 512] = [0; 512];
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut dwInstanceLength: DWORD;
    let mut dwPathLength: DWORD;
    let mut dwUsedLength: DWORD = 0;
    let mut dwIndex: DWORD;
    let mut dwError: DWORD = 0;
    let mut pPtr: PWSTR;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwError = (RegOpenKeyExW(
        hEnumKey,
        pszDevice,
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hDeviceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwUsedLength = 0;
    pPtr = pszBuffer;

    dwIndex = 0;
    loop {
        dwInstanceLength = (MAX_DEVICE_ID_LEN) as u32;
        dwError = (RegEnumKeyExW(
            hDeviceKey,
            dwIndex,
            szInstanceBuffer.as_mut_ptr(),
            &mut dwInstanceLength,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        wsprintf(
            szPathBuffer.as_mut_ptr(),
            w!("%s\\%s") as LPCWSTR,
            pszDevice,
            szInstanceBuffer.as_ptr(),
        );
        // DPRINT removed

        dwPathLength = (wcslen(szPathBuffer.as_ptr()) + 1) as u32;
        if dwUsedLength + dwPathLength + 1 > *pulLength {
            ret = (CR_BUFFER_SMALL) as u32;
            break;
        }

        wcscpy(pPtr, szPathBuffer.as_ptr());
        dwUsedLength += dwPathLength;
        pPtr = pPtr.add(dwPathLength as usize);

        *pPtr = UNICODE_NULL;

        dwIndex += 1;
    }

    RegCloseKey(hDeviceKey);

    if ret == (CR_SUCCESS) as u32 {
        *pulLength = dwUsedLength + 1;
    } else {
        *pulLength = 0;
    }

    ret
}
#[no_mangle]
pub unsafe extern "C" fn GetEnumeratorInstanceList(
    pszEnumerator: PWSTR,
    pszBuffer: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut szDeviceBuffer: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szPathBuffer: [u16; 512] = [0; 512];
    let mut hEnumeratorKey: HKEY = core::ptr::null_mut();
    let mut pPtr: PWSTR;
    let mut dwIndex: u32;
    let mut dwDeviceLength: u32;
    let mut dwUsedLength: u32 = 0;
    let mut dwRemainingLength: u32;
    let mut dwPathLength: u32;
    let mut dwError: u32 = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwError = (RegOpenKeyExW(
        hEnumKey,
        pszEnumerator,
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hEnumeratorKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwUsedLength = 0;
    dwRemainingLength = *pulLength;
    pPtr = pszBuffer;

    dwIndex = 0;
    loop {
        dwDeviceLength = (MAX_DEVICE_ID_LEN) as u32;
        dwError = (RegEnumKeyExW(
            hEnumeratorKey,
            dwIndex,
            szDeviceBuffer.as_mut_ptr(),
            &mut dwDeviceLength,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        wsprintf(
            szPathBuffer.as_mut_ptr(),
            w!("%s\\%s") as PCWSTR,
            pszEnumerator,
            szDeviceBuffer.as_ptr(),
        );
        // DPRINT removed

        dwPathLength = dwRemainingLength;
        ret = GetDeviceInstanceList(szPathBuffer.as_mut_ptr(), pPtr, &mut dwPathLength);
        if ret != (CR_SUCCESS) as u32 {
            break;
        }

        dwUsedLength += dwPathLength - 1;
        dwRemainingLength -= dwPathLength - 1;
        pPtr = pPtr.offset((dwPathLength - 1) as isize);

        dwIndex += 1;
    }

    RegCloseKey(hEnumeratorKey);

    if ret == (CR_SUCCESS) as u32 {
        *pulLength = dwUsedLength + 1;
    } else {
        *pulLength = 0;
    }

    ret
}
unsafe extern "C" fn GetAllInstanceList(
    pszBuffer: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut szEnumeratorBuffer: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut pPtr: PWSTR;
    let mut dwIndex: DWORD;
    let mut dwEnumeratorLength: DWORD;
    let mut dwUsedLength: DWORD = 0;
    let mut dwRemainingLength: DWORD;
    let mut dwPathLength: DWORD;
    let mut dwError: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwUsedLength = 0;
    dwRemainingLength = *pulLength;
    pPtr = pszBuffer;

    dwIndex = 0;
    loop {
        dwEnumeratorLength = (MAX_DEVICE_ID_LEN) as u32;
        dwError = (RegEnumKeyExW(
            hEnumKey,
            dwIndex,
            szEnumeratorBuffer.as_mut_ptr(),
            core::ptr::addr_of_mut!(dwEnumeratorLength),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        dwPathLength = dwRemainingLength;
        ret = GetEnumeratorInstanceList(
            szEnumeratorBuffer.as_mut_ptr(),
            pPtr,
            core::ptr::addr_of_mut!(dwPathLength),
        );
        if ret != (CR_SUCCESS) as u32 {
            break;
        }

        dwUsedLength += dwPathLength - 1;
        dwRemainingLength -= dwPathLength - 1;
        pPtr = pPtr.add((dwPathLength - 1) as usize);

        dwIndex += 1;
    }

    if ret == (CR_SUCCESS) as u32 {
        *pulLength = dwUsedLength + 1;
    } else {
        *pulLength = 0;
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetDeviceList(
    hBinding: handle_t,
    pszFilter: LPWSTR,
    Buffer: LPWSTR,
    pulLength: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut szEnumerator: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szDevice: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szInstance: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if (ulFlags & (!CM_GETIDLIST_FILTER_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if pulLength.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if (ulFlags != (CM_GETIDLIST_FILTER_NONE) as u32) && pszFilter.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if (ulFlags & ((CM_GETIDLIST_FILTER_BUSRELATIONS | CM_GETIDLIST_FILTER_POWERRELATIONS | CM_GETIDLIST_FILTER_REMOVALRELATIONS | CM_GETIDLIST_FILTER_EJECTRELATIONS)) as u32) != 0 {
        ret = GetRelationsInstanceList(pszFilter, ulFlags, Buffer, pulLength);
    } else if (ulFlags & (CM_GETIDLIST_FILTER_SERVICE) as u32) != 0 {
        ret = GetServiceInstanceList(pszFilter, Buffer, pulLength);
    } else if (ulFlags & (CM_GETIDLIST_FILTER_ENUMERATOR) as u32) != 0 {
        SplitDeviceInstanceID(
            pszFilter,
            szEnumerator.as_mut_ptr(),
            szDevice.as_mut_ptr(),
            szInstance.as_mut_ptr(),
        );

        if szEnumerator[0] != UNICODE_NULL && szDevice[0] != UNICODE_NULL {
            ret = GetDeviceInstanceList(pszFilter, Buffer, pulLength);
        } else {
            ret = GetEnumeratorInstanceList(pszFilter, Buffer, pulLength);
        }
    } else {
        ret = GetAllInstanceList(Buffer, pulLength);
    }

    ret
}
unsafe extern "C" fn GetRelationsInstanceListSize(
    pszDevice: PWSTR,
    ulFlags: DWORD,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut PlugPlayData: PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    RtlInitUnicodeString(
        &mut PlugPlayData.DeviceInstance,
        pszDevice,
    );

    if (ulFlags & (CM_GETIDLIST_FILTER_BUSRELATIONS) as u32) != 0 {
        PlugPlayData.Relations = 3;
    } else if (ulFlags & (CM_GETIDLIST_FILTER_POWERRELATIONS) as u32) != 0 {
        PlugPlayData.Relations = 2;
    } else if (ulFlags & (CM_GETIDLIST_FILTER_REMOVALRELATIONS) as u32) != 0 {
        PlugPlayData.Relations = 1;
    } else if (ulFlags & (CM_GETIDLIST_FILTER_EJECTRELATIONS) as u32) != 0 {
        PlugPlayData.Relations = 0;
    }

    PlugPlayData.BufferSize = 0;
    PlugPlayData.Buffer = core::ptr::null_mut();

    Status = NtPlugPlayControl(
        PlugPlayControlQueryDeviceRelations,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA>() as u32,
    );
    if NT_SUCCESS(Status) {
        *pulLength = PlugPlayData.BufferSize / core::mem::size_of::<WCHAR>() as u32;
    } else {
        ret = NtStatusToCrError(Status);
    }

    ret
}
// Local shadowing the module global, as the C does (rpcserver.c:1111). Renamed the LOCAL.
unsafe extern "C" fn GetServiceInstanceListSize(
    pszService: PWSTR,
    pulLength: PDWORD,
) -> CONFIGRET {
    let mut hServicesKey: HKEY = core::ptr::null_mut();
    let mut hServiceKey: HKEY = core::ptr::null_mut();
    let mut hEnumKeyLocal: HKEY = core::ptr::null_mut();
    let mut dwValues: DWORD = 0;
    let mut dwMaxValueLength: DWORD = 0;
    let mut dwSize: DWORD;
    let mut dwError: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        w!("System\\CurrentControlSet\\Services") as *const u16,
        0,
        KEY_READ,
        &mut hServicesKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwError = (RegOpenKeyExW(
        hServicesKey,
        pszService,
        0,
        KEY_READ,
        &mut hServiceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        ret = (CR_REGISTRY_ERROR) as u32;
        if !hEnumKeyLocal.is_null() {
            RegCloseKey(hEnumKeyLocal);
        }
        if !hServiceKey.is_null() {
            RegCloseKey(hServiceKey);
        }
        if !hServicesKey.is_null() {
            RegCloseKey(hServicesKey);
        }
        return ret;
    }

    dwError = (RegOpenKeyExW(
        hServiceKey,
        w!("Enum") as *const u16,
        0,
        KEY_READ,
        &mut hEnumKeyLocal,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        ret = (CR_REGISTRY_ERROR) as u32;
        if !hEnumKeyLocal.is_null() {
            RegCloseKey(hEnumKeyLocal);
        }
        if !hServiceKey.is_null() {
            RegCloseKey(hServiceKey);
        }
        if !hServicesKey.is_null() {
            RegCloseKey(hServicesKey);
        }
        return ret;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = (RegQueryValueExW(
        hEnumKeyLocal,
        w!("Count") as *const u16,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwValues as *mut DWORD as LPBYTE,
        &mut dwSize,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        dwValues = 1;
    }

    dwError = (RegQueryInfoKeyW(
        hEnumKeyLocal,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwMaxValueLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        dwMaxValueLength = (MAX_DEVICE_ID_LEN) as u32;
    }

    // DPRINT removed

    *pulLength = dwValues * dwMaxValueLength / core::mem::size_of::<u16>() as DWORD + 2;

    if !hEnumKeyLocal.is_null() {
        RegCloseKey(hEnumKeyLocal);
    }
    if !hServiceKey.is_null() {
        RegCloseKey(hServiceKey);
    }
    if !hServicesKey.is_null() {
        RegCloseKey(hServicesKey);
    }

    ret
}
unsafe extern "C" fn GetDeviceInstanceListSize(
    pszDevice: LPCWSTR,
    pulLength: PULONG,
) -> CONFIGRET {
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut dwSubKeys: DWORD = 0;
    let mut dwMaxSubKeyLength: DWORD = 0;
    let mut dwError: DWORD;

    dwError = (RegOpenKeyExW(
        hEnumKey,
        pszDevice,
        0,
        KEY_READ,
        &mut hDeviceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwError = (RegQueryInfoKeyW(
        hDeviceKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut dwSubKeys,
        &mut dwMaxSubKeyLength,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        dwSubKeys = 0;
        dwMaxSubKeyLength = 0;
    }

    RegCloseKey(hDeviceKey);

    *pulLength = dwSubKeys * ((wcslen(pszDevice) + 1 + (dwMaxSubKeyLength) as usize + 1)) as u32;

    return (CR_SUCCESS) as u32;
}
unsafe extern "C" fn GetEnumeratorInstanceListSize(
    pszEnumerator: LPCWSTR,
    pulLength: PULONG,
) -> CONFIGRET {
    let mut szDeviceBuffer: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szPathBuffer: [WCHAR; 512] = [0; 512];
    let mut hEnumeratorKey: HKEY = core::ptr::null_mut();
    let mut dwIndex: DWORD;
    let mut dwDeviceLength: DWORD;
    let mut dwBufferLength: DWORD = 0;
    let mut dwError: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    *pulLength = 0;

    dwError = (RegOpenKeyExW(
        hEnumKey,
        pszEnumerator,
        0,
        KEY_ENUMERATE_SUB_KEYS,
        &mut hEnumeratorKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        // DPRINT removed
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwIndex = 0;
    loop {
        dwDeviceLength = (MAX_DEVICE_ID_LEN) as u32;
        dwError = (RegEnumKeyExW(
            hEnumeratorKey,
            dwIndex,
            szDeviceBuffer.as_mut_ptr(),
            &mut dwDeviceLength,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        wsprintf(
            szPathBuffer.as_mut_ptr(),
            w!("%s\\%s") as LPCWSTR,
            pszEnumerator,
            szDeviceBuffer.as_ptr(),
        );
        // DPRINT removed

        ret = GetDeviceInstanceListSize(szPathBuffer.as_ptr(), &mut dwBufferLength);
        if ret != (CR_SUCCESS) as u32 {
            *pulLength = 0;
            break;
        }

        *pulLength += dwBufferLength;
        dwIndex += 1;
    }

    RegCloseKey(hEnumeratorKey);

    ret
}
unsafe extern "C" fn GetAllInstanceListSize(
    pulLength: PULONG,
) -> CONFIGRET {
    let mut szEnumeratorBuffer: [u16; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut dwIndex: u32;
    let mut dwEnumeratorLength: u32;
    let mut dwBufferLength: u32 = 0;
    let mut dwError: u32 = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    dwIndex = 0;
    loop {
        dwEnumeratorLength = (MAX_DEVICE_ID_LEN) as u32;
        dwError = (RegEnumKeyExW(
            hEnumKey,
            dwIndex,
            szEnumeratorBuffer.as_mut_ptr(),
            core::ptr::addr_of_mut!(dwEnumeratorLength),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            break;
        }

        ret = GetEnumeratorInstanceListSize(
            szEnumeratorBuffer.as_ptr(),
            core::ptr::addr_of_mut!(dwBufferLength),
        );
        if ret != (CR_SUCCESS) as u32 {
            break;
        }

        *pulLength = (*pulLength).wrapping_add(dwBufferLength);
        dwIndex = dwIndex.wrapping_add(1);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetDeviceListSize(
    hBinding: handle_t,
    pszFilter: LPWSTR,
    pulLength: *mut PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    let mut szEnumerator: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szDevice: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szInstance: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if (ulFlags & (!CM_GETIDLIST_FILTER_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if pulLength.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if (ulFlags != (CM_GETIDLIST_FILTER_NONE) as u32) &&
        pszFilter.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pulLength = 0;

    if (ulFlags &
        ((CM_GETIDLIST_FILTER_BUSRELATIONS |
         CM_GETIDLIST_FILTER_POWERRELATIONS |
         CM_GETIDLIST_FILTER_REMOVALRELATIONS |
         CM_GETIDLIST_FILTER_EJECTRELATIONS)) as u32) != 0 {
        ret = GetRelationsInstanceListSize(pszFilter,
                                      ulFlags,
                                      pulLength);
    } else if (ulFlags & (CM_GETIDLIST_FILTER_SERVICE) as u32) != 0 {
        ret = GetServiceInstanceListSize(pszFilter,
                                      pulLength);
    } else if (ulFlags & (CM_GETIDLIST_FILTER_ENUMERATOR) as u32) != 0 {
        SplitDeviceInstanceID(pszFilter,
                               szEnumerator.as_mut_ptr(),
                               szDevice.as_mut_ptr(),
                               szInstance.as_mut_ptr());

        if *szEnumerator.as_ptr().offset(0) != UNICODE_NULL && *szDevice.as_ptr().offset(0) != UNICODE_NULL {
            ret = GetDeviceInstanceListSize(pszFilter,
                                         pulLength);
        } else {
            ret = GetEnumeratorInstanceListSize(pszFilter,
                                         pulLength);
        }
    } else {
        ret = GetAllInstanceListSize(pulLength);
    }

    if ret == (CR_SUCCESS) as u32 {
        (*pulLength) = (*pulLength).wrapping_add(1);
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetDepth(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pulDepth: LPDWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_DEPTH_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );

    Status = NtPlugPlayControl(
        PlugPlayControlGetDeviceDepth,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_DEPTH_DATA>() as ULONG,
    );
    
    if NT_SUCCESS(Status) {
        *pulDepth = PlugPlayData.Depth;
    } else {
        ret = NtStatusToCrError(Status);
    }

    // DPRINT removed

    ret
}
#[no_mangle]
pub unsafe extern "system" fn PNP_GetDeviceRegProp(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulProperty: DWORD,
    pulRegDataType: *mut DWORD,
    Buffer: *mut BYTE,
    pulTransferLen: *mut PNP_PROP_SIZE,
    pulLength: *mut PNP_PROP_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_PROPERTY_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut lpValueName: LPWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;
    let mut Status: NTSTATUS;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'done: loop {
        if pulTransferLen.is_null() || pulLength.is_null() {
            ret = (CR_INVALID_POINTER) as u32;
            break 'done;
        }

        if ulFlags != 0 {
            ret = (CR_INVALID_FLAG) as u32;
            break 'done;
        }

        if IsValidDeviceInstanceID(pDeviceID) == 0 {
            ret = (CR_INVALID_DEVINST) as u32;
            break 'done;
        }

        if (*pulLength) < (*pulTransferLen) {
            *pulLength = *pulTransferLen;
        }

        *pulTransferLen = 0;

        match ulProperty {
            CM_DRP_DEVICEDESC => {
                lpValueName = w!("DeviceDesc") as LPWSTR;
            }
            CM_DRP_HARDWAREID => {
                lpValueName = w!("HardwareID") as LPWSTR;
            }
            CM_DRP_COMPATIBLEIDS => {
                lpValueName = w!("CompatibleIDs") as LPWSTR;
            }
            CM_DRP_SERVICE => {
                lpValueName = w!("Service") as LPWSTR;
            }
            CM_DRP_CLASS => {
                lpValueName = w!("Class") as LPWSTR;
            }
            CM_DRP_CLASSGUID => {
                lpValueName = w!("ClassGUID") as LPWSTR;
            }
            CM_DRP_DRIVER => {
                lpValueName = w!("Driver") as LPWSTR;
            }
            CM_DRP_CONFIGFLAGS => {
                lpValueName = w!("ConfigFlags") as LPWSTR;
            }
            CM_DRP_MFG => {
                lpValueName = w!("Mfg") as LPWSTR;
            }
            CM_DRP_FRIENDLYNAME => {
                lpValueName = w!("FriendlyName") as LPWSTR;
            }
            CM_DRP_LOCATION_INFORMATION => {
                lpValueName = w!("LocationInformation") as LPWSTR;
            }
            CM_DRP_PHYSICAL_DEVICE_OBJECT_NAME => {
                PlugPlayData.Property = PNP_PROPERTY_PHYSICAL_DEVICE_OBJECT_NAME;
                *pulRegDataType = (REG_SZ) as u32;
            }
            CM_DRP_CAPABILITIES => {
                lpValueName = w!("Capabilities") as LPWSTR;
            }
            CM_DRP_UI_NUMBER => {
                PlugPlayData.Property = PNP_PROPERTY_UI_NUMBER;
            }
            CM_DRP_UPPERFILTERS => {
                lpValueName = w!("UpperFilters") as LPWSTR;
            }
            CM_DRP_LOWERFILTERS => {
                lpValueName = w!("LowerFilters") as LPWSTR;
            }
            CM_DRP_BUSTYPEGUID => {
                PlugPlayData.Property = PNP_PROPERTY_BUSTYPEGUID;
                *pulRegDataType = (REG_BINARY) as u32;
            }
            CM_DRP_LEGACYBUSTYPE => {
                PlugPlayData.Property = PNP_PROPERTY_LEGACYBUSTYPE;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_BUSNUMBER => {
                PlugPlayData.Property = PNP_PROPERTY_BUSNUMBER;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_ENUMERATOR_NAME => {
                PlugPlayData.Property = PNP_PROPERTY_ENUMERATOR_NAME;
                *pulRegDataType = (REG_SZ) as u32;
            }
            CM_DRP_SECURITY => {
                lpValueName = w!("Security") as LPWSTR;
            }
            CM_DRP_DEVTYPE => {
                lpValueName = w!("DeviceType") as LPWSTR;
            }
            CM_DRP_EXCLUSIVE => {
                lpValueName = w!("Exclusive") as LPWSTR;
            }
            CM_DRP_CHARACTERISTICS => {
                lpValueName = w!("DeviceCharacteristics") as LPWSTR;
            }
            CM_DRP_ADDRESS => {
                PlugPlayData.Property = PNP_PROPERTY_ADDRESS;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_UI_NUMBER_DESC_FORMAT => {
                lpValueName = w!("UINumberDescFormat") as LPWSTR;
            }
            CM_DRP_DEVICE_POWER_DATA => {
                PlugPlayData.Property = PNP_PROPERTY_POWER_DATA;
                *pulRegDataType = (REG_BINARY) as u32;
            }
            CM_DRP_REMOVAL_POLICY => {
                PlugPlayData.Property = PNP_PROPERTY_REMOVAL_POLICY;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_REMOVAL_POLICY_HW_DEFAULT => {
                PlugPlayData.Property = PNP_PROPERTY_REMOVAL_POLICY_HARDWARE_DEFAULT;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_REMOVAL_POLICY_OVERRIDE => {
                lpValueName = w!("RemovalPolicy") as LPWSTR;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_INSTALL_STATE => {
                PlugPlayData.Property = PNP_PROPERTY_INSTALL_STATE;
                *pulRegDataType = (REG_DWORD) as u32;
            }
            CM_DRP_LOCATION_PATHS => {
                PlugPlayData.Property = PNP_PROPERTY_LOCATION_PATHS;
                *pulRegDataType = (REG_MULTI_SZ) as u32;
            }
            // rpcserver.c:2094 guards this case with `#if (WINVER >= _WIN32_WINNT_WIN7)`, and
            // umpnpmgr is compiled at WINVER=0x502 — so the C does NOT compile this arm, and
            // cfgmgr32.h does not even define CM_DRP_BASE_CONTAINERID at that level. The
            // translation flattened the guard away; left in, the unresolved name would have been
            // a fresh BINDING that matched every remaining property id and made the `_` arm
            // below unreachable (§9.112). Restored to what the build actually compiles.
            _ => {
                ret = (CR_INVALID_PROPERTY) as u32;
                break 'done;
            }
        }

        // DPRINT removed

        if !lpValueName.is_null() {
            lError = RegOpenKeyExW(
                hEnumKey,
                pDeviceID,
                0,
                KEY_QUERY_VALUE,
                &mut hKey,
            );
            if lError != (ERROR_SUCCESS) as i32 {
                hKey = core::ptr::null_mut();
                *pulLength = 0;
                ret = (CR_INVALID_DEVNODE) as u32;
                break 'done;
            }

            lError = RegQueryValueExW(
                hKey,
                lpValueName,
                core::ptr::null_mut(),
                pulRegDataType,
                Buffer,
                pulLength,
            );
            if lError != (ERROR_SUCCESS) as i32 {
                if lError == (ERROR_MORE_DATA) as i32 {
                    ret = (CR_BUFFER_SMALL) as u32;
                } else {
                    *pulLength = 0;
                    ret = (CR_NO_SUCH_VALUE) as u32;
                }
            }
        } else {
            RtlInitUnicodeString(
                &mut PlugPlayData.DeviceInstance,
                pDeviceID,
            );
            PlugPlayData.Buffer = Buffer as PVOID;
            PlugPlayData.BufferSize = *pulLength;

            Status = NtPlugPlayControl(
                PlugPlayControlProperty,
                core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
                core::mem::size_of::<PLUGPLAY_CONTROL_PROPERTY_DATA>() as ULONG,
            );
            if NT_SUCCESS(Status) {
                *pulLength = PlugPlayData.BufferSize;
            } else {
                ret = NtStatusToCrError(Status);
            }
        }

        break 'done;
    }

    if !pulTransferLen.is_null() {
        *pulTransferLen = if ret == (CR_SUCCESS) as u32 {
            *pulLength
        } else {
            0
        };
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetDeviceRegProp(
    hBinding: handle_t,
    pDeviceId: LPWSTR,
    ulProperty: DWORD,
    ulDataType: DWORD,
    Buffer: *mut BYTE,
    ulLength: PNP_PROP_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut lpValueName: LPWSTR = core::ptr::null_mut();
    let mut hKey: HKEY = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    if IsValidDeviceInstanceID(pDeviceId) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    match ulProperty {
        CM_DRP_DEVICEDESC => {
            lpValueName = w!("DeviceDesc") as LPWSTR;
        }
        CM_DRP_HARDWAREID => {
            lpValueName = w!("HardwareID") as LPWSTR;
        }
        CM_DRP_COMPATIBLEIDS => {
            lpValueName = w!("CompatibleIDs") as LPWSTR;
        }
        CM_DRP_SERVICE => {
            lpValueName = w!("Service") as LPWSTR;
        }
        CM_DRP_CLASS => {
            lpValueName = w!("Class") as LPWSTR;
        }
        CM_DRP_CLASSGUID => {
            lpValueName = w!("ClassGUID") as LPWSTR;
        }
        CM_DRP_DRIVER => {
            lpValueName = w!("Driver") as LPWSTR;
        }
        CM_DRP_CONFIGFLAGS => {
            lpValueName = w!("ConfigFlags") as LPWSTR;
        }
        CM_DRP_MFG => {
            lpValueName = w!("Mfg") as LPWSTR;
        }
        CM_DRP_FRIENDLYNAME => {
            lpValueName = w!("FriendlyName") as LPWSTR;
        }
        CM_DRP_LOCATION_INFORMATION => {
            lpValueName = w!("LocationInformation") as LPWSTR;
        }
        CM_DRP_UPPERFILTERS => {
            lpValueName = w!("UpperFilters") as LPWSTR;
        }
        CM_DRP_LOWERFILTERS => {
            lpValueName = w!("LowerFilters") as LPWSTR;
        }
        CM_DRP_SECURITY => {
            lpValueName = w!("Security") as LPWSTR;
        }
        CM_DRP_DEVTYPE => {
            lpValueName = w!("DeviceType") as LPWSTR;
        }
        CM_DRP_EXCLUSIVE => {
            lpValueName = w!("Exclusive") as LPWSTR;
        }
        CM_DRP_CHARACTERISTICS => {
            lpValueName = w!("DeviceCharacteristics") as LPWSTR;
        }
        CM_DRP_UI_NUMBER_DESC_FORMAT => {
            lpValueName = w!("UINumberDescFormat") as LPWSTR;
        }
        CM_DRP_REMOVAL_POLICY_OVERRIDE => {
            lpValueName = w!("RemovalPolicy") as LPWSTR;
        }
        _ => {
            return (CR_INVALID_PROPERTY) as u32;
        }
    }

    // DPRINT removed

    if RegOpenKeyExW(
        hEnumKey,
        pDeviceId,
        0,
        KEY_SET_VALUE,
        &mut hKey,
    ) != 0
    {
        return (CR_INVALID_DEVNODE) as u32;
    }

    if ulLength == 0 {
        if RegDeleteValueW(hKey, lpValueName) != 0 {
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    } else {
        if RegSetValueExW(
            hKey,
            lpValueName,
            0,
            ulDataType,
            Buffer,
            ulLength,
        ) != 0
        {
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    }

    RegCloseKey(hKey);

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetClassInstance(
    hBinding: handle_t,
    pDeviceId: LPWSTR,
    pszClassInstance: LPWSTR,
    mut ulLength: PNP_RPC_STRING_LEN,
) -> DWORD {
    let mut szClassGuid: [WCHAR; 40] = [0; 40];
    let mut szClassInstance: [WCHAR; 5] = [0; 5];
    let mut hDeviceClassKey: HKEY = core::ptr::null_mut();
    let mut hClassInstanceKey: HKEY = core::ptr::null_mut();
    let mut ulTransferLength: ULONG;
    let mut ulDataLength: ULONG;
    let mut dwDataType: DWORD = 0;
    let mut dwDisposition: DWORD = 0;
    let mut i: DWORD;
    let mut dwError: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if IsValidDeviceInstanceID(pDeviceId) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ulTransferLength = ulLength;
    ret = PNP_GetDeviceRegProp(
        hBinding,
        pDeviceId,
        (CM_DRP_DRIVER) as u32,
        &mut dwDataType,
        pszClassInstance as *mut BYTE,
        &mut ulTransferLength,
        &mut ulLength,
        0,
    );
    if ret == (CR_SUCCESS) as u32 {
        return ret;
    }

    ulTransferLength = core::mem::size_of_val(&szClassGuid) as ULONG;
    ulDataLength = core::mem::size_of_val(&szClassGuid) as ULONG;
    ret = PNP_GetDeviceRegProp(
        hBinding,
        pDeviceId,
        (CM_DRP_CLASSGUID) as u32,
        &mut dwDataType,
        szClassGuid.as_mut_ptr() as *mut BYTE,
        &mut ulTransferLength,
        &mut ulDataLength,
        0,
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = 'done: {
            break 'done ret;
        };
    } else {
        dwError = (RegOpenKeyExW(
            hClassKey,
            szClassGuid.as_mut_ptr(),
            0,
            KEY_READ,
            &mut hDeviceClassKey,
        )) as u32;
        if dwError != ERROR_SUCCESS {
            // DPRINT removed
            ret = (CR_FAILURE) as u32;
            ret = 'done: {
                break 'done ret;
            };
        } else {
            i = 0;
            'for_loop: loop {
                if i >= 10000 {
                    break 'for_loop;
                }

                wsprintf(
                    szClassInstance.as_mut_ptr(),
                    w!("%04lu") as LPCWSTR,
                    i,
                );

                dwError = (RegCreateKeyExW(
                    hDeviceClassKey,
                    szClassInstance.as_mut_ptr(),
                    0,
                    core::ptr::null_mut(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_ALL_ACCESS,
                    core::ptr::null_mut(),
                    &mut hClassInstanceKey,
                    &mut dwDisposition,
                )) as u32;
                if dwError == ERROR_SUCCESS {
                    RegCloseKey(hClassInstanceKey);

                    if dwDisposition == (REG_CREATED_NEW_KEY) as u32 {
                        wsprintf(
                            pszClassInstance,
                            w!("%s\\%s") as LPCWSTR,
                            szClassGuid.as_mut_ptr(),
                            szClassInstance.as_mut_ptr(),
                        );

                        ulDataLength = ((wcslen(pszClassInstance) + 1)
                            * (core::mem::size_of::<WCHAR>() as ULONG) as usize) as u32;
                        ret = PNP_SetDeviceRegProp(
                            hBinding,
                            pDeviceId,
                            (CM_DRP_DRIVER) as u32,
                            (REG_SZ) as u32,
                            pszClassInstance as *mut BYTE,
                            ulDataLength,
                            0,
                        );
                        if ret != (CR_SUCCESS) as u32 {
                            // DPRINT removed
                            RegDeleteKeyW(hDeviceClassKey, szClassInstance.as_mut_ptr());
                        }

                        break 'for_loop;
                    }
                }

                i += 1;
            }

            ret = 'done: {
                break 'done ret;
            };
        }
    }

    if hDeviceClassKey != core::ptr::null_mut() {
        RegCloseKey(hDeviceClassKey);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_CreateKey(
    hBinding: handle_t,
    pszSubKey: LPWSTR,
    samDesired: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut hParametersKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(samDesired);

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pszSubKey) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    dwError = (RegOpenKeyExW(
        hEnumKey,
        pszSubKey,
        0,
        KEY_WRITE,
        &mut hDeviceKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        ret = (CR_INVALID_DEVNODE) as u32;
    } else {
        dwError = (RegCreateKeyExW(
            hDeviceKey,
            w!("Device Parameters") as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            &mut hParametersKey,
            core::ptr::null_mut(),
        )) as u32;
        if dwError != ERROR_SUCCESS {
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    }

    if !hParametersKey.is_null() {
        RegCloseKey(hParametersKey);
    }

    if !hDeviceKey.is_null() {
        RegCloseKey(hDeviceKey);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DeleteRegistryKey(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pszParentKey: LPWSTR,
    pszChildKey: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetClassCount(
    hBinding: handle_t,
    pulClassCount: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwError: DWORD = 0;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    dwError = (RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        REGSTR_PATH_CLASS!(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    )) as u32;
    if dwError != ERROR_SUCCESS {
        return (CR_INVALID_DATA) as u32;
    }

    dwError = (RegQueryInfoKeyW(
        hKey,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        pulClassCount,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )) as u32;
    RegCloseKey(hKey);
    if dwError != ERROR_SUCCESS {
        return (CR_INVALID_DATA) as u32;
    }

    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetClassName(
    hBinding: handle_t,
    pszClassGuid: LPWSTR,
    Buffer: LPWSTR,
    pulLength: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut szKeyName: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    lstrcpyW(szKeyName.as_mut_ptr(), w!("System\\CurrentControlSet\\Control\\Class\\") as LPWSTR);
    if lstrlenW(pszClassGuid) + 1 < (core::mem::size_of_val(&szKeyName) / core::mem::size_of::<WCHAR>()) as i32 - (lstrlenW(szKeyName.as_ptr()) as i32 * core::mem::size_of::<WCHAR>() as i32) {
        lstrcatW(szKeyName.as_mut_ptr(), pszClassGuid);
    } else {
        return (CR_INVALID_DATA) as u32;
    }

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyName.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != 0 {
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwSize = (*pulLength) * core::mem::size_of::<WCHAR>() as u32;
    if RegQueryValueExW(
        hKey,
        w!("Class") as LPWSTR,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        Buffer as LPBYTE,
        &mut dwSize,
    ) != 0 {
        *pulLength = 0;
        ret = (CR_REGISTRY_ERROR) as u32;
    } else {
        *pulLength = dwSize / core::mem::size_of::<WCHAR>() as u32;
    }

    RegCloseKey(hKey);

    // DPRINT removed

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DeleteClassKey(
    hBinding: handle_t,
    pszClassGuid: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if (ulFlags & (CM_DELETE_CLASS_SUBKEYS) as u32) != 0 {
        if SHDeleteKeyW(hClassKey, pszClassGuid) != ERROR_SUCCESS {
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    } else {
        if RegDeleteKeyW(hClassKey, pszClassGuid) != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    }

    // DPRINT removed

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetInterfaceDeviceAlias(
    hBinding: handle_t,
    pszInterfaceDevice: LPWSTR,
    AliasInterfaceGuid: *mut GUID,
    pszAliasInterfaceDevice: LPWSTR,
    pulLength: *mut PNP_RPC_STRING_LEN,
    pulTransferLen: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut PlugPlayData: PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA = core::mem::zeroed();
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if AliasInterfaceGuid.is_null() || pulLength.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.SymbolicLinkName),
        pszInterfaceDevice,
    );
    PlugPlayData.AliasInterfaceClassGuid = AliasInterfaceGuid;
    PlugPlayData.AliasSymbolicLinkName = pszAliasInterfaceDevice;
    PlugPlayData.AliasSymbolicLinkNameLength = *pulTransferLen;

    Status = NtPlugPlayControl(
        PlugPlayControlGetInterfaceDeviceAlias,
        core::ptr::addr_of_mut!(PlugPlayData) as *mut core::ffi::c_void,
        core::mem::size_of::<PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        *pulLength = PlugPlayData.AliasSymbolicLinkNameLength;
        *pulTransferLen = *pulLength + 1;
    } else {
        *pulLength = 0;
        *pulTransferLen = 0;
        ret = NtStatusToCrError(Status);
    }

    // DPRINT removed
    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetInterfaceDeviceList(
    hBinding: handle_t,
    InterfaceGuid: *mut GUID,
    pszDeviceID: LPWSTR,
    Buffer: *mut BYTE,
    pulLength: *mut PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut PlugPlayData: PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA = core::mem::zeroed();
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );

    PlugPlayData.Flags = ulFlags;
    PlugPlayData.FilterGuid = InterfaceGuid;
    PlugPlayData.Buffer = (Buffer).cast();
    PlugPlayData.BufferSize = *pulLength;

    Status = NtPlugPlayControl(
        PlugPlayControlGetInterfaceDeviceList,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        *pulLength = PlugPlayData.BufferSize;
    } else {
        ret = NtStatusToCrError(Status);
    }

    // DPRINT removed
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetInterfaceDeviceListSize(
    hBinding: handle_t,
    pulLen: *mut PNP_RPC_BUFFER_SIZE,
    InterfaceGuid: *mut GUID,
    pszDeviceID: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    let mut Status: NTSTATUS;
    let mut PlugPlayData: PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA = core::mem::zeroed();
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );

    PlugPlayData.FilterGuid = InterfaceGuid;
    PlugPlayData.Buffer = core::ptr::null_mut();
    PlugPlayData.BufferSize = 0;
    PlugPlayData.Flags = ulFlags;

    Status = NtPlugPlayControl(
        PlugPlayControlGetInterfaceDeviceList,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        *pulLen = PlugPlayData.BufferSize;
    } else {
        ret = NtStatusToCrError(Status);
    }

    // DPRINT removed
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RegisterDeviceClassAssociation(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    InterfaceGuid: *mut GUID,
    pszReference: LPWSTR,
    pszSymLink: LPWSTR,
    pulLength: *mut PNP_RPC_STRING_LEN,
    pulTransferLen: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if InterfaceGuid.is_null()
        || pszSymLink.is_null()
        || pulLength.is_null()
        || pulTransferLen.is_null()
    {
        return (CR_INVALID_POINTER) as u32;
    }

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );
    PlugPlayData.InterfaceGuid = InterfaceGuid;
    RtlInitUnicodeString(core::ptr::addr_of_mut!(PlugPlayData.Reference), pszReference);
    PlugPlayData.Register = (TRUE) as u8;
    PlugPlayData.SymbolicLinkName = pszSymLink;
    PlugPlayData.SymbolicLinkNameLength = *pulLength;

    Status = NtPlugPlayControl(
        PlugPlayControlDeviceClassAssociation,
        core::ptr::addr_of_mut!(PlugPlayData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA>() as ULONG,
    );
    if NT_SUCCESS(Status) {
        *pulLength = PlugPlayData.SymbolicLinkNameLength;
        *pulTransferLen = *pulLength;
    } else {
        *pulLength = 0;
        ret = NtStatusToCrError(Status);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_UnregisterDeviceClassAssociation(
    hBinding: handle_t,
    pszInterfaceDevice: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if pszInterfaceDevice.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    ZeroMemory!(
        &mut PlugPlayData as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of_val(&PlugPlayData),
    );
    PlugPlayData.Register = (FALSE) as u8;
    PlugPlayData.SymbolicLinkName = pszInterfaceDevice;
    PlugPlayData.SymbolicLinkNameLength = (wcslen(pszInterfaceDevice) + 1) as u32;

    Status = NtPlugPlayControl(
        PlugPlayControlDeviceClassAssociation,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA>() as u32,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn PNP_GetClassRegProp(
    hBinding: handle_t,
    pszClassGuid: LPWSTR,
    ulProperty: DWORD,
    pulRegDataType: *mut DWORD,
    Buffer: *mut BYTE,
    pulTransferLen: *mut PNP_RPC_STRING_LEN,
    pulLength: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut lpValueName: LPWSTR = core::ptr::null_mut();
    let mut hInstKey: HKEY = core::ptr::null_mut();
    let mut hPropKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'done: loop {
        if pulTransferLen.is_null() || pulLength.is_null() {
            ret = (CR_INVALID_POINTER) as u32;
            break 'done;
        }

        if ulFlags != 0 {
            ret = (CR_INVALID_FLAG) as u32;
            break 'done;
        }

        if *pulLength < *pulTransferLen {
            *pulLength = *pulTransferLen;
        }

        *pulTransferLen = 0;

        match ulProperty {
            CM_CRP_SECURITY => {
                lpValueName = w!("Security") as LPWSTR;
            }
            CM_CRP_DEVTYPE => {
                lpValueName = w!("DeviceType") as LPWSTR;
            }
            CM_CRP_EXCLUSIVE => {
                lpValueName = w!("Exclusive") as LPWSTR;
            }
            CM_CRP_CHARACTERISTICS => {
                lpValueName = w!("DeviceCharacteristics") as LPWSTR;
            }
            _ => {
                ret = (CR_INVALID_PROPERTY) as u32;
                break 'done;
            }
        }

        // DPRINT removed

        lError = RegOpenKeyExW(
            hClassKey,
            pszClassGuid,
            0,
            KEY_READ,
            &mut hInstKey,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            *pulLength = 0;
            ret = (CR_NO_SUCH_REGISTRY_KEY) as u32;
            break 'done;
        }

        lError = RegOpenKeyExW(
            hInstKey,
            w!("Properties") as LPCWSTR,
            0,
            KEY_READ,
            &mut hPropKey,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            *pulLength = 0;
            ret = (CR_NO_SUCH_REGISTRY_KEY) as u32;
            break 'done;
        }

        lError = RegQueryValueExW(
            hPropKey,
            lpValueName,
            core::ptr::null_mut(),
            pulRegDataType,
            Buffer,
            pulLength,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            if lError == (ERROR_MORE_DATA) as i32 {
                ret = (CR_BUFFER_SMALL) as u32;
            } else {
                *pulLength = 0;
                ret = (CR_NO_SUCH_VALUE) as u32;
            }
        }

        break;
    }

    if ret == (CR_SUCCESS) as u32 {
        *pulTransferLen = *pulLength;
    }

    if !hPropKey.is_null() {
        RegCloseKey(hPropKey);
    }

    if !hInstKey.is_null() {
        RegCloseKey(hInstKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetClassRegProp(
    hBinding: handle_t,
    pszClassGuid: LPWSTR,
    ulProperty: DWORD,
    ulDataType: DWORD,
    Buffer: *mut BYTE,
    ulLength: PNP_PROP_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut lpValueName: LPWSTR = core::ptr::null_mut();
    let mut hInstKey: HKEY = core::ptr::null_mut();
    let mut hPropKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    match ulProperty {
        CM_CRP_SECURITY => {
            lpValueName = w!("Security") as LPWSTR;
        }
        CM_CRP_DEVTYPE => {
            lpValueName = w!("DeviceType") as LPWSTR;
        }
        CM_CRP_EXCLUSIVE => {
            lpValueName = w!("Exclusive") as LPWSTR;
        }
        CM_CRP_CHARACTERISTICS => {
            lpValueName = w!("DeviceCharacteristics") as LPWSTR;
        }
        _ => {
            return (CR_INVALID_PROPERTY) as u32;
        }
    }

    lError = RegOpenKeyExW(
        hClassKey,
        pszClassGuid,
        0,
        KEY_WRITE,
        &mut hInstKey,
    );
    if lError != (ERROR_SUCCESS) as i32 {
        ret = (CR_NO_SUCH_REGISTRY_KEY) as u32;
        // goto done
    } else {
        lError = RegCreateKeyExW(
            hInstKey,
            w!("Properties") as LPWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ALL_ACCESS,
            core::ptr::null_mut(),
            &mut hPropKey,
            core::ptr::null_mut(),
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            // goto done
        } else {
            if ulLength == 0 {
                if RegDeleteValueW(hPropKey, lpValueName) != 0 {
                    ret = (CR_REGISTRY_ERROR) as u32;
                }
            } else {
                if RegSetValueExW(
                    hPropKey,
                    lpValueName,
                    0,
                    ulDataType,
                    Buffer,
                    ulLength,
                ) != 0
                {
                    ret = (CR_REGISTRY_ERROR) as u32;
                }
            }
        }
    }

    if hPropKey != core::ptr::null_mut() {
        RegCloseKey(hPropKey);
    }

    if hInstKey != core::ptr::null_mut() {
        RegCloseKey(hInstKey);
    }

    ret
}
unsafe extern "C" fn CreateDeviceInstance(
    pszDeviceID: LPWSTR,
    bPhantomDevice: BOOL,
) -> CONFIGRET {
    let mut szEnumerator: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szDevice: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut szInstance: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut hKeyEnumerator: HKEY = core::ptr::null_mut();
    let mut hKeyDevice: HKEY = core::ptr::null_mut();
    let mut hKeyInstance: HKEY = core::ptr::null_mut();
    let mut hKeyControl: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;

    SplitDeviceInstanceID(
        pszDeviceID,
        szEnumerator.as_mut_ptr(),
        szDevice.as_mut_ptr(),
        szInstance.as_mut_ptr(),
    );

    lError = RegCreateKeyExW(
        hEnumKey,
        szEnumerator.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKeyEnumerator,
        core::ptr::null_mut(),
    );
    if lError != (ERROR_SUCCESS) as i32 {
        return (CR_REGISTRY_ERROR) as u32;
    }

    lError = RegCreateKeyExW(
        hKeyEnumerator,
        szDevice.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKeyDevice,
        core::ptr::null_mut(),
    );

    RegCloseKey(hKeyEnumerator);

    if lError != (ERROR_SUCCESS) as i32 {
        return (CR_REGISTRY_ERROR) as u32;
    }

    lError = RegOpenKeyExW(
        hKeyDevice,
        szInstance.as_ptr(),
        0,
        KEY_SET_VALUE,
        &mut hKeyInstance,
    );
    if lError == (ERROR_SUCCESS) as i32 {
        // DPRINT removed
        RegCloseKey(hKeyInstance);
        RegCloseKey(hKeyDevice);
        return (CR_ALREADY_SUCH_DEVINST) as u32;
    }

    lError = RegCreateKeyExW(
        hKeyDevice,
        szInstance.as_ptr(),
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKeyInstance,
        core::ptr::null_mut(),
    );

    RegCloseKey(hKeyDevice);

    if lError != (ERROR_SUCCESS) as i32 {
        return (CR_REGISTRY_ERROR) as u32;
    }

    if bPhantomDevice != 0 {
        let dwPhantomValue: DWORD = 1;
        RegSetValueExW(
            hKeyInstance,
            w!("Phantom") as LPCWSTR,
            0,
            (REG_DWORD) as u32,
            &dwPhantomValue as *const DWORD as PBYTE,
            core::mem::size_of::<DWORD>() as DWORD,
        );
    }

    lError = RegCreateKeyExW(
        hKeyInstance,
        w!("Control") as LPCWSTR,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        KEY_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut hKeyControl,
        core::ptr::null_mut(),
    );
    if lError == (ERROR_SUCCESS) as i32 {
        RegCloseKey(hKeyControl);
    }

    RegCloseKey(hKeyInstance);

    if lError == (ERROR_SUCCESS) as i32 {
        (CR_SUCCESS) as u32
    } else {
        (CR_REGISTRY_ERROR) as u32
    }
}
unsafe extern "C" fn GenerateDeviceID(
    pszDeviceID: LPWSTR,
    mut ulLength: PNP_RPC_STRING_LEN,
) -> CONFIGRET {
    let mut szGeneratedInstance: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwInstanceNumber: DWORD;
    let mut dwError: DWORD = ERROR_SUCCESS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    if wcschr(pszDeviceID, '\\' as u16) != core::ptr::null_mut() {
        return (CR_INVALID_DEVICE_ID) as u32;
    }

    dwInstanceNumber = 0;
    while dwError == ERROR_SUCCESS {
        if dwInstanceNumber >= 10000 {
            return (CR_FAILURE) as u32;
        }

        swprintf(
            szGeneratedInstance.as_mut_ptr(),
            w!("Root\\%ls\\%04lu") as PCWSTR,
            pszDeviceID,
            dwInstanceNumber,
        );

        dwError = (RegOpenKeyEx(
            hEnumKey,
            szGeneratedInstance.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hKey,
        )) as u32;
        if dwError == ERROR_SUCCESS {
            RegCloseKey(hKey);
            dwInstanceNumber += 1;
        }
    }

    if wcslen(szGeneratedInstance.as_ptr()) > (ulLength) as usize {
        ret = (CR_BUFFER_SMALL) as u32;
    } else {
        wcscpy(pszDeviceID, szGeneratedInstance.as_ptr());
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_CreateDevInst(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pszParentDeviceID: LPWSTR,
    mut ulLength: PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut ControlData: PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA = core::mem::zeroed();
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD;
    let mut dwPhantom: DWORD;
    let Status: NTSTATUS;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if ulFlags & (!CM_CREATE_DEVNODE_BITS) as u32 != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if pszDeviceID.is_null() || pszParentDeviceID.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if IsRootDeviceInstanceID(pszParentDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if ulFlags & (CM_CREATE_DEVNODE_GENERATE_ID) as u32 != 0 {
        ret = GenerateDeviceID(pszDeviceID, ulLength);
        if ret != (CR_SUCCESS) as u32 {
            return ret;
        }
    }

    RegOpenKeyEx(
        hEnumKey,
        pszDeviceID,
        0,
        KEY_READ | KEY_WRITE,
        &mut hKey,
    );

    'done: {
        if ulFlags & (CM_CREATE_DEVNODE_PHANTOM) as u32 != 0 {
            if !hKey.is_null() {
                ret = (CR_ALREADY_SUCH_DEVINST) as u32;
                break 'done;
            }

            ret = CreateDeviceInstance(pszDeviceID, (TRUE) as i32);
        } else {
            if !hKey.is_null() && IsPresentDeviceInstanceID(pszDeviceID) != 0 {
                ret = (CR_ALREADY_SUCH_DEVINST) as u32;
                break 'done;
            }

            if hKey.is_null() {
                ret = CreateDeviceInstance(pszDeviceID, (FALSE) as i32);

                RegOpenKeyEx(
                    hEnumKey,
                    pszDeviceID,
                    0,
                    KEY_READ | KEY_WRITE,
                    &mut hKey,
                );
            }

            RtlInitUnicodeString(&mut ControlData.DeviceInstance, pszDeviceID);
            Status = NtPlugPlayControl(
                PlugPlayControlInitializeDevice,
                &mut ControlData as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of_val(&ControlData) as ULONG,
            );
            if !NT_SUCCESS(Status) {
                ret = (CR_FAILURE) as u32;
                break 'done;
            }

            if !hKey.is_null() {
                dwPhantom = 0;
                dwSize = core::mem::size_of::<DWORD>() as DWORD;
                RegQueryValueEx(
                    hKey,
                    w!("Phantom") as LPCWSTR,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    &mut dwPhantom as *mut _ as PBYTE,
                    &mut dwSize,
                );

                if dwPhantom != 0 {
                    RegDeleteValue(hKey, w!("Phantom") as LPCWSTR);
                }
            }
        }
    }

    if !hKey.is_null() {
        RegCloseKey(hKey);
    }

    // DPRINT removed

    ret
}
unsafe extern "C" fn SetupDeviceInstance(
    pszDeviceInstance: LPWSTR,
    ulMinorAction: DWORD,
) -> CONFIGRET {
    let mut ControlData: PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA = core::mem::zeroed();
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut dwDisableCount: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut ulStatus: DWORD = 0;
    let mut ulProblem: DWORD = 0;
    let mut dwError: DWORD;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    if IsRootDeviceInstanceID(pszDeviceInstance) != 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if (ulMinorAction & (!CM_SETUP_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if (ulMinorAction == (CM_SETUP_DOWNLOAD) as u32) ||
       (ulMinorAction == (CM_SETUP_WRITE_LOG_CONFS) as u32) {
        return (CR_SUCCESS) as u32;
    }

    dwError = (RegOpenKeyExW(hEnumKey,
                            pszDeviceInstance,
                            0,
                            KEY_READ,
                            &mut hDeviceKey)) as u32;
    if dwError != ERROR_SUCCESS {
        return (CR_INVALID_DEVNODE) as u32;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    dwError = (RegQueryValueExW(hDeviceKey,
                               w!("DisableCount") as LPCWSTR,
                               core::ptr::null_mut(),
                               core::ptr::null_mut(),
                               &mut dwDisableCount as *mut _ as LPBYTE,
                               &mut dwSize)) as u32;
    if (dwError == ERROR_SUCCESS) &&
       (dwDisableCount > 0) {
        ret = ret;
    } else {
        GetDeviceStatus(pszDeviceInstance,
                        &mut ulStatus,
                        &mut ulProblem);

        if (ulStatus & DN_STARTED) == 0 {
            if (ulStatus & DN_HAS_PROBLEM) != 0 {
                ret = ClearDeviceStatus(pszDeviceInstance,
                                        DN_HAS_PROBLEM,
                                        ulProblem);
            }

            if ret == (CR_SUCCESS) as u32 {
                RtlInitUnicodeString(&mut ControlData.DeviceInstance,
                                     pszDeviceInstance);
                Status = NtPlugPlayControl(PlugPlayControlStartDevice,
                                           &mut ControlData as *mut _ as PVOID,
                                           core::mem::size_of::<PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA>() as ULONG);
                if !NT_SUCCESS(Status) {
                    ret = NtStatusToCrError(Status);
                }
            }
        }
    }

    if !hDeviceKey.is_null() {
        RegCloseKey(hDeviceKey);
    }

    ret
}
unsafe extern "C" fn EnableDeviceInstance(
    pszDeviceInstance: LPWSTR,
) -> CONFIGRET {
    let mut ControlData: PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    RtlInitUnicodeString(
        &mut ControlData.DeviceInstance,
        pszDeviceInstance,
    );
    Status = NtPlugPlayControl(
        PlugPlayControlStartDevice,
        &mut ControlData as *mut _ as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
unsafe extern "C" fn ReenumerateDeviceInstance(
    pszDeviceInstance: LPWSTR,
    ulMinorAction: ULONG,
) -> CONFIGRET {
    let mut EnumerateDeviceData: PLUGPLAY_CONTROL_ENUMERATE_DEVICE_DATA = core::mem::zeroed();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let Status: NTSTATUS;

    // DPRINT removed

    if (ulMinorAction & (!CM_REENUMERATE_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if (ulMinorAction & (CM_REENUMERATE_RETRY_INSTALLATION) as u32) != 0 {
        // DPRINT removed
    }

    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(EnumerateDeviceData.DeviceInstance),
        pszDeviceInstance,
    );
    EnumerateDeviceData.Flags = 0;

    Status = NtPlugPlayControl(
        PlugPlayControlEnumerateDevice,
        core::ptr::addr_of_mut!(EnumerateDeviceData) as PVOID,
        core::mem::size_of::<PLUGPLAY_CONTROL_ENUMERATE_DEVICE_DATA>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DeviceInstanceAction(
    hBinding: handle_t,
    ulMajorAction: DWORD,
    ulMinorAction: DWORD,
    pszDeviceInstance1: LPWSTR,
    pszDeviceInstance2: LPWSTR,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    match ulMajorAction {
        PNP_DEVINST_SETUP => {
            ret = SetupDeviceInstance(pszDeviceInstance1, ulMinorAction);
        }

        PNP_DEVINST_ENABLE => {
            ret = EnableDeviceInstance(pszDeviceInstance1);
        }

        PNP_DEVINST_REENUMERATE => {
            ret = ReenumerateDeviceInstance(pszDeviceInstance1, ulMinorAction);
        }

        _ => {
            // DPRINT removed
            ret = (CR_CALL_NOT_IMPLEMENTED) as u32;
        }
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetDeviceStatus(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    pulStatus: *mut DWORD,
    pulProblem: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut ulDataType: DWORD = 0;
    let mut ulTransferLength: DWORD = 0;
    let mut ulLength: DWORD = 0;
    let mut ulCapabilities: DWORD = 0;
    let mut ulConfigFlags: DWORD = 0;
    let mut ret: CONFIGRET;

    UNREFERENCED_PARAMETER!(hBinding);
    UNREFERENCED_PARAMETER!(ulFlags);

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if pulStatus.is_null() || pulProblem.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ret = GetDeviceStatus(pDeviceID, pulStatus, pulProblem);
    if ret != (CR_SUCCESS) as u32 {
        return ret;
    }

    ulTransferLength = core::mem::size_of::<DWORD>() as DWORD;
    ulLength = core::mem::size_of::<DWORD>() as DWORD;
    ret = PNP_GetDeviceRegProp(
        core::ptr::null_mut(),
        pDeviceID,
        (CM_DRP_CAPABILITIES) as u32,
        &mut ulDataType,
        &mut ulCapabilities as *mut DWORD as PBYTE,
        &mut ulTransferLength,
        &mut ulLength,
        0,
    );
    if ret != (CR_SUCCESS) as u32 {
        ulCapabilities = 0;
    }

    if (ulCapabilities & (CM_DEVCAP_REMOVABLE) as u32) != 0 {
        *pulStatus |= DN_REMOVABLE;
    }

    ulTransferLength = core::mem::size_of::<DWORD>() as DWORD;
    ulLength = core::mem::size_of::<DWORD>() as DWORD;
    ret = PNP_GetDeviceRegProp(
        core::ptr::null_mut(),
        pDeviceID,
        (CM_DRP_CONFIGFLAGS) as u32,
        &mut ulDataType,
        &mut ulConfigFlags as *mut DWORD as PBYTE,
        &mut ulTransferLength,
        &mut ulLength,
        0,
    );
    if ret != (CR_SUCCESS) as u32 {
        ulConfigFlags = 0;
    }

    if (ulConfigFlags & (CONFIGFLAG_MANUAL_INSTALL) as u32) != 0 {
        *pulStatus |= DN_MANUAL;
    }

    if ((*pulStatus & DN_HAS_PROBLEM) == 0)
        && ((ulConfigFlags & (CONFIGFLAG_FAILEDINSTALL) as u32) != 0)
    {
        *pulStatus |= DN_HAS_PROBLEM;
        *pulProblem = CM_PROB_FAILED_INSTALL;
    }

    (CR_SUCCESS) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetDeviceProblem(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulProblem: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut ulOldStatus: ULONG = 0;
    let mut ulOldProblem: ULONG = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if (ulFlags & (!CM_SET_DEVNODE_PROBLEM_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ret = GetDeviceStatus(
        pDeviceID,
        &mut ulOldStatus,
        &mut ulOldProblem,
    );
    if ret != (CR_SUCCESS) as u32 {
        return ret;
    }

    if ((ulFlags & (CM_SET_DEVNODE_PROBLEM_OVERRIDE) as u32) == 0)
        && (ulOldProblem != 0)
        && (ulOldProblem != ulProblem)
    {
        return (CR_FAILURE) as u32;
    }

    if ulProblem == 0 {
        ret = ClearDeviceStatus(
            pDeviceID,
            DN_HAS_PROBLEM,
            ulOldProblem,
        );
    } else {
        ret = SetDeviceStatus(
            pDeviceID,
            DN_HAS_PROBLEM,
            ulProblem,
        );
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DisableDevInst(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if ulFlags & (!CM_DISABLE_BITS) as u32 != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 || IsRootDeviceInstanceID(pDeviceID) != 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    return DisableDeviceInstance(pDeviceID, pVetoType, pszVetoName, ulNameLength);
}

#[no_mangle]
pub unsafe extern "system" fn PNP_UninstallDevInst(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}
unsafe extern "C" fn CheckForDeviceId(lpDeviceIdList: LPWSTR, lpDeviceId: LPWSTR) -> BOOL {
    let mut lpPtr: LPWSTR;
    let mut dwLength: DWORD;

    lpPtr = lpDeviceIdList;
    while *lpPtr != 0 {
        dwLength = (wcslen(lpPtr)) as u32;
        if 0 == _wcsicmp(lpPtr, lpDeviceId) {
            return (TRUE) as i32;
        }

        lpPtr = lpPtr.offset((dwLength + 1) as isize);
    }

    return (FALSE) as i32;
}
unsafe extern "C" fn AppendDeviceId(
    lpDeviceIdList: LPWSTR,
    lpDeviceIdListSize: LPDWORD,
    lpDeviceId: LPWSTR,
) {
    let dwLen: DWORD;
    let dwPos: DWORD;

    dwLen = (wcslen(lpDeviceId)) as u32;
    dwPos = (*lpDeviceIdListSize / core::mem::size_of::<WCHAR>() as u32) - 1;

    wcscpy(lpDeviceIdList.offset(dwPos as isize), lpDeviceId);

    let dwPos = dwPos + (dwLen + 1);

    *lpDeviceIdList.offset(dwPos as isize) = 0;

    *lpDeviceIdListSize = dwPos * core::mem::size_of::<WCHAR>() as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_AddID(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pszID: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let pszSubKey: LPWSTR;
    let mut dwDeviceIdListSize: DWORD = 0;
    let dwNewDeviceIdSize: DWORD;
    let mut pszDeviceIdList: *mut WCHAR = core::ptr::null_mut();

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'Done: {
        if RegOpenKeyExW(
            hEnumKey,
            pszDeviceID,
            0,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            &mut hDeviceKey,
        ) != (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            return (CR_INVALID_DEVNODE) as u32;
        }

        pszSubKey = if (ulFlags & (CM_ADD_ID_COMPATIBLE) as u32) != 0 {
            w!("CompatibleIDs") as LPWSTR
        } else {
            w!("HardwareID") as LPWSTR
        };

        if RegQueryValueExW(
            hDeviceKey,
            pszSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut dwDeviceIdListSize,
        ) != (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'Done;
        }

        dwNewDeviceIdSize = (lstrlenW(pszDeviceID)) as u32;
        if dwNewDeviceIdSize == 0 {
            ret = (CR_INVALID_POINTER) as u32;
            break 'Done;
        }

        dwDeviceIdListSize =
            dwDeviceIdListSize
                .wrapping_add((dwNewDeviceIdSize.wrapping_add(2))
                    .wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));

        pszDeviceIdList = HeapAlloc(
            GetProcessHeap(),
            HEAP_ZERO_MEMORY,
            dwDeviceIdListSize as SIZE_T,
        ) as *mut WCHAR;
        if pszDeviceIdList.is_null() {
            // DPRINT removed
            ret = (CR_OUT_OF_MEMORY) as u32;
            break 'Done;
        }

        if RegQueryValueExW(
            hDeviceKey,
            pszSubKey,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            pszDeviceIdList as LPBYTE,
            &mut dwDeviceIdListSize,
        ) != (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'Done;
        }

        if CheckForDeviceId(pszDeviceIdList, pszDeviceID) != 0 {
            // DPRINT removed
            ret = (CR_SUCCESS) as u32;
            break 'Done;
        }

        AppendDeviceId(pszDeviceIdList, &mut dwDeviceIdListSize, pszID);

        if RegSetValueExW(
            hDeviceKey,
            pszSubKey,
            0,
            (REG_MULTI_SZ) as u32,
            pszDeviceIdList as LPBYTE,
            dwDeviceIdListSize,
        ) != (ERROR_SUCCESS) as i32
        {
            // DPRINT removed
            ret = (CR_REGISTRY_ERROR) as u32;
        }
    }

    RegCloseKey(hDeviceKey);
    if !pszDeviceIdList.is_null() {
        HeapFree(GetProcessHeap(), 0, pszDeviceIdList as LPVOID);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RegisterDriver(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    // DPRINT removed

    if ulFlags & (!CM_REGISTER_DEVICE_DRIVER_BITS) as u32 != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    SetDeviceStatus(pszDeviceID, 0, 0);

    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_QueryRemove(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_QUERY_REMOVE_DATA = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    // DPRINT removed

    if (ulFlags & (!CM_REMOVE_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pszDeviceID) == 0
        || IsRootDeviceInstanceID(pszDeviceID) != 0
    {
        return (CR_INVALID_DEVINST) as u32;
    }

    if !pVetoType.is_null() {
        *pVetoType = PNP_VetoTypeUnknown;
    }

    if !pszVetoName.is_null() && ulNameLength > 0 {
        *pszVetoName = UNICODE_NULL;
    }

    RtlZeroMemory!(
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of_val(&PlugPlayData),
    );
    RtlInitUnicodeString(
        core::ptr::addr_of_mut!(PlugPlayData.DeviceInstance),
        pszDeviceID,
    );
    PlugPlayData.VetoName = pszVetoName;
    PlugPlayData.NameLength = ulNameLength;

    Status = NtPlugPlayControl(
        PlugPlayControlQueryAndRemoveDevice,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of_val(&PlugPlayData) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RequestDeviceEject(
    hBinding: handle_t,
    pszDeviceID: LPWSTR,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut PlugPlayData: PLUGPLAY_CONTROL_QUERY_REMOVE_DATA = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pszDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if !pVetoType.is_null() {
        *pVetoType = PNP_VetoTypeUnknown;
    }

    if !pszVetoName.is_null() && ulNameLength > 0 {
        *pszVetoName = UNICODE_NULL;
    }

    RtlZeroMemory!(
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of_val(&PlugPlayData) as SIZE_T,
    );
    RtlInitUnicodeString(
        &mut PlugPlayData.DeviceInstance,
        pszDeviceID,
    );
    PlugPlayData.VetoName = pszVetoName;
    PlugPlayData.NameLength = ulNameLength;

    Status = NtPlugPlayControl(
        PlugPlayControlQueryAndRemoveDevice,
        &mut PlugPlayData as *mut _ as PVOID,
        core::mem::size_of_val(&PlugPlayData) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        ret = NtStatusToCrError(Status);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn PNP_IsDockStationPresent(
    hBinding: handle_t,
    Present: *mut BOOL,
) -> CONFIGRET {
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut dwType: DWORD = 0;
    let mut dwValue: DWORD = 0;
    let mut dwSize: DWORD = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    *Present = (FALSE) as i32;

    if RegOpenKeyExW(
        HKEY_CURRENT_CONFIG,
        w!("CurrentDockInfo") as LPCWSTR,
        0,
        KEY_READ,
        &mut hKey,
    ) != (ERROR_SUCCESS) as i32
    {
        return (CR_REGISTRY_ERROR) as u32;
    }

    dwSize = core::mem::size_of::<DWORD>() as DWORD;
    if RegQueryValueExW(
        hKey,
        w!("DockingState") as LPCWSTR,
        core::ptr::null_mut(),
        &mut dwType,
        &mut dwValue as *mut DWORD as LPBYTE,
        &mut dwSize,
    ) != (ERROR_SUCCESS) as i32
    {
        ret = (CR_REGISTRY_ERROR) as u32;
    }

    RegCloseKey(hKey);

    if ret == (CR_SUCCESS) as u32 {
        if dwType != (REG_DWORD) as u32 || dwSize != core::mem::size_of::<DWORD>() as DWORD {
            ret = (CR_REGISTRY_ERROR) as u32;
        } else if dwValue != 0 {
            *Present = (TRUE) as i32;
        }
    }

    // DPRINT removed

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RequestEjectPC(
    hBinding: handle_t,
) -> DWORD {
    let mut szDockDeviceInstance: [WCHAR; (MAX_DEVICE_ID_LEN) as usize] = [0; (MAX_DEVICE_ID_LEN) as usize];
    let mut DockData: PLUGPLAY_CONTROL_RETRIEVE_DOCK_DATA = core::mem::zeroed();
    let Status: NTSTATUS;

    DockData.DeviceInstanceLength = ARRAYSIZE!(szDockDeviceInstance);
    DockData.DeviceInstance = szDockDeviceInstance.as_mut_ptr();

    Status = NtPlugPlayControl(PlugPlayControlRetrieveDock,
                               &mut DockData as *mut _ as PVOID,
                               core::mem::size_of::<PLUGPLAY_CONTROL_RETRIEVE_DOCK_DATA>() as ULONG);
    if !NT_SUCCESS(Status) {
        return NtStatusToCrError(Status);
    }

    return PNP_RequestDeviceEject(hBinding,
                                  szDockDeviceInstance.as_mut_ptr(),
                                  core::ptr::null_mut(),
                                  core::ptr::null_mut(),
                                  0,
                                  0);
}

#[no_mangle]
pub unsafe extern "system" fn PNP_HwProfFlags(
    hBinding: handle_t,
    ulAction: DWORD,
    pDeviceID: LPWSTR,
    ulConfig: DWORD,
    pulValue: *mut DWORD,
    pVetoType: PPNP_VETO_TYPE,
    pszVetoName: LPWSTR,
    ulNameLength: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;
    let mut szKeyName: [WCHAR; (MAX_PATH) as usize] = [0; (MAX_PATH) as usize];
    let mut hKey: HKEY = core::ptr::null_mut();
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut dwSize: DWORD;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    if ulConfig == 0 {
        wcscpy(
            szKeyName.as_mut_ptr(),
            w!("System\\CurrentControlSet\\HardwareProfiles\\Current\\System\\CurrentControlSet\\Enum") as LPCWSTR,
        );
    } else {
        swprintf(
            szKeyName.as_mut_ptr(),
            w!("System\\CurrentControlSet\\HardwareProfiles\\%04lu\\System\\CurrentControlSet\\Enum") as LPCWSTR,
            ulConfig,
        );
    }

    if RegOpenKeyExW(
        HKEY_LOCAL_MACHINE,
        szKeyName.as_ptr(),
        0,
        KEY_QUERY_VALUE,
        &mut hKey,
    ) != (ERROR_SUCCESS) as i32
    {
        return (CR_REGISTRY_ERROR) as u32;
    }

    if ulAction == (PNP_GET_HWPROFFLAGS) as u32 {
        if RegOpenKeyExW(
            hKey,
            pDeviceID,
            0,
            KEY_QUERY_VALUE,
            &mut hDeviceKey,
        ) != (ERROR_SUCCESS) as i32
        {
            *pulValue = 0;
        } else {
            dwSize = core::mem::size_of::<DWORD>() as DWORD;
            if RegQueryValueExW(
                hDeviceKey,
                w!("CSConfigFlags") as LPCWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                pulValue as LPBYTE,
                &mut dwSize,
            ) != (ERROR_SUCCESS) as i32
            {
                *pulValue = 0;
            }

            RegCloseKey(hDeviceKey);
        }
    } else if ulAction == (PNP_SET_HWPROFFLAGS) as u32 {
        ret = (CR_CALL_NOT_IMPLEMENTED) as u32;
    }

    RegCloseKey(hKey);

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetHwProfInfo(
    hBinding: handle_t,
    ulIndex: DWORD,
    pHWProfileInfo: *mut HWPROFILEINFO,
    ulProfileInfoSize: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut szProfileName: [WCHAR; 5] = [0; 5];
    let mut hKeyConfig: HKEY = core::ptr::null_mut();
    let mut hKeyProfiles: HKEY = core::ptr::null_mut();
    let mut hKeyProfile: HKEY = core::ptr::null_mut();
    let mut dwDisposition: DWORD = 0;
    let mut dwSize: DWORD;
    let mut lError: LONG;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'done: loop {
        if ulProfileInfoSize == 0 {
            ret = (CR_INVALID_DATA) as u32;
            break 'done;
        }

        if ulFlags != 0 {
            ret = (CR_INVALID_FLAG) as u32;
            break 'done;
        }

        (*pHWProfileInfo).HWPI_ulHWProfile = 0;
        (*pHWProfileInfo).HWPI_szFriendlyName[0] = 0;
        (*pHWProfileInfo).HWPI_dwFlags = 0;

        lError = RegCreateKeyExW(
            HKEY_LOCAL_MACHINE,
            w!("System\\CurrentControlSet\\Control\\IDConfigDB") as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_QUERY_VALUE,
            core::ptr::null_mut(),
            &mut hKeyConfig,
            &mut dwDisposition,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        lError = RegCreateKeyExW(
            hKeyConfig,
            w!("Hardware Profiles") as LPCWSTR,
            0,
            core::ptr::null_mut(),
            REG_OPTION_NON_VOLATILE,
            KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
            core::ptr::null_mut(),
            &mut hKeyProfiles,
            &mut dwDisposition,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        if ulIndex == (-1 as i32) as ULONG {
            dwSize = core::mem::size_of::<ULONG>() as DWORD;
            lError = RegQueryValueExW(
                hKeyConfig,
                w!("CurrentConfig") as LPCWSTR,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!((*pHWProfileInfo).HWPI_ulHWProfile) as LPBYTE,
                &mut dwSize,
            );
            if lError != (ERROR_SUCCESS) as i32 {
                (*pHWProfileInfo).HWPI_ulHWProfile = 0;
                ret = (CR_REGISTRY_ERROR) as u32;
                break 'done;
            }
        } else {
            ret = (CR_CALL_NOT_IMPLEMENTED) as u32;
            break 'done;
        }

        swprintf(
            szProfileName.as_mut_ptr(),
            w!("%04lu") as LPCWSTR,
            (*pHWProfileInfo).HWPI_ulHWProfile,
        );

        lError = RegOpenKeyExW(
            hKeyProfiles,
            szProfileName.as_ptr(),
            0,
            KEY_QUERY_VALUE,
            &mut hKeyProfile,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        dwSize = core::mem::size_of::<[WCHAR; MAX_PROFILE_LEN as usize]>() as DWORD;
        lError = RegQueryValueExW(
            hKeyProfile,
            w!("FriendlyName") as LPCWSTR,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*pHWProfileInfo).HWPI_szFriendlyName) as LPBYTE,
            &mut dwSize,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        break 'done;
    }

    if !hKeyProfile.is_null() {
        RegCloseKey(hKeyProfile);
    }

    if !hKeyProfiles.is_null() {
        RegCloseKey(hKeyProfiles);
    }

    if !hKeyConfig.is_null() {
        RegCloseKey(hKeyConfig);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn PNP_AddEmptyLogConf(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulPriority: DWORD,
    pulLogConfTag: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut RegDataType: DWORD = 0;
    let mut ulDataSize: ULONG = 0;
    let mut ulNewSize: ULONG = 0;
    let mut pDataBuffer: PBYTE = core::ptr::null_mut();
    let mut szValueNameBuffer: [WCHAR; (LOGCONF_NAME_BUFFER_SIZE) as usize] = [0; (LOGCONF_NAME_BUFFER_SIZE) as usize];
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if pulLogConfTag.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pulLogConfTag = 0;

    if (ulFlags & (!(LOG_CONF_BITS | PRIORITY_BIT)) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 || IsRootDeviceInstanceID(pDeviceID) != 0 {
        return (CR_INVALID_DEVNODE) as u32;
    }

    ret = OpenConfigurationKey(pDeviceID, ulFlags & (LOG_CONF_BITS) as u32, &mut hConfigKey);
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        hConfigKey = core::ptr::null_mut();
        pDataBuffer = core::ptr::null_mut();
    } else {
        ret = GetConfigurationData(
            hConfigKey,
            ulFlags & (LOG_CONF_BITS) as u32,
            &mut RegDataType,
            &mut ulDataSize,
            &mut pDataBuffer,
            (LOGCONF_NAME_BUFFER_SIZE) as u32,
            szValueNameBuffer.as_mut_ptr(),
        );

        if ret != (CR_SUCCESS) as u32 || ulDataSize == 0 {
            ret = (CR_SUCCESS) as u32;

            if RegDataType == (REG_RESOURCE_LIST) as u32 {
                ulDataSize = core::mem::size_of::<CM_RESOURCE_LIST>() as ULONG - core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG;
                pDataBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ulDataSize as SIZE_T) as PBYTE;
                if pDataBuffer.is_null() {
                    ret = (CR_OUT_OF_MEMORY) as u32;
                } else {
                    let pResourceList: PCM_RESOURCE_LIST = pDataBuffer as PCM_RESOURCE_LIST;
                    (*pResourceList).Count = 1;

                    let pResource: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::addr_of_mut!((*pResourceList).List[0]) as PCM_FULL_RESOURCE_DESCRIPTOR;
                    (*pResource).InterfaceType = InterfaceTypeUndefined;
                    (*pResource).BusNumber = 0;
                    (*pResource).PartialResourceList.Version = 1;
                    (*pResource).PartialResourceList.Revision = 1;
                    (*pResource).PartialResourceList.Count = 0;
                }
            } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
                ulDataSize = core::mem::size_of::<IO_RESOURCE_REQUIREMENTS_LIST>() as ULONG;
                pDataBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ulDataSize as SIZE_T) as PBYTE;
                if pDataBuffer.is_null() {
                    ret = (CR_OUT_OF_MEMORY) as u32;
                } else {
                    let pRequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST = pDataBuffer as PIO_RESOURCE_REQUIREMENTS_LIST;
                    (*pRequirementsList).ListSize = ulDataSize;
                    (*pRequirementsList).InterfaceType = InterfaceTypeUndefined;
                    (*pRequirementsList).BusNumber = 0;
                    (*pRequirementsList).SlotNumber = 0;
                    (*pRequirementsList).AlternativeLists = 1;

                    let pResourceList: PIO_RESOURCE_LIST = core::ptr::addr_of_mut!((*pRequirementsList).List[0]) as PIO_RESOURCE_LIST;
                    (*pResourceList).Version = 1;
                    (*pResourceList).Revision = 1;
                    (*pResourceList).Count = 1;

                    (*pResourceList).Descriptors[0].Option = (IO_RESOURCE_PREFERRED) as u8;
                    (*pResourceList).Descriptors[0].Type = (CmResourceTypeConfigData) as u8;
                    (*pResourceList).Descriptors[0].u.ConfigData.Priority = ulPriority;
                }
            } else {
                ret = (CR_FAILURE) as u32;
            }
        } else {
            if RegDataType == (REG_RESOURCE_LIST) as u32 {
                ulNewSize = core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() as ULONG - core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as ULONG;
                pDataBuffer = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    pDataBuffer as LPVOID,
                    (ulDataSize + ulNewSize) as SIZE_T,
                ) as PBYTE;
                if pDataBuffer.is_null() {
                    ret = (CR_OUT_OF_MEMORY) as u32;
                } else {
                    let pResourceList: PCM_RESOURCE_LIST = pDataBuffer as PCM_RESOURCE_LIST;

                    let mut pResource: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::addr_of_mut!((*pResourceList).List[0]) as PCM_FULL_RESOURCE_DESCRIPTOR;
                    let mut ulIndex: ULONG = 0;
                    while ulIndex < (*pResourceList).Count {
                        pResource = NextResourceDescriptor(pResource);
                        ulIndex += 1;
                    }

                    (*pResourceList).Count += 1;
                    (*pResource).InterfaceType = InterfaceTypeUndefined;
                    (*pResource).BusNumber = 0;
                    (*pResource).PartialResourceList.Version = 1;
                    (*pResource).PartialResourceList.Revision = 1;
                    (*pResource).PartialResourceList.Count = 0;

                    *pulLogConfTag = ulIndex;
                }
            } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
                ulNewSize = core::mem::size_of::<IO_RESOURCE_REQUIREMENTS_LIST>() as ULONG;
                pDataBuffer = HeapReAlloc(
                    GetProcessHeap(),
                    0,
                    pDataBuffer as LPVOID,
                    (ulDataSize + ulNewSize) as SIZE_T,
                ) as PBYTE;
                if pDataBuffer.is_null() {
                    ret = (CR_OUT_OF_MEMORY) as u32;
                } else {
                    let pRequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST = pDataBuffer as PIO_RESOURCE_REQUIREMENTS_LIST;
                    let mut pResourceList: PIO_RESOURCE_LIST = core::ptr::addr_of_mut!((*pRequirementsList).List[0]) as PIO_RESOURCE_LIST;
                    let mut ulIndex: ULONG = 0;
                    while ulIndex < (*pRequirementsList).AlternativeLists - 1 {
                        pResourceList = NextResourceRequirement(pResourceList);
                        ulIndex += 1;
                    }

                    (*pRequirementsList).ListSize = ulDataSize + ulNewSize;
                    (*pRequirementsList).AlternativeLists += 1;

                    (*pResourceList).Version = 1;
                    (*pResourceList).Revision = 1;
                    (*pResourceList).Count = 1;

                    (*pResourceList).Descriptors[0].Option = (IO_RESOURCE_PREFERRED) as u8;
                    (*pResourceList).Descriptors[0].Type = (CmResourceTypeConfigData) as u8;
                    (*pResourceList).Descriptors[0].u.ConfigData.Priority = ulPriority;

                    *pulLogConfTag = ulIndex;
                }
            } else {
                ret = (CR_FAILURE) as u32;
            }
        }

        if ret == (CR_SUCCESS) as u32 {
            if RegSetValueEx(
                hConfigKey,
                szValueNameBuffer.as_mut_ptr(),
                0,
                RegDataType,
                pDataBuffer as *const u8,
                ulDataSize + ulNewSize,
            ) != (ERROR_SUCCESS) as i32
            {
                ret = (CR_REGISTRY_ERROR) as u32;
            }
        }
    }

    if !pDataBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pDataBuffer as LPVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_FreeLogConf(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfType: DWORD,
    ulLogConfTag: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut RegDataType: DWORD = 0;
    let mut ulDataSize: ULONG = 0;
    let mut pDataBuffer: LPBYTE = core::ptr::null_mut();
    let mut szValueNameBuffer: [WCHAR; (LOGCONF_NAME_BUFFER_SIZE) as usize] = [0; (LOGCONF_NAME_BUFFER_SIZE) as usize];
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVNODE) as u32;
    }

    ret = OpenConfigurationKey(
        pDeviceID,
        ulLogConfType,
        &mut hConfigKey,
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        hConfigKey = core::ptr::null_mut();
        pDataBuffer = core::ptr::null_mut();
    } else {
        ret = GetConfigurationData(
            hConfigKey,
            ulLogConfType,
            &mut RegDataType,
            &mut ulDataSize,
            &mut pDataBuffer,
            (LOGCONF_NAME_BUFFER_SIZE) as u32,
            szValueNameBuffer.as_mut_ptr(),
        );
        if ret != (CR_SUCCESS) as u32 {
            ret = (CR_INVALID_LOG_CONF) as u32;
        } else {
            if RegDataType == (REG_RESOURCE_LIST) as u32 {
                let pResourceList: PCM_RESOURCE_LIST = pDataBuffer as PCM_RESOURCE_LIST;

                if (*pResourceList).Count <= 1 {
                    // DPRINT removed
                    RegDeleteValue(hConfigKey, szValueNameBuffer.as_ptr());
                } else {
                    let mut pResource: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
                    let mut pNextResource: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
                    let mut ulIndex: ULONG;
                    let mut ulMoveSize: ULONG;
                    let mut dwError: DWORD = 0;

                    if ulLogConfTag >= (*pResourceList).Count {
                        ret = (CR_INVALID_LOG_CONF) as u32;
                    } else {
                        pResource = &(*pResourceList).List[0] as *const _ as PCM_FULL_RESOURCE_DESCRIPTOR;
                        ulIndex = 0;
                        while ulIndex < ulLogConfTag {
                            pResource = NextResourceDescriptor(pResource);
                            ulIndex += 1;
                        }

                        if ulLogConfTag == (*pResourceList).Count - 1 {
                            ulDataSize = (pResource as ULONG_PTR - pResourceList as ULONG_PTR) as ULONG;
                        } else {
                            pNextResource = NextResourceDescriptor(pResource);

                            ulMoveSize = ulDataSize - (pNextResource as ULONG_PTR - pResourceList as ULONG_PTR) as DWORD;
                            MoveMemory!(pResource as LPVOID, pNextResource as LPVOID, ulMoveSize as usize);

                            ulDataSize -= (pNextResource as ULONG_PTR - pResource as ULONG_PTR) as DWORD;
                        }

                        (*pResourceList).Count -= 1;

                        dwError = (RegSetValueEx(
                            hConfigKey,
                            szValueNameBuffer.as_ptr(),
                            0,
                            RegDataType,
                            pDataBuffer,
                            ulDataSize,
                        )) as u32;
                        if dwError != ERROR_SUCCESS {
                            ret = (CR_REGISTRY_ERROR) as u32;
                        }
                    }
                }
            } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
                if (*(pDataBuffer as PIO_RESOURCE_REQUIREMENTS_LIST)).AlternativeLists <= 1 {
                    // DPRINT removed
                    RegDeleteValue(hConfigKey, szValueNameBuffer.as_ptr());
                } else {
                }
            } else {
                ret = (CR_FAILURE) as u32;
            }
        }
    }

    if !pDataBuffer.is_null() {
        HeapFree(GetProcessHeap(), 0, pDataBuffer as LPVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetFirstLogConf(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfType: DWORD,
    pulLogConfTag: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut RegDataType: DWORD = 0;
    let mut ulDataSize: ULONG = 0;
    let mut lpData: LPBYTE = core::ptr::null_mut();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if pulLogConfTag.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pulLogConfTag = 0 as DWORD;

    if (ulFlags & (!LOG_CONF_BITS) as u32) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ret = OpenConfigurationKey(
        pDeviceID,
        ulLogConfType,
        &mut hConfigKey,
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    ret = GetConfigurationData(
        hConfigKey,
        ulLogConfType,
        &mut RegDataType,
        &mut ulDataSize,
        &mut lpData,
        0,
        core::ptr::null_mut(),
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    // DPRINT removed
    if ulDataSize == 0 || lpData.is_null() {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    if RegDataType == (REG_RESOURCE_LIST) as u32 {
        // DPRINT removed

        if (*(lpData as PCM_RESOURCE_LIST)).Count == 0 {
            // DPRINT removed
            ret = (CR_NO_MORE_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }
    } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
        // DPRINT removed

        if (*(lpData as PIO_RESOURCE_REQUIREMENTS_LIST)).AlternativeLists == 0 {
            ret = (CR_NO_MORE_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }
    }

    if !lpData.is_null() {
        HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetNextLogConf(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfType: DWORD,
    ulCurrentTag: DWORD,
    pulNextTag: LPDWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut RegDataType: DWORD = 0;
    let mut ulDataSize: ULONG = 0;
    let mut lpData: LPBYTE = core::ptr::null_mut();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if pulNextTag.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pulNextTag = 0 as DWORD;

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ret = OpenConfigurationKey(
        pDeviceID,
        ulLogConfType,
        &mut hConfigKey,
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    ret = GetConfigurationData(
        hConfigKey,
        ulLogConfType,
        &mut RegDataType,
        &mut ulDataSize,
        &mut lpData,
        0,
        core::ptr::null_mut(),
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    // DPRINT removed

    if ulDataSize == 0 || lpData.is_null() {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        return 'done: {
            if !lpData.is_null() {
                HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
            }

            if !hConfigKey.is_null() {
                RegCloseKey(hConfigKey);
            }

            // DPRINT removed

            break 'done ret;
        };
    }

    if RegDataType == (REG_RESOURCE_LIST) as u32 {
        // DPRINT removed

        if ulCurrentTag >= (*(lpData as PCM_RESOURCE_LIST)).Count {
            ret = (CR_INVALID_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }

        if ulCurrentTag == (*(lpData as PCM_RESOURCE_LIST)).Count - 1 {
            ret = (CR_NO_MORE_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }
    } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
        // DPRINT removed

        if ulCurrentTag >= (*(lpData as PIO_RESOURCE_REQUIREMENTS_LIST)).AlternativeLists {
            ret = (CR_INVALID_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }

        if ulCurrentTag == (*(lpData as PIO_RESOURCE_REQUIREMENTS_LIST)).AlternativeLists - 1 {
            ret = (CR_NO_MORE_LOG_CONF) as u32;
            return 'done: {
                if !lpData.is_null() {
                    HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
                }

                if !hConfigKey.is_null() {
                    RegCloseKey(hConfigKey);
                }

                // DPRINT removed

                break 'done ret;
            };
        }
    }

    *pulNextTag = ulCurrentTag + 1;

    if !lpData.is_null() {
        HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetLogConfPriority(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulType: DWORD,
    ulTag: DWORD,
    pPriority: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_AddResDes(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    ResourceID: RESOURCEID,
    pulResourceTag: *mut DWORD,
    ResourceData: *mut BYTE,
    ResourceLen: PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_FreeResDes(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    ResourceID: RESOURCEID,
    ulResourceTag: DWORD,
    pulPreviousResType: *mut DWORD,
    pulPreviousResTag: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetNextResDes(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    ResourceID: RESOURCEID,
    ulResourceTag: DWORD,
    pulNextResType: *mut DWORD,
    pulNextResTag: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    let mut hConfigKey: HKEY = core::ptr::null_mut();
    let mut RegDataType: DWORD = 0;
    let mut ulDataSize: ULONG = 0;
    let mut lpData: LPBYTE = core::ptr::null_mut();
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    // DPRINT removed

    if pulNextResType.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pulNextResType = 0;

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if IsValidDeviceInstanceID(pDeviceID) == 0 {
        return (CR_INVALID_DEVINST) as u32;
    }

    ret = OpenConfigurationKey(
        pDeviceID,
        ulLogConfType,
        &mut hConfigKey,
    );
    if ret != (CR_SUCCESS) as u32 {
        // DPRINT removed
        ret = (CR_NO_MORE_LOG_CONF) as u32;
        // goto done
    } else {
        ret = GetConfigurationData(
            hConfigKey,
            ulLogConfType,
            &mut RegDataType,
            &mut ulDataSize,
            &mut lpData,
            0,
            core::ptr::null_mut(),
        );
        if ret != (CR_SUCCESS) as u32 {
            // DPRINT removed
            ret = (CR_NO_MORE_LOG_CONF) as u32;
            // goto done
        } else {
            // DPRINT removed

            if ulDataSize == 0 || lpData.is_null() {
                // DPRINT removed
                ret = (CR_NO_MORE_LOG_CONF) as u32;
                // goto done
            } else {
                if RegDataType == (REG_RESOURCE_LIST) as u32 {
                    // DPRINT removed
                    ret = (CR_NO_MORE_LOG_CONF) as u32;
                    // goto done
                } else if RegDataType == (REG_RESOURCE_REQUIREMENTS_LIST) as u32 {
                    // DPRINT removed
                    ret = (CR_NO_MORE_LOG_CONF) as u32;
                    // goto done
                }
            }
        }
    }

    if !lpData.is_null() {
        HeapFree(GetProcessHeap(), 0, lpData as LPVOID);
    }

    if !hConfigKey.is_null() {
        RegCloseKey(hConfigKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetResDesData(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    ResourceID: RESOURCEID,
    ulResourceTag: DWORD,
    Buffer: *mut BYTE,
    BufferLen: PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetResDesDataSize(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    ResourceID: RESOURCEID,
    ulResourceTag: DWORD,
    pulSize: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_ModifyResDes(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ulLogConfTag: DWORD,
    ulLogConfType: DWORD,
    CurrentResourceID: RESOURCEID,
    NewResourceID: RESOURCEID,
    ulResourceTag: DWORD,
    ResourceData: *mut BYTE,
    ResourceLen: PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}
#[no_mangle]
pub unsafe extern "system" fn PNP_DetectResourceConflict(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ResourceID: RESOURCEID,
    ResourceData: *mut BYTE,
    ResourceLen: PNP_RPC_BUFFER_SIZE,
    pbConflictDetected: *mut BOOL,
    ulFlags: DWORD,
) -> DWORD {
    if !pbConflictDetected.is_null() {
        *pbConflictDetected = (FALSE) as i32;
    }

    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_QueryResConfList(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    ResourceID: RESOURCEID,
    ResourceData: *mut BYTE,
    ResourceLen: PNP_RPC_BUFFER_SIZE,
    Buffer: *mut BYTE,
    BufferLen: PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetHwProf(
    hBinding: handle_t,
    ulHardwareProfile: DWORD,
    ulFlags: DWORD,
) -> DWORD {
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_QueryArbitratorFreeData(
    hBinding: handle_t,
    pData: *mut BYTE,
    DataLen: DWORD,
    pDeviceID: LPWSTR,
    ResourceID: RESOURCEID,
    ulFlags: DWORD,
) -> DWORD {
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_QueryArbitratorFreeSize(
    hBinding: handle_t,
    pulSize: *mut DWORD,
    pDeviceID: LPWSTR,
    ResourceID: RESOURCEID,
    ulFlags: DWORD,
) -> DWORD {
    if !pulSize.is_null() {
        *pulSize = 0;
    }

    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RunDetection(
    hBinding: handle_t,
    ulFlags: DWORD,
) -> CONFIGRET {
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RegisterNotification(
    hBinding: handle_t,
    hRecipient: DWORD_PTR,
    pszName: LPWSTR,
    pNotificationFilter: *mut BYTE,
    ulNotificationFilterSize: DWORD,
    ulFlags: DWORD,
    pNotifyHandle: *mut PNP_NOTIFY_HANDLE,
    ulProcessId: DWORD,
    pulUnknown9: PDWORD,
) -> DWORD {
    let pBroadcastDeviceInterface: PDEV_BROADCAST_DEVICEINTERFACE_W;
    let pBroadcastDeviceHandle: PDEV_BROADCAST_HANDLE;
    let mut pNotifyData: PNOTIFY_ENTRY = core::ptr::null_mut();
    let mut ret: DWORD = (CR_SUCCESS) as u32;

    // DPRINT removed

    if pNotifyHandle.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    *pNotifyHandle = core::ptr::null_mut();

    if pNotificationFilter.is_null() || pulUnknown9.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if (ulFlags & !0x7) != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    if (ulNotificationFilterSize < core::mem::size_of::<DEV_BROADCAST_HDR>() as DWORD)
        || ((*(pNotificationFilter as PDEV_BROADCAST_HDR)).dbch_size
            < core::mem::size_of::<DEV_BROADCAST_HDR>() as DWORD)
    {
        return (CR_INVALID_DATA) as u32;
    }

    'done: loop {
        if (*(pNotificationFilter as PDEV_BROADCAST_HDR)).dbch_devicetype == (DBT_DEVTYP_DEVICEINTERFACE) as u32 {
            // DPRINT removed
            pBroadcastDeviceInterface = pNotificationFilter as PDEV_BROADCAST_DEVICEINTERFACE_W;

            if (ulNotificationFilterSize < core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as DWORD)
                || ((*pBroadcastDeviceInterface).dbcc_size
                    < core::mem::size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as DWORD)
            {
                ret = (CR_INVALID_DATA) as u32;
                break 'done;
            }

            pNotifyData = RtlAllocateHeap(
                GetProcessHeap(),
                HEAP_ZERO_MEMORY,
                core::mem::size_of::<NOTIFY_ENTRY>() as SIZE_T,
            ) as PNOTIFY_ENTRY;
            if pNotifyData.is_null() {
                ret = (CR_OUT_OF_MEMORY) as u32;
                break 'done;
            }

            (*pNotifyData).dwType = CLASS_NOTIFICATION;

            if !pszName.is_null() {
                (*pNotifyData).pszName = RtlAllocateHeap(
                    GetProcessHeap(),
                    HEAP_ZERO_MEMORY,
                    ((wcslen(pszName) + 1) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as LPWSTR;
                if (*pNotifyData).pszName.is_null() {
                    ret = (CR_OUT_OF_MEMORY) as u32;
                    break 'done;
                }

                wcscpy((*pNotifyData).pszName, pszName);
            }

            if (ulFlags & (DEVICE_NOTIFY_SERVICE_HANDLE) as u32) == (DEVICE_NOTIFY_WINDOW_HANDLE) as u32 {
                (*pNotifyData).hRecipient = hRecipient;
            } else {
                // DPRINT removed
                if I_ScValidatePnpService(
                    core::ptr::null_mut(),
                    pszName,
                    core::ptr::addr_of_mut!((*pNotifyData).hRecipient) as *mut SERVICE_STATUS_HANDLE,
                ) != ERROR_SUCCESS
                {
                    // DPRINT removed
                    ret = (CR_INVALID_DATA) as u32;
                    break 'done;
                }

                // DPRINT removed
            }

            (*pNotifyData).ulFlags = ulFlags;

            if (ulFlags & (DEVICE_NOTIFY_ALL_INTERFACE_CLASSES) as u32) == 0 {
                CopyMemory!(
                    core::ptr::addr_of_mut!((*pNotifyData).ClassGuid) as LPVOID,
                    core::ptr::addr_of!((*pBroadcastDeviceInterface).dbcc_classguid) as *const _
                        as LPVOID,
                    core::mem::size_of::<GUID>(),
                );
            }

            RtlAcquireResourceExclusive(
                core::ptr::addr_of_mut!(NotificationListLock),
                (TRUE) as u8,
            );
            InsertTailList(
                core::ptr::addr_of_mut!(NotificationListHead),
                core::ptr::addr_of_mut!((*pNotifyData).ListEntry),
            );
            RtlReleaseResource(core::ptr::addr_of_mut!(NotificationListLock));

            // DPRINT removed
            *pNotifyHandle = pNotifyData as PNP_NOTIFY_HANDLE;
        } else if (*(pNotificationFilter as PDEV_BROADCAST_HDR)).dbch_devicetype
            == (DBT_DEVTYP_HANDLE) as u32
        {
            // DPRINT removed
            pBroadcastDeviceHandle = pNotificationFilter as PDEV_BROADCAST_HANDLE;

            if (ulNotificationFilterSize < core::mem::size_of::<DEV_BROADCAST_HANDLE>() as DWORD)
                || ((*pBroadcastDeviceHandle).dbch_size
                    < core::mem::size_of::<DEV_BROADCAST_HANDLE>() as DWORD)
            {
                ret = (CR_INVALID_DATA) as u32;
                break 'done;
            }

            if (ulFlags & (DEVICE_NOTIFY_ALL_INTERFACE_CLASSES) as u32) != 0 {
                ret = (CR_INVALID_FLAG) as u32;
                break 'done;
            }
        } else {
            // DPRINT removed
            ret = (CR_INVALID_DATA) as u32;
        }

        break;
    }

    if ret != (CR_SUCCESS) as u32 {
        if !pNotifyData.is_null() {
            if !(*pNotifyData).pszName.is_null() {
                RtlFreeHeap(
                    GetProcessHeap(),
                    0,
                    (*pNotifyData).pszName as LPVOID,
                );
            }

            RtlFreeHeap(
                GetProcessHeap(),
                0,
                pNotifyData as LPVOID,
            );
        }
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_UnregisterNotification(
    hBinding: handle_t,
    pNotifyHandle: *mut PNP_NOTIFY_HANDLE,
) -> DWORD {
    let mut pEntry: PNOTIFY_ENTRY;

    // DPRINT removed

    pEntry = (*pNotifyHandle) as PNOTIFY_ENTRY;
    if pEntry.is_null() {
        return (CR_INVALID_DATA) as u32;
    }

    RtlAcquireResourceExclusive(
        core::ptr::addr_of_mut!(NotificationListLock),
        (TRUE) as u8,
    );
    RemoveEntryList(core::ptr::addr_of_mut!((*pEntry).ListEntry));
    RtlReleaseResource(core::ptr::addr_of_mut!(NotificationListLock));

    if !(*pEntry).pszName.is_null() {
        RtlFreeHeap(RtlGetProcessHeap!(), 0, (*pEntry).pszName as PVOID);
    }
    RtlFreeHeap(RtlGetProcessHeap!(), 0, pEntry as PVOID);
    *pNotifyHandle = core::ptr::null_mut();

    (CR_SUCCESS) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetCustomDevProp(
    hBinding: handle_t,
    pDeviceID: LPWSTR,
    CustomPropName: LPWSTR,
    pulRegDataType: *mut DWORD,
    Buffer: *mut BYTE,
    pulTransferLen: *mut PNP_RPC_STRING_LEN,
    pulLength: *mut PNP_RPC_STRING_LEN,
    ulFlags: DWORD,
) -> DWORD {
    let mut hDeviceKey: HKEY = core::ptr::null_mut();
    let mut hParamKey: HKEY = core::ptr::null_mut();
    let mut lError: LONG = 0;
    let mut ret: CONFIGRET = (CR_SUCCESS) as u32;

    UNREFERENCED_PARAMETER!(hBinding);

    // DPRINT removed

    'done: loop {
        if pulTransferLen.is_null() || pulLength.is_null() {
            ret = (CR_INVALID_POINTER) as u32;
            break 'done;
        }

        if (ulFlags & (!CM_CUSTOMDEVPROP_BITS) as u32) != 0 {
            ret = (CR_INVALID_FLAG) as u32;
            break 'done;
        }

        if IsValidDeviceInstanceID(pDeviceID) == 0 {
            return (CR_INVALID_DEVINST) as u32;
        }

        if *pulLength < *pulTransferLen {
            *pulLength = *pulTransferLen;
        }

        *pulTransferLen = 0;

        lError = RegOpenKeyExW(
            hEnumKey,
            pDeviceID,
            0,
            KEY_READ,
            &mut hDeviceKey,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        lError = RegOpenKeyExW(
            hDeviceKey,
            w!("Device Parameters") as LPCWSTR,
            0,
            KEY_READ,
            &mut hParamKey,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            ret = (CR_REGISTRY_ERROR) as u32;
            break 'done;
        }

        lError = RegQueryValueExW(
            hParamKey,
            CustomPropName,
            core::ptr::null_mut(),
            pulRegDataType,
            Buffer,
            pulLength,
        );
        if lError != (ERROR_SUCCESS) as i32 {
            if lError == (ERROR_MORE_DATA) as i32 {
                ret = (CR_BUFFER_SMALL) as u32;
            } else {
                *pulLength = 0;
                ret = (CR_NO_SUCH_VALUE) as u32;
            }
        }

        break;
    }

    if ret == (CR_SUCCESS) as u32 {
        *pulTransferLen = *pulLength;
    }

    if !hParamKey.is_null() {
        RegCloseKey(hParamKey);
    }

    if !hDeviceKey.is_null() {
        RegCloseKey(hDeviceKey);
    }

    // DPRINT removed

    ret
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetVersionInternal(
    hBinding: handle_t,
    pwVersion: *mut WORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    *pwVersion = UMPNPMGR_WINVER;
    return (CR_SUCCESS) as u32;
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetBlockedDriverInfo(
    hBinding: handle_t,
    Buffer: *mut BYTE,
    pulTransferLen: *mut PNP_RPC_BUFFER_SIZE,
    pulLength: *mut PNP_RPC_BUFFER_SIZE,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetServerSideDeviceInstallFlags(
    hBinding: handle_t,
    pulSSDIFlags: *mut DWORD,
    ulFlags: DWORD,
) -> DWORD {
    UNREFERENCED_PARAMETER!(hBinding);

    if pulSSDIFlags.is_null() {
        return (CR_INVALID_POINTER) as u32;
    }

    if ulFlags != 0 {
        return (CR_INVALID_FLAG) as u32;
    }

    *pulSSDIFlags = 0;

    (CR_SUCCESS) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetObjectPropKeys(
    hBinding: handle_t,
    ObjectName: LPWSTR,
    ObjectType: DWORD,
    PropertyCultureName: LPWSTR,
    PropertyCount: *mut PNP_PROP_COUNT,
    TransferLen: *mut PNP_PROP_COUNT,
    PropertyKeys: *mut DEVPROPKEY,
    Flags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_GetObjectProp(
    hBinding: handle_t,
    ObjectName: LPWSTR,
    ObjectType: DWORD,
    PropertyCultureName: LPWSTR,
    PropertyKey: *const DEVPROPKEY,
    PropertyType: *mut DEVPROPTYPE,
    PropertySize: *mut PNP_PROP_SIZE,
    TransferLen: *mut PNP_PROP_SIZE,
    PropertyBuffer: *mut BYTE,
    Flags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetObjectProp(
    hBinding: handle_t,
    ObjectName: LPWSTR,
    ObjectType: DWORD,
    PropertyCultureName: LPWSTR,
    PropertyKey: *const DEVPROPKEY,
    PropertyType: DEVPROPTYPE,
    PropertySize: PNP_PROP_SIZE,
    PropertyBuffer: *mut BYTE,
    Flags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_InstallDevInst(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_ApplyPowerSettings(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DriverStoreAddDriverPackage(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DriverStoreDeleteDriverPackage(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_RegisterServiceNotification(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_SetActiveService(
    hBinding: handle_t,
    pszFilter: LPWSTR,
    ulFlags: DWORD,
) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}

#[no_mangle]
pub unsafe extern "system" fn PNP_DeleteServiceDevices(hBinding: handle_t) -> DWORD {
    // UNIMPLEMENTED;
    (CR_CALL_NOT_IMPLEMENTED) as u32
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
//  *  Copyright (C) 2005 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * COPYRIGHT:        See COPYING in the top level directory
//  * PROJECT:          ReactOS kernel
//  * FILE:             base/services/umpnpmgr/rpcserver.c
//  * PURPOSE:          RPC server
//  * PROGRAMMER:       Eric Kohl (eric.kohl@reactos.org)
//  *                   Hervé Poussineau (hpoussin@reactos.org)
//  *                   Colin Finck (colin@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS ******************************************************************/
//
// static WCHAR szRootDeviceInstanceID[] = L"HTREE\\ROOT\\0";
// LUID LoadDriverPrivilege = {SE_LOAD_DRIVER_PRIVILEGE, 0};
//
// LIST_ENTRY NotificationListHead;
// RTL_RESOURCE NotificationListLock;
//
// /* FUNCTIONS *****************************************************************/
//
// DWORD WINAPI
// RpcServerThread(LPVOID lpParameter)
// {
//     RPC_STATUS Status;
//     BOOLEAN RegisteredProtSeq = FALSE;
//
//     UNREFERENCED_PARAMETER!(lpParameter);
//
//     DPRINT("RpcServerThread() called\n");
//
//     InitializeListHead(&NotificationListHead);
//     RtlInitializeResource(&NotificationListLock);
//
//
//     /* Vista/7-compatible protocol sequence/endpoint */
//     Status = RpcServerUseProtseqEpW(L"ncacn_np",
//                                     20,
//                                     L"\\pipe\\plugplay",
//                                     NULL);  // Security descriptor
//     if (Status == RPC_S_OK)
//         RegisteredProtSeq = TRUE;
//     else
//         DPRINT1("RpcServerUseProtseqEpW() failed (Status %lx)\n", Status);
//
//     /* Make sure there's a usable endpoint */
//     if (RegisteredProtSeq == FALSE)
//         return 0;
//
//     Status = RpcServerRegisterIf(pnp_v1_0_s_ifspec,
//                                  NULL,
//                                  NULL);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT1("RpcServerRegisterIf() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     Status = RpcServerListen(1,
//                              20,
//                              FALSE);
//     if (Status != RPC_S_OK)
//     {
//         DPRINT1("RpcServerListen() failed (Status %lx)\n", Status);
//         return 0;
//     }
//
//     /* ROS HACK (this should never happen...) */
//     DPRINT1("*** Other devices won't be installed correctly. If something\n");
//     DPRINT1("*** doesn't work, try to reboot to get a new chance.\n");
//
//     DPRINT("RpcServerThread() done\n");
//
//     return 0;
// }
//
//
// void __RPC_FAR *  midl_user_allocate(SIZE_T len)
// {
//     return HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len);
// }
//
//
// void __RPC_USER midl_user_free(void __RPC_FAR * ptr)
// {
//     HeapFree(GetProcessHeap(), 0, ptr);
// }
//
//
// static CONFIGRET WINAPI
// NtStatusToCrError(NTSTATUS Status)
// {
//     switch (Status)
//     {
//         case STATUS_NOT_IMPLEMENTED:
//             return CR_CALL_NOT_IMPLEMENTED;
//
//         case STATUS_INVALID_PARAMETER:
//             return CR_INVALID_DATA;
//
//         case STATUS_NO_SUCH_DEVICE:
//             return CR_NO_SUCH_DEVINST;
//
//         case STATUS_ACCESS_DENIED:
//             return CR_ACCESS_DENIED;
//
//         case STATUS_BUFFER_TOO_SMALL:
//             return CR_BUFFER_SMALL;
//
//         case STATUS_OBJECT_NAME_NOT_FOUND:
//             return CR_NO_SUCH_VALUE;
//
//         default:
//             return CR_FAILURE;
//     }
// }
//
//
// static VOID
// SplitDeviceInstanceID( LPWSTR pszDeviceInstanceID,
//                        LPWSTR pszEnumerator,
//                        LPWSTR pszDevice,
//                        LPWSTR pszInstance)
// {
//     WCHAR szLocalDeviceInstanceID[MAX_DEVICE_ID_LEN];
//     LPWSTR lpEnumerator = NULL;
//     LPWSTR lpDevice = NULL;
//     LPWSTR lpInstance = NULL;
//     LPWSTR ptr;
//
//     wcscpy(szLocalDeviceInstanceID, pszDeviceInstanceID);
//
//     *pszEnumerator = 0;
//     *pszDevice = 0;
//     *pszInstance = 0;
//
//     lpEnumerator = szLocalDeviceInstanceID;
//
//     ptr = wcschr(lpEnumerator, L'\\');
//     if (ptr != NULL)
//     {
//         *ptr = 0;
//         lpDevice = ++ptr;
//
//         ptr = wcschr(lpDevice, L'\\');
//         if (ptr != NULL)
//         {
//             *ptr = 0;
//             lpInstance = ++ptr;
//         }
//     }
//
//     if (lpEnumerator != NULL)
//         wcscpy(pszEnumerator, lpEnumerator);
//
//     if (lpDevice != NULL)
//         wcscpy(pszDevice, lpDevice);
//
//     if (lpInstance != NULL)
//         wcscpy(pszInstance, lpInstance);
// }
//
//
// static
// CONFIGRET
// ClearDeviceStatus(
//      LPWSTR pszDeviceID,
//      DWORD ulStatus,
//      DWORD ulProblem)
// {
//     PLUGPLAY_CONTROL_STATUS_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT1("ClearDeviceStatus(%S 0x%lx 0x%lx)\n",
//             pszDeviceID, ulStatus, ulProblem);
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//     PlugPlayData.Operation = PNP_CLEAR_DEVICE_STATUS;
//     PlugPlayData.DeviceStatus = ulStatus;
//     PlugPlayData.DeviceProblem = ulProblem;
//
//     Status = NtPlugPlayControl(PlugPlayControlDeviceStatus,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_STATUS_DATA));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetDeviceStatus(
//      LPWSTR pszDeviceID,
//      DWORD *pulStatus,
//      DWORD *pulProblem)
// {
//     PLUGPLAY_CONTROL_STATUS_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT("GetDeviceStatus(%S %p %p)\n",
//            pszDeviceID, pulStatus, pulProblem);
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//     PlugPlayData.Operation = PNP_GET_DEVICE_STATUS;
//
//     Status = NtPlugPlayControl(PlugPlayControlDeviceStatus,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_STATUS_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulStatus = PlugPlayData.DeviceStatus;
//         *pulProblem = PlugPlayData.DeviceProblem;
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// SetDeviceStatus(
//      LPWSTR pszDeviceID,
//      DWORD ulStatus,
//      DWORD ulProblem)
// {
//     PLUGPLAY_CONTROL_STATUS_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT1("SetDeviceStatus(%S 0x%lx 0x%lx)\n",
//             pszDeviceID, ulStatus, ulProblem);
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//     PlugPlayData.Operation = PNP_SET_DEVICE_STATUS;
//     PlugPlayData.DeviceStatus = ulStatus;
//     PlugPlayData.DeviceProblem = ulProblem;
//
//     Status = NtPlugPlayControl(PlugPlayControlDeviceStatus,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_STATUS_DATA));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// DisableDeviceInstance(
//      LPWSTR pszDeviceInstance,
//      PPNP_VETO_TYPE pVetoType,
//      LPWSTR pszVetoName,
//      DWORD ulNameLength)
// {
//     PLUGPLAY_CONTROL_QUERY_REMOVE_DATA QueryRemoveData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT1("DisableDeviceInstance(%S %p %p %lu)\n",
//             pszDeviceInstance, pVetoType, pszVetoName, ulNameLength);
//
//     RtlInitUnicodeString(&QueryRemoveData.DeviceInstance,
//                          pszDeviceInstance);
//
//     QueryRemoveData.Flags = 0;
//     QueryRemoveData.VetoType = 0;
//     QueryRemoveData.VetoName = pszVetoName;
//     QueryRemoveData.NameLength = ulNameLength;
//
//     Status = NtPlugPlayControl(PlugPlayControlQueryAndRemoveDevice,
//                                &QueryRemoveData,
//                                sizeof(PLUGPLAY_CONTROL_QUERY_REMOVE_DATA));
//     if (Status == STATUS_NO_SUCH_DEVICE)
//     {
//         ret = CR_INVALID_DEVNODE;
//     }
//     else if (Status == STATUS_PLUGPLAY_QUERY_VETOED)
//     {
//         if (pVetoType != NULL)
//             *pVetoType = QueryRemoveData.VetoType;
//
//         ret = CR_REMOVE_VETOED;
//     }
//     else if (!NT_SUCCESS(Status))
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     return ret;
// }
//
//
// static
// BOOL
// IsValidDeviceInstanceID(
//      PWSTR pszDeviceInstanceID)
// {
//     INT nPartLength[3] = {0, 0, 0};
//     INT nLength = 0, nParts = 0;
//     PWCHAR p;
//
//     DPRINT("IsValidDeviceInstanceID(%S)\n",
//            pszDeviceInstanceID);
//
//     if (pszDeviceInstanceID == NULL)
//     {
//         DPRINT("Device instance ID is NULL!\n");
//         return FALSE;
//     }
//
//     p = pszDeviceInstanceID;
//     while (*p != UNICODE_NULL)
//     {
//         if (*p == L'\\')
//         {
//             nParts++;
//             if (nParts >= 3)
//             {
//                 DPRINT("Too many separators: %d\n", nParts);
//                 return FALSE;
//             }
//         }
//         else
//         {
//             nPartLength[nParts]++;
//         }
//
//         nLength++;
//         if (nLength >= MAX_DEVICE_ID_LEN)
//         {
//             DPRINT("Too long: %d\n", nLength);
//             return FALSE;
//         }
//
//         p++;
//     }
//
//     if (nParts != 2)
//     {
//         DPRINT("Invalid number of separtors: %d\n", nParts);
//         return FALSE;
//     }
//
//     if ((nPartLength[0] == 0) ||
//         (nPartLength[1] == 0) ||
//         (nPartLength[2] == 0))
//     {
//         DPRINT("Invalid part lengths: %d %d %d\n",
//                nPartLength[0], nPartLength[1], nPartLength[2]);
//         return FALSE;
//     }
//
//     DPRINT("Valid device instance ID!\n");
//
//     return TRUE;
// }
//
//
// static
// BOOL
// IsRootDeviceInstanceID(
//      PWSTR pszDeviceInstanceID)
// {
//     if (_wcsicmp(pszDeviceInstanceID, szRootDeviceInstanceID) == 0)
//         return TRUE;
//
//     return FALSE;
// }
//
//
// static
// BOOL
// IsPresentDeviceInstanceID(
//      LPWSTR pszDeviceInstanceID)
// {
//     DWORD ulStatus, ulProblem;
//
//     return (GetDeviceStatus(pszDeviceInstanceID, &ulStatus, &ulProblem) == CR_SUCCESS);
// }
//
//
// static
// CONFIGRET
// OpenConfigurationKey(
//      LPCWSTR pszDeviceID,
//      DWORD ulLogConfType,
//      PHKEY phKey)
// {
//     WCHAR szKeyName[MAX_PATH];
//     PCWSTR pszSubKeyName;
//     HKEY hInstanceKey;
//     DWORD dwError;
//
//     /* Build the full device instance key name */
//     wcscpy(szKeyName, L"System\\CurrentControlSet\\Enum\\");
//     wcscat(szKeyName, pszDeviceID);
//
//     /* Open the device instance key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             szKeyName,
//                             0,
//                             KEY_QUERY_VALUE | KEY_SET_VALUE | KEY_CREATE_SUB_KEY,
//                             &hInstanceKey);
//     if (dwError != ERROR_SUCCESS)
//         return CR_INVALID_DEVINST;
//
//     switch (ulLogConfType)
//     {
//         case BOOT_LOG_CONF:
//         case BASIC_LOG_CONF:
//             pszSubKeyName = L"LogConf";
//             break;
//
//         case ALLOC_LOG_CONF:
//         case FILTERED_LOG_CONF:
//             pszSubKeyName = L"Control";
//             break;
//
//         default:
//             DPRINT1("Unsupported configuration type!\n");
//             return CR_FAILURE;
//     }
//
//     /* Create or open the LogConf key */
//     dwError = RegCreateKeyExW(hInstanceKey,
//                               pszSubKeyName,
//                               0,
//                               NULL,
//                               REG_OPTION_NON_VOLATILE,
//                               KEY_ALL_ACCESS,
//                               NULL,
//                               phKey,
//                               NULL);
//
//     /* Close the device instance key */
//     RegCloseKey(hInstanceKey);
//
//     if (dwError != ERROR_SUCCESS)
//         return CR_REGISTRY_ERROR;
//
//     return CR_SUCCESS;
// }
//
//
// static
// CONFIGRET
// GetConfigurationData(
//      HKEY hKey,
//      ULONG ulLogConfType,
//      PULONG pulRegDataType,
//      PULONG pulDataSize,
//      LPBYTE *ppBuffer,
//      ULONG ulValueNameBufferSize,
//      LPWSTR pszValueNameBuffer)
// {
//     LPCWSTR pszValueName;
//
//     switch (ulLogConfType)
//     {
//         case BOOT_LOG_CONF:
//             pszValueName = L"BootConfig";
//             *pulRegDataType = REG_RESOURCE_LIST;
//             break;
//
//         case ALLOC_LOG_CONF:
//             pszValueName = L"AllocConfig";
//             *pulRegDataType = REG_RESOURCE_LIST;
//             break;
//
//         case FORCED_LOG_CONF:
//             pszValueName = L"ForcedConfig";
//             *pulRegDataType = REG_RESOURCE_LIST;
//             break;
//
//         case FILTERED_LOG_CONF:
//             pszValueName = L"FilteredConfigVector";
//             *pulRegDataType = REG_RESOURCE_REQUIREMENTS_LIST;
//             break;
//
//         case BASIC_LOG_CONF:
//             pszValueName = L"BasicConfigVector";
//             *pulRegDataType = REG_RESOURCE_REQUIREMENTS_LIST;
//             break;
//
//         case OVERRIDE_LOG_CONF:
//             pszValueName = L"OverrideConfigVector";
//             *pulRegDataType = REG_RESOURCE_REQUIREMENTS_LIST;
//             break;
//
//         default:
//             DPRINT1("Unsupported configuration type!\n");
//             return CR_FAILURE;
//     }
//
//     /* Return the selected configuration value name */
//     if ((ulValueNameBufferSize > 0) && (pszValueNameBuffer != NULL) &&
//         (wcslen(pszValueName) + 1 <= ulValueNameBufferSize))
//         wcscpy(pszValueNameBuffer, pszValueName);
//
//     /* Get the configuration data size */
//     if (RegQueryValueExW(hKey,
//                          pszValueName,
//                          NULL,
//                          NULL,
//                          NULL,
//                          pulDataSize) != ERROR_SUCCESS)
//     {
//         return CR_INVALID_LOG_CONF;
//     }
//
//     /* Allocate the buffer */
//     *ppBuffer = HeapAlloc(GetProcessHeap(), 0, *pulDataSize);
//     if (*ppBuffer == NULL)
//     {
//         return CR_OUT_OF_MEMORY;
//     }
//
//     /* Retrieve the configuration data */
//     if (RegQueryValueExW(hKey,
//                          pszValueName,
//                          NULL,
//                          NULL,
//                          (LPBYTE)*ppBuffer,
//                          pulDataSize) != ERROR_SUCCESS)
//     {
//         return CR_INVALID_LOG_CONF;
//     }
//
//     return CR_SUCCESS;
// }
//
//
// static
// PCM_FULL_RESOURCE_DESCRIPTOR
// NextResourceDescriptor(
//      PCM_FULL_RESOURCE_DESCRIPTOR pDescriptor)
// {
//     PBYTE pNext = NULL;
//     ULONG ulLastIndex = 0;
//
//     if (pDescriptor == NULL)
//         return NULL;
//
//     /* Skip the full resource descriptor */
//     pNext = (LPBYTE)pDescriptor + sizeof(CM_FULL_RESOURCE_DESCRIPTOR);
//
//     /* Skip the partial resource descriptors */
//     pNext += (pDescriptor->PartialResourceList.Count - 1) *
//          sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//
//     /* Skip the device specific data, if present */
//     if (pDescriptor->PartialResourceList.Count > 0)
//     {
//         ulLastIndex = pDescriptor->PartialResourceList.Count - 1;
//
//         if (pDescriptor->PartialResourceList.PartialDescriptors[ulLastIndex].Type ==
//             CmResourceTypeDeviceSpecific)
//         {
//             pNext += pDescriptor->PartialResourceList.PartialDescriptors[ulLastIndex].
//                      u.DeviceSpecificData.DataSize;
//         }
//     }
//
//     return (PCM_FULL_RESOURCE_DESCRIPTOR)pNext;
// }
//
//
// static
// PIO_RESOURCE_LIST
// NextResourceRequirement(
//      PIO_RESOURCE_LIST pResourceList)
// {
//     LPBYTE pNext = NULL;
//
//     if (pResourceList == NULL)
//         return NULL;
//
//     /* Skip the resource list */
//     pNext = (LPBYTE)pResourceList + sizeof(IO_RESOURCE_LIST);
//
//     /* Skip the resource descriptors */
//     pNext += (pResourceList->Count - 1) * sizeof(IO_RESOURCE_DESCRIPTOR);
//
//     return (PIO_RESOURCE_LIST)pNext;
// }
//
//
// static
// BOOL
// IsCallerInteractive(
//      handle_t hBinding)
// {
//     SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     HANDLE hToken;
//     PSID pInteractiveSid = NULL;
//     BOOL bInteractive = FALSE;
//     RPC_STATUS RpcStatus;
//
//     DPRINT("IsCallerInteractive(%p)\n", hBinding);
//
//     /* Allocate an interactive user sid */
//     if (!AllocateAndInitializeSid(&NtAuthority,
//                                   1,
//                                   SECURITY_INTERACTIVE_RID,
//                                   0, 0, 0, 0, 0, 0, 0,
//                                   &pInteractiveSid))
//     {
//         DPRINT1("AllocateAndInitializeSid failed\n");
//         return FALSE;
//     }
//
//     /* Impersonate the client */
//     RpcStatus = RpcImpersonateClient(hBinding);
//     if (RpcStatus != RPC_S_OK)
//     {
//         DPRINT1("RpcImpersonateClient failed (Status 0x%08lx)\n", RpcStatus);
//         goto done;
//     }
//
//     /* Open the thread token and check for interactive user membership */
//     if (OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, FALSE, &hToken))
//     {
//         CheckTokenMembership(hToken, pInteractiveSid, &bInteractive);
//         CloseHandle(hToken);
//     }
//
//     /* Revert the impersonation */
//     RpcRevertToSelf();
//
// done:
//     if (pInteractiveSid)
//         FreeSid(pInteractiveSid);
//
//     return bInteractive;
// }
//
//
// VOID
// __RPC_USER
// PNP_NOTIFY_HANDLE_rundown(
//     PNP_NOTIFY_HANDLE pHandle)
// {
//     DPRINT1("PNP_NOTIFY_HANDLE_rundown(%p)\n", pHandle);
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /* Function 0 */
// DWORD
// WINAPI
// PNP_Disconnect(
//     handle_t hBinding)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//     return CR_SUCCESS;
// }
//
//
// /* Function 1 */
// DWORD
// WINAPI
// PNP_Connect(
//     handle_t hBinding)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//     return CR_SUCCESS;
// }
//
//
// /* Function 2 */
// DWORD
// WINAPI
// PNP_GetVersion(
//     handle_t hBinding,
//     WORD *pVersion)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetVersion(%p %p)\n",
//            hBinding, pVersion);
//
//     *pVersion = CONFIGMG_VERSION;
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 3 */
// DWORD
// WINAPI
// PNP_GetGlobalState(
//     handle_t hBinding,
//     DWORD *pulState,
//     DWORD ulFlags)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetGlobalState(%p %p 0x%08lx)\n",
//            hBinding, pulState, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     *pulState = CM_GLOBAL_STATE_CAN_DO_UI | CM_GLOBAL_STATE_SERVICES_AVAILABLE;
//
//     if (g_ShuttingDown)
//         *pulState |= CM_GLOBAL_STATE_SHUTTING_DOWN;
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 4 */
// DWORD
// WINAPI
// PNP_InitDetection(
//     handle_t hBinding)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_InitDetection(%p)\n",
//            hBinding);
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 5 */
// DWORD
// WINAPI
// PNP_ReportLogOn(
//     handle_t hBinding,
//     BOOL Admin,
//     DWORD ProcessId)
// {
//     DWORD ReturnValue = CR_FAILURE;
//     HANDLE hProcess = NULL;
//
//     UNREFERENCED_PARAMETER!(Admin);
//
//     DPRINT("PNP_ReportLogOn(%p %u, %u)\n",
//            hBinding, Admin, ProcessId);
//
//     /* Fail, if the caller is not an interactive user */
//     if ((g_IsLiveMedium == FALSE) && (IsCallerInteractive(hBinding) == FALSE))
//         goto cleanup;
//
//     /* Get the users token */
//     hProcess = OpenProcess(PROCESS_ALL_ACCESS, TRUE, ProcessId);
//
//     if (!hProcess)
//     {
//         DPRINT1("OpenProcess failed with error %u\n", GetLastError());
//         goto cleanup;
//     }
//
//     if (hUserToken)
//     {
//         CloseHandle(hUserToken);
//         hUserToken = NULL;
//     }
//
//     if (!OpenProcessToken(hProcess, TOKEN_ASSIGN_PRIMARY | TOKEN_DUPLICATE | TOKEN_QUERY, &hUserToken))
//     {
//         DPRINT1("OpenProcessToken failed with error %u\n", GetLastError());
//         goto cleanup;
//     }
//
//     /* Trigger the installer thread */
//     if (hInstallEvent)
//         SetEvent(hInstallEvent);
//
//     ReturnValue = CR_SUCCESS;
//
// cleanup:
//     if (hProcess)
//         CloseHandle(hProcess);
//
//     return ReturnValue;
// }
//
//
// /* Function 6 */
// DWORD
// WINAPI
// PNP_ValidateDeviceInstance(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     HKEY hDeviceKey = NULL;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_ValidateDeviceInstance(%p %S 0x%08lx)\n",
//            hBinding, pDeviceID, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (RegOpenKeyExW(hEnumKey,
//                       pDeviceID,
//                       0,
//                       KEY_READ,
//                       &hDeviceKey))
//     {
//         DPRINT("Could not open the Device Key!\n");
//         ret = CR_NO_SUCH_DEVNODE;
//         goto Done;
//     }
//
//     /* FIXME: add more tests */
//
// Done:
//     if (hDeviceKey != NULL)
//         RegCloseKey(hDeviceKey);
//
//     DPRINT("PNP_ValidateDeviceInstance() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 7 */
// DWORD
// WINAPI
// PNP_GetRootDeviceInstance(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     PNP_RPC_STRING_LEN ulLength)
// {
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetRootDeviceInstance(%p %S %lu)\n",
//            hBinding, pDeviceID, ulLength);
//
//     if (!pDeviceID)
//     {
//         ret = CR_INVALID_POINTER;
//         goto Done;
//     }
//
//     if (ulLength < lstrlenW(szRootDeviceInstanceID) + 1)
//     {
//         ret = CR_BUFFER_SMALL;
//         goto Done;
//     }
//
//     lstrcpyW(pDeviceID,
//              szRootDeviceInstanceID);
//
// Done:
//     DPRINT("PNP_GetRootDeviceInstance() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 8 */
// DWORD
// WINAPI
// PNP_GetRelatedDeviceInstance(
//     handle_t hBinding,
//     DWORD ulRelationship,
//     LPWSTR pDeviceID,
//     LPWSTR pRelatedDeviceId,
//     PNP_RPC_STRING_LEN *pulLength,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_RELATED_DEVICE_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_GetRelatedDeviceInstance(%p %lu %S %p %p 0x%lx)\n",
//            hBinding, ulRelationship, pDeviceID, pRelatedDeviceId,
//            pulLength, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (ulRelationship == PNP_GET_PARENT_DEVICE_INSTANCE)
//     {
//         /* The root device does not have a parent */
//         if (IsRootDeviceInstanceID(pDeviceID))
//             return CR_NO_SUCH_DEVINST;
//
//         /* Return the root device for non existing devices */
//         if (!IsPresentDeviceInstanceID(pDeviceID))
//         {
//             if ((wcslen(szRootDeviceInstanceID) + 1) > *pulLength)
//             {
//                 *pulLength = wcslen(szRootDeviceInstanceID) + 1;
//                 return CR_BUFFER_SMALL;
//             }
//
//             wcscpy(pRelatedDeviceId, szRootDeviceInstanceID);
//             *pulLength = wcslen(szRootDeviceInstanceID) + 1;
//             return CR_SUCCESS;
//         }
//     }
//     else if (ulRelationship == PNP_GET_SIBLING_DEVICE_INSTANCE)
//     {
//         /* The root device does not have siblings */
//         if (IsRootDeviceInstanceID(pDeviceID))
//             return CR_NO_SUCH_DEVINST;
//     }
//
//     RtlInitUnicodeString(&PlugPlayData.TargetDeviceInstance,
//                          pDeviceID);
//
//     PlugPlayData.Relation = ulRelationship;
//
//     PlugPlayData.RelatedDeviceInstanceLength = *pulLength;
//     PlugPlayData.RelatedDeviceInstance = pRelatedDeviceId;
//
//     Status = NtPlugPlayControl(PlugPlayControlGetRelatedDevice,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_RELATED_DEVICE_DATA));
//     if (!NT_SUCCESS(Status))
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     DPRINT("PNP_GetRelatedDeviceInstance() done (returns %lx)\n", ret);
//     if (ret == CR_SUCCESS)
//     {
//         DPRINT("RelatedDevice: %wZ\n", &PlugPlayData.RelatedDeviceInstance);
//     }
//
//     return ret;
// }
//
//
// /* Function 9 */
// DWORD
// WINAPI
// PNP_EnumerateSubKeys(
//     handle_t hBinding,
//     DWORD ulBranch,
//     DWORD ulIndex,
//     LPWSTR Buffer,
//     PNP_RPC_STRING_LEN ulLength,
//     PNP_RPC_STRING_LEN *pulRequiredLen,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     HKEY hKey;
//     DWORD dwError;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_EnumerateSubKeys(%p %lu %lu %p %lu %p 0x%08lx)\n",
//            hBinding, ulBranch, ulIndex, Buffer, ulLength,
//            pulRequiredLen, ulFlags);
//
//     switch (ulBranch)
//     {
//         case PNP_ENUMERATOR_SUBKEYS:
//             hKey = hEnumKey;
//             break;
//
//         case PNP_CLASS_SUBKEYS:
//             hKey = hClassKey;
//             break;
//
//         default:
//             return CR_FAILURE;
//     }
//
//     *pulRequiredLen = ulLength;
//     dwError = RegEnumKeyExW(hKey,
//                             ulIndex,
//                             Buffer,
//                             pulRequiredLen,
//                             NULL,
//                             NULL,
//                             NULL,
//                             NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         ret = (dwError == ERROR_NO_MORE_ITEMS) ? CR_NO_SUCH_VALUE : CR_FAILURE;
//     }
//     else
//     {
//         (*pulRequiredLen)++;
//     }
//
//     DPRINT("PNP_EnumerateSubKeys() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetRelationsInstanceList(
//      PWSTR pszDevice,
//      DWORD ulFlags,
//      PWSTR pszBuffer,
//      PDWORD pulLength)
// {
//     PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA PlugPlayData;
//     NTSTATUS Status;
//     CONFIGRET ret = CR_SUCCESS;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDevice);
//
//     if (ulFlags & CM_GETIDLIST_FILTER_BUSRELATIONS)
//     {
//         PlugPlayData.Relations = 3;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_POWERRELATIONS)
//     {
//         PlugPlayData.Relations = 2;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_REMOVALRELATIONS)
//     {
//         PlugPlayData.Relations = 1;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_EJECTRELATIONS)
//     {
//         PlugPlayData.Relations = 0;
//     }
//
//     PlugPlayData.BufferSize = *pulLength * sizeof(WCHAR);
//     PlugPlayData.Buffer = pszBuffer;
//
//     Status = NtPlugPlayControl(PlugPlayControlQueryDeviceRelations,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLength = PlugPlayData.BufferSize / sizeof(WCHAR);
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetServiceInstanceList(
//      PWSTR pszService,
//      PWSTR pszBuffer,
//      PDWORD pulLength)
// {
//     WCHAR szPathBuffer[512];
//     WCHAR szName[16];
//     HKEY hServicesKey = NULL, hServiceKey = NULL, hEnumKey = NULL;
//     DWORD dwValues, dwSize, dwIndex, dwUsedLength, dwPathLength;
//     DWORD dwError;
//     PWSTR pPtr;
//     CONFIGRET ret = CR_SUCCESS;
//
//     /* Open the device key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_READ,
//                             &hServicesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the services key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     dwError = RegOpenKeyExW(hServicesKey,
//                             pszService,
//                             0,
//                             KEY_READ,
//                             &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the service key (Error %lu)\n", dwError);
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     dwError = RegOpenKeyExW(hServiceKey,
//                             L"Enum",
//                             0,
//                             KEY_READ,
//                             &hEnumKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the service enum key (Error %lu)\n", dwError);
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     /* Retrieve the number of device instances */
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hEnumKey,
//                                L"Count",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwValues,
//                                &dwSize);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("RegQueryValueExW failed (Error %lu)\n", dwError);
//         dwValues = 1;
//     }
//
//     DPRINT("dwValues %lu\n", dwValues);
//
//     dwUsedLength = 0;
//     pPtr = pszBuffer;
//
//     for (dwIndex = 0; dwIndex < dwValues; dwIndex++)
//     {
//         wsprintf(szName, L"%lu", dwIndex);
//
//         dwSize = sizeof(szPathBuffer);
//         dwError = RegQueryValueExW(hEnumKey,
//                                    szName,
//                                    NULL,
//                                    NULL,
//                                    (LPBYTE)szPathBuffer,
//                                    &dwSize);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         DPRINT("Path: %S\n", szPathBuffer);
//
//         dwPathLength = wcslen(szPathBuffer) + 1;
//         if (dwUsedLength + dwPathLength + 1 > *pulLength)
//         {
//             ret = CR_BUFFER_SMALL;
//             break;
//         }
//
//         wcscpy(pPtr, szPathBuffer);
//         dwUsedLength += dwPathLength;
//         pPtr += dwPathLength;
//
//         *pPtr = UNICODE_NULL;
//     }
//
// Done:
//     if (hEnumKey != NULL)
//         RegCloseKey(hEnumKey);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     if (hServicesKey != NULL)
//         RegCloseKey(hServicesKey);
//
//     if (ret == CR_SUCCESS)
//         *pulLength = dwUsedLength + 1;
//     else
//         *pulLength = 0;
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetDeviceInstanceList(
//      PWSTR pszDevice,
//      PWSTR pszBuffer,
//      PDWORD pulLength)
// {
//     WCHAR szInstanceBuffer[MAX_DEVICE_ID_LEN];
//     WCHAR szPathBuffer[512];
//     HKEY hDeviceKey;
//     DWORD dwInstanceLength, dwPathLength, dwUsedLength;
//     DWORD dwIndex, dwError;
//     PWSTR pPtr;
//     CONFIGRET ret = CR_SUCCESS;
//
//     /* Open the device key */
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszDevice,
//                             0,
//                             KEY_ENUMERATE_SUB_KEYS,
//                             &hDeviceKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the device key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     dwUsedLength = 0;
//     pPtr = pszBuffer;
//
//     for (dwIndex = 0; ; dwIndex++)
//     {
//         dwInstanceLength = MAX_DEVICE_ID_LEN;
//         dwError = RegEnumKeyExW(hDeviceKey,
//                                 dwIndex,
//                                 szInstanceBuffer,
//                                 &dwInstanceLength,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 NULL);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         wsprintf(szPathBuffer, L"%s\\%s", pszDevice, szInstanceBuffer);
//         DPRINT("Path: %S\n", szPathBuffer);
//
//         dwPathLength = wcslen(szPathBuffer) + 1;
//         if (dwUsedLength + dwPathLength + 1 > *pulLength)
//         {
//             ret = CR_BUFFER_SMALL;
//             break;
//         }
//
//         wcscpy(pPtr, szPathBuffer);
//         dwUsedLength += dwPathLength;
//         pPtr += dwPathLength;
//
//         *pPtr = UNICODE_NULL;
//     }
//
//     RegCloseKey(hDeviceKey);
//
//     if (ret == CR_SUCCESS)
//         *pulLength = dwUsedLength + 1;
//     else
//         *pulLength = 0;
//
//     return ret;
// }
//
//
// CONFIGRET
// GetEnumeratorInstanceList(
//      PWSTR pszEnumerator,
//      PWSTR pszBuffer,
//      PDWORD pulLength)
// {
//     WCHAR szDeviceBuffer[MAX_DEVICE_ID_LEN];
//     WCHAR szPathBuffer[512];
//     HKEY hEnumeratorKey;
//     PWSTR pPtr;
//     DWORD dwIndex, dwDeviceLength, dwUsedLength, dwRemainingLength, dwPathLength;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     /* Open the enumerator key */
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszEnumerator,
//                             0,
//                             KEY_ENUMERATE_SUB_KEYS,
//                             &hEnumeratorKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the enumerator key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     dwUsedLength = 0;
//     dwRemainingLength = *pulLength;
//     pPtr = pszBuffer;
//
//     for (dwIndex = 0; ; dwIndex++)
//     {
//         dwDeviceLength = MAX_DEVICE_ID_LEN;
//         dwError = RegEnumKeyExW(hEnumeratorKey,
//                                 dwIndex,
//                                 szDeviceBuffer,
//                                 &dwDeviceLength,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 NULL);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         wsprintf(szPathBuffer, L"%s\\%s", pszEnumerator, szDeviceBuffer);
//         DPRINT("Path: %S\n", szPathBuffer);
//
//         dwPathLength = dwRemainingLength;
//         ret = GetDeviceInstanceList(szPathBuffer,
//                                     pPtr,
//                                     &dwPathLength);
//         if (ret != CR_SUCCESS)
//             break;
//
//         dwUsedLength += dwPathLength - 1;
//         dwRemainingLength -= dwPathLength - 1;
//         pPtr += dwPathLength - 1;
//     }
//
//     RegCloseKey(hEnumeratorKey);
//
//     if (ret == CR_SUCCESS)
//         *pulLength = dwUsedLength + 1;
//     else
//         *pulLength = 0;
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetAllInstanceList(
//      PWSTR pszBuffer,
//      PDWORD pulLength)
// {
//     WCHAR szEnumeratorBuffer[MAX_DEVICE_ID_LEN];
//     PWSTR pPtr;
//     DWORD dwIndex, dwEnumeratorLength, dwUsedLength, dwRemainingLength, dwPathLength;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     dwUsedLength = 0;
//     dwRemainingLength = *pulLength;
//     pPtr = pszBuffer;
//
//     for (dwIndex = 0; ; dwIndex++)
//     {
//         dwEnumeratorLength = MAX_DEVICE_ID_LEN;
//         dwError = RegEnumKeyExW(hEnumKey,
//                                 dwIndex,
//                                 szEnumeratorBuffer,
//                                 &dwEnumeratorLength,
//                                 NULL, NULL, NULL, NULL);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         dwPathLength = dwRemainingLength;
//         ret = GetEnumeratorInstanceList(szEnumeratorBuffer,
//                                         pPtr,
//                                         &dwPathLength);
//         if (ret != CR_SUCCESS)
//             break;
//
//         dwUsedLength += dwPathLength - 1;
//         dwRemainingLength -= dwPathLength - 1;
//         pPtr += dwPathLength - 1;
//     }
//
//     if (ret == CR_SUCCESS)
//         *pulLength = dwUsedLength + 1;
//     else
//         *pulLength = 0;
//
//     return ret;
// }
//
//
// /* Function 10 */
// DWORD
// WINAPI
// PNP_GetDeviceList(
//     handle_t hBinding,
//     LPWSTR pszFilter,
//     LPWSTR Buffer,
//     PNP_RPC_STRING_LEN *pulLength,
//     DWORD ulFlags)
// {
//     WCHAR szEnumerator[MAX_DEVICE_ID_LEN];
//     WCHAR szDevice[MAX_DEVICE_ID_LEN];
//     WCHAR szInstance[MAX_DEVICE_ID_LEN];
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_GetDeviceList(%p %S %p %p 0x%08lx)\n",
//            hBinding, pszFilter, Buffer, pulLength, ulFlags);
//
//     if (ulFlags & ~CM_GETIDLIST_FILTER_BITS)
//         return CR_INVALID_FLAG;
//
//     if (pulLength == NULL)
//         return CR_INVALID_POINTER;
//
//     if ((ulFlags != CM_GETIDLIST_FILTER_NONE) &&
//         (pszFilter == NULL))
//         return CR_INVALID_POINTER;
//
//     if (ulFlags &
//         (CM_GETIDLIST_FILTER_BUSRELATIONS |
//          CM_GETIDLIST_FILTER_POWERRELATIONS |
//          CM_GETIDLIST_FILTER_REMOVALRELATIONS |
//          CM_GETIDLIST_FILTER_EJECTRELATIONS))
//     {
//         ret = GetRelationsInstanceList(pszFilter,
//                                        ulFlags,
//                                        Buffer,
//                                        pulLength);
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_SERVICE)
//     {
//         ret = GetServiceInstanceList(pszFilter,
//                                      Buffer,
//                                      pulLength);
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_ENUMERATOR)
//     {
//         SplitDeviceInstanceID(pszFilter,
//                               szEnumerator,
//                               szDevice,
//                               szInstance);
//
//         if (*szEnumerator != UNICODE_NULL && *szDevice != UNICODE_NULL)
//         {
//             ret = GetDeviceInstanceList(pszFilter,
//                                         Buffer,
//                                         pulLength);
//         }
//         else
//         {
//             ret = GetEnumeratorInstanceList(pszFilter,
//                                             Buffer,
//                                             pulLength);
//         }
//     }
//     else /* CM_GETIDLIST_FILTER_NONE */
//     {
//         ret = GetAllInstanceList(Buffer,
//                                  pulLength);
//     }
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetRelationsInstanceListSize(
//      PWSTR pszDevice,
//      DWORD ulFlags,
//      PDWORD pulLength)
// {
//     PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA PlugPlayData;
//     NTSTATUS Status;
//     CONFIGRET ret = CR_SUCCESS;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDevice);
//
//     if (ulFlags & CM_GETIDLIST_FILTER_BUSRELATIONS)
//     {
//         PlugPlayData.Relations = 3;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_POWERRELATIONS)
//     {
//         PlugPlayData.Relations = 2;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_REMOVALRELATIONS)
//     {
//         PlugPlayData.Relations = 1;
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_EJECTRELATIONS)
//     {
//         PlugPlayData.Relations = 0;
//     }
//
//     PlugPlayData.BufferSize = 0;
//     PlugPlayData.Buffer = NULL;
//
//     Status = NtPlugPlayControl(PlugPlayControlQueryDeviceRelations,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_DEVICE_RELATIONS_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLength = PlugPlayData.BufferSize / sizeof(WCHAR);
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetServiceInstanceListSize(
//      PWSTR pszService,
//      PDWORD pulLength)
// {
//     HKEY hServicesKey = NULL, hServiceKey = NULL, hEnumKey = NULL;
//     DWORD dwValues, dwMaxValueLength, dwSize;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     /* Open the device key */
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             L"System\\CurrentControlSet\\Services",
//                             0,
//                             KEY_READ,
//                             &hServicesKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the services key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     dwError = RegOpenKeyExW(hServicesKey,
//                             pszService,
//                             0,
//                             KEY_READ,
//                             &hServiceKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the service key (Error %lu)\n", dwError);
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     dwError = RegOpenKeyExW(hServiceKey,
//                             L"Enum",
//                             0,
//                             KEY_READ,
//                             &hEnumKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the service enum key (Error %lu)\n", dwError);
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     /* Retrieve the number of device instances */
//     dwSize = sizeof(DWORD);
//     dwError = RegQueryValueExW(hEnumKey,
//                                L"Count",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwValues,
//                                &dwSize);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("RegQueryValueExW failed (Error %lu)\n", dwError);
//         dwValues = 1;
//     }
//
//     /* Retrieve the maximum instance name length */
//     dwError = RegQueryInfoKeyW(hEnumKey,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &dwMaxValueLength,
//                                NULL,
//                                NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("RegQueryInfoKeyW failed (Error %lu)\n", dwError);
//         dwMaxValueLength = MAX_DEVICE_ID_LEN;
//     }
//
//     DPRINT("dwValues %lu  dwMaxValueLength %lu\n", dwValues, dwMaxValueLength / sizeof(WCHAR));
//
//     /* Return the largest possible buffer size */
//     *pulLength = dwValues * dwMaxValueLength / sizeof(WCHAR) + 2;
//
// Done:
//     if (hEnumKey != NULL)
//         RegCloseKey(hEnumKey);
//
//     if (hServiceKey != NULL)
//         RegCloseKey(hServiceKey);
//
//     if (hServicesKey != NULL)
//         RegCloseKey(hServicesKey);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetDeviceInstanceListSize(
//      LPCWSTR pszDevice,
//      PULONG pulLength)
// {
//     HKEY hDeviceKey;
//     DWORD dwSubKeys, dwMaxSubKeyLength;
//     DWORD dwError;
//
//     /* Open the device key */
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszDevice,
//                             0,
//                             KEY_READ,
//                             &hDeviceKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the device key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     /* Retrieve the number of device instances and the maximum name length */
//     dwError = RegQueryInfoKeyW(hDeviceKey,
//                                NULL,
//                                NULL,
//                                NULL,
//                                &dwSubKeys,
//                                &dwMaxSubKeyLength,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("RegQueryInfoKeyW failed (Error %lu)\n", dwError);
//         dwSubKeys = 0;
//         dwMaxSubKeyLength = 0;
//     }
//
//     /* Close the device key */
//     RegCloseKey(hDeviceKey);
//
//     /* Return the largest possible buffer size */
//     *pulLength = dwSubKeys * (wcslen(pszDevice) + 1 + dwMaxSubKeyLength + 1);
//
//     return CR_SUCCESS;
// }
//
//
// static
// CONFIGRET
// GetEnumeratorInstanceListSize(
//      LPCWSTR pszEnumerator,
//      PULONG pulLength)
// {
//     WCHAR szDeviceBuffer[MAX_DEVICE_ID_LEN];
//     WCHAR szPathBuffer[512];
//     HKEY hEnumeratorKey;
//     DWORD dwIndex, dwDeviceLength, dwBufferLength;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     *pulLength = 0;
//
//     /* Open the enumerator key */
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszEnumerator,
//                             0,
//                             KEY_ENUMERATE_SUB_KEYS,
//                             &hEnumeratorKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the enumerator key (Error %lu)\n", dwError);
//         return CR_REGISTRY_ERROR;
//     }
//
//     for (dwIndex = 0; ; dwIndex++)
//     {
//         dwDeviceLength = MAX_DEVICE_ID_LEN;
//         dwError = RegEnumKeyExW(hEnumeratorKey,
//                                 dwIndex,
//                                 szDeviceBuffer,
//                                 &dwDeviceLength,
//                                 NULL,
//                                 NULL,
//                                 NULL,
//                                 NULL);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         wsprintf(szPathBuffer, L"%s\\%s", pszEnumerator, szDeviceBuffer);
//         DPRINT("Path: %S\n", szPathBuffer);
//
//         ret = GetDeviceInstanceListSize(szPathBuffer, &dwBufferLength);
//         if (ret != CR_SUCCESS)
//         {
//             *pulLength = 0;
//             break;
//         }
//
//         *pulLength += dwBufferLength;
//     }
//
//     /* Close the enumerator key */
//     RegCloseKey(hEnumeratorKey);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// GetAllInstanceListSize(
//      PULONG pulLength)
// {
//     WCHAR szEnumeratorBuffer[MAX_DEVICE_ID_LEN];
//     DWORD dwIndex, dwEnumeratorLength, dwBufferLength;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     for (dwIndex = 0; ; dwIndex++)
//     {
//         dwEnumeratorLength = MAX_DEVICE_ID_LEN;
//         dwError = RegEnumKeyExW(hEnumKey,
//                                 dwIndex,
//                                 szEnumeratorBuffer,
//                                 &dwEnumeratorLength,
//                                 NULL, NULL, NULL, NULL);
//         if (dwError != ERROR_SUCCESS)
//             break;
//
//         /* Get the size of all device instances for the enumerator */
//         ret = GetEnumeratorInstanceListSize(szEnumeratorBuffer,
//                                             &dwBufferLength);
//         if (ret != CR_SUCCESS)
//             break;
//
//         *pulLength += dwBufferLength;
//     }
//
//     return ret;
// }
//
//
// /* Function 11 */
// DWORD
// WINAPI
// PNP_GetDeviceListSize(
//     handle_t hBinding,
//     LPWSTR pszFilter,
//     PNP_RPC_BUFFER_SIZE *pulLength,
//     DWORD ulFlags)
// {
//     WCHAR szEnumerator[MAX_DEVICE_ID_LEN];
//     WCHAR szDevice[MAX_DEVICE_ID_LEN];
//     WCHAR szInstance[MAX_DEVICE_ID_LEN];
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_GetDeviceListSize(%p %S %p 0x%08lx)\n",
//            hBinding, pszFilter, pulLength, ulFlags);
//
//     if (ulFlags & ~CM_GETIDLIST_FILTER_BITS)
//         return CR_INVALID_FLAG;
//
//     if (pulLength == NULL)
//         return CR_INVALID_POINTER;
//
//     if ((ulFlags != CM_GETIDLIST_FILTER_NONE) &&
//         (pszFilter == NULL))
//         return CR_INVALID_POINTER;
//
//     *pulLength = 0;
//
//     if (ulFlags &
//         (CM_GETIDLIST_FILTER_BUSRELATIONS |
//          CM_GETIDLIST_FILTER_POWERRELATIONS |
//          CM_GETIDLIST_FILTER_REMOVALRELATIONS |
//          CM_GETIDLIST_FILTER_EJECTRELATIONS))
//     {
//         ret = GetRelationsInstanceListSize(pszFilter,
//                                            ulFlags,
//                                            pulLength);
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_SERVICE)
//     {
//         ret = GetServiceInstanceListSize(pszFilter,
//                                          pulLength);
//     }
//     else if (ulFlags & CM_GETIDLIST_FILTER_ENUMERATOR)
//     {
//         SplitDeviceInstanceID(pszFilter,
//                               szEnumerator,
//                               szDevice,
//                               szInstance);
//
//         if (*szEnumerator != UNICODE_NULL && *szDevice != UNICODE_NULL)
//         {
//             ret = GetDeviceInstanceListSize(pszFilter,
//                                             pulLength);
//         }
//         else
//         {
//             ret = GetEnumeratorInstanceListSize(pszFilter,
//                                                 pulLength);
//         }
//     }
//     else /* CM_GETIDLIST_FILTER_NONE */
//     {
//         ret = GetAllInstanceListSize(pulLength);
//     }
//
//     /* Add one character for the terminating double UNICODE_NULL */
//     if (ret == CR_SUCCESS)
//         (*pulLength) += 1;
//
//     return ret;
// }
//
//
// /* Function 12 */
// DWORD
// WINAPI
// PNP_GetDepth(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     DWORD *pulDepth,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_DEPTH_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_GetDepth(%p %S %p 0x%08lx)\n",
//            hBinding, pszDeviceID, pulDepth, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//
//     Status = NtPlugPlayControl(PlugPlayControlGetDeviceDepth,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_DEPTH_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulDepth = PlugPlayData.Depth;
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     DPRINT("PNP_GetDepth() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 13 */
// DWORD
// WINAPI
// PNP_GetDeviceRegProp(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulProperty,
//     DWORD *pulRegDataType,
//     BYTE *Buffer,
//     PNP_PROP_SIZE *pulTransferLen,
//     PNP_PROP_SIZE *pulLength,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_PROPERTY_DATA PlugPlayData;
//     CONFIGRET ret = CR_SUCCESS;
//     LPWSTR lpValueName = NULL;
//     HKEY hKey = NULL;
//     LONG lError;
//     NTSTATUS Status;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetDeviceRegProp(%p %S %lu %p %p %p %p 0x%08lx)\n",
//            hBinding, pDeviceID, ulProperty, pulRegDataType, Buffer,
//            pulTransferLen, pulLength, ulFlags);
//
//     if (pulTransferLen == NULL || pulLength == NULL)
//     {
//         ret = CR_INVALID_POINTER;
//         goto done;
//     }
//
//     if (ulFlags != 0)
//     {
//         ret = CR_INVALID_FLAG;
//         goto done;
//     }
//
//     /* Check pDeviceID */
//     if (!IsValidDeviceInstanceID(pDeviceID))
//     {
//         ret = CR_INVALID_DEVINST;
//         goto done;
//     }
//
//     if (*pulLength < *pulTransferLen)
//         *pulLength = *pulTransferLen;
//
//     *pulTransferLen = 0;
//
//     switch (ulProperty)
//     {
//         case CM_DRP_DEVICEDESC:
//             lpValueName = L"DeviceDesc";
//             break;
//
//         case CM_DRP_HARDWAREID:
//             lpValueName = L"HardwareID";
//             break;
//
//         case CM_DRP_COMPATIBLEIDS:
//             lpValueName = L"CompatibleIDs";
//             break;
//
//         case CM_DRP_SERVICE:
//             lpValueName = L"Service";
//             break;
//
//         case CM_DRP_CLASS:
//             lpValueName = L"Class";
//             break;
//
//         case CM_DRP_CLASSGUID:
//             lpValueName = L"ClassGUID";
//             break;
//
//         case CM_DRP_DRIVER:
//             lpValueName = L"Driver";
//             break;
//
//         case CM_DRP_CONFIGFLAGS:
//             lpValueName = L"ConfigFlags";
//             break;
//
//         case CM_DRP_MFG:
//             lpValueName = L"Mfg";
//             break;
//
//         case CM_DRP_FRIENDLYNAME:
//             lpValueName = L"FriendlyName";
//             break;
//
//         case CM_DRP_LOCATION_INFORMATION:
//             lpValueName = L"LocationInformation";
//             break;
//
//         case CM_DRP_PHYSICAL_DEVICE_OBJECT_NAME:
//             PlugPlayData.Property = PNP_PROPERTY_PHYSICAL_DEVICE_OBJECT_NAME;
//             *pulRegDataType = REG_SZ;
//             break;
//
//         case CM_DRP_CAPABILITIES:
//             lpValueName = L"Capabilities";
//             break;
//
//         case CM_DRP_UI_NUMBER:
//             PlugPlayData.Property = PNP_PROPERTY_UI_NUMBER;
//             break;
//
//         case CM_DRP_UPPERFILTERS:
//             lpValueName = L"UpperFilters";
//             break;
//
//         case CM_DRP_LOWERFILTERS:
//             lpValueName = L"LowerFilters";
//             break;
//
//         case CM_DRP_BUSTYPEGUID:
//             PlugPlayData.Property = PNP_PROPERTY_BUSTYPEGUID;
//             *pulRegDataType = REG_BINARY;
//             break;
//
//         case CM_DRP_LEGACYBUSTYPE:
//             PlugPlayData.Property = PNP_PROPERTY_LEGACYBUSTYPE;
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_BUSNUMBER:
//             PlugPlayData.Property = PNP_PROPERTY_BUSNUMBER;
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_ENUMERATOR_NAME:
//             PlugPlayData.Property = PNP_PROPERTY_ENUMERATOR_NAME;
//             *pulRegDataType = REG_SZ;
//             break;
//
//         case CM_DRP_SECURITY:
//             lpValueName = L"Security";
//             break;
//
//         case CM_DRP_DEVTYPE:
//             lpValueName = L"DeviceType";
//             break;
//
//         case CM_DRP_EXCLUSIVE:
//             lpValueName = L"Exclusive";
//             break;
//
//         case CM_DRP_CHARACTERISTICS:
//             lpValueName = L"DeviceCharacteristics";
//             break;
//
//         case CM_DRP_ADDRESS:
//             PlugPlayData.Property = PNP_PROPERTY_ADDRESS;
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_UI_NUMBER_DESC_FORMAT:
//             lpValueName = L"UINumberDescFormat";
//             break;
//
//         case CM_DRP_DEVICE_POWER_DATA:
//             PlugPlayData.Property = PNP_PROPERTY_POWER_DATA;
//             *pulRegDataType = REG_BINARY;
//             break;
//
//         case CM_DRP_REMOVAL_POLICY:
//             PlugPlayData.Property = PNP_PROPERTY_REMOVAL_POLICY;
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_REMOVAL_POLICY_HW_DEFAULT:
//             PlugPlayData.Property = PNP_PROPERTY_REMOVAL_POLICY_HARDWARE_DEFAULT;
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_REMOVAL_POLICY_OVERRIDE:
//             lpValueName = L"RemovalPolicy";
//             *pulRegDataType = REG_DWORD;
//             break;
//
//         case CM_DRP_INSTALL_STATE:
//             PlugPlayData.Property = PNP_PROPERTY_INSTALL_STATE;
//             *pulRegDataType = REG_DWORD;
//             break;
//
// #if (WINVER >= _WIN32_WINNT_WS03)
//         case CM_DRP_LOCATION_PATHS:
//             PlugPlayData.Property = PNP_PROPERTY_LOCATION_PATHS;
//             *pulRegDataType = REG_MULTI_SZ;
//             break;
// #endif
//
// #if (WINVER >= _WIN32_WINNT_WIN7)
//         case CM_DRP_BASE_CONTAINERID:
//             PlugPlayData.Property = PNP_PROPERTY_CONTAINERID;
//             *pulRegDataType = REG_SZ;
//             break;
// #endif
//
//         default:
//             ret = CR_INVALID_PROPERTY;
//             goto done;
//     }
//
//     DPRINT("Value name: %S\n", lpValueName);
//
//     if (lpValueName)
//     {
//         /* Retrieve information from the Registry */
//         lError = RegOpenKeyExW(hEnumKey,
//                                pDeviceID,
//                                0,
//                                KEY_QUERY_VALUE,
//                                &hKey);
//         if (lError != ERROR_SUCCESS)
//         {
//             hKey = NULL;
//             *pulLength = 0;
//             ret = CR_INVALID_DEVNODE;
//             goto done;
//         }
//
//         lError = RegQueryValueExW(hKey,
//                                   lpValueName,
//                                   NULL,
//                                   pulRegDataType,
//                                   Buffer,
//                                   pulLength);
//         if (lError != ERROR_SUCCESS)
//         {
//             if (lError == ERROR_MORE_DATA)
//             {
//                 ret = CR_BUFFER_SMALL;
//             }
//             else
//             {
//                 *pulLength = 0;
//                 ret = CR_NO_SUCH_VALUE;
//             }
//         }
//     }
//     else
//     {
//         /* Retrieve information from the Device Node */
//         RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                              pDeviceID);
//         PlugPlayData.Buffer = Buffer;
//         PlugPlayData.BufferSize = *pulLength;
//
//         Status = NtPlugPlayControl(PlugPlayControlProperty,
//                                    (PVOID)&PlugPlayData,
//                                    sizeof(PLUGPLAY_CONTROL_PROPERTY_DATA));
//         if (NT_SUCCESS(Status))
//         {
//             *pulLength = PlugPlayData.BufferSize;
//         }
//         else
//         {
//             ret = NtStatusToCrError(Status);
//         }
//     }
//
// done:
//     if (pulTransferLen)
//         *pulTransferLen = (ret == CR_SUCCESS) ? *pulLength : 0;
//
//     if (hKey != NULL)
//         RegCloseKey(hKey);
//
//     DPRINT("PNP_GetDeviceRegProp() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 14 */
// DWORD
// WINAPI
// PNP_SetDeviceRegProp(
//     handle_t hBinding,
//     LPWSTR pDeviceId,
//     DWORD ulProperty,
//     DWORD ulDataType,
//     BYTE *Buffer,
//     PNP_PROP_SIZE ulLength,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     LPWSTR lpValueName = NULL;
//     HKEY hKey = 0;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_SetDeviceRegProp(%p %S %lu %lu %p %lu 0x%08lx)\n",
//            hBinding, pDeviceId, ulProperty, ulDataType, Buffer,
//            ulLength, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pDeviceId))
//         return CR_INVALID_DEVINST;
//
//     switch (ulProperty)
//     {
//         case CM_DRP_DEVICEDESC:
//             lpValueName = L"DeviceDesc";
//             break;
//
//         case CM_DRP_HARDWAREID:
//             lpValueName = L"HardwareID";
//             break;
//
//         case CM_DRP_COMPATIBLEIDS:
//             lpValueName = L"CompatibleIDs";
//             break;
//
//         case CM_DRP_SERVICE:
//             lpValueName = L"Service";
//             break;
//
//         case CM_DRP_CLASS:
//             lpValueName = L"Class";
//             break;
//
//         case CM_DRP_CLASSGUID:
//             lpValueName = L"ClassGUID";
//             break;
//
//         case CM_DRP_DRIVER:
//             lpValueName = L"Driver";
//             break;
//
//         case CM_DRP_CONFIGFLAGS:
//             lpValueName = L"ConfigFlags";
//             break;
//
//         case CM_DRP_MFG:
//             lpValueName = L"Mfg";
//             break;
//
//         case CM_DRP_FRIENDLYNAME:
//             lpValueName = L"FriendlyName";
//             break;
//
//         case CM_DRP_LOCATION_INFORMATION:
//             lpValueName = L"LocationInformation";
//             break;
//
//         case CM_DRP_UPPERFILTERS:
//             lpValueName = L"UpperFilters";
//             break;
//
//         case CM_DRP_LOWERFILTERS:
//             lpValueName = L"LowerFilters";
//             break;
//
//         case CM_DRP_SECURITY:
//             lpValueName = L"Security";
//             break;
//
//         case CM_DRP_DEVTYPE:
//             lpValueName = L"DeviceType";
//             break;
//
//         case CM_DRP_EXCLUSIVE:
//             lpValueName = L"Exclusive";
//             break;
//
//         case CM_DRP_CHARACTERISTICS:
//             lpValueName = L"DeviceCharacteristics";
//             break;
//
//         case CM_DRP_UI_NUMBER_DESC_FORMAT:
//             lpValueName = L"UINumberDescFormat";
//             break;
//
//         case CM_DRP_REMOVAL_POLICY_OVERRIDE:
//             lpValueName = L"RemovalPolicy";
//             break;
//
//         default:
//             return CR_INVALID_PROPERTY;
//     }
//
//     DPRINT("Value name: %S\n", lpValueName);
//
//     if (RegOpenKeyExW(hEnumKey,
//                       pDeviceId,
//                       0,
//                       KEY_SET_VALUE,
//                       &hKey))
//         return CR_INVALID_DEVNODE;
//
//     if (ulLength == 0)
//     {
//         if (RegDeleteValueW(hKey,
//                             lpValueName))
//             ret = CR_REGISTRY_ERROR;
//     }
//     else
//     {
//         if (RegSetValueExW(hKey,
//                            lpValueName,
//                            0,
//                            ulDataType,
//                            Buffer,
//                            ulLength))
//             ret = CR_REGISTRY_ERROR;
//     }
//
//     RegCloseKey(hKey);
//
//     DPRINT("PNP_SetDeviceRegProp() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 15 */
// DWORD
// WINAPI
// PNP_GetClassInstance(
//     handle_t hBinding,
//     LPWSTR pDeviceId,
//     LPWSTR pszClassInstance,
//     PNP_RPC_STRING_LEN ulLength)
// {
//     WCHAR szClassGuid[40];
//     WCHAR szClassInstance[5];
//     HKEY hDeviceClassKey = NULL;
//     HKEY hClassInstanceKey;
//     ULONG ulTransferLength, ulDataLength;
//     DWORD dwDataType, dwDisposition, i;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_GetClassInstance(%p %S %p %lu)\n",
//            hBinding, pDeviceId, pszClassInstance, ulLength);
//
//     if (!IsValidDeviceInstanceID(pDeviceId))
//         return CR_INVALID_DEVINST;
//
//     ulTransferLength = ulLength;
//     ret = PNP_GetDeviceRegProp(hBinding,
//                                pDeviceId,
//                                CM_DRP_DRIVER,
//                                &dwDataType,
//                                (BYTE *)pszClassInstance,
//                                &ulTransferLength,
//                                &ulLength,
//                                0);
//     if (ret == CR_SUCCESS)
//         return ret;
//
//     ulTransferLength = sizeof(szClassGuid);
//     ulDataLength = sizeof(szClassGuid);
//     ret = PNP_GetDeviceRegProp(hBinding,
//                                pDeviceId,
//                                CM_DRP_CLASSGUID,
//                                &dwDataType,
//                                (BYTE *)szClassGuid,
//                                &ulTransferLength,
//                                &ulDataLength,
//                                0);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("PNP_GetDeviceRegProp() failed (Error %lu)\n", ret);
//         goto done;
//     }
//
//     dwError = RegOpenKeyExW(hClassKey,
//                             szClassGuid,
//                             0,
//                             KEY_READ,
//                             &hDeviceClassKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         DPRINT1("RegOpenKeyExW() failed (Error %lu)\n", dwError);
//         ret = CR_FAILURE;
//         goto done;
//     }
//
//     for (i = 0; i < 10000; i++)
//     {
//         wsprintf(szClassInstance, L"%04lu", i);
//
//         dwError = RegCreateKeyExW(hDeviceClassKey,
//                                   szClassInstance,
//                                   0,
//                                   NULL,
//                                   REG_OPTION_NON_VOLATILE,
//                                   KEY_ALL_ACCESS,
//                                   NULL,
//                                   &hClassInstanceKey,
//                                   &dwDisposition);
//         if (dwError == ERROR_SUCCESS)
//         {
//             RegCloseKey(hClassInstanceKey);
//
//             if (dwDisposition == REG_CREATED_NEW_KEY)
//             {
//                 wsprintf(pszClassInstance,
//                          L"%s\\%s",
//                          szClassGuid,
//                          szClassInstance);
//
//                 ulDataLength = (wcslen(pszClassInstance) + 1) * sizeof(WCHAR);
//                 ret = PNP_SetDeviceRegProp(hBinding,
//                                            pDeviceId,
//                                            CM_DRP_DRIVER,
//                                            REG_SZ,
//                                            (BYTE *)pszClassInstance,
//                                            ulDataLength,
//                                            0);
//                 if (ret != CR_SUCCESS)
//                 {
//                     DPRINT1("PNP_SetDeviceRegProp() failed (Error %lu)\n", ret);
//                     RegDeleteKeyW(hDeviceClassKey,
//                                   szClassInstance);
//                 }
//
//                 break;
//             }
//         }
//     }
//
// done:
//     if (hDeviceClassKey != NULL)
//         RegCloseKey(hDeviceClassKey);
//
//     return ret;
// }
//
//
// /* Function 16 */
// DWORD
// WINAPI
// PNP_CreateKey(
//     handle_t hBinding,
//     LPWSTR pszSubKey,
//     DWORD samDesired,
//     DWORD ulFlags)
// {
//     HKEY hDeviceKey = NULL, hParametersKey = NULL;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(samDesired);
//
//     DPRINT("PNP_CreateKey(%p %S 0x%lx 0x%08lx)\n",
//            hBinding, pszSubKey, samDesired, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pszSubKey))
//         return CR_INVALID_DEVINST;
//
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszSubKey,
//                             0,
//                             KEY_WRITE,
//                             &hDeviceKey);
//     if (dwError != ERROR_SUCCESS)
//     {
//         ret = CR_INVALID_DEVNODE;
//         goto done;
//     }
//
//     dwError = RegCreateKeyExW(hDeviceKey,
//                               L"Device Parameters",
//                               0,
//                               NULL,
//                               REG_OPTION_NON_VOLATILE,
//                               KEY_ALL_ACCESS,
//                               NULL,
//                               &hParametersKey,
//                               NULL);
//     if (dwError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     /* FIXME: Set key security */
//
// done:
//     if (hParametersKey != NULL)
//         RegCloseKey(hParametersKey);
//
//     if (hDeviceKey != NULL)
//         RegCloseKey(hDeviceKey);
//
//     return ret;
// }
//
//
// /* Function 17 */
// DWORD
// WINAPI
// PNP_DeleteRegistryKey(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     LPWSTR pszParentKey,
//     LPWSTR pszChildKey,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 18 */
// DWORD
// WINAPI
// PNP_GetClassCount(
//     handle_t hBinding,
//     DWORD *pulClassCount,
//     DWORD ulFlags)
// {
//     HKEY hKey;
//     DWORD dwError;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_GetClassCount(%p %p 0x%08lx)\n",
//            hBinding, pulClassCount, ulFlags);
//
//     dwError = RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                             REGSTR_PATH_CLASS!(),
//                             0,
//                             KEY_QUERY_VALUE,
//                             &hKey);
//     if (dwError != ERROR_SUCCESS)
//         return CR_INVALID_DATA;
//
//     dwError = RegQueryInfoKeyW(hKey,
//                                NULL,
//                                NULL,
//                                NULL,
//                                pulClassCount,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL,
//                                NULL);
//     RegCloseKey(hKey);
//     if (dwError != ERROR_SUCCESS)
//         return CR_INVALID_DATA;
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 19 */
// DWORD
// WINAPI
// PNP_GetClassName(
//     handle_t hBinding,
//     LPWSTR pszClassGuid,
//     LPWSTR Buffer,
//     PNP_RPC_STRING_LEN *pulLength,
//     DWORD ulFlags)
// {
//     WCHAR szKeyName[MAX_PATH];
//     CONFIGRET ret = CR_SUCCESS;
//     HKEY hKey;
//     DWORD dwSize;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_GetClassName(%p %S %p %p 0x%08lx)\n",
//            hBinding, pszClassGuid, Buffer, pulLength, ulFlags);
//
//     lstrcpyW(szKeyName, L"System\\CurrentControlSet\\Control\\Class\\");
//     if (lstrlenW(pszClassGuid) + 1 < sizeof(szKeyName)/sizeof(WCHAR)-(lstrlenW(szKeyName) * sizeof(WCHAR)))
//         lstrcatW(szKeyName, pszClassGuid);
//     else
//         return CR_INVALID_DATA;
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       szKeyName,
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hKey))
//         return CR_REGISTRY_ERROR;
//
//     dwSize = *pulLength * sizeof(WCHAR);
//     if (RegQueryValueExW(hKey,
//                          L"Class",
//                          NULL,
//                          NULL,
//                          (LPBYTE)Buffer,
//                          &dwSize))
//     {
//         *pulLength = 0;
//         ret = CR_REGISTRY_ERROR;
//     }
//     else
//     {
//         *pulLength = dwSize / sizeof(WCHAR);
//     }
//
//     RegCloseKey(hKey);
//
//     DPRINT("PNP_GetClassName() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 20 */
// DWORD
// WINAPI
// PNP_DeleteClassKey(
//     handle_t hBinding,
//     LPWSTR pszClassGuid,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_DeleteClassKey(%p %S 0x%08lx)\n",
//            hBinding, pszClassGuid, ulFlags);
//
//     if (ulFlags & CM_DELETE_CLASS_SUBKEYS)
//     {
//         if (SHDeleteKeyW(hClassKey, pszClassGuid) != ERROR_SUCCESS)
//             ret = CR_REGISTRY_ERROR;
//     }
//     else
//     {
//         if (RegDeleteKeyW(hClassKey, pszClassGuid) != ERROR_SUCCESS)
//             ret = CR_REGISTRY_ERROR;
//     }
//
//     DPRINT("PNP_DeleteClassKey() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 21 */
// DWORD
// WINAPI
// PNP_GetInterfaceDeviceAlias(
//     handle_t hBinding,
//     LPWSTR pszInterfaceDevice,
//     GUID *AliasInterfaceGuid,
//     LPWSTR pszAliasInterfaceDevice,
//     PNP_RPC_STRING_LEN *pulLength,
//     PNP_RPC_STRING_LEN *pulTransferLen,
//     DWORD ulFlags)
// {
//     NTSTATUS Status;
//     PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA PlugPlayData;
//     DWORD ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetInterfaceDeviceAlias(%p %S %p %p %p %p 0x%08lx)\n",
//            hBinding, pszInterfaceDevice, AliasInterfaceGuid, pszAliasInterfaceDevice, pulLength,
//            pulTransferLen, ulFlags);
//
//     if ((AliasInterfaceGuid == NULL) ||
//         (pulLength == NULL))
//         return  CR_INVALID_POINTER;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     RtlInitUnicodeString(&PlugPlayData.SymbolicLinkName, pszInterfaceDevice);
//     PlugPlayData.AliasInterfaceClassGuid = AliasInterfaceGuid;
//     PlugPlayData.AliasSymbolicLinkName = pszAliasInterfaceDevice;
//     PlugPlayData.AliasSymbolicLinkNameLength = *pulTransferLen;
//
//     Status = NtPlugPlayControl(PlugPlayControlGetInterfaceDeviceAlias,
//                                &PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_INTERFACE_ALIAS_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLength = PlugPlayData.AliasSymbolicLinkNameLength;
//         *pulTransferLen = *pulLength + 1;
//     }
//     else
//     {
//         *pulLength = 0;
//         *pulTransferLen = 0;
//         ret = NtStatusToCrError(Status);
//     }
//
//     DPRINT("PNP_GetInterfaceDeviceAlias() done (returns %lx)\n", ret);
//     return ret;
// }
//
//
// /* Function 22 */
// DWORD
// WINAPI
// PNP_GetInterfaceDeviceList(
//     handle_t hBinding,
//     GUID *InterfaceGuid,
//     LPWSTR pszDeviceID,
//     BYTE *Buffer,
//     PNP_RPC_BUFFER_SIZE *pulLength,
//     DWORD ulFlags)
// {
//     NTSTATUS Status;
//     PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA PlugPlayData;
//     DWORD ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetInterfaceDeviceList(%p %p %S %p %p 0x%08lx)\n",
//            hBinding, InterfaceGuid, pszDeviceID, Buffer, pulLength, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//
//     PlugPlayData.Flags = ulFlags;
//     PlugPlayData.FilterGuid = InterfaceGuid;
//     PlugPlayData.Buffer = Buffer;
//     PlugPlayData.BufferSize = *pulLength;
//
//     Status = NtPlugPlayControl(PlugPlayControlGetInterfaceDeviceList,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLength = PlugPlayData.BufferSize;
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     DPRINT("PNP_GetInterfaceDeviceList() done (returns %lx)\n", ret);
//     return ret;
// }
//
//
// /* Function 23 */
// DWORD
// WINAPI
// PNP_GetInterfaceDeviceListSize(
//     handle_t hBinding,
//     PNP_RPC_BUFFER_SIZE *pulLen,
//     GUID *InterfaceGuid,
//     LPWSTR pszDeviceID,
//     DWORD ulFlags)
// {
//     NTSTATUS Status;
//     PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA PlugPlayData;
//     DWORD ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetInterfaceDeviceListSize(%p %p %p %S 0x%08lx)\n",
//            hBinding, pulLen, InterfaceGuid, pszDeviceID, ulFlags);
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//
//     PlugPlayData.FilterGuid = InterfaceGuid;
//     PlugPlayData.Buffer = NULL;
//     PlugPlayData.BufferSize = 0;
//     PlugPlayData.Flags = ulFlags;
//
//     Status = NtPlugPlayControl(PlugPlayControlGetInterfaceDeviceList,
//                                (PVOID)&PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_INTERFACE_DEVICE_LIST_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLen = PlugPlayData.BufferSize;
//     }
//     else
//     {
//         ret = NtStatusToCrError(Status);
//     }
//
//     DPRINT("PNP_GetInterfaceDeviceListSize() done (returns %lx)\n", ret);
//     return ret;
// }
//
//
// /* Function 24 */
// DWORD
// WINAPI
// PNP_RegisterDeviceClassAssociation(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     GUID *InterfaceGuid,
//     LPWSTR pszReference,
//     LPWSTR pszSymLink,
//     PNP_RPC_STRING_LEN *pulLength,
//     PNP_RPC_STRING_LEN *pulTransferLen,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA PlugPlayData;
//     NTSTATUS Status;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_RegisterDeviceClassAssociation(%p %S %p %S %S %p %p 0x%08lx)\n",
//            hBinding, pszDeviceID, InterfaceGuid, pszReference, pszSymLink,
//            pulLength, pulTransferLen, ulFlags);
//
//     if ((InterfaceGuid == NULL) ||
//         (pszSymLink == NULL) ||
//         (pulLength == NULL) ||
//         (pulTransferLen == NULL))
//         return CR_INVALID_POINTER;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance, pszDeviceID);
//     PlugPlayData.InterfaceGuid = InterfaceGuid;
//     RtlInitUnicodeString(&PlugPlayData.Reference, pszReference);
//     PlugPlayData.Register = TRUE;
//     PlugPlayData.SymbolicLinkName = pszSymLink;
//     PlugPlayData.SymbolicLinkNameLength = *pulLength;
//
//     Status = NtPlugPlayControl(PlugPlayControlDeviceClassAssociation,
//                                &PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA));
//     if (NT_SUCCESS(Status))
//     {
//         *pulLength = PlugPlayData.SymbolicLinkNameLength;
//         *pulTransferLen = *pulLength;
//     }
//     else
//     {
//         *pulLength = 0;
//         ret = NtStatusToCrError(Status);
//     }
//
//     return ret;
// }
//
//
// /* Function 25 */
// DWORD
// WINAPI
// PNP_UnregisterDeviceClassAssociation(
//     handle_t hBinding,
//     LPWSTR pszInterfaceDevice,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA PlugPlayData;
//     NTSTATUS Status;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_UnregisterDeviceClassAssociation(%p %S 0x%08lx)\n",
//            hBinding, pszInterfaceDevice, ulFlags);
//
//     if (pszInterfaceDevice == NULL)
//         return CR_INVALID_POINTER;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     ZeroMemory!(&PlugPlayData, sizeof(PlugPlayData));
//     PlugPlayData.Register = FALSE;
//     PlugPlayData.SymbolicLinkName = pszInterfaceDevice;
//     PlugPlayData.SymbolicLinkNameLength = wcslen(pszInterfaceDevice) + 1;
//
//     Status = NtPlugPlayControl(PlugPlayControlDeviceClassAssociation,
//                                &PlugPlayData,
//                                sizeof(PLUGPLAY_CONTROL_CLASS_ASSOCIATION_DATA));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// /* Function 26 */
// DWORD
// WINAPI
// PNP_GetClassRegProp(
//     handle_t hBinding,
//     LPWSTR pszClassGuid,
//     DWORD ulProperty,
//     DWORD *pulRegDataType,
//     BYTE *Buffer,
//     PNP_RPC_STRING_LEN *pulTransferLen,
//     PNP_RPC_STRING_LEN *pulLength,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     LPWSTR lpValueName = NULL;
//     HKEY hInstKey = NULL;
//     HKEY hPropKey = NULL;
//     LONG lError;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetClassRegProp(%p %S %lu %p %p %p %p 0x%08lx)\n",
//            hBinding, pszClassGuid, ulProperty, pulRegDataType,
//            Buffer, pulTransferLen, pulLength, ulFlags);
//
//     if (pulTransferLen == NULL || pulLength == NULL)
//     {
//         ret = CR_INVALID_POINTER;
//         goto done;
//     }
//
//     if (ulFlags != 0)
//     {
//         ret = CR_INVALID_FLAG;
//         goto done;
//     }
//
//     if (*pulLength < *pulTransferLen)
//         *pulLength = *pulTransferLen;
//
//     *pulTransferLen = 0;
//
//     switch (ulProperty)
//     {
//         case CM_CRP_SECURITY:
//             lpValueName = L"Security";
//             break;
//
//         case CM_CRP_DEVTYPE:
//             lpValueName = L"DeviceType";
//             break;
//
//         case CM_CRP_EXCLUSIVE:
//             lpValueName = L"Exclusive";
//             break;
//
//         case CM_CRP_CHARACTERISTICS:
//             lpValueName = L"DeviceCharacteristics";
//             break;
//
//         default:
//             ret = CR_INVALID_PROPERTY;
//             goto done;
//     }
//
//     DPRINT("Value name: %S\n", lpValueName);
//
//     lError = RegOpenKeyExW(hClassKey,
//                            pszClassGuid,
//                            0,
//                            KEY_READ,
//                            &hInstKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         *pulLength = 0;
//         ret = CR_NO_SUCH_REGISTRY_KEY;
//         goto done;
//     }
//
//     lError = RegOpenKeyExW(hInstKey,
//                            L"Properties",
//                            0,
//                            KEY_READ,
//                            &hPropKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         *pulLength = 0;
//         ret = CR_NO_SUCH_REGISTRY_KEY;
//         goto done;
//     }
//
//     lError = RegQueryValueExW(hPropKey,
//                               lpValueName,
//                               NULL,
//                               pulRegDataType,
//                               Buffer,
//                               pulLength);
//     if (lError != ERROR_SUCCESS)
//     {
//         if (lError == ERROR_MORE_DATA)
//         {
//             ret = CR_BUFFER_SMALL;
//         }
//         else
//         {
//             *pulLength = 0;
//             ret = CR_NO_SUCH_VALUE;
//         }
//     }
//
// done:
//     if (ret == CR_SUCCESS)
//         *pulTransferLen = *pulLength;
//
//     if (hPropKey != NULL)
//         RegCloseKey(hPropKey);
//
//     if (hInstKey != NULL)
//         RegCloseKey(hInstKey);
//
//     DPRINT("PNP_GetClassRegProp() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 27 */
// DWORD
// WINAPI
// PNP_SetClassRegProp(
//     handle_t hBinding,
//     LPWSTR pszClassGuid,
//     DWORD ulProperty,
//     DWORD ulDataType,
//     BYTE *Buffer,
//     PNP_PROP_SIZE ulLength,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     LPWSTR lpValueName = NULL;
//     HKEY hInstKey = 0;
//     HKEY hPropKey = 0;
//     LONG lError;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_SetClassRegProp(%p %S %lu %lu %p %lu 0x%08lx)\n",
//            hBinding, pszClassGuid, ulProperty, ulDataType,
//            Buffer, ulLength, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     switch (ulProperty)
//     {
//         case CM_CRP_SECURITY:
//             lpValueName = L"Security";
//             break;
//
//         case CM_CRP_DEVTYPE:
//             lpValueName = L"DeviceType";
//             break;
//
//         case CM_CRP_EXCLUSIVE:
//             lpValueName = L"Exclusive";
//             break;
//
//         case CM_CRP_CHARACTERISTICS:
//             lpValueName = L"DeviceCharacteristics";
//             break;
//
//         default:
//             return CR_INVALID_PROPERTY;
//     }
//
//     lError = RegOpenKeyExW(hClassKey,
//                            pszClassGuid,
//                            0,
//                            KEY_WRITE,
//                            &hInstKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_NO_SUCH_REGISTRY_KEY;
//         goto done;
//     }
//
//     /* FIXME: Set security descriptor */
//     lError = RegCreateKeyExW(hInstKey,
//                              L"Properties",
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ALL_ACCESS,
//                              NULL,
//                              &hPropKey,
//                              NULL);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     if (ulLength == 0)
//     {
//         if (RegDeleteValueW(hPropKey,
//                             lpValueName))
//             ret = CR_REGISTRY_ERROR;
//     }
//     else
//     {
//         if (RegSetValueExW(hPropKey,
//                            lpValueName,
//                            0,
//                            ulDataType,
//                            Buffer,
//                            ulLength))
//             ret = CR_REGISTRY_ERROR;
//     }
//
// done:
//     if (hPropKey != NULL)
//         RegCloseKey(hPropKey);
//
//     if (hInstKey != NULL)
//         RegCloseKey(hInstKey);
//
//     return ret;
// }
//
//
// static
// CONFIGRET
// CreateDeviceInstance(
//      LPWSTR pszDeviceID,
//      BOOL bPhantomDevice)
// {
//     WCHAR szEnumerator[MAX_DEVICE_ID_LEN];
//     WCHAR szDevice[MAX_DEVICE_ID_LEN];
//     WCHAR szInstance[MAX_DEVICE_ID_LEN];
//     HKEY hKeyEnumerator;
//     HKEY hKeyDevice;
//     HKEY hKeyInstance;
//     HKEY hKeyControl;
//     LONG lError;
//
//     /* Split the instance ID */
//     SplitDeviceInstanceID(pszDeviceID,
//                           szEnumerator,
//                           szDevice,
//                           szInstance);
//
//     /* Open or create the enumerator key */
//     lError = RegCreateKeyExW(hEnumKey,
//                              szEnumerator,
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ALL_ACCESS,
//                              NULL,
//                              &hKeyEnumerator,
//                              NULL);
//     if (lError != ERROR_SUCCESS)
//     {
//         return CR_REGISTRY_ERROR;
//     }
//
//     /* Open or create the device key */
//     lError = RegCreateKeyExW(hKeyEnumerator,
//                              szDevice,
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ALL_ACCESS,
//                              NULL,
//                              &hKeyDevice,
//                              NULL);
//
//     /* Close the enumerator key */
//     RegCloseKey(hKeyEnumerator);
//
//     if (lError != ERROR_SUCCESS)
//     {
//         return CR_REGISTRY_ERROR;
//     }
//
//     /* Try to open the instance key and fail if it exists */
//     lError = RegOpenKeyExW(hKeyDevice,
//                            szInstance,
//                            0,
//                            KEY_SET_VALUE,
//                            &hKeyInstance);
//     if (lError == ERROR_SUCCESS)
//     {
//         DPRINT1("Instance %S already exists!\n", szInstance);
//         RegCloseKey(hKeyInstance);
//         RegCloseKey(hKeyDevice);
//         return CR_ALREADY_SUCH_DEVINST;
//     }
//
//     /* Create a new instance key */
//     lError = RegCreateKeyExW(hKeyDevice,
//                              szInstance,
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ALL_ACCESS,
//                              NULL,
//                              &hKeyInstance,
//                              NULL);
//
//     /* Close the device key */
//     RegCloseKey(hKeyDevice);
//
//     if (lError != ERROR_SUCCESS)
//     {
//         return CR_REGISTRY_ERROR;
//     }
//
//     if (bPhantomDevice)
//     {
//         DWORD dwPhantomValue = 1;
//         RegSetValueExW(hKeyInstance,
//                        L"Phantom",
//                        0,
//                        REG_DWORD,
//                        (PBYTE)&dwPhantomValue,
//                        sizeof(dwPhantomValue));
//     }
//
//     /* Create the 'Control' sub key */
//     lError = RegCreateKeyExW(hKeyInstance,
//                              L"Control",
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ALL_ACCESS,
//                              NULL,
//                              &hKeyControl,
//                              NULL);
//     if (lError == ERROR_SUCCESS)
//     {
//         RegCloseKey(hKeyControl);
//     }
//
//     RegCloseKey(hKeyInstance);
//
//     return (lError == ERROR_SUCCESS) ? CR_SUCCESS : CR_REGISTRY_ERROR;
// }
//
//
// static
// CONFIGRET
// GenerateDeviceID(
//      LPWSTR pszDeviceID,
//      PNP_RPC_STRING_LEN ulLength)
// {
//     WCHAR szGeneratedInstance[MAX_DEVICE_ID_LEN];
//     HKEY hKey;
//     DWORD dwInstanceNumber;
//     DWORD dwError = ERROR_SUCCESS;
//     CONFIGRET ret = CR_SUCCESS;
//
//     /* Fail, if the device name contains backslashes */
//     if (wcschr(pszDeviceID, L'\\') != NULL)
//         return CR_INVALID_DEVICE_ID;
//
//     /* Generated ID is: Root\<Device ID>\<Instance number> */
//     dwInstanceNumber = 0;
//     while (dwError == ERROR_SUCCESS)
//     {
//         if (dwInstanceNumber >= 10000)
//             return CR_FAILURE;
//
//         swprintf(szGeneratedInstance, L"Root\\%ls\\%04lu",
//                  pszDeviceID, dwInstanceNumber);
//
//         /* Try to open the enum key of the device instance */
//         dwError = RegOpenKeyEx(hEnumKey, szGeneratedInstance, 0, KEY_QUERY_VALUE, &hKey);
//         if (dwError == ERROR_SUCCESS)
//         {
//             RegCloseKey(hKey);
//             dwInstanceNumber++;
//         }
//     }
//
//     /* pszDeviceID is an out parameter too for generated IDs */
//     if (wcslen(szGeneratedInstance) > ulLength)
//     {
//         ret = CR_BUFFER_SMALL;
//     }
//     else
//     {
//         wcscpy(pszDeviceID, szGeneratedInstance);
//     }
//
//     return ret;
// }
//
//
// /* Function 28 */
// DWORD
// WINAPI
// PNP_CreateDevInst(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     LPWSTR pszParentDeviceID,
//     PNP_RPC_STRING_LEN ulLength,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA ControlData;
//     HKEY hKey = NULL;
//     DWORD dwSize, dwPhantom;
//     NTSTATUS Status;
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_CreateDevInst(%p %S %S %lu 0x%08lx)\n",
//            hBinding, pszParentDeviceID, pszDeviceID, ulLength, ulFlags);
//
//     if (ulFlags & ~CM_CREATE_DEVNODE_BITS)
//         return CR_INVALID_FLAG;
//
//     if (pszDeviceID == NULL || pszParentDeviceID == NULL)
//         return CR_INVALID_POINTER;
//
//     /* Fail, if the parent device is not the root device */
//     if (!IsRootDeviceInstanceID(pszParentDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (ulFlags & CM_CREATE_DEVNODE_GENERATE_ID)
//     {
//         ret = GenerateDeviceID(pszDeviceID,
//                                ulLength);
//         if (ret != CR_SUCCESS)
//             return ret;
//     }
//
//     /* Try to open the device instance key */
//     RegOpenKeyEx(hEnumKey, pszDeviceID, 0, KEY_READ | KEY_WRITE, &hKey);
//
//     if (ulFlags & CM_CREATE_DEVNODE_PHANTOM)
//     {
//         /* Fail, if the device already exists */
//         if (hKey != NULL)
//         {
//             ret = CR_ALREADY_SUCH_DEVINST;
//             goto done;
//         }
//
//         /* Create the phantom device instance */
//         ret = CreateDeviceInstance(pszDeviceID, TRUE);
//     }
//     else
//     {
//         /* Fail, if the device exists and is present */
//         if ((hKey != NULL) && (IsPresentDeviceInstanceID(pszDeviceID)))
//         {
//             ret = CR_ALREADY_SUCH_DEVINST;
//             goto done;
//         }
//
//         /* If it does not already exist ... */
//         if (hKey == NULL)
//         {
//             /* Create the device instance */
//             ret = CreateDeviceInstance(pszDeviceID, FALSE);
//
//             /* Open the device instance key */
//             RegOpenKeyEx(hEnumKey, pszDeviceID, 0, KEY_READ | KEY_WRITE, &hKey);
//         }
//
//         /* Create a device node for the device */
//         RtlInitUnicodeString(&ControlData.DeviceInstance, pszDeviceID);
//         Status = NtPlugPlayControl(PlugPlayControlInitializeDevice,
//                                    &ControlData,
//                                    sizeof(ControlData));
//         if (!NT_SUCCESS(Status))
//         {
//             ret = CR_FAILURE;
//             goto done;
//         }
//
//         /* If the device is a phantom device, turn it into a normal device */
//         if (hKey != NULL)
//         {
//             dwPhantom = 0;
//             dwSize = sizeof(DWORD);
//             RegQueryValueEx(hKey, L"Phantom", NULL, NULL, (PBYTE)&dwPhantom, &dwSize);
//
//             if (dwPhantom != 0)
//                 RegDeleteValue(hKey, L"Phantom");
//         }
//     }
//
// done:
//     if (hKey)
//         RegCloseKey(hKey);
//
//     DPRINT("PNP_CreateDevInst() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// static CONFIGRET
// SetupDeviceInstance(
//      LPWSTR pszDeviceInstance,
//      DWORD ulMinorAction)
// {
//     PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA ControlData;
//     HKEY hDeviceKey = NULL;
//     DWORD dwDisableCount, dwSize;
//     DWORD ulStatus, ulProblem;
//     DWORD dwError;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT1("SetupDeviceInstance(%S 0x%08lx)\n",
//             pszDeviceInstance, ulMinorAction);
//
//     if (IsRootDeviceInstanceID(pszDeviceInstance))
//         return CR_INVALID_DEVINST;
//
//     if (ulMinorAction & ~CM_SETUP_BITS)
//         return CR_INVALID_FLAG;
//
//     if ((ulMinorAction == CM_SETUP_DOWNLOAD) ||
//         (ulMinorAction == CM_SETUP_WRITE_LOG_CONFS))
//         return CR_SUCCESS;
//
//     dwError = RegOpenKeyExW(hEnumKey,
//                             pszDeviceInstance,
//                             0,
//                             KEY_READ,
//                             &hDeviceKey);
//     if (dwError != ERROR_SUCCESS)
//         return CR_INVALID_DEVNODE;
//
//     dwSize = sizeof(dwDisableCount);
//     dwError = RegQueryValueExW(hDeviceKey,
//                                L"DisableCount",
//                                NULL,
//                                NULL,
//                                (LPBYTE)&dwDisableCount,
//                                &dwSize);
//     if ((dwError == ERROR_SUCCESS) &&
//         (dwDisableCount > 0))
//     {
//         goto done;
//     }
//
//     GetDeviceStatus(pszDeviceInstance,
//                     &ulStatus,
//                     &ulProblem);
//
//     if (ulStatus & DN_STARTED)
//     {
//         goto done;
//     }
//
//     if (ulStatus & DN_HAS_PROBLEM)
//     {
//         ret = ClearDeviceStatus(pszDeviceInstance,
//                                 DN_HAS_PROBLEM,
//                                 ulProblem);
//     }
//
//     if (ret != CR_SUCCESS)
//         goto done;
//
//     /* Start the device */
//     RtlInitUnicodeString(&ControlData.DeviceInstance,
//                          pszDeviceInstance);
//     Status = NtPlugPlayControl(PlugPlayControlStartDevice,
//                                &ControlData,
//                                sizeof(PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
// done:
//     if (hDeviceKey != NULL)
//         RegCloseKey(hDeviceKey);
//
//     return ret;
// }
//
//
// static CONFIGRET
// EnableDeviceInstance(
//      LPWSTR pszDeviceInstance)
// {
//     PLUGPLAY_CONTROL_DEVICE_CONTROL_DATA ControlData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT("Enable device instance %S\n", pszDeviceInstance);
//
//     RtlInitUnicodeString(&ControlData.DeviceInstance, pszDeviceInstance);
//     Status = NtPlugPlayControl(PlugPlayControlStartDevice, &ControlData, sizeof(ControlData));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// static CONFIGRET
// ReenumerateDeviceInstance(
//      LPWSTR pszDeviceInstance,
//      ULONG ulMinorAction)
// {
//     PLUGPLAY_CONTROL_ENUMERATE_DEVICE_DATA EnumerateDeviceData;
//     CONFIGRET ret = CR_SUCCESS;
//     NTSTATUS Status;
//
//     DPRINT1("ReenumerateDeviceInstance(%S 0x%08lx)\n",
//            pszDeviceInstance, ulMinorAction);
//
//     if (ulMinorAction & ~CM_REENUMERATE_BITS)
//         return CR_INVALID_FLAG;
//
//     if (ulMinorAction & CM_REENUMERATE_RETRY_INSTALLATION)
//     {
//         DPRINT1("CM_REENUMERATE_RETRY_INSTALLATION not implemented!\n");
//     }
//
//     RtlInitUnicodeString(&EnumerateDeviceData.DeviceInstance,
//                          pszDeviceInstance);
//     EnumerateDeviceData.Flags = 0;
//
//     Status = NtPlugPlayControl(PlugPlayControlEnumerateDevice,
//                                &EnumerateDeviceData,
//                                sizeof(PLUGPLAY_CONTROL_ENUMERATE_DEVICE_DATA));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// /* Function 29 */
// DWORD
// WINAPI
// PNP_DeviceInstanceAction(
//     handle_t hBinding,
//     DWORD ulMajorAction,
//     DWORD ulMinorAction,
//     LPWSTR pszDeviceInstance1,
//     LPWSTR pszDeviceInstance2)
// {
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_DeviceInstanceAction(%p %lu 0x%08lx %S %S)\n",
//            hBinding, ulMajorAction, ulMinorAction,
//            pszDeviceInstance1, pszDeviceInstance2);
//
//     switch (ulMajorAction)
//     {
//         case PNP_DEVINST_SETUP:
//             ret = SetupDeviceInstance(pszDeviceInstance1,
//                                       ulMinorAction);
//             break;
//
//         case PNP_DEVINST_ENABLE:
//             ret = EnableDeviceInstance(pszDeviceInstance1);
//             break;
//
//         case PNP_DEVINST_REENUMERATE:
//             ret = ReenumerateDeviceInstance(pszDeviceInstance1,
//                                             ulMinorAction);
//             break;
//
//         default:
//             DPRINT1("Unknown device action %lu: not implemented\n", ulMajorAction);
//             ret = CR_CALL_NOT_IMPLEMENTED;
//     }
//
//     DPRINT("PNP_DeviceInstanceAction() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 30 */
// DWORD
// WINAPI
// PNP_GetDeviceStatus(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD *pulStatus,
//     DWORD *pulProblem,
//     DWORD ulFlags)
// {
//     DWORD ulDataType, ulTransferLength, ulLength;
//     DWORD ulCapabilities, ulConfigFlags;
//     CONFIGRET ret;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//     UNREFERENCED_PARAMETER!(ulFlags);
//
//     DPRINT("PNP_GetDeviceStatus(%p %S %p %p 0x%08lx)\n",
//            hBinding, pDeviceID, pulStatus, pulProblem, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if ((pulStatus == NULL) || (pulProblem == NULL))
//         return CR_INVALID_POINTER;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     ret = GetDeviceStatus(pDeviceID, pulStatus, pulProblem);
//     if (ret != CR_SUCCESS)
//         return ret;
//
//     /* Check for DN_REMOVABLE */
//     ulTransferLength = sizeof(ulCapabilities);
//     ulLength = sizeof(ulCapabilities);
//     ret = PNP_GetDeviceRegProp(NULL,
//                                pDeviceID,
//                                CM_DRP_CAPABILITIES,
//                                &ulDataType,
//                                (PBYTE)&ulCapabilities,
//                                &ulTransferLength,
//                                &ulLength,
//                                0);
//     if (ret != CR_SUCCESS)
//         ulCapabilities = 0;
//
//     if (ulCapabilities & CM_DEVCAP_REMOVABLE)
//         *pulStatus |= DN_REMOVABLE;
//
//     /* Check for DN_MANUAL */
//     ulTransferLength = sizeof(ulConfigFlags);
//     ulLength = sizeof(ulConfigFlags);
//     ret = PNP_GetDeviceRegProp(NULL,
//                                pDeviceID,
//                                CM_DRP_CONFIGFLAGS,
//                                &ulDataType,
//                                (PBYTE)&ulConfigFlags,
//                                &ulTransferLength,
//                                &ulLength,
//                                0);
//     if (ret != CR_SUCCESS)
//         ulConfigFlags = 0;
//
//     if (ulConfigFlags & CONFIGFLAG_MANUAL_INSTALL)
//         *pulStatus |= DN_MANUAL;
//
//     /* Check for failed install */
//     if (((*pulStatus & DN_HAS_PROBLEM) == 0) && (ulConfigFlags & CONFIGFLAG_FAILEDINSTALL))
//     {
//         *pulStatus |= DN_HAS_PROBLEM;
//         *pulProblem = CM_PROB_FAILED_INSTALL;
//     }
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 31 */
// DWORD
// WINAPI
// PNP_SetDeviceProblem(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulProblem,
//     DWORD ulFlags)
// {
//     ULONG ulOldStatus, ulOldProblem;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_SetDeviceProblem(%p %S %lu 0x%08lx)\n",
//            hBinding, pDeviceID, ulProblem, ulFlags);
//
//     if (ulFlags & ~CM_SET_DEVNODE_PROBLEM_BITS)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     ret = GetDeviceStatus(pDeviceID,
//                           &ulOldStatus,
//                           &ulOldProblem);
//     if (ret != CR_SUCCESS)
//         return ret;
//
//     if (((ulFlags & CM_SET_DEVNODE_PROBLEM_OVERRIDE) == 0) &&
//         (ulOldProblem != 0) &&
//         (ulOldProblem != ulProblem))
//     {
//         return CR_FAILURE;
//     }
//
//     if (ulProblem == 0)
//     {
//         ret = ClearDeviceStatus(pDeviceID,
//                                 DN_HAS_PROBLEM,
//                                 ulOldProblem);
//     }
//     else
//     {
//         ret = SetDeviceStatus(pDeviceID,
//                               DN_HAS_PROBLEM,
//                               ulProblem);
//     }
//
//     return ret;
// }
//
//
// /* Function 32 */
// DWORD
// WINAPI
// PNP_DisableDevInst(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     PPNP_VETO_TYPE pVetoType,
//     LPWSTR pszVetoName,
//     DWORD ulNameLength,
//     DWORD ulFlags)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_DisableDevInst(%p %S %p %p %lu 0x%08lx)\n",
//             hBinding, pDeviceID, pVetoType, pszVetoName, ulNameLength, ulFlags);
//
//     if (ulFlags & ~CM_DISABLE_BITS)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID) ||
//         IsRootDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     return DisableDeviceInstance(pDeviceID,
//                                  pVetoType,
//                                  pszVetoName,
//                                  ulNameLength);
// }
//
//
// /* Function 33 */
// DWORD
// WINAPI
// PNP_UninstallDevInst(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// static BOOL
// CheckForDeviceId(LPWSTR lpDeviceIdList,
//                  LPWSTR lpDeviceId)
// {
//     LPWSTR lpPtr;
//     DWORD dwLength;
//
//     lpPtr = lpDeviceIdList;
//     while (*lpPtr != 0)
//     {
//         dwLength = wcslen(lpPtr);
//         if (0 == _wcsicmp(lpPtr, lpDeviceId))
//             return TRUE;
//
//         lpPtr += (dwLength + 1);
//     }
//
//     return FALSE;
// }
//
//
// static VOID
// AppendDeviceId(LPWSTR lpDeviceIdList,
//                LPDWORD lpDeviceIdListSize,
//                LPWSTR lpDeviceId)
// {
//     DWORD dwLen;
//     DWORD dwPos;
//
//     dwLen = wcslen(lpDeviceId);
//     dwPos = (*lpDeviceIdListSize / sizeof(WCHAR)) - 1;
//
//     wcscpy(&lpDeviceIdList[dwPos], lpDeviceId);
//
//     dwPos += (dwLen + 1);
//
//     lpDeviceIdList[dwPos] = 0;
//
//     *lpDeviceIdListSize = dwPos * sizeof(WCHAR);
// }
//
//
// /* Function 34 */
// DWORD
// WINAPI
// PNP_AddID(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     LPWSTR pszID,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     HKEY hDeviceKey;
//     LPWSTR pszSubKey;
//     DWORD dwDeviceIdListSize;
//     DWORD dwNewDeviceIdSize;
//     WCHAR * pszDeviceIdList = NULL;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_AddID(%p %S %S 0x%08lx)\n",
//            hBinding, pszDeviceID, pszID, ulFlags);
//
//     if (RegOpenKeyExW(hEnumKey,
//                       pszDeviceID,
//                       0,
//                       KEY_QUERY_VALUE | KEY_SET_VALUE,
//                       &hDeviceKey) != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to open the device key!\n");
//         return CR_INVALID_DEVNODE;
//     }
//
//     pszSubKey = (ulFlags & CM_ADD_ID_COMPATIBLE) ? L"CompatibleIDs" : L"HardwareID";
//
//     if (RegQueryValueExW(hDeviceKey,
//                          pszSubKey,
//                          NULL,
//                          NULL,
//                          NULL,
//                          &dwDeviceIdListSize) != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to query the desired ID string!\n");
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     dwNewDeviceIdSize = lstrlenW(pszDeviceID);
//     if (!dwNewDeviceIdSize)
//     {
//         ret = CR_INVALID_POINTER;
//         goto Done;
//     }
//
//     dwDeviceIdListSize += (dwNewDeviceIdSize + 2) * sizeof(WCHAR);
//
//     pszDeviceIdList = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, dwDeviceIdListSize);
//     if (!pszDeviceIdList)
//     {
//         DPRINT("Failed to allocate memory for the desired ID string!\n");
//         ret = CR_OUT_OF_MEMORY;
//         goto Done;
//     }
//
//     if (RegQueryValueExW(hDeviceKey,
//                          pszSubKey,
//                          NULL,
//                          NULL,
//                          (LPBYTE)pszDeviceIdList,
//                          &dwDeviceIdListSize) != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to query the desired ID string!\n");
//         ret = CR_REGISTRY_ERROR;
//         goto Done;
//     }
//
//     /* Check whether the device ID is already in use */
//     if (CheckForDeviceId(pszDeviceIdList, pszDeviceID))
//     {
//         DPRINT("Device ID was found in the ID string!\n");
//         ret = CR_SUCCESS;
//         goto Done;
//     }
//
//     /* Append the Device ID */
//     AppendDeviceId(pszDeviceIdList, &dwDeviceIdListSize, pszID);
//
//     if (RegSetValueExW(hDeviceKey,
//                        pszSubKey,
//                        0,
//                        REG_MULTI_SZ,
//                        (LPBYTE)pszDeviceIdList,
//                        dwDeviceIdListSize) != ERROR_SUCCESS)
//     {
//         DPRINT("Failed to set the desired ID string!\n");
//         ret = CR_REGISTRY_ERROR;
//     }
//
// Done:
//     RegCloseKey(hDeviceKey);
//     if (pszDeviceIdList)
//         HeapFree(GetProcessHeap(), 0, pszDeviceIdList);
//
//     DPRINT("PNP_AddID() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 35 */
// DWORD
// WINAPI
// PNP_RegisterDriver(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     DWORD ulFlags)
// {
//     DPRINT("PNP_RegisterDriver(%p %S 0x%lx)\n",
//            hBinding, pszDeviceID, ulFlags);
//
//     if (ulFlags & ~CM_REGISTER_DEVICE_DRIVER_BITS)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     SetDeviceStatus(pszDeviceID, 0, 0);
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 36 */
// DWORD
// WINAPI
// PNP_QueryRemove(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     PPNP_VETO_TYPE pVetoType,
//     LPWSTR pszVetoName,
//     DWORD ulNameLength,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_QUERY_REMOVE_DATA PlugPlayData;
//     NTSTATUS Status;
//     DWORD ret = CR_SUCCESS;
//
//     DPRINT1("PNP_QueryRemove(%p %S %p %p %lu 0x%lx)\n",
//             hBinding, pszDeviceID, pVetoType, pszVetoName,
//             ulNameLength, ulFlags);
//
//     if (ulFlags & ~CM_REMOVE_BITS)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pszDeviceID) ||
//         IsRootDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (pVetoType != NULL)
//         *pVetoType = PNP_VetoTypeUnknown;
//
//     if (pszVetoName != NULL && ulNameLength > 0)
//         *pszVetoName = UNICODE_NULL;
//
//     RtlZeroMemory!(&PlugPlayData, sizeof(PlugPlayData));
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//     PlugPlayData.VetoName = pszVetoName;
//     PlugPlayData.NameLength = ulNameLength;
// //    PlugPlayData.Flags =
//
//     Status = NtPlugPlayControl(PlugPlayControlQueryAndRemoveDevice,
//                                &PlugPlayData,
//                                sizeof(PlugPlayData));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// /* Function 37 */
// DWORD
// WINAPI
// PNP_RequestDeviceEject(
//     handle_t hBinding,
//     LPWSTR pszDeviceID,
//     PPNP_VETO_TYPE pVetoType,
//     LPWSTR pszVetoName,
//     DWORD ulNameLength,
//     DWORD ulFlags)
// {
//     PLUGPLAY_CONTROL_QUERY_REMOVE_DATA PlugPlayData;
//     NTSTATUS Status;
//     DWORD ret = CR_SUCCESS;
//
//     DPRINT1("PNP_RequestDeviceEject(%p %S %p %p %lu 0x%lx)\n",
//             hBinding, pszDeviceID, pVetoType, pszVetoName,
//             ulNameLength, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pszDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (pVetoType != NULL)
//         *pVetoType = PNP_VetoTypeUnknown;
//
//     if (pszVetoName != NULL && ulNameLength > 0)
//         *pszVetoName = UNICODE_NULL;
//
//     RtlZeroMemory!(&PlugPlayData, sizeof(PlugPlayData));
//     RtlInitUnicodeString(&PlugPlayData.DeviceInstance,
//                          pszDeviceID);
//     PlugPlayData.VetoName = pszVetoName;
//     PlugPlayData.NameLength = ulNameLength;
// //    PlugPlayData.Flags =
//
//     Status = NtPlugPlayControl(PlugPlayControlQueryAndRemoveDevice,
//                                &PlugPlayData,
//                                sizeof(PlugPlayData));
//     if (!NT_SUCCESS(Status))
//         ret = NtStatusToCrError(Status);
//
//     return ret;
// }
//
//
// /* Function 38 */
// CONFIGRET
// WINAPI
// PNP_IsDockStationPresent(
//     handle_t hBinding,
//     BOOL *Present)
// {
//     HKEY hKey;
//     DWORD dwType;
//     DWORD dwValue;
//     DWORD dwSize;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_IsDockStationPresent(%p %p)\n",
//             hBinding, Present);
//
//     *Present = FALSE;
//
//     if (RegOpenKeyExW(HKEY_CURRENT_CONFIG,
//                       L"CurrentDockInfo",
//                       0,
//                       KEY_READ,
//                       &hKey) != ERROR_SUCCESS)
//         return CR_REGISTRY_ERROR;
//
//     dwSize = sizeof(DWORD);
//     if (RegQueryValueExW(hKey,
//                          L"DockingState",
//                          NULL,
//                          &dwType,
//                          (LPBYTE)&dwValue,
//                          &dwSize) != ERROR_SUCCESS)
//         ret = CR_REGISTRY_ERROR;
//
//     RegCloseKey(hKey);
//
//     if (ret == CR_SUCCESS)
//     {
//         if (dwType != REG_DWORD || dwSize != sizeof(DWORD))
//         {
//             ret = CR_REGISTRY_ERROR;
//         }
//         else if (dwValue != 0)
//         {
//             *Present = TRUE;
//         }
//     }
//
//     DPRINT1("PNP_IsDockStationPresent() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 39 */
// DWORD
// WINAPI
// PNP_RequestEjectPC(
//     handle_t hBinding)
// {
//     WCHAR szDockDeviceInstance[MAX_DEVICE_ID_LEN];
//     PLUGPLAY_CONTROL_RETRIEVE_DOCK_DATA DockData;
//     NTSTATUS Status;
//
//     DPRINT("PNP_RequestEjectPC(%p)\n", hBinding);
//
//     /* Retrieve the dock device */
//     DockData.DeviceInstanceLength = ARRAYSIZE(szDockDeviceInstance);
//     DockData.DeviceInstance = szDockDeviceInstance;
//
//     Status = NtPlugPlayControl(PlugPlayControlRetrieveDock,
//                                &DockData,
//                                sizeof(DockData));
//     if (!NT_SUCCESS(Status))
//         return NtStatusToCrError(Status);
//
//     /* Eject the dock device */
//     return PNP_RequestDeviceEject(hBinding,
//                                   szDockDeviceInstance,
//                                   NULL,
//                                   NULL,
//                                   0,
//                                   0);
// }
//
//
// /* Function 40 */
// DWORD
// WINAPI
// PNP_HwProfFlags(
//     handle_t hBinding,
//     DWORD ulAction,
//     LPWSTR pDeviceID,
//     DWORD ulConfig,
//     DWORD *pulValue,
//     PPNP_VETO_TYPE pVetoType,
//     LPWSTR pszVetoName,
//     DWORD ulNameLength,
//     DWORD ulFlags)
// {
//     CONFIGRET ret = CR_SUCCESS;
//     WCHAR szKeyName[MAX_PATH];
//     HKEY hKey;
//     HKEY hDeviceKey;
//     DWORD dwSize;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_HwProfFlags() called\n");
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (ulConfig == 0)
//     {
//         wcscpy(szKeyName,
//                L"System\\CurrentControlSet\\HardwareProfiles\\Current\\System\\CurrentControlSet\\Enum");
//     }
//     else
//     {
//         swprintf(szKeyName,
//                  L"System\\CurrentControlSet\\HardwareProfiles\\%04lu\\System\\CurrentControlSet\\Enum",
//                  ulConfig);
//     }
//
//     if (RegOpenKeyExW(HKEY_LOCAL_MACHINE,
//                       szKeyName,
//                       0,
//                       KEY_QUERY_VALUE,
//                       &hKey) != ERROR_SUCCESS)
//         return CR_REGISTRY_ERROR;
//
//     if (ulAction == PNP_GET_HWPROFFLAGS)
//     {
//          if (RegOpenKeyExW(hKey,
//                            pDeviceID,
//                            0,
//                            KEY_QUERY_VALUE,
//                            &hDeviceKey) != ERROR_SUCCESS)
//          {
//             *pulValue = 0;
//          }
//          else
//          {
//              dwSize = sizeof(DWORD);
//              if (RegQueryValueExW(hDeviceKey,
//                                   L"CSConfigFlags",
//                                   NULL,
//                                   NULL,
//                                   (LPBYTE)pulValue,
//                                   &dwSize) != ERROR_SUCCESS)
//              {
//                  *pulValue = 0;
//              }
//
//              RegCloseKey(hDeviceKey);
//          }
//     }
//     else if (ulAction == PNP_SET_HWPROFFLAGS)
//     {
//         /* FIXME: not implemented yet */
//         ret = CR_CALL_NOT_IMPLEMENTED;
//     }
//
//     RegCloseKey(hKey);
//
//     return ret;
// }
//
//
// /* Function 41 */
// DWORD
// WINAPI
// PNP_GetHwProfInfo(
//     handle_t hBinding,
//     DWORD ulIndex,
//     HWPROFILEINFO *pHWProfileInfo,
//     DWORD ulProfileInfoSize,
//     DWORD ulFlags)
// {
//     WCHAR szProfileName[5];
//     HKEY hKeyConfig = NULL;
//     HKEY hKeyProfiles = NULL;
//     HKEY hKeyProfile = NULL;
//     DWORD dwDisposition;
//     DWORD dwSize;
//     LONG lError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetHwProfInfo() called\n");
//
//     if (ulProfileInfoSize == 0)
//     {
//         ret = CR_INVALID_DATA;
//         goto done;
//     }
//
//     if (ulFlags != 0)
//     {
//         ret = CR_INVALID_FLAG;
//         goto done;
//     }
//
//     /* Initialize the profile information */
//     pHWProfileInfo->HWPI_ulHWProfile = 0;
//     pHWProfileInfo->HWPI_szFriendlyName[0] = 0;
//     pHWProfileInfo->HWPI_dwFlags = 0;
//
//     /* Open the 'IDConfigDB' key */
//     lError = RegCreateKeyExW(HKEY_LOCAL_MACHINE,
//                              L"System\\CurrentControlSet\\Control\\IDConfigDB",
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_QUERY_VALUE,
//                              NULL,
//                              &hKeyConfig,
//                              &dwDisposition);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     /* Open the 'Hardware Profiles' subkey */
//     lError = RegCreateKeyExW(hKeyConfig,
//                              L"Hardware Profiles",
//                              0,
//                              NULL,
//                              REG_OPTION_NON_VOLATILE,
//                              KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
//                              NULL,
//                              &hKeyProfiles,
//                              &dwDisposition);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     if (ulIndex == (ULONG)-1)
//     {
//         dwSize = sizeof(ULONG);
//         lError = RegQueryValueExW(hKeyConfig,
//                                   L"CurrentConfig",
//                                   NULL,
//                                   NULL,
//                                   (LPBYTE)&pHWProfileInfo->HWPI_ulHWProfile,
//                                   &dwSize);
//         if (lError != ERROR_SUCCESS)
//         {
//             pHWProfileInfo->HWPI_ulHWProfile = 0;
//             ret = CR_REGISTRY_ERROR;
//             goto done;
//         }
//     }
//     else
//     {
//         /* FIXME: not implemented yet */
//         ret = CR_CALL_NOT_IMPLEMENTED;
//         goto done;
//     }
//
//     swprintf(szProfileName, L"%04lu", pHWProfileInfo->HWPI_ulHWProfile);
//
//     lError = RegOpenKeyExW(hKeyProfiles,
//                            szProfileName,
//                            0,
//                            KEY_QUERY_VALUE,
//                            &hKeyProfile);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     dwSize = sizeof(pHWProfileInfo->HWPI_szFriendlyName);
//     lError = RegQueryValueExW(hKeyProfile,
//                               L"FriendlyName",
//                               NULL,
//                               NULL,
//                               (LPBYTE)&pHWProfileInfo->HWPI_szFriendlyName,
//                               &dwSize);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
// done:
//     if (hKeyProfile != NULL)
//         RegCloseKey(hKeyProfile);
//
//     if (hKeyProfiles != NULL)
//         RegCloseKey(hKeyProfiles);
//
//     if (hKeyConfig != NULL)
//         RegCloseKey(hKeyConfig);
//
//     return ret;
// }
//
//
// /* Function 42 */
// DWORD
// WINAPI
// PNP_AddEmptyLogConf(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulPriority,
//     DWORD *pulLogConfTag,
//     DWORD ulFlags)
// {
//     HKEY hConfigKey = NULL;
//     DWORD RegDataType = 0;
//     ULONG ulDataSize = 0, ulNewSize = 0;
//     PBYTE pDataBuffer = NULL;
//     WCHAR szValueNameBuffer[LOGCONF_NAME_BUFFER_SIZE];
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_AddEmptyLogConf(%p %S %lu %p 0x%08lx)\n",
//            hBinding, pDeviceID, ulPriority, pulLogConfTag, ulFlags);
//
//     if (pulLogConfTag == NULL)
//         return CR_INVALID_POINTER;
//
//     *pulLogConfTag = 0;
//
//     if (ulFlags & ~(LOG_CONF_BITS | PRIORITY_BIT))
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID) || IsRootDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVNODE;
//
//     ret = OpenConfigurationKey(pDeviceID,
//                                ulFlags & LOG_CONF_BITS,
//                                &hConfigKey);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("OpenConfigurationKey() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     ret = GetConfigurationData(hConfigKey,
//                                ulFlags & LOG_CONF_BITS,
//                                &RegDataType,
//                                &ulDataSize,
//                                &pDataBuffer,
//                                LOGCONF_NAME_BUFFER_SIZE,
//                                szValueNameBuffer);
//
//     if (ret != CR_SUCCESS || ulDataSize == 0)
//     {
//         ret = CR_SUCCESS;
//
//         if (RegDataType == REG_RESOURCE_LIST)
//         {
//             PCM_RESOURCE_LIST pResourceList = NULL;
//             PCM_FULL_RESOURCE_DESCRIPTOR pResource = NULL;
//
//             /* Allocate a buffer for the new configuration */
//             ulDataSize = sizeof(CM_RESOURCE_LIST) - sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//             pDataBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ulDataSize);
//             if (pDataBuffer == NULL)
//             {
//                 ret = CR_OUT_OF_MEMORY;
//                 goto done;
//             }
//
//             pResourceList = (PCM_RESOURCE_LIST)pDataBuffer;
//             pResourceList->Count = 1;
//
//             pResource = (PCM_FULL_RESOURCE_DESCRIPTOR)&pResourceList->List[0];
//             pResource->InterfaceType = InterfaceTypeUndefined;
//             pResource->BusNumber = 0;
//             pResource->PartialResourceList.Version = 1;
//             pResource->PartialResourceList.Revision = 1;
//             pResource->PartialResourceList.Count = 0;
//         }
//         else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//         {
//             PIO_RESOURCE_REQUIREMENTS_LIST pRequirementsList = NULL;
//             PIO_RESOURCE_LIST pResourceList = NULL;
//
//             ulDataSize = sizeof(IO_RESOURCE_REQUIREMENTS_LIST);
//             pDataBuffer = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, ulDataSize);
//             if (pDataBuffer == NULL)
//             {
//                 ret = CR_OUT_OF_MEMORY;
//                 goto done;
//             }
//
//             pRequirementsList = (PIO_RESOURCE_REQUIREMENTS_LIST)pDataBuffer;
//             pRequirementsList->ListSize = ulDataSize;
//             pRequirementsList->InterfaceType = InterfaceTypeUndefined;
//             pRequirementsList->BusNumber = 0;
//             pRequirementsList->SlotNumber = 0;
//             pRequirementsList->AlternativeLists = 1;
//
//             pResourceList = (PIO_RESOURCE_LIST)&pRequirementsList->List[0];
//             pResourceList->Version = 1;
//             pResourceList->Revision = 1;
//             pResourceList->Count = 1;
//
//             pResourceList->Descriptors[0].Option = IO_RESOURCE_PREFERRED;
//             pResourceList->Descriptors[0].Type = CmResourceTypeConfigData;
//             pResourceList->Descriptors[0].u.ConfigData.Priority = ulPriority;
//         }
//         else
//         {
//             ret = CR_FAILURE;
//             goto done;
//         }
//     }
//     else
//     {
//         if (RegDataType == REG_RESOURCE_LIST)
//         {
//             PCM_RESOURCE_LIST pResourceList = NULL;
//             PCM_FULL_RESOURCE_DESCRIPTOR pResource = NULL;
//             ULONG ulIndex;
//
//             /* Reallocate a larger buffer in order to add the new configuration */
//             ulNewSize = sizeof(CM_FULL_RESOURCE_DESCRIPTOR) - sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//             pDataBuffer = HeapReAlloc(GetProcessHeap(),
//                                       0,
//                                       pDataBuffer,
//                                       ulDataSize + ulNewSize);
//             if (pDataBuffer == NULL)
//             {
//                 ret = CR_OUT_OF_MEMORY;
//                 goto done;
//             }
//
//             pResourceList = (PCM_RESOURCE_LIST)pDataBuffer;
//
//             /* Get a pointer to the new (uninitialized) resource descriptor */
//             pResource = (PCM_FULL_RESOURCE_DESCRIPTOR)&pResourceList->List[0];
//             for (ulIndex = 0; ulIndex < pResourceList->Count; ulIndex++)
//                 pResource = NextResourceDescriptor(pResource);
//
//             /* Initialize the new resource descriptor */
//             pResourceList->Count++;
//             pResource->InterfaceType= InterfaceTypeUndefined;
//             pResource->BusNumber = 0;
//             pResource->PartialResourceList.Version = 1;
//             pResource->PartialResourceList.Revision = 1;
//             pResource->PartialResourceList.Count = 0;
//
//             *pulLogConfTag = ulIndex;
//         }
//         else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//         {
//             PIO_RESOURCE_REQUIREMENTS_LIST pRequirementsList = NULL;
//             PIO_RESOURCE_LIST pResourceList = NULL;
//             ULONG ulIndex;
//
//             /* Reallocate a larger buffer in order to add the new configuration */
//             ulNewSize = sizeof(IO_RESOURCE_REQUIREMENTS_LIST);
//             pDataBuffer = HeapReAlloc(GetProcessHeap(),
//                                       0,
//                                       pDataBuffer,
//                                       ulDataSize + ulNewSize);
//             if (pDataBuffer == NULL)
//             {
//                 ret = CR_OUT_OF_MEMORY;
//                 goto done;
//             }
//
//             pRequirementsList = (PIO_RESOURCE_REQUIREMENTS_LIST)pDataBuffer;
//             pResourceList = (PIO_RESOURCE_LIST)&pRequirementsList->List[0];
//             for (ulIndex = 0; ulIndex < pRequirementsList->AlternativeLists - 1; ulIndex++)
//                 pResourceList = NextResourceRequirement(pResourceList);
//
//             pRequirementsList->ListSize = ulDataSize + ulNewSize;
//             pRequirementsList->AlternativeLists++;
//
//             pResourceList->Version = 1;
//             pResourceList->Revision = 1;
//             pResourceList->Count = 1;
//
//             pResourceList->Descriptors[0].Option = IO_RESOURCE_PREFERRED;
//             pResourceList->Descriptors[0].Type = CmResourceTypeConfigData;
//             pResourceList->Descriptors[0].u.ConfigData.Priority = ulPriority;
//
//             *pulLogConfTag = ulIndex;
//         }
//         else
//         {
//             ret = CR_FAILURE;
//             goto done;
//         }
//     }
//
//     /* Store the configuration */
//     if (RegSetValueEx(hConfigKey,
//                       szValueNameBuffer,
//                       0,
//                       RegDataType,
//                       pDataBuffer,
//                       ulDataSize + ulNewSize) != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
// done:
//     if (pDataBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, pDataBuffer);
//
//     if (hConfigKey != NULL)
//         RegCloseKey(hConfigKey);
//
//     DPRINT("PNP_AddEmptyLogConf() returns %lu\n", ret);
//
//     return ret;
// }
//
//
// /* Function 43 */
// DWORD
// WINAPI
// PNP_FreeLogConf(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfType,
//     DWORD ulLogConfTag,
//     DWORD ulFlags)
// {
//     HKEY hConfigKey = NULL;
//     DWORD RegDataType = 0;
//     ULONG ulDataSize = 0;
//     LPBYTE pDataBuffer = NULL;
//     WCHAR szValueNameBuffer[LOGCONF_NAME_BUFFER_SIZE];
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_FreeLogConf(%p %S %lu %lu 0x%08lx)\n",
//            hBinding, pDeviceID, ulLogConfType, ulLogConfTag, ulFlags);
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVNODE;
//
//     ret = OpenConfigurationKey(pDeviceID,
//                                ulLogConfType,
//                                &hConfigKey);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("OpenConfigurationKey() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     ret = GetConfigurationData(hConfigKey,
//                                ulLogConfType,
//                                &RegDataType,
//                                &ulDataSize,
//                                &pDataBuffer,
//                                LOGCONF_NAME_BUFFER_SIZE,
//                                szValueNameBuffer);
//     if (ret != CR_SUCCESS)
//     {
//         ret = CR_INVALID_LOG_CONF;
//         goto done;
//     }
//
//     if (RegDataType == REG_RESOURCE_LIST)
//     {
//         PCM_RESOURCE_LIST pResourceList = (PCM_RESOURCE_LIST)pDataBuffer;
//
//         if (pResourceList->Count <= 1)
//         {
//             /* Delete the key if there is only one or no configuration in the key */
//             DPRINT("Delete value %S\n", szValueNameBuffer);
//             RegDeleteValue(hConfigKey, szValueNameBuffer);
//         }
//         else
//         {
//             PCM_FULL_RESOURCE_DESCRIPTOR pResource = NULL, pNextResource = NULL;
//             ULONG ulIndex, ulMoveSize;
//             DWORD dwError;
//
//             /* Fail if we try to delete an entry outside of our resource list */
//             if (ulLogConfTag >= pResourceList->Count)
//             {
//                 ret = CR_INVALID_LOG_CONF;
//                 goto done;
//             }
//
//             /* Get a pointer to the resource descriptor to be deleted */
//             pResource = (PCM_FULL_RESOURCE_DESCRIPTOR)&pResourceList->List[0];
//             for (ulIndex = 0; ulIndex < ulLogConfTag; ulIndex++)
//                 pResource = NextResourceDescriptor(pResource);
//
//             /* Delete the resource descriptor */
//             if (ulLogConfTag == pResourceList->Count - 1)
//             {
//                 /* Delete the last resource descriptor */
//                 ulDataSize = (ULONG)((ULONG_PTR)pResource - (ULONG_PTR)pResourceList);
//             }
//             else
//             {
//                 pNextResource = NextResourceDescriptor(pResource);
//
//                 ulMoveSize = ulDataSize - (DWORD)((ULONG_PTR)pNextResource - (ULONG_PTR)pResourceList);
//                 MoveMemory!(pResource, pNextResource, ulMoveSize);
//
//                 ulDataSize -= (DWORD)((ULONG_PTR)pNextResource - (ULONG_PTR)pResource);
//             }
//
//             pResourceList->Count--;
//
//             /* Store the new configuration */
//             dwError = RegSetValueEx(hConfigKey,
//                                     szValueNameBuffer,
//                                     0,
//                                     RegDataType,
//                                     pDataBuffer,
//                                     ulDataSize);
//             if (dwError != ERROR_SUCCESS)
//             {
//                 ret = CR_REGISTRY_ERROR;
//                 goto done;
//             }
//         }
//     }
//     else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//     {
//         if (((PIO_RESOURCE_REQUIREMENTS_LIST)pDataBuffer)->AlternativeLists <= 1)
//         {
//             /* Delete the key if there is only one or no configuration in the key */
//             DPRINT("Delete value %S\n", szValueNameBuffer);
//             RegDeleteValue(hConfigKey, szValueNameBuffer);
//         }
//         else
//         {
//             /* FIXME */
//         }
//     }
//     else
//     {
//         ret = CR_FAILURE;
//         goto done;
//     }
//
// done:
//     if (pDataBuffer != NULL)
//         HeapFree(GetProcessHeap(), 0, pDataBuffer);
//
//     if (hConfigKey != NULL)
//         RegCloseKey(hConfigKey);
//
//     DPRINT("PNP_FreeLogConf() returns %lu\n", ret);
//
//     return ret;
// }
//
//
// /* Function 44 */
// DWORD
// WINAPI
// PNP_GetFirstLogConf(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfType,
//     DWORD *pulLogConfTag,
//     DWORD ulFlags)
// {
//     HKEY hConfigKey = NULL;
//     DWORD RegDataType = 0;
//     ULONG ulDataSize = 0;
//     LPBYTE lpData = NULL;
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_GetFirstLogConf(%p %S %lu %p 0x%08lx)\n",
//            hBinding, pDeviceID, ulLogConfType, pulLogConfTag, ulFlags);
//
//     if (pulLogConfTag == NULL)
//         return CR_INVALID_POINTER;
//
//     *pulLogConfTag = (DWORD)0;
//
//     if (ulFlags & ~LOG_CONF_BITS)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     ret = OpenConfigurationKey(pDeviceID,
//                                ulLogConfType,
//                                &hConfigKey);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("OpenConfigurationKey() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     ret = GetConfigurationData(hConfigKey,
//                                ulLogConfType,
//                                &RegDataType,
//                                &ulDataSize,
//                                &lpData,
//                                0,
//                                NULL);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("GetConfigurationData() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     DPRINT1("Data size %lu\n", ulDataSize);
//     if (ulDataSize == 0 || lpData == NULL)
//     {
//         DPRINT1("No config data available!\n");
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     /* Get the first tag */
//     if (RegDataType == REG_RESOURCE_LIST)
//     {
//         DPRINT("REG_RESOURCE_LIST->Count %lu\n", ((PCM_RESOURCE_LIST)lpData)->Count);
//
//         /* Fail, if we do not have any resource */
//         if (((PCM_RESOURCE_LIST)lpData)->Count == 0)
//         {
//             DPRINT1("No resource descriptors!\n");
//             ret = CR_NO_MORE_LOG_CONF;
//             goto done;
//         }
//     }
//     else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//     {
//         DPRINT("REG_RESOURCE_REQUIREMENTS_LIST->AlternativeLists %lu\n",
//                ((PIO_RESOURCE_REQUIREMENTS_LIST)lpData)->AlternativeLists);
//
//         /* Fail, if we do not have any requirements */
//         if (((PIO_RESOURCE_REQUIREMENTS_LIST)lpData)->AlternativeLists == 0)
//         {
//             ret = CR_NO_MORE_LOG_CONF;
//             goto done;
//         }
//     }
//
// done:
//     if (lpData != NULL)
//         HeapFree(GetProcessHeap(), 0, lpData);
//
//     if (hConfigKey != NULL)
//         RegCloseKey(hConfigKey);
//
//     DPRINT("PNP_GetFirstLogConf() returns %lu\n", ret);
//
//     return ret;
// }
//
//
// /* Function 45 */
// DWORD
// WINAPI
// PNP_GetNextLogConf(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfType,
//     DWORD ulCurrentTag,
//     DWORD *pulNextTag,
//     DWORD ulFlags)
// {
//     HKEY hConfigKey = NULL;
//     DWORD RegDataType = 0;
//     ULONG ulDataSize = 0;
//     LPBYTE lpData = NULL;
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT("PNP_GetNextLogConf(%p %S %lu %lu %p 0x%08lx)\n",
//            hBinding, pDeviceID, ulLogConfType, ulCurrentTag, pulNextTag, ulFlags);
//
//     if (pulNextTag == NULL)
//         return CR_INVALID_POINTER;
//
//     *pulNextTag = (DWORD)0;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     ret = OpenConfigurationKey(pDeviceID,
//                                ulLogConfType,
//                                &hConfigKey);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("OpenConfigurationKey() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     ret = GetConfigurationData(hConfigKey,
//                                ulLogConfType,
//                                &RegDataType,
//                                &ulDataSize,
//                                &lpData,
//                                0,
//                                NULL);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("GetConfigurationData() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     DPRINT("Data size %lu\n", ulDataSize);
//
//     if (ulDataSize == 0 || lpData == NULL)
//     {
//         DPRINT1("No config data available!\n");
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     /* Check if the next entry is available */
//     if (RegDataType == REG_RESOURCE_LIST)
//     {
//         DPRINT("REG_RESOURCE_LIST->Count %lu\n", ((PCM_RESOURCE_LIST)lpData)->Count);
//
//         /* Fail, if we are beyond the end of the list */
//         if (ulCurrentTag >= ((PCM_RESOURCE_LIST)lpData)->Count)
//         {
//             ret = CR_INVALID_LOG_CONF;
//             goto done;
//         }
//
//         /* Indicate that we reached the end of the list */
//         if (ulCurrentTag == ((PCM_RESOURCE_LIST)lpData)->Count - 1)
//         {
//             ret = CR_NO_MORE_LOG_CONF;
//             goto done;
//         }
//     }
//     else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//     {
//         DPRINT("REG_RESOURCE_REQUIREMENTS_LIST->AlternativeLists %lu\n",
//                ((PIO_RESOURCE_REQUIREMENTS_LIST)lpData)->AlternativeLists);
//
//         /* Fail, if we are beyond the end of the list */
//         if (ulCurrentTag >= ((PIO_RESOURCE_REQUIREMENTS_LIST)lpData)->AlternativeLists)
//         {
//             ret = CR_INVALID_LOG_CONF;
//             goto done;
//         }
//
//         /* Indicate that we reached the end of the list */
//         if (ulCurrentTag == ((PIO_RESOURCE_REQUIREMENTS_LIST)lpData)->AlternativeLists - 1)
//         {
//             ret = CR_NO_MORE_LOG_CONF;
//             goto done;
//         }
//     }
//
//     /* Return the next tag value */
//     *pulNextTag = ulCurrentTag + 1;
//
// done:
//     if (lpData != NULL)
//         HeapFree(GetProcessHeap(), 0, lpData);
//
//     if (hConfigKey != NULL)
//         RegCloseKey(hConfigKey);
//
//     DPRINT("PNP_GetNextLogConf() returns %lu\n", ret);
//
//     return ret;
// }
//
//
// /* Function 46 */
// DWORD
// WINAPI
// PNP_GetLogConfPriority(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulType,
//     DWORD ulTag,
//     DWORD *pPriority,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 47 */
// DWORD
// WINAPI
// PNP_AddResDes(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID ResourceID,
//     DWORD *pulResourceTag,
//     BYTE *ResourceData,
//     PNP_RPC_BUFFER_SIZE ResourceLen,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 48 */
// DWORD
// WINAPI
// PNP_FreeResDes(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID ResourceID,
//     DWORD ulResourceTag,
//     DWORD *pulPreviousResType,
//     DWORD *pulPreviousResTag,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 49 */
// DWORD
// WINAPI
// PNP_GetNextResDes(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID ResourceID,
//     DWORD ulResourceTag,
//     DWORD *pulNextResType,
//     DWORD *pulNextResTag,
//     DWORD ulFlags)
// {
//     HKEY hConfigKey = NULL;
//     DWORD RegDataType = 0;
//     ULONG ulDataSize = 0;
//     LPBYTE lpData = NULL;
//     CONFIGRET ret = CR_SUCCESS;
//
//     DPRINT1("PNP_GetNextResDes(%p %S 0x%lx %lu %lu %ul %p %p 0x%08lx)\n",
//            hBinding, pDeviceID, ulLogConfTag, ulLogConfType, ResourceID,
//            ulResourceTag, pulNextResType, pulNextResTag, ulFlags);
//
//     if (pulNextResType == NULL)
//         return CR_INVALID_POINTER;
//
//     *pulNextResType = 0;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     ret = OpenConfigurationKey(pDeviceID,
//                                ulLogConfType,
//                                &hConfigKey);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("OpenConfigurationKey() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     ret = GetConfigurationData(hConfigKey,
//                                ulLogConfType,
//                                &RegDataType,
//                                &ulDataSize,
//                                &lpData,
//                                0,
//                                NULL);
//     if (ret != CR_SUCCESS)
//     {
//         DPRINT1("GetConfigurationData() failed (Error %lu)\n", ret);
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     DPRINT1("Data size %lu\n", ulDataSize);
//
//     if (ulDataSize == 0 || lpData == NULL)
//     {
//         DPRINT1("No config data available!\n");
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
//     /* Get the next resource descriptor */
//     if (RegDataType == REG_RESOURCE_LIST)
//     {
//         DPRINT1("FIXME: REG_RESOURCE_LIST\n");
//         /* FIXME */
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//     else if (RegDataType == REG_RESOURCE_REQUIREMENTS_LIST)
//     {
//         DPRINT1("FIXME: REG_RESOURCE_REQUIREMENTS_LIST\n");
//         /* FIXME */
//         ret = CR_NO_MORE_LOG_CONF;
//         goto done;
//     }
//
// done:
//     if (lpData != NULL)
//         HeapFree(GetProcessHeap(), 0, lpData);
//
//     if (hConfigKey != NULL)
//         RegCloseKey(hConfigKey);
//
//     DPRINT1("PNP_GetNextResDes() returns %lu\n", ret);
//
//     return ret;
// }
//
//
// /* Function 50 */
// DWORD
// WINAPI
// PNP_GetResDesData(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID ResourceID,
//     DWORD ulResourceTag,
//     BYTE *Buffer,
//     PNP_RPC_BUFFER_SIZE BufferLen,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 51 */
// DWORD
// WINAPI
// PNP_GetResDesDataSize(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID ResourceID,
//     DWORD ulResourceTag,
//     DWORD *pulSize,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 52 */
// DWORD
// WINAPI
// PNP_ModifyResDes(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     DWORD ulLogConfTag,
//     DWORD ulLogConfType,
//     RESOURCEID CurrentResourceID,
//     RESOURCEID NewResourceID,
//     DWORD ulResourceTag,
//     BYTE *ResourceData,
//     PNP_RPC_BUFFER_SIZE ResourceLen,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 53 */
// DWORD
// WINAPI
// PNP_DetectResourceConflict(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     RESOURCEID ResourceID,
//     BYTE *ResourceData,
//     PNP_RPC_BUFFER_SIZE ResourceLen,
//     BOOL *pbConflictDetected,
//     DWORD ulFlags)
// {
//     DPRINT("PNP_DetectResourceConflict()\n");
//
//     if (pbConflictDetected != NULL)
//         *pbConflictDetected = FALSE;
//
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 54 */
// DWORD
// WINAPI
// PNP_QueryResConfList(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     RESOURCEID ResourceID,
//     BYTE *ResourceData,
//     PNP_RPC_BUFFER_SIZE ResourceLen,
//     BYTE *Buffer,
//     PNP_RPC_BUFFER_SIZE BufferLen,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 55 */
// DWORD
// WINAPI
// PNP_SetHwProf(
//     handle_t hBinding,
//     DWORD ulHardwareProfile,
//     DWORD ulFlags)
// {
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 56 */
// DWORD
// WINAPI
// PNP_QueryArbitratorFreeData(
//     handle_t hBinding,
//     BYTE *pData,
//     DWORD DataLen,
//     LPWSTR pDeviceID,
//     RESOURCEID ResourceID,
//     DWORD ulFlags)
// {
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 57 */
// DWORD
// WINAPI
// PNP_QueryArbitratorFreeSize(
//     handle_t hBinding,
//     DWORD *pulSize,
//     LPWSTR pDeviceID,
//     RESOURCEID ResourceID,
//     DWORD ulFlags)
// {
//     if (pulSize != NULL)
//         *pulSize = 0;
//
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 58 */
// CONFIGRET
// WINAPI
// PNP_RunDetection(
//     handle_t hBinding,
//     DWORD ulFlags)
// {
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 59 */
// DWORD
// WINAPI
// PNP_RegisterNotification(
//     handle_t hBinding,
//     DWORD_PTR hRecipient,
//     LPWSTR pszName,
//     BYTE *pNotificationFilter,
//     DWORD ulNotificationFilterSize,
//     DWORD ulFlags,
//     PNP_NOTIFY_HANDLE *pNotifyHandle,
//     DWORD ulProcessId,
//     DWORD *pulUnknown9)
// {
//     PDEV_BROADCAST_DEVICEINTERFACE_W pBroadcastDeviceInterface;
//     PDEV_BROADCAST_HANDLE pBroadcastDeviceHandle;
//     PNOTIFY_ENTRY pNotifyData = NULL;
//     DWORD ret = CR_SUCCESS;
//
//     DPRINT1("PNP_RegisterNotification(%p %p '%S' %p %lu 0x%lx %p %lx %p)\n",
//            hBinding, hRecipient, pszName, pNotificationFilter,
//            ulNotificationFilterSize, ulFlags, pNotifyHandle, ulProcessId, pulUnknown9);
//
//     if (pNotifyHandle == NULL)
//         return CR_INVALID_POINTER;
//
//     *pNotifyHandle = NULL;
//
//     if (pNotificationFilter == NULL ||
//         pulUnknown9 == NULL)
//         return CR_INVALID_POINTER;
//
//     if (ulFlags & ~0x7)
//         return CR_INVALID_FLAG;
//
//     if ((ulNotificationFilterSize < sizeof(DEV_BROADCAST_HDR)) ||
//         (((PDEV_BROADCAST_HDR)pNotificationFilter)->dbch_size < sizeof(DEV_BROADCAST_HDR)))
//         return CR_INVALID_DATA;
//
//     if (((PDEV_BROADCAST_HDR)pNotificationFilter)->dbch_devicetype == DBT_DEVTYP_DEVICEINTERFACE)
//     {
//         DPRINT1("DBT_DEVTYP_DEVICEINTERFACE\n");
//         pBroadcastDeviceInterface = (PDEV_BROADCAST_DEVICEINTERFACE_W)pNotificationFilter;
//
//         if ((ulNotificationFilterSize < sizeof(DEV_BROADCAST_DEVICEINTERFACE_W)) ||
//             (pBroadcastDeviceInterface->dbcc_size < sizeof(DEV_BROADCAST_DEVICEINTERFACE_W)))
//             return CR_INVALID_DATA;
//
//         pNotifyData = RtlAllocateHeap(GetProcessHeap(), HEAP_ZERO_MEMORY, sizeof(NOTIFY_ENTRY));
//         if (pNotifyData == NULL)
//         {
//             ret = CR_OUT_OF_MEMORY;
//             goto done;
//         }
//
//         pNotifyData->dwType = CLASS_NOTIFICATION;
//
//         if (pszName != NULL)
//         {
//             pNotifyData->pszName = RtlAllocateHeap(GetProcessHeap(),
//                                                    HEAP_ZERO_MEMORY,
//                                                    (wcslen(pszName) + 1) * sizeof(WCHAR));
//             if (pNotifyData->pszName == NULL)
//             {
//                 ret = CR_OUT_OF_MEMORY;
//                 goto done;
//             }
//
//             wcscpy(pNotifyData->pszName, pszName);
//         }
//
//         if ((ulFlags & DEVICE_NOTIFY_SERVICE_HANDLE) == DEVICE_NOTIFY_WINDOW_HANDLE)
//         {
//             pNotifyData->hRecipient = hRecipient;
//         }
//         else
//         {
//             DPRINT("Validate service: %S\n", pszName);
//             if (I_ScValidatePnpService(NULL,
//                                        pszName,
//                                        (SERVICE_STATUS_HANDLE *)&pNotifyData->hRecipient) != ERROR_SUCCESS)
//             {
//                 DPRINT1("I_ScValidatePnpService failed!\n");
//                 ret = CR_INVALID_DATA;
//                 goto done;
//             }
//
//             DPRINT("Service status handle: %p\n", pNotifyData->hRecipient);
//         }
//
//         pNotifyData->ulFlags = ulFlags;
//
//         if ((ulFlags & DEVICE_NOTIFY_ALL_INTERFACE_CLASSES) == 0)
//         {
//             CopyMemory!(&pNotifyData->ClassGuid,
//                        &pBroadcastDeviceInterface->dbcc_classguid,
//                        sizeof(GUID));
//         }
//
//         /* Add the entry to the notification list */
//         RtlAcquireResourceExclusive(&NotificationListLock, TRUE);
//         InsertTailList(&NotificationListHead, &pNotifyData->ListEntry);
//         RtlReleaseResource(&NotificationListLock);
//
//         DPRINT("pNotifyData: %p\n", pNotifyData);
//         *pNotifyHandle = (PNP_NOTIFY_HANDLE)pNotifyData;
//     }
//     else if (((PDEV_BROADCAST_HDR)pNotificationFilter)->dbch_devicetype == DBT_DEVTYP_HANDLE)
//     {
//         DPRINT1("DBT_DEVTYP_HANDLE\n");
//         pBroadcastDeviceHandle = (PDEV_BROADCAST_HANDLE)pNotificationFilter;
//
//         if ((ulNotificationFilterSize < sizeof(DEV_BROADCAST_HANDLE)) ||
//             (pBroadcastDeviceHandle->dbch_size < sizeof(DEV_BROADCAST_HANDLE)))
//         {
//             ret = CR_INVALID_DATA;
//             goto done;
//         }
//
//         if (ulFlags & DEVICE_NOTIFY_ALL_INTERFACE_CLASSES)
//         {
//             ret = CR_INVALID_FLAG;
//             goto done;
//         }
//
//         /* FIXME */
//     }
//     else
//     {
//         DPRINT1("Invalid device type %lu\n", ((PDEV_BROADCAST_HDR)pNotificationFilter)->dbch_devicetype);
//         ret = CR_INVALID_DATA;
//     }
//
// done:
//     if (ret != CR_SUCCESS)
//     {
//         if (pNotifyData)
//         {
//             if (pNotifyData->pszName)
//                 RtlFreeHeap(GetProcessHeap(), 0, pNotifyData->pszName);
//
//             RtlFreeHeap(GetProcessHeap(), 0, pNotifyData);
//         }
//     }
//
//     return ret;
// }
//
//
// /* Function 60 */
// DWORD
// WINAPI
// PNP_UnregisterNotification(
//     handle_t hBinding,
//     PNP_NOTIFY_HANDLE *pNotifyHandle)
// {
//     PNOTIFY_ENTRY pEntry;
//
//     DPRINT1("PNP_UnregisterNotification(%p %p)\n",
//            hBinding, pNotifyHandle);
//
//     pEntry = (PNOTIFY_ENTRY)*pNotifyHandle;
//     if (pEntry == NULL)
//         return CR_INVALID_DATA;
//
//     RtlAcquireResourceExclusive(&NotificationListLock, TRUE);
//     RemoveEntryList(&pEntry->ListEntry);
//     RtlReleaseResource(&NotificationListLock);
//
//     if (pEntry->pszName)
//         RtlFreeHeap(RtlGetProcessHeap!(), 0, pEntry->pszName);
//     RtlFreeHeap(RtlGetProcessHeap!(), 0, pEntry);
//     *pNotifyHandle = NULL;
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 61 */
// DWORD
// WINAPI
// PNP_GetCustomDevProp(
//     handle_t hBinding,
//     LPWSTR pDeviceID,
//     LPWSTR CustomPropName,
//     DWORD *pulRegDataType,
//     BYTE *Buffer,
//     PNP_RPC_STRING_LEN *pulTransferLen,
//     PNP_RPC_STRING_LEN *pulLength,
//     DWORD ulFlags)
// {
//     HKEY hDeviceKey = NULL;
//     HKEY hParamKey = NULL;
//     LONG lError;
//     CONFIGRET ret = CR_SUCCESS;
//
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT("PNP_GetCustomDevProp() called\n");
//
//     if (pulTransferLen == NULL || pulLength == NULL)
//     {
//         ret = CR_INVALID_POINTER;
//         goto done;
//     }
//
//     if (ulFlags & ~CM_CUSTOMDEVPROP_BITS)
//     {
//         ret = CR_INVALID_FLAG;
//         goto done;
//     }
//
//     if (!IsValidDeviceInstanceID(pDeviceID))
//         return CR_INVALID_DEVINST;
//
//     if (*pulLength < *pulTransferLen)
//         *pulLength = *pulTransferLen;
//
//     *pulTransferLen = 0;
//
//     lError = RegOpenKeyExW(hEnumKey,
//                            pDeviceID,
//                            0,
//                            KEY_READ,
//                            &hDeviceKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     lError = RegOpenKeyExW(hDeviceKey,
//                            L"Device Parameters",
//                            0,
//                            KEY_READ,
//                            &hParamKey);
//     if (lError != ERROR_SUCCESS)
//     {
//         ret = CR_REGISTRY_ERROR;
//         goto done;
//     }
//
//     lError = RegQueryValueExW(hParamKey,
//                               CustomPropName,
//                               NULL,
//                               pulRegDataType,
//                               Buffer,
//                               pulLength);
//     if (lError != ERROR_SUCCESS)
//     {
//         if (lError == ERROR_MORE_DATA)
//         {
//             ret = CR_BUFFER_SMALL;
//         }
//         else
//         {
//             *pulLength = 0;
//             ret = CR_NO_SUCH_VALUE;
//         }
//     }
//
// done:
//     if (ret == CR_SUCCESS)
//         *pulTransferLen = *pulLength;
//
//     if (hParamKey != NULL)
//         RegCloseKey(hParamKey);
//
//     if (hDeviceKey != NULL)
//         RegCloseKey(hDeviceKey);
//
//     DPRINT("PNP_GetCustomDevProp() done (returns %lx)\n", ret);
//
//     return ret;
// }
//
//
// /* Function 62 */
// DWORD
// WINAPI
// PNP_GetVersionInternal(
//     handle_t hBinding,
//     WORD *pwVersion)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     *pwVersion = UMPNPMGR_WINVER;
//     return CR_SUCCESS;
// }
//
//
// /* Function 63 */
// DWORD
// WINAPI
// PNP_GetBlockedDriverInfo(
//     handle_t hBinding,
//     BYTE *Buffer,
//     PNP_RPC_BUFFER_SIZE *pulTransferLen,
//     PNP_RPC_BUFFER_SIZE *pulLength,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 64 */
// DWORD
// WINAPI
// PNP_GetServerSideDeviceInstallFlags(
//     handle_t hBinding,
//     DWORD *pulSSDIFlags,
//     DWORD ulFlags)
// {
//     UNREFERENCED_PARAMETER!(hBinding);
//
//     DPRINT1("PNP_GetServerSideDeviceInstallFlags(%p %p %lu)\n",
//             hBinding, pulSSDIFlags, ulFlags);
//
//     if (pulSSDIFlags == NULL)
//         return CR_INVALID_POINTER;
//
//     if (ulFlags != 0)
//         return CR_INVALID_FLAG;
//
//     /* FIXME */
//     *pulSSDIFlags = 0;
//
//     return CR_SUCCESS;
// }
//
//
// /* Function 65 */
// DWORD
// WINAPI
// PNP_GetObjectPropKeys(
//     handle_t hBinding,
//     LPWSTR ObjectName,
//     DWORD ObjectType,
//     LPWSTR PropertyCultureName,
//     PNP_PROP_COUNT *PropertyCount,
//     PNP_PROP_COUNT *TransferLen,
//     DEVPROPKEY *PropertyKeys,
//     DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 66 */
// DWORD
// WINAPI
// PNP_GetObjectProp(
//     handle_t hBinding,
//     LPWSTR ObjectName,
//     DWORD ObjectType,
//     LPWSTR PropertyCultureName,
//     const DEVPROPKEY *PropertyKey,
//     DEVPROPTYPE *PropertyType,
//     PNP_PROP_SIZE *PropertySize,
//     PNP_PROP_SIZE *TransferLen,
//     BYTE *PropertyBuffer,
//     DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 67 */
// DWORD
// WINAPI
// PNP_SetObjectProp(
//     handle_t hBinding,
//     LPWSTR ObjectName,
//     DWORD ObjectType,
//     LPWSTR PropertyCultureName,
//     const DEVPROPKEY *PropertyKey,
//     DEVPROPTYPE PropertyType,
//     PNP_PROP_SIZE PropertySize,
//     BYTE *PropertyBuffer,
//     DWORD Flags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 68 */
// DWORD
// WINAPI
// PNP_InstallDevInst(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 69 */
// DWORD
// WINAPI
// PNP_ApplyPowerSettings(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 70 */
// DWORD
// WINAPI
// PNP_DriverStoreAddDriverPackage(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 71 */
// DWORD
// WINAPI
// PNP_DriverStoreDeleteDriverPackage(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 72 */
// DWORD
// WINAPI
// PNP_RegisterServiceNotification(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 73 */
// DWORD
// WINAPI
// PNP_SetActiveService(
//     handle_t hBinding,
//     LPWSTR pszFilter,
//     DWORD ulFlags)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
//
// /* Function 74 */
// DWORD
// WINAPI
// PNP_DeleteServiceDevices(
//     handle_t hBinding)
// {
//     UNIMPLEMENTED;
//     return CR_CALL_NOT_IMPLEMENTED;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
