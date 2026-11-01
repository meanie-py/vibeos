use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut BackSlash: [WCHAR; 2] = ['\\' as i32 as WCHAR, 0 as ::core::ffi::c_int as WCHAR];

static mut ClassGUID: [WCHAR; 10] = [
    'C' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    's' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'G' as i32 as WCHAR,
    'U' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'D' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut DotCoInstallers: [WCHAR; 14] = [
    '.' as i32 as WCHAR,
    'C' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'I' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    's' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'l' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut InfDirectory: [WCHAR; 5] = [
    'i' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    '\\' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut Version: [WCHAR; 8] = [
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    's' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut INF_MANUFACTURER: [WCHAR; 13] = [
    'M' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'n' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'f' as i32 as WCHAR,
    'a' as i32 as WCHAR,
    'c' as i32 as WCHAR,
    't' as i32 as WCHAR,
    'u' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut INF_PROVIDER: [WCHAR; 9] = [
    'P' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'o' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'd' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

static mut INF_DRIVER_VER: [WCHAR; 10] = [
    'D' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'i' as i32 as WCHAR,
    'v' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    'V' as i32 as WCHAR,
    'e' as i32 as WCHAR,
    'r' as i32 as WCHAR,
    0 as ::core::ffi::c_int as WCHAR,
];

unsafe extern "C" fn ReferenceInfFile(mut infFile: *mut InfFileDetails) {
    InterlockedIncrement(
        core::ptr::addr_of_mut!((*infFile).References) as *mut ::core::ffi::c_long,
    );
}
#[no_mangle]
pub unsafe extern "C" fn DereferenceInfFile(infFile: *mut InfFileDetails) {
    if InterlockedDecrement(core::ptr::addr_of_mut!((*infFile).References) as *mut ::core::ffi::c_long) == 0 {
        SetupCloseInfFile((*infFile).hInf);
        HeapFree(GetProcessHeap(), 0, infFile as PVOID);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CreateInfFileDetails(FullInfFileName: LPCWSTR) -> *mut InfFileDetails {
    let mut details: *mut InfFileDetails = ::core::ptr::null_mut::<InfFileDetails>();
    let mut last: PWCHAR = ::core::ptr::null_mut::<WCHAR>();
    let mut Needed: DWORD = 0;
    Needed = (core::mem::offset_of!(InfFileDetails, szData) as DWORD)
        .wrapping_add((strlenW(FullInfFileName) as DWORD).wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD))
        .wrapping_add(core::mem::size_of_val(&UNICODE_NULL) as DWORD);

    details = HeapAlloc(GetProcessHeap(), 0, (Needed) as SIZE_T) as *mut InfFileDetails;
    if details.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return ::core::ptr::null_mut::<InfFileDetails>();
    }

    memset(details as *mut ::core::ffi::c_void, 0 as ::core::ffi::c_int, Needed as usize);
    strcpyW(&raw mut (*details).szData as *mut WCHAR, FullInfFileName);
    last = strrchrW(&raw mut (*details).szData as *mut WCHAR, '\\' as WCHAR) as PWCHAR;
    if !last.is_null() {
        (*details).DirectoryName = &raw mut (*details).szData as *mut WCHAR as PCWSTR;
        (*details).FileName = last.offset(1 as isize) as PCWSTR;
        *last = 0 as WCHAR;
    } else {
        (*details).FileName = &raw mut (*details).szData as *mut WCHAR as PCWSTR;
    }

    ReferenceInfFile(details);
    (*details).hInf = SetupOpenInfFileW(
        FullInfFileName as PCWSTR,
        ::core::ptr::null::<WCHAR>(),
        INF_STYLE_WIN4 as DWORD,
        ::core::ptr::null_mut::<::core::ffi::c_uint>(),
    );
    if (*details).hInf == INVALID_HANDLE_VALUE {
        HeapFree(GetProcessHeap(), 0, details as PVOID);
        return ::core::ptr::null_mut::<InfFileDetails>();
    }
    return details;
}
#[no_mangle]
pub unsafe extern "C" fn DestroyDriverInfoElement(driverInfo: *mut DriverInfoElement) -> BOOL {
    DereferenceInfFile((*driverInfo).InfFileDetails);
    HeapFree(GetProcessHeap(), 0, (*driverInfo).MatchingId as PVOID);
    HeapFree(GetProcessHeap(), 0, driverInfo as PVOID);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn AddKnownDriverToList(
    mut DriverListHead: PLIST_ENTRY,
    mut DriverType_local: DWORD,
    mut ClassGuid: LPGUID,
    mut InfFileDetails: *mut InfFileDetails,
    mut InfFile: LPCWSTR,
    mut SectionName: LPCWSTR,
    mut DriverDescription: LPCWSTR,
    mut ProviderName: LPCWSTR,
    mut ManufacturerName: LPCWSTR,
    mut MatchingId: LPCWSTR,
    mut DriverDate: FILETIME,
    mut DriverVersion: DWORDLONG,
    mut Rank: DWORD,
) -> BOOL {
    let mut driverInfo: *mut DriverInfoElement = core::ptr::null_mut::<DriverInfoElement>();
    let mut hFile: HANDLE = INVALID_HANDLE_VALUE;
    let mut Result: BOOL = (FALSE as i32);
    let mut PreviousEntry: PLIST_ENTRY = core::ptr::null_mut::<_LIST_ENTRY>();
    let mut ret: BOOL = (FALSE as i32);

    'cleanup: {
        driverInfo = HeapAlloc(
            GetProcessHeap(),
            0,
            core::mem::size_of::<DriverInfoElement>() as SIZE_T,
        ) as *mut DriverInfoElement;
        if driverInfo.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }
        memset(
            driverInfo as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<DriverInfoElement>() as size_t,
        );

        (*driverInfo).Params.cbSize =
            core::mem::size_of::<SP_DRVINSTALL_PARAMS>() as DWORD;
        (*driverInfo).Params.Reserved = driverInfo as ULONG_PTR as DWORD;

        (*driverInfo).Details.cbSize =
            core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as DWORD;
        (*driverInfo).Details.Reserved = driverInfo as ULONG_PTR;

        /* Copy InfFileName field */
        lstrcpynW(
            &raw mut (*driverInfo).Details.InfFileName as LPWSTR,
            InfFile,
            (MAX_PATH - 1) as i32,
        );
        (*driverInfo).Details.InfFileName[(MAX_PATH - 1) as usize] =
            '\0' as i32 as WCHAR;

        /* Fill InfDate field */
        hFile = CreateFileW(
            InfFile,
            GENERIC_READ,
            FILE_SHARE_READ,
            core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
            OPEN_EXISTING,
            0,
            NULL,
        );
        if hFile == INVALID_HANDLE_VALUE {
            break 'cleanup;
        }
        Result = GetFileTime(
            hFile,
            core::ptr::null_mut::<_FILETIME>(),
            core::ptr::null_mut::<_FILETIME>(),
            &raw mut (*driverInfo).Details.InfDate,
        );
        if Result == 0 {
            break 'cleanup;
        }

        /* Fill SectionName field */
        lstrcpynW(
            &raw mut (*driverInfo).Details.SectionName as LPWSTR,
            SectionName,
            (LINE_LEN as i32),
        );

        /* Fill DrvDescription field */
        lstrcpynW(
            &raw mut (*driverInfo).Details.DrvDescription as LPWSTR,
            DriverDescription,
            (LINE_LEN as i32),
        );

        /* Copy MatchingId information */
        if !MatchingId.is_null() {
            (*driverInfo).MatchingId = HeapAlloc(
                GetProcessHeap(),
                0,
                ((strlenW(MatchingId) as SIZE_T) + (1 as SIZE_T))
                    * (core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if (*driverInfo).MatchingId.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            RtlCopyMemory!(
                (*driverInfo).MatchingId as *mut core::ffi::c_void,
                MatchingId as *const core::ffi::c_void,
                ((strlenW(MatchingId) as size_t) + (1 as size_t))
                    * (core::mem::size_of::<WCHAR>() as size_t),
            );
        } else {
            (*driverInfo).MatchingId = core::ptr::null_mut::<WCHAR>();
        }

        (*driverInfo).Params.Rank = Rank;
        memcpy(
            &raw mut (*driverInfo).DriverDate as *mut core::ffi::c_void,
            &raw mut DriverDate as *const core::ffi::c_void,
            core::mem::size_of::<FILETIME>() as size_t,
        );
        memcpy(
            &raw mut (*driverInfo).ClassGuid as *mut core::ffi::c_void,
            ClassGuid as *const core::ffi::c_void,
            core::mem::size_of::<GUID>() as size_t,
        );
        (*driverInfo).Info.DriverType = DriverType_local;
        (*driverInfo).Info.Reserved = driverInfo as ULONG_PTR;
        lstrcpynW(
            &raw mut (*driverInfo).Info.Description as LPWSTR,
            &raw mut (*driverInfo).Details.DrvDescription as *mut WCHAR as LPCWSTR,
            (LINE_LEN - 1) as i32,
        );
        (*driverInfo).Info.Description[(LINE_LEN - 1) as usize] =
            '\0' as i32 as WCHAR;
        lstrcpynW(
            &raw mut (*driverInfo).Info.MfgName as LPWSTR,
            ManufacturerName,
            (LINE_LEN - 1) as i32,
        );
        (*driverInfo).Info.MfgName[(LINE_LEN - 1) as usize] =
            '\0' as i32 as WCHAR;
        if !ProviderName.is_null() {
            lstrcpynW(
                &raw mut (*driverInfo).Info.ProviderName as LPWSTR,
                ProviderName,
                (LINE_LEN - 1) as i32,
            );
            (*driverInfo).Info.ProviderName[(LINE_LEN - 1) as usize] =
                '\0' as i32 as WCHAR;
        } else {
            (*driverInfo).Info.ProviderName[0 as usize] =
                '\0' as i32 as WCHAR;
        }
        (*driverInfo).Info.DriverDate = DriverDate;
        (*driverInfo).Info.DriverVersion = DriverVersion;
        ReferenceInfFile(InfFileDetails);
        (*driverInfo).InfFileDetails = InfFileDetails;

        /* Insert current driver in driver list, according to its rank */
        PreviousEntry = (*DriverListHead).Flink as PLIST_ENTRY;
        while PreviousEntry != DriverListHead {
            let mut CurrentDriver: *mut DriverInfoElement =
                core::ptr::null_mut::<DriverInfoElement>();
            CurrentDriver = (PreviousEntry as ULONG_PTR).wrapping_sub(
                core::mem::offset_of!(DriverInfoElement, ListEntry) as ULONG_PTR,
            ) as *mut DriverInfoElement;
            if (*CurrentDriver).Params.Rank > Rank
                || (*CurrentDriver).Params.Rank == Rank
                    && (*CurrentDriver).DriverDate.QuadPart
                        < (*driverInfo).DriverDate.QuadPart
            {
                /* Insert before the current item */
                InsertHeadList(
                    (*PreviousEntry).Blink as PLIST_ENTRY,
                    &raw mut (*driverInfo).ListEntry,
                );
                break;
            }
            PreviousEntry = (*PreviousEntry).Flink as PLIST_ENTRY;
        }
        if PreviousEntry == DriverListHead {
            /* Insert at the end of the list */
            InsertTailList(DriverListHead, &raw mut (*driverInfo).ListEntry);
        }

        ret = (TRUE as i32);
    }

    /* cleanup: */
    if ret == 0 {
        if !driverInfo.is_null() {
            HeapFree(
                GetProcessHeap(),
                0,
                (*driverInfo).MatchingId as PVOID,
            );
        }
        HeapFree(
            GetProcessHeap(),
            0,
            driverInfo as PVOID,
        );
    }
    if hFile != INVALID_HANDLE_VALUE {
        CloseHandle(hFile);
    }

    return ret;
}

unsafe extern "C" fn AddDriverToList(
    mut DriverListHead: PLIST_ENTRY,
    mut DriverType_local: DWORD,
    mut ClassGuid: LPGUID,
    mut ContextDevice: INFCONTEXT,
    mut InfFileDetails: *mut InfFileDetails,
    mut InfFile: LPCWSTR,
    mut ProviderName: LPCWSTR,
    mut ManufacturerName: LPCWSTR,
    mut MatchingId: LPCWSTR,
    mut DriverDate: FILETIME,
    mut DriverVersion: DWORDLONG,
    mut Rank: DWORD,
) -> BOOL {
    let mut SectionName: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut DriverDescription: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut Result: BOOL = 0;
    let mut ret: BOOL = (FALSE as i32);

    'cleanup: {
        /* Read SectionName */
        SectionName = MyMalloc(LINE_LEN) as LPWSTR;
        if SectionName.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }
        ZeroMemory!(SectionName as *mut core::ffi::c_void, LINE_LEN);
        Result = SetupGetStringFieldW(
            core::ptr::addr_of_mut!(ContextDevice),
            1,
            SectionName,
            LINE_LEN,
            core::ptr::null_mut(),
        );
        if Result == 0 {
            break 'cleanup;
        }

        /* Read DriverDescription */
        DriverDescription = MyMalloc(LINE_LEN) as LPWSTR;
        if DriverDescription.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }
        ZeroMemory!(DriverDescription as *mut core::ffi::c_void, LINE_LEN);
        Result = SetupGetStringFieldW(
            core::ptr::addr_of_mut!(ContextDevice),
            0, /* Field index */
            DriverDescription,
            LINE_LEN,
            core::ptr::null_mut(),
        );

        ret = AddKnownDriverToList(
            DriverListHead,
            DriverType_local,
            ClassGuid,
            InfFileDetails,
            InfFile,
            SectionName,
            DriverDescription,
            ProviderName,
            ManufacturerName,
            MatchingId,
            DriverDate,
            DriverVersion,
            Rank,
        );
    }

    MyFree(SectionName as PVOID);
    MyFree(DriverDescription as PVOID);

    return ret;
}

unsafe extern "C" fn GetVersionInformationFromInfFile(
    hInf: HINF,
    ClassGuid: LPGUID,
    pProviderName: *mut LPWSTR,
    DriverDate: *mut FILETIME,
    DriverVersion: *mut DWORDLONG,
) -> BOOL {
    let mut RequiredSize: DWORD = 0;
    let mut guidW: [WCHAR; (MAX_GUID_STRING_LEN + 1) as _] =
        [0; (MAX_GUID_STRING_LEN + 1) as _];
    let mut DriverVer: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut ProviderName: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pComma: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut pVersion: LPWSTR = core::ptr::null_mut::<WCHAR>();
    let mut SystemTime: SYSTEMTIME = core::mem::zeroed::<SYSTEMTIME>();
    let mut Result: BOOL = 0;
    let mut ret: BOOL = (FALSE as i32);

    'cleanup: {
        /* Get class Guid */
        if SetupGetLineTextW(
            core::ptr::null_mut(), /* Context */
            hInf,
            core::ptr::addr_of!(Version) as PCWSTR,
            core::ptr::addr_of!(ClassGUID) as PCWSTR,
            core::ptr::addr_of_mut!(guidW) as PWSTR,
            core::mem::size_of::<[WCHAR; (MAX_GUID_STRING_LEN + 1) as _]>() as DWORD,
            core::ptr::null_mut::<DWORD>(), /* Required size */
        ) == 0
        {
            break 'cleanup;
        }
        guidW[37 as usize] = '\0' as WCHAR; /* Replace the } by a NULL character */
        if UuidFromStringW(
            (core::ptr::addr_of_mut!(guidW) as *mut WCHAR).offset(1 as isize)
                as RPC_WSTR,
            ClassGuid as *mut UUID,
        ) != RPC_S_OK
        {
            SetLastError(ERROR_GEN_FAILURE);
            break 'cleanup;
        }

        /* Get provider name */
        Result = SetupGetLineTextW(
            core::ptr::null_mut(), /* Context */
            hInf,
            core::ptr::addr_of!(Version) as PCWSTR,
            core::ptr::addr_of!(INF_PROVIDER) as PCWSTR,
            core::ptr::null_mut::<WCHAR>(),
            0,
            &mut RequiredSize,
        );
        if Result != 0 {
            /* We know the needed buffer size */
            ProviderName = HeapAlloc(
                GetProcessHeap(),
                0,
                (RequiredSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if ProviderName.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            Result = SetupGetLineTextW(
                core::ptr::null_mut(), /* Context */
                hInf,
                core::ptr::addr_of!(Version) as PCWSTR,
                core::ptr::addr_of!(INF_PROVIDER) as PCWSTR,
                ProviderName as PWSTR,
                RequiredSize,
                &mut RequiredSize,
            );
        }
        if Result == 0 {
            break 'cleanup;
        }
        *pProviderName = ProviderName;

        /* Read the "DriverVer" value */
        Result = SetupGetLineTextW(
            core::ptr::null_mut(), /* Context */
            hInf,
            core::ptr::addr_of!(Version) as PCWSTR,
            core::ptr::addr_of!(INF_DRIVER_VER) as PCWSTR,
            core::ptr::null_mut::<WCHAR>(),
            0,
            &mut RequiredSize,
        );
        if Result != 0 {
            /* We know know the needed buffer size */
            DriverVer = HeapAlloc(
                GetProcessHeap(),
                0,
                (RequiredSize as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
            ) as LPWSTR;
            if DriverVer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
            Result = SetupGetLineTextW(
                core::ptr::null_mut(), /* Context */
                hInf,
                core::ptr::addr_of!(Version) as PCWSTR,
                core::ptr::addr_of!(INF_DRIVER_VER) as PCWSTR,
                DriverVer as PWSTR,
                RequiredSize,
                &mut RequiredSize,
            );
        } else {
            /* windows sets default date of 00/00/0000 when this directive is missing*/
            memset(
                DriverDate as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                core::mem::size_of::<FILETIME>() as size_t,
            );
            *DriverVersion = 0 as DWORDLONG;
            return (TRUE) as BOOL;
        }

        /* Get driver date and driver version, by analyzing the "DriverVer" value */
        pComma = strchrW(DriverVer, ',' as WCHAR) as LPWSTR;
        if !pComma.is_null() {
            *pComma = UNICODE_NULL as WCHAR;
            pVersion = pComma.offset(1 as isize);
        }
        /* Get driver date version. Invalid date = 00/00/00 */
        memset(
            DriverDate as *mut core::ffi::c_void,
            0 as core::ffi::c_int,
            core::mem::size_of::<FILETIME>() as size_t,
        );
        if strlenW(DriverVer) == 10
            && (*DriverVer.offset(2 as isize) == '-' as WCHAR
                || *DriverVer.offset(2 as isize) == '/' as WCHAR)
            && (*DriverVer.offset(5 as isize) == '-' as WCHAR
                || *DriverVer.offset(5 as isize) == '/' as WCHAR)
        {
            memset(
                core::ptr::addr_of_mut!(SystemTime) as *mut core::ffi::c_void,
                0 as core::ffi::c_int,
                core::mem::size_of::<SYSTEMTIME>() as size_t,
            );
            *DriverVer.offset(5 as isize) = 0;
            *DriverVer.offset(2 as isize) = 0;
            SystemTime.wMonth = (((*DriverVer.offset(0 as isize) as i32)
                - '0' as i32)
                * (10 as i32)
                + (*DriverVer.offset(1 as isize) as i32)
                - '0' as i32) as WORD;
            SystemTime.wDay = (((*DriverVer.offset(3 as isize) as i32)
                - '0' as i32)
                * (10 as i32)
                + (*DriverVer.offset(4 as isize) as i32)
                - '0' as i32) as WORD;
            SystemTime.wYear = ((((*DriverVer.offset(6 as isize) as i32)
                - '0' as i32)
                * (1000 as i32))
                + (((*DriverVer.offset(7 as isize) as i32) - '0' as i32)
                    * (100 as i32))
                + (((*DriverVer.offset(8 as isize) as i32) - '0' as i32)
                    * (10 as i32))
                + (*DriverVer.offset(9 as isize) as i32)
                - '0' as i32) as WORD;
            SystemTimeToFileTime(core::ptr::addr_of_mut!(SystemTime), DriverDate as LPFILETIME);
        }
        /* Get driver version. Invalid version = 0.0.0.0 */
        *DriverVersion = 0 as DWORDLONG;
        if !pVersion.is_null() {
            let mut Major: WORD = 0;
            let mut Minor: WORD = 0 as WORD;
            let mut Revision: WORD = 0 as WORD;
            let mut Build: WORD = 0 as WORD;
            let mut pMinor: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut pRevision: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut pBuild: LPWSTR = core::ptr::null_mut::<WCHAR>();
            let mut fullVersion: LARGE_INTEGER = core::mem::zeroed::<LARGE_INTEGER>();

            pMinor = strchrW(pVersion, '.' as WCHAR) as LPWSTR;
            if !pMinor.is_null() {
                *pMinor = 0 as WCHAR;
                pMinor = pMinor.offset(1);
                pRevision = strchrW(pMinor, '.' as WCHAR) as LPWSTR;
                Minor = atoiW(pMinor) as WORD;
            }
            if !pRevision.is_null() {
                *pRevision = 0 as WCHAR;
                pRevision = pRevision.offset(1);
                pBuild = strchrW(pRevision, '.' as WCHAR) as LPWSTR;
                Revision = atoiW(pRevision) as WORD;
            }
            if !pBuild.is_null() {
                *pBuild = 0 as WCHAR;
                pBuild = pBuild.offset(1);
                Build = atoiW(pBuild) as WORD;
            }
            Major = atoiW(pVersion) as WORD;
            fullVersion.u.HighPart = (((Major as core::ffi::c_int)
                << (16 as core::ffi::c_int))
                | (Minor as core::ffi::c_int)) as LONG;
            fullVersion.u.LowPart = (((Revision as core::ffi::c_int)
                << (16 as core::ffi::c_int))
                | (Build as core::ffi::c_int)) as DWORD;
            memcpy(
                DriverVersion as *mut core::ffi::c_void,
                core::ptr::addr_of_mut!(fullVersion) as *const core::ffi::c_void,
                core::mem::size_of::<LARGE_INTEGER>() as size_t,
            );
        }

        ret = (TRUE as i32);
    }

    /* cleanup: */
    if ret == 0 {
        HeapFree(
            GetProcessHeap(),
            0,
            ProviderName as PVOID,
        );
        *pProviderName = core::ptr::null_mut::<WCHAR>();
    }
    HeapFree(
        GetProcessHeap(),
        0,
        DriverVer as PVOID,
    );

    return ret;
}

unsafe extern "C" fn GetHardwareAndCompatibleIDsLists(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    pHardwareIDs: *mut LPWSTR,
    pHardwareIDsRequiredSize: LPDWORD,
    pCompatibleIDs: *mut LPWSTR,
    pCompatibleIDsRequiredSize: LPDWORD,
) -> BOOL {
    let mut HardwareIDs: LPWSTR = core::ptr::null_mut();
    let mut CompatibleIDs: LPWSTR = core::ptr::null_mut();
    let mut RequiredSize: DWORD = 0;
    let mut Result: BOOL = 0;

    'done: {
        /* Get hardware IDs list */
        Result = (FALSE as i32);
        RequiredSize = 512; /* Initial buffer size */
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        while Result == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            MyFree(HardwareIDs as PVOID);
            HardwareIDs = MyMalloc(RequiredSize) as LPWSTR;
            if HardwareIDs.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'done;
            }
            Result = SetupDiGetDeviceRegistryPropertyW(
                DeviceInfoSet,
                DeviceInfoData,
                SPDRP_HARDWAREID,
                core::ptr::null_mut(),
                HardwareIDs as PBYTE,
                RequiredSize,
                core::ptr::addr_of_mut!(RequiredSize),
            );
        }
        if Result == 0 {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                /* No hardware ID for this device */
                MyFree(HardwareIDs as PVOID);
                HardwareIDs = core::ptr::null_mut();
                RequiredSize = 0;
            } else {
                break 'done;
            }
        }
        if !pHardwareIDs.is_null() {
            *pHardwareIDs = HardwareIDs;
        }
        if !pHardwareIDsRequiredSize.is_null() {
            *pHardwareIDsRequiredSize = RequiredSize;
        }

        /* Get compatible IDs list */
        Result = (FALSE as i32);
        RequiredSize = 512; /* Initial buffer size */
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        while Result == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
            MyFree(CompatibleIDs as PVOID);
            CompatibleIDs = MyMalloc(RequiredSize) as LPWSTR;
            if CompatibleIDs.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'done;
            }
            Result = SetupDiGetDeviceRegistryPropertyW(
                DeviceInfoSet,
                DeviceInfoData,
                SPDRP_COMPATIBLEIDS,
                core::ptr::null_mut(),
                CompatibleIDs as PBYTE,
                RequiredSize,
                core::ptr::addr_of_mut!(RequiredSize),
            );
        }
        if Result == 0 {
            if GetLastError() == ERROR_FILE_NOT_FOUND {
                /* No compatible ID for this device */
                MyFree(CompatibleIDs as PVOID);
                CompatibleIDs = core::ptr::null_mut();
                RequiredSize = 0;
            } else {
                break 'done;
            }
        }
        if !pCompatibleIDs.is_null() {
            *pCompatibleIDs = CompatibleIDs;
        }
        if !pCompatibleIDsRequiredSize.is_null() {
            *pCompatibleIDsRequiredSize = RequiredSize;
        }

        Result = (TRUE as i32);
    }

    if Result == 0 {
        MyFree(HardwareIDs as PVOID);
        MyFree(CompatibleIDs as PVOID);
    }
    return Result;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiDestroyDriverInfoList(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    mut DriverType_local: DWORD,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);
    'done: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else {
            list = DeviceInfoSet as *mut DeviceInfoSet;
            if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
                SetLastError(ERROR_INVALID_HANDLE as DWORD);
            } else if DriverType_local != SPDIT_CLASSDRIVER as DWORD
                && DriverType_local != SPDIT_COMPATDRIVER as DWORD
            {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            } else if DriverType_local == SPDIT_COMPATDRIVER as DWORD && DeviceInfoData.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            } else if !DeviceInfoData.is_null()
                && (*DeviceInfoData).cbSize as usize
                    != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
            {
                SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
            } else {
                let mut ListEntry: PLIST_ENTRY = core::ptr::null_mut();
                let mut driverInfo: *mut DriverInfoElement = core::ptr::null_mut();
                let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();

                InstallParams.cbSize =
                    core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
                if SetupDiGetDeviceInstallParamsW(
                    DeviceInfoSet,
                    DeviceInfoData,
                    core::ptr::addr_of_mut!(InstallParams),
                ) == 0
                {
                    break 'done;
                }

                if DeviceInfoData.is_null() {
                    /* Fall back to destroying class driver list */
                    DriverType_local = SPDIT_CLASSDRIVER as DWORD;
                }

                if DriverType_local == SPDIT_CLASSDRIVER as DWORD {
                    while IsListEmpty(core::ptr::addr_of_mut!((*list).DriverListHead))
                        == false
                    {
                        ListEntry =
                            RemoveHeadList(core::ptr::addr_of_mut!((*list).DriverListHead));
                        driverInfo =
                            CONTAINING_RECORD!(ListEntry, DriverInfoElement, ListEntry);
                        DestroyDriverInfoElement(driverInfo);
                    }
                    InstallParams.ClassInstallReserved = 0 as ULONG_PTR;
                    InstallParams.Flags &=
                        !(DI_DIDCLASS | DI_MULTMFGS) as DWORD;
                    InstallParams.FlagsEx &= !DI_FLAGSEX_DIDINFOLIST as DWORD;
                    ret = SetupDiSetDeviceInstallParamsW(
                        DeviceInfoSet,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(InstallParams),
                    );
                } else {
                    let mut InstallParamsSet: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
                    let mut deviceInfo: *mut DeviceInfo = core::ptr::null_mut();

                    InstallParamsSet.cbSize =
                        core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
                    if SetupDiGetDeviceInstallParamsW(
                        DeviceInfoSet,
                        core::ptr::null_mut(),
                        core::ptr::addr_of_mut!(InstallParamsSet),
                    ) == 0
                    {
                        break 'done;
                    }
                    deviceInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
                    while IsListEmpty(core::ptr::addr_of_mut!(
                        (*deviceInfo).DriverListHead
                    )) == false
                    {
                        ListEntry = RemoveHeadList(core::ptr::addr_of_mut!(
                            (*deviceInfo).DriverListHead
                        ));
                        driverInfo =
                            CONTAINING_RECORD!(ListEntry, DriverInfoElement, ListEntry);
                        if InstallParamsSet.ClassInstallReserved as PVOID == driverInfo as PVOID
                        {
                            InstallParamsSet.ClassInstallReserved =
                                0 as ULONG_PTR;
                            SetupDiSetDeviceInstallParamsW(
                                DeviceInfoSet,
                                core::ptr::null_mut(),
                                core::ptr::addr_of_mut!(InstallParamsSet),
                            );
                        }
                        DestroyDriverInfoElement(driverInfo);
                    }
                    InstallParams.ClassInstallReserved = 0 as ULONG_PTR;
                    InstallParams.Flags &= !DI_DIDCOMPAT as DWORD;
                    InstallParams.FlagsEx &= !DI_FLAGSEX_DIDCOMPATINFO as DWORD;
                    ret = SetupDiSetDeviceInstallParamsW(
                        DeviceInfoSet,
                        DeviceInfoData,
                        core::ptr::addr_of_mut!(InstallParams),
                    );
                }
            }
        }
    }
    TRACE!("Returning %d\n", ret);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiEnumDriverInfoA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverType_local: DWORD,
    MemberIndex: DWORD,
    DriverInfoData: PSP_DRVINFO_DATA_A,
) -> BOOL {
    let mut driverInfoData2W: SP_DRVINFO_DATA_V2_W = core::mem::zeroed();
    let mut ret: BOOL = (FALSE as i32);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDriverInfoA\0" as *const u8 as *const ::core::ffi::c_char,
            1201 as ::core::ffi::c_int,
            b"%p %p 0x%lx %ld %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            DriverType_local,
            MemberIndex,
            DriverInfoData,
        );
    }

    if DriverInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*DriverInfoData).cbSize as usize
        != core::mem::size_of::<SP_DRVINFO_DATA_V1_A>() as usize
        && (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V2_A>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        driverInfoData2W.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_V2_W>() as DWORD;
        ret = SetupDiEnumDriverInfoW(
            DeviceInfoSet,
            DeviceInfoData,
            DriverType_local,
            MemberIndex,
            core::ptr::addr_of_mut!(driverInfoData2W),
        );

        if ret != 0 {
            (*DriverInfoData).DriverType = driverInfoData2W.DriverType;
            (*DriverInfoData).Reserved = driverInfoData2W.Reserved;
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.Description) as *mut WCHAR as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*DriverInfoData).Description) as LPSTR,
                (LINE_LEN as i32),
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                (*DriverInfoData).Description[0 as usize] = 0 as CHAR;
                ret = (FALSE as i32);
            }
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.MfgName) as *mut WCHAR as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*DriverInfoData).MfgName) as LPSTR,
                (LINE_LEN as i32),
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                (*DriverInfoData).MfgName[0 as usize] = 0 as CHAR;
                ret = (FALSE as i32);
            }
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.ProviderName) as *mut WCHAR as LPCWSTR,
                -1,
                core::ptr::addr_of_mut!((*DriverInfoData).ProviderName) as LPSTR,
                (LINE_LEN as i32),
                core::ptr::null::<CHAR>(),
                core::ptr::null_mut::<BOOL>(),
            ) == 0
            {
                (*DriverInfoData).ProviderName[0 as usize] = 0 as CHAR;
                ret = (FALSE as i32);
            }
            if (*DriverInfoData).cbSize as usize == core::mem::size_of::<SP_DRVINFO_DATA_V2_A>() as usize {
                (*DriverInfoData).DriverDate = driverInfoData2W.DriverDate;
                (*DriverInfoData).DriverVersion = driverInfoData2W.DriverVersion;
            }
        }
    }

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiEnumDriverInfoA\0" as *const u8 as *const ::core::ffi::c_char,
            1245 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiEnumDriverInfoW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverType_local: DWORD,
    MemberIndex: DWORD,
    DriverInfoData: PSP_DRVINFO_DATA_W,
) -> BOOL {
    let mut ListHead: PLIST_ENTRY = core::ptr::null_mut::<_LIST_ENTRY>();
    let mut ret: BOOL = (FALSE as i32);
    let mut MemberIndex = MemberIndex;

    if DeviceInfoSet.is_null() || DriverInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if DriverType_local != SPDIT_CLASSDRIVER as DWORD && DriverType_local != SPDIT_COMPATDRIVER as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DriverType_local == SPDIT_COMPATDRIVER as DWORD && DeviceInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V1_W>()
        && (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V2_W>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        let mut devInfo: *mut DeviceInfo = core::ptr::null_mut::<DeviceInfo>();
        let mut ItemList: PLIST_ENTRY = core::ptr::null_mut::<_LIST_ENTRY>();
        if !DeviceInfoData.is_null() {
            devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
        }
        if devInfo.is_null() || (*devInfo).CreationFlags & DICD_INHERIT_CLASSDRVS as DWORD != 0 {
            ListHead = core::ptr::addr_of_mut!((*(DeviceInfoSet as *mut DeviceInfoSet)).DriverListHead) as PLIST_ENTRY;
        } else {
            ListHead = core::ptr::addr_of_mut!((*devInfo).DriverListHead) as PLIST_ENTRY;
        }

        ItemList = (*ListHead).Flink as PLIST_ENTRY;
        while ItemList != ListHead && {
            let fresh = MemberIndex;
            MemberIndex = MemberIndex.wrapping_sub(1);
            fresh > 0 as DWORD
        } {
            ItemList = (*ItemList).Flink as PLIST_ENTRY;
        }
        if ItemList == ListHead {
            SetLastError(ERROR_NO_MORE_ITEMS as DWORD);
        } else {
            let DrvInfo: *mut DriverInfoElement = (ItemList as usize).wrapping_sub(
                core::mem::offset_of!(DriverInfoElement, ListEntry) as usize,
            ) as *mut DriverInfoElement;

            memcpy(
                core::ptr::addr_of_mut!((*DriverInfoData).DriverType) as *mut core::ffi::c_void,
                core::ptr::addr_of!((*DrvInfo).Info.DriverType) as *const core::ffi::c_void,
                (*DriverInfoData)
                    .cbSize
                    .wrapping_sub(core::mem::offset_of!(SP_DRVINFO_DATA_W, DriverType) as DWORD)
                    as usize,
            );
            ret = (TRUE as i32);
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetSelectedDriverA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_A,
) -> BOOL {
    let mut driverInfoData2W: SP_DRVINFO_DATA_V2_W = core::mem::zeroed();
    let mut ret: BOOL = FALSE as BOOL;

    if DriverInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V1_A>() as usize
        && (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V2_A>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        driverInfoData2W.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_V2_W>() as DWORD;

        ret = SetupDiGetSelectedDriverW(
            DeviceInfoSet,
            DeviceInfoData,
            &raw mut driverInfoData2W,
        );

        if ret != 0 {
            /* Do W->A conversion */
            (*DriverInfoData).DriverType = driverInfoData2W.DriverType;
            (*DriverInfoData).Reserved = driverInfoData2W.Reserved;
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.Description) as *mut WCHAR as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                core::ptr::addr_of_mut!((*DriverInfoData).Description) as LPSTR,
                LINE_LEN as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) == 0 as ::core::ffi::c_int
            {
                (*DriverInfoData).Description[0 as usize] = '\0' as i32 as CHAR;
                ret = FALSE as BOOL;
            }
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.MfgName) as *mut WCHAR as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                core::ptr::addr_of_mut!((*DriverInfoData).MfgName) as LPSTR,
                LINE_LEN as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) == 0 as ::core::ffi::c_int
            {
                (*DriverInfoData).MfgName[0 as usize] = '\0' as i32 as CHAR;
                ret = FALSE as BOOL;
            }
            if WideCharToMultiByte(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!(driverInfoData2W.ProviderName) as *mut WCHAR as LPCWSTR,
                -(1 as ::core::ffi::c_int),
                core::ptr::addr_of_mut!((*DriverInfoData).ProviderName) as LPSTR,
                LINE_LEN as ::core::ffi::c_int,
                ::core::ptr::null::<CHAR>(),
                ::core::ptr::null_mut::<BOOL>(),
            ) == 0 as ::core::ffi::c_int
            {
                (*DriverInfoData).ProviderName[0 as usize] = '\0' as i32 as CHAR;
                ret = FALSE as BOOL;
            }
            if (*DriverInfoData).cbSize as usize == core::mem::size_of::<SP_DRVINFO_DATA_V2_A>() as usize {
                /* Copy more fields */
                (*DriverInfoData).DriverDate = driverInfoData2W.DriverDate;
                (*DriverInfoData).DriverVersion = driverInfoData2W.DriverVersion;
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetSelectedDriverW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_W,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() || DriverInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if (*DriverInfoData).cbSize as usize
        != core::mem::size_of::<SP_DRVINFO_DATA_V1_W>()
        && (*DriverInfoData).cbSize as usize
            != core::mem::size_of::<SP_DRVINFO_DATA_V2_W>()
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        let mut InstallParams: SP_DEVINSTALL_PARAMS = core::mem::zeroed();

        InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        if SetupDiGetDeviceInstallParamsW(
            DeviceInfoSet,
            DeviceInfoData,
            core::ptr::addr_of_mut!(InstallParams),
        ) != 0
        {
            let mut driverInfo: *mut DriverInfoElement = core::ptr::null_mut();
            driverInfo = InstallParams.ClassInstallReserved as *mut DriverInfoElement;
            if driverInfo.is_null() {
                SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
            } else {
                memcpy(
                    core::ptr::addr_of_mut!((*DriverInfoData).DriverType) as *mut core::ffi::c_void,
                    core::ptr::addr_of_mut!((*driverInfo).Info.DriverType) as *const core::ffi::c_void,
                    (*DriverInfoData)
                        .cbSize
                        .wrapping_sub(FIELD_OFFSET!(SP_DRVINFO_DATA_W, DriverType) as DWORD)
                        as usize,
                );
                ret = (TRUE as i32);
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetSelectedDriverA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_A,
) -> BOOL {
    let mut DriverInfoDataW: SP_DRVINFO_DATA_V1_W = core::mem::zeroed::<SP_DRVINFO_DATA_V1_W>();
    let mut pDriverInfoDataW: PSP_DRVINFO_DATA_W = core::ptr::null_mut();
    let mut ret: BOOL = (FALSE as i32);

    if !DriverInfoData.is_null() {
        if (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V2_A>() as usize
            && (*DriverInfoData).cbSize as usize
                != core::mem::size_of::<SP_DRVINFO_DATA_V1_A>() as usize
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            return (FALSE) as BOOL;
        }

        DriverInfoDataW.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_V1_W>() as DWORD;
        DriverInfoDataW.Reserved = (*DriverInfoData).Reserved;

        if DriverInfoDataW.Reserved == 0 as ULONG_PTR {
            DriverInfoDataW.DriverType = (*DriverInfoData).DriverType;

            /* convert the strings to unicode */
            if MultiByteToWideChar(
                CP_ACP as UINT,
                0 as DWORD,
                core::ptr::addr_of_mut!((*DriverInfoData).Description) as *mut CHAR as LPCSTR,
                (LINE_LEN as i32),
                core::ptr::addr_of_mut!(DriverInfoDataW.Description) as LPWSTR,
                (LINE_LEN as i32),
            ) == 0
                || MultiByteToWideChar(
                    CP_ACP as UINT,
                    0 as DWORD,
                    core::ptr::addr_of_mut!((*DriverInfoData).ProviderName) as *mut CHAR as LPCSTR,
                    (LINE_LEN as i32),
                    core::ptr::addr_of_mut!(DriverInfoDataW.ProviderName) as LPWSTR,
                    (LINE_LEN as i32),
                ) == 0
            {
                return (FALSE) as BOOL;
            }
        }

        pDriverInfoDataW = core::ptr::addr_of_mut!(DriverInfoDataW) as PSP_DRVINFO_DATA_W;
    }

    ret = SetupDiSetSelectedDriverW(DeviceInfoSet, DeviceInfoData, pDriverInfoDataW);

    if ret != 0 && !pDriverInfoDataW.is_null() {
        (*DriverInfoData).Reserved = DriverInfoDataW.Reserved;
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSetSelectedDriverW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_W,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if !DriverInfoData.is_null()
        && (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V1_W>() as usize
        && (*DriverInfoData).cbSize as usize != core::mem::size_of::<SP_DRVINFO_DATA_V2_W>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        let mut pDriverInfo: *mut *mut DriverInfoElement = core::ptr::null_mut();
        let mut ListHead: PLIST_ENTRY = core::ptr::null_mut();
        let mut ItemList: PLIST_ENTRY = core::ptr::null_mut();

        if !DeviceInfoData.is_null() {
            pDriverInfo = core::ptr::addr_of_mut!(
                (*((*DeviceInfoData).Reserved as *mut DeviceInfo)).InstallParams.ClassInstallReserved
            ) as *mut *mut DriverInfoElement;
            ListHead = core::ptr::addr_of_mut!(
                (*((*DeviceInfoData).Reserved as *mut DeviceInfo)).DriverListHead
            ) as PLIST_ENTRY;
        } else {
            pDriverInfo = core::ptr::addr_of_mut!(
                (*(DeviceInfoSet as *mut DeviceInfoSet)).InstallParams.ClassInstallReserved
            ) as *mut *mut DriverInfoElement;
            ListHead = core::ptr::addr_of_mut!(
                (*(DeviceInfoSet as *mut DeviceInfoSet)).DriverListHead
            ) as PLIST_ENTRY;
        }

        if DriverInfoData.is_null() {
            *pDriverInfo = core::ptr::null_mut();
            ret = (TRUE as i32);
        } else {
            /* Search selected driver in list */
            ItemList = (*ListHead).Flink as PLIST_ENTRY;
            while ItemList != ListHead {
                if (*DriverInfoData).Reserved != 0 as ULONG_PTR {
                    if (*DriverInfoData).Reserved == ItemList as ULONG_PTR {
                        break;
                    }
                } else {
                    /* The caller wants to compare only DriverType, Description and ProviderName fields */
                    let mut driverInfo: *mut DriverInfoElement =
                        (ItemList as ULONG_PTR).wrapping_sub(
                            core::mem::offset_of!(DriverInfoElement, ListEntry) as ULONG_PTR,
                        ) as *mut DriverInfoElement;
                    if (*driverInfo).Info.DriverType == (*DriverInfoData).DriverType
                        && strcmpW(
                            core::ptr::addr_of_mut!((*driverInfo).Info.Description) as *mut WCHAR,
                            core::ptr::addr_of_mut!((*DriverInfoData).Description) as *mut WCHAR,
                        ) == 0
                        && strcmpW(
                            core::ptr::addr_of_mut!((*driverInfo).Info.ProviderName) as *mut WCHAR,
                            core::ptr::addr_of_mut!((*DriverInfoData).ProviderName) as *mut WCHAR,
                        ) == 0
                    {
                        break;
                    }
                }
                ItemList = (*ItemList).Flink as PLIST_ENTRY;
            }

            if ItemList == ListHead {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
            } else {
                *pDriverInfo = (ItemList as ULONG_PTR).wrapping_sub(
                    core::mem::offset_of!(DriverInfoElement, ListEntry) as ULONG_PTR,
                ) as *mut DriverInfoElement;
                (*DriverInfoData).Reserved = ItemList as ULONG_PTR;
                ret = (TRUE as i32);

                if !DeviceInfoData.is_null() {
                    memcpy(
                        core::ptr::addr_of_mut!((*DeviceInfoData).ClassGuid) as *mut core::ffi::c_void,
                        core::ptr::addr_of_mut!((**pDriverInfo).ClassGuid) as *const core::ffi::c_void,
                        core::mem::size_of::<GUID>() as size_t,
                    );
                }
            }
        }
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDriverInfoDetailA(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_A,
    DriverInfoDetailData: PSP_DRVINFO_DETAIL_DATA_A,
    DriverInfoDetailDataSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut DriverInfoDataW: SP_DRVINFO_DATA_V2_W = core::mem::zeroed();
    let mut DriverInfoDetailDataW: PSP_DRVINFO_DETAIL_DATA_W = core::ptr::null_mut();
    let mut BufSize: DWORD = 0;
    let mut HardwareIDLen: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    'Cleanup: {
        /* do some sanity checks, the unicode version might do more thorough checks */
        if DriverInfoData.is_null()
            || (DriverInfoDetailData.is_null() && DriverInfoDetailDataSize != 0)
            || (!DriverInfoDetailData.is_null()
                && ((DriverInfoDetailDataSize as usize)
                    < core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_A, HardwareID)
                        + core::mem::size_of::<CHAR>()
                    || (*DriverInfoDetailData).cbSize as usize
                        != core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_A>()))
        {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'Cleanup;
        }

        /* make sure we support both versions of the SP_DRVINFO_DATA structure */
        if (*DriverInfoData).cbSize as usize == core::mem::size_of::<SP_DRVINFO_DATA_V1_A>() {
            DriverInfoDataW.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_V1_W>() as DWORD;
        } else if (*DriverInfoData).cbSize as usize
            == core::mem::size_of::<SP_DRVINFO_DATA_V2_A>()
        {
            DriverInfoDataW.cbSize = core::mem::size_of::<SP_DRVINFO_DATA_V2_W>() as DWORD;
        } else {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'Cleanup;
        }
        DriverInfoDataW.DriverType = (*DriverInfoData).DriverType;
        DriverInfoDataW.Reserved = (*DriverInfoData).Reserved;

        /* convert the strings to unicode */
        if MultiByteToWideChar(
            CP_ACP,
            0,
            core::ptr::addr_of_mut!((*DriverInfoData).Description) as LPCSTR,
            (LINE_LEN as i32),
            core::ptr::addr_of_mut!(DriverInfoDataW.Description) as LPWSTR,
            (LINE_LEN as i32),
        ) != 0
            && MultiByteToWideChar(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!((*DriverInfoData).MfgName) as LPCSTR,
                (LINE_LEN as i32),
                core::ptr::addr_of_mut!(DriverInfoDataW.MfgName) as LPWSTR,
                (LINE_LEN as i32),
            ) != 0
            && MultiByteToWideChar(
                CP_ACP,
                0,
                core::ptr::addr_of_mut!((*DriverInfoData).ProviderName) as LPCSTR,
                (LINE_LEN as i32),
                core::ptr::addr_of_mut!(DriverInfoDataW.ProviderName) as LPWSTR,
                (LINE_LEN as i32),
            ) != 0
        {
            if DriverInfoDataW.cbSize as usize
                == core::mem::size_of::<SP_DRVINFO_DATA_V2_W>()
            {
                DriverInfoDataW.DriverDate =
                    (*(DriverInfoData as PSP_DRVINFO_DATA_V2_A)).DriverDate;
                DriverInfoDataW.DriverVersion =
                    (*(DriverInfoData as PSP_DRVINFO_DATA_V2_A)).DriverVersion;
            }

            if !DriverInfoDetailData.is_null() {
                /* calculate the unicode buffer size from the ansi buffer size */
                HardwareIDLen = DriverInfoDetailDataSize.wrapping_sub(
                    core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_A, HardwareID) as DWORD,
                );
                BufSize = (core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_W, HardwareID)
                    as DWORD)
                    .wrapping_add(HardwareIDLen.wrapping_mul(core::mem::size_of::<WCHAR>() as DWORD));

                DriverInfoDetailDataW =
                    MyMalloc(BufSize) as PSP_DRVINFO_DETAIL_DATA_W;
                if DriverInfoDetailDataW.is_null() {
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break 'Cleanup;
                }

                /* initialize the buffer */
                core::ptr::write_bytes(DriverInfoDetailDataW as *mut u8, 0, BufSize as usize);
                (*DriverInfoDetailDataW).cbSize =
                    core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as DWORD;
            }

            /* call the unicode version */
            ret = SetupDiGetDriverInfoDetailW(
                DeviceInfoSet,
                DeviceInfoData,
                core::ptr::addr_of_mut!(DriverInfoDataW),
                DriverInfoDetailDataW,
                BufSize,
                RequiredSize,
            );

            if ret != 0 {
                if !DriverInfoDetailDataW.is_null() {
                    /* convert the SP_DRVINFO_DETAIL_DATA_W structure to ansi */
                    (*DriverInfoDetailData).cbSize =
                        core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_A>() as DWORD;
                    (*DriverInfoDetailData).InfDate = (*DriverInfoDetailDataW).InfDate;
                    (*DriverInfoDetailData).Reserved = (*DriverInfoDetailDataW).Reserved;
                    if WideCharToMultiByte(
                        CP_ACP,
                        0,
                        core::ptr::addr_of_mut!((*DriverInfoDetailDataW).SectionName) as LPCWSTR,
                        (LINE_LEN as i32),
                        core::ptr::addr_of_mut!((*DriverInfoDetailData).SectionName) as LPSTR,
                        (LINE_LEN as i32),
                        core::ptr::null::<CHAR>(),
                        core::ptr::null_mut::<BOOL>(),
                    ) != 0
                        && WideCharToMultiByte(
                            CP_ACP,
                            0,
                            core::ptr::addr_of_mut!((*DriverInfoDetailDataW).InfFileName)
                                as LPCWSTR,
                            (MAX_PATH as i32),
                            core::ptr::addr_of_mut!((*DriverInfoDetailData).InfFileName) as LPSTR,
                            (MAX_PATH as i32),
                            core::ptr::null::<CHAR>(),
                            core::ptr::null_mut::<BOOL>(),
                        ) != 0
                        && WideCharToMultiByte(
                            CP_ACP,
                            0,
                            core::ptr::addr_of_mut!((*DriverInfoDetailDataW).DrvDescription)
                                as LPCWSTR,
                            (LINE_LEN as i32),
                            core::ptr::addr_of_mut!((*DriverInfoDetailData).DrvDescription)
                                as LPSTR,
                            (LINE_LEN as i32),
                            core::ptr::null::<CHAR>(),
                            core::ptr::null_mut::<BOOL>(),
                        ) != 0
                        && WideCharToMultiByte(
                            CP_ACP,
                            0,
                            core::ptr::addr_of_mut!((*DriverInfoDetailDataW).HardwareID)
                                as LPCWSTR,
                            HardwareIDLen as i32,
                            core::ptr::addr_of_mut!((*DriverInfoDetailData).HardwareID) as LPSTR,
                            HardwareIDLen as i32,
                            core::ptr::null::<CHAR>(),
                            core::ptr::null_mut::<BOOL>(),
                        ) != 0
                    {
                        let mut len: DWORD = 0;
                        let mut cnt: DWORD = 0;
                        let mut hwidlen: DWORD = HardwareIDLen;
                        let mut s: *mut CHAR =
                            core::ptr::addr_of_mut!((*DriverInfoDetailData).HardwareID) as *mut CHAR;

                        /* count the strings in the list */
                        while *s != 0 {
                            len = (lstrlenA(s as LPCSTR) + 1)
                                as DWORD;
                            if hwidlen > len {
                                cnt = cnt.wrapping_add(1);
                                s = s.offset(len as isize);
                                hwidlen = hwidlen.wrapping_sub(len);
                            } else {
                                /* looks like the string list wasn't terminated... */
                                SetLastError(ERROR_INVALID_USER_BUFFER);
                                ret = (FALSE as i32);
                                break;
                            }
                        }

                        /* make sure CompatIDsOffset points to the second string in the
                           list, if present */
                        if cnt > 1 {
                            (*DriverInfoDetailData).CompatIDsOffset = (lstrlenA(
                                core::ptr::addr_of_mut!((*DriverInfoDetailData).HardwareID)
                                    as LPCSTR,
                            ) + 1)
                                as DWORD;
                            (*DriverInfoDetailData).CompatIDsLength = (s.offset_from(
                                core::ptr::addr_of_mut!((*DriverInfoDetailData).HardwareID)
                                    as *mut CHAR,
                            ) as DWORD)
                                .wrapping_sub((*DriverInfoDetailData).CompatIDsOffset)
                                .wrapping_add(1);
                        } else {
                            (*DriverInfoDetailData).CompatIDsOffset = 0;
                            (*DriverInfoDetailData).CompatIDsLength = 0;
                        }
                    } else {
                        ret = (FALSE as i32);
                    }
                }

                if !RequiredSize.is_null() {
                    *RequiredSize = (core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_A, HardwareID)
                        as DWORD)
                        .wrapping_add(
                            (*RequiredSize).wrapping_sub(
                                core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_W, HardwareID)
                                    as DWORD,
                            ) / (core::mem::size_of::<WCHAR>() as DWORD),
                        );
                }
            }
        }
    }

    if !DriverInfoDetailDataW.is_null() {
        MyFree((DriverInfoDetailDataW).cast());
    }

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDriverInfoDetailW(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverInfoData: PSP_DRVINFO_DATA_W,
    DriverInfoDetailData: PSP_DRVINFO_DETAIL_DATA_W,
    DriverInfoDetailDataSize: DWORD,
    RequiredSize: PDWORD,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/setupapi/driver.c\0".as_ptr() as *const core::ffi::c_char,
            b"SetupDiGetDriverInfoDetailW\0".as_ptr() as *const core::ffi::c_char,
            1789 as core::ffi::c_int,
            b"%p %p %p %p %lu %p\n\0".as_ptr() as *const core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            DriverInfoData,
            DriverInfoDetailData,
            DriverInfoDetailDataSize,
            RequiredSize,
        );
    }

    if DeviceInfoSet.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize != core::mem::size_of::<SP_DEVINFO_DATA>() as DWORD
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if DriverInfoData.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DriverInfoDetailData.is_null() && DriverInfoDetailDataSize != 0 as DWORD {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if !DriverInfoDetailData.is_null()
        && DriverInfoDetailDataSize < core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as DWORD
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if !DriverInfoDetailData.is_null()
        && (*DriverInfoDetailData).cbSize != core::mem::size_of::<SP_DRVINFO_DETAIL_DATA_W>() as DWORD
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if (*DriverInfoData).Reserved == 0 as ULONG_PTR {
        SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
    } else {
        let mut driverInfoElement: *mut DriverInfoElement = core::ptr::null_mut();
        let mut HardwareIDs: LPWSTR = core::ptr::null_mut();
        let mut CompatibleIDs: LPWSTR = core::ptr::null_mut();
        let mut pBuffer: LPWSTR = core::ptr::null_mut();
        let mut DeviceID: LPCWSTR = core::ptr::null();
        let mut HardwareIDsSize: ULONG = 0;
        let mut CompatibleIDsSize: ULONG = 0;
        let mut sizeNeeded: ULONG = 0;
        let mut sizeLeft: ULONG = 0;
        let mut size: ULONG = 0;
        let mut Result: BOOL = 0;

        'done: {
            driverInfoElement = (*DriverInfoData).Reserved as *mut DriverInfoElement;

            /* Get hardware and compatible IDs lists */
            Result = GetHardwareAndCompatibleIDsLists(
                DeviceInfoSet,
                DeviceInfoData,
                core::ptr::addr_of_mut!(HardwareIDs),
                core::ptr::addr_of_mut!(HardwareIDsSize),
                core::ptr::addr_of_mut!(CompatibleIDs),
                core::ptr::addr_of_mut!(CompatibleIDsSize),
            );
            if Result == 0 {
                break 'done;
            }

            sizeNeeded = (core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_W, HardwareID) as ULONG)
                .wrapping_add(HardwareIDsSize)
                .wrapping_add(CompatibleIDsSize);
            if !RequiredSize.is_null() {
                *RequiredSize = sizeNeeded as DWORD;
            }

            if DriverInfoDetailData.is_null() {
                ret = TRUE as BOOL;
                break 'done;
            }

            memcpy(
                DriverInfoDetailData as *mut core::ffi::c_void,
                core::ptr::addr_of!((*driverInfoElement).Details) as *const core::ffi::c_void,
                (*driverInfoElement).Details.cbSize as usize,
            );
            (*DriverInfoDetailData).CompatIDsOffset = 0 as DWORD;
            (*DriverInfoDetailData).CompatIDsLength = 0 as DWORD;

            sizeLeft = (((DriverInfoDetailDataSize
                .wrapping_sub(core::mem::offset_of!(SP_DRVINFO_DETAIL_DATA_W, HardwareID) as DWORD))
                as usize)
                / core::mem::size_of::<WCHAR>()) as ULONG;
            pBuffer = core::ptr::addr_of_mut!((*DriverInfoDetailData).HardwareID) as *mut WCHAR as LPWSTR;
            /* Add as many as possible HardwareIDs in the list */
            DeviceID = HardwareIDs as LPCWSTR;
            loop {
                if DeviceID.is_null() || *DeviceID == 0 {
                    break;
                }
                size = wcslen(DeviceID) as ULONG;
                if !(size.wrapping_add(1 as ULONG) < sizeLeft) {
                    break;
                }

                if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/setupapi/driver.c\0".as_ptr() as *const core::ffi::c_char,
                        b"SetupDiGetDriverInfoDetailW\0".as_ptr() as *const core::ffi::c_char,
                        1855 as core::ffi::c_int,
                        b"Adding %s to list\n\0".as_ptr() as *const core::ffi::c_char,
                        debugstr_w(DeviceID),
                    );
                }
                wcscpy(pBuffer, DeviceID);
                DeviceID = DeviceID.offset(size.wrapping_add(1 as ULONG) as isize);
                pBuffer = pBuffer.offset(size.wrapping_add(1 as ULONG) as isize);
                sizeLeft = sizeLeft.wrapping_sub(size.wrapping_add(1 as ULONG));
                (*DriverInfoDetailData).CompatIDsOffset = (*DriverInfoDetailData)
                    .CompatIDsOffset
                    .wrapping_add(size.wrapping_add(1 as ULONG) as DWORD);
            }
            if sizeLeft > 0 as ULONG {
                *pBuffer = UNICODE_NULL as WCHAR;
                sizeLeft = sizeLeft.wrapping_sub(1);
                (*DriverInfoDetailData).CompatIDsOffset =
                    (*DriverInfoDetailData).CompatIDsOffset.wrapping_add(1);
            }

            /* Add as many as possible CompatibleIDs in the list */
            DeviceID = CompatibleIDs as LPCWSTR;
            loop {
                if DeviceID.is_null() || *DeviceID == 0 {
                    break;
                }
                size = wcslen(DeviceID) as ULONG;
                if !(size.wrapping_add(1 as ULONG) < sizeLeft) {
                    break;
                }

                if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
                    ros_dbg_log(
                        __WINE_DBCL_TRACE,
                        __wine_dbch___default,
                        b"dll/win32/setupapi/driver.c\0".as_ptr() as *const core::ffi::c_char,
                        b"SetupDiGetDriverInfoDetailW\0".as_ptr() as *const core::ffi::c_char,
                        1872 as core::ffi::c_int,
                        b"Adding %s to list\n\0".as_ptr() as *const core::ffi::c_char,
                        debugstr_w(DeviceID),
                    );
                }
                wcscpy(pBuffer, DeviceID);
                DeviceID = DeviceID.offset(size.wrapping_add(1 as ULONG) as isize);
                pBuffer = pBuffer.offset(size.wrapping_add(1 as ULONG) as isize);
                sizeLeft = sizeLeft.wrapping_sub(size.wrapping_add(1 as ULONG));
                (*DriverInfoDetailData).CompatIDsLength = (*DriverInfoDetailData)
                    .CompatIDsLength
                    .wrapping_add(size.wrapping_add(1 as ULONG) as DWORD);
            }
            if sizeLeft > 0 as ULONG {
                *pBuffer = UNICODE_NULL as WCHAR;
                sizeLeft = sizeLeft.wrapping_sub(1);
                (*DriverInfoDetailData).CompatIDsLength =
                    (*DriverInfoDetailData).CompatIDsLength.wrapping_add(1);
            }

            if sizeNeeded > DriverInfoDetailDataSize {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            } else {
                ret = TRUE as BOOL;
            }
        }

        MyFree(HardwareIDs as PVOID);
        MyFree(CompatibleIDs as PVOID);
    }

    if ((*__wine_dbch___default).flags as i32 & ((1i32) << (__WINE_DBCL_TRACE as i32))) != 0 {
        ros_dbg_log(
            __WINE_DBCL_TRACE,
            __wine_dbch___default,
            b"dll/win32/setupapi/driver.c\0".as_ptr() as *const core::ffi::c_char,
            b"SetupDiGetDriverInfoDetailW\0".as_ptr() as *const core::ffi::c_char,
            1896 as core::ffi::c_int,
            b"Returning %d\n\0".as_ptr() as *const core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiGetDriverInstallParamsW(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
    mut DriverInfoData: PSP_DRVINFO_DATA_W,
    mut DriverInstallParams: PSP_DRVINSTALL_PARAMS,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDriverInstallParamsW\0" as *const u8 as *const ::core::ffi::c_char,
            1912 as ::core::ffi::c_int,
            b"%p %p %p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
            DriverInfoData,
            DriverInstallParams,
        );
    }
    if DeviceInfoSet.is_null() || DriverInfoData.is_null() || DriverInstallParams.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    } else if DeviceInfoSet == INVALID_HANDLE_VALUE as HDEVINFO {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic
        != SETUP_DEVICE_INFO_SET_MAGIC as DWORD
    {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
    } else if !DeviceInfoData.is_null()
        && (*DeviceInfoData).cbSize as usize
            != ::core::mem::size_of::<SP_DEVINFO_DATA>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if (*DriverInfoData).cbSize as usize
        != ::core::mem::size_of::<SP_DRVINFO_DATA_V1_W>() as usize
        && (*DriverInfoData).cbSize as usize
            != ::core::mem::size_of::<SP_DRVINFO_DATA_V2_W>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else if (*DriverInstallParams).cbSize as usize
        != ::core::mem::size_of::<SP_DRVINSTALL_PARAMS>() as usize
    {
        SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
    } else {
        let mut InstallParams: SP_DEVINSTALL_PARAMS = _SP_DEVINSTALL_PARAMS_W {
            cbSize: 0,
            Flags: 0,
            FlagsEx: 0,
            hwndParent: ::core::ptr::null_mut::<HWND__>(),
            InstallMsgHandler: None,
            InstallMsgHandlerContext: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            FileQueue: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            ClassInstallReserved: 0,
            Reserved: 0,
            DriverPath: [0; 260],
        };
        InstallParams.cbSize =
            ::core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        if SetupDiGetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &raw mut InstallParams) != 0 {
            let mut driverInfo: *mut DriverInfoElement =
                ::core::ptr::null_mut::<DriverInfoElement>();
            driverInfo = InstallParams.ClassInstallReserved as *mut DriverInfoElement;
            if driverInfo.is_null() {
                SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
            } else {
                memcpy(
                    DriverInstallParams as *mut ::core::ffi::c_void,
                    &raw mut (*driverInfo).Params as *const ::core::ffi::c_void,
                    (*DriverInstallParams).cbSize as size_t,
                );
                ret = TRUE as BOOL;
            }
        }
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiGetDriverInstallParamsW\0" as *const u8 as *const ::core::ffi::c_char,
            1948 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiSelectBestCompatDrv(
    mut DeviceInfoSet: HDEVINFO,
    mut DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut drvInfoData: SP_DRVINFO_DATA_W = core::mem::zeroed();
    let mut ret: BOOL = 0;
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiSelectBestCompatDrv\0" as *const u8 as *const ::core::ffi::c_char,
            1963 as ::core::ffi::c_int,
            b"%p %p\n\0" as *const u8 as *const ::core::ffi::c_char,
            DeviceInfoSet,
            DeviceInfoData,
        );
    }
    /* Drivers are sorted by rank in the driver list, so
     * the first driver in the list is the best one.
     */
    drvInfoData.cbSize = ::core::mem::size_of::<SP_DRVINFO_DATA_W>() as DWORD;
    ret = SetupDiEnumDriverInfoW(
        DeviceInfoSet,
        DeviceInfoData,
        SPDIT_COMPATDRIVER as DWORD,
        0 as DWORD, /* Member index */
        &raw mut drvInfoData,
    );
    if ret != 0 {
        ret = SetupDiSetSelectedDriverW(DeviceInfoSet, DeviceInfoData, &raw mut drvInfoData);
    }
    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl_0: __wine_debug_class = __WINE_DBCL_TRACE;
        ros_dbg_log(
            __dbcl_0,
            __dbch_0,
            b"dll/win32/setupapi/driver.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupDiSelectBestCompatDrv\0" as *const u8 as *const ::core::ffi::c_char,
            1984 as ::core::ffi::c_int,
            b"Returning %d\n\0" as *const u8 as *const ::core::ffi::c_char,
            ret,
        );
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDiInstallDriverFiles(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
) -> BOOL {
    let mut ret: BOOL = (FALSE as i32);

    'done: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        } else if DeviceInfoSet == (INVALID_HANDLE_VALUE as HDEVINFO) {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if (*(DeviceInfoSet as *mut DeviceInfoSet)).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD {
            SetLastError(ERROR_INVALID_HANDLE as DWORD);
        } else if !DeviceInfoData.is_null()
            && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>()
        {
            SetLastError(ERROR_INVALID_USER_BUFFER as DWORD);
        } else if !DeviceInfoData.is_null()
            && (*((*DeviceInfoData).Reserved as *mut DeviceInfo))
                .InstallParams
                .ClassInstallReserved
                == 0 as ULONG_PTR
        {
            SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
        } else if DeviceInfoData.is_null()
            && (*(DeviceInfoSet as *mut DeviceInfoSet))
                .InstallParams
                .ClassInstallReserved
                == 0 as ULONG_PTR
        {
            SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
        } else {
            let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
            let mut SelectedDriver: *mut DriverInfoElement =
                core::ptr::null_mut::<DriverInfoElement>();
            let mut SectionName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
            let mut SectionNameLength: DWORD = 0 as DWORD;
            let mut InstallMsgHandler: PVOID = core::ptr::null_mut();
            let mut InstallMsgHandlerContext: PVOID = core::ptr::null_mut();
            let mut Context: PVOID = core::ptr::null_mut();

            InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
            ret = SetupDiGetDeviceInstallParamsW(
                DeviceInfoSet,
                DeviceInfoData,
                core::ptr::addr_of_mut!(InstallParams),
            );
            if ret == 0 {
                break 'done;
            }

            SelectedDriver = InstallParams.ClassInstallReserved as *mut DriverInfoElement;
            if SelectedDriver.is_null() {
                SetLastError(ERROR_NO_DRIVER_SELECTED as DWORD);
                break 'done;
            }

            ret = SetupDiGetActualSectionToInstallW(
                (*(*SelectedDriver).InfFileDetails).hInf,
                core::ptr::addr_of_mut!((*SelectedDriver).Details.SectionName) as *mut WCHAR
                    as PCWSTR,
                core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PWSTR,
                ((MAX_PATH as usize)
                    .wrapping_sub(strlenW(core::ptr::addr_of!(DotCoInstallers) as *const WCHAR)
                        as usize)) as DWORD,
                core::ptr::addr_of_mut!(SectionNameLength),
                core::ptr::null_mut(),
            );
            if ret == 0 {
                break 'done;
            }

            'cleanup: {
                if { InstallParams.InstallMsgHandler }.is_some() {
                    InstallMsgHandler = core::mem::transmute::<PSP_FILE_CALLBACK_W, PVOID>(
                        InstallParams.InstallMsgHandler,
                    );
                    InstallMsgHandlerContext = InstallParams.InstallMsgHandlerContext;
                } else {
                    Context = SetupInitDefaultQueueCallback(InstallParams.hwndParent);
                    if Context.is_null() {
                        break 'cleanup;
                    }
                    InstallMsgHandler = SetupDefaultQueueCallbackW as PVOID;
                    InstallMsgHandlerContext = Context;
                }

                ret = SetupInstallFromInfSectionW(
                    InstallParams.hwndParent,
                    (*(*SelectedDriver).InfFileDetails).hInf,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PCWSTR,
                    SPINST_FILES as UINT,
                    core::ptr::null_mut(),
                    (*(*SelectedDriver).InfFileDetails).DirectoryName,
                    SP_COPY_NEWER as UINT,
                    core::mem::transmute::<PVOID, PSP_FILE_CALLBACK_W>(InstallMsgHandler),
                    InstallMsgHandlerContext,
                    DeviceInfoSet,
                    DeviceInfoData,
                );
                if ret == 0 {
                    break 'done;
                }

                /* Install files from .CoInstallers section */
                lstrcatW(
                    core::ptr::addr_of_mut!(SectionName) as LPWSTR,
                    core::ptr::addr_of!(DotCoInstallers) as LPCWSTR,
                );
                ret = SetupInstallFromInfSectionW(
                    InstallParams.hwndParent,
                    (*(*SelectedDriver).InfFileDetails).hInf,
                    core::ptr::addr_of_mut!(SectionName) as *mut WCHAR as PCWSTR,
                    SPINST_FILES as UINT,
                    core::ptr::null_mut(),
                    (*(*SelectedDriver).InfFileDetails).DirectoryName,
                    SP_COPY_NEWER as UINT,
                    core::mem::transmute::<PVOID, PSP_FILE_CALLBACK_W>(InstallMsgHandler),
                    InstallMsgHandlerContext,
                    DeviceInfoSet,
                    DeviceInfoData,
                );
                if ret == 0 {
                    break 'done;
                }

                /* Set the DI_NOFILECOPY flag to prevent another
                 * installation during SetupDiInstallDevice */
                InstallParams.Flags |= DI_NOFILECOPY as DWORD;
                ret = SetupDiSetDeviceInstallParamsW(
                    DeviceInfoSet,
                    DeviceInfoData,
                    core::ptr::addr_of_mut!(InstallParams),
                );
            }

            if !Context.is_null() {
                SetupTermDefaultQueueCallback(Context);
            }
        }
    }

    TRACE!("Returning %d\n", ret);
    return ret;
}

/* Hand-ported: Haiku hit max_tokens (32768, empty) on both samples of this 529-line body. */
#[no_mangle]
pub unsafe extern "system" fn SetupDiBuildDriverInfoList(
    DeviceInfoSet: HDEVINFO,
    DeviceInfoData: PSP_DEVINFO_DATA,
    DriverType_local: DWORD,
) -> BOOL {
    let mut list: *mut DeviceInfoSet = core::ptr::null_mut();
    let mut InstallParams: SP_DEVINSTALL_PARAMS_W = core::mem::zeroed();
    let mut Buffer: PVOID = core::ptr::null_mut();
    let mut currentInfFileDetails: *mut InfFileDetails = core::ptr::null_mut();
    let mut ProviderName: LPWSTR = core::ptr::null_mut();
    let mut ManufacturerName: LPWSTR = core::ptr::null_mut();
    let mut ManufacturerSection: [WCHAR; LINE_LEN as usize + 1] = [0; LINE_LEN as usize + 1];
    let mut HardwareIDs: LPWSTR = core::ptr::null_mut();
    let mut CompatibleIDs: LPWSTR = core::ptr::null_mut();
    let mut FullInfFileName: LPWSTR = core::ptr::null_mut();
    let mut ExcludeFromSelect: LPWSTR = core::ptr::null_mut();
    let mut DriverDate: FILETIME = core::mem::zeroed();
    let mut DriverVersion: DWORDLONG = 0;
    let mut RequiredSize: DWORD = 0;
    let mut ret: BOOL = (FALSE as i32);

    TRACE!("%p %p %ld\n", DeviceInfoSet, DeviceInfoData, DriverType_local);

    'done: {
        if DeviceInfoSet.is_null() {
            SetLastError(ERROR_INVALID_HANDLE);
            break 'done;
        }
        list = DeviceInfoSet as *mut DeviceInfoSet;
        if (*list).magic != SETUP_DEVICE_INFO_SET_MAGIC as DWORD || (*list).HKLM != HKEY_LOCAL_MACHINE {
            SetLastError(ERROR_INVALID_HANDLE);
            break 'done;
        }
        if DriverType_local != SPDIT_CLASSDRIVER as DWORD && DriverType_local != SPDIT_COMPATDRIVER as DWORD {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'done;
        }
        if DriverType_local == SPDIT_COMPATDRIVER as DWORD && DeviceInfoData.is_null() {
            SetLastError(ERROR_INVALID_PARAMETER);
            break 'done;
        }
        if !DeviceInfoData.is_null() && (*DeviceInfoData).cbSize as usize != core::mem::size_of::<SP_DEVINFO_DATA>() {
            SetLastError(ERROR_INVALID_USER_BUFFER);
            break 'done;
        }

        let mut pDriverListHead: PLIST_ENTRY = &raw mut (*list).DriverListHead;
        let mut Result: BOOL;

        InstallParams.cbSize = core::mem::size_of::<SP_DEVINSTALL_PARAMS_W>() as DWORD;
        Result = SetupDiGetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &mut InstallParams);
        if Result == 0 {
            break 'done;
        }

        if !DeviceInfoData.is_null() {
            let devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
            if (*devInfo).CreationFlags & DICD_INHERIT_CLASSDRVS as DWORD == 0 {
                pDriverListHead = &raw mut (*devInfo).DriverListHead;
            }
        }

        if DriverType_local == SPDIT_COMPATDRIVER as DWORD {
            /* Get hardware and compatible IDs lists */
            Result = GetHardwareAndCompatibleIDsLists(
                DeviceInfoSet,
                DeviceInfoData,
                &mut HardwareIDs,
                core::ptr::null_mut(),
                &mut CompatibleIDs,
                core::ptr::null_mut(),
            );
            if Result == 0 {
                break 'done;
            }
            if HardwareIDs.is_null() && CompatibleIDs.is_null() {
                SetLastError(ERROR_FILE_NOT_FOUND);
                break 'done;
            }
        }

        if InstallParams.FlagsEx & DI_FLAGSEX_INSTALLEDDRIVER as DWORD != 0 {
            let mut InfFileName: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
            let mut InfFileSection: [WCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
            /* The C reaches here with DeviceInfoData unchecked and dereferences Reserved; a
             * class-driver list (DeviceInfoData NULL) with this flag would fault. Refuse it. */
            if DeviceInfoData.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
                break 'done;
            }
            let devInfo = (*DeviceInfoData).Reserved as *mut DeviceInfo;
            let len: DWORD = InfFileName.len() as DWORD;

            /* Prepend inf directory name to file name */
            let mut Size: ULONG = GetSystemWindowsDirectoryW(InfFileName.as_mut_ptr(), len);
            if Size == 0 || Size >= len {
                break 'done;
            }
            if InfFileName[0] != 0 && InfFileName[strlenW(InfFileName.as_ptr()) as usize - 1] != '\\' as WCHAR {
                strcatW(InfFileName.as_mut_ptr(), (&raw const BackSlash) as *const WCHAR);
            }
            strcatW(InfFileName.as_mut_ptr(), (&raw const InfDirectory) as *const WCHAR);

            /* Read some information from registry, before creating the driver structure */
            let hDriverKey = SETUPDI_OpenDrvKey((*list).HKLM, devInfo, KEY_QUERY_VALUE as REGSAM);
            if hDriverKey == INVALID_HANDLE_VALUE as HKEY {
                break 'done;
            }
            let used = strlenW(InfFileName.as_ptr()) as usize;
            Size = ((len as usize - used) * core::mem::size_of::<WCHAR>()) as ULONG;
            let mut rc: LONG = RegGetValueW(
                hDriverKey,
                core::ptr::null(),
                REGSTR_VAL_INFPATH,
                RRF_RT_REG_SZ as DWORD,
                core::ptr::null_mut(),
                InfFileName.as_mut_ptr().add(used) as PVOID,
                &mut Size,
            );
            if rc != ERROR_SUCCESS as LONG {
                SetLastError(rc as DWORD);
                CloseHandle(hDriverKey as HANDLE);
                break 'done;
            }
            Size = core::mem::size_of_val(&InfFileSection) as ULONG;
            rc = RegGetValueW(
                hDriverKey,
                core::ptr::null(),
                REGSTR_VAL_INFSECTION,
                RRF_RT_REG_SZ as DWORD,
                core::ptr::null_mut(),
                InfFileSection.as_mut_ptr() as PVOID,
                &mut Size,
            );
            if rc != ERROR_SUCCESS as LONG {
                SetLastError(rc as DWORD);
                CloseHandle(hDriverKey as HANDLE);
                break 'done;
            }
            TRACE!("Current driver in %s/%s\n", debugstr_w(InfFileName.as_ptr()), debugstr_w(InfFileSection.as_ptr()));
            let infFileDetails = CreateInfFileDetails(InfFileName.as_ptr());
            if infFileDetails.is_null() {
                CloseHandle(hDriverKey as HANDLE);
                break 'done;
            }
            let KnownDate: FILETIME = core::mem::zeroed(); /* FIXME */
            CloseHandle(hDriverKey as HANDLE);
            ret = AddKnownDriverToList(
                pDriverListHead,
                SPDIT_COMPATDRIVER as DWORD,
                &raw mut (*devInfo).ClassGuid,
                infFileDetails,
                InfFileName.as_ptr(),
                InfFileSection.as_ptr(), /* Yes, we don't care of section extension */
                wz!("DriverDescription"), /* FIXME */
                wz!("ProviderName"),      /* FIXME */
                wz!("ManufacturerName"),  /* FIXME */
                wz!("MatchingId"),        /* FIXME */
                KnownDate,
                0, /* FIXME: DriverVersion */
                0,
            );
            if ret == 0 {
                DereferenceInfFile(infFileDetails);
            }
            Result = (FALSE as i32);
        } else if InstallParams.Flags & DI_ENUMSINGLEINF as DWORD != 0 {
            /* InstallParams.DriverPath contains the name of a .inf file */
            RequiredSize = strlenW(((&raw const InstallParams.DriverPath) as *const WCHAR)) as DWORD + 2;
            Buffer = HeapAlloc(GetProcessHeap(), 0, (RequiredSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T);
            if Buffer.is_null() {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'done;
            }
            strcpyW(Buffer as LPWSTR, ((&raw const InstallParams.DriverPath) as *const WCHAR));
            *(Buffer as LPWSTR).add(RequiredSize as usize - 1) = 0;
            Result = (TRUE as i32);
        } else {
            /* Enumerate .inf files */
            Result = (FALSE as i32);
            RequiredSize = 32768; /* Initial buffer size */
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            while Result == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                HeapFree(GetProcessHeap(), 0, Buffer);
                Buffer = HeapAlloc(GetProcessHeap(), 0, (RequiredSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T);
                if Buffer.is_null() {
                    Result = (FALSE as i32);
                    SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                    break;
                }
                Result = SetupGetInfFileListW(
                    if InstallParams.DriverPath[0] != 0 { ((&raw const InstallParams.DriverPath) as *const WCHAR) } else { core::ptr::null() },
                    INF_STYLE_WIN4 as DWORD,
                    Buffer as LPWSTR,
                    RequiredSize,
                    &mut RequiredSize,
                );
            }
            if Result == 0 && GetLastError() == ERROR_FILE_NOT_FOUND {
                /* No .inf file in specified directory. So, we should
                 * success as we created an empty driver info list.
                 */
                ret = (TRUE as i32);
                break 'done;
            }
        }

        if Result != 0 {
            let pFullFilename: LPWSTR;

            if InstallParams.Flags & DI_ENUMSINGLEINF as DWORD != 0 {
                /* Only a filename */
                FullInfFileName = HeapAlloc(GetProcessHeap(), 0, (MAX_PATH as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
                if FullInfFileName.is_null() {
                    break 'done;
                }
                pFullFilename = FullInfFileName;
            } else if InstallParams.DriverPath[0] != 0 {
                /* Directory name specified */
                let mut len = GetFullPathNameW(((&raw const InstallParams.DriverPath) as *const WCHAR), 0, core::ptr::null_mut(), core::ptr::null_mut());
                if len == 0 {
                    break 'done;
                }
                FullInfFileName = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    ((len as usize + 1 + MAX_PATH as usize) * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as LPWSTR;
                if FullInfFileName.is_null() {
                    break 'done;
                }
                len = GetFullPathNameW(((&raw const InstallParams.DriverPath) as *const WCHAR), len, FullInfFileName, core::ptr::null_mut());
                if len == 0 {
                    break 'done;
                }
                if *FullInfFileName != 0 && *FullInfFileName.add(strlenW(FullInfFileName) as usize - 1) != '\\' as WCHAR {
                    strcatW(FullInfFileName, (&raw const BackSlash) as *const WCHAR);
                }
                pFullFilename = FullInfFileName.add(strlenW(FullInfFileName) as usize);
            } else {
                /* Nothing specified ; need to get the %SYSTEMROOT%\ directory */
                let mut len = GetSystemWindowsDirectoryW(core::ptr::null_mut(), 0);
                if len == 0 {
                    break 'done;
                }
                FullInfFileName = HeapAlloc(
                    GetProcessHeap(),
                    0,
                    ((len as usize + 1 + strlenW((&raw const InfDirectory) as *const WCHAR) as usize + MAX_PATH as usize)
                        * core::mem::size_of::<WCHAR>()) as SIZE_T,
                ) as LPWSTR;
                if FullInfFileName.is_null() {
                    break 'done;
                }
                len = GetSystemWindowsDirectoryW(FullInfFileName, len);
                if len == 0 {
                    break 'done;
                }
                if *FullInfFileName != 0 && *FullInfFileName.add(strlenW(FullInfFileName) as usize - 1) != '\\' as WCHAR {
                    strcatW(FullInfFileName, (&raw const BackSlash) as *const WCHAR);
                }
                strcatW(FullInfFileName, (&raw const InfDirectory) as *const WCHAR);
                pFullFilename = FullInfFileName.add(strlenW(FullInfFileName) as usize);
            }

            let mut filename: LPCWSTR = Buffer as LPCWSTR;
            while *filename != 0 {
                let mut ContextManufacturer: INFCONTEXT = core::mem::zeroed();
                let mut ContextDevice: INFCONTEXT = core::mem::zeroed();
                let mut ClassGuid: GUID = core::mem::zeroed();

                strcpyW(pFullFilename, filename);
                TRACE!("Opening file %s\n", debugstr_w(FullInfFileName));

                'next: {
                    currentInfFileDetails = CreateInfFileDetails(FullInfFileName);
                    if currentInfFileDetails.is_null() {
                        break 'next;
                    }

                    if GetVersionInformationFromInfFile(
                        (*currentInfFileDetails).hInf,
                        &mut ClassGuid,
                        &mut ProviderName,
                        &mut DriverDate,
                        &mut DriverVersion,
                    ) == 0
                    {
                        break 'next;
                    }

                    if DriverType_local == SPDIT_CLASSDRIVER as DWORD {
                        /* Check if the ClassGuid in this .inf file is corresponding with our needs */
                        if !IsEqualGUID!(&(*list).ClassGuid, &GUID_NULL) && !IsEqualGUID!(&(*list).ClassGuid, &ClassGuid) {
                            break 'next;
                        }
                    }

                    if InstallParams.FlagsEx & DI_FLAGSEX_ALLOWEXCLUDEDDRVS as DWORD != 0 {
                        /* Read ExcludeFromSelect control flags */
                        /* FIXME */
                    } else {
                        FIXME!("ExcludeFromSelect list ignored\n");
                    }

                    /* Get the manufacturers list */
                    Result = SetupFindFirstLineW(
                        (*currentInfFileDetails).hInf,
                        (&raw const INF_MANUFACTURER) as PCWSTR,
                        core::ptr::null(),
                        &mut ContextManufacturer,
                    );
                    while Result != 0 {
                        Result = SetupGetStringFieldW(
                            &mut ContextManufacturer,
                            0, /* Field index */
                            core::ptr::null_mut(),
                            0,
                            &mut RequiredSize,
                        );
                        if Result != 0 {
                            /* We got the needed size for the buffer */
                            ManufacturerName =
                                HeapAlloc(GetProcessHeap(), 0, (RequiredSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T) as LPWSTR;
                            if ManufacturerName.is_null() {
                                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                                break 'done;
                            }
                            Result = SetupGetStringFieldW(
                                &mut ContextManufacturer,
                                0, /* Field index */
                                ManufacturerName,
                                RequiredSize,
                                &mut RequiredSize,
                            );
                        }
                        /* Get manufacturer section name */
                        Result = SetupGetStringFieldW(
                            &mut ContextManufacturer,
                            1, /* Field index */
                            ManufacturerSection.as_mut_ptr(),
                            LINE_LEN as DWORD,
                            &mut RequiredSize,
                        );
                        if Result != 0 {
                            ManufacturerSection[RequiredSize as usize] = 0; /* Final NULL char */
                            /* Add (possible) extension to manufacturer section name */
                            Result = SetupDiGetActualSectionToInstallW(
                                (*currentInfFileDetails).hInf,
                                ManufacturerSection.as_ptr(),
                                ManufacturerSection.as_mut_ptr(),
                                LINE_LEN as DWORD,
                                core::ptr::null_mut(),
                                core::ptr::null_mut(),
                            );
                            if Result != 0
                                && SetupGetLineCountW((*currentInfFileDetails).hInf, ManufacturerSection.as_ptr()) <= 0
                            {
                                /* VibeOS: the models section for the version we report (5.2) is
                                 * empty, which current vendor INFs do on purpose ("[Realtek] ;Empty
                                 * section, so this INf does not support win2000") and put their
                                 * models under NTamd64.10.0. Windows would offer no driver; we load
                                 * NDIS 6 and KMDF drivers, so try the NT 10.0 models before giving
                                 * up. An INF with models for our version never gets here. */
                                let mut Nt10: SP_ALTPLATFORM_INFO = core::mem::zeroed();
                                let mut SystemInfo: SYSTEM_INFO = core::mem::zeroed();
                                let mut Nt10Section: [WCHAR; LINE_LEN as usize] = [0; LINE_LEN as usize];
                                let BaseLength = strlenW(ManufacturerSection.as_ptr()) as usize;

                                GetSystemInfo(&mut SystemInfo);
                                Nt10.cbSize = core::mem::size_of::<SP_ALTPLATFORM_INFO>() as DWORD;
                                Nt10.Platform = VER_PLATFORM_WIN32_NT as DWORD;
                                Nt10.MajorVersion = 10;
                                Nt10.MinorVersion = 0;
                                Nt10.ProcessorArchitecture = SystemInfo.__bindgen_anon_1.__bindgen_anon_1.wProcessorArchitecture;
                                if SetupDiGetActualSectionToInstallExW(
                                    (*currentInfFileDetails).hInf,
                                    ManufacturerSection.as_ptr(),
                                    &mut Nt10,
                                    Nt10Section.as_mut_ptr(),
                                    LINE_LEN as DWORD,
                                    core::ptr::null_mut(),
                                    core::ptr::null_mut(),
                                    core::ptr::null_mut(),
                                ) != 0
                                    && strlenW(Nt10Section.as_ptr()) as usize > BaseLength
                                    && SetupGetLineCountW((*currentInfFileDetails).hInf, Nt10Section.as_ptr()) > 0
                                {
                                    /* Once per INF per process: every device install rescans every
                                     * INF. A WARN, not an ERR: this is the expected path for every
                                     * current vendor INF, and setupapi loads afresh in each install
                                     * process, so an ERR here printed hundreds of times a boot. */
                                    static mut Reported: [ULONG; 16] = [0; 16];
                                    let mut Hash: ULONG = 5381;
                                    let mut c: LPCWSTR = FullInfFileName;
                                    while *c != 0 {
                                        Hash = Hash.wrapping_mul(33).wrapping_add(tolowerW(*c) as ULONG);
                                        c = c.add(1);
                                    }
                                    let reported = &mut *(&raw mut Reported);
                                    let mut k = 0usize;
                                    let mut Seen = false;
                                    while k < reported.len() && reported[k] != 0 {
                                        if reported[k] == Hash {
                                            Seen = true;
                                        }
                                        k += 1;
                                    }
                                    if !Seen {
                                        if k < reported.len() {
                                            reported[k] = Hash;
                                        }
                                        WARN!(
                                            "%s: no models for this version; using the NT 10.0 models section %s\n",
                                            debugstr_w(FullInfFileName),
                                            debugstr_w(Nt10Section.as_ptr())
                                        );
                                    }
                                    strcpyW(ManufacturerSection.as_mut_ptr(), Nt10Section.as_ptr());
                                }
                            }
                            if Result != 0 {
                                TRACE!("Enumerating devices in manufacturer %s\n", debugstr_w(ManufacturerSection.as_ptr()));
                                Result = SetupFindFirstLineW(
                                    (*currentInfFileDetails).hInf,
                                    ManufacturerSection.as_ptr(),
                                    core::ptr::null(),
                                    &mut ContextDevice,
                                );
                            }
                        }
                        while Result != 0 {
                            if DriverType_local == SPDIT_CLASSDRIVER as DWORD {
                                /* FIXME: Check ExcludeFromSelect list */
                                if AddDriverToList(
                                    pDriverListHead,
                                    DriverType_local,
                                    &mut ClassGuid,
                                    ContextDevice,
                                    currentInfFileDetails,
                                    FullInfFileName,
                                    ProviderName,
                                    ManufacturerName,
                                    core::ptr::null(),
                                    DriverDate,
                                    DriverVersion,
                                    0,
                                ) == 0
                                {
                                    break;
                                }
                            } else {
                                /* DriverType_local = SPDIT_COMPATDRIVER */
                                /* 1. Get all fields */
                                let FieldCount: DWORD = SetupGetFieldCount(&mut ContextDevice);
                                let mut i: DWORD = 2;
                                while i <= FieldCount {
                                    let mut DeviceId: LPWSTR = core::ptr::null_mut();
                                    Result = (FALSE as i32);
                                    RequiredSize = 128; /* Initial buffer size */
                                    SetLastError(ERROR_INSUFFICIENT_BUFFER);
                                    while Result == 0 && GetLastError() == ERROR_INSUFFICIENT_BUFFER {
                                        HeapFree(GetProcessHeap(), 0, DeviceId as PVOID);
                                        DeviceId = HeapAlloc(
                                            GetProcessHeap(),
                                            0,
                                            (RequiredSize as usize * core::mem::size_of::<WCHAR>()) as SIZE_T,
                                        ) as LPWSTR;
                                        if DeviceId.is_null() {
                                            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                                            break 'done;
                                        }
                                        Result = SetupGetStringFieldW(
                                            &mut ContextDevice,
                                            i,
                                            DeviceId,
                                            RequiredSize,
                                            &mut RequiredSize,
                                        );
                                    }
                                    if Result == 0 {
                                        HeapFree(GetProcessHeap(), 0, DeviceId as PVOID);
                                        break 'done;
                                    }
                                    /* FIXME: Check ExcludeFromSelect list */
                                    let mut DriverAlreadyAdded = false;
                                    for (ids, exact, extra) in [(HardwareIDs, 0u32, 0x1000u32), (CompatibleIDs, 0x2000, 0x3000)] {
                                        if ids.is_null() {
                                            continue;
                                        }
                                        let mut DriverRank: DWORD = 0;
                                        let mut currentId: LPCWSTR = ids;
                                        while !DriverAlreadyAdded && *currentId != 0 {
                                            if strcmpiW(DeviceId, currentId) == 0 {
                                                AddDriverToList(
                                                    pDriverListHead,
                                                    DriverType_local,
                                                    &mut ClassGuid,
                                                    ContextDevice,
                                                    currentInfFileDetails,
                                                    FullInfFileName,
                                                    ProviderName,
                                                    ManufacturerName,
                                                    currentId,
                                                    DriverDate,
                                                    DriverVersion,
                                                    DriverRank + if i == 2 { exact } else { extra + i - 3 },
                                                );
                                                DriverAlreadyAdded = true;
                                            }
                                            currentId = currentId.add(strlenW(currentId) as usize + 1);
                                            DriverRank += 1;
                                        }
                                    }
                                    HeapFree(GetProcessHeap(), 0, DeviceId as PVOID);
                                    i += 1;
                                }
                            }
                            Result = SetupFindNextLine(&mut ContextDevice, &mut ContextDevice);
                        }

                        HeapFree(GetProcessHeap(), 0, ManufacturerName as PVOID);
                        ManufacturerName = core::ptr::null_mut();
                        Result = SetupFindNextLine(&mut ContextManufacturer, &mut ContextManufacturer);
                    }

                    ret = (TRUE as i32);
                }
                /* next: */
                HeapFree(GetProcessHeap(), 0, ProviderName as PVOID);
                HeapFree(GetProcessHeap(), 0, ExcludeFromSelect as PVOID);
                ProviderName = core::ptr::null_mut();
                ExcludeFromSelect = core::ptr::null_mut();

                if !currentInfFileDetails.is_null() {
                    DereferenceInfFile(currentInfFileDetails);
                }
                currentInfFileDetails = core::ptr::null_mut();

                filename = filename.add(strlenW(filename) as usize + 1);
            }
            ret = (TRUE as i32);
        }
    }

    /* done: */
    if ret != 0 {
        if !DeviceInfoData.is_null() {
            InstallParams.Flags |= DI_DIDCOMPAT as DWORD;
            InstallParams.FlagsEx |= DI_FLAGSEX_DIDCOMPATINFO as DWORD;
        } else {
            InstallParams.Flags |= DI_DIDCLASS as DWORD;
            InstallParams.FlagsEx |= DI_FLAGSEX_DIDINFOLIST as DWORD;
        }
        ret = SetupDiSetDeviceInstallParamsW(DeviceInfoSet, DeviceInfoData, &mut InstallParams);
    }

    HeapFree(GetProcessHeap(), 0, ProviderName as PVOID);
    HeapFree(GetProcessHeap(), 0, ManufacturerName as PVOID);
    MyFree(HardwareIDs as PVOID);
    MyFree(CompatibleIDs as PVOID);
    HeapFree(GetProcessHeap(), 0, FullInfFileName as PVOID);
    HeapFree(GetProcessHeap(), 0, ExcludeFromSelect as PVOID);
    if !currentInfFileDetails.is_null() {
        DereferenceInfFile(currentInfFileDetails);
    }
    HeapFree(GetProcessHeap(), 0, Buffer);

    TRACE!("Returning %d\n", ret);
    return ret;
}
