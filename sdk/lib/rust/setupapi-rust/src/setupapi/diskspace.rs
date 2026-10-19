use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct DRIVE_ENTRY {
    pub lpzName: [WCHAR; 20],
    pub dwFreeSpace: LONGLONG,
    pub dwWantedSpace: LONGLONG,
}
pub type LPDRIVE_ENTRY = *mut DRIVE_ENTRY;

#[repr(C)]
pub struct DISKSPACELIST {
    pub dwDriveCount: DWORD,
    pub Drives: [DRIVE_ENTRY; 26],
}
pub type LPDISKSPACELIST = *mut DISKSPACELIST;

#[no_mangle]
pub unsafe extern "system" fn SetupCreateDiskSpaceListW(Reserved1: PVOID, Reserved2: DWORD, Flags: UINT) -> HDSKSPC {
    let mut drives: [WCHAR; 255] = [0; 255];
    let mut rc: DWORD = 0;
    let mut ptr: *mut WCHAR = core::ptr::null_mut();
    let mut list: LPDISKSPACELIST = core::ptr::null_mut();

    if !Reserved1.is_null() || Reserved2 != 0 || Flags & !(SPDSL_IGNORE_DISK as UINT) != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }

    rc = GetLogicalDriveStringsW(255 as DWORD, core::ptr::addr_of_mut!(drives) as LPWSTR);

    if rc == 0 as DWORD {
        return core::ptr::null_mut();
    }

    list = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<DISKSPACELIST>() as SIZE_T,
    ) as LPDISKSPACELIST;
    // Divergence: the C writes through this pointer unchecked.
    if list.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return core::ptr::null_mut();
    }

    (*list).dwDriveCount = 0 as DWORD;

    ptr = core::ptr::addr_of_mut!(drives) as *mut WCHAR;

    while *ptr != 0 {
        let type_0: DWORD = GetDriveTypeW(ptr as LPCWSTR) as DWORD;
        if type_0 == DRIVE_FIXED as DWORD {
            let mut clusters: DWORD = 0;
            let mut sectors: DWORD = 0;
            let mut bytes: DWORD = 0;
            let mut total: DWORD = 0;
            lstrcpyW(
                core::ptr::addr_of_mut!((*list).Drives[(*list).dwDriveCount as usize].lpzName) as LPWSTR,
                ptr as LPCWSTR,
            );
            GetDiskFreeSpaceW(
                ptr as LPCWSTR,
                core::ptr::addr_of_mut!(sectors),
                core::ptr::addr_of_mut!(bytes),
                core::ptr::addr_of_mut!(clusters),
                core::ptr::addr_of_mut!(total),
            );
            (*list).Drives[(*list).dwDriveCount as usize].dwFreeSpace =
                // Divergence: the C multiplies in DWORD and widens after, so any drive
                // with 4 GB or more free reported the size modulo 4 GB.
                (clusters as LONGLONG) * (sectors as LONGLONG) * (bytes as LONGLONG);
            (*list).Drives[(*list).dwDriveCount as usize].dwWantedSpace = 0 as LONGLONG;
            (*list).dwDriveCount = (*list).dwDriveCount.wrapping_add(1);
        }
        ptr = ptr.offset((lstrlenW(ptr as LPCWSTR) + 1 as i32) as isize);
    }
    return list as HDSKSPC;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCreateDiskSpaceListA(
    Reserved1: PVOID,
    Reserved2: DWORD,
    Flags: UINT,
) -> HDSKSPC {
    SetupCreateDiskSpaceListW(Reserved1, Reserved2, Flags)
}

#[no_mangle]
pub unsafe extern "system" fn SetupDuplicateDiskSpaceListW(
    mut DiskSpace: HDSKSPC,
    mut Reserved1: PVOID,
    mut Reserved2: DWORD,
    mut Flags: UINT,
) -> HDSKSPC {
    let mut list_copy: *mut DISKSPACELIST = core::ptr::null_mut::<DISKSPACELIST>();
    let mut list_original: *mut DISKSPACELIST = DiskSpace as *mut DISKSPACELIST;
    if !Reserved1.is_null() || Reserved2 != 0 || Flags != 0 {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    if DiskSpace.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return core::ptr::null_mut();
    }
    list_copy = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        core::mem::size_of::<DISKSPACELIST>() as SIZE_T,
    ) as *mut DISKSPACELIST;
    if list_copy.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return core::ptr::null_mut();
    }
    // C's struct assignment: a bitwise copy (DISKSPACELIST is not Copy in Rust).
    core::ptr::copy_nonoverlapping(list_original, list_copy, 1);
    return list_copy as HDSKSPC;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDuplicateDiskSpaceListA(
    DiskSpace: HDSKSPC,
    Reserved1: PVOID,
    Reserved2: DWORD,
    Flags: UINT,
) -> HDSKSPC {
    SetupDuplicateDiskSpaceListW(DiskSpace, Reserved1, Reserved2, Flags)
}

#[no_mangle]
pub unsafe extern "system" fn SetupAddInstallSectionToDiskSpaceListA(
    DiskSpace: HDSKSPC,
    InfHandle: HINF,
    LayoutInfHandle: HINF,
    SectionName: LPCSTR,
    Reserved1: PVOID,
    Reserved2: UINT,
) -> BOOL {
    // FIXME ("Stub\n");
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn SetupQuerySpaceRequiredOnDriveW(
    DiskSpace: HDSKSPC,
    DriveSpec: LPCWSTR,
    SpaceRequired: *mut LONGLONG,
    Reserved1: PVOID,
    Reserved2: UINT,
) -> BOOL {
    let mut driveW: *mut WCHAR = core::ptr::null_mut::<WCHAR>();
    let mut i: ::core::ffi::c_uint = 0;
    let mut list: LPDISKSPACELIST = DiskSpace as LPDISKSPACELIST;
    let mut rc: BOOL = (FALSE as i32);
    static bkslsh: [WCHAR; 2] = [0x005Cu16, 0x0000];

    if DiskSpace.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return (FALSE) as BOOL;
    }

    if DriveSpec.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return (FALSE) as BOOL;
    }

    driveW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        ((lstrlenW(DriveSpec) + 2 as ::core::ffi::c_int) as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as *mut WCHAR;
    if driveW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return (FALSE) as BOOL;
    }

    lstrcpyW(driveW as LPWSTR, DriveSpec);
    lstrcatW(driveW as LPWSTR, core::ptr::addr_of!(bkslsh) as LPCWSTR);

    if (*__wine_dbch___default).flags as ::core::ffi::c_int
        & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
        != 0
    {
        let __dbch: *mut __wine_debug_channel = __wine_dbch___default;
        let __dbcl: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
        ros_dbg_log(
            __dbcl,
            __dbch,
            b"dll/win32/setupapi/diskspace.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            b"SetupQuerySpaceRequiredOnDriveW\0" as *const u8 as *const ::core::ffi::c_char,
            179 as ::core::ffi::c_int,
            b"Looking for drive %s\n\0" as *const u8 as *const ::core::ffi::c_char,
            debugstr_w(driveW),
        );
    }

    i = 0 as ::core::ffi::c_uint;
    while (i as DWORD) < (*list).dwDriveCount {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & (1 as ::core::ffi::c_int) << __WINE_DBCL_TRACE as ::core::ffi::c_int
            != 0
        {
            let __dbch_0: *mut __wine_debug_channel = __wine_dbch___default;
            let __dbcl_0: __wine_debug_class = (__WINE_DBCL_TRACE as i32);
            ros_dbg_log(
                __dbcl_0,
                __dbch_0,
                b"dll/win32/setupapi/diskspace.c\0"
                    as *const u8 as *const ::core::ffi::c_char,
                b"SetupQuerySpaceRequiredOnDriveW\0" as *const u8
                    as *const ::core::ffi::c_char,
                183 as ::core::ffi::c_int,
                b"checking drive %s\n\0" as *const u8 as *const ::core::ffi::c_char,
                debugstr_w(
                    &raw mut (*(&raw mut (*list).Drives as *mut DRIVE_ENTRY)
                        .offset(i as isize))
                        .lpzName as *mut WCHAR,
                ),
            );
        }
        if lstrcmpW(
            driveW as LPCWSTR,
            &raw mut (*(&raw mut (*list).Drives as *mut DRIVE_ENTRY).offset(i as isize)).lpzName
                as *mut WCHAR as LPCWSTR,
        ) == 0 as ::core::ffi::c_int
        {
            rc = TRUE as BOOL;
            *SpaceRequired = (*list).Drives[i as usize].dwWantedSpace;
            break;
        } else {
            i = i.wrapping_add(1);
        }
    }

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        driveW as PVOID,
    );

    if rc == 0 {
        SetLastError(ERROR_INVALID_DRIVE as DWORD);
    }
    return rc;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQuerySpaceRequiredOnDriveA(
    DiskSpace: HDSKSPC,
    DriveSpec: LPCSTR,
    SpaceRequired: *mut LONGLONG,
    Reserved1: PVOID,
    Reserved2: UINT,
) -> BOOL {
    let mut len: DWORD = 0;
    let mut DriveSpecW: LPWSTR = core::ptr::null_mut();
    let mut ret: BOOL = 0;

    if DriveSpec.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE as BOOL;
    }

    if DiskSpace.is_null() {
        SetLastError(ERROR_INVALID_HANDLE as DWORD);
        return FALSE as BOOL;
    }

    len = MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        DriveSpec,
        -(1 as ::core::ffi::c_int),
        core::ptr::null_mut(),
        0 as ::core::ffi::c_int,
    ) as DWORD;

    DriveSpecW = HeapAlloc(
        GetProcessHeap(),
        0 as DWORD,
        (len as SIZE_T).wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T),
    ) as LPWSTR;
    if DriveSpecW.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE as BOOL;
    }

    MultiByteToWideChar(
        CP_ACP as UINT,
        0 as DWORD,
        DriveSpec,
        -(1 as ::core::ffi::c_int),
        DriveSpecW,
        len as ::core::ffi::c_int,
    );

    ret = SetupQuerySpaceRequiredOnDriveW(
        DiskSpace,
        DriveSpecW as LPCWSTR,
        SpaceRequired,
        Reserved1,
        Reserved2,
    );

    HeapFree(
        GetProcessHeap(),
        0 as DWORD,
        DriveSpecW as PVOID,
    );

    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn SetupDestroyDiskSpaceList(DiskSpace: HDSKSPC) -> BOOL {
    let list: LPDISKSPACELIST = DiskSpace as LPDISKSPACELIST;
    HeapFree(GetProcessHeap(), 0, list as PVOID);
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupAddToDiskSpaceListA(
    diskspace: HDSKSPC,
    targetfile: PCSTR,
    filesize: LONGLONG,
    operation: UINT,
    reserved1: PVOID,
    reserved2: UINT,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupAddToDiskSpaceListW(diskspace: HDSKSPC, targetfile: PCWSTR, filesize: LONGLONG, operation: UINT, reserved1: PVOID, reserved2: UINT) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED);
    return (FALSE) as BOOL;
}
