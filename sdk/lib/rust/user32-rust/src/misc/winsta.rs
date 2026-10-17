use win32k_sys::win32k::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::windows::window::HeapAlloc;
#[allow(unused_imports)]
use core::ffi::c_void;

// ntdll Rtl* string helpers live in crate::internal (NOT the kernel ntkrnl_sys crate).
use crate::internal::{
    RtlCreateUnicodeStringFromAsciiz, RtlFreeUnicodeString, RtlInitUnicodeString,
};
use crate::winbase::{wcslen, NT_SUCCESS};
// GetProcessWindowStation is an inline ntwrapper wrapper (NtUser* multiplexer),
// not an external link symbol — import it instead of declaring it extern.
use crate::ntwrapper::GetProcessWindowStation;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// Nt*/Rtl error-mapping syscalls + the Logon notifier (cross-TU, cdecl on amd64/arm64,
// fastcall for Logon on i386 — declared per its export ABI).
// ntdll Nt*/Rtl* exports are stdcall on i386 (`_Name@N`) — declare extern "system".
extern "system" {
    fn NtClose(Handle: HANDLE) -> NTSTATUS;
    fn NtOpenDirectoryObject(
        DirectoryHandle: *mut HANDLE,
        DesiredAccess: ACCESS_MASK,
        ObjectAttributes: *mut _OBJECT_ATTRIBUTES,
    ) -> NTSTATUS;
    fn RtlNtStatusToDosError(Status: NTSTATUS) -> ULONG;
}

#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn Logon(IsLogon: BOOL);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn Logon(IsLogon: BOOL);
}

// STATUS_BUFFER_TOO_SMALL (ntstatus.h)
const STATUS_BUFFER_TOO_SMALL: NTSTATUS = 0xC0000023u32 as NTSTATUS;

// SECURITY_ATTRIBUTES — bindgen renders _SECURITY_ATTRIBUTES as opaque; reproduce the
// real layout locally so the field reads type-check (faithful to winbase.h).
#[repr(C)]
struct SECURITY_ATTRIBUTES_LAYOUT {
    nLength: DWORD,
    lpSecurityDescriptor: *mut core::ffi::c_void,
    bInheritHandle: BOOL,
}
extern "system" {
    fn GetProcessHeap() -> *mut core::ffi::c_void;
    fn HeapFree( hHeap: *mut core::ffi::c_void, dwFlags: u32, lpMem: *mut core::ffi::c_void, ) -> i32;
    fn SetLastError(dwErrCode: u32);
    fn WideCharToMultiByte( CodePage: u32, dwFlags: u32, lpWideCharStr: LPCWSTR, cchWideChar: i32, lpMultiByteStr: LPSTR, cbMultiByte: i32, lpDefaultChar: LPCSTR, lpUsedDefaultChar: *mut i32, ) -> i32;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct tagENUMNAMESASCIICONTEXT {
    pub UserEnumFunc: NAMEENUMPROCA,
    pub UserContext: LPARAM,
}
pub type ENUMNAMESASCIICONTEXT = tagENUMNAMESASCIICONTEXT;
pub type PENUMNAMESASCIICONTEXT = *mut tagENUMNAMESASCIICONTEXT;

#[no_mangle]
pub unsafe extern "system" fn CreateWindowStationA(
    lpwinsta: LPCSTR,
    dwReserved: u32,
    dwDesiredAccess: ACCESS_MASK,
    lpsa: LPSECURITY_ATTRIBUTES,
) -> HWINSTA {
    let hWinSta: HWINSTA;
    let mut WindowStationNameU: _UNICODE_STRING = core::mem::zeroed();

    if !lpwinsta.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut WindowStationNameU, lpwinsta);
    } else {
        RtlInitUnicodeString(&mut WindowStationNameU, core::ptr::null_mut());
    }

    hWinSta = CreateWindowStationW(
        WindowStationNameU.Buffer,
        dwReserved,
        dwDesiredAccess,
        lpsa,
    );

    if !lpwinsta.is_null() {
        RtlFreeUnicodeString(&mut WindowStationNameU);
    }

    hWinSta
}

#[no_mangle]
pub unsafe extern "system" fn CreateWindowStationW(
    lpwinsta: LPCWSTR,
    dwReserved: u32,
    dwDesiredAccess: ACCESS_MASK,
    lpsa: LPSECURITY_ATTRIBUTES,
) -> HWINSTA {
    let Status: i32;
    let hWinSta: HWINSTA;
    let mut WindowStationName: _UNICODE_STRING = core::mem::zeroed();
    let mut WindowStationsDir: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(
        &mut WindowStationsDir,
        w!("\\Windows\\WindowStations").as_ptr() as LPCWSTR,
    );
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut hWindowStationsDir: *mut core::ffi::c_void = core::ptr::null_mut();

    let mut lpwinsta = lpwinsta;

    if !lpwinsta.is_null() && *lpwinsta != 0 {
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut WindowStationsDir;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenDirectoryObject(
            &mut hWindowStationsDir,
            DIRECTORY_CREATE_OBJECT,
            &mut ObjectAttributes,
        );
        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }

        RtlInitUnicodeString(&mut WindowStationName, lpwinsta);
    } else {
        lpwinsta = core::ptr::null_mut();
        hWindowStationsDir = core::ptr::null_mut();
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = hWindowStationsDir;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE | OBJ_OPENIF;
    ObjectAttributes.ObjectName = if !lpwinsta.is_null() {
        &mut WindowStationName
    } else {
        core::ptr::null_mut()
    };
    ObjectAttributes.SecurityDescriptor = if !lpsa.is_null() {
        (*(lpsa as *const SECURITY_ATTRIBUTES_LAYOUT)).lpSecurityDescriptor
    } else {
        core::ptr::null_mut()
    };
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if !lpsa.is_null() && (*(lpsa as *const SECURITY_ATTRIBUTES_LAYOUT)).bInheritHandle != 0 {
        ObjectAttributes.Attributes |= OBJ_INHERIT;
    }

    hWinSta = NtUserCreateWindowStation(&mut ObjectAttributes, dwDesiredAccess, 0, 0, 0, 0, 0);

    if !hWindowStationsDir.is_null() {
        NtClose(hWindowStationsDir);
    }

    hWinSta
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn EnumNamesW(
    mut WindowStation: HWINSTA,
    EnumFunc: NAMEENUMPROCW,
    Context: LPARAM,
    Desktops: i32,
) -> i32 {
    EnumNamesW_impl(WindowStation, EnumFunc, Context, Desktops)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EnumNamesW(
    WindowStation: HWINSTA,
    EnumFunc: NAMEENUMPROCW,
    Context: LPARAM,
    Desktops: i32,
) -> i32 {
    EnumNamesW_impl(WindowStation, EnumFunc, Context, Desktops)
}

unsafe fn EnumNamesW_impl(
    mut WindowStation: HWINSTA,
    EnumFunc: NAMEENUMPROCW,
    Context: LPARAM,
    Desktops: i32,
) -> i32 {

    let mut Buffer: [i8; 256] = [0; 256];
    let NameList: *mut core::ffi::c_void;
    let mut Name: *mut u16;
    let mut Status;
    let mut RequiredSize: u32 = 0;
    let EntryCount: u32;
    let mut Ret: i32;

    if WindowStation.is_null() && Desktops != 0 {
        WindowStation = GetProcessWindowStation();
    }

    Status = NtUserBuildNameList(
        WindowStation,
        core::mem::size_of::<[i8; 256]>() as u32,
        Buffer.as_mut_ptr() as *mut core::ffi::c_void,
        &mut RequiredSize,
    );
    if NT_SUCCESS(Status) {
        NameList = Buffer.as_mut_ptr() as *mut core::ffi::c_void;
    } else if Status == STATUS_BUFFER_TOO_SMALL {
        NameList = HeapAlloc(GetProcessHeap(), 0, (RequiredSize) as SIZE_T);
        if NameList.is_null() {
            return 0;
        }

        Status = NtUserBuildNameList(
            WindowStation,
            RequiredSize,
            NameList,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            HeapFree(GetProcessHeap(), 0, NameList);
            SetLastError(RtlNtStatusToDosError(Status));
            return 0;
        }
    } else {
        SetLastError(RtlNtStatusToDosError(Status));
        return 0;
    }

    EntryCount = *(NameList as *mut u32);
    Name = (NameList as *mut i8).add(core::mem::size_of::<u32>()) as *mut u16;
    Ret = 1;
    let mut CurrentEntry: u32 = 0;
    while CurrentEntry < EntryCount && Ret != 0 {
        Ret = (EnumFunc.unwrap())(Name, Context);
        Name = Name.add(wcslen(Name) + 1);
        CurrentEntry += 1;
    }

    if NameList != Buffer.as_mut_ptr() as *mut core::ffi::c_void {
        HeapFree(GetProcessHeap(), 0, NameList);
    }

    Ret
}

#[allow(non_snake_case)]
unsafe extern "system" fn EnumNamesCallback(Name: LPWSTR, Param: LPARAM) -> i32 {

    const CP_ACP: u32 = 0;

    let Context = Param as *mut tagENUMNAMESASCIICONTEXT;
    let mut FixedNameA: [core::ffi::c_char; 32] = [0; 32];
    let NameA: LPSTR;
    let Len: INT;
    let Ret: i32;

    Len = WideCharToMultiByte(
        CP_ACP,
        0,
        Name,
        -1,
        core::ptr::null_mut(),
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    if Len <= 0 {
        return 0;
    } else if (Len as usize) <= core::mem::size_of_val(&FixedNameA) {
        NameA = FixedNameA.as_mut_ptr();
    } else {
        NameA = HeapAlloc(GetProcessHeap(), 0, (Len) as SIZE_T) as LPSTR;
        if NameA.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    }

    if 0 == WideCharToMultiByte(
        CP_ACP,
        0,
        Name,
        -1,
        NameA,
        Len,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ) {
        if NameA != FixedNameA.as_mut_ptr() {
            HeapFree(GetProcessHeap(), 0, NameA as *mut core::ffi::c_void);
        }
        return 0;
    }

    Ret = (*Context).UserEnumFunc.unwrap()(NameA, (*Context).UserContext);

    if NameA != FixedNameA.as_mut_ptr() {
        HeapFree(GetProcessHeap(), 0, NameA as *mut core::ffi::c_void);
    }

    Ret
}

// (Duplicate tagENUMNAMESASCIICONTEXT / INT / CHAR / BOOL definitions removed — the
//  struct + its ENUMNAMESASCIICONTEXT alias are defined in the file-scope preamble above,
//  and INT/BOOL come from the win32k_sys glob import.)

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn EnumNamesA(
    WindowStation: HWINSTA,
    EnumFunc: NAMEENUMPROCA,
    Context: LPARAM,
    Desktops: i32,
) -> i32 {
    let mut PrivateContext: ENUMNAMESASCIICONTEXT = core::mem::zeroed();

    PrivateContext.UserEnumFunc = EnumFunc;
    PrivateContext.UserContext = Context;

    EnumNamesW(
        WindowStation,
        core::mem::transmute::<unsafe extern "system" fn(LPWSTR, LPARAM) -> i32, NAMEENUMPROCW>(EnumNamesCallback),
        core::ptr::addr_of_mut!(PrivateContext) as LPARAM,
        Desktops,
    )
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn EnumNamesA(
    WindowStation: HWINSTA,
    EnumFunc: NAMEENUMPROCA,
    Context: LPARAM,
    Desktops: i32,
) -> i32 {
    let mut PrivateContext: ENUMNAMESASCIICONTEXT = core::mem::zeroed();

    PrivateContext.UserEnumFunc = EnumFunc;
    PrivateContext.UserContext = Context;

    EnumNamesW(
        WindowStation,
        core::mem::transmute::<unsafe extern "system" fn(LPWSTR, LPARAM) -> i32, NAMEENUMPROCW>(EnumNamesCallback),
        core::ptr::addr_of_mut!(PrivateContext) as LPARAM,
        Desktops,
    )
}

#[no_mangle]
pub unsafe extern "system" fn EnumWindowStationsA(
    EnumFunc: WINSTAENUMPROCA,
    Context: LPARAM,
) -> i32 {
    EnumNamesA(core::ptr::null_mut(), EnumFunc, Context, 0)
}

#[no_mangle]
pub unsafe extern "system" fn EnumWindowStationsW(
    EnumFunc: WINSTAENUMPROCW,
    Context: LPARAM,
) -> i32 {
    EnumNamesW(core::ptr::null_mut(), EnumFunc, Context, 0)
}

#[no_mangle]
pub unsafe extern "system" fn GetWinStationInfo(pUnknown: *mut core::ffi::c_void) -> i32 {
    NtUserCallOneParam(pUnknown as DWORD_PTR, ONEPARAM_ROUTINE_GETWINSTAINFO as _) as i32
}

#[no_mangle]
pub unsafe extern "system" fn OpenWindowStationA(
    lpszWinSta: LPCSTR,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HWINSTA {
    let mut WindowStationNameU: _UNICODE_STRING = core::mem::zeroed();

    if !lpszWinSta.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut WindowStationNameU, lpszWinSta);
    } else {
        RtlInitUnicodeString(&mut WindowStationNameU, core::ptr::null_mut());
    }

    let hWinSta = OpenWindowStationW(WindowStationNameU.Buffer, fInherit, dwDesiredAccess);

    if !lpszWinSta.is_null() {
        RtlFreeUnicodeString(&mut WindowStationNameU);
    }

    hWinSta
}

#[no_mangle]
pub unsafe extern "system" fn OpenWindowStationW(
    lpszWinSta: LPCWSTR,
    fInherit: i32,
    dwDesiredAccess: ACCESS_MASK,
) -> HWINSTA {
    let Status: i32;
    let hWinSta: HWINSTA;
    let mut WindowStationName: _UNICODE_STRING = core::mem::zeroed();

    let mut WindowStationsDir: _UNICODE_STRING = core::mem::zeroed();
    let dir_str: &[u16] = &[
        b'\\' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16,
        b'o' as u16, b'w' as u16, b's' as u16, b'\\' as u16, b'W' as u16,
        b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16,
        b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'i' as u16,
        b'o' as u16, b'n' as u16, b's' as u16, 0u16,
    ];
    WindowStationsDir.Length = (dir_str.len() as u16 - 1) * 2;
    WindowStationsDir.MaximumLength = (dir_str.len() as u16) * 2;
    WindowStationsDir.Buffer = dir_str.as_ptr() as *mut u16;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let hWindowStationsDir: *mut core::ffi::c_void;
    let mut hWindowStationsDir_local: *mut core::ffi::c_void = core::ptr::null_mut();

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(WindowStationsDir);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenDirectoryObject(
        core::ptr::addr_of_mut!(hWindowStationsDir_local),
        DIRECTORY_TRAVERSE,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }
    hWindowStationsDir = hWindowStationsDir_local;

    RtlInitUnicodeString(core::ptr::addr_of_mut!(WindowStationName), lpszWinSta);

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = hWindowStationsDir;
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(WindowStationName);
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    if fInherit != 0 {
        ObjectAttributes.Attributes |= OBJ_INHERIT;
    }

    hWinSta = NtUserOpenWindowStation(core::ptr::addr_of_mut!(ObjectAttributes), dwDesiredAccess);

    NtClose(hWindowStationsDir);

    hWinSta
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowStationUser(
    hWindowStation: HWINSTA,
    pluid: PLUID,
    psid: PSID,
    size: DWORD,
) -> BOOL {
    let mut Success: BOOL;

    Success = NtUserSetWindowStationUser(hWindowStation, pluid, psid, size);
    if Success != 0 {
        let LuidNone = LUID { LowPart: 0, HighPart: 0 };
        let IsLogon: BOOL = if !pluid.is_null()
            && !((*pluid).LowPart == LuidNone.LowPart && (*pluid).HighPart == LuidNone.HighPart)
        {
            1
        } else {
            0
        };

        Logon(IsLogon);
    }

    Success
}
