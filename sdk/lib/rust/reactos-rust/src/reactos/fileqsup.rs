//! ORPHAN-BY-DESIGN: superseded by sdk/lib/rust/spapisup-rust (the one copy of usetup's
//! cabinet.c + fileqsup.c, which this TU #included). Same banked translation: 29 of 34
//! bodies byte-identical, the other 5 differ only in integrate spelling (`!NT_SUCCESS!`,
//! `as UINT`). Kept on disk only until its deletion is approved (paid artifact).
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
use core::ffi::c_void;

// base/setup/reactos/spapisup/fileqsup.c (WINE_SETUPAPI_CAB_EXTRACTION_IS_NOT_SLOW_ANYMORE
// undefined, the active branch): the TU #includes usetup's spapisup/cabinet.c and
// spapisup/fileqsup.c, with SetupOpenFileQueue/SetupCloseFileQueue/SetupQueueDeleteW/
// SetupQueueRenameW/SetupCommitFileQueueW #define'd to their `_`-prefixed names so they do
// not collide with setupapi.h's. Those two included files are ported below from
// usetup-rust's translation of the very same C sources (cabinet.rs, fileqsup.rs), with the
// same renames applied; SpFileExports is this TU's own table.

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpFileExports: SPFILE_EXPORTS = _SPFILE_EXPORTS {
    SpFileQueueOpen: Some(_SetupOpenFileQueue),
    SpFileQueueClose: Some(_SetupCloseFileQueue),
    SpFileQueueCopy: Some(SetupQueueCopyWithCab),
    SpFileQueueDelete: Some(_SetupQueueDeleteW),
    SpFileQueueRename: Some(_SetupQueueRenameW),
    SpFileQueueCommit: Some(_SetupCommitFileQueueW),
};

// ======== #include "../../usetup/spapisup/cabinet.c" ========
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_96A296D224F2: [u16; 1] = [0x0000u16];
pub static RUSTCONV_WSTR_E8A4B2EE7EDE: [u16; 2] = [0x002Au16, 0x0000];


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut RawCodec: CAB_CODEC = unsafe {
    _CAB_CODEC {
        Uncompress: Some(
            RawCodecUncompress
                as unsafe extern "C" fn(PCAB_CODEC, PVOID, PVOID, PLONG, PLONG) -> ULONG,
        ),
        ZStream: z_stream_s {
            next_in: ::core::ptr::null::<Bytef>() as *mut Bytef,
            avail_in: 0,
            total_in: 0,
            next_out: ::core::ptr::null::<Bytef>() as *mut Bytef,
            avail_out: 0,
            total_out: 0,
            msg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
            state: ::core::ptr::null::<internal_state>() as *mut internal_state,
            zalloc: None,
            zfree: None,
            opaque: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            data_type: 0,
            adler: 0,
            reserved: 0,
        },
    }
};

static mut MSZipCodec: CAB_CODEC = unsafe {
    _CAB_CODEC {
        Uncompress: Some(
            MSZipCodecUncompress
                as unsafe extern "C" fn(PCAB_CODEC, PVOID, PVOID, PLONG, PLONG) -> ULONG,
        ),
        ZStream: z_stream_s {
            next_in: ::core::ptr::null::<Bytef>() as *mut Bytef,
            avail_in: 0,
            total_in: 0,
            next_out: ::core::ptr::null::<Bytef>() as *mut Bytef,
            avail_out: 0,
            total_out: 0,
            msg: ::core::ptr::null::<::core::ffi::c_char>() as *mut ::core::ffi::c_char,
            state: ::core::ptr::null::<internal_state>() as *mut internal_state,
            zalloc: None,
            zfree: None,
            opaque: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
            data_type: 0,
            adler: 0,
            reserved: 0,
        },
    }
};

pub const SEEK_BEGIN: i32 = 0;
pub const SEEK_CURRENT: i32 = 1;
pub const SEEK_END: i32 = 2;

#[repr(C)]
pub struct _DOSTIME {
    pub Second: u16,
    pub Minute: u16,
    pub Hour: u16,
}
pub type DOSTIME = _DOSTIME;
pub type PDOSTIME = *mut _DOSTIME;

#[repr(C)]
pub struct _DOSDATE {
    pub Day: u16,
    pub Month: u16,
    pub Year: u16,
}
pub type DOSDATE = _DOSDATE;
pub type PDOSDATE = *mut _DOSDATE;

pub const CAB_SIGNATURE: u32 = 0x4643534D;
pub const CAB_VERSION: u16 = 0x0103;
pub const CAB_BLOCKSIZE: u32 = 32768;
pub const CAB_COMP_MASK: u16 = 0x00FF;
pub const CAB_COMP_NONE: u16 = 0x0000;
pub const CAB_COMP_MSZIP: u16 = 0x0001;
pub const CAB_COMP_QUANTUM: u16 = 0x0002;
pub const CAB_COMP_LZX: u16 = 0x0003;
pub const CAB_FLAG_HASPREV: u16 = 0x0001;
pub const CAB_FLAG_HASNEXT: u16 = 0x0002;
pub const CAB_FLAG_RESERVE: u16 = 0x0004;
pub const CAB_ATTRIB_READONLY: u16 = 0x0001;
pub const CAB_ATTRIB_HIDDEN: u16 = 0x0002;
pub const CAB_ATTRIB_SYSTEM: u16 = 0x0004;
pub const CAB_ATTRIB_VOLUME: u16 = 0x0008;
pub const CAB_ATTRIB_DIRECTORY: u16 = 0x0010;
pub const CAB_ATTRIB_ARCHIVE: u16 = 0x0020;
pub const CAB_ATTRIB_EXECUTE: u16 = 0x0040;
pub const CAB_ATTRIB_UTF_NAME: u16 = 0x0080;
pub const CAB_FILE_MAX_FOLDER: u16 = 0xFFFC;
pub const CAB_FILE_CONTINUED: u16 = 0xFFFD;
pub const CAB_FILE_SPLIT: u16 = 0xFFFE;
pub const CAB_FILE_PREV_NEXT: u16 = 0xFFFF;

pub type CFFOLDER = _CFFOLDER;
pub type CFFILE = _CFFILE;
pub type CFDATA = _CFDATA;
pub const MSZIP_MAGIC: u16 = 0x4B43;
#[no_mangle]
pub unsafe extern "C" fn RawCodecUncompress(
    Codec: PCAB_CODEC,
    OutputBuffer: PVOID,
    InputBuffer: PVOID,
    InputLength: PLONG,
    OutputLength: PLONG,
) -> ULONG {
    let mut Len: LONG = min!(abs(*InputLength), abs(*OutputLength));

    memcpy(OutputBuffer, InputBuffer, (Len) as usize);
    *OutputLength = Len;
    *InputLength = Len;

    return CS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MSZipCodecUncompress(
    Codec: *mut CAB_CODEC,
    OutputBuffer: *mut core::ffi::c_void,
    InputBuffer: *mut core::ffi::c_void,
    InputLength: *mut i32,
    OutputLength: *mut i32,
) -> u32 {
    let mut Magic: u16;
    let mut Status: i32;

    if *InputLength > 0 {
        Magic = *(InputBuffer as *mut u16);

        if Magic != MSZIP_MAGIC {
            return CS_BADSTREAM;
        }

        (*Codec).ZStream.next_in = (InputBuffer as *mut u8).add(2 as usize);
        (*Codec).ZStream.avail_in = (*InputLength - 2) as u32;
        (*Codec).ZStream.next_out = OutputBuffer as *mut u8;
        (*Codec).ZStream.avail_out = (abs(*OutputLength)) as u32;

        Status = inflateInit2(&mut (*Codec).ZStream, -MAX_WBITS);
        if (Status) as u32 != Z_OK {
            return CS_BADSTREAM;
        }
        (*Codec).ZStream.total_in = 2;
    } else {
        (*Codec).ZStream.avail_in = -*InputLength as u32;
        (*Codec).ZStream.next_in = InputBuffer as *mut u8;
        (*Codec).ZStream.next_out = OutputBuffer as *mut u8;
        (*Codec).ZStream.avail_out = (abs(*OutputLength)) as u32;
        (*Codec).ZStream.total_in = 0;
    }

    (*Codec).ZStream.total_out = 0;
    Status = inflate(&mut (*Codec).ZStream, (Z_SYNC_FLUSH) as i32);
    if (Status) as u32 != Z_OK && (Status) as u32 != Z_STREAM_END {
        if Status == Z_MEM_ERROR {
            return CS_NOMEMORY;
        }
        return CS_BADSTREAM;
    }

    if *OutputLength > 0 {
        Status = inflateEnd(&mut (*Codec).ZStream);
        if (Status) as u32 != Z_OK {
            return CS_BADSTREAM;
        }
    }

    *InputLength = (*Codec).ZStream.total_in as i32;
    *OutputLength = (*Codec).ZStream.total_out as i32;

    return CS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn MSZipAlloc(
    opaque: *mut core::ffi::c_void,
    items: u32,
    size: u32,
) -> *mut core::ffi::c_void {
    return RtlAllocateHeap(
        ProcessHeap,
        0,
        (items.wrapping_mul(size)) as SIZE_T,
    );
}
#[no_mangle]
pub unsafe extern "C" fn MSZipFree(opaque: *mut core::ffi::c_void, address: *mut core::ffi::c_void) {
    RtlFreeHeap(ProcessHeap, 0, address);
}

unsafe extern "C" fn ConvertSystemTimeToFileTime(
    lpSystemTime: *const SYSTEMTIME,
    lpFileTime: LPFILETIME,
) -> BOOL {
    let mut TimeFields: TIME_FIELDS = TIME_FIELDS {
        Year: 0,
        Month: 0,
        Day: 0,
        Hour: 0,
        Minute: 0,
        Second: 0,
        Milliseconds: 0,
        Weekday: 0,
    };
    let mut liTime: LARGE_INTEGER = LARGE_INTEGER {
        u: core::mem::zeroed(),
    };

    TimeFields.Year = (*lpSystemTime).wYear as i16;
    TimeFields.Month = (*lpSystemTime).wMonth as i16;
    TimeFields.Day = (*lpSystemTime).wDay as i16;
    TimeFields.Hour = (*lpSystemTime).wHour as i16;
    TimeFields.Minute = (*lpSystemTime).wMinute as i16;
    TimeFields.Second = (*lpSystemTime).wSecond as i16;
    TimeFields.Milliseconds = (*lpSystemTime).wMilliseconds as i16;

    if RtlTimeFieldsToTime(&mut TimeFields, &mut liTime) != 0 {
        (*lpFileTime).dwLowDateTime = liTime.u.LowPart;
        (*lpFileTime).dwHighDateTime = liTime.u.HighPart as u32;
        return (TRUE) as BOOL;
    }

    return (FALSE) as BOOL;
}

unsafe extern "C" fn ConvertDosDateTimeToFileTime(
    mut wFatDate: WORD,
    mut wFatTime: WORD,
    mut lpFileTime: LPFILETIME,
) -> BOOL {
    let mut pdtime: PDOSTIME = &raw mut wFatTime as PDOSTIME;
    let mut pddate: PDOSDATE = &raw mut wFatDate as PDOSDATE;
    let mut SystemTime: SYSTEMTIME = SYSTEMTIME {
        wYear: 0,
        wMonth: 0,
        wDayOfWeek: 0,
        wDay: 0,
        wHour: 0,
        wMinute: 0,
        wSecond: 0,
        wMilliseconds: 0,
    };
    if lpFileTime.is_null() {
        return (FALSE) as BOOL;
    }
    SystemTime.wMilliseconds = 0;
    SystemTime.wSecond = (*pdtime).Second;
    SystemTime.wMinute = (*pdtime).Minute;
    SystemTime.wHour = (*pdtime).Hour;
    SystemTime.wDay = (*pddate).Day;
    SystemTime.wMonth = (*pddate).Month;
    SystemTime.wYear = (1980 + (*pddate).Year as i32) as WORD;
    ConvertSystemTimeToFileTime(&raw mut SystemTime, lpFileTime);
    return (TRUE) as BOOL;
}

unsafe extern "C" fn GetFileName(mut Path: PWCHAR) -> PWCHAR {
    let mut i: ULONG = 0;
    let mut j: ULONG = 0;
    i = 0;
    j = i;
    loop {
        let fresh31 = i;
        i = i.wrapping_add(1);
        if !(*Path.offset(fresh31 as isize) != 0) {
            break;
        }
        if *Path.offset(i.wrapping_sub(1) as isize) as ::core::ffi::c_int
            == 0x005Cu16 as ::core::ffi::c_int
        {
            j = i;
        }
    }
    return Path.offset(j as isize);
}

unsafe extern "C" fn RemoveFileName(mut Path: PWCHAR) {
    let mut FileName: PWCHAR = core::ptr::null_mut();
    let mut i: u32 = 0;
    i = 0;
    FileName = GetFileName(Path.add(i as usize));
    if FileName != Path.add(i as usize)
        && *FileName.sub(1 as usize) == 0x005Cu16
    {
        FileName = FileName.sub(1);
    }
    if FileName == Path.add(i as usize)
        && *FileName.add(0 as usize) == 0x005Cu16
    {
        FileName = FileName.add(1);
    }
    *FileName.add(0 as usize) = 0;
}

unsafe extern "C" fn SetAttributesOnFile(File: PCFFILE, hFile: HANDLE) -> BOOL {
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut NtStatus: NTSTATUS;
    let mut Attributes: ULONG = 0;

    if (*File).Attributes & CAB_ATTRIB_READONLY != 0 {
        Attributes |= FILE_ATTRIBUTE_READONLY;
    }
    if (*File).Attributes & CAB_ATTRIB_HIDDEN != 0 {
        Attributes |= FILE_ATTRIBUTE_HIDDEN;
    }
    if (*File).Attributes & CAB_ATTRIB_SYSTEM != 0 {
        Attributes |= FILE_ATTRIBUTE_SYSTEM;
    }
    if (*File).Attributes & CAB_ATTRIB_DIRECTORY != 0 {
        Attributes |= FILE_ATTRIBUTE_DIRECTORY;
    }
    if (*File).Attributes & CAB_ATTRIB_ARCHIVE != 0 {
        Attributes |= FILE_ATTRIBUTE_ARCHIVE;
    }

    NtStatus = NtQueryInformationFile(
        hFile,
        &mut IoStatusBlock,
        &mut FileBasic as *mut _ as PVOID,
        core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
        FileBasicInformation,
    );
    if (!NT_SUCCESS!(NtStatus)) {
        DPRINT!(b"NtQueryInformationFile() failed (%x)\n\0".as_ptr() as *const i8, NtStatus);
    } else {
        FileBasic.FileAttributes = Attributes;
        NtStatus = NtSetInformationFile(
            hFile,
            &mut IoStatusBlock,
            &mut FileBasic as *mut _ as PVOID,
            core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
            FileBasicInformation,
        );
        if (!NT_SUCCESS!(NtStatus)) {
            DPRINT!(b"NtSetInformationFile() failed (%x)\n\0".as_ptr() as *const i8, NtStatus);
        }
    }

    return NT_SUCCESS!(NtStatus) as BOOL;
}

unsafe extern "C" fn CloseCabinet(mut CabinetContext: PCABINET_CONTEXT) -> ULONG {
    if !(*CabinetContext).FileBuffer.is_null() {
        NtUnmapViewOfSection(NtCurrentProcess(), ((*CabinetContext).FileBuffer).cast());
        NtClose((*CabinetContext).FileSectionHandle);
        NtClose((*CabinetContext).FileHandle);
        (*CabinetContext).FileBuffer = core::ptr::null_mut();
    }
    return 0;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetInitialize(CabinetContext: PCABINET_CONTEXT) {
    RtlZeroMemory!(CabinetContext as *mut core::ffi::c_void, core::mem::size_of::<CABINET_CONTEXT>());

    (*CabinetContext).FileOpen = (FALSE) as i32;
    wcscpy((*CabinetContext).DestPath.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr());

    (*CabinetContext).CodecSelected = (FALSE) as i32;
    CabinetSelectCodec(CabinetContext, CAB_CODEC_RAW);

    (*CabinetContext).OverwriteHandler = None;
    (*CabinetContext).ExtractHandler = None;
    (*CabinetContext).DiskChangeHandler = None;

    (*CabinetContext).FolderUncompSize = 0;
    (*CabinetContext).BytesLeftInBlock = 0;
    (*CabinetContext).CabinetReserved = 0;
    (*CabinetContext).FolderReserved = 0;
    (*CabinetContext).DataReserved = 0;
    (*CabinetContext).CabinetReservedArea = core::ptr::null_mut();
    (*CabinetContext).LastFileOffset = 0;
}

#[no_mangle]
pub unsafe extern "C" fn CabinetCleanup(CabinetContext: PCABINET_CONTEXT) {
    CabinetClose(CabinetContext);
}

unsafe extern "C" fn CabinetNormalizePath(Path: *mut u16, Length: u32) -> i32 {
    let mut n: u32;
    let mut Ok_: i32;

    n = (wcslen(Path)) as _;
    Ok_ = ((n + 1) < Length) as i32;

    if n != 0 && *Path.add((n - 1) as usize) != 0x005Cu16 && Ok_ != 0 {
        *Path.add(n as usize) = 0x005Cu16;
        *Path.add((n + 1) as usize) = 0;
    }

    return Ok_;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetGetCabinetName(CabinetContext: PCABINET_CONTEXT) -> PCWSTR {
    return ((*CabinetContext).CabinetName).as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn CabinetSetCabinetName(CabinetContext: PCABINET_CONTEXT, FileName: PCWSTR) {
    wcscpy((*CabinetContext).CabinetName.as_mut_ptr(), FileName);
}
#[no_mangle]
pub unsafe extern "C" fn CabinetSetDestinationPath(CabinetContext: PCABINET_CONTEXT, DestinationPath: PCWSTR) {
    wcscpy((*CabinetContext).DestPath.as_mut_ptr(), DestinationPath);

    if wcslen((*CabinetContext).DestPath.as_ptr()) > 0 {
        CabinetNormalizePath((*CabinetContext).DestPath.as_mut_ptr(), MAX_PATH);
    }
}
#[no_mangle]
pub unsafe extern "C" fn CabinetGetDestinationPath(
    CabinetContext: *mut CABINET_CONTEXT,
) -> PCWSTR {
    return ((*CabinetContext).DestPath).as_ptr();
}
#[no_mangle]
pub unsafe extern "C" fn CabinetOpen(CabinetContext: PCABINET_CONTEXT) -> ULONG {
    let mut Buffer: *mut u8;
    let mut ustring: _UNICODE_STRING = core::mem::zeroed();
    let mut astring: ANSI_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut FileName: _UNICODE_STRING = core::mem::zeroed();
    let mut StringLength: u16;
    let mut NtStatus: i32;

    if (*CabinetContext).FileOpen != 0 {
        return CAB_STATUS_SUCCESS;
    }

    RtlInitUnicodeString(&mut FileName, (*CabinetContext).CabinetName.as_mut_ptr());

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut FileName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    NtStatus = NtOpenFile(
        &mut (*CabinetContext).FileHandle,
        GENERIC_READ | SYNCHRONIZE,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        FILE_SHARE_READ,
        FILE_SYNCHRONOUS_IO_NONALERT,
    );

    if (!NT_SUCCESS!(NtStatus)) {
        return CAB_STATUS_CANNOT_OPEN;
    }

    (*CabinetContext).FileOpen = (TRUE) as i32;

    NtStatus = NtCreateSection(
        &mut (*CabinetContext).FileSectionHandle,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        PAGE_READONLY,
        SEC_COMMIT,
        (*CabinetContext).FileHandle,
    );

    if (!NT_SUCCESS!(NtStatus)) {
        return CAB_STATUS_NOMEMORY;
    }

    (*CabinetContext).FileBuffer = core::ptr::null_mut();
    (*CabinetContext).FileSize = 0;

    NtStatus = NtMapViewOfSection(
        (*CabinetContext).FileSectionHandle,
        NtCurrentProcess(),
        &mut (*CabinetContext).FileBuffer as *mut *mut u8 as *mut *mut core::ffi::c_void,
        0,
        0,
        core::ptr::null_mut(),
        &mut (*CabinetContext).FileSize,
        ViewUnmap,
        0,
        PAGE_READONLY,
    );

    if (!NT_SUCCESS!(NtStatus)) {
        return CAB_STATUS_NOMEMORY;
    }

    (*CabinetContext).PCABHeader = (*CabinetContext).FileBuffer as *mut CFHEADER;

    if (*CabinetContext).FileSize <= core::mem::size_of::<CFHEADER>() as SIZE_T
        || (*(*CabinetContext).PCABHeader).Signature != CAB_SIGNATURE
        || (*(*CabinetContext).PCABHeader).Version != CAB_VERSION
        || (*(*CabinetContext).PCABHeader).FolderCount == 0
        || (*(*CabinetContext).PCABHeader).FileCount == 0
        || (*(*CabinetContext).PCABHeader).FileTableOffset < core::mem::size_of::<CFHEADER>() as u32
    {
        CloseCabinet(CabinetContext);
        return CAB_STATUS_INVALID_CAB;
    }

    // C: (PUCHAR)(PCABHeader + 1) -- past the whole CFHEADER. Casting to bytes FIRST
    // stepped one byte, so the folder table was read from inside the header and the first
    // file extracted faulted on a garbage CFDATA pointer (the install smoke's first copy).
    Buffer = (*CabinetContext).PCABHeader.add(1) as *mut u8;

    if (*(*CabinetContext).PCABHeader).Flags & CAB_FLAG_RESERVE != 0 {
        (*CabinetContext).CabinetReserved = *(Buffer as *mut u16) as u32;
        Buffer = Buffer.add(2);
        (*CabinetContext).FolderReserved = *Buffer as u32;
        Buffer = Buffer.add(1);
        (*CabinetContext).DataReserved = *Buffer as u32;
        Buffer = Buffer.add(1);

        if (*CabinetContext).CabinetReserved > 0 {
            (*CabinetContext).CabinetReservedArea = Buffer as *mut core::ffi::c_void;
            Buffer = Buffer.add((*CabinetContext).CabinetReserved as usize);
        }
    }

    if (*(*CabinetContext).PCABHeader).Flags & CAB_FLAG_HASPREV != 0 {
        wcscpy((*CabinetContext).CabinetPrev.as_mut_ptr(), (*CabinetContext).CabinetName.as_mut_ptr());
        RemoveFileName((*CabinetContext).CabinetPrev.as_mut_ptr());
        CabinetNormalizePath((*CabinetContext).CabinetPrev.as_mut_ptr(), (core::mem::size_of_val(&(*CabinetContext).CabinetPrev)) as _);
        RtlInitAnsiString(&mut astring, Buffer as *const i8);

        StringLength = (wcslen((*CabinetContext).CabinetPrev.as_mut_ptr()) as u16) * core::mem::size_of::<u16>() as u16;
        ustring.Buffer = (*CabinetContext).CabinetPrev.as_mut_ptr().add(StringLength as usize);
        ustring.MaximumLength = (core::mem::size_of_val(&(*CabinetContext).CabinetPrev) - StringLength as usize) as u16;
        ustring.Length = 0;
        RtlAnsiStringToUnicodeString(&mut ustring, &mut astring, (FALSE) as BOOLEAN);
        Buffer = Buffer.add(astring.Length as usize + 1);

        RtlInitAnsiString(&mut astring, Buffer as *const i8);
        ustring.Length = 0;
        ustring.Buffer = (*CabinetContext).DiskPrev.as_mut_ptr();
        ustring.MaximumLength = core::mem::size_of_val(&(*CabinetContext).DiskPrev) as u16;
        RtlAnsiStringToUnicodeString(&mut ustring, &mut astring, (FALSE) as BOOLEAN);
        Buffer = Buffer.add(astring.Length as usize + 1);
    } else {
        wcscpy((*CabinetContext).CabinetPrev.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr());
        wcscpy((*CabinetContext).DiskPrev.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    }

    if (*(*CabinetContext).PCABHeader).Flags & CAB_FLAG_HASNEXT != 0 {
        wcscpy((*CabinetContext).CabinetNext.as_mut_ptr(), (*CabinetContext).CabinetName.as_mut_ptr());
        RemoveFileName((*CabinetContext).CabinetNext.as_mut_ptr());
        CabinetNormalizePath((*CabinetContext).CabinetNext.as_mut_ptr(), 256);
        RtlInitAnsiString(&mut astring, Buffer as *const i8);

        StringLength = (wcslen((*CabinetContext).CabinetNext.as_mut_ptr()) as u16) * core::mem::size_of::<u16>() as u16;
        ustring.Buffer = (*CabinetContext).CabinetNext.as_mut_ptr().add(StringLength as usize);
        ustring.MaximumLength = (core::mem::size_of_val(&(*CabinetContext).CabinetNext) - StringLength as usize) as u16;
        ustring.Length = 0;
        RtlAnsiStringToUnicodeString(&mut ustring, &mut astring, (FALSE) as BOOLEAN);
        Buffer = Buffer.add(astring.Length as usize + 1);

        RtlInitAnsiString(&mut astring, Buffer as *const i8);
        ustring.Length = 0;
        ustring.Buffer = (*CabinetContext).DiskNext.as_mut_ptr();
        ustring.MaximumLength = core::mem::size_of_val(&(*CabinetContext).DiskNext) as u16;
        RtlAnsiStringToUnicodeString(&mut ustring, &mut astring, (FALSE) as BOOLEAN);
        Buffer = Buffer.add(astring.Length as usize + 1);
    } else {
        wcscpy((*CabinetContext).CabinetNext.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr());
        wcscpy((*CabinetContext).DiskNext.as_mut_ptr(), RUSTCONV_WSTR_96A296D224F2.as_ptr());
    }

    (*CabinetContext).CabinetFolders = Buffer as *mut CFFOLDER;

    return CAB_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetClose(CabinetContext: PCABINET_CONTEXT) {
    if (*CabinetContext).FileOpen == 0 {
        return;
    }
    CloseCabinet(CabinetContext);
    (*CabinetContext).FileOpen = (FALSE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetFindFirst(CabinetContext: PCABINET_CONTEXT, FileName: PCWSTR, Search: PCAB_SEARCH) -> ULONG {
    DPRINT!(
        b"CabinetFindFirst(FileName = %S)\n\0".as_ptr() as *const i8,
        FileName,
    );
    wcsncpy(
        (*Search).Search.as_mut_ptr(),
        FileName,
        (MAX_PATH) as _,
    );
    wcsncpy(
        (*Search).Cabinet.as_mut_ptr(),
        (*CabinetContext).CabinetName.as_mut_ptr(),
        (MAX_PATH) as _,
    );
    (*Search).File = core::ptr::null_mut();
    return CabinetFindNext(CabinetContext, Search);
}
#[no_mangle]
pub unsafe extern "C" fn CabinetFindNext(
    CabinetContext: PCABINET_CONTEXT,
    Search: PCAB_SEARCH,
) -> ULONG {
    let mut Prev: PCFFILE = core::ptr::null_mut();
    let mut AnsiString: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut UnicodeString: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut FileName: [u16; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if wcscmp(((*Search).Cabinet).as_ptr(), ((*CabinetContext).CabinetName).as_ptr()) != 0 {
        (*Search).File = core::ptr::null_mut();
    }

    if (*Search).File.is_null() {
        (*Search).File = ((*CabinetContext).FileBuffer as *const u8)
            .add((*(*CabinetContext).PCABHeader).FileTableOffset as usize) as PCFFILE;
        (*Search).Index = 0;
        Prev = core::ptr::null_mut();
    } else {
        Prev = (*Search).File;
    }

    loop {
        if (*Search).File.is_null() {
            break;
        }
        if (*(*Search).File).FolderIndex == 0xFFFD
            || (*(*Search).File).FolderIndex == 0xFFFF
        {
            DPRINT!(
                b"Skipping file (%s): FileOffset (0x%X), LastFileOffset (0x%X)\n\0".as_ptr() as *const i8,
                (*(*Search).File).FileName.as_ptr() as *const i8,
                (*(*Search).File).FileOffset,
                (*CabinetContext).LastFileOffset,
            );
        } else {
            if (*Search).File != Prev {
                if Prev.is_null() || (*(*Search).File).FolderIndex != (*Prev).FolderIndex {
                    (*Search).CFData = core::ptr::null_mut();
                    (*Search).Offset = 0;
                }

                if wcscmp(((*Search).Search).as_ptr(), RUSTCONV_WSTR_E8A4B2EE7EDE.as_ptr()) == 0 {
                    break;
                } else {
                    RtlInitAnsiString(&mut AnsiString, (*(*Search).File).FileName.as_mut_ptr());
                    UnicodeString.Buffer = FileName.as_mut_ptr();
                    *UnicodeString.Buffer.add(0) = 0;
                    UnicodeString.Length = 0;
                    UnicodeString.MaximumLength = core::mem::size_of::<[u16; (MAX_PATH) as _]>() as u16;
                    RtlAnsiStringToUnicodeString(&mut UnicodeString, &mut AnsiString, (FALSE) as BOOLEAN);
                    if wcscmp(((*Search).Search).as_ptr(), UnicodeString.Buffer) == 0 {
                        break;
                    }
                }
            }
        }

        (*Search).Index = (*Search).Index.wrapping_add(1);
        if (*Search).Index >= (*(*CabinetContext).PCABHeader).FileCount {
            DPRINT!(b"End of cabinet reached\n\0".as_ptr() as *const i8);
            return CAB_STATUS_NOFILE;
        } else {
            // C: strchr((char *)(Search->File + 1), 0) + 1. CFFILE ends in FileName[1], so
            // the C starts its scan 4 bytes into the name and skips a name of 3 characters
            // or fewer. Scan from the name itself. (The byte cast this replaces scanned from
            // inside FileSize, met its zero high byte, and walked into garbage.)
            (*Search).File = strchr((*(*Search).File).FileName.as_ptr() as *const i8, 0).add(1) as PCFFILE;
        }
    }

    DPRINT!(
        b"Found file %s\n\0".as_ptr() as *const i8,
        (*(*Search).File).FileName.as_ptr(),
    );
    return CAB_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetFindNextFileSequential(CabinetContext: PCABINET_CONTEXT, FileName: PCWSTR, Search: PCAB_SEARCH) -> ULONG {
    wcsncpy((*Search).Search.as_mut_ptr(), FileName, (MAX_PATH) as _);
    return CabinetFindNext(CabinetContext, Search);
}

#[no_mangle]
pub unsafe extern "C" fn CabinetExtractFile(CabinetContext: PCABINET_CONTEXT, Search: PCAB_SEARCH) -> ULONG {
    let mut Size: ULONG = 0;
    let mut CurrentOffset: ULONG = 0;
    let mut CurrentBuffer: PUCHAR = core::ptr::null_mut();
    let mut RemainingBlock: LONG = 0;
    let mut DestFile: HANDLE = core::ptr::null_mut();
    let mut DestFileSection: HANDLE = core::ptr::null_mut();
    let mut DestFileBuffer: PVOID = core::ptr::null_mut();
    let mut CurrentDestBuffer: PVOID = core::ptr::null_mut();
    let mut CFData: PCFDATA = core::ptr::null_mut();
    let mut Status: ULONG = 0;
    let mut FileTime: FILETIME = core::mem::zeroed();
    let mut DestName: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut NtStatus: NTSTATUS = 0;
    let mut UnicodeString: _UNICODE_STRING = core::mem::zeroed();
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileBasic: FILE_BASIC_INFORMATION = core::mem::zeroed();
    let mut CurrentFolder: PCFFOLDER = core::ptr::null_mut();
    let mut MaxDestFileSize: LARGE_INTEGER = core::mem::zeroed();
    let mut InputLength: LONG = 0;
    let mut OutputLength: LONG = 0;
    let mut StringLength: SIZE_T = 0;
    let mut Chunk: [core::ffi::c_char; 512] = [0; 512];
    let mut cleanup: u8 = 0;

    'main: {
        if wcscmp(
            &raw mut (*Search).Cabinet as *mut WCHAR,
            &raw mut (*CabinetContext).CabinetName as *mut WCHAR,
        ) != 0
        {
            DPRINT!(
                b"File is not in this cabinet (%S != %S)\n\0".as_ptr() as *const i8,
                &raw mut (*Search).Cabinet as *mut WCHAR,
                &raw mut (*CabinetContext).CabinetName as *mut WCHAR,
            );
            return CAB_STATUS_NOFILE;
        }

        if (*(*Search).File).FolderIndex as core::ffi::c_int == 0xFFFD as core::ffi::c_int
            || (*(*Search).File).FolderIndex as core::ffi::c_int == 0xFFFF as core::ffi::c_int
        {
            return CAB_STATUS_NOFILE;
        } else if (*(*Search).File).FolderIndex as core::ffi::c_int == 0xFFFE as core::ffi::c_int {
            CurrentFolder = (*CabinetContext).CabinetFolders.offset(
                ((*(*CabinetContext).PCABHeader).FolderCount as core::ffi::c_int
                    - 1 as core::ffi::c_int) as isize,
            ) as PCFFOLDER;
        } else {
            CurrentFolder = (*CabinetContext).CabinetFolders.offset(
                (*(*Search).File).FolderIndex as isize,
            ) as PCFFOLDER;
        }

        let comp_type = (*CurrentFolder).CompressionType & CAB_COMP_MASK;
        match comp_type {
            CAB_COMP_NONE => {
                CabinetSelectCodec(CabinetContext, CAB_CODEC_RAW);
            }
            CAB_COMP_MSZIP => {
                CabinetSelectCodec(CabinetContext, CAB_CODEC_MSZIP);
            }
            _ => return CAB_STATUS_UNSUPPCOMP,
        }

        DPRINT!(
            b"Extracting file at uncompressed offset (0x%X) Size (%d bytes)\n\0".as_ptr() as *const i8,
            (*(*Search).File).FileOffset as u32,
            (*(*Search).File).FileSize as u32,
        );

        if (*CabinetContext).CreateFileHandler.is_some() {
            CurrentDestBuffer = (*CabinetContext).CreateFileHandler.unwrap_unchecked()(
                CabinetContext as PCABINET_CONTEXT,
                (*(*Search).File).FileSize,
            );
            if CurrentDestBuffer.is_null() {
                DPRINT1!(b"CreateFileHandler() failed\n\0".as_ptr() as *const i8);
                return CAB_STATUS_CANNOT_CREATE;
            }
        } else {
            RtlInitAnsiString(
                &raw mut AnsiString,
                &raw mut (*(*Search).File).FileName as *mut core::ffi::c_char,
            );
            wcscpy(
                &raw mut DestName as *mut WCHAR,
                &raw mut (*CabinetContext).DestPath as *mut WCHAR,
            );
            StringLength = wcslen(&raw mut DestName as *mut WCHAR) as SIZE_T;
            UnicodeString.MaximumLength = (core::mem::size_of::<[WCHAR; (MAX_PATH) as _]>() as u16)
                .wrapping_sub((StringLength as u16).wrapping_mul(core::mem::size_of::<WCHAR>() as u16));
            UnicodeString.Buffer = (&raw mut DestName as *mut WCHAR).offset(StringLength as isize) as PWSTR;
            UnicodeString.Length = 0;
            RtlAnsiStringToUnicodeString(
                &raw mut UnicodeString,
                &raw mut AnsiString,
                (FALSE) as BOOLEAN,
            );

            RtlInitUnicodeString(
                &raw mut UnicodeString,
                &raw mut DestName as *mut WCHAR as PCWSTR,
            );

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = &raw mut UnicodeString;
            ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

            NtStatus = NtCreateFile(
                &raw mut DestFile,
                GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
                &raw mut ObjectAttributes,
                &raw mut IoStatusBlock,
                core::ptr::null_mut::<LARGE_INTEGER>(),
                FILE_ATTRIBUTE_NORMAL,
                0,
                FILE_CREATE,
                FILE_SYNCHRONOUS_IO_NONALERT,
                core::ptr::null_mut::<core::ffi::c_void>(),
                0,
            );

            if (!NT_SUCCESS!(NtStatus)) {
                DPRINT!(
                    b"NtCreateFile() failed (%S) (%x)\n\0".as_ptr() as *const i8,
                    &raw mut DestName as *mut WCHAR,
                    NtStatus,
                );

                if (*CabinetContext).OverwriteHandler.is_none()
                    || ((*CabinetContext).OverwriteHandler.unwrap_unchecked()(
                        CabinetContext as PCABINET_CONTEXT,
                        (*Search).File,
                        &raw mut DestName as *mut WCHAR as PCWSTR,
                    ) != 0)
                {
                    NtStatus = NtCreateFile(
                        &raw mut DestFile,
                        GENERIC_READ | GENERIC_WRITE | SYNCHRONIZE,
                        &raw mut ObjectAttributes,
                        &raw mut IoStatusBlock,
                        core::ptr::null_mut::<LARGE_INTEGER>(),
                        FILE_ATTRIBUTE_NORMAL,
                        0,
                        FILE_OVERWRITE,
                        FILE_SYNCHRONOUS_IO_ALERT,
                        core::ptr::null_mut::<core::ffi::c_void>(),
                        0,
                    );

                    if (!NT_SUCCESS!(NtStatus)) {
                        DPRINT1!(
                            b"NtCreateFile() failed (%S) (%x)\n\0".as_ptr() as *const i8,
                            &raw mut DestName as *mut WCHAR,
                            NtStatus,
                        );
                        return CAB_STATUS_CANNOT_CREATE;
                    }
                } else {
                    DPRINT1!(
                        b"File (%S) exists\n\0".as_ptr() as *const i8,
                        &raw mut DestName as *mut WCHAR,
                    );
                    return CAB_STATUS_FILE_EXISTS;
                }
            }

            if ConvertDosDateTimeToFileTime(
                (*(*Search).File).FileDate as WORD,
                (*(*Search).File).FileTime as WORD,
                &raw mut FileTime,
            ) == 0
            {
                DPRINT1!(b"DosDateTimeToFileTime() failed\n\0".as_ptr() as *const i8);
                Status = CAB_STATUS_CANNOT_WRITE;
                cleanup = 3;
                break 'main;
            }

            NtStatus = NtQueryInformationFile(
                DestFile,
                &raw mut IoStatusBlock,
                &raw mut FileBasic as PVOID,
                core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                FileBasicInformation,
            );

            if (!NT_SUCCESS!(NtStatus)) {
                DPRINT!(
                    b"NtQueryInformationFile() failed (%x)\n\0".as_ptr() as *const i8,
                    NtStatus,
                );
            } else {
                memcpy(
                    &raw mut FileBasic.LastAccessTime as *mut core::ffi::c_void,
                    &raw mut FileTime as *const core::ffi::c_void,
                    core::mem::size_of::<FILETIME>(),
                );

                NtStatus = NtSetInformationFile(
                    DestFile,
                    &raw mut IoStatusBlock,
                    &raw mut FileBasic as PVOID,
                    core::mem::size_of::<FILE_BASIC_INFORMATION>() as ULONG,
                    FileBasicInformation,
                );

                if (!NT_SUCCESS!(NtStatus)) {
                    DPRINT!(
                        b"NtSetInformationFile() failed (%x)\n\0".as_ptr() as *const i8,
                        NtStatus,
                    );
                }
            }

            SetAttributesOnFile((*Search).File, DestFile);

            if (*(*Search).File).FileSize == 0 {
                Status = CAB_STATUS_SUCCESS;
                cleanup = 3;
                break 'main;
            }

            MaxDestFileSize.QuadPart = (*(*Search).File).FileSize as LONGLONG;

            NtStatus = NtCreateSection(
                &raw mut DestFileSection,
                SECTION_ALL_ACCESS,
                core::ptr::null_mut::<_OBJECT_ATTRIBUTES>(),
                &raw mut MaxDestFileSize,
                PAGE_READWRITE,
                SEC_COMMIT,
                DestFile,
            );

            if (!NT_SUCCESS!(NtStatus)) {
                DPRINT1!(
                    b"NtCreateSection failed for %ls: %x\n\0".as_ptr() as *const i8,
                    &raw mut DestName as *mut WCHAR,
                    NtStatus,
                );
                Status = CAB_STATUS_NOMEMORY;
                cleanup = 3;
                break 'main;
            }

            DestFileBuffer = core::ptr::null_mut();
            (*CabinetContext).DestFileSize = 0;

            NtStatus = NtMapViewOfSection(
                DestFileSection,
                NtCurrentProcess(),
                &raw mut DestFileBuffer,
                0,
                0,
                core::ptr::null_mut::<_LARGE_INTEGER>(),
                &raw mut (*CabinetContext).DestFileSize,
                ViewUnmap,
                0,
                PAGE_READWRITE,
            );

            if (!NT_SUCCESS!(NtStatus)) {
                DPRINT1!(
                    b"NtMapViewOfSection failed: %x\n\0".as_ptr() as *const i8,
                    NtStatus,
                );
                Status = CAB_STATUS_NOMEMORY;
                cleanup = 2;
                break 'main;
            }

            CurrentDestBuffer = DestFileBuffer;
        }

        if (*CabinetContext).ExtractHandler.is_some() {
            ((*CabinetContext).ExtractHandler.unwrap_unchecked())(
                CabinetContext as PCABINET_CONTEXT,
                (*Search).File,
                &raw mut DestName as *mut WCHAR as PCWSTR,
            );
        }

        if !(*Search).CFData.is_null() {
            CFData = (*Search).CFData;
        } else {
            CFData = ((*CabinetContext).FileBuffer.offset(
                (*(*CabinetContext).CabinetFolders.offset(
                    (*(*Search).File).FolderIndex as isize,
                )).DataOffset as isize,
            )) as PCFDATA;
        }

        CurrentOffset = (*Search).Offset;

        while CurrentOffset.wrapping_add((*CFData).UncompSize as ULONG) <= (*(*Search).File).FileOffset {
            CurrentOffset = CurrentOffset.wrapping_add((*CFData).UncompSize as ULONG);
            CFData = ((CFData.offset(1 as isize) as *mut core::ffi::c_char)
                .offset((*CabinetContext).DataReserved as isize)
                .offset((*CFData).CompSize as isize)) as PCFDATA;
        }

        (*Search).CFData = CFData;
        (*Search).Offset = CurrentOffset;

        CurrentBuffer = ((CFData.offset(1 as isize) as *mut u8)
            .offset((*CabinetContext).DataReserved as isize)) as PUCHAR;
        RemainingBlock = (*CFData).CompSize as LONG;
        InputLength = RemainingBlock;

        while CurrentOffset < (*(*Search).File).FileOffset {
            OutputLength = (*(*Search).File).FileOffset.wrapping_sub(CurrentOffset) as LONG;
            if OutputLength > core::mem::size_of::<[core::ffi::c_char; 512]>() as LONG {
                OutputLength = core::mem::size_of::<[core::ffi::c_char; 512]>() as LONG;
            }
            OutputLength = -OutputLength;

            ((*(*CabinetContext).Codec).Uncompress).unwrap_unchecked()(
                (*CabinetContext).Codec,
                &raw mut Chunk as *mut core::ffi::c_void,
                CurrentBuffer as PVOID,
                &raw mut InputLength,
                &raw mut OutputLength,
            );

            CurrentOffset = CurrentOffset.wrapping_add(OutputLength as ULONG);
            CurrentBuffer = CurrentBuffer.offset(InputLength as isize);
            RemainingBlock -= InputLength;
            InputLength = -RemainingBlock;
        }

        Size = (*(*Search).File).FileSize;

        while Size > 0 {
            OutputLength = Size as LONG;

            DPRINT!(
                b"Decompressing block at %x with RemainingBlock = %d, Size = %d\n\0".as_ptr() as *const i8,
                CurrentBuffer,
                RemainingBlock,
                Size,
            );

            Status = ((*(*CabinetContext).Codec).Uncompress).unwrap_unchecked()(
                (*CabinetContext).Codec,
                CurrentDestBuffer,
                CurrentBuffer as PVOID,
                &raw mut InputLength,
                &raw mut OutputLength,
            );

            if Status != CS_SUCCESS {
                DPRINT!(b"Cannot uncompress block\n\0".as_ptr() as *const i8);
                if Status == CS_NOMEMORY {
                    Status = CAB_STATUS_NOMEMORY;
                } else {
                    Status = CAB_STATUS_INVALID_CAB;
                }
                cleanup = 1;
                break 'main;
            }

            CurrentDestBuffer = ((CurrentDestBuffer as usize).wrapping_add(OutputLength as usize)) as PVOID;
            CurrentBuffer = CurrentBuffer.offset(InputLength as isize);
            Size = Size.wrapping_sub(OutputLength as ULONG);
            RemainingBlock -= InputLength;

            if Size > 0 && RemainingBlock == 0 {
                DPRINT!(b"Out of block data\n\0".as_ptr() as *const i8);
                CFData = CurrentBuffer as PCFDATA;
                RemainingBlock = (*CFData).CompSize as LONG;
                CurrentBuffer = ((CFData.offset(1 as isize) as *mut u8)
                    .offset((*CabinetContext).DataReserved as isize)) as PUCHAR;
                InputLength = RemainingBlock;
            }
        }

        Status = CAB_STATUS_SUCCESS;
        cleanup = 1;
    }

    loop {
        match cleanup {
            1 => {
                if !(*CabinetContext).CreateFileHandler.is_some() {
                    NtUnmapViewOfSection(NtCurrentProcess(), DestFileBuffer);
                }
                cleanup = 2;
                continue;
            }
            2 => {
                if !(*CabinetContext).CreateFileHandler.is_some() {
                    NtClose(DestFileSection);
                }
                cleanup = 3;
                continue;
            }
            3 => {
                if !(*CabinetContext).CreateFileHandler.is_some() {
                    NtClose(DestFile);
                }
                break;
            }
            _ => break,
        }
    }

    Status
}
#[no_mangle]
pub unsafe extern "C" fn CabinetSelectCodec(
    CabinetContext: *mut CABINET_CONTEXT,
    Id: u32,
) {
    if (*CabinetContext).CodecSelected != 0 {
        if Id == (*CabinetContext).CodecId {
            return;
        }
        (*CabinetContext).CodecSelected = (FALSE) as i32;
    }
    match Id {
        CAB_CODEC_RAW => {
            (*CabinetContext).Codec = &mut RawCodec as *mut CAB_CODEC;
        }
        CAB_CODEC_MSZIP => {
            (*CabinetContext).Codec = &mut MSZipCodec as *mut CAB_CODEC;
            (*(*CabinetContext).Codec).ZStream.zalloc = Some(
                MSZipAlloc
                    as unsafe extern "C" fn(*mut core::ffi::c_void, u32, u32) -> *mut core::ffi::c_void,
            );
            (*(*CabinetContext).Codec).ZStream.zfree = Some(
                MSZipFree
                    as unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void),
            );
            (*(*CabinetContext).Codec).ZStream.opaque = core::ptr::null_mut();
        }
        _ => return,
    }
    (*CabinetContext).CodecId = Id;
    (*CabinetContext).CodecSelected = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetSetEventHandlers(CabinetContext: PCABINET_CONTEXT, Overwrite: PCABINET_OVERWRITE, Extract: PCABINET_EXTRACT, DiskChange: PCABINET_DISK_CHANGE, CreateFile: PCABINET_CREATE_FILE) {
    (*CabinetContext).OverwriteHandler = Overwrite;
    (*CabinetContext).ExtractHandler = Extract;
    (*CabinetContext).DiskChangeHandler = DiskChange;
    (*CabinetContext).CreateFileHandler = CreateFile;
}
#[no_mangle]
pub unsafe extern "C" fn CabinetGetCabinetReservedArea(
    CabinetContext: *mut CABINET_CONTEXT,
    Size: *mut u32,
) -> *mut core::ffi::c_void {
    if !(*CabinetContext).CabinetReservedArea.is_null() {
        if !Size.is_null() {
            *Size = (*CabinetContext).CabinetReserved;
        }
        return (*CabinetContext).CabinetReservedArea;
    } else {
        if !Size.is_null() {
            *Size = 0;
        }
        return core::ptr::null_mut();
    }
}

// ======== #include "../../usetup/spapisup/fileqsup.c" ========
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
pub static RUSTCONV_WSTR_6AF5D4B374C5: [u16; 4] = [0x006Eu16, 0x002F, 0x0061, 0x0000];


// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)

#[repr(C)]
pub struct _QUEUEENTRY {
    pub ListEntry: LIST_ENTRY,
    pub SourceCabinet: PWSTR,
    pub SourceRootPath: PWSTR,
    pub SourcePath: PWSTR,
    pub SourceFileName: PWSTR,
    pub TargetDirectory: PWSTR,
    pub TargetFileName: PWSTR,
}
pub type QUEUEENTRY = _QUEUEENTRY;
pub type PQUEUEENTRY = *mut _QUEUEENTRY;

#[repr(C)]
pub struct _FILEQUEUEHEADER {
    pub DeleteQueue: LIST_ENTRY,
    pub DeleteCount: ULONG,
    pub RenameQueue: LIST_ENTRY,
    pub RenameCount: ULONG,
    pub CopyQueue: LIST_ENTRY,
    pub CopyCount: ULONG,
    pub HasCurrentCabinet: BOOLEAN,
    pub CabinetContext: CABINET_CONTEXT,
    pub Search: CAB_SEARCH,
    pub CurrentCabinetName: [WCHAR; (MAX_PATH) as _],
}
pub type FILEQUEUEHEADER = _FILEQUEUEHEADER;
pub type PFILEQUEUEHEADER = *mut _FILEQUEUEHEADER;

unsafe extern "C" fn SetupExtractFile(
    mut QueueHeader: PFILEQUEUEHEADER,
    mut CabinetFileName: PCWSTR,
    mut SourceFileName: PCWSTR,
    mut DestinationPathName: PCWSTR,
) -> NTSTATUS {
    let mut CabStatus: ULONG = 0;
    DPRINT!(
        b"SetupExtractFile(CabinetFileName: '%S', SourceFileName: '%S', DestinationPathName: '%S')\n\0"
            .as_ptr() as *const i8,
        CabinetFileName,
        SourceFileName,
        DestinationPathName,
    );
    if (*QueueHeader).HasCurrentCabinet != 0 {
        DPRINT!(
            b"CurrentCabinetName: '%S'\n\0".as_ptr() as *const i8,
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
        );
    }
    if (*QueueHeader).HasCurrentCabinet != 0
        && wcscmp(
            CabinetFileName,
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
        ) == 0
    {
        DPRINT!(b"Using same cabinet as last time\n\0".as_ptr() as *const i8);
        CabStatus = CabinetFindNextFileSequential(
            &raw mut (*QueueHeader).CabinetContext,
            SourceFileName,
            &raw mut (*QueueHeader).Search,
        );
        if CabStatus != CAB_STATUS_SUCCESS {
            DPRINT!(b"Sequential miss on file: %S\n\0".as_ptr() as *const i8, SourceFileName);
            CabStatus = CabinetFindFirst(
                &raw mut (*QueueHeader).CabinetContext,
                SourceFileName,
                &raw mut (*QueueHeader).Search,
            );
        }
    } else {
        DPRINT!(b"Using new cabinet\n\0".as_ptr() as *const i8);
        if (*QueueHeader).HasCurrentCabinet != 0 {
            (*QueueHeader).HasCurrentCabinet = (FALSE) as u8;
            CabinetCleanup(&raw mut (*QueueHeader).CabinetContext);
        }
        RtlStringCchCopyW(
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
            (ARRAYSIZE!((*QueueHeader).CurrentCabinetName)) as _,
            CabinetFileName,
        );
        CabinetInitialize(&raw mut (*QueueHeader).CabinetContext);
        CabinetSetEventHandlers(
            &raw mut (*QueueHeader).CabinetContext,
            None,
            None,
            None,
            None,
        );
        CabinetSetCabinetName(&raw mut (*QueueHeader).CabinetContext, CabinetFileName);
        CabStatus = CabinetOpen(&raw mut (*QueueHeader).CabinetContext);
        if CabStatus == CAB_STATUS_SUCCESS {
            DPRINT!(b"Opened cabinet %S\n\0".as_ptr() as *const i8, CabinetFileName);
            (*QueueHeader).HasCurrentCabinet = (TRUE) as u8;
        } else {
            DPRINT!(b"Cannot open cabinet (%d)\n\0".as_ptr() as *const i8, CabStatus);
            return STATUS_UNSUCCESSFUL;
        }
        CabStatus = CabinetFindFirst(
            &raw mut (*QueueHeader).CabinetContext,
            SourceFileName,
            &raw mut (*QueueHeader).Search,
        );
    }
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT1!(
            b"Unable to find '%S' in cabinet '%S'\n\0".as_ptr() as *const i8,
            SourceFileName,
            CabinetGetCabinetName(&raw mut (*QueueHeader).CabinetContext),
        );
        return STATUS_UNSUCCESSFUL;
    }
    CabinetSetDestinationPath(&raw mut (*QueueHeader).CabinetContext, DestinationPathName);
    CabStatus = CabinetExtractFile(
        &raw mut (*QueueHeader).CabinetContext,
        &raw mut (*QueueHeader).Search,
    );
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT!(b"Cannot extract file %S (%d)\n\0".as_ptr() as *const i8, SourceFileName, CabStatus);
        return STATUS_UNSUCCESSFUL;
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn _SetupOpenFileQueue() -> HSPFILEQ {
    let mut QueueHeader: *mut FILEQUEUEHEADER = core::ptr::null_mut();

    QueueHeader = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<FILEQUEUEHEADER>()) as SIZE_T,
    ) as *mut FILEQUEUEHEADER;
    if QueueHeader.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory!(
        QueueHeader as *mut core::ffi::c_void,
        core::mem::size_of::<FILEQUEUEHEADER>(),
    );

    InitializeListHead(&raw mut (*QueueHeader).DeleteQueue);
    (*QueueHeader).DeleteCount = 0;
    InitializeListHead(&raw mut (*QueueHeader).RenameQueue);
    (*QueueHeader).RenameCount = 0;
    InitializeListHead(&raw mut (*QueueHeader).CopyQueue);
    (*QueueHeader).CopyCount = 0;

    (*QueueHeader).HasCurrentCabinet = (FALSE) as u8;

    QueueHeader as HSPFILEQ
}

unsafe extern "C" fn SetupDeleteQueueEntry(mut Entry: PQUEUEENTRY) {
    if Entry.is_null() {
        return;
    }
    if !(*Entry).SourceCabinet.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceCabinet as PVOID);
    }
    if !(*Entry).SourceRootPath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceRootPath as PVOID);
    }
    if !(*Entry).SourcePath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourcePath as PVOID);
    }
    if !(*Entry).SourceFileName.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceFileName as PVOID);
    }
    if !(*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetDirectory as PVOID);
    }
    if !(*Entry).TargetFileName.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetFileName as PVOID);
    }
    RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn _SetupCloseFileQueue(
    QueueHandle: HSPFILEQ,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut ListEntry: PLIST_ENTRY;
    let mut Entry: PQUEUEENTRY;

    if QueueHandle.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    /* Delete the delete queue */
    while !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).DeleteQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete the rename queue */
    while !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).RenameQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete the copy queue */
    while !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).CopyQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete queue header */
    RtlFreeHeap(ProcessHeap, 0, (QueueHeader).cast());

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyWithCab(
    QueueHandle: HSPFILEQ,
    SourceRootPath: PCWSTR,
    SourcePath: PCWSTR,
    SourceFileName: PCWSTR,
    SourceDescription: PCWSTR,
    SourceCabinet: PCWSTR,
    SourceTagFile: PCWSTR,
    TargetDirectory: PCWSTR,
    TargetFileName: PCWSTR,
    CopyStyle: u32,
) -> i32 {
    let mut QueueHeader: *mut FILEQUEUEHEADER;
    let mut Entry: *mut QUEUEENTRY;
    let mut Length: u32;

    if QueueHandle.is_null()
        || SourceRootPath.is_null()
        || SourceFileName.is_null()
        || TargetDirectory.is_null()
    {
        return (FALSE) as i32;
    }

    QueueHeader = QueueHandle as *mut FILEQUEUEHEADER;

    DPRINT!(
        b"SetupQueueCopy(Cab '%S', SrcRootPath '%S', SrcPath '%S', SrcFN '%S' --> DstPath '%S', DstFN '%S')\n\0"
            .as_ptr() as *const i8,
        if !SourceCabinet.is_null() {
            SourceCabinet
        } else {
            RUSTCONV_WSTR_6AF5D4B374C5.as_ptr()
        },
        SourceRootPath,
        SourcePath,
        SourceFileName,
        TargetDirectory,
        TargetFileName,
    );

    Entry = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T,
    ) as *mut QUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as i32;
    }

    RtlZeroMemory!(
        Entry as *mut core::ffi::c_void,
        core::mem::size_of::<QUEUEENTRY>(),
    );

    (*Entry).SourceCabinet = core::ptr::null_mut();
    if !SourceCabinet.is_null() {
        Length = wcslen(SourceCabinet) as u32;
        (*Entry).SourceCabinet = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).SourceCabinet,
            (Length + 1) as _,
            SourceCabinet,
        );
    }

    Length = wcslen(SourceRootPath) as u32;
    (*Entry).SourceRootPath = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).SourceRootPath.is_null() {
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).SourceRootPath,
        (Length + 1) as _,
        SourceRootPath,
    );

    (*Entry).SourcePath = core::ptr::null_mut();
    if !SourcePath.is_null() {
        Length = wcslen(SourcePath) as u32;
        if (Length > 0)
            && (*SourcePath.offset((Length - 1) as isize) as u16)
                == 0x005Cu16
        {
            Length -= 1;
        }
        (*Entry).SourcePath = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).SourcePath.is_null() {
            if !(*Entry).SourceCabinet.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceCabinet as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceRootPath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).SourcePath,
            (Length + 1) as _,
            SourcePath,
        );
    }

    Length = wcslen(SourceFileName) as u32;
    (*Entry).SourceFileName = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).SourceFileName.is_null() {
        if !(*Entry).SourcePath.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceRootPath as *mut core::ffi::c_void,
        );
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).SourceFileName,
        (Length + 1) as _,
        SourceFileName,
    );

    Length = wcslen(TargetDirectory) as u32;
    if (Length > 0)
        && (*TargetDirectory.offset((Length - 1) as isize) as u16)
            == 0x005Cu16
    {
        Length -= 1;
    }
    (*Entry).TargetDirectory = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceFileName as *mut core::ffi::c_void,
        );
        if !(*Entry).SourcePath.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceRootPath as *mut core::ffi::c_void,
        );
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).TargetDirectory,
        (Length + 1) as _,
        TargetDirectory,
    );

    (*Entry).TargetFileName = core::ptr::null_mut();
    if !TargetFileName.is_null() {
        Length = wcslen(TargetFileName) as u32;
        (*Entry).TargetFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).TargetFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).TargetDirectory as *mut core::ffi::c_void,
            );
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceFileName as *mut core::ffi::c_void,
            );
            if !(*Entry).SourcePath.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourcePath as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceRootPath as *mut core::ffi::c_void,
            );
            if !(*Entry).SourceCabinet.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceCabinet as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                Entry as *mut core::ffi::c_void,
            );
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).TargetFileName,
            (Length + 1) as _,
            TargetFileName,
        );
    }

    InsertTailList(
        &mut (*QueueHeader).CopyQueue as *mut _,
        &mut (*Entry).ListEntry as *mut _,
    );
    (*QueueHeader).CopyCount += 1;

    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn _SetupQueueDeleteW(
    QueueHandle: HSPFILEQ,
    PathPart1: PCWSTR,
    PathPart2: PCWSTR,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut Entry: PQUEUEENTRY;
    let mut Length: ULONG;

    if QueueHandle.is_null() || PathPart1.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    DPRINT1!("_SetupQueueDeleteW(PathPart1 '%S', PathPart2 '%S')\n", PathPart1, PathPart2);

    Entry = RtlAllocateHeap(ProcessHeap, 0, (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T) as PQUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as BOOL;
    }

    RtlZeroMemory!(Entry as *mut core::ffi::c_void, core::mem::size_of::<QUEUEENTRY>());

    (*Entry).SourceCabinet = core::ptr::null_mut();
    (*Entry).SourceRootPath = core::ptr::null_mut();
    (*Entry).SourcePath = core::ptr::null_mut();
    (*Entry).SourceFileName = core::ptr::null_mut();

    Length = wcslen(PathPart1) as ULONG;
    (*Entry).TargetDirectory = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * core::mem::size_of::<WCHAR>() as ULONG) as SIZE_T,
    ) as PWSTR;
    if (*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).TargetDirectory, (Length + 1) as _, PathPart1);

    if !PathPart2.is_null() {
        Length = wcslen(PathPart2) as ULONG;
        (*Entry).TargetFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * core::mem::size_of::<WCHAR>() as ULONG) as SIZE_T,
        ) as PWSTR;
        if (*Entry).TargetFileName.is_null() {
            RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetDirectory as PVOID);
            RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).TargetFileName, (Length + 1) as _, PathPart2);
    }

    InsertTailList(&mut (*QueueHeader).DeleteQueue, &mut (*Entry).ListEntry);
    (*QueueHeader).DeleteCount += 1;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn _SetupQueueRenameW(
    QueueHandle: HSPFILEQ,
    SourcePath: PCWSTR,
    SourceFileName: PCWSTR,
    TargetPath: PCWSTR,
    TargetFileName: PCWSTR,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut Entry: PQUEUEENTRY;
    let mut Length: ULONG;

    if QueueHandle.is_null() || SourcePath.is_null() || TargetFileName.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    DPRINT1!(
        b"_SetupQueueRenameW(SrcPath '%S', SrcFN '%S' --> DstPath '%S', DstFN '%S')\n\0"
            .as_ptr() as *const i8,
        SourcePath,
        SourceFileName,
        TargetPath,
        TargetFileName,
    );

    Entry = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T,
    ) as PQUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as BOOL;
    }

    RtlZeroMemory!(Entry as *mut core::ffi::c_void, core::mem::size_of::<QUEUEENTRY>());

    (*Entry).SourceCabinet = core::ptr::null_mut();
    (*Entry).SourceRootPath = core::ptr::null_mut();

    Length = (wcslen(SourcePath)) as _;
    if (Length > 0)
        && (*SourcePath.offset((Length - 1) as isize) as u16
            == 0x005Cu16)
    {
        Length -= 1;
    }
    (*Entry).SourcePath = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
    ) as PWSTR;
    if (*Entry).SourcePath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).SourcePath, (Length + 1) as _, SourcePath);

    (*Entry).SourceFileName = core::ptr::null_mut();
    if !SourceFileName.is_null() {
        Length = (wcslen(SourceFileName)) as _;
        (*Entry).SourceFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
        ) as PWSTR;
        if (*Entry).SourceFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).SourceFileName, (Length + 1) as _, SourceFileName);
    }

    (*Entry).TargetDirectory = core::ptr::null_mut();
    if !TargetPath.is_null() {
        Length = (wcslen(TargetPath)) as _;
        if (Length > 0)
            && (*TargetPath.offset((Length - 1) as isize) as u16
                == 0x005Cu16)
        {
            Length -= 1;
        }
        (*Entry).TargetDirectory = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
        ) as PWSTR;
        if (*Entry).TargetDirectory.is_null() {
            if !(*Entry).SourceFileName.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceFileName as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).TargetDirectory, (Length + 1) as _, TargetPath);
    }

    Length = (wcslen(TargetFileName)) as _;
    (*Entry).TargetFileName = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
    ) as PWSTR;
    if (*Entry).TargetFileName.is_null() {
        if !(*Entry).TargetDirectory.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).TargetDirectory as *mut core::ffi::c_void,
            );
        }
        if !(*Entry).SourceFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceFileName as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourcePath as *mut core::ffi::c_void,
        );
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).TargetFileName, (Length + 1) as _, TargetFileName);

    InsertTailList(
        core::ptr::addr_of_mut!((*QueueHeader).RenameQueue),
        core::ptr::addr_of_mut!((*Entry).ListEntry),
    );
    (*QueueHeader).RenameCount += 1;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn _SetupCommitFileQueueW(
    Owner: HWND,
    QueueHandle: HSPFILEQ,
    MsgHandler: PSP_FILE_CALLBACK_W,
    Context: PVOID,
) -> BOOL {
    let mut Success: BOOL = (TRUE) as i32;
    let mut Result: UINT;
    // C leaves Status uninitialized; the ABORT/SKIP paths of the first queue entry then
    // report it in Win32Error before any operation assigned it (a C bug). Zero here.
    let mut Status: NTSTATUS = 0;
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut ListEntry: PLIST_ENTRY;
    let mut Entry: PQUEUEENTRY;
    let mut FilePathInfo: FILEPATHS_W = core::mem::zeroed();
    let mut FileSrcPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FileDstPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if QueueHandle.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    Result = (MsgHandler.unwrap_unchecked())(
        Context,
        SPFILENOTIFY_STARTQUEUE as UINT,
        Owner as UINT_PTR,
        0,
    );
    if Result == FILEOP_ABORT {
        return (FALSE) as BOOL;
    }

    'quit: {
        if !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE as UINT,
                (FILEOP_DELETE) as _,
                ((*QueueHeader).DeleteCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).DeleteQueue.Flink;
        while ListEntry != &mut (*QueueHeader).DeleteQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            CombinePaths(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                2,
                (*Entry).TargetDirectory,
                (*Entry).TargetFileName,
            );

            DPRINT1!(" -----> Delete: '%S'\n", FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = core::ptr::null();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTDELETE as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_DELETE) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndDelete
            } else if Result == FILEOP_SKIP {
                // goto EndDelete
            } else {
                'retry_delete: loop {
                    Status = SetupDeleteFile(FileDstPath.as_ptr(), (TRUE) as BOOLEAN);
                    if (!NT_SUCCESS!(Status)) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_DELETEERROR as UINT,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            0,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_delete;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_delete;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_delete;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_delete;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDDELETE as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE as UINT,
                (FILEOP_DELETE) as _,
                0,
            );
        }

        if !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE as UINT,
                (FILEOP_RENAME) as _,
                ((*QueueHeader).RenameCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).RenameQueue.Flink;
        while ListEntry != &mut (*QueueHeader).RenameQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            CombinePaths(
                FileSrcPath.as_mut_ptr(),
                (ARRAYSIZE!(FileSrcPath)) as _,
                2,
                (*Entry).SourcePath,
                (*Entry).SourceFileName,
            );

            CombinePaths(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                2,
                (*Entry).TargetDirectory,
                (*Entry).TargetFileName,
            );

            DPRINT1!(" -----> Rename: '%S' ==> '%S'\n", FileSrcPath.as_ptr(), FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = FileSrcPath.as_ptr();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTRENAME as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_RENAME) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndRename
            } else if Result == FILEOP_SKIP {
                // goto EndRename
            } else {
                'retry_rename: loop {
                    Status = SetupMoveFile(
                        FileSrcPath.as_ptr(),
                        FileDstPath.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_COPY_ALLOWED | MOVEFILE_WRITE_THROUGH,
                    );
                    if (!NT_SUCCESS!(Status)) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_RENAMEERROR as UINT,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            0,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_rename;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_rename;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_rename;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_rename;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDRENAME as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE as UINT,
                (FILEOP_RENAME) as _,
                0,
            );
        }

        if !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE as UINT,
                (FILEOP_COPY) as _,
                ((*QueueHeader).CopyCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).CopyQueue.Flink;
        while ListEntry != &mut (*QueueHeader).CopyQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            if (*Entry).SourceCabinet.is_null() {
                CombinePaths(
                    FileSrcPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileSrcPath)) as _,
                    3,
                    (*Entry).SourceRootPath,
                    (*Entry).SourcePath,
                    (*Entry).SourceFileName,
                );
            } else {
                CombinePaths(
                    FileSrcPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileSrcPath)) as _,
                    3,
                    (*Entry).SourceRootPath,
                    (*Entry).SourcePath,
                    (*Entry).SourceCabinet,
                );
            }

            RtlStringCchCopyW(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                (*Entry).TargetDirectory,
            );
            if (*Entry).SourceCabinet.is_null() {
                if !(*Entry).TargetFileName.is_null() {
                    ConcatPaths(
                        FileDstPath.as_mut_ptr(),
                        (ARRAYSIZE!(FileDstPath)) as _,
                        1,
                        (*Entry).TargetFileName,
                    );
                } else {
                    ConcatPaths(
                        FileDstPath.as_mut_ptr(),
                        (ARRAYSIZE!(FileDstPath)) as _,
                        1,
                        (*Entry).SourceFileName,
                    );
                }
            } else {
                ConcatPaths(
                    FileDstPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileDstPath)) as _,
                    1,
                    (*Entry).SourceFileName,
                );
            }

            DPRINT!(" -----> Copy: '%S' ==> '%S'\n", FileSrcPath.as_ptr(), FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = FileSrcPath.as_ptr();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTCOPY as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_COPY) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndCopy
            } else if Result == FILEOP_SKIP {
                // goto EndCopy
            } else {
                'retry_copy: loop {
                    if !(*Entry).SourceCabinet.is_null() {
                        Status = SetupExtractFile(
                            QueueHeader,
                            FileSrcPath.as_ptr(),
                            (*Entry).SourceFileName,
                            (*Entry).TargetDirectory,
                        );
                    } else {
                        Status = SetupCopyFile(
                            FileSrcPath.as_ptr(),
                            FileDstPath.as_ptr(),
                            (FALSE) as BOOLEAN,
                        );
                    }

                    if (!NT_SUCCESS!(Status)) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_COPYERROR as UINT,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            core::ptr::null_mut::<c_void>() as UINT_PTR,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_copy;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_copy;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_copy;
                        } else if Result == FILEOP_NEWPATH {
                            continue 'retry_copy;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_copy;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDCOPY as UINT,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE as UINT,
                (FILEOP_COPY) as _,
                0,
            );
        }
    }

    (MsgHandler.unwrap_unchecked())(
        Context,
        SPFILENOTIFY_ENDQUEUE as UINT,
        Success as UINT_PTR,
        0,
    );

    Success
}
