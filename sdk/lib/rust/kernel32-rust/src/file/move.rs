use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{InitializeObjectAttributes, RtlCopyMemory, CloseHandle};

extern "system" {
    fn Basep8BitStringToDynamicUnicodeString( DestinationString: *mut _UNICODE_STRING, Source: LPCSTR, ) -> BOOLEAN;
    // sibling kernel32 modules (cohab-resolved at link)
    fn QueryDosDeviceW(lpDeviceName: LPCWSTR, lpTargetPath: LPWSTR, ucchMax: DWORD) -> DWORD;
    fn GetComputerNameW(lpBuffer: LPWSTR, lpnSize: LPDWORD) -> BOOL;
    fn FlushFileBuffers(hFile: HANDLE) -> BOOL;
    fn DeleteFileW(lpFileName: LPCWSTR) -> BOOL;
    fn SetFileAttributesW(lpFileName: LPCWSTR, dwFileAttributes: DWORD) -> BOOL;
    fn CopyFileW(lpExistingFileName: LPCWSTR, lpNewFileName: LPCWSTR, bFailIfExists: BOOL) -> BOOL;
    fn SetEndOfFile(hFile: HANDLE) -> BOOL;
    fn GetProcessHeap() -> HANDLE;
}

extern "C" {
    // ntstrsafe RtlStringCbPrintfW is FORCEINLINE (no symbol) → call the CRT
    // _snwprintf directly (crate idiom; see client/except.rs / ntstrsafe.rs).
    fn _snwprintf(buffer: *mut u16, count: usize, format: *const u16, ...) -> i32;
    fn BasepCopyFileExW(
        lpExistingFileName: LPCWSTR,
        lpNewFileName: LPCWSTR,
        lpProgressRoutine: LPPROGRESS_ROUTINE,
        lpData: LPVOID,
        pbCancel: LPBOOL,
        dwCopyFlags: DWORD,
        dwBasepFlags: DWORD,
        lpExistingHandle: LPHANDLE,
        lpNewHandle: LPHANDLE,
    ) -> BOOL;
}

// dll/win32/kernel32/include/kernel32.h
const PRIV_DELETE_ON_SUCCESS: DWORD = 0x1;
const PRIV_ALLOW_NON_TRACKABLE: DWORD = 0x2;
// ntstatus.h
const STATUS_NOT_SAME_DEVICE: NTSTATUS = 0xC000_00D4u32 as NTSTATUS;

// RTL_CONSTANT_STRING builds a UNICODE_STRING from a wide literal. The translation
// passed a NUL-terminated UTF-16 byte buffer cast to *const i8; reinterpret as wide.
unsafe fn RTL_CONSTANT_STRING(s: *const i8) -> _UNICODE_STRING {
    let p = s as *const u16;
    let mut len: USHORT = 0;
    while *p.add((len as usize) / 2) != 0 {
        len += core::mem::size_of::<WCHAR>() as USHORT;
    }
    _UNICODE_STRING {
        Length: len,
        MaximumLength: len + core::mem::size_of::<WCHAR>() as USHORT,
        Buffer: p as PWSTR,
    }
}

// NDEBUG: Wine/ReactOS debug shims compile out. Macros swallow mixed args.
#[inline] unsafe fn UNIMPLEMENTED_DBGBREAK(_msg: *const i8) {}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _COPY_PROGRESS_CONTEXT {
    pub Flags: ULONG,
    pub UserRoutine: LPPROGRESS_ROUTINE,
    pub UserData: LPVOID,
}
pub type COPY_PROGRESS_CONTEXT = _COPY_PROGRESS_CONTEXT;
pub type PCOPY_PROGRESS_CONTEXT = *mut _COPY_PROGRESS_CONTEXT;

#[no_mangle]
pub unsafe extern "system" fn BasepMoveFileDelayed(
    ExistingPath: PUNICODE_STRING,
    NewPath: PUNICODE_STRING,
    KeyId: INT,
    CreateIfNotFound: BOOL,
) -> NTSTATUS {
    const STRING_LENGTH: ULONG = 0x400;
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Buffer: PVOID;
    let mut BufferBegin: PVOID;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut PendingOperations: PWSTR;
    let mut BufferWrite: PWSTR;
    let mut DataSize: ULONG = 0;
    let mut BufferLength: ULONG;
    let mut StringLength: ULONG = STRING_LENGTH;
    let mut SessionManagerString: _UNICODE_STRING = core::mem::zeroed();
    let mut PendingOperationsString: _UNICODE_STRING = core::mem::zeroed();
    let mut PendingOperationsBuffer: [WCHAR; (core::mem::size_of::<WCHAR>() * 32) / core::mem::size_of::<WCHAR>() + 6] = [0; (core::mem::size_of::<WCHAR>() * 32) / core::mem::size_of::<WCHAR>() + 6];

    RtlInitUnicodeString(
        &mut SessionManagerString,
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Session Manager").as_ptr(),
    );

    if KeyId == 1 {
        PendingOperations = w!("PendingFileRenameOperations").as_ptr() as PWSTR;
    } else {
        // C ignores the RtlStringCbPrintfW return here; _snwprintf with
        // count = cch-1 leaves room for the NUL _snwprintf writes on a short fit.
        _snwprintf(
            PendingOperationsBuffer.as_mut_ptr(),
            (core::mem::size_of_val(&PendingOperationsBuffer) / core::mem::size_of::<WCHAR>())
                .saturating_sub(1),
            w!("PendingFileRenameOperations%d").as_ptr(),
            KeyId,
        );
        PendingOperations = PendingOperationsBuffer.as_mut_ptr();
    }
    RtlInitUnicodeString(&mut PendingOperationsString, PendingOperations);

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut SessionManagerString,
        OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtCreateKey(
        &mut KeyHandle,
        GENERIC_READ | GENERIC_WRITE,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        REG_OPTION_NON_VOLATILE,
        core::ptr::null_mut(),
    );
    if Status == STATUS_ACCESS_DENIED {
        Status = NtCreateKey(
            &mut KeyHandle,
            GENERIC_READ | GENERIC_WRITE,
            &mut ObjectAttributes,
            0,
            core::ptr::null_mut(),
            REG_OPTION_BACKUP_RESTORE,
            core::ptr::null_mut(),
        );
    }

    if !NT_SUCCESS(Status) {
        return Status;
    }

    BufferLength = (*NewPath).Length as u32 + (*ExistingPath).Length as u32 + STRING_LENGTH + 3 * core::mem::size_of::<WCHAR>() as u32;

    loop {
        Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength as SIZE_T);
        if Buffer.is_null() {
            NtClose(KeyHandle);
            return STATUS_NO_MEMORY;
        }

        Status = NtQueryValueKey(
            KeyHandle,
            &mut PendingOperationsString,
            KeyValuePartialInformation,
            Buffer,
            StringLength,
            &mut DataSize,
        );
        if Status != STATUS_BUFFER_OVERFLOW {
            break;
        }

        StringLength = DataSize;
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
        BufferLength = (*ExistingPath).Length as u32 + StringLength + (*NewPath).Length as u32 + 3 * core::mem::size_of::<WCHAR>() as u32;
        if BufferLength < StringLength {
            NtClose(KeyHandle);
            return STATUS_BUFFER_TOO_SMALL;
        }
    }

    if !NT_SUCCESS(Status) && (Status != STATUS_OBJECT_NAME_NOT_FOUND || CreateIfNotFound == 0) {
        NtClose(KeyHandle);
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
        return Status;
    }

    if !NT_SUCCESS(Status) {
        BufferBegin = Buffer;
        BufferWrite = Buffer as PWSTR;
    } else {
        let PartialInfo: PKEY_VALUE_PARTIAL_INFORMATION = Buffer as PKEY_VALUE_PARTIAL_INFORMATION;

        BufferBegin = (*PartialInfo).Data.as_ptr() as PVOID;
        BufferWrite = (((*PartialInfo).Data.as_ptr() as ULONG_PTR) + (*PartialInfo).DataLength as ULONG_PTR - core::mem::size_of::<WCHAR>() as ULONG_PTR) as PWSTR;
    }

    RtlCopyMemory(
        BufferWrite as PVOID,
        (*ExistingPath).Buffer as *const c_void,
        (*ExistingPath).Length as SIZE_T,
    );
    BufferWrite = BufferWrite.add(((*ExistingPath).Length as u32 / core::mem::size_of::<WCHAR>() as u32) as usize);
    *BufferWrite = UNICODE_NULL;
    BufferWrite = BufferWrite.add(1);
    RtlCopyMemory(
        BufferWrite as PVOID,
        (*NewPath).Buffer as *const c_void,
        (*NewPath).Length as SIZE_T,
    );
    BufferWrite = BufferWrite.add(((*NewPath).Length as u32 / core::mem::size_of::<WCHAR>() as u32) as usize);
    *BufferWrite = UNICODE_NULL;
    BufferWrite = BufferWrite.add(1);
    *BufferWrite = UNICODE_NULL;

    Status = NtSetValueKey(
        KeyHandle,
        &mut PendingOperationsString,
        0,
        REG_MULTI_SZ,
        BufferBegin,
        ((BufferWrite as ULONG_PTR).wrapping_sub(BufferBegin as ULONG_PTR) + core::mem::size_of::<WCHAR>() as ULONG_PTR) as ULONG,
    );

    NtClose(KeyHandle);
    RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BasepGetComputerNameFromNtPath(
    NewPath: *const _UNICODE_STRING,
    NewHandle: HANDLE,
    ComputerName: PWSTR,
    ComputerNameLength: *mut ULONG,
) -> DWORD {
    let mut Query: BOOL = 0;
    let mut Letter: WCHAR;
    let mut AbsolutePath: PWSTR = core::ptr::null_mut();
    let mut EndOfName: PWSTR;
    let mut AbsolutePathLength: USHORT = 0;
    let mut NameLength: USHORT;
    let mut TargetDevice: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut DeviceName: [WCHAR; 3] = ['A' as WCHAR, ':' as WCHAR, '\0' as WCHAR];
    let UncString: _UNICODE_STRING = RTL_CONSTANT_STRING(b"\\??\\UNC\0".as_ptr() as *const i8);
    let GlobalString: _UNICODE_STRING = RTL_CONSTANT_STRING(b"\\??\\\0".as_ptr() as *const i8);

    // L"\Device\LanmanRedirector\;"
    static LANMAN_REDIRECTOR_PREFIX: [u16; 27] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'L' as u16, b'a' as u16,
        b'n' as u16, b'm' as u16, b'a' as u16, b'n' as u16, b'R' as u16, b'e' as u16, b'd' as u16, b'i' as u16, b'r' as u16, b'e' as u16,
        b'c' as u16, b't' as u16, b'o' as u16, b'r' as u16, b'\\' as u16, b';' as u16, 0,
    ];
    // L"\Device\Harddisk"
    static HARDDISK_PREFIX: [u16; 17] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'H' as u16, b'a' as u16,
        b'r' as u16, b'd' as u16, b'd' as u16, b'i' as u16, b's' as u16, b'k' as u16, 0,
    ];
    // L"\Device\CdRom"
    static CDROM_PREFIX: [u16; 14] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'C' as u16, b'd' as u16,
        b'R' as u16, b'o' as u16, b'm' as u16, 0,
    ];
    // L"\Device\Floppy"
    static FLOPPY_PREFIX: [u16; 15] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'F' as u16, b'l' as u16,
        b'o' as u16, b'p' as u16, b'p' as u16, b'y' as u16, 0,
    ];
    // L"\Device\WinDfs\"
    static WINDFS_PREFIX: [u16; 16] = [
        b'\\' as u16, b'D' as u16, b'e' as u16, b'v' as u16, b'i' as u16, b'c' as u16, b'e' as u16, b'\\' as u16, b'W' as u16, b'i' as u16,
        b'n' as u16, b'D' as u16, b'f' as u16, b's' as u16, b'\\' as u16, 0,
    ];

    // DPRINT removed

    if RtlPrefixUnicodeString(&UncString as *const _UNICODE_STRING, NewPath, 1) != 0 {
        if (*NewPath).Length <= UncString.Length {
            return ERROR_BAD_PATHNAME;
        }

        AbsolutePath = (*NewPath).Buffer.offset((UncString.Length / core::mem::size_of::<WCHAR>() as u16) as isize);
        AbsolutePathLength = (*NewPath).Length - UncString.Length;

        Query = 1;
    } else if RtlPrefixUnicodeString(&GlobalString as *const _UNICODE_STRING, NewPath, 1) != 0
        && *((*NewPath).Buffer.offset(5)) == ':' as WCHAR
    {
        Letter = *((*NewPath).Buffer.offset(4));
        if Letter >= 'a' as WCHAR && Letter <= 'z' as WCHAR {
            Letter -= ('a' as WCHAR - 'A' as WCHAR);
        }
        DeviceName[0] = Letter;

        if QueryDosDeviceW(
            DeviceName.as_ptr(),
            TargetDevice.as_mut_ptr(),
            TargetDevice.len() as u32,
        ) == 0
        {
            return GetLastError();
        }

        if TargetDevice.as_ptr() == wcsstr(
            TargetDevice.as_ptr(),
            LANMAN_REDIRECTOR_PREFIX.as_ptr() as *const WCHAR,
        ) {
            if TargetDevice[26] == DeviceName[0] && TargetDevice[27] == ':' as WCHAR {
                let Path: PWSTR = wcschr(
                    TargetDevice.as_ptr().offset(28),
                    '\\' as WCHAR,
                );
                if Path.is_null() {
                    return ERROR_BAD_PATHNAME;
                }

                AbsolutePath = Path.offset(1);
                AbsolutePathLength = (core::mem::size_of::<WCHAR>() as u16)
                    // C `AbsolutePath - TargetDevice` counts WCHARs; subtracting addresses
                    // counted bytes, shrinking the bound and wrapping it past 130 WCHARs.
                    * ((TargetDevice.len() as isize - AbsolutePath.offset_from(TargetDevice.as_ptr()))
                        as u16);
            } else {
                return ERROR_BAD_PATHNAME;
            }
        } else if TargetDevice.as_ptr() == wcsstr(
            TargetDevice.as_ptr(),
            HARDDISK_PREFIX.as_ptr() as *const WCHAR,
        ) || TargetDevice.as_ptr() == wcsstr(
            TargetDevice.as_ptr(),
            CDROM_PREFIX.as_ptr() as *const WCHAR,
        ) || TargetDevice.as_ptr() == wcsstr(
            TargetDevice.as_ptr(),
            FLOPPY_PREFIX.as_ptr() as *const WCHAR,
        ) {
            if GetComputerNameW(ComputerName, ComputerNameLength) == 0 {
                return GetLastError();
            }

            return ERROR_SUCCESS;
        } else if TargetDevice.as_ptr() == wcsstr(
            TargetDevice.as_ptr(),
            WINDFS_PREFIX.as_ptr() as *const WCHAR,
        ) {
            Query = 1;
        } else {
            return ERROR_BAD_PATHNAME;
        }
    } else {
        return ERROR_BAD_PATHNAME;
    }

    if Query != 0 {
        UNIMPLEMENTED_DBGBREAK(b"Querying DFS not implemented!\n\0".as_ptr() as *const i8);
        AbsolutePath = core::ptr::null_mut();
        AbsolutePathLength = 0;
    }

    EndOfName = AbsolutePath;
    if AbsolutePathLength != 0 {
        NameLength = 0;
        while NameLength < AbsolutePathLength {
            if *EndOfName == '\\' as WCHAR {
                break;
            } else if *EndOfName == '.' as WCHAR {
                return ERROR_BAD_PATHNAME;
            }

            EndOfName = EndOfName.offset(1);
            NameLength += core::mem::size_of::<WCHAR>() as u16;
        }
    }

    // DELIBERATE divergence from move.c, which has `NameLength = EndOfName - AbsolutePath`
    // (a WCHAR count) and then uses NameLength as BYTES everywhere after: the byte-length
    // limits, `RtlCopyMemory(.., NameLength)` and `/ sizeof(WCHAR)` — so the C returns half
    // the computer name. The byte difference here is what that code means; keep it.
    NameLength = (EndOfName as isize - AbsolutePath as isize) as u16; // nolint: ptr-byte-diff
    if NameLength >= AbsolutePathLength
        || NameLength >= (MAX_COMPUTERNAME_LENGTH as u16) * core::mem::size_of::<WCHAR>() as u16
    {
        return ERROR_BAD_PATHNAME;
    }

    if (NameLength as u32) + core::mem::size_of::<WCHAR>() as u32
        > (*ComputerNameLength) * core::mem::size_of::<WCHAR>() as u32
    {
        return ERROR_BUFFER_OVERFLOW;
    }

    RtlCopyMemory(
        ComputerName as *mut core::ffi::c_void,
        AbsolutePath as *const core::ffi::c_void,
        NameLength as SIZE_T,
    );
    *ComputerNameLength = (NameLength / core::mem::size_of::<WCHAR>() as u16) as u32;
    *ComputerName.offset((NameLength / core::mem::size_of::<WCHAR>() as u16) as isize) = UNICODE_NULL;

    return ERROR_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn BasepNotifyTrackingService(
    ExistingHandle: *mut HANDLE,
    ObjectAttributes: *const _OBJECT_ATTRIBUTES,
    NewHandle: HANDLE,
    NewPath: *const _UNICODE_STRING,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut ComputerNameLength: ULONG;
    let mut FileAttributes: ULONG;
    let mut ComputerName: [WCHAR; MAX_COMPUTERNAME_LENGTH as usize + 1] = [0; MAX_COMPUTERNAME_LENGTH as usize + 1];
    let mut ComputerNameStringA: OEM_STRING = core::mem::zeroed();
    let mut ComputerNameStringBuffer: [CHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut ComputerNameStringW: _UNICODE_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut BasicInfo: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut hFullWrite: HANDLE = core::ptr::null_mut();
    let mut FileTrackingInfo: FILE_TRACKING_INFORMATION = core::mem::zeroed();

    Status = STATUS_SUCCESS;
    ComputerNameLength = ComputerName.len() as ULONG;

    if BasepGetComputerNameFromNtPath(NewPath, NewHandle, ComputerName.as_mut_ptr(), &mut ComputerNameLength) != 0
    {
        FileTrackingInfo.ObjectInformationLength = 0;
    } else {
        RtlInitEmptyAnsiString(
            &mut ComputerNameStringA,
            ComputerNameStringBuffer.as_mut_ptr(),
            core::mem::size_of::<[CHAR; MAX_PATH as usize + 1]>() as USHORT,
        );

        RtlInitUnicodeString(&mut ComputerNameStringW, ComputerName.as_ptr());
        Status = RtlUnicodeStringToOemString(&mut ComputerNameStringA, &ComputerNameStringW, 0);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        core::ptr::copy_nonoverlapping(
            ComputerNameStringA.Buffer,
            FileTrackingInfo.ObjectInformation.as_mut_ptr(),
            ComputerNameStringA.Length as usize,
        );
        *FileTrackingInfo.ObjectInformation.as_mut_ptr().add(ComputerNameStringA.Length as usize) = 0i8;
        FileTrackingInfo.ObjectInformationLength = ComputerNameStringA.Length as ULONG + 1;
    }

    FileTrackingInfo.DestinationFile = NewHandle;

    Status = NtSetInformationFile(
        *ExistingHandle,
        &mut IoStatusBlock,
        &mut FileTrackingInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_TRACKING_INFORMATION>() as u32,
        FileTrackingInformation,
    );
    if Status != STATUS_ACCESS_DENIED {
        return Status;
    }

    CloseHandle(*ExistingHandle);

    Status = NtOpenFile(
        ExistingHandle,
        SYNCHRONIZE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES,
        ObjectAttributes as POBJECT_ATTRIBUTES,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if !NT_SUCCESS(Status) {
        *ExistingHandle = INVALID_HANDLE_VALUE;
        return Status;
    }

    Status = NtQueryInformationFile(
        *ExistingHandle,
        &mut IoStatusBlock,
        &mut BasicInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    FileAttributes = BasicInfo.FileAttributes & !FILE_ATTRIBUTE_READONLY;
    core::ptr::write_bytes(&mut BasicInfo as *mut _, 0, 1);
    BasicInfo.FileAttributes = FileAttributes;

    Status = NtSetInformationFile(
        *ExistingHandle,
        &mut IoStatusBlock,
        &mut BasicInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
        FileBasicInformation,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = NtOpenFile(
        &mut hFullWrite,
        GENERIC_WRITE | SYNCHRONIZE,
        ObjectAttributes as POBJECT_ATTRIBUTES,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );
    if NT_SUCCESS(Status) {
        NtClose(*ExistingHandle);
        *ExistingHandle = hFullWrite;

        Status = NtSetInformationFile(
            *ExistingHandle,
            &mut IoStatusBlock,
            &mut FileTrackingInfo as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_TRACKING_INFORMATION>() as u32,
            FileTrackingInformation,
        );
    }

    if !NT_SUCCESS(Status) {
        BasicInfo.FileAttributes |= FILE_ATTRIBUTE_READONLY;

        Status = NtSetInformationFile(
            *ExistingHandle,
            &mut IoStatusBlock,
            &mut BasicInfo as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
            FileBasicInformation,
        );
    }

    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn BasepOpenFileForMove(
    File: LPCWSTR,
    RelativeNtName: PUNICODE_STRING,
    NtName: *mut LPWSTR,
    FileHandle: PHANDLE,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    DesiredAccess: ACCESS_MASK,
    ShareAccess: ULONG,
    mut OpenOptions: ULONG,
) -> NTSTATUS {
    let mut RelativeName: RTL_RELATIVE_NAME_U = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut TagInfo: FILE_ATTRIBUTE_TAG_INFORMATION = core::mem::zeroed();
    let mut IntShareAccess: ULONG;
    let mut HasRelative: BOOLEAN = 0;

    // _SEH2_TRY
    {
        // Zero output
        RtlInitEmptyUnicodeString(RelativeNtName, core::ptr::null_mut(), 0);
        *NtName = core::ptr::null_mut();

        if RtlDosPathNameToRelativeNtPathName_U(
            File,
            RelativeNtName,
            core::ptr::null_mut(),
            &mut RelativeName,
        ) == 0
        {
            Status = STATUS_OBJECT_PATH_NOT_FOUND;
            // _SEH2_LEAVE
            // goto cleanup
        } else {
            HasRelative = 1;
            *NtName = (*RelativeNtName).Buffer;

            if RelativeName.RelativeName.Length != 0 {
                (*RelativeNtName).Length = RelativeName.RelativeName.Length;
                (*RelativeNtName).MaximumLength = RelativeName.RelativeName.MaximumLength;
                (*RelativeNtName).Buffer = RelativeName.RelativeName.Buffer;
            } else {
                RelativeName.ContainingDirectory = core::ptr::null_mut();
            }

            InitializeObjectAttributes(
                ObjectAttributes,
                RelativeNtName,
                OBJ_CASE_INSENSITIVE,
                RelativeName.ContainingDirectory,
                core::ptr::null_mut(),
            );
            // Force certain flags here, given ops we'll do
            IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_WRITE;
            OpenOptions |= FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT;

            // We'll try to read reparse tag
            Status = NtOpenFile(
                FileHandle,
                DesiredAccess | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                ObjectAttributes,
                &mut IoStatusBlock,
                IntShareAccess,
                OpenOptions | FILE_OPEN_REPARSE_POINT,
            );
            if NT_SUCCESS(Status) {
                // Attempt the read
                Status = NtQueryInformationFile(
                    *FileHandle,
                    &mut IoStatusBlock,
                    &mut TagInfo as *mut _ as *mut core::ffi::c_void,
                    core::mem::size_of::<FILE_ATTRIBUTE_TAG_INFORMATION>() as ULONG,
                    FileAttributeTagInformation,
                );

                // Return if failure with a status that wouldn't mean the FSD cannot support reparse points
                if !NT_SUCCESS(Status)
                    && (Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_PARAMETER)
                {
                    // _SEH2_LEAVE
                    // goto cleanup
                } else {
                    if NT_SUCCESS(Status) {
                        // This cannot happen on mount points
                        if (TagInfo.FileAttributes & FILE_ATTRIBUTE_DEVICE) != 0
                            || TagInfo.ReparseTag == IO_REPARSE_TAG_MOUNT_POINT
                        {
                            // _SEH2_LEAVE
                            // goto cleanup
                        } else {
                            NtClose(*FileHandle);
                            *FileHandle = INVALID_HANDLE_VALUE;

                            IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_DELETE;
                        }
                    } else {
                        NtClose(*FileHandle);
                        *FileHandle = INVALID_HANDLE_VALUE;

                        IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_DELETE;
                    }
                }
            } else if Status == STATUS_INVALID_PARAMETER {
                IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_WRITE;
            } else {
                // _SEH2_LEAVE
                // goto cleanup
            }

            // Reattempt to open normally, following reparse point if needed
            Status = NtOpenFile(
                FileHandle,
                DesiredAccess | SYNCHRONIZE,
                ObjectAttributes,
                &mut IoStatusBlock,
                IntShareAccess,
                OpenOptions,
            );
        }
    }

    // _SEH2_FINALLY
    if HasRelative != 0 {
        RtlReleaseRelativeName(&mut RelativeName);
    }
    // _SEH2_END

    Status
}

#[no_mangle]
pub unsafe extern "system" fn BasepMoveFileCopyProgress(
    TotalFileSize: LARGE_INTEGER,
    TotalBytesTransferred: LARGE_INTEGER,
    StreamSize: LARGE_INTEGER,
    StreamBytesTransferred: LARGE_INTEGER,
    dwStreamNumber: DWORD,
    dwCallbackReason: DWORD,
    hSourceFile: HANDLE,
    hDestinationFile: HANDLE,
    lpData: LPVOID,
) -> DWORD {
    let mut Ret: DWORD = 0;
    let Context: PCOPY_PROGRESS_CONTEXT = lpData as PCOPY_PROGRESS_CONTEXT;

    if (*Context).Flags & MOVEFILE_WRITE_THROUGH != 0 {
        if dwCallbackReason == 0 {
            if StreamBytesTransferred.QuadPart == StreamSize.QuadPart {
                FlushFileBuffers(hDestinationFile);
            }
        }
    }

    if let Some(UserRoutine) = (*Context).UserRoutine {
        Ret = UserRoutine(
            TotalFileSize,
            TotalBytesTransferred,
            StreamSize,
            StreamBytesTransferred,
            dwStreamNumber,
            dwCallbackReason,
            hSourceFile,
            hDestinationFile,
            (*Context).UserData,
        );
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileWithProgressW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    dwFlags: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut NewBuffer: PWSTR;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut CopyContext: COPY_PROGRESS_CONTEXT = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut RenameInfo: *mut FILE_RENAME_INFORMATION;
    let mut NewPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut ExistingPathU: _UNICODE_STRING = core::mem::zeroed();
    let mut TagInfo: FILE_ATTRIBUTE_TAG_INFORMATION = core::mem::zeroed();
    let mut SourceHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut NewHandle: HANDLE;
    let mut ExistingHandle: HANDLE;
    let mut Ret: BOOL = 0;
    let mut ReplaceIfExists: BOOL;
    let mut DelayUntilReboot: BOOL;
    let mut AttemptReopenWithoutReparse: BOOL;

    // DPRINT removed

    NewPathU.Buffer = core::ptr::null_mut();
    ExistingPathU.Buffer = core::ptr::null_mut();

    /* _SEH2_TRY */
    'leave: {
        /* Don't allow renaming to a disk */
        if (!lpNewFileName.is_null() && RtlIsDosDeviceName_U(lpNewFileName) != 0) {
            BaseSetLastNTError(STATUS_OBJECT_NAME_COLLISION);
            /* _SEH2_LEAVE */
            break 'leave;
        }

        ReplaceIfExists = if (dwFlags & MOVEFILE_REPLACE_EXISTING) != 0 { 1 } else { 0 };

        /* Get file path */
        if RtlDosPathNameToNtPathName_U(lpExistingFileName, &mut ExistingPathU, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
            BaseSetLastNTError(STATUS_OBJECT_PATH_NOT_FOUND);
            /* _SEH2_LEAVE */
            break 'leave;
        }

        /* Sanitize input */
        DelayUntilReboot = if (dwFlags & MOVEFILE_DELAY_UNTIL_REBOOT) != 0 { 1 } else { 0 };
        if DelayUntilReboot != 0 && (dwFlags & MOVEFILE_CREATE_HARDLINK) != 0 {
            BaseSetLastNTError(STATUS_INVALID_PARAMETER);
            /* _SEH2_LEAVE */
            break 'leave;
        }

        /* Unless we manage a proper opening, we'll attempt to reopen without reparse support */
        AttemptReopenWithoutReparse = 1;
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut ExistingPathU,
            OBJ_CASE_INSENSITIVE,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        /* Attempt to open source file */
        Status = NtOpenFile(
            &mut SourceHandle,
            FILE_READ_ATTRIBUTES | DELETE | SYNCHRONIZE,
            &mut ObjectAttributes,
            &mut IoStatusBlock,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            FILE_OPEN_FOR_BACKUP_INTENT
                | if (dwFlags & MOVEFILE_WRITE_THROUGH) != 0 {
                    FILE_WRITE_THROUGH
                } else {
                    0
                },
        );
        if !NT_SUCCESS(Status) {
            /* If we failed and the file doesn't exist, don't attempt to reopen without reparse */
            if DelayUntilReboot != 0
                && (Status == STATUS_SHARING_VIOLATION
                    || Status == STATUS_OBJECT_NAME_NOT_FOUND
                    || Status == STATUS_OBJECT_PATH_NOT_FOUND)
            {
                /* Here we don't fail completely, as we postpone the operation to reboot.
                 * File might exist afterwards, and we don't need a handle here. */
                SourceHandle = INVALID_HANDLE_VALUE;
                AttemptReopenWithoutReparse = 0;
            }
            /* If we failed for any reason than unsupported reparse, fail completely */
            else if Status != STATUS_INVALID_PARAMETER {
                BaseSetLastNTError(Status);
                /* _SEH2_LEAVE */
                break 'leave;
            }
        } else {
            /* We managed to open, so query information */
            Status = NtQueryInformationFile(
                SourceHandle,
                &mut IoStatusBlock,
                &mut TagInfo as *mut FILE_ATTRIBUTE_TAG_INFORMATION as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_ATTRIBUTE_TAG_INFORMATION>() as u32,
                FileAttributeTagInformation,
            );
            if !NT_SUCCESS(Status) {
                /* Do not tolerate any other error than something related to not supported operation */
                if Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_PARAMETER {
                    BaseSetLastNTError(Status);
                    /* _SEH2_LEAVE */
                    break 'leave;
                }

                /* Not a reparse point, no need to reopen, it's fine */
                AttemptReopenWithoutReparse = 0;
            }
            /* Validate the reparse point (do we support it?) */
            else if (TagInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0
                && TagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT
            {
                NtClose(SourceHandle);
                SourceHandle = INVALID_HANDLE_VALUE;
            } else {
                /* Mount point, let's rename it */
                AttemptReopenWithoutReparse = 0;
            }
        }

        /* Simply reopen if required */
        if AttemptReopenWithoutReparse != 0 {
            Status = NtOpenFile(
                &mut SourceHandle,
                DELETE | SYNCHRONIZE,
                &mut ObjectAttributes,
                &mut IoStatusBlock,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                if (dwFlags & MOVEFILE_WRITE_THROUGH) != 0 {
                    FILE_WRITE_THROUGH
                } else {
                    0
                },
            );
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                /* _SEH2_LEAVE */
                break 'leave;
            }
        }

        /* Nullify string if we're to use it */
        if DelayUntilReboot != 0 && lpNewFileName.is_null() {
            RtlInitUnicodeString(&mut NewPathU, core::ptr::null());
        }
        /* Check whether path exists */
        else if RtlDosPathNameToNtPathName_U(lpNewFileName, &mut NewPathU, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
            BaseSetLastNTError(STATUS_OBJECT_PATH_NOT_FOUND);
            /* _SEH2_LEAVE */
            break 'leave;
        }

        /* Handle postponed renaming */
        if DelayUntilReboot != 0 {
            /* If new file exists and we're allowed to replace, then mark the path with ! */
            if ReplaceIfExists != 0 && NewPathU.Length != 0 {
                NewBuffer = RtlAllocateHeap(
                    RtlGetProcessHeap(),
                    0,
                    (NewPathU.Length as SIZE_T + core::mem::size_of::<u16>() as SIZE_T) as SIZE_T,
                ) as PWSTR;
                if NewBuffer.is_null() {
                    BaseSetLastNTError(STATUS_NO_MEMORY);
                    /* _SEH2_LEAVE */
                    break 'leave;
                }

                *NewBuffer.offset(0) = '!' as u16;
                RtlCopyMemory(
                    NewBuffer.offset(1) as *mut core::ffi::c_void,
                    NewPathU.Buffer as *const core::ffi::c_void,
                    NewPathU.Length as SIZE_T,
                );
                NewPathU.Length += core::mem::size_of::<u16>() as u16;
                NewPathU.MaximumLength += core::mem::size_of::<u16>() as u16;
                RtlFreeHeap(RtlGetProcessHeap(), 0, NewPathU.Buffer as *mut core::ffi::c_void);
                NewPathU.Buffer = NewBuffer;
            }

            /* Check whether 'copy' renaming is allowed if required */
            if RtlDetermineDosPathNameType_U(lpExistingFileName) == RtlPathTypeUncAbsolute
                || (dwFlags & MOVEFILE_COPY_ALLOWED) != 0
            {
                Status = STATUS_INVALID_PARAMETER;
            } else {
                /* First, probe 2nd key to see whether it exists - if so, it will be appended there */
                Status = BasepMoveFileDelayed(&mut ExistingPathU, &mut NewPathU, 2, 0);
                if Status == STATUS_OBJECT_NAME_NOT_FOUND {
                    /* If doesn't exist, append to first key first, creating it if it doesn't exist */
                    Status = BasepMoveFileDelayed(&mut ExistingPathU, &mut NewPathU, 1, 1);
                    if Status == STATUS_INSUFFICIENT_RESOURCES {
                        /* If it failed because it's too big, then create 2nd key and put it there */
                        Status = BasepMoveFileDelayed(&mut ExistingPathU, &mut NewPathU, 2, 1);
                    }
                }
            }

            /* If we failed at some point, return the error */
            if !NT_SUCCESS(Status) {
                BaseSetLastNTError(Status);
                /* _SEH2_LEAVE */
                break 'leave;
            }

            Ret = 1;
            /* _SEH2_LEAVE */
            break 'leave;
        }

        /* At that point, we MUST have a source handle */
        /* ASSERT(SourceHandle != INVALID_HANDLE_VALUE); */

        /* Allocate renaming buffer and fill it */
        RenameInfo = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            NewPathU.Length as SIZE_T + core::mem::size_of::<FILE_RENAME_INFORMATION>() as SIZE_T,
        ) as *mut FILE_RENAME_INFORMATION;
        if RenameInfo.is_null() {
            BaseSetLastNTError(STATUS_NO_MEMORY);
            /* _SEH2_LEAVE */
            break 'leave;
        }

        RtlCopyMemory(
            (*RenameInfo).FileName.as_mut_ptr() as *mut core::ffi::c_void,
            NewPathU.Buffer as *const core::ffi::c_void,
            NewPathU.Length as SIZE_T,
        );
        (*RenameInfo).ReplaceIfExists = if ReplaceIfExists != 0 { 1 } else { 0 };
        (*RenameInfo).RootDirectory = core::ptr::null_mut();
        (*RenameInfo).FileNameLength = NewPathU.Length as ULONG;

        /* Attempt to rename the file */
        Status = NtSetInformationFile(
            SourceHandle,
            &mut IoStatusBlock,
            RenameInfo as *mut core::ffi::c_void,
            NewPathU.Length as ULONG + core::mem::size_of::<FILE_RENAME_INFORMATION>() as ULONG,
            if (dwFlags & MOVEFILE_CREATE_HARDLINK) != 0 {
                FileLinkInformation
            } else {
                FileRenameInformation
            },
        );
        RtlFreeHeap(RtlGetProcessHeap(), 0, RenameInfo as *mut core::ffi::c_void);
        if NT_SUCCESS(Status) {
            /* If it succeed, all fine, quit */
            Ret = 1;
            /* _SEH2_LEAVE */
            break 'leave;
        }
        /* If we failed for any other reason than not the same device, fail.
         * If we failed because of different devices, only allow renaming if user allowed copy.
         */
        if Status != STATUS_NOT_SAME_DEVICE || (dwFlags & MOVEFILE_COPY_ALLOWED) == 0 {
            /* ReactOS HACK! To be removed once all FSD have proper renaming support.
             * Just leave status to error and leave. */
            if Status == STATUS_NOT_IMPLEMENTED {
                // DPRINT removed
            } else {
                BaseSetLastNTError(Status);
                /* _SEH2_LEAVE */
                break 'leave;
            }
        }

        /* Close source file */
        NtClose(SourceHandle);
        SourceHandle = INVALID_HANDLE_VALUE;

        /* Issue the copy of the file */
        CopyContext.Flags = dwFlags;
        CopyContext.UserRoutine = lpProgressRoutine;
        CopyContext.UserData = lpData;
        NewHandle = INVALID_HANDLE_VALUE;
        ExistingHandle = INVALID_HANDLE_VALUE;

        Ret = BasepCopyFileExW(
            lpExistingFileName,
            lpNewFileName,
            // extern "system" == extern "stdcall" ABI here; bridge the nominal fn-ptr type.
            core::mem::transmute::<
                unsafe extern "system" fn(
                    LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER, LARGE_INTEGER,
                    DWORD, DWORD, HANDLE, HANDLE, LPVOID,
                ) -> DWORD,
                LPPROGRESS_ROUTINE,
            >(BasepMoveFileCopyProgress),
            &mut CopyContext as *mut COPY_PROGRESS_CONTEXT as *mut core::ffi::c_void,
            core::ptr::null_mut(),
            (if ReplaceIfExists != 0 { 0 } else { COPY_FILE_FAIL_IF_EXISTS })
                | COPY_FILE_OPEN_SOURCE_FOR_WRITE,
            0,
            &mut ExistingHandle,
            &mut NewHandle,
        );
        if Ret == 0 {
            /* If it failed, don't leak any handle */
            if ExistingHandle != INVALID_HANDLE_VALUE {
                CloseHandle(ExistingHandle);
                ExistingHandle = INVALID_HANDLE_VALUE;
            }
        } else if ExistingHandle != INVALID_HANDLE_VALUE {
            if NewHandle != INVALID_HANDLE_VALUE {
                /* If copying succeed, notify */
                Status = BasepNotifyTrackingService(
                    &mut ExistingHandle,
                    &ObjectAttributes,
                    NewHandle,
                    &NewPathU,
                );
                if !NT_SUCCESS(Status) {
                    /* Fail in case it had to succeed */
                    if (dwFlags & MOVEFILE_FAIL_IF_NOT_TRACKABLE) != 0 {
                        if NewHandle != INVALID_HANDLE_VALUE {
                            CloseHandle(NewHandle);
                        }
                        NewHandle = INVALID_HANDLE_VALUE;
                        DeleteFileW(lpNewFileName);
                        Ret = 0;
                        BaseSetLastNTError(Status);
                    }
                }
            }

            CloseHandle(ExistingHandle);
            ExistingHandle = INVALID_HANDLE_VALUE;
        }

        /* In case copy worked, close file */
        if NewHandle != INVALID_HANDLE_VALUE {
            CloseHandle(NewHandle);
            NewHandle = INVALID_HANDLE_VALUE;
        }

        /* If it succeed, delete source file */
        if Ret != 0 {
            if DeleteFileW(lpExistingFileName) == 0 {
                /* Reset file attributes if required */
                SetFileAttributesW(lpExistingFileName, FILE_ATTRIBUTE_NORMAL);
                DeleteFileW(lpExistingFileName);
            }
        }
    }
    /* _SEH2_FINALLY */
    {
        if SourceHandle != INVALID_HANDLE_VALUE {
            NtClose(SourceHandle);
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, ExistingPathU.Buffer as *mut core::ffi::c_void);
        RtlFreeHeap(RtlGetProcessHeap(), 0, NewPathU.Buffer as *mut core::ffi::c_void);
    }
    /* _SEH2_END */

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileWithProgressA(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
    lpProgressRoutine: LPPROGRESS_ROUTINE,
    lpData: LPVOID,
    dwFlags: DWORD,
) -> BOOL {

    let mut Ret: BOOL;
    let mut ExistingFileNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut NewFileNameW: _UNICODE_STRING = core::mem::zeroed();

    if Basep8BitStringToDynamicUnicodeString(&mut ExistingFileNameW, lpExistingFileName) == 0 {
        return 0;
    }

    if !lpNewFileName.is_null() {
        if Basep8BitStringToDynamicUnicodeString(&mut NewFileNameW, lpNewFileName) == 0 {
            RtlFreeUnicodeString(&mut ExistingFileNameW);
            return 0;
        }
    } else {
        NewFileNameW.Buffer = core::ptr::null_mut();
    }

    Ret = MoveFileWithProgressW(
        ExistingFileNameW.Buffer,
        NewFileNameW.Buffer,
        lpProgressRoutine,
        lpData,
        dwFlags,
    );

    RtlFreeUnicodeString(&mut ExistingFileNameW);
    RtlFreeUnicodeString(&mut NewFileNameW);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
) -> BOOL {
    return MoveFileWithProgressW(
        lpExistingFileName,
        lpNewFileName,
        None,
        core::ptr::null_mut(),
        MOVEFILE_COPY_ALLOWED,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileExW(
    lpExistingFileName: LPCWSTR,
    lpNewFileName: LPCWSTR,
    dwFlags: DWORD,
) -> BOOL {
    return MoveFileWithProgressW(
        lpExistingFileName,
        lpNewFileName,
        None,
        core::ptr::null_mut(),
        dwFlags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileA(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
) -> BOOL {
    return MoveFileWithProgressA(
        lpExistingFileName,
        lpNewFileName,
        None,
        core::ptr::null_mut(),
        MOVEFILE_COPY_ALLOWED,
    );
}

#[no_mangle]
pub unsafe extern "system" fn MoveFileExA(
    lpExistingFileName: LPCSTR,
    lpNewFileName: LPCSTR,
    dwFlags: DWORD,
) -> BOOL {
    return MoveFileWithProgressA(
        lpExistingFileName,
        lpNewFileName,
        None,
        core::ptr::null_mut(),
        dwFlags,
    );
}

#[no_mangle]
pub unsafe extern "system" fn ReplaceFileA(
    lpReplacedFileName: LPCSTR,
    lpReplacementFileName: LPCSTR,
    lpBackupFileName: LPCSTR,
    dwReplaceFlags: DWORD,
    lpExclude: LPVOID,
    lpReserved: LPVOID,
) -> BOOL {
    let mut Ret: BOOL;
    let mut ReplacedFileNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut ReplacementFileNameW: _UNICODE_STRING = core::mem::zeroed();
    let mut BackupFileNameW: _UNICODE_STRING = core::mem::zeroed();

    if lpReplacedFileName.is_null()
        || lpReplacementFileName.is_null()
        || !lpExclude.is_null()
        || !lpReserved.is_null()
        || (dwReplaceFlags & !(REPLACEFILE_WRITE_THROUGH | REPLACEFILE_IGNORE_MERGE_ERRORS)) != 0
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut ReplacedFileNameW, lpReplacedFileName) == 0 {
        return 0;
    }

    if Basep8BitStringToDynamicUnicodeString(&mut ReplacementFileNameW, lpReplacementFileName) == 0 {
        RtlFreeUnicodeString(&mut ReplacedFileNameW);
        return 0;
    }

    if !lpBackupFileName.is_null() {
        if Basep8BitStringToDynamicUnicodeString(&mut BackupFileNameW, lpBackupFileName) == 0 {
            RtlFreeUnicodeString(&mut ReplacementFileNameW);
            RtlFreeUnicodeString(&mut ReplacedFileNameW);
            return 0;
        }
    } else {
        BackupFileNameW.Buffer = core::ptr::null_mut();
    }

    Ret = ReplaceFileW(
        ReplacedFileNameW.Buffer,
        ReplacementFileNameW.Buffer,
        BackupFileNameW.Buffer,
        dwReplaceFlags,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    if !lpBackupFileName.is_null() {
        RtlFreeUnicodeString(&mut BackupFileNameW);
    }
    RtlFreeUnicodeString(&mut ReplacementFileNameW);
    RtlFreeUnicodeString(&mut ReplacedFileNameW);

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn ReplaceFileW(
    lpReplacedFileName: LPCWSTR,
    lpReplacementFileName: LPCWSTR,
    lpBackupFileName: LPCWSTR,
    dwReplaceFlags: DWORD,
    lpExclude: LPVOID,
    lpReserved: LPVOID,
) -> BOOL {
    let mut hReplaced: HANDLE = core::ptr::null_mut();
    let mut hReplacement: HANDLE = core::ptr::null_mut();
    let mut NtReplacedName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut NtReplacementName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut Error: DWORD = ERROR_SUCCESS;
    let mut Status: NTSTATUS;
    let mut Ret: BOOL = 0;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Buffer: PVOID = core::ptr::null_mut();

    if dwReplaceFlags != 0 {
        FIXME!("Ignoring flags %x\n", dwReplaceFlags);
    }

    if lpReplacedFileName.is_null() || lpReplacementFileName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    'cleanup: {
    if !lpBackupFileName.is_null() {
        if CopyFileW(lpReplacedFileName, lpBackupFileName, 0) == 0 {
            Error = GetLastError();
            break 'cleanup;
        }
    }

    if RtlDosPathNameToNtPathName_U(lpReplacedFileName, &mut NtReplacedName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        Error = ERROR_PATH_NOT_FOUND;
        break 'cleanup;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtReplacedName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut hReplaced,
        GENERIC_READ | GENERIC_WRITE | DELETE | SYNCHRONIZE | WRITE_DAC,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE,
    );

    if !NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_NOT_FOUND {
            Error = ERROR_FILE_NOT_FOUND;
        } else {
            Error = ERROR_UNABLE_TO_REMOVE_REPLACED;
        }
        break 'cleanup;
    }

    SetEndOfFile(hReplaced);

    if RtlDosPathNameToNtPathName_U(lpReplacementFileName, &mut NtReplacementName, core::ptr::null_mut(), core::ptr::null_mut()) == 0 {
        Error = ERROR_PATH_NOT_FOUND;
        break 'cleanup;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut NtReplacementName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );

    Status = NtOpenFile(
        &mut hReplacement,
        GENERIC_READ | DELETE | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        0,
        FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE | FILE_DELETE_ON_CLOSE,
    );

    if !NT_SUCCESS(Status) {
        Error = RtlNtStatusToDosError(Status);
        break 'cleanup;
    }

    Buffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, 0x10000);
    if Buffer.is_null() {
        Error = ERROR_NOT_ENOUGH_MEMORY;
        break 'cleanup;
    }

    while Status != STATUS_END_OF_FILE {
        Status = NtReadFile(
            hReplacement,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &mut IoStatusBlock,
            Buffer,
            0x10000,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(Status) {
            Status = NtWriteFile(
                hReplaced,
                core::ptr::null_mut(),
                None,
                core::ptr::null_mut(),
                &mut IoStatusBlock,
                Buffer,
                IoStatusBlock.Information as ULONG,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if !NT_SUCCESS(Status) {
                Error = RtlNtStatusToDosError(Status);
                break 'cleanup;
            }
        } else if Status != STATUS_END_OF_FILE {
            Error = RtlNtStatusToDosError(Status);
            break 'cleanup;
        }
    }

    Ret = 1;
    } // 'cleanup

    // Cleanup:
    if !hReplaced.is_null() {
        NtClose(hReplaced);
    }
    if !hReplacement.is_null() {
        NtClose(hReplacement);
    }
    if !Buffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
    }

    if !NtReplacementName.Buffer.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, NtReplacementName.Buffer as *mut c_void);
    }
    if !NtReplacedName.Buffer.is_null() {
        RtlFreeHeap(GetProcessHeap(), 0, NtReplacedName.Buffer as *mut c_void);
    }

    if Ret == 0 {
        TRACE!(b"ReplaceFileW failed (error=%lu)\n\0".as_ptr() as LPCSTR, Error);
        SetLastError(Error);
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn PrivMoveFileIdentityW(
    lpSource: LPCWSTR,
    lpDestination: LPCWSTR,
    dwFlags: DWORD,
) -> BOOL {
    let mut SourceAccess: ACCESS_MASK;
    let mut NtSource: _UNICODE_STRING = core::mem::zeroed();
    let mut NtDestination: _UNICODE_STRING = core::mem::zeroed();
    let mut RelativeSource: LPWSTR;
    let mut RelativeDestination: LPWSTR;
    let mut SourceHandle: HANDLE;
    let mut DestinationHandle: HANDLE;
    let mut ObjectAttributesSource: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectAttributesDestination: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut OldStatus: NTSTATUS = STATUS_SUCCESS;
    let mut DestAccess: ACCESS_MASK;
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut SourceInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut DestinationInformation: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut DispositionInfo: FILE_DISPOSITION_INFORMATION = core::mem::zeroed();

    // DPRINT("PrivMoveFileIdentityW(%S, %S, %x)\n", lpSource, lpDestination, dwFlags);

    SourceHandle = INVALID_HANDLE_VALUE;
    RtlInitEmptyUnicodeString(&mut NtSource, core::ptr::null_mut(), 0);
    RelativeSource = core::ptr::null_mut();
    DestinationHandle = INVALID_HANDLE_VALUE;
    RtlInitEmptyUnicodeString(&mut NtDestination, core::ptr::null_mut(), 0);
    RelativeDestination = core::ptr::null_mut();

    /* FILE_WRITE_DATA is required for later on notification */
    SourceAccess = FILE_READ_ATTRIBUTES | FILE_WRITE_DATA;
    if (dwFlags & PRIV_DELETE_ON_SUCCESS) != 0 {
        SourceAccess |= DELETE;
    }

    // _SEH2_TRY
    {
        /* We will loop twice:
         * First we attempt to open with FILE_WRITE_DATA for notification.
         * If it fails and we have flag for non-trackable files, we retry
         * without FILE_WRITE_DATA.
         * If that one fails, then, we quit for real.
         */
        loop {
            Status = BasepOpenFileForMove(
                lpSource,
                &mut NtSource,
                &mut RelativeSource,
                &mut SourceHandle,
                &mut ObjectAttributesSource,
                SourceAccess,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_OPEN_NO_RECALL,
            );
            if NT_SUCCESS(Status) {
                break;
            }

            /* If we already attempted the opening without FILE_WRITE_DATA
             * or if we cannot move on non-trackable files, fail.
             */
            if (SourceAccess & FILE_WRITE_DATA) == 0 || (dwFlags & PRIV_ALLOW_NON_TRACKABLE) == 0 {
                break; // _SEH2_LEAVE
            }

            if !RelativeSource.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSource as *mut _);
                RelativeSource = core::ptr::null_mut();
            }

            if SourceHandle != INVALID_HANDLE_VALUE {
                NtClose(SourceHandle);
                SourceHandle = INVALID_HANDLE_VALUE;
            }

            SourceAccess &= !FILE_WRITE_DATA;

            /* Remember fist failure in the path */
            if NT_SUCCESS(OldStatus) {
                OldStatus = Status;
            }
        }

        DestAccess = FILE_WRITE_ATTRIBUTES;
        /* If we could preserve FILE_WRITE_DATA for source, attempt to
         * get it for destination, still for notification purposes. */
        if (SourceAccess & FILE_WRITE_DATA) != 0 {
            DestAccess |= FILE_WRITE_DATA;
        }

        /* cf comment for first loop */
        loop {
            Status = BasepOpenFileForMove(
                lpDestination,
                &mut NtDestination,
                &mut RelativeDestination,
                &mut DestinationHandle,
                &mut ObjectAttributesDestination,
                DestAccess,
                FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                FILE_OPEN_NO_RECALL,
            );
            if NT_SUCCESS(Status) {
                break;
            }

            /* If we already attempted the opening without FILE_WRITE_DATA
             * or if we cannot move on non-trackable files, fail.
             */
            if (DestAccess & FILE_WRITE_DATA) == 0 || (dwFlags & PRIV_ALLOW_NON_TRACKABLE) == 0 {
                break; // _SEH2_LEAVE
            }

            if !RelativeDestination.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeDestination as *mut _);
                RelativeDestination = core::ptr::null_mut();
            }

            if DestinationHandle != INVALID_HANDLE_VALUE {
                NtClose(DestinationHandle);
                DestinationHandle = INVALID_HANDLE_VALUE;
            }

            DestAccess &= !FILE_WRITE_DATA;

            /* Remember fist failure in the path */
            if NT_SUCCESS(OldStatus) {
                OldStatus = Status;
            }
        }

        /* Get the creation time from source */
        Status = NtQueryInformationFile(
            SourceHandle,
            &mut IoStatusBlock,
            &mut SourceInformation as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
            FileBasicInformation,
        );
        if NT_SUCCESS(Status) {
            /* Then, prepare to set it for destination */
            core::ptr::write_bytes(
                &mut DestinationInformation as *mut _ as *mut core::ffi::c_void,
                0,
                core::mem::size_of::<FILE_BASIC_INFORMATION>(),
            );
            DestinationInformation.CreationTime.QuadPart = SourceInformation.CreationTime.QuadPart;

            /* And set it, that's all folks! */
            Status = NtSetInformationFile(
                DestinationHandle,
                &mut IoStatusBlock,
                &mut DestinationInformation as *mut _ as *mut core::ffi::c_void,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as u32,
                FileBasicInformation,
            );
        }

        if !NT_SUCCESS(Status) {
            if (dwFlags & PRIV_ALLOW_NON_TRACKABLE) == 0 {
                // _SEH2_LEAVE — but we're in a block, so we just skip to finally
                // Actually we need to handle the finally block. We'll use a flag.
                // For now, just continue to the finally section.
            }

            /* Remember the failure for later notification */
            if NT_SUCCESS(OldStatus) {
                OldStatus = Status;
            }
        }

        /* If we could open with FILE_WRITE_DATA both source and destination,
         * then, notify
         */
        if (DestAccess & FILE_WRITE_DATA) != 0 && (SourceAccess & FILE_WRITE_DATA) != 0 {
            Status = BasepNotifyTrackingService(
                &mut SourceHandle,
                &mut ObjectAttributesSource,
                DestinationHandle,
                &mut NtDestination,
            );
            if !NT_SUCCESS(Status) {
                if (dwFlags & PRIV_ALLOW_NON_TRACKABLE) != 0 {
                    if NT_SUCCESS(OldStatus) {
                        OldStatus = Status;
                    }

                    /* Reset status, we allow non trackable files */
                    Status = STATUS_SUCCESS;
                }
            }
        }
    }

    // _SEH2_FINALLY
    {
        if !RelativeSource.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSource as *mut _);
        }

        if !RelativeDestination.is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeDestination as *mut _);
        }
    }
    // _SEH2_END;

    /* If caller asked for source deletion, if everything succeed, proceed */
    if NT_SUCCESS(Status) && (dwFlags & PRIV_DELETE_ON_SUCCESS) != 0 {
        DispositionInfo.DeleteFileA = 1; // TRUE

        Status = NtSetInformationFile(
            SourceHandle,
            &mut IoStatusBlock,
            &mut DispositionInfo as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<FILE_DISPOSITION_INFORMATION>() as u32,
            FileDispositionInformation,
        );
    }

    /* Cleanup/close portion */
    if DestinationHandle != INVALID_HANDLE_VALUE {
        NtClose(DestinationHandle);
    }

    if SourceHandle != INVALID_HANDLE_VALUE {
        NtClose(SourceHandle);
    }

    /* Set last error if any, and quit */
    if NT_SUCCESS(Status) {
        if !NT_SUCCESS(OldStatus) {
            BaseSetLastNTError(OldStatus);
        }
    } else {
        BaseSetLastNTError(Status);
    }

    if NT_SUCCESS(Status) { 1 } else { 0 }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/file/move.c
//  * PURPOSE:         Directory functions
//  * PROGRAMMER:      Ariadne ( ariadne@xs4all.nl)
//  *                  Gerhard W. Gruber (sparhawk_at_gmx.at)
//  *                  Dmitry Philippov (shedon@mail.ru)
//  *                  Pierre Schweitzer (pierre@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <k32.h>
// #include <malloc.h>
//
// #define NDEBUG
// #include <debug.h>
// DEBUG_CHANNEL(kernel32file);
//
// /* GLOBALS *****************************************************************/
//
// /* DEFINES *****************************************************************/
// typedef struct _COPY_PROGRESS_CONTEXT
// {
//     ULONG Flags;
//     LPPROGRESS_ROUTINE UserRoutine;
//     LPVOID UserData;
// } COPY_PROGRESS_CONTEXT, *PCOPY_PROGRESS_CONTEXT;
//
// /* FUNCTIONS ****************************************************************/
//
// /**
//  * @brief
//  * Adds an entry in the "PendingFileRenameOperations" registry value, that is
//  * parsed at boot-time by SMSS.EXE to check whether there are some files to be
//  * renamed/moved or deleted.
//  *
//  * @param[in]   ExistingPath
//  * Full NT path to the file to rename/move or delete.
//  *
//  * @param[in]   NewPath
//  * Full NT path to the moved/renamed file; or an empty string if the file is
//  * to be deleted.
//  *
//  * @param[in]   KeyId
//  * Selects an alternate "PendingFileRenameOperationsXXX" registry value.
//  *
//  * @param[in]   CreateIfNotFound
//  * TRUE if the file needs to be created if it does not already exist,
//  * FALSE if not.
//  *
//  * @remark
//  * If both ExistingPath and NewPath strings are non-empty, the file is moved,
//  * otherwise it is deleted.
//  *
//  * @note
//  * The registry value is a list of NULL-terminated strings, which is itself
//  * terminated with an empty NULL-terminated string (single 0-byte).
//  * When a new entry is added, if NewPath is empty, then the second entry is
//  * simply a single NULL. Otherwise the second file path goes there.
//  * Each path is in NT format (e.g. path prepended with \??\) and the second
//  * file path gets also a '!' as the first character if MOVEFILE_REPLACE_EXISTING
//  * is specified.
//  *
//  * Examples:
//  *
//  * \??\D:\test\file1[0]
//  * !\??\D:\test\file1_renamed[0]
//  * \??\D:\test\delete[0]
//  * [0]                        <- file is to be deleted, second string empty
//  * \??\D:\test\file2[0]
//  * !\??\D:\test\file2_renamed[0]
//  * [0]                        <- indicates end of strings
//  *
//  * or:
//  *
//  * \??\D:\test\file1[0]
//  * !\??\D:\test\file1_renamed[0]
//  * \??\D:\test\delete[0]
//  * [0]                        <- file is to be deleted, second string empty
//  * [0]                        <- indicates end of strings
//  *
//  * @implemented
//  **/
// NTSTATUS
// WINAPI
// BasepMoveFileDelayed(
//      PUNICODE_STRING ExistingPath,
//      PUNICODE_STRING NewPath,
//      INT KeyId,
//      BOOL CreateIfNotFound)
// {
// #define STRING_LENGTH 0x400
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     PVOID Buffer, BufferBegin;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PWSTR PendingOperations, BufferWrite;
//     ULONG DataSize, BufferLength, StringLength = STRING_LENGTH;
//     UNICODE_STRING SessionManagerString, PendingOperationsString;
//     /* +6 because a INT shouldn't take more than 6 chars. Especially given the call path */
//     WCHAR PendingOperationsBuffer[sizeof(L"PendingFileRenameOperations") / sizeof(WCHAR) + 6];
//
//     RtlInitUnicodeString(&SessionManagerString,
//                          L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Session Manager");
//
//     /* Select appropriate key for adding our file */
//     if (KeyId == 1)
//     {
//         PendingOperations = L"PendingFileRenameOperations";
//     }
//     else
//     {
//         RtlStringCbPrintfW(PendingOperationsBuffer,
//                            sizeof(PendingOperationsBuffer),
//                            L"PendingFileRenameOperations%d", KeyId);
//         PendingOperations = PendingOperationsBuffer;
//     }
//     RtlInitUnicodeString(&PendingOperationsString, PendingOperations);
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &SessionManagerString,
//                                OBJ_OPENIF | OBJ_CASE_INSENSITIVE,
//                                NULL, NULL);
//
//     /* Open parent key */
//     Status = NtCreateKey(&KeyHandle,
//                          GENERIC_READ | GENERIC_WRITE,
//                          &ObjectAttributes, 0, NULL,
//                          REG_OPTION_NON_VOLATILE, NULL);
//     if (Status == STATUS_ACCESS_DENIED)
//     {
//         Status = NtCreateKey(&KeyHandle,
//                              GENERIC_READ | GENERIC_WRITE,
//                              &ObjectAttributes, 0, NULL,
//                              REG_OPTION_BACKUP_RESTORE, NULL);
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Reserve enough to read previous string + to append ours with required null chars */
//     BufferLength = NewPath->Length + ExistingPath->Length + STRING_LENGTH + 3 * sizeof(UNICODE_NULL);
//
//     while (TRUE)
//     {
//         /* Allocate output buffer */
//         Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, BufferLength);
//         if (Buffer == NULL)
//         {
//             NtClose(KeyHandle);
//             return STATUS_NO_MEMORY;
//         }
//
//         Status = NtQueryValueKey(KeyHandle,
//                                  &PendingOperationsString,
//                                  KeyValuePartialInformation,
//                                  Buffer, StringLength, &DataSize);
//         if (Status != STATUS_BUFFER_OVERFLOW)
//         {
//             break;
//         }
//
//         /* If buffer was too small, reallocate one which is big enough */
//         StringLength = DataSize;
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         BufferLength = ExistingPath->Length + StringLength + NewPath->Length + 3 * sizeof(UNICODE_NULL);
//         /* Check we didn't overflow */
//         if (BufferLength < StringLength)
//         {
//             NtClose(KeyHandle);
//             return STATUS_BUFFER_TOO_SMALL;
//         }
//     }
//
//     /* Check if it existed; if not, create only IF asked to */
//     if (!NT_SUCCESS(Status) && (Status != STATUS_OBJECT_NAME_NOT_FOUND || !CreateIfNotFound))
//     {
//         NtClose(KeyHandle);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//         return Status;
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* We didn't find any: we create, so use complete buffer */
//         BufferBegin = Buffer;
//         BufferWrite = Buffer;
//     }
//     else
//     {
//         PKEY_VALUE_PARTIAL_INFORMATION PartialInfo = (PKEY_VALUE_PARTIAL_INFORMATION)Buffer;
//
//         /* Get data, the buffer beginning, and where data should be appended
//          * (minus NULL char: this is REG_MULTI_SZ, it already includes double
//          * termination, but we keep only one). */
//         BufferBegin = PartialInfo->Data;
//         BufferWrite = (PWSTR)((ULONG_PTR)PartialInfo->Data + PartialInfo->DataLength - sizeof(UNICODE_NULL));
//     }
//
//     /* First copy existing */
//     RtlCopyMemory(BufferWrite, ExistingPath->Buffer, ExistingPath->Length);
//     BufferWrite += ExistingPath->Length / sizeof(WCHAR);
//     /* And append null char */
//     *BufferWrite = UNICODE_NULL;
//     ++BufferWrite;
//     /* Append destination */
//     RtlCopyMemory(BufferWrite, NewPath->Buffer, NewPath->Length);
//     BufferWrite += NewPath->Length / sizeof(WCHAR);
//     /* And append two null char (end of string) */
//     *BufferWrite = UNICODE_NULL;
//     ++BufferWrite;
//     *BufferWrite = UNICODE_NULL;
//
//     /* Set new value */
//     Status = NtSetValueKey(KeyHandle,
//                            &PendingOperationsString,
//                            0, REG_MULTI_SZ, BufferBegin,
//                            (ULONG_PTR)BufferWrite - (ULONG_PTR)BufferBegin + sizeof(WCHAR));
//
//     NtClose(KeyHandle);
//     RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// BasepGetComputerNameFromNtPath(IN PUNICODE_STRING NewPath,
//                                IN HANDLE NewHandle,
//                                OUT PWSTR ComputerName,
//                                IN OUT PULONG ComputerNameLength)
// {
//     BOOL Query = FALSE;
//     WCHAR Letter;
//     PWSTR AbsolutePath, EndOfName;
//     USHORT AbsolutePathLength, NameLength;
//     WCHAR TargetDevice[MAX_PATH + 1];
//     WCHAR DeviceName[] = {'A', ':', '\0'}; /* Init to something, will be set later */
//     UNICODE_STRING UncString = RTL_CONSTANT_STRING(L"\\??\\UNC\\");
//     UNICODE_STRING GlobalString = RTL_CONSTANT_STRING(L"\\??\\");
//
//     DPRINT("BasepGetComputerNameFromNtPath(%wZ, %p, %p, %lu)\n",
//            NewPath, NewHandle, ComputerName, ComputerNameLength);
//
//     /* If it's an UNC path */
//     if (RtlPrefixUnicodeString(&UncString, NewPath, TRUE))
//     {
//         /* Check for broken caller */
//         if (NewPath->Length <= UncString.Length)
//         {
//             return ERROR_BAD_PATHNAME;
//         }
//
//         /* Skip UNC prefix */
//         AbsolutePath = &NewPath->Buffer[UncString.Length / sizeof(WCHAR)];
//         AbsolutePathLength = NewPath->Length - UncString.Length;
//
//         /* And query DFS */
//         Query = TRUE;
//     }
//     /* Otherwise, we have to be in global (NT path!), with drive letter */
//     else if (RtlPrefixUnicodeString(&GlobalString, NewPath, TRUE) && NewPath->Buffer[5] == ':')
//     {
//         /* Path is like that: \??\C:\Complete Path\To File.ext */
//         /* Get the letter and upcase it if required */
//         Letter = NewPath->Buffer[4];
//         if (Letter >= 'a' && Letter <= 'z')
//         {
//             Letter -= ('a' - 'A');
//         }
//         DeviceName[0] = Letter;
//
//         /* Query the associated DOS device */
//         if (!QueryDosDeviceW(DeviceName, TargetDevice, ARRAYSIZE(TargetDevice)))
//         {
//             return GetLastError();
//         }
//
//         /* If that's a network share */
//         if (TargetDevice == wcsstr(TargetDevice, L"\\Device\\LanmanRedirector\\;"))
//         {
//             /* Path is like that: \Device\LanmanRedirector\;C:0000000000000000\Complete Path\To File.ext */
//             /* Check we have the correct drive letter */
//             if (TargetDevice[26] == DeviceName[0] &&
//                 TargetDevice[27] == ':')
//             {
//                 /* Check for the path begin, computer name is before */
//                 PWSTR Path = wcschr(&TargetDevice[28], L'\\');
//                 if (Path == NULL)
//                 {
//                     return ERROR_BAD_PATHNAME;
//                 }
//
//                 AbsolutePath = Path + 1;
//                 AbsolutePathLength = sizeof(WCHAR) * (ARRAYSIZE(TargetDevice) - (AbsolutePath - TargetDevice));
//             }
//             else
//             {
//                 return ERROR_BAD_PATHNAME;
//             }
//         }
//         /* If it's a local device */
//         else if (TargetDevice == wcsstr(TargetDevice, L"\\Device\\Harddisk")
//                  || TargetDevice == wcsstr(TargetDevice, L"\\Device\\CdRom")
//                  || TargetDevice == wcsstr(TargetDevice, L"\\Device\\Floppy"))
//         {
//             /* Just query the computer name */
//             if (!GetComputerNameW(ComputerName, ComputerNameLength))
//             {
//                 return GetLastError();
//             }
//
//             return ERROR_SUCCESS;
//         }
//         /* If it's a DFS share */
//         else if (TargetDevice == wcsstr(TargetDevice, L"\\Device\\WinDfs\\"))
//         {
//             /* Obviously, query DFS */
//             Query = TRUE;
//         }
//         else
//         {
//             return ERROR_BAD_PATHNAME;
//         }
//     }
//     else
//     {
//         return ERROR_BAD_PATHNAME;
//     }
//
//     /* Query DFS, currently not implemented - shouldn't be missing in ReactOS yet ;-) */
//     if (Query)
//     {
//         UNIMPLEMENTED_DBGBREAK("Querying DFS not implemented!\n");
//         AbsolutePath = NULL;
//         AbsolutePathLength = 0;
//     }
//
//     /* Now, properly extract the computer name from the full path */
//     EndOfName = AbsolutePath;
//     if (AbsolutePathLength)
//     {
//         for (NameLength = 0; NameLength < AbsolutePathLength; NameLength += sizeof(WCHAR))
//         {
//             /* Look for the next \, it will be the end of computer name */
//             if (EndOfName[0] == L'\\')
//             {
//                 break;
//             }
//             /* Computer name cannot contain ., if we get to that point, something went wrong... */
//             else if (EndOfName[0] == L'.')
//             {
//                 return ERROR_BAD_PATHNAME;
//             }
//
//             ++EndOfName;
//         }
//     }
//
//     NameLength = EndOfName - AbsolutePath;
//     /* Check we didn't overflow and that our computer name isn't ill-formed */
//     if (NameLength >= AbsolutePathLength || NameLength >= MAX_COMPUTERNAME_LENGTH * sizeof(WCHAR))
//     {
//         return ERROR_BAD_PATHNAME;
//     }
//
//     /* Check we can fit */
//     if (NameLength + sizeof(UNICODE_NULL) > *ComputerNameLength * sizeof(WCHAR))
//     {
//         return ERROR_BUFFER_OVERFLOW;
//     }
//
//     /* Write, zero and done! */
//     RtlCopyMemory(ComputerName, AbsolutePath, NameLength);
//     *ComputerNameLength = NameLength / sizeof(WCHAR);
//     ComputerName[NameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//     return ERROR_SUCCESS;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// BasepNotifyTrackingService(IN OUT PHANDLE ExistingHandle,
//                            IN POBJECT_ATTRIBUTES ObjectAttributes,
//                            IN HANDLE NewHandle,
//                            IN PUNICODE_STRING NewPath)
// {
//     NTSTATUS Status;
//     ULONG ComputerNameLength, FileAttributes;
//     WCHAR ComputerName[MAX_COMPUTERNAME_LENGTH + 1];
//     OEM_STRING ComputerNameStringA;
//     CHAR ComputerNameStringBuffer[MAX_PATH + 1];
//     UNICODE_STRING ComputerNameStringW;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION FileBasicInfo;
//     HANDLE hFullWrite;
//     struct
//     {
//         FILE_TRACKING_INFORMATION;
//         CHAR Buffer[(MAX_COMPUTERNAME_LENGTH + 1) * sizeof(WCHAR)];
//     } FileTrackingInfo;
//
//     DPRINT("BasepNotifyTrackingService(%p, %p, %p, %wZ)\n",
//            *ExistingHandle, ObjectAttributes, NewHandle, NewPath);
//
//     Status = STATUS_SUCCESS;
//     ComputerNameLength = ARRAYSIZE(ComputerName);
//
//     /* Attempt to get computer name of target handle */
//     if (BasepGetComputerNameFromNtPath(NewPath, NewHandle, ComputerName, &ComputerNameLength))
//     {
//         /* If we failed to get it, we will just notify with the handle */
//         FileTrackingInfo.ObjectInformationLength = 0;
//     }
//     else
//     {
//         /* Convert the retrieved computer name to ANSI and attach it to the notification */
//         RtlInitEmptyAnsiString(&ComputerNameStringA,
//                                ComputerNameStringBuffer,
//                                sizeof(ComputerNameStringBuffer));
//
//         RtlInitUnicodeString(&ComputerNameStringW, ComputerName);
//         Status = RtlUnicodeStringToOemString(&ComputerNameStringA, &ComputerNameStringW, FALSE);
//         if (!NT_SUCCESS(Status))
//         {
//             return Status;
//         }
//
//         RtlCopyMemory(FileTrackingInfo.ObjectInformation, ComputerNameStringA.Buffer, ComputerNameStringA.Length);
//         FileTrackingInfo.ObjectInformation[ComputerNameStringA.Length] = ANSI_NULL;
//         FileTrackingInfo.ObjectInformationLength = ComputerNameStringA.Length + 1;
//     }
//
//     /* Attach the handle we moved */
//     FileTrackingInfo.DestinationFile = NewHandle;
//
//     /* Final, notify */
//     Status = NtSetInformationFile(*ExistingHandle,
//                                   &IoStatusBlock,
//                                   &FileTrackingInfo,
//                                   sizeof(FileTrackingInfo),
//                                   FileTrackingInformation);
//     if (Status != STATUS_ACCESS_DENIED)
//     {
//         return Status;
//     }
//
//     /* If we get here, we got access denied error, this comes from a
//      * read-only flag. So, close the file, in order to reopen it with enough
//      * rights to remove said flag and reattempt notification.
//      */
//     CloseHandle(*ExistingHandle);
//
//     /* Reopen it, to be able to change the destination file attributes */
//     Status = NtOpenFile(ExistingHandle,
//                         SYNCHRONIZE | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES,
//                         ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (!NT_SUCCESS(Status))
//     {
//         *ExistingHandle = INVALID_HANDLE_VALUE;
//         return Status;
//     }
//
//     /* Get the file attributes */
//     Status = NtQueryInformationFile(*ExistingHandle,
//                                     &IoStatusBlock,
//                                     &FileBasicInfo,
//                                     sizeof(FileBasicInfo),
//                                     FileBasicInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Get rid of the read only flag */
//     FileAttributes = FileBasicInfo.FileAttributes & ~FILE_ATTRIBUTE_READONLY;
//     RtlZeroMemory(&FileBasicInfo, sizeof(FileBasicInfo));
//     FileBasicInfo.FileAttributes = FileAttributes;
//
//     /* Attempt... */
//     Status = NtSetInformationFile(*ExistingHandle,
//                                   &IoStatusBlock,
//                                   &FileBasicInfo,
//                                   sizeof(FileBasicInfo),
//                                   FileBasicInformation);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     /* Now, reopen with maximum accesses to notify */
//     Status = NtOpenFile(&hFullWrite,
//                         GENERIC_WRITE | SYNCHRONIZE,
//                         ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT);
//     if (NT_SUCCESS(Status))
//     {
//         NtClose(*ExistingHandle);
//         *ExistingHandle = hFullWrite;
//
//         /* Full success, notify! */
//         Status = NtSetInformationFile(*ExistingHandle,
//                                       &IoStatusBlock,
//                                       &FileTrackingInfo,
//                                       sizeof(FileTrackingInfo),
//                                       FileTrackingInformation);
//     }
//
//     /* If opening with full access failed or if notify failed, restore read-only */
//     if (!NT_SUCCESS(Status))
//     {
//         FileBasicInfo.FileAttributes |= FILE_ATTRIBUTE_READONLY;
//
//         Status = NtSetInformationFile(*ExistingHandle,
//                                       &IoStatusBlock,
//                                       &FileBasicInfo,
//                                       sizeof(FileBasicInfo),
//                                       FileBasicInformation);
//     }
//
//     /* We're done */
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// NTSTATUS
// WINAPI
// BasepOpenFileForMove(IN LPCWSTR File,
//                      OUT PUNICODE_STRING RelativeNtName,
//                      OUT LPWSTR * NtName,
//                      OUT PHANDLE FileHandle,
//                      OUT POBJECT_ATTRIBUTES ObjectAttributes,
//                      IN ACCESS_MASK DesiredAccess,
//                      IN ULONG ShareAccess,
//                      IN ULONG OpenOptions)
// {
//     RTL_RELATIVE_NAME_U RelativeName;
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_ATTRIBUTE_TAG_INFORMATION TagInfo;
//     ULONG IntShareAccess;
//     BOOLEAN HasRelative = FALSE;
//
//     _SEH2_TRY
//     {
//         /* Zero output */
//         RtlInitEmptyUnicodeString(RelativeNtName, NULL, 0);
//         *NtName = NULL;
//
//         if (!RtlDosPathNameToRelativeNtPathName_U(File, RelativeNtName, NULL, &RelativeName))
//         {
//             Status = STATUS_OBJECT_PATH_NOT_FOUND;
//             _SEH2_LEAVE;
//         }
//
//         HasRelative = TRUE;
//         *NtName = RelativeNtName->Buffer;
//
//         if (RelativeName.RelativeName.Length)
//         {
//             RelativeNtName->Length = RelativeName.RelativeName.Length;
//             RelativeNtName->MaximumLength = RelativeName.RelativeName.MaximumLength;
//             RelativeNtName->Buffer = RelativeName.RelativeName.Buffer;
//         }
//         else
//         {
//             RelativeName.ContainingDirectory = NULL;
//         }
//
//         InitializeObjectAttributes(ObjectAttributes,
//                                    RelativeNtName,
//                                    OBJ_CASE_INSENSITIVE,
//                                    RelativeName.ContainingDirectory,
//                                    NULL);
//         /* Force certain flags here, given ops we'll do */
//         IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_WRITE;
//         OpenOptions |= FILE_OPEN_FOR_BACKUP_INTENT | FILE_SYNCHRONOUS_IO_NONALERT;
//
//         /* We'll try to read reparse tag */
//         Status = NtOpenFile(FileHandle,
//                             DesiredAccess | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
//                             ObjectAttributes,
//                             &IoStatusBlock,
//                             IntShareAccess,
//                             OpenOptions | FILE_OPEN_REPARSE_POINT);
//         if (NT_SUCCESS(Status))
//         {
//             /* Attempt the read */
//             Status = NtQueryInformationFile(*FileHandle,
//                                             &IoStatusBlock,
//                                             &TagInfo,
//                                             sizeof(TagInfo),
//                                             FileAttributeTagInformation);
//
//             /* Return if failure with a status that wouldn't mean the FSD cannot support reparse points */
//             if (!NT_SUCCESS(Status) &&
//                 (Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_PARAMETER))
//             {
//                 _SEH2_LEAVE;
//             }
//
//             if (NT_SUCCESS(Status))
//             {
//                 /* This cannot happen on mount points */
//                 if (TagInfo.FileAttributes & FILE_ATTRIBUTE_DEVICE ||
//                     TagInfo.ReparseTag == IO_REPARSE_TAG_MOUNT_POINT)
//                 {
//                     _SEH2_LEAVE;
//                 }
//             }
//
//             NtClose(*FileHandle);
//             *FileHandle = INVALID_HANDLE_VALUE;
//
//             IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_DELETE;
//         }
//         else if (Status == STATUS_INVALID_PARAMETER)
//         {
//             IntShareAccess = ShareAccess | FILE_SHARE_READ | FILE_SHARE_WRITE;
//         }
//         else
//         {
//             _SEH2_LEAVE;
//         }
//
//         /* Reattempt to open normally, following reparse point if needed */
//         Status = NtOpenFile(FileHandle,
//                             DesiredAccess | SYNCHRONIZE,
//                             ObjectAttributes,
//                             &IoStatusBlock,
//                             IntShareAccess,
//                             OpenOptions);
//     }
//     _SEH2_FINALLY
//     {
//         if (HasRelative)
//         {
//             RtlReleaseRelativeName(&RelativeName);
//         }
//     }
//     _SEH2_END;
//
//     return Status;
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// BasepMoveFileCopyProgress(IN LARGE_INTEGER TotalFileSize,
//                           IN LARGE_INTEGER TotalBytesTransferred,
//                           IN LARGE_INTEGER StreamSize,
//                           IN LARGE_INTEGER StreamBytesTransferred,
//                           IN DWORD dwStreamNumber,
//                           IN DWORD dwCallbackReason,
//                           IN HANDLE hSourceFile,
//                           IN HANDLE hDestinationFile,
//                           IN LPVOID lpData OPTIONAL)
// {
//     DWORD Ret = 0;
//     PCOPY_PROGRESS_CONTEXT Context = (PCOPY_PROGRESS_CONTEXT)lpData;
//
//     if (Context->Flags & MOVEFILE_WRITE_THROUGH)
//     {
//         if (!dwCallbackReason)
//         {
//             if (StreamBytesTransferred.QuadPart == StreamSize.QuadPart)
//             {
//                 FlushFileBuffers(hDestinationFile);
//             }
//         }
//     }
//
//     if (Context->UserRoutine)
//     {
//         Ret = Context->UserRoutine(TotalFileSize,
//                                    TotalBytesTransferred,
//                                    StreamSize,
//                                    StreamBytesTransferred,
//                                    dwStreamNumber,
//                                    dwCallbackReason,
//                                    hSourceFile,
//                                    hDestinationFile,
//                                    Context->UserData);
//     }
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileWithProgressW(IN LPCWSTR lpExistingFileName,
//                       IN LPCWSTR lpNewFileName,
//                       IN LPPROGRESS_ROUTINE lpProgressRoutine,
//                       IN LPVOID lpData,
//                       IN DWORD dwFlags)
// {
//     NTSTATUS Status;
//     PWSTR NewBuffer;
//     IO_STATUS_BLOCK IoStatusBlock;
//     COPY_PROGRESS_CONTEXT CopyContext;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PFILE_RENAME_INFORMATION RenameInfo;
//     UNICODE_STRING NewPathU, ExistingPathU;
//     FILE_ATTRIBUTE_TAG_INFORMATION TagInfo;
//     HANDLE SourceHandle = INVALID_HANDLE_VALUE, NewHandle, ExistingHandle;
//     BOOL Ret = FALSE, ReplaceIfExists, DelayUntilReboot, AttemptReopenWithoutReparse;
//
//     DPRINT("MoveFileWithProgressW(%S, %S, %p, %p, %x)\n",
//            lpExistingFileName, lpNewFileName, lpProgressRoutine, lpData, dwFlags);
//
//     NewPathU.Buffer = NULL;
//     ExistingPathU.Buffer = NULL;
//
//     _SEH2_TRY
//     {
//         /* Don't allow renaming to a disk */
//         if (lpNewFileName && RtlIsDosDeviceName_U(lpNewFileName))
//         {
//             BaseSetLastNTError(STATUS_OBJECT_NAME_COLLISION);
//             _SEH2_LEAVE;
//         }
//
//         ReplaceIfExists = !!(dwFlags & MOVEFILE_REPLACE_EXISTING);
//
//         /* Get file path */
//         if (!RtlDosPathNameToNtPathName_U(lpExistingFileName, &ExistingPathU, NULL, NULL))
//         {
//             BaseSetLastNTError(STATUS_OBJECT_PATH_NOT_FOUND);
//             _SEH2_LEAVE;
//         }
//
//         /* Sanitize input */
//         DelayUntilReboot = !!(dwFlags & MOVEFILE_DELAY_UNTIL_REBOOT);
//         if (DelayUntilReboot && (dwFlags & MOVEFILE_CREATE_HARDLINK))
//         {
//             BaseSetLastNTError(STATUS_INVALID_PARAMETER);
//             _SEH2_LEAVE;
//         }
//
//         /* Unless we manage a proper opening, we'll attempt to reopen without reparse support */
//         AttemptReopenWithoutReparse = TRUE;
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &ExistingPathU,
//                                    OBJ_CASE_INSENSITIVE,
//                                    NULL,
//                                    NULL);
//         /* Attempt to open source file */
//         Status = NtOpenFile(&SourceHandle,
//                             FILE_READ_ATTRIBUTES | DELETE | SYNCHRONIZE,
//                             &ObjectAttributes,
//                             &IoStatusBlock,
//                             FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                             FILE_OPEN_FOR_BACKUP_INTENT | ((dwFlags & MOVEFILE_WRITE_THROUGH) ? FILE_WRITE_THROUGH : 0));
//         if (!NT_SUCCESS(Status))
//         {
//             /* If we failed and the file doesn't exist, don't attempt to reopen without reparse */
//             if (DelayUntilReboot &&
//                 (Status == STATUS_SHARING_VIOLATION || Status == STATUS_OBJECT_NAME_NOT_FOUND || Status == STATUS_OBJECT_PATH_NOT_FOUND))
//             {
//                 /* Here we don't fail completely, as we postpone the operation to reboot.
//                  * File might exist afterwards, and we don't need a handle here. */
//                 SourceHandle = INVALID_HANDLE_VALUE;
//                 AttemptReopenWithoutReparse = FALSE;
//             }
//             /* If we failed for any reason than unsupported reparse, fail completely */
//             else if (Status != STATUS_INVALID_PARAMETER)
//             {
//                 BaseSetLastNTError(Status);
//                 _SEH2_LEAVE;
//             }
//         }
//         else
//         {
//             /* We managed to open, so query information */
//             Status = NtQueryInformationFile(SourceHandle,
//                                             &IoStatusBlock,
//                                             &TagInfo,
//                                             sizeof(TagInfo),
//                                             FileAttributeTagInformation);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Do not tolerate any other error than something related to not supported operation */
//                 if (Status != STATUS_NOT_IMPLEMENTED && Status != STATUS_INVALID_PARAMETER)
//                 {
//                     BaseSetLastNTError(Status);
//                     _SEH2_LEAVE;
//                 }
//
//                 /* Not a reparse point, no need to reopen, it's fine */
//                 AttemptReopenWithoutReparse = FALSE;
//             }
//             /* Validate the reparse point (do we support it?) */
//             else if ((TagInfo.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT) &&
//                      (TagInfo.ReparseTag != IO_REPARSE_TAG_MOUNT_POINT))
//             {
//                 NtClose(SourceHandle);
//                 SourceHandle = INVALID_HANDLE_VALUE;
//             }
//             else
//             {
//                 /* Mount point, let's rename it */
//                 AttemptReopenWithoutReparse = FALSE;
//             }
//         }
//
//         /* Simply reopen if required */
//         if (AttemptReopenWithoutReparse)
//         {
//             Status = NtOpenFile(&SourceHandle,
//                                 DELETE | SYNCHRONIZE,
//                                 &ObjectAttributes,
//                                 &IoStatusBlock,
//                                 FILE_SHARE_READ | FILE_SHARE_WRITE,
//                                 ((dwFlags & MOVEFILE_WRITE_THROUGH) ? FILE_WRITE_THROUGH : 0));
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 _SEH2_LEAVE;
//             }
//         }
//
//         /* Nullify string if we're to use it */
//         if (DelayUntilReboot && !lpNewFileName)
//         {
//             RtlInitUnicodeString(&NewPathU, NULL);
//         }
//         /* Check whether path exists */
//         else if (!RtlDosPathNameToNtPathName_U(lpNewFileName, &NewPathU, NULL, NULL))
//         {
//             BaseSetLastNTError(STATUS_OBJECT_PATH_NOT_FOUND);
//             _SEH2_LEAVE;
//         }
//
//         /* Handle postponed renaming */
//         if (DelayUntilReboot)
//         {
//             /* If new file exists and we're allowed to replace, then mark the path with ! */
//             if (ReplaceIfExists && NewPathU.Length)
//             {
//                 NewBuffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, NewPathU.Length + sizeof(WCHAR));
//                 if (NewBuffer == NULL)
//                 {
//                     BaseSetLastNTError(STATUS_NO_MEMORY);
//                     _SEH2_LEAVE;
//                 }
//
//                 NewBuffer[0] = L'!';
//                 RtlCopyMemory(&NewBuffer[1], NewPathU.Buffer, NewPathU.Length);
//                 NewPathU.Length += sizeof(WCHAR);
//                 NewPathU.MaximumLength += sizeof(WCHAR);
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, NewPathU.Buffer);
//                 NewPathU.Buffer = NewBuffer;
//             }
//
//             /* Check whether 'copy' renaming is allowed if required */
//             if ((RtlDetermineDosPathNameType_U(lpExistingFileName) == RtlPathTypeUncAbsolute) ||
//                 (dwFlags & MOVEFILE_COPY_ALLOWED))
//             {
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//             else
//             {
//                 /* First, probe 2nd key to see whether it exists - if so, it will be appended there */
//                 Status = BasepMoveFileDelayed(&ExistingPathU, &NewPathU, 2, FALSE);
//                 if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//                 {
//                     /* If doesn't exist, append to first key first, creating it if it doesn't exist */
//                     Status = BasepMoveFileDelayed(&ExistingPathU, &NewPathU, 1, TRUE);
//                     if (Status == STATUS_INSUFFICIENT_RESOURCES)
//                     {
//                         /* If it failed because it's too big, then create 2nd key and put it there */
//                         Status = BasepMoveFileDelayed(&ExistingPathU, &NewPathU, 2, TRUE);
//                     }
//                 }
//             }
//
//             /* If we failed at some point, return the error */
//             if (!NT_SUCCESS(Status))
//             {
//                 BaseSetLastNTError(Status);
//                 _SEH2_LEAVE;
//             }
//
//             Ret = TRUE;
//             _SEH2_LEAVE;
//         }
//
//         /* At that point, we MUST have a source handle */
//         ASSERT(SourceHandle != INVALID_HANDLE_VALUE);
//
//         /* Allocate renaming buffer and fill it */
//         RenameInfo = RtlAllocateHeap(RtlGetProcessHeap(), 0,
//                                      NewPathU.Length + sizeof(FILE_RENAME_INFORMATION));
//         if (RenameInfo == NULL)
//         {
//             BaseSetLastNTError(STATUS_NO_MEMORY);
//             _SEH2_LEAVE;
//         }
//
//         RtlCopyMemory(&RenameInfo->FileName, NewPathU.Buffer, NewPathU.Length);
//         RenameInfo->ReplaceIfExists = ReplaceIfExists;
//         RenameInfo->RootDirectory = NULL;
//         RenameInfo->FileNameLength = NewPathU.Length;
//
//         /* Attempt to rename the file */
//         Status = NtSetInformationFile(SourceHandle,
//                                       &IoStatusBlock,
//                                       RenameInfo,
//                                       NewPathU.Length + sizeof(FILE_RENAME_INFORMATION),
//                                       ((dwFlags & MOVEFILE_CREATE_HARDLINK)
//                                         ? FileLinkInformation : FileRenameInformation));
//         RtlFreeHeap(RtlGetProcessHeap(), 0, RenameInfo);
//         if (NT_SUCCESS(Status))
//         {
//             /* If it succeed, all fine, quit */
//             Ret = TRUE;
//             _SEH2_LEAVE;
//         }
//         /* If we failed for any other reason than not the same device, fail.
//          * If we failed because of different devices, only allow renaming if user allowed copy.
//          */
//         if (Status != STATUS_NOT_SAME_DEVICE || !(dwFlags & MOVEFILE_COPY_ALLOWED))
//         {
//             /* ReactOS HACK! To be removed once all FSD have proper renaming support.
//              * Just leave status to error and leave. */
//             if (Status == STATUS_NOT_IMPLEMENTED)
//             {
//                 DPRINT1("Forcing copy, renaming not supported by FSD\n");
//             }
//             else
//             {
//                 BaseSetLastNTError(Status);
//                 _SEH2_LEAVE;
//             }
//         }
//
//         /* Close source file */
//         NtClose(SourceHandle);
//         SourceHandle = INVALID_HANDLE_VALUE;
//
//         /* Issue the copy of the file */
//         CopyContext.Flags = dwFlags;
//         CopyContext.UserRoutine = lpProgressRoutine;
//         CopyContext.UserData = lpData;
//         NewHandle = INVALID_HANDLE_VALUE;
//         ExistingHandle = INVALID_HANDLE_VALUE;
//
//         Ret = BasepCopyFileExW(lpExistingFileName,
//                                lpNewFileName,
//                                BasepMoveFileCopyProgress,
//                                &CopyContext,
//                                NULL,
//                                (!ReplaceIfExists ? COPY_FILE_FAIL_IF_EXISTS : 0)
//                                     | COPY_FILE_OPEN_SOURCE_FOR_WRITE,
//                                0,
//                                &ExistingHandle,
//                                &NewHandle);
//         if (!Ret)
//         {
//             /* If it failed, don't leak any handle */
//             if (ExistingHandle != INVALID_HANDLE_VALUE)
//             {
//                 CloseHandle(ExistingHandle);
//                 ExistingHandle = INVALID_HANDLE_VALUE;
//             }
//         }
//         else if (ExistingHandle != INVALID_HANDLE_VALUE)
//         {
//             if (NewHandle != INVALID_HANDLE_VALUE)
//             {
//                 /* If copying succeed, notify */
//                 Status = BasepNotifyTrackingService(&ExistingHandle,
//                                                     &ObjectAttributes,
//                                                     NewHandle,
//                                                     &NewPathU);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Fail in case it had to succeed */
//                     if (dwFlags & MOVEFILE_FAIL_IF_NOT_TRACKABLE)
//                     {
//                         if (NewHandle != INVALID_HANDLE_VALUE)
//                             CloseHandle(NewHandle);
//                         NewHandle = INVALID_HANDLE_VALUE;
//                         DeleteFileW(lpNewFileName);
//                         Ret = FALSE;
//                         BaseSetLastNTError(Status);
//                     }
//                 }
//             }
//
//             CloseHandle(ExistingHandle);
//             ExistingHandle = INVALID_HANDLE_VALUE;
//         }
//
//         /* In case copy worked, close file */
//         if (NewHandle != INVALID_HANDLE_VALUE)
//         {
//             CloseHandle(NewHandle);
//             NewHandle = INVALID_HANDLE_VALUE;
//         }
//
//         /* If it succeed, delete source file */
//         if (Ret)
//         {
//             if (!DeleteFileW(lpExistingFileName))
//             {
//                 /* Reset file attributes if required */
//                 SetFileAttributesW(lpExistingFileName, FILE_ATTRIBUTE_NORMAL);
//                 DeleteFileW(lpExistingFileName);
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         if (SourceHandle != INVALID_HANDLE_VALUE)
//             NtClose(SourceHandle);
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, ExistingPathU.Buffer);
//         RtlFreeHeap(RtlGetProcessHeap(), 0, NewPathU.Buffer);
//     }
//     _SEH2_END;
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileWithProgressA(IN LPCSTR lpExistingFileName,
//                       IN LPCSTR lpNewFileName OPTIONAL,
//                       IN LPPROGRESS_ROUTINE lpProgressRoutine OPTIONAL,
//                       IN LPVOID lpData OPTIONAL,
//                       IN DWORD dwFlags)
// {
//     BOOL Ret;
//     UNICODE_STRING ExistingFileNameW, NewFileNameW;
//
//     if (!Basep8BitStringToDynamicUnicodeString(&ExistingFileNameW, lpExistingFileName))
//     {
//         return FALSE;
//     }
//
//     if (lpNewFileName)
//     {
//         if (!Basep8BitStringToDynamicUnicodeString(&NewFileNameW, lpNewFileName))
//         {
//             RtlFreeUnicodeString(&ExistingFileNameW);
//             return FALSE;
//         }
//     }
//     else
//     {
//         NewFileNameW.Buffer = NULL;
//     }
//
//     Ret = MoveFileWithProgressW(ExistingFileNameW.Buffer,
//                                 NewFileNameW.Buffer,
//                                 lpProgressRoutine,
//                                 lpData,
//                                 dwFlags);
//
//     RtlFreeUnicodeString(&ExistingFileNameW);
//     RtlFreeUnicodeString(&NewFileNameW);
//
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileW(IN LPCWSTR lpExistingFileName,
//           IN LPCWSTR lpNewFileName)
// {
//     return MoveFileWithProgressW(lpExistingFileName,
//                                  lpNewFileName,
//                                  NULL,
//                                  NULL,
//                                  MOVEFILE_COPY_ALLOWED);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileExW(IN LPCWSTR lpExistingFileName,
//             IN LPCWSTR lpNewFileName OPTIONAL,
//             IN DWORD dwFlags)
// {
//     return MoveFileWithProgressW(lpExistingFileName,
//                                  lpNewFileName,
//                                  NULL,
//                                  NULL,
//                                  dwFlags);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileA(IN LPCSTR lpExistingFileName,
//           IN LPCSTR lpNewFileName)
// {
//     return MoveFileWithProgressA(lpExistingFileName,
//                                  lpNewFileName,
//                                  NULL,
//                                  NULL,
//                                  MOVEFILE_COPY_ALLOWED);
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// MoveFileExA(IN LPCSTR lpExistingFileName,
//             IN LPCSTR lpNewFileName OPTIONAL,
//             IN DWORD dwFlags)
// {
//     return MoveFileWithProgressA(lpExistingFileName,
//                                  lpNewFileName,
//                                  NULL,
//                                  NULL,
//                                  dwFlags);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// ReplaceFileA(IN LPCSTR lpReplacedFileName,
//              IN LPCSTR lpReplacementFileName,
//              IN LPCSTR lpBackupFileName OPTIONAL,
//              IN DWORD dwReplaceFlags,
//              IN LPVOID lpExclude,
//              IN LPVOID lpReserved)
// {
//     BOOL Ret;
//     UNICODE_STRING ReplacedFileNameW, ReplacementFileNameW, BackupFileNameW;
//
//     if (!lpReplacedFileName || !lpReplacementFileName || lpExclude || lpReserved ||
//         (dwReplaceFlags & ~(REPLACEFILE_WRITE_THROUGH | REPLACEFILE_IGNORE_MERGE_ERRORS)))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     if (!Basep8BitStringToDynamicUnicodeString(&ReplacedFileNameW, lpReplacedFileName))
//     {
//         return FALSE;
//     }
//
//     if (!Basep8BitStringToDynamicUnicodeString(&ReplacementFileNameW, lpReplacementFileName))
//     {
//         RtlFreeUnicodeString(&ReplacedFileNameW);
//         return FALSE;
//     }
//
//     if (lpBackupFileName)
//     {
//         if (!Basep8BitStringToDynamicUnicodeString(&BackupFileNameW, lpBackupFileName))
//         {
//             RtlFreeUnicodeString(&ReplacementFileNameW);
//             RtlFreeUnicodeString(&ReplacedFileNameW);
//             return FALSE;
//         }
//     }
//     else
//     {
//         BackupFileNameW.Buffer = NULL;
//     }
//
//     Ret = ReplaceFileW(ReplacedFileNameW.Buffer,
//                        ReplacementFileNameW.Buffer,
//                        BackupFileNameW.Buffer,
//                        dwReplaceFlags, 0, 0);
//
//     if (lpBackupFileName)
//     {
//         RtlFreeUnicodeString(&BackupFileNameW);
//     }
//     RtlFreeUnicodeString(&ReplacementFileNameW);
//     RtlFreeUnicodeString(&ReplacedFileNameW);
//
//     return Ret;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// ReplaceFileW(
//     LPCWSTR lpReplacedFileName,
//     LPCWSTR lpReplacementFileName,
//     LPCWSTR lpBackupFileName,
//     DWORD   dwReplaceFlags,
//     LPVOID  lpExclude,
//     LPVOID  lpReserved
//     )
// {
//     HANDLE hReplaced = NULL, hReplacement = NULL;
//     UNICODE_STRING NtReplacedName = { 0, 0, NULL };
//     UNICODE_STRING NtReplacementName = { 0, 0, NULL };
//     DWORD Error = ERROR_SUCCESS;
//     NTSTATUS Status;
//     BOOL Ret = FALSE;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     PVOID Buffer = NULL ;
//
//     if (dwReplaceFlags)
//         FIXME("Ignoring flags %x\n", dwReplaceFlags);
//
//     /* First two arguments are mandatory */
//     if (!lpReplacedFileName || !lpReplacementFileName)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     /* Back it up */
//     if(lpBackupFileName)
//     {
//         if(!CopyFileW(lpReplacedFileName, lpBackupFileName, FALSE))
//         {
//             Error = GetLastError();
//             goto Cleanup ;
//         }
//     }
//
//     /* Open the "replaced" file for reading and writing */
//     if (!(RtlDosPathNameToNtPathName_U(lpReplacedFileName, &NtReplacedName, NULL, NULL)))
//     {
//         Error = ERROR_PATH_NOT_FOUND;
//         goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtReplacedName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenFile(&hReplaced,
//                         GENERIC_READ | GENERIC_WRITE | DELETE | SYNCHRONIZE | WRITE_DAC,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         if (Status == STATUS_OBJECT_NAME_NOT_FOUND)
//             Error = ERROR_FILE_NOT_FOUND;
//         else
//             Error = ERROR_UNABLE_TO_REMOVE_REPLACED;
//         goto Cleanup;
//     }
//
//     /* Blank it */
//     SetEndOfFile(hReplaced) ;
//
//     /*
//      * Open the replacement file for reading, writing, and deleting
//      * (deleting is needed when finished)
//      */
//     if (!(RtlDosPathNameToNtPathName_U(lpReplacementFileName, &NtReplacementName, NULL, NULL)))
//     {
//         Error = ERROR_PATH_NOT_FOUND;
//         goto Cleanup;
//     }
//
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &NtReplacementName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     Status = NtOpenFile(&hReplacement,
//                         GENERIC_READ | DELETE | SYNCHRONIZE,
//                         &ObjectAttributes,
//                         &IoStatusBlock,
//                         0,
//                         FILE_SYNCHRONOUS_IO_NONALERT | FILE_NON_DIRECTORY_FILE | FILE_DELETE_ON_CLOSE);
//
//     if (!NT_SUCCESS(Status))
//     {
//         Error = RtlNtStatusToDosError(Status);
//         goto Cleanup;
//     }
//
//     Buffer = RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, 0x10000) ;
//     if (!Buffer)
//     {
//         Error = ERROR_NOT_ENOUGH_MEMORY;
//         goto Cleanup ;
//     }
//     while (Status != STATUS_END_OF_FILE)
//     {
//         Status = NtReadFile(hReplacement, NULL, NULL, NULL, &IoStatusBlock, Buffer, 0x10000, NULL, NULL) ;
//         if (NT_SUCCESS(Status))
//         {
//             Status = NtWriteFile(hReplaced, NULL, NULL, NULL, &IoStatusBlock, Buffer,
//                     IoStatusBlock.Information, NULL, NULL) ;
//             if (!NT_SUCCESS(Status))
//             {
//                 Error = RtlNtStatusToDosError(Status);
//                 goto Cleanup;
//             }
//         }
//         else if (Status != STATUS_END_OF_FILE)
//         {
//             Error = RtlNtStatusToDosError(Status);
//             goto Cleanup;
//         }
//     }
//
//     Ret = TRUE;
//
//     /* Perform resource cleanup */
// Cleanup:
//     if (hReplaced) NtClose(hReplaced);
//     if (hReplacement) NtClose(hReplacement);
//     if (Buffer) RtlFreeHeap(RtlGetProcessHeap(), 0, Buffer);
//
//     if (NtReplacementName.Buffer)
//         RtlFreeHeap(GetProcessHeap(), 0, NtReplacementName.Buffer);
//     if (NtReplacedName.Buffer)
//         RtlFreeHeap(GetProcessHeap(), 0, NtReplacedName.Buffer);
//
//     /* If there was an error, set the error code */
//     if(!Ret)
//     {
//         TRACE("ReplaceFileW failed (error=%lu)\n", Error);
//         SetLastError(Error);
//     }
//     return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// PrivMoveFileIdentityW(IN LPCWSTR lpSource, IN LPCWSTR lpDestination, IN DWORD dwFlags)
// {
//     ACCESS_MASK SourceAccess;
//     UNICODE_STRING NtSource, NtDestination;
//     LPWSTR RelativeSource, RelativeDestination;
//     HANDLE SourceHandle, DestinationHandle;
//     OBJECT_ATTRIBUTES ObjectAttributesSource, ObjectAttributesDestination;
//     NTSTATUS Status, OldStatus = STATUS_SUCCESS;
//     ACCESS_MASK DestAccess;
//     IO_STATUS_BLOCK IoStatusBlock;
//     FILE_BASIC_INFORMATION SourceInformation, DestinationInformation;
//     FILE_DISPOSITION_INFORMATION FileDispositionInfo;
//
//     DPRINT("PrivMoveFileIdentityW(%S, %S, %x)\n", lpSource, lpDestination, dwFlags);
//
//     SourceHandle = INVALID_HANDLE_VALUE;
//     RtlInitEmptyUnicodeString(&NtSource, NULL, 0);
//     RelativeSource = NULL;
//     DestinationHandle = INVALID_HANDLE_VALUE;
//     RtlInitEmptyUnicodeString(&NtDestination, NULL, 0);
//     RelativeDestination = NULL;
//
//     /* FILE_WRITE_DATA is required for later on notification */
//     SourceAccess = FILE_READ_ATTRIBUTES | FILE_WRITE_DATA;
//     if (dwFlags & PRIV_DELETE_ON_SUCCESS)
//     {
//         SourceAccess |= DELETE;
//     }
//
//     _SEH2_TRY
//     {
//         /* We will loop twice:
//          * First we attempt to open with FILE_WRITE_DATA for notification.
//          * If it fails and we have flag for non-trackable files, we retry
//          * without FILE_WRITE_DATA.
//          * If that one fails, then, we quit for real.
//          */
//         while (TRUE)
//         {
//             Status = BasepOpenFileForMove(lpSource,
//                                           &NtSource,
//                                           &RelativeSource,
//                                           &SourceHandle,
//                                           &ObjectAttributesSource,
//                                           SourceAccess,
//                                           FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                                           FILE_OPEN_NO_RECALL);
//             if (NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* If we already attempted the opening without FILE_WRITE_DATA
//              * or if we cannot move on non-trackable files, fail.
//              */
//             if (!(SourceAccess & FILE_WRITE_DATA) || !(dwFlags & PRIV_ALLOW_NON_TRACKABLE))
//             {
//                 _SEH2_LEAVE;
//             }
//
//             if (RelativeSource)
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSource);
//                 RelativeSource = NULL;
//             }
//
//             if (SourceHandle != INVALID_HANDLE_VALUE)
//             {
//                 NtClose(SourceHandle);
//                 SourceHandle = INVALID_HANDLE_VALUE;
//             }
//
//             SourceAccess &= ~FILE_WRITE_DATA;
//
//             /* Remember fist failure in the path */
//             if (NT_SUCCESS(OldStatus))
//             {
//                 OldStatus = Status;
//             }
//         }
//
//         DestAccess = FILE_WRITE_ATTRIBUTES;
//         /* If we could preserve FILE_WRITE_DATA for source, attempt to
//          * get it for destination, still for notification purposes. */
//         if (SourceAccess & FILE_WRITE_DATA)
//         {
//             DestAccess |= FILE_WRITE_DATA;
//         }
//
//         /* cf comment for first loop */
//         while (TRUE)
//         {
//             Status = BasepOpenFileForMove(lpDestination,
//                                           &NtDestination,
//                                           &RelativeDestination,
//                                           &DestinationHandle,
//                                           &ObjectAttributesDestination,
//                                           DestAccess,
//                                           FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
//                                           FILE_OPEN_NO_RECALL);
//             if (NT_SUCCESS(Status))
//             {
//                 break;
//             }
//
//             /* If we already attempted the opening without FILE_WRITE_DATA
//              * or if we cannot move on non-trackable files, fail.
//              */
//             if (!(DestAccess & FILE_WRITE_DATA) || !(dwFlags & PRIV_ALLOW_NON_TRACKABLE))
//             {
//                 _SEH2_LEAVE;
//             }
//
//             if (RelativeDestination)
//             {
//                 RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeDestination);
//                 RelativeDestination = NULL;
//             }
//
//             if (DestinationHandle != INVALID_HANDLE_VALUE)
//             {
//                 NtClose(DestinationHandle);
//                 DestinationHandle = INVALID_HANDLE_VALUE;
//             }
//
//             DestAccess &= ~FILE_WRITE_DATA;
//
//             /* Remember fist failure in the path */
//             if (NT_SUCCESS(OldStatus))
//             {
//                 OldStatus = Status;
//             }
//         }
//
//         /* Get the creation time from source */
//         Status = NtQueryInformationFile(SourceHandle,
//                                         &IoStatusBlock,
//                                         &SourceInformation,
//                                         sizeof(SourceInformation),
//                                         FileBasicInformation);
//         if (NT_SUCCESS(Status))
//         {
//             /* Then, prepare to set it for destination */
//             RtlZeroMemory(&DestinationInformation, sizeof(DestinationInformation));
//             DestinationInformation.CreationTime.QuadPart = SourceInformation.CreationTime.QuadPart;
//
//             /* And set it, that's all folks! */
//             Status = NtSetInformationFile(DestinationHandle,
//                                           &IoStatusBlock,
//                                           &DestinationInformation,
//                                           sizeof(DestinationInformation),
//                                           FileBasicInformation);
//         }
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (!(dwFlags & PRIV_ALLOW_NON_TRACKABLE))
//             {
//                 _SEH2_LEAVE;
//             }
//
//             /* Remember the failure for later notification */
//             if (NT_SUCCESS(OldStatus))
//             {
//                 OldStatus = Status;
//             }
//         }
//
//         /* If we could open with FILE_WRITE_DATA both source and destination,
//          * then, notify
//          */
//         if (DestAccess & FILE_WRITE_DATA && SourceAccess & FILE_WRITE_DATA)
//         {
//             Status = BasepNotifyTrackingService(&SourceHandle,
//                                                 &ObjectAttributesSource,
//                                                 DestinationHandle,
//                                                 &NtDestination);
//             if (!NT_SUCCESS(Status))
//             {
//                 if (dwFlags & PRIV_ALLOW_NON_TRACKABLE)
//                 {
//                     if (NT_SUCCESS(OldStatus))
//                         OldStatus = Status;
//
//                     /* Reset status, we allow non trackable files */
//                     Status = STATUS_SUCCESS;
//                 }
//             }
//         }
//     }
//     _SEH2_FINALLY
//     {
//         if (RelativeSource)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeSource);
//
//         if (RelativeDestination)
//             RtlFreeHeap(RtlGetProcessHeap(), 0, RelativeDestination);
//     }
//     _SEH2_END;
//
//     /* If caller asked for source deletion, if everything succeed, proceed */
//     if (NT_SUCCESS(Status) && dwFlags & PRIV_DELETE_ON_SUCCESS)
//     {
//         FileDispositionInfo.DeleteFile = TRUE;
//
//         Status = NtSetInformationFile(SourceHandle,
//                                       &IoStatusBlock,
//                                       &FileDispositionInfo,
//                                       sizeof(FileDispositionInfo),
//                                       FileDispositionInformation);
//     }
//
//     /* Cleanup/close portion */
//     if (DestinationHandle != INVALID_HANDLE_VALUE)
//     {
//         NtClose(DestinationHandle);
//     }
//
//     if (SourceHandle != INVALID_HANDLE_VALUE)
//     {
//         NtClose(SourceHandle);
//     }
//
//     /* Set last error if any, and quit */
//     if (NT_SUCCESS(Status))
//     {
//         if (!NT_SUCCESS(OldStatus))
//         {
//             BaseSetLastNTError(OldStatus);
//         }
//     }
//     else
//     {
//         BaseSetLastNTError(Status);
//     }
//
//     return NT_SUCCESS(Status);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
