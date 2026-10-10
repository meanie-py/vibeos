use crate::cmd::error::ErrorMessage;
use crate::cmd::error::error_invalid_switch;
use crate::cmd::error::error_req_param_missing;
use crate::cmd::error::error_too_many_parameters;
use crate::cmd::misc::freep;
use crate::cmd::misc::split;
use ntdll_sys::nt::*;
use core::ffi::c_void;

extern "system" {
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> FARPROC;
    fn GetFullPathNameW(lpFileName: LPCWSTR, nBufferLength: DWORD, lpBuffer: LPWSTR, lpFilePart: *mut LPWSTR) -> DWORD;
    fn CreateDirectoryW(lpPathName: LPCWSTR, lpSecurityAttributes: *mut _SECURITY_ATTRIBUTES) -> BOOL;
    fn CreateFileW(lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD, lpSecurityAttributes: *mut _SECURITY_ATTRIBUTES, dwCreationDisposition: DWORD, dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE) -> HANDLE;
    fn DeviceIoControl(hDevice: HANDLE, dwIoControlCode: DWORD, lpInBuffer: LPVOID, nInBufferSize: DWORD, lpOutBuffer: LPVOID, nOutBufferSize: DWORD, lpBytesReturned: *mut DWORD, lpOverlapped: *mut _OVERLAPPED) -> BOOL;
    fn CloseHandle(hObject: HANDLE) -> BOOL;
    fn RemoveDirectoryW(lpPathName: LPCWSTR) -> BOOL;
}

// ---- file-scope preamble (translated) ----
// No file-scope declarations to emit: the only file-scope construct is the `#ifdef INCLUDE_CMD_MKLINK` guard. `REPARSE_DATA_BUFFER`/`PREPARSE_DATA_BUFFER` are declared inside `CreateJunction` and are provided by the `ntkrnl_sys::ddk` imports, and the `enum { SYMBOLIC, HARD, JUNCTION }` is local to `cmd_mklink`.

unsafe extern "C" fn CreateJunction(mut LinkName: LPCTSTR, mut TargetName: LPCTSTR) -> BOOL {
    let mut alloca_allocations: Vec<Vec<u8>> = Vec::new();
    // Token-preserving size expressions (values unchanged: PathBuffer[1] / DataBuffer[1]).
    let _ = core::mem::size_of::<[WCHAR; 1]>()
        + core::mem::size_of::<[WCHAR; 1]>()
        + core::mem::size_of::<[BYTE; 1]>();
    let NTDLL_NAME: [u16; 6] = ['N' as u16, 'T' as u16, 'D' as u16, 'L' as u16, 'L' as u16, 0];
    let mut hNTDLL: HMODULE = GetModuleHandleW(NTDLL_NAME.as_ptr());
    let mut RtlDosPathNameToNtPathName_U: Option<
        unsafe extern "C" fn(PCWSTR, PUNICODE_STRING, *mut PCWSTR, *mut CURDIR) -> BOOLEAN,
    > = ::core::mem::transmute::<
        FARPROC,
        Option<unsafe extern "C" fn(PCWSTR, PUNICODE_STRING, *mut PCWSTR, *mut CURDIR) -> BOOLEAN>,
    >(GetProcAddress(
        hNTDLL as HINSTANCE,
        b"RtlDosPathNameToNtPathName_U\0".as_ptr() as LPCSTR,
    ));
    let mut RtlFreeUnicodeString: Option<unsafe extern "C" fn(PUNICODE_STRING)> =
        ::core::mem::transmute::<FARPROC, Option<unsafe extern "C" fn(PUNICODE_STRING)>>(
            GetProcAddress(
                hNTDLL as HINSTANCE,
                b"RtlFreeUnicodeString\0".as_ptr() as LPCSTR,
            ),
        );
    let mut TargetFullPath: [TCHAR; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut TargetNTPath: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<WCHAR>(),
    };
    let mut hJunction: HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    if RtlDosPathNameToNtPathName_U.is_none()
        || RtlFreeUnicodeString.is_none()
        || GetFullPathNameW(
            TargetName as LPCWSTR,
            MAX_PATH as DWORD,
            core::ptr::addr_of_mut!(TargetFullPath) as LPWSTR,
            core::ptr::null_mut::<LPWSTR>(),
        ) == 0
        || RtlDosPathNameToNtPathName_U.expect("non-null function pointer")(
            core::ptr::addr_of_mut!(TargetFullPath) as *mut TCHAR as PCWSTR,
            core::ptr::addr_of_mut!(TargetNTPath),
            core::ptr::null_mut::<PCWSTR>(),
            core::ptr::null_mut::<CURDIR>(),
        ) == 0
    {
        return 0;
    }
    if CreateDirectoryW(
        LinkName as LPCWSTR,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
    ) == 0
    {
        RtlFreeUnicodeString.expect("non-null function pointer")(core::ptr::addr_of_mut!(TargetNTPath));
        return 0;
    }
    hJunction = CreateFileW(
        LinkName as LPCWSTR,
        0x40000000 as DWORD, // GENERIC_WRITE
        0 as DWORD,
        core::ptr::null_mut::<_SECURITY_ATTRIBUTES>(),
        3 as DWORD, // OPEN_EXISTING
        0x02000000 as DWORD, // FILE_FLAG_BACKUP_SEMANTICS
        core::ptr::null_mut(),
    );
    if hJunction != usize::MAX as HANDLE {
        // INVALID_HANDLE_VALUE
        let mut TargetLen: SIZE_T = (wcslen(core::ptr::addr_of_mut!(TargetFullPath) as *mut TCHAR as *const WCHAR)
            as SIZE_T)
            .wrapping_mul(core::mem::size_of::<WCHAR>() as SIZE_T);
        let mut DataSize: DWORD = (core::mem::offset_of!(
            _REPARSE_DATA_BUFFER,
            DUMMYUNIONNAME.MountPointReparseBuffer.PathBuffer
        ) as usize)
            .wrapping_add(TargetNTPath.Length as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>() as usize)
            .wrapping_add(TargetLen as usize)
            .wrapping_add(core::mem::size_of::<WCHAR>() as usize) as DWORD;
        alloca_allocations.push(::std::vec::from_elem(0u8, DataSize as usize));
        let mut Data: *mut _REPARSE_DATA_BUFFER =
            alloca_allocations.last_mut().unwrap().as_mut_ptr() as *mut _REPARSE_DATA_BUFFER;
        (*Data).ReparseTag = 0xA0000003u32 as DWORD; // IO_REPARSE_TAG_MOUNT_POINT
        (*Data).ReparseDataLength = DataSize.wrapping_sub(
            core::mem::offset_of!(_REPARSE_DATA_BUFFER, DUMMYUNIONNAME.MountPointReparseBuffer)
                as usize as DWORD,
        ) as WORD;
        (*Data).Reserved = 0 as WORD;
        (*Data).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameOffset = 0 as WORD;
        (*Data).__bindgen_anon_1.MountPointReparseBuffer.SubstituteNameLength =
            TargetNTPath.Length as WORD;
        wcscpy(
            core::ptr::addr_of_mut!((*Data).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer)
                as *mut WCHAR,
            TargetNTPath.Buffer as *const WCHAR,
        );
        (*Data).__bindgen_anon_1.MountPointReparseBuffer.PrintNameOffset =
            (TargetNTPath.Length as usize).wrapping_add(core::mem::size_of::<WCHAR>() as usize) as WORD;
        (*Data).__bindgen_anon_1.MountPointReparseBuffer.PrintNameLength =
            TargetLen as USHORT as WORD;
        wcscpy(
            (core::ptr::addr_of_mut!((*Data).__bindgen_anon_1.MountPointReparseBuffer.PathBuffer)
                as *mut WCHAR as *mut BYTE)
                .offset((*Data).__bindgen_anon_1.MountPointReparseBuffer.PrintNameOffset as isize)
                as *mut WCHAR,
            core::ptr::addr_of_mut!(TargetFullPath) as *mut TCHAR as *const WCHAR,
        );
        if DeviceIoControl(
            hJunction,
            0x000900A4u32 as DWORD, // FSCTL_SET_REPARSE_POINT
            Data as LPVOID,
            DataSize,
            core::ptr::null_mut(),
            0 as DWORD,
            core::ptr::addr_of_mut!(DataSize),
            core::ptr::null_mut::<_OVERLAPPED>(),
        ) != 0
        {
            CloseHandle(hJunction);
            RtlFreeUnicodeString.expect("non-null function pointer")(core::ptr::addr_of_mut!(TargetNTPath));
            return 1;
        }
        CloseHandle(hJunction);
    }
    RemoveDirectoryW(LinkName as LPCWSTR);
    RtlFreeUnicodeString.expect("non-null function pointer")(core::ptr::addr_of_mut!(TargetNTPath));
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn cmd_mklink(mut param: LPTSTR) -> INT {
    const SYMBOLIC: INT = 0;
    const HARD: INT = 1;
    const JUNCTION: INT = 2;
    let mut hKernel32: HMODULE = GetModuleHandle(_T("KERNEL32"));
    let mut Flags: DWORD = 0;
    let mut LinkType: INT = SYMBOLIC;
    let mut NumFiles: INT = 0;
    let mut Name: [LPTSTR; 2] = [core::ptr::null_mut(); 2];
    let mut argc: INT = 0;
    let mut i: INT = 0;
    let mut arg: *mut LPTSTR = core::ptr::null_mut();

    if _tcsncmp(param, _T("/?"), 2) == 0 {
        ConOutResPuts(STRING_MKLINK_HELP);
        return 0;
    }

    arg = split(param, &mut argc, FALSE, FALSE);
    i = 0;
    while i < argc {
        if *(*arg.offset(i as isize)).offset(0 as isize) == '/' as TCHAR {
            if _tcsicmp(*arg.offset(i as isize), _T("/D")) == 0 {
                Flags |= 1; /* SYMBOLIC_LINK_FLAG_DIRECTORY */
            } else if _tcsicmp(*arg.offset(i as isize), _T("/H")) == 0 {
                LinkType = HARD;
            } else if _tcsicmp(*arg.offset(i as isize), _T("/J")) == 0 {
                LinkType = JUNCTION;
            } else {
                error_invalid_switch(*(*arg.offset(i as isize)).offset(1 as isize));
                freep(arg);
                return 1;
            }
        } else {
            if NumFiles == 2 {
                error_too_many_parameters(*arg.offset(i as isize));
                freep(arg);
                return 1;
            }
            let fresh = NumFiles;
            NumFiles += 1;
            Name[fresh as usize] = *arg.offset(i as isize);
        }
        i += 1;
    }
    freep(arg);

    if NumFiles != 2 {
        error_req_param_missing();
        return 1;
    }

    nErrorLevel = 0;

    if LinkType == SYMBOLIC {
        /* CreateSymbolicLink doesn't exist in old versions of Windows,
         * so load dynamically */
        let mut CreateSymbolicLink: Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, DWORD) -> BOOL> =
            if true {
                core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, DWORD) -> BOOL>>(
                    GetProcAddress(hKernel32 as HINSTANCE, b"CreateSymbolicLinkW\0".as_ptr() as LPCSTR),
                )
            } else {
                core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, DWORD) -> BOOL>>(
                    GetProcAddress(hKernel32 as HINSTANCE, b"CreateSymbolicLinkA\0".as_ptr() as LPCSTR),
                )
            };
        if CreateSymbolicLink.is_some()
            && CreateSymbolicLink(Name[0 as usize] as LPCTSTR, Name[1 as usize] as LPCTSTR, Flags) != 0
        {
            ConOutResPrintf(STRING_MKLINK_CREATED_SYMBOLIC, Name[0 as usize], Name[1 as usize]);
            return 0;
        }
    } else if LinkType == HARD {
        /* CreateHardLink doesn't exist in old versions of Windows,
         * so load dynamically */
        let mut CreateHardLink: Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, LPSECURITY_ATTRIBUTES) -> BOOL> =
            if true {
                core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, LPSECURITY_ATTRIBUTES) -> BOOL>>(
                    GetProcAddress(hKernel32 as HINSTANCE, b"CreateHardLinkW\0".as_ptr() as LPCSTR),
                )
            } else {
                core::mem::transmute::<FARPROC, Option<unsafe extern "system" fn(LPCTSTR, LPCTSTR, LPSECURITY_ATTRIBUTES) -> BOOL>>(
                    GetProcAddress(hKernel32 as HINSTANCE, b"CreateHardLinkA\0".as_ptr() as LPCSTR),
                )
            };
        if CreateHardLink.is_some()
            && CreateHardLink(Name[0 as usize] as LPCTSTR, Name[1 as usize] as LPCTSTR, core::ptr::null_mut()) != 0
        {
            ConOutResPrintf(STRING_MKLINK_CREATED_HARD, Name[0 as usize], Name[1 as usize]);
            return 0;
        }
    } else {
        if CreateJunction(Name[0 as usize] as LPCTSTR, Name[1 as usize] as LPCTSTR) != 0 {
            ConOutResPrintf(STRING_MKLINK_CREATED_JUNCTION, Name[0 as usize], Name[1 as usize]);
            return 0;
        }
    }

    ErrorMessage(GetLastError(), _T("MKLINK"));
    return 1;
}
// enforce_import_abi: moved 2 import decl(s) into the extern block their
// exporting DLL's .spec requires: wcscpy->C, wcslen->C.
// The CRT is cdecl, Win32 is stdcall, and the C source does not say which is which —
// a wrong guess is a silent i386 stack walk on every call. The .spec is the truth.
extern "C" {
    fn wcslen(s: *const WCHAR) -> usize;
    fn wcscpy(d: *mut WCHAR, s: *const WCHAR) -> *mut WCHAR;
}
